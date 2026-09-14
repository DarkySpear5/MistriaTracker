use super::{AssetsZip, CatalogCacheDir, CatalogError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    time::UNIX_EPOCH,
};
use tempfile::NamedTempFile;
use zip::ZipArchive;

pub const TRACKER_ICON_PLACEHOLDER: &str = "tracker://icons/item-placeholder";

const ITEM_ICON_PREFIX: &str = "assets/animations/Item Icons/";
const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
const MAX_ICON_BYTES: u64 = 4 * 1024 * 1024;
const ITEM_PACK_MANIFEST: &str = "item-icon-pack.json";

#[derive(Debug, Deserialize, Serialize)]
struct ItemIconPackManifest {
    source_fingerprint: String,
    item_count: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct IconCacheReport {
    pub item_count: usize,
}

/// Builds a tracker-owned item-art pack once per game archive revision.
///
/// The source archive is only read, never altered. On later launches we use its
/// file metadata to reuse the completed local pack without reopening the ZIP.
pub fn prepare_item_icon_cache(
    source: &AssetsZip,
    cache: &CatalogCacheDir,
) -> Result<IconCacheReport, CatalogError> {
    let fingerprint = source_fingerprint(source)?;
    let manifest_path = cache.as_path().join(ITEM_PACK_MANIFEST);
    if let Some(manifest) = read_manifest(&manifest_path)? {
        let pack_directory = cache
            .as_path()
            .join("item-icon-packs")
            .join(&manifest.source_fingerprint);
        if manifest.source_fingerprint == fingerprint && pack_directory.is_dir() {
            return Ok(IconCacheReport {
                item_count: manifest.item_count,
            });
        }
    }

    let pack_directory = cache.as_path().join("item-icon-packs").join(&fingerprint);
    fs::create_dir_all(&pack_directory)?;

    let mut archive = ZipArchive::new(File::open(source.as_path())?)?;
    let mut copied_sprites = BTreeSet::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        if entry.enclosed_name().is_none() {
            return Err(CatalogError::InvalidSource(
                "archive contains an unsafe path",
            ));
        }
        let Some(sprite_name) = item_sprite_name(&name) else {
            continue;
        };
        if !copied_sprites.insert(sprite_name.to_owned()) {
            return Err(CatalogError::InvalidSource("item icon is ambiguous"));
        }
        if entry.size() > MAX_ICON_BYTES {
            return Err(CatalogError::InvalidSource("item icon exceeds size limit"));
        }
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut bytes)?;
        if !bytes.starts_with(&PNG_SIGNATURE) {
            return Err(CatalogError::InvalidSource("item icon is not a PNG"));
        }
        write_file_atomically(&pack_directory.join(format!("{sprite_name}.png")), &bytes)?;
    }

    write_manifest(
        &manifest_path,
        &ItemIconPackManifest {
            source_fingerprint: fingerprint,
            item_count: copied_sprites.len(),
        },
    )?;
    Ok(IconCacheReport {
        item_count: copied_sprites.len(),
    })
}

/// Looks up a validated image that was previously copied into tracker storage.
/// This never opens the game's assets archive.
pub fn cached_item_icon(
    sprite_name: &str,
    cache: &CatalogCacheDir,
) -> Result<Option<PathBuf>, CatalogError> {
    if !valid_sprite_name(sprite_name) {
        return Ok(None);
    }
    let Some(manifest) = read_manifest(&cache.as_path().join(ITEM_PACK_MANIFEST))? else {
        return Ok(None);
    };
    if !valid_pack_fingerprint(&manifest.source_fingerprint) {
        return Ok(None);
    }
    let icon = cache
        .as_path()
        .join("item-icon-packs")
        .join(manifest.source_fingerprint)
        .join(format!("{sprite_name}.png"));
    Ok(icon.is_file().then_some(icon))
}

/// Compatibility helper for callers that have not yet prepared the local pack.
pub fn cache_item_icon(
    source: &AssetsZip,
    sprite_name: &str,
    cache: &CatalogCacheDir,
) -> Result<Option<PathBuf>, CatalogError> {
    prepare_item_icon_cache(source, cache)?;
    cached_item_icon(sprite_name, cache)
}

