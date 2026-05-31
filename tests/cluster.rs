use ratatui::{Terminal, backend::TestBackend};
use tersh::{
    cluster::{
        ClusterApp, ClusterCommand, ClusterInventory, ConnectionState, HostKind, HostSnapshot,
        ProbeReport, host_session_command, host_workbench_command, ssh_probe_args,
        ssh_session_args,
    },
    cluster_ui,
};

const CAMPUS_JSON: &str = r#"
{
  "main_machine": {
    "alias": "qiushan-mbp",
    "tailscale_ip": "100.116.123.45",
    "role": "Codex runs here"
  },
  "jump_host": {
    "alias": "campus-mac",
    "device_name": "joshuamacbook-pro",
    "tailscale_ip": "100.90.116.54",
    "ssh_user": "joshua",
    "role": "Campus network jump host"
  },
  "servers": [
    {
      "alias": "school-star",
      "ssh_user": "star",
      "campus_ip": "10.13.7.138",
      "proxy_jump": "campus-mac",
      "workdir": "/srv/star"
    },
    {
      "alias": "school-hkust",
      "ssh_user": "hkust",
      "campus_ip": "10.4.9.241",
      "proxy_jump": "campus-mac"
    }
  ]
}
"#;

const DIRECT_JSON: &str = r#"
{
  "servers": [
    {
      "alias": "direct-box",
      "ssh_user": "ops",
      "campus_ip": "203.0.113.10",
      "role": "Direct remote server"
    }
  ]
}
"#;

#[test]
fn inventory_parses_campus_access_json_into_monitorable_hosts() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();

    let hosts = inventory.hosts();
    assert_eq!(hosts.len(), 4);
    assert_eq!(hosts[0].alias(), "qiushan-mbp");
    assert_eq!(hosts[0].kind(), HostKind::Local);
    assert_eq!(hosts[1].alias(), "campus-mac");
    assert_eq!(hosts[1].kind(), HostKind::Jump);
    assert_eq!(hosts[1].address(), "100.90.116.54");
    assert_eq!(hosts[2].alias(), "school-star");
    assert_eq!(hosts[2].kind(), HostKind::Server);
    assert_eq!(hosts[2].proxy_jump(), Some("campus-mac"));
    assert_eq!(hosts[2].proxy_jump_target(), Some("joshua@100.90.116.54"));
    assert_eq!(hosts[2].ssh_target(), "star@10.13.7.138");
    assert_eq!(hosts[2].workdir(), Some("/srv/star"));
    assert_eq!(hosts[3].alias(), "school-hkust");
}

#[test]
fn probe_report_parses_resource_and_task_fields() {
    let report = ProbeReport::parse(
        "hostname=starbox\nsystem=Linux x86_64\nuptime=up 4 days\nload=0.12 0.20 0.30\nmemory=512/1024 MB (50%)\nstorage=8G/20G 40% used\ntasks=72 processes\ngpu=A100, 11 %, 2048 MiB / 40960 MiB\n",
    );

    assert_eq!(report.hostname.as_deref(), Some("starbox"));
    assert_eq!(report.system.as_deref(), Some("Linux x86_64"));
    assert_eq!(report.uptime.as_deref(), Some("up 4 days"));
    assert_eq!(report.cpu_load.as_deref(), Some("0.12 0.20 0.30"));
    assert_eq!(report.memory.as_deref(), Some("512/1024 MB (50%)"));
    assert_eq!(report.storage.as_deref(), Some("8G/20G 40% used"));
    assert_eq!(report.tasks.as_deref(), Some("72 processes"));
    assert!(report.gpu.as_deref().unwrap().contains("A100"));
}

#[test]
fn failed_snapshots_classify_timeout_and_authentication_errors() {
    let timeout = HostSnapshot::failed("school-star", "probe timed out after 6s");
    let auth = HostSnapshot::failed("school-star", "Permission denied (publickey)");

    assert_eq!(timeout.connection, ConnectionState::Timeout);
    assert_eq!(auth.connection, ConnectionState::AuthFailed);
}

