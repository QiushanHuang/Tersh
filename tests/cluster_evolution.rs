use ratatui::{Terminal, backend::TestBackend};
use tersh::{
    cluster::{
        ClusterApp, ClusterCommand, ClusterInventory, ConnectionState, HostSnapshot, ProbeReport,
    },
    cluster_ui,
};

fn app() -> ClusterApp {
    let inventory = ClusterInventory::from_json(r#"{"servers":[
        {"alias":"long-worker-name-that-should-not-hide-metrics","campus_ip":"worker.invalid","role":"GPU训练"},
        {"alias":"other","campus_ip":"other.invalid","role":"Storage"}
    ]}"#).unwrap();
    ClusterApp::from_inventory(inventory)
}
fn render(app: &ClusterApp, width: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, 30)).unwrap();
    terminal.draw(|frame| cluster_ui::draw(frame, app)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .chunks(width as usize)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn checking_preserves_last_good_metrics() {
    let mut app = app();
    let alias = app.hosts()[0].alias().to_owned();
    let report = ProbeReport::parse("load=0.2\nmemory=42% used\nstorage=81% used");
    app.apply_snapshot(HostSnapshot::online(&alias, report.clone(), 12));
    app.begin_refresh(std::slice::from_ref(&alias));
    let snapshot = app.snapshot_for(&alias).unwrap();
    assert_eq!(snapshot.connection, ConnectionState::Checking);
    assert_eq!(
        snapshot.report, report,
        "checking must not blank last valid data"
    );
}

#[test]
fn wide_rows_keep_metrics_and_show_configured_role() {
    let mut app = app();
    let alias = app.hosts()[0].alias().to_owned();
    app.apply_snapshot(HostSnapshot::online(
        &alias,
        ProbeReport::parse("memory=42% used\nstorage=81% used"),
        12,
    ));
    // At 90 columns the host panel spans the full terminal width.
    let text = render(&app, 90);
    let row = text
        .lines()
        .find(|line| line.contains("long-worker"))
        .unwrap();
    assert!(row.contains("GPU"), "configured role missing: {row}");
    assert!(
        row.contains("42%") && row.contains("81%"),
        "metrics clipped: {row}"
    );
}

#[test]
fn narrow_footer_exposes_workbench_and_actions() {
    let text = render(&app(), 40);
    let footer = text.lines().rev().take(3).collect::<Vec<_>>().join("\n");
    assert!(
        footer.contains("t tersh"),
        "primary launch missing: {footer}"
    );
    assert!(footer.contains("o actions"), "discovery missing: {footer}");
    assert!(footer.contains("q"), "exit missing: {footer}");
}

#[test]
fn attention_events_and_pause_are_discoverable_actions() {
    for id in ["toggle_attention", "open_events", "toggle_pause"] {
        assert!(ClusterCommand::from_action(id).is_some(), "missing {id}");
    }
}

#[test]
fn timeout_is_not_counted_as_offline() {
    let mut app = app();
    app.apply_snapshot(HostSnapshot::failed("other", "probe timed out after 30s"));
    assert_eq!(
        app.offline_count(),
        0,
        "timeout does not establish host is offline"
    );
}

#[test]
fn attention_uses_existing_data_and_keeps_failed_host_visible_while_checking() {
    let mut app = app();
    let alias = app.hosts()[0].alias().to_owned();
    app.apply_snapshot(HostSnapshot::online(
        &alias,
        ProbeReport::parse("storage=81% used"),
        12,
    ));
    app.apply(ClusterCommand::from_action("toggle_attention").unwrap());
    assert_eq!(app.visible_count(), 1);
    assert_eq!(app.selected_host().unwrap().alias(), "other");
    app.begin_refresh(&["other".into()]);
    assert_eq!(
        app.visible_count(),
        1,
        "first-time checking is still unknown"
    );
    app.set_disk_threshold(80).unwrap();
    assert_eq!(app.visible_count(), 2);
    assert_eq!(app.hosts().len(), 2);
    assert_eq!(
        app.history_for(&alias).unwrap().len(),
        1,
        "view must not collect data"
    );
}

#[test]
fn events_record_changes_not_each_success_and_remain_bounded() {
    let mut app = app();
    for _ in 0..3 {
        app.apply_snapshot(HostSnapshot::online(
            "other",
            ProbeReport::parse("load=1"),
            1,
        ));
    }
    assert_eq!(app.events().len(), 1);
    for _ in 0..60 {
        app.apply_snapshot(HostSnapshot::failed("other", "connection refused"));
        app.apply_snapshot(HostSnapshot::online(
            "other",
            ProbeReport::parse("load=1"),
            1,
        ));
    }
    assert_eq!(app.events().len(), 100);
    app.apply(ClusterCommand::from_action("open_events").unwrap());
    assert!(render(&app, 80).contains("Events"));
    app.apply(ClusterCommand::Cancel);
    assert_eq!(app.key_context(), "cluster");
}

#[test]
fn disk_threshold_rejects_out_of_range_and_non_integer_values() {
    use tersh::cluster::parse_disk_threshold;
    assert_eq!(parse_disk_threshold(None).unwrap(), 90);
    for value in ["0", "100", "85"] {
        assert!(parse_disk_threshold(Some(value)).is_ok());
    }
    for value in ["101", "-1", "NaN", "90.5", ""] {
        assert!(parse_disk_threshold(Some(value)).is_err());
    }
}

#[test]
fn last_good_age_does_not_reset_when_checking_or_failing() {
    use std::time::{Duration, SystemTime};
    let mut app = app();
    let mut good = HostSnapshot::online("other", ProbeReport::parse("load=1"), 1);
    good.refreshed_at = Some(SystemTime::now() - Duration::from_secs(100));
    app.apply_snapshot(good);
    app.begin_refresh(&["other".into()]);
    assert!(app.data_age("other").unwrap().as_secs() >= 100);
    app.apply_completed_refresh_snapshot(HostSnapshot::failed("other", "probe timed out"));
    assert!(app.data_age("other").unwrap().as_secs() >= 100);
}

#[test]
fn remote_workbench_forwards_only_valid_presentation_values() {
    const MARKER: &str = "TERSH_EVOLUTION_ENV_CHILD";
    if std::env::var_os(MARKER).is_none() {
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "remote_workbench_forwards_only_valid_presentation_values",
                "--nocapture",
            ])
            .env(MARKER, "1")
            .env("TERSH_THEME", "aurora")
            .env("TERSH_MOTION", "off")
            .env("TERSH_GLYPHS", "unicode")
            .env("TERSH_FOOTER", "$(touch /tmp/forbidden)")
            .env("TERSH_KEYMAP", "secret-keymap-path")
            .env("TERSH_SECRET", "secret-token")
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        return;
    }
    let app = app();
    let args = tersh::cluster::ssh_workbench_args(&app.hosts()[0]);
    let script = args.last().unwrap();
    assert!(
        script.contains("TERSH_THEME") && script.contains("aurora"),
        "{script}"
    );
    assert!(script.contains("TERSH_MOTION") && script.contains("off"));
    assert!(
        !script.contains("forbidden")
            && !script.contains("secret-")
            && !script.contains("TERSH_KEYMAP")
    );
}

