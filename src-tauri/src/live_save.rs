use crate::{
    domain::{CompanionEvent, EventEnvelope, ProfileId},
    safety::paths::{GameSavePath, GameSavePathError},
    tracking::log_lines::event_json_from_log_line,
};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::Path,
    process::Command,
};

const LOG_TAIL_LIMIT: usize = 256 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum LiveSaveError {
    #[error("could not inspect Fields of Mistria saves: {0}")]
    SavesDirectory(#[from] std::io::Error),
    #[error(transparent)]
    SavePath(#[from] GameSavePathError),
}

/// Selects only saves whose filename begins with the exact profile id emitted
/// by the companion. It never falls back to another profile's newer save.
pub fn newest_save_for_profile(
    saves_directory: &Path,
    profile_id: &ProfileId,
) -> Result<Option<GameSavePath>, LiveSaveError> {
    let expected = format!("game-{}-", profile_id.as_str());
    let candidate = fs::read_dir(saves_directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&expected) && name.ends_with(".sav"))
        })
        .filter_map(|path| {
            let modified = fs::metadata(&path).ok()?.modified().ok()?;
            GameSavePath::new(saves_directory, &path)
                .ok()
                .map(|save| (modified, save))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, save)| save);
    Ok(candidate)
}

/// Reads at most the end of the companion's isolated log and returns the last
/// exact profile activation. Item events are deliberately not replayed here;
/// the live tail owns those instant updates.
pub fn active_profile_from_log(log: &Path) -> Result<Option<ProfileId>, LiveSaveError> {
    let bytes = read_log_tail(log)?;
    let tail = String::from_utf8_lossy(&bytes);
    for line in tail.lines().rev() {
        let Some(json) = event_json_from_log_line(line) else {
            continue;
        };
        let Ok(event) = EventEnvelope::from_json(json) else {
            continue;
        };
        if matches!(event.event, CompanionEvent::ProfileActivated) {
            return Ok(Some(event.profile_id));
        }
    }
    Ok(None)
}

fn read_log_tail(log: &Path) -> Result<Vec<u8>, LiveSaveError> {
    let mut file = File::open(log)?;
    let length = file.metadata()?.len();
    let length = length.min(LOG_TAIL_LIMIT as u64);
    file.seek(SeekFrom::End(-(length as i64)))?;
    let mut bytes = vec![0; length as usize];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

/// Process detection is merely a safety signal. Save selection still requires
/// the companion-confirmed profile and a validated path under `saves`.
pub fn game_is_running() -> bool {
    #[cfg(windows)]
    {
        Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq FieldsOfMistria.exe", "/NH"])
            .output()
            .ok()
            .is_some_and(|output| game_process_is_running(&String::from_utf8_lossy(&output.stdout)))
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn game_process_is_running(tasklist: &str) -> bool {
    tasklist.lines().any(|line| {
        line.trim_start()
            .to_ascii_lowercase()
            .starts_with("fieldsofmistria.exe")
    })
}

#[cfg(test)]
mod tests {
    use super::{
        active_profile_from_log, game_process_is_running, newest_save_for_profile, read_log_tail,
        LOG_TAIL_LIMIT,
    };
    use crate::domain::ProfileId;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn selects_only_the_companion_confirmed_profiles_save() {
        let directory = tempdir().unwrap();
        let saves = directory.path().join("saves");
        fs::create_dir(&saves).unwrap();
        let matching = saves.join("game-331655283-100.sav");
        fs::write(&matching, b"ari").unwrap();
        fs::write(saves.join("game-1849811906-999.sav"), b"amelia").unwrap();
        fs::write(saves.join("game-331655283.txt"), b"not a save").unwrap();

        let selected = newest_save_for_profile(&saves, &ProfileId::new("331655283").unwrap())
            .unwrap()
            .unwrap();

        assert_eq!(selected.as_path(), fs::canonicalize(matching).unwrap());
    }

    #[test]
    fn finds_the_latest_profile_activation_without_replaying_old_item_events() {
        let directory = tempdir().unwrap();
        let log = directory.path().join("mistria_tracker_companion.log");
        fs::write(
            &log,
            concat!(
                "MISTRIA_TRACKER_EVENT| {\"schema_version\":1,\"companion_version\":\"0.1.0\",\"game_version\":\"1.0.4\",\"profile_id\":\"1849811906\",\"session_id\":\"00000000-0000-0000-0000-000000000000\",\"sequence\":1,\"type\":\"profile_activated\"}\n",
                "MISTRIA_TRACKER_EVENT| {\"schema_version\":1,\"companion_version\":\"0.1.0\",\"game_version\":\"1.0.4\",\"profile_id\":\"331655283\",\"session_id\":\"00000000-0000-0000-0000-000000000000\",\"sequence\":2,\"type\":\"item_obtained\",\"payload\":{\"item_id\":\"fiber\",\"count\":1}}\n",
                "MISTRIA_TRACKER_EVENT| {\"schema_version\":1,\"companion_version\":\"0.1.0\",\"game_version\":\"1.0.4\",\"profile_id\":\"331655283\",\"session_id\":\"00000000-0000-0000-0000-000000000000\",\"sequence\":3,\"type\":\"profile_activated\"}\n"
            ),
        )
        .unwrap();

        assert_eq!(
            active_profile_from_log(&log).unwrap().unwrap().as_str(),
            "331655283"
        );
    }

    #[test]
    fn reads_only_the_bounded_end_of_a_large_companion_log() {
        let directory = tempdir().unwrap();
        let log = directory.path().join("mistria_tracker_companion.log");
        let activation = "MISTRIA_TRACKER_EVENT| {\"schema_version\":1,\"companion_version\":\"0.1.0\",\"game_version\":\"1.0.4\",\"profile_id\":\"331655283\",\"session_id\":\"00000000-0000-0000-0000-000000000000\",\"sequence\":1,\"type\":\"profile_activated\"}\n";
        let mut contents = vec![b'x'; LOG_TAIL_LIMIT];
        contents.extend_from_slice(b"\n");
        contents.extend_from_slice(activation.as_bytes());
        fs::write(&log, contents).unwrap();

        let tail = read_log_tail(&log).unwrap();

        assert!(tail.len() <= LOG_TAIL_LIMIT);
        assert!(String::from_utf8_lossy(&tail).contains(activation.trim()));
    }

    #[test]
    fn recognizes_only_the_fields_of_mistria_process_from_tasklist_output() {
        assert!(game_process_is_running(
            "FieldsOfMistria.exe            1048 Console                    1     92,000 K"
        ));
        assert!(!game_process_is_running(
            "MistriaTracker.exe             1180 Console                    1     65,000 K"
        ));
    }
}