#[test]
fn ssh_probe_args_are_non_interactive_and_use_configured_proxy_jump() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let host = inventory
        .hosts()
        .iter()
        .find(|host| host.alias() == "school-star")
        .unwrap();

    let args = ssh_probe_args(host);

    assert!(args.contains(&"-n".to_string()));
    assert!(
        args.windows(2)
            .any(|pair| pair == ["-J", "joshua@100.90.116.54"])
    );
    assert!(args.windows(2).any(|pair| pair == ["-o", "BatchMode=yes"]));
    assert!(
        args.windows(2)
            .any(|pair| pair == ["-o", "ConnectTimeout=3"])
    );
    assert!(
        args.windows(2)
            .any(|pair| pair == ["-o", "StrictHostKeyChecking=accept-new"])
    );
    assert!(args.iter().any(|arg| arg == "star@10.13.7.138"));
}

#[test]
fn ssh_session_args_are_interactive_and_use_configured_proxy_jump() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let host = inventory
        .hosts()
        .iter()
        .find(|host| host.alias() == "school-star")
        .unwrap();

    let args = ssh_session_args(host);

    assert!(
        args.windows(2)
            .any(|pair| pair == ["-J", "joshua@100.90.116.54"])
    );
    assert_eq!(args.last().map(String::as_str), Some("star@10.13.7.138"));
    assert!(!args.contains(&"-n".to_string()));
    assert!(!args.windows(2).any(|pair| pair == ["-o", "BatchMode=yes"]));
    assert!(!args.iter().any(|arg| arg.contains("sh -lc")));
}

#[test]
fn host_session_command_uses_local_shell_or_interactive_ssh() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let local = inventory
        .hosts()
        .iter()
        .find(|host| host.alias() == "qiushan-mbp")
        .unwrap();
    let server = inventory
        .hosts()
        .iter()
        .find(|host| host.alias() == "school-star")
        .unwrap();

    let local_command = host_session_command(local, Some("/bin/zsh"));
    assert_eq!(local_command.program(), "/bin/zsh");
    assert!(local_command.args().is_empty());

    let server_command = host_session_command(server, Some("/bin/zsh"));
    assert_eq!(server_command.program(), "ssh");
    assert_eq!(server_command.args(), ssh_session_args(server).as_slice());
}

#[test]
fn host_workbench_command_opens_local_or_remote_tersh() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let local = inventory
        .hosts()
        .iter()
        .find(|host| host.alias() == "qiushan-mbp")
        .unwrap();
    let server = inventory
        .hosts()
        .iter()
        .find(|host| host.alias() == "school-star")
        .unwrap();

    let local_command = host_workbench_command(local, "/tmp/current-tersh");
    assert_eq!(local_command.program(), "/tmp/current-tersh");
    assert!(local_command.args().is_empty());

    let server_command = host_workbench_command(server, "/tmp/current-tersh");
    let args = server_command.args();
    assert_eq!(server_command.program(), "ssh");
    assert!(args.contains(&"-t".to_string()));
    assert!(
        args.windows(2)
            .any(|pair| pair == ["-J", "joshua@100.90.116.54"])
    );
    assert!(args.iter().any(|arg| arg == "star@10.13.7.138"));
    assert!(args.last().unwrap().contains("exec tersh"));
    assert!(args.last().unwrap().contains("/srv/star"));
    assert!(!args.windows(2).any(|pair| pair == ["-o", "BatchMode=yes"]));
}

#[test]
fn cluster_app_returns_open_session_command_for_s_key() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());

    let command = app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('s'),
        crossterm::event::KeyModifiers::NONE,
    ));

    assert_eq!(command, Some(ClusterCommand::OpenSession));
}

#[test]
fn cluster_app_returns_open_workbench_command_for_t_key() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());

    let command = app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('t'),
        crossterm::event::KeyModifiers::NONE,
    ));

    assert_eq!(command, Some(ClusterCommand::OpenWorkbench));
}

#[test]
fn cluster_app_does_not_open_session_from_help_overlay() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    app.apply(ClusterCommand::OpenHelp);

    let command = app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('s'),
        crossterm::event::KeyModifiers::NONE,
    ));

    assert_eq!(command, None);
}

