# Cluster Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every declared cluster refresh generation finish with bounded real process concurrency, truthful cancellation, and launch-attempt-scoped remote readiness using the Plan 1 source-bound single-process protocol.

**Architecture:** This file is a component-recipe catalog, not iteration orchestration or acceptance evidence. Split the current monolithic cluster loop at two ownership boundaries: `cluster_scheduler.rs` is a pure generation/queue reducer, and `cluster_probe.rs` owns real process groups, deadlines, bounded readers, termination, and reaping. `cluster.rs` remains the inventory, UI coordinator, and event-loop adapter; it composes the scheduler and probe supervisor and uses Plan 1 `remote_launch.rs` unchanged. Readiness is a stamped result of one explicit launch attempt, not a periodic fleet probe; only the same-candidate `impl-07` manifest produced by `2026-08-10-tersh-implementation-iteration-evidence.md` accepts the slice or iteration.

**Tech Stack:** Rust 1.88, `std::process`/threads/channels, Unix process groups via `libc`, Plan 1 remote-launch and build-identity contracts, `ratatui` TestBackend, Cargo integration tests with scripted child processes.

---

## Locked boundaries and existing anchors

- Baseline used for source anchors: `799cf08f1abd9b546133ed419bf6d4341714e292`.
- Current cluster state: `src/cluster.rs:336-493` (`ConnectionState`, `HostSnapshot`, `ProbeSnapshot`, `ActiveProbe`, `ClusterCommand`, `ClusterApp`).
- Current capped-but-incomplete refresh: `src/cluster.rs:577-672` (`begin_refresh`, `apply_probe_snapshot`) and `src/cluster.rs:999-1042` (`start_refresh_all`, `start_refresh_selected`).
- Current timeout/process ownership: `src/cluster.rs:1259-1369` (`run_command_with_timeout`, bounded readers, process group, immediate timeout kill).
- Current event loop and launcher: `src/cluster.rs:909-970`, `src/cluster.rs:1088-1247`.
- Current dashboard footer/help: `src/cluster_ui.rs:357-423`, `src/cluster_ui.rs:599-702`.
- Existing tests that expose the missing refill: `tests/cluster.rs:380-408`; launcher assertions begin at `tests/cluster.rs:182-216`.
- Design authority: launcher outcome protocol `docs/superpowers/specs/2026-08-10-tersh-trusted-core-design.md:473-539`; G3 `:979-1020`; UI `:1022-1044`; gate row `:1103`; planning boundary `:1223`; lifecycle containment `:1429-1441`; exact-test inventory `:1471-1474`; ADD-009/010 `:1488-1505`.

Required Plan 1 interfaces:

```rust
use crate::build_identity::BuildIdentity;
use crate::process_outcome::InterruptSignal;
use crate::remote_launch::{
    ChildOutcome, CompatibilityError, CompatibilityRegistry, LaunchIdentity,
    ProxyTerminalOutcome, RemoteLaunchRequest, RemoteLaunchRequestError,
    RemoteProxyCompletion,
    RemoteProxySession, RemoteProxySpec, RemoteProxyStartError,
};
use crate::terminal_session::{SuspendedTerminal, TerminalSession};

#[cfg(test)]
use crate::remote_launch::CompatibilityFixture;
```

The concrete Plan 1 handoff used here is:

```rust
impl BuildIdentity {
    pub fn embedded() -> Self;
    pub fn launch_identity(&self) -> Option<&LaunchIdentity>;
}

impl RemoteLaunchRequest {
    pub fn new(nonce: [u8; 16], workdir: &OsStr)
        -> Result<Self, RemoteLaunchRequestError>;
    pub fn nonce(&self) -> [u8; 16];
    pub fn remote_exec_command(&self) -> OsString;
}

impl CompatibilityRegistry {
    pub fn from_embedded_build_identity(identity: &BuildIdentity) -> Result<Self, CompatibilityError>;
    pub fn accepts(&self, identity: &LaunchIdentity) -> bool;

    #[cfg(test)]
    pub(crate) fn from_fixture(fixture: CompatibilityFixture) -> Result<Self, CompatibilityError>;
}

impl RemoteProxySpec {
    pub fn for_request(
        program: OsString,
        args: Vec<OsString>,
        request: &RemoteLaunchRequest,
    ) -> Self;
}

impl<'terminal> RemoteProxySession<'terminal> {
    pub fn spawn(
        dashboard: SuspendedTerminal<'terminal>,
        spec: RemoteProxySpec,
    ) -> Result<Self, RemoteProxyStartError>;
    pub fn run(self, registry: &CompatibilityRegistry) -> RemoteProxyCompletion;
}

impl RemoteProxyCompletion {
    pub fn ready_identity(&self) -> Option<&LaunchIdentity>;
    pub fn child_outcome(&self) -> &ChildOutcome;
    pub fn escaped_diagnostic(&self) -> Option<&str>;
    pub fn terminal_outcome(&self) -> &ProxyTerminalOutcome;
    pub fn local_interrupt(&self) -> Option<InterruptSignal>;
    pub fn into_parts(
        self,
    ) -> (
        Option<LaunchIdentity>,
        ChildOutcome,
        Option<String>,
        ProxyTerminalOutcome,
        Option<InterruptSignal>,
    );
}

impl RemoteProxyStartError {
    pub fn child_outcome(&self) -> Option<&ChildOutcome>;
    pub fn escaped_diagnostic(&self) -> &str;
    pub fn terminal_outcome(&self) -> &ProxyTerminalOutcome;
    pub fn local_interrupt(&self) -> Option<InterruptSignal>;
    pub fn into_parts(
        self,
    ) -> (Option<ChildOutcome>, String, ProxyTerminalOutcome, Option<InterruptSignal>);
}
```

`CompatibilityRegistry` fields remain private. Production code has no registry constructor from JSON bytes, environment, caller-supplied pairs, or a fixture: it may call only `from_embedded_build_identity`, which verifies the embedded accepted-manifest index against the current re-derived `BuildIdentity`. `from_fixture` and `CompatibilityFixture` exist only under `cfg(test)`. The outer cluster loop owns the only `TerminalSession`, calls `suspend` once, and transfers that private `SuspendedTerminal` into `RemoteProxySession::spawn`. The Plan 1 session then solely owns the one child process group and PTY, exclusive access to the terminal's production signal/resize broker, stdin and PTY readers, distinct decision/cleanup wakes, post-READY proxying, bounded diagnostic retention, drain, TERM/KILL, reap, reader joins, and exactly-once resume/restore including `Drop`. Its `run` is total after successful spawn. Plan 5 consumes `RemoteProxyCompletion` or `RemoteProxyStartError` through the accessors/`into_parts` above and must not redefine, wrap, parse around, or bypass this lifecycle.

Before Task 1, run `cargo test --locked --test remote_launch --test process_outcome --test cluster`. All must pass. Plan 5 must not change READY framing, build identity, compatibility keys, local exit mapping, or CLI validation; any failure there returns to Plan 1. Plans 2-4 are not Cluster code dependencies, and Workbench Trusted Core release is not blocked on this plan. Tasks 1-6 may produce component commits only: they do not close `impl-07`, claim iteration 7 complete, or create its evidence manifest. After the final component candidate is frozen, only `docs/superpowers/plans/2026-08-10-tersh-implementation-iteration-evidence.md` may run the exact-candidate gates, five-role closure, and create `docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-07.json`.

Locked external evidence IDs are identical across Plan 1, Plan 4, the implementation-evidence plan, and hardening:

- `impl-07` CI: `quality-stable`, `msrv-1-88`, `policy`, `native-exdev-linux`, `native-exdev-macos`. The two native jobs must retain Plan 4's exact artifacts `native-exdev-{linux,macos}-{candidate}-run-{run_id}-attempt-{run_attempt}`, root schema `tersh-native-exdev-evidence-v1`, nonempty manifest-declared payload, unique pinned `upload-evidence`, and immediately following runtime producer join. Task 8 rejects a job-only success without either exact artifact.
- `impl-07` release: `tier1-macos-arm64`, `tier1-linux-x86_64`, `tier2-macos-x86_64-source`, `tier2-linux-arm64-source`, `install-msrv-1-88`, `install-current-stable`, `assemble-manifest`, `verify-release-candidate`. The two Tier-1 jobs retain Plan 4 steps `native-exdev-macos` and `native-exdev-linux`, respectively.
- Later hardening extends the exact CI set with, but this component plan does not prematurely require or create, `terminal-multiplexer-linux` and `terminal-multiplexer-macos`; it retains the five `impl-07` CI IDs above. Hardening release verification retains the same eight release IDs above, including `assemble-manifest` and `verify-release-candidate`; a six-job invocation is incomplete. No plan may introduce aliases such as `native-exdev`, `linux`, `macos`, `tier1-linux`, or `tier1-macos` as evidence IDs.

### Task 1: Pure bounded refresh-generation scheduler

