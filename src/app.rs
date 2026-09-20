use crate::{
    fs_core::{FileEntry, FileKind, read_dir_entries_with_diagnostics},
    fs_ops::{destination_for_paste, rename_path, validate_file_name},
    preview::{Preview, PreviewKind, preview_file},
};
use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
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
    Jobs,
    Trash,
    ConfirmRestore,
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
    OpenActions,
    OpenJobs,
    CancelJob,
    OpenTrash,
    RestoreTrash,
    RefreshTrash,
}

macro_rules! command_actions {
    ($($variant:ident => $id:literal),* $(,)?) => {
        impl Command {
            pub fn action_id(&self) -> &'static str { match self { $(Self::$variant => $id,)* Self::Input(_) => "input" } }
            pub fn from_action(id: &str) -> Option<Self> { match id { $($id => Some(Self::$variant),)* _ => None } }
        }
    }
}
command_actions! {
    Down=>"down", Up=>"up", HalfDown=>"half_down", HalfUp=>"half_up", First=>"first", Last=>"last",
    Parent=>"parent", Open=>"open", OpenFilter=>"open_filter", OpenGoto=>"open_goto", OpenPreviewSearch=>"open_preview_search",
    ToggleHidden=>"toggle_hidden", ToggleSelect=>"toggle_select", SelectAll=>"select_all", ClearSelection=>"clear_selection",
    Copy=>"copy", Cut=>"cut", Paste=>"paste", CopyName=>"copy_name", CopyRelativePath=>"copy_relative_path", CopyAbsolutePath=>"copy_absolute_path",
    CopyTo=>"copy_to", MoveTo=>"move_to", Rename=>"rename", Trash=>"trash", PermanentDelete=>"permanent_delete", Refresh=>"refresh",
    OpenHelp=>"open_help", PreviewSearchNext=>"preview_search_next", PreviewSearchPrev=>"preview_search_prev",
    CycleSort=>"cycle_sort", ReverseSort=>"reverse_sort", Cancel=>"cancel", Quit=>"quit", ForceQuit=>"force_quit",
    Edit=>"edit", Backspace=>"backspace", Submit=>"submit", OpenActions=>"open_actions", OpenJobs=>"open_jobs",
    CancelJob=>"cancel_job", OpenTrash=>"open_trash", RestoreTrash=>"restore", RefreshTrash=>"refresh_trash",
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Kind,
    Size,
    Modified,
}

#[derive(Debug)]
pub struct App {
    keymap: std::sync::Arc<crate::keymap::Keymap>,
    key_state: crate::bindings::KeyState,
    help_offset: usize,
    help_context: String,
    active_job: Option<crate::jobs::JobHandle>,
    job_progress: Option<crate::jobs::JobProgress>,
    last_job: Option<crate::jobs::JobResult>,
    job_cut_revision: Option<u64>,
    transfer_revision: u64,
    exit_after_job: bool,
    trash_entries: Vec<crate::trash::TrashEntry>,
    trash_cursor: usize,
    trash_error: Option<String>,
    restore_pending: Option<PathBuf>,
    actions: Option<crate::actions::ActionMenu<Command>>,
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
    pending_rename: Option<BufferedPath>,
    pending_direct_targets: Option<Vec<BufferedPath>>,
    pending_destructive_targets: Option<Vec<BufferedPath>>,
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
    paths: Vec<BufferedPath>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileOperationSource {
    TransferBuffer,
    Direct,
}

#[derive(Debug, Clone)]
struct PendingFileOperation {
    kind: TransferKind,
    paths: Vec<BufferedPath>,
    destination: PathBuf,
    source: FileOperationSource,
    allow_replace: bool,
    approved_conflicts: Vec<BufferedPath>,
}

#[derive(Debug, Clone)]
struct BufferedPath {
    path: PathBuf,
    identity: PathIdentity,
}

impl BufferedPath {
    fn new(path: PathBuf) -> Result<Self> {
        let identity = capture_path_identity(&path)?;
        Ok(Self { path, identity })
    }

    fn from_entry(entry: &FileEntry) -> Self {
        Self {
            path: entry.path.clone(),
            identity: PathIdentity::from_entry(entry),
        }
    }