#[test]
fn cluster_help_closes_on_question_mark_and_enter() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    app.apply(ClusterCommand::OpenHelp);

    let command = app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('?'),
        crossterm::event::KeyModifiers::NONE,
    ));

    assert_eq!(command, None);
    assert_eq!(app.mode(), tersh::cluster::ClusterMode::Normal);

    app.apply(ClusterCommand::OpenHelp);
    let command = app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Enter,
        crossterm::event::KeyModifiers::NONE,
    ));

    assert_eq!(command, None);
    assert_eq!(app.mode(), tersh::cluster::ClusterMode::Normal);
}

#[test]
fn cluster_app_does_not_open_workbench_from_help_overlay() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    app.apply(ClusterCommand::OpenHelp);

    let command = app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('t'),
        crossterm::event::KeyModifiers::NONE,
    ));

    assert_eq!(command, None);
}

#[test]
fn cluster_app_tracks_selection_updates_and_summary_counts() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());

    app.apply(ClusterCommand::Down);
    app.apply(ClusterCommand::Down);
    assert_eq!(app.selected_host().alias(), "school-star");

    app.apply_snapshot(HostSnapshot::online(
        "school-star",
        ProbeReport::parse(
            "hostname=starbox\nload=0.10 0.10 0.10\nmemory=1/2 GB (50%)\nstorage=4/8 GB 50% used\ntasks=10 processes\n",
        ),
        123,
    ));

    assert_eq!(app.online_count(), 1);
    assert_eq!(app.offline_count(), 0);
    assert_eq!(
        app.selected_snapshot().unwrap().connection,
        ConnectionState::Online
    );
}

#[test]
fn cluster_app_keeps_last_good_metrics_when_refresh_fails() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    app.apply(ClusterCommand::Down);
    app.apply(ClusterCommand::Down);
    app.apply_snapshot(HostSnapshot::online(
        "school-star",
        ProbeReport::parse(
            "hostname=starbox\nload=0.10 0.10 0.10\nmemory=1/2 GB (50%)\nstorage=4/8 GB 50% used\ntasks=10 processes\n",
        ),
        123,
    ));

    let aliases = vec!["school-star".to_string()];
    assert_eq!(app.begin_refresh(&aliases), aliases);
    app.apply_snapshot(HostSnapshot::offline(
        "school-star",
        "probe timed out after 6s",
    ));

    let snapshot = app.selected_snapshot().unwrap();
    assert_eq!(snapshot.connection, ConnectionState::Stale);
    assert_eq!(snapshot.report.memory.as_deref(), Some("1/2 GB (50%)"));
    assert_eq!(snapshot.error.as_deref(), Some("probe timed out after 6s"));
    assert_eq!(app.stale_count(), 1);
}

#[test]
fn cluster_app_ignores_duplicate_refresh_for_in_flight_host() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());

    let aliases = vec!["school-star".to_string()];

    assert_eq!(app.begin_refresh(&aliases), aliases);
    assert!(app.begin_refresh(&["school-star".to_string()]).is_empty());
}

#[test]
fn begin_refresh_caps_concurrent_hosts() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    let aliases = (0..40)
        .map(|index| format!("host-{index:02}"))
        .collect::<Vec<_>>();

    let started = app.begin_refresh(&aliases);

    assert_eq!(started.len(), 16);
}

#[test]
fn begin_refresh_rotates_after_capped_batch_completes() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    let aliases = (0..40)
        .map(|index| format!("host-{index:02}"))
        .collect::<Vec<_>>();

    let first = app.begin_refresh(&aliases);
    for alias in first {
        app.apply_snapshot(HostSnapshot::offline(alias, "offline"));
    }
    let second = app.begin_refresh(&aliases);

    assert!(second.iter().any(|alias| alias == "host-16"));
    assert!(!second.iter().any(|alias| alias == "host-00"));
}

#[test]
fn inventory_rejects_duplicate_aliases() {
    let json = r#"
{
  "servers": [
    { "alias": "dup", "ssh_user": "ops", "campus_ip": "203.0.113.10" },
    { "alias": "dup", "ssh_user": "ops", "campus_ip": "203.0.113.11" }
  ]
}
"#;

    let err = ClusterInventory::from_json(json).unwrap_err();

    assert!(err.to_string().contains("duplicate alias"));
}

#[test]
fn inventory_rejects_option_like_ssh_fields() {
    let json = r#"
{
  "servers": [
    { "alias": "bad", "ssh_user": "-oProxyCommand=bad", "campus_ip": "203.0.113.10" }
  ]
}
"#;

    let err = ClusterInventory::from_json(json).unwrap_err();

    assert!(err.to_string().contains("ssh_user"));
}

