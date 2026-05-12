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
    assert!(stdout.contains("Usage: tersh [PATH]"));
    assert!(!stdout.contains("Usage: Tersh"));
}
