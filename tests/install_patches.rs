use std::{fs, process::Command};

fn command(home: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tersh"));
    command.env("HOME", home).env_remove("TERSH_KEYMAP");
    command
}

#[test]
fn installation_check_is_headless_read_only_and_reports_missing_protection() {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir(home.path().join(".ssh")).unwrap();
    let config = home.path().join(".ssh/config");
    let original = "Host sample\n  ProxyCommand ssh gateway ssh win wsl.exe -d Debian -- /usr/bin/nc 127.0.0.1 2222\n";
    fs::write(&config, original).unwrap();
    let output = command(home.path())
        .arg("--check-install-patches")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(&config).unwrap(), original);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(text.contains("--install-patches"), "{text}");
    assert!(!text.contains("unexpected argument"), "{text}");
    assert!(!text.contains('\u{1b}'));
    assert_eq!(fs::read_dir(home.path().join(".ssh")).unwrap().count(), 1);
}

#[test]
fn install_patches_preserves_other_hosts_and_is_idempotent_with_one_backup() {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir(home.path().join(".ssh")).unwrap();
    let config = home.path().join(".ssh/config");
    let original = "Host other\n  ProxyJump campus\nHost sample\n  ProxyCommand ssh gateway ssh win wsl.exe -d Debian -- /usr/bin/nc 127.0.0.1 2207\n";
    fs::write(&config, original).unwrap();
    let apply = command(home.path())
        .arg("--install-patches")
        .output()
        .unwrap();
    assert!(
        apply.status.success(),
        "{}",
        String::from_utf8_lossy(&apply.stderr)
    );
    let patched = fs::read_to_string(&config).unwrap();
    assert!(patched.starts_with("Host other\n  ProxyJump campus\n"));
    assert!(patched.contains("/usr/bin/nc -q 0 -w 30 127.0.0.1 2207"));
    let entries = fs::read_dir(home.path().join(".ssh")).unwrap().count();
    assert_eq!(entries, 2, "one original-config backup");
    let before = fs::metadata(&config).unwrap().modified().unwrap();
    assert!(
        command(home.path())
            .arg("--install-patches")
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(fs::metadata(&config).unwrap().modified().unwrap(), before);
    assert_eq!(
        fs::read_dir(home.path().join(".ssh")).unwrap().count(),
        entries
    );
    assert!(
        command(home.path())
            .arg("--check-install-patches")
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn openssh_effective_configuration_uses_heartbeat_for_the_patched_host_only() {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir(home.path().join(".ssh")).unwrap();
    let config = home.path().join(".ssh/config");
    fs::write(&config, "Host ordinary\n  ServerAliveInterval 45\n  ServerAliveCountMax 7\nHost sample\n  ProxyCommand ssh gateway ssh win wsl.exe -d Debian -- /usr/bin/nc 127.0.0.1 2207\n  ServerAliveInterval 30\n").unwrap();
    let output = command(home.path())
        .arg("--install-patches")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for (host, interval, count) in [("sample", "10", "2"), ("ordinary", "45", "7")] {
        let output = Command::new("ssh")
            .args(["-G", "-F"])
            .arg(&config)
            .args(["-o", "CanonicalizeHostname=no", host])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let effective = String::from_utf8(output.stdout).unwrap();
        assert!(
            effective
                .lines()
                .any(|l| l == format!("serveraliveinterval {interval}"))
        );
        assert!(
            effective
                .lines()
                .any(|l| l == format!("serveralivecountmax {count}"))
        );
        if host == "sample" {
            assert!(effective.contains("/usr/bin/nc -q 0 -w 30 127.0.0.1 2207"));
        }
    }
}

#[test]
fn install_patch_mode_does_not_load_ui_keymaps_or_start_tui() {
    let home = tempfile::tempdir().unwrap();
    let output = command(home.path())
        .env("TERSH_KEYMAP", home.path().join("missing-keymap"))
        .arg("--check-install-patches")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!output.stdout.contains(&0x1b));
}

#[test]
fn include_equals_cannot_hide_defaults_that_override_wsl_heartbeats() {
    for include in [
        "Include=defaults",
        "Include = defaults",
        "iNcLuDe\tdefaults",
    ] {
        let home = tempfile::tempdir().unwrap();
        fs::create_dir(home.path().join(".ssh")).unwrap();
        let config = home.path().join(".ssh/config");
        let original = format!(
            "{include}\nHost sample\n  ProxyCommand ssh win wsl.exe -d Ubuntu -- /usr/bin/nc 127.0.0.1 2222\n"
        );
        fs::write(&config, &original).unwrap();
        let output = command(home.path())
            .arg("--install-patches")
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "must reject cross-file precedence: {include}"
        );
        assert_eq!(fs::read_to_string(config).unwrap(), original);
        assert_eq!(fs::read_dir(home.path().join(".ssh")).unwrap().count(), 1);
    }
}

#[cfg(unix)]
#[test]
fn install_patch_refuses_symlink_without_changing_target() {
    use std::os::unix::fs::symlink;
    let home = tempfile::tempdir().unwrap();
    fs::create_dir(home.path().join(".ssh")).unwrap();
    let target = home.path().join("actual");
    let original = "ProxyCommand ssh win wsl.exe -d Ubuntu -- /usr/bin/nc 127.0.0.1 2222\n";
    fs::write(&target, original).unwrap();
    symlink(&target, home.path().join(".ssh/config")).unwrap();
    let output = command(home.path())
        .arg("--install-patches")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(&target).unwrap(), original);
    assert!(String::from_utf8_lossy(&output.stderr).contains("symlink"));
}
