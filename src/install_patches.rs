//! Installation-only protection for the WSL SSH/nc lifecycle. Normal TUI startup
//! never writes SSH policy or connects to a remote machine.
mod config;

use anyhow::{Context, Result, anyhow, bail, ensure};
use std::{
    fs::{self, File, OpenOptions, Permissions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const ENDPOINT: &str = "/etc/ssh/tersh_sshd_config";
const BACKUP_SUFFIX: &str = ".tersh-before-wsl-lifecycle";

pub fn run(apply: bool, ssh_config: Option<&Path>) -> Result<()> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let client = ssh_config
        .map(Path::to_path_buf)
        .or_else(|| home.map(|p| p.join(".ssh/config")));
    let endpoint = is_wsl().then(|| PathBuf::from(ENDPOINT));
    let mut changes = Vec::new();
    if let Some(path) = client
        && let Some(original) = Original::read(&path)?
    {
        let patch = config::patch_client_config(&original.content)?;
        println!(
            "{}: {} recognized WSL route(s), {} need patching",
            path.display(),
            patch.applicable,
            patch.changed
        );
        if config::has_client_includes(&original.content) {
            ensure!(
                patch.applicable == 0,
                "{} combines WSL routes and Include; included defaults can override lifecycle limits. Resolve/flatten this custom configuration before patching",
                path.display()
            );
            println!(
                "Included SSH files are outside this file's check; use --patch-ssh-config FILE for each WSL route file."
            );
        }
        if patch.content != original.content {
            changes.push(Change {
                original,
                content: patch.content,
                endpoint: false,
            });
        }
    }
    if let Some(path) = endpoint {
        if let Some(original) = Original::read(&path)? {
            let content = config::patch_endpoint_config(&original.content)?;
            if content != original.content {
                changes.push(Change {
                    original,
                    content,
                    endpoint: true,
                });
            }
        } else {
            println!(
                "No Tersh WSL SSH endpoint found; no SSH service is created. Configure a Tersh endpoint before running --install-patches again."
            );
        }
    }
    if changes.is_empty() {
        println!("No changes needed in the inspected files.");
        return Ok(());
    }
    if !apply {
        bail!(
            "{} file(s) need WSL lifecycle protection; run tersh --install-patches{}",
            changes.len(),
            ssh_config
                .map(|p| format!(" --patch-ssh-config {}", p.display()))
                .unwrap_or_default()
        );
    }
    apply_changes(&changes, &SystemEndpoint)
        .context("WSL installation patch incomplete; resolve the reported error/permissions, then rerun tersh --install-patches")?;
    for change in &changes {
        println!(
            "Patched {}; backup: {}",
            change.original.path.display(),
            change.original.backup_path().display()
        );
    }
    Ok(())
}

fn is_wsl() -> bool {
    cfg!(target_os = "linux")
        && fs::read_to_string("/proc/sys/kernel/osrelease")
            .is_ok_and(|s| s.to_ascii_lowercase().contains("microsoft"))
}

struct Original {
    path: PathBuf,
    content: String,
    permissions: Permissions,
    _file: File,
}

impl Original {
    fn read(path: &Path) -> Result<Option<Self>> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e).with_context(|| format!("inspect {}", path.display())),
        };
        ensure!(
            !metadata.file_type().is_symlink(),
            "refusing symlink {}; select its real path with --patch-ssh-config",
            path.display()
        );
        ensure!(
            metadata.is_file(),
            "not a regular configuration file: {}",
            path.display()
        );
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let mut file = options
            .open(path)
            .with_context(|| format!("read {}", path.display()))?;
        ensure!(
            file.metadata()?.is_file(),
            "not a regular configuration file"
        );
        let mut content = String::new();
        Read::by_ref(&mut file)
            .take(1024 * 1024 + 1)
            .read_to_string(&mut content)?;
        ensure!(content.len() <= 1024 * 1024, "configuration exceeds 1 MiB");
        Ok(Some(Self {
            path: path.to_path_buf(),
            content,
            permissions: file.metadata()?.permissions(),
            _file: file,
        }))
    }

    fn backup_path(&self) -> PathBuf {
        let mut name = self.path.as_os_str().to_os_string();
        name.push(BACKUP_SUFFIX);
        PathBuf::from(name)
    }

    fn unchanged(&self) -> Result<()> {
        let now = Self::read(&self.path)?.context("configuration disappeared during patch")?;
        ensure!(
            now.content == self.content,
            "{} changed during patch; retry after editing finishes",
            self.path.display()
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            let a = self._file.metadata()?;
            let b = now._file.metadata()?;
            ensure!(
                a.dev() == b.dev() && a.ino() == b.ino() && b.mode() == self.permissions.mode(),
                "configuration identity/permissions changed during patch"
            );
        }
        Ok(())
    }

    fn backup(&self) -> Result<()> {
        let path = self.backup_path();
        if let Ok(m) = fs::symlink_metadata(&path) {
            ensure!(
                m.is_file() && !m.file_type().is_symlink(),
                "backup is not a regular file: {}",
                path.display()
            );
            return Ok(());
        }
        // Publish only a complete, synced backup. An interrupted installer must
        // never leave a half-written file that a later run treats as a backup.
        let staged = Staged::new(self, &self.content)?;
        fs::hard_link(&staged.path, &path)
            .with_context(|| format!("publish backup {}", path.display()))?;
        Ok(())
    }
}

