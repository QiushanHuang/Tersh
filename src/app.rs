use crate::{
    fs_core::{FileEntry, FileKind, read_dir_entries_with_diagnostics},
    fs_ops::{
        copy_path, destination_for_paste, permanent_delete, rename_path, trash_path,
        validate_file_name,
    },
    preview::{Preview, PreviewKind, preview_file},
};
use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    collections::hash_map::DefaultHasher,
    collections::{BTreeSet, VecDeque},
    ffi::{CStr, CString, OsStr, OsString},
    fs,
    hash::{Hash, Hasher},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
    time::{Duration, SystemTime},
};

const PREVIEW_SIGNATURE_HASH_LIMIT: u64 = (2 * 1024 * 1024) + 1;
const PREVIEW_CACHE_LIMIT: usize = 32;
const PREVIEW_CACHE_BYTES_LIMIT: usize = 4 * 1024 * 1024;
const PREVIEW_CACHE_ENTRY_BYTES_LIMIT: usize = 512 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Filter,
    Goto,
    Help,
    Preview,
    PreviewSearch,
    Rename,
    CopyTo,
    MoveTo,
    ConfirmTrash,
    ConfirmDelete,
    Conflict,
    Message,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Down,
    Up,
    HalfDown,
    HalfUp,
    First,
    Last,
    Parent,
    Open,
    OpenFilter,
    OpenGoto,
    OpenPreviewSearch,
    ToggleHidden,
    ToggleSelect,
    SelectAll,
    ClearSelection,
    Copy,
    Cut,
    Paste,
    CopyName,
    CopyRelativePath,
    CopyAbsolutePath,
    CopyTo,
    MoveTo,
    Rename,
    Trash,
    PermanentDelete,
    Refresh,
    OpenHelp,
    PreviewSearchNext,
    PreviewSearchPrev,
    CycleSort,
    ReverseSort,
    Cancel,
    Quit,
    ForceQuit,
    Input(char),
    Edit,
    Backspace,
    Submit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Kind,
    Size,
    Modified,
}

#[derive(Debug, Clone)]
pub struct App {
    cwd: PathBuf,
    work_root: PathBuf,
    all_entries: Vec<FileEntry>,
    entries: Vec<FileEntry>,
    cursor: usize,
    selected: BTreeSet<PathBuf>,
    transfer_buffer: Option<TransferBuffer>,
    preview: Preview,
    preview_offset: usize,
    preview_search_query: String,
    preview_search_matches: Vec<usize>,
    preview_search_index: Option<usize>,
    mode: Mode,
    should_quit: bool,
    terminal_output: TerminalOutput,
    show_hidden: bool,
    filter: String,
    input: String,
    logs: Vec<String>,
    pending_g: bool,
    pending_y: bool,
    clipboard_text: Option<String>,
    last_clipboard_text: Option<String>,
    sort_key: SortKey,
    sort_reverse: bool,
    preview_cache: VecDeque<CachedPreview>,
    pending_file_operation: Option<PendingFileOperation>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RunOptions {
    pub print_cwd: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransferKind {
    Copy,
    Cut,
}

#[derive(Debug, Clone)]
struct TransferBuffer {
    kind: TransferKind,
    paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileOperationSource {
    TransferBuffer,
    Direct,
}

#[derive(Debug, Clone)]
struct PendingFileOperation {
    kind: TransferKind,
    paths: Vec<PathBuf>,
    destination: PathBuf,
    source: FileOperationSource,
    label: &'static str,
    allow_replace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PreviewSignature {
    len: u64,
    modified: Option<SystemTime>,
    kind: FileKind,
    symlink_target: Option<PathBuf>,
    sample_hash: Option<u64>,
}

#[derive(Debug, Clone)]
struct CachedPreview {
    path: PathBuf,
    signature: PreviewSignature,
    preview: Preview,
    estimated_bytes: usize,
}

struct InitialLocation {
    cwd: PathBuf,
    focus_name: Option<OsString>,
    open_preview: bool,
    show_hidden: bool,
}

fn resolve_initial_location(path: &Path) -> Result<InitialLocation> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to resolve {}", path.display()))?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        return Ok(InitialLocation {
            cwd: path
                .canonicalize()
                .with_context(|| format!("failed to resolve {}", path.display()))?,
            focus_name: None,
            open_preview: false,
            show_hidden: false,
        });
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let focus_name = path
        .file_name()
        .map(OsStr::to_os_string)
        .ok_or_else(|| anyhow::anyhow!("path has no file name: {}", path.display()))?;
    let show_hidden = focus_name
        .to_string_lossy()
        .chars()
        .next()
        .map(|ch| ch == '.')
        .unwrap_or(false);

    Ok(InitialLocation {
        cwd: parent
            .canonicalize()
            .with_context(|| format!("failed to resolve {}", parent.display()))?,
        focus_name: Some(focus_name),
        open_preview: metadata.file_type().is_file(),
        show_hidden,
    })
}

impl App {
    pub fn new(path: PathBuf) -> Result<Self> {
        Self::new_with_output(path, TerminalOutput::Stdout)
    }

    fn new_with_output(path: PathBuf, terminal_output: TerminalOutput) -> Result<Self> {
        let initial = resolve_initial_location(&path)?;
        let mut app = Self {
            work_root: initial.cwd.clone(),
            cwd: initial.cwd,
            all_entries: Vec::new(),
            entries: Vec::new(),
            cursor: 0,
            selected: BTreeSet::new(),
            transfer_buffer: None,
            preview: Preview::message(PathBuf::new(), PreviewKind::Empty, "No file selected"),
            preview_offset: 0,
            preview_search_query: String::new(),
            preview_search_matches: Vec::new(),
            preview_search_index: None,
            mode: Mode::Normal,
            should_quit: false,
            terminal_output,
            show_hidden: initial.show_hidden,
            filter: String::new(),
            input: String::new(),
            logs: Vec::new(),
            pending_g: false,
            pending_y: false,
            clipboard_text: None,
            last_clipboard_text: None,
            sort_key: SortKey::Kind,
            sort_reverse: false,
            preview_cache: VecDeque::new(),
            pending_file_operation: None,
        };
        app.reload();
        if let Some(name) = initial.focus_name {
            app.focus_raw_name(&name);
            if initial.open_preview
                && app
                    .focused()
                    .map(|entry| entry.kind == FileKind::File)
                    .unwrap_or(false)
            {
                app.mode = Mode::Preview;
            }
        }
        Ok(app)
    }

