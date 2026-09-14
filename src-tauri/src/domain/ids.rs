use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ProfileId(String);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ItemId(String);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct NpcId(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdError {
    InvalidProfileId,
    InvalidGameIdentifier,
}

impl fmt::Display for IdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProfileId => {
                formatter.write_str("profile IDs must be non-empty numeric save prefixes")
            }
            Self::InvalidGameIdentifier => {
                formatter.write_str("game identifiers must be non-empty ASCII snake case")
            }
        }
    }
}

impl std::error::Error for IdError {}

impl ProfileId {
    pub fn new(value: impl Into<String>) -> Result<Self, IdError> {
        let value = value.into();
        if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) {
            Ok(Self(value))
        } else {
            Err(IdError::InvalidProfileId)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

macro_rules! game_identifier {
    ($type:ident) => {
        impl $type {
            pub fn new(value: impl Into<String>) -> Result<Self, IdError> {
                let value = value.into();
                if is_ascii_snake_case(&value) {
                    Ok(Self(value))
                } else {
                    Err(IdError::InvalidGameIdentifier)
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

game_identifier!(ItemId);
game_identifier!(NpcId);

fn is_ascii_snake_case(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes[0].is_ascii_lowercase()
        && bytes[bytes.len() - 1].is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
        && !value.contains("__")
}

macro_rules! deserialize_identifier {
    ($type:ident) => {
        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
            }
        }
    };
}

deserialize_identifier!(ProfileId);
deserialize_identifier!(ItemId);
deserialize_identifier!(NpcId);

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum EntityId {
    Item(ItemId),
    Npc(NpcId),
    MuseumSet(String),
}
