//! Read-only discovery of a normal Steam installation.
//!
//! The functions in this module never crawl drives, modify Steam configuration,
//! or open game saves. A result is returned only after opening the game's direct
//! `assets.zip` file.

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

pub const MISTRIA_APP_ID: &str = "2142790";
const GAME_DIRECTORY_SEGMENTS: [&str; 4] = ["steamapps", "common", "Fields of Mistria", "assets.zip"];
const MAX_STEAM_TEXT_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscoveryResult {
    Found(PathBuf),
    NotFound,
}

/// Inputs are injectable so tests can exercise discovery without reading the
/// machine's registry, Steam configuration, or mounted drives.
#[derive(Clone, Debug, Default)]
pub struct DiscoverySources {
    pub saved_game: Option<PathBuf>,
    pub steam_libraries: Vec<PathBuf>,
    pub drive_roots: Vec<PathBuf>,
}

impl DiscoverySources {
    pub fn from_windows_system(saved_game: Option<PathBuf>) -> Self {
        let steam_root = registered_steam_root();
        let mut steam_libraries = steam_root
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        if let Some(root) = steam_root {
            steam_libraries.extend(steam_libraries_from_vdf(&root));
        }
        steam_libraries.sort();
        steam_libraries.dedup();

        Self {
            saved_game,
            steam_libraries,
            drive_roots: mounted_drive_roots(),
        }
    }
}

/// Returns a canonical game directory only when its direct `assets.zip` can be
/// opened as a regular file. This is the sole path form passed back to the UI.
pub fn validate_game_directory(candidate: &Path) -> Option<PathBuf> {
    let game = fs::canonicalize(candidate).ok()?;
    if !fs::metadata(&game).ok()?.is_dir() {
        return None;
    }
    let assets = game.join("assets.zip");
    if !fs::metadata(&assets).ok()?.is_file() || File::open(assets).is_err() {
        return None;
    }
    // Keep the caller's normal Windows path form. `canonicalize` above is only
    // used for validation; returning its `\\\\?\\`-prefixed form makes a valid
    // folder look unfamiliar in the interface and breaks normal path handling.
    Some(candidate.to_path_buf())
}

pub fn discover_game_directory(sources: &DiscoverySources) -> DiscoveryResult {
    sources
        .saved_game
        .as_deref()
        .and_then(validate_game_directory)
        .or_else(|| steam_library_candidates(&sources.steam_libraries).find_map(|path| validate_game_directory(&path)))
        .or_else(|| bounded_drive_candidates(&sources.drive_roots).find_map(|path| validate_game_directory(&path)))
        .map_or(DiscoveryResult::NotFound, DiscoveryResult::Found)
}

pub fn discover_windows_game_directory(saved_game: Option<PathBuf>) -> DiscoveryResult {
    discover_game_directory(&DiscoverySources::from_windows_system(saved_game))
}

fn steam_library_candidates(libraries: &[PathBuf]) -> impl Iterator<Item = PathBuf> + '_ {
    libraries
        .iter()
        .filter(|library| has_mistria_manifest(library))
        .map(|library| game_directory_for_library(library))
}

fn bounded_drive_candidates(roots: &[PathBuf]) -> impl Iterator<Item = PathBuf> + '_ {
    roots.iter().flat_map(|root| {
        [root.join("SteamLibrary"), root.join("Steam"), root.join("Program Files (x86)/Steam"), root.join("Program Files/Steam")]
            .into_iter()
            .map(|steam_root| game_directory_for_library(&steam_root))
    })
}

fn game_directory_for_library(library: &Path) -> PathBuf {
    GAME_DIRECTORY_SEGMENTS
        .iter()
        .fold(library.to_path_buf(), |path, segment| path.join(segment))
        .parent()
        .expect("game directory segments include assets.zip")
        .to_path_buf()
}

fn has_mistria_manifest(library: &Path) -> bool {
    let manifest = library.join(format!("steamapps/appmanifest_{MISTRIA_APP_ID}.acf"));
    fs::metadata(&manifest)
        .ok()
        .filter(|metadata| metadata.is_file() && metadata.len() <= MAX_STEAM_TEXT_BYTES)
        .and_then(|_| fs::read_to_string(manifest).ok())
        .is_some_and(|contents| contents.contains(&format!("\"appid\" \"{MISTRIA_APP_ID}\"")))
}

fn steam_libraries_from_vdf(steam_root: &Path) -> Vec<PathBuf> {
    let vdf = steam_root.join("steamapps/libraryfolders.vdf");
    fs::metadata(&vdf)
        .ok()
        .filter(|metadata| metadata.is_file() && metadata.len() <= MAX_STEAM_TEXT_BYTES)
        .and_then(|_| fs::read_to_string(vdf).ok())
        .map(|contents| parse_library_paths(&contents))
        .unwrap_or_default()
}

fn parse_library_paths(contents: &str) -> Vec<PathBuf> {
    let quoted = quoted_vdf_values(contents);
    quoted
        .windows(2)
        .filter(|pair| pair[0].eq_ignore_ascii_case("path"))
        .map(|pair| PathBuf::from(&pair[1]))
        .collect()
}

