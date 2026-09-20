//! Persistent, no-clobber recovery for managed `.tersh-trash` entries.
//!
//! Receipts are written before the rename: interruption may leave a stale receipt,
//! but never an unrecorded successful move. Legacy entries are never guessed.
use crate::fs_ops::{
    DeleteDecision, FileIdentity, capture_path_identity, delete_identity, ensure_path_identity,
    ensure_same_object, guard_delete_target, path_exists_no_follow, prepare_trash_dir,
    rename_no_replace, unique_suffix,
};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone)]
pub struct TrashEntry {
    pub receipt_path: PathBuf,
    pub original_path: PathBuf,
    pub trashed_path: PathBuf,
    pub deleted_at: u64,
}

#[derive(Debug, Clone, Default)]
pub struct TrashScan {
    pub entries: Vec<TrashEntry>,
    /// First 20 invalid receipt diagnostics, in directory traversal order.
    pub warnings: Vec<String>,
    pub warning_count: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    version: u8,
    id: String,
    work_root: PathBuf,
    original_path: PathBuf,
    original_parent: FileIdentity,
    payload_identity: FileIdentity,
    deleted_at: u64,
}

pub(crate) fn trash_path(path: &Path, work_root: &Path) -> Result<DeleteDecision> {
    let guarded = guard_delete_target(path, work_root)?;
    let original = delete_identity(path)?;
    // serde's Path representation requires UTF-8. Reject before moving anything.
    if original.to_str().is_none() || work_root.to_str().is_none() {
        bail!("trash recovery metadata requires a UTF-8 path; source was not moved");
    }
    let root = work_root.canonicalize()?;
    let trash = prepare_trash_dir(&root)?;
    let receipts = prepare_receipts(&trash)?;
    let trash_identity = capture_path_identity(&trash)?;
    let receipts_identity = capture_path_identity(&receipts)?;
    let original_parent = capture_path_identity(original.parent().context("missing parent")?)?;
    for _ in 0..100 {
        let id = format!("{}-{}", std::process::id(), unique_suffix());
        let receipt_path = receipts.join(format!("{id}.json"));
        let payload = trash.join(&id);
        if path_exists_no_follow(&payload)? {
            continue;
        }
        let receipt = Receipt {
            version: 1,
            id,
            work_root: root.clone(),
            original_path: original.clone(),
            original_parent: original_parent.clone(),
            payload_identity: guarded.identity.clone(),
            deleted_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        let bytes = serde_json::to_vec(&receipt)?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        let mut output = match options.open(&receipt_path) {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err).context("failed to create trash receipt"),
        };
        let result = (|| {
            output.write_all(&bytes)?;
            output.sync_all()?;
            drop(output);
            ensure_same_object(&trash, &trash_identity)?;
            ensure_same_object(&receipts, &receipts_identity)?;
            ensure_same_object(original.parent().unwrap(), &original_parent)?;
            ensure_path_identity(path, &guarded.identity)?;
            rename_no_replace(path, &payload).context("failed to move source to trash")?;
            Ok(())
        })();
        if let Err(err) = result {
            let _ = fs::remove_file(&receipt_path);
            return Err(err);
        }
        return Ok(DeleteDecision::MovedToTrash {
            from: path.to_path_buf(),
            to: payload,
        });
    }
    bail!("could not allocate a unique trash receipt")
}

/// Lists valid managed entries newest first, rejecting the entire listing if
/// any managed receipt is malformed or stale. Use `scan_trash` for a recovery UI
/// that displays independently verified entries alongside bounded diagnostics.
pub fn list_trash(work_root: &Path) -> Result<Vec<TrashEntry>> {
    let scan = scan_trash(work_root)?;
    if scan.warning_count > 0 {
        bail!(
            "{} invalid trash receipt(s): {}",
            scan.warning_count,
            scan.warnings
                .first()
                .map(String::as_str)
                .unwrap_or("unreadable receipt")
        );
    }
    Ok(scan.entries)
}

/// Scans every managed receipt independently. Invalid entries are never offered
/// for recovery, while valid entries remain usable. Directory-level safety
/// failures abort the scan. At most 20 warning strings are retained.
pub fn scan_trash(work_root: &Path) -> Result<TrashScan> {
    let root = work_root.canonicalize()?;
    let trash = root.join(".tersh-trash");
    if !path_exists_no_follow(&trash)? {
        return Ok(TrashScan::default());
    }
    check_directory(&trash, false)?;
    let receipts = trash.join(".receipts");
    if !path_exists_no_follow(&receipts)? {
        return Ok(TrashScan::default());
    }
    check_directory(&receipts, true)?;
    let mut scan = TrashScan::default();
    for entry in fs::read_dir(&receipts)? {
        let path = match entry {
            Ok(entry) => entry.path(),
            Err(error) => {
                scan.warn(format!("unreadable trash receipt entry: {error}"));
                continue;
            }
        };
        if path.extension().is_some_and(|ext| ext == "json") {
            match read_receipt(&path, &root) {
                Ok((receipt, payload)) => scan.entries.push(TrashEntry {
                    receipt_path: path,
                    original_path: receipt.original_path,
                    trashed_path: payload,
                    deleted_at: receipt.deleted_at,
                }),
                Err(error) => scan.warn(format!("{}: {error:#}", path.display())),
            }
        }
    }
    // If another process changed the storage directories mid-scan, do not
    // present the partially collected entries as a trusted recovery list.
    check_directory(&trash, false)?;
    check_directory(&receipts, true)?;
    scan.entries.sort_by(|a, b| {
        b.deleted_at
            .cmp(&a.deleted_at)
            .then_with(|| b.receipt_path.cmp(&a.receipt_path))
    });
    Ok(scan)
}

