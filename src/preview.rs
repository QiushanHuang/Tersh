use crate::fs_core::{display_path, escape_display, format_size};
use anyhow::{Context, Result};
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

const DETECT_LIMIT: usize = 64 * 1024;
const PREVIEW_LIMIT: usize = 2 * 1024 * 1024;
const MAX_LINE_BYTES: usize = 4_096;
const MAX_PREVIEW_LINES: usize = 20_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewKind {
    Text,
    Binary,
    Symlink,
    Directory,
    Empty,
    Unsupported,
    Error,
}

#[derive(Debug, Clone)]
pub struct Preview {
    pub path: PathBuf,
    pub kind: PreviewKind,
    pub lines: Vec<String>,
    pub truncated: bool,
}

impl Preview {
    pub fn message(path: PathBuf, kind: PreviewKind, message: impl Into<String>) -> Self {
        Self {
            path,
            kind,
            lines: vec![message.into()],
            truncated: false,
        }
    }
}

pub fn preview_file(path: &Path) -> Result<Preview> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to preview {}", display_path(path)))?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        let target = fs::read_link(path)
            .map(|target| target.display().to_string())
            .unwrap_or_else(|_| "unreadable target".to_string());
        return Ok(Preview {
            path: path.to_path_buf(),
            kind: PreviewKind::Symlink,
            lines: vec![
                format!("Symlink - {}", format_size(metadata.len())),
                format!("Target: {}", escape_display(&target)),
                "Preview does not follow symlinks.".to_string(),
            ],
            truncated: false,
        });
    }
    if metadata.is_dir() {
        return Ok(Preview::message(
            path.to_path_buf(),
            PreviewKind::Directory,
            "Directory selected",
        ));
    }
    if !file_type.is_file() {
        return Ok(Preview::message(
            path.to_path_buf(),
            PreviewKind::Unsupported,
            "Unsupported file type for preview",
        ));
    }
    let (mut file, metadata) = open_regular_file(path)?;

    let detect_capacity = if metadata.len() == 0 {
        DETECT_LIMIT
    } else {
        DETECT_LIMIT.min(metadata.len() as usize)
    };
    let mut detect_bytes = vec![0; detect_capacity];
    let detected = file.read(&mut detect_bytes)?;
    detect_bytes.truncate(detected);

    if looks_binary(&detect_bytes) {
        let hex = detect_bytes
            .iter()
            .take(256)
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<_>>()
            .join(" ");
        return Ok(Preview {
            path: path.to_path_buf(),
            kind: PreviewKind::Binary,
            lines: vec![
                format!("Binary file - {}", format_size(metadata.len())),
                format!("Hex preview: {hex}"),
            ],
            truncated: metadata.len() as usize > DETECT_LIMIT,
        });
    }

    let mut bytes = detect_bytes;
    file.by_ref()
        .take((PREVIEW_LIMIT + 1 - bytes.len()) as u64)
        .read_to_end(&mut bytes)?;
    let mut truncated = bytes.len() > PREVIEW_LIMIT;
    if truncated {
        bytes.truncate(PREVIEW_LIMIT);
    }
    if bytes.is_empty() {
        return Ok(Preview::message(
            path.to_path_buf(),
            PreviewKind::Empty,
            "Empty file",
        ));
    }
    if looks_binary(&bytes) {
        let hex = bytes
            .iter()
            .take(256)
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<_>>()
            .join(" ");
        return Ok(Preview {
            path: path.to_path_buf(),
            kind: PreviewKind::Binary,
            lines: vec![
                format!("Binary file - {}", format_size(metadata.len())),
                format!("Hex preview: {hex}"),
            ],
            truncated,
        });
    }

    let text = String::from_utf8_lossy(&bytes);
    let mut lines = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if index >= MAX_PREVIEW_LINES {
            truncated = true;
            break;
        }
        let visible = truncate_line(line);
        lines.push(format!("{:>4}  {}", index + 1, escape_display(&visible)));
    }
    if truncated {
        lines.push("[preview truncated]".to_string());
    }

    Ok(Preview {
        path: path.to_path_buf(),
        kind: PreviewKind::Text,
        lines,
        truncated,
    })
}

fn open_regular_file(path: &Path) -> Result<(File, fs::Metadata)> {
    let file = open_no_follow(path)?;
    let metadata = file
        .metadata()
        .with_context(|| format!("failed to inspect opened file {}", display_path(path)))?;
    if !metadata.file_type().is_file() {
        anyhow::bail!("unsupported file type for preview: {}", display_path(path));
    }
    Ok((file, metadata))
}

#[cfg(unix)]
fn open_no_follow(path: &Path) -> Result<File> {
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .with_context(|| format!("failed to open {}", display_path(path)))
}

#[cfg(not(unix))]
fn open_no_follow(path: &Path) -> Result<File> {
    File::open(path).with_context(|| format!("failed to open {}", display_path(path)))
}

fn looks_binary(bytes: &[u8]) -> bool {
    if bytes.contains(&0) {
        return true;
    }
    std::str::from_utf8(bytes).is_err()
}

fn truncate_line(line: &str) -> String {
    if line.len() <= MAX_LINE_BYTES {
        return line.to_string();
    }
    let boundary = line
        .char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= MAX_LINE_BYTES)
        .last()
        .unwrap_or(0);
    let mut truncated = line[..boundary].to_string();
    truncated.push_str(" ... [truncated]");
    truncated
}
