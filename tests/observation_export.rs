use tersh::{
    cluster::{ClusterInventory, HostSnapshot, ProbeReport},
    observation_export::observation,
};

fn host() -> tersh::cluster::HostConfig {
    ClusterInventory::from_json(r#"{"servers":[{"alias":"demo-node","campus_ip":"demo-node"}]}"#)
        .unwrap()
        .hosts()[0]
        .clone()
}

#[test]
fn missing_gpu_command_is_unknown_not_no_gpu_or_idle() {
    let value = observation(
        &host(),
        "resource:demo-node",
        HostSnapshot::online(
            "demo-node",
            ProbeReport::parse("hostname=star\nload=0.1 0.2 0.3\ngpu=none\n"),
            10,
        ),
        100,
        101,
    );
    assert_eq!(value["kind"], "tersh.telemetry");
    assert_eq!(value["outcome"], "partial");
    assert!(value["fields"]["gpu"]["value"].is_null());
    assert_eq!(value["fields"]["gpu"]["status"], "unknown");
    assert_eq!(
        value["fields"]["cpu_load"]["unit"],
        "runnable_tasks_1_5_15_min"
    );
    assert!(value["scheduling_permission"].is_null());
}

#[test]
fn timeout_keeps_unknown_metrics_and_terminal_failure() {
    let value = observation(
        &host(),
        "resource:demo-node",
        HostSnapshot::failed("demo-node", "probe timed out after 30s"),
        100,
        130,
    );
    assert_eq!(value["outcome"], "timeout");
    assert_eq!(value["collection"]["deadline_seconds"], 30);
    assert!(value["fields"]["memory"]["value"].is_null());
    assert!(value["collection"]["exit_code"].is_null());
}

#[test]
fn darwin_export_distinguishes_estimated_ram_from_legacy_pressure_text() {
    for (raw, unit_part) in [
        ("35840/65536 MiB (55%)", "MiB"),
        ("90% free", "legacy_macos_memory_pressure_index_percent"),
    ] {
        let report = ProbeReport::parse(&format!("system=Darwin arm64\nmemory={raw}"));
        let snapshot = HostSnapshot::online("demo-node", report, 10);
        let value = observation(&host(), "resource:demo-node", snapshot, 100, 101);
        let memory = &value["fields"]["memory"];
        assert_eq!(memory["value"], raw);
        assert!(memory["unit"].as_str().unwrap().contains(unit_part));
    }
}

#[test]
fn gpu_values_and_remote_clock_have_explicit_units() {
    let value = observation(
        &host(),
        "resource:demo-node",
        HostSnapshot::online(
            "demo-node",
            ProbeReport::parse(
                "source_time=2026-10-24T00:00:00Z\nhostname=star\nsystem=Linux x86_64\nuptime=1 day\nload=0 0 0\nmemory=10/100 MB (10%)\nstorage=2G/10G 20% used\ntasks=20 processes\ngpu=RTX 5080, 15, 20, 16000\n",
            ),
            11,
        ),
        100,
        101,
    );
    assert_eq!(value["outcome"], "success");
    assert_eq!(value["source_observed_at"], "2026-10-24T00:00:00Z");
    assert_eq!(
        value["fields"]["gpu"]["value"][0]["utilization_percent"],
        15.0
    );
    assert_eq!(value["fields"]["gpu"]["value"][0]["memory_unit"], "MiB");
    assert_eq!(value["fields"]["gpu"]["ttl_seconds"], 60);
    assert_eq!(value["fields"]["system"]["ttl_seconds"], 86400);
}
