use std::io::Write;
use tersh::preview::{PreviewKind, preview_file};

#[test]
fn previews_utf8_text_with_line_numbers() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.txt");
    std::fs::write(&path, "alpha\nbeta\n").unwrap();

    let preview = preview_file(&path).unwrap();

    assert_eq!(preview.kind, PreviewKind::Text);
    assert!(preview.lines.iter().any(|line| line.contains("1  alpha")));
}

#[test]
fn binary_preview_never_emits_raw_control_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bin.dat");
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(&[0, 159, 146, 150, 27]).unwrap();

    let preview = preview_file(&path).unwrap();

    assert_eq!(preview.kind, PreviewKind::Binary);
    assert!(preview.lines.join("\n").contains("Binary file"));
}

#[test]
fn long_unicode_lines_truncate_without_panicking() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("unicode.txt");
    std::fs::write(&path, "😀".repeat(5000)).unwrap();

    let preview = preview_file(&path).unwrap();

    assert_eq!(preview.kind, PreviewKind::Text);
    assert!(preview.lines[0].contains("[truncated]"));
}

#[test]
fn directory_preview_is_safe_message() {
    let dir = tempfile::tempdir().unwrap();

    let preview = preview_file(dir.path()).unwrap();

    assert_eq!(preview.kind, PreviewKind::Directory);
    assert!(preview.lines.join("\n").contains("Directory selected"));
}

#[cfg(unix)]
#[test]
fn symlink_preview_does_not_follow_target() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.txt");
    let link = dir.path().join("link.txt");
    std::fs::write(&target, "secret target content").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let preview = preview_file(&link).unwrap();
    let rendered = preview.lines.join("\n");

    assert_eq!(preview.kind, PreviewKind::Symlink);
    assert!(rendered.contains("Symlink"));
    assert!(rendered.contains("target.txt"));
    assert!(!rendered.contains("secret target content"));
}

#[cfg(unix)]
#[test]
fn fifo_preview_is_unsupported_without_blocking() {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let dir = tempfile::tempdir().unwrap();
    let fifo = dir.path().join("pipe");
    let fifo_c = CString::new(fifo.as_os_str().as_bytes()).unwrap();
    let result = unsafe { libc::mkfifo(fifo_c.as_ptr(), 0o600) };
    assert_eq!(result, 0);

    let preview = preview_file(&fifo).unwrap();

    assert_eq!(preview.kind, PreviewKind::Unsupported);
    assert!(preview.lines.join("\n").contains("Unsupported"));
}

#[test]
fn many_short_lines_are_capped() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("many.txt");
    let body = (0..25_000)
        .map(|index| format!("line-{index}"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, body).unwrap();

    let preview = preview_file(&path).unwrap();

    assert_eq!(preview.kind, PreviewKind::Text);
    assert!(preview.truncated);
    assert!(preview.lines.len() <= 20_001);
    assert!(preview.lines.last().unwrap().contains("truncated"));
}
