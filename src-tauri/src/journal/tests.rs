use super::{
    catalog::{Entry, JournalCatalog, MuseumSet, Villager},
    evidence::JournalEvidence,
    view,
};
use crate::{
    catalog::AssetsZip,
    domain::{ItemId, Language, SpoilerMode},
    tracking::ProfileProgress,
};
use std::{fs::File, io::Write};
use tempfile::tempdir;
use zip::{write::SimpleFileOptions, ZipWriter};

fn hint_fixture(files: &[(&str, &str)]) -> JournalCatalog {
    let directory = tempdir().unwrap();
    let archive_path = directory.path().join("hint-fixture.zip");
    let mut archive = ZipWriter::new(File::create(&archive_path).unwrap());
    for (path, text) in files {
        archive
            .start_file(*path, SimpleFileOptions::default())
            .unwrap();
        archive.write_all(text.as_bytes()).unwrap();
    }
    archive.finish().unwrap();
    JournalCatalog::extract(&AssetsZip::new(archive_path).unwrap()).unwrap()
}

fn hidden_snapshot(catalog: &JournalCatalog) -> serde_json::Value {
    view::snapshot(
        catalog,
        &JournalEvidence::default(),
        &ProfileProgress::default(),
        Language::Eng,
        SpoilerMode::Free,
    )
}
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
            hint_facts: super::hints::HintFacts {
                areas: vec!["mines".into()],
                ..Default::default()
            },
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

    assert_eq!(entry["hint"]["kind"], "fish");
    assert_eq!(entry["hint"]["areas"], serde_json::json!(["mines"]));
    assert!(entry["name"].is_null());
    assert_eq!(entry["places"], serde_json::json!([]));
}

#[test]
fn hint_mine_fish_uses_mines_instead_of_default_river() {
    let catalog = hint_fixture(&[
        (
            "assets/fiddle/items/fish/misc.toml",
            "[cave_shrimp]\nname = \"Secret Shrimp\"\ndescription = \"Hidden.\"",
        ),
        (
            "assets/fiddle/fish.toml",
            "[default]\nseasons = false\nwater_type = \"river\"\nretrieval = \"fishing\"\n[cave_shrimp]\nretrieval = \"mines\"",
        ),
    ]);
    let snapshot = hidden_snapshot(&catalog);
    let hint = &snapshot["entries"][0]["hint"];

    assert_eq!(hint["areas"], serde_json::json!(["mines"]));
    assert!(!snapshot.to_string().contains("Secret Shrimp"));
    assert!(snapshot["entries"][0]["id"].is_null());
}

#[test]
fn hint_spawn_and_growth_metadata_provide_seasons_without_item_names() {
    let catalog = hint_fixture(&[
        (
            "assets/fiddle/items/other/bugs.toml",
            "[beach_bug]\nname = \"Secret Bug\"\ndescription = \"Hidden.\"",
        ),
        (
            "assets/fiddle/items/other/crops_and_forage.toml",
            "[turnip]\nname = \"Secret Crop\"\ndescription = \"Hidden.\"\ntags = [\"crop\"]\n[sand_dollar]\nname = \"Secret Forage\"\ndescription = \"Hidden.\"\ntags = [\"forageable\"]",
        ),
        (
            "assets/fiddle/bugs.toml",
            "[default]\nseasons = [\"spring\", \"summer\", \"fall\", \"winter\"]\ntag = [\"standard\"]\n[beach_bug]\nseasons = [\"summer\"]\ntag = [\"beach\"]",
        ),
        (
            "assets/fiddle/object_prototypes/crop.toml",
            "[turnip]\nseasons = \"spring\"",
        ),
        (
            "assets/fiddle/forageables.toml",
            "sand_forageables = [\"sand_dollar\"]\n[summer]\ncommon = [\"sand_dollar\"]",
        ),
    ]);
    let snapshot = hidden_snapshot(&catalog);
    let entries = snapshot["entries"].as_array().unwrap();
    let bug = entries
        .iter()
        .find(|entry| entry["category"] == "bugs")
        .unwrap();
    let crop = entries
        .iter()
        .find(|entry| entry["category"] == "crops")
        .unwrap();
    let forage = entries
        .iter()
        .find(|entry| entry["category"] == "forageables")
        .unwrap();

    assert_eq!(bug["hint"]["areas"], serde_json::json!(["beach"]));
    assert_eq!(bug["hint"]["seasons"], serde_json::json!(["summer"]));
    assert_eq!(crop["hint"]["seasons"], serde_json::json!(["spring"]));
    assert_eq!(forage["hint"]["areas"], serde_json::json!(["beach"]));
    assert_eq!(forage["hint"]["seasons"], serde_json::json!(["summer"]));
    assert!(!snapshot.to_string().contains("Secret Bug"));
    assert!(!snapshot.to_string().contains("Secret Crop"));
    assert!(!snapshot.to_string().contains("Secret Forage"));
}

