use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    env, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Child, Command as ProcessCommand, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant, SystemTime},
};

const DEFAULT_REFRESH_INTERVAL: Duration = Duration::from_secs(15);
// Includes the jump connection, target handshake, and remote metrics collection.
const PROBE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_CONCURRENT_PROBES: usize = 16;
const MAX_PROBE_OUTPUT_BYTES: u64 = 1024 * 1024;
const EVENT_LIMIT: usize = 100;

/// User threshold is configuration, not a new monitoring collector.
pub fn parse_disk_threshold(value: Option<&str>) -> Result<u16> {
    let value = value.unwrap_or("90");
    let threshold = value
        .parse::<u16>()
        .context("TERSH_DISK_WARN must be an integer from 0 to 100")?;
    anyhow::ensure!(threshold <= 100, "TERSH_DISK_WARN must be from 0 to 100");
    Ok(threshold)
}

#[derive(Debug, Clone)]
pub struct HostEvent {
    pub at: SystemTime,
    pub alias: String,
    pub previous: ConnectionState,
    pub current: ConnectionState,
    pub message: String,
}

const PROBE_SCRIPT: &str = r#"
PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/local/cuda/bin:$PATH"
export PATH
printf 'hostname=%s\n' "$(hostname 2>/dev/null || echo unknown)"
printf 'system=%s\n' "$(uname -srm 2>/dev/null || uname -a 2>/dev/null || echo unknown)"
printf 'uptime=%s\n' "$(uptime 2>/dev/null | sed 's/^ *//')"
if [ -r /proc/loadavg ]; then
  awk '{printf "load=%s %s %s\n", $1, $2, $3}' /proc/loadavg
else
  printf 'load=%s\n' "$(sysctl -n vm.loadavg 2>/dev/null | tr -d '{}')"
fi
if command -v free >/dev/null 2>&1; then
  free -m | awk '/^Mem:/ {printf "memory=%s/%s MB (%d%%)\n", $3, $2, ($3 * 100) / $2}'
elif command -v memory_pressure >/dev/null 2>&1; then
  memory_pressure 2>/dev/null | awk '/System-wide memory free percentage/ {printf "memory=%s free\n", $5}'
else
  printf 'memory=unknown\n'
fi
df -hP / 2>/dev/null | awk 'NR==2 {printf "storage=%s/%s %s used\n", $3, $2, $5}'
if command -v ps >/dev/null 2>&1; then
  printf 'tasks=%s processes\n' "$(ps -e 2>/dev/null | wc -l | tr -d ' ')"
else
  printf 'tasks=unknown\n'
fi
if command -v nvidia-smi >/dev/null 2>&1; then
  printf 'gpu=%s\n' "$(nvidia-smi --query-gpu=name,utilization.gpu,memory.used,memory.total --format=csv,noheader,nounits 2>/dev/null | paste -sd ';' -)"
else
  printf 'gpu=none\n'
fi
"#;

const PATH_BOOTSTRAP_SCRIPT: &str = r#"PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/local/cuda/bin:$PATH"; export PATH"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostKind {
    Local,
    Jump,
    Server,
}

impl HostKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Jump => "jump",
            Self::Server => "server",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostConfig {
    alias: String,
    kind: HostKind,
    address: String,
    user: Option<String>,
    role: String,
    proxy_jump: Option<String>,
    proxy_jump_target: Option<String>,
    ssh_target: String,
    workdir: Option<String>,
}

