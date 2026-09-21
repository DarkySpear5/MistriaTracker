use super::ids::{ItemId, NpcId, ProfileId};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use std::fmt;
use uuid::Uuid;

pub const EVENT_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, Serialize)]
pub struct EventEnvelope {
    pub schema_version: u16,
    pub companion_version: String,
    pub game_version: String,
    pub profile_id: ProfileId,
    pub session_id: Uuid,
    pub sequence: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_file: Option<String>,
    #[serde(flatten)]
    pub event: CompanionEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum CompanionEvent {
    ProfileActivated,
    ItemObtained {
        item_id: ItemId,
        count: u32,
    },
    GiftGiven {
        npc_id: NpcId,
        item_id: ItemId,
        reaction: GiftReaction,
    },
    MuseumDonated {
        item_id: ItemId,
        set_id: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GiftReaction {
    Loved,
    Liked,
    Neutral,
    Disliked,
    Hated,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Eng,
    Fra,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpoilerMode {
    Free,
    All,
}

#[derive(Debug)]
pub enum EventError {
    UnsupportedSchema(u16),
    InvalidItemCount,
    InvalidContract(String),
    InvalidEnvelope(serde_json::Error),
}

impl EventEnvelope {
    pub fn from_json(json: &str) -> Result<Self, EventError> {
        let value = serde_json::from_str(json).map_err(EventError::InvalidEnvelope)?;
        Self::from_value(value)
    }

    fn from_value(value: Value) -> Result<Self, EventError> {
        validate_event_shape(&value)?;
        let envelope: RawEventEnvelope =
            serde_json::from_value(value).map_err(EventError::InvalidEnvelope)?;
        Ok(Self {
            schema_version: envelope.schema_version,
            companion_version: envelope.companion_version,
            game_version: envelope.game_version,
            profile_id: envelope.profile_id,
            session_id: envelope.session_id,
            sequence: envelope.sequence,
            save_file: envelope.save_file,
            event: envelope.event,
        })
    }
}

impl<'de> Deserialize<'de> for EventEnvelope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Self::from_value(value).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for EventError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported event schema version {version}")
            }
            Self::InvalidItemCount => formatter.write_str("item event count must be at least one"),
            Self::InvalidContract(message) => formatter.write_str(message),
            Self::InvalidEnvelope(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for EventError {}

#[derive(Deserialize)]
struct RawEventEnvelope {
    schema_version: u16,
    companion_version: String,
    game_version: String,
    profile_id: ProfileId,
    session_id: Uuid,
    sequence: u64,
    #[serde(default)]
    save_file: Option<String>,
    #[serde(flatten)]
    event: CompanionEvent,
}

fn validate_event_shape(value: &Value) -> Result<(), EventError> {
    let envelope = value.as_object().ok_or_else(|| {
        EventError::InvalidContract("event envelope must be an object".to_owned())
    })?;
    if let Some(schema_version) = envelope.get("schema_version").and_then(Value::as_u64) {
        let schema_version = u16::try_from(schema_version).map_err(|_| {
            EventError::InvalidContract(
                "schema_version must be an unsigned 16-bit integer".to_owned(),
            )
        })?;
        if schema_version != EVENT_SCHEMA_VERSION {
            return Err(EventError::UnsupportedSchema(schema_version));
        }
    }
    for field in ["companion_version", "game_version"] {
        if !matches!(envelope.get(field), Some(Value::String(value)) if !value.is_empty()) {
            return Err(EventError::InvalidContract(format!(
                "{field} must be a non-empty string"
            )));
        }
    }
    let event_type = envelope
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| EventError::InvalidContract("event type must be a string".to_owned()))?;
    let mut allowed = vec![
        "schema_version",
        "companion_version",
        "game_version",
        "profile_id",
        "session_id",
        "sequence",
        "type",
    ];

    match event_type {
        "profile_activated" => {
            allowed.push("save_file");
            if let Some(save_file) = envelope.get("save_file") {
                let save_file = save_file.as_str().ok_or_else(|| {
                    EventError::InvalidContract("save_file must be a string".to_owned())
                })?;
                let profile_id = envelope
                    .get("profile_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        EventError::InvalidContract("profile_id must be a string".to_owned())
                    })?;
                let prefix = format!("game-{profile_id}-");
                let slot = save_file
                    .strip_prefix(&prefix)
                    .and_then(|value| value.strip_suffix(".sav"));
                if !slot.is_some_and(|slot| {
                    slot == "autosave"
                        || (!slot.is_empty() && slot.bytes().all(|byte| byte.is_ascii_digit()))
                }) {
                    return Err(EventError::InvalidContract(
                        "save_file must be the active Fields of Mistria save basename".to_owned(),
                    ));
                }
            }
        }
        "item_obtained" => {
            allowed.push("payload");
            validate_payload(envelope, &["item_id", "count"])?;
            if envelope["payload"]["count"].as_u64() == Some(0) {
                return Err(EventError::InvalidItemCount);
            }
        }
        "gift_given" => {
            allowed.push("payload");
            validate_payload(envelope, &["npc_id", "item_id", "reaction"])?;
        }
        "museum_donated" => {
            allowed.push("payload");
            validate_payload(envelope, &["item_id", "set_id"])?;
        }
        _ => {
            allowed.push("payload");
        }
    }
    ensure_only_fields(envelope, &allowed, "envelope")?;

    Ok(())
}

fn validate_payload(envelope: &Map<String, Value>, allowed: &[&str]) -> Result<(), EventError> {
    let payload = envelope
        .get("payload")
        .and_then(Value::as_object)
        .ok_or_else(|| EventError::InvalidContract("event payload must be an object".to_owned()))?;
    ensure_only_fields(payload, allowed, "payload")
}

fn ensure_only_fields(
    object: &Map<String, Value>,
    allowed: &[&str],
    location: &str,
) -> Result<(), EventError> {
    if let Some(field) = object
        .keys()
        .find(|field| !allowed.contains(&field.as_str()))
    {
        return Err(EventError::InvalidContract(format!(
            "unexpected {location} field: {field}"
        )));
    }
    Ok(())
}
