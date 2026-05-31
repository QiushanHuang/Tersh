use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};
use tersh::{
    app::{App, Command},
    ui::draw,
};

fn render_app(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| draw(frame, app)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>()
}

#[test]
fn wide_layout_contains_compact_info_pane_and_quit_keys() {
    let app = App::for_test();

    let buffer = render_app(&app, 120, 30);
    assert!(buffer.contains("Tersh"));
    assert!(buffer.contains("items 2"));
    assert!(buffer.contains("sort kind asc"));
    assert!(buffer.contains("Files"));
    assert!(buffer.contains("Preview"));
    assert!(buffer.contains("Inspector"));
    assert!(buffer.contains("q quit"));
    assert!(buffer.contains("^C"));
}

#[test]
fn medium_layout_keeps_core_exit_keys_visible() {
    let app = App::for_test();

    let buffer = render_app(&app, 100, 30);
    assert!(buffer.contains("q quit"));
    assert!(buffer.contains("^C"));
    assert!(buffer.contains("^G"));
}

#[test]
fn prompt_footer_is_mode_specific() {
    let mut app = App::for_test();
    app.apply(Command::OpenFilter);

    let buffer = render_app(&app, 100, 30);

    assert!(buffer.contains("filter"));
    assert!(buffer.contains("Enter apply"));
    assert!(buffer.contains("^G cancel"));
    assert!(buffer.contains("^C force"));
    assert!(!buffer.contains("yy/"));
    assert!(!buffer.contains("q quit"));
}

#[test]
fn y_prefix_footer_shows_chord_options() {
    let mut app = App::for_test();
    app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));

    let buffer = render_app(&app, 100, 30);

    assert!(buffer.contains("y_"));
    assert!(buffer.contains("f name"));
    assert!(buffer.contains("r rel"));
    assert!(buffer.contains("a abs"));
}

#[test]
fn workbench_operational_chrome_is_ascii() {
    let app = App::for_test();

    let buffer = render_app(&app, 120, 30);

    assert!(buffer.is_ascii());
}

#[test]
fn compact_prompt_footer_keeps_escape_controls_visible() {
    let mut app = App::for_test();
    app.apply(Command::OpenFilter);

    let buffer = render_app(&app, 40, 10);

    assert!(buffer.contains("filter"));
    assert!(buffer.contains("^G"));
    assert!(buffer.contains("^C"));
    assert!(!buffer.contains("q quit"));
}

#[test]
fn preview_search_footer_is_mode_specific() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("long.txt"), "alpha\nbeta\n").unwrap();
    let mut app = App::new(dir.path().to_path_buf()).unwrap();
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));

    let buffer = render_app(&app, 100, 30);

    assert!(buffer.contains("find"));
    assert!(buffer.contains("Enter find"));
    assert!(buffer.contains("^G cancel"));
    assert!(!buffer.contains("n/N"));
    assert!(!buffer.contains("q/^G close"));
}

#[test]
fn compact_preview_search_footer_keeps_escape_controls_visible() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("long.txt"), "alpha\nbeta\n").unwrap();
    let mut app = App::new(dir.path().to_path_buf()).unwrap();
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));

    let buffer = render_app(&app, 40, 10);

    assert!(buffer.contains("find"));
    assert!(buffer.contains("^G"));
    assert!(buffer.contains("^C"));
}

#[test]
fn delete_confirmation_names_target() {
    let mut app = App::for_test();
    app.apply(Command::PermanentDelete);

    let buffer = render_app(&app, 100, 30);

    assert!(buffer.contains("Permanent delete"));
    assert!(buffer.contains("targets: 1"));
    assert!(buffer.contains("/tmp/tersh-test/src"));
    assert!(buffer.contains("type delete"));
}

#[test]
fn delete_confirmation_lists_multiple_targets() {
    let mut app = App::for_test();
    app.handle_command(Command::ToggleSelect);
    app.handle_command(Command::Down);
    app.handle_command(Command::ToggleSelect);
    app.apply(Command::PermanentDelete);

    let buffer = render_app(&app, 100, 30);

    assert!(buffer.contains("targets: 2 selected"));
    assert!(buffer.contains("1. /tmp/tersh-test/README.md"));
    assert!(buffer.contains("2. /tmp/tersh-test/src"));
}

#[test]
fn render_escapes_paths_and_prompt_input() {
    let dir = tempfile::tempdir().unwrap();
    let weird = dir.path().join("line\n\u{1b}[31m");
    std::fs::create_dir(&weird).unwrap();
    std::fs::write(weird.join("file.txt"), "body").unwrap();
    let mut app = App::new(weird).unwrap();
    app.apply(Command::OpenFilter);
    app.handle_key(KeyEvent::new(KeyCode::Char('\u{1b}'), KeyModifiers::NONE));

    let buffer = render_app(&app, 100, 30);

    assert!(buffer.contains("\\n"));
    assert!(buffer.contains("\\u{1b}"));
}

#[test]
fn preview_errors_and_logs_escape_control_characters() {
    let dir = tempfile::tempdir().unwrap();
    let weird = dir.path().join("bad\n\u{1b}[31m");
    let mut app = App::new(dir.path().to_path_buf()).unwrap();
    app.force_cwd_for_test(weird);

    let buffer = render_app(&app, 100, 30);

    assert!(buffer.contains("\\n"));
    assert!(buffer.contains("\\u{1b}"));
}

#[test]
fn trash_confirmation_shows_multi_selection_count() {
    let mut app = App::for_test();
    app.handle_command(Command::ToggleSelect);
    app.handle_command(Command::Down);
    app.handle_command(Command::ToggleSelect);
    app.apply(Command::Trash);

    let buffer = render_app(&app, 100, 30);

    assert!(buffer.contains("Move to .tersh-trash"));
    assert!(buffer.contains("targets: 2"));
    assert!(buffer.contains("type trash"));
}

#[test]
fn compact_layout_keeps_survival_controls_visible() {
    let app = App::for_test();

    let buffer = render_app(&app, 40, 10);

    assert!(buffer.contains("Files"));
    assert!(buffer.contains("q quit"));
    assert!(buffer.contains("? help"));
    assert!(buffer.contains("^G"));
    assert!(buffer.contains("^C"));
}

#[test]
fn compact_layout_shows_focused_status_context() {
    let app = App::for_test();

    let buffer = render_app(&app, 70, 12);

    assert!(buffer.contains("Status"));
    assert!(buffer.contains("dir src"));
    assert!(buffer.contains("selected 0"));
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

    let buffer = render_app(&app, 60, 12);
    assert!(buffer.contains("item-29.txt"));
}
