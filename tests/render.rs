use ratatui::{Terminal, backend::TestBackend};
use tersh::{app::App, ui::draw};

#[test]
fn wide_layout_contains_compact_info_pane_and_quit_keys() {
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::for_test();

    terminal.draw(|frame| draw(frame, &app)).unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(buffer.contains("Files"));
    assert!(buffer.contains("Preview"));
    assert!(buffer.contains("Info"));
    assert!(buffer.contains("q quit"));
    assert!(buffer.contains("^C"));
}

#[test]
fn medium_layout_keeps_core_exit_keys_visible() {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::for_test();

    terminal.draw(|frame| draw(frame, &app)).unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(buffer.contains("q quit"));
    assert!(buffer.contains("^C"));
    assert!(buffer.contains("Esc"));
}