**Files:**
- Create: `src/cluster_scheduler.rs`
- Modify: `src/lib.rs`
- Create: `tests/cluster_scheduler.rs`

- [ ] **Step 1: Write failing scheduler state tests**

Add exact tests:

```rust
#[test]
fn sweep_one_host_reaches_one_terminal_result() {
    let mut scheduler = RefreshScheduler::default();
    let start = scheduler.request_sweep(vec!["h00".into()]);
    let (alias, token) = only_start(start);
    let finish = scheduler.finish(&alias, token, ProbeTerminal::Completed);
    assert!(matches!(finish.as_slice(), [SchedulerAction::GenerationFinished { .. }]));
    assert!(scheduler.is_terminal());
}

#[test]
fn sweep_seventeen_hosts_refills_one_slot_after_reap() {
    let mut scheduler = RefreshScheduler::default();
    let starts = scheduler.request_sweep(aliases(17));
    assert_eq!(starts.len(), 16);
    let (alias, token) = first_start(&starts);
    let refill = scheduler.finish(alias, token, ProbeTerminal::Completed);
    assert_eq!(refill.iter().filter(|a| matches!(a, SchedulerAction::Start { .. })).count(), 1);
    assert_eq!(scheduler.snapshot().active.len(), 16);
}

#[test]
fn refresh_during_sweep_coalesces_to_one_next_generation() {
    let mut scheduler = RefreshScheduler::default();
    scheduler.request_sweep(aliases(17));
    scheduler.request_sweep(aliases(40));
    scheduler.request_sweep(aliases(40));
    assert_eq!(scheduler.snapshot().pending_refresh.unwrap().len(), 40);
}
```

In the same test file define `aliases`, `only_start`, and `first_start`, then add `sweep_sixteen_hosts_starts_all_without_queue`, `sweep_forty_hosts_never_exposes_more_than_sixteen_start_actions`, `stale_token_or_generation_cannot_finish_current_host`, and `quit_cancels_queued_and_requests_stop_for_active` with assertions over immutable `snapshot()` values and returned actions. Add `quit_preserves_completed_cancels_queued_and_keeps_active_stopping_until_reaped` and `quit_is_monotonic_and_rejects_refresh_after_request`. The monotonic test calls both all-host and selected-host `request_sweep` after `request_quit`, including after the last active token finishes, and requires no action, no `pending_refresh`, no new generation, and no change to the terminal snapshot.

Use deterministic aliases `h00` through `h39`; assert one terminal state per captured alias and monotonic generation IDs.

- [ ] **Step 2: Run scheduler tests and confirm RED**

Run every focused case through the shared exact helper:

```bash
python3 scripts/run_exact_test.py --test cluster_scheduler --name sweep_one_host_reaches_one_terminal_result
python3 scripts/run_exact_test.py --test cluster_scheduler --name sweep_seventeen_hosts_refills_one_slot_after_reap
python3 scripts/run_exact_test.py --test cluster_scheduler --name refresh_during_sweep_coalesces_to_one_next_generation
python3 scripts/run_exact_test.py --test cluster_scheduler --name sweep_sixteen_hosts_starts_all_without_queue
python3 scripts/run_exact_test.py --test cluster_scheduler --name sweep_forty_hosts_never_exposes_more_than_sixteen_start_actions
python3 scripts/run_exact_test.py --test cluster_scheduler --name stale_token_or_generation_cannot_finish_current_host
python3 scripts/run_exact_test.py --test cluster_scheduler --name quit_cancels_queued_and_requests_stop_for_active
python3 scripts/run_exact_test.py --test cluster_scheduler --name quit_preserves_completed_cancels_queued_and_keeps_active_stopping_until_reaped
python3 scripts/run_exact_test.py --test cluster_scheduler --name quit_is_monotonic_and_rejects_refresh_after_request
```

Expected: each command discovers exactly one test and FAILS because `tersh::cluster_scheduler::{RefreshScheduler, SchedulerAction}` does not exist.

- [ ] **Step 3: Implement the pure reducer**

Define:

```rust
pub const MAX_ACTIVE_PROBES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RefreshGeneration(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProbeToken { pub generation: RefreshGeneration, pub sequence: u64 }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeTerminal { Completed, TimedOut, Failed, Cancelled }

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchedulerAction {
    Start { alias: String, token: ProbeToken },
    Stop { alias: String, token: ProbeToken },
    GenerationFinished { generation: RefreshGeneration },
}

pub struct RefreshScheduler {
    generation: Option<RefreshGeneration>,
    pending: VecDeque<String>,
    active: HashMap<String, ProbeToken>,
    terminal: BTreeMap<String, ProbeTerminal>,
    pending_refresh: Option<Vec<String>>,
    quitting: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefreshSnapshot {
    pub generation: Option<RefreshGeneration>,
    pub pending: Vec<String>,
    pub active: BTreeMap<String, ProbeToken>,
    pub terminal: BTreeMap<String, ProbeTerminal>,
    pub pending_refresh: Option<Vec<String>>,
    pub quitting: bool,
}

impl Default for RefreshScheduler {
    fn default() -> Self;
}

impl RefreshScheduler {
    pub fn request_sweep(&mut self, aliases: Vec<String>) -> Vec<SchedulerAction>;
    pub fn finish(&mut self, alias: &str, token: ProbeToken, result: ProbeTerminal) -> Vec<SchedulerAction>;
    pub fn request_quit(&mut self) -> Vec<SchedulerAction>;
    pub fn is_terminal(&self) -> bool;
    pub fn snapshot(&self) -> RefreshSnapshot;
}
```

`request_sweep` sorts/deduplicates and freezes the alias set. Its first branch checks `quitting`; once true it returns an empty action vector without changing generation, pending, active, terminal, or `pending_refresh`, even after the current generation becomes terminal. Otherwise, if the current generation still has queued or active work, replace `pending_refresh` with the latest sorted/deduplicated alias snapshot captured at keypress time; never enqueue more than that one follow-up sweep. If no generation has work, begin the request as the next monotonically numbered generation, including emitting `GenerationFinished` immediately for an empty captured set. `finish` ignores stale token/generation, marks exactly one terminal result, and emits enough `Start` actions to refill free slots. After all captured aliases are terminal, emit `GenerationFinished`; if `pending_refresh` is present and `quitting` is false, consume it and begin exactly one next generation.
All reducer fields remain private; tests and UI read immutable snapshots and can never mutate pending/active/terminal state around the reducer. Quit is monotonic: it sets `quitting=true` once, clears `pending_refresh` permanently, preserves already completed host results, moves the current generation's queued hosts directly to `Cancelled`, leaves real active hosts in `active`, and emits one `Stop` per active token. Repeated quit is idempotent. An active host becomes terminal `Cancelled` only when the supervisor reports that exact token after direct-child reap and reader join; that `finish` releases the slot but starts no refill while quitting. No all-host or selected-host request can reopen a quitting scheduler.

- [ ] **Step 4: Run scheduler tests and confirm GREEN**

Run: `cargo test --locked --test cluster_scheduler -- --nocapture`

Expected: PASS; all nine pure scheduler tests pass without spawning a process, and quit is an irreversible reducer state.

- [ ] **Step 5: Commit scheduler state**

```bash
git add src/cluster_scheduler.rs src/lib.rs tests/cluster_scheduler.rs
git commit -m "feat: add bounded cluster refresh scheduler"
```

### Task 2: Real probe process-group ownership and bounded shutdown

**Files:**
- Create: `src/cluster_probe.rs`
- Create: `tests/fixtures/probe_tree.sh`
- Modify: `src/lib.rs`
- Modify: `tests/cluster_scheduler.rs`

- [ ] **Step 1: Write failing process lifecycle tests**

Add exact tests `probe_supervisor_real_process_count_never_exceeds_sixteen`, `probe_spawn_failure_is_terminal_and_refills_slot`, `probe_post_spawn_reader_setup_failure_remains_owned_until_reaped`, `probe_timeout_sends_term_waits_500ms_then_kills_group`, `probe_quit_terminates_queued_and_active_groups`, `probe_shutdown_poll_keeps_rendering_during_term_grace`, `probe_shutdown_aggregates_errors_only_after_every_record_finishes`, `probe_slot_releases_only_after_direct_child_reap`, `probe_readers_are_joined_before_terminal_event`, `probe_each_reader_has_an_independent_nonblocking_cleanup_wake`, `probe_diagnostics_are_bounded_and_escaped`, `probe_over_cap_output_is_drained_to_eof_then_reaped_and_refilled`, `probe_direct_child_exit_with_grandchild_pipe_enters_bounded_drain_then_kills_group`, `probe_descendant_pipe_and_signal_error_still_wake_and_join_readers_boundedly`, `probe_reader_panic_fails_only_its_record_and_other_probes_finish`, `probe_poll_error_never_abandons_other_active_records`, and `probe_supervisor_drop_is_a_bounded_fail_safe`. Create executable `tests/fixtures/probe_tree.sh` with a shebang and invoke it directly so it is the process-group leader. It must spawn a child and grandchild, record TERM receipt, optionally let the direct child exit while a grandchild retains a pipe, optionally move that pipe holder into a new session so group signaling alone cannot produce EOF, optionally ignore TERM, write controlled stdout/stderr beyond the capture cap, and wait on a separate test-owned release channel so the harness can terminate and observe disappearance of even the deliberately escaped fixture process after proving reader cleanup. Deterministic seams fail stdout-reader creation or stderr-reader creation only after the direct child has spawned, and fail group signaling during bounded cleanup; no seam is public in production.