    pub fn for_test() -> Self {
        let cwd = PathBuf::from("/tmp/tersh-test");
        let entries = vec![
            FileEntry {
                path: PathBuf::from("/tmp/tersh-test/src"),
                raw_name: "src".into(),
                name: "src".to_string(),
                kind: FileKind::Directory,
                size: 0,
                readonly: false,
                modified: None,
                symlink_target: None,
                name_lower: "src".to_string(),
            },
            FileEntry {
                path: PathBuf::from("/tmp/tersh-test/README.md"),
                raw_name: "README.md".into(),
                name: "README.md".to_string(),
                name_lower: "readme.md".to_string(),
                kind: FileKind::File,
                size: 128,
                readonly: false,
                modified: None,
                symlink_target: None,
            },
        ];
        Self {
            work_root: cwd.clone(),
            cwd,
            all_entries: entries.clone(),
            entries,
            cursor: 0,
            selected: BTreeSet::new(),
            transfer_buffer: None,
            preview: Preview {
                path: PathBuf::from("/tmp/tersh-test/README.md"),
                kind: PreviewKind::Text,
                lines: vec!["   1  # tersh".to_string(), "   2  preview".to_string()],
                truncated: false,
            },
            preview_offset: 0,
            preview_search_query: String::new(),
            preview_search_matches: Vec::new(),
            preview_search_index: None,
            mode: Mode::Normal,
            should_quit: false,
            terminal_output: TerminalOutput::Stdout,
            show_hidden: false,
            filter: String::new(),
            input: String::new(),
            logs: vec!["ready".to_string()],
            pending_g: false,
            pending_y: false,
            clipboard_text: None,
            last_clipboard_text: None,
            sort_key: SortKey::Kind,
            sort_reverse: false,
            preview_cache: VecDeque::new(),
            pending_file_operation: None,
        }
    }

    pub fn apply(&mut self, command: Command) {
        match command {
            Command::Cancel => self.cancel(),
            Command::Quit => {
                if self.mode == Mode::Normal {
                    self.should_quit = true;
                } else {
                    self.mode = Mode::Normal;
                    self.input.clear();
                }
            }
            Command::ForceQuit => self.should_quit = true,
            Command::OpenFilter => {
                self.mode = Mode::Filter;
                self.input = self.filter.clone();
            }
            Command::OpenGoto => {
                self.mode = Mode::Goto;
                self.input.clear();
            }
            Command::OpenHelp => self.mode = Mode::Help,
            Command::Trash => {
                self.mode = Mode::ConfirmTrash;
                self.input.clear();
            }
            Command::PermanentDelete => {
                self.mode = Mode::ConfirmDelete;
                self.input.clear();
            }
            _ => {}
        }
    }

    pub fn handle_command(&mut self, command: Command) {
        match command {
            Command::Down => {
                if self.mode == Mode::Preview {
                    self.scroll_preview(1);
                } else {
                    self.move_cursor(1);
                }
            }
            Command::Up => {
                if self.mode == Mode::Preview {
                    self.scroll_preview(-1);
                } else {
                    self.move_cursor(-1);
                }
            }
            Command::HalfDown => {
                if self.mode == Mode::Preview {
                    self.scroll_preview(10);
                } else {
                    self.move_cursor(10);
                }
            }
            Command::HalfUp => {
                if self.mode == Mode::Preview {
                    self.scroll_preview(-10);
                } else {
                    self.move_cursor(-10);
                }
            }
            Command::First => {
                if self.mode == Mode::Preview {
                    self.scroll_preview_to_top();
                } else {
                    self.cursor = 0;
                    self.update_preview();
                }
            }
            Command::Last => {
                if self.mode == Mode::Preview {
                    self.scroll_preview_to_bottom();
                } else {
                    self.cursor = self.entries.len().saturating_sub(1);
                    self.update_preview();
                }
            }
            Command::Parent => self.go_parent(),
            Command::Open => self.open_focused(),
            Command::OpenFilter => self.apply(command),
            Command::OpenGoto => self.apply(command),
            Command::OpenPreviewSearch => self.enter_preview_search(),
            Command::ToggleHidden => {
                self.show_hidden = !self.show_hidden;
                self.reload();
            }
            Command::ToggleSelect => self.toggle_selected(),
            Command::SelectAll => {
                for entry in &self.entries {
                    self.selected.insert(entry.path.clone());
                }
                self.log(format!("selected {}", self.selected.len()));
            }
            Command::ClearSelection => {
                self.selected.clear();
                self.log("selection cleared");
            }
            Command::Copy => self.copy_selection(),
            Command::Cut => self.cut_selection(),
            Command::Paste => self.paste_buffer(),
            Command::CopyName => self.copy_focused_name(),
            Command::CopyRelativePath => self.copy_focused_relative_path(),
            Command::CopyAbsolutePath => self.copy_focused_absolute_path(),
            Command::CopyTo => {
                self.mode = Mode::CopyTo;
                self.input.clear();
            }
            Command::MoveTo => {
                self.mode = Mode::MoveTo;
                self.input.clear();
            }
            Command::PreviewSearchNext => self.preview_search_next(),
            Command::PreviewSearchPrev => self.preview_search_prev(),
            Command::CycleSort => self.cycle_sort(),
            Command::ReverseSort => self.reverse_sort(),
            Command::Rename => {
                if let Some(entry) = self.focused() {
                    if let Some(name) = entry.raw_name.to_str() {
                        self.input = name.to_string();
                        self.mode = Mode::Rename;
                    } else {
                        self.log("rename unsupported for non-UTF-8 file name");
                    }
                }
            }
            Command::Trash => {
                self.mode = Mode::ConfirmTrash;
                self.input.clear();
            }
            Command::PermanentDelete => {
                self.mode = Mode::ConfirmDelete;
                self.input.clear();
            }
            Command::Refresh => self.reload(),
            Command::OpenHelp | Command::Cancel | Command::Quit | Command::ForceQuit => {
                self.apply(command)
            }
            Command::Edit => self.open_in_editor(),
            Command::Input(ch) => self.handle_input(ch),
            Command::Backspace => {
                self.input.pop();
                if self.mode == Mode::Filter {
                    self.filter = self.input.clone();
                    self.refresh_visible_entries();
                    self.preview_offset = 0;
                }
            }
            Command::Submit => self.submit(),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.handle_command(Command::ForceQuit);
            return;
        }
        if key.code == KeyCode::Esc {
            self.handle_command(Command::Cancel);
            return;
        }
        if is_cancel_key(key) {
            self.handle_command(Command::Cancel);
            return;
        }
        if matches!(
            self.mode,
            Mode::Filter
                | Mode::Goto
                | Mode::Rename
                | Mode::CopyTo
                | Mode::MoveTo
                | Mode::ConfirmTrash
                | Mode::ConfirmDelete
                | Mode::Conflict
        ) {
            let command = match key.code {
                KeyCode::Enter => Some(Command::Submit),
                KeyCode::Backspace => Some(Command::Backspace),
                KeyCode::Char(ch) => Some(Command::Input(ch)),
                _ => None,
            };
            if let Some(command) = command {
                self.handle_command(command);
            }
            return;
        }
        if self.mode == Mode::PreviewSearch {
            let command = match key.code {
                KeyCode::Enter => Some(Command::Submit),
                KeyCode::Backspace => Some(Command::Backspace),
                KeyCode::Char(ch) => Some(Command::Input(ch)),
                _ => None,
            };
            if let Some(command) = command {
                self.handle_command(command);
            }
            return;
        }
        if self.mode == Mode::Help {
            match key.code {
                KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('?') => {
                    self.handle_command(Command::Cancel)
                }
                KeyCode::Char('Q') => self.handle_command(Command::ForceQuit),
                _ => {}
            }
            return;
        }
        if self.mode == Mode::Preview {
            if key.code == KeyCode::Char('g') && key.modifiers.is_empty() {
                if self.pending_g {
                    self.pending_g = false;
                    self.handle_command(Command::First);
                } else {
                    self.pending_g = true;
                }
                return;
            }
            if self.pending_g {
                self.pending_g = false;
            }
            let command = match key.code {
                KeyCode::Enter | KeyCode::Char('q') => Some(Command::Cancel),
                KeyCode::Char('Q') => Some(Command::ForceQuit),
                KeyCode::Char('j') => Some(Command::HalfDown),
                KeyCode::Char('k') => Some(Command::HalfUp),
                KeyCode::Down => Some(Command::Down),
                KeyCode::Up => Some(Command::Up),
                KeyCode::PageDown => Some(Command::HalfDown),
                KeyCode::PageUp => Some(Command::HalfUp),
                KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(Command::Down)
                }
                KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(Command::Up)
                }
                KeyCode::Char(' ') => Some(Command::HalfDown),
                KeyCode::Home => Some(Command::First),
                KeyCode::End => Some(Command::Last),
                KeyCode::Char('G') => Some(Command::Last),
                KeyCode::Char('e') => Some(Command::Edit),
                KeyCode::Char('/') => Some(Command::OpenPreviewSearch),
                KeyCode::Char('n') => Some(Command::PreviewSearchNext),
                KeyCode::Char('N') => Some(Command::PreviewSearchPrev),
                KeyCode::Char('g') => None,
                _ => None,
            };
            if let Some(command) = command {
                self.handle_command(command);
            }
            return;
        }
        if self.pending_y {
            let command = match key.code {
                KeyCode::Char('y') => Some(Command::Copy),
                KeyCode::Char('f') => Some(Command::CopyName),
                KeyCode::Char('r') => Some(Command::CopyRelativePath),
                KeyCode::Char('a') => Some(Command::CopyAbsolutePath),
                _ => None,
            };
            self.pending_y = false;
            if let Some(command) = command {
                self.handle_command(command);
                return;
            }
        }
        if key.code == KeyCode::Char('y') && key.modifiers.is_empty() {
            self.pending_y = true;
            return;
        }
        if key.code == KeyCode::Char('g') && key.modifiers.is_empty() {
            if self.pending_g {
                self.pending_g = false;
                self.handle_command(Command::First);
            } else {
                self.pending_g = true;
            }
            return;
        }
        if let Some(command) = key_to_command(key) {
            self.handle_command(command);
        }
        self.pending_g = false;
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    pub fn entries(&self) -> &[FileEntry] {
        &self.entries
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn is_selected(&self, path: &Path) -> bool {
        self.selected.contains(path)
    }

    pub fn preview(&self) -> &Preview {
        &self.preview
    }

    pub fn copy_buffer_len(&self) -> usize {
        self.transfer_buffer
            .as_ref()
            .map(|buffer| buffer.paths.len())
            .unwrap_or(0)
    }

    pub fn transfer_marker_for(&self, path: &Path) -> &'static str {
        let Some(buffer) = &self.transfer_buffer else {
            return " ";
        };
        if !buffer.paths.iter().any(|candidate| candidate == path) {
            return " ";
        }
        match buffer.kind {
            TransferKind::Copy => "C",
            TransferKind::Cut => "X",
        }
    }

