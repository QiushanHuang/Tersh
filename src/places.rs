//! Small, explicit host/path history. Never discovers or connects to hosts.
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const PLACE_LIMIT: usize = 128;
const BYTE_LIMIT: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Place {
    pub host: String,
    pub identity: String,
    pub path: PathBuf,
    pub pinned: bool,
    pub last_used: u64,
}

#[derive(Debug)]
pub struct Places {
    entries: Vec<Place>,
    file: Option<PathBuf>,
    original: Option<Vec<u8>>,
    enabled: bool,
}

impl Places {
    pub fn memory() -> Self {
        Self {
            entries: Vec::new(),
            file: None,
            original: None,
            enabled: true,
        }
    }
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::memory()
        }
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    pub fn entries(&self) -> &[Place] {
        &self.entries
    }
    pub fn load_default() -> Result<Self> {
        if std::env::var("TERSH_PLACES").is_ok_and(|s| matches!(s.as_str(), "off" | "0")) {
            return Ok(Self::disabled());
        }
        let path = std::env::var_os("TERSH_PLACES_FILE")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("XDG_STATE_HOME")
                    .filter(|s| !s.is_empty())
                    .map(PathBuf::from)
                    .or_else(|| {
                        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state"))
                    })
                    .map(|p| p.join("tersh/places.json"))
            })
            .context("No location for places state; set TERSH_PLACES_FILE or TERSH_PLACES=off")?;
        Self::load(path)
    }
    pub fn load(path: PathBuf) -> Result<Self> {
        let original = read_state(&path)?;
        let entries: Vec<Place> = original
            .as_deref()
            .map(serde_json::from_slice)
            .transpose()
            .context("invalid places state; file was not changed")?
            .unwrap_or_default();
        if entries.len() > PLACE_LIMIT {
            bail!("too many saved places");
        }
        for p in &entries {
            validate(&p.host, &p.identity, &p.path)?;
        }
        Ok(Self {
            entries,
            file: Some(path),
            original,
            enabled: true,
        })
    }
    pub fn recent(&self, host: &str, identity: &str) -> Vec<&Place> {
        let mut entries: Vec<_> = self
            .entries
            .iter()
            .filter(|p| p.host == host && p.identity == identity)
            .collect();
        entries.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then_with(|| b.last_used.cmp(&a.last_used))
                .then_with(|| a.path.cmp(&b.path))
        });
        entries
    }
    pub fn visit(&mut self, host: &str, identity: &str, path: &Path) -> Result<()> {
        self.change(host, identity, path, false)
    }
    pub fn toggle_pin(&mut self, host: &str, identity: &str, path: &Path) -> Result<()> {
        self.change(host, identity, path, true)
    }
    fn change(&mut self, host: &str, identity: &str, path: &Path, pin: bool) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        validate(host, identity, path)?;
        let old = self.entries.clone();
        let now = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .min(u64::MAX as u128) as u64)
            .max(
                self.entries
                    .iter()
                    .map(|p| p.last_used)
                    .max()
                    .unwrap_or(0)
                    .saturating_add(1),
            );
        if let Some(p) = self
            .entries
            .iter_mut()
            .find(|p| p.host == host && p.identity == identity && p.path == path)
        {
            p.last_used = now;
            if pin {
                p.pinned = !p.pinned;
            }
        } else {
            self.entries.push(Place {
                host: host.into(),
                identity: identity.into(),
                path: path.into(),
                pinned: pin,
                last_used: now,
            });
        }
        if self.entries.iter().filter(|p| p.pinned).count() > 64 {
            self.entries = old;
            bail!("At most 64 places can be pinned; unpin a place first");
        }
        self.entries.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then_with(|| b.last_used.cmp(&a.last_used))
        });
        self.entries.truncate(PLACE_LIMIT);
        while serde_json::to_vec(&self.entries)?.len() > BYTE_LIMIT {
            if let Some(i) = self.entries.iter().rposition(|p| !p.pinned) {
                self.entries.remove(i);
            } else {
                self.entries = old;
                bail!("Pinned places exceed the 64 KiB state limit");
            }
        }
        if let Err(e) = self.save() {
            self.entries = old;
            return Err(e);
        }
        Ok(())
    }
    pub fn remove(&mut self, host: &str, identity: &str, path: &Path) -> Result<()> {
        let old = self.entries.clone();
        self.entries
            .retain(|p| !(p.host == host && p.identity == identity && p.path == path));
        if let Err(e) = self.save() {
            self.entries = old;
            return Err(e);
        }
        Ok(())
    }
    pub fn clear_recent(&mut self, host: &str, identity: &str) -> Result<()> {
        let old = self.entries.clone();
        self.entries
            .retain(|p| p.pinned || p.host != host || p.identity != identity);
        if let Err(e) = self.save() {
            self.entries = old;
            return Err(e);
        }
        Ok(())
    }
    fn save(&mut self) -> Result<()> {
        let Some(path) = self.file.as_ref() else {
            return Ok(());
        };
        let parent = path.parent().context("places file has no parent")?;
        fs::create_dir_all(parent)?;
        let _guard = lock_state(&path.with_extension("json.lock"))?;
        let mut opts = OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        (|| {
            if read_state(path)? != self.original {
                bail!("places changed in another session; reopen Tersh before saving");
            }
            let bytes = serde_json::to_vec(&self.entries)?;
            if bytes.len() > BYTE_LIMIT {
                bail!("places state exceeds 64 KiB");
            }
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let temp = parent.join(format!(".places-{}-{nonce}.tmp", std::process::id()));
            let mut output = opts
                .open(&temp)
                .context("cannot create places temporary state")?;
            let write_result = (|| {
                output.write_all(&bytes)?;
                output.sync_all()?;
                drop(output);
                fs::rename(&temp, path)?;
                Ok::<(), anyhow::Error>(())
            })();
            if write_result.is_err() {
                let _ = fs::remove_file(&temp);
            }
            write_result?;
            self.original = Some(bytes);
            Ok(())
        })()
    }
}

