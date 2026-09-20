use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};
use std::time::{Duration, Instant};
use tersh::{
    app::{App, Command, Mode},
    cluster::{ClusterApp, ClusterCommand},
    keymap::Keymap,
};

fn key(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)
}
fn enter() -> KeyEvent {
    KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
}
fn wait(app: &mut App) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while app.job_active() {
        app.poll_job();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn render(app: &App) -> String {
    let mut terminal = Terminal::new(TestBackend::new(120, 30)).unwrap();
    terminal.draw(|f| tersh::ui::draw(f, app)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect()
}

#[test]
fn background_copy_allows_browsing_blocks_second_writer_and_quits_safely() {
    let root = tempfile::tempdir().unwrap();
    let destination = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("a.txt"), "payload").unwrap();
    std::fs::write(root.path().join("b.txt"), "other").unwrap();
    let mut app = App::new(root.path().into()).unwrap();
    app.handle_command(Command::Copy);
    app.force_cwd_for_test(destination.path().into());
    app.handle_command(Command::Paste);
    assert!(app.job_active());
    app.handle_command(Command::OpenFilter);
    assert_eq!(app.mode(), Mode::Filter);
    app.handle_command(Command::Cancel);
    app.handle_command(Command::Rename);
    assert_eq!(app.mode(), Mode::Normal);
    assert!(app.logs().iter().any(|l| l.contains("job active")));
    app.handle_command(Command::ForceQuit);
    assert!(!app.should_quit());
    assert!(app.exit_after_job());
    wait(&mut app);
    assert!(app.should_quit());
    assert!(root.path().join("a.txt").exists());
    assert!(app.last_job().is_some());
}

#[test]
fn trash_ui_roundtrip_survives_new_app_and_refuses_restore_conflict() {
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("keep.txt");
    std::fs::write(&original, "original").unwrap();
    let mut app = App::new(root.path().into()).unwrap();
    app.handle_command(Command::Trash);
    for ch in "trash".chars() {
        app.handle_key(key(ch));
    }
    app.handle_key(enter());
    wait(&mut app);
    assert!(!original.exists());
    drop(app);
    let mut app = App::new(root.path().into()).unwrap();
    app.handle_key(key('u'));
    assert_eq!(app.mode(), Mode::Trash);
    assert_eq!(app.trash_entries().len(), 1);
    assert!(render(&app).contains("keep.txt"));
    std::fs::write(&original, "conflict").unwrap();
    app.handle_key(enter());
    assert_eq!(app.mode(), Mode::ConfirmRestore);
    app.handle_key(enter());
    wait(&mut app);
    assert_eq!(std::fs::read_to_string(&original).unwrap(), "conflict");
    assert_eq!(app.trash_entries().len(), 1);
    assert_eq!(app.last_job().unwrap().failed.len(), 1);
    std::fs::remove_file(&original).unwrap();
    app.handle_key(enter());
    app.handle_key(enter());
    wait(&mut app);
    assert_eq!(std::fs::read_to_string(&original).unwrap(), "original");
    assert!(app.trash_entries().is_empty());
}

#[test]
fn remapped_file_keys_replace_legacy_and_match_menu_footer_and_prompt() {
    let map=Keymap::from_json(r#"{"files":{"rename":["F2"],"open_actions":["F3"],"open_filter":["F4"]},"input":{"submit":["F5"],"cancel":["F6"]},"actions":{"submit":["F7"],"cancel":["F8"]}}"#).unwrap();
    let mut app = App::for_test();
    app.set_keymap(map);
    app.handle_key(key('n'));
    assert_eq!(app.mode(), Mode::Normal);
    assert!(render(&app).contains("F3 actions"));
    assert!(render(&app).contains("F4 filter"));
    app.handle_key(KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE));
    for ch in "rename".chars() {
        app.handle_key(key(ch));
    }
    let text = render(&app);
    assert!(text.contains("F2"));
    assert!(text.contains("F7 run"));
    assert!(text.contains("F8"));
    app.handle_key(enter());
    assert!(app.actions().is_some());
    app.handle_key(KeyEvent::new(KeyCode::F(7), KeyModifiers::NONE));
    assert_eq!(app.mode(), Mode::Rename);
    assert!(render(&app).contains("F5 confirm"));
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(app.mode(), Mode::Rename);
    app.handle_key(KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE));
    assert_eq!(app.mode(), Mode::Normal);
}