Also add exact tests `probe_startup_failure_then_quit_preserves_failed_intent`, `probe_timeout_then_quit_preserves_timed_out_intent`, and `probe_signal_error_during_shutdown_is_bounded_and_fails_record` for monotonic terminal intent and bounded error retention.

Keep public process-tree behavior in `tests/cluster_scheduler.rs`. Put
`probe_post_spawn_reader_setup_failure_remains_owned_until_reaped`,
`probe_descendant_pipe_and_signal_error_still_wake_and_join_readers_boundedly`,
`probe_reader_panic_fails_only_its_record_and_other_probes_finish`,
`probe_poll_error_never_abandons_other_active_records`, and the three monotonic
intent/error-cap tests in `cluster_probe::tests` under `src/cluster_probe.rs`.
Those seven gates inspect private owning records or drive the private
`#[cfg(test)]` deterministic reader/signal/poll seams, so they run with `--lib`;
the seams remain absent from normal library, integration dependency, and release
builds.

- [ ] **Step 2: Run lifecycle tests and confirm RED**

Run each target exactly:

```bash
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_supervisor_real_process_count_never_exceeds_sixteen --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_spawn_failure_is_terminal_and_refills_slot --serial
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::probe_post_spawn_reader_setup_failure_remains_owned_until_reaped --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_timeout_sends_term_waits_500ms_then_kills_group --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_quit_terminates_queued_and_active_groups --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_shutdown_poll_keeps_rendering_during_term_grace --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_shutdown_aggregates_errors_only_after_every_record_finishes --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_slot_releases_only_after_direct_child_reap --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_readers_are_joined_before_terminal_event --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_each_reader_has_an_independent_nonblocking_cleanup_wake --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_diagnostics_are_bounded_and_escaped --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_over_cap_output_is_drained_to_eof_then_reaped_and_refilled --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_direct_child_exit_with_grandchild_pipe_enters_bounded_drain_then_kills_group --serial
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::probe_descendant_pipe_and_signal_error_still_wake_and_join_readers_boundedly --serial
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::probe_reader_panic_fails_only_its_record_and_other_probes_finish --serial
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::probe_poll_error_never_abandons_other_active_records --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_supervisor_drop_is_a_bounded_fail_safe --serial
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::probe_startup_failure_then_quit_preserves_failed_intent --serial
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::probe_timeout_then_quit_preserves_timed_out_intent --serial
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::probe_signal_error_during_shutdown_is_bounded_and_fails_record --serial
```

Expected: FAIL because `ProbeSupervisor` does not exist and current timeout sends immediate KILL without the required wait/join sequence.

- [ ] **Step 3: Implement one owner for child, group, readers, and deadline**

Define:

```rust
pub struct ProbeSpec { pub alias: String, pub program: OsString, pub args: Vec<OsString> }
pub struct ProbeCompletion {
    pub alias: String,
    pub token: ProbeToken,
    pub terminal: ProbeTerminal,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub suppressed_error_count: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeErrorStage { Capacity, Spawn, ReaderSetup, Poll, Wait, Term, Kill, ReaderJoin, Reap }
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeRecordError {
    pub alias: String,
    pub token: ProbeToken,
    pub stage: ProbeErrorStage,
    pub escaped_error: String,
}

pub const MAX_PROBE_ERRORS_PER_RECORD: usize = 16;

struct ActiveProbeRecord {
    alias: String,
    child: Child,
    process_group: i32,
    deadline: Instant,
    phase: ProbePhase,
    terminal_intent: Option<ProbeTerminal>,
    contained_errors: Vec<ProbeRecordError>,
    suppressed_error_count: u64,
    stdout_reader: Option<ProbeReaderTask>,
    stderr_reader: Option<ProbeReaderTask>,
}
struct ProbeReaderTask {
    cleanup_wake_write: OwnedFd,
    reader: Option<JoinHandle<ProbeReaderResult>>,
}
struct ProbeReaderResult {
    retained: Vec<u8>,
    total_bytes: u64,
    read_error: Option<ProbeReaderError>,
}
struct ProbeReaderError {
    escaped_error: String,
}
struct ProbeStartupGuard {
    child: Option<Child>,
    process_group: i32,
    stdout_reader: Option<ProbeReaderTask>,
    stderr_reader: Option<ProbeReaderTask>,
}
enum ProbePhase {
    Running,
    Draining { deadline: Instant },
    TermSent { kill_at: Instant },
    KillSent,
}
pub struct ProbeSupervisor {
    active: HashMap<ProbeToken, ActiveProbeRecord>,
    ready_completions: VecDeque<ProbeCompletion>,
    contained_errors: VecDeque<ProbeRecordError>,
    shutdown_phase: SupervisorShutdownPhase,
    shutdown_terminal_records: usize,
    shutdown_errors: Vec<ProbeRecordError>,
}
pub struct ProbeShutdown {
    pub terminal_records: usize,
    pub contained_errors: Vec<ProbeRecordError>,
    pub suppressed_error_count: u64,
}
pub struct ProbePollBatch {
    pub completions: Vec<ProbeCompletion>,
    pub contained_errors: Vec<ProbeRecordError>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SupervisorShutdownPhase { Running, TermGrace, KillIssued, Reaping, Complete }

impl ProbeSupervisor {
    pub fn new() -> Self;
    pub fn start(&mut self, token: ProbeToken, spec: ProbeSpec, now: Instant) -> Result<(), ProbeRecordError>;
    pub fn poll(&mut self, now: Instant) -> ProbePollBatch;
    pub fn terminate(&mut self, token: ProbeToken, now: Instant) -> bool;
    pub fn begin_shutdown(&mut self, now: Instant);
    pub fn shutdown_phase(&self) -> SupervisorShutdownPhase;
    pub fn take_shutdown_summary(&mut self) -> Option<ProbeShutdown>;
    pub fn real_active_count(&self) -> usize;
}
```

`ProbeSupervisor::new` always uses `MAX_ACTIVE_PROBES`; there is no constructor or setter that accepts a larger value, and `start` rejects a seventeenth live record. A returned `Err` is strictly pre-spawn and proves no child, group, pipe, or reader was created. Immediately after `Command::spawn` succeeds, move the child and group into `ProbeStartupGuard`; no `?` or naked error may cross that point. Create independent close-on-exec nonblocking cleanup-wake pipes for stdout and stderr. Each reader thread owns exactly one data-read fd and one cleanup-wake read fd and polls both; the active record owns the matching wake writer and join handle. If either reader setup fails after spawn, convert the guard into an owned active record with `terminal_intent = Some(Failed)`, the exact startup error already appended to `contained_errors`, and whichever reader exists; return `Ok(())`, then let normal nonblocking TERM/KILL/reap/wake/join produce the one terminal completion without overwriting that intent/error. `ProbeStartupGuard::Drop` is only a panic fail-safe before insertion and performs the same bounded cleanup; it never detaches its child.

Spawn each probe in its own session/process group. Keep the direct `Child`, group ID, deadline at six seconds, bounded stdout/stderr reader tasks, and termination phase in one active record. Readers retain bytes only to the diagnostic cap, then continue drain-and-discard until EOF so a chatty child cannot block on a full pipe. A direct child exit is not terminal while a descendant still owns a pipe: set an implementation-private 100-ms drain deadline, then TERM the group, wait 500 ms, KILL if needed, reap the direct child, write and close both independent cleanup wakes, and join both readers. If group TERM/KILL reports an error, retain that error and use the owned direct `Child` kill fallback at the KILL deadline so group-signal failure cannot make direct-child reap unbounded. The wake path is mandatory even after KILL or any signal error, so a descendant that escaped the group or retained a pipe cannot block either reader join forever. Wait/poll/signal/read/join errors stay on the owned record until direct-child reap and reader join are complete, then produce that record's `Failed` completion; an error never releases the slot early. `poll` appends the contained error and completion and continues every other record, never returning early because one wait/read/kill failed.