// Kernel-managed locks are released on process death. The small regular sidecar
// stays in place, so an abandoned file never disables future saves and another
// session can never bypass an active lock by recreating its pathname.
fn lock_state(path: &Path) -> Result<fs::File> {
    #[cfg(unix)]
    {
        use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(path)?;
        if !file.metadata()?.is_file() {
            bail!("places lock must be a regular file");
        }
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(std::io::Error::last_os_error())
                .context("places state is busy; retry after the other session saves");
        }
        Ok(file)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        bail!("persistent places locking requires a Unix platform")
    }
}

fn validate(host: &str, identity: &str, path: &Path) -> Result<()> {
    if host.is_empty()
        || host.len() > 256
        || identity.is_empty()
        || identity.len() > 1024
        || host.chars().any(char::is_control)
        || identity.chars().any(char::is_control)
    {
        bail!("invalid saved host identity");
    }
    let text = path.to_str().context("saved places require a UTF-8 path")?;
    if !path.is_absolute() || text.len() > 4096 || text.chars().any(char::is_control) {
        bail!("saved places require an absolute path of at most 4096 bytes");
    }
    Ok(())
}

fn read_state(path: &Path) -> Result<Option<Vec<u8>>> {
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > BYTE_LIMIT as u64 {
        bail!("places must be a regular file of at most 64 KiB");
    }
    let mut opts = OpenOptions::new();
    opts.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = opts.open(path)?;
    if !file.metadata()?.is_file() {
        bail!("places must be a regular file");
    }
    let mut data = Vec::new();
    file.take(BYTE_LIMIT as u64 + 1).read_to_end(&mut data)?;
    if data.len() > BYTE_LIMIT {
        bail!("places exceed 64 KiB");
    }
    Ok(Some(data))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rapid_recent_visits_keep_the_newest_location() {
        let mut store = Places::memory();
        for n in 0..140 {
            store
                .visit("local", "local", &PathBuf::from(format!("/work/{n}")))
                .unwrap();
        }
        assert_eq!(store.entries().len(), PLACE_LIMIT);
        assert_eq!(
            store.recent("local", "local")[0].path,
            Path::new("/work/139")
        );
    }
    #[test]
    fn an_abandoned_lock_file_does_not_disable_future_saves() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("places.json");
        fs::write(file.with_extension("json.lock"), b"").unwrap();
        let mut store = Places::load(file).unwrap();
        store.visit("local", "local", Path::new("/work")).unwrap();
        assert_eq!(store.entries().len(), 1);
    }
    #[test]
    #[cfg(unix)]
    fn an_active_lock_cannot_be_bypassed_and_drop_releases_it() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("places.json");
        let guard = lock_state(&file.with_extension("json.lock")).unwrap();
        let mut store = Places::load(file.clone()).unwrap();
        assert!(store.visit("local", "local", Path::new("/work")).is_err());
        assert!(!file.exists());
        drop(guard);
        store.visit("local", "local", Path::new("/work")).unwrap();
        assert!(file.exists());
    }
    #[test]
    fn remembers_pins_without_crossing_host_identity_and_survives_restart() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("places.json");
        let mut store = Places::load(file.clone()).unwrap();
        store
            .visit("host", "alice@old", Path::new("/work"))
            .unwrap();
        store
            .toggle_pin("host", "alice@old", Path::new("/work"))
            .unwrap();
        let store = Places::load(file).unwrap();
        assert!(store.recent("host", "alice@old")[0].pinned);
        assert!(store.recent("host", "alice@new").is_empty());
    }
    #[test]
    fn history_is_bounded_and_clear_keeps_pins() {
        let mut store = Places::memory();
        store
            .toggle_pin("local", "local", Path::new("/pinned"))
            .unwrap();
        for n in 0..160 {
            store
                .visit("local", "local", &PathBuf::from(format!("/work/{n}")))
                .unwrap();
        }
        assert_eq!(store.entries().len(), PLACE_LIMIT);
        store.clear_recent("local", "local").unwrap();
        assert_eq!(store.entries().len(), 1);
        assert_eq!(store.entries()[0].path, Path::new("/pinned"));
    }
    #[test]
    fn competing_sessions_and_symlinks_do_not_overwrite_state() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("places.json");
        let mut first = Places::load(file.clone()).unwrap();
        let mut second = Places::load(file.clone()).unwrap();
        first.visit("local", "local", Path::new("/first")).unwrap();
        assert!(
            second
                .visit("local", "local", Path::new("/second"))
                .is_err()
        );
        assert!(second.entries().is_empty());
        assert_eq!(
            Places::load(file.clone()).unwrap().entries()[0].path,
            Path::new("/first")
        );
        #[cfg(unix)]
        {
            let link = dir.path().join("link");
            std::os::unix::fs::symlink(&file, &link).unwrap();
            assert!(Places::load(link).is_err());
        }
    }
    #[test]
    fn disabled_state_never_records_and_invalid_paths_are_rejected() {
        let mut store = Places::disabled();
        store
            .visit("local", "local", Path::new("/private"))
            .unwrap();
        assert!(store.entries().is_empty());
        assert!(
            Places::memory()
                .visit("local", "local", Path::new("relative"))
                .is_err()
        );
    }
}
