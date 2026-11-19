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

fn render(app: &ClusterApp, width: u16, height: u16) -> Vec<String> {
    let mut terminal =
        ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| tersh::cluster_ui::draw(frame, app))
        .unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .chunks(width as usize)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect())
        .collect()
}

#[test]
fn filter_shows_live_matches_and_query_without_covering_host_rows() {
    let mut app = app();
    app.handle_key(key('/'));
    app.handle_key(key('g'));
    app.handle_key(key('p'));
    for width in [40, 110] {
        let rows = render(&app, width, 12);
        assert!(rows[0].contains("Filter hosts") && rows[0].contains("2/3"));
        assert!(rows[1].contains("gp"));
        assert!(rows.iter().any(|row| row.contains("zebra")));
        assert!(rows.iter().any(|row| row.contains("beta")));
        assert!(!rows.iter().any(|row| row.contains("alpha")));
    }
    for _ in 0..50 {
        app.handle_key(key('x'));
    }
    app.handle_key(key('z'));
    let rows = render(&app, 40, 12);
    assert!(rows[0].contains("0/3") && rows[1].contains("xz"));
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(app.selected_host().unwrap().alias(), "zebra");
    assert_eq!(app.visible_count(), 3);
}

#[test]
fn narrow_host_table_preserves_names_and_aligns_column_headings() {
    let mut app = app();
    app.apply_snapshot(HostSnapshot::online(
        "zebra",
        ProbeReport::parse("memory=42% used\nstorage=81% used"),
        17,
    ));
    for width in [28, 40, 65, 80, 120] {
        let rows = render(&app, width, 24);
        let header = rows.iter().find(|s| s.contains("alias")).unwrap();
        let host = rows
            .iter()
            .find(|s| s.contains("zebra") && s.contains('>'))
            .unwrap();
        assert_eq!(
            header.find("alias"),
            host.find("zebra"),
            "{width} cols: {host}"
        );
        if width >= 40 {
            assert_eq!(
                header.find("mem").unwrap() + 3,
                host.find("42%").unwrap() + 3
            );
            assert_eq!(
                header.find("disk").unwrap() + 4,
                host.find("81%").unwrap() + 3
            );
        }
    }
}

#[test]
fn overview_trends_follow_the_selected_host_when_the_list_has_room() {
    let mut app = app();
    for latency in [17, 21] {
        app.apply_snapshot(HostSnapshot::online(
            "zebra",
            ProbeReport::parse("memory=42% used\nstorage=81% used"),
            latency,
        ));
    }
    let wide = render(&app, 140, 30).join("\n");
    assert!(wide.contains("Trends | zebra") && wide.contains("2 samples"));
    app.apply(ClusterCommand::Down);
    let other = render(&app, 140, 30).join("\n");
    assert!(!other.contains("Trends |") && !other.contains("2 samples"));
    let short = render(&app, 140, 12).join("\n");
    assert!(short.contains("zebra") && short.contains("beta") && !short.contains("Trends |"));
}

#[test]
fn memory_percent_matches_across_list_node_meter_and_history() {
    let mut app = app();
    for native in ["590/1000 MB (59%)", "78% free"] {
        let system = if native.ends_with("% free") {
            "Darwin"
        } else {
            "Linux"
        };
        let report = ProbeReport::parse(&format!(
            "system={system}\nmemory={native}\nmemory_used_bytes=600000000\nmemory_total_bytes=1000000000"
        ));
        app.apply_snapshot(HostSnapshot::online("zebra", report, 8));
        let rows = render(&app, 182, 60);
        assert!(
            rows.iter()
                .any(|row| row.contains("> ok") && row.contains("zebra") && row.contains("60%"))
        );
        assert!(
            rows.iter()
                .any(|row| row.contains("Memory: [") && row.contains("60% used"))
        );
        let summary = if native.ends_with("% free") {
            "Legacy pressure index: 78%"
        } else {
            "Native memory summary: 590/1000 MB (59%)"
        };
        assert!(rows.iter().any(|row| row.contains(summary)));
        app.apply(ClusterCommand::OpenTopology);
        let rows = render(&app, 182, 60);
        assert!(rows.iter().any(|row| row.contains("RAM 60%")));
        assert!(
            rows.iter()
                .any(|row| row.contains("RAM ") && row.contains('[') && row.contains("60%"))
        );
        assert_eq!(
            app.history_for("zebra").unwrap().back().unwrap().values[1],
            Some(60.0)
        );
        app.apply(ClusterCommand::Cancel);
    }
    let older = ProbeReport::parse("memory=42% free");
    assert_eq!(older.memory_used_percent(), Some(58));
    let mac = ProbeReport::parse("system=Darwin 25 arm64\nmemory=90% free");
    assert_eq!(mac.memory_used_percent(), None);
    assert_eq!(mac.legacy_darwin_pressure_index(), Some(90));
}