`terminate(token, now)` is idempotent and nonblocking: for that owned record it sets `terminal_intent = Some(Cancelled)` only when no earlier intent exists, sends TERM best-effort, installs `TermSent { kill_at: now + 500 ms }`, and returns. `begin_shutdown(now)` applies that same monotonic transition to every live record and returns to the event loop. Independently, when `poll(now)` first observes a six-second probe deadline, it records `TimedOut` unless a prior `Failed`/terminal intent already exists, then begins the same TERM/500-ms/KILL sequence; the intent remains on the record rather than being copied into and overwritten with `ProbePhase`. A wait/signal/read/join/reap error appends to `contained_errors`, makes the emitted result `Failed`, and never erases the original reason cleanup began. During shutdown, each per-poll error is returned for immediate display and cloned into a private shutdown accumulator, so `take_shutdown_summary` can return the complete aggregate without withholding earlier batches. Repeated `poll(now)` calls continue input handling and rendering, advance expired records to KILL, reap direct children, wake/close and join readers, and emit each record exactly once. `take_shutdown_summary` returns `None` until all records are terminal/reaped/joined, then returns one aggregate summary and only then permits terminal restoration. A bounded `Drop` fail-safe may synchronously perform the same TERM/KILL/reap/wake/join sequence only when explicit shutdown was missed; normal quit never calls a blocking join. A slot is released only after its direct child is reaped and both reader threads have observed EOF or their own cleanup wake and joined. Preserve bounded escaped diagnostics.

Post-spawn setup failure inserts an `ActiveProbeRecord` with `terminal_intent = Some(Failed)` and the exact startup error in its bounded `contained_errors`; it does not encode the only copy of either fact in `ProbePhase`. The first non-`None` terminal intent is monotonic and can never be weakened: deadline records `TimedOut`, explicit stop/quit records `Cancelled` only if no prior intent exists, and later shutdown cannot replace `Failed` or `TimedOut`. `record_error` deduplicates an identical `(stage, escaped_error)` pair, retains at most `MAX_PROBE_ERRORS_PER_RECORD = 16` distinct bounded errors, and increments a saturating `suppressed_error_count` after the cap; it never appends once per unchanged poll tick. Completion and shutdown summaries expose the retained errors plus that count. The emitted terminal is `Failed` whenever either is nonzero, while the immutable original intent remains available for diagnosis; reap/join is still mandatory before emission. Exact tests `probe_startup_failure_then_quit_preserves_failed_intent`, `probe_timeout_then_quit_preserves_timed_out_intent`, and `probe_signal_error_during_shutdown_is_bounded_and_fails_record` assert precedence, the 16-entry cap/deduplication, suppressed count, and one terminal emission.

- [ ] **Step 4: Run lifecycle tests serially and confirm GREEN**

Run:

```bash
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_supervisor_real_process_count_never_exceeds_sixteen --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_spawn_failure_is_terminal_and_refills_slot --serial
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::probe_post_spawn_reader_setup_failure_remains_owned_until_reaped --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_timeout_sends_term_waits_500ms_then_kills_group --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_quit_terminates_queued_and_active_groups --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_shutdown_poll_keeps_rendering_during_term_grace --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_shutdown_aggregates_errors_only_after_every_record_finishes --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_slot_releases_only_after_direct_child_reap --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_readers_are_joined_before_terminal_event --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_each_reader_has_an_independent_nonblocking_cleanup_wake --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_diagnostics_are_bounded_and_escaped --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_over_cap_output_is_drained_to_eof_then_reaped_and_refilled --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_direct_child_exit_with_grandchild_pipe_enters_bounded_drain_then_kills_group --serial
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::probe_descendant_pipe_and_signal_error_still_wake_and_join_readers_boundedly --serial
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::probe_reader_panic_fails_only_its_record_and_other_probes_finish --serial
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::probe_poll_error_never_abandons_other_active_records --serial
python3 scripts/run_exact_test.py --test cluster_scheduler --name probe_supervisor_drop_is_a_bounded_fail_safe --serial
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::probe_startup_failure_then_quit_preserves_failed_intent --serial
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::probe_timeout_then_quit_preserves_timed_out_intent --serial
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::probe_signal_error_during_shutdown_is_bounded_and_fails_record --serial
cargo test --locked --test cluster_scheduler -- --nocapture --test-threads=1
```

Expected: PASS; observed process-group concurrency never exceeds 16 and no child or reader thread remains after completion.

- [ ] **Step 5: Commit process ownership**

```bash
git add src/cluster_probe.rs src/lib.rs tests/cluster_scheduler.rs tests/fixtures/probe_tree.sh
git commit -m "feat: make cluster probe shutdown bounded"
```

### Task 3: Integrate full sweeps into ClusterApp and the event loop

**Files:**
- Modify: `src/cluster.rs:336-672`
- Modify: `src/cluster.rs:800-822`
- Modify: `src/cluster.rs:909-1042`
- Modify: `src/cluster.rs:1259-1369`
- Modify: `tests/cluster.rs:380-408`
- Modify: `tests/cluster_scheduler.rs`

- [ ] **Step 1: Replace the manual-second-refresh tests with failing full-sweep tests**

Replace `begin_refresh_rotates_after_capped_batch_completes` with exact tests `refresh_all_seventeen_refills_without_second_keypress`, `refresh_all_forty_finishes_every_captured_alias_once`, `refresh_spawn_failure_marks_failed_and_refills`, `refresh_post_spawn_setup_failure_remains_active_until_reaped_then_refills`, `refresh_dispatch_error_is_contained_and_later_records_start`, `refresh_forty_spawn_failures_yield_after_sixteen_attempts_per_tick`, `refresh_drain_error_is_contained_and_other_records_finish`, `manual_refresh_coalesces_one_followup_generation`, `late_previous_generation_result_is_ignored`, `refresh_input_while_stopping_is_ignored_without_followup`, `quit_starts_no_new_probe_and_marks_every_alias_terminal_cancelled`, and `normal_quit_polls_and_renders_through_term_kill_reap_join`.

- [ ] **Step 2: Run cluster integration tests and confirm RED**

Run each target exactly:

```bash
python3 scripts/run_exact_test.py --test cluster --name refresh_all_seventeen_refills_without_second_keypress
python3 scripts/run_exact_test.py --test cluster --name refresh_all_forty_finishes_every_captured_alias_once
python3 scripts/run_exact_test.py --test cluster --name refresh_spawn_failure_marks_failed_and_refills
python3 scripts/run_exact_test.py --test cluster --name refresh_post_spawn_setup_failure_remains_active_until_reaped_then_refills --serial
python3 scripts/run_exact_test.py --test cluster --name refresh_dispatch_error_is_contained_and_later_records_start
python3 scripts/run_exact_test.py --test cluster --name refresh_forty_spawn_failures_yield_after_sixteen_attempts_per_tick
python3 scripts/run_exact_test.py --test cluster --name refresh_drain_error_is_contained_and_other_records_finish
python3 scripts/run_exact_test.py --test cluster --name manual_refresh_coalesces_one_followup_generation
python3 scripts/run_exact_test.py --test cluster --name late_previous_generation_result_is_ignored
python3 scripts/run_exact_test.py --test cluster --name refresh_input_while_stopping_is_ignored_without_followup
python3 scripts/run_exact_test.py --test cluster --name quit_starts_no_new_probe_and_marks_every_alias_terminal_cancelled
python3 scripts/run_exact_test.py --test cluster --name normal_quit_polls_and_renders_through_term_kill_reap_join --serial
```

Expected: FAIL because `start_refresh_all` starts only the first available batch and does not refill from a captured queue.

- [ ] **Step 3: Make ClusterApp coordinate scheduler actions**

Replace `ClusterApp`'s refresh cursor/active maps with `RefreshScheduler` plus UI snapshots keyed by alias. Replace free functions `start_refresh_all`/`start_refresh_selected` with contained batch results:

```rust
pub struct DispatchRecordOutcome {
    pub alias: String,
    pub token: ProbeToken,
    pub result: Result<(), ProbeRecordError>,
}
pub struct DispatchBatch {
    pub records: Vec<DispatchRecordOutcome>,
    pub deferred_actions: VecDeque<SchedulerAction>,
}
pub struct DrainBatch {
    pub completions: Vec<ProbeCompletion>,
    pub contained_errors: Vec<ProbeRecordError>,
    pub generated_actions: Vec<SchedulerAction>,
}
pub enum ClusterQuitState {
    Running,
    Stopping { requested_at: Instant },
    Complete { shutdown: ProbeShutdown },
}

fn dispatch_scheduler_actions(
    app: &mut ClusterApp,
    supervisor: &mut ProbeSupervisor,
    actions: Vec<SchedulerAction>,
) -> DispatchBatch;

fn drain_probe_completions(
    app: &mut ClusterApp,
    supervisor: &mut ProbeSupervisor,
    now: Instant,
) -> DrainBatch;
```

