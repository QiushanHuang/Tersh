use crate::fs_core::{escape_display, format_size};
use anyhow::{Context, Result};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const DETECT_LIMIT: usize = 64 * 1024;
const PREVIEW_LIMIT: usize = 2 * 1024 * 1024;
const MAX_LINES: usize = 20_000;
const MAX_LINE_BYTES: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewKind {
    Text,
    Binary,
    Directory,
    Empty,
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
        .with_context(|| format!("failed to preview {}", path.display()))?;
    if metadata.is_dir() {
        return Ok(Preview::message(
            path.to_path_buf(),
            PreviewKind::Directory,
            "Directory selected",
        ));
    }
    if metadata.len() == 0 {
        return Ok(Preview::message(
            path.to_path_buf(),
            PreviewKind::Empty,
            "Empty file",
        ));
    }

    let mut file =
        fs::File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut detect_bytes = vec![0; DETECT_LIMIT.min(metadata.len() as usize)];
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
                format!("Binary file · {}", format_size(metadata.len())),
                format!("Hex preview: {hex}"),
            ],
            truncated: metadata.len() as usize > DETECT_LIMIT,
        });
    }

    let mut bytes = detect_bytes;
    file.by_ref()
        .take((PREVIEW_LIMIT + 1 - bytes.len()) as u64)
        .read_to_end(&mut bytes)?;
    let truncated = bytes.len() > PREVIEW_LIMIT;
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
    let detect_slice = &bytes[..bytes.len().min(DETECT_LIMIT)];
    if looks_binary(detect_slice) {
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
                format!("Binary file · {}", format_size(metadata.len())),
                format!("Hex preview: {hex}"),
            ],
            truncated,
        });
    }

    let text = String::from_utf8_lossy(&bytes);
    let mut lines = Vec::new();
    for (index, line) in text.lines().take(MAX_LINES).enumerate() {
        let visible = truncate_line(line);
        lines.push(format!("{:>4}  {}", index + 1, escape_display(&visible)));
    }
    if text.lines().count() > MAX_LINES {
        lines.push("[preview truncated at 20000 lines]".to_string());
    }
    if truncated {
        lines.push("[preview truncated at 2 MiB]".to_string());
    }

    Ok(Preview {
        path: path.to_path_buf(),
        kind: PreviewKind::Text,
        lines,
        truncated,
    })
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