struct Change {
    original: Original,
    content: String,
    endpoint: bool,
}

struct Staged {
    path: PathBuf,
}
impl Staged {
    fn new(original: &Original, content: &str) -> Result<Self> {
        let parent = original
            .path
            .parent()
            .context("config has no parent directory")?;
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = parent.join(format!(".tersh-wsl-patch-{}-{nonce}", std::process::id()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        let mut file = options.open(&path).with_context(|| {
            format!(
                "stage {}; run this patch with write permission for its directory",
                original.path.display()
            )
        })?;
        let staged = Self { path };
        file.write_all(content.as_bytes())?;
        file.set_permissions(original.permissions.clone())?;
        file.sync_all()?;
        Ok(staged)
    }
}
impl Drop for Staged {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

trait EndpointRuntime {
    fn validate(&self, candidate: &Path) -> Result<()>;
    fn reload(&self) -> Result<()>;
}

fn apply_changes(changes: &[Change], runtime: &dyn EndpointRuntime) -> Result<()> {
    let staged = changes
        .iter()
        .map(|c| Staged::new(&c.original, &c.content))
        .collect::<Result<Vec<_>>>()?;
    for (change, stage) in changes.iter().zip(&staged) {
        if change.endpoint {
            runtime.validate(&stage.path)?;
        }
        change.original.unchanged()?;
    }
    for change in changes {
        change.original.backup()?;
    }
    let mut committed = 0;
    let result: Result<()> = (|| {
        for (change, stage) in changes.iter().zip(&staged) {
            change.original.unchanged()?;
            fs::rename(&stage.path, &change.original.path)?;
            committed += 1;
        }
        if changes.iter().any(|c| c.endpoint) {
            runtime.reload()?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        let mut failures = Vec::new();
        let mut endpoint_restored = false;
        for change in changes[..committed].iter().rev() {
            let rollback = (|| -> Result<()> {
                let now = Original::read(&change.original.path)?
                    .context("patched configuration disappeared")?;
                ensure!(
                    now.content == change.content,
                    "configuration edited after patch; refusing to overwrite that edit"
                );
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    ensure!(
                        now.permissions.mode() == change.original.permissions.mode(),
                        "configuration permissions edited after patch; refusing to overwrite that edit"
                    );
                }
                let restore = Staged::new(&change.original, &change.original.content)?;
                fs::rename(&restore.path, &change.original.path)?;
                Ok(())
            })();
            match rollback {
                Ok(()) => endpoint_restored |= change.endpoint,
                Err(e) => failures.push(format!("{}: {e:#}", change.original.path.display())),
            }
        }
        if endpoint_restored && let Err(e) = runtime.reload() {
            failures.push(format!("reload restored endpoint: {e:#}"));
        }
        bail!(
            "{error:#}; rollback {}",
            if failures.is_empty() {
                "completed".to_string()
            } else {
                failures.join("; ")
            }
        );
    }
    Ok(())
}

struct SystemEndpoint;
impl EndpointRuntime for SystemEndpoint {
    fn validate(&self, candidate: &Path) -> Result<()> {
        let out = bounded_command(
            Command::new("/usr/sbin/sshd")
                .args(["-t", "-f"])
                .arg(candidate),
        )?;
        ensure!(
            out.status.success(),
            "sshd rejected candidate: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        Ok(())
    }
    fn reload(&self) -> Result<()> {
        let out = bounded_command(Command::new("systemctl").args([
            "--no-ask-password",
            "show",
            "tersh-sshd.service",
            "--property=MainPID",
            "--value",
        ]))?;
        ensure!(
            out.status.success(),
            "cannot inspect dedicated tersh-sshd.service: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let pid: u32 = String::from_utf8_lossy(&out.stdout)
            .trim()
            .parse()
            .context("invalid dedicated SSH listener PID")?;
        if pid == 0 {
            println!(
                "Tersh SSH endpoint is inactive; protected configuration will load at its next start."
            );
            return Ok(());
        }
        let cmdline = fs::read(format!("/proc/{pid}/cmdline"))?;
        ensure!(
            fs::canonicalize(format!("/proc/{pid}/exe"))? == fs::canonicalize("/usr/sbin/sshd")?,
            "dedicated service main process is not the expected sshd executable"
        );
        ensure!(
            listener_has_endpoint_config(&cmdline),
            "dedicated SSH service does not use {ENDPOINT}; refusing to signal it"
        );
        let out = bounded_command(Command::new("systemctl").args([
            "--no-ask-password",
            "kill",
            "--kill-whom=main",
            "--signal=HUP",
            "tersh-sshd.service",
        ]))?;
        ensure!(
            out.status.success(),
            "cannot reload dedicated SSH listener: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        Ok(())
    }
}

fn listener_has_endpoint_config(cmdline: &[u8]) -> bool {
    let args = cmdline.split(|b| *b == 0).collect::<Vec<_>>();
    if args
        .windows(2)
        .any(|a| a[0] == b"-f" && a[1] == ENDPOINT.as_bytes())
    {
        return true;
    }
    // OpenSSH rewrites argv into one process-title string after listening.
    // Accept that exact listener form, together with the executable identity
    // check above; do not accept a substring of some other configuration path.
    let Ok(title) = std::str::from_utf8(args.first().copied().unwrap_or_default()) else {
        return false;
    };
    let words = title.split_ascii_whitespace().collect::<Vec<_>>();
    words.first() == Some(&"sshd:")
        && words.get(1) == Some(&"/usr/sbin/sshd")
        && words.contains(&"[listener]")
        && words.windows(2).any(|a| a == ["-f", ENDPOINT])
}

fn bounded_command(command: &mut Command) -> Result<Output> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("start SSH policy validation/reload tool")?;
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() >= Duration::from_secs(10) {
            let _ = child.kill();
            let _ = child.wait();
            bail!("SSH policy validation/reload exceeded 10 seconds");
        }
        thread::sleep(Duration::from_millis(10));
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("missing tool stdout"))?
        .take(65536)
        .read_to_end(&mut stdout)?;
    child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("missing tool stderr"))?
        .take(65536)
        .read_to_end(&mut stderr)?;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn recognizes_openssh_listener_process_titles_and_rejects_other_configs() {
        assert!(listener_has_endpoint_config(
            b"/usr/sbin/sshd\0-D\0-f\0/etc/ssh/tersh_sshd_config\0"
        ));
        assert!(listener_has_endpoint_config(b"sshd: /usr/sbin/sshd -D -f /etc/ssh/tersh_sshd_config [listener] 0 of 10-100 startups\0"));
        assert!(!listener_has_endpoint_config(
            b"sshd: /usr/sbin/sshd -D -f /etc/ssh/other_config [listener] 0 of 10-100 startups\0"
        ));
        assert!(!listener_has_endpoint_config(
            b"sshd: /usr/sbin/sshd -D -f /etc/ssh/tersh_sshd_config.extra [listener]\0"
        ));
        assert!(!listener_has_endpoint_config(
            b"unrelated -f /etc/ssh/tersh_sshd_config [listener]\0"
        ));
    }

    struct Runtime {
        reject: bool,
        reload_fails: bool,
        reloads: Cell<usize>,
    }
    impl EndpointRuntime for Runtime {
        fn validate(&self, candidate: &Path) -> Result<()> {
            ensure!(
                fs::read_to_string(candidate)?.contains("ClientAliveInterval 10"),
                "candidate missing heartbeat"
            );
            ensure!(!self.reject, "synthetic sshd validation rejection");
            Ok(())
        }
        fn reload(&self) -> Result<()> {
            self.reloads.set(self.reloads.get() + 1);
            ensure!(
                !self.reload_fails || self.reloads.get() > 1,
                "synthetic listener reload failure"
            );
            Ok(())
        }
    }
    fn endpoint(root: &Path) -> Change {
        let path = root.join("tersh_sshd_config");
        fs::write(&path, "Port 2222\nClientAliveInterval 0\n").unwrap();
        let original = Original::read(&path).unwrap().unwrap();
        let content = config::patch_endpoint_config(&original.content).unwrap();
        Change {
            original,
            content,
            endpoint: true,
        }
    }
    #[test]
    fn rejected_endpoint_candidate_never_replaces_config_or_creates_backup() {
        let root = tempfile::tempdir().unwrap();
        let change = endpoint(root.path());
        let runtime = Runtime {
            reject: true,
            reload_fails: false,
            reloads: Cell::new(0),
        };
        let path = change.original.path.clone();
        assert!(apply_changes(&[change], &runtime).is_err());
        assert_eq!(
            fs::read_to_string(path).unwrap(),
            "Port 2222\nClientAliveInterval 0\n"
        );
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
        assert_eq!(runtime.reloads.get(), 0);
    }
    #[test]
    fn listener_reload_failure_restores_original_and_reloads_restored_policy() {
        let root = tempfile::tempdir().unwrap();
        let change = endpoint(root.path());
        let path = change.original.path.clone();
        let runtime = Runtime {
            reject: false,
            reload_fails: true,
            reloads: Cell::new(0),
        };
        assert!(apply_changes(&[change], &runtime).is_err());
        assert_eq!(
            fs::read_to_string(path).unwrap(),
            "Port 2222\nClientAliveInterval 0\n"
        );
        assert_eq!(runtime.reloads.get(), 2);
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
    }
    #[test]
    fn concurrent_configuration_edit_is_not_overwritten() {
        let root = tempfile::tempdir().unwrap();
        let change = endpoint(root.path());
        fs::write(&change.original.path, "Port 2345\n").unwrap();
        let runtime = Runtime {
            reject: false,
            reload_fails: false,
            reloads: Cell::new(0),
        };
        assert!(apply_changes(&[change], &runtime).is_err());
        assert_eq!(
            fs::read_to_string(root.path().join("tersh_sshd_config")).unwrap(),
            "Port 2345\n"
        );
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
    }
    #[cfg(unix)]
    #[test]
    fn concurrent_permission_tightening_is_not_overwritten() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config");
        fs::write(&path, "Host sample\n").unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o644)).unwrap();
        let original = Original::read(&path).unwrap().unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o600)).unwrap();
        assert!(
            original.unchanged().is_err(),
            "must compare with the captured permission snapshot"
        );
    }
    #[cfg(unix)]
    #[test]
    fn failed_reload_does_not_undo_a_concurrent_permission_edit() {
        use std::os::unix::fs::PermissionsExt;
        struct TightenOnReload {
            path: PathBuf,
            calls: Cell<usize>,
        }
        impl EndpointRuntime for TightenOnReload {
            fn validate(&self, _: &Path) -> Result<()> {
                Ok(())
            }
            fn reload(&self) -> Result<()> {
                self.calls.set(self.calls.get() + 1);
                fs::set_permissions(&self.path, Permissions::from_mode(0o600))?;
                bail!("listener reload failed after concurrent chmod");
            }
        }
        let root = tempfile::tempdir().unwrap();
        let change = endpoint(root.path());
        fs::set_permissions(&change.original.path, Permissions::from_mode(0o644)).unwrap();
        let original = Original::read(&change.original.path).unwrap().unwrap();
        let path = original.path.clone();
        let change = Change { original, ..change };
        let runtime = TightenOnReload {
            path: path.clone(),
            calls: Cell::new(0),
        };
        assert!(apply_changes(&[change], &runtime).is_err());
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            runtime.calls.get(),
            1,
            "do not reload a config whose concurrent edit prevented rollback"
        );
    }
    #[test]
    fn successful_endpoint_patch_preserves_original_permissions_and_backup_bytes() {
        let root = tempfile::tempdir().unwrap();
        let change = endpoint(root.path());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&change.original.path, Permissions::from_mode(0o640)).unwrap();
        }
        // Re-read the permission snapshot after setting the test fixture mode.
        let original = Original::read(&change.original.path).unwrap().unwrap();
        let backup = original.backup_path();
        let path = original.path.clone();
        let change = Change { original, ..change };
        let runtime = Runtime {
            reject: false,
            reload_fails: false,
            reloads: Cell::new(0),
        };
        apply_changes(&[change], &runtime).unwrap();
        assert!(
            fs::read_to_string(&path)
                .unwrap()
                .contains("ClientAliveInterval 10")
        );
        assert_eq!(
            fs::read_to_string(backup).unwrap(),
            "Port 2222\nClientAliveInterval 0\n"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o640
            );
        }
        assert_eq!(runtime.reloads.get(), 1);
    }
}
