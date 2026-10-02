//! Bounded in-memory task summaries. A history entry never executes a retry.
use crate::{
    fs_core::display_path,
    jobs::{JobFailure, JobKind, JobProgress, JobRequest, JobResult},
};
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::{
    collections::{HashSet, VecDeque},
    fs::OpenOptions,
    io::Write,
    mem::size_of,
    path::{Path, PathBuf},
    time::Duration,
};

pub const HISTORY_ENTRIES: usize = 20;
pub const HISTORY_BYTES: usize = 1024 * 1024;
pub const HISTORY_ENTRY_BYTES: usize = 64 * 1024;
pub const HISTORY_EXPORT_BYTES: usize = 512 * 1024;
const CONTEXT_BYTES: usize = 8 * 1024;
const ITEM_LIMIT: usize = 256;
const DETAIL_BYTES: usize = 1024;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HistoryCounts {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub remaining: usize,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: u64,
    pub kind: JobKind,
    pub destination: Option<PathBuf>,
    pub work_root: PathBuf,
    pub counts: HistoryCounts,
    pub cancelled: bool,
    pub elapsed: Duration,
    pub copied_bytes: u64,
    pub failures: Vec<JobFailure>,
    pub unprocessed: Vec<PathBuf>,
    pub details: Vec<String>,
    pub omitted_details: usize,
    pub omitted_retry_items: usize,
    pub retry_context_retained: bool,
}

#[derive(Debug, Default)]
pub struct JobHistory {
    entries: VecDeque<HistoryEntry>,
    next_id: u64,
}

impl JobHistory {
    /// Store counts from the complete result, then retain bounded details with
    /// unresolved paths taking precedence. The original full request/result is
    /// never retained. Newest entries are first.
    pub fn push(
        &mut self,
        request: &JobRequest,
        result: &JobResult,
        progress: &JobProgress,
        elapsed: Duration,
    ) -> u64 {
        self.next_id = self.next_id.wrapping_add(1).max(1);
        while self.entries.iter().any(|entry| entry.id == self.next_id) {
            self.next_id = self.next_id.wrapping_add(1).max(1);
        }
        let id = self.next_id;
        let entry = HistoryEntry::capture(id, request, result, progress, elapsed);
        self.entries.push_front(entry);
        while self.entries.len() > HISTORY_ENTRIES {
            self.entries.pop_back();
        }
        // Retain all twenty summaries even when details hit the global cap.
        // Drop oldest detail payloads first; exact counts remain visible.
        let mut bytes = self.retained_bytes();
        for entry in self.entries.iter_mut().rev() {
            if bytes <= HISTORY_BYTES {
                break;
            }
            let before = entry.retained_bytes();
            entry.discard_details();
            bytes -= before - entry.retained_bytes();
        }
        debug_assert!(self.retained_bytes() <= HISTORY_BYTES);
        id
    }
    pub fn entries(&self) -> &VecDeque<HistoryEntry> {
        &self.entries
    }
    pub fn get(&self, id: u64) -> Option<&HistoryEntry> {
        self.entries.iter().find(|entry| entry.id == id)
    }
    /// Allocated buffer capacities plus owned struct storage (allocator and
    /// process overhead are outside this retained-data budget).
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self
                .entries
                .iter()
                .map(HistoryEntry::retained_bytes)
                .sum::<usize>()
            + (self.entries.capacity() - self.entries.len()) * size_of::<HistoryEntry>()
    }
}