fn valid_sprite_name(sprite_name: &str) -> bool {
    sprite_name.starts_with("spr_ui_item_")
        && sprite_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn item_sprite_name(path: &str) -> Option<&str> {
    let sprite_name = path
        .strip_prefix(ITEM_ICON_PREFIX)?
        .strip_suffix(".png")?
        .rsplit('/')
        .next()?;
    valid_sprite_name(sprite_name).then_some(sprite_name)
}

fn source_fingerprint(source: &AssetsZip) -> Result<String, CatalogError> {
    let metadata = fs::metadata(source.as_path())?;
    let modified_nanos = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    Ok(hex_digest(
        format!("{}:{modified_nanos}", metadata.len()).as_bytes(),
    ))
}

fn valid_pack_fingerprint(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn read_manifest(path: &std::path::Path) -> Result<Option<ItemIconPackManifest>, CatalogError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn write_manifest(
    path: &std::path::Path,
    manifest: &ItemIconPackManifest,
) -> Result<(), CatalogError> {
    let bytes = serde_json::to_vec(manifest)?;
    write_file_atomically(path, &bytes)
}

fn write_file_atomically(path: &std::path::Path, bytes: &[u8]) -> Result<(), CatalogError> {
    let directory = path
        .parent()
        .ok_or(CatalogError::InvalidSource("cache path has no parent"))?;
    fs::create_dir_all(directory)?;
    let mut temporary = NamedTempFile::new_in(directory)?;
    temporary.write_all(bytes)?;
    temporary.flush()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}

fn hex_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io::Write};
    use tempfile::tempdir;
    use zip::{write::SimpleFileOptions, ZipWriter};

    const PNG: [u8; 12] = [137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 0];

    #[test]
    fn caches_only_an_allowlisted_item_icon_under_tracker_storage() {
        let directory = tempdir().unwrap();
        let zip_path = directory.path().join("assets.zip");
        let mut archive = ZipWriter::new(File::create(&zip_path).unwrap());
        archive
            .start_file(
                "assets/animations/Item Icons/Materials/spr_ui_item_fiber.png",
                SimpleFileOptions::default(),
            )
            .unwrap();
        archive.write_all(&PNG).unwrap();
        archive.finish().unwrap();

        let cache = CatalogCacheDir::new(directory.path().join("tracker-cache"));
        let icon = cache_item_icon(
            &AssetsZip::new(zip_path).unwrap(),
            "spr_ui_item_fiber",
            &cache,
        )
        .unwrap()
        .unwrap();

        assert!(icon.starts_with(cache.as_path()));
        assert_eq!(fs::read(icon).unwrap(), PNG);
    }

    #[test]
    fn prepares_a_complete_local_item_pack_that_is_reused_without_the_game_archive() {
        let directory = tempdir().unwrap();
        let zip_path = directory.path().join("assets.zip");
        let mut archive = ZipWriter::new(File::create(&zip_path).unwrap());
        archive
            .start_file(
                "assets/animations/Item Icons/Materials/spr_ui_item_fiber.png",
                SimpleFileOptions::default(),
            )
            .unwrap();
        archive.write_all(&PNG).unwrap();
        archive.finish().unwrap();
        let source = AssetsZip::new(&zip_path).unwrap();
        let cache = CatalogCacheDir::new(directory.path().join("tracker-cache"));

        assert_eq!(
            prepare_item_icon_cache(&source, &cache).unwrap().item_count,
            1
        );
        fs::remove_file(&zip_path).unwrap();
        let icon = cached_item_icon("spr_ui_item_fiber", &cache)
            .unwrap()
            .unwrap();
        assert_eq!(fs::read(icon).unwrap(), PNG);
    }

    #[test]
    fn refuses_paths_and_non_item_sprite_names() {
        let directory = tempdir().unwrap();
        let zip_path = directory.path().join("assets.zip");
        ZipWriter::new(File::create(&zip_path).unwrap())
            .finish()
            .unwrap();
        let source = AssetsZip::new(zip_path).unwrap();
        let cache = CatalogCacheDir::new(directory.path().join("tracker-cache"));

        assert!(cache_item_icon(&source, "../spr_ui_item_fiber", &cache)
            .unwrap()
            .is_none());
        assert!(cache_item_icon(&source, "spr_npc_ari", &cache)
            .unwrap()
            .is_none());
    }
}
