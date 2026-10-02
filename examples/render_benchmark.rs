//! Offline renderer cost with synthetic host histories. No probes are launched.
use ratatui::{Terminal, backend::TestBackend};
use std::time::Instant;
use tersh::cluster::{ClusterApp, ClusterInventory, HostSnapshot, ProbeReport};

fn main() {
    let mut cases = Vec::new();
    for count in [20, 200, 1000] {
        let data = serde_json::json!({"servers": (0..count).map(|n| serde_json::json!({
            "alias": format!("node-{n:04}"), "campus_ip": format!("node-{n:04}.invalid"),
            "ssh_user": "fixture", "role": "Offline render fixture"
        })).collect::<Vec<_>>() });
        let inventory = ClusterInventory::from_json(&data.to_string()).unwrap();
        let mut app = ClusterApp::new(inventory.hosts().to_vec());
        for n in 0..count {
            for sample in 0..60 {
                app.apply_snapshot(HostSnapshot::online(
                    format!("node-{n:04}"),
                    ProbeReport::parse(&format!(
                        "load=1.2\nmemory={}% used\nstorage=42% used\n",
                        20 + sample
                    )),
                    24,
                ));
            }
        }
        for (width, height) in [(40, 18), (80, 24), (120, 36), (160, 40)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            let mut times = Vec::new();
            for run in 0..35 {
                let start = Instant::now();
                terminal.draw(|f| tersh::cluster_ui::draw(f, &app)).unwrap();
                if run >= 5 {
                    times.push(start.elapsed().as_secs_f64() * 1000.0);
                }
            }
            times.sort_by(f64::total_cmp);
            cases.push(serde_json::json!({"hosts":count,"samples_per_host":60,"width":width,"height":height,
                "render_ms_p50":times[15],"render_ms_p95":times[28],"runs":times.len()}));
        }
    }
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({
        "remote_hosts_contacted":false,
        "scope":"Synthetic snapshots and ratatui TestBackend; elapsed render time includes buffer work, not terminal transport or probes.",
        "cases":cases
    })).unwrap());
}