#[test]
fn inventory_rejects_option_like_address_fields() {
    let json = r#"
{
  "servers": [
    { "alias": "bad", "ssh_user": "ops", "campus_ip": "-bad" }
  ]
}
"#;

    let err = ClusterInventory::from_json(json).unwrap_err();

    assert!(err.to_string().contains("address"));
}

#[test]
fn inventory_rejects_control_characters_in_alias_role_and_workdir() {
    let json = "{ \"servers\": [ { \"alias\": \"bad\\nname\", \"role\": \"ops\", \"workdir\": \"/srv/app\" } ] }";
    let err = ClusterInventory::from_json(json).unwrap_err();
    assert!(err.to_string().contains("alias"));

    let json = "{ \"servers\": [ { \"alias\": \"server\", \"role\": \"ops\\nteam\", \"workdir\": \"/srv/app\" } ] }";
    let err = ClusterInventory::from_json(json).unwrap_err();
    assert!(err.to_string().contains("role"));

    let json = "{ \"servers\": [ { \"alias\": \"server\", \"role\": \"ops\", \"workdir\": \"/srv/app\\nnext\" } ] }";
    let err = ClusterInventory::from_json(json).unwrap_err();
    assert!(err.to_string().contains("workdir"));
}

#[test]
fn inventory_rejects_duplicate_alias_across_local_jump_and_servers() {
    let json = r#"
{
  "main_machine": { "alias": "dup" },
  "jump_host": { "alias": "dup", "tailscale_ip": "100.90.116.54" }
}
"#;

    let err = ClusterInventory::from_json(json).unwrap_err();

    assert!(err.to_string().contains("duplicate alias"));
}

#[test]
fn inventory_rejects_unresolved_proxy_jump() {
    let json = r#"
{
  "servers": [
    {
      "alias": "server",
      "ssh_user": "ops",
      "campus_ip": "203.0.113.10",
      "proxy_jump": "missing-jump"
    }
  ]
}
"#;

    let err = ClusterInventory::from_json(json).unwrap_err();

    assert!(err.to_string().contains("proxy_jump"));
}

#[test]
fn cluster_render_shows_host_list_detail_metrics_and_footer_keys() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    app.apply(ClusterCommand::Down);
    app.apply(ClusterCommand::Down);
    app.apply_snapshot(HostSnapshot::online(
        "school-star",
        ProbeReport::parse(
            "hostname=starbox\nsystem=Linux x86_64\nuptime=up 4 days\nload=0.12 0.20 0.30\nmemory=512/1024 MB (50%)\nstorage=8G/20G 40% used\ntasks=72 processes\ngpu=none\n",
        ),
        87,
    ));

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(buffer.contains("Cluster Status"));
    assert!(buffer.contains("OK 1"));
    assert!(buffer.contains("OLD 0"));
    assert!(buffer.contains("FAIL 0"));
    assert!(buffer.contains("CHK 0/4"));
    assert!(buffer.contains("Route"));
    assert!(buffer.contains("LOCAL"));
    assert!(buffer.contains("JUMP"));
    assert!(buffer.contains("SERVER"));
    assert!(buffer.contains("ssh -J"));
    assert!(buffer.contains("joshua@100.90.116.54"));
    assert!(buffer.contains("school-star"));
    assert!(buffer.contains("87ms"));
    assert!(buffer.contains("50%"));
    assert!(buffer.contains("CPU load"));
    assert!(buffer.contains("CPU load: 0.12 0.20 0.30"));
    assert!(!buffer.contains("CPU load: ["));
    assert!(buffer.contains("Memory"));
    assert!(buffer.contains("Memory: ["));
    assert!(buffer.contains("512/1024 MB (50%)"));
    assert!(buffer.contains("Storage"));
    assert!(buffer.contains("Storage: ["));
    assert!(buffer.contains("8G/20G 40% used"));
    assert!(buffer.contains("Tasks"));
    assert!(buffer.contains("GPU: ["));
    assert!(buffer.contains("Health"));
    assert!(buffer.contains("System"));
    assert!(buffer.contains("Log"));
    assert!(buffer.contains("r refresh"));
    assert!(buffer.contains("s shell/ssh"));
    assert!(buffer.contains("t tersh"));
    assert!(buffer.contains("q quit"));
}

