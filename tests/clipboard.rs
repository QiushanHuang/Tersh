use tersh::clipboard::{MAX_OSC52_INPUT_BYTES, osc52_sequence};

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
