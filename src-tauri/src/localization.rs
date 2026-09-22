use crate::{domain::Language, steam_discovery::validate_game_directory};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fs, path::Path};

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LanguagePreference {
    Auto,
    Manual(Language),
}

impl Default for LanguagePreference {
    fn default() -> Self {
        Self::Auto
    }
}

impl Serialize for LanguagePreference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Self::Auto => "auto",
            Self::Manual(Language::Eng) => "eng",
            Self::Manual(Language::Fra) => "fra",
            Self::Manual(Language::Spa) => "spa",
            Self::Manual(Language::Chs) => "chs",
            Self::Manual(Language::Cht) => "cht",
            Self::Manual(Language::Jpn) => "jpn",
            Self::Manual(Language::Kor) => "kor",
            Self::Manual(Language::Rus) => "rus",
        })
    }
}

impl<'de> Deserialize<'de> for LanguagePreference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.as_str() {
            "auto" => Ok(Self::Auto),
            "eng" => Ok(Self::Manual(Language::Eng)),
            "fra" => Ok(Self::Manual(Language::Fra)),
            "spa" => Ok(Self::Manual(Language::Spa)),
            "chs" => Ok(Self::Manual(Language::Chs)),
            "cht" => Ok(Self::Manual(Language::Cht)),
            "jpn" => Ok(Self::Manual(Language::Jpn)),
            "kor" => Ok(Self::Manual(Language::Kor)),
            "rus" => Ok(Self::Manual(Language::Rus)),
            _ => Err(serde::de::Error::custom(
                "unsupported tracker language preference",
            )),
        }
    }
}

pub fn effective_language(
    preference: LanguagePreference,
    game_directory: Option<&Path>,
) -> Language {
    match preference {
        LanguagePreference::Manual(language) => language,
        LanguagePreference::Auto => {
            steam_manifest_language(game_directory).unwrap_or(Language::Eng)
        }
    }
}

fn steam_manifest_language(game_directory: Option<&Path>) -> Option<Language> {
    let game_directory = validate_game_directory(game_directory?)?;
    let library = game_directory.parent()?.parent()?.parent()?;
    let manifest = library.join("steamapps/appmanifest_2142790.acf");
    let metadata = fs::metadata(&manifest).ok()?;
    if !metadata.is_file() || metadata.len() > MAX_MANIFEST_BYTES {
        return None;
    }
    let contents = fs::read_to_string(manifest).ok()?;
    match user_config_language(&contents)?.as_str() {
        "english" => Some(Language::Eng),
        "french" => Some(Language::Fra),
        "spanish" => Some(Language::Spa),
        "schinese" => Some(Language::Chs),
        "tchinese" => Some(Language::Cht),
        "japanese" => Some(Language::Jpn),
        "koreana" => Some(Language::Kor),
        "russian" => Some(Language::Rus),
        _ => None,
    }
}

#[derive(Debug, Eq, PartialEq)]
enum VdfToken {
    String(String),
    Open,
    Close,
}

fn user_config_language(contents: &str) -> Option<String> {
    let tokens = tokenize_vdf(contents)?;
    let user_config = tokens
        .iter()
        .position(|token| matches!(token, VdfToken::String(value) if value == "UserConfig"))?;
    if tokens.get(user_config + 1) != Some(&VdfToken::Open) {
        return None;
    }

    let mut depth = 1usize;
    let mut index = user_config + 2;
    while index < tokens.len() && depth > 0 {
        match tokens.get(index)? {
            VdfToken::Open => depth += 1,
            VdfToken::Close => depth -= 1,
            VdfToken::String(key) if depth == 1 && key == "language" => {
                if let Some(VdfToken::String(value)) = tokens.get(index + 1) {
                    return Some(value.clone());
                }
                return None;
            }
            VdfToken::String(_) => {}
        }
        index += 1;
    }
    None
}

fn tokenize_vdf(contents: &str) -> Option<Vec<VdfToken>> {
    let mut tokens = Vec::new();
    let mut characters = contents.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '{' => tokens.push(VdfToken::Open),
            '}' => tokens.push(VdfToken::Close),
            '"' => {
                let mut value = String::new();
                let mut closed = false;
                while let Some(character) = characters.next() {
                    match character {
                        '"' => {
                            closed = true;
                            break;
                        }
                        '\\' => value.push(characters.next()?),
                        _ => value.push(character),
                    }
                }
                if !closed {
                    return None;
                }
                tokens.push(VdfToken::String(value));
            }
            _ => {}
        }
    }
    Some(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Language;
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    struct GameFixture {
        _temporary: tempfile::TempDir,
        game: PathBuf,
    }

    fn fixture_game_directory_with_manifest(language: &str) -> GameFixture {
        let temporary = tempfile::tempdir().unwrap();
        let game = temporary.path().join("steamapps/common/Fields of Mistria");
        fs::create_dir_all(&game).unwrap();
        fs::write(
            game.join("Maybe.toml"),
            "name = \"Fields of Mistria\"\nexecutable_name = \"FieldsOfMistria\"\n",
        )
        .unwrap();
        fs::write(game.join("assets.zip"), b"fixture assets").unwrap();
        fs::write(
            temporary.path().join("steamapps/appmanifest_2142790.acf"),
            format!("\"AppState\" {{ \"UserConfig\" {{ \"language\" \"{language}\" }} }}"),
        )
        .unwrap();
        GameFixture {
            _temporary: temporary,
            game,
        }
    }

    #[test]
    fn auto_detects_french_only_from_the_mistria_manifest() {
        let game = fixture_game_directory_with_manifest("french");

        assert_eq!(
            effective_language(LanguagePreference::Auto, Some(&game.game)),
            Language::Fra
        );
    }

    #[test]
    fn auto_detects_english_from_the_mistria_manifest() {
        let game = fixture_game_directory_with_manifest("english");

        assert_eq!(
            effective_language(LanguagePreference::Auto, Some(&game.game)),
            Language::Eng
        );
    }

    #[test]
    fn russian_is_supported_while_malformed_steam_languages_fall_back_to_english() {
        let russian = fixture_game_directory_with_manifest("russian");
        let malformed = fixture_game_directory_with_manifest("<bad>");

        assert_eq!(
            effective_language(LanguagePreference::Auto, Some(&russian.game)),
            Language::Rus
        );
        assert_eq!(
            effective_language(LanguagePreference::Auto, Some(&malformed.game)),
            Language::Eng
        );
    }

    #[test]
    fn a_manual_language_choice_wins_over_auto_detect() {
        let game = fixture_game_directory_with_manifest("french");

        assert_eq!(
            effective_language(LanguagePreference::Manual(Language::Eng), Some(&game.game)),
            Language::Eng
        );
    }

    #[test]
    fn missing_or_non_game_folders_fall_back_to_english() {
        assert_eq!(
            effective_language(LanguagePreference::Auto, Some(Path::new("C:/missing"))),
            Language::Eng
        );
    }
}