impl HistoryEntry {
    fn capture(
        id: u64,
        request: &JobRequest,
        result: &JobResult,
        progress: &JobProgress,
        elapsed: Duration,
    ) -> Self {
        let retry_context_retained = request.work_root.as_os_str().len().saturating_add(
            request
                .destination
                .as_ref()
                .map_or(0, |path| path.as_os_str().len()),
        ) <= CONTEXT_BYTES;
        let mut entry = Self {
            id,
            kind: request.kind,
            destination: if retry_context_retained {
                request.destination.clone()
            } else {
                None
            },
            work_root: if retry_context_retained {
                request.work_root.clone()
            } else {
                PathBuf::new()
            },
            counts: HistoryCounts {
                total: request.sources.len(),
                succeeded: result.succeeded.len(),
                failed: result.failed.len(),
                skipped: result.skipped.len(),
                remaining: result.unprocessed.len(),
            },
            cancelled: result.cancelled,
            elapsed,
            copied_bytes: progress.copied_bytes,
            failures: Vec::new(),
            unprocessed: Vec::new(),
            details: Vec::new(),
            omitted_details: 0,
            omitted_retry_items: 0,
            retry_context_retained,
        };
        for failure in result.failed.iter().take(ITEM_LIMIT) {
            let error = bounded_text(&failure.error, DETAIL_BYTES);
            let extra = size_of::<JobFailure>() + failure.path.as_os_str().len() + error.capacity();
            if !entry.fits(extra) {
                break;
            }
            entry.failures.reserve_exact(1);
            entry.failures.push(JobFailure {
                path: failure.path.clone(),
                error,
            });
        }
        for path in result
            .unprocessed
            .iter()
            .take(ITEM_LIMIT.saturating_sub(entry.failures.len()))
        {
            if !entry.fits(size_of::<PathBuf>() + path.as_os_str().len()) {
                break;
            }
            entry.unprocessed.reserve_exact(1);
            entry.unprocessed.push(path.clone());
        }
        entry.omitted_retry_items = result.failed.len() - entry.failures.len()
            + result.unprocessed.len()
            - entry.unprocessed.len();
        let result_count = result
            .succeeded
            .len()
            .saturating_add(result.failed.len())
            .saturating_add(result.skipped.len())
            .saturating_add(result.unprocessed.len());
        // Bound inspection work as well as stored output. A million successes
        // should not require formatting a million paths for twenty summaries.
        for failure in result.failed.iter().take(ITEM_LIMIT) {
            let line = format!(
                "Failed  {}: {}",
                bounded_path(&failure.path),
                bounded_text(&failure.error, DETAIL_BYTES)
            );
            if !entry.add_detail(line) {
                break;
            }
        }
        for (label, paths) in [
            ("Remaining", &result.unprocessed),
            ("Skipped", &result.skipped),
            ("Succeeded", &result.succeeded),
        ] {
            for path in paths
                .iter()
                .take(ITEM_LIMIT.saturating_sub(entry.details.len()))
            {
                if !entry.add_detail(format!("{label}  {}", bounded_path(path))) {
                    break;
                }
            }
        }
        entry.omitted_details = result_count.saturating_sub(entry.details.len());
        // reserve_exact may still receive allocator-rounded capacities. Keep
        // the public hard cap even on allocators with unusual growth policies.
        if entry.retained_bytes() > HISTORY_ENTRY_BYTES {
            entry.discard_details();
        }
        entry
    }

    fn fits(&self, extra: usize) -> bool {
        self.retained_bytes().saturating_add(extra) <= HISTORY_ENTRY_BYTES
    }

    fn add_detail(&mut self, line: String) -> bool {
        if self.details.len() >= ITEM_LIMIT {
            return false;
        }
        let line = bounded_text(&line, DETAIL_BYTES);
        if !self.fits(size_of::<String>() + line.capacity()) {
            return false;
        }
        self.details.reserve_exact(1);
        self.details.push(line);
        true
    }

    fn discard_details(&mut self) {
        self.omitted_details += self.details.len();
        self.omitted_retry_items += self.failures.len() + self.unprocessed.len();
        self.details = Vec::new();
        self.failures = Vec::new();
        self.unprocessed = Vec::new();
    }

