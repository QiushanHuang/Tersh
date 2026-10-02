use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};
use tersh::{
    actions::{Action, ActionMenu},
    app::{App, Command},
    keymap::{KeyMatch, Keymap},
};

#[test]
fn recovery_can_be_found_by_user_task_words() {
    for query in ["recover", "恢复", "undo"] {
        let mut menu = ActionMenu::new(vec![Action::new("Trash recovery", "u", ())]);
        for ch in query.chars() {
            menu.input_char(ch);
        }
        assert_eq!(menu.matches().len(), 1, "{query}");
    }
}

#[test]
fn compact_file_footer_keeps_current_primary_action() {
    let mut app = App::for_test();
    app.handle_command(Command::Down);
    app.handle_command(Command::Copy);
    let mut term = Terminal::new(TestBackend::new(40, 18)).unwrap();
    term.draw(|f| tersh::ui::draw(f, &app)).unwrap();
    let buffer = term.backend().buffer();
    let footer = (16..18)
        .flat_map(|y| (0..40).map(move |x| buffer[(x, y)].symbol()))
        .collect::<String>();
    assert!(footer.contains("p paste"), "{footer}");
    assert!(footer.contains("o actions"), "{footer}");
}

#[test]
fn compact_footer_prioritizes_remapped_paste() {
    let mut app = App::for_test();
    app.set_keymap(Keymap::from_json(r#"{"files":{"paste":["Ctrl+Alt+x Ctrl+Alt+y"]}}"#).unwrap());
    app.handle_command(Command::Down);
    app.handle_command(Command::Copy);
    let mut term = Terminal::new(TestBackend::new(40, 18)).unwrap();
    term.draw(|f| tersh::ui::draw(f, &app)).unwrap();
    let text = term
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();
    assert!(text.contains(" paste"));
}

#[test]
fn new_cluster_controls_have_contextual_remappable_bindings() {
    let map = Keymap::default();
    for context in ["cluster", "cluster_detail"] {
        for (key, action) in [
            ('a', "toggle_attention"),
            ('E', "open_events"),
            ('P', "toggle_pause"),
        ] {
            assert_eq!(
                map.resolve(
                    context,
                    &[KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE)]
                ),
                KeyMatch::Action(action.into())
            );
        }
    }
}

#[test]
fn custom_chord_hints_follow_effective_bindings() {
    let map = Keymap::from_json(r#"{"files":{"copy":["z c"],"copy_name":["z n"]}}"#).unwrap();
    let hints = map.completions(
        "files",
        &[KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE)],
    );
    assert!(hints.contains(&("c".into(), "copy".into())));
    assert!(hints.contains(&("n".into(), "copy_name".into())));
    assert!(
        map.completions(
            "input",
            &[KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE)]
        )
        .is_empty()
    );
}

#[test]
fn disabled_actions_explain_but_do_not_dispatch() {
    use tersh::actions::MenuResult;
    let mut menu = ActionMenu::new(vec![
        Action::new("Paste buffer", "p", 42).disabled("Copy an item first"),
    ]);
    assert_eq!(
        menu.matches()[0].disabled_reason,
        Some("Copy an item first")
    );
    assert!(matches!(menu.handle_action("submit"), MenuResult::Pending));
}

#[test]
fn action_groups_keep_navigation_before_recovery() {
    let menu = ActionMenu::new(vec![
        Action::new("Restore", "u", 1).with_metadata("open_trash"),
        Action::new("Go to", ":", 2).with_metadata("open_goto"),
    ]);
    assert_eq!(menu.matches()[0].group, "Navigation");
}

#[test]
fn directory_metadata_is_not_presented_as_recursive_size() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("folder")).unwrap();
    let mut app = App::new(dir.path().into()).unwrap();
    app.handle_command(Command::ToggleSelect);
    let mut term = Terminal::new(TestBackend::new(160, 30)).unwrap();
    term.draw(|f| tersh::ui::draw(f, &app)).unwrap();
    let text = term
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();
    assert!(text.contains("not scanned"), "{text}");
    assert!(text.contains("1 dir"), "{text}");
}

#[test]
fn places_page_shows_saved_path_and_effective_controls() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = App::new(dir.path().into()).unwrap();
    let mut places = tersh::places::Places::memory();
    places
        .visit("local", "local", std::path::Path::new("/saved-project"))
        .unwrap();
    app.set_places(places);
    app.handle_command(Command::OpenPlaces);
    let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
    term.draw(|f| tersh::ui::draw(f, &app)).unwrap();
    let text = term
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();
    assert!(text.contains("Places"));
    assert!(text.contains("/saved-project"));
    assert!(text.contains("pin"));
}

#[test]
fn inspector_can_be_collapsed_without_hiding_preview() {
    let mut app = App::for_test();
    app.handle_command(Command::ToggleInspector);
    let mut term = Terminal::new(TestBackend::new(160, 30)).unwrap();
    term.draw(|f| tersh::ui::draw(f, &app)).unwrap();
    let text = term
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();
    assert!(!text.contains("Inspector"));
    assert!(text.contains("Preview"));
}