#[test]
fn places_action_opens_navigation_without_launching_ssh() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut app = app();
    assert_eq!(
        app.handle_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE)),
        None
    );
    assert_eq!(app.key_context(), "places");
    let text = render(&app, 80);
    assert!(text.contains("Places"));
    assert!(
        text.contains("workdir"),
        "empty places should explain how to get started: {text}"
    );
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(app.key_context(), "cluster");
}

#[test]
fn saved_place_cannot_launch_when_alias_now_points_to_another_identity() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut app = app();
    let mut places = tersh::places::Places::memory();
    places
        .visit(
            "other",
            "old-user@old-host||server",
            std::path::Path::new("/work"),
        )
        .unwrap();
    app.set_places(places);
    app.handle_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
    assert!(!app.place_choices()[0].valid);
    assert_eq!(
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        None
    );
    assert!(app.pending_workbench().is_none());
    assert!(app.place_message().unwrap().contains("identity"));
}

#[test]
fn searching_saved_places_routes_to_exact_host_and_directory() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut app = app();
    let (scope, identity) = app.hosts()[1].place_scope();
    let mut places = tersh::places::Places::memory();
    places
        .visit(&scope, &identity, std::path::Path::new("/work/logs"))
        .unwrap();
    app.set_places(places);
    app.handle_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
    for ch in "other /work".chars() {
        app.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    assert_eq!(app.place_choices().len(), 1);
    assert_eq!(
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        Some(ClusterCommand::OpenWorkbench)
    );
    let launch = app.pending_workbench().unwrap();
    assert_eq!(launch.alias(), "other");
    assert_eq!(launch.workdir(), Some("/work/logs"));
    assert_eq!(launch.place_scope(), (scope, identity));
}

#[test]
fn configured_place_can_be_pinned_without_changing_inventory() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let inventory = ClusterInventory::from_json(
        r#"{"servers":[{"alias":"host","campus_ip":"host.invalid","workdir":"/srv/project"}]}"#,
    )
    .unwrap();
    let mut app = ClusterApp::from_inventory(inventory);
    app.handle_key(KeyEvent::new(KeyCode::Char('B'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
    assert!(app.place_choices()[0].pinned);
    assert!(app.place_choices()[0].configured);
    assert_eq!(app.hosts()[0].workdir(), Some("/srv/project"));
}

#[test]
fn unicode_profile_preserves_host_text_and_column_positions() {
    const MARKER: &str = "TERSH_EVOLUTION_UNICODE_CHILD";
    if std::env::var_os(MARKER).is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "unicode_profile_preserves_host_text_and_column_positions",
                "--nocapture",
            ])
            .env(MARKER, "1")
            .env("TERSH_GLYPHS", "unicode")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let mut app = app();
    let alias = app.hosts()[0].alias().to_owned();
    app.apply_snapshot(HostSnapshot::online(
        &alias,
        ProbeReport::parse("memory=42% used\nstorage=81% used"),
        12,
    ));
    let text = render(&app, 90);
    let row = text.lines().find(|line| line.contains("GPU")).unwrap();
    assert!(
        row.contains('训') && row.contains('练') && row.contains("42%") && row.contains("81%"),
        "{row}"
    );
}

