use std::process::Command;

#[test]
fn help_separates_product_name_from_cli_tool_name() {
    let binary = std::env::var("CARGO_BIN_EXE_tersh").expect("tersh binary target exists");
    let output = Command::new(binary)
        .arg("--help")
        .output()
        .expect("run tersh --help");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("help output is utf-8");
    assert!(stdout.contains("Tersh is a lightweight terminal file workbench"));
    assert!(stdout.contains("Usage: tersh [OPTIONS] [PATH]"));
    assert!(!stdout.contains("Usage: Tersh"));
}

#[test]
fn help_documents_print_cwd_for_shell_cd_wrappers() {
    let binary = std::env::var("CARGO_BIN_EXE_tersh").expect("tersh binary target exists");
    let output = Command::new(binary)
        .arg("--help")
        .output()
        .expect("run tersh --help");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("help output is utf-8");
    assert!(stdout.contains("--print-cwd"));
    assert!(stdout.contains("shell cd"));
}

#[test]
fn help_documents_cluster_status_manager_flag() {
    let binary = std::env::var("CARGO_BIN_EXE_tersh").expect("tersh binary target exists");
    let output = Command::new(binary)
        .arg("--help")
        .output()
        .expect("run tersh --help");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("help output is utf-8");
    assert!(stdout.contains("--cluster"));
    assert!(stdout.contains("--c"));
    assert!(stdout.contains("read-only cluster health"));
    assert!(stdout.contains("route"));
    assert!(stdout.contains("selected host"));
}

#[test]
fn cluster_long_alias_keeps_existing_cluster_mode_contracts() {
    let binary = std::env::var("CARGO_BIN_EXE_tersh").expect("tersh binary target exists");
    let output = Command::new(binary)
        .args(["--cluster", "--print-cwd"])
        .output()
        .expect("run tersh --cluster --print-cwd");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert!(stderr.contains("cannot be used with"));
}

#[test]
fn version_reports_minor_release() {
    let binary = std::env::var("CARGO_BIN_EXE_tersh").expect("tersh binary target exists");
    let output = Command::new(binary)
        .arg("--version")
        .output()
        .expect("run tersh --version");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("version output is utf-8");
    assert!(stdout.contains("tersh 1.2.0"));
}

#[test]
fn cluster_status_conflicts_with_print_cwd_wrapper_mode() {
    let binary = std::env::var("CARGO_BIN_EXE_tersh").expect("tersh binary target exists");
    let output = Command::new(binary)
        .args(["--c", "--print-cwd"])
        .output()
        .expect("run tersh --c --print-cwd");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert!(stderr.contains("cannot be used with"));
}

#[test]
fn cluster_status_conflicts_with_file_workbench_path_argument() {
    let binary = std::env::var("CARGO_BIN_EXE_tersh").expect("tersh binary target exists");
    let output = Command::new(binary)
        .args(["--c", "/tmp"])
        .output()
        .expect("run tersh --c /tmp");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert!(stderr.contains("cannot be used with"));
}
#[test]
fn help_exposes_device_profiles_and_rejects_unknown_profile() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tersh"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("--ui-profile"));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tersh"))
        .args(["--ui-profile", "unknown"])
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn keymap_dump_and_invalid_config_are_headless() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tersh"))
        .arg("--dump-keymap")
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["files"]["copy"].is_array());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("keymap.json");
    std::fs::write(&path, r#"{"files":{"copy":["q"]}}"#).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tersh"))
        .arg("--keymap")
        .arg(path)
        .arg("--dump-keymap")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!output.stdout.contains(&0x1b));
}
