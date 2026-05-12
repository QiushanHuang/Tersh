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
