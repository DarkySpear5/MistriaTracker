use super::{LogTailError, ReadOnlyLogTail};
use crate::{
    app_state::{TrackerState, TrackerStateError},
    catalog::{prepare_item_icon_cache, AssetsZip, CatalogCacheDir},
    compatibility::CompatibilityMatrix,
    persistence::AcceptResult,
    safety::paths::{CompanionLogPath, GameAssetsPath},
};

#[derive(Debug, thiserror::Error)]
pub enum PassiveSessionError {
    #[error(transparent)]
    State(#[from] TrackerStateError),
    #[error(transparent)]
    LogTail(#[from] LogTailError),
}

/// A read-only runtime session. It never writes to game assets, logs, or saves.
pub struct PassiveTrackingSession {
    tail: ReadOnlyLogTail,
}

impl PassiveTrackingSession {
    pub fn start(
        state: &TrackerState,
        assets: &GameAssetsPath,
        log: &CompanionLogPath,
        cache: &CatalogCacheDir,
        compatibility: &CompatibilityMatrix,
    ) -> Result<Self, PassiveSessionError> {
        state.load_catalog_from_game_assets(assets, cache, compatibility)?;
        let source = AssetsZip::new(assets.as_path()).map_err(TrackerStateError::from)?;
        prepare_item_icon_cache(&source, cache).map_err(TrackerStateError::from)?;
        Ok(Self {
            tail: ReadOnlyLogTail::open(log.as_path())?,
        })
    }

    pub fn poll(
        &mut self,
        state: &TrackerState,
        compatibility: &CompatibilityMatrix,
    ) -> Result<Vec<AcceptResult>, PassiveSessionError> {
        Ok(state.poll_companion_log(&mut self.tail, compatibility)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        persistence::Repository,
        safety::paths::{CompanionLogPath, GameAssetsPath},
        test_support::catalog::fixture_assets,
    };
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn session_reads_only_new_events_after_an_approved_catalog_load() {
        let directory = tempdir().unwrap();
        let game = directory.path().join("game");
        fs::create_dir(&game).unwrap();
        let fixture = fixture_assets();
        let assets_path = game.join("assets.zip");
        fs::copy(fixture.source.as_path(), &assets_path).unwrap();
        let assets = GameAssetsPath::new(&game, &assets_path).unwrap();
        let mod_data = directory.path().join("mod_data");
        let log_directory = mod_data.join("mistria_tracker_companion/logs");
        fs::create_dir_all(&log_directory).unwrap();
        let log_path = log_directory.join("mistria_tracker_companion.log");
        fs::write(&log_path, "old history\n").unwrap();
        let log = CompanionLogPath::new(&mod_data, &log_path).unwrap();
        let state = TrackerState::from_repository(Repository::in_memory().unwrap());
        state.activate_profile("1849811906").unwrap();
        let matrix = verified_matrix();

        let mut session = PassiveTrackingSession::start(
            &state,
            &assets,
            &log,
            &CatalogCacheDir::new(directory.path().join("cache")),
            &matrix,
        )
        .unwrap();
        fs::write(
            &log_path,
            format!("old history\nMISTRIA_TRACKER_EVENT|{}\n", event()),
        )
        .unwrap();

        assert_eq!(
            session.poll(&state, &matrix).unwrap(),
            vec![AcceptResult::Inserted]
        );
        assert_eq!(
            state
                .active_profile_snapshot()
                .unwrap()
                .unwrap()
                .items
                .len(),
            1
        );
    }

    #[test]
    fn probe_gated_session_does_not_start_or_install_a_catalog() {
        let directory = tempdir().unwrap();
        let game = directory.path().join("game");
        fs::create_dir(&game).unwrap();
        let fixture = fixture_assets();
        let assets_path = game.join("assets.zip");
        fs::copy(fixture.source.as_path(), &assets_path).unwrap();
        let assets = GameAssetsPath::new(&game, &assets_path).unwrap();
        let mod_data = directory.path().join("mod_data");
        let log_directory = mod_data.join("mistria_tracker_companion/logs");
        fs::create_dir_all(&log_directory).unwrap();
        let log_path = log_directory.join("mistria_tracker_companion.log");
        fs::write(&log_path, "old history\n").unwrap();
        let log = CompanionLogPath::new(&mod_data, &log_path).unwrap();
        let state = TrackerState::from_repository(Repository::in_memory().unwrap());

        assert!(PassiveTrackingSession::start(
            &state,
            &assets,
            &log,
            &CatalogCacheDir::new(directory.path().join("cache")),
            &CompatibilityMatrix::fixture_for("synthetic-1"),
        )
        .is_err());
        assert!(state.active_profile_snapshot().unwrap().is_none());
    }

    fn verified_matrix() -> CompatibilityMatrix {
        serde_json::from_value(serde_json::json!({
            "records": [{"game_version":"synthetic-1","event_schema_versions":[1],"companion_versions":["0.1.x"],"save_parser_versions":[1],"catalog_parser_versions":[1],"probe_required":false}]
        }))
        .unwrap()
    }

    fn event() -> &'static str {
        r#"{"schema_version":1,"companion_version":"0.1.0","game_version":"synthetic-1","profile_id":"1849811906","session_id":"00000000-0000-0000-0000-000000000000","sequence":1,"type":"item_obtained","payload":{"item_id":"paper_pondshell","count":1}}"#
    }
}
