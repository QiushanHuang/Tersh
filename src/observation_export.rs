//! One bounded snapshot through the existing collector. This module neither
//! installs an agent nor schedules refreshes or file operations.
use crate::cluster::{
    ClusterInventory, ConnectionState, HostConfig, HostSnapshot, collect_host_snapshot,
};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    fs::OpenOptions,
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

fn stamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn field(
    raw: Option<&str>,
    unit: &str,
    ttl: u64,
    parse: impl FnOnce(&str) -> Option<Value>,
) -> Value {
    let usable = raw.filter(|v| !v.is_empty() && *v != "none" && !v.starts_with("unknown"));
    let value = usable.and_then(parse);
    json!({"value": value, "raw": raw, "unit": unit, "ttl_seconds": ttl,
        "status": if value.is_some() {"observed"} else {"unknown"},
        "reason": if value.is_some() {Value::Null} else {json!("missing_command_output_or_unparseable; never proof of absence")}})
}

fn text(value: &str) -> Option<Value> {
    Some(json!(value))
}

pub fn observation(
    host: &HostConfig,
    resource_id: &str,
    snapshot: HostSnapshot,
    started: u64,
    ended: u64,
) -> Value {
    let r = &snapshot.report;
    let memory_unit = if r.legacy_darwin_pressure_index().is_some() {
        "legacy_macos_memory_pressure_index_percent"
    } else {
        "native_text; free_-m_and_vm_stat_estimate_in_MiB"
    };
    let fields = json!({
        "hostname": field(r.hostname.as_deref(), "name", 86400, text),
        "system": field(r.system.as_deref(), "os_release_architecture", 86400, text),
        "uptime": field(r.uptime.as_deref(), "native_uptime_text", 60, text),
        "cpu_load": field(r.cpu_load.as_deref(), "runnable_tasks_1_5_15_min", 60, |v| {
            let values: Option<Vec<f64>> = v.split_whitespace().map(|n| n.parse().ok().filter(|n: &f64| n.is_finite() && *n >= 0.0)).collect();
            values.filter(|v| v.len() == 3).map(|v| json!(v))
        }),
        "memory": field(r.memory.as_deref(), memory_unit, 60, |v| {
            (v.contains(" MB (") || v.contains(" MiB (") || v.ends_with("% free")).then(|| json!(v))
        }),
        "storage": field(r.storage.as_deref(), "df_-hP_native_human_units", 60, |v| v.ends_with("% used").then(|| json!(v))),
        "tasks": field(r.tasks.as_deref(), "process_count_including_ps_header", 60, |v| v.strip_suffix(" processes")?.parse::<u64>().ok().map(|n| json!(n))),
        "gpu": field(r.gpu.as_deref(), "name; utilization_percent; memory_MiB", 60, |v| {
            let rows: Option<Vec<Value>> = v.split(';').map(|line| {
                let parts: Vec<_> = line.split(',').map(str::trim).collect();
                if parts.len() != 4 || parts[0].is_empty() { return None; }
                let util = parts[1].parse::<f64>().ok()?;
                let used = parts[2].parse::<f64>().ok()?;
                let total = parts[3].parse::<f64>().ok()?;
                if !util.is_finite() || !used.is_finite() || !total.is_finite() || !(0.0..=100.0).contains(&util) || used < 0.0 || total < used { return None; }
                Some(json!({"name":parts[0],"utilization_percent":util,"memory_used":used,"memory_total":total,"memory_unit":"MiB"}))
            }).collect();
            rows.map(|r| json!(r))
        })
    });
    let partial = fields
        .as_object()
        .unwrap()
        .values()
        .any(|v| v["status"] == "unknown");
    let outcome = match snapshot.connection {
        ConnectionState::Online if partial => "partial",
        ConnectionState::Online => "success",
        ConnectionState::Timeout => "timeout",
        _ => "failure",
    };
    json!({"schema":"workweft.tersh-observation.v1", "kind":"tersh.telemetry",
        "id":format!("tersh:{}:{}:{}:{}",host.alias(),started,ended,std::process::id()),
        "resource_id":resource_id, "producer":{"name":"Tersh","version":env!("CARGO_PKG_VERSION"),"method":"cluster::collect_host_snapshot"},
        "target":{"alias":host.alias(),"address":host.address(),"user":host.user(),"ssh_target":host.ssh_target(),"proxy_jump":host.proxy_jump_target(),"kind":host.kind().label()},
        "observed_at_unix_ms":ended,"source_observed_at":r.source_time.as_deref().filter(|v| *v != "unknown"),
        "collection":{"started_at_unix_ms":started,"ended_at_unix_ms":ended,"deadline_seconds":30,"elapsed_ms":ended.saturating_sub(started),"exit_code":if snapshot.connection == ConnectionState::Online {Some(0)} else {None::<i32>},"latency_ms":snapshot.latency_ms},
        "outcome":outcome,"reachability":snapshot.connection.label(),"error":snapshot.error,"fields":fields,
        "reservation":null,"quota":null,"scheduling_permission":null,"automatic_refresh":false,
        "limitations":["One selected host; an observation is not scheduling permission.","No hardware absence is inferred from missing tools.","Failure snapshots may omit partial stdout; exit code remains unknown when the existing collector did not expose it."]})
}

pub fn run(inventory: &Path, alias: &str, resource_id: &str, output: &Path) -> Result<()> {
    ensure!(
        resource_id.starts_with("resource:") && resource_id.len() > 9,
        "explicit resource identity required"
    );
    ensure!(
        !output.exists(),
        "output already exists; refusing to re-probe or overwrite"
    );
    let inventory = ClusterInventory::from_path(inventory)?;
    let host = inventory
        .hosts()
        .iter()
        .find(|h| h.alias() == alias)
        .context("selected host absent from inventory")?;
    let started = stamp();
    let snapshot = collect_host_snapshot(host);
    let value = observation(host, resource_id, snapshot, started, stamp());
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    let mut file = options
        .open(output)
        .context("create new observation receipt")?;
    file.write_all(&serde_json::to_vec_pretty(&value)?)?;
    file.sync_all()?;
    println!("{}", output.display());
    ensure!(
        matches!(value["outcome"].as_str(), Some("success" | "partial")),
        "collection failed; receipt preserved"
    );
    Ok(())
}
