use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};
use tersh::{
    cluster::{ClusterApp, ClusterCommand, ClusterInventory, HostSnapshot, ProbeReport},
    cluster_ui,
};

const HARDWARE: &str = "hostname=worker\nsystem=Linux x86_64\n\
    hardware_model=Compute station\narchitecture=x86_64\n\
    cpu_model=Example CPU\ncpu_cores=16\ncpu_threads=32\ncpu_usage=37.5\n\
    memory_total_bytes=68719476736\nmemory_used_bytes=17179869184\n\
    memory_basis=Linux MemTotal - MemAvailable\n\
    hardware_gpu=0|NVIDIA Test GPU|72|4294967296|25769803776|56|dedicated\n\
    hardware_gpu=1|NVIDIA Second GPU|0|0|25769803776|32|dedicated\n";

fn key(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)
}

fn app() -> ClusterApp {
    let inventory = ClusterInventory::from_json(
        r#"{"servers":[{"alias":"worker","campus_ip":"worker.invalid"},{"alias":"other","campus_ip":"other.invalid"}]}"#,
    ).unwrap();
    let mut app = ClusterApp::from_inventory(inventory);
    app.apply_snapshot(HostSnapshot::online(
        "worker",
        ProbeReport::parse(HARDWARE),
        12,
    ));
    app
}