fn quoted_vdf_values(contents: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut characters = contents.chars();
    while let Some(character) = characters.next() {
        if character != '"' {
            continue;
        }
        let mut value = String::new();
        while let Some(character) = characters.next() {
            match character {
                '"' => break,
                '\\' => {
                    if let Some(escaped) = characters.next() {
                        value.push(escaped);
                    }
                }
                _ => value.push(character),
            }
        }
        values.push(value);
    }
    values
}

#[cfg(windows)]
fn registered_steam_root() -> Option<PathBuf> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Valve\Steam")
        .ok()?
        .get_value::<String, _>("SteamPath")
        .ok()
        .map(PathBuf::from)
}

#[cfg(not(windows))]
fn registered_steam_root() -> Option<PathBuf> {
    None
}

#[cfg(windows)]
fn mounted_drive_roots() -> Vec<PathBuf> {
    (b'A'..=b'Z')
        .map(|letter| PathBuf::from(format!("{}:\\", letter as char)))
        .filter(|root| root.is_dir())
        .collect()
}

#[cfg(not(windows))]
fn mounted_drive_roots() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    fn fixture_game_directory(root: &Path, with_assets: bool) -> std::path::PathBuf {
        let game = root.join("steamapps/common/Fields of Mistria");
        fs::create_dir_all(&game).unwrap();
        if with_assets {
            fs::write(game.join("assets.zip"), b"fixture assets").unwrap();
        }
        game
    }

    fn write_manifest(library: &Path) {
        let manifest = library.join("steamapps/appmanifest_2142790.acf");
        fs::create_dir_all(manifest.parent().unwrap()).unwrap();
        fs::write(manifest, "\"AppState\" { \"appid\" \"2142790\" }").unwrap();
    }

    #[test]
    fn accepts_only_a_directory_with_readable_assets_zip() {
        let temporary = tempfile::tempdir().unwrap();
        let game = fixture_game_directory(temporary.path(), true);

        assert_eq!(validate_game_directory(&game), Some(game));
        assert_eq!(validate_game_directory(&temporary.path().join("missing")), None);
    }

    #[test]
    fn finds_a_secondary_steam_library_before_drive_fallback() {
        let temporary = tempfile::tempdir().unwrap();
        let saved = temporary.path().join("old-game");
        let secondary_library = temporary.path().join("secondary-library");
        let secondary_game = fixture_game_directory(&secondary_library, true);
        write_manifest(&secondary_library);
        let fallback_root = temporary.path().join("fallback-root");
        let fallback_game = fixture_game_directory(&fallback_root.join("SteamLibrary"), true);

        let sources = DiscoverySources {
            saved_game: Some(saved),
            steam_libraries: vec![secondary_library],
            drive_roots: vec![fallback_root],
        };

        assert_eq!(
            discover_game_directory(&sources),
            DiscoveryResult::Found(secondary_game)
        );
        assert_ne!(
            discover_game_directory(&sources),
            DiscoveryResult::Found(fallback_game)
        );
    }

    #[test]
    fn ignores_libraries_without_the_mistria_manifest() {
        let temporary = tempfile::tempdir().unwrap();
        let library = temporary.path().join("library");
        fixture_game_directory(&library, true);
        let fallback_root = temporary.path().join("fallback-root");
        let fallback_game = fixture_game_directory(&fallback_root.join("Steam"), true);

        let sources = DiscoverySources {
            saved_game: None,
            steam_libraries: vec![library],
            drive_roots: vec![fallback_root],
        };

        assert_eq!(
            discover_game_directory(&sources),
            DiscoveryResult::Found(fallback_game)
        );
    }

    #[test]
    fn uses_a_stable_non_recursive_fallback_order() {
        let temporary = tempfile::tempdir().unwrap();
        let drive = temporary.path().join("drive");
        let steam_library_game = fixture_game_directory(&drive.join("SteamLibrary"), true);
        let steam_game = fixture_game_directory(&drive.join("Steam"), true);

        let sources = DiscoverySources {
            saved_game: None,
            steam_libraries: vec![],
            drive_roots: vec![drive],
        };

        assert_eq!(
            discover_game_directory(&sources),
            DiscoveryResult::Found(steam_library_game)
        );
        assert_ne!(
            discover_game_directory(&sources),
            DiscoveryResult::Found(steam_game)
        );
    }

    #[test]
    fn ignores_malformed_library_data_and_missing_roots() {
        assert!(parse_library_paths("\"path\"").is_empty());

        let temporary = tempfile::tempdir().unwrap();
        let sources = DiscoverySources {
            saved_game: Some(temporary.path().join("stale-game")),
            steam_libraries: vec![temporary.path().join("missing-library")],
            drive_roots: vec![temporary.path().join("missing-drive")],
        };

        assert_eq!(discover_game_directory(&sources), DiscoveryResult::NotFound);
    }

    #[test]
    fn parses_escaped_steam_library_paths() {
        assert_eq!(
            parse_library_paths(
                r#""libraryfolders" { "1" { "path" "D:\\SteamLibrary" } }"#
            ),
            vec![std::path::PathBuf::from(r"D:\SteamLibrary")]
        );
    }
}
