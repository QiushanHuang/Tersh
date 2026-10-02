//! One reusable, on-demand log worker per workbench. Inactive sessions schedule
//! no filesystem I/O; closing a view clears visible data and queues handle release.
use crate::{
    log_reader::{LogReader, LogSnapshot},
    reader::LatestReader,
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const POLL_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug)]
enum LogRequest {
    Open(PathBuf),
    Poll(PathBuf),
    Close,
}

#[derive(Debug)]
pub(crate) struct LogSession {
    path: PathBuf,
    worker: LatestReader<LogRequest, Option<Result<LogSnapshot, String>>>,
    generation: u64,
    active: bool,
    in_flight: bool,
    next_poll: Instant,
    paused: bool,
    snapshot: Option<LogSnapshot>,
}

impl LogSession {
    pub(crate) fn new(path: PathBuf) -> std::io::Result<Self> {
        Self::new_with_opener(path, LogReader::open)
    }

    pub(crate) fn new_with_opener(
        path: PathBuf,
        mut open: impl FnMut(&Path) -> anyhow::Result<LogReader> + Send + 'static,
    ) -> std::io::Result<Self> {
        let mut worker_path: Option<PathBuf> = None;
        let mut reader: Option<LogReader> = None;
        let worker = LatestReader::new("tersh-log-reader", move |request| {
            match request {
                LogRequest::Close => {
                    reader = None;
                    worker_path = None;
                    return None;
                }
                LogRequest::Open(path) => {
                    reader = None;
                    worker_path = Some(path);
                }
                LogRequest::Poll(path) => {
                    if worker_path.as_ref() != Some(&path) {
                        reader = None;
                        worker_path = Some(path);
                    }
                }
            }
            if let Some(reader) = &mut reader {
                let _ = reader.poll();
                Some(Ok(reader.snapshot()))
            } else {
                let result = open(worker_path.as_deref().expect("active log path"))
                    .map(|opened| {
                        let snapshot = opened.snapshot();
                        reader = Some(opened);
                        snapshot
                    })
                    .map_err(|error| format!("{error:#}"));
                Some(result)
            }
        })?;
        let mut session = Self {
            path: PathBuf::new(),
            worker,
            generation: 0,
            active: false,
            in_flight: false,
            next_poll: Instant::now(),
            paused: false,
            snapshot: None,
        };
        session.reopen(path);
        Ok(session)
    }

    pub(crate) fn reopen(&mut self, path: PathBuf) {
        self.path = path;
        self.generation = self.generation.wrapping_add(1);
        self.active = true;
        self.paused = false;
        self.snapshot = None;
        self.in_flight = true;
        self.next_poll = Instant::now() + POLL_INTERVAL;
        self.worker
            .submit(self.generation, LogRequest::Open(self.path.clone()));
    }

    pub(crate) fn deactivate(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.active = false;
        self.in_flight = false;
        self.paused = false;
        self.snapshot = None;
        // Close supersedes queued reads. A currently blocked read is allowed to
        // return on this same worker; it cannot publish into the inactive view.
        self.worker.submit(self.generation, LogRequest::Close);
    }

    pub(crate) fn active(&self) -> bool {
        self.active
    }
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
    pub(crate) fn snapshot(&self) -> Option<&LogSnapshot> {
        self.snapshot.as_ref()
    }
    pub(crate) fn paused(&self) -> bool {
        self.paused
    }

    pub(crate) fn set_paused(&mut self, paused: bool) {
        if !self.active || self.paused == paused {
            return;
        }
        self.paused = paused;
        self.generation = self.generation.wrapping_add(1);
        self.worker.clear_pending();
        self.in_flight = false;
        self.next_poll = Instant::now();
        if let Some(snapshot) = &mut self.snapshot {
            snapshot.paused = paused;
        }
    }

    pub(crate) fn poll(&mut self) -> bool {
        if !self.active {
            return false;
        }
        let mut changed = false;
        if let Some((generation, result)) = self.worker.take_result()
            && generation == self.generation
            && !self.paused
        {
            self.in_flight = false;
            if let Some(result) = result {
                let snapshot = result.unwrap_or_else(|error| LogSnapshot {
                    lines: Vec::new(),
                    status: format!("Read error: {}", crate::fs_core::escape_display(&error)),
                    paused: false,
                    truncated: false,
                    byte_count: 0,
                    omitted_lines: 0,
                    offset: 0,
                });
                changed = self.snapshot.as_ref().is_none_or(|old| {
                    old.offset != snapshot.offset
                        || old.status != snapshot.status
                        || old.omitted_lines != snapshot.omitted_lines
                        || old.lines != snapshot.lines
                });
                self.snapshot = Some(snapshot);
            }
        }
        if !self.paused && !self.in_flight && Instant::now() >= self.next_poll {
            self.worker
                .submit(self.generation, LogRequest::Poll(self.path.clone()));
            self.in_flight = true;
            self.next_poll = Instant::now() + POLL_INTERVAL;
        }
        changed
    }
}