    fn ensure_current(&self) -> Result<()> {
        ensure_path_identity(&self.path, &self.identity)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PathIdentity {
    synthetic: bool,
    is_dir: bool,
    is_file: bool,
    is_symlink: bool,
    len: u64,
    modified: Option<SystemTime>,
    #[cfg(unix)]
    dev: u64,
    #[cfg(unix)]
    ino: u64,
}

impl PathIdentity {
    fn from_metadata(metadata: &fs::Metadata) -> Self {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;

        Self {
            synthetic: false,
            is_dir: metadata.is_dir(),
            is_file: metadata.is_file(),
            is_symlink: metadata.file_type().is_symlink(),
            len: metadata.len(),
            modified: metadata.modified().ok(),
            #[cfg(unix)]
            dev: metadata.dev(),
            #[cfg(unix)]
            ino: metadata.ino(),
        }
    }

    fn from_entry(entry: &FileEntry) -> Self {
        Self {
            synthetic: true,
            is_dir: entry.kind == FileKind::Directory,
            is_file: entry.kind == FileKind::File,
            is_symlink: entry.kind == FileKind::Symlink,
            len: entry.size,
            modified: entry.modified,
            #[cfg(unix)]
            dev: 0,
            #[cfg(unix)]
            ino: 0,
        }
    }
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
            keymap: std::sync::Arc::new(crate::keymap::Keymap::default()),
            key_state: crate::bindings::KeyState::default(),
            help_offset: 0,
            help_context: "files".into(),
            active_job: None,
            job_progress: None,
            last_job: None,
            job_cut_revision: None,
            transfer_revision: 0,
            exit_after_job: false,
            trash_entries: Vec::new(),
            trash_cursor: 0,
            trash_error: None,
            restore_pending: None,
            actions: None,
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
            pending_rename: None,
            pending_direct_targets: None,
            pending_destructive_targets: None,
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
            keymap: std::sync::Arc::new(crate::keymap::Keymap::default()),
            key_state: crate::bindings::KeyState::default(),
            help_offset: 0,
            help_context: "files".into(),
            active_job: None,
            job_progress: None,
            last_job: None,
            job_cut_revision: None,
            transfer_revision: 0,
            exit_after_job: false,
            trash_entries: Vec::new(),
            trash_cursor: 0,
            trash_error: None,
            restore_pending: None,
            actions: None,
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
            pending_rename: None,
            pending_direct_targets: None,
            pending_destructive_targets: None,
        }
    }

    pub fn apply(&mut self, command: Command) {
        match command {
            Command::Cancel => self.cancel(),
            Command::Quit => {
                if self.mode == Mode::Normal {
                    self.request_quit();
                } else {
                    self.mode = Mode::Normal;
                    self.input.clear();
                }
            }
            Command::ForceQuit => self.request_quit(),
            Command::OpenFilter => {
                self.mode = Mode::Filter;
                self.input = self.filter.clone();
            }
            Command::OpenGoto => {
                self.mode = Mode::Goto;
                self.input.clear();
            }
            Command::OpenHelp => {
                self.help_context = self.key_context().into();
                self.help_offset = 0;
                self.mode = Mode::Help;
            }
            Command::Trash => {
                self.begin_destructive_confirmation(Mode::ConfirmTrash);
            }
            Command::PermanentDelete => {
                self.begin_destructive_confirmation(Mode::ConfirmDelete);
            }
            _ => {}
        }
    }

