//! Offline release exercises. Each child runs a disposable copy of the real CLI.
//! A recording curl substitute serves metadata, the sidecar and the archive only.
#![cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "x86_64")
    )
))]

use std::{
    fs,
    os::unix::{fs::PermissionsExt, io::AsRawFd},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};
use tempfile::TempDir;

fn command(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\nset -eu\n{body}\n")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

struct ReleaseFixture {
    root: TempDir,
    binary: PathBuf,
    asset: String,
}
impl ReleaseFixture {
    fn new(version: &str) -> Self {
        let root = tempfile::tempdir().unwrap();
        let p = root.path();
        for dir in ["install", "bin", "contents"] {
            fs::create_dir(p.join(dir)).unwrap();
        }
        let binary = p.join("install/tersh");
        fs::copy(env!("CARGO_BIN_EXE_tersh"), &binary).unwrap();
        let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("macos", "aarch64") => "macos-arm64",
            ("linux", "aarch64") => "linux-arm64",
            ("linux", "x86_64") => "linux-x86_64",
            _ => panic!("unsupported release platform"),
        };
        let asset = format!("tersh-v{version}-{platform}.tar.gz");
        command(
            &p.join("contents/tersh"),
            &format!("echo 'tersh {version}'"),
        );
        command(
            &p.join("contents/install.sh"),
            "touch \"$RELEASE_MOCK/installer-ran\"",
        );
        assert!(
            Command::new("tar")
                .arg("-czf")
                .arg(p.join(&asset))
                .arg("-C")
                .arg(p.join("contents"))
                .args(["tersh", "install.sh"])
                .status()
                .unwrap()
                .success()
        );
        let (hash_program, hash_args): (_, &[&str]) = if cfg!(target_os = "macos") {
            ("shasum", &["-a", "256"])
        } else {
            ("sha256sum", &[])
        };
        let hash = Command::new(hash_program)
            .args(hash_args)
            .arg(p.join(&asset))
            .output()
            .unwrap();
        let hash = std::str::from_utf8(&hash.stdout)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap();
        fs::write(
            p.join(format!("{asset}.sha256")),
            format!("{hash}  {asset}\n"),
        )
        .unwrap();
        fs::write(p.join("metadata"),serde_json::to_vec(&serde_json::json!({
            "tag_name":format!("v{version}"),"draft":false,"prerelease":false,
            "assets":[{"name":asset,"digest":format!("sha256:{hash}")},{"name":format!("{asset}.sha256")}]
        })).unwrap()).unwrap();
        command(
            &p.join("bin/curl"),
            r#"
while [ "$#" -gt 1 ]; do
    case "$1" in --output) dest=$2; shift;; esac
    shift
done
url=$1
printf '%s\n' "$url" >>"$RELEASE_MOCK/requests"
case "$url" in
    https://api.github.com/repos/QiushanHuang/Tersh/releases/latest) src=metadata ;;
    "https://github.com/QiushanHuang/Tersh/releases/download/$MOCK_TAG/$MOCK_ASSET"|\
    "https://github.com/QiushanHuang/Tersh/releases/download/$MOCK_TAG/$MOCK_ASSET.sha256") src=${url##*/} ;;
    *) exit 39 ;;
esac
cp "$RELEASE_MOCK/$src" "$dest"
"#,
        );
        Self {
            root,
            binary,
            asset,
        }
    }
    fn run(&self, args: &[&str]) -> Output {
        use std::os::unix::process::CommandExt;
        let mut child = Command::new(&self.binary);
        // Never open the developer's controlling terminal during the noninteractive tests.
        unsafe {
            child.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        child
            .args(args)
            .stdin(Stdio::null())
            .env("RELEASE_MOCK", self.root.path())
            .env("MOCK_ASSET", &self.asset)
            .env("MOCK_TAG", self.asset.split('-').nth(1).unwrap())
            .env(
                "PATH",
                format!(
                    "{}:/usr/bin:/bin:/usr/sbin:/sbin",
                    self.root.path().join("bin").display()
                ),
            )
            .output()
            .unwrap()
    }
    fn requests(&self) -> usize {
        fs::read_to_string(self.root.path().join("requests"))
            .unwrap_or_default()
            .lines()
            .count()
    }
}
#[test]
fn check_is_metadata_only_and_explicit_update_replaces_exact_running_copy() {
    let f = ReleaseFixture::new("90.0.1");
    let check = f.run(&["--check-update"]);
    assert!(check.status.success(), "{check:?}");
    assert_eq!(f.requests(), 1, "check must not fetch assets");
    let original = fs::read(&f.binary).unwrap();
    let declined = f.run(&["--update"]);
    assert!(
        !declined.status.success() || f.requests() <= 2,
        "noninteractive update needs explicit consent"
    );
    assert_eq!(fs::read(&f.binary).unwrap(), original);
    let before = f.requests();
    let updated = f.run(&["--update", "--yes"]);
    assert!(updated.status.success(), "{updated:?}");
    assert_eq!(f.requests(), before + 3);
    assert_eq!(f.run(&["--version"]).stdout, b"tersh 90.0.1\n");
    assert!(!f.root.path().join("installer-ran").exists());
}
#[test]
fn digest_disagreement_or_corrupt_archive_preserves_installed_program() {
    let f = ReleaseFixture::new("90.0.1");
    let old = fs::metadata(&f.binary).unwrap().len();
    let checksum = f.root.path().join(format!("{}.sha256", f.asset));
    let valid = fs::read(&checksum).unwrap();
    fs::write(&checksum, format!("{}  {}\n", "f".repeat(64), f.asset)).unwrap();
    assert!(!f.run(&["--update", "--yes"]).status.success());
    assert_eq!(
        f.requests(),
        2,
        "GitHub digest contradicts sidecar before fetching archive"
    );
    fs::write(checksum, valid).unwrap();
    fs::write(f.root.path().join(&f.asset), b"corrupt archive").unwrap();
    assert!(!f.run(&["--update", "--yes"]).status.success());
    assert_eq!(fs::metadata(&f.binary).unwrap().len(), old);
    assert!(f.run(&["--version"]).status.success());
}
#[test]
fn competing_updater_does_not_fetch_assets_or_replace_the_program() {
    let f = ReleaseFixture::new("90.0.1");
    let directory = fs::File::open(f.binary.parent().unwrap()).unwrap();
    assert_eq!(
        unsafe { libc::flock(directory.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
        0
    );
    let result = f.run(&["--update", "--yes"]);
    assert!(!result.status.success(), "{result:?}");
    assert_eq!(f.requests(), 1);
    drop(directory);
    assert!(f.run(&["--update", "--yes"]).status.success());
}

#[test]
fn startup_checks_both_tuis_and_only_installs_after_explicit_yes() {
    for (mode, answer) in [("files", ""), ("cluster", "y")] {
        let f = ReleaseFixture::new("90.0.1");
        let root = f.root.path();
        let output = Command::new("python3")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/update_startup_pty.py"
            ))
            .arg(&f.binary)
            .args([mode, answer])
            .env("RELEASE_MOCK", root)
            .env("MOCK_ASSET", &f.asset)
            .env("MOCK_TAG", "v90.0.1")
            .env(
                "PATH",
                format!(
                    "{}:/usr/bin:/bin:/usr/sbin:/sbin",
                    root.join("bin").display()
                ),
            )
            .env("TERM", "xterm-256color")
            .env("TERSH_UPDATE_CHECK", "on")
            .env("TERSH_PLACES", "off")
            .env("XDG_CONFIG_HOME", root.join("config"))
            .env("XDG_STATE_HOME", root.join("state"))
            .env_remove("TERSH_KEYMAP")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}: {}",
            mode,
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(f.run(&["--version"]).stdout)
                .unwrap()
                .trim(),
            format!(
                "tersh {}",
                if answer == "y" {
                    "90.0.1"
                } else {
                    env!("CARGO_PKG_VERSION")
                }
            )
        );
    }
}

