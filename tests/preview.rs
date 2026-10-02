use std::io::Write;
use tersh::preview::{PreviewKind, preview_file, preview_file_structured};

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
fn binary_preview_detects_control_bytes_after_initial_probe_window() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("late-bin.dat");
    let mut bytes = vec![b'a'; 70 * 1024];
    bytes.push(0);
    std::fs::write(&path, bytes).unwrap();

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

#[test]
fn structured_json_preserves_raw_access_and_shows_nested_fields() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("record.json");
    std::fs::write(&path, r#"{"name":"alpha","nested":{"count":2}}"#).unwrap();
    let raw = preview_file(&path).unwrap();
    let structured = preview_file_structured(&path).unwrap();
    assert_eq!(raw.lines.len(), 1);
    assert!(structured.lines[0].starts_with("[JSON"));
    assert!(
        structured
            .lines
            .iter()
            .any(|line| line.contains("\"count\": 2"))
    );
    assert!(structured.lines.len() > raw.lines.len());
}

#[test]
fn structured_json_preserves_number_tokens_without_float_rounding() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("exact.json");
    std::fs::write(&path, r#"{"id":18446744073709551617,"tiny":1e-400}"#).unwrap();
    let preview = preview_file_structured(&path).unwrap();
    let rendered = preview.lines.join("\n");
    assert!(rendered.contains("18446744073709551617"), "{rendered}");
    assert!(rendered.contains("1e-400"), "{rendered}");
}

#[test]
fn structured_json_preserves_duplicate_fields_order_and_string_escapes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("duplicates.json");
    std::fs::write(
        &path,
        r#"{"z":1,"a":"quote: \" slash: \\ newline: \n","z":2}"#,
    )
    .unwrap();
    let preview = preview_file_structured(&path).unwrap();
    let rendered = preview.lines.join("\n");
    assert_eq!(rendered.matches("\"z\":").count(), 2, "{rendered}");
    assert!(rendered.find("\"z\": 1").unwrap() < rendered.find("\"a\":").unwrap());
    assert!(
        rendered.contains(r#"quote: \" slash: \\ newline: \n"#),
        "{rendered}"
    );
}

#[test]
fn structured_csv_handles_quoted_commas_quotes_and_embedded_newlines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("records.csv");
    std::fs::write(
        &path,
        "name,note,count\r\n\"a,b\",\"say \"\"hi\"\"\nthen go\",3\r\n",
    )
    .unwrap();
    let preview = preview_file_structured(&path).unwrap();
    assert!(preview.lines[0].starts_with("[CSV"));
    assert!(preview.lines[0].contains("2 rows"));
    assert!(preview.lines[0].contains("3 columns"));
    let joined = preview.lines.join("\n");
    assert!(joined.contains("a,b"));
    assert!(joined.contains("say \"hi\"\\nthen go"));
    assert!(!preview.truncated);
}

#[test]
fn invalid_and_oversized_structures_explicitly_fall_back_to_bounded_text() {
    let dir = tempfile::tempdir().unwrap();
    for (name, body) in [
        ("invalid.json", "{broken".to_string()),
        ("invalid.csv", "a,b\n\"unfinished".to_string()),
        (
            "columns.csv",
            (0..80).map(|_| "x").collect::<Vec<_>>().join(","),
        ),
        ("large.json", format!("\"{}\"", "x".repeat(300 * 1024))),
    ] {
        let path = dir.path().join(name);
        std::fs::write(&path, body).unwrap();
        let preview = preview_file_structured(&path).unwrap();
        assert!(
            preview.lines[0].contains("raw text"),
            "{}: {:?}",
            name,
            preview.lines
        );
        assert_eq!(preview.kind, PreviewKind::Text);
        assert!(preview.lines.len() <= 2_002);
    }
}

#[test]
fn structured_preview_caps_rows_and_display_width_with_visible_truncation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("large.csv");
    let body = format!("name,value\n{}", "abc,def\n".repeat(3_000));
    std::fs::write(&path, body).unwrap();
    let preview = preview_file_structured(&path).unwrap();
    assert!(preview.truncated);
    assert!(preview.lines.len() <= 2_002);

    std::fs::write(&path, format!("name,value\n{},other\n", "界".repeat(400))).unwrap();
    let preview = preview_file_structured(&path).unwrap();
    assert!(preview.truncated);
    assert!(
        preview
            .lines
            .iter()
            .all(|line| unicode_width::UnicodeWidthStr::width(line.as_str()) <= 512)
    );
}

#[test]
fn unified_diff_preview_keeps_hunks_and_add_remove_prefixes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("change.patch");
    std::fs::write(&path, "--- a/file\n+++ b/file\n@@ -1 +1 @@\n-old\n+new\n").unwrap();
    let preview = preview_file_structured(&path).unwrap();
    assert!(preview.lines[0].starts_with("[Diff"));
    assert!(preview.lines.iter().any(|line| line == "-old"));
    assert!(preview.lines.iter().any(|line| line == "+new"));
}