#[test]
fn nine_machine_inventory_keeps_overview_and_expanded_map_distinct() {
    let inventory = ClusterInventory::from_json(r#"{
        "main_machine":{"alias":"local-box"},"jump_host":{"alias":"gateway"},
        "servers":[{"alias":"hpc-alpha","proxy_jump":"gateway"},{"alias":"hpc-beta","proxy_jump":"gateway"},
        {"alias":"hpc-gamma","proxy_jump":"gateway"},{"alias":"studio","proxy_jump":"gateway"},
        {"alias":"nas-box"},{"alias":"edge-box"},{"alias":"linux-box"}]
    }"#).unwrap();
    let mut app = ClusterApp::from_inventory(inventory);
    let mut aliases = app
        .hosts()
        .iter()
        .map(|h| h.alias().to_owned())
        .collect::<Vec<_>>();
    aliases.push("hpc-alpha".into());
    for alias in aliases {
        app.apply_snapshot(HostSnapshot::online(
            alias,
            ProbeReport::parse("cpu_usage=60\nmemory=50% used"),
            6,
        ));
    }
    app.apply(ClusterCommand::Down);
    app.apply(ClusterCommand::Down);
    let tall = render(&app, 182, 60);
    let topology = tall.iter().position(|r| r.contains("Topology")).unwrap();
    let trend = tall
        .iter()
        .position(|r| r.contains("Trends | hpc-alpha"))
        .unwrap();
    assert!(
        tall[..topology].iter().any(|r| r.contains("linux-box")),
        "table must show all nine"
    );
    assert!(tall[topology..trend].iter().any(|r| r.contains("gateway")));
    assert!(
        !tall[topology..trend]
            .iter()
            .any(|r| r.contains(" RAM 50% ")),
        "overview route sketch stays compact"
    );
    assert!(
        60 - 3 - trend >= 12,
        "tall overview has room for multi-row trend plots"
    );

    app.apply(ClusterCommand::OpenTopology);
    for (w, h) in [(182, 60), (144, 36), (120, 30)] {
        let rows = render(&app, w, h);
        let meters = rows
            .iter()
            .position(|r| r.contains("hpc-alpha | usage"))
            .unwrap();
        assert!(
            rows[meters].contains("Trends | hpc-alpha"),
            "HPC panel shares its row with trends"
        );
        assert!(rows[meters..].iter().any(|r| r.contains("GPU peak")));
        assert_eq!(
            rows[..meters]
                .iter()
                .map(|r| r.matches(" RAM 50% ").count())
                .sum::<usize>(),
            9,
            "expanded map needs all device cards at {w}×{h}"
        );
        if h < 40 {
            let row = rows.iter().find(|r| r.contains("nas-box")).unwrap();
            assert!(
                row.contains("hpc-beta"),
                "direct and routed hosts can share rows in separate columns"
            );
        } else {
            assert!(h as usize - 3 - meters >= 12);
        }
    }
    app.apply(ClusterCommand::Down);
    app.apply(ClusterCommand::Down);
    app.apply(ClusterCommand::Down);
    assert_eq!(app.selected_host().unwrap().alias(), "studio");
    let studio = render(&app, 144, 36);
    assert!(studio.iter().any(|r| r.contains("studio | usage")));
    assert!(
        studio.iter().any(|r| r.contains("Trends | studio")),
        "new selections preserve the user's chosen layout before a second observation"
    );
    let phone = render(&app, 40, 18);
    assert!(phone.iter().any(|r| r.contains("> studio")));
    assert!(phone.iter().any(|r| r.contains("gateway")));
}

