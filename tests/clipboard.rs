use tersh::clipboard::osc52_sequence;

#[test]
fn osc52_sequence_encodes_clipboard_payload() {
    assert_eq!(osc52_sequence("abc"), "\u{1b}]52;c;YWJj\u{7}");
}
