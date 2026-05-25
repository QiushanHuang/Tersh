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
    assert!(stdout.contains("--c"));
    assert!(stdout.contains("multi-server status"));
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