The event loop snapshots configured aliases when `r` begins a sweep, dispatches at most 16 `Start` actions, and calls `RefreshScheduler::finish` only after `ProbeSupervisor` has reaped/joined a completion. `dispatch_scheduler_actions` processes at most `MAX_ACTIVE_PROBES` actions from a local worklist per event-loop tick. A `ProbeSupervisor::start` `Err` is known pre-spawn/no-effect: record that alias as `Failed` with bounded escaped diagnostics, call `finish`, append the returned refill actions, and continue until the per-tick bound. Any failure after child spawn returns `Ok(())`, remains an owned supervisor record, and reaches `finish` only through the later reaped/joined `ProbeCompletion`. The dispatcher returns remaining actions in `deferred_actions`, which `ClusterApp` owns and dispatches before new refresh actions on the next tick. It never uses `?`, returns `Result<()>`, abandons an attempted record, leaves a pre-spawn failure active, treats a post-spawn failure as absent, or loops over an unbounded all-failure fleet in one tick. `drain_probe_completions` likewise consumes the whole returned batch, calls `finish` exactly once per `ProbeCompletion`, attaches `contained_errors` for reporting without calling `finish` a second time, appends refill actions to the same bounded deferred queue, and returns an aggregate only after that received batch is exhausted. Selected-host refresh remains one-host scope and uses the same scheduler.

Normal quit is an event-loop state machine. The first quit event discards every deferred `Start` without spawning it, calls `request_quit`, records queued aliases `Cancelled`, and dispatches every `Stop`. A `Stop` token that has no `ProbeSupervisor` record because its `Start` was deferred is immediately passed to `scheduler.finish(..., Cancelled)`; a token with a real record—including a post-spawn record whose immutable intent is already `Failed`—remains active until reap/join. Then the app calls `supervisor.begin_shutdown(now)`, sets `ClusterQuitState::Stopping`, and returns immediately. Subsequent ticks poll only shutdown/signal/resize events needed for containment and render `stopping probes`; all-host and selected-host refresh commands are ignored, and the scheduler independently rejects them because quit is monotonic. `ProbeSupervisor::poll` sends KILL only after each 500-ms TERM grace, then reaps and wakes/joins readers. Each real completion makes its active scheduler record terminal exactly once. Only when `take_shutdown_summary` returns `Some` and the scheduler has no queued/active aliases may the app set `Complete`, aggregate contained errors, and restore the terminal. No normal quit path calls `terminate_all_and_join`, blocks for the grace period, or propagates a per-record `Result` out of the loop. Already completed hosts remain completed; queued and cleanly contained active hosts become `Cancelled`, a containment failure becomes `Failed`, and late tokens remain ignored.

- [ ] **Step 4: Run cluster/scheduler suites**

Run: `cargo test --locked --test cluster --test cluster_scheduler -- --nocapture --test-threads=1`

Expected: PASS; 17/40-host sweeps self-complete, refresh coalesces, quit reaps all processes, and inventory/config tests remain green.

- [ ] **Step 5: Commit integrated generations**

```bash
git add src/cluster.rs tests/cluster.rs tests/cluster_scheduler.rs
git commit -m "feat: complete cluster refresh generations"
```

### Task 4: Launch-attempt readiness using the Plan 1 protocol

**Files:**
- Create: `src/cluster_launch.rs`
- Modify: `src/lib.rs`
- Modify: `src/cluster.rs:1088-1247`
- Modify: `src/cluster_ui.rs:357-423`
- Modify: `tests/remote_launch.rs`
- Modify: `tests/cluster.rs:182-216`
- Modify: `tests/fixtures/fake_remote_launcher.sh`

- [ ] **Step 1: Write failing readiness-lifetime tests**

Add exact tests `readiness_is_unknown_before_launch_attempt`, `unavailable_embedded_registry_keeps_dashboard_usable_until_launch`, `launch_attempt_allocates_id_before_request_suspend_or_spawn_failure`, `launch_attempt_uses_one_remote_process_and_one_lookup`, `valid_ready_is_stamped_with_attempt_id_and_time`, `readiness_record_does_not_become_periodic_host_state`, `pre_ready_spawn_failure_still_records_attempt_id_time_and_optional_child_outcome`, `failed_frame_preserves_launch_failure_and_bounded_diagnostic`, `path_replacement_after_ready_cannot_change_classified_process`, `cluster_launcher_registry_fields_are_private_and_production_uses_embedded_manifest`, `cluster_launcher_fixture_registry_is_cfg_test_only`, `launch_attempt_suspends_and_transfers_guard_once`, `launch_total_paths_never_short_circuit_before_record`, `proxy_failure_records_attempt_only_after_child_reap_and_stream_join`, `cluster_proxy_post_ready_descendant_pipe_is_killed_reaped_and_joined_before_resume`, `cluster_proxy_user_interrupt_terminates_group_and_restores_once`, `cluster_proxy_reader_panic_is_contained_reaped_joined_and_reported`, and `cluster_proxy_os_resize_uses_suspended_terminal_signal_broker`. Add the crate-unit exact test `cluster_launch::tests::fixture_registry_is_available_only_to_unit_tests`; it positively exercises the `cfg(test)` constructor without widening it for an integration test. Retain all Plan 1 frame/nonce/source-pair/exit-code/proxy-lifecycle tests unchanged.

Extend `tests/fixtures/fake_remote_launcher.sh` with explicit modes `post-ready-grandchild-pipe`, `post-ready-wait-for-interrupt`, and `post-ready-reader-panic-marker`. Each mode emits one valid READY frame from the same process, records process-group TERM/KILL and direct-child reap facts, and gives the test deterministic pipe/reader synchronization; it never performs a second `tersh` lookup or exec.

Every test that needs an accepted compatibility fixture or a successful READY/proxy lifecycle lives in `src/cluster_launch.rs`'s `#[cfg(test)] mod tests` and is invoked as `--lib --name cluster_launch::tests::<name>`. That module can use Plan 1's crate-unit-only fixture while still spawning real PTYs/processes. Integration targets retain only production unavailable/invalid paths, public source-contract assertions, and isolated clean-build black-box cases; they never expect a dirty development build to mint official launch identity. No fixture or test feature is exposed to production merely to satisfy an integration test.

- [ ] **Step 2: Run launch and cluster tests and confirm RED**

Run each target exactly:

```bash
python3 scripts/run_exact_test.py --test cluster --name readiness_is_unknown_before_launch_attempt --serial
python3 scripts/run_exact_test.py --test cluster --name unavailable_embedded_registry_keeps_dashboard_usable_until_launch --serial
python3 scripts/run_exact_test.py --test cluster --name launch_attempt_allocates_id_before_request_suspend_or_spawn_failure --serial
python3 scripts/run_exact_test.py --lib --name cluster_launch::tests::launch_attempt_uses_one_remote_process_and_one_lookup --serial
python3 scripts/run_exact_test.py --lib --name cluster_launch::tests::valid_ready_is_stamped_with_attempt_id_and_time --serial
python3 scripts/run_exact_test.py --test cluster --name readiness_record_does_not_become_periodic_host_state --serial
python3 scripts/run_exact_test.py --lib --name cluster_launch::tests::pre_ready_spawn_failure_still_records_attempt_id_time_and_optional_child_outcome --serial
python3 scripts/run_exact_test.py --lib --name cluster_launch::tests::failed_frame_preserves_launch_failure_and_bounded_diagnostic --serial
python3 scripts/run_exact_test.py --lib --name cluster_launch::tests::path_replacement_after_ready_cannot_change_classified_process --serial
python3 scripts/run_exact_test.py --test cluster --name cluster_launcher_registry_fields_are_private_and_production_uses_embedded_manifest --serial
python3 scripts/run_exact_test.py --test cluster --name cluster_launcher_fixture_registry_is_cfg_test_only --serial
python3 scripts/run_exact_test.py --lib --name cluster_launch::tests::fixture_registry_is_available_only_to_unit_tests --serial
python3 scripts/run_exact_test.py --lib --name cluster_launch::tests::launch_attempt_suspends_and_transfers_guard_once --serial
python3 scripts/run_exact_test.py --lib --name cluster_launch::tests::launch_total_paths_never_short_circuit_before_record --serial
python3 scripts/run_exact_test.py --lib --name cluster_launch::tests::proxy_failure_records_attempt_only_after_child_reap_and_stream_join --serial
python3 scripts/run_exact_test.py --lib --name cluster_launch::tests::cluster_proxy_post_ready_descendant_pipe_is_killed_reaped_and_joined_before_resume --serial
python3 scripts/run_exact_test.py --lib --name cluster_launch::tests::cluster_proxy_user_interrupt_terminates_group_and_restores_once --serial
python3 scripts/run_exact_test.py --lib --name cluster_launch::tests::cluster_proxy_reader_panic_is_contained_reaped_joined_and_reported --serial
python3 scripts/run_exact_test.py --lib --name cluster_launch::tests::cluster_proxy_os_resize_uses_suspended_terminal_signal_broker --serial
```

Expected: FAIL because cluster state has no attempt-scoped readiness record and currently conflates launch log text with host state.

- [ ] **Step 3: Add orchestration records without changing the wire protocol**

