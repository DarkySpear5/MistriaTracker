use mistria_tracker_lib::tracking::ReadOnlyLogTail;
use std::fs;
use tempfile::tempdir;

#[test]
fn tail_reads_only_complete_lines_appended_after_opening_without_changing_the_log() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("mmapi.log");
    fs::write(&path, "old line\n").unwrap();
    let mut tail = ReadOnlyLogTail::open(&path).unwrap();

    fs::write(&path, "old line\nfirst new line\npartial").unwrap();
    assert_eq!(tail.poll().unwrap(), vec!["first new line"]);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "old line\nfirst new line\npartial"
    );

    fs::write(&path, "old line\nfirst new line\npartial line\n").unwrap();
    assert_eq!(tail.poll().unwrap(), vec!["partial line"]);
}

#[test]
fn tail_resets_its_read_cursor_when_a_log_is_rotated_or_truncated() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("mmapi.log");
    fs::write(&path, "a deliberately longer old line\n").unwrap();
    let mut tail = ReadOnlyLogTail::open(&path).unwrap();

    fs::write(&path, "new\n").unwrap();

    assert_eq!(tail.poll().unwrap(), vec!["new"]);
}