    pub fn copy_buffer_label(&self) -> String {
        self.transfer_buffer
            .as_ref()
            .map(|buffer| {
                let kind = match buffer.kind {
                    TransferKind::Copy => "COPY",
                    TransferKind::Cut => "CUT",
                };
                format!("{kind} {}", buffer.paths.len())
            })
            .unwrap_or_else(|| "EMPTY 0".to_string())
    }

    pub fn sort_label(&self) -> String {
        let key = match self.sort_key {
            SortKey::Kind => "kind",
            SortKey::Size => "size",
            SortKey::Modified => "mtime",
        };
        let direction = if self.sort_reverse { "desc" } else { "asc" };
        format!("{key} {direction}")
    }

    pub fn show_hidden(&self) -> bool {
        self.show_hidden
    }

    pub fn pending_y(&self) -> bool {
        self.pending_y
    }

    pub fn pending_g(&self) -> bool {
        self.pending_g
    }

    pub fn selected_total_size(&self) -> u64 {
        self.entries
            .iter()
            .filter(|entry| self.selected.contains(&entry.path))
            .map(|entry| entry.size)
            .sum()
    }

    pub fn selected_len(&self) -> usize {
        self.selected.len()
    }

    pub fn logs(&self) -> &[String] {
        &self.logs
    }

    pub fn filter(&self) -> &str {
        &self.filter
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn preview_offset(&self) -> usize {
        self.preview_offset
    }

    pub fn preview_search_query(&self) -> &str {
        &self.preview_search_query
    }

    pub fn preview_matches(&self) -> &[usize] {
        &self.preview_search_matches
    }

    pub fn preview_active_match(&self) -> Option<usize> {
        self.preview_search_index
    }

    pub fn last_clipboard_text(&self) -> Option<&str> {
        self.last_clipboard_text.as_deref()
    }

    pub fn operation_target_count(&self) -> usize {
        self.operation_targets().len()
    }

    pub fn operation_target_first(&self) -> Option<PathBuf> {
        self.operation_targets().into_iter().next()
    }

    pub fn operation_target_source(&self) -> &'static str {
        if self.selected.is_empty() {
            "focused"
        } else {
            "selected"
        }
    }

    pub fn operation_target_labels(&self, limit: usize) -> Vec<String> {
        self.operation_targets()
            .into_iter()
            .take(limit)
            .map(|path| crate::fs_core::display_path(&path))
            .collect()
    }

    pub fn conflict_count(&self) -> usize {
        self.pending_file_operation
            .as_ref()
            .map(|operation| self.conflict_targets(operation).len())
            .unwrap_or(0)
    }

    pub fn conflict_allows_replace(&self) -> bool {
        self.pending_file_operation
            .as_ref()
            .map(|operation| operation.allow_replace)
            .unwrap_or(false)
    }

    pub fn conflict_destination(&self) -> Option<&Path> {
        self.pending_file_operation
            .as_ref()
            .map(|operation| operation.destination.as_path())
    }