Define:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaunchAttemptId(pub u64);

pub struct LaunchAttemptRecord {
    pub id: LaunchAttemptId,
    pub alias: String,
    pub attempted_at: SystemTime,
    pub ready_identity: Option<LaunchIdentity>,
    pub outcome: LaunchAttemptOutcome,
    pub escaped_diagnostic: Option<String>,
    pub proxy_terminal_outcome: Option<ProxyTerminalOutcome>,
    pub local_interrupt: Option<InterruptSignal>,
}

pub enum LaunchAttemptOutcome {
    Child(ChildOutcome),
    CompatibilityUnavailable,
    RequestInvalid,
    TerminalSuspendFailed,
    ProxyStartFailed { child_outcome: Option<ChildOutcome> },
}

enum RegistryAvailability {
    Available(CompatibilityRegistry),
    Unavailable { escaped_diagnostic: String },
}

pub struct ClusterLauncher {
    registry: RegistryAvailability,
    next_attempt: u64,
}

impl ClusterLauncher {
    pub fn from_embedded() -> Self;

    #[cfg(test)]
    pub(crate) fn from_fixture(fixture: CompatibilityFixture) -> Result<Self, CompatibilityError>;

    pub fn launch(
        &mut self,
        terminal: &mut TerminalSession,
        host: &HostConfig,
        workdir: &OsStr,
    ) -> LaunchAttemptRecord;
}
```

`ClusterLauncher::from_embedded` always returns a launcher. It obtains `BuildIdentity::embedded()` and calls only `CompatibilityRegistry::from_embedded_build_identity`; any identity/registry failure is retained as bounded private `RegistryAvailability::Unavailable` rather than aborting dashboard construction. Its private fields prevent production struct literals or registry substitution. Before a launch, readiness therefore remains `Unknown` even in an unofficial build; the explicit attempt is what records `CompatibilityUnavailable`. The `cfg(test)` constructor accepts only `CompatibilityFixture`, never a production `CompatibilityRegistry` value, JSON blob, environment pair, or path. A crate-unit test under `cluster_launch::tests` proves the constructor works only in that configuration. Integration source-contract tests compile a production-facing fixture that must fail to find it and inspect the module source to reject public fields or a non-test injection API.

`ClusterLauncher::launch` is total and allocates attempt ID/time before checking registry availability or performing any fallible request, command, terminal, or proxy operation. It uses explicit `match` at each boundary; no `?`, early `return` without a `LaunchAttemptRecord`, log-only failure, or conversion to a bare `Result<()>` is permitted. Registry unavailability, request validation, and `TerminalSession::suspend` failure produce a record with `ready_identity=None`, the corresponding no-child `LaunchAttemptOutcome`, bounded diagnostic, and `proxy_terminal_outcome=None`. They do not fabricate a `ChildOutcome`.

For a valid request, the launcher calls `RemoteLaunchRequest::new`, uses only its canonical `remote_exec_command`, and builds the private-field proxy configuration through `RemoteProxySpec::for_request`; the nonce, five-second deadline, and 512-byte diagnostic bound therefore come from Plan 1 rather than caller literals. The outer cluster loop calls its sole `TerminalSession::suspend` once and transfers that guard to `RemoteProxySession::spawn`; it never resumes the guard itself after transfer. While `session.run` is synchronous, the transferred guard remains the unique production source for `SIGWINCH` and HUP/INT/TERM through Plan 1's already-installed `TerminalSession` signal broker, so resize/interrupt does not depend on an outer loop or a test sender. `spawn` failure is consumed through `RemoteProxyStartError::into_parts` and records `ProxyStartFailed { child_outcome }`, diagnostic, and the exact `ProxyTerminalOutcome`. Successful spawn calls exactly `session.run(&registry)`, then consumes `RemoteProxyCompletion::into_parts`. Those returned parts populate the one cluster-domain `LaunchAttemptRecord` only after Plan 1 has terminated/reaped the group, drained/joined both readers, and resumed/restored the dashboard. Plan 5 never defines a wrapper around `RemoteProxyCompletion`/`RemoteProxyStartError`, constructs a second proxy result, parses READY, starts a control thread or stdin reader, owns a wake fd, holds child/PTY/reader handles, or calls terminal resume around Plan 1.

The real-process integration tests exercise Plan 1 through `ClusterLauncher`, not a mock proxy. One post-READY trace leaves a grandchild holding an output pipe; another launches the full cluster path in an isolated controlling PTY, changes that PTY with `TIOCSWINSZ`, sends real `SIGWINCH`, verifies the remote PTY size, then sends real HUP/INT/TERM and verifies the corresponding `local_interrupt` survives cleanup and outer exit. Separate bound-child fixtures independently exit 129/130/143 and verify only `RemoteInterrupted` with no local intent; a final trace fires the deterministic reader-panic seam. They do not construct or send through a test control channel. In every trace, the entire group is TERM/KILLed as required, the direct child is reaped, descendant pipes close, readers join, one stdin owner remains, and dashboard resume occurs exactly once after those facts. No `LaunchAttemptRecord` outcome is published earlier. Protocol, nonce, source-pair, timeout, EOF, proxy, and transport failures before an accepted READY have `ready_identity=None`, the exact terminal `ChildOutcome`, bounded diagnostic, and truthful `ProxyTerminalOutcome`; a local spawn failure instead retains its optional child outcome and local interrupt without inventing either. Once READY is accepted, later `Closed`, `UserAborted`, `RemoteInterrupted`, proxy, EOF, or transport termination retains that accepted `ready_identity` together with the exact child, terminal, and local-signal truths. A restore failure makes the outer cluster run fail after first retaining the attempt record; it cannot resume rendering into an unproved terminal. The already-started remote Tersh enters its TUI directly, with no readiness probe, second lookup, or second exec. `ClusterApp` stores the result for display, labels readiness `Unknown` only when no attempt exists, and never feeds the record into refresh generation state.

`ClusterLauncher` copies the returned `local_interrupt` into the attempt record on both completion and proxy-start-error paths. The outer cluster coordinator must first store that complete record, then, if the field is present, stop rendering and return the matching `RunOutcome::Interrupted` only after Plan 1 reports terminal restoration. It emits no cwd/stdout and exits 129/130/143. A remote child that independently reports 129/130/143 leaves this field empty and changes only the visible `ChildOutcome`; it never exits the local dashboard by numerical coincidence. The real PTY HUP/INT/TERM cases assert record-before-exit ordering, exact local status, empty stdout, child reap, reader join, and restoration.

- [ ] **Step 4: Run complete remote-launch and cluster suites**

Run: `cargo test --locked --lib --test remote_launch --test cluster -- --nocapture --test-threads=1`

Expected: PASS; the private fixture crate-unit test and every Plan 1 protocol/integration test remain green, and readiness is truthful and attempt-scoped.

- [ ] **Step 5: Commit launch integration**

```bash
git add src/cluster_launch.rs src/lib.rs src/cluster.rs src/cluster_ui.rs tests/remote_launch.rs tests/cluster.rs tests/fixtures/fake_remote_launcher.sh
git commit -m "feat: scope cluster readiness to launch attempts"
```

### Task 5: Compact and normal truthful dashboard states

**Files:**
- Modify: `src/cluster_ui.rs:17-702`
- Modify: `tests/cluster.rs:575-880`

- [ ] **Step 1: Write failing dashboard tests**

Add exact tests `cluster_40x10_shows_mode_back_generation_counts_cancel_help_and_quit`, `cluster_40x10_shows_active_stopping_and_error_without_hiding_summary`, `cluster_40x10_scrolls_failure_detail_without_hiding_summary`, `cluster_80x24_shows_pending_active_terminal_counts`, `cluster_help_promises_full_configured_refresh`, `cluster_quit_never_says_cancel_while_probes_continue`, and `cluster_launch_labels_all_child_outcomes_exactly`.

- [ ] **Step 2: Run rendering tests and confirm RED**

Run each target exactly:

```bash
python3 scripts/run_exact_test.py --test cluster --name cluster_40x10_shows_mode_back_generation_counts_cancel_help_and_quit
python3 scripts/run_exact_test.py --test cluster --name cluster_40x10_shows_active_stopping_and_error_without_hiding_summary
python3 scripts/run_exact_test.py --test cluster --name cluster_40x10_scrolls_failure_detail_without_hiding_summary
python3 scripts/run_exact_test.py --test cluster --name cluster_80x24_shows_pending_active_terminal_counts
python3 scripts/run_exact_test.py --test cluster --name cluster_help_promises_full_configured_refresh
python3 scripts/run_exact_test.py --test cluster --name cluster_quit_never_says_cancel_while_probes_continue
python3 scripts/run_exact_test.py --test cluster --name cluster_launch_labels_all_child_outcomes_exactly
```

Expected: FAIL because the current UI has no generation counts/readiness lifetime and cancellation copy does not prove children have stopped.

- [ ] **Step 3: Render scheduler truth and bounded detail**

Render current cluster mode, Back, generation ID plus pending/active/terminal counts, one active/stopping/error summary line at 40x10, and scrollable bounded escaped detail. Help may say `r refresh every configured host` only after Task 3 proves the full captured sweep. While shutdown is draining, display `stopping probes`; show `Cancelled` only after reaping. Show `Unknown` readiness before launch and the attempt timestamp/outcome after launch. Preserve the exact `Closed`, `UserAborted`, `RemoteInterrupted`, `LaunchFailed`, `TransportFailed`, and `Signaled` labels.

- [ ] **Step 4: Run rendering and remote-outcome regressions**

Run: `cargo test --locked --test cluster --test remote_launch -- --nocapture --test-threads=1`

Expected: PASS at 40x10 and normal sizes; every launcher classification remains exact.

- [ ] **Step 5: Commit dashboard truth**

```bash
git add src/cluster_ui.rs tests/cluster.rs
git commit -m "feat: render truthful cluster sweep status"
```

### Task 6: G3 and repository regression gate

**Files:**
- Modify if the RED matrix requires the smallest G3 production correction: `src/cluster_scheduler.rs`
- Modify if the RED matrix requires the smallest G3 production correction: `src/cluster_probe.rs`
- Modify if the RED matrix requires the smallest G3 production correction: `src/cluster_launch.rs`
- Modify if the RED matrix requires the smallest G3 production correction: `src/cluster.rs`
- Modify if the RED matrix requires the smallest G3 production correction: `src/cluster_ui.rs`
- Modify if ADD-009 requires the smallest substrate correction: `src/trusted_fs.rs`
- Modify if ADD-009 requires the smallest EXDEV host correction: `src/exdev.rs`
- Modify: `README.md`
- Modify: `CHANGELOG.md`
- Modify: `tests/cluster_scheduler.rs`
- Modify: `tests/remote_launch.rs`
- Modify: `tests/cluster.rs`
- Modify: `tests/exdev.rs`

- [ ] **Step 1: Add frozen scheduler/process/launcher and ADD-009 matrix; retain Plan 4's private ADD-010 gates**

First require the Task 5 component boundary to be clean:

```bash
test -z "$(git status --porcelain=v1 --untracked-files=all)"
git rev-parse HEAD
```

Expected: status is empty and `HEAD` is one 40-hex baseline commit. Stop rather than staging pre-existing changes in any Task 6 production file.

Add exact parameterized tests `g3_sweeps_1_16_17_40_hosts_to_one_terminal_each`, `g3_real_process_group_count_never_exceeds_16`, `cluster_probe::tests::g3_timeout_and_quit_term_wait_kill_reap_and_join`, `g3_refresh_again_and_late_generation_matrix`, `cluster_launch::tests::g3_launch_frame_and_child_outcome_matrix`, and `add_009_exdev_receipt_deserialization_rejects_forged_raw_capabilities`. The G3 shutdown matrix is a crate-unit test because its reader-panic/signal-failure rows use the private `#[cfg(test)]` Probe fault controller; it still spawns the real process groups/readers and observes bounded TERM/KILL/reap/join. The G3 launch matrix is likewise a crate-unit test because its `ready-valid`, bound-child, and local-signal rows require the same `cfg(test)` compatibility fixture as Task 4; it still spawns the real proxy, process group, PTY, signal broker, readers, and cleanup lifecycle. Neither fixture exists in an integration dependency or production build. Do not create a new integration-test ADD-010 matrix. Retain and rerun Plan 4's private crate-unit `exdev::tests::exdev_transition_rejects_genuine_token_from_other_bundle_revision_or_edge` and separate `exdev::tests::exdev_consumed_transition_token_cannot_be_used_twice` unchanged.