#[test]
fn hint_artifact_area_and_confirmed_recipe_source_are_specific() {
    let catalog = hint_fixture(&[
        (
            "assets/fiddle/items/other/artifacts.toml",
            "[old_coin]\nname = \"Secret Coin\"\ndescription = \"Hidden.\"",
        ),
        (
            "assets/fiddle/museum_wings/archaeology.toml",
            "[sets.prehistoric]\nname = \"History\"\nitems = [\"old_coin\"]",
        ),
        (
            "assets/fiddle/artifacts.toml",
            "[locations]\nbeach = \"prehistoric\"",
        ),
        (
            "assets/fiddle/items/other/cooked_dishes.toml",
            "[apple_pie]\nname = \"Secret Pie\"\ndescription = \"Hidden.\"\nrecipe = [{ count = 1, item = \"apple\" }]\n[fruit_salad]\nname = \"Secret Salad\"\ndescription = \"Hidden.\"\nrecipe = [{ count = 1, item = \"fruit\" }]",
        ),
        (
            "assets/fiddle/stores.toml",
            "[general]\nstock = [{ recipe_scroll = \"apple_pie\" }]",
        ),
        (
            "assets/fiddle/letters.toml",
            "[recipe_letter]\nitems = [{ recipe_scroll = \"apple_pie\" }]",
        ),
        (
            "assets/fiddle/wishing_well.toml",
            "[well]\nrewards = [{ recipe_scroll = \"fruit_salad\" }]",
        ),
    ]);
    let snapshot = hidden_snapshot(&catalog);
    let entries = snapshot["entries"].as_array().unwrap();
    let artifact = entries
        .iter()
        .find(|entry| entry["category"] == "artifacts")
        .unwrap();
    let recipes: Vec<_> = entries
        .iter()
        .filter(|entry| entry["category"] == "recipes")
        .collect();

    assert_eq!(artifact["hint"]["areas"], serde_json::json!(["beach"]));
    assert!(recipes
        .iter()
        .any(|entry| entry["hint"]["source"] == "store"));
    assert!(recipes
        .iter()
        .any(|entry| entry["hint"]["source"] == "random"));
    assert!(!snapshot.to_string().contains("Secret Pie"));
}

#[test]
fn hint_hidden_gift_shows_an_activity_without_its_identity_or_reaction() {
    let mut catalog = JournalCatalog::default();
    catalog.entries.insert(
        "secret_gift".into(),
        Entry {
            id: "secret_gift".into(),
            category: "crops".into(),
            name: "Secret Gift Name".into(),
            ..Entry::default()
        },
    );
    catalog.villagers.push(Villager {
        id: "friend".into(),
        name: "Known Friend".into(),
        loved: vec!["secret_gift".into()],
        ..Villager::default()
    });
    let mut evidence = JournalEvidence::default();
    evidence.met.insert("friend".into());
    let snapshot = view::snapshot(
        &catalog,
        &evidence,
        &ProfileProgress::default(),
        Language::Eng,
        SpoilerMode::Free,
    );
    let gift = &snapshot["villagers"][0]["loved"][0];

    assert_eq!(gift["hint"]["kind"], "gift");
    assert_eq!(gift["hint"]["activity"], "crops");
    assert!(gift["entry"].is_null());
    assert!(gift["name"].is_null());
    assert!(!snapshot.to_string().contains("secret_gift"));
    assert!(!snapshot.to_string().contains("Secret Gift Name"));
}
