use super::{
    availability::LocationTag,
    items::{Catalog, CatalogItem, CatalogVersion},
    localization::LocalizedItemText,
};
use crate::domain::ItemId;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use zip::ZipArchive;

const ITEM_PREFIX: &str = "assets/fiddle/items/";
const TRANSLATION_PREFIX: &str = "assets/localization/translations/";
const VERSION_METADATA: &str = "assets/fiddle/fiddle.meta.toml";
const MAX_SOURCE_ENTRY_BYTES: u64 = 4 * 1024 * 1024;
const MAX_TRANSLATION_ENTRY_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct AssetsZip(PathBuf);

impl AssetsZip {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, CatalogError> {
        let path = path.into();
        if !path.is_file() {
            return Err(CatalogError::InvalidSource(
                "assets ZIP must be a regular file",
            ));
        }
        Ok(Self(path))
    }

    fn open(&self) -> Result<ZipArchive<File>, CatalogError> {
        Ok(ZipArchive::new(File::open(&self.0)?)?)
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct CatalogCacheDir(PathBuf);

impl CatalogCacheDir {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("could not read assets ZIP: {0}")]
    Io(#[from] std::io::Error),
    #[error("assets ZIP is invalid: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("catalog TOML is invalid: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("catalog cache metadata is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("catalog source is invalid: {0}")]
    InvalidSource(&'static str),
    #[error("catalog item is invalid: {0}")]
    InvalidItem(String),
}

pub struct CatalogExtractor;

impl CatalogExtractor {
    pub fn extract(source: &AssetsZip, _cache: &CatalogCacheDir) -> Result<Catalog, CatalogError> {
        let mut archive = source.open()?;
        let mut item_sources = Vec::new();
        let mut translation_sources = BTreeMap::new();
        let mut version = None;

        for index in 0..archive.len() {
            let mut entry = archive.by_index(index)?;
            if entry.is_dir() {
                continue;
            }
            let name = entry.name().to_owned();
            if entry.enclosed_name().is_none() {
                return Err(CatalogError::InvalidSource(
                    "archive contains an unsafe path",
                ));
            }
            let translation_locale = translation_locale(&name);
            if !(name.starts_with(ITEM_PREFIX) && name.ends_with(".toml")
                || translation_locale.is_some()
                || name == VERSION_METADATA)
            {
                continue;
            }
            let maximum_bytes = if translation_locale.is_some() {
                MAX_TRANSLATION_ENTRY_BYTES
            } else {
                MAX_SOURCE_ENTRY_BYTES
            };
            let content = read_entry(&mut entry, maximum_bytes)?;
            if name.starts_with(ITEM_PREFIX) {
                item_sources.push((name, content));
            } else if let Some(locale) = translation_locale {
                translation_sources.insert(locale.to_owned(), content);
            } else {
                version = Some(content);
            }
        }

        let translations = translation_sources
            .iter()
            .map(|(locale, source)| Ok((locale.clone(), parse_translations(Some(source))?)))
            .collect::<Result<BTreeMap<_, _>, CatalogError>>()?;
        let version = parse_version(version.as_deref(), &item_sources, &translation_sources)?;
        let mut seen = BTreeSet::new();
        let mut items = Vec::new();
        for (path, source) in item_sources {
            for item in parse_items(&source, &translations, locations_for_path(&path))? {
                if !seen.insert(item.id.as_str().to_owned()) {
                    return Err(CatalogError::InvalidItem(
                        "duplicate stable item key".to_owned(),
                    ));
                }
                items.push(item);
            }
        }
        Ok(Catalog::new(
            CatalogVersion {
                source_game_version: version,
            },
            items,
        ))
    }
}

fn read_entry(
    entry: &mut zip::read::ZipFile<'_>,
    maximum_bytes: u64,
) -> Result<String, CatalogError> {
    if entry.size() > maximum_bytes {
        return Err(CatalogError::InvalidSource(
            "allowlisted archive entry exceeds size limit",
        ));
    }
    let mut content = String::with_capacity(entry.size() as usize);
    entry.read_to_string(&mut content)?;
    Ok(content)
}

fn parse_version(
    source: Option<&str>,
    item_sources: &[(String, String)],
    translations: &BTreeMap<String, String>,
) -> Result<String, CatalogError> {
    let Some(source) = source else {
        return Ok(catalog_fingerprint(item_sources, translations));
    };
    let document = toml::from_str::<toml::Value>(source)?;
    Ok(document
        .get("game_version")
        .and_then(toml::Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| catalog_fingerprint(item_sources, translations)))
}

fn catalog_fingerprint(
    item_sources: &[(String, String)],
    translations: &BTreeMap<String, String>,
) -> String {
    let mut entries = item_sources
        .iter()
        .map(|(path, content)| (path.clone(), content.as_bytes()))
        .collect::<Vec<_>>();
    for (locale, content) in translations {
        entries.push((
            format!("{TRANSLATION_PREFIX}{locale}.meta.toml"),
            content.as_bytes(),
        ));
    }
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let mut digest = Sha256::new();
    for (path, content) in entries {
        digest.update(path.as_bytes());
        digest.update([0]);
        digest.update((content.len() as u64).to_be_bytes());
        digest.update(content);
    }
    format!("catalog-sha256:{:x}", digest.finalize())
}

fn parse_translations(source: Option<&str>) -> Result<BTreeMap<String, Translation>, CatalogError> {
    let Some(source) = source else {
        return Ok(BTreeMap::new());
    };
    let document = toml::from_str::<toml::Value>(source)?;
    let Some(items) = document.get("items").and_then(toml::Value::as_table) else {
        return parse_flat_translations(
            document
                .get("asset_properties")
                .and_then(toml::Value::as_table),
        );
    };
    items
        .iter()
        .map(|(key, value)| {
            let table = value
                .as_table()
                .ok_or_else(|| CatalogError::InvalidItem(format!("translation {key}")))?;
            Ok((
                key.clone(),
                Translation {
                    name: optional_string(table, "name", key)?.map(str::to_owned),
                    description: optional_string(table, "description", key)?.map(str::to_owned),
                },
            ))
        })
        .collect()
}

fn parse_flat_translations(
    properties: Option<&toml::map::Map<String, toml::Value>>,
) -> Result<BTreeMap<String, Translation>, CatalogError> {
    let mut translations = BTreeMap::<String, Translation>::new();
    for (path, value) in properties.into_iter().flatten() {
        let Some((item_path, field)) = path.rsplit_once('/') else {
            continue;
        };
        let Some(item_id) = item_path.rsplit('/').next() else {
            continue;
        };
        let Some(value) = value.as_str() else {
            continue;
        };
        let translation = translations
            .entry(item_id.to_owned())
            .or_insert(Translation {
                name: None,
                description: None,
            });
        match field {
            "name" => translation.name = Some(value.to_owned()),
            "description" => translation.description = Some(value.to_owned()),
            _ => {}
        }
    }
    Ok(translations)
}

fn parse_items(
    source: &str,
    translations: &BTreeMap<String, BTreeMap<String, Translation>>,
    fallback_locations: Vec<LocationTag>,
) -> Result<Vec<CatalogItem>, CatalogError> {
    let document = toml::from_str::<toml::Value>(source)?;
    let tables = document
        .get("items")
        .and_then(toml::Value::as_table)
        .or_else(|| document.as_table())
        .ok_or(CatalogError::InvalidSource(
            "item TOML must contain item tables",
        ))?;
    tables
        .iter()
        .map(|(key, value)| parse_item(key, value, translations, fallback_locations.clone()))
        .collect::<Result<Vec<_>, _>>()
        .map(|items| items.into_iter().flatten().collect())
}

fn parse_item(
    key: &str,
    value: &toml::Value,
    translations: &BTreeMap<String, BTreeMap<String, Translation>>,
    fallback_locations: Vec<LocationTag>,
) -> Result<Option<CatalogItem>, CatalogError> {
    let table = value
        .as_table()
        .ok_or_else(|| CatalogError::InvalidItem(key.to_owned()))?;
    let id = ItemId::new(key).map_err(|_| CatalogError::InvalidItem(key.to_owned()))?;
    let Some(name) = optional_string(table, "name", key)? else {
        return Ok(None);
    };
    let Some(description) = optional_string(table, "description", key)? else {
        return Ok(None);
    };
    let english = LocalizedItemText {
        name: name.to_owned(),
        description: description.to_owned(),
    };
    let mut text = BTreeMap::from([("eng".to_owned(), english.clone())]);
    for (locale, translations) in translations {
        let translation = translations.get(key);
        text.insert(
            locale.clone(),
            LocalizedItemText {
                name: translation
                    .and_then(|value| value.name.as_deref())
                    .unwrap_or(&english.name)
                    .to_owned(),
                description: translation
                    .and_then(|value| value.description.as_deref())
                    .unwrap_or(&english.description)
                    .to_owned(),
            },
        );
    }
    let locations = table
        .get("locations")
        .map(|value| {
            value
                .as_array()
                .ok_or_else(|| CatalogError::InvalidItem(key.to_owned()))?
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(LocationTag::parse)
                        .ok_or_else(|| CatalogError::InvalidItem(key.to_owned()))
                })
                .collect()
        })
        .transpose()?
        .unwrap_or(fallback_locations);
    let seasons = string_array(table, "seasons", key)?;
    let icon_sprite = optional_string(table, "icon_sprite", key)?.map(str::to_owned);
    Ok(Some(CatalogItem::new(
        id,
        icon_sprite,
        seasons,
        locations,
        text,
    )))
}

fn locations_for_path(path: &str) -> Vec<LocationTag> {
    let normalized = path.replace('\\', "/");
    for (segment, location) in [
        ("/fish/pond", LocationTag::Pond),
        ("/fish/ocean", LocationTag::Ocean),
        ("/fish/river", LocationTag::River),
        ("/mines/", LocationTag::Mines),
    ] {
        if normalized.contains(segment) {
            return vec![location];
        }
    }
    Vec::new()
}

fn string_array(
    table: &toml::map::Map<String, toml::Value>,
    field: &str,
    item: &str,
) -> Result<Vec<String>, CatalogError> {
    table
        .get(field)
        .map(|value| {
            value
                .as_array()
                .ok_or_else(|| CatalogError::InvalidItem(item.to_owned()))?
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| CatalogError::InvalidItem(item.to_owned()))
                })
                .collect()
        })
        .transpose()
        .map(Option::unwrap_or_default)
}

fn optional_string<'a>(
    table: &'a toml::map::Map<String, toml::Value>,
    field: &str,
    item: &str,
) -> Result<Option<&'a str>, CatalogError> {
    table
        .get(field)
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| CatalogError::InvalidItem(item.to_owned()))
        })
        .transpose()
}

struct Translation {
    name: Option<String>,
    description: Option<String>,
}

fn translation_locale(path: &str) -> Option<&'static str> {
    let filename = path
        .strip_prefix(TRANSLATION_PREFIX)?
        .strip_suffix(".meta.toml")?;
    match filename {
        "fra" | "fr" | "french" => Some("fra"),
        "spa" | "es" | "spanish" => Some("spa"),
        "chs" | "zhs" | "zh_cn" | "zh-CN" | "zh-Hans" | "schinese" => Some("chs"),
        "cht" | "zht" | "zh_tw" | "zh-TW" | "zh-Hant" | "tchinese" => Some("cht"),
        "jpn" | "ja" | "japanese" => Some("jpn"),
        "kor" | "ko" | "korean" | "koreana" => Some("kor"),
        "rus" | "ru" | "russian" => Some("rus"),
        _ => None,
    }
}