#[test]
fn failed_printable_chords_replay_all_text_including_shift() {
    let map =
        Keymap::from_json(r#"{"input":{"submit":["Z Z"]},"cluster_filter":{"submit":["Z Z"]}}"#)
            .unwrap();
    let mut app = App::for_test();
    app.set_keymap(map.clone());
    app.handle_command(Command::OpenFilter);
    app.handle_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::SHIFT));
    app.handle_key(key('x'));
    assert_eq!(app.input(), "Zx");
    let mut cluster = ClusterApp::new(vec![]);
    cluster.set_keymap(map);
    cluster.apply(ClusterCommand::OpenFilter);
    cluster.handle_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::SHIFT));
    cluster.handle_key(key('x'));
    assert_eq!(cluster.filter(), "Zx");
}

#[test]
fn remapped_cluster_launch_filter_and_detail_keys_route_correctly() {
    let map=Keymap::from_json(r#"{"cluster":{"open_session":["F2"],"open_filter":["F3"],"open_detail":["F4"]},"cluster_detail":{"detail_down":["F5"]}}"#).unwrap();
    let mut app = ClusterApp::new(vec![]);
    app.set_keymap(map);
    assert_eq!(app.handle_key(key('s')), None);
    assert_eq!(
        app.handle_key(KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE)),
        Some(ClusterCommand::OpenSession)
    );
    app.handle_key(KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE));
    assert_eq!(app.mode(), tersh::cluster::ClusterMode::Filter);
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::F(4), KeyModifiers::NONE));
    let mut terminal = Terminal::new(TestBackend::new(120, 30)).unwrap();
    terminal.draw(|f| tersh::cluster_ui::draw(f, &app)).unwrap();
    let text = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();
    assert!(text.contains("F5"));
}

#[test]
fn finishing_old_move_does_not_clear_a_new_cut_buffer_at_the_same_path() {
    let root = tempfile::tempdir().unwrap();
    let destination = tempfile::tempdir().unwrap();
    let canonical = root.path().canonicalize().unwrap();
    let path = canonical.join("item.txt");
    std::fs::write(&path, "old").unwrap();
    let mut app = App::new(root.path().into()).unwrap();
    app.handle_command(Command::Cut);
    app.force_cwd_for_test(destination.path().into());
    app.handle_command(Command::Paste);
    let deadline = Instant::now() + Duration::from_secs(5);
    while path.exists() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    std::fs::write(&path, "new").unwrap();
    app.force_cwd_for_test(canonical);
    app.handle_command(Command::Cut);
    wait(&mut app);
    assert_eq!(app.copy_buffer_len(), 1);
    assert_eq!(std::fs::read_to_string(path).unwrap(), "new");
}

#[test]
fn restore_confirmation_keeps_target_filename_visible_on_narrow_terminals() {
    let root = tempfile::tempdir().unwrap();
    let parent = root.path().join("long-parent-directory-".repeat(4));
    std::fs::create_dir(&parent).unwrap();
    let original = parent.join("important-file.txt");
    std::fs::write(&original, "payload").unwrap();
    tersh::fs_ops::trash_path(&original, root.path()).unwrap();
    let mut app = App::new(root.path().into()).unwrap();
    app.handle_command(Command::OpenTrash);
    app.handle_command(Command::RestoreTrash);
    let mut terminal = Terminal::new(TestBackend::new(60, 20)).unwrap();
    terminal.draw(|f| tersh::ui::draw(f, &app)).unwrap();
    let text = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();
    assert!(text.contains("important-file.txt"));
    assert!(text.contains("Enter restore"));
}
