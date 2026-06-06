use anyhow::{Context, Result};
use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Directory,
    File,
    Symlink,
    Other,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub raw_name: OsString,
    pub name: String,
    pub kind: FileKind,
    pub size: u64,
    pub readonly: bool,
    pub modified: Option<SystemTime>,
    pub symlink_target: Option<PathBuf>,
}

impl FileEntry {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let metadata = fs::symlink_metadata(&path)
            .with_context(|| format!("failed to read metadata for {}", path.display()))?;
        let file_type = metadata.file_type();
        let kind = if file_type.is_dir() {
            FileKind::Directory
        } else if file_type.is_file() {
            FileKind::File
        } else if file_type.is_symlink() {
            FileKind::Symlink
        } else {
            FileKind::Other
        };
        let symlink_target = if kind == FileKind::Symlink {
            fs::read_link(&path).ok()
        } else {
            None
        };
        let raw_name = path
            .file_name()
            .map(OsStr::to_os_string)
            .unwrap_or_else(|| path.as_os_str().to_os_string());
        let name = display_os_str(&raw_name);

        Ok(Self {
            path,
            raw_name,
            name,
            kind,
            size: metadata.len(),
            readonly: metadata.permissions().readonly(),
            modified: metadata.modified().ok(),
            symlink_target,
        })
    }

    pub fn kind_marker(&self) -> &'static str {
        match self.kind {
            FileKind::Directory => "dir",
            FileKind::File => "file",
            FileKind::Symlink => "link",
            FileKind::Other => "other",
        }
    }
}

pub fn read_dir_entries(path: &Path, show_hidden: bool, filter: &str) -> Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    let filter = filter.to_lowercase();
    for entry in fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))? {
        let Ok(entry) = entry else {
            continue;
        };
        let name = display_os_str(&entry.file_name());
        if !show_hidden && name.starts_with('.') {
            continue;
        }
        if !filter.is_empty() && !name.to_lowercase().contains(&filter) {
            continue;
        }
        if let Ok(entry) = FileEntry::from_path(entry.path()) {
            entries.push(entry);
        }
    }
    entries.sort_by(|a, b| {
        let ak = match a.kind {
            FileKind::Directory => 0,
            FileKind::Symlink => 1,
            FileKind::File => 2,
            FileKind::Other => 3,
        };
        let bk = match b.kind {
            FileKind::Directory => 0,
            FileKind::Symlink => 1,
            FileKind::File => 2,
            FileKind::Other => 3,
        };
        ak.cmp(&bk)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(entries)
}

pub fn display_os_str(value: &OsStr) -> String {
    escape_display(&value.to_string_lossy())
}

pub fn display_path(path: &Path) -> String {
    escape_display(&path.display().to_string())
}

pub fn escape_display(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => {
                escaped.push_str(&format!("\\u{{{:x}}}", ch as u32));
            }
            ch => escaped.push(ch),
        }
    }
    escaped
}

pub fn format_size(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    if size < 1024 {
        return format!("{size} B");
    }
    let mut value = size as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

pub fn is_hidden_name(name: &str) -> bool {
    name.starts_with('.')
}
