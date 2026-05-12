use tersh::fs_core::{escape_display, format_size};

#[test]
fn escapes_control_characters_for_terminal_display() {
    assert_eq!(escape_display("a\nb\t\u{1b}[31m"), "a\\nb\\t\\u{1b}[31m");
}

#[test]
fn formats_sizes_for_compact_terminal_rows() {
    assert_eq!(format_size(42), "42 B");
    assert_eq!(format_size(2048), "2.0 KiB");
}
