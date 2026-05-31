use crate::{
    fs_core::{FileEntry, FileKind, read_dir_entries},
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
    collections::BTreeSet,
    ffi::{CStr, CString},
    fs,
    hash::{Hash, Hasher},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
    time::{Duration, SystemTime},
};

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
    preview_cache: Option<CachedPreview>,
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
}

impl App {
    pub fn new(path: PathBuf) -> Result<Self> {
        Self::new_with_output(path, TerminalOutput::Stdout)
    }

    fn new_with_output(path: PathBuf, terminal_output: TerminalOutput) -> Result<Self> {
        let cwd = path
            .canonicalize()
            .with_context(|| format!("failed to resolve {}", path.display()))?;
        let mut app = Self {
            work_root: cwd.clone(),
            cwd,
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
            show_hidden: false,
            filter: String::new(),
            input: String::new(),
            logs: Vec::new(),
            pending_g: false,
            pending_y: false,
            clipboard_text: None,
            last_clipboard_text: None,
            sort_key: SortKey::Kind,
            sort_reverse: false,
            preview_cache: None,
        };
        app.reload();
        Ok(app)
    }

    pub fn for_test() -> Self {
        let cwd = PathBuf::from("/tmp/tersh-test");
        Self {
            work_root: cwd.clone(),
            cwd,
            entries: vec![
                FileEntry {
                    path: PathBuf::from("/tmp/tersh-test/src"),
                    raw_name: "src".into(),
                    name: "src".to_string(),
                    kind: FileKind::Directory,
                    size: 0,
                    readonly: false,
                    modified: None,
                    symlink_target: None,
                },
                FileEntry {
                    path: PathBuf::from("/tmp/tersh-test/README.md"),
                    raw_name: "README.md".into(),
                    name: "README.md".to_string(),
                    kind: FileKind::File,
                    size: 128,
                    readonly: false,
                    modified: None,
                    symlink_target: None,
                },
            ],
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
            preview_cache: None,
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
                    self.reload();
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

    pub fn take_clipboard_text(&mut self) -> Option<String> {
        self.clipboard_text.take()
    }

    pub fn force_cwd_for_test(&mut self, cwd: PathBuf) {
        self.cwd = cwd;
        self.reload();
    }

    fn reload(&mut self) {
        match read_dir_entries(&self.cwd, self.show_hidden, &self.filter) {
            Ok(entries) => {
                self.entries = entries;
                self.sort_entries();
                self.retain_visible_selection();
                self.cursor = self.cursor.min(self.entries.len().saturating_sub(1));
                self.update_preview();
                self.preview_offset = 0;
            }
            Err(err) => {
                self.log(format!("error: {err}"));
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
        if let (Some(signature), Some(cache)) = (&signature, &self.preview_cache)
            && cache.path == path
            && &cache.signature == signature
        {
            return cache.preview.clone();
        }
        let preview = match preview_file(path) {
            Ok(preview) => preview,
            Err(err) => Preview::message(
                path.to_path_buf(),
                PreviewKind::Error,
                crate::fs_core::escape_display(&err.to_string()),
            ),
        };
        if let Some(signature) = signature {
            self.preview_cache = Some(CachedPreview {
                path: path.to_path_buf(),
                signature,
                preview: preview.clone(),
            });
        }
        preview
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
        let editor_command = std::env::var_os("VISUAL")
            .or_else(|| std::env::var_os("EDITOR"))
            .unwrap_or_else(|| "nano".into())
            .to_string_lossy()
            .to_string();
        let editor_parts = parse_command(editor_command).unwrap_or_else(|| vec!["nano".to_string()]);
        let Some((editor, args)) = editor_parts.split_first() else {
            return Err(anyhow::anyhow!("no editor command resolved"));
        };

        let mut output = self.terminal_output.writer();
        execute!(output, LeaveAlternateScreen)?;
        disable_raw_mode()?;

        let status = ProcessCommand::new(editor)
            .args(args)
            .arg(path)
            .status();
        let status = match status {
            Ok(status) => status,
            Err(err) => {
                let mut output = self.terminal_output.writer();
                let _ = execute!(output, EnterAlternateScreen);
                let _ = enable_raw_mode();
                return Err(err.into());
            }
        };

        let mut output = self.terminal_output.writer();
        execute!(output, EnterAlternateScreen)?;
        enable_raw_mode()?;
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
        let key = self.sort_key;
        self.entries.sort_by(|a, b| {
            let ordering = match key {
                SortKey::Kind => kind_rank(a.kind)
                    .cmp(&kind_rank(b.kind))
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
                SortKey::Size => a
                    .size
                    .cmp(&b.size)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
                SortKey::Modified => a
                    .modified
                    .cmp(&b.modified)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
            };
            if self.sort_reverse {
                ordering.reverse()
            } else {
                ordering
            }
        });
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
        let mut copied = 0;
        for source in &buffer.paths {
            let result =
                destination_for_paste(source, &self.cwd).and_then(|target| match buffer.kind {
                    TransferKind::Copy => copy_path(source, &target, false),
                    TransferKind::Cut => rename_path(source, &target),
                });
            match result {
                Ok(()) => copied += 1,
                Err(err) => self.log(format!("paste skipped: {err}")),
            }
        }
        self.log(format!("pasted {copied} item(s)"));
        if buffer.kind == TransferKind::Cut {
            self.transfer_buffer = None;
        }
        self.reload();
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
        let targets = self.operation_targets();
        let mut completed = 0;
        for source in targets {
            let result = destination_for_paste(&source, &destination).and_then(|target| {
                if move_items {
                    rename_path(&source, &target)
                } else {
                    copy_path(&source, &target, false)
                }
            });
            match result {
                Ok(()) => completed += 1,
                Err(err) => self.log(format!("copy/move skipped: {err}")),
            }
        }
        self.log(format!(
            "{} {completed} item(s)",
            if move_items { "moved" } else { "copied" }
        ));
        self.mode = Mode::Normal;
        self.input.clear();
        self.reload();
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
            | Mode::ConfirmDelete => {
                self.input.push(ch);
                if self.mode == Mode::Filter {
                    self.filter = self.input.clone();
                    self.reload();
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

    while !app.should_quit() {
        terminal.draw(|frame| crate::ui::draw(frame, &app))?;
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            app.handle_key(key);
            if let Some(text) = app.take_clipboard_text() {
                crate::clipboard::write_clipboard(&mut output.writer(), &text)?;
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
        preview_file_hash(path)
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

fn preview_file_hash(path: &Path) -> Option<u64> {
    let mut file = fs::File::open(path).ok()?;
    let mut buffer = [0; 4096];
    let read = file.read(&mut buffer).ok()?;
    let bytes = &buffer[..read];
    let mut hasher = DefaultHasher::new();
    metadata_size(path).hash(&mut hasher);
    bytes.hash(&mut hasher);
    Some(hasher.finish())
}

fn metadata_size(path: &Path) -> u64 {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or_default()
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
            libc::getpwnam(c_name.map_or(std::ptr::null(), |value| value.as_ptr()))
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
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match state {
            State::Normal => match ch {
                '\'' => state = State::Single,
                '"' => state = State::Double,
                '\\' => {
                    if let Some(next) = chars.next() {
                        current.push(next);
                    }
                }
                c if c.is_whitespace() => {
                    if !current.is_empty() {
                        args.push(std::mem::take(&mut current));
                    }
                }
                c => current.push(c),
            },
            State::Single => match ch {
                '\'' => state = State::Normal,
                _ => current.push(ch),
            },
            State::Double => match ch {
                '"' => state = State::Normal,
                '\\' => {
                    if let Some(next) = chars.next() {
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
                _ => current.push(ch),
            },
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    match state {
        State::Normal if !args.is_empty() => Some(args),
        _ => None,
    }
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