#[test]
fn cluster_escape_closes_detail_view() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    app.apply(ClusterCommand::OpenDetail);

    let command = app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Esc,
        crossterm::event::KeyModifiers::NONE,
    ));

    assert_eq!(command, None);
    assert_eq!(app.mode(), tersh::cluster::ClusterMode::Normal);
}

#[test]
fn cluster_render_tiny_keeps_exit_visible() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let app = ClusterApp::new(inventory.hosts().to_vec());

    let backend = TestBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(buffer.contains("q quit"));
    assert!(buffer.contains("? help"));
    assert!(buffer.contains("^G"));
    assert!(buffer.contains("^C"));
}

#[test]
fn cluster_render_long_host_rows_stay_single_line() {
    let json = r#"
{
  "servers": [
    {
      "alias": "very-long-host-alias-for-rendering",
      "ssh_user": "ops",
      "campus_ip": "203.0.113.12345678901234567890",
      "role": "Long address test"
    },
    {
      "alias": "second-host",
      "ssh_user": "ops",
      "campus_ip": "203.0.113.20",
      "role": "Visible after long row"
    }
  ]
}
"#;
    let inventory = ClusterInventory::from_json(json).unwrap();
    let app = ClusterApp::new(inventory.hosts().to_vec());

    let backend = TestBackend::new(70, 12);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    assert!(buffer.contains("second-host"));
    assert!(buffer.contains("..."));
}

#[test]
fn cluster_render_unicode_host_rows_stay_ascii_and_truncated() {
    let json = r#"
{
  "servers": [
    {
      "alias": "服务器服务器服务器服务器服务器",
      "ssh_user": "ops",
      "campus_ip": "地址地址地址地址地址地址地址",
      "role": "unicode"
    },
    {
      "alias": "after-unicode",
      "ssh_user": "ops",
      "campus_ip": "203.0.113.30",
      "role": "Visible after unicode row"
    }
  ]
}
"#;
    let inventory = ClusterInventory::from_json(json).unwrap();
    let app = ClusterApp::new(inventory.hosts().to_vec());

    let backend = TestBackend::new(70, 12);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    assert!(buffer.is_ascii());
    assert!(buffer.contains("after-unicode"));
}

#[test]
fn cluster_detail_escapes_dynamic_probe_and_inventory_text() {
    let json = r#"
{
  "servers": [
    {
      "alias": "unicode-detail",
      "ssh_user": "ops",
      "campus_ip": "203.0.113.40",
      "role": "Detail"
    }
  ]
}
"#;
    let inventory = ClusterInventory::from_json(json).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    app.apply_snapshot(HostSnapshot::stale(
        "unicode-detail",
        ProbeReport::parse(
            "hostname=主机\nsystem=Linux\u{1b}[31m\nuptime=up\nload=0.12\u{1b}[31m\nmemory=42% free\nstorage=8G/20G 40% used\ntasks=72\u{1b}[31m processes\ngpu=GPU😀 9%\n",
        ),
        "bad\n\u{1b}[31m error",
    ));
    app.apply(ClusterCommand::OpenDetail);

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    assert!(buffer.is_ascii());
    assert!(buffer.contains("Error: bad"));
    assert!(buffer.contains("?[31m"));
    assert!(buffer.contains("Hostname: ??"));
}

#[test]
fn cluster_operational_chrome_is_ascii() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let app = ClusterApp::new(inventory.hosts().to_vec());

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    assert!(buffer.is_ascii());
}

#[test]
fn cluster_help_footer_is_mode_specific() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    app.apply(ClusterCommand::OpenHelp);

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(buffer.contains("help | q/?/Enter/Esc close"));
    assert!(!buffer.contains("q quit | ? help | ^G back | ^C force | r refresh"));
}

#[test]
fn cluster_detail_footer_describes_back_action() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    app.apply(ClusterCommand::OpenDetail);

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(buffer.contains("detail | q/Esc back"));
    assert!(!buffer.contains("q quit | ? help | ^G back | ^C force | r refresh"));
}

