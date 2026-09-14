use crate::catalog::{AssetsZip, Catalog, CatalogCacheDir, CatalogExtractor};
use std::{fs::File, io::Write};
use tempfile::{tempdir, TempDir};
use zip::{write::SimpleFileOptions, ZipWriter};

pub struct AssetsFixture {
    _directory: TempDir,
    pub source: AssetsZip,
    pub cache: CatalogCacheDir,
}

pub fn fixture_assets() -> AssetsFixture {
    fixture_assets_with_numeric_id(7)
}

pub fn fixture_assets_without_version() -> AssetsFixture {
    let directory = tempdir().unwrap();
    let zip_path = directory.path().join("synthetic-assets.zip");
    let mut archive = ZipWriter::new(File::create(&zip_path).unwrap());
    let options = SimpleFileOptions::default();
    write_entry(
        &mut archive,
        options,
        "assets/fiddle/items/fish/pond.toml",
        "[pond_fish]\nname = \"Pond Fish\"\ndescription = \"A fish.\"\n\n[internal_definition]\nkind = \"internal\"",
    );
    write_entry(
        &mut archive,
        options,
        "assets/localization/translations/fra.meta.toml",
        "[asset_properties]\n\"items/fish/pond/pond_fish/name\" = \"Poisson d'étang\"",
    );
    archive.finish().unwrap();
    AssetsFixture {
        source: AssetsZip::new(zip_path).unwrap(),
        cache: CatalogCacheDir::new(directory.path().join("cache")),
        _directory: directory,
    }
}

pub fn extract_fixture_catalog() -> Catalog {
    extract_with_numeric_id(7)
}

pub fn extract_reordered_fixture() -> Catalog {
    extract_with_numeric_id(9001)
}

fn extract_with_numeric_id(numeric_id: u64) -> Catalog {
    let fixture = fixture_assets_with_numeric_id(numeric_id);
    CatalogExtractor::extract(&fixture.source, &fixture.cache).unwrap()
}

fn fixture_assets_with_numeric_id(numeric_id: u64) -> AssetsFixture {
    let directory = tempdir().unwrap();
    let zip_path = directory.path().join("synthetic-assets.zip");
    let mut archive = ZipWriter::new(File::create(&zip_path).unwrap());
    let options = SimpleFileOptions::default();
    write_entry(
        &mut archive,
        options,
        "assets/fiddle/fiddle.meta.toml",
        "game_version = \"synthetic-1\"",
    );
    write_entry(&mut archive, options, "assets/fiddle/items/synthetic.toml", &format!("[paper_pondshell]\nnumeric_id = {numeric_id}\nname = \"Paper Pondshell\"\ndescription = \"A shell from a calm pond.\"\nicon_sprite = \"items/paper_pondshell\"\nseasons = [\"spring\"]\nlocations = [\"pond\"]"));
    write_entry(
        &mut archive,
        options,
        "assets/localization/translations/fra.meta.toml",
        "[items.paper_pondshell]\nname = \"Coquille de papier\"",
    );
    archive.finish().unwrap();
    AssetsFixture {
        source: AssetsZip::new(zip_path).unwrap(),
        cache: CatalogCacheDir::new(directory.path().join("cache")),
        _directory: directory,
    }
}

fn write_entry(
    archive: &mut ZipWriter<File>,
    options: SimpleFileOptions,
    name: &str,
    contents: &str,
) {
    archive.start_file(name, options).unwrap();
    archive.write_all(contents.as_bytes()).unwrap();
}