impl HostConfig {
    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub fn kind(&self) -> HostKind {
        self.kind
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn user(&self) -> Option<&str> {
        self.user.as_deref()
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn proxy_jump(&self) -> Option<&str> {
        self.proxy_jump.as_deref()
    }

    pub fn proxy_jump_target(&self) -> Option<&str> {
        self.proxy_jump_target.as_deref()
    }

    pub fn ssh_target(&self) -> &str {
        &self.ssh_target
    }

    pub fn workdir(&self) -> Option<&str> {
        self.workdir.as_deref()
    }

    pub fn place_scope(&self) -> (String, String) {
        if self.kind == HostKind::Local {
            return ("local".into(), "local".into());
        }
        (
            self.alias.clone(),
            format!(
                "{}|{}|{}",
                self.ssh_target,
                self.proxy_jump_target()
                    .or_else(|| self.proxy_jump())
                    .unwrap_or(""),
                self.kind.label()
            ),
        )
    }
}

#[derive(Debug, Clone)]
pub struct ClusterInventory {
    hosts: Vec<HostConfig>,
    source: Option<PathBuf>,
}

impl ClusterInventory {
    pub fn from_json(input: &str) -> Result<Self> {
        let file: InventoryFile = serde_json::from_str(input).context("parse servers.json")?;
        let mut hosts = Vec::new();

        if let Some(main) = file.main_machine {
            let alias = normalize_optional(main.alias).unwrap_or_else(|| "local".to_string());
            hosts.push(HostConfig {
                address: normalize_optional(main.tailscale_ip).unwrap_or_else(|| alias.clone()),
                role: normalize_optional(main.role).unwrap_or_else(|| "Codex host".to_string()),
                ssh_target: alias.clone(),
                alias,
                kind: HostKind::Local,
                user: None,
                proxy_jump: None,
                proxy_jump_target: None,
                workdir: normalize_workdir(main.workdir),
            });
        }

        let mut proxy_targets = BTreeMap::new();
        if let Some(jump) = file.jump_host {
            let alias = normalize_optional(jump.alias).unwrap_or_else(|| "jump-host".to_string());
            let user = normalize_optional(jump.ssh_user);
            let address = normalize_optional(jump.tailscale_ip)
                .or_else(|| normalize_optional(jump.device_name))
                .unwrap_or_else(|| alias.clone());
            let ssh_target = connection_target(user.as_deref(), &address);
            proxy_targets.insert(alias.clone(), ssh_target.clone());
            hosts.push(HostConfig {
                address,
                role: normalize_optional(jump.role)
                    .unwrap_or_else(|| "Network jump host".to_string()),
                ssh_target,
                alias,
                kind: HostKind::Jump,
                user,
                proxy_jump: None,
                proxy_jump_target: None,
                workdir: normalize_workdir(jump.workdir),
            });
        }

        for server in file.servers.unwrap_or_default() {
            let alias = normalize_optional(server.alias).unwrap_or_else(|| {
                normalize_optional(server.campus_ip.clone()).unwrap_or_else(|| "server".to_string())
            });
            let user = normalize_optional(server.ssh_user);
            let address = normalize_optional(server.campus_ip).unwrap_or_else(|| alias.clone());
            let proxy_jump = normalize_optional(server.proxy_jump);
            let proxy_jump_target = proxy_jump
                .as_ref()
                .and_then(|alias| proxy_targets.get(alias).cloned())
                .or_else(|| proxy_jump.clone());
            hosts.push(HostConfig {
                ssh_target: connection_target(user.as_deref(), &address),
                address,
                role: normalize_optional(server.role)
                    .unwrap_or_else(|| "Remote server".to_string()),
                alias,
                kind: HostKind::Server,
                user,
                proxy_jump,
                proxy_jump_target,
                workdir: normalize_workdir(server.workdir),
            });
        }

        if hosts.is_empty() {
            hosts.push(local_fallback_host());
        }
        validate_hosts(&hosts)?;

        Ok(Self {
            hosts,
            source: None,
        })
    }

    pub fn load_default() -> Result<Self> {
        if let Some(path) = env::var_os("TERSH_SERVERS_JSON").map(PathBuf::from) {
            return Self::from_path(&path);
        }

        for path in default_inventory_candidates() {
            if path.exists() {
                return Self::from_path(&path);
            }
        }

        Ok(Self {
            hosts: vec![local_fallback_host()],
            source: None,
        })
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let input = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let mut inventory = Self::from_json(&input)?;
        inventory.source = Some(path.to_path_buf());
        Ok(inventory)
    }

    pub fn hosts(&self) -> &[HostConfig] {
        &self.hosts
    }

    pub fn source(&self) -> Option<&Path> {
        self.source.as_deref()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InventoryFile {
    main_machine: Option<MainMachineRecord>,
    jump_host: Option<JumpHostRecord>,
    servers: Option<Vec<ServerRecord>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MainMachineRecord {
    alias: Option<String>,
    tailscale_ip: Option<String>,
    role: Option<String>,
    #[serde(alias = "directory", alias = "tersh_dir")]
    workdir: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct JumpHostRecord {
    alias: Option<String>,
    device_name: Option<String>,
    tailscale_ip: Option<String>,
    ssh_user: Option<String>,
    role: Option<String>,
    #[serde(alias = "directory", alias = "tersh_dir")]
    workdir: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServerRecord {
    alias: Option<String>,
    ssh_user: Option<String>,
    campus_ip: Option<String>,
    proxy_jump: Option<String>,
    role: Option<String>,
    #[serde(alias = "directory", alias = "tersh_dir")]
    workdir: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProbeReport {
    pub hostname: Option<String>,
    pub system: Option<String>,
    pub uptime: Option<String>,
    pub cpu_load: Option<String>,
    pub memory: Option<String>,
    pub storage: Option<String>,
    pub tasks: Option<String>,
    pub gpu: Option<String>,
}

impl ProbeReport {
    pub fn parse(output: &str) -> Self {
        let mut report = Self::default();
        for line in output.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            if value.is_empty() {
                continue;
            }
            match key.trim() {
                "hostname" => report.hostname = Some(value.to_string()),
                "system" => report.system = Some(value.to_string()),
                "uptime" => report.uptime = Some(value.to_string()),
                "load" => report.cpu_load = Some(value.to_string()),
                "memory" => report.memory = Some(value.to_string()),
                "storage" => report.storage = Some(value.to_string()),
                "tasks" => report.tasks = Some(value.to_string()),
                "gpu" => report.gpu = Some(value.to_string()),
                _ => {}
            }
        }
        report
    }

    pub fn is_empty(&self) -> bool {
        self.hostname.is_none()
            && self.system.is_none()
            && self.uptime.is_none()
            && self.cpu_load.is_none()
            && self.memory.is_none()
            && self.storage.is_none()
            && self.tasks.is_none()
            && self.gpu.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Unknown,
    Checking,
    Online,
    Stale,
    Timeout,
    AuthFailed,
    Offline,
}

impl ConnectionState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Checking => "checking",
            Self::Online => "online",
            Self::Stale => "stale",
            Self::Timeout => "timeout",
            Self::AuthFailed => "auth-failed",
            Self::Offline => "offline",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostSnapshot {
    pub alias: String,
    pub connection: ConnectionState,
    pub report: ProbeReport,
    pub latency_ms: Option<u128>,
    pub error: Option<String>,
    pub refreshed_at: Option<SystemTime>,
}

#[derive(Debug, Clone)]
struct ProbeSnapshot {
    token: u64,
    snapshot: HostSnapshot,
}

#[derive(Debug, Clone, Copy)]
struct ActiveProbe {
    token: u64,
    timed_out: bool,
}

/// Own every probe thread for this dashboard. Leaving the TUI cancels all
/// commands and reaps their processes before the terminal guard is restored.
#[derive(Default)]
struct ProbeWorkers {
    cancel: Arc<AtomicBool>,
    threads: Vec<(Arc<AtomicBool>, thread::JoinHandle<()>)>,
}

impl ProbeWorkers {
    fn reap(&mut self) {
        let mut index = 0;
        while index < self.threads.len() {
            if self.threads[index].0.load(Ordering::Acquire) || self.threads[index].1.is_finished()
            {
                let (_, thread) = self.threads.swap_remove(index);
                let _ = thread.join();
            } else {
                index += 1;
            }
        }
    }

    fn spawn(&mut self, host: HostConfig, token: u64, tx: mpsc::Sender<ProbeSnapshot>) {
        self.reap();
        let cancel = self.cancel.clone();
        let done = Arc::new(AtomicBool::new(false));
        let worker_done = done.clone();
        let thread = thread::spawn(move || {
            let snapshot = collect_host_snapshot_cancellable(&host, &cancel);
            worker_done.store(true, Ordering::Release);
            let _ = tx.send(ProbeSnapshot { token, snapshot });
        });
        self.threads.push((done, thread));
    }
}

impl Drop for ProbeWorkers {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        for (_, worker) in self.threads.drain(..) {
            let _ = worker.join();
        }
    }
}

impl HostSnapshot {
    pub fn unknown(alias: impl Into<String>) -> Self {
        Self {
            alias: alias.into(),
            connection: ConnectionState::Unknown,
            report: ProbeReport::default(),
            latency_ms: None,
            error: None,
            refreshed_at: None,
        }
    }

    pub fn checking(alias: impl Into<String>) -> Self {
        Self {
            alias: alias.into(),
            connection: ConnectionState::Checking,
            report: ProbeReport::default(),
            latency_ms: None,
            error: None,
            refreshed_at: Some(SystemTime::now()),
        }
    }

    pub fn online(alias: impl Into<String>, report: ProbeReport, latency_ms: u128) -> Self {
        Self {
            alias: alias.into(),
            connection: ConnectionState::Online,
            report,
            latency_ms: Some(latency_ms),
            error: None,
            refreshed_at: Some(SystemTime::now()),
        }
    }

    pub fn offline(alias: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            alias: alias.into(),
            connection: ConnectionState::Offline,
            report: ProbeReport::default(),
            latency_ms: None,
            error: Some(error.into()),
            refreshed_at: Some(SystemTime::now()),
        }
    }

    pub fn failed(alias: impl Into<String>, error: impl Into<String>) -> Self {
        let error = error.into();
        let connection = classify_failure(&error);
        Self {
            alias: alias.into(),
            connection,
            report: ProbeReport::default(),
            latency_ms: None,
            error: Some(error),
            refreshed_at: Some(SystemTime::now()),
        }
    }

    pub fn stale(alias: impl Into<String>, report: ProbeReport, error: impl Into<String>) -> Self {
        Self {
            alias: alias.into(),
            connection: ConnectionState::Stale,
            report,
            latency_ms: None,
            error: Some(error.into()),
            refreshed_at: Some(SystemTime::now()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClusterMode {
    Normal,
    Detail,
    Help,
    Filter,
    Events,
    Places,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClusterCommand {
    Down,
    Up,
    First,
    Last,
    RefreshAll,
    RefreshSelected,
    OpenSession,
    OpenWorkbench,
    OpenDetail,
    OpenHelp,
    Cancel,
    Quit,
    ForceQuit,
    DetailDown,
    DetailUp,
    OpenFilter,
    ClearFilter,
    CycleSort,
    ReverseSort,
    OpenActions,
    ToggleAttention,
    OpenEvents,
    TogglePause,
    OpenPlaces,
    PinPlace,
}

macro_rules! cluster_actions {
    ($($variant:ident=>$id:literal),* $(,)?) => {
        impl ClusterCommand {
            pub fn action_id(self)-> &'static str { match self { $(Self::$variant=>$id,)* } }
            pub fn from_action(id:&str)->Option<Self> { match id { $($id=>Some(Self::$variant),)* _=>None } }
        }
    }
}
cluster_actions! { Down=>"down",Up=>"up",First=>"first",Last=>"last",RefreshAll=>"refresh_all",RefreshSelected=>"refresh_selected",
OpenSession=>"open_session",OpenWorkbench=>"open_workbench",OpenDetail=>"open_detail",OpenHelp=>"open_help",Cancel=>"cancel",
Quit=>"quit",ForceQuit=>"force_quit",DetailDown=>"detail_down",DetailUp=>"detail_up",OpenFilter=>"open_filter",ClearFilter=>"clear_filter",
CycleSort=>"cycle_sort",ReverseSort=>"reverse_sort",OpenActions=>"open_actions",
ToggleAttention=>"toggle_attention",OpenEvents=>"open_events",TogglePause=>"toggle_pause",
OpenPlaces=>"open_places",PinPlace=>"pin_place" }

#[derive(Debug, Clone)]
pub struct HostPlace {
    pub host: String,
    pub identity: String,
    pub path: PathBuf,
    pub pinned: bool,
    pub valid: bool,
    pub configured: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HostSort {
    #[default]
    Inventory,
    Alias,
    State,
    Load,
    Memory,
    Disk,
    Probe,
}

impl HostSort {
    pub fn label(self) -> &'static str {
        match self {
            Self::Inventory => "inventory",
            Self::Alias => "alias",
            Self::State => "state",
            Self::Load => "load",
            Self::Memory => "memory",
            Self::Disk => "disk",
            Self::Probe => "probe",
        }
    }
    fn next(self) -> Self {
        match self {
            Self::Inventory => Self::Alias,
            Self::Alias => Self::State,
            Self::State => Self::Load,
            Self::Load => Self::Memory,
            Self::Memory => Self::Disk,
            Self::Disk => Self::Probe,
            Self::Probe => Self::Inventory,
        }
    }
}

#[derive(Debug)]
pub struct ClusterApp {
    keymap: std::sync::Arc<crate::keymap::Keymap>,
    key_state: crate::bindings::KeyState,
    help_offset: usize,
    help_context: String,
    actions: Option<crate::actions::ActionMenu<ClusterCommand>>,
    history: BTreeMap<String, VecDeque<crate::metrics::MetricSample>>,
    detail_offset: u16,
    detail_limit: std::cell::Cell<u16>,
    animation_frame: u64,
    hosts: Vec<HostConfig>,
    visible: Vec<usize>,
    filter: String,
    filter_before: String,
    filter_origin: ClusterMode,
    filter_anchor: Option<String>,
    sort: HostSort,
    sort_reverse: bool,
    snapshots: BTreeMap<String, HostSnapshot>,
    last_good_reports: BTreeMap<String, ProbeReport>,
    last_good_at: BTreeMap<String, SystemTime>,
    expired_data: BTreeSet<String>,
    final_states: BTreeMap<String, ConnectionState>,
    events: VecDeque<HostEvent>,
    event_limit: std::cell::Cell<usize>,
    attention_only: bool,
    refresh_paused: bool,
    disk_threshold: u16,
    observations: BTreeMap<String, u64>,
    last_attempt: BTreeMap<String, Instant>,
    next_due: BTreeMap<String, Instant>,
    failure_streak: BTreeMap<String, u32>,
    places: crate::places::Places,
    place_choices: Vec<HostPlace>,
    place_cursor: usize,
    place_query: String,
    place_message: Option<String>,
    pending_workbench: Option<HostConfig>,
    refresh_deadlines: BTreeMap<String, Instant>,
    active_probes: BTreeMap<String, ActiveProbe>,
    cursor: usize,
    mode: ClusterMode,
    should_quit: bool,
    logs: Vec<String>,
    refreshing: BTreeSet<String>,
    inventory_source: Option<PathBuf>,
    last_refresh_started: Option<Instant>,
    refresh_cursor: usize,
    refresh_epoch: u64,
}

impl ClusterApp {
    pub fn new(mut hosts: Vec<HostConfig>) -> Self {
        if hosts.is_empty() {
            hosts.push(local_fallback_host());
        }
        let mut seen = BTreeSet::new();
        hosts.retain(|host| seen.insert(host.alias().to_string()));
        let snapshots = hosts
            .iter()
            .map(|host| {
                (
                    host.alias().to_string(),
                    HostSnapshot::unknown(host.alias().to_string()),
                )
            })
            .collect();
        Self {
            keymap: std::sync::Arc::new(crate::keymap::Keymap::default()),
            key_state: crate::bindings::KeyState::default(),
            help_offset: 0,
            help_context: "cluster".into(),
            visible: (0..hosts.len()).collect(),
            filter: String::new(),
            filter_before: String::new(),
            filter_origin: ClusterMode::Normal,
            filter_anchor: None,
            sort: HostSort::Inventory,
            sort_reverse: false,
            hosts,
            actions: None,
            history: BTreeMap::new(),
            detail_offset: 0,
            detail_limit: std::cell::Cell::new(0),
            animation_frame: 0,
            snapshots,
            last_good_reports: BTreeMap::new(),
            last_good_at: BTreeMap::new(),
            expired_data: BTreeSet::new(),
            final_states: BTreeMap::new(),
            events: VecDeque::new(),
            event_limit: std::cell::Cell::new(0),
            attention_only: false,
            refresh_paused: false,
            disk_threshold: 90,
            observations: BTreeMap::new(),
            last_attempt: BTreeMap::new(),
            next_due: BTreeMap::new(),
            failure_streak: BTreeMap::new(),
            places: crate::places::Places::memory(),
            place_choices: Vec::new(),
            place_cursor: 0,
            place_query: String::new(),
            place_message: None,
            pending_workbench: None,
            refresh_deadlines: BTreeMap::new(),
            active_probes: BTreeMap::new(),
            cursor: 0,
            mode: ClusterMode::Normal,
            should_quit: false,
            logs: vec!["ready".to_string()],
            refreshing: BTreeSet::new(),
            inventory_source: None,
            last_refresh_started: None,
            refresh_cursor: 0,
            refresh_epoch: 0,
        }
    }

    pub fn from_inventory(inventory: ClusterInventory) -> Self {
        let mut app = Self::new(inventory.hosts);
        app.inventory_source = inventory.source;
        app
    }

    pub fn apply(&mut self, command: ClusterCommand) {
        if matches!(self.mode, ClusterMode::Help | ClusterMode::Events) {
            let max_offset = if self.mode == ClusterMode::Events {
                self.event_limit.get()
            } else {
                self.keymap
                    .bindings(&self.help_context)
                    .len()
                    .saturating_sub(1)
            };
            match command {
                ClusterCommand::Down => {
                    self.help_offset = self.help_offset.saturating_add(1).min(max_offset);
                    return;
                }
                ClusterCommand::Up => {
                    self.help_offset = self.help_offset.saturating_sub(1);
                    return;
                }
                ClusterCommand::First => {
                    self.help_offset = 0;
                    return;
                }
                ClusterCommand::Last => {
                    self.help_offset = max_offset;
                    return;
                }
                _ => {}
            }
        }
        match command {
            ClusterCommand::OpenPlaces => {
                self.place_query.clear();
                self.place_cursor = 0;
                self.place_message = None;
                self.rebuild_places();
                self.mode = ClusterMode::Places;
            }
            ClusterCommand::PinPlace => self.pin_configured_place(),
            ClusterCommand::ToggleAttention => {
                self.attention_only = !self.attention_only;
                self.rebuild_view();
            }
            ClusterCommand::OpenEvents => {
                self.help_offset = 0;
                self.mode = ClusterMode::Events;
            }
            ClusterCommand::TogglePause => {
                self.refresh_paused = !self.refresh_paused;
                self.log(if self.refresh_paused {
                    "Automatic refresh paused; active probes finish; manual refresh available"
                } else {
                    "Automatic refresh resumed"
                });
            }
            ClusterCommand::OpenActions => self.open_actions(),
            ClusterCommand::OpenFilter => {
                self.filter_before = self.filter.clone();
                self.filter_origin = self.mode;
                self.filter_anchor = self.selected_host().map(|host| host.alias().to_owned());
                self.mode = ClusterMode::Filter;
            }
            ClusterCommand::ClearFilter => {
                self.filter.clear();
                self.attention_only = false;
                self.rebuild_view();
            }
            ClusterCommand::CycleSort => {
                self.sort = self.sort.next();
                self.rebuild_view();
            }
            ClusterCommand::ReverseSort => {
                self.sort_reverse = !self.sort_reverse;
                self.rebuild_view();
            }
            ClusterCommand::DetailDown => {
                self.detail_offset = self
                    .detail_offset()
                    .saturating_add(5)
                    .min(self.detail_limit.get())
            }
            ClusterCommand::DetailUp => self.detail_offset = self.detail_offset().saturating_sub(5),
            ClusterCommand::Down => self.move_cursor(1),
            ClusterCommand::Up => self.move_cursor(-1),
            ClusterCommand::First => self.cursor = 0,
            ClusterCommand::Last => self.cursor = self.visible.len().saturating_sub(1),
            ClusterCommand::OpenDetail => {
                self.mode = ClusterMode::Detail;
                self.detail_offset = 0;
            }
            ClusterCommand::OpenHelp => {
                self.help_context = self.key_context().into();
                self.help_offset = 0;
                self.mode = ClusterMode::Help;
            }
            ClusterCommand::Cancel => {
                if self.mode == ClusterMode::Filter {
                    self.filter = self.filter_before.clone();
                    self.rebuild_view();
                    if let Some(alias) = &self.filter_anchor
                        && let Some(cursor) = self
                            .visible
                            .iter()
                            .position(|index| self.hosts[*index].alias() == alias)
                    {
                        self.cursor = cursor;
                    }
                    self.mode = self.filter_origin;
                } else {
                    self.mode = ClusterMode::Normal;
                }
            }
            ClusterCommand::Quit => {
                if matches!(
                    self.mode,
                    ClusterMode::Help
                        | ClusterMode::Detail
                        | ClusterMode::Events
                        | ClusterMode::Places
                ) {
                    self.mode = ClusterMode::Normal;
                } else {
                    self.should_quit = true;
                }
            }
            ClusterCommand::ForceQuit => self.should_quit = true,
            ClusterCommand::RefreshAll => self.log("refresh requested"),
            ClusterCommand::RefreshSelected => {
                let alias = match self.selected_host() {
                    Some(host) => host.alias().to_string(),
                    None => return,
                };
                self.log(format!("refresh requested: {alias}"));
            }
            ClusterCommand::OpenSession => {
                let alias = match self.selected_host() {
                    Some(host) => host.alias().to_string(),
                    None => return,
                };
                self.log(format!("opening session: {alias}"));
            }
            ClusterCommand::OpenWorkbench => {
                let alias = match self.selected_host() {
                    Some(host) => host.alias().to_string(),
                    None => return,
                };
                self.log(format!("opening tersh: {alias}"));
            }
        }
    }

    pub fn begin_refresh(&mut self, aliases: &[String]) -> Vec<String> {
        self.begin_refresh_ordered(aliases, true)
    }

    fn begin_refresh_ordered(&mut self, aliases: &[String], rotate: bool) -> Vec<String> {
        let mut started = Vec::new();
        let now = Instant::now();
        if aliases.is_empty() {
            self.log("no hosts to refresh");
            return started;
        }
        if self.active_probes.len() >= MAX_CONCURRENT_PROBES {
            self.last_refresh_started = Some(now);
            self.log("refresh already in progress");
            return started;
        }
        let slots = MAX_CONCURRENT_PROBES.saturating_sub(self.active_probes.len());
        let start = if rotate {
            self.refresh_cursor % aliases.len()
        } else {
            0
        };
        let mut last_index = start;
        let mut skipped_active = false;
        for offset in 0..aliases.len() {
            if started.len() >= slots {
                break;
            }
            let index = (start + offset) % aliases.len();
            let alias = &aliases[index];
            if !self.has_host_alias(alias) {
                continue;
            }
            if self.active_probes.contains_key(alias) {
                skipped_active = true;
                continue;
            }
            let token = self.next_refresh_token();
            self.active_probes.insert(
                alias.clone(),
                ActiveProbe {
                    token,
                    timed_out: false,
                },
            );
            self.refreshing.insert(alias.clone());
            self.refresh_deadlines
                .insert(alias.clone(), now + PROBE_TIMEOUT);
            self.last_attempt.insert(alias.clone(), now);
            started.push(alias.clone());
            last_index = index;
            let mut checking = self
                .snapshots
                .get(alias)
                .cloned()
                .unwrap_or_else(|| HostSnapshot::unknown(alias));
            checking.connection = ConnectionState::Checking;
            self.snapshots.insert(alias.clone(), checking);
        }
        if rotate && aliases.len() > 1 && !started.is_empty() {
            self.refresh_cursor = (last_index + 1) % aliases.len();
        }
        if started.is_empty() {
            if skipped_active {
                self.last_refresh_started = Some(now);
                self.log("refresh already in progress");
            } else {
                self.log("no eligible hosts to refresh");
            }
        } else {
            self.last_refresh_started = Some(now);
            self.log(format!("refreshing {} host(s)", started.len()));
        }
        self.rebuild_view();
        started
    }

    pub fn apply_snapshot(&mut self, snapshot: HostSnapshot) {
        if !self.has_host_alias(&snapshot.alias) {
            return;
        }
        self.apply_snapshot_inner(snapshot);
    }

    #[doc(hidden)]
    pub fn apply_completed_refresh_snapshot(&mut self, snapshot: HostSnapshot) {
        if !self.has_host_alias(&snapshot.alias) {
            return;
        }
        self.active_probes.remove(&snapshot.alias);
        self.refreshing.remove(&snapshot.alias);
        self.refresh_deadlines.remove(&snapshot.alias);
        self.apply_snapshot_inner(snapshot);
    }

    fn apply_probe_snapshot(&mut self, result: ProbeSnapshot) {
        let alias = result.snapshot.alias.clone();
        let Some(active) = self.active_probes.get(&alias).copied() else {
            return;
        };
        if active.token != result.token {
            return;
        }
        self.active_probes.remove(&alias);
        self.refreshing.remove(&alias);
        self.refresh_deadlines.remove(&alias);
        if active.timed_out {
            return;
        }
        self.apply_snapshot_inner(result.snapshot);
    }

    fn apply_snapshot_inner(&mut self, snapshot: HostSnapshot) {
        let alias = snapshot.alias.clone();
        if !matches!(
            snapshot.connection,
            ConnectionState::Checking | ConnectionState::Unknown
        ) {
            let delay = if snapshot.connection == ConnectionState::Online {
                self.failure_streak.remove(&alias);
                DEFAULT_REFRESH_INTERVAL
            } else {
                let streak = self.failure_streak.entry(alias.clone()).or_default();
                *streak = streak.saturating_add(1);
                Duration::from_secs((15u64 << (*streak).min(3)).min(120))
            };
            self.next_due.insert(alias.clone(), Instant::now() + delay);
            let history = self.history.entry(alias.clone()).or_default();
            if history.len() == crate::metrics::HISTORY_LIMIT {
                history.pop_front();
            }
            history.push_back(crate::metrics::MetricSample::from_snapshot(&snapshot));
            *self.observations.entry(alias.clone()).or_default() += 1;
        }
        let snapshot = self.merge_last_good_snapshot(snapshot);
        let state = snapshot.connection.label();
        if snapshot.connection == ConnectionState::Online && !snapshot.report.is_empty() {
            self.last_good_reports
                .insert(alias.clone(), snapshot.report.clone());
            self.last_good_at.insert(
                alias.clone(),
                snapshot.refreshed_at.unwrap_or_else(SystemTime::now),
            );
        }
        if !matches!(
            snapshot.connection,
            ConnectionState::Checking | ConnectionState::Unknown
        ) {
            let previous = self
                .final_states
                .insert(alias.clone(), snapshot.connection)
                .unwrap_or(ConnectionState::Unknown);
            if previous != snapshot.connection {
                if self.events.len() == EVENT_LIMIT {
                    self.events.pop_front();
                }
                self.events.push_back(HostEvent {
                    at: snapshot.refreshed_at.unwrap_or_else(SystemTime::now),
                    alias: alias.clone(),
                    previous,
                    current: snapshot.connection,
                    message: snapshot
                        .error
                        .as_deref()
                        .unwrap_or("probe completed")
                        .chars()
                        .take(512)
                        .collect(),
                });
            }
        }
        self.snapshots.insert(alias.clone(), snapshot);
        self.update_expired_data();
        self.rebuild_view();
        self.log(format!("{alias}: {state}"));
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ClusterCommand> {
        if key.kind == crossterm::event::KeyEventKind::Release {
            return None;
        }
        let context = self.key_context();
        let matched = self.key_state.feed(&self.keymap, context, key);
        for ch in std::mem::take(&mut self.key_state.replay) {
            self.input_text(ch);
        }
        match matched {
            crate::keymap::KeyMatch::Action(action) => {
                if action == "force_quit" {
                    self.apply(ClusterCommand::ForceQuit);
                    return None;
                }
                if let Some(menu) = self.actions.as_mut() {
                    match menu.handle_action(&action) {
                        crate::actions::MenuResult::Pending => {}
                        crate::actions::MenuResult::Cancel => self.actions = None,
                        crate::actions::MenuResult::Run(command) => {
                            self.actions = None;
                            return self.dispatch(command);
                        }
                    }
                } else if self.mode == ClusterMode::Places {
                    return self.handle_place_action(&action);
                } else if self.mode == ClusterMode::Filter {
                    match action.as_str() {
                        "cancel" => self.apply(ClusterCommand::Cancel),
                        "submit" => self.mode = self.filter_origin,
                        "backspace" => {
                            self.filter.pop();
                            self.rebuild_view();
                        }
                        _ => {}
                    }
                } else if let Some(command) = ClusterCommand::from_action(&action) {
                    return self.dispatch(command);
                }
            }
            crate::keymap::KeyMatch::Unbound => {
                if let Some(ch) = crate::bindings::printable(key) {
                    self.input_text(ch);
                }
            }
            crate::keymap::KeyMatch::Pending => {}
        }
        None
    }

    fn input_text(&mut self, ch: char) {
        if let Some(menu) = self.actions.as_mut() {
            menu.input_char(ch);
        } else if self.mode == ClusterMode::Places
            && !ch.is_control()
            && self.place_query.chars().count() < 256
        {
            self.place_query.push(ch);
            self.place_cursor = 0;
            self.rebuild_places();
        } else if self.mode == ClusterMode::Filter
            && !ch.is_control()
            && self.filter.chars().count() < 256
        {
            self.filter.push(ch);
            self.rebuild_view();
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
            ClusterMode::Normal => "cluster",
            ClusterMode::Detail => "cluster_detail",
            ClusterMode::Help | ClusterMode::Events => "help",
            ClusterMode::Filter => "cluster_filter",
            ClusterMode::Places => "places",
        }
    }
    pub fn help_context(&self) -> &str {
        &self.help_context
    }
    pub fn help_offset(&self) -> usize {
        self.help_offset
    }

    fn open_actions(&mut self) {
        use crate::actions::{Action, ActionMenu};
        let mut actions = vec![
            Action::new(
                "Refresh selected host",
                "Enter",
                ClusterCommand::RefreshSelected,
            ),
            Action::new("Open Tersh workbench", "t", ClusterCommand::OpenWorkbench),
            Action::new("Open shell / SSH", "s", ClusterCommand::OpenSession),
            Action::new("Host detail and trends", "l", ClusterCommand::OpenDetail),
            Action::new("Filter hosts", "/", ClusterCommand::OpenFilter),
            Action::new(
                "Clear host filter",
                "Backspace",
                ClusterCommand::ClearFilter,
            ),
            Action::new("Cycle host sort", "v", ClusterCommand::CycleSort),
            Action::new("Reverse host sort", "V", ClusterCommand::ReverseSort),
            Action::new("Refresh all hosts", "r", ClusterCommand::RefreshAll),
            Action::new(
                "Toggle hosts needing attention",
                "a",
                ClusterCommand::ToggleAttention,
            ),
            Action::new("State change events", "E", ClusterCommand::OpenEvents),
            Action::new(
                "Pause / resume automatic refresh",
                "P",
                ClusterCommand::TogglePause,
            ),
            Action::new(
                "Recent and pinned host directories",
                "b",
                ClusterCommand::OpenPlaces,
            ),
            Action::new(
                "Pin / unpin configured work directory",
                "B",
                ClusterCommand::PinPlace,
            ),
            Action::new("Help", "?", ClusterCommand::OpenHelp),
        ];
        if self.mode == ClusterMode::Detail {
            actions.extend([
                Action::new("Scroll detail down", "PgDn", ClusterCommand::DetailDown),
                Action::new("Scroll detail up", "PgUp", ClusterCommand::DetailUp),
                Action::new("Back to hosts", "q", ClusterCommand::Cancel),
            ]);
        }
        if self.selected_host().is_none() {
            actions.retain(|action| {
                !matches!(
                    action.command,
                    ClusterCommand::RefreshSelected
                        | ClusterCommand::OpenSession
                        | ClusterCommand::OpenWorkbench
                        | ClusterCommand::OpenDetail
                )
            });
        }
        for action in &mut actions {
            let (group, description, keywords) =
                crate::actions::metadata(action.command.action_id());
            action.group = group;
            action.description = description;
            action.keywords = keywords;
            action.key = crate::bindings::label(
                &self.keymap,
                self.key_context(),
                action.command.action_id(),
            )
            .unwrap_or_else(|| "unbound".into());
        }
        self.actions = Some(ActionMenu::new(actions).with_bindings(&self.keymap));
    }

    fn dispatch(&mut self, command: ClusterCommand) -> Option<ClusterCommand> {
        if self.selected_host().is_none()
            && matches!(
                command,
                ClusterCommand::RefreshSelected
                    | ClusterCommand::OpenSession
                    | ClusterCommand::OpenWorkbench
            )
        {
            self.log("no matching host; clear the filter first");
            return None;
        }
        match command {
            ClusterCommand::RefreshAll
            | ClusterCommand::RefreshSelected
            | ClusterCommand::OpenSession
            | ClusterCommand::OpenWorkbench => Some(command),
            _ => {
                self.apply(command);
                None
            }
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn actions(&self) -> Option<&crate::actions::ActionMenu<ClusterCommand>> {
        self.actions.as_ref()
    }

    pub fn set_places(&mut self, places: crate::places::Places) {
        self.places = places;
        self.rebuild_places();
    }
    pub fn place_choices(&self) -> &[HostPlace] {
        &self.place_choices
    }
    pub fn place_cursor(&self) -> usize {
        self.place_cursor
    }
    pub fn place_query(&self) -> &str {
        &self.place_query
    }
    pub fn place_message(&self) -> Option<&str> {
        self.place_message.as_deref()
    }
    pub fn places_enabled(&self) -> bool {
        self.places.is_enabled()
    }
    pub fn pending_workbench(&self) -> Option<&HostConfig> {
        self.pending_workbench.as_ref()
    }

    fn rebuild_places(&mut self) {
        let mut choices =
            self.places
                .entries()
                .iter()
                .map(|place| HostPlace {
                    host: place.host.clone(),
                    identity: place.identity.clone(),
                    path: place.path.clone(),
                    pinned: place.pinned,
                    valid: self.hosts.iter().any(|host| {
                        host.place_scope() == (place.host.clone(), place.identity.clone())
                    }),
                    configured: false,
                })
                .collect::<Vec<_>>();
        for host in &self.hosts {
            let Some(path) = host.workdir().filter(|path| Path::new(path).is_absolute()) else {
                continue;
            };
            let (scope, identity) = host.place_scope();
            if let Some(place) = choices.iter_mut().find(|place| {
                place.host == scope && place.identity == identity && place.path == Path::new(path)
            }) {
                place.configured = true;
            } else {
                choices.push(HostPlace {
                    host: scope,
                    identity,
                    path: path.into(),
                    pinned: false,
                    valid: true,
                    configured: true,
                });
            }
        }
        let selected = self.selected_host().map(HostConfig::place_scope);
        // Stable sorting preserves the store's recency order within each group.
        choices.sort_by_key(|place| {
            (
                selected.as_ref() != Some(&(place.host.clone(), place.identity.clone())),
                !place.pinned,
            )
        });
        let query = self.place_query.to_lowercase();
        choices.retain(|place| {
            format!("{} {}", place.host, place.path.display())
                .to_lowercase()
                .contains(&query)
        });
        self.place_choices = choices;
        self.place_cursor = self
            .place_cursor
            .min(self.place_choices.len().saturating_sub(1));
    }

    fn pin_configured_place(&mut self) {
        let Some(host) = self.selected_host().cloned() else {
            return;
        };
        let Some(path) = host.workdir().filter(|path| Path::new(path).is_absolute()) else {
            self.log("Set an absolute workdir in the host inventory before pinning");
            return;
        };
        let (scope, identity) = host.place_scope();
        let result = self.places.toggle_pin(&scope, &identity, Path::new(path));
        match result {
            Ok(()) => self.log(if self.places.is_enabled() {
                "Updated workdir pin"
            } else {
                "Places history is disabled"
            }),
            Err(error) => self.log(format!("Places: {error}")),
        }
        self.rebuild_places();
    }

    fn handle_place_action(&mut self, action: &str) -> Option<ClusterCommand> {
        let choice = self.place_choices.get(self.place_cursor).cloned();
        match action {
            "cancel" => {
                self.mode = ClusterMode::Normal;
                self.pending_workbench = None;
            }
            "down" => {
                self.place_cursor = self
                    .place_cursor
                    .saturating_add(1)
                    .min(self.place_choices.len().saturating_sub(1))
            }
            "up" => self.place_cursor = self.place_cursor.saturating_sub(1),
            "first" => self.place_cursor = 0,
            "last" => self.place_cursor = self.place_choices.len().saturating_sub(1),
            "backspace" => {
                self.place_query.pop();
                self.place_cursor = 0;
                self.rebuild_places();
            }
            "submit" => {
                if let Some(choice) = choice {
                    let host = self
                        .hosts
                        .iter()
                        .find(|host| {
                            host.place_scope() == (choice.host.clone(), choice.identity.clone())
                        })
                        .cloned();
                    if let Some(mut host) = host {
                        if host.kind == HostKind::Local && !choice.path.is_dir() {
                            self.place_message = Some(
                                "Directory is missing or unavailable; choose another place".into(),
                            );
                            return None;
                        }
                        host.workdir = choice.path.to_str().map(str::to_owned);
                        self.pending_workbench = Some(host);
                        self.mode = ClusterMode::Normal;
                        return Some(ClusterCommand::OpenWorkbench);
                    }
                    self.place_message = Some(
                        "Saved host identity no longer matches inventory; remove this entry".into(),
                    );
                }
            }
            "pin_place" | "remove_place" => {
                if let Some(choice) = choice {
                    let result = if action == "pin_place" {
                        self.places
                            .toggle_pin(&choice.host, &choice.identity, &choice.path)
                    } else {
                        self.places
                            .remove(&choice.host, &choice.identity, &choice.path)
                    };
                    self.place_message = Some(match result {
                        Ok(()) if !self.places.is_enabled() => "Places history is disabled".into(),
                        Ok(()) if action == "remove_place" && choice.configured => {
                            "Saved entry removed; configured workdir remains available".into()
                        }
                        Ok(()) => "Saved place updated".into(),
                        Err(error) => error.to_string(),
                    });
                    self.rebuild_places();
                }
            }
            "clear_recent" => {
                if let Some(choice) = choice {
                    self.place_message = Some(
                        match self.places.clear_recent(&choice.host, &choice.identity) {
                            Ok(()) => {
                                format!("Recent places cleared for {}; pins kept", choice.host)
                            }
                            Err(error) => error.to_string(),
                        },
                    );
                    self.rebuild_places();
                } else {
                    self.place_message = Some(
                        "Select a place to choose which host's recent entries to clear".into(),
                    );
                }
            }
            _ => {}
        }
        None
    }
    pub fn history_for(&self, alias: &str) -> Option<&VecDeque<crate::metrics::MetricSample>> {
        self.history.get(alias)
    }
    pub fn detail_offset(&self) -> u16 {
        self.detail_offset.min(self.detail_limit.get())
    }
    pub fn set_detail_limit(&self, limit: u16) {
        self.detail_limit.set(limit);
    }
    pub fn activity_symbol(&self) -> char {
        ['|', '/', '-', '\\'][(self.animation_frame % 4) as usize]
    }

    /// Only active probes request animation frames; idle state never does.
    pub fn animate(&mut self, elapsed: Duration, enabled: bool) -> bool {
        let frame = (elapsed.as_millis() / 250) as u64;
        if enabled
            && self.is_refreshing()
            && self.mode != ClusterMode::Help
            && self.actions.is_none()
            && frame != self.animation_frame
        {
            self.animation_frame = frame;
            true
        } else {
            false
        }
    }

    pub fn mode(&self) -> ClusterMode {
        self.mode
    }

    pub fn hosts(&self) -> &[HostConfig] {
        &self.hosts
    }

    pub fn visible_hosts(&self) -> impl Iterator<Item = &HostConfig> {
        self.visible.iter().map(|index| &self.hosts[*index])
    }
    pub fn visible_count(&self) -> usize {
        self.visible.len()
    }
    pub fn filter(&self) -> &str {
        &self.filter
    }
    pub fn sort_label(&self) -> String {
        format!(
            "{} {}",
            self.sort.label(),
            if self.sort_reverse { "desc" } else { "asc" }
        )
    }

    fn rebuild_view(&mut self) {
        let selected = self.selected_host().map(|host| host.alias().to_owned());
        let query = self.filter.to_lowercase();
        let mut visible = (0..self.hosts.len())
            .filter(|index| {
                let host = &self.hosts[*index];
                (!self.attention_only || self.needs_attention(host.alias()))
                    && [host.alias(), host.address(), host.role()]
                        .iter()
                        .any(|s| s.to_lowercase().contains(&query))
            })
            .collect::<Vec<_>>();
        let metric = |index: usize| -> Option<f64> {
            let snapshot = self.snapshots.get(self.hosts[index].alias())?;
            match self.sort {
                HostSort::Load => snapshot
                    .report
                    .cpu_load
                    .as_deref()?
                    .split_whitespace()
                    .next()?
                    .parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite() && *v >= 0.0),
                HostSort::Memory => snapshot
                    .report
                    .memory
                    .as_deref()
                    .and_then(crate::metrics::memory_used)
                    .map(f64::from),
                HostSort::Disk => snapshot
                    .report
                    .storage
                    .as_deref()
                    .and_then(crate::metrics::percent)
                    .map(f64::from),
                HostSort::Probe => snapshot.latency_ms.map(|value| value as f64),
                _ => None,
            }
        };
        visible.sort_by(|a, b| {
            use std::cmp::Ordering;
            let ordering = match self.sort {
                HostSort::Inventory => a.cmp(b),
                HostSort::Alias => self.hosts[*a]
                    .alias()
                    .to_lowercase()
                    .cmp(&self.hosts[*b].alias().to_lowercase()),
                HostSort::State => {
                    let rank = |index: usize| match self
                        .snapshots
                        .get(self.hosts[index].alias())
                        .map(|s| s.connection)
                        .unwrap_or(ConnectionState::Unknown)
                    {
                        ConnectionState::Offline
                        | ConnectionState::Timeout
                        | ConnectionState::AuthFailed => 0,
                        ConnectionState::Stale => 1,
                        ConnectionState::Unknown => 2,
                        ConnectionState::Checking => 3,
                        ConnectionState::Online => 4,
                    };
                    rank(*a).cmp(&rank(*b))
                }
                _ => match (metric(*a), metric(*b)) {
                    (Some(a), Some(b)) => a.total_cmp(&b),
                    (Some(_), None) => return Ordering::Less,
                    (None, Some(_)) => return Ordering::Greater,
                    (None, None) => Ordering::Equal,
                },
            };
            let ordering = if self.sort_reverse {
                ordering.reverse()
            } else {
                ordering
            };
            ordering.then_with(|| self.hosts[*a].alias().cmp(self.hosts[*b].alias()))
        });
        self.visible = visible;
        self.cursor = selected
            .and_then(|alias| {
                self.visible
                    .iter()
                    .position(|index| self.hosts[*index].alias() == alias)
            })
            .unwrap_or(0);
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn selected_host(&self) -> Option<&HostConfig> {
        self.visible
            .get(self.cursor)
            .map(|index| &self.hosts[*index])
    }

    pub fn selected_snapshot(&self) -> Option<&HostSnapshot> {
        self.selected_host()
            .and_then(|host| self.snapshots.get(host.alias()))
    }

    pub fn snapshot_for(&self, alias: &str) -> Option<&HostSnapshot> {
        self.snapshots.get(alias)
    }

    pub fn online_count(&self) -> usize {
        self.count_state(ConnectionState::Online)
    }

    pub fn offline_count(&self) -> usize {
        self.count_state(ConnectionState::Offline)
    }

    pub fn failed_count(&self) -> usize {
        self.offline_count()
            + self.count_state(ConnectionState::Timeout)
            + self.count_state(ConnectionState::AuthFailed)
    }

    pub fn attention_only(&self) -> bool {
        self.attention_only
    }
    pub fn refresh_paused(&self) -> bool {
        self.refresh_paused
    }
    pub fn events(&self) -> &VecDeque<HostEvent> {
        &self.events
    }
    pub fn set_event_limit(&self, limit: usize) {
        self.event_limit.set(limit);
    }
    pub fn observation_count(&self, alias: &str) -> u64 {
        self.observations.get(alias).copied().unwrap_or(0)
    }
    pub fn data_age(&self, alias: &str) -> Option<Duration> {
        self.last_good_at
            .get(alias)
            .map(|at| at.elapsed().unwrap_or_default())
    }
    /// Update freshness-derived visibility without starting a probe.
    pub fn tick_freshness(&mut self) -> bool {
        if !self.update_expired_data() {
            return false;
        }
        self.rebuild_view();
        true
    }

    fn update_expired_data(&mut self) -> bool {
        let now = SystemTime::now();
        let mut changed = false;
        for (alias, at) in &self.last_good_at {
            if now.duration_since(*at).unwrap_or_default() > Duration::from_secs(45) {
                if !self.expired_data.contains(alias) {
                    self.expired_data.insert(alias.clone());
                    changed = true;
                }
            } else {
                changed |= self.expired_data.remove(alias);
            }
        }
        changed
    }
    pub fn set_disk_threshold(&mut self, threshold: u16) -> Result<()> {
        anyhow::ensure!(threshold <= 100, "disk threshold must be from 0 to 100");
        self.disk_threshold = threshold;
        self.rebuild_view();
        Ok(())
    }
    pub fn attention_reason(&self, alias: &str) -> Option<String> {
        let snapshot = self.snapshots.get(alias)?;
        let state = if snapshot.connection == ConnectionState::Checking {
            self.final_states
                .get(alias)
                .copied()
                .unwrap_or(ConnectionState::Unknown)
        } else {
            snapshot.connection
        };
        if state != ConnectionState::Online {
            return Some(format!("{}; inspect details", state.label()));
        }
        if snapshot.report.is_empty() {
            return Some("metrics unknown; no valid observation yet".into());
        }
        if self.expired_data.contains(alias) {
            return Some("last data is over 45s old; refresh when ready".into());
        }
        if let Some(disk) = snapshot
            .report
            .storage
            .as_deref()
            .and_then(crate::metrics::percent)
            && disk >= self.disk_threshold
        {
            return Some(format!("disk {disk}% >= {}%", self.disk_threshold));
        }
        None
    }
    pub fn needs_attention(&self, alias: &str) -> bool {
        self.attention_reason(alias).is_some()
    }

    pub fn checking_count(&self) -> usize {
        self.count_state(ConnectionState::Checking)
    }

    pub fn stale_count(&self) -> usize {
        self.count_state(ConnectionState::Stale)
    }

    pub fn logs(&self) -> &[String] {
        &self.logs
    }

    pub fn inventory_label(&self) -> String {
        self.inventory_source
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "fallback local inventory".to_string())
    }

    fn is_refreshing(&self) -> bool {
        !self.refreshing.is_empty()
    }

    fn refresh_due(&self, interval: Duration) -> bool {
        !self.refresh_paused
            && self.active_probes.len() < MAX_CONCURRENT_PROBES
            && !self.scheduled_aliases(interval).is_empty()
    }

    fn scheduled_aliases(&self, interval: Duration) -> Vec<String> {
        if self.refresh_paused {
            return Vec::new();
        }
        let now = Instant::now();
        let selected = self.selected_host().map(HostConfig::alias);
        let mut aliases = self
            .hosts
            .iter()
            .filter(|host| {
                !self.active_probes.contains_key(host.alias())
                    && self
                        .next_due
                        .get(host.alias())
                        .map(|due| *due <= now)
                        .unwrap_or_else(|| {
                            self.last_attempt
                                .get(host.alias())
                                .is_none_or(|at| at.elapsed() >= interval)
                        })
            })
            .map(|host| host.alias().to_owned())
            .collect::<Vec<_>>();
        // Never-probed and oldest-attempt hosts win. The selected host wins a
        // tie, so priority cannot starve hosts in a larger inventory.
        aliases.sort_by_key(|alias| {
            (
                self.last_attempt.get(alias).copied(),
                selected != Some(alias.as_str()),
            )
        });
        aliases
    }

    fn count_state(&self, state: ConnectionState) -> usize {
        self.host_snapshots()
            .filter(|snapshot| snapshot.connection == state)
            .count()
    }

    fn mark_timed_out_refreshes(&mut self) -> bool {
        let now = Instant::now();
        let timed_out_aliases = self
            .refresh_deadlines
            .iter()
            .filter(|(_, deadline)| now >= **deadline)
            .map(|(alias, _)| alias.clone())
            .collect::<Vec<_>>();
        let changed = !timed_out_aliases.is_empty();

        for alias in timed_out_aliases {
            if let Some(active) = self.active_probes.get_mut(&alias) {
                active.timed_out = true;
            }
            self.refreshing.remove(&alias);
            self.refresh_deadlines.remove(&alias);
            self.apply_snapshot_inner(HostSnapshot::failed(
                &alias,
                format!("probe timed out after {}s", PROBE_TIMEOUT.as_secs()),
            ));
        }
        changed
    }

    fn has_host_alias(&self, alias: &str) -> bool {
        self.hosts.iter().any(|host| host.alias() == alias)
    }

    fn host_snapshots(&self) -> impl Iterator<Item = &HostSnapshot> {
        self.hosts
            .iter()
            .filter_map(|host| self.snapshots.get(host.alias()))
    }

    fn next_refresh_token(&mut self) -> u64 {
        self.refresh_epoch = self.refresh_epoch.wrapping_add(1).max(1);
        self.refresh_epoch
    }

    fn refresh_token(&self, alias: &str) -> Option<u64> {
        self.active_probes.get(alias).map(|probe| probe.token)
    }

    #[cfg(test)]
    fn refresh_token_for_test(&self, alias: &str) -> Option<u64> {
        self.refresh_token(alias)
    }

    fn merge_last_good_snapshot(&self, snapshot: HostSnapshot) -> HostSnapshot {
        if !matches!(
            snapshot.connection,
            ConnectionState::Offline | ConnectionState::Timeout | ConnectionState::AuthFailed
        ) {
            return snapshot;
        }
        let Some(previous) = self.snapshots.get(&snapshot.alias) else {
            return snapshot;
        };
        let last_good_report = self
            .last_good_reports
            .get(&snapshot.alias)
            .cloned()
            .or_else(|| {
                if previous.connection == ConnectionState::Online && !previous.report.is_empty() {
                    Some(previous.report.clone())
                } else {
                    None
                }
            });
        let Some(report) = last_good_report else {
            return snapshot;
        };
        HostSnapshot::stale(
            snapshot.alias,
            report,
            snapshot
                .error
                .unwrap_or_else(|| "refresh failed".to_string()),
        )
    }

    fn move_cursor(&mut self, delta: isize) {
        self.detail_offset = 0;
        self.cursor = self
            .cursor
            .saturating_add_signed(delta)
            .min(self.visible.len().saturating_sub(1));
    }

    fn log(&mut self, message: impl Into<String>) {
        self.logs.push(message.into());
        if self.logs.len() > 6 {
            self.logs.remove(0);
        }
    }
}

pub fn run() -> Result<()> {
    let inventory = ClusterInventory::load_default()?;
    run_with_inventory(inventory)
}

pub fn run_with_config_path(path: Option<&Path>) -> Result<()> {
    run_with_keymap(path, crate::keymap::Keymap::default())
}

pub fn run_with_keymap(path: Option<&Path>, keymap: crate::keymap::Keymap) -> Result<()> {
    let inventory = match path {
        Some(path) => ClusterInventory::from_path(path)?,
        None => ClusterInventory::load_default()?,
    };
    run_inventory_with_keymap(inventory, keymap)
}

pub fn run_with_inventory(inventory: ClusterInventory) -> Result<()> {
    run_inventory_with_keymap(inventory, crate::keymap::Keymap::default())
}

fn run_inventory_with_keymap(
    inventory: ClusterInventory,
    keymap: crate::keymap::Keymap,
) -> Result<()> {
    let disk_threshold = parse_disk_threshold(env::var("TERSH_DISK_WARN").ok().as_deref())?;
    let guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut app = ClusterApp::from_inventory(inventory);
    app.set_disk_threshold(disk_threshold)?;
    match crate::places::Places::load_default() {
        Ok(places) => app.set_places(places),
        Err(error) => {
            app.set_places(crate::places::Places::disabled());
            app.log(format!("Places unavailable: {error}"));
        }
    }
    app.set_keymap(keymap);
    let (tx, rx) = mpsc::channel();
    let mut workers = ProbeWorkers::default();
    start_scheduled_refresh(&mut app, tx.clone(), &mut workers);
    let mut dirty = true;
    let animation_started = Instant::now();
    let motion = crate::theme::motion_enabled();

    while !app.should_quit() {
        workers.reap();
        if app.tick_freshness() {
            dirty = true;
        }
        if app.animate(animation_started.elapsed(), motion) {
            dirty = true;
        }
        if drain_snapshots(&mut app, &rx) {
            dirty = true;
        }
        if app.mark_timed_out_refreshes() {
            dirty = true;
        }

        if app.refresh_due(DEFAULT_REFRESH_INTERVAL) {
            start_scheduled_refresh(&mut app, tx.clone(), &mut workers);
            dirty = true;
        }

        if dirty {
            terminal.draw(|frame| crate::cluster_ui::draw(frame, &app))?;
            dirty = false;
        }
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    dirty = true;
                    match app.handle_key(key) {
                        Some(ClusterCommand::RefreshAll) => {
                            start_refresh_all(&mut app, tx.clone(), &mut workers)
                        }
                        Some(ClusterCommand::RefreshSelected) => {
                            start_refresh_selected(&mut app, tx.clone(), &mut workers)
                        }
                        Some(ClusterCommand::OpenSession) => {
                            terminal.show_cursor()?;
                            open_selected_session(&mut app, &guard)?;
                            terminal.clear()?;
                            drain_snapshots(&mut app, &rx);
                            start_refresh_selected(&mut app, tx.clone(), &mut workers);
                        }
                        Some(ClusterCommand::OpenWorkbench) => {
                            terminal.show_cursor()?;
                            open_selected_workbench(&mut app, &guard)?;
                            terminal.clear()?;
                            drain_snapshots(&mut app, &rx);
                            start_refresh_selected(&mut app, tx.clone(), &mut workers);
                        }
                        _ => {}
                    }
                }
                Event::Resize(_, _) => {
                    terminal.clear()?;
                    dirty = true;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn drain_snapshots(app: &mut ClusterApp, rx: &mpsc::Receiver<ProbeSnapshot>) -> bool {
    let mut changed = false;
    while let Ok(snapshot) = rx.try_recv() {
        app.apply_probe_snapshot(snapshot);
        changed = true;
    }
    changed
}

pub fn collect_host_snapshot(host: &HostConfig) -> HostSnapshot {
    collect_host_snapshot_cancellable(host, &AtomicBool::new(false))
}

fn collect_host_snapshot_cancellable(host: &HostConfig, cancel: &AtomicBool) -> HostSnapshot {
    let started = Instant::now();
    let mut command = if host.kind == HostKind::Local {
        let (program, args) = local_probe_shell();
        let mut command = ProcessCommand::new(program);
        command.args(args).arg(PROBE_SCRIPT);
        command
    } else {
        let mut command = ProcessCommand::new("ssh");
        command.args(ssh_probe_args(host));
        command
    };
    command.stdin(Stdio::null());
    let result = run_command_with_cancellation(command, PROBE_TIMEOUT, cancel);
    match result {
        Ok(output) => HostSnapshot::online(
            host.alias(),
            ProbeReport::parse(&output),
            started.elapsed().as_millis(),
        ),
        Err(err) => HostSnapshot::failed(host.alias(), err.to_string()),
    }
}

fn start_refresh_all(
    app: &mut ClusterApp,
    tx: mpsc::Sender<ProbeSnapshot>,
    workers: &mut ProbeWorkers,
) {
    let hosts = app.hosts().to_vec();
    let aliases = hosts
        .iter()
        .map(|host| host.alias().to_string())
        .collect::<Vec<_>>();
    let started = app.begin_refresh(&aliases);
    spawn_probes(app, started, tx, workers);
}

fn start_scheduled_refresh(
    app: &mut ClusterApp,
    tx: mpsc::Sender<ProbeSnapshot>,
    workers: &mut ProbeWorkers,
) {
    let aliases = app.scheduled_aliases(DEFAULT_REFRESH_INTERVAL);
    let started = app.begin_refresh_ordered(&aliases, false);
    spawn_probes(app, started, tx, workers);
}

fn spawn_probes(
    app: &ClusterApp,
    aliases: Vec<String>,
    tx: mpsc::Sender<ProbeSnapshot>,
    workers: &mut ProbeWorkers,
) {
    let started = aliases.into_iter().collect::<BTreeSet<_>>();
    for host in app.hosts() {
        if !started.contains(host.alias()) {
            continue;
        }
        let Some(token) = app.refresh_token(host.alias()) else {
            continue;
        };
        workers.spawn(host.clone(), token, tx.clone());
    }
}

fn start_refresh_selected(
    app: &mut ClusterApp,
    tx: mpsc::Sender<ProbeSnapshot>,
    workers: &mut ProbeWorkers,
) {
    let Some(host) = app.selected_host().cloned() else {
        return;
    };
    if app.begin_refresh(&[host.alias().to_string()]).is_empty() {
        return;
    }
    let Some(token) = app.refresh_token(host.alias()) else {
        return;
    };
    workers.spawn(host, token, tx);
}

pub fn ssh_probe_args(host: &HostConfig) -> Vec<String> {
    let mut args = vec![
        "-n".to_string(),
        "-o".to_string(),
        "BatchMode=yes".to_string(),
        "-o".to_string(),
        "ClearAllForwardings=yes".to_string(),
        "-o".to_string(),
        "PermitLocalCommand=no".to_string(),
        "-o".to_string(),
        "ConnectTimeout=15".to_string(),
        "-o".to_string(),
        "ConnectionAttempts=1".to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=yes".to_string(),
        "-o".to_string(),
        "ServerAliveInterval=10".to_string(),
        "-o".to_string(),
        "ServerAliveCountMax=2".to_string(),
    ];
    if let Some(proxy_jump) = host.proxy_jump() {
        args.push("-J".to_string());
        args.push(host.proxy_jump_target().unwrap_or(proxy_jump).to_string());
    }
    args.push(host.ssh_target().to_string());
    args.push(remote_probe_command(PROBE_SCRIPT));
    args
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCommand {
    program: String,
    args: Vec<String>,
    environment: Vec<(String, String)>,
}

impl SessionCommand {
    pub fn program(&self) -> &str {
        &self.program
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    fn display(&self) -> String {
        if self.args.is_empty() {
            return self.program.clone();
        }
        format!("{} {}", self.program, self.args.join(" "))
    }
}

pub fn host_session_command(host: &HostConfig, local_shell: Option<&str>) -> SessionCommand {
    if host.kind() == HostKind::Local {
        let shell = local_shell
            .map(str::trim)
            .filter(|shell| !shell.is_empty())
            .unwrap_or("/bin/sh");
        return SessionCommand {
            program: shell.to_string(),
            args: Vec::new(),
            environment: Vec::new(),
        };
    }

    SessionCommand {
        program: "ssh".to_string(),
        args: ssh_session_args(host),
        environment: Vec::new(),
    }
}

pub fn ssh_session_args(host: &HostConfig) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(proxy_jump) = host.proxy_jump() {
        args.push("-J".to_string());
        args.push(host.proxy_jump_target().unwrap_or(proxy_jump).to_string());
    }
    args.push(host.ssh_target().to_string());
    args
}

pub fn host_workbench_command(host: &HostConfig, local_tersh_program: &str) -> SessionCommand {
    if host.kind() == HostKind::Local {
        let program = if local_tersh_program.trim().is_empty() {
            "tersh"
        } else {
            local_tersh_program
        };
        let mut args = Vec::new();
        if let Some(workdir) = host.workdir().filter(|workdir| !workdir.trim().is_empty()) {
            args.push("--".to_string());
            args.push(workdir.to_string());
        }
        return SessionCommand {
            program: program.to_string(),
            args,
            environment: vec![
                ("TERSH_HOST_ALIAS".into(), "local".into()),
                ("TERSH_HOST_ID".into(), "local".into()),
            ],
        };
    }

    SessionCommand {
        program: "ssh".to_string(),
        args: ssh_workbench_args(host),
        environment: Vec::new(),
    }
}

pub fn ssh_workbench_args(host: &HostConfig) -> Vec<String> {
    let mut args = vec!["-t".to_string()];
    if let Some(proxy_jump) = host.proxy_jump() {
        args.push("-J".to_string());
        args.push(host.proxy_jump_target().unwrap_or(proxy_jump).to_string());
    }
    args.push(host.ssh_target().to_string());
    args.push(remote_workbench_command(host));
    args
}

fn open_selected_session(app: &mut ClusterApp, guard: &TerminalGuard) -> Result<()> {
    let Some(host) = app.selected_host().cloned() else {
        return Ok(());
    };
    let local_shell = env::var("SHELL").ok();
    let command = host_session_command(&host, local_shell.as_deref());

    app.log(format!("opening session: {}", host.alias()));
    let result = guard.suspend(|| run_session_command(&command))?;
    match result {
        Ok(status) => app.log(format!("session closed: {} ({status})", host.alias())),
        Err(err) => app.log(format!("session failed: {}: {err}", host.alias())),
    }
    Ok(())
}

fn open_selected_workbench(app: &mut ClusterApp, guard: &TerminalGuard) -> Result<()> {
    let Some(host) = app
        .pending_workbench
        .take()
        .or_else(|| app.selected_host().cloned())
    else {
        return Ok(());
    };
    let current_tersh = current_tersh_program();
    let command = host_workbench_command(&host, &current_tersh);

    app.log(format!("opening tersh: {}", host.alias()));
    let result = guard.suspend(|| run_session_command(&command))?;
    // The child workbench may also have saved local places. Reload before an
    // explicit visit so the parent never overwrites a concurrent state change.
    if let Ok(places) = crate::places::Places::load_default() {
        app.set_places(places);
    }
    match result {
        Ok(status) => {
            app.log(format!("tersh closed: {} ({status})", host.alias()));
            if status.success()
                && let Some(path) = host.workdir().filter(|path| Path::new(path).is_absolute())
            {
                let (scope, identity) = host.place_scope();
                if let Err(error) = app.places.visit(&scope, &identity, Path::new(path)) {
                    app.log(format!("Places: {error}"));
                }
            }
        }
        Err(err) => app.log(format!("tersh failed: {}: {err}", host.alias())),
    }
    Ok(())
}

fn run_session_command(command: &SessionCommand) -> Result<std::process::ExitStatus> {
    ProcessCommand::new(command.program())
        .args(command.args())
        .envs(command.environment.iter().cloned())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("start {}", command.display()))
}

fn current_tersh_program() -> String {
    env::current_exe()
        .ok()
        .and_then(|path| path.into_os_string().into_string().ok())
        .unwrap_or_else(|| "tersh".to_string())
}

fn remote_workbench_command(host: &HostConfig) -> String {
    let mut script = format!(
        "{}; if ! command -v tersh >/dev/null 2>&1; then printf '%s\\n' {} >&2; exit 127; fi",
        PATH_BOOTSTRAP_SCRIPT,
        shell_quote(
            "tersh is not installed or not in PATH. Install: cargo install --locked --git https://github.com/QiushanHuang/Tersh.git --bin tersh --force"
        )
    );
    if let Some(workdir) = host.workdir().filter(|workdir| !workdir.trim().is_empty()) {
        script.push_str(&format!(
            "; cd -- {} || {{ printf '%s\\n' {} >&2; exit 1; }}",
            shell_quote(workdir),
            shell_quote(&format!("tersh workdir not found: {workdir}"))
        ));
    }
    for (key, value) in presentation_environment() {
        script.push_str(&format!("; {key}={}; export {key}", shell_quote(&value)));
    }
    let (scope, identity) = host.place_scope();
    for (key, value) in [("TERSH_HOST_ALIAS", scope), ("TERSH_HOST_ID", identity)] {
        script.push_str(&format!("; {key}={}; export {key}", shell_quote(&value)));
    }
    script.push_str("; exec tersh");
    remote_probe_command(&script)
}

fn presentation_environment() -> Vec<(&'static str, String)> {
    // Only validated presentation enums cross the SSH boundary. In particular,
    // never forward arbitrary environment, paths, credentials or keymap files.
    let allowed: &[(&str, &[&str])] = &[
        ("TERSH_THEME", &["btop", "aurora", "contrast", "mono"]),
        ("TERSH_BORDER", &["ascii", "rounded", "thick"]),
        ("TERSH_FOOTER", &["auto", "compact", "full"]),
        ("TERSH_GLYPHS", &["ascii", "unicode"]),
        ("TERSH_MOTION", &["off", "0", "reduced", "on", "1"]),
        ("TERSH_COLOR", &["off", "on"]),
        ("TERSH_CLIPBOARD", &["off", "on"]),
    ];
    let mut values = allowed
        .iter()
        .filter_map(|(key, valid)| {
            let value = env::var(key).ok()?.to_ascii_lowercase();
            valid.contains(&value.as_str()).then_some((*key, value))
        })
        .collect::<Vec<_>>();
    if env::var_os("NO_COLOR").is_some() {
        values.push(("NO_COLOR", "1".into()));
    }
    values
}

fn remote_probe_command(script: &str) -> String {
    if cfg!(windows) {
        format!("cmd /C \"{}\"", script.replace('"', "\\\""))
    } else {
        format!("sh -c {}", shell_quote(script))
    }
}

#[cfg(not(windows))]
fn local_probe_shell() -> (&'static str, Vec<String>) {
    ("sh", vec!["-c".to_string()])
}

#[cfg(windows)]
fn local_probe_shell() -> (&'static str, Vec<String>) {
    ("cmd", vec!["/C".to_string()])
}

#[cfg(test)]
fn run_command_with_timeout(command: ProcessCommand, timeout: Duration) -> Result<String> {
    run_command_with_cancellation(command, timeout, &AtomicBool::new(false))
}

fn run_command_with_cancellation(
    mut command: ProcessCommand,
    timeout: Duration,
    cancel: &AtomicBool,
) -> Result<String> {
    anyhow::ensure!(!cancel.load(Ordering::Acquire), "probe cancelled");
    configure_probe_command(&mut command);
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("start probe command")?;
    let result = (|| {
        let mut stdout = child.stdout.take().context("capture probe stdout")?;
        let mut stderr = child.stderr.take().context("capture probe stderr")?;
        stdout.prepare().context("prepare probe stdout")?;
        stderr.prepare().context("prepare probe stderr")?;
        let mut stdout_bytes = Vec::new();
        let mut stderr_bytes = Vec::new();
        let mut stdout_done = false;
        let mut stderr_done = false;
        let mut status = None;
        let started = Instant::now();
        loop {
            anyhow::ensure!(!cancel.load(Ordering::Acquire), "probe cancelled");
            let previous_bytes = stdout_bytes.len() + stderr_bytes.len();
            if !stdout_done {
                stdout_done = drain_probe_pipe(&mut stdout, "stdout", &mut stdout_bytes)?;
            }
            if !stderr_done {
                stderr_done = drain_probe_pipe(&mut stderr, "stderr", &mut stderr_bytes)?;
            }
            if status.is_none() {
                status = child.try_wait()?;
            }
            // A completed shell can leave descendants holding either pipe open.
            // Readiness checks and the deadline stay on this worker: no blocking
            // reader threads or inherited pipe handles outlive the probe.
            if let Some(status) = status
                && stdout_done
                && stderr_done
            {
                if status.success() {
                    return Ok(String::from_utf8_lossy(&stdout_bytes).into_owned());
                }
                let stderr = String::from_utf8_lossy(&stderr_bytes);
                if stderr.trim().is_empty() {
                    anyhow::bail!("probe exited with {status}");
                }
                anyhow::bail!("{}", stderr.trim());
            }
            if started.elapsed() >= timeout {
                let mut message = format!("probe timed out after {}s", timeout.as_secs());
                let stderr = String::from_utf8_lossy(&stderr_bytes);
                if !stderr.trim().is_empty() {
                    message.push_str(": ");
                    message.push_str(stderr.trim());
                }
                anyhow::bail!(message);
            }
            if previous_bytes == stdout_bytes.len() + stderr_bytes.len() {
                thread::sleep(
                    Duration::from_millis(50).min(timeout.saturating_sub(started.elapsed())),
                );
            }
        }
    })();
    // The closure has already dropped both read ends, even if an escaped
    // descendant kept the writers open. Kill/reap the direct process without
    // waiting for arbitrary descendants or their pipe EOF.
    if result.is_err() {
        terminate_probe_child(&mut child);
        let cleanup_started = Instant::now();
        while matches!(child.try_wait(), Ok(None))
            && cleanup_started.elapsed() < Duration::from_millis(100)
        {
            thread::sleep(Duration::from_millis(5));
        }
    }
    result
}

trait ProbePipe: Read {
    fn prepare(&self) -> io::Result<()>;
    fn read_available(&mut self, buffer: &mut [u8]) -> io::Result<usize>;
}

#[cfg(unix)]
impl<T: Read + std::os::fd::AsRawFd> ProbePipe for T {
    fn prepare(&self) -> io::Result<()> {
        let descriptor = self.as_raw_fd();
        let flags = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
        if flags == -1
            || unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK) } == -1
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn read_available(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.read(buffer)
    }
}

#[cfg(windows)]
impl<T: Read + std::os::windows::io::AsRawHandle> ProbePipe for T {
    fn prepare(&self) -> io::Result<()> {
        Ok(())
    }

    fn read_available(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        // Anonymous child pipes support PeekNamedPipe. Read only bytes already
        // available, avoiding an uncancellable blocking ReadFile operation.
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn PeekNamedPipe(
                handle: *mut std::ffi::c_void,
                buffer: *mut std::ffi::c_void,
                size: u32,
                read: *mut u32,
                available: *mut u32,
                left: *mut u32,
            ) -> i32;
        }
        let mut available = 0;
        let success = unsafe {
            PeekNamedPipe(
                self.as_raw_handle(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut available,
                std::ptr::null_mut(),
            )
        };
        if success == 0 {
            let error = io::Error::last_os_error();
            return if error.raw_os_error() == Some(109) {
                Ok(0)
            } else {
                Err(error)
            };
        }
        if available == 0 {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let count = buffer.len().min(available as usize);
        self.read(&mut buffer[..count])
    }
}

fn drain_probe_pipe(
    reader: &mut impl ProbePipe,
    stream: &str,
    bytes: &mut Vec<u8>,
) -> Result<bool> {
    let mut buffer = [0; 8192];
    // Bound each drain so noisy output cannot starve stderr or deadline checks.
    for _ in 0..16 {
        let read = match reader.read_available(&mut buffer) {
            Ok(0) => return Ok(true),
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(false),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => {
                return Err(error).with_context(|| format!("failed to read probe {stream}"));
            }
        };
        if bytes.len().saturating_add(read) > MAX_PROBE_OUTPUT_BYTES as usize {
            anyhow::bail!(
                "probe output too large: {stream} exceeded {MAX_PROBE_OUTPUT_BYTES} bytes"
            );
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    Ok(false)
}

#[cfg(unix)]
fn configure_probe_command(command: &mut ProcessCommand) {
    use std::os::unix::process::CommandExt;

    // Put each probe in its own process group so timeout cleanup reaches shell children too.
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        });
    }
}

#[cfg(not(unix))]
fn configure_probe_command(_command: &mut ProcessCommand) {}

#[cfg(unix)]
fn terminate_probe_child(child: &mut Child) {
    let pid = child.id() as libc::pid_t;
    unsafe {
        let _ = libc::killpg(pid, libc::SIGKILL);
    }
    let _ = child.kill();
}

#[cfg(not(unix))]
fn terminate_probe_child(child: &mut Child) {
    let _ = child.kill();
}

fn default_inventory_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        paths.push(cwd.join("ssh/servers.json"));
    }
    if let Some(home) = home_dir() {
        paths.push(home.join(".config/tersh/servers.json"));
    }
    paths
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

fn local_fallback_host() -> HostConfig {
    HostConfig {
        alias: "local".to_string(),
        kind: HostKind::Local,
        address: "127.0.0.1".to_string(),
        user: None,
        role: "Local machine".to_string(),
        proxy_jump: None,
        proxy_jump_target: None,
        ssh_target: "local".to_string(),
        workdir: None,
    }
}

fn connection_target(user: Option<&str>, address: &str) -> String {
    let address = address.trim();
    match user {
        Some(user) if !user.trim().is_empty() => format!("{}@{}", user.trim(), address),
        _ => address.to_string(),
    }
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_workdir(value: Option<String>) -> Option<String> {
    // Whitespace can be part of a real path. Only an all-whitespace setting
    // means no configured directory; never rewrite a nonempty path.
    value.filter(|value| !value.trim().is_empty())
}

fn validate_hosts(hosts: &[HostConfig]) -> Result<()> {
    let mut aliases = BTreeSet::new();
    let jump_aliases = hosts
        .iter()
        .filter(|host| host.kind == HostKind::Jump)
        .map(|host| host.alias.as_str())
        .collect::<BTreeSet<_>>();
    for host in hosts {
        validate_label("alias", &host.alias)?;
        if !aliases.insert(host.alias.clone()) {
            anyhow::bail!("duplicate alias: {}", host.alias);
        }
        validate_ssh_field("address", &host.address)?;
        if let Some(user) = &host.user {
            validate_ssh_field("ssh_user", user)?;
        }
        validate_ssh_field("ssh_target", &host.ssh_target)?;
        if let Some(proxy_jump) = &host.proxy_jump {
            validate_ssh_field("proxy_jump", proxy_jump)?;
            if !jump_aliases.contains(proxy_jump.as_str()) {
                anyhow::bail!("unresolved proxy_jump: {proxy_jump}");
            }
        }
        if let Some(proxy_target) = &host.proxy_jump_target {
            validate_ssh_field("proxy_jump_target", proxy_target)?;
        }
        validate_display_field("role", &host.role)?;
        if let Some(workdir) = &host.workdir {
            validate_display_field("workdir", workdir)?;
        }
    }
    Ok(())
}

fn validate_label(field: &'static str, value: &str) -> Result<()> {
    let value = value.trim();
    if value.is_empty() {
        anyhow::bail!("{field} must not be empty");
    }
    if value.chars().any(char::is_control) {
        anyhow::bail!("{field} contains control characters");
    }
    Ok(())
}

fn validate_ssh_field(field: &'static str, value: &str) -> Result<()> {
    validate_label(field, value)?;
    if value.chars().any(char::is_whitespace) {
        anyhow::bail!("{field} must not contain whitespace");
    }
    if value.trim_start().starts_with('-') {
        anyhow::bail!("{field} must not start with '-'");
    }
    Ok(())
}

fn validate_display_field(field: &'static str, value: &str) -> Result<()> {
    if value.chars().any(char::is_control) {
        anyhow::bail!("{field} contains control characters");
    }
    Ok(())
}

fn classify_failure(error: &str) -> ConnectionState {
    let lower = error.to_lowercase();
    if lower.contains("timed out")
        || lower.contains("operation timed out")
        || lower.contains("connection timeout")
    {
        ConnectionState::Timeout
    } else if lower.contains("permission denied")
        || lower.contains("publickey")
        || lower.contains("authentication")
    {
        ConnectionState::AuthFailed
    } else {
        ConnectionState::Offline
    }
}

fn shell_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', "'\\''"))
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

    fn suspend<T>(&self, action: impl FnOnce() -> T) -> Result<T> {
        execute!(io::stdout(), LeaveAlternateScreen).context("leave cluster status screen")?;
        if let Err(err) = disable_raw_mode() {
            if enable_raw_mode().is_ok() {
                let _ = execute!(io::stdout(), EnterAlternateScreen);
            }
            return Err(err.into());
        }

        let mut resume = TerminalResume::new();
        let result = action();

        resume.restore()?;
        Ok(result)
    }
}

struct TerminalResume {
    restored: bool,
}

impl TerminalResume {
    fn new() -> Self {
        Self { restored: false }
    }

    fn restore(&mut self) -> Result<()> {
        enable_raw_mode().context("restore raw mode")?;
        if let Err(err) = execute!(io::stdout(), EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(err).context("restore cluster status screen");
        }
        self.restored = true;
        Ok(())
    }
}

impl Drop for TerminalResume {
    fn drop(&mut self) {
        if !self.restored && enable_raw_mode().is_ok() {
            let _ = execute!(io::stdout(), EnterAlternateScreen);
        }
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeSet, sync::Mutex};

    static PROBE_TEMP_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn freshness_crossing_updates_paused_attention_once() {
        let mut app = ClusterApp::new(vec![test_host("host")]);
        app.apply_snapshot(HostSnapshot::online(
            "host",
            ProbeReport::parse("load=1"),
            1,
        ));
        app.apply(ClusterCommand::TogglePause);
        app.apply(ClusterCommand::ToggleAttention);
        assert_eq!(app.visible_count(), 0);
        // Advance just the observation age; no sleeping or remote I/O.
        app.last_good_at
            .insert("host".into(), SystemTime::now() - Duration::from_secs(46));
        assert!(
            app.tick_freshness(),
            "crossing freshness threshold needs one render"
        );
        assert_eq!(app.visible_count(), 1);
        assert!(app.needs_attention("host"));
        assert!(
            !app.tick_freshness(),
            "unchanged stale data must not continuously redraw"
        );
        assert!(!app.refresh_due(DEFAULT_REFRESH_INTERVAL));
    }

    #[test]
    fn online_connection_with_empty_metrics_still_needs_attention() {
        let mut app = ClusterApp::new(vec![test_host("host")]);
        app.apply_snapshot(HostSnapshot::online("host", ProbeReport::default(), 1));
        assert_eq!(
            app.snapshot_for("host").unwrap().connection,
            ConnectionState::Online
        );
        assert!(app.data_age("host").is_none());
        assert!(
            app.needs_attention("host"),
            "online connection is not a valid metric observation"
        );
        app.apply(ClusterCommand::ToggleAttention);
        assert_eq!(app.visible_count(), 1);
    }

    #[cfg(unix)]
    fn wait_for_fixture_pid(path: &Path) -> libc::pid_t {
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            if let Ok(contents) = fs::read_to_string(path)
                && let Ok(pid) = contents.trim().parse::<libc::pid_t>()
                && pid > 0
            {
                return pid;
            }
            assert!(
                Instant::now() < deadline,
                "fixture did not publish a complete PID"
            );
            thread::sleep(Duration::from_millis(5));
        }
    }

    #[cfg(unix)]
    #[test]
    fn cancelling_active_probe_reaps_process_promptly() {
        let cancel = Arc::new(AtomicBool::new(false));
        let child_cancel = cancel.clone();
        let dir = tempfile::tempdir().unwrap();
        let pid_path = dir.path().join("pid");
        let child_pid_path = pid_path.clone();
        let worker = thread::spawn(move || {
            let mut command = ProcessCommand::new("sh");
            command
                .args(["-c", "echo $$ > \"$1\"; exec sleep 0.4", "sh"])
                .arg(child_pid_path);
            run_command_with_cancellation(command, Duration::from_secs(1), &child_cancel)
        });
        let pid = wait_for_fixture_pid(&pid_path);
        let started = Instant::now();
        cancel.store(true, Ordering::Release);
        let error = worker.join().unwrap().unwrap_err();
        assert!(error.to_string().contains("cancelled"));
        assert!(started.elapsed() < Duration::from_millis(300));
        assert_eq!(
            unsafe { libc::kill(pid, 0) },
            -1,
            "probe process survived cancellation"
        );
    }

    #[test]
    fn automatic_refresh_pause_and_other_hosts_are_independent_of_inflight_batch() {
        let mut app = ClusterApp::new(
            (0..17)
                .map(|n| test_host(&format!("host-{n:02}")))
                .collect(),
        );
        app.apply(ClusterCommand::TogglePause);
        assert!(
            !app.refresh_due(DEFAULT_REFRESH_INTERVAL),
            "paused means no automatic probes"
        );
        app.apply(ClusterCommand::TogglePause);
        app.begin_refresh(&["host-00".into()]);
        assert!(
            app.refresh_due(DEFAULT_REFRESH_INTERVAL),
            "one slow host must not delay others"
        );
    }

    #[test]
    fn scheduler_prioritizes_selected_ties_and_visits_every_host_before_repeats() {
        let mut app = ClusterApp::new(
            (0..40)
                .map(|n| test_host(&format!("host-{n:02}")))
                .collect(),
        );
        for _ in 0..30 {
            app.apply(ClusterCommand::Down);
        }
        let mut seen = BTreeSet::new();
        for batch in 0..3 {
            let aliases = app.scheduled_aliases(DEFAULT_REFRESH_INTERVAL);
            if batch == 0 {
                assert_eq!(aliases[0], "host-30");
            }
            let started = app.begin_refresh_ordered(&aliases, false);
            assert!(started.len() <= MAX_CONCURRENT_PROBES);
            for alias in started {
                assert!(
                    seen.insert(alias.clone()),
                    "scheduler repeated {alias} before inventory complete"
                );
                app.apply_completed_refresh_snapshot(HostSnapshot::online(
                    alias,
                    ProbeReport::parse("load=1"),
                    1,
                ));
            }
        }
        assert_eq!(seen.len(), 40);
        assert!(app.scheduled_aliases(DEFAULT_REFRESH_INTERVAL).is_empty());
    }

    #[test]
    fn failed_hosts_back_off_but_manual_refresh_remains_available_when_paused() {
        let mut app = ClusterApp::new(vec![test_host("host")]);
        for seconds in [30, 60, 120, 120] {
            app.apply_completed_refresh_snapshot(HostSnapshot::failed("host", "probe timed out"));
            let remaining = app.next_due["host"].saturating_duration_since(Instant::now());
            assert!(
                remaining >= Duration::from_secs(seconds - 1)
                    && remaining <= Duration::from_secs(seconds)
            );
            assert!(app.scheduled_aliases(DEFAULT_REFRESH_INTERVAL).is_empty());
        }
        app.apply(ClusterCommand::TogglePause);
        assert_eq!(app.begin_refresh(&["host".into()]), vec!["host"]);
    }

    #[cfg(unix)]
    #[test]
    fn dashboard_probe_owner_drop_cancels_fake_ssh_and_reaps_it() {
        const TEST: &str =
            "cluster::tests::dashboard_probe_owner_drop_cancels_fake_ssh_and_reaps_it";
        const MARKER: &str = "TERSH_FAKE_SSH_PID";
        if env::var_os(MARKER).is_none() {
            use std::os::unix::fs::PermissionsExt;
            let temp = tempfile::tempdir().unwrap();
            let script = temp.path().join("ssh");
            fs::write(
                &script,
                "#!/bin/sh\necho $$ > \"$TERSH_FAKE_SSH_PID\"\nexec /bin/sleep 5\n",
            )
            .unwrap();
            fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
            let output = ProcessCommand::new(env::current_exe().unwrap())
                .args(["--exact", TEST, "--nocapture"])
                .env("PATH", temp.path())
                .env(MARKER, temp.path().join("pid"))
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }
        let pid_file = PathBuf::from(env::var_os(MARKER).unwrap());
        let (tx, _rx) = mpsc::channel();
        let mut workers = ProbeWorkers::default();
        workers.spawn(test_host("fake"), 1, tx);
        let pid = wait_for_fixture_pid(&pid_file);
        let started = Instant::now();
        drop(workers);
        assert!(started.elapsed() < Duration::from_secs(1));
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
    }

    #[test]
    fn slow_jump_probe_remains_checking_until_its_budget_expires() {
        let mut app = ClusterApp::new(vec![test_host("slow-jump")]);
        app.begin_refresh(&["slow-jump".to_string()]);
        // Advance the probe age without sleeping. A measured healthy jump
        // handshake took nearly ten seconds, before metrics collection.
        *app.refresh_deadlines.get_mut("slow-jump").unwrap() -= Duration::from_secs(10);
        assert!(!app.mark_timed_out_refreshes());
        assert_eq!(
            app.snapshot_for("slow-jump").unwrap().connection,
            ConnectionState::Checking
        );
        app.refresh_deadlines.insert(
            "slow-jump".to_string(),
            Instant::now() - Duration::from_secs(1),
        );
        assert!(app.mark_timed_out_refreshes());
        assert_eq!(
            app.snapshot_for("slow-jump").unwrap().connection,
            ConnectionState::Timeout
        );
    }

    #[test]
    fn begin_refresh_empty_aliases_does_not_reset_refresh_timer() {
        let mut app = ClusterApp::new(Vec::new());

        assert!(app.refresh_due(Duration::from_secs(60)));
        assert!(app.begin_refresh(&[]).is_empty());
        assert!(app.refresh_due(Duration::from_secs(60)));
    }

    #[test]
    fn timed_out_probe_result_does_not_overwrite_timeout_state() {
        let mut app = ClusterApp::new(vec![test_host("school-star")]);
        let aliases = vec!["school-star".to_string()];
        assert_eq!(app.begin_refresh(&aliases), aliases);
        let token = app.refresh_token_for_test("school-star").unwrap();
        app.refresh_deadlines.insert(
            "school-star".to_string(),
            Instant::now() - Duration::from_secs(1),
        );

        app.mark_timed_out_refreshes();
        app.apply_probe_snapshot(ProbeSnapshot {
            token,
            snapshot: HostSnapshot::online("school-star", ProbeReport::default(), 1),
        });

        assert_eq!(
            app.snapshot_for("school-star").unwrap().connection,
            ConnectionState::Timeout
        );
        assert_eq!(app.begin_refresh(&aliases), aliases);
    }

    #[test]
    fn timed_out_active_probe_retry_is_throttled_when_no_host_is_eligible() {
        let mut app = ClusterApp::new(vec![test_host("school-star")]);
        let aliases = vec!["school-star".to_string()];
        assert_eq!(app.begin_refresh(&aliases), aliases);
        app.refresh_deadlines.insert(
            "school-star".to_string(),
            Instant::now() - Duration::from_secs(1),
        );
        app.last_refresh_started = Some(Instant::now() - Duration::from_secs(120));

        app.mark_timed_out_refreshes();
        assert!(
            !app.refresh_due(Duration::from_secs(60)),
            "timeout must stay accounted until worker cleanup"
        );
        assert!(app.begin_refresh(&aliases).is_empty());

        assert!(!app.refresh_due(Duration::from_secs(60)));
    }

    #[test]
    fn external_snapshot_does_not_clear_active_refresh_accounting() {
        let mut app = ClusterApp::new(vec![test_host("school-star")]);
        let aliases = vec!["school-star".to_string()];
        assert_eq!(app.begin_refresh(&aliases), aliases);

        app.apply_snapshot(HostSnapshot::offline("school-star", "manual update"));

        assert!(app.begin_refresh(&aliases).is_empty());
    }

    #[test]
    fn cluster_app_deduplicates_constructor_hosts() {
        let app = ClusterApp::new(vec![test_host("dup"), test_host("dup")]);

        assert_eq!(app.hosts().len(), 1);
        assert!(app.snapshot_for("dup").is_some());
    }

    #[test]
    fn probe_script_bootstraps_common_non_login_shell_paths() {
        assert!(PROBE_SCRIPT.contains("$HOME/.cargo/bin"));
        assert!(PROBE_SCRIPT.contains("/usr/local/cuda/bin"));
        assert!(PROBE_SCRIPT.contains("export PATH"));
    }

    #[test]
    fn failed_probe_spawn_cleans_temp_files() {
        let _guard = PROBE_TEMP_LOCK.lock().unwrap();
        let before = probe_temp_files();
        let command = ProcessCommand::new("__tersh_missing_probe_binary__");

        let result = run_command_with_timeout(command, Duration::from_millis(1));

        assert!(result.is_err());
        let after = probe_temp_files();
        assert_eq!(after.difference(&before).count(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn probe_failure_preserves_non_utf8_stderr_lossily() {
        let _guard = PROBE_TEMP_LOCK.lock().unwrap();
        let mut command = ProcessCommand::new("sh");
        command.args(["-c", "printf '\\377' >&2; exit 2"]);

        let err = run_command_with_timeout(command, Duration::from_secs(1)).unwrap_err();

        assert!(err.to_string().contains('\u{fffd}'));
        assert!(!err.to_string().contains("failed to read"));
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "subprocess fixture for inherited-pipe regression"]
    fn escaped_probe_pipe_holder() {
        let Some(pid_file) = std::env::var_os("TERSH_TEST_PIPE_HOLDER_PID") else {
            return;
        };
        assert!(unsafe { libc::setsid() } >= 0);
        std::fs::write(pid_file, std::process::id().to_string()).unwrap();
        eprintln!("escaped-pipe diagnostic");
        thread::sleep(Duration::from_secs(10));
    }

    #[cfg(unix)]
    #[test]
    fn repeated_escaped_pipe_timeouts_do_not_leak_descriptors() {
        const TEST: &str = "cluster::tests::repeated_escaped_pipe_timeouts_do_not_leak_descriptors";
        if std::env::var_os("TERSH_TEST_PIPE_LEAK_ISOLATED").is_none() {
            // Isolate descriptor counts from other concurrently running tests.
            let output = ProcessCommand::new(std::env::current_exe().unwrap())
                .args(["--exact", TEST, "--nocapture"])
                .env("TERSH_TEST_PIPE_LEAK_ISOLATED", "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }
        struct KillHolders(Vec<PathBuf>);
        impl Drop for KillHolders {
            fn drop(&mut self) {
                for path in &self.0 {
                    if let Ok(text) = std::fs::read_to_string(path)
                        && let Ok(pid) = text.parse::<libc::pid_t>()
                    {
                        unsafe {
                            libc::kill(pid, libc::SIGKILL);
                        }
                    }
                }
            }
        }
        fn open_descriptors() -> usize {
            (0..1024)
                .filter(|fd| unsafe { libc::fcntl(*fd, libc::F_GETFD) } >= 0)
                .count()
        }
        let temp = tempfile::tempdir().unwrap();
        let mut holders = KillHolders(Vec::new());
        let before = open_descriptors();
        for attempt in 0..3 {
            let pid_file = temp.path().join(format!("holder-{attempt}"));
            holders.0.push(pid_file.clone());
            let mut command = ProcessCommand::new("sh");
            command.args([
                "-c",
                "\"$1\" --exact cluster::tests::escaped_probe_pipe_holder --ignored --nocapture & exit 0",
                "sh",
            ]).arg(std::env::current_exe().unwrap()).env("TERSH_TEST_PIPE_HOLDER_PID", &pid_file);
            let started = Instant::now();
            let error = run_command_with_timeout(command, Duration::from_millis(150)).unwrap_err();
            assert!(started.elapsed() < Duration::from_secs(1));
            assert!(pid_file.exists(), "setsid descendant did not start");
            assert!(error.to_string().contains("timed out"));
            assert!(error.to_string().contains("escaped-pipe diagnostic"));
        }
        assert_eq!(
            open_descriptors(),
            before,
            "timed-out readers retained pipe descriptors"
        );
    }

    #[cfg(unix)]
    #[test]
    fn completed_probe_preserves_stdout_before_deadline() {
        let mut command = ProcessCommand::new("sh");
        command.args([
            "-c",
            "printf 'hostname=early\\n'; printf 'diagnostic\\n' >&2",
        ]);

        let output = run_command_with_timeout(command, Duration::from_secs(1)).unwrap();

        assert_eq!(output, "hostname=early\n");
    }

    #[cfg(unix)]
    #[test]
    fn probe_deadline_covers_pipes_after_direct_child_exits() {
        let mut command = ProcessCommand::new("sh");
        command.args([
            "-c",
            "printf 'hostname=early\\n'; printf 'before timeout\\n' >&2; sleep 2 & exit 0",
        ]);
        let started = Instant::now();
        let result = run_command_with_timeout(command, Duration::from_millis(100));

        assert!(
            started.elapsed() < Duration::from_secs(1),
            "pipe readers exceeded the probe deadline"
        );
        let error = result.unwrap_err().to_string();
        assert!(error.contains("timed out"));
        assert!(error.contains("before timeout"));
    }

    #[cfg(unix)]
    #[test]
    fn probe_output_over_limit_is_rejected() {
        let _guard = PROBE_TEMP_LOCK.lock().unwrap();
        let mut command = ProcessCommand::new("sh");
        command.args([
            "-c",
            "dd if=/dev/zero bs=1024 count=1100 2>/dev/null | tr '\\0' x",
        ]);

        let err = run_command_with_timeout(command, Duration::from_secs(2)).unwrap_err();

        assert!(err.to_string().contains("probe output too large"));
    }

    fn probe_temp_files() -> BTreeSet<PathBuf> {
        let pid = std::process::id();
        let prefixes = [
            format!("tersh-probe-stdout-{pid}-"),
            format!("tersh-probe-stderr-{pid}-"),
        ];
        fs::read_dir(std::env::temp_dir())
            .into_iter()
            .flatten()
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| prefixes.iter().any(|prefix| name.starts_with(prefix)))
                    .unwrap_or(false)
            })
            .collect()
    }

    fn test_host(alias: &str) -> HostConfig {
        HostConfig {
            alias: alias.to_string(),
            kind: HostKind::Server,
            address: "203.0.113.10".to_string(),
            user: Some("ops".to_string()),
            role: "Remote server".to_string(),
            proxy_jump: None,
            proxy_jump_target: None,
            ssh_target: "ops@203.0.113.10".to_string(),
            workdir: None,
        }
    }
}