#[test]
fn cluster_render_local_host_shows_local_only_route() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let app = ClusterApp::new(inventory.hosts().to_vec());

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(buffer.contains("Route"));
    assert!(buffer.contains("LOCAL ONLY"));
    assert!(buffer.contains("qiushan-mbp"));
    assert!(!buffer.contains("ssh -J"));
}

#[test]
fn cluster_render_direct_server_route_has_no_jump_hop() {
    let inventory = ClusterInventory::from_json(DIRECT_JSON).unwrap();
    let app = ClusterApp::new(inventory.hosts().to_vec());

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(buffer.contains("Route"));
    assert!(buffer.contains("LOCAL"));
    assert!(buffer.contains("SERVER"));
    assert!(buffer.contains("ssh ops@203.0.113.10"));
    assert!(!buffer.contains("JUMP"));
    assert!(!buffer.contains("ssh -J"));
}

#[test]
fn cluster_render_medium_boundary_keeps_route_and_metrics_visible() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    app.apply(ClusterCommand::Down);
    app.apply(ClusterCommand::Down);
    app.apply_snapshot(HostSnapshot::online(
        "school-star",
        ProbeReport::parse(
            "hostname=starbox\nload=0.12 0.20 0.30\nmemory=512/1024 MB (50%)\nstorage=8G/20G 40% used\ntasks=72 processes\ngpu=none\n",
        ),
        87,
    ));

    let backend = TestBackend::new(72, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(buffer.contains("Hosts"));
    assert!(buffer.contains("Route"));
    assert!(buffer.contains("LOCAL"));
    assert!(buffer.contains("JUMP"));
    assert!(buffer.contains("SERVER"));
    assert!(buffer.contains("Memory"));
    assert!(buffer.contains("Storage"));
    assert!(buffer.contains("Log:"));
    assert!(buffer.contains("r refresh"));
}

#[test]
fn cluster_render_resource_bars_keep_none_gpu_neutral_and_parse_free_memory() {
    let inventory = ClusterInventory::from_json(DIRECT_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    app.apply_snapshot(HostSnapshot::online(
        "direct-box",
        ProbeReport::parse(
            "hostname=direct\nload=0.12 0.20 0.30\nmemory=42% free\nstorage=8G/20G 40% used\ntasks=72 processes\ngpu=none\n",
        ),
        42,
    ));

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(buffer.contains("Memory: [######----]  58% used  42% free"));
    assert!(buffer.contains("GPU: [----------]  --  none"));
}

#[test]
fn cluster_render_narrow_normal_mode_keeps_host_list_primary() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    app.apply(ClusterCommand::Down);
    app.apply(ClusterCommand::Down);
    app.apply_snapshot(HostSnapshot::online(
        "school-star",
        ProbeReport::parse(
            "hostname=starbox\nload=0.12 0.20 0.30\nmemory=512/1024 MB (50%)\nstorage=8G/20G 40% used\ntasks=72 processes\ngpu=none\n",
        ),
        87,
    ));

    let backend = TestBackend::new(71, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(buffer.contains("Hosts"));
    assert!(buffer.contains("school-star"));
    assert!(!buffer.contains("Route"));
    assert!(!buffer.contains("Memory: ["));
    assert!(buffer.contains("r refresh"));
}

#[test]
fn cluster_render_narrow_detail_mode_shows_metrics() {
    let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
    let mut app = ClusterApp::new(inventory.hosts().to_vec());
    app.apply(ClusterCommand::Down);
    app.apply(ClusterCommand::Down);
    app.apply(ClusterCommand::OpenDetail);
    app.apply_snapshot(HostSnapshot::online(
        "school-star",
        ProbeReport::parse(
            "hostname=starbox\nload=0.12 0.20 0.30\nmemory=512/1024 MB (50%)\nstorage=8G/20G 40% used\ntasks=72 processes\n",
        ),
        87,
    ));

    let backend = TestBackend::new(60, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| cluster_ui::draw(frame, &app))
        .unwrap();

    let buffer = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(buffer.contains("Route"));
    assert!(buffer.contains("LOCAL"));
    assert!(buffer.contains("JUMP"));
    assert!(buffer.contains("SERVER"));
    assert!(buffer.contains("CPU load"));
    assert!(buffer.contains("Memory"));
    assert!(buffer.contains("Storage"));
    assert!(buffer.contains("Tasks"));
}
