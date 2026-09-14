use super::{
    catalog::{Entry, JournalCatalog, MuseumSet, Villager},
    evidence::JournalEvidence,
    view,
};
use crate::{
    domain::{ItemId, Language, SpoilerMode},
    tracking::ProfileProgress,
};
#[test]
fn hidden_art_has_no_color_and_preserves_transparency() {
    let mut source = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut source, 3, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&[255, 90, 17, 255, 12, 140, 60, 80, 60, 80, 240, 0])
            .unwrap();
    }
    let output = super::art::silhouette(&source).unwrap();
    let mut reader = png::Decoder::new(std::io::Cursor::new(output))
        .read_info()
        .unwrap();
    let mut pixels = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut pixels).unwrap();
    assert_eq!(pixels, [0, 0, 0, 255, 0, 0, 0, 80, 0, 0, 0, 0]);
    assert!(super::art::silhouette(b"invalid PNG").is_err());
}

#[test]
fn portrait_extraction_selects_one_frame_and_trims_only_transparent_padding() {
    let mut source = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut source, 4, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&[0, 0, 0, 0, 30, 60, 90, 255, 200, 0, 0, 255, 200, 0, 0, 255])
            .unwrap();
    }
    let output = super::art::first_frame(&source, Some((2, 1)), true).unwrap();
    let mut reader = png::Decoder::new(std::io::Cursor::new(output))
        .read_info()
        .unwrap();
    assert_eq!((reader.info().width, reader.info().height), (1, 1));
    let mut pixels = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut pixels).unwrap();
    assert_eq!(pixels, [30, 60, 90, 255]);
    assert!(super::art::first_frame(&source, Some((5, 1)), false).is_err());
}
#[test]
fn registry_does_not_confuse_recipes_donations_or_gifts_with_item_discovery() {
    let mut catalog = JournalCatalog::default();
    catalog.entries.insert(
        "trout".into(),
        Entry {
            id: "trout".into(),
            category: "fish".into(),
            name: "Trout".into(),
            sprite: "spr_ui_item_trout".into(),
            ..Entry::default()
        },
    );
    catalog.entries.insert(
        "recipe:trout".into(),
        Entry {
            id: "recipe:trout".into(),
            category: "recipes".into(),
            name: "Trout recipe".into(),
            ..Entry::default()
        },
    );
    catalog.sets.push(MuseumSet {
        id: "fish:spring".into(),
        wing: "fish".into(),
        items: vec!["trout".into()],
        ..MuseumSet::default()
    });
    catalog.villagers.push(Villager {
        id: "hidden_npc".into(),
        name: "Secret Person".into(),
        loved: vec!["trout".into()],
        ..Villager::default()
    });
    let mut evidence = JournalEvidence::default();
    evidence.recipes.insert("trout".into());
    let initial = view::snapshot(
        &catalog,
        &evidence,
        &ProfileProgress::default(),
        Language::Eng,
        SpoilerMode::Free,
    );
    let entries = initial["entries"].as_array().unwrap();
    assert_eq!(entries.iter().filter(|e| e["revealed"] == true).count(), 1);
    assert_eq!(initial["sets"][0]["completed"], 0);
    assert!(initial["villagers"][0]["name"].is_null());
    assert!(!initial.to_string().contains("Secret Person"));
    assert!(!initial.to_string().contains("hidden_npc"));
    assert!(view::artwork_for(&catalog, &initial, "n0")
        .unwrap()
        .ends_with("_hidden"));
    let discovered = view::snapshot(
        &catalog,
        &evidence,
        &ProfileProgress::from_discovered([ItemId::new("trout").unwrap()]),
        Language::Eng,
        SpoilerMode::Free,
    );
    assert_eq!(discovered["sets"][0]["completed"], 0);
    let trout = discovered["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["id"] == "trout")
        .unwrap();
    assert!(trout["sources"]
        .as_array()
        .unwrap()
        .iter()
        .all(|s| s["view"] != "villagers"));
    let spoiled = view::snapshot(
        &catalog,
        &evidence,
        &ProfileProgress::default(),
        Language::Eng,
        SpoilerMode::All,
    );
    assert_eq!(spoiled["villagers"][0]["name"], "Secret Person");
    assert_eq!(spoiled["villagers"][0]["loved"][0]["found"], false);
}

#[test]
fn hidden_entries_publish_only_a_coarse_hint_not_their_location_or_name() {
    let mut catalog = JournalCatalog::default();
    catalog.entries.insert(
        "secret_fish".into(),
        Entry {
            id: "secret_fish".into(),
            category: "fish".into(),
            name: "Named Fish".into(),
            places: vec!["mines".into()],
            ..Entry::default()
        },
    );

    let snapshot = view::snapshot(
        &catalog,
        &JournalEvidence::default(),
        &ProfileProgress::default(),
        Language::Eng,
        SpoilerMode::Free,
    );
    let entry = &snapshot["entries"][0];

    assert_eq!(entry["hint"], "fish_cave");
    assert!(entry["name"].is_null());
    assert_eq!(entry["places"], serde_json::json!([]));
}
