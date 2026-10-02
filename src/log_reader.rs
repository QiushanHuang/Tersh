//! Bounded, on-demand regular-file log reader. The caller owns polling cadence.

use crate::fs_core::{display_path, escape_display};
use anyhow::{Context, Result, bail};
use std::{
    collections::VecDeque,
    fs::{self, File, Metadata, OpenOptions},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

pub const LOG_READ_BYTES: usize = 64 * 1024;
pub const LOG_BUFFER_BYTES: usize = 256 * 1024;
pub const LOG_BUFFER_LINES: usize = 2_000;
const RAW_LINE_BYTES: usize = 4_096;
const DISPLAY_LINE_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone)]
pub struct LogSnapshot {
    pub lines: Vec<String>,
    pub status: String,
    pub paused: bool,
    pub truncated: bool,
    pub byte_count: usize,
    /// Known lines discarded from the read window. Older unscanned history is
    /// intentionally not counted; `truncated` also indicates an initial tail.
    pub omitted_lines: u64,
    pub offset: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct LogPoll {
    pub changed: bool,
    pub bytes_read: usize,
    pub has_more: bool,
}

/// Own this value on a worker thread when filesystem latency must not block UI
/// input. No threads, timers or background reads are created by this module.
#[derive(Debug)]
pub struct LogReader {
    path: PathBuf,
    file: File,
    identity: Metadata,
    offset: u64,
    paused: bool,
    lines: VecDeque<String>,
    bytes: usize,
    pending: Vec<u8>,
    pending_truncated: bool,
    discard_initial_fragment: bool,
    truncated: bool,
    omitted_lines: u64,
    status: String,
}

impl LogReader {
    /// Read at most the final 64 KiB. Subsequent polls follow the same path and
    /// detect observed truncation and inode replacement (rename rotation).
    pub fn open(path: &Path) -> Result<Self> {
        let (mut file, identity) = open_regular(path)?;
        let offset = identity.len().saturating_sub(LOG_READ_BYTES as u64);
        file.seek(SeekFrom::Start(offset))?;
        let mut reader = Self {
            path: path.to_path_buf(),
            file,
            identity,
            offset,
            paused: false,
            lines: VecDeque::new(),
            bytes: 0,
            pending: Vec::new(),
            pending_truncated: false,
            discard_initial_fragment: offset > 0,
            truncated: offset > 0,
            omitted_lines: 0,
            status: if offset > 0 {
                "Following; initial tail (older bytes omitted)"
            } else {
                "Following"
            }
            .to_string(),
        };
        reader.read_chunk()?;
        Ok(reader)
    }

    /// A paused reader performs no filesystem calls and preserves its offset.
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// One bounded read, with at most 64 KiB consumed. A true `has_more` is a
    /// scheduling hint, not a request to busy-loop. Repeated polls must still be
    /// scheduled at a bounded cadence by the caller.
    pub fn poll(&mut self) -> Result<LogPoll> {
        if self.paused {
            return Ok(LogPoll {
                changed: false,
                bytes_read: 0,
                has_more: false,
            });
        }
        match self.poll_inner() {
            Ok(result) => Ok(result),
            Err(error) => {
                self.status = format!("Read error: {}", escape_display(&error.to_string()));
                Err(error)
            }
        }
    }

    fn poll_inner(&mut self) -> Result<LogPoll> {
        let metadata = fs::symlink_metadata(&self.path)
            .with_context(|| format!("cannot inspect log {}", display_path(&self.path)))?;
        if !metadata.file_type().is_file() {
            bail!(
                "log path is not a regular file: {}",
                display_path(&self.path)
            );
        }
        let mut changed = false;
        if !same_file(&self.identity, &metadata) {
            let (file, identity) = open_regular(&self.path)?;
            self.finish_pending();
            self.file = file;
            self.identity = identity;
            self.offset = 0;
            self.discard_initial_fragment = false;
            self.status = "Following; log rotated (new file)".to_string();
            self.push_line("[log rotated; new file follows]".to_string());
            changed = true;
        } else if metadata.len() < self.offset {
            self.finish_pending();
            self.file.seek(SeekFrom::Start(0))?;
            self.offset = 0;
            self.discard_initial_fragment = false;
            self.status = "Following; log truncated (restarted at byte 0)".to_string();
            self.push_line("[log truncated; restarted at byte 0]".to_string());
            changed = true;
        } else if self.status.starts_with("Read error:") {
            self.status = "Following; read recovered".to_string();
            changed = true;
        }
        let bytes_read = self.read_chunk()?;
        let has_more = self.file.metadata()?.len() > self.offset;
        Ok(LogPoll {
            changed: changed || bytes_read > 0,
            bytes_read,
            has_more,
        })
    }

