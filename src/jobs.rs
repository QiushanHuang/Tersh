//! One worker per active job with a coalesced, constant-size progress snapshot.
use crate::{fs_ops, trash};
use anyhow::{Context, Result, bail};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobKind {
    Copy { replace: bool, skip_conflicts: bool },
    Move { skip_conflicts: bool },
    Trash,
    Delete,
    Restore,
}

impl JobKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Copy { .. } => "Copy",
            Self::Move { .. } => "Move",
            Self::Trash => "Trash",
            Self::Delete => "Delete",
            Self::Restore => "Restore",
        }
    }
}

#[derive(Debug, Clone)]
pub struct JobRequest {
    pub kind: JobKind,
    pub sources: Vec<PathBuf>,
    pub destination: Option<PathBuf>,
    pub work_root: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct JobProgress {
    pub label: String,
    pub current_path: Option<PathBuf>,
    pub copied_bytes: u64,
    pub completed: usize,
    pub total: usize,
    pub cancelling: bool,
}

#[derive(Debug, Clone)]
pub struct JobFailure {
    pub path: PathBuf,
    pub error: String,
}

#[derive(Debug, Clone, Default)]
pub struct JobResult {
    pub succeeded: Vec<PathBuf>,
    pub failed: Vec<JobFailure>,
    pub skipped: Vec<PathBuf>,
    pub unprocessed: Vec<PathBuf>,
    pub cancelled: bool,
}

#[derive(Debug)]
struct Cancelled;
impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("operation cancelled")
    }
}
impl std::error::Error for Cancelled {}

#[derive(Debug)]
pub struct JobHandle {
    cancel: Arc<AtomicBool>,
    progress: Arc<Mutex<JobProgress>>,
    result: Arc<Mutex<Option<JobResult>>>,
    worker: Option<JoinHandle<()>>,
}

impl JobHandle {
    pub fn spawn(request: JobRequest) -> Result<Self> {
        if request.sources.is_empty() {
            bail!("file job has no sources");
        }
        if matches!(request.kind, JobKind::Copy { .. } | JobKind::Move { .. }) {
            let destination = request
                .destination
                .as_ref()
                .context("copy/move job requires a destination")?;
            if !destination.is_dir() {
                bail!("job destination is not a directory");
            }
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(Mutex::new(JobProgress {
            label: request.kind.label().into(),
            total: request.sources.len(),
            ..JobProgress::default()
        }));
        let result = Arc::new(Mutex::new(None));
        let worker_cancel = cancel.clone();
        let worker_progress = progress.clone();
        let worker_result = result.clone();
        let worker = thread::Builder::new()
            .name("tersh-file-job".into())
            .spawn(move || {
                let output = run(request, &worker_cancel, &worker_progress);
                *worker_result.lock().unwrap_or_else(|p| p.into_inner()) = Some(output);
            })
            .context("failed to start file worker")?;
        Ok(Self {
            cancel,
            progress,
            result,
            worker: Some(worker),
        })
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Release);
        self.progress
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .cancelling = true;
    }

    pub fn progress(&self) -> JobProgress {
        self.progress
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }

    /// Consumes the result once. Never blocks waiting for filesystem I/O.
    pub fn try_result(&mut self) -> Option<JobResult> {
        self.result.lock().unwrap_or_else(|p| p.into_inner()).take()
    }

    pub fn is_finished(&self) -> bool {
        self.worker.as_ref().is_none_or(JoinHandle::is_finished)
    }
}

impl Drop for JobHandle {
    fn drop(&mut self) {
        self.cancel();
        // Do not detach a writer on quit; the caller keeps rendering cancellation
        // progress until is_finished before dropping its active handle.
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn run(request: JobRequest, cancel: &AtomicBool, progress: &Mutex<JobProgress>) -> JobResult {
    let mut result = JobResult::default();
    for (index, source) in request.sources.iter().enumerate() {
        if cancel.load(Ordering::Acquire) {
            result.cancelled = true;
            result
                .unprocessed
                .extend_from_slice(&request.sources[index..]);
            break;
        }
        {
            let mut snapshot = progress.lock().unwrap_or_else(|p| p.into_inner());
            snapshot.current_path = Some(source.clone());
        }
        let mut observe = |path: &Path, bytes: u64| -> Result<()> {
            {
                let mut snapshot = progress.lock().unwrap_or_else(|p| p.into_inner());
                if snapshot.current_path.as_deref() != Some(path) {
                    snapshot.current_path = Some(path.to_owned());
                }
                snapshot.copied_bytes = snapshot.copied_bytes.saturating_add(bytes);
            }
            if cancel.load(Ordering::Acquire) {
                return Err(Cancelled.into());
            }
            Ok(())
        };
        let operation = perform(&request, source, &mut observe);
        match operation {
            Ok(true) => result.succeeded.push(source.clone()),
            Ok(false) => result.skipped.push(source.clone()),
            Err(err) if err.downcast_ref::<Cancelled>().is_some() => {
                result.cancelled = true;
                // A delete can be partial; preserve its root in failed with that
                // explicit warning. Other aborted current items remain unresolved.
                if request.kind == JobKind::Delete {
                    result.failed.push(JobFailure {
                        path: source.clone(),
                        error: "cancelled; this directory may be partially deleted".into(),
                    });
                } else {
                    result.unprocessed.push(source.clone());
                }
                result
                    .unprocessed
                    .extend_from_slice(&request.sources[index + 1..]);
                break;
            }
            Err(err) => result.failed.push(JobFailure {
                path: source.clone(),
                error: format!("{err:#}"),
            }),
        }
        progress.lock().unwrap_or_else(|p| p.into_inner()).completed = index + 1;
    }
    result
}

fn perform(
    request: &JobRequest,
    source: &Path,
    observe: &mut dyn FnMut(&Path, u64) -> Result<()>,
) -> Result<bool> {
    observe(source, 0)?;
    match request.kind {
        JobKind::Copy {
            replace,
            skip_conflicts,
        } => {
            let target =
                fs_ops::destination_for_paste(source, request.destination.as_ref().unwrap())?;
            if skip_conflicts && fs_ops::path_exists_no_follow(&target)? {
                return Ok(false);
            }
            fs_ops::copy_path_cancellable(source, &target, replace, observe)?;
        }
        JobKind::Move { skip_conflicts } => {
            let target =
                fs_ops::destination_for_paste(source, request.destination.as_ref().unwrap())?;
            if skip_conflicts && fs_ops::path_exists_no_follow(&target)? {
                return Ok(false);
            }
            fs_ops::rename_path(source, &target)?;
        }
        JobKind::Trash => {
            fs_ops::trash_path(source, &request.work_root)?;
        }
        JobKind::Delete => {
            fs_ops::permanent_delete_cancellable(source, &request.work_root, observe)?
        }
        JobKind::Restore => {
            trash::restore_entry(source, &request.work_root)?;
        }
    }
    Ok(true)
}