#[test]
fn numbered_layouts_work_before_a_draw_and_while_waiting_for_samples() {
    let mut app = app();
    let selected = app.selected_host().unwrap().alias().to_owned();
    for (ch, expected) in [('3', false), ('3', false), ('2', true), ('2', true)] {
        assert_eq!(app.handle_key(key(ch)), None);
        assert_eq!(app.key_context(), "cluster_topology");
        assert_eq!(app.topology_shows_trends(), expected);
        assert_eq!(app.selected_host().unwrap().alias(), selected);
    }
    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert!(!app.topology_shows_trends());
    assert!(!render(&app, 144, 36).join("\n").contains("Trends |"));
    app.handle_key(KeyEvent::from(KeyCode::Enter));
    assert_eq!(app.key_context(), "cluster_detail");
    app.handle_key(KeyEvent::from(KeyCode::Esc));
    assert!(!app.topology_shows_trends());
    app.handle_key(key('1'));
    assert_eq!(app.key_context(), "cluster");
    app.handle_key(key('m'));
    assert!(app.topology_shows_trends());
    app.handle_key(KeyEvent::from(KeyCode::Tab));
    render(&app, 40, 18);
    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert!(app.topology_shows_trends());
    app.handle_key(key('H'));
    assert_eq!(app.key_context(), "cluster_detail");
    app.handle_key(key('3'));
    assert_eq!(app.key_context(), "cluster_topology");
    assert!(!app.topology_shows_trends());
    app.handle_key(key('m'));
    assert_eq!(app.key_context(), "cluster");
}

#[test]
fn layout_remaps_replace_defaults_and_match_the_action_picker() {
    let mut app = app();
    app.set_keymap(tersh::keymap::Keymap::from_json(
        r#"{"cluster":{"show_topology_meters":["F7"]},"cluster_topology":{"toggle_topology_trends":["F6"]}}"#,
    ).unwrap());
    assert_eq!(app.handle_key(key('3')), None);
    assert_eq!(app.key_context(), "cluster");
    app.handle_key(key('o'));
    let menus = app.actions().unwrap().matches();
    assert!(
        menus
            .iter()
            .any(|a| a.command.action_id() == "show_topology_meters" && a.key == "F7")
    );
    app.handle_key(KeyEvent::from(KeyCode::Esc));
    app.handle_key(KeyEvent::from(KeyCode::F(7)));
    assert_eq!(app.key_context(), "cluster_topology");
    assert!(!app.topology_shows_trends());
    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert!(!app.topology_shows_trends());
    app.handle_key(KeyEvent::from(KeyCode::F(6)));
    assert!(app.topology_shows_trends());
}

#[test]
fn filter_selects_a_device_by_exact_alias_and_arrows_choose_among_matches() {
    let inventory = ClusterInventory::from_json(
        r#"{"jump_host":{"alias":"gateway"},
        "servers":[{"alias":"studio3-archive"},{"alias":"studio3","proxy_jump":"gateway"}]}"#,
    )
    .unwrap();
    let mut app = ClusterApp::from_inventory(inventory);
    app.handle_key(key('2'));
    app.handle_key(key('/'));
    for ch in "studio3".chars() {
        assert_eq!(app.handle_key(key(ch)), None);
    }
    assert_eq!(app.key_context(), "cluster_filter");
    assert_eq!(app.selected_host().unwrap().alias(), "studio3");
    app.handle_key(KeyEvent::from(KeyCode::Up));
    assert_eq!(app.selected_host().unwrap().alias(), "studio3-archive");
    assert_eq!(app.handle_key(KeyEvent::from(KeyCode::Enter)), None);
    assert_eq!(app.selected_host().unwrap().alias(), "studio3-archive");
    app.handle_key(key('/'));
    app.handle_key(KeyEvent::from(KeyCode::Down));
    assert_eq!(app.handle_key(KeyEvent::from(KeyCode::Enter)), None);
    assert_eq!(app.key_context(), "cluster_topology");
    assert_eq!(app.visible_count(), 2);
    assert_eq!(app.selected_host().unwrap().alias(), "studio3");
    app.handle_key(KeyEvent::from(KeyCode::Backspace));
    assert_eq!(app.visible_count(), 3);
    assert_eq!(app.selected_host().unwrap().alias(), "studio3");
}