    fn read_chunk(&mut self) -> Result<usize> {
        let mut bytes = Vec::with_capacity(LOG_READ_BYTES);
        self.file
            .by_ref()
            .take(LOG_READ_BYTES as u64)
            .read_to_end(&mut bytes)
            .with_context(|| format!("cannot read log {}", display_path(&self.path)))?;
        self.offset = self.offset.saturating_add(bytes.len() as u64);
        for byte in &bytes {
            if self.discard_initial_fragment {
                if *byte == b'\n' {
                    self.discard_initial_fragment = false;
                    self.omitted_lines = self.omitted_lines.saturating_add(1);
                }
                continue;
            }
            if *byte == b'\n' {
                let (line, clipped) = self.pending_line(true);
                self.truncated |= clipped;
                self.pending.clear();
                self.pending_truncated = false;
                self.push_line(line);
            } else if self.pending.len() < RAW_LINE_BYTES {
                self.pending.push(*byte);
            } else {
                self.pending_truncated = true;
                self.truncated = true;
            }
        }
        self.trim_buffer();
        Ok(bytes.len())
    }

    fn finish_pending(&mut self) {
        if !self.pending.is_empty() || self.pending_truncated {
            let (line, clipped) = self.pending_line(true);
            self.truncated |= clipped;
            self.pending.clear();
            self.pending_truncated = false;
            self.push_line(line);
        }
    }

    fn pending_line(&self, complete: bool) -> (String, bool) {
        let mut bytes = self.pending.as_slice();
        if complete && bytes.last() == Some(&b'\r') {
            bytes = &bytes[..bytes.len() - 1];
        }
        // Do not display a replacement glyph merely because a valid UTF-8
        // sequence has been split across writes. Actual invalid bytes remain
        // visible as replacement glyphs instead of terminal control sequences.
        if !complete {
            let mut start = 0;
            while start < bytes.len() {
                match std::str::from_utf8(&bytes[start..]) {
                    Ok(_) => break,
                    Err(error) => match error.error_len() {
                        Some(length) => start += error.valid_up_to() + length,
                        None => {
                            bytes = &bytes[..start + error.valid_up_to()];
                            break;
                        }
                    },
                }
            }
        }
        let mut line = escape_display(&String::from_utf8_lossy(bytes));
        let clipped = line.len() > DISPLAY_LINE_BYTES - 32;
        if clipped {
            let mut end = DISPLAY_LINE_BYTES - 32;
            while !line.is_char_boundary(end) {
                end -= 1;
            }
            line.truncate(end);
        }
        if clipped || self.pending_truncated {
            line.push_str(" [line truncated]");
        }
        (line, clipped || self.pending_truncated)
    }

    fn push_line(&mut self, line: String) {
        self.bytes += line.len();
        self.lines.push_back(line);
        self.trim_buffer();
    }

    fn trim_buffer(&mut self) {
        let pending_lines = usize::from(!self.pending.is_empty() || self.pending_truncated);
        let pending_bytes = if pending_lines > 0 {
            DISPLAY_LINE_BYTES
        } else {
            0
        };
        while self.lines.len() + pending_lines > LOG_BUFFER_LINES
            || self.bytes + pending_bytes > LOG_BUFFER_BYTES
        {
            let Some(line) = self.lines.pop_front() else {
                break;
            };
            self.bytes -= line.len();
            self.omitted_lines = self.omitted_lines.saturating_add(1);
            self.truncated = true;
        }
    }

    /// Copy a bounded presentation snapshot. Search should operate on these
    /// retained lines, and must be labelled as a search of the current buffer.
    pub fn snapshot(&self) -> LogSnapshot {
        let mut lines: Vec<String> = self.lines.iter().cloned().collect();
        let mut truncated = self.truncated;
        if !self.pending.is_empty() || self.pending_truncated {
            let (line, clipped) = self.pending_line(false);
            truncated |= clipped;
            lines.push(line);
        }
        let byte_count = lines.iter().map(String::len).sum();
        LogSnapshot {
            lines,
            status: if self.paused {
                format!("Paused; {}", self.status)
            } else {
                self.status.clone()
            },
            paused: self.paused,
            truncated,
            byte_count,
            omitted_lines: self.omitted_lines,
            offset: self.offset,
        }
    }
}

fn open_regular(path: &Path) -> Result<(File, Metadata)> {
    let before = fs::symlink_metadata(path)
        .with_context(|| format!("cannot inspect log {}", display_path(path)))?;
    if !before.file_type().is_file() {
        bail!("log path is not a regular file: {}", display_path(path));
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options
        .open(path)
        .with_context(|| format!("cannot open log {}", display_path(path)))?;
    let metadata = file.metadata()?;
    if !metadata.file_type().is_file() {
        bail!("opened log is not a regular file: {}", display_path(path));
    }
    Ok((file, metadata))
}

#[cfg(unix)]
fn same_file(left: &Metadata, right: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(not(unix))]
fn same_file(left: &Metadata, right: &Metadata) -> bool {
    left.created().ok() == right.created().ok()
}
