use mistria_tracker_lib::compatibility::matrix::{
    CompatibilityDecision, CompatibilityMatrix, VersionSet,
};
use mistria_tracker_lib::domain::events::{CompanionEvent, EventEnvelope, EventError};
use mistria_tracker_lib::domain::ids::{ItemId, ProfileId};
use mistria_tracker_lib::domain::{EntityId, Language, SpoilerMode};
use serde_json::json;

const ITEM_EVENT: &str = r#"{"schema_version":1,"companion_version":"0.1.0","game_version":"1.0.4","profile_id":"1849811906","session_id":"018f0000-0000-7000-8000-000000000001","sequence":7,"type":"item_obtained","payload":{"item_id":"wild_berry","count":2}}"#;

#[test]
fn parses_item_event_and_rejects_unknown_schema() {
    let ok = EventEnvelope::from_json(ITEM_EVENT);
    assert!(matches!(
        ok.unwrap().event,
        CompanionEvent::ItemObtained { count: 2, .. }
    ));
    assert!(matches!(
        EventEnvelope::from_json(r#"{"schema_version":99}"#),
        Err(EventError::UnsupportedSchema(99))
    ));
}

#[test]
fn rejects_zero_item_count() {
    assert!(matches!(
        EventEnvelope::from_json(
            r#"{"schema_version":1,"companion_version":"0.1.0","game_version":"1.0.4","profile_id":"1849811906","session_id":"018f0000-0000-7000-8000-000000000001","sequence":7,"type":"item_obtained","payload":{"item_id":"wild_berry","count":0}}"#
        ),
        Err(EventError::InvalidItemCount)
    ));
}

#[test]
fn unknown_game_version_fails_closed() {
    let matrix = CompatibilityMatrix::fixture_for("1.0.4");
    assert_eq!(
        matrix.decision(&VersionSet::new("9.9.9", "0.1.0", 1)),
        CompatibilityDecision::PauseLiveAndImport
    );
}

#[test]
fn identifiers_accept_only_their_canonical_formats() {
    assert!(ProfileId::new("1849811906").is_ok());
    assert!(ProfileId::new("profile_1").is_err());
    assert!(ItemId::new("wild_berry").is_ok());
    assert!(ItemId::new("WildBerry").is_err());
    assert!(ItemId::new("wild__berry").is_err());
}

#[test]
fn language_spoiler_mode_and_entity_id_use_stable_wire_values() {
    assert_eq!(serde_json::to_value(Language::Eng).unwrap(), json!("eng"));
    assert_eq!(serde_json::to_value(Language::Fra).unwrap(), json!("fra"));
    assert_eq!(
        serde_json::to_value(SpoilerMode::Free).unwrap(),
        json!("free")
    );
    assert_eq!(
        serde_json::to_value(SpoilerMode::All).unwrap(),
        json!("all")
    );
    assert_eq!(
        serde_json::to_value(EntityId::Item(ItemId::new("wild_berry").unwrap())).unwrap(),
        json!({ "kind": "item", "id": "wild_berry" })
    );
}

#[test]
fn probe_required_match_pauses_live_ingestion() {
    let matrix = CompatibilityMatrix::fixture_for("1.0.4");
    assert_eq!(
        matrix.decision(&VersionSet::new("1.0.4", "0.1.0", 1)),
        CompatibilityDecision::PauseLive
    );
}

#[test]
fn published_schema_forbids_profile_activation_payloads() {
    let schema: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../contracts/companion-event.schema.json"
    )))
    .unwrap();
    let profile_branch = schema["allOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|branch| branch["if"]["properties"]["type"]["const"] == "profile_activated")
        .unwrap();
    assert_eq!(
        profile_branch["then"]["not"]["required"],
        json!(["payload"])
    );

    let profile_with_payload = r#"{"schema_version":1,"companion_version":"0.1.0","game_version":"1.0.4","profile_id":"1849811906","session_id":"018f0000-0000-7000-8000-000000000001","sequence":7,"type":"profile_activated","payload":{"unexpected":"value"}}"#;
    assert!(EventEnvelope::from_json(profile_with_payload).is_err());
}

#[test]
fn profile_activation_accepts_only_a_sanitized_save_basename() {
    let exact = r#"{"schema_version":1,"companion_version":"0.1.5","game_version":"1.0.5","profile_id":"331655283","session_id":"018f0000-0000-7000-8000-000000000001","sequence":7,"type":"profile_activated","save_file":"game-331655283-1725841886.sav"}"#;
    assert_eq!(
        EventEnvelope::from_json(exact)
            .unwrap()
            .save_file
            .as_deref(),
        Some("game-331655283-1725841886.sav")
    );
    for invalid in [
        "../game-331655283-1725841886.sav",
        "game-1849811906-1725841886.sav",
        "game-331655283-1725841886.txt",
    ] {
        assert!(EventEnvelope::from_json(
            &exact.replace("game-331655283-1725841886.sav", invalid,)
        )
        .is_err());
    }
}

#[test]
fn direct_deserialization_enforces_the_event_contract() {
    assert!(serde_json::from_str::<EventEnvelope>(ITEM_EVENT).is_ok());
    assert!(serde_json::from_str::<EventEnvelope>(&ITEM_EVENT.replacen(
        "\"schema_version\":1",
        "\"schema_version\":99",
        1
    ))
    .is_err());
    assert!(serde_json::from_str::<EventEnvelope>(&ITEM_EVENT.replacen(
        "\"count\":2",
        "\"count\":0",
        1
    ))
    .is_err());
    assert!(serde_json::from_str::<EventEnvelope>(r#"{"schema_version":1,"companion_version":"0.1.0","game_version":"1.0.4","profile_id":"1849811906","session_id":"018f0000-0000-7000-8000-000000000001","sequence":7,"type":"item_obtained","payload":{"item_id":"wild_berry","count":2},"unexpected":true}"#).is_err());
    assert!(serde_json::from_str::<EventEnvelope>(&ITEM_EVENT.replacen(
        "\"count\":2",
        "\"count\":2,\"unexpected\":true",
        1
    ))
    .is_err());
}

#[test]
fn version_fields_must_not_be_empty_in_either_parsing_path() {
    for empty_version_event in [
        ITEM_EVENT.replacen(
            "\"companion_version\":\"0.1.0\"",
            "\"companion_version\":\"\"",
            1,
        ),
        ITEM_EVENT.replacen("\"game_version\":\"1.0.4\"", "\"game_version\":\"\"", 1),
    ] {
        assert!(EventEnvelope::from_json(&empty_version_event).is_err());
        assert!(serde_json::from_str::<EventEnvelope>(&empty_version_event).is_err());
    }
}

#[test]
fn embedded_1_0_4_record_is_live_approved_after_the_real_game_probe() {
    let matrix = CompatibilityMatrix::embedded().unwrap();
    assert_eq!(
        matrix.decision(&VersionSet::new("1.0.4", "0.1.0", 1)),
        CompatibilityDecision::FullySupported
    );
}
