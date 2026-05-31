use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tersh::app::{App, Command, Mode};

#[test]
fn ctrl_g_closes_filter_before_quitting() {
    let mut app = App::for_test();

    app.apply(Command::OpenFilter);
    app.handle_key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL));

    assert_eq!(app.mode(), Mode::Normal);
    assert!(!app.should_quit());
}

#[test]
fn escape_cancels_filter_mode() {
    let mut app = App::for_test();

    app.apply(Command::OpenFilter);
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

    assert_eq!(app.mode(), Mode::Normal);
}

#[test]
fn q_quits_only_from_normal_mode() {
    let mut app = App::for_test();

    app.apply(Command::OpenHelp);
    app.apply(Command::Quit);

    assert_eq!(app.mode(), Mode::Normal);
    assert!(!app.should_quit());

    app.apply(Command::Quit);

    assert!(app.should_quit());
}

#[test]
fn force_quit_sets_quit_from_any_mode() {
    let mut app = App::for_test();

    app.apply(Command::OpenHelp);
    app.apply(Command::ForceQuit);

    assert!(app.should_quit());
}

#[test]
fn filter_mode_treats_letters_and_backspace_as_text_input() {
    let mut app = App::for_test();

    app.apply(Command::OpenFilter);
    app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));

    assert_eq!(app.input(), "r");
    assert_eq!(app.mode(), Mode::Filter);
}

#[test]
fn q_is_text_inside_filter_mode() {
    let mut app = App::for_test();

    app.apply(Command::OpenFilter);
    app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));

    assert_eq!(app.input(), "q");
    assert_eq!(app.mode(), Mode::Filter);
    assert!(!app.should_quit());
}

#[test]
fn ctrl_c_force_quits_from_input_mode() {
    let mut app = App::for_test();

    app.apply(Command::OpenFilter);
    app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));

    assert!(app.should_quit());
}

#[test]
fn confirm_mode_accepts_delete_confirmation_text() {
    let mut app = App::for_test();

    app.apply(Command::PermanentDelete);
    for ch in "delete".chars() {
        app.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }

    assert_eq!(app.input(), "delete");
    assert_eq!(app.mode(), Mode::ConfirmDelete);
}

#[test]
fn changing_directory_clears_previous_selection() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("old.txt"), "old").unwrap();
    std::fs::create_dir(dir.path().join("subdir")).unwrap();
    let mut app = App::new(dir.path().to_path_buf()).unwrap();

    app.handle_command(Command::Down);
    app.handle_command(Command::ToggleSelect);
    assert_eq!(app.selected_len(), 1);
    app.handle_command(Command::Up);
    app.handle_command(Command::Open);

    assert_eq!(app.selected_len(), 0);
}

#[test]
fn gg_requires_two_g_keypresses_to_jump_to_first_item() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), "a").unwrap();
    std::fs::write(dir.path().join("b.txt"), "b").unwrap();
    let mut app = App::new(dir.path().to_path_buf()).unwrap();
    app.handle_command(Command::Last);

    app.handle_key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
    assert_eq!(app.cursor(), 1);

    app.handle_key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
    assert_eq!(app.cursor(), 0);
}

#[test]
fn cancel_clears_pending_y_and_g_prefixes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), "a").unwrap();
    std::fs::write(dir.path().join("b.txt"), "b").unwrap();
    let mut app = App::new(dir.path().to_path_buf()).unwrap();
    app.handle_command(Command::Last);

    app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL));
    app.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE));
    assert_eq!(app.last_clipboard_text(), None);

    app.handle_key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
    assert_eq!(app.cursor(), 1);
}

#[test]
fn page_keys_move_directory_cursor_by_page() {
    let dir = tempfile::tempdir().unwrap();
    for index in 0..25 {
        std::fs::write(dir.path().join(format!("item-{index:02}.txt")), "x").unwrap();
    }
    let mut app = App::new(dir.path().to_path_buf()).unwrap();

    app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
    assert_eq!(app.cursor(), 10);

    app.handle_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE));
    assert_eq!(app.cursor(), 0);
}

#[test]
fn sort_keys_cycle_sort_and_reverse_file_order() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("small.txt"), "x").unwrap();
    std::fs::write(dir.path().join("large.txt"), "xxxx").unwrap();
    let mut app = App::new(dir.path().to_path_buf()).unwrap();

    app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE));
    assert_eq!(app.sort_label(), "size asc");
    assert_eq!(app.entries()[0].name, "small.txt");

    app.handle_key(KeyEvent::new(KeyCode::Char('S'), KeyModifiers::SHIFT));
    assert_eq!(app.sort_label(), "size desc");
    assert_eq!(app.entries()[0].name, "large.txt");
}

#[cfg(unix)]
#[test]
fn edit_rejects_symlink_targets() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.txt");
    let link = dir.path().join("link.txt");
    std::fs::write(&target, "target").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let mut app = App::new(dir.path().to_path_buf()).unwrap();

    assert_eq!(app.entries()[app.cursor()].name, "link.txt");
    app.handle_command(Command::Edit);

    assert!(app.logs().iter().any(|log| log.contains("regular files")));
}

