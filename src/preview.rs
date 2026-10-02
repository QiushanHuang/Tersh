use crate::fs_core::{display_path, escape_display, format_size};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

const DETECT_LIMIT: usize = 64 * 1024;
const PREVIEW_LIMIT: usize = 2 * 1024 * 1024;
const MAX_LINE_BYTES: usize = 4_096;
const MAX_PREVIEW_LINES: usize = 20_000;
const STRUCTURED_BYTES: usize = 256 * 1024;
const STRUCTURED_LINES: usize = 2_000;
const STRUCTURED_WIDTH: usize = 512;
const CSV_COLUMNS: usize = 32;

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

pub fn preview_file_structured(path: &Path) -> Result<Preview> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(extension.as_str(), "json" | "csv" | "diff" | "patch") {
        return preview_file(path);
    }
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() {
        return preview_file(path);
    }
    let (file, _) = open_regular_file(path)?;
    let mut bytes = Vec::new();
    file.take((STRUCTURED_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    let oversized = bytes.len() > STRUCTURED_BYTES;
    bytes.truncate(STRUCTURED_BYTES);
    let text = match std::str::from_utf8(&bytes) {
        Ok(text) if !text.contains('\0') => text,
        _ => {
            return Ok(structured_fallback(
                path,
                &bytes,
                "invalid UTF-8 or binary data",
                oversized,
            ));
        }
    };
    if oversized {
        return Ok(structured_fallback(
            path,
            &bytes,
            "256 KiB input limit exceeded",
            true,
        ));
    }
    match extension.as_str() {
        "json" => {
            let mut parser = serde_json::Deserializer::from_str(text);
            if serde::de::IgnoredAny::deserialize(&mut parser).is_err() || parser.end().is_err() {
                return Ok(structured_fallback(
                    path,
                    &bytes,
                    "invalid JSON or validator limit exceeded",
                    false,
                ));
            }
            let (formatted, truncated) = format_json_tokens(text);
            Ok(structured_lines(
                path,
                "JSON · formatted view; raw view available",
                formatted.lines(),
                truncated,
            ))
        }
        "csv" => match parse_csv(text) {
            Ok((rows, mut truncated)) => {
                let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
                let mut widths = vec![1; columns];
                let escaped: Vec<Vec<String>> = rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .enumerate()
                            .map(|(column, cell)| {
                                let (cell, clipped) = clip_display(&escape_display(cell), 48);
                                truncated |= clipped;
                                widths[column] = widths[column]
                                    .max(unicode_width::UnicodeWidthStr::width(cell.as_str()));
                                cell
                            })
                            .collect()
                    })
                    .collect();
                let label = format!(
                    "CSV · {} rows · {columns} columns; raw view available",
                    rows.len()
                );
                let lines = escaped
                    .iter()
                    .map(|row| {
                        (0..columns)
                            .map(|column| {
                                let cell = row.get(column).map(String::as_str).unwrap_or("");
                                format!(
                                    "{}{}",
                                    cell,
                                    " ".repeat(widths[column].saturating_sub(
                                        unicode_width::UnicodeWidthStr::width(cell)
                                    ))
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(" | ")
                    })
                    .collect::<Vec<_>>();
                Ok(structured_lines(
                    path,
                    &label,
                    lines.iter().map(String::as_str),
                    truncated,
                ))
            }
            Err(reason) => Ok(structured_fallback(path, &bytes, reason, false)),
        },
        _ => Ok(structured_lines(
            path,
            "Diff · unified text; raw view available",
            text.lines(),
            false,
        )),
    }
}

// Format validated tokens without reserializing a Value: converting numbers to
// f64 or objects to maps would change numeric precision and erase duplicate keys.
// Only insignificant whitespace outside strings changes; emitted bytes are capped
// during formatting, including expansion from deeply nested input.
fn format_json_tokens(text: &str) -> (String, bool) {
    let mut output = String::with_capacity(STRUCTURED_BYTES);
    let mut chars = text.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    let mut depth = 0usize;
    let mut previous = None;
    while let Some(ch) = chars.next() {
        if in_string {
            if !push_json_char(&mut output, ch) {
                return (output, true);
            }
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
                previous = Some('"');
            }
            continue;
        }
        if ch.is_ascii_whitespace() {
            continue;
        }
        let written = match ch {
            '"' => {
                in_string = true;
                push_json_char(&mut output, ch)
            }
            '{' | '[' => {
                depth += 1;
                let closing = if ch == '{' { '}' } else { ']' };
                let empty = chars.clone().find(|ch| !ch.is_ascii_whitespace()) == Some(closing);
                push_json_char(&mut output, ch) && (empty || push_json_indent(&mut output, depth))
            }
            '}' | ']' => {
                depth = depth.saturating_sub(1);
                let opening = if ch == '}' { '{' } else { '[' };
                (previous == Some(opening) || push_json_indent(&mut output, depth))
                    && push_json_char(&mut output, ch)
            }
            ',' => push_json_piece(&mut output, ",") && push_json_indent(&mut output, depth),
            ':' => push_json_piece(&mut output, ": "),
            _ => push_json_char(&mut output, ch),
        };
        if !written {
            return (output, true);
        }
        previous = Some(ch);
    }
    (output, false)
}

fn push_json_indent(output: &mut String, depth: usize) -> bool {
    push_json_piece(output, "\n") && (0..depth).all(|_| push_json_piece(output, "  "))
}

fn push_json_char(output: &mut String, ch: char) -> bool {
    push_json_piece(output, ch.encode_utf8(&mut [0; 4]))
}

fn push_json_piece(output: &mut String, piece: &str) -> bool {
    if output.len() + piece.len() > STRUCTURED_BYTES {
        return false;
    }
    output.push_str(piece);
    true
}

fn structured_fallback(path: &Path, bytes: &[u8], reason: &str, truncated: bool) -> Preview {
    let text = String::from_utf8_lossy(bytes);
    structured_lines(
        path,
        &format!("Structured preview unavailable: {reason}; raw text follows"),
        text.lines(),
        truncated,
    )
}

fn structured_lines<'a>(
    path: &Path,
    label: &str,
    source: impl Iterator<Item = &'a str>,
    mut truncated: bool,
) -> Preview {
    let mut lines = vec![format!("[{label}]")];
    let mut byte_count = lines[0].len();
    for line in source {
        if lines.len() > STRUCTURED_LINES {
            truncated = true;
            break;
        }
        let (line, clipped) = clip_display(&escape_display(line), STRUCTURED_WIDTH);
        truncated |= clipped;
        if byte_count + line.len() > STRUCTURED_BYTES - 64 {
            truncated = true;
            break;
        }
        byte_count += line.len();
        lines.push(line);
    }
    if truncated {
        lines.push("[structured preview truncated; raw view available]".to_string());
    }
    Preview {
        path: path.to_path_buf(),
        kind: PreviewKind::Text,
        lines,
        truncated,
    }
}