    /// This only constructs a new proposal. The caller must capture current
    /// source identities, revalidate the destination and request fresh conflict
    /// and destructive-operation confirmation through its normal action flow.
    pub fn retry_request(&self) -> Result<JobRequest> {
        if !self.retry_context_retained || self.work_root.as_os_str().is_empty() {
            bail!("original task context was omitted; retry is unavailable");
        }
        if self.omitted_retry_items > 0 {
            bail!(
                "{} unresolved paths were omitted; retry is unavailable to avoid a partial retry",
                self.omitted_retry_items
            );
        }
        let mut seen = HashSet::new();
        let sources = self
            .failures
            .iter()
            .map(|failure| &failure.path)
            .chain(self.unprocessed.iter())
            .filter(|path| seen.insert((*path).clone()))
            .cloned()
            .collect::<Vec<_>>();
        if sources.is_empty() {
            bail!("this task has no failed or unprocessed items to retry");
        }
        let kind = match self.kind {
            JobKind::Copy { .. } => JobKind::Copy {
                replace: false,
                skip_conflicts: false,
            },
            JobKind::Move { .. } => JobKind::Move {
                skip_conflicts: false,
            },
            kind => kind,
        };
        if matches!(kind, JobKind::Copy { .. } | JobKind::Move { .. }) && self.destination.is_none()
        {
            bail!("original task destination is unavailable");
        }
        Ok(JobRequest {
            kind,
            sources,
            destination: self.destination.clone(),
            work_root: self.work_root.clone(),
        })
    }

    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.work_root.capacity()
            + self.destination.as_ref().map_or(0, PathBuf::capacity)
            + self.failures.capacity() * size_of::<JobFailure>()
            + self
                .failures
                .iter()
                .map(|failure| failure.path.capacity() + failure.error.capacity())
                .sum::<usize>()
            + self.unprocessed.capacity() * size_of::<PathBuf>()
            + self
                .unprocessed
                .iter()
                .map(PathBuf::capacity)
                .sum::<usize>()
            + self.details.capacity() * size_of::<String>()
            + self.details.iter().map(String::capacity).sum::<usize>()
    }

    /// Export a reviewable report to a new private regular file. Never overwrite
    /// any existing path, and never follow a symlink at the output leaf.
    pub fn export(&self, path: &Path) -> Result<()> {
        let kind = match self.kind {
            JobKind::Copy {
                replace,
                skip_conflicts,
            } => json!({"name": "Copy", "replace": replace, "skip_conflicts": skip_conflicts}),
            JobKind::Move { skip_conflicts } => {
                json!({"name": "Move", "skip_conflicts": skip_conflicts})
            }
            kind => json!({"name": kind.label()}),
        };
        let report = json!({
            "schema_version": 1, "id": self.id, "kind": kind,
            "destination": self.destination.as_deref().map(export_path),
            "work_root": if self.retry_context_retained { Some(export_path(&self.work_root)) } else { None },
            "counts": {"total": self.counts.total, "succeeded": self.counts.succeeded, "failed": self.counts.failed, "skipped": self.counts.skipped, "remaining": self.counts.remaining},
            "cancelled": self.cancelled,
            "elapsed": {"seconds": self.elapsed.as_secs(), "subsec_nanoseconds": self.elapsed.subsec_nanos()},
            "copied_bytes": self.copied_bytes,
            "failures": self.failures.iter().map(|failure| json!({"path": export_path(&failure.path), "error": failure.error})).collect::<Vec<_>>(),
            "unprocessed": self.unprocessed.iter().map(|path| export_path(path)).collect::<Vec<_>>(),
            "details": self.details, "omitted_details": self.omitted_details,
            "omitted_retry_items": self.omitted_retry_items,
            "retry_context_retained": self.retry_context_retained,
            "limits": {"recent_entries": HISTORY_ENTRIES, "retained_entry_bytes": HISTORY_ENTRY_BYTES, "retained_history_bytes": HISTORY_BYTES, "detail_items": ITEM_LIMIT, "detail_line_bytes": DETAIL_BYTES},
            "limitations": "This is a bounded result report, not a resumable copy journal. Counts describe the complete recorded result; details and unresolved paths may be omitted by the stated limits. Detail/error text may carry an explicit truncation marker. Cancellation does not undo completed work. Retry requires fresh source/destination validation and confirmation."
        });
        let bytes = serde_json::to_vec_pretty(&report)?;
        if bytes.len() > HISTORY_EXPORT_BYTES {
            bail!("history report exceeds the 512 KiB export limit");
        }
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let mut file = options
            .open(path)
            .with_context(|| format!("cannot create new history export {}", display_path(path)))?;
        if !file.metadata()?.file_type().is_file() {
            bail!("history export is not a regular file");
        }
        file.write_all(&bytes)
            .context("export write failed; newly created file may be incomplete")?;
        file.sync_all()
            .context("export sync failed; durability is unverified")?;
        Ok(())
    }
}

fn bounded_text(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_string();
    }
    let mut end = limit.saturating_sub(16);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{} [truncated]", &text[..end])
}

fn bounded_path(path: &Path) -> String {
    // Bound before lossy decoding: an arbitrary synthetic path must not cause
    // unbounded allocation just to construct a one-line diagnostic.
    let raw = path.as_os_str().as_encoded_bytes();
    if raw.len() <= DETAIL_BYTES {
        return display_path(path);
    }
    format!(
        "{} [path truncated]",
        String::from_utf8_lossy(&raw[..DETAIL_BYTES - 32])
    )
}

fn export_path(path: &Path) -> Value {
    if let Some(path) = path.to_str() {
        return json!({"text": path});
    }
    #[cfg(unix)]
    {
        use base64::Engine;
        use std::os::unix::ffi::OsStrExt;
        json!({"display": path.to_string_lossy(), "unix_bytes_base64": base64::engine::general_purpose::STANDARD.encode(path.as_os_str().as_bytes())})
    }
    #[cfg(not(unix))]
    json!({"display": path.to_string_lossy(), "encoding": "lossy display only"})
}
