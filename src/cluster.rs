use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs, io,
    path::{Path, PathBuf},
    process::{Child, Command as ProcessCommand, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const DEFAULT_REFRESH_INTERVAL: Duration = Duration::from_secs(15);
const PROBE_TIMEOUT: Duration = Duration::from_secs(6);
const MAX_CONCURRENT_PROBES: usize = 16;
const MAX_PROBE_OUTPUT_BYTES: u64 = 1024 * 1024;

const PROBE_SCRIPT: &str = r#"
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
                workdir: normalize_optional(main.workdir),
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
                workdir: normalize_optional(jump.workdir),
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
                workdir: normalize_optional(server.workdir),
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
}

#[derive(Debug, Clone)]
pub struct ClusterApp {
    hosts: Vec<HostConfig>,
    snapshots: BTreeMap<String, HostSnapshot>,
    last_good_reports: BTreeMap<String, ProbeReport>,
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
            hosts,
            snapshots,
            last_good_reports: BTreeMap::new(),
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
        match command {
            ClusterCommand::Down => self.move_cursor(1),
            ClusterCommand::Up => self.move_cursor(-1),
            ClusterCommand::First => self.cursor = 0,
            ClusterCommand::Last => self.cursor = self.hosts.len().saturating_sub(1),
            ClusterCommand::OpenDetail => self.mode = ClusterMode::Detail,
            ClusterCommand::OpenHelp => self.mode = ClusterMode::Help,
            ClusterCommand::Cancel => self.mode = ClusterMode::Normal,
            ClusterCommand::Quit => {
                if matches!(self.mode, ClusterMode::Help | ClusterMode::Detail) {
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
        let start = self.refresh_cursor % aliases.len();
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
            started.push(alias.clone());
            last_index = index;
            self.snapshots
                .insert(alias.clone(), HostSnapshot::checking(alias.clone()));
        }
        if !started.is_empty() {
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
        let snapshot = self.merge_last_good_snapshot(snapshot);
        let state = snapshot.connection.label();
        if snapshot.connection == ConnectionState::Online && !snapshot.report.is_empty() {
            self.last_good_reports
                .insert(alias.clone(), snapshot.report.clone());
        }
        self.snapshots.insert(alias.clone(), snapshot);
        self.log(format!("{alias}: {state}"));
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ClusterCommand> {
        let command = key_to_command(key)?;
        if self.mode == ClusterMode::Help
            && matches!(
                command,
                ClusterCommand::OpenHelp | ClusterCommand::RefreshSelected
            )
        {
            self.apply(ClusterCommand::Cancel);
            return None;
        }
        if self.mode == ClusterMode::Help
            && !matches!(
                command,
                ClusterCommand::Cancel | ClusterCommand::Quit | ClusterCommand::ForceQuit
            )
        {
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

    pub fn mode(&self) -> ClusterMode {
        self.mode
    }

    pub fn hosts(&self) -> &[HostConfig] {
        &self.hosts
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn selected_host(&self) -> Option<&HostConfig> {
        self.hosts.get(self.cursor)
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
        self.host_snapshots()
            .filter(|snapshot| {
                matches!(
                    snapshot.connection,
                    ConnectionState::Offline
                        | ConnectionState::Timeout
                        | ConnectionState::AuthFailed
                )
            })
            .count()
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
        !self.is_refreshing()
            && self
                .last_refresh_started
                .map(|started| started.elapsed() >= interval)
                .unwrap_or(true)
    }

    fn count_state(&self, state: ConnectionState) -> usize {
        self.host_snapshots()
            .filter(|snapshot| snapshot.connection == state)
            .count()
    }

    fn mark_timed_out_refreshes(&mut self) {
        let now = Instant::now();
        let timed_out_aliases = self
            .refresh_deadlines
            .iter()
            .filter(|(_, deadline)| now >= **deadline)
            .map(|(alias, _)| alias.clone())
            .collect::<Vec<_>>();

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
        self.cursor = self
            .cursor
            .saturating_add_signed(delta)
            .min(self.hosts.len().saturating_sub(1));
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
    let inventory = match path {
        Some(path) => ClusterInventory::from_path(path)?,
        None => ClusterInventory::load_default()?,
    };
    run_with_inventory(inventory)
}

pub fn run_with_inventory(inventory: ClusterInventory) -> Result<()> {
    let guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut app = ClusterApp::from_inventory(inventory);
    let (tx, rx) = mpsc::channel();
    start_refresh_all(&mut app, tx.clone());

    while !app.should_quit() {
        drain_snapshots(&mut app, &rx);
        app.mark_timed_out_refreshes();

        if app.refresh_due(DEFAULT_REFRESH_INTERVAL) {
            start_refresh_all(&mut app, tx.clone());
        }

        terminal.draw(|frame| crate::cluster_ui::draw(frame, &app))?;
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match app.handle_key(key) {
                Some(ClusterCommand::RefreshAll) => start_refresh_all(&mut app, tx.clone()),
                Some(ClusterCommand::RefreshSelected) => {
                    start_refresh_selected(&mut app, tx.clone())
                }
                Some(ClusterCommand::OpenSession) => {
                    terminal.show_cursor()?;
                    open_selected_session(&mut app, &guard)?;
                    terminal.clear()?;
                    drain_snapshots(&mut app, &rx);
                    start_refresh_selected(&mut app, tx.clone());
                }
                Some(ClusterCommand::OpenWorkbench) => {
                    terminal.show_cursor()?;
                    open_selected_workbench(&mut app, &guard)?;
                    terminal.clear()?;
                    drain_snapshots(&mut app, &rx);
                    start_refresh_selected(&mut app, tx.clone());
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn drain_snapshots(app: &mut ClusterApp, rx: &mpsc::Receiver<ProbeSnapshot>) {
    while let Ok(snapshot) = rx.try_recv() {
        app.apply_probe_snapshot(snapshot);
    }
}

pub fn collect_host_snapshot(host: &HostConfig) -> HostSnapshot {
    let started = Instant::now();
    let result = if host.kind == HostKind::Local {
        run_local_probe()
    } else {
        run_ssh_probe(host)
    };
    match result {
        Ok(output) => HostSnapshot::online(
            host.alias(),
            ProbeReport::parse(&output),
            started.elapsed().as_millis(),
        ),
        Err(err) => HostSnapshot::failed(host.alias(), err.to_string()),
    }
}

fn start_refresh_all(app: &mut ClusterApp, tx: mpsc::Sender<ProbeSnapshot>) {
    let hosts = app.hosts().to_vec();
    let aliases = hosts
        .iter()
        .map(|host| host.alias().to_string())
        .collect::<Vec<_>>();
    let started = app
        .begin_refresh(&aliases)
        .into_iter()
        .collect::<BTreeSet<_>>();
    for host in hosts {
        if !started.contains(host.alias()) {
            continue;
        }
        let Some(token) = app.refresh_token(host.alias()) else {
            continue;
        };
        let tx = tx.clone();
        thread::spawn(move || {
            let _ = tx.send(ProbeSnapshot {
                token,
                snapshot: collect_host_snapshot(&host),
            });
        });
    }
}

fn start_refresh_selected(app: &mut ClusterApp, tx: mpsc::Sender<ProbeSnapshot>) {
    let Some(host) = app.selected_host().cloned() else {
        return;
    };
    if app.begin_refresh(&[host.alias().to_string()]).is_empty() {
        return;
    }
    let Some(token) = app.refresh_token(host.alias()) else {
        return;
    };
    thread::spawn(move || {
        let _ = tx.send(ProbeSnapshot {
            token,
            snapshot: collect_host_snapshot(&host),
        });
    });
}

fn run_local_probe() -> Result<String> {
    let (program, mut args) = local_probe_shell();
    let mut command = ProcessCommand::new(program);
    command.args(args.drain(..));
    command.arg(PROBE_SCRIPT);
    run_command_with_timeout(command, PROBE_TIMEOUT)
}

fn run_ssh_probe(host: &HostConfig) -> Result<String> {
    let mut command = ProcessCommand::new("ssh");
    command.args(ssh_probe_args(host));
    run_command_with_timeout(command, PROBE_TIMEOUT)
}

pub fn ssh_probe_args(host: &HostConfig) -> Vec<String> {
    let mut args = vec![
        "-n".to_string(),
        "-o".to_string(),
        "BatchMode=yes".to_string(),
        "-o".to_string(),
        "ConnectTimeout=3".to_string(),
        "-o".to_string(),
        "ConnectionAttempts=1".to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=yes".to_string(),
        "-o".to_string(),
        "ServerAliveInterval=2".to_string(),
        "-o".to_string(),
        "ServerAliveCountMax=1".to_string(),
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
        };
    }

    SessionCommand {
        program: "ssh".to_string(),
        args: ssh_session_args(host),
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
            args.push(workdir.to_string());
        }
        return SessionCommand {
            program: program.to_string(),
            args,
        };
    }

    SessionCommand {
        program: "ssh".to_string(),
        args: ssh_workbench_args(host),
    }
}

pub fn ssh_workbench_args(host: &HostConfig) -> Vec<String> {
    let mut args = vec!["-t".to_string()];
    if let Some(proxy_jump) = host.proxy_jump() {
        args.push("-J".to_string());
        args.push(host.proxy_jump_target().unwrap_or(proxy_jump).to_string());
    }
    args.push(host.ssh_target().to_string());
    args.push(remote_workbench_command(host.workdir()));
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
    let Some(host) = app.selected_host().cloned() else {
        return Ok(());
    };
    let current_tersh = current_tersh_program();
    let command = host_workbench_command(&host, &current_tersh);

    app.log(format!("opening tersh: {}", host.alias()));
    let result = guard.suspend(|| run_session_command(&command))?;
    match result {
        Ok(status) => app.log(format!("tersh closed: {} ({status})", host.alias())),
        Err(err) => app.log(format!("tersh failed: {}: {err}", host.alias())),
    }
    Ok(())
}

fn run_session_command(command: &SessionCommand) -> Result<std::process::ExitStatus> {
    ProcessCommand::new(command.program())
        .args(command.args())
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

fn remote_workbench_command(workdir: Option<&str>) -> String {
    let mut script = format!(
        "if ! command -v tersh >/dev/null 2>&1; then printf '%s\\n' {} >&2; exit 127; fi",
        shell_quote(
            "tersh is not installed or not in PATH. Install: cargo install --git https://github.com/QiushanHuang/Tersh.git --tag v1.1.0 --bin tersh --force"
        )
    );
    if let Some(workdir) = workdir.map(str::trim).filter(|workdir| !workdir.is_empty()) {
        script.push_str(&format!(
            "; cd -- {} || {{ printf '%s\\n' {} >&2; exit 1; }}",
            shell_quote(workdir),
            shell_quote(&format!("tersh workdir not found: {workdir}"))
        ));
    }
    script.push_str("; exec tersh");
    remote_probe_command(&script)
}

fn remote_probe_command(script: &str) -> String {
    if cfg!(windows) {
        format!("cmd /C \"{}\"", script.replace('"', "\\\""))
    } else {
        format!("sh -lc {}", shell_quote(script))
    }
}

#[cfg(not(windows))]
fn local_probe_shell() -> (&'static str, Vec<String>) {
    ("sh", vec!["-lc".to_string()])
}

#[cfg(windows)]
fn local_probe_shell() -> (&'static str, Vec<String>) {
    ("cmd", vec!["/C".to_string()])
}

fn run_command_with_timeout(mut command: ProcessCommand, timeout: Duration) -> Result<String> {
    let temp_files = TempProbeFiles::new();
    let stdout = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .truncate(true)
        .open(&temp_files.stdout)
        .context("create temp stdout file")?;
    let stderr = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .truncate(true)
        .open(&temp_files.stderr)
        .context("create temp stderr file")?;

    configure_probe_command(&mut command);

    let mut child = command
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .context("start probe command")?;
    let started = Instant::now();
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() >= timeout {
            timed_out = true;
            terminate_probe_child(&mut child);
            break child.wait().context("wait probe command after timeout")?;
        }
        thread::sleep(Duration::from_millis(50));
    };

    let stdout = read_lossy_limited(&temp_files.stdout, "stdout")?;
    let stderr = read_lossy_limited(&temp_files.stderr, "stderr")?;

    if timed_out {
        let mut message = format!("probe timed out after {}s", timeout.as_secs());
        let details = stderr.trim();
        if !details.is_empty() {
            message = format!("{message}: {details}");
        }
        anyhow::bail!(message);
    }

    if status.success() {
        return Ok(stdout);
    }
    let stderr = stderr.trim().to_string();
    if stderr.is_empty() {
        anyhow::bail!("probe exited with {}", status);
    }
    anyhow::bail!("{stderr}");
}

fn read_lossy_limited(path: &Path, stream: &str) -> Result<String> {
    let metadata =
        fs::metadata(path).with_context(|| format!("failed to inspect {}", path.display()))?;
    if metadata.len() > MAX_PROBE_OUTPUT_BYTES {
        anyhow::bail!(
            "probe output too large: {stream} exceeded {} bytes",
            MAX_PROBE_OUTPUT_BYTES
        );
    }
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
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

struct TempProbeFiles {
    stdout: PathBuf,
    stderr: PathBuf,
}

impl TempProbeFiles {
    fn new() -> Self {
        Self {
            stdout: tempfile_path("tersh-probe-stdout"),
            stderr: tempfile_path("tersh-probe-stderr"),
        }
    }
}

impl Drop for TempProbeFiles {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.stdout);
        let _ = fs::remove_file(&self.stderr);
    }
}

fn tempfile_path(prefix: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|time| time.as_nanos())
        .unwrap_or_default();
    let pid = std::process::id();
    std::env::temp_dir().join(format!("{prefix}-{pid}-{now}.log"))
}

fn key_to_command(key: KeyEvent) -> Option<ClusterCommand> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => Some(ClusterCommand::ForceQuit),
            KeyCode::Char('g') | KeyCode::Char('G') => Some(ClusterCommand::Cancel),
            _ => None,
        };
    }
    match key.code {
        KeyCode::Esc => Some(ClusterCommand::Cancel),
        KeyCode::Char('j') | KeyCode::Down => Some(ClusterCommand::Down),
        KeyCode::Char('k') | KeyCode::Up => Some(ClusterCommand::Up),
        KeyCode::Home => Some(ClusterCommand::First),
        KeyCode::End | KeyCode::Char('G') => Some(ClusterCommand::Last),
        KeyCode::Char('r') => Some(ClusterCommand::RefreshAll),
        KeyCode::Enter => Some(ClusterCommand::RefreshSelected),
        KeyCode::Char('s') => Some(ClusterCommand::OpenSession),
        KeyCode::Char('t') => Some(ClusterCommand::OpenWorkbench),
        KeyCode::Char('l') => Some(ClusterCommand::OpenDetail),
        KeyCode::Char('?') => Some(ClusterCommand::OpenHelp),
        KeyCode::Char('q') => Some(ClusterCommand::Quit),
        KeyCode::Char('Q') => Some(ClusterCommand::ForceQuit),
        _ => None,
    }
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
        assert!(app.refresh_due(Duration::from_secs(60)));
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