    pub fn conflict_target_labels(&self, limit: usize) -> Vec<String> {
        self.pending_file_operation
            .as_ref()
            .map(|operation| {
                self.conflict_targets(operation)
                    .into_iter()
                    .take(limit)
                    .map(|path| crate::fs_core::display_path(&path))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn take_clipboard_text(&mut self) -> Option<String> {
        self.clipboard_text.take()
    }

    pub fn force_cwd_for_test(&mut self, cwd: PathBuf) {
        self.cwd = cwd;
        self.reload();
    }

    fn reload(&mut self) {
        match read_dir_entries_with_diagnostics(&self.cwd, self.show_hidden, "") {
            Ok(result) => {
                self.all_entries = result.entries;
                self.sort_all_entries();
                self.refresh_visible_entries();
                self.preview_offset = 0;
                if result.skipped > 0 {
                    self.log(format!("skipped {} unreadable item(s)", result.skipped));
                }
            }
            Err(err) => {
                self.log(format!("error: {err}"));
                self.all_entries.clear();
                self.entries.clear();
                self.selected.clear();
                self.update_preview();
                self.preview_offset = 0;
            }
        }
    }

    fn update_preview(&mut self) {
        if let Some(path) = self.focused().map(|entry| entry.path.clone()) {
            self.preview = self.preview_for_path(&path);
        } else {
            self.preview =
                Preview::message(self.cwd.clone(), PreviewKind::Empty, "No file selected");
        }
    }

    fn preview_for_path(&mut self, path: &Path) -> Preview {
        let signature = preview_signature(path);
        if let Some(signature) = &signature
            && let Some(index) = self
                .preview_cache
                .iter()
                .position(|cache| cache.path == path && &cache.signature == signature)
            && let Some(cache) = self.preview_cache.remove(index)
        {
            let preview = cache.preview.clone();
            self.preview_cache.push_back(cache);
            return preview;
        }
        let preview = match preview_file(path) {
            Ok(preview) => preview,
            Err(err) => Preview::message(
                path.to_path_buf(),
                PreviewKind::Error,
                crate::fs_core::escape_display(&err.to_string()),
            ),
        };
        if let Some(signature) = signature
            && preview.kind != PreviewKind::Error
        {
            let estimated_bytes = preview_cache_bytes(&preview);
            self.preview_cache.retain(|cache| cache.path != path);
            if estimated_bytes <= PREVIEW_CACHE_ENTRY_BYTES_LIMIT {
                self.preview_cache.push_back(CachedPreview {
                    path: path.to_path_buf(),
                    signature,
                    preview: preview.clone(),
                    estimated_bytes,
                });
                self.trim_preview_cache();
            }
        }
        preview
    }

    fn trim_preview_cache(&mut self) {
        while self.preview_cache.len() > PREVIEW_CACHE_LIMIT
            || self.preview_cache_bytes() > PREVIEW_CACHE_BYTES_LIMIT
        {
            if self.preview_cache.pop_front().is_none() {
                break;
            }
        }
    }

    fn preview_cache_bytes(&self) -> usize {
        self.preview_cache
            .iter()
            .map(|cache| cache.estimated_bytes)
            .sum()
    }

    fn scroll_preview(&mut self, delta: isize) {
        if self.preview.lines.is_empty() {
            self.preview_offset = 0;
            return;
        }
        if delta.is_negative() {
            self.preview_offset = self.preview_offset.saturating_sub(delta.unsigned_abs());
        } else {
            self.preview_offset = self
                .preview_offset
                .saturating_add(delta as usize)
                .min(self.preview.lines.len().saturating_sub(1));
        }
    }

    fn scroll_preview_to_top(&mut self) {
        self.preview_offset = 0;
        self.pending_g = false;
    }

    fn scroll_preview_to_bottom(&mut self) {
        self.preview_offset = self.preview.lines.len().saturating_sub(1);
        self.pending_g = false;
    }

    fn clear_preview_search(&mut self) {
        self.preview_search_query.clear();
        self.preview_search_matches.clear();
        self.preview_search_index = None;
        self.input.clear();
    }

    fn enter_preview_search(&mut self) {
        if self.mode != Mode::Preview {
            return;
        }
        self.input = self.preview_search_query.clone();
        self.mode = Mode::PreviewSearch;
    }

    fn execute_preview_search(&mut self) {
        let query = self.input.trim().to_string();
        self.preview_search_query = query.clone();
        self.input.clear();
        self.mode = Mode::Preview;
        if query.is_empty() {
            self.preview_search_matches.clear();
            self.preview_search_index = None;
            return;
        }

        let lower_query = query.to_lowercase();
        self.preview_search_matches = self
            .preview
            .lines
            .iter()
            .enumerate()
            .filter_map(|(index, line)| {
                if line.to_lowercase().contains(&lower_query) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect();
        if self.preview_search_matches.is_empty() {
            self.preview_search_index = None;
            self.log(format!("no match: {query}"));
            return;
        }
        self.preview_search_index = Some(0);
        self.preview_offset = self.preview_search_matches[0];
        self.log(format!("search: {query}"));
    }

    fn preview_search_next(&mut self) {
        if self.preview_search_matches.is_empty() {
            return;
        }
        let next = match self.preview_search_index {
            Some(current) => (current + 1) % self.preview_search_matches.len(),
            None => 0,
        };
        self.preview_search_index = Some(next);
        self.preview_offset = self.preview_search_matches[next];
    }

    fn preview_search_prev(&mut self) {
        if self.preview_search_matches.is_empty() {
            return;
        }
        let prev = match self.preview_search_index {
            Some(current) if current > 0 => current - 1,
            Some(_) => self.preview_search_matches.len().saturating_sub(1),
            None => 0,
        };
        self.preview_search_index = Some(prev);
        self.preview_offset = self.preview_search_matches[prev];
    }

    fn open_in_editor(&mut self) {
        let Some(entry) = self.focused() else {
            self.log("no file selected");
            return;
        };
        if entry.kind != FileKind::File {
            self.log("can only edit regular files");
            return;
        }
        let path = entry.path.clone();
        if let Err(err) = validate_editor_target(&path) {
            self.log(format!("can only edit regular files: {err}"));
            return;
        }
        match self.launch_editor(&path) {
            Ok(()) => {
                self.log(format!(
                    "saved in editor: {}",
                    crate::fs_core::display_path(&path)
                ));
            }
            Err(err) => {
                self.log(format!("edit failed: {err}"));
                return;
            }
        }
        self.update_preview();
        self.scroll_preview_to_top();
    }

    fn launch_editor(&self, path: &Path) -> Result<()> {
        let editor_parts = editor_command_parts()?;
        let Some((editor, args)) = editor_parts.split_first() else {
            return Err(anyhow::anyhow!("no editor command resolved"));
        };

        let mut suspension = TerminalSuspension::suspend(self.terminal_output)?;
        let status = ProcessCommand::new(editor).args(args).arg(path).status();
        suspension.restore()?;
        let status = status?;
        if !status.success() {
            return Err(anyhow::anyhow!("editor exited with {status}"));
        }
        Ok(())
    }

    fn move_cursor(&mut self, delta: isize) {
        if self.entries.is_empty() {
            self.cursor = 0;
            return;
        }
        self.cursor = self
            .cursor
            .saturating_add_signed(delta)
            .min(self.entries.len() - 1);
        self.update_preview();
    }

    fn cycle_sort(&mut self) {
        self.sort_key = match self.sort_key {
            SortKey::Kind => SortKey::Size,
            SortKey::Size => SortKey::Modified,
            SortKey::Modified => SortKey::Kind,
        };
        self.sort_reverse = false;
        self.sort_entries();
        self.cursor = self.cursor.min(self.entries.len().saturating_sub(1));
        self.update_preview();
        self.log(format!("sort {}", self.sort_label()));
    }

    fn reverse_sort(&mut self) {
        self.sort_reverse = !self.sort_reverse;
        self.sort_entries();
        self.cursor = self.cursor.min(self.entries.len().saturating_sub(1));
        self.update_preview();
        self.log(format!("sort {}", self.sort_label()));
    }

    fn sort_entries(&mut self) {
        self.sort_all_entries();
        self.refresh_visible_entries();
    }

    fn sort_all_entries(&mut self) {
        let key = self.sort_key;
        self.all_entries.sort_by(|a, b| {
            let ordering = match key {
                SortKey::Kind => kind_rank(a.kind)
                    .cmp(&kind_rank(b.kind))
                    .then_with(|| a.name_lower.cmp(&b.name_lower)),
                SortKey::Size => a
                    .size
                    .cmp(&b.size)
                    .then_with(|| a.name_lower.cmp(&b.name_lower)),
                SortKey::Modified => a
                    .modified
                    .cmp(&b.modified)
                    .then_with(|| a.name_lower.cmp(&b.name_lower)),
            };
            if self.sort_reverse {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    fn refresh_visible_entries(&mut self) {
        if self.filter.is_empty() {
            self.entries = self.all_entries.clone();
        } else {
            let filter = self.filter.to_lowercase();
            self.entries = self
                .all_entries
                .iter()
                .filter(|entry| entry.name_lower.contains(&filter))
                .cloned()
                .collect();
        }
        self.retain_visible_selection();
        self.cursor = self.cursor.min(self.entries.len().saturating_sub(1));
        self.update_preview();
    }

    fn go_parent(&mut self) {
        if let Some(parent) = self.cwd.parent() {
            self.cwd = parent.to_path_buf();
            self.selected.clear();
            self.filter.clear();
            self.reload();
        }
    }

    fn open_focused(&mut self) {
        if self.mode != Mode::Normal {
            self.submit();
            return;
        }
        if let Some(entry) = self.focused() {
            if entry.kind == FileKind::Directory {
                self.cwd = entry.path.clone();
                self.selected.clear();
                self.filter.clear();
                self.cursor = 0;
                self.reload();
            } else {
                self.mode = Mode::Preview;
                self.update_preview();
                self.clear_preview_search();
                self.preview_offset = 0;
            }
        }
    }

    fn toggle_selected(&mut self) {
        if let Some(path) = self.focused().map(|entry| entry.path.clone())
            && !self.selected.remove(&path)
        {
            self.selected.insert(path);
        }
    }

    fn copy_selection(&mut self) {
        let paths = self.operation_targets();
        let len = paths.len();
        self.transfer_buffer = Some(TransferBuffer {
            kind: TransferKind::Copy,
            paths,
        });
        self.log(format!("copied {len} item(s)"));
    }

    fn cut_selection(&mut self) {
        let paths = self.operation_targets();
        let len = paths.len();
        self.transfer_buffer = Some(TransferBuffer {
            kind: TransferKind::Cut,
            paths,
        });
        self.log(format!("cut {len} item(s)"));
    }

    fn paste_buffer(&mut self) {
        let Some(buffer) = self.transfer_buffer.clone() else {
            self.log("transfer buffer empty");
            return;
        };
        let operation = PendingFileOperation {
            kind: buffer.kind,
            paths: buffer.paths,
            destination: self.cwd.clone(),
            source: FileOperationSource::TransferBuffer,
            label: "pasted",
            allow_replace: buffer.kind == TransferKind::Copy,
        };
        self.start_file_operation(operation);
    }

    fn copy_to_destination(&mut self, move_items: bool) {
        let destination = match self.resolve_dir_input() {
            Ok(path) => path,
            Err(err) => {
                self.log(format!("destination rejected: {err}"));
                self.mode = Mode::Normal;
                self.input.clear();
                return;
            }
        };
        let kind = if move_items {
            TransferKind::Cut
        } else {
            TransferKind::Copy
        };
        let operation = PendingFileOperation {
            kind,
            paths: self.operation_targets(),
            destination,
            source: FileOperationSource::Direct,
            label: if move_items { "moved" } else { "copied" },
            allow_replace: !move_items,
        };
        self.start_file_operation(operation);
    }

    fn start_file_operation(&mut self, operation: PendingFileOperation) {
        let conflicts = self.conflict_targets(&operation);
        if !conflicts.is_empty() {
            if !operation.allow_replace {
                self.execute_file_operation(operation, false, true);
                return;
            }
            self.log(format!("{} conflict(s)", conflicts.len()));
            self.pending_file_operation = Some(operation);
            self.mode = Mode::Conflict;
            self.input.clear();
            return;
        }
        self.execute_file_operation(operation, false, false);
    }

    fn submit_conflict(&mut self) {
        let Some(operation) = self.pending_file_operation.take() else {
            self.mode = Mode::Normal;
            self.input.clear();
            return;
        };
        let decision = self.input.trim().to_ascii_lowercase();
        match decision.as_str() {
            "replace" if operation.allow_replace => {
                self.execute_file_operation(operation, true, false);
            }
            "replace" => {
                self.pending_file_operation = Some(operation);
                self.log("replace is only available for copy operations");
            }
            "skip" => {
                self.execute_file_operation(operation, false, true);
            }
            _ => {
                self.pending_file_operation = Some(operation);
                self.log("type replace or skip to resolve conflict");
            }
        }
        if self.mode == Mode::Conflict {
            self.input.clear();
        }
    }

    fn execute_file_operation(
        &mut self,
        operation: PendingFileOperation,
        replace_existing: bool,
        skip_conflicts: bool,
    ) {
        let mut completed = 0;
        let mut skipped = 0;
        let mut failed_cut_paths = Vec::new();
        for source in &operation.paths {
            let target = match destination_for_paste(source, &operation.destination) {
                Ok(target) => target,
                Err(err) => {
                    self.log(format!("{} skipped: {err}", operation.label));
                    continue;
                }
            };
            if skip_conflicts && target_exists(&target) {
                skipped += 1;
                if operation.kind == TransferKind::Cut
                    && operation.source == FileOperationSource::TransferBuffer
                {
                    failed_cut_paths.push(source.clone());
                }
                continue;
            }
            let result = match operation.kind {
                TransferKind::Copy => copy_path(source, &target, replace_existing),
                TransferKind::Cut => rename_path(source, &target),
            };
            match result {
                Ok(()) => completed += 1,
                Err(err) => {
                    if operation.kind == TransferKind::Cut
                        && operation.source == FileOperationSource::TransferBuffer
                    {
                        failed_cut_paths.push(source.clone());
                    }
                    self.log(format!("{} skipped: {err}", operation.label));
                }
            }
        }
        if skipped > 0 {
            self.log(format!("skipped {skipped} existing item(s)"));
        }
        self.log(format!("{} {completed} item(s)", operation.label));
        if operation.kind == TransferKind::Cut
            && operation.source == FileOperationSource::TransferBuffer
        {
            self.transfer_buffer = if failed_cut_paths.is_empty() {
                None
            } else {
                Some(TransferBuffer {
                    kind: TransferKind::Cut,
                    paths: failed_cut_paths,
                })
            };
        }
        self.mode = Mode::Normal;
        self.input.clear();
        self.reload();
    }

    fn conflict_targets(&self, operation: &PendingFileOperation) -> Vec<PathBuf> {
        operation
            .paths
            .iter()
            .filter_map(|source| destination_for_paste(source, &operation.destination).ok())
            .filter(|target| target_exists(target))
            .collect()
    }

    fn submit(&mut self) {
        match self.mode {
            Mode::Filter => {
                self.filter = self.input.clone();
                self.mode = Mode::Normal;
                self.reload();
            }
            Mode::PreviewSearch => self.execute_preview_search(),
            Mode::Preview => {}
            Mode::Goto => self.submit_goto(),
            Mode::Rename => self.submit_rename(),
            Mode::CopyTo => self.copy_to_destination(false),
            Mode::MoveTo => self.copy_to_destination(true),
            Mode::ConfirmTrash => {
                if self.input == "trash" {
                    self.submit_trash();
                } else {
                    self.log("type trash then Enter to move to .tersh-trash");
                }
            }
            Mode::ConfirmDelete => {
                if self.input == "delete" {
                    self.submit_delete();
                } else {
                    self.log("type delete then Enter for permanent delete");
                }
            }
            Mode::Conflict => self.submit_conflict(),
            Mode::Help | Mode::Message => self.mode = Mode::Normal,
            Mode::Normal => {}
        }
    }

    fn submit_rename(&mut self) {
        let Some(entry) = self.focused() else {
            self.mode = Mode::Normal;
            return;
        };
        let new_name = self.input.clone();
        if let Err(err) = validate_file_name(&new_name) {
            self.log(format!("rename rejected: {err}"));
            self.mode = Mode::Normal;
            self.input.clear();
            return;
        }
        let target = self.cwd.join(new_name);
        match rename_path(&entry.path, &target) {
            Ok(()) => self.log("renamed item"),
            Err(err) => self.log(format!("rename failed: {err}")),
        }
        self.mode = Mode::Normal;
        self.input.clear();
        self.reload();
    }

    fn submit_goto(&mut self) {
        match self.resolve_dir_input() {
            Ok(path) => {
                self.cwd = path;
                self.selected.clear();
                self.filter.clear();
                self.mode = Mode::Normal;
                self.input.clear();
                self.reload();
            }
            Err(err) => {
                self.log(format!("goto failed: {err}"));
                self.mode = Mode::Normal;
                self.input.clear();
            }
        }
    }

    fn submit_trash(&mut self) {
        let targets = self.operation_targets();
        let mut moved = 0;
        for target in targets {
            match trash_path(&target, &self.work_root) {
                Ok(_) => moved += 1,
                Err(err) => self.log(format!("trash failed: {err}")),
            }
        }
        self.log(format!("trashed {moved} item(s)"));
        self.mode = Mode::Normal;
        self.input.clear();
        self.selected.clear();
        self.reload();
    }

    fn submit_delete(&mut self) {
        let targets = self.operation_targets();
        let mut deleted = 0;
        for target in targets {
            match permanent_delete(&target, &self.work_root) {
                Ok(_) => deleted += 1,
                Err(err) => self.log(format!("delete failed: {err}")),
            }
        }
        self.log(format!("deleted {deleted} item(s)"));
        self.mode = Mode::Normal;
        self.input.clear();
        self.selected.clear();
        self.reload();
    }

    fn handle_input(&mut self, ch: char) {
        match self.mode {
            Mode::Filter
            | Mode::Goto
            | Mode::PreviewSearch
            | Mode::Rename
            | Mode::CopyTo
            | Mode::MoveTo
            | Mode::ConfirmTrash
            | Mode::ConfirmDelete
            | Mode::Conflict => {
                self.input.push(ch);
                if self.mode == Mode::Filter {
                    self.filter = self.input.clone();
                    self.refresh_visible_entries();
                    self.preview_offset = 0;
                }
            }
            _ => {}
        }
    }

    fn cancel(&mut self) {
        self.pending_g = false;
        self.pending_y = false;
        match self.mode {
            Mode::Normal => {
                self.selected.clear();
                self.input.clear();
            }
            Mode::PreviewSearch => {
                self.mode = Mode::Preview;
                self.input.clear();
            }
            _ => {
                self.mode = Mode::Normal;
                self.input.clear();
            }
        }
    }

    fn operation_targets(&self) -> Vec<PathBuf> {
        if !self.selected.is_empty() {
            self.selected.iter().cloned().collect()
        } else {
            self.focused()
                .map(|entry| vec![entry.path.clone()])
                .unwrap_or_default()
        }
    }

    fn focused(&self) -> Option<&FileEntry> {
        self.entries.get(self.cursor)
    }

    fn focus_raw_name(&mut self, name: &OsStr) {
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.raw_name.as_os_str() == name)
        {
            self.cursor = index;
            self.update_preview();
            self.preview_offset = 0;
        }
    }

    fn copy_focused_name(&mut self) {
        if let Some(entry) = self.focused() {
            self.copy_text(entry.name.clone(), "copied file name");
        }
    }

    fn copy_focused_relative_path(&mut self) {
        if let Some(entry) = self.focused() {
            let text = entry
                .path
                .strip_prefix(&self.cwd)
                .unwrap_or(&entry.path)
                .display()
                .to_string();
            self.copy_text(text, "copied relative path");
        }
    }

    fn copy_focused_absolute_path(&mut self) {
        if let Some(entry) = self.focused() {
            self.copy_text(entry.path.display().to_string(), "copied absolute path");
        }
    }

    fn copy_text(&mut self, text: String, log: &'static str) {
        self.clipboard_text = Some(text.clone());
        self.last_clipboard_text = Some(text);
        self.log(log);
    }

    fn resolve_dir_input(&self) -> Result<PathBuf> {
        let path = expand_path(&self.input);
        let path = if path.is_absolute() {
            path
        } else {
            self.cwd.join(path)
        };
        let canonical = path.canonicalize()?;
        if canonical.is_dir() {
            Ok(canonical)
        } else {
            anyhow::bail!("not a directory: {}", canonical.display())
        }
    }

    fn retain_visible_selection(&mut self) {
        let visible = self
            .entries
            .iter()
            .map(|entry| entry.path.clone())
            .collect::<BTreeSet<_>>();
        self.selected.retain(|path| visible.contains(path));
    }

    fn log(&mut self, message: impl Into<String>) {
        self.logs.push(message.into());
        if self.logs.len() > 6 {
            self.logs.remove(0);
        }
    }
}

pub fn run(path: PathBuf) -> Result<()> {
    run_with_options(path, RunOptions::default())
}

pub fn run_with_options(path: PathBuf, options: RunOptions) -> Result<()> {
    let output = if options.print_cwd {
        TerminalOutput::Stderr
    } else {
        TerminalOutput::Stdout
    };
    let final_cwd = run_tui(path, output)?;
    if options.print_cwd {
        println!("{}", final_cwd.display());
    }
    Ok(())
}

fn run_tui(path: PathBuf, output: TerminalOutput) -> Result<PathBuf> {
    let _guard = TerminalGuard::enter(output)?;
    let backend = CrosstermBackend::new(output.writer());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut app = App::new_with_output(path, output)?;
    let mut dirty = true;

    while !app.should_quit() {
        if dirty {
            terminal.draw(|frame| crate::ui::draw(frame, &app))?;
            dirty = false;
        }
        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => {
                    app.handle_key(key);
                    if let Some(text) = app.take_clipboard_text() {
                        crate::clipboard::write_clipboard(&mut output.writer(), &text)?;
                    }
                    dirty = true;
                }
                Event::Resize(_, _) => {
                    terminal.clear()?;
                    dirty = true;
                }
                _ => {}
            }
        }
    }
    Ok(app.cwd().to_path_buf())
}

fn key_to_command(key: KeyEvent) -> Option<Command> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => Some(Command::ForceQuit),
            KeyCode::Char('g') => Some(Command::Cancel),
            KeyCode::Char('d') => Some(Command::HalfDown),
            KeyCode::Char('u') => Some(Command::HalfUp),
            _ => None,
        };
    }
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Command::Down),
        KeyCode::Char('k') | KeyCode::Up => Some(Command::Up),
        KeyCode::PageDown => Some(Command::HalfDown),
        KeyCode::PageUp => Some(Command::HalfUp),
        KeyCode::Home => Some(Command::First),
        KeyCode::End => Some(Command::Last),
        KeyCode::Char('G') => Some(Command::Last),
        KeyCode::Char('h') | KeyCode::Backspace => Some(Command::Parent),
        KeyCode::Char('l') | KeyCode::Enter => Some(Command::Open),
        KeyCode::Char(':') => Some(Command::OpenGoto),
        KeyCode::Char('/') => Some(Command::OpenFilter),
        KeyCode::Char('.') => Some(Command::ToggleHidden),
        KeyCode::Char(' ') => Some(Command::ToggleSelect),
        KeyCode::Char('a') => Some(Command::SelectAll),
        KeyCode::Char('A') => Some(Command::ClearSelection),
        KeyCode::Char('x') => Some(Command::Cut),
        KeyCode::Char('p') => Some(Command::Paste),
        KeyCode::Char('c') => Some(Command::CopyTo),
        KeyCode::Char('m') => Some(Command::MoveTo),
        KeyCode::Char('n') => Some(Command::Rename),
        KeyCode::Char('d') => Some(Command::Trash),
        KeyCode::Char('D') => Some(Command::PermanentDelete),
        KeyCode::Char('r') => Some(Command::Refresh),
        KeyCode::Char('s') => Some(Command::CycleSort),
        KeyCode::Char('S') => Some(Command::ReverseSort),
        KeyCode::Char('e') => Some(Command::Edit),
        KeyCode::Char('?') => Some(Command::OpenHelp),
        KeyCode::Char('q') => Some(Command::Quit),
        KeyCode::Char('Q') => Some(Command::ForceQuit),
        KeyCode::Char(ch) => Some(Command::Input(ch)),
        _ => None,
    }
}

fn preview_signature(path: &Path) -> Option<PreviewSignature> {
    let metadata = fs::symlink_metadata(path).ok()?;
    let kind = if metadata.is_dir() {
        FileKind::Directory
    } else if metadata.is_file() {
        FileKind::File
    } else if metadata.file_type().is_symlink() {
        FileKind::Symlink
    } else {
        FileKind::Other
    };

    let symlink_target = if kind == FileKind::Symlink {
        fs::read_link(path).ok()
    } else {
        None
    };

    let sample_hash = if kind == FileKind::File {
        Some(preview_file_hash(path)?)
    } else {
        None
    };

    Some(PreviewSignature {
        len: metadata.len(),
        modified: metadata.modified().ok(),
        kind,
        symlink_target,
        sample_hash,
    })
}

fn target_exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

fn preview_cache_bytes(preview: &Preview) -> usize {
    preview.path.as_os_str().len() + preview.lines.iter().map(|line| line.len()).sum::<usize>()
}

fn preview_file_hash(path: &Path) -> Option<u64> {
    let mut file = open_hash_file_no_follow(path)?;
    let metadata = file.metadata().ok()?;
    if !metadata.file_type().is_file() {
        return None;
    }
    let mut reader = std::io::Read::by_ref(&mut file).take(PREVIEW_SIGNATURE_HASH_LIMIT);
    let mut buffer = [0; 64 * 1024];
    let mut hasher = DefaultHasher::new();
    metadata.len().hash(&mut hasher);
    loop {
        let read = reader.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        buffer[..read].hash(&mut hasher);
    }
    Some(hasher.finish())
}

#[cfg(unix)]
fn open_hash_file_no_follow(path: &Path) -> Option<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;

    fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .ok()
}

#[cfg(not(unix))]
fn open_hash_file_no_follow(path: &Path) -> Option<fs::File> {
    fs::File::open(path).ok()
}

fn validate_editor_target(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to inspect {}", crate::fs_core::display_path(path)))?;
    if !metadata.file_type().is_file() {
        anyhow::bail!("not a regular file");
    }
    let file = open_editor_target_no_follow(path)?;
    let metadata = file.metadata().with_context(|| {
        format!(
            "failed to inspect opened {}",
            crate::fs_core::display_path(path)
        )
    })?;
    if !metadata.file_type().is_file() {
        anyhow::bail!("not a regular file");
    }
    Ok(())
}

#[cfg(unix)]
fn open_editor_target_no_follow(path: &Path) -> Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;

    fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .with_context(|| format!("failed to open {}", crate::fs_core::display_path(path)))
}

#[cfg(not(unix))]
fn open_editor_target_no_follow(path: &Path) -> Result<fs::File> {
    fs::File::open(path)
        .with_context(|| format!("failed to open {}", crate::fs_core::display_path(path)))
}

fn kind_rank(kind: FileKind) -> u8 {
    match kind {
        FileKind::Directory => 0,
        FileKind::Symlink => 1,
        FileKind::File => 2,
        FileKind::Other => 3,
    }
}

