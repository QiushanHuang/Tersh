use tersh::fs_core::{escape_display, format_size, read_dir_entries};

#[test]
fn escapes_control_characters_for_terminal_display() {
    assert_eq!(escape_display("a\nb\t\u{1b}[31m"), "a\\nb\\t\\u{1b}[31m");
}

#[test]
fn formats_sizes_for_compact_terminal_rows() {
    assert_eq!(format_size(42), "42 B");
    assert_eq!(format_size(2048), "2.0 KiB");
}

#[test]
fn directory_entries_cache_lowercase_names_for_filtering_and_sorting() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Alpha.TXT"), "a").unwrap();

    let entries = read_dir_entries(dir.path(), false, "").unwrap();

    assert_eq!(entries[0].name, "Alpha.TXT");
    assert_eq!(entries[0].name_lower, "alpha.txt");
}
