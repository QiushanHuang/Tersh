use std::fs::OpenOptions;
use std::io::Write;
use tersh::log_reader::LogReader;

fn append(path: &std::path::Path, bytes: &[u8]) {
    OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap()
        .write_all(bytes)
        .unwrap();
}

#[test]
fn initial_tail_is_bounded_and_reads_latest_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("service.log");
    std::fs::write(&path, format!("{}latest\n", "old\n".repeat(100_000))).unwrap();
    let reader = LogReader::open(&path).unwrap();
    let snapshot = reader.snapshot();
    assert!(snapshot.lines.last().unwrap().contains("latest"));
    assert!(snapshot.truncated);
    assert!(snapshot.lines.len() <= 2_000);
    assert!(snapshot.byte_count <= 256 * 1024);
    assert!(snapshot.omitted_lines > 0);
}

#[test]
fn pause_preserves_read_position_and_resume_catches_up_in_bounded_polls() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("service.log");
    std::fs::write(&path, "first\n").unwrap();
    let mut reader = LogReader::open(&path).unwrap();
    reader.set_paused(true);
    let before = reader.snapshot();
    append(&path, "next\n".repeat(30_000).as_bytes());
    assert_eq!(reader.poll().unwrap().bytes_read, 0);
    assert_eq!(reader.snapshot().offset, before.offset);
    assert_eq!(reader.snapshot().lines, before.lines);
    reader.set_paused(false);
    let first_poll = reader.poll().unwrap();
    assert!(first_poll.bytes_read <= 64 * 1024);
    assert!(first_poll.has_more);
    assert!(first_poll.changed);
    for _ in 0..4 {
        reader.poll().unwrap();
    }
    let snapshot = reader.snapshot();
    assert!(snapshot.lines.last().unwrap().contains("next"));
    assert!(snapshot.lines.len() <= 2_000);
    assert!(snapshot.byte_count <= 256 * 1024);
}

#[test]
fn partial_utf8_and_partial_lines_survive_poll_boundaries() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("unicode.log");
    std::fs::write(&path, b"hello ").unwrap();
    let mut reader = LogReader::open(&path).unwrap();
    append(&path, &[0xf0, 0x9f]);
    reader.poll().unwrap();
    assert!(!reader.snapshot().lines.join("\n").contains('\u{fffd}'));
    append(&path, &[0x98, 0x80, b'\n']);
    reader.poll().unwrap();
    assert_eq!(reader.snapshot().lines, vec!["hello 😀"]);
}

#[test]
fn oversized_lines_and_terminal_controls_are_bounded_and_visible() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("long.log");
    std::fs::write(&path, "").unwrap();
    let mut reader = LogReader::open(&path).unwrap();
    append(&path, &vec![0x1b; 300_000]);
    for _ in 0..6 {
        reader.poll().unwrap();
    }
    append(&path, b"\nend\n");
    reader.poll().unwrap();
    let snapshot = reader.snapshot();
    assert!(snapshot.truncated);
    assert!(snapshot.byte_count <= 256 * 1024);
    assert!(snapshot.lines.iter().all(|line| !line.contains('\x1b')));
    assert!(
        snapshot
            .lines
            .iter()
            .any(|line| line.contains("line truncated"))
    );
    assert_eq!(snapshot.lines.last().unwrap(), "end");
}

#[test]
fn file_truncation_and_rename_rotation_are_reported_and_followed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rotate.log");
    std::fs::write(&path, "old old old old\n").unwrap();
    let mut reader = LogReader::open(&path).unwrap();
    std::fs::write(&path, "new\n").unwrap();
    reader.poll().unwrap();
    assert!(reader.snapshot().status.contains("truncated"));
    assert_eq!(reader.snapshot().lines.last().unwrap(), "new");
    std::fs::rename(&path, dir.path().join("rotate.log.1")).unwrap();
    std::fs::write(&path, "rotated\n").unwrap();
    reader.poll().unwrap();
    assert!(reader.snapshot().status.contains("rotated"));
    assert_eq!(reader.snapshot().lines.last().unwrap(), "rotated");
}

#[cfg(unix)]
#[test]
fn symlinks_and_fifos_are_rejected_including_after_rotation() {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target");
    let link = dir.path().join("link");
    let fifo = dir.path().join("pipe");
    std::fs::write(&target, "private\n").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(LogReader::open(&link).is_err());
    let path_c = CString::new(fifo.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(path_c.as_ptr(), 0o600) }, 0);
    assert!(LogReader::open(&fifo).is_err());

    let path = dir.path().join("service.log");
    std::fs::write(&path, "safe\n").unwrap();
    let mut reader = LogReader::open(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(&target, &path).unwrap();
    assert!(reader.poll().is_err());
    assert!(!reader.snapshot().lines.join("\n").contains("private"));
}

#[test]
fn truncation_flag_reflects_omission_not_words_in_the_log() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("words.log");
    std::fs::write(&path, "message says [line truncated]\n").unwrap();
    let reader = LogReader::open(&path).unwrap();
    assert!(!reader.snapshot().truncated);

    std::fs::write(&path, vec![0x1b; 4_000]).unwrap();
    let reader = LogReader::open(&path).unwrap();
    assert!(reader.snapshot().truncated);
    assert!(reader.snapshot().lines[0].len() <= 16 * 1024);
}

#[test]
fn byte_budget_is_enforced_for_long_retained_lines_and_partial_tail() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("budget.log");
    std::fs::write(&path, "").unwrap();
    let mut reader = LogReader::open(&path).unwrap();
    for _ in 0..120 {
        append(&path, format!("{}\n", "x".repeat(4_000)).as_bytes());
        reader.poll().unwrap();
    }
    append(&path, &vec![0x1b; 4_000]);
    reader.poll().unwrap();
    let snapshot = reader.snapshot();
    assert!(snapshot.byte_count <= 256 * 1024);
    assert_eq!(
        snapshot.byte_count,
        snapshot.lines.iter().map(String::len).sum::<usize>()
    );
    assert!(snapshot.omitted_lines > 0);
    assert!(snapshot.lines.len() <= 2_000);
}

#[test]
fn missing_path_reports_error_and_recovers_when_rotation_completes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wait.log");
    std::fs::write(&path, "old\n").unwrap();
    let mut reader = LogReader::open(&path).unwrap();
    std::fs::rename(&path, dir.path().join("wait.log.1")).unwrap();
    assert!(reader.poll().is_err());
    assert!(reader.snapshot().status.contains("error"));
    std::fs::write(&path, "new\n").unwrap();
    assert!(reader.poll().unwrap().changed);
    assert!(reader.snapshot().status.contains("rotated"));
    assert_eq!(reader.snapshot().lines.last().unwrap(), "new");
}