fn expand_path(input: &str) -> PathBuf {
    if input == "~" {
        return expand_user_home("~").unwrap_or_else(|| PathBuf::from(input));
    }
    if let Some(rest) = input.strip_prefix("~/") {
        return expand_user_home("~")
            .unwrap_or_else(|| PathBuf::from("~"))
            .join(rest);
    }
    if let Some((user, rest)) = input.split_once('/')
        && user.starts_with('~')
        && user.len() > 1
    {
        if let Some(home) = expand_user_home(user) {
            return home.join(rest);
        }
    }
    if input.starts_with('~')
        && input.len() > 1
        && let Some(home) = expand_user_home(input)
    {
        return home;
    }
    PathBuf::from(input)
}

#[cfg(unix)]
fn expand_user_home(user: &str) -> Option<PathBuf> {
    let name = user.strip_prefix('~')?;
    let (c_name, is_current) = if name.is_empty() {
        (None, true)
    } else {
        (Some(CString::new(name).ok()?), false)
    };

    // SAFETY: `libc::getpwuid` and `libc::getpwnam` return pointers that are valid until next call.
    unsafe {
        let entry = if is_current {
            libc::getpwuid(libc::geteuid())
        } else {
            libc::getpwnam(
                c_name
                    .as_ref()
                    .map_or(std::ptr::null(), |value| value.as_ptr()),
            )
        };
        if entry.is_null() {
            return None;
        }
        let home = CStr::from_ptr((*entry).pw_dir);
        Some(PathBuf::from(home.to_string_lossy().to_string()))
    }
}