    pub fn handle_command(&mut self, command: Command) {
        if self.active_job.is_some()
            && matches!(
                command,
                Command::Paste
                    | Command::CopyTo
                    | Command::MoveTo
                    | Command::Rename
                    | Command::Trash
                    | Command::PermanentDelete
                    | Command::Edit
                    | Command::RestoreTrash
            )
        {
            self.log("file job active; inspect jobs or cancel before another write");
            return;
        }
        if matches!(self.mode, Mode::Help | Mode::Jobs) {
            match command {
                Command::Down | Command::HalfDown => {
                    self.help_offset = self
                        .help_offset
                        .saturating_add(1)
                        .min(self.overlay_line_count().saturating_sub(1));
                    return;
                }
                Command::Up | Command::HalfUp => {
                    self.help_offset = self.help_offset.saturating_sub(1);
                    return;
                }
                Command::First => {
                    self.help_offset = 0;
                    return;
                }
                Command::Last => {
                    self.help_offset = self.overlay_line_count().saturating_sub(1);
                    return;
                }
                _ => {}
            }
        }
        if self.mode == Mode::Trash {
            match command {
                Command::Down => {
                    self.trash_cursor = self
                        .trash_cursor
                        .saturating_add(1)
                        .min(self.trash_entries.len().saturating_sub(1));
                    return;
                }
                Command::Up => {
                    self.trash_cursor = self.trash_cursor.saturating_sub(1);
                    return;
                }
                Command::First => {
                    self.trash_cursor = 0;
                    return;
                }
                Command::Last => {
                    self.trash_cursor = self.trash_entries.len().saturating_sub(1);
                    return;
                }
                _ => {}
            }
        }
        match command {
            Command::OpenActions => self.open_actions(),
            Command::OpenJobs => {
                self.help_offset = 0;
                self.mode = Mode::Jobs;
            }
            Command::CancelJob => {
                if let Some(job) = &self.active_job {
                    job.cancel();
                    self.log("cancellation requested; completed items are retained");
                }
            }
            Command::OpenTrash => {
                self.mode = Mode::Trash;
                self.load_trash();
            }
            Command::RefreshTrash => self.load_trash(),
            Command::RestoreTrash => {
                if let Some(entry) = self.trash_entries.get(self.trash_cursor) {
                    self.restore_pending = Some(entry.receipt_path.clone());
                    self.mode = Mode::ConfirmRestore;
                    self.input.clear();
                }
            }
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
                self.pending_direct_targets = Some(self.capture_operation_targets("copy"));
                self.mode = Mode::CopyTo;
                self.input.clear();
                self.pending_rename = None;
                self.pending_destructive_targets = None;
            }
            Command::MoveTo => {
                self.pending_direct_targets = Some(self.capture_operation_targets("move"));
                self.mode = Mode::MoveTo;
                self.input.clear();
                self.pending_rename = None;
                self.pending_destructive_targets = None;
            }
            Command::PreviewSearchNext => self.preview_search_next(),
            Command::PreviewSearchPrev => self.preview_search_prev(),
            Command::CycleSort => self.cycle_sort(),
            Command::ReverseSort => self.reverse_sort(),
            Command::Rename => {
                if let Some((path, name)) = self.focused().and_then(|entry| {
                    entry
                        .raw_name
                        .to_str()
                        .map(|name| (entry.path.clone(), name.to_string()))
                }) {
                    match BufferedPath::new(path) {
                        Ok(buffered) => {
                            self.pending_rename = Some(buffered);
                            self.pending_direct_targets = None;
                            self.pending_destructive_targets = None;
                            self.input = name;
                            self.mode = Mode::Rename;
                        }
                        Err(err) => self.log(format!("rename skipped: {err}")),
                    }
                } else if self.focused().is_some() {
                    self.log("rename unsupported for non-UTF-8 file name");
                }
            }
            Command::Trash => {
                self.begin_destructive_confirmation(Mode::ConfirmTrash);
            }
            Command::PermanentDelete => {
                self.begin_destructive_confirmation(Mode::ConfirmDelete);
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
        if key.kind == crossterm::event::KeyEventKind::Release {
            return;
        }
        let context = self.key_context();
        let matched = self.key_state.feed(&self.keymap, context, key);
        self.pending_y = self
            .key_state
            .pending
            .first()
            .is_some_and(|key| key.code == KeyCode::Char('y'));
        self.pending_g = self
            .key_state
            .pending
            .first()
            .is_some_and(|key| key.code == KeyCode::Char('g'));
        for ch in std::mem::take(&mut self.key_state.replay) {
            self.input_text(ch);
        }
        match matched {
            crate::keymap::KeyMatch::Action(action) => {
                if action == "force_quit" {
                    self.handle_command(Command::ForceQuit);
                    return;
                }
                if let Some(menu) = self.actions.as_mut() {
                    match menu.handle_action(&action) {
                        crate::actions::MenuResult::Pending => {}
                        crate::actions::MenuResult::Cancel => self.actions = None,
                        crate::actions::MenuResult::Run(command) => {
                            self.actions = None;
                            self.handle_command(command);
                        }
                    }
                } else if self.mode == Mode::Trash && action == "refresh" {
                    self.load_trash();
                } else if let Some(command) = Command::from_action(&action) {
                    self.handle_command(command);
                }
            }
            crate::keymap::KeyMatch::Pending => {}
            crate::keymap::KeyMatch::Unbound => {
                if let Some(ch) = crate::bindings::printable(key) {
                    self.input_text(ch);
                }
            }
        }
    }

    fn input_text(&mut self, ch: char) {
        if let Some(menu) = self.actions.as_mut() {
            menu.input_char(ch);
        } else {
            self.handle_input(ch);
        }
    }

    pub fn set_keymap(&mut self, map: crate::keymap::Keymap) {
        self.keymap = std::sync::Arc::new(map);
        self.key_state.clear();
        self.actions = None;
    }
    pub fn keymap(&self) -> &crate::keymap::Keymap {
        &self.keymap
    }
    pub fn key_context(&self) -> &'static str {
        if self.actions.is_some() {
            return "actions";
        }
        match self.mode {
            Mode::Normal => "files",
            Mode::Preview => "preview",
            Mode::Help | Mode::Message => "help",
            Mode::Jobs => "jobs",
            Mode::Trash => "trash",
            _ => "input",
        }
    }
    pub fn help_context(&self) -> &str {
        &self.help_context
    }
    pub fn help_offset(&self) -> usize {
        self.help_offset
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn actions(&self) -> Option<&crate::actions::ActionMenu<Command>> {
        self.actions.as_ref()
    }

    fn open_actions(&mut self) {
        use crate::actions::{Action, ActionMenu};
        let mut actions = vec![Action::new("Trash recovery", "u", Command::OpenTrash)];
        actions.push(Action::new("File jobs and results", "J", Command::OpenJobs));
        if self.active_job.is_some() {
            actions.push(Action::new(
                "Cancel active file job",
                "^X",
                Command::CancelJob,
            ));
        }
        if self.mode == Mode::Preview {
            actions.extend([
                Action::new("Find in preview", "/", Command::OpenPreviewSearch),
                Action::new("Next match", "n", Command::PreviewSearchNext),
                Action::new("Previous match", "N", Command::PreviewSearchPrev),
                Action::new("Top of preview", "gg", Command::First),
                Action::new("Bottom of preview", "G", Command::Last),
                Action::new("Close preview", "q", Command::Cancel),
            ]);
        } else {
            if self.copy_buffer_len() > 0 {
                actions.push(Action::new("Paste buffer", "p", Command::Paste));
            }
            if let Some(entry) = self.focused() {
                actions.push(Action::new(
                    if entry.kind == FileKind::Directory {
                        "Open directory"
                    } else {
                        "Preview item"
                    },
                    "Enter",
                    Command::Open,
                ));
                actions.extend([
                    Action::new("Mark / unmark item", "Space", Command::ToggleSelect),
                    Action::new("Copy items", "yy", Command::Copy),
                    Action::new("Cut items", "x", Command::Cut),
                    Action::new("Copy to directory", "c", Command::CopyTo),
                    Action::new("Move to directory", "m", Command::MoveTo),
                    Action::new("Rename item", "n", Command::Rename),
                    Action::new("Copy name", "yf", Command::CopyName),
                    Action::new("Copy relative path", "yr", Command::CopyRelativePath),
                    Action::new("Copy absolute path", "ya", Command::CopyAbsolutePath),
                ]);
            }
            actions.extend([
                Action::new("Filter files", "/", Command::OpenFilter),
                Action::new("Go to directory", ":", Command::OpenGoto),
                Action::new("Parent directory", "h", Command::Parent),
                Action::new("Cycle sort", "s", Command::CycleSort),
                Action::new("Reverse sort", "S", Command::ReverseSort),
                Action::new("Toggle hidden files", ".", Command::ToggleHidden),
                Action::new("Select all visible", "a", Command::SelectAll),
                Action::new("Clear selection", "A", Command::ClearSelection),
                Action::new("Refresh directory", "r", Command::Refresh),
                Action::new("Help", "?", Command::OpenHelp),
            ]);
            if self.operation_target_count() > 0 {
                actions.push(Action::new("Trash items (confirm)", "d", Command::Trash).dangerous());
                actions.push(
                    Action::new(
                        "Permanently delete (confirm)",
                        "D",
                        Command::PermanentDelete,
                    )
                    .dangerous(),
                );
            }
        }
        if self.focused().is_some_and(|e| e.kind == FileKind::File) {
            actions.push(Action::new("Edit file", "e", Command::Edit));
        }
        let context = self.key_context();
        for action in &mut actions {
            action.key = crate::bindings::label(&self.keymap, context, action.command.action_id())
                .unwrap_or_else(|| "unbound".into());
        }
        self.actions = Some(ActionMenu::new(actions).with_bindings(&self.keymap));
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
        if !buffer.paths.iter().any(|candidate| candidate.path == path) {
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
        self.operation_target_paths_for_display().len()
    }

    pub fn operation_target_first(&self) -> Option<PathBuf> {
        self.operation_target_paths_for_display().into_iter().next()
    }

    pub fn operation_target_source(&self) -> &'static str {
        if self.selected.is_empty() {
            "focused"
        } else {
            "selected"
        }
    }

    pub fn operation_target_labels(&self, limit: usize) -> Vec<String> {
        self.operation_target_paths_for_display()
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
        let identity = match capture_path_identity(&path) {
            Ok(identity) => identity,
            Err(err) => {
                self.log(format!("can only edit regular files: {err}"));
                return;
            }
        };
        if let Err(err) = validate_editor_target(&path) {
            self.log(format!("can only edit regular files: {err}"));
            return;
        }
        if let Err(err) = ensure_path_identity(&path, &identity) {
            self.log(format!("edit failed: {err}"));
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

    fn begin_destructive_confirmation(&mut self, mode: Mode) {
        let action = match mode {
            Mode::ConfirmTrash => "trash",
            Mode::ConfirmDelete => "delete",
            _ => "operation",
        };
        self.pending_destructive_targets = Some(self.capture_operation_targets(action));
        self.pending_rename = None;
        self.pending_direct_targets = None;
        self.mode = mode;
        self.input.clear();
    }

    fn copy_selection(&mut self) {
        self.transfer_revision = self.transfer_revision.wrapping_add(1);
        let paths = self.capture_operation_targets("copy");
        let len = paths.len();
        self.transfer_buffer = Some(TransferBuffer {
            kind: TransferKind::Copy,
            paths,
        });
        self.log(format!("copied {len} item(s)"));
    }

    fn cut_selection(&mut self) {
        self.transfer_revision = self.transfer_revision.wrapping_add(1);
        let paths = self.capture_operation_targets("cut");
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
            allow_replace: buffer.kind == TransferKind::Copy,
            approved_conflicts: Vec::new(),
        };
        self.start_file_operation(operation);
    }

    fn copy_to_destination(&mut self, move_items: bool) {
        let destination = match self.resolve_dir_input() {
            Ok(path) => path,
            Err(err) => {
                self.log(format!("destination rejected: {err}"));
                self.pending_direct_targets = None;
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
            paths: self.pending_direct_targets.take().unwrap_or_else(|| {
                self.capture_operation_targets(if move_items { "move" } else { "copy" })
            }),
            destination,
            source: FileOperationSource::Direct,
            allow_replace: !move_items,
            approved_conflicts: Vec::new(),
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
            let mut operation = operation;
            operation.approved_conflicts = conflicts
                .iter()
                .filter_map(|target| match BufferedPath::new(target.clone()) {
                    Ok(buffered) => Some(buffered),
                    Err(err) => {
                        self.log(format!("copy skipped: {err}"));
                        None
                    }
                })
                .collect();
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
        let kind = match operation.kind {
            TransferKind::Copy => crate::jobs::JobKind::Copy {
                replace: replace_existing,
                skip_conflicts,
            },
            TransferKind::Cut => crate::jobs::JobKind::Move { skip_conflicts },
        };
        let cut_revision = if operation.kind == TransferKind::Cut
            && operation.source == FileOperationSource::TransferBuffer
        {
            Some(self.transfer_revision)
        } else {
            None
        };
        let sources = operation.paths;
        if self.start_checked_job(
            crate::jobs::JobRequest {
                kind,
                sources: sources.iter().map(|source| source.path.clone()).collect(),
                destination: Some(operation.destination),
                work_root: self.work_root.clone(),
            },
            sources,
            operation.approved_conflicts,
        ) {
            self.job_cut_revision = cut_revision;
        }
    }

    fn start_job(&mut self, request: crate::jobs::JobRequest) -> bool {
        let sources = request
            .sources
            .iter()
            .filter_map(|path| BufferedPath::new(path.clone()).ok())
            .collect();
        self.start_checked_job(request, sources, Vec::new())
    }

    fn start_checked_job(
        &mut self,
        request: crate::jobs::JobRequest,
        sources: Vec<BufferedPath>,
        conflicts: Vec<BufferedPath>,
    ) -> bool {
        if self.active_job.is_some() {
            self.log("file job already active");
            return false;
        }
        let replacing = matches!(
            request.kind,
            crate::jobs::JobKind::Copy { replace: true, .. }
        );
        match crate::jobs::JobHandle::spawn_validated(request, move |source, target| {
            sources
                .iter()
                .find(|entry| entry.path == source)
                .context("source identity unavailable")?
                .ensure_current()?;
            if replacing
                && let Some(target) = target
                && target_exists(target)
            {
                conflicts
                    .iter()
                    .find(|entry| entry.path == target)
                    .with_context(|| {
                        format!("target changed during operation: {}", target.display())
                    })?
                    .ensure_current()?;
            }
            Ok(())
        }) {
            Ok(job) => {
                self.job_progress = Some(job.progress());
                self.last_job = None;
                self.active_job = Some(job);
                self.mode = Mode::Normal;
                self.input.clear();
                self.log("file job started; browsing remains available");
                true
            }
            Err(error) => {
                self.log(format!("job rejected: {error:#}"));
                false
            }
        }
    }

    fn request_quit(&mut self) {
        if let Some(job) = &self.active_job {
            job.cancel();
            self.exit_after_job = true;
            self.mode = Mode::Jobs;
            self.log("waiting for cancellation and cleanup before exit");
        } else {
            self.should_quit = true;
        }
    }

    pub fn job_active(&self) -> bool {
        self.active_job.is_some()
    }
    pub fn job_progress(&self) -> Option<&crate::jobs::JobProgress> {
        self.job_progress.as_ref()
    }
    pub fn last_job(&self) -> Option<&crate::jobs::JobResult> {
        self.last_job.as_ref()
    }
    pub fn exit_after_job(&self) -> bool {
        self.exit_after_job
    }
    pub fn trash_entries(&self) -> &[crate::trash::TrashEntry] {
        &self.trash_entries
    }
    pub fn trash_cursor(&self) -> usize {
        self.trash_cursor
    }
    pub fn trash_error(&self) -> Option<&str> {
        self.trash_error.as_deref()
    }
    pub fn restore_target(&self) -> Option<&Path> {
        let receipt = self.restore_pending.as_ref()?;
        self.trash_entries
            .iter()
            .find(|entry| &entry.receipt_path == receipt)
            .map(|entry| entry.original_path.as_path())
    }
    pub fn overlay_line_count(&self) -> usize {
        if self.mode == Mode::Help {
            self.keymap.bindings(&self.help_context).len() + 2
        } else {
            12 + self
                .last_job
                .as_ref()
                .map(|r| (r.failed.len() + r.skipped.len() + r.unprocessed.len()).min(100))
                .unwrap_or(0)
        }
    }

    fn load_trash(&mut self) {
        match crate::trash::scan_trash(&self.work_root) {
            Ok(scan) => {
                self.trash_entries = scan.entries;
                self.trash_error = if scan.warning_count > 0 {
                    Some(format!(
                        "{} invalid receipts skipped: {}",
                        scan.warning_count,
                        scan.warnings
                            .first()
                            .map(String::as_str)
                            .unwrap_or("inspect metadata")
                    ))
                } else {
                    None
                };
            }
            Err(error) => {
                self.trash_entries.clear();
                self.trash_error = Some(format!("{error:#}"));
            }
        }
        self.trash_cursor = self
            .trash_cursor
            .min(self.trash_entries.len().saturating_sub(1));
    }

    /// Consume coalesced progress and a completion receipt without waiting on I/O.
    pub fn poll_job(&mut self) -> bool {
        let Some(job) = self.active_job.as_mut() else {
            return false;
        };
        let progress = job.progress();
        let changed = self.job_progress.as_ref().is_none_or(|old| {
            old.current_path != progress.current_path
                || old.copied_bytes != progress.copied_bytes
                || old.completed != progress.completed
                || old.cancelling != progress.cancelling
        });
        self.job_progress = Some(progress);
        if !job.is_finished() {
            return changed;
        }
        let result = job.try_result().unwrap_or_else(|| crate::jobs::JobResult {
            failed: vec![crate::jobs::JobFailure {
                path: self.cwd.clone(),
                error: "worker stopped without a receipt; inspect targets before retry".into(),
            }],
            ..Default::default()
        });
        self.active_job = None;
        if self.job_cut_revision.take() == Some(self.transfer_revision)
            && let Some(buffer) = &mut self.transfer_buffer
            && buffer.kind == TransferKind::Cut
        {
            buffer
                .paths
                .retain(|path| !result.succeeded.contains(&path.path));
            if buffer.paths.is_empty() {
                self.transfer_buffer = None;
            }
        }
        self.log(format!(
            "{}{}: {} completed, {} failed, {} skipped, {} remaining",
            self.job_progress
                .as_ref()
                .map(|p| p.label.as_str())
                .unwrap_or("Job"),
            if result.cancelled {
                " cancelled"
            } else {
                " finished"
            },
            result.succeeded.len(),
            result.failed.len(),
            result.skipped.len(),
            result.unprocessed.len()
        ));
        for error in result.failed.iter().take(3) {
            self.log(format!(
                "job failed: {}: {}",
                error.path.display(),
                error.error
            ));
        }
        self.last_job = Some(result);
        self.reload();
        if self.mode == Mode::Trash {
            self.load_trash();
        }
        if self.exit_after_job {
            self.should_quit = true;
        }
        true
    }

    fn conflict_targets(&self, operation: &PendingFileOperation) -> Vec<PathBuf> {
        operation
            .paths
            .iter()
            .filter_map(|source| destination_for_paste(&source.path, &operation.destination).ok())
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
            Mode::Preview | Mode::Jobs | Mode::Trash => {}
            Mode::Goto => self.submit_goto(),
            Mode::Rename => self.submit_rename(),
            Mode::CopyTo => self.copy_to_destination(false),
            Mode::MoveTo => self.copy_to_destination(true),
            Mode::ConfirmTrash => {
                if self.input == "trash" {
                    self.submit_trash();
                } else {
                    self.log("type trash then submit to move to .tersh-trash");
                }
            }
            Mode::ConfirmDelete => {
                if self.input == "delete" {
                    self.submit_delete();
                } else {
                    self.log("type delete then submit for permanent delete");
                }
            }
            Mode::Conflict => self.submit_conflict(),
            Mode::ConfirmRestore => {
                if let Some(receipt) = self.restore_pending.take() {
                    self.start_job(crate::jobs::JobRequest {
                        kind: crate::jobs::JobKind::Restore,
                        sources: vec![receipt],
                        destination: None,
                        work_root: self.work_root.clone(),
                    });
                    self.mode = Mode::Trash;
                }
            }
            Mode::Help | Mode::Message => self.mode = Mode::Normal,
            Mode::Normal => {}
        }
    }

    fn submit_rename(&mut self) {
        let Some(source) = self.pending_rename.take().or_else(|| {
            self.focused()
                .and_then(|entry| BufferedPath::new(entry.path.clone()).ok())
        }) else {
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
        match source
            .ensure_current()
            .and_then(|()| rename_path(&source.path, &target))
        {
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
        self.submit_destructive_job(crate::jobs::JobKind::Trash, "trash");
    }

    fn submit_delete(&mut self) {
        self.submit_destructive_job(crate::jobs::JobKind::Delete, "delete");
    }

    fn submit_destructive_job(&mut self, kind: crate::jobs::JobKind, label: &str) {
        let sources = self
            .pending_destructive_targets
            .take()
            .unwrap_or_else(|| self.capture_operation_targets(label));
        self.start_checked_job(
            crate::jobs::JobRequest {
                kind,
                sources: sources.iter().map(|source| source.path.clone()).collect(),
                destination: None,
                work_root: self.work_root.clone(),
            },
            sources,
            Vec::new(),
        );
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
        if self.mode == Mode::ConfirmRestore {
            self.restore_pending = None;
            self.mode = Mode::Trash;
            self.input.clear();
            return;
        }
        self.pending_g = false;
        self.pending_y = false;
        self.pending_rename = None;
        self.pending_direct_targets = None;
        self.pending_destructive_targets = None;
        self.pending_file_operation = None;
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

    fn operation_target_paths_for_display(&self) -> Vec<PathBuf> {
        if matches!(self.mode, Mode::ConfirmTrash | Mode::ConfirmDelete)
            && let Some(targets) = &self.pending_destructive_targets
        {
            return targets.iter().map(|target| target.path.clone()).collect();
        }
        self.operation_targets()
    }

    fn capture_operation_targets(&mut self, action: &str) -> Vec<BufferedPath> {
        let mut buffered = Vec::new();
        for path in self.operation_targets() {
            match BufferedPath::new(path.clone()) {
                Ok(target) => buffered.push(target),
                Err(err) => {
                    if let Some(entry) = self.entry_for_path(&path) {
                        buffered.push(BufferedPath::from_entry(entry));
                    } else {
                        self.log(format!("{action} skipped: {err}"));
                    }
                }
            }
        }
        buffered
    }

    fn entry_for_path(&self, path: &Path) -> Option<&FileEntry> {
        self.entries
            .iter()
            .chain(self.all_entries.iter())
            .find(|entry| entry.path == path)
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
    run_with_keymap(path, options, crate::keymap::Keymap::default())
}

pub fn run_with_keymap(
    path: PathBuf,
    options: RunOptions,
    keymap: crate::keymap::Keymap,
) -> Result<()> {
    let output = if options.print_cwd {
        TerminalOutput::Stderr
    } else {
        TerminalOutput::Stdout
    };
    let final_cwd = run_tui(path, output, keymap)?;
    if options.print_cwd {
        println!("{}", final_cwd.display());
    }
    Ok(())
}

fn run_tui(
    path: PathBuf,
    output: TerminalOutput,
    keymap: crate::keymap::Keymap,
) -> Result<PathBuf> {
    let _guard = TerminalGuard::enter(output)?;
    let backend = CrosstermBackend::new(output.writer());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut app = App::new_with_output(path, output)?;
    app.set_keymap(keymap);
    let mut dirty = true;

    while !app.should_quit() {
        dirty |= app.poll_job();
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

fn capture_path_identity(path: &Path) -> Result<PathIdentity> {
    fs::symlink_metadata(path)
        .map(|metadata| PathIdentity::from_metadata(&metadata))
        .with_context(|| format!("failed to inspect {}", crate::fs_core::display_path(path)))
}

fn ensure_path_identity(path: &Path, expected: &PathIdentity) -> Result<()> {
    let actual = capture_path_identity(path)?;
    if &actual != expected {
        anyhow::bail!(
            "path changed during operation: {}",
            crate::fs_core::display_path(path)
        );
    }
    Ok(())
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
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
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
        && let Some(home) = expand_user_home(user)
    {
        return home.join(rest);
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