Each test emits exactly one canonical `tersh-case-count-v1` line. Freeze these ordered IDs and counts; a missing, duplicate, extra, or reordered ID fails even if the Rust test otherwise returns success:

| Matrix ID | Frozen ordered case IDs | Count |
| --- | --- | ---: |
| `g3-sweeps` | `hosts-1`, `hosts-16`, `hosts-17`, `hosts-40` | 4 |
| `g3-process-count` | `live-1`, `live-16`, `refill-17`, `refill-40` | 4 |
| `g3-shutdown` | `queued-quit`, `active-quit-term`, `active-quit-kill`, `timeout-term`, `timeout-kill`, `grandchild-pipe`, `reader-eof`, `reader-panic` | 8 |
| `g3-refresh` | `one-followup`, `latest-followup-wins`, `late-token`, `late-generation` | 4 |
| `g3-launch` | `ready-valid`, `ready-malformed`, `ready-oversize`, `ready-timeout`, `source-pair-unknown`, `exit-0`, `exit-2`, `exit-127`, `exit-129`, `exit-130`, `exit-143`, `exit-255`, `local-signal` | 13 |
| `add-009-exdev-serde` | `empty-name`, `dot-name`, `dotdot-name`, `slash-name`, `nul-name`, `padded-base64`, `aliased-base64`, `malformed-base64`, `invalid-path-component` | 9 |
| `exdev-transition-replay` (inherited Plan 4 crate-unit) | `other-bundle`, `other-revision`, `other-edge` | 3 |

ADD-009 deserializes forged EXDEV receipt JSON through the real `ExdevReceipt` and Plan 2 custom `RawUnixName`/`RawUnixPath` deserializers; none of the nine cases may yield a child/path capability or reach a syscall. The inherited three-case ADD-010 replay gate runs with `--lib` because it obtains a fresh genuine non-clone/non-serde proof from the owning locked verifier and consumes it against another bundle, current revision, or legal edge; all three reject without advancing either receipt. By-value second use is not a fourth runtime matrix case: the separate crate-unit compile/API gate `exdev_consumed_transition_token_cannot_be_used_twice` proves the consumed token cannot be supplied again. An `Option::take`, fabricated marker, public constructor, or integration target cannot substitute for either private gate.

- [ ] **Step 2: Run the G3 gate and expose missing evidence**

Run each new gate target through the shared exact helper with its frozen case inventory:

```bash
python3 scripts/run_exact_test.py --test cluster_scheduler --name g3_sweeps_1_16_17_40_hosts_to_one_terminal_each --serial --case-matrix g3-sweeps --expect-case hosts-1 --expect-case hosts-16 --expect-case hosts-17 --expect-case hosts-40
python3 scripts/run_exact_test.py --test cluster_scheduler --name g3_real_process_group_count_never_exceeds_16 --serial --case-matrix g3-process-count --expect-case live-1 --expect-case live-16 --expect-case refill-17 --expect-case refill-40
python3 scripts/run_exact_test.py --lib --name cluster_probe::tests::g3_timeout_and_quit_term_wait_kill_reap_and_join --serial --case-matrix g3-shutdown --expect-case queued-quit --expect-case active-quit-term --expect-case active-quit-kill --expect-case timeout-term --expect-case timeout-kill --expect-case grandchild-pipe --expect-case reader-eof --expect-case reader-panic
python3 scripts/run_exact_test.py --test cluster_scheduler --name g3_refresh_again_and_late_generation_matrix --serial --case-matrix g3-refresh --expect-case one-followup --expect-case latest-followup-wins --expect-case late-token --expect-case late-generation
python3 scripts/run_exact_test.py --lib --name cluster_launch::tests::g3_launch_frame_and_child_outcome_matrix --serial --case-matrix g3-launch --expect-case ready-valid --expect-case ready-malformed --expect-case ready-oversize --expect-case ready-timeout --expect-case source-pair-unknown --expect-case exit-0 --expect-case exit-2 --expect-case exit-127 --expect-case exit-129 --expect-case exit-130 --expect-case exit-143 --expect-case exit-255 --expect-case local-signal
python3 scripts/run_exact_test.py --test exdev --name add_009_exdev_receipt_deserialization_rejects_forged_raw_capabilities --serial --case-matrix add-009-exdev-serde --expect-case empty-name --expect-case dot-name --expect-case dotdot-name --expect-case slash-name --expect-case nul-name --expect-case padded-base64 --expect-case aliased-base64 --expect-case malformed-base64 --expect-case invalid-path-component
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_transition_rejects_genuine_token_from_other_bundle_revision_or_edge --serial --case-matrix exdev-transition-replay --expect-case other-bundle --expect-case other-revision --expect-case other-edge
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_consumed_transition_token_cannot_be_used_twice
```

Expected: each helper discovers and executes exactly one test. The six new G3/ADD-009 tests FAIL on their first unimplemented or incorrectly counted case; both inherited Plan 4 ADD-010 crate-unit gates PASS unchanged. A zero-test pass, a test with a different ordered case set/count, a private gate moved to `--test exdev`, or a forged rather than genuine proof is failure.

- [ ] **Step 3: Apply only the smallest production correction exposed by RED**