#[cfg(not(unix))]
fn expand_user_home(user: &str) -> Option<PathBuf> {
    if user != "~" {
        return None;
    }
    std::env::var_os("HOME").map(PathBuf::from)
}

fn parse_command(input: String) -> Option<Vec<String>> {
    enum State {
        Normal,
        Single,
        Double,
    }
    let mut state = State::Normal;
    let mut args = Vec::new();
    let mut current = String::new();
    let mut arg_started = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match state {
            State::Normal => match ch {
                '\'' => {
                    arg_started = true;
                    state = State::Single;
                }
                '"' => {
                    arg_started = true;
                    state = State::Double;
                }
                '\\' => {
                    if let Some(next) = chars.next() {
                        arg_started = true;
                        current.push(next);
                    }
                }
                c if c.is_whitespace() => {
                    if arg_started {
                        args.push(std::mem::take(&mut current));
                        arg_started = false;
                    }
                }
                c => {
                    arg_started = true;
                    current.push(c);
                }
            },
            State::Single => match ch {
                '\'' => state = State::Normal,
                _ => {
                    arg_started = true;
                    current.push(ch);
                }
            },
            State::Double => match ch {
                '"' => state = State::Normal,
                '\\' => {
                    if let Some(next) = chars.next() {
                        arg_started = true;
                        match next {
                            '\n' => current.push('\n'),
                            '"' | '\\' => current.push(next),
                            _ => {
                                current.push('\\');
                                current.push(next);
                            }
                        }
                    }
                }
                _ => {
                    arg_started = true;
                    current.push(ch);
                }
            },
        }
    }

    if arg_started {
        args.push(current);
    }

    match state {
        State::Normal if !args.is_empty() => Some(args),
        _ => None,
    }
}

