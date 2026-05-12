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
    collections::BTreeSet,
    io,
    process::Command as ProcessCommand,
    path::{Path, PathBuf},
    time::Duration,
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
    Cancel,
    Quit,
    ForceQuit,
    Input(char),
    Edit,
    Backspace,
    Submit,
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
    show_hidden: bool,
    filter: String,
    input: String,
    logs: Vec<String>,
    pending_g: bool,
    pending_y: bool,
    clipboard_text: Option<String>,
    last_clipboard_text: Option<String>,
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

impl App {
    pub fn new(path: PathBuf) -> Result<Self> {
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
            show_hidden: false,
            filter: String::new(),
            input: String::new(),
            logs: Vec::new(),
            pending_g: false,
            pending_y: false,
            clipboard_text: None,
            last_clipboard_text: None,
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
            show_hidden: false,
            filter: String::new(),
            input: String::new(),
            logs: vec!["ready".to_string()],
            pending_g: false,
            pending_y: false,
            clipboard_text: None,
            last_clipboard_text: None,
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
                KeyCode::Esc => Some(Command::Cancel),
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
                KeyCode::Esc => Some(Command::Cancel),
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
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('?') => {
                    self.handle_command(Command::Cancel)
                }
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
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => Some(Command::Cancel),
                KeyCode::Char('j') | KeyCode::Down => Some(Command::Down),
                KeyCode::Char('k') | KeyCode::Up => Some(Command::Up),
                KeyCode::PageDown | KeyCode::Char(' ') => Some(Command::HalfDown),
                KeyCode::PageUp => Some(Command::HalfUp),
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
        if let Some(entry) = self.focused() {
            self.preview = match preview_file(&entry.path) {
                Ok(preview) => preview,
                Err(err) => {
                    Preview::message(entry.path.clone(), PreviewKind::Error, err.to_string())
                }
            };
        } else {
            self.preview =
                Preview::message(self.cwd.clone(), PreviewKind::Empty, "No file selected");
        }
    }

    fn scroll_preview(&mut self, delta: isize) {
        if self.preview.lines.is_empty() {
            self.preview_offset = 0;
            return;
        }
        if delta.is_negative() {
            self.preview_offset = self
                .preview_offset
                .saturating_sub(delta.unsigned_abs());
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
        if entry.kind == FileKind::Directory {
            self.log("can only edit files");
            return;
        }
        let path = entry.path.clone();
        match self.launch_editor(&path) {
            Ok(()) => {
                self.log(format!("saved in nano: {}", path.display()));
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
        let mut stdout = io::stdout();
        execute!(stdout, LeaveAlternateScreen)?;
        disable_raw_mode()?;
        let status = ProcessCommand::new("nano").arg(path).status();
        let status = match status {
            Ok(status) => status,
            Err(err) => {
                let _ = execute!(stdout, EnterAlternateScreen);
                let _ = enable_raw_mode();
                return Err(err.into());
            }
        };
        execute!(stdout, EnterAlternateScreen)?;
        enable_raw_mode()?;
        if !status.success() {
            return Err(anyhow::anyhow!("nano exited with {status}"));
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
        if let Some(path) = self.focused().map(|entry| entry.path.clone()) {
            if !self.selected.remove(&path) {
                self.selected.insert(path);
            }
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
    let _guard = TerminalGuard::enter()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut app = App::new(path)?;

    while !app.should_quit() {
        terminal.draw(|frame| crate::ui::draw(frame, &app))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
                if let Some(text) = app.take_clipboard_text() {
                    crate::clipboard::write_clipboard(&mut io::stdout(), &text)?;
                }
            }
        }
    }
    Ok(())
}

fn key_to_command(key: KeyEvent) -> Option<Command> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => Some(Command::ForceQuit),
            KeyCode::Char('d') => Some(Command::HalfDown),
            KeyCode::Char('u') => Some(Command::HalfUp),
            _ => None,
        };
    }
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Command::Down),
        KeyCode::Char('k') | KeyCode::Up => Some(Command::Up),
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
        KeyCode::Char('e') => Some(Command::Edit),
        KeyCode::Char('?') => Some(Command::OpenHelp),
        KeyCode::Esc => Some(Command::Cancel),
        KeyCode::Char('q') => Some(Command::Quit),
        KeyCode::Char('Q') => Some(Command::ForceQuit),
        KeyCode::Char(ch) => Some(Command::Input(ch)),
        _ => None,
    }
}

fn expand_path(input: &str) -> PathBuf {
    if input == "~" {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(input));
    }
    if let Some(rest) = input.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
            return home.join(rest);
        }
    }
    PathBuf::from(input)
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        if let Err(err) = execute!(io::stdout(), EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(err.into());
        }
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}