fn render(app: &ClusterApp, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
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
fn hardware_only_report_is_a_usable_observation() {
    assert!(!ProbeReport::parse("cpu_model=Example CPU\ncpu_usage=0\n").is_empty());
}

#[test]
fn hardware_key_opens_per_device_inventory_and_live_usage() {
    let mut app = app();
    app.handle_key(key('H'));
    let text = render(&app, 110, 40);
    for expected in [
        "Hardware",
        "Example CPU",
        "16 cores",
        "32 threads",
        "38%",
        "64.0 GiB",
        "16.0 GiB",
        "NVIDIA Test GPU",
        "NVIDIA Second GPU",
        "72%",
        "56 C",
    ] {
        assert!(text.contains(expected), "missing {expected}:\n{text}");
    }
    app.handle_key(key('j'));
    let other = render(&app, 80, 30);
    assert!(
        other.contains("other") && other.contains("unknown"),
        "{other}"
    );
    assert!(
        !other.contains("Example CPU"),
        "hardware leaked between hosts: {other}"
    );
    app.handle_key(key('q'));
    assert!(!app.should_quit(), "q should return to hosts");
    assert_eq!(app.key_context(), "cluster");
}

#[test]
fn hardware_action_is_searchable_and_can_be_remapped() {
    let mut app = app();
    assert!(ClusterCommand::from_action("open_hardware").is_some());
    app.set_keymap(
        tersh::keymap::Keymap::from_json(
            r#"{"cluster":{"open_hardware":["F8"]},"cluster_detail":{"open_hardware":["F8"]}}"#,
        )
        .unwrap(),
    );
    app.handle_key(key('o'));
    for ch in "hardware".chars() {
        app.handle_key(key(ch));
    }
    let menu = render(&app, 100, 30);
    assert!(menu.contains("Hardware") && menu.contains("F8"), "{menu}");
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(render(&app, 100, 40).contains("Example CPU"));
}

#[test]
fn hardware_retains_last_reading_with_stale_status_and_scrolls_on_narrow_screens() {
    let mut app = app();
    app.apply_snapshot(HostSnapshot::failed("worker", "probe timed out"));
    app.handle_key(key('H'));
    let text = render(&app, 40, 12);
    assert!(
        text.contains("stale") && text.contains("Last good"),
        "{text}"
    );
    let mut all = text;
    for _ in 0..18 {
        app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
        all.push_str(&render(&app, 40, 12));
    }
    for expected in ["Example CPU", "NVIDIA Test GPU", "NVIDIA Second GPU"] {
        assert!(all.contains(expected), "cannot reach {expected}: {all}");
    }
    assert!(all.contains("back"), "narrow screen must expose exit");
}

#[test]
fn local_probe_collects_inventory_and_sampled_cpu_without_remote_access() {
    let inventory =
        ClusterInventory::from_json(r#"{"main_machine":{"alias":"local-test"}}"#).unwrap();
    let snapshot = tersh::cluster::collect_host_snapshot(&inventory.hosts()[0]);
    assert_eq!(
        snapshot.connection,
        tersh::cluster::ConnectionState::Online,
        "{:?}",
        snapshot.error
    );
    let h = snapshot.report.hardware;
    assert!(h.cpu_threads.is_some(), "logical CPU count missing");
    assert!(h.cpu_usage_percent.is_some(), "interval CPU sample missing");
    assert!(h.memory_total_bytes.is_some(), "RAM capacity missing");
    assert!(h.memory_used_bytes.is_some(), "RAM usage missing");
    assert!(h.architecture.is_some());
}

#[test]
fn linux_cpu_uses_counter_deltas_excluding_guest_and_iowait() {
    let h = ProbeReport::parse("cpu_ticks_start=100 20 30 400 50 4 6 10 100 20\ncpu_ticks_end=120 20 40 440 70 4 16 20 120 20\n").hardware;
    // Busy delta=20+10+10+10=50; total delta=110; not load/cores or since-boot CPU.
    assert_eq!(h.cpu_usage_percent, Some(45));
    for invalid in [
        "cpu_ticks_start=1 2 3 4 5 6 7 8\ncpu_ticks_end=1 2 3 4 5 6 7 8",
        "cpu_ticks_start=1 2 3 4 5 6 7 8\ncpu_ticks_end=0 3 4 5 6 7 8 9",
        "cpu_usage=NaN",
        "cpu_usage=101",
        "cpu_usage=-0.1",
    ] {
        assert_eq!(
            ProbeReport::parse(invalid).hardware.cpu_usage_percent,
            None,
            "{invalid}"
        );
    }
}

#[test]
fn partial_gpu_counters_keep_identity_without_inventing_zero_usage() {
    let h = ProbeReport::parse("hardware_gpu=0|Example GPU|N/A|N/A|25769803776|N/A|dedicated\nhardware_gpu=1|Apple Test|15|1073741824|68719476736||shared\n").hardware;
    assert_eq!(h.gpus.len(), 2);
    assert_eq!(h.gpus[0].usage_percent, None);
    assert_eq!(h.gpus[0].memory_used_bytes, None);
    assert_eq!(h.gpus[0].memory_total_bytes, Some(25769803776));
    assert_eq!(h.gpus[0].temperature_c, None);
    assert!(h.gpus[1].shared_memory);
    assert_eq!(
        h.gpus[1].memory_total_bytes, None,
        "unified RAM is not dedicated VRAM"
    );
}

#[test]
fn invalid_capacity_and_out_of_range_counters_remain_unknown() {
    let h = ProbeReport::parse("cpu_cores=0\ncpu_threads=-1\nmemory_total_bytes=1024\nmemory_used_bytes=2048\nhardware_gpu=0|Test|NaN|2048|1024|999|dedicated").hardware;
    assert_eq!(h.cpu_cores, None);
    assert_eq!(h.cpu_threads, None);
    assert_eq!(h.memory_used_bytes, None);
    assert_eq!(h.memory_percent(), None);
    assert_eq!(h.gpus[0].usage_percent, None);
    assert_eq!(h.gpus[0].memory_used_bytes, None);
    assert_eq!(h.gpus[0].temperature_c, None);
    let idle = ProbeReport::parse("cpu_usage=0\nmemory_total_bytes=1024\nmemory_used_bytes=0\nhardware_gpu=0|Idle GPU|0|0|1024|0|dedicated").hardware;
    assert_eq!(idle.cpu_usage_percent, Some(0));
    assert_eq!(idle.memory_percent(), Some(0));
    assert_eq!(idle.gpus[0].usage_percent, Some(0));
}

#[test]
fn hardware_small_terminal_keeps_all_fields_reachable_and_controls_visible() {
    let mut app = app();
    app.handle_key(key('H'));
    let first = render(&app, 40, 10);
    assert!(first.contains("scroll"), "{first}");
    assert!(
        first.contains("device"),
        "device navigation missing: {first}"
    );
    let mut all = first;
    for _ in 0..70 {
        app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
        all.push_str(&render(&app, 40, 10));
    }
    for field in [
        "Compute station",
        "Example CPU",
        "16 cores",
        "38%",
        "64.0 GiB",
        "72%",
        "56 C",
        "NVIDIA Second GPU",
    ] {
        assert!(
            all.contains(field),
            "scrolled past {field} without showing it"
        );
    }
}

#[test]
fn hardware_overview_marks_retained_values_even_without_color() {
    let mut app = app();
    app.apply_snapshot(HostSnapshot::failed("worker", "timed out"));
    app.handle_key(key('H'));
    let text = render(&app, 110, 40);
    let row = text
        .lines()
        .find(|line| line.contains("38%") && line.contains("worker"))
        .unwrap();
    assert!(
        row.contains("OLD"),
        "retained summary must be labeled: {row}"
    );
}
