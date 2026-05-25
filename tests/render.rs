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
    assert!(buffer.contains("^G"));
}

#[test]
fn long_directory_list_keeps_focused_item_visible() {
    let dir = tempfile::Builder::new()
        .prefix("tersh-list-")
        .tempdir_in("/tmp")
        .unwrap();
    for index in 0..30 {
        std::fs::write(dir.path().join(format!("item-{index:02}.txt")), "x").unwrap();
    }
    let mut app = App::new(dir.path().to_path_buf()).unwrap();
    app.handle_command(tersh::app::Command::Last);

    let backend = TestBackend::new(60, 12);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| draw(frame, &app)).unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(buffer.contains("item-29.txt"));
}