fn clip_display(value: &str, maximum: usize) -> (String, bool) {
    use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
    if UnicodeWidthStr::width(value) <= maximum {
        return (value.to_string(), false);
    }
    let mut width = 0;
    let mut output = String::new();
    for ch in value.chars() {
        let next = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + next > maximum.saturating_sub(3) {
            break;
        }
        output.push(ch);
        width += next;
    }
    output.push_str("...");
    (output, true)
}

fn parse_csv(text: &str) -> std::result::Result<(Vec<Vec<String>>, bool), &'static str> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut cell = String::new();
    let mut chars = text.chars().peekable();
    let mut quoted = false;
    let mut closed_quote = false;
    let mut has_content = false;
    while let Some(ch) = chars.next() {
        has_content = true;
        if quoted {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    cell.push('"');
                } else {
                    quoted = false;
                    closed_quote = true;
                }
            } else {
                cell.push(ch);
            }
            continue;
        }
        if closed_quote && !matches!(ch, ',' | '\n' | '\r') {
            return Err("invalid CSV after closing quote");
        }
        match ch {
            '"' if cell.is_empty() && !closed_quote => quoted = true,
            '"' => return Err("invalid quote in CSV cell"),
            ',' => {
                row.push(std::mem::take(&mut cell));
                closed_quote = false;
                if row.len() >= CSV_COLUMNS {
                    return Err("CSV exceeds 32 column limit");
                }
            }
            '\r' | '\n' => {
                if ch == '\r' && chars.peek() == Some(&'\n') {
                    chars.next();
                }
                row.push(std::mem::take(&mut cell));
                rows.push(std::mem::take(&mut row));
                closed_quote = false;
                has_content = false;
                if rows.len() >= STRUCTURED_LINES {
                    return Ok((rows, chars.peek().is_some()));
                }
            }
            _ => cell.push(ch),
        }
    }
    if quoted {
        return Err("unterminated quoted CSV cell");
    }
    if has_content || !row.is_empty() {
        row.push(cell);
        rows.push(row);
    }
    Ok((rows, false))
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