#[test]
fn preview_page_keys_scroll_fullscreen_preview() {
    let dir = tempfile::tempdir().unwrap();
    let body = (0..40)
        .map(|index| format!("line-{index:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(dir.path().join("long.txt"), body).unwrap();
    let mut app = App::new(dir.path().to_path_buf()).unwrap();

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(app.mode(), Mode::Preview);

    app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
    assert_eq!(app.preview_offset(), 10);

    app.handle_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE));
    assert_eq!(app.preview_offset(), 0);
}

#[test]
fn preview_q_closes_and_q_force_quits() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("item.txt"), "item").unwrap();
    let mut app = App::new(dir.path().to_path_buf()).unwrap();

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
    assert_eq!(app.mode(), Mode::Normal);
    assert!(!app.should_quit());

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::NONE));
    assert!(app.should_quit());
}

#[test]
fn help_q_closes_and_q_force_quits() {
    let mut app = App::for_test();

    app.apply(Command::OpenHelp);
    app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
    assert_eq!(app.mode(), Mode::Normal);
    assert!(!app.should_quit());

    app.apply(Command::OpenHelp);
    app.handle_key(KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::NONE));
    assert!(app.should_quit());
}

#[test]
fn preview_arrow_keys_scroll_by_line() {
    let dir = tempfile::tempdir().unwrap();
    let body = (0..20)
        .map(|index| format!("line-{index:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(dir.path().join("long.txt"), body).unwrap();
    let mut app = App::new(dir.path().to_path_buf()).unwrap();

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    assert_eq!(app.preview_offset(), 1);

    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    assert_eq!(app.preview_offset(), 0);
}

#[test]
fn home_and_end_keys_jump_directory_cursor_to_edges() {
    let dir = tempfile::tempdir().unwrap();
    for index in 0..5 {
        std::fs::write(dir.path().join(format!("item-{index:02}.txt")), "x").unwrap();
    }
    let mut app = App::new(dir.path().to_path_buf()).unwrap();

    app.handle_key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
    assert_eq!(app.cursor(), 4);

    app.handle_key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));
    assert_eq!(app.cursor(), 0);
}

#[test]
fn reload_failure_clears_previous_selection() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("old.txt"), "old").unwrap();
    let mut app = App::new(dir.path().to_path_buf()).unwrap();
    app.handle_command(Command::ToggleSelect);
    assert_eq!(app.selected_len(), 1);

    app.force_cwd_for_test(dir.path().join("missing"));

    assert_eq!(app.selected_len(), 0);
    assert!(app.entries().is_empty());
}

#[test]
fn goto_prompt_changes_to_specified_directory() {
    let dir = tempfile::tempdir().unwrap();
    let child = dir.path().join("child");
    std::fs::create_dir(&child).unwrap();
    let mut app = App::new(dir.path().to_path_buf()).unwrap();

    app.handle_key(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE));
    for ch in child.to_string_lossy().chars() {
        app.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert_eq!(app.cwd(), child.canonicalize().unwrap());
}

#[test]
fn y_prefix_copies_name_relative_and_absolute_paths_as_text() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("file name.txt"), "x").unwrap();
    let mut app = App::new(dir.path().to_path_buf()).unwrap();

    app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE));
    assert_eq!(app.last_clipboard_text().unwrap(), "file name.txt");

    app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));
    assert_eq!(app.last_clipboard_text().unwrap(), "file name.txt");

    app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
    assert!(
        app.last_clipboard_text()
            .unwrap()
            .ends_with("file name.txt")
    );
}

#[test]
fn yy_copies_file_and_p_pastes_into_current_directory() {
    let source_dir = tempfile::tempdir().unwrap();
    let target_dir = tempfile::tempdir().unwrap();
    std::fs::write(source_dir.path().join("item.txt"), "item").unwrap();
    let mut app = App::new(source_dir.path().to_path_buf()).unwrap();

    app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));
    app.force_cwd_for_test(target_dir.path().to_path_buf());
    app.handle_key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE));

    assert_eq!(
        std::fs::read_to_string(target_dir.path().join("item.txt")).unwrap(),
        "item"
    );
}

#[test]
fn x_cuts_file_and_p_moves_into_current_directory() {
    let source_dir = tempfile::tempdir().unwrap();
    let target_dir = tempfile::tempdir().unwrap();
    let source = source_dir.path().join("cut.txt");
    std::fs::write(&source, "cut").unwrap();
    let mut app = App::new(source_dir.path().to_path_buf()).unwrap();

    app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
    app.force_cwd_for_test(target_dir.path().to_path_buf());
    app.handle_key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE));

    assert!(!source.exists());
    assert_eq!(
        std::fs::read_to_string(target_dir.path().join("cut.txt")).unwrap(),
        "cut"
    );
}

#[test]
fn copy_to_and_move_to_use_typed_destination_directory() {
    let source_dir = tempfile::tempdir().unwrap();
    let target_dir = tempfile::tempdir().unwrap();
    std::fs::write(source_dir.path().join("copy.txt"), "copy").unwrap();
    std::fs::write(source_dir.path().join("move.txt"), "move").unwrap();
    let mut app = App::new(source_dir.path().to_path_buf()).unwrap();

    app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
    for ch in target_dir.path().to_string_lossy().chars() {
        app.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(target_dir.path().join("copy.txt").exists());

    app.handle_command(Command::Down);
    let moving = source_dir.path().join("move.txt");
    app.handle_key(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE));
    for ch in target_dir.path().to_string_lossy().chars() {
        app.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(!moving.exists());
    assert!(target_dir.path().join("move.txt").exists());
}
