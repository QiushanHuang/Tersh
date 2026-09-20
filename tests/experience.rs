use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};
use std::time::Duration;
use tersh::{
    app::{App, Command, Mode},
    cluster::{ClusterApp, ClusterCommand, HostSnapshot, ProbeReport},
    cluster_ui, metrics,
    theme::{self, Theme},
};

fn key(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)
}
fn enter() -> KeyEvent {
    KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
}

#[test]
fn menu_delete_still_requires_typed_confirmation_and_escape_preserves_target() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("keep.txt");
    std::fs::write(&path, "keep").unwrap();
    let mut app = App::new(dir.path().into()).unwrap();
    app.handle_key(key('o'));
    for ch in "permanently".chars() {
        app.handle_key(key(ch));
    }
    app.handle_key(enter());
    assert_eq!(app.mode(), Mode::ConfirmDelete);
    app.handle_key(enter());
    assert!(path.exists());
    assert_eq!(app.mode(), Mode::ConfirmDelete);
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(path.exists());
    assert_eq!(app.mode(), Mode::Normal);
}

#[test]
fn closing_menu_restores_preview_and_enter_with_no_results_does_nothing() {
    let mut app = App::for_test();
    app.handle_command(Command::Down);
    app.handle_command(Command::Open);
    app.handle_key(key('o'));
    for ch in "does-not-exist".chars() {
        app.handle_key(key(ch));
    }
    app.handle_key(enter());
    assert!(app.actions().is_some());
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(app.actions().is_none());
    assert_eq!(app.mode(), Mode::Preview);
}

#[test]
fn footer_never_splits_shortcuts_or_exceeds_cell_budget() {
    use unicode_width::UnicodeWidthStr;
    let text = "normal | next: Enter preview file | q quit | ? help | o actions | Esc/^G clear | ^C force | p paste | Space mark";
    let allowed = text.split('|').map(str::trim).collect::<Vec<_>>();
    for width in 12..=180 {
        for rows in [1, 2] {
            let lines = theme::footer_rows(Theme::Btop, text, width, rows);
            for line in lines {
                let string = line.to_string();
                assert!(string.width() <= width as usize);
                for segment in string.split('|').map(str::trim).filter(|s| !s.is_empty()) {
                    assert!(allowed.contains(&segment), "partial shortcut: {segment}");
                }
            }
        }
    }
}

#[test]
fn history_is_bounded_and_failures_create_gaps_instead_of_repeating_good_data() {
    let mut app = ClusterApp::new(vec![]);
    let alias = app.hosts()[0].alias().to_owned();
    for n in 0..70 {
        app.apply_snapshot(HostSnapshot::online(
            &alias,
            ProbeReport::parse(&format!("load={n}.0\nmemory=42% free\nstorage=90% used\n")),
            n,
        ));
    }
    let history = app.history_for(&alias).unwrap();
    assert_eq!(history.len(), metrics::HISTORY_LIMIT);
    assert_eq!(history.front().unwrap().values[0], Some(10.0));
    assert_eq!(history.back().unwrap().values[1], Some(58.0));
    let old_report = app.selected_snapshot().unwrap().report.clone();
    app.apply_snapshot(HostSnapshot::failed(&alias, "probe timed out"));
    assert_eq!(
        app.history_for(&alias).unwrap().back().unwrap().values,
        [None; 4]
    );
    assert_eq!(app.selected_snapshot().unwrap().report, old_report);
    assert_eq!(app.history_for(&alias).unwrap().len(), 60);
    app.apply_snapshot(HostSnapshot::online(
        "unknown-alias",
        ProbeReport::default(),
        0,
    ));
    assert!(app.history_for("unknown-alias").is_none());
}

#[test]
fn resource_parsing_rejects_invalid_percentages_and_graph_preserves_missing_samples() {
    for value in ["unknown", "-3%", "105%", "NaN%", "inf%"] {
        assert_eq!(metrics::percent(value), None, "{value}");
    }
    assert_eq!(metrics::memory_used("42% free"), Some(58));
    assert_eq!(metrics::memory_used("42% used"), Some(42));
    let values = [Some(0.0), None, Some(100.0)];
    assert_eq!(metrics::sparkline(&values, 100.0, false), "_ #");
    assert_eq!(metrics::sparkline(&values, 100.0, true), "▁ █");
}

#[test]
fn animation_is_bounded_and_does_not_redraw_idle_or_reduced_motion() {
    let mut app = ClusterApp::new(vec![]);
    assert!(!app.animate(Duration::from_secs(1), true));
    let alias = app.hosts()[0].alias().to_owned();
    app.begin_refresh(std::slice::from_ref(&alias));
    assert!(!app.animate(Duration::from_millis(249), true));
    assert!(app.animate(Duration::from_millis(250), true));
    assert!(!app.animate(Duration::from_millis(499), true));
    assert!(!app.animate(Duration::from_millis(500), false));
    app.apply(ClusterCommand::OpenHelp);
    assert!(!app.animate(Duration::from_millis(750), true));
    app.apply_completed_refresh_snapshot(HostSnapshot::online(alias, ProbeReport::default(), 0));
    assert!(!app.animate(Duration::from_secs(1), true));
}

#[test]
fn detail_scroll_reveals_logs_and_does_not_trigger_remote_commands() {
    let mut app = ClusterApp::new(vec![]);
    let alias = app.hosts()[0].alias().to_owned();
    for _ in 0..4 {
        app.apply_snapshot(HostSnapshot::online(
            &alias,
            ProbeReport::parse("hostname=example\nload=1\nmemory=10% used\n"),
            1,
        ));
    }
    app.apply(ClusterCommand::OpenDetail);
    let mut terminal = Terminal::new(TestBackend::new(60, 18)).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();
    for _ in 0..10 {
        assert_eq!(
            app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)),
            None
        );
    }
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();
    let text = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();
    assert!(text.contains("Log"));
    assert!(text.contains("Network route"));
    app.handle_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE));
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();
    let after_up = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();
    assert_ne!(
        text, after_up,
        "one PageUp must move immediately after reaching bottom"
    );
}