#[test]
fn topology_navigation_follows_routes_and_restores_selected_device() {
    let inventory = ClusterInventory::from_json(
        r#"{
        "main_machine":{"alias":"local"},
        "jump_host":{"alias":"gateway"},
        "servers":[{"alias":"nas"},{"alias":"hpc","proxy_jump":"gateway"},
                   {"alias":"studio","proxy_jump":"gateway"}]
    }"#,
    )
    .unwrap();
    let mut app = ClusterApp::from_inventory(inventory);
    let press = |app: &mut ClusterApp, code| {
        assert!(app.handle_key(KeyEvent::from(code)).is_none());
    };
    press(&mut app, KeyCode::Char('m'));
    assert_eq!(app.key_context(), "cluster_topology");
    for (code, alias) in [
        (KeyCode::Right, "gateway"),
        (KeyCode::Down, "hpc"),
        (KeyCode::Down, "studio"),
        (KeyCode::Left, "gateway"),
        (KeyCode::Right, "hpc"),
        (KeyCode::End, "nas"),
        (KeyCode::Home, "local"),
        (KeyCode::Down, "gateway"),
    ] {
        press(&mut app, code);
        assert_eq!(app.selected_host().unwrap().alias(), alias);
    }
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.key_context(), "cluster_detail");
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.key_context(), "cluster_topology");
    press(&mut app, KeyCode::Char('?'));
    assert_eq!(app.help_context(), "cluster_topology");
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.key_context(), "cluster_topology");
    assert_eq!(app.handle_key(key('r')), Some(ClusterCommand::RefreshAll));
    assert_eq!(
        app.handle_key(key('R')),
        Some(ClusterCommand::RefreshSelected)
    );
    assert_eq!(
        app.handle_key(key('t')),
        Some(ClusterCommand::OpenWorkbench)
    );
    press(&mut app, KeyCode::Char('q'));
    assert_eq!(app.key_context(), "cluster");
    assert!(!app.should_quit());
    assert_eq!(app.selected_host().unwrap().alias(), "gateway");
}

#[test]
fn filtered_topology_never_selects_a_hidden_jump_or_an_implicit_origin() {
    let inventory = ClusterInventory::from_json(
        r#"{
        "jump_host":{"alias":"bridge"},
        "servers":[{"alias":"hpc","proxy_jump":"bridge"},{"alias":"studio"}]
    }"#,
    )
    .unwrap();
    let mut app = ClusterApp::from_inventory(inventory);
    app.set_keymap(
        tersh::keymap::Keymap::from_json(
            r#"{"cluster":{"open_topology":["F3"]},"cluster_topology":{"topology_child":["F4"]}}"#,
        )
        .unwrap(),
    );
    assert_eq!(app.handle_key(key('m')), None);
    assert_eq!(app.key_context(), "cluster");
    app.handle_key(KeyEvent::from(KeyCode::F(3)));
    app.handle_key(KeyEvent::from(KeyCode::Left));
    assert_eq!(app.selected_host().unwrap().alias(), "bridge");
    app.handle_key(KeyEvent::from(KeyCode::F(4)));
    assert_eq!(app.selected_host().unwrap().alias(), "hpc");
    app.handle_key(key('/'));
    for ch in "hpc".chars() {
        app.handle_key(key(ch));
    }
    app.handle_key(KeyEvent::from(KeyCode::Enter));
    assert_eq!(app.key_context(), "cluster_topology");
    assert_eq!(app.visible_count(), 1);
    app.handle_key(key('o'));
    let actions = app.actions().unwrap().matches();
    assert!(
        actions
            .iter()
            .any(|action| action.command == ClusterCommand::TopologyChild
                && action.key == "F4"
                && action.disabled_reason.is_some())
    );
    assert!(
        actions
            .iter()
            .any(|action| action.command == ClusterCommand::TopologyParent
                && action.disabled_reason.is_some())
    );
    app.handle_key(KeyEvent::from(KeyCode::Esc));
    app.handle_key(KeyEvent::from(KeyCode::Left));
    assert_eq!(app.selected_host().unwrap().alias(), "hpc");
    app.handle_key(key('/'));
    app.handle_key(key('z'));
    app.handle_key(KeyEvent::from(KeyCode::Enter));
    assert_eq!(app.selected_host(), None);
    for code in [KeyCode::Enter, KeyCode::Right, KeyCode::Char('t')] {
        assert_eq!(app.handle_key(KeyEvent::from(code)), None);
        assert_eq!(app.key_context(), "cluster_topology");
    }
    app.handle_key(key('m'));
    assert_eq!(app.key_context(), "cluster");
}