impl TrashScan {
    fn warn(&mut self, message: String) {
        self.warning_count = self.warning_count.saturating_add(1);
        if self.warnings.len() < 20 {
            self.warnings.push(message);
        }
    }
}

/// Restores one persisted receipt. Parent identity and payload identity must still
/// match. Existing destinations (including dangling links) are never overwritten.
pub fn restore_entry(receipt_path: &Path, work_root: &Path) -> Result<PathBuf> {
    let root = work_root.canonicalize()?;
    let (receipt, payload) = read_receipt(receipt_path, &root)?;
    let receipt_identity = capture_path_identity(receipt_path)?;
    let parent = receipt
        .original_path
        .parent()
        .context("invalid original parent")?;
    ensure_same_object(parent, &receipt.original_parent)?;
    ensure_path_identity(&payload, &receipt.payload_identity)?;
    ensure_path_identity(receipt_path, &receipt_identity)?;
    rename_no_replace(&payload, &receipt.original_path).with_context(|| {
        format!(
            "could not restore to {}; destination must be absent",
            receipt.original_path.display()
        )
    })?;
    // A failed cleanup leaves a stale receipt which fails closed on the next read;
    // recovery itself has succeeded and should not be offered for retry.
    let _ = fs::remove_file(receipt_path);
    Ok(receipt.original_path)
}

fn read_receipt(receipt_path: &Path, root: &Path) -> Result<(Receipt, PathBuf)> {
    let trash = root.join(".tersh-trash");
    let receipts = trash.join(".receipts");
    check_directory(&trash, false)?;
    check_directory(&receipts, true)?;
    if receipt_path.parent() != Some(receipts.as_path()) {
        bail!("receipt is outside the managed trash metadata directory");
    }
    let metadata = fs::symlink_metadata(receipt_path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 64 * 1024 {
        bail!("unsafe trash receipt: {}", receipt_path.display());
    }
    check_private_owner(&metadata)?;
    let identity = capture_path_identity(receipt_path)?;
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let mut input = options.open(receipt_path)?;
    let opened = input.metadata()?;
    if opened.len() > 64 * 1024 {
        bail!("trash receipt exceeds size limit");
    }
    ensure_path_identity(receipt_path, &identity)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut input)
        .take(64 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 64 * 1024 {
        bail!("trash receipt exceeds size limit");
    }
    let receipt: Receipt = serde_json::from_slice(&bytes).context("invalid trash receipt")?;
    ensure_path_identity(receipt_path, &identity)?;
    if receipt.version != 1
        || receipt.work_root != root
        || receipt.id.is_empty()
        || !receipt.id.bytes().all(|b| b.is_ascii_digit() || b == b'-')
        || receipt_path.file_name().and_then(|n| n.to_str())
            != Some(format!("{}.json", receipt.id).as_str())
    {
        bail!("trash receipt does not match its root or entry identity");
    }
    let original = &receipt.original_path;
    if !original.is_absolute()
        || original.to_str().is_none()
        || original
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
        || original.file_name().is_none()
        || original == root
        || original.starts_with(&trash)
    {
        bail!("unsafe original location in trash receipt");
    }
    let parent = original.parent().context("missing original parent")?;
    if parent.canonicalize()? != parent {
        bail!("original parent is no longer the recorded canonical location");
    }
    ensure_same_object(parent, &receipt.original_parent)?;
    if std::env::var_os("HOME")
        .map(PathBuf::from)
        .and_then(|p| p.canonicalize().ok())
        .as_deref()
        == Some(original)
    {
        bail!("refusing to restore over a home directory");
    }
    let payload = trash.join(&receipt.id);
    ensure_path_identity(&payload, &receipt.payload_identity)
        .context("trash payload is missing or changed; refusing recovery")?;
    Ok((receipt, payload))
}

fn prepare_receipts(trash: &Path) -> Result<PathBuf> {
    let receipts = trash.join(".receipts");
    if !path_exists_no_follow(&receipts)? {
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder
            .create(&receipts)
            .context("failed to create private trash metadata directory")?;
    }
    check_directory(&receipts, true)?;
    Ok(receipts)
}

fn check_directory(path: &Path, private: bool) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        bail!("refusing to use unsafe trash directory {}", path.display());
    }
    if private {
        check_private_owner(&metadata)?;
    }
    Ok(())
}

fn check_private_owner(metadata: &fs::Metadata) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o077 != 0 {
            bail!("trash metadata must be private and owned by the current user");
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;
    Ok(())
}
