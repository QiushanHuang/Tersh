use tersh::clipboard::{
    ClipboardMode, MAX_OSC52_INPUT_BYTES, osc52_sequence, write_clipboard_with_mode,
};

#[test]
fn osc52_sequence_encodes_clipboard_payload() {
    assert_eq!(osc52_sequence("abc").unwrap(), "\u{1b}]52;c;YWJj\u{7}");
}

#[test]
fn osc52_sequence_rejects_oversized_clipboard_payloads() {
    let payload = "x".repeat(MAX_OSC52_INPUT_BYTES + 1);

    let err = osc52_sequence(&payload).unwrap_err();

    assert!(err.to_string().contains("too large"));
}

#[test]
fn clipboard_off_mode_does_not_emit_terminal_escape_sequence() {
    let mut output = Vec::new();

    let emitted =
        write_clipboard_with_mode(&mut output, "secret path", ClipboardMode::Off).unwrap();

    assert!(!emitted);
    assert!(output.is_empty());
}