Map each failure to the narrow file list above. Scheduler/refill/generation defects may change only `cluster_scheduler.rs`; process-count/TERM/KILL/reap/join containment only `cluster_probe.rs`; launch orchestration/registry construction only `cluster_launch.rs`; event-loop dispatch/drain/quit integration only `cluster.rs`; rendering truth only `cluster_ui.rs`; ADD-009 decoding only `trusted_fs.rs` and, if the host receipt itself bypasses it, `exdev.rs`; an inherited ADD-010 failure returns only to its owning Plan 4 `state_root.rs`/`exdev.rs` component before Plan 5 resumes. Do not modify `remote_launch.rs`, `terminal_session.rs`, build identity, workflows, manifests, or any evidence file in this task. A Plan 1 lifecycle failure returns to its component recipe and creates a new upstream candidate before this task resumes.

Implement the minimum correction needed for the named failing case. Per-record scheduler/probe failures remain contained outcomes; no correction may reintroduce `Result<()>` early return, synchronous normal-quit join, public registry fields, production fixture injection, raw receipt replacement, clone/serde proof tokens, or a second remote proxy/terminal owner.

- [ ] **Step 4: Rerun every focused matrix and the complete component suites**

Rerun all eight exact-helper commands from Step 2, in the same order and with the same `--test`/`--lib`, `--name`, `--serial`, matrix ID, and ordered case arguments. Then run:

```bash
cargo test --locked --test cluster_scheduler --test remote_launch --test cluster --test exdev -- --nocapture --test-threads=1
```

Expected: every frozen matrix prints its exact declared count and exits 0; all G3, Plan 1, ADD-009, and the two inherited private ADD-010 gates exit 0; no child, descendant, PTY reader, or probe reader remains. This component plan does not approximate cumulative evidence with a local "prior gates" script. Exact same-candidate cumulative local gates plus the single external CI/release job-and-artifact invocation run only after the candidate is frozen, using Task 8 and the locked per-iteration argv in the implementation-evidence plan.

- [ ] **Step 5: Commit the GREEN production and test boundary before documentation**

Do not edit or stage `README.md` or `CHANGELOG.md` yet. Stage only Task 6 production and test paths that actually changed, reject every unexpected path, commit them, and require a clean boundary:

```bash
git add src/cluster_scheduler.rs src/cluster_probe.rs src/cluster_launch.rs src/cluster.rs src/cluster_ui.rs src/trusted_fs.rs src/exdev.rs tests/cluster_scheduler.rs tests/remote_launch.rs tests/cluster.rs tests/exdev.rs
git diff --cached --name-only | while IFS= read -r TERSH_P5_PATH; do
  case "$TERSH_P5_PATH" in
    src/cluster_scheduler.rs|src/cluster_probe.rs|src/cluster_launch.rs|src/cluster.rs|src/cluster_ui.rs|src/trusted_fs.rs|src/exdev.rs|tests/cluster_scheduler.rs|tests/remote_launch.rs|tests/cluster.rs|tests/exdev.rs) ;;
    *) printf 'unexpected staged production/test path: %s\n' "$TERSH_P5_PATH" >&2; exit 1 ;;
  esac
done
git diff --cached --name-only | grep -Eq '^(src/|tests/)'
git commit -m "test: close g3 cluster component gates"
test -z "$(git status --porcelain=v1 --untracked-files=all)"
git rev-parse HEAD
```

Expected: the staged set contains only changed production/tests, never documentation or evidence, the commit succeeds, status is empty, and the final command prints one 40-hex production/test boundary SHA. If an inherited ADD-010 gate failed, return to the owning Plan 4 component and establish a new upstream boundary before resuming; do not smuggle that correction into this Plan 5 commit.

- [ ] **Step 6: Document, verify, and freeze a docs-only component candidate**

Document full configured-host sweeps, maximum 16 real probes, six-second timeout with nonblocking TERM/500-ms/KILL/reap/join, refresh coalescing, truthful cancellation, and launch-attempt-only readiness in the English and Chinese sections of `README.md`. Under the existing `CHANGELOG.md` `Unreleased` heading, add only the implemented G3 scheduler/shutdown/readiness facts and do not create or rename a release heading. Retain explicit non-goals: no transfer, synchronization, credentials, agent, fleet monitoring, topology, or added metrics. State that these are the Plan5 G3 component-candidate facts, that G3 did not block the earlier Workbench Trusted Core milestone, and that only the later same-candidate `impl-07.json` manifest may accept iteration 7. Do not claim this task or plan completed an iteration or any hardening cycle.

With only those two documentation files modified, run repository-wide verification:

```bash
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo build --locked --release --bin tersh
git diff --check
```

Expected: all five commands exit 0; Plan 1 remote-launch tests and Plans 2-4 Workbench tests remain green. Then stage only the two documentation files, prove that the final candidate commit is docs-only, and create it:

```bash
git add README.md CHANGELOG.md
git diff --cached --name-only | while IFS= read -r TERSH_P5_PATH; do
  case "$TERSH_P5_PATH" in
    README.md|CHANGELOG.md) ;;
    *) printf 'unexpected staged candidate path: %s\n' "$TERSH_P5_PATH" >&2; exit 1 ;;
  esac
done
test "$(git diff --cached --name-only | wc -l | tr -d ' ')" -eq 2
git commit -m "docs: freeze g3 cluster component candidate"
test -z "$(git status --porcelain=v1 --untracked-files=all)"
git rev-parse HEAD
```

Expected: the final commit contains exactly `README.md` and `CHANGELOG.md`, status is empty, and the final command prints one 40-hex candidate SHA whose parent is the GREEN production/test boundary. This SHA is the Plan5 component candidate only. Do not run cumulative CI/release evidence before this commit, create `impl-07.json`, call `finalize_iteration.py`, or claim slice/iteration acceptance here. Hand this exact SHA to Task 8 of the implementation-iteration evidence plan; that task alone uses its complete locked cumulative-gate argv and single external-helper CI/release job-and-artifact argv for `impl-07`, closes five roles on the same candidate, and commits only the resulting `impl-07.json`.

## Spec-to-task map and acceptance boundary

| Design requirement | Implemented and proven by |
| --- | --- |
| Fixed alias set, queue, 16 active, refill, generations (`:982-999`) | Tasks 1 and 3 |
| Process group timeout/quit/reap/readers (`:986-1004`) | Task 2's immediate post-spawn owner, per-reader nonblocking cleanup wakes, bounded drain/join, and Task 6 |
| Per-probe bounded drain and fail-contained aggregate shutdown (normative `:1437-1441`) | Tasks 2, 3, and 6 |
| Dispatch/drain contain every record; normal quit yields through TERM/500-ms/KILL/reap/join | Tasks 2-3 contained batches, bounded deferred actions, and render-through-shutdown tests |
| Refresh coalescing and late-result rejection (`:995-1004`) | Tasks 1 and 3 |
| Single-process protocol and truthful child outcome (`:473-539`) | Plan 1 implementation; regression/integration in Task 4 |
| One outer terminal/proxy owner, reap/drain/join/resume on every exit (normative `:1429-1435`) | Plan 1 plus Task 4 integration tests |
| Production compatibility comes only from embedded accepted manifest/build identity; fixture injection is `cfg(test)` | Locked Plan 1 handoff and Task 4 source-contract/integration tests |
| Attempt-scoped readiness (`:1005-1017`) | Task 4 |
| 40x10 summary and normal details (`:1022-1044`) | Task 5 |
| G3 evidence matrix (`:1103`, `:1157-1167`) | Task 6 |
| Focused tests reject zero discovery/execution (normative `:1471-1474`) | Plan 1 exact-test runner used by Tasks 2-6 |
| Raw capability serde forgery is rejected (ADD-009, `:1488-1497`) | Task 6 exact genuine EXDEV receipt matrix `add-009-exdev-serde` |
| Genuine proof cannot replay across bundle/revision/edge or be consumed twice (ADD-010, `:1499-1505`) | Task 6 reruns Plan 4 private `--lib` three-case `exdev-transition-replay` gate plus the separate single-use compile/API gate |
| Exact external job IDs remain aligned across release, EXDEV, implementation evidence, and hardening | Locked boundary above; acceptance performed only by the `impl-07` manifest plan |
| Cluster scope non-goals (`:1018-1020`, `:1187-1207`) | Task 6 documentation |
| Component recipes do not self-accept an iteration | Header/boundary plus Task 6 component candidate; only implementation-evidence `impl-07.json` accepts |

Tasks 1-6 can make one clean Plan5 component candidate eligible for `impl-07`; they cannot independently accept the feature slice or implementation iteration. G3 remains outside the Workbench Trusted Core release boundary. Acceptance occurs only when `2026-08-10-tersh-implementation-iteration-evidence.md` runs G3 and every prior gate plus the locked external job IDs, closes all five roles on this exact candidate, and commits only the resulting `impl-07.json`. The later seven-cycle hardening plan remains separate and incomplete at that point.