#[test]
fn older_release_is_metadata_only_and_verified_but_wrong_binary_is_rejected() {
    let old = ReleaseFixture::new("0.9.1");
    assert!(old.run(&["--update", "--yes"]).status.success());
    assert_eq!(old.requests(), 1);
    let f = ReleaseFixture::new("90.0.1");
    let metadata = f.root.path().join("metadata");
    // Relabel an otherwise authentic, checksummed package as a different tag.
    let archive = f.root.path().join(&f.asset);
    let old_name = f.asset.clone();
    let mut f = f;
    f.asset = f.asset.replace("90.0.1", "90.0.2");
    fs::write(
        &metadata,
        fs::read_to_string(&metadata)
            .unwrap()
            .replace("90.0.1", "90.0.2"),
    )
    .unwrap();
    fs::rename(archive, f.root.path().join(&f.asset)).unwrap();
    fs::write(
        f.root.path().join(format!("{}.sha256", f.asset)),
        fs::read_to_string(f.root.path().join(format!("{old_name}.sha256")))
            .unwrap()
            .replace("90.0.1", "90.0.2"),
    )
    .unwrap();
    assert!(!f.run(&["--update", "--yes"]).status.success());
    assert_eq!(f.requests(), 3);
    assert_eq!(
        String::from_utf8(f.run(&["--version"]).stdout)
            .unwrap()
            .trim(),
        format!("tersh {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn bootstrap_reuses_the_packaged_installer_and_then_the_builtin_updater() {
    let output = Command::new("python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/update_bootstrap.py"
        ))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