#[test]
fn remote_scope_is_derived_from_inventory_not_inherited_environment() {
    let app = app();
    let host = &app.hosts()[1];
    let command = tersh::cluster::ssh_workbench_args(host);
    let script = command.last().unwrap();
    assert!(
        script.contains("TERSH_HOST_ALIAS"),
        "missing explicit host scope: {script}"
    );
    assert!(script.contains("TERSH_HOST_ID"));
    assert!(script.contains("other.invalid||server"));
}

#[test]
fn stale_trends_keep_last_valid_value_labeled_old() {
    let mut app = app();
    app.apply_snapshot(HostSnapshot::online(
        "other",
        ProbeReport::parse("load=1\nmemory=42% used\nstorage=81% used"),
        12,
    ));
    app.apply_snapshot(HostSnapshot::failed("other", "probe timed out"));
    app.apply(ClusterCommand::Down);
    app.apply(ClusterCommand::OpenDetail);
    let text = render(&app, 80);
    let memory = text.lines().find(|line| line.contains("Mem used")).unwrap();
    assert!(memory.contains("42%") && memory.contains("old"), "{memory}");
}

#[test]
fn event_scroll_can_reach_older_events_after_wrapped_diagnostics() {
    let mut app = app();
    app.apply_snapshot(HostSnapshot::failed("other", "oldest-unique"));
    for _ in 0..4 {
        app.apply_snapshot(HostSnapshot::online(
            "other",
            ProbeReport::parse("load=1"),
            12,
        ));
        app.apply_snapshot(HostSnapshot::failed(
            "other",
            format!("connection refused {}", "diagnostic ".repeat(40)),
        ));
    }
    app.apply(ClusterCommand::OpenEvents);
    render(&app, 40);
    app.apply(ClusterCommand::Last);
    let text = render(&app, 40);
    assert!(text.contains("oldest-unique"), "{text}");
}

#[test]
fn saved_place_remote_command_preserves_trailing_space_and_quotes() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut app = app();
    let (scope, identity) = app.hosts()[1].place_scope();
    let mut places = tersh::places::Places::memory();
    let exact_path = "/work/project's result ";
    places
        .visit(&scope, &identity, std::path::Path::new(exact_path))
        .unwrap();
    app.set_places(places);
    app.handle_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
    assert_eq!(
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        Some(ClusterCommand::OpenWorkbench)
    );
    let host = app.pending_workbench().unwrap();
    assert_eq!(host.workdir(), Some(exact_path));
    let args = tersh::cluster::ssh_workbench_args(host);
    let script = args.last().unwrap();
    // The remote script is itself one shell-quoted argument. Verify both
    // quoting layers preserve the complete path, including the final space.
    fn quote(value: &str) -> String {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
    let cd = format!("; cd -- {} ||", quote(exact_path));
    let quoted_cd = quote(&cd);
    assert!(
        script.contains(&quoted_cd[1..quoted_cd.len() - 1]),
        "{script}"
    );
}

#[test]
fn inventory_preserves_exact_workdir_for_all_host_kinds() {
    let inventory = ClusterInventory::from_json(
        r#"{
      "main_machine":{"alias":"local-box","workdir":" /local's work "},
      "jump_host":{"alias":"jump","device_name":"jump.invalid","workdir":"/jump's work "},
      "servers":[{"alias":"worker","campus_ip":"worker.invalid","workdir":"/worker's work "}]
    }"#,
    )
    .unwrap();
    assert_eq!(inventory.hosts()[0].workdir(), Some(" /local's work "));
    assert_eq!(inventory.hosts()[1].workdir(), Some("/jump's work "));
    assert_eq!(inventory.hosts()[2].workdir(), Some("/worker's work "));
}

#[test]
fn clear_recent_targets_highlighted_place_host_not_background_selection() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut app = app();
    let mut places = tersh::places::Places::memory();
    for host in app.hosts() {
        let (scope, identity) = host.place_scope();
        places
            .visit(&scope, &identity, std::path::Path::new("/work"))
            .unwrap();
    }
    let background = app.selected_host().unwrap().alias().to_owned();
    app.set_places(places);
    app.apply(ClusterCommand::OpenPlaces);
    for ch in "other".chars() {
        app.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    assert_eq!(app.place_choices()[0].host, "other");
    app.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL));
    assert!(app.place_message().unwrap().contains("other"));
    app.apply(ClusterCommand::OpenPlaces);
    assert_eq!(app.place_choices().len(), 1);
    assert_eq!(app.place_choices()[0].host, background);

    for ch in "no-match".chars() {
        app.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    assert!(app.place_choices().is_empty());
    app.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL));
    app.apply(ClusterCommand::OpenPlaces);
    assert_eq!(
        app.place_choices().len(),
        1,
        "empty query result must not clear another host"
    );
}