fn editor_command_parts() -> Result<Vec<String>> {
    let Some(command) = std::env::var_os("VISUAL").or_else(|| std::env::var_os("EDITOR")) else {
        return Ok(vec!["nano".to_string()]);
    };
    let command = command.to_string_lossy().to_string();
    parse_command(command).ok_or_else(|| anyhow::anyhow!("invalid editor command"))
}

fn is_cancel_key(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('g') | KeyCode::Char('G'))
}

#[derive(Debug, Clone, Copy)]
enum TerminalOutput {
    Stdout,
    Stderr,
}

impl TerminalOutput {
    fn writer(self) -> Box<dyn Write> {
        match self {
            Self::Stdout => Box::new(io::stdout()),
            Self::Stderr => Box::new(io::stderr()),
        }
    }
}

struct TerminalGuard {
    output: TerminalOutput,
}

impl TerminalGuard {
    fn enter(output: TerminalOutput) -> Result<Self> {
        enable_raw_mode()?;
        if let Err(err) = execute!(output.writer(), EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(err.into());
        }
        Ok(Self { output })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.output.writer(), LeaveAlternateScreen);
    }
}

struct TerminalSuspension {
    output: TerminalOutput,
    restored: bool,
}

impl TerminalSuspension {
    fn suspend(output: TerminalOutput) -> Result<Self> {
        execute!(output.writer(), LeaveAlternateScreen)?;
        if let Err(err) = disable_raw_mode() {
            let _ = execute!(output.writer(), EnterAlternateScreen);
            return Err(err.into());
        }
        Ok(Self {
            output,
            restored: false,
        })
    }

    fn restore(&mut self) -> Result<()> {
        enable_raw_mode()?;
        if let Err(err) = execute!(self.output.writer(), EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(err.into());
        }
        self.restored = true;
        Ok(())
    }
}

impl Drop for TerminalSuspension {
    fn drop(&mut self) {
        if !self.restored && enable_raw_mode().is_ok() {
            let _ = execute!(self.output.writer(), EnterAlternateScreen);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Seek, SeekFrom, Write};

    #[test]
    fn preview_hash_covers_bytes_after_first_four_kib() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("body.txt");
        let body = vec![b'a'; 8 * 1024];
        fs::write(&path, body).unwrap();
        let first = preview_file_hash(&path).unwrap();

        let mut file = fs::OpenOptions::new().write(true).open(&path).unwrap();
        file.seek(SeekFrom::Start(5 * 1024)).unwrap();
        file.write_all(b"b").unwrap();

        assert_ne!(first, preview_file_hash(&path).unwrap());
    }

    #[test]
    fn preview_cache_retains_multiple_recent_entries() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "a").unwrap();
        fs::write(dir.path().join("b.txt"), "b").unwrap();
        fs::write(dir.path().join("c.txt"), "c").unwrap();
        let mut app = App::new(dir.path().to_path_buf()).unwrap();

        app.handle_command(Command::Down);
        app.handle_command(Command::Down);

        assert!(app.preview_cache.len() >= 2);
    }

    #[test]
    fn preview_cache_replaces_stale_entry_for_same_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        fs::write(&path, "a").unwrap();
        let mut app = App::new(dir.path().to_path_buf()).unwrap();
        let cached_path = app.cwd().join("a.txt");

        fs::write(&path, "b").unwrap();
        app.handle_command(Command::Refresh);

        assert_eq!(
            app.preview_cache
                .iter()
                .filter(|cache| cache.path == cached_path)
                .count(),
            1
        );
        assert!(app.preview.lines.iter().any(|line| line.contains("b")));
    }

    #[test]
    fn preview_cache_enforces_lru_limit() {
        let dir = tempfile::tempdir().unwrap();
        for index in 0..(PREVIEW_CACHE_LIMIT + 5) {
            fs::write(dir.path().join(format!("item-{index:02}.txt")), "x").unwrap();
        }
        let mut app = App::new(dir.path().to_path_buf()).unwrap();

        for _ in 0..(PREVIEW_CACHE_LIMIT + 4) {
            app.handle_command(Command::Down);
        }

        assert_eq!(app.preview_cache.len(), PREVIEW_CACHE_LIMIT);
        let newest_cached = format!("item-{:02}.txt", PREVIEW_CACHE_LIMIT + 4);
        assert!(!app.preview_cache.iter().any(|cache| {
            cache
                .path
                .file_name()
                .is_some_and(|name| name == "item-00.txt")
        }));
        assert!(app.preview_cache.iter().any(|cache| {
            cache
                .path
                .file_name()
                .is_some_and(|name| name == newest_cached.as_str())
        }));
    }

    #[test]
    fn parse_command_rejects_unclosed_quotes() {
        assert_eq!(parse_command("\"vim".to_string()), None);
    }

    #[test]
    fn parse_command_preserves_empty_quoted_arguments() {
        assert_eq!(
            parse_command("emacsclient -a \"\"".to_string()),
            Some(vec![
                "emacsclient".to_string(),
                "-a".to_string(),
                String::new()
            ])
        );
    }

    #[cfg(unix)]
    #[test]
    fn expand_path_supports_named_user_without_trailing_slash() {
        let home = expand_user_home("~").unwrap();
        let user = unsafe {
            let entry = libc::getpwuid(libc::geteuid());
            assert!(!entry.is_null());
            CStr::from_ptr((*entry).pw_name)
                .to_string_lossy()
                .to_string()
        };

        assert_eq!(expand_path(&format!("~{user}")), home);
    }
}
