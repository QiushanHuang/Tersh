//! Export the real ratatui cell buffers with synthetic, offline fixtures.
//! cargo run --example ui_gallery -- target/ui-gallery
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};
use std::{
    fs,
    path::Path,
    time::{Duration, SystemTime},
};
use tersh::{
    app::{App, Command},
    cluster::{ClusterApp, ClusterCommand, ClusterInventory, HostSnapshot, ProbeReport},
};

fn export(name: &str, width: u16, height: u16, out: &Path, draw: impl FnOnce(&mut ratatui::Frame)) {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(draw).unwrap();
    let cells = terminal.backend().buffer().content().iter().map(|cell| serde_json::json!({
        "text": cell.symbol(), "fg": format!("{:?}", cell.fg), "bg": format!("{:?}", cell.bg),
        "bold": cell.modifier.contains(ratatui::style::Modifier::BOLD),
        "reverse": cell.modifier.contains(ratatui::style::Modifier::REVERSED),
    })).collect::<Vec<_>>();
    fs::write(
        out.join(format!("{name}.json")),
        serde_json::to_vec(&serde_json::json!({"width":width,"height":height,"cells":cells}))
            .unwrap(),
    )
    .unwrap();
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or("target/ui-gallery".into());
    let out = Path::new(&out);
    fs::create_dir_all(out).unwrap();
    let fixture = tempfile::Builder::new()
        .prefix("tersh-demo-")
        .tempdir_in("/tmp")
        .unwrap();
    for name in ["analysis", "logs", "results"] {
        fs::create_dir(fixture.path().join(name)).unwrap();
    }
    for (name, content) in [
        (
            "README.md",
            "# Tersh workspace\n\nLocal and SSH file work, in one terminal.\n\n- Inspect files\n- Preview before opening\n- Select, copy, and move safely\n",
        ),
        ("experiment-results-2026.csv", "time,value\n0,1.2\n1,1.4\n"),
        ("config.json", "{\n  \"refresh_seconds\": 15\n}\n"),
        ("run.sh", "#!/bin/sh\nprintf 'ready\\n'\n"),
    ] {
        fs::write(fixture.path().join(name), content).unwrap();
    }
    let mut app = App::new(fixture.path().into()).unwrap();
    app.handle_command(Command::Last);
    app.handle_command(Command::ToggleSelect);
    app.handle_command(Command::Copy);
    for (name, w, h) in [
        ("readme-workbench", 120, 28),
        ("workbench-wide", 160, 38),
        ("workbench-tablet", 100, 28),
        ("workbench-phone", 40, 18),
    ] {
        export(name, w, h, out, |f| tersh::ui::draw(f, &app));
    }
    app.handle_key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE));
    export("actions", 100, 28, out, |f| tersh::ui::draw(f, &app));
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    app.handle_command(Command::Trash);
    for ch in "trash".chars() {
        app.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while app.job_active() {
        app.poll_job();
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    app.handle_command(Command::OpenJobs);
    export("file-jobs", 100, 28, out, |f| tersh::ui::draw(f, &app));
    app.handle_command(Command::Cancel);
    app.handle_command(Command::OpenTrash);
    export("trash-recovery", 100, 28, out, |f| tersh::ui::draw(f, &app));
    app.handle_command(Command::RestoreTrash);
    export("restore-confirmation", 60, 20, out, |f| {
        tersh::ui::draw(f, &app)
    });
    app.handle_command(Command::Cancel);
    app.handle_command(Command::Cancel);
    app.set_keymap(
        tersh::keymap::Keymap::from_json(
            r#"{"files":{"copy":["Ctrl+y"],"open_jobs":["F5","J"],"open_trash":["F6","u"]}}"#,
        )
        .unwrap(),
    );
    app.handle_command(Command::OpenActions);
    export("custom-keys", 100, 28, out, |f| tersh::ui::draw(f, &app));
    let inventory = ClusterInventory::from_json(r#"{"servers":[
        {"alias":"compute-a","ssh_user":"demo","campus_ip":"compute-a.invalid","role":"Synthetic compute host"},
        {"alias":"compute-b","ssh_user":"demo","campus_ip":"compute-b.invalid","role":"Synthetic compute host"},
        {"alias":"archive","ssh_user":"demo","campus_ip":"archive.invalid","role":"Synthetic storage host"}
    ]}"#).unwrap();
    let mut cluster = ClusterApp::new(inventory.hosts().to_vec());
    let base = SystemTime::now();
    for n in 0..30 {
        let load = 1.5 + (n as f64 / 4.0).sin() * 0.7;
        let report = ProbeReport::parse(&format!(
            "hostname=compute-a\nsystem=Linux x86_64\nuptime=up 4 days\nload={load:.2} 1.5 1.4\nmemory={} % used\nstorage=40% used\ntasks=72 processes\ngpu=A100, 24%, 2048 MiB\n",
            45 + n
        ));
        let mut snapshot = if n == 18 {
            HostSnapshot::failed("compute-a", "synthetic timeout")
        } else {
            HostSnapshot::online("compute-a", report, 20 + n)
        };
        snapshot.refreshed_at = Some(base - Duration::from_secs((29 - n) as u64 * 15));
        cluster.apply_snapshot(snapshot);
    }
    cluster.apply_snapshot(HostSnapshot::online(
        "compute-b",
        ProbeReport::parse("load=0.8\nmemory=31% used\nstorage=66% used\n"),
        35,
    ));
    cluster.apply_snapshot(HostSnapshot::failed(
        "archive",
        "synthetic timeout; no connection attempted",
    ));
    export("cluster-wide", 160, 38, out, |f| {
        tersh::cluster_ui::draw(f, &cluster)
    });
    export("cluster-phone", 40, 18, out, |f| {
        tersh::cluster_ui::draw(f, &cluster)
    });
    cluster.apply(ClusterCommand::OpenDetail);
    export("readme-cluster", 100, 34, out, |f| {
        tersh::cluster_ui::draw(f, &cluster)
    });
    export("cluster-trends", 100, 38, out, |f| {
        tersh::cluster_ui::draw(f, &cluster)
    });
    export("cluster-detail-phone", 40, 24, out, |f| {
        tersh::cluster_ui::draw(f, &cluster)
    });
    cluster.apply(ClusterCommand::Cancel);
    cluster.apply(ClusterCommand::OpenFilter);
    for ch in "compute".chars() {
        cluster.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    cluster.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    cluster.apply(ClusterCommand::CycleSort);
    export("cluster-filtered", 100, 28, out, |f| {
        tersh::cluster_ui::draw(f, &cluster)
    });
    println!("Offline synthetic UI cell buffers: {}", out.display());
}
