use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tersh::cluster::{ClusterApp, ClusterCommand, ClusterInventory, HostSnapshot, ProbeReport};

fn key(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)
}
fn app() -> ClusterApp {
    let inventory = ClusterInventory::from_json(
        r#"{"servers":[
        {"alias":"zebra","ssh_user":"ops","campus_ip":"zebra.invalid","role":"GPU"},
        {"alias":"alpha","ssh_user":"ops","campus_ip":"alpha.invalid","role":"Storage"},
        {"alias":"beta","ssh_user":"ops","campus_ip":"beta.invalid","role":"GPU"}
    ]}"#,
    )
    .unwrap();
    ClusterApp::new(inventory.hosts().to_vec())
}

#[test]
fn host_filter_changes_selection_but_not_inventory_or_refresh_scope() {
    let mut app = app();
    app.handle_key(key('/'));
    for ch in "STORAGE".chars() {
        app.handle_key(key(ch));
    }
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(app.selected_host().unwrap().alias(), "alpha");
    assert_eq!(app.hosts().len(), 3);
    let aliases = app
        .hosts()
        .iter()
        .map(|host| host.alias().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(app.begin_refresh(&aliases).len(), 3);
}

#[test]
fn host_filter_escape_restores_query_and_empty_results_are_safe() {
    let mut app = app();
    app.handle_key(key('/'));
    for ch in "no-match".chars() {
        app.handle_key(key(ch));
    }
    assert!(app.selected_host().is_none());
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(app.selected_host().unwrap().alias(), "zebra");
    assert!(!app.should_quit());
}

#[test]
fn sorting_preserves_selected_alias_and_changes_navigation_order() {
    let mut app = app();
    app.handle_key(key('v'));
    assert_eq!(app.selected_host().unwrap().alias(), "zebra");
    app.apply(ClusterCommand::First);
    assert_eq!(app.selected_host().unwrap().alias(), "alpha");
    app.handle_key(key('V'));
    assert_eq!(app.selected_host().unwrap().alias(), "alpha");
    app.apply(ClusterCommand::First);
    assert_eq!(app.selected_host().unwrap().alias(), "zebra");
}

#[test]
fn metric_sort_keeps_unknown_last_in_both_directions() {
    let mut app = app();
    app.apply_snapshot(HostSnapshot::online(
        "zebra",
        ProbeReport::parse("load=9"),
        9,
    ));
    app.apply_snapshot(HostSnapshot::online(
        "alpha",
        ProbeReport::parse("load=2"),
        2,
    ));
    for _ in 0..3 {
        app.handle_key(key('v'));
    }
    app.apply(ClusterCommand::First);
    assert_eq!(app.selected_host().unwrap().alias(), "alpha");
    app.apply(ClusterCommand::Last);
    assert_eq!(app.selected_host().unwrap().alias(), "beta");
    app.handle_key(key('V'));
    app.apply(ClusterCommand::First);
    assert_eq!(app.selected_host().unwrap().alias(), "zebra");
    app.apply(ClusterCommand::Last);
    assert_eq!(app.selected_host().unwrap().alias(), "beta");
}

#[test]
fn empty_filter_results_do_not_dispatch_host_launches() {
    let mut app = app();
    app.handle_key(key('/'));
    for ch in "no-match".chars() {
        app.handle_key(key(ch));
    }
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(app.handle_key(key('s')), None);
    assert_eq!(app.handle_key(key('t')), None);
    assert_eq!(
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        None
    );
    app.handle_key(key('o'));
    assert!(
        !app.actions()
            .unwrap()
            .matches()
            .iter()
            .any(|action| action.command == ClusterCommand::OpenSession)
    );
}
