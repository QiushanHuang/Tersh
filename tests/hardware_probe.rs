//! Execute the actual POSIX probe with synthetic commands and kernel files.
//! No SSH, privileged commands, or dependencies on the machine's GPU hardware.
use std::{fs, os::unix::fs::PermissionsExt, process::Command};
use tersh::cluster::ProbeReport;

fn probe(os: &str, commands: &[(&str, &str)], files: &[(&str, &str)]) -> ProbeReport {
    let fixture = tempfile::tempdir().unwrap();
    let bin = fixture.path().join("bin");
    fs::create_dir(&bin).unwrap();
    let uname = format!(
        "case \"$1\" in -s) echo {os} ;; -m) echo test-arch ;; *) echo '{os} test-arch' ;; esac"
    );
    let defaults = [
        ("uname", uname.as_str()),
        ("sleep", "exit 0"),
        ("nvidia-smi", "exit 1"),
        ("lspci", "exit 1"),
        ("system_profiler", "exit 1"),
        ("ioreg", "exit 1"),
        ("getconf", "echo 8"),
        ("top", "exit 1"),
        ("memory_pressure", "exit 1"),
        (
            "sysctl",
            "case \"$2\" in hw.memsize) echo 68719476736 ;; hw.physicalcpu) echo 4 ;; hw.logicalcpu) echo 8 ;; hw.model) echo MacTest ;; machdep.cpu.brand_string) echo TestCPU ;; esac",
        ),
        (
            "vm_stat",
            "printf '%s\n' 'Mach Virtual Memory Statistics: (page size of 16384 bytes)' 'Pages free: 1048576.' 'Pages speculative: 0.'",
        ),
    ];
    for (name, body) in defaults.iter().chain(commands) {
        let file = bin.join(name);
        fs::write(&file, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(file, fs::Permissions::from_mode(0o700)).unwrap();
    }
    for (name, contents) in files {
        let file = fixture.path().join(name);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, contents).unwrap();
    }
    let script = include_str!("../src/hardware_probe.sh");
    let script = script
        .lines()
        .map(|line| {
            if line.starts_with("PATH=") {
                format!(
                    "PATH='{}/bin:/usr/bin:/bin:/usr/sbin:/sbin'",
                    fixture.path().display()
                )
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .replace("/proc/", &format!("{}/proc/", fixture.path().display()))
        .replace("/sys/", &format!("{}/sys/", fixture.path().display()));
    let result = Command::new("sh").args(["-c", &script]).output().unwrap();
    assert!(result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(!stderr.contains("awk:"), "probe parser failed: {stderr}");
    ProbeReport::parse(&String::from_utf8(result.stdout).unwrap())
}

#[test]
fn darwin_probe_reads_second_cpu_sample_and_each_accelerator() {
    let report = probe(
        "Darwin",
        &[
            (
                "top",
                "printf '%s\n' 'CPU usage: 1% user, 1% sys, 98% idle' 'CPU usage: 20% user, 35% sys, 45% idle'",
            ),
            (
                "ioreg",
                r#"cat <<'FIXTURE'
+-o AGXAccelerator <class AGXAccelerator>
    "PerformanceStatistics" = {"Device Utilization %"=31,"In use system memory (driver)"=0,"In use system memory"=2147483648}
    "model" = "Apple Test GPU"
+-o OtherAccelerator <class OtherAccelerator>
    "model" = "Discrete GPU"
    "PerformanceStatistics" = {"vramUsedBytes"=1073741824,"vramTotalBytes"=8589934592}
FIXTURE"#,
            ),
        ],
        &[],
    );
    let h = report.hardware;
    assert_eq!(h.cpu_usage_percent, Some(55));
    assert_eq!(h.memory_total_bytes, Some(68719476736));
    assert_eq!(h.memory_used_bytes, Some(51539607552));
    assert!(h.memory_basis.unwrap().contains("includes caches"));
    assert_eq!(h.gpus.len(), 2);
    assert_eq!(h.gpus[0].usage_percent, Some(31));
    assert!(h.gpus[0].shared_memory);
    assert_eq!(h.gpus[0].memory_used_bytes, Some(2147483648));
    assert_eq!(h.gpus[0].memory_total_bytes, None);
    assert_eq!(h.gpus[1].name, "Discrete GPU");
    assert_eq!(h.gpus[1].usage_percent, None);
    assert_eq!(h.gpus[1].memory_total_bytes, Some(8589934592));
}

#[test]
fn linux_probe_reads_capacity_topology_and_partial_nvidia_metrics() {
    let report = probe(
        "Linux",
        &[(
            "nvidia-smi",
            "printf '%s\n' '0, NVIDIA Test, 70, 1024, 24576, 56' '1, NVIDIA Other, [N/A], [N/A], 24576, [N/A]'",
        )],
        &[
            (
                "proc/meminfo",
                "MemTotal: 67108864 kB\nMemAvailable: 50331648 kB\n",
            ),
            (
                "proc/cpuinfo",
                "processor: 0\nmodel name: Example Linux CPU\nphysical id: 0\ncore id: 0\n\nprocessor: 1\nphysical id: 0\ncore id: 0\n\nprocessor: 2\nphysical id: 0\ncore id: 1\n\nprocessor: 3\nphysical id: 1\ncore id: 0\n",
            ),
            ("proc/stat", "cpu  10 20 30 40 50 60 70 80 90 100\n"),
        ],
    );
    let h = report.hardware;
    assert_eq!(h.cpu_model.as_deref(), Some("Example Linux CPU"));
    assert_eq!(h.cpu_cores, Some(3));
    assert_eq!(h.cpu_threads, Some(8));
    assert_eq!(
        h.cpu_usage_percent, None,
        "identical counter samples stay unknown"
    );
    assert_eq!(h.memory_total_bytes, Some(68719476736));
    assert_eq!(h.memory_used_bytes, Some(17179869184));
    assert_eq!(h.gpus.len(), 2);
    assert_eq!(h.gpus[0].usage_percent, Some(70));
    assert_eq!(h.gpus[0].memory_used_bytes, Some(1073741824));
    assert_eq!(h.gpus[0].temperature_c, Some(56));
    assert_eq!(h.gpus[1].usage_percent, None);
    assert_eq!(h.gpus[1].memory_used_bytes, None);
    assert_eq!(
        report.gpu.as_deref(),
        Some("NVIDIA Test, 70, 1024, 24576;NVIDIA Other, [N/A], [N/A], 24576")
    );
}

#[test]
fn linux_drm_keeps_gpu_identity_when_utilization_is_unsupported() {
    let report = probe(
        "Linux",
        &[],
        &[
            ("sys/class/drm/card0/device/vendor", "0x1002\n"),
            ("sys/class/drm/card0/device/device", "0x1234\n"),
            ("sys/class/drm/card0/device/gpu_busy_percent", "42\n"),
            (
                "sys/class/drm/card0/device/mem_info_vram_total",
                "8589934592\n",
            ),
            ("sys/class/drm/card1/device/vendor", "0x8086\n"),
            ("sys/class/drm/card1/device/device", "0x5678\n"),
        ],
    );
    assert_eq!(report.hardware.gpus.len(), 2);
    assert_eq!(report.hardware.gpus[0].name, "AMD 0x1234");
    assert_eq!(report.hardware.gpus[0].usage_percent, Some(42));
    assert_eq!(report.hardware.gpus[1].name, "Intel 0x5678");
    assert_eq!(report.hardware.gpus[1].usage_percent, None);
}

#[test]
fn missing_tools_and_kernel_fields_stay_unknown() {
    let report = probe("Linux", &[], &[("proc/meminfo", "MemTotal: 1024 kB\n")]);
    assert_eq!(report.hardware.memory_total_bytes, Some(1048576));
    assert_eq!(report.hardware.memory_used_bytes, None);
    assert_eq!(report.hardware.cpu_cores, None);
    assert_eq!(report.hardware.cpu_usage_percent, None);
    assert!(report.hardware.gpus.is_empty());
    assert!(report.gpu.unwrap().starts_with("unknown:"));
}

#[test]
fn nvidia_probe_does_not_hide_other_gpu_vendors() {
    let report = probe(
        "Linux",
        &[("nvidia-smi", "echo '0, NVIDIA Test, 70, 1024, 24576, 56'")],
        &[
            ("sys/class/drm/card0/device/vendor", "0x10de\n"),
            ("sys/class/drm/card0/device/device", "0x1111\n"),
            ("sys/class/drm/card1/device/vendor", "0x8086\n"),
            ("sys/class/drm/card1/device/device", "0x2222\n"),
        ],
    );
    assert_eq!(report.hardware.gpus.len(), 2);
    assert_eq!(report.hardware.gpus[0].name, "NVIDIA Test");
    assert_eq!(report.hardware.gpus[1].name, "Intel 0x2222");
}

#[test]
fn malformed_nvidia_output_does_not_suppress_drm_inventory() {
    let report = probe(
        "Linux",
        &[("nvidia-smi", "echo 'Driver not ready'")],
        &[
            ("sys/class/drm/card0/device/vendor", "0x10de\n"),
            ("sys/class/drm/card0/device/device", "0x1111\n"),
        ],
    );
    assert_eq!(report.hardware.gpus.len(), 1);
    assert_eq!(report.hardware.gpus[0].name, "NVIDIA 0x1111");
    assert_eq!(report.hardware.gpus[0].usage_percent, None);
}

#[test]
fn unidentifiable_drm_display_node_is_not_an_extra_physical_gpu() {
    let report = probe(
        "Linux",
        &[("nvidia-smi", "echo '0, NVIDIA Test, 70, 1024, 24576, 56'")],
        &[("sys/class/drm/card0/device/uevent", "DRIVER=faux_driver\n")],
    );
    assert_eq!(
        report.hardware.gpus.len(),
        1,
        "a display node without GPU identity must not become a second GPU"
    );
    assert_eq!(report.hardware.gpus[0].name, "NVIDIA Test");
}
