use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct GameSavePath(PathBuf);

/// A user-selected save file. Unlike `GameSavePath`, this path is not required
/// to live beneath the game's default saves directory.
#[derive(Clone, Debug)]
pub struct SelectedSavePath(PathBuf);

#[derive(Clone, Debug)]
pub struct GameAssetsPath(PathBuf);

#[derive(Clone, Debug)]
pub struct CompanionLogPath(PathBuf);

#[derive(Debug, thiserror::Error)]
pub enum GameSavePathError {
    #[error("the configured saves directory cannot be resolved: {0}")]
    SavesDirectory(#[source] std::io::Error),
    #[error("the selected save cannot be resolved: {0}")]
    SavePath(#[source] std::io::Error),
    #[error("the selected path is not a .sav file in the configured saves directory")]
    OutsideSavesDirectory,
    #[error("the selected save is not a regular file")]
    NotAFile,
}

impl GameSavePath {
    pub fn new(saves_directory: &Path, candidate: &Path) -> Result<Self, GameSavePathError> {
        let saves_directory =
            fs::canonicalize(saves_directory).map_err(GameSavePathError::SavesDirectory)?;
        let candidate = fs::canonicalize(candidate).map_err(GameSavePathError::SavePath)?;

        if candidate
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("sav")
            || !candidate.starts_with(&saves_directory)
        {
            return Err(GameSavePathError::OutsideSavesDirectory);
        }

        if !fs::metadata(&candidate)
            .map_err(GameSavePathError::SavePath)?
            .is_file()
        {
            return Err(GameSavePathError::NotAFile);
        }

        Ok(Self(candidate))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl SelectedSavePath {
    pub fn new(candidate: &Path) -> Result<Self, GameSavePathError> {
        let candidate = fs::canonicalize(candidate).map_err(GameSavePathError::SavePath)?;
        if candidate
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("sav")
        {
            return Err(GameSavePathError::OutsideSavesDirectory);
        }
        if !fs::metadata(&candidate)
            .map_err(GameSavePathError::SavePath)?
            .is_file()
        {
            return Err(GameSavePathError::NotAFile);
        }
        Ok(Self(candidate))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl GameAssetsPath {
    pub fn new(game_directory: &Path, candidate: &Path) -> Result<Self, GameSavePathError> {
        let game_directory =
            fs::canonicalize(game_directory).map_err(GameSavePathError::SavesDirectory)?;
        let candidate = fs::canonicalize(candidate).map_err(GameSavePathError::SavePath)?;
        if candidate.file_name().and_then(|name| name.to_str()) != Some("assets.zip")
            || !candidate.starts_with(&game_directory)
        {
            return Err(GameSavePathError::OutsideSavesDirectory);
        }
        if !fs::metadata(&candidate)
            .map_err(GameSavePathError::SavePath)?
            .is_file()
        {
            return Err(GameSavePathError::NotAFile);
        }
        Ok(Self(candidate))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl CompanionLogPath {
    pub fn new(mod_data_directory: &Path, candidate: &Path) -> Result<Self, GameSavePathError> {
        let mod_data_directory =
            fs::canonicalize(mod_data_directory).map_err(GameSavePathError::SavesDirectory)?;
        let candidate = fs::canonicalize(candidate).map_err(GameSavePathError::SavePath)?;
        if candidate.file_name().and_then(|name| name.to_str())
            != Some("mistria_tracker_companion.log")
            || !candidate.starts_with(&mod_data_directory)
        {
            return Err(GameSavePathError::OutsideSavesDirectory);
        }
        if !fs::metadata(&candidate)
            .map_err(GameSavePathError::SavePath)?
            .is_file()
        {
            return Err(GameSavePathError::NotAFile);
        }
        Ok(Self(candidate))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::vault::Fixture;
    use std::fs;

    #[test]
    fn rejects_outside_files_directories_and_wrong_extensions() {
        let fixture = Fixture::new();
        assert!(GameSavePath::new(&fixture.saves, &fixture.source).is_ok());
        let outside = fixture.local.join("outside.sav");
        fs::write(&outside, b"synthetic").unwrap();
        assert!(GameSavePath::new(&fixture.saves, &outside).is_err());
        assert!(GameSavePath::new(&fixture.saves, &fixture.saves).is_err());
        let wrong = fixture.saves.join("wrong.json");
        fs::write(&wrong, b"synthetic").unwrap();
        assert!(GameSavePath::new(&fixture.saves, &wrong).is_err());
        assert!(
            GameSavePath::new(&fixture.saves, &fixture.saves.join("../local/outside.sav")).is_err()
        );
    }

    #[test]
    fn accepts_only_assets_zip_under_the_selected_game_directory() {
        let fixture = Fixture::new();
        let game = fixture.local.join("game");
        fs::create_dir(&game).unwrap();
        let assets = game.join("assets.zip");
        fs::write(&assets, b"synthetic").unwrap();
        let other_zip = fixture.local.join("assets.zip");
        fs::write(&other_zip, b"synthetic").unwrap();

        assert!(GameAssetsPath::new(&game, &assets).is_ok());
        assert!(GameAssetsPath::new(&game, &other_zip).is_err());
        assert!(GameAssetsPath::new(&game, &fixture.source).is_err());
    }

    #[test]
    fn accepts_only_the_isolated_companion_log_file() {
        let fixture = Fixture::new();
        let logs = fixture
            .local
            .join("mod_data/mistria_tracker_companion/logs");
        fs::create_dir_all(&logs).unwrap();
        let log = logs.join("mistria_tracker_companion.log");
        fs::write(&log, "synthetic").unwrap();
        let other = logs.join("other.log");
        fs::write(&other, "synthetic").unwrap();

        assert!(CompanionLogPath::new(&fixture.local.join("mod_data"), &log).is_ok());
        assert!(CompanionLogPath::new(&fixture.local.join("mod_data"), &other).is_err());
        assert!(CompanionLogPath::new(&fixture.local.join("mod_data"), &fixture.source).is_err());
    }

    #[test]
    fn selected_save_accepts_a_regular_sav_from_any_directory() {
        let fixture = Fixture::new();
        let outside = fixture.local.join("secondary-drive-copy.sav");
        fs::write(&outside, b"synthetic").unwrap();

        assert_eq!(
            SelectedSavePath::new(&outside).unwrap().as_path(),
            fs::canonicalize(outside).unwrap()
        );
        assert!(SelectedSavePath::new(&fixture.saves).is_err());
        assert!(SelectedSavePath::new(&fixture.local.join("wrong.json")).is_err());
    }
}
