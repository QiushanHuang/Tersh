# Tersh Responsive Mutation Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver G1a asynchronous scan/preview responsiveness and G1b truthful serial mutations, including the fixed trusted state root and durable `SourceClaim` substrate, without implementing EXDEV move or recoverable-trash UI.

**Architecture:** Keep `App` as the sole UI coordinator. Add one concrete scan worker with keyed latest-wins work, one concrete preview worker, and one concrete serial mutation worker; all communication remains bounded. The mutation worker performs filesystem preflight and create-new reservation before it freezes Plan 1's `OperationRequest`, then waits for a non-droppable UI fence acknowledgement before any user-visible effect. Filesystem trust, owning fixed-control claims, monotonic receipt transitions, and nested `SourceClaim` behavior stay below the worker so Plans 3 and 4 can reuse them without depending on `App`.

**Tech Stack:** Rust 2024, standard threads/channels/atomics, crossterm, ratatui, serde/serde_json, sha2, libc, anyhow, tempfile, Tier-1 macOS/Linux no-follow and no-replace syscalls.

---

## Scope and prerequisite contract

Start from `codex-tersh-trusted-core` commit `799cf08`. Before this plan, `cargo test --locked --all-targets` passes 185 tests. Plan 1 must already provide:

- `src/operation.rs`: `OperationId`, `ItemId`, `OperationKind`, `ConflictPolicy`, `OperationRequest`, `ItemPlan`, `OperationEvent::{Started, Progress, ItemOutcome, Finished}`, `ItemOutcome`, `NotStartedReason`, `EffectRole`, `CompletionState`, `OperationReport`, `OperationSummary`, and `ReportStore`;
- `src/process_outcome.rs`: `RunOutcome` and `ExitCode`; and
- the deterministic reducer, first-final-wins rule, 10,000 top-level target cap, active/latest full report plus 20 summaries, and retry-as-new-preflight behavior from design lines 202-296 and 640-679.

Plan 2 may add accessors to those types but must not duplicate or rename them. G1b follows the normative addendum at design lines 1324-1427: the UI submits only a confirmed `MutationIntent` plus ephemeral `SubmissionId`; the worker performs final filesystem preflight, generates the final `OperationId`/`ItemId`s, reserves every applicable marker create-new, and only then constructs the immutable `OperationRequest` and `ItemPlan`s. A retry creates a new intent and therefore receives new final IDs and freshly captured identities.

Plan 1 must also provide `python3 scripts/run_exact_test.py (--test <target> | --lib) --name <full-name> [--ignored] [--serial] [--case-matrix <id> --expect-case <case> ...]`. The mutually exclusive selector first lists the chosen integration-test binary or library tests, requires the exact full name exactly once, then runs it and fails if zero tests execute or a frozen case matrix differs. Every Plan 2 focused exact gate below uses this runner; if it is absent, stop before Task 1 and finish the Plan 1 prerequisite rather than substituting a raw Cargo filter.

Plan 2 explicitly excludes cross-filesystem move, the G2 trash receipt/restore engine, Recovery overlay, cluster scheduling, a generic executor, a generic thread pool, and a database. It does, however, lock the bounded preparation, keyed-scan, claimed-control, mirror-handoff, and exact-bundle capability seams that Plans 3 and 4 must extend instead of bypassing.

## Current-code anchors

- `src/app.rs:101-132`: `App` owns all state but has no generations, epoch, fences, workers, or report store.
- `src/app.rs:301-417`: construction immediately calls synchronous `reload`.
- `src/app.rs:451-583`: command dispatch calls reads and writes directly.
- `src/app.rs:913-980`: directory scan and preview both block the caller; failed reload clears the last good list and selection.
- `src/app.rs:1219-1263`: cursor/cwd changes synchronously cascade into preview and reload.
- `src/app.rs:1274-1407`: captures operation targets and conflict decisions; replace remains reachable.
- `src/app.rs:1409-1492`: copy/move loop executes synchronously and reduces truth to log lines.
- `src/app.rs:1503-1624`: rename/trash/delete execute synchronously and clear input/selection even on failure.
- `src/app.rs:1783-1787`: six-line log is the only current operation history.
- `src/app.rs:1791-1839`: event loop polls only terminal input and returns only a cwd.
- `src/fs_core.rs:85-133`: reusable directory scan backend.
- `src/preview.rs:44-159`: reusable bounded, no-follow preview backend.
- `src/fs_ops.rs:14-180`: copy/rename/trash/delete policy and syscall behavior are mixed.
- `src/fs_ops.rs:207-297`: copy publishes directly to the final path and uses `io::copy` without cancellation.
- `src/fs_ops.rs:307-373`: private identity duplicates `app.rs:192-242`.
- `src/fs_ops.rs:431-488`: cleanup/delete performs identity-check-then-path-delete.
- `src/fs_ops.rs:490-542`: platform no-replace primitives are useful but path-based.
- `src/ui.rs:368-430`: Inspector currently renders target metadata plus recent log lines.
- `src/ui.rs:450-494` and `539-614`: footer/modal are the integration points for loading, stale, active, cancel, and validation states.
- `tests/app_keys.rs:262-340`: current retry/source-drift regression cases.
- `tests/app_keys.rs:437-448`: current test asserts the unsafe reload-clears-selection behavior and must be reversed.
- `tests/fs_ops.rs:6-337`: legacy operation coverage to retain while safe callers replace old entry points.
- `tests/render.rs:21-165` and `284-351`: Inspector/footer and 40x10 survival rendering anchors.

## Locked module boundaries and interfaces

### `src/read_lane.rs`

```rust
pub type CwdGeneration = u64;
pub type PreviewGeneration = u64;
pub type FsEpoch = u64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadFailure { pub escaped_message: String }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadLaneClosed;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanToken {
    pub cwd_generation: CwdGeneration,
    pub fs_epoch: FsEpoch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewToken {
    pub cwd_generation: CwdGeneration,
    pub preview_generation: PreviewGeneration,
    pub fs_epoch: FsEpoch,
}

pub enum ScanRequestKind {
    Directory { cwd: PathBuf, show_hidden: bool },
}

pub struct ScanRequest {
    pub token: ScanToken,
    pub kind: ScanRequestKind,
}

pub struct PreviewRequest {
    pub token: PreviewToken,
    pub path: PathBuf,
}

pub enum ReadEvent {
    ScanFinished { token: ScanToken, cwd: PathBuf, result: Result<DirectoryEntries, ReadFailure> },
    PreviewFinished { token: PreviewToken, path: PathBuf, result: Result<Preview, ReadFailure> },
    ScanWorkerLost,
    PreviewWorkerLost,
}

pub struct ReadLanes {
    scan: ScanLane,
    preview: PreviewLane,
    event_rx: Receiver<ReadEvent>,
}

pub type DirectoryScanBackend = Arc<dyn Fn(&Path, bool) -> Result<DirectoryEntries, ReadFailure> + Send + Sync>;
pub type PreviewBackend = Arc<dyn Fn(PreviewRequest) -> Result<Preview, ReadFailure> + Send + Sync>;

pub struct ScanBackendSet {
    pub directory: DirectoryScanBackend,
}

impl ReadLanes {
    pub fn start() -> Self;
    #[doc(hidden)]
    pub fn start_with(backends: ScanBackendSet, preview: PreviewBackend) -> Self;
    pub fn request_scan(&self, request: ScanRequest) -> Result<(), ReadLaneClosed>;
    pub fn request_preview(&self, request: PreviewRequest) -> Result<(), ReadLaneClosed>;
    pub fn try_recv(&self) -> Result<Option<ReadEvent>, ReadLaneClosed>;
    pub fn close(self);
}
```

The preview lane owns one `Mutex<Option<PreviewRequest>>` plus `Condvar`. The scan worker instead owns a concrete `ScanMailbox`; in Plan 2 it contains only `directory: Option<ScanRequest>`, `closed`, and a next-kind cursor. Plan 4 extends that exact mailbox with `recovery_catalog: Option<ScanRequest>` and `ScanWorkKey::{Directory,RecoveryCatalog}`; the `ScanRequest` wrapper, rather than the recovery-domain payload, carries the mandatory `ScanToken`. Submission replaces only the matching key. The worker alternates fairly whenever both keys are present, so Directory and RecoveryCatalog never supersede each other and the post-Plan-4 bound is exactly two pending scan requests, not a queue.

The shared result channel is `sync_channel(4)`. A worker may block on that bounded channel; the UI drains it on every event-loop turn. Closing the mailbox and preview slot, then dropping the receiver, releases blocked sends. The scan worker pattern-matches concrete work, invokes the corresponding fixed field in `ScanBackendSet`, and itself constructs the matching `ReadEvent` with the request token; a backend cannot forge a token or event kind. Plan 2 keeps the public work types directory-only and invents no G2 record. Plan 4 modifies the concrete `ScanMailbox`, `ScanRequestKind`, `ReadEvent`, and `ScanBackendSet` on the same feature branch to add its typed recovery request/page/backend while retaining the same scan thread, keyed replacement, fair alternation, `sync_channel(4)`, and two-request maximum. It may not add a third worker, a second catalog queue, `Any`, or a generic job interface.

### `src/trusted_fs.rs`

```rust
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ObjectKind { RegularFile, Directory, Symlink, Other }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathIdentity {
    device: u64,
    inode: u64,
    kind: ObjectKind,
    len: u64,
    mode: u32,
    uid: u32,
    gid: u32,
    mtime_sec: i64,
    mtime_nsec: i64,
    ctime_sec: i64,
    ctime_nsec: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PathIdentityKey {
    device: u64,
    inode: u64,
    kind: ObjectKind,
    len: u64,
    mode: u32,
    uid: u32,
    gid: u32,
    mtime_sec: i64,
    mtime_nsec: i64,
    ctime_sec: i64,
    ctime_nsec: i64,
}

impl PathIdentity {
    pub fn capture_no_follow(path: &Path) -> Result<Self, TrustedFsError>;
    pub fn same_object(&self, other: &Self) -> bool;
    pub fn same_snapshot(&self, other: &Self) -> bool;
    pub fn stable_key(&self) -> PathIdentityKey;
    pub fn kind(&self) -> ObjectKind;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RawUnixPath { base64url_no_pad: String }

impl RawUnixPath {
    pub fn capture(value: &Path) -> Result<Self, TrustedFsError>;
    pub fn from_bytes(value: Vec<u8>) -> Result<Self, TrustedFsError>;
    pub fn to_bytes(&self) -> Result<Vec<u8>, TrustedFsError>;
    pub fn encoded(&self) -> &str;
    pub fn to_path_buf(&self) -> Result<PathBuf, TrustedFsError>;
}

impl<'de> Deserialize<'de> for RawUnixPath {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RawUnixName { base64url_no_pad: String }

impl RawUnixName {
    pub fn capture(value: &OsStr) -> Result<Self, TrustedFsError>;
    pub fn from_bytes(value: Vec<u8>) -> Result<Self, TrustedFsError>;
    pub fn to_bytes(&self) -> Result<Vec<u8>, TrustedFsError>;
    pub fn encoded(&self) -> &str;
    pub fn to_os_string(&self) -> Result<OsString, TrustedFsError>;
    pub fn into_observed(self) -> ObservedRawName;
}

impl<'de> Deserialize<'de> for RawUnixName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>;
}

#[derive(Clone, PartialEq, Eq)]
pub struct ObservedRawName { base64url_no_pad: String }

impl ObservedRawName {
    pub fn escaped_display(&self) -> String;
}

impl fmt::Debug for ObservedRawName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl Ord for ObservedRawName {
    fn cmp(&self, other: &Self) -> Ordering;
}

impl PartialOrd for ObservedRawName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnershipPolicy {
    SystemAncestor,
    EffectiveUserPrivate { directory_mode: u32, file_mode: u32 },
}

#[derive(Debug)]
pub enum TrustedFsError {
    UnsafeComponent,
    InvalidName,
    IdentityChanged,
    UnsupportedNoReplace,
    Io(io::Error),
    Json(serde_json::Error),
}

pub struct TrustedDir {
    fd: OwnedFd,
    identity: PathIdentity,
    diagnostic_path: PathBuf,
}

pub enum ChildObservation {
    Validated { name: RawUnixName, identity: PathIdentity },
    InspectOnly { name: ObservedRawName, escaped_error: String },
}

struct OwnedDirStream {
    dirp: NonNull<libc::DIR>,
}

pub struct ChildEnumerator {
    directory: Arc<TrustedDir>,
    stream: OwnedDirStream,
    finished: bool,
}

impl Drop for OwnedDirStream {
    fn drop(&mut self);
}

impl Iterator for ChildEnumerator {
    type Item = Result<ChildObservation, TrustedFsError>;
}

pub struct ClaimedChildLock {
    file: File,
    name: RawUnixName,
    identity: PathIdentity,
    parent_identity: PathIdentity,
}

impl ClaimedChildLock {
    pub(crate) fn name(&self) -> &RawUnixName;
    pub fn identity(&self) -> &PathIdentity;
    pub(crate) fn parent_identity(&self) -> &PathIdentity;
}

impl TrustedDir {
    pub fn open_absolute(path: &Path, policy: &OwnershipPolicy) -> Result<Self, TrustedFsError>;
    pub fn try_clone(&self) -> Result<Self, TrustedFsError>;
    pub fn open_child_dir(&self, name: &RawUnixName, policy: &OwnershipPolicy) -> Result<Self, TrustedFsError>;
    pub fn create_child_dir_new(&self, name: &RawUnixName, mode: u32) -> Result<Self, TrustedFsError>;
    pub fn identity(&self) -> &PathIdentity;
    pub fn refresh_identity(&self) -> Result<PathIdentity, TrustedFsError>;
    pub fn enumerate_children(&self) -> Result<ChildEnumerator, TrustedFsError>;
    pub fn stat_child_no_follow(&self, name: &RawUnixName) -> Result<PathIdentity, TrustedFsError>;
    pub fn try_lock_regular_child(&self, name: &RawUnixName, mode: u32) -> Result<Option<ClaimedChildLock>, TrustedFsError>;
    pub fn rename_child_no_replace(&self, from: &RawUnixName, to_dir: &TrustedDir, to: &RawUnixName) -> Result<(), TrustedFsError>;
    pub fn unlink_verified_child(&self, name: &RawUnixName, expected: &PathIdentity, directory: bool) -> Result<(), TrustedFsError>;
    pub fn sync(&self) -> Result<(), TrustedFsError>;
}

pub(crate) struct AtomicReceiptFile {
    dir: Arc<TrustedDir>,
    name: RawUnixName,
    expected_lock_name: RawUnixName,
    identity: PathIdentity,
}

pub(crate) struct PartialAtomicReceiptFile {
    dir: Arc<TrustedDir>,
    name: RawUnixName,
    expected_lock_name: RawUnixName,
    created_file: Option<File>,
    captured_identity: Option<PathIdentity>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReceiptCreateStage {
    PreCreate,
    FileCreated,
    BytesWritten,
    FileSynced,
    ParentSynced,
}

pub(crate) enum ReceiptCreateDiskTruth {
    Absent,
    Present(PathIdentityKey),
    Unreadable { escaped_error: String },
}

pub(crate) struct AtomicReceiptCreateFailure {
    recovery: PartialAtomicReceiptFile,
    stage: ReceiptCreateStage,
    disk_truth: ReceiptCreateDiskTruth,
    source: TrustedFsError,
}

pub(crate) struct AtomicReceiptCreateAbortFailure {
    recovery: PartialAtomicReceiptFile,
    stage: ReceiptCreateStage,
    disk_truth: ReceiptCreateDiskTruth,
    source: TrustedFsError,
}

pub(crate) enum AtomicReceiptCreation {
    NotStarted,
    Created(AtomicReceiptFile),
    CreateFailed(AtomicReceiptCreateFailure),
}

pub(crate) const MAX_RECEIPT_BYTES: usize = 64 * 1024;

pub(crate) struct VerifiedReceiptSnapshot {
    file: File,
    identity: PathIdentity,
    canonical_bytes: Vec<u8>,
    revision: u64,
    canonical_sha256: [u8; 32],
}

pub(crate) struct AdjacentReceiptFacts<'lock> {
    bundle_identity: PathIdentity,
    snapshot: VerifiedReceiptSnapshot,
    owning_lock: &'lock ClaimedChildLock,
    receipt_sync_completed: ReceiptSyncCompleted,
    parent_sync_completed: ParentSyncCompleted,
}

struct ReceiptSyncCompleted;
struct ParentSyncCompleted;

pub(crate) struct OwnedLockedReceipt<T> {
    receipt: AtomicReceiptFile,
    owning_lock: ClaimedChildLock,
    current: T,
    snapshot: VerifiedReceiptSnapshot,
    receipt_sync_completed: ReceiptSyncCompleted,
    parent_sync_completed: ParentSyncCompleted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReceiptAdvanceStage {
    PreWriteValidation,
    TempCreated,
    TempSynced,
    ReceiptPublished,
    ParentSynced,
}

pub(crate) enum ReceiptAfterFailure<T> {
    Unchanged(T),
    Advanced(T),
    Unreadable { escaped_error: String },
}

pub(crate) struct AtomicReceiptAdvanceError<P> {
    proof: P,
    stage: ReceiptAdvanceStage,
    observed_after_failure: ReceiptAfterFailureBytes,
    source: TrustedFsError,
}

pub(crate) enum ReceiptAfterFailureBytes {
    Unchanged(Vec<u8>),
    Changed(Vec<u8>),
    Unreadable { escaped_error: String },
}

pub(crate) struct OwnedLockedReceiptAdvanceError<T: DurableReceipt> {
    locked_receipt: OwnedLockedReceipt<T>,
    proof: T::Proof,
    stage: ReceiptAdvanceStage,
    observed_after_failure: ReceiptAfterFailure<T>,
    source: TrustedFsError,
}

pub(crate) struct OwnedLockedReceiptVerifyError {
    receipt: AtomicReceiptFile,
    owning_lock: ClaimedChildLock,
    observed: ReceiptAfterFailureBytes,
    source: TrustedFsError,
}

pub(crate) enum ReceiptRemoveStage {
    PreUnlinkValidation,
    ReceiptUnlinked,
    ParentSynced,
}

pub(crate) enum ReceiptRemoveDiskTruth {
    Present { identity: PathIdentity, canonical_bytes: Vec<u8> },
    Absent,
    Unreadable { escaped_error: String },
}

pub(crate) struct AtomicReceiptRemoveFailure {
    receipt: AtomicReceiptFile,
    stage: ReceiptRemoveStage,
    disk_truth: ReceiptRemoveDiskTruth,
    source: TrustedFsError,
}

impl AdjacentReceiptFacts<'_> {
    pub(crate) fn bundle_identity(&self) -> &PathIdentity;
    pub(crate) fn receipt_identity(&self) -> &PathIdentity;
    pub(crate) fn revision(&self) -> u64;
    pub(crate) fn canonical_sha256(&self) -> [u8; 32];
}

impl<T: DurableReceipt> OwnedLockedReceipt<T> {
    pub(crate) fn current(&self) -> &T;
    pub(crate) fn advance(
        self,
        expected_revision: u64,
        next: &T,
        proved_facts: T::Proof,
    ) -> Result<
        (AtomicReceiptFile, ClaimedChildLock),
        OwnedLockedReceiptAdvanceError<T>,
    >;
}

impl<T: DurableReceipt> OwnedLockedReceiptAdvanceError<T> {
    pub(crate) fn stage(&self) -> ReceiptAdvanceStage;
    pub(crate) fn observed_after_failure(&self) -> &ReceiptAfterFailure<T>;
    pub(crate) fn source(&self) -> &TrustedFsError;
    pub(crate) fn into_recovery_discarding_proof(
        self,
    ) -> (
        AtomicReceiptFile,
        ClaimedChildLock,
        ReceiptAdvanceStage,
        ReceiptAfterFailure<T>,
        TrustedFsError,
    );
}

impl OwnedLockedReceiptVerifyError {
    pub(crate) fn into_parts(
        self,
    ) -> (AtomicReceiptFile, ClaimedChildLock, ReceiptAfterFailureBytes, TrustedFsError);
}

impl AtomicReceiptRemoveFailure {
    pub(crate) fn into_parts(
        self,
    ) -> (AtomicReceiptFile, ReceiptRemoveStage, ReceiptRemoveDiskTruth, TrustedFsError);
}

impl AtomicReceiptCreateFailure {
    pub(crate) fn stage(&self) -> ReceiptCreateStage;
    pub(crate) fn disk_truth(&self) -> &ReceiptCreateDiskTruth;
    pub(crate) fn source(&self) -> &TrustedFsError;
    pub(crate) fn abort(self) -> Result<(), AtomicReceiptCreateAbortFailure>;
}

impl AtomicReceiptCreateAbortFailure {
    pub(crate) fn stage(&self) -> ReceiptCreateStage;
    pub(crate) fn disk_truth(&self) -> &ReceiptCreateDiskTruth;
    pub(crate) fn source(&self) -> &TrustedFsError;
    pub(crate) fn retry(self) -> Result<(), AtomicReceiptCreateAbortFailure>;
}

pub(crate) trait DurableReceipt: Serialize + DeserializeOwned {
    type Proof;
    fn revision(&self) -> u64;
    fn validate_next(&self, next: &Self, proof: &Self::Proof) -> Result<(), TrustedFsError>;
}

impl AtomicReceiptFile {
    pub(crate) fn create_new_json<T: Serialize>(
        dir: Arc<TrustedDir>,
        name: RawUnixName,
        expected_lock_name: RawUnixName,
        value: &T,
    ) -> Result<Self, AtomicReceiptCreateFailure>;
    pub(crate) fn open_existing(
        dir: Arc<TrustedDir>,
        name: RawUnixName,
        expected_lock_name: RawUnixName,
    ) -> Result<Self, TrustedFsError>;
    pub(crate) fn identity(&self) -> &PathIdentity;
    pub(crate) fn read_json<T: DeserializeOwned>(&self) -> Result<T, TrustedFsError>;
    fn advance<T: DurableReceipt>(
        &mut self,
        expected_revision: u64,
        next: &T,
        proved_facts: T::Proof,
    ) -> Result<(), AtomicReceiptAdvanceError<T::Proof>>;
    pub(crate) fn verify_current_locked_synced<'lock, T: DurableReceipt>(
        &mut self,
        owning_lock: &'lock ClaimedChildLock,
    ) -> Result<(T, AdjacentReceiptFacts<'lock>), TrustedFsError>;
    pub(crate) fn into_verified_owned_locked<T: DurableReceipt>(
        self,
        owning_lock: ClaimedChildLock,
    ) -> Result<OwnedLockedReceipt<T>, OwnedLockedReceiptVerifyError>;
    pub(crate) fn remove_verified(self) -> Result<(), AtomicReceiptRemoveFailure>;
}
```

All single-child operations accept `RawUnixName` and use `openat`/`fstatat(AT_SYMLINK_NOFOLLOW)`/`renameatx_np(RENAME_EXCL)` or `renameat2(RENAME_NOREPLACE)`/`unlinkat`; no Tier-1 fallback uses `AT_FDCWD`, an unchecked `OsStr`, or a check-then-plain-rename. `enumerate_children` validates every returned component; malformed/reserved/stat-failed named entries become `Ok(InspectOnly)` and never a mutation capability, while a name-less `readdir`/stream I/O failure is `Err(TrustedFsError)` and terminates that scan as incomplete. No caller may reinterpret an incomplete stream as absence or uniqueness. `ClaimedChildLock` owns a verified regular mode-0600 file and its advisory lock plus the exact full parent identity captured by `TrustedDir::try_lock_regular_child`; no caller receives the directory fd or can construct/rebind that parent fact.

Canonical serialized output and every no-follow receipt read/re-read are capped by the protocol constant `MAX_RECEIPT_BYTES = 64 KiB`: serialization rejects an oversized value before creating a child; reads first reject an oversized regular-file snapshot and then use a bounded `take(MAX_RECEIPT_BYTES + 1)` loop so replacement or growth cannot trigger an unbounded allocation. `VerifiedReceiptSnapshot`, `ReceiptAfterFailureBytes`, and `ReceiptRemoveDiskTruth::Present` therefore never retain more than this bound; an oversized or concurrently growing receipt is `Unreadable`/inspect-only and never authorizes a transition.

`AtomicReceiptFile` writes canonical JSON through a create-new same-directory file, syncs it, and syncs the directory. `create_new_json` takes the trusted parent, validated receipt name, and protocol-exact lock name by value so ownership cannot disappear after the create syscall. Any failure returns `AtomicReceiptCreateFailure` with a monotonic stage, exact absent/present/unreadable reread truth, and `PartialAtomicReceiptFile` retaining the parent, both raw names, any created fd, and captured identity. Its consuming `abort`/`retry` may remove only that exact verified child and sync the parent; an unproved cleanup retains the same recovery object for the host's partial journal. Plans 3/4 must store that owning failure in their partial adjacent state rather than converting it to a naked `TrustedFsError` or a boolean “receipt exists.” `open_existing` likewise takes the trusted parent plus exact receipt/lock names, opens fd-relatively with no-follow, verifies a regular user-owned mode-0600 receipt, and captures the exact identity returned by `identity()`; it never creates or follows a symlink. Before `verify_current_locked_synced` or `into_verified_owned_locked` can mint/own facts, the substrate requires `ClaimedChildLock.parent_identity().same_object(receipt.dir.identity())`, exact equality with the receipt's stored `expected_lock_name`, and a fresh no-follow stat of that child name whose full snapshot equals `ClaimedChildLock.identity()`; directory ctime/mtime is intentionally not compared because creating/syncing its children changes those fields. A genuine live lock from bundle B or an unlinked/replaced lock inode therefore cannot authorize bundle A even if both locks share a conventional filename. Its write/remove methods are substrate-private. `advance` rereads the current durable value and requires the expected revision, revision `+1`, matching immutable header fields, a legal edge, and the receipt type's by-value `proved_facts`; there is no raw public replace. On every error it retains that same proof plus the exact failure stage and a bounded re-read observation inside one opaque owning error, rather than dropping authorization or claiming the prior revision still holds after publication. `into_verified_owned_locked` consumes both the receipt and its actual no-follow owning lock, reopens and retains the verified receipt snapshot, and returns one non-Clone/non-serde `OwnedLockedReceipt`; verification/deserialize/binding failure returns `OwnedLockedReceiptVerifyError` with those same two handles and observed bytes. Its consuming `advance` returns the receipt plus lock only after the transition succeeds. On pre-write validation/binding failure it returns `OwnedLockedReceiptAdvanceError` containing the same locked authority and proof with `ReceiptAfterFailure::Unchanged`; on any failure after a write may have begun it retains the same lock/receipt/proof, re-reads disk truth, and classifies it as `Advanced` or `Unreadable` for reconciliation/`Indeterminate`. The error exposes stage/truth/source only by borrow. Its sole consuming recovery method destroys the failed proof and stale verified snapshot internally, then returns the original `AtomicReceiptFile` plus actual `ClaimedChildLock` and stage/truth/source; it never returns `T::Proof`. The owning host must call `into_verified_owned_locked` again to obtain a fresh snapshot before any later transition, so an `Advanced`/`Unreadable` result is recoverable rather than a stale-snapshot dead end. `remove_verified` likewise consumes but never discards its receipt capability: a failure returns `AtomicReceiptRemoveFailure` with the receipt, exact pre-unlink/unlinked/parent-synced stage, and no-follow present/absent/unreadable truth. Other crate-private consuming `into_parts` methods return every capability they received, but no transition proof. No error path silently drops a lock/capability or fabricates a revision. Plans 3 and 4 wrap these primitives in owning Trash/EXDEV/Restore bundle APIs and expose only their typed, consuming transition methods.
Implement `Display`, `std::error::Error`, `From<io::Error>`, and `From<serde_json::Error>` for `TrustedFsError` directly; this plan adds no error-derive dependency.
For avoidance of doubt, the private encoded fields do not by themselves make deserialization safe. `RawUnixPath` and `RawUnixName` must not derive `Deserialize`. Each custom implementation decodes, re-encodes, requires a byte-for-byte canonical URL-safe-no-pad spelling, and invokes the same constructor validation used by live capture. `RawUnixPath::capture` is fallible and both live capture and deserialization accept only a non-empty absolute Unix path with no NUL and no lexical `.` or `..` component. Detect empty/interior components and `.`/`..` by scanning the original Unix bytes split on `/` **before** constructing `Path` or calling `Path::components()`, because those APIs normalize away evidence; reject rather than normalize, since lexical normalization across a symlink changes meaning. The leading root slash is the only permitted empty split element; the canonical root path `/` is the sole all-empty exception, while a trailing slash on any other path or a repeated interior slash is rejected as noncanonical. Protocols still perform their own trusted-root/parent and no-follow checks. Receipt input therefore cannot fabricate a relative or cwd-dependent durable path, a `RawUnixName` containing empty bytes, `.`, `..`, slash, or NUL, nor either raw type from padded, aliased, or malformed Base64.
`RawUnixPath` and `RawUnixName` use URL-safe Base64 without padding everywhere. Their encoded fields are private, so every value passes canonical decoding. `RawUnixName::capture` rejects empty names, `.`, `..`, slash bytes, and NUL bytes before any syscall; it is the only type accepted for one-component destination, internal, lock, or receipt names. `ObservedRawName` is a distinct non-capability emitted only by directory enumeration for an inspect-only entry. It has no `Serialize` or `Deserialize` implementation, preserves canonical raw bytes internally only for a manual raw-byte `Ord` and escaped display, and exposes neither encoded/raw bytes nor a conversion to `OsString` or `RawUnixName`. The sole bridge is the consuming, one-way `RawUnixName::into_observed`, used whenever a validated child is downgraded after receipt/header/stat validation fails; no reverse bridge exists. Thus an inspect-only observation cannot be serialized to recover bytes or fed back into mutation code. `AdjacentReceiptFacts<'lock>` is likewise opaque/non-Clone/non-serde: `verify_current_locked_synced` reopens the receipt no-follow, retains that verified `File` and its canonical bytes/identity/revision/hash in `VerifiedReceiptSnapshot`, re-fstats the already-open bundle directory through `TrustedDir::refresh_identity` after child and parent sync, verifies the lock parent/name/current inode binding, and returns facts borrowing the actual owning lock. It never treats the directory identity captured before child creation as the confirmed mirror identity. The facts do not borrow `AtomicReceiptFile`, so the fixed-control field may advance while the adjacent lock and snapshot remain live; they still cannot outlive or be produced without that lock. `MirrorConfirmationProof<'lock>` owns those facts, preserves the lock lifetime through fixed confirmation, and is consumed by the fixed transition. Dropping the lock before confirmation/advance is unrepresentable in safe Rust. `PathIdentity` includes dev/inode/kind plus ctime to make inode reuse observable, len/mtime for content drift, and mode/uid/gid for permission and trust checks. `same_object` compares only device, inode, and kind and is used immediately after an atomic claim rename because rename may change ctime. `same_snapshot` compares every captured field and is used for pre-claim drift, copy-source stability, trusted metadata, and the final private-payload snapshot. `stable_key` copies those fields into the explicitly ordered serialized `PathIdentityKey`; catalogs never sort debug/display text. After claim, capture a new tombstone snapshot; `unlink_verified_child` requires `same_snapshot` against that post-claim identity.

### `src/state_root.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallationId([u8; 16]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateRootErrorKind {
    Io,
    Encoding,
    Trust,
    Protocol,
    Sync,
    Unsupported,
    Corrupt,
}

#[derive(Debug)]
pub struct StateRootError {
    kind: StateRootErrorKind,
    escaped_detail: String,
}

impl StateRootError {
    pub fn kind(&self) -> StateRootErrorKind;
    pub fn escaped_detail(&self) -> &str;
    pub(crate) fn new(kind: StateRootErrorKind, escaped_detail: String) -> Self;
}

impl std::fmt::Display for StateRootError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl std::error::Error for StateRootError {}

pub struct StateRoot {
    root: TrustedDir,
    installation_id: InstallationId,
}

pub struct ControlBundle {
    pending_parent: Arc<TrustedDir>,
    bundle_name: RawUnixName,
    dir: TrustedDir,
    receipt: AtomicReceiptFile,
    identity: PathIdentity,
}

pub struct ClaimedControlBundle {
    bundle: ControlBundle,
    lock_file: File,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlProtocol { SourceClaimV1, TrashIngestV1, ExdevMoveV1, RestoreV1 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlHeader {
    pub schema: u32,
    pub operation_id: OperationId,
    pub item_id: ItemId,
    pub protocol: ControlProtocol,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MirrorIntent {
    pub adjacent_revision: u64,
    pub canonical_receipt_sha256: [u8; 32],
}

pub struct VerifiedTerminalControl {
    claimed: ClaimedControlBundle,
    installation_id: InstallationId,
    operation_id: OperationId,
    item_id: ItemId,
    current_revision: u64,
    bundle_identity: PathIdentity,
    receipt_snapshot: ControlReceiptSnapshot,
    expectation_sha256: [u8; 32],
    expectation: TerminalExpectation,
    pending_parent_synced: PendingParentSyncProof,
}

pub struct TerminalRemoveFailure {
    verified: VerifiedTerminalControl,
    stage: TerminalRemoveStage,
    disk_truth: TerminalControlDiskTruth,
    source: StateRootError,
}

pub struct VerifiedPreparedControlAbort {
    claimed: ClaimedControlBundle,
    installation_id: InstallationId,
    operation_id: OperationId,
    item_id: ItemId,
    control_identity: PathIdentityKey,
    current_revision: u64,
    receipt_snapshot: ControlReceiptSnapshot,
    envelope_sha256: [u8; 32],
    expectation_sha256: [u8; 32],
    expectation: PreparedControlAbortExpectation,
}

pub enum PreparedControlAbortExpectation {
    SourceClaimInitial,
    #[cfg(test)]
    TestMirror(TestPreparedAbortFacts),
    // Plan 3 adds Trash/Restore facts; Plan 4 adds EXDEV facts. Each host
    // validator proves its adjacent reservation is absent or unchanged.
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreparedControlAbortStage { Validate, BundleUnlink, ParentSync }

pub enum PreparedControlAbortDiskTruth {
    Present(PathIdentityKey),
    Absent,
    Unreadable { escaped_error: String },
}

pub struct PreparedControlAbortFailure {
    verified: VerifiedPreparedControlAbort,
    stage: PreparedControlAbortStage,
    disk_truth: PreparedControlAbortDiskTruth,
    source: StateRootError,
}

pub struct PreparedControlAbortVerifyFailure {
    claimed: ClaimedControlBundle,
    expectation: PreparedControlAbortExpectation,
    source: StateRootError,
}

pub struct TerminalVerifyFailure {
    claimed: ClaimedControlBundle,
    expectation: TerminalExpectation,
    source: StateRootError,
}

pub enum TerminalRemoveStage { PreUnlinkValidation, Unlinked, ParentSync }
pub enum TerminalControlDiskTruth { Present(ControlEnvelope), Absent, Unreadable { escaped_error: String } }

struct ControlReceiptSnapshot {
    file: File,
    identity: PathIdentity,
    envelope: ControlEnvelope,
    canonical_sha256: [u8; 32],
}

struct PendingParentSyncProof;

pub enum TerminalExpectation {
    SourceClaim(SourceClaimTerminalFacts),
    #[cfg(test)]
    TestMirror(TestTerminalFacts),
    // Plans 3/4 add exact Trash, Exdev, and Restore terminal expectations.
}

pub struct MirrorConfirmationProof<'lock> {
    installation_id: InstallationId,
    operation_id: OperationId,
    item_id: ItemId,
    current_revision: u64,
    expected_mirror: MirrorIntent,
    adjacent: AdjacentReceiptFacts<'lock>,
    next_state_sha256: [u8; 32],
}

pub struct FixedMirrorIntentProof<'lock> {
    installation_id: InstallationId,
    operation_id: OperationId,
    item_id: ItemId,
    control_identity: PathIdentityKey,
    current_revision: u64,
    exact_edge: HostMirrorEdge,
    current_adjacent: AdjacentReceiptFacts<'lock>,
    next_intent: MirrorIntent,
    next_state_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostMirrorEdge {
    #[cfg(test)]
    TestAdvance,
    // No production variant exists at the independently compiling Plan 2
    // boundary. Plan 3 adds exact Trash/Restore variants and Plan 4 adds EXDEV.
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlState {
    SourceClaim { source_claim: Option<SourceClaimState> },
    #[cfg(test)]
    TestMirror {
        host: TestMirrorControlState,
        mirror_intent: Option<MirrorIntent>,
    },
    // Plan 3 adds Trash/Restore { host, source_claim, mirror_intent }.
    // Plan 4 adds Exdev { host, source_claim, mirror_intent }.
}

#[cfg(test)]
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestMirrorControlState {
    phase: u8,
    confirmed_revision: Option<u64>,
    confirmed_sha256: Option<[u8; 32]>,
}

#[cfg(test)]
#[doc(hidden)]
pub struct TestTerminalFacts {
    // Private owner capabilities/sync witness, minted only by state_root tests.
    trusted_parent: Arc<TrustedDir>,
    bundle_name: RawUnixName,
}

#[cfg(test)]
#[doc(hidden)]
pub struct TestPreparedAbortFacts {
    // Private absence capability/sync witness, minted only by state_root tests.
    trusted_parent: Arc<TrustedDir>,
    bundle_name: RawUnixName,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlEnvelope {
    pub header: ControlHeader,
    pub revision: u64,
    pub state: ControlState,
}

pub enum ControlTransitionProof<'lock> {
    SourceClaim(SourceClaimProof<'lock>),
    MirrorIntentInstalled(FixedMirrorIntentProof<'lock>),
    MirrorConfirmed(MirrorConfirmationProof<'lock>),
}

pub enum ControlClaimAttempt {
    Claimed(ClaimedControlBundle),
    InUse(ControlBundle),
}

#[derive(Clone)]
pub struct ControlRecoveryRef {
    parent: Arc<TrustedDir>,
    bundle_name: RawUnixName,
    bundle_identity: PathIdentity,
    header: ControlHeader,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlRecoveryClaimDiskTruth {
    Present(PathIdentityKey),
    Absent,
    Unreadable,
}

pub struct ControlRecoveryClaimFailure {
    observed: ControlRecoveryRef,
    disk_truth: ControlRecoveryClaimDiskTruth,
    source: StateRootError,
}

pub struct ControlClaimFailure {
    bundle: ControlBundle,
    source: StateRootError,
}

pub struct PartialControlReservation {
    parent: Arc<TrustedDir>,
    bundle_name: RawUnixName,
    bundle_dir: Option<TrustedDir>,
    bundle_identity: Option<PathIdentity>,
    claim_lock: Option<ClaimedChildLock>,
    receipt: AtomicReceiptCreation,
    intended: ControlEnvelope,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlReservationStage {
    BundleCreate,
    LockCreate,
    ReceiptCreate,
    ReceiptSync,
    ParentSync,
    VerifyClaim,
}

pub enum ControlReservationDiskTruth {
    Absent,
    Partial {
        bundle: ControlReservationChildDiskTruth,
        lock: ControlReservationChildDiskTruth,
        receipt: ControlReservationChildDiskTruth,
    },
    Verified(ControlEnvelope),
    Unreadable { escaped_error: String },
}

pub enum ControlReservationChildDiskTruth {
    NotStarted,
    Absent,
    Present(PathIdentityKey),
    Unreadable { escaped_error: String },
}

pub struct ControlReservationFailure {
    recovery: PartialControlReservation,
    stage: ControlReservationStage,
    disk_truth: ControlReservationDiskTruth,
    source: StateRootError,
}

pub struct ControlReservationAbortFailure {
    recovery: PartialControlReservation,
    stage: ControlReservationStage,
    disk_truth: ControlReservationDiskTruth,
    source: StateRootError,
}

pub enum ReserveControlError {
    Collision(ItemId),
    NoEffect(StateRootError),
    Owned(ControlReservationFailure),
}

pub enum PendingControl {
    Verified { header: ControlHeader, bundle: ControlBundle },
    InspectOnly { bundle_name: ObservedRawName, escaped_error: String },
}

pub struct PendingControlStream {
    pending_dir: Arc<TrustedDir>,
    children: ChildEnumerator,
}

impl Iterator for PendingControlStream {
    type Item = Result<PendingControl, StateRootError>;
}

impl StateRoot {
    pub fn open_or_initialize() -> Result<Self, StateRootError>;
    #[doc(hidden)]
    pub fn open_or_initialize_with(config: StateRootConfig) -> Result<Self, StateRootError>;
    pub fn installation_id(&self) -> InstallationId;
    pub fn reserve_control(&self, initial: &ControlEnvelope) -> Result<ClaimedControlBundle, ReserveControlError>;
    pub(crate) fn claim_recovery_ref(
        &self,
        observed: ControlRecoveryRef,
    ) -> Result<ControlClaimAttempt, ControlRecoveryClaimFailure>;
    #[cfg(test)]
    pub(crate) fn open_control_bundle(&self, item_id: ItemId) -> Result<ControlBundle, StateRootError>;
    pub fn pending_controls(&self) -> Result<PendingControlStream, StateRootError>;
}

impl InstallationId {
    pub fn generate() -> Result<Self, StateRootError>;
    pub fn parse_exact(value: &[u8]) -> Result<Self, StateRootError>;
    pub fn to_lower_hex(self) -> String;
}

#[derive(Clone, Debug)]
pub struct StateRootConfig {
    pub root: PathBuf,
    pub ownership_start: PathBuf,
}

impl ControlBundle {
    pub fn read(&self) -> Result<ControlEnvelope, StateRootError>;
    pub fn identity(&self) -> &PathIdentity;
    #[cfg_attr(not(test), expect(dead_code, reason = "Plan 4 adds the first read-only recovery selector consumer"))]
    pub(crate) fn recovery_ref(&self) -> ControlRecoveryRef;
    pub fn try_claim(self) -> Result<ControlClaimAttempt, ControlClaimFailure>;
}

impl ControlClaimFailure {
    pub fn into_parts(self) -> (ControlBundle, StateRootError);
}

impl ClaimedControlBundle {
    pub fn read(&self) -> Result<ControlEnvelope, StateRootError>;
    pub fn identity(&self) -> &PathIdentity;
    pub(crate) fn recovery_ref(&self) -> ControlRecoveryRef;
    pub fn advance(
        &mut self,
        expected_revision: u64,
        next: ControlState,
        proved_facts: ControlTransitionProof<'_>,
    ) -> Result<ControlEnvelope, StateRootError>;
    #[cfg_attr(not(test), expect(dead_code, reason = "Plan 3 adds the first production mirror-confirmation host"))]
    pub(crate) fn confirm_mirror<'lock>(
        &self,
        adjacent: AdjacentReceiptFacts<'lock>,
        next_state: &ControlState,
    ) -> Result<MirrorConfirmationProof<'lock>, StateRootError>;
    #[cfg_attr(not(test), expect(dead_code, reason = "Plan 3 adds the first production post-initial mirror edge"))]
    pub(crate) fn verify_host_mirror_intent<'lock>(
        &self,
        current_adjacent: AdjacentReceiptFacts<'lock>,
        edge: HostMirrorEdge,
        next_state: &ControlState,
        next_intent: &MirrorIntent,
    ) -> Result<FixedMirrorIntentProof<'lock>, StateRootError>;
    pub(crate) fn verify_prepared_abort(
        self,
        expectation: PreparedControlAbortExpectation,
    ) -> Result<VerifiedPreparedControlAbort, PreparedControlAbortVerifyFailure>;
    pub fn verify_terminal(
        self,
        expectation: TerminalExpectation,
    ) -> Result<VerifiedTerminalControl, TerminalVerifyFailure>;
}

impl ControlRecoveryClaimFailure {
    pub(crate) fn disk_truth(&self) -> &ControlRecoveryClaimDiskTruth;
    pub(crate) fn source(&self) -> &StateRootError;
    pub(crate) fn into_parts(
        self,
    ) -> (ControlRecoveryRef, ControlRecoveryClaimDiskTruth, StateRootError);
}

impl ControlReservationFailure {
    pub fn stage(&self) -> ControlReservationStage;
    pub fn disk_truth(&self) -> &ControlReservationDiskTruth;
    pub fn source(&self) -> &StateRootError;
    pub fn abort(self) -> Result<(), ControlReservationAbortFailure>;
}

impl ControlReservationAbortFailure {
    pub fn stage(&self) -> ControlReservationStage;
    pub fn disk_truth(&self) -> &ControlReservationDiskTruth;
    pub fn source(&self) -> &StateRootError;
    pub fn retry(self) -> Result<(), ControlReservationAbortFailure>;
}

impl VerifiedTerminalControl {
    pub fn remove(self) -> Result<(), TerminalRemoveFailure>;
}

impl TerminalRemoveFailure {
    pub fn into_parts(
        self,
    ) -> (VerifiedTerminalControl, TerminalRemoveStage, TerminalControlDiskTruth, StateRootError);
}

impl VerifiedPreparedControlAbort {
    pub(crate) fn remove(self) -> Result<(), PreparedControlAbortFailure>;
}

impl PreparedControlAbortFailure {
    pub fn stage(&self) -> PreparedControlAbortStage;
    pub fn disk_truth(&self) -> &PreparedControlAbortDiskTruth;
    pub fn source(&self) -> &StateRootError;
    pub(crate) fn retry(self) -> Result<(), PreparedControlAbortFailure>;
}

impl PreparedControlAbortVerifyFailure {
    pub fn source(&self) -> &StateRootError;
    pub(crate) fn retry(self) -> Result<VerifiedPreparedControlAbort, PreparedControlAbortVerifyFailure>;
}

impl TerminalVerifyFailure {
    pub fn source(&self) -> &StateRootError;
    pub(crate) fn retry(self) -> Result<VerifiedTerminalControl, TerminalVerifyFailure>;
}
```

Tests use a library-visible, `#[doc(hidden)]` `StateRootConfig { root, ownership_start }` and a unit-test-only deterministic sync-fault hook. Production path detection is exactly the design's macOS and Linux rule. Every process that accepts the installation-ID winner validates it and personally syncs the state-root directory before destructive work is enabled.

`StateRootError` is the public, private-field, non-authorizing diagnostic used by every public state-root signature. Its crate-private constructors and `From<io::Error>`, `From<serde_json::Error>`, and `From<TrustedFsError>` conversions escape and truncate detail to 512 bytes; `Display`/`Error` expose no raw path, descriptor, receipt bytes, or capability. Capability-bearing failures never substitute this diagnostic for their owning recovery value.

`ControlBundle` is read-only. Every reserve and enumeration constructs it with the actual `pending_parent: Arc<TrustedDir>` and validated `bundle_name: RawUnixName` captured before any path display; those capabilities survive consuming claim and make `recovery_ref` expressible without parsing a diagnostic path or deriving a name from `ItemId`. The only `open_control_bundle(ItemId)` symbol is a `#[cfg(test)] pub(crate)` zero-call instrumentation seam; it does not exist in production. Both read-only and claimed bundles can mint the same Clone, nonauthorizing exact `ControlRecoveryRef`; it contains the trusted parent, validated raw name, full observed bundle identity, and header but no lock or mutation method. Later `StateRoot::claim_recovery_ref` must revalidate and consume that exact selector, so caching a cleanup action never falls back to an ItemId open. Claiming consumes a `ControlBundle` and returns `ClaimedControlBundle`, which owns both the bundle and its no-follow lock throughout read/advance/remove; safe Rust never needs an immutable borrow guard plus a simultaneous mutable bundle borrow. There is at most one fixed bundle for an `ItemId`. `ControlEnvelope` is a concrete typed outer state, not `ControlEnvelope<T>`: Plan 2 implements standalone `SourceClaim`, while Plans 3 and 4 add concrete Trash/EXDEV/Restore variants that embed `Option<SourceClaimState>` in the same envelope. `SourceClaim` never reserves another fixed bundle for a host item. Terminal verification is an owning typestate transition: `verify_terminal(self, expectation)` moves the original `ClaimedControlBundle`, its already-held lock, a newly opened no-follow receipt snapshot, and the complete concrete owner expectation/sync capabilities—not only its hash—into `VerifiedTerminalControl`. Verification error returns `TerminalVerifyFailure`, which still owns both the same claim and moved expectation and exposes only diagnostic borrow plus consuming `retry`; no tuple/accessor can detach either half. Only `VerifiedTerminalControl::remove(self)` can remove and sync the bundle; immediately before unlink it delegates a second owner-specific revalidation through the retained expectation so a required adjacent-present/adjacent-absent/source/destination fact cannot drift between verify and remove. Any pre-unlink, post-unlink, or parent-sync failure returns `TerminalRemoveFailure` containing that same owning typestate plus a stage and re-read present/absent/unreadable truth; it never silently drops the claim or promises presence after unlink. No second advisory lock is acquired, no fact token is detached from the original claim, and verification/remove replay is unrepresentable.

Successful preparation has a separate nonterminal removal path: an owner first consumes or releases every subordinate reservation, then calls the consuming `verify_prepared_abort(self, expectation)` to validate the exact initial/pre-effect host state and its owner-issued adjacent-absence/unchanged facts. Plan 2 validates `SourceClaimInitial`; Plans 3/4 extend the concrete expectation and delegate private host-field validation to their owner modules. Success moves the original claim/lock, a fresh no-follow receipt snapshot, installation/operation/item IDs, current revision, envelope hash, fixed-bundle identity, and the complete concrete owner expectation into the non-Clone/non-serde `VerifiedPreparedControlAbort`; the typestate does not retain only a hash. Verification error returns `PreparedControlAbortVerifyFailure`, which still owns both the original claim and moved concrete expectation and can only retry consumingly; no tuple/accessor detaches either half. Only `VerifiedPreparedControlAbort::remove(self)` may unlink and sync the successfully reserved fixed bundle, and immediately before unlink it delegates a second no-follow revalidation through the retained expectation's trusted parent/name/identity/sync capabilities. If a subordinate reservation was recreated or the observed restore bundle drifted, removal is refused while the fixed claim remains owned. A failure retains that same owning typestate, reports the exact unlink/sync stage plus no-follow present/absent/unreadable truth, and can be consumed only through `retry`; no detachable token can survive unlocking or be paired with a later re-claim. This API is distinct from partial `ControlReservationFailure::abort` and terminal `verify_terminal/remove`.

Every transition checks schema, outer protocol, operation/item IDs, expected current revision, revision `+1`, legal edge, and typed proof. Proof types are non-serializable opaque capability tokens with private fields and no public or generic boolean constructor. `SourceClaimProof` can be constructed only inside `source_claim.rs` after its fd-relative observation. For every post-initial host edge, an owning Plan 3/4 wrapper first verifies the currently confirmed adjacent receipt under its actual lock and calls `ClaimedControlBundle::verify_host_mirror_intent`; that factory matches the concrete current/next host states and `HostMirrorEdge`, rejects any unrelated field change, binds the live current-adjacent facts plus next revision/hash/state, and returns the only `FixedMirrorIntentProof` accepted by `ControlTransitionProof::MirrorIntentInstalled`. The initial fixed envelope may contain the initial intent before any adjacent object exists; no later transition has an unproved shortcut. After the adjacent advance, the wrapper obtains fresh `AdjacentReceiptFacts` and passes both those facts and the proposed confirmation `next_state` to `ClaimedControlBundle::confirm_mirror`. The factory rereads the current fixed `MirrorIntent`, compares verifier-computed revision/hash and recorded adjacent identity, and exhaustively delegates `(current host, next host, expected intent, adjacent facts)` to the concrete Plan 3/4 owner confirmation validator before binding the canonical next-state hash into `MirrorConfirmationProof`. Thus private host confirmation fields cannot change through an unchecked generic branch. Neither factory accepts caller booleans or a caller-asserted hash. Terminal authority exists only as `VerifiedTerminalControl`, produced by consuming the original claimed bundle and retaining its actual lock, verified receipt snapshot, exact terminal expectation, and sync facts through `remove`. A caller-provided JSON object, path string, identity value, hash, or set of `true` booleans can never become a proof token. Adjacent receipts are subordinate: a host consumes `FixedMirrorIntentProof` to record `MirrorIntent` in fixed control, then advances the adjacent receipt, then consumes `MirrorConfirmationProof` to confirm the exact adjacent revision, SHA-256, identity, and owner-approved host delta in fixed control. Source removal is forbidden until both match. Adjacent-ahead without a matching fixed intent, disagreement, or missing fixed control is inspect-only `Indeterminate`. All code acquires fixed control before any adjacent bundle lock.

Proof-token validity is also bundle-specific: each transition token privately binds installation ID, operation/item IDs, current revision, retained object identity, the exact edge, actual live lock/receipt snapshot, and required sync observations. `advance` takes tokens by value and compares every binding, so a genuine token issued for another bundle, revision, or edge is rejected and even a valid same-edge token cannot be reused after one attempt. Terminal verification/removal uses the consuming `ClaimedControlBundle -> VerifiedTerminalControl -> removed` typestate instead of a detachable token.

`ControlBundle::try_claim(self)` also has an owning error: `ControlClaimFailure` returns the original read-only bundle on lock/open failure. `InUse(ControlBundle)` remains a non-error observation. No caller repairs a failed claim by reopening a bare `ItemId`.

### `src/source_claim.rs`

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimAction {
    PublishNoReplace { destination_parent: RawUnixPath, destination_name: RawUnixName },
    PermanentDelete { directory: bool },
    TrashPublish { trash_parent: RawUnixPath, destination_name: RawUnixName },
    ExdevSourceCleanup,
    RestorePayloadCleanup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClaimErrorKind {
    InvalidState,
    InUse,
    SourceDrift,
    DestinationConflict,
    PermissionDenied,
    Unsupported,
    Filesystem,
    CleanupRequired,
    Indeterminate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimError {
    kind: ClaimErrorKind,
    escaped_detail: String,
}

impl ClaimError {
    pub fn kind(&self) -> ClaimErrorKind;
    pub fn escaped_detail(&self) -> &str;
    pub(crate) fn new(kind: ClaimErrorKind, escaped_detail: String) -> Self;
}

pub struct TrustedAdjacentRoot<'a> {
    control: &'a mut ClaimedControlBundle,
    root: Arc<TrustedDir>,
    bundle_name: RawUnixName,
    bundle_dir: TrustedDir,
    bundle_identity: PathIdentity,
    receipt: AtomicReceiptFile,
    receipt_lock: ClaimedChildLock,
    installation_id: InstallationId,
    operation_id: OperationId,
    item_id: ItemId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceClaimIntent {
    pub expected_identity: PathIdentity,
    pub source_parent: RawUnixPath,
    pub source_parent_identity: PathIdentity,
    pub source_name: RawUnixName,
    pub tombstone_parent: RawUnixPath,
    pub tombstone_parent_identity: PathIdentity,
    pub tombstone_name: RawUnixName,
    pub destination_parent_identity: Option<PathIdentity>,
    pub expected_destination_identity: Option<PathIdentity>,
    pub expected_payload_identity: Option<PathIdentity>,
    pub action: ClaimAction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceClaimFinalFacts {
    pub source_absent: bool,
    pub destination_identity: Option<PathIdentity>,
    pub private_payload_absent: bool,
    pub affected_parents_synced: bool,
}

#[doc(hidden)]
pub struct SourceClaimTerminalFacts {
    original_parent: TrustedDir,
    original_name: RawUnixName,
    destination_parent: Option<TrustedDir>,
    destination_name: Option<RawUnixName>,
    private_parent: TrustedDir,
    private_name: RawUnixName,
    expected_destination: Option<PathIdentity>,
    committed_revision: u64,
    affected_parents_synced: AffectedParentsSyncProof,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceClaimRecoveryFacts {
    pub original_path_identity: Option<PathIdentity>,
    pub private_payload_identity: Option<PathIdentity>,
    pub destination_identity: Option<PathIdentity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceClaimPhase {
    Intent,
    Claimed { post_claim_identity: PathIdentity },
    PublishOrCleanupIntent { claimed_identity: PathIdentity },
    Committed(SourceClaimFinalFacts),
    RestoreRequired(SourceClaimRecoveryFacts),
    CleanupRequired(SourceClaimRecoveryFacts),
    Indeterminate(SourceClaimRecoveryFacts),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceClaimState {
    pub revision: u64,
    pub intent: SourceClaimIntent,
    pub phase: SourceClaimPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceClaimEdge {
    IntentToClaimed,
    ClaimedToPublishOrCleanupIntent,
    PublishOrCleanupIntentToCommitted,
    AnyPreterminalToRestoreRequired,
    AnyPreterminalToCleanupRequired,
    AnyPreterminalToIndeterminate,
}

pub struct SourceClaimProof<'lock> {
    installation_id: InstallationId,
    operation_id: OperationId,
    item_id: ItemId,
    current_revision: u64,
    exact_edge: SourceClaimEdge,
    exact_action: ClaimAction,
    adjacent: AdjacentReceiptFacts<'lock>,
    observed_source: Option<PathIdentity>,
    observed_private_payload: Option<PathIdentity>,
    observed_destination: Option<PathIdentity>,
    affected_parents_synced: AffectedParentsSyncProof,
}

pub(crate) struct SourceClaimAdjacentAuthorization {
    installation_id: InstallationId,
    operation_id: OperationId,
    item_id: ItemId,
    current_revision: u64,
    exact_edge: SourceClaimEdge,
    next_state_sha256: [u8; 32],
    observed_source: Option<PathIdentity>,
    observed_private_payload: Option<PathIdentity>,
    observed_destination: Option<PathIdentity>,
    affected_parents_synced: AffectedParentsSyncProof,
}

struct AffectedParentsSyncProof;

pub struct SourceClaim<'a> {
    adjacent: TrustedAdjacentRoot<'a>,
    source_parent: TrustedDir,
    destination_parent: Option<TrustedDir>,
    private_parent: TrustedDir,
    payload_identity: PathIdentity,
    original_name: RawUnixName,
    action: ClaimAction,
}

pub struct SourceClaimRecovery<'a> {
    adjacent: SourceClaimAdjacentRecovery<'a>,
    source_parent: Option<TrustedDir>,
    destination_parent: Option<TrustedDir>,
    private_parent: Option<TrustedDir>,
    payload_identity: Option<PathIdentity>,
    original_name: Option<RawUnixName>,
    action: ClaimAction,
    observation: ClaimObservation,
}

enum SourceClaimAdjacentRecovery<'a> {
    Verified(TrustedAdjacentRoot<'a>),
    Partial(PartialSourceClaimAdjacent<'a>),
    NeedsReverify {
        control: &'a mut ClaimedControlBundle,
        root: Arc<TrustedDir>,
        bundle_name: RawUnixName,
        bundle_dir: TrustedDir,
        bundle_identity: PathIdentity,
        receipt: AtomicReceiptFile,
        owning_lock: ClaimedChildLock,
        stage: ReceiptAdvanceStage,
        observed: ReceiptAfterFailure<SourceClaimState>,
    },
}

struct PartialSourceClaimAdjacent<'a> {
    control: &'a mut ClaimedControlBundle,
    root: Option<Arc<TrustedDir>>,
    bundle_name: RawUnixName,
    bundle_dir: Option<TrustedDir>,
    bundle_identity: Option<PathIdentity>,
    claim_lock: Option<ClaimedChildLock>,
    receipt: AtomicReceiptCreation,
    stage: SourceClaimAdjacentOpenStage,
    disk_truth: SourceClaimAdjacentDiskTruth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceClaimAdjacentOpenStage {
    RootOpen,
    BundleCreateOrOpen,
    LockClaim,
    ReceiptCreateOrOpen,
    ReceiptVerify,
}

enum SourceClaimAdjacentChildDiskTruth {
    NotObserved,
    Absent,
    Present(PathIdentityKey),
    Unreadable { escaped_error: String },
}

enum SourceClaimAdjacentDiskTruth {
    NoObject,
    BundlePresent {
        bundle: PathIdentityKey,
        lock: SourceClaimAdjacentChildDiskTruth,
        receipt: SourceClaimAdjacentChildDiskTruth,
    },
    StreamUnreadable { escaped_error: String },
}

pub enum SourceClaimAcquireFailure<'a> {
    NoAdjacentEffect(ClaimError),
    AdjacentOwned { recovery: SourceClaimRecovery<'a>, source: ClaimError },
}

struct AdjacentRootOpenFailure<'a> {
    recovery: PartialSourceClaimAdjacent<'a>,
    source: ClaimError,
}

pub enum ClaimResult<'a> {
    Published { destination: PathIdentity },
    Deleted,
    RestoredNoEffect,
    RestoreRequired(SourceClaimRecovery<'a>),
    CleanupRequired(SourceClaimRecovery<'a>),
    Indeterminate(SourceClaimRecovery<'a>),
}

pub enum ClaimObservation {
    RestoredNoEffect,
    Published,
    Deleted,
    RestoreRequired,
    CleanupRequired,
    Indeterminate,
}

impl<'a> SourceClaim<'a> {
    pub fn acquire(
        state: &StateRoot,
        trusted_source_parent: &TrustedDir,
        control: &'a mut ClaimedControlBundle,
        plan: &ItemPlan,
        action: ClaimAction,
    ) -> Result<Self, SourceClaimAcquireFailure<'a>>;
    pub fn publish_no_replace(self) -> ClaimResult<'a>;
    pub fn delete_owned(self) -> ClaimResult<'a>;
    pub fn restore_no_replace(self) -> ClaimResult<'a>;
    pub fn reconcile_pending(
        state: &StateRoot,
        trusted_source_parent: &TrustedDir,
        control: &'a mut ClaimedControlBundle,
    ) -> Result<ClaimResult<'a>, SourceClaimAcquireFailure<'a>>;
    #[cfg_attr(not(test), expect(dead_code, reason = "Plan 4 adds the only retained-source private-cleanup caller"))]
    pub(crate) fn reconcile_private_cleanup(
        state: &StateRoot,
        trusted_tombstone_parent: &TrustedDir,
        control: &'a mut ClaimedControlBundle,
    ) -> Result<ClaimResult<'a>, SourceClaimAcquireFailure<'a>>;
}

impl<'a> SourceClaimRecovery<'a> {
    pub fn observation(&self) -> ClaimObservation;
    pub fn reconcile(self) -> ClaimResult<'a>;
    #[cfg_attr(not(test), expect(dead_code, reason = "Plan 4 adds the only retained-source private-cleanup caller"))]
    pub(crate) fn reconcile_private_cleanup(self) -> ClaimResult<'a>;
    #[cfg_attr(not(test), expect(dead_code, reason = "Plan 4 adds the bounded nonmutating release caller"))]
    pub(crate) fn release_classified(self) -> ClaimObservation;
}

impl DurableReceipt for SourceClaimState {
    type Proof = SourceClaimAdjacentAuthorization;
    fn revision(&self) -> u64;
    fn validate_next(
        &self,
        next: &Self,
        proof: &Self::Proof,
    ) -> Result<(), TrustedFsError>;
}

impl<'a> SourceClaimAcquireFailure<'a> {
    pub fn source(&self) -> &ClaimError;
    pub fn reconcile_owned(self) -> Result<ClaimResult<'a>, ClaimError>;
}

pub(crate) fn validate_source_claim_terminal(
    current: &SourceClaimState,
    facts: &SourceClaimTerminalFacts,
) -> Result<(), ClaimError>;

impl<'a> AdjacentRootOpenFailure<'a> {
    fn into_recovery(self, action: ClaimAction) -> SourceClaimRecovery<'a>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdjacentArea { SourceClaims, Staging, Claims, Quarantine }

impl<'a> TrustedAdjacentRoot<'a> {
    fn open_or_initialize(
        state: &StateRoot,
        parent: &TrustedDir,
        control: &'a mut ClaimedControlBundle,
    ) -> Result<Self, AdjacentRootOpenFailure<'a>>;
    pub fn reserve_bound_item_dir(&mut self, area: AdjacentArea) -> Result<TrustedDir, ClaimError>;
    pub fn open_bound_item_dir(&mut self, area: AdjacentArea) -> Result<TrustedDir, ClaimError>;
    pub fn move_bound_item_no_replace(&mut self, from: AdjacentArea, to: AdjacentArea) -> Result<(), ClaimError>;
    pub fn sync(&self) -> Result<(), ClaimError>;
}
```

The implementation follows design steps 1-7 verbatim. The preparation phase already owns the one fixed claimed control before `SourceClaim::acquire`; immediately before the call, execution no-follow reopens the source parent from the immutable `ItemPlan`, verifies its prepared full identity, and passes that `TrustedDir` plus `MutationContext.state_root`. Acquisition calls `TrustedAdjacentRoot::open_or_initialize(state, trusted_source_parent, control)` and moves the exclusive fixed-control borrow into the result; its private operation/item IDs are read from the claimed header. This exact signature supplies every capability required to construct the adjacent root and never stores it self-referentially beside its control. Adjacent methods have no caller-supplied `ItemId`, so safe Rust cannot reserve or move another item's directory after dropping or swapping the fixed claim. `SourceClaimIntent` is an immutable recovery record, not a transient preflight view: every phase preserves the original source/tombstone/destination raw selectors, their trusted parent identities, the pre-claim source identity, action, and expected destination/payload identities. Only `phase` and its freshly verified observations advance. Restart recovery therefore reopens those persisted selectors fd-relatively and never consults cwd, a display path, or a bare `ItemId`.

`TrustedAdjacentRoot` owns the actual bundle directory, bundle name/full identity, `AtomicReceiptFile`, and its actual claim lock for its complete lifetime. At the start of each transition, its owner temporarily moves those two fields out, calls `verify_current_locked_synced` to mint lifetime-bound facts while that exact lock is live, consumes the fixed intent proof, then consumes the same receipt+lock through `into_verified_owned_locked` and `OwnedLockedReceipt::advance`. A successful adjacent advance returns those exact raw handles to the adjacent root; confirmation obtains another fresh locked/synced snapshot before advancing fixed state. No accessor copies or detaches the retained `File` snapshot, and no long-lived structure must copy a non-Clone `File`. Every consuming/error path remains owning: adjacent-root open failure returns the same fixed-control borrow plus every opened partial handle; acquire after adjacent effect returns `SourceClaimRecovery`; consuming publish/delete/restore failure carries that recovery object in `ClaimResult`. `reconcile_pending` also returns the owning `ClaimResult<'a>`, never an observation-only success that could discard `RestoreRequired`/`CleanupRequired`/`Indeterminate` authority. `SourceClaimRecovery::reconcile` and `SourceClaimAcquireFailure::reconcile_owned` consume the retained actual handles; host modules never inspect private fields or call `reconcile_pending` with a reconstructed ID. Thus the actual fixed claim, adjacent handles/receipt lock, and observed payload remain live through reconciliation rather than being recreated from an ItemId. `SourceClaim` advances only the host envelope's embedded `SourceClaimState` through that owning adjacent root. Standalone same-filesystem operations use `ControlState::SourceClaim`; Plan 3 Trash/Restore and Plan 4 EXDEV pass their own host state with the same nested slot. No caller receives a pathname it can unlink directly. `SourceClaim` is the only Plan 2 path to same-filesystem move/rename/trash source removal and permanent deletion. The serialized `SourceClaimFinalFacts` is state, never terminal authority. Only source_claim.rs can mint non-Clone/non-serde `SourceClaimTerminalFacts` from actual trusted parents/raw names, no-follow presence/absence checks, destination identity, and sync witnesses; `state_root.rs` delegates both terminal verification and the immediate pre-unlink revalidation to `validate_source_claim_terminal` while retaining those facts inside `VerifiedTerminalControl`. Task 10 later adds `recover_source_claim_startup`, after the mutation notice/context types exist; it consumes an already claimed fixed control and routes through this same owner-private reconcile core without constructing an `ItemPlan`.

`reconcile_private_cleanup` is the only EXDEV retained-source cleanup entry. It accepts a no-follow trusted tombstone parent—not the original source parent—and only when the persisted action is `ExdevSourceCleanup` in an already-claimed/delete-capable phase with the exact private parent/name/identity. It never opens, stats, restores, renames, or unlinks the original user path and cannot take a caller-supplied source selector. Its recovery counterpart preserves that restriction. A defensive `RestoreRequired` is never executed in this mode. After a bounded retry, or immediately for `RestoreRequired`, the host consumes `release_classified`: this infallible, non-I/O exit closes/releases every adjacent receipt/lock/root handle, ends the mutable fixed-control borrow, and returns only the conservative `ClaimObservation`; the same fixed durable state remains discoverable for a later startup/user-directed recovery. It cannot publish/delete/restore, advance either receipt, or manufacture terminal success. Therefore a persistent permission/I/O fault cannot hold the single mutation worker forever, and no path drops an owning recovery merely to form an `ItemOutcome`.

### `src/mutation.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SubmissionId(u64);

pub(crate) struct SubmissionIdAllocator {
    next: u64,
}

impl SubmissionId {
    pub fn get(self) -> u64;
    #[doc(hidden)]
    pub fn from_nonzero_for_test(value: u64) -> Result<Self, InvalidSubmissionId>;
}

impl SubmissionIdAllocator {
    pub(crate) fn new() -> Self;
    pub(crate) fn allocate(&mut self) -> Result<SubmissionId, SubmissionIdExhausted>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SubmissionIdExhausted;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidSubmissionId;

#[derive(Clone, Debug)]
pub struct ItemDraft {
    pub source: RawUnixPath,
    pub requested_destination_parent: Option<RawUnixPath>,
    pub requested_destination_name: Option<RawUnixName>,
    pub recursive_scope: bool,
}

#[derive(Clone, Debug)]
pub struct PathMutationIntent {
    pub kind: OperationKind,
    pub conflict_policy: ConflictPolicy,
    pub protected_work_root: RawUnixPath,
    pub items: Vec<ItemDraft>,
}

#[derive(Clone, Debug)]
pub enum MutationIntentBody {
    Paths(PathMutationIntent),
    // Plan 3 adds Restore(RestoreIntent); Plan 4 adds CleanupRetainedSource.
}

#[derive(Clone, Debug)]
pub struct MutationIntent {
    pub submission_id: SubmissionId,
    pub body: MutationIntentBody,
}

impl MutationIntent {
    pub fn paths(submission_id: SubmissionId, paths: PathMutationIntent) -> Self;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FenceRoot { pub path: RawUnixPath, pub recursive: bool }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MutationFenceSpec { pub roots: Vec<FenceRoot> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreparationFailureKind {
    InvalidIntent,
    UnsupportedObject,
    SourceDrift,
    DestinationDrift,
    ReservationCollision,
    PermissionDenied,
    OutOfSpace,
    Filesystem,
    CleanupRequired,
    Indeterminate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparationError {
    kind: PreparationFailureKind,
    escaped_detail: String,
}

impl PreparationError {
    pub fn kind(&self) -> PreparationFailureKind;
    pub fn escaped_detail(&self) -> &str;
    pub(crate) fn new(kind: PreparationFailureKind, escaped_detail: String) -> Self;
}

#[derive(Clone, Debug)]
pub struct PreparedNotice {
    pub submission_id: SubmissionId,
    pub request: Arc<OperationRequest>,
    pub final_fences: MutationFenceSpec,
}

pub struct OwnedStagingReservation {
    parent: Arc<TrustedDir>,
    name: RawUnixName,
    identity: PathIdentity,
}

pub(crate) struct OwnedStagingRecoveryRef {
    parent: Arc<TrustedDir>,
    name: RawUnixName,
    identity: PathIdentity,
}

pub enum PreparedReservation {
    None,
    FixedControl(ClaimedControlBundle),
    OwnedStaging(OwnedStagingReservation),
    // Plan 3 adds concrete Trash/Restore reservations; Plan 4 adds EXDEV.
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreparedAbortOutcome {
    ReleasedNoEffect,
    CleanupRequired,
    Indeterminate,
}

pub(crate) enum PreparedRecoverySeed {
    None,
    OwnedStaging(OwnedStagingRecoveryRef),
    Fixed(ControlRecoveryRef),
}

pub(crate) enum ItemExecutionObservation {
    // Plan 4 adds the concrete Exdev(CleanupAvailable) variant. This closed
    // enum is not a generic payload or extension registry.
}

pub(crate) struct ItemExecutionResult {
    outcome: ItemOutcome,
    observation: Option<ItemExecutionObservation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartupRecoveryDisposition {
    RecoveredNoEffect,
    Completed,
    CleanupRequired,
    Indeterminate,
    InspectOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartupRecoveryNotice {
    pub item_id: Option<ItemId>,
    pub protocol: Option<ControlProtocol>,
    pub disposition: StartupRecoveryDisposition,
    pub escaped_detail: Option<String>,
}

impl ItemExecutionResult {
    pub(crate) fn outcome_only(outcome: ItemOutcome) -> Self;
    pub(crate) fn into_parts(self) -> (ItemOutcome, Option<ItemExecutionObservation>);
}

impl PreparedReservation {
    pub(crate) fn recovery_seed(&self) -> PreparedRecoverySeed;
}

pub(crate) enum PlainPreparationOwnership {
    Draft(ItemDraft),
    FixedReservationOwned {
        draft: ItemDraft,
        failure: ControlReservationFailure,
    },
    Reservations {
        draft: ItemDraft,
        reservations: Vec<PreparedReservation>,
    },
}

pub(crate) struct PlainPreparationFailure {
    ownership: PlainPreparationOwnership,
    source: PreparationError,
}

pub struct ItemPreparationFailure {
    inner: ItemPreparationFailureInner,
}

pub(crate) enum ItemPreparationFailureInner {
    Plain(PlainPreparationFailure),
    // Plan 3 adds Trash(PrepareTrashError) and Restore(PrepareRestoreError).
    // Plan 4 adds Exdev(PrepareExdevError) and ExdevCleanup(PrepareExdevCleanupError).
}

impl PlainPreparationFailure {
    pub(crate) fn source(&self) -> &PreparationError;
    pub(crate) fn into_parts(self) -> (PlainPreparationOwnership, PreparationError);
}

impl ItemPreparationFailure {
    pub fn kind(&self) -> PreparationFailureKind;
    pub fn escaped_detail(&self) -> &str;
    pub(crate) fn from_inner(inner: ItemPreparationFailureInner) -> Self;
    pub(crate) fn into_inner(self) -> ItemPreparationFailureInner;
}

impl std::fmt::Debug for ItemPreparationFailure {
    // Render only kind plus bounded escaped detail; never capabilities.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl std::fmt::Display for ItemPreparationFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl std::error::Error for ItemPreparationFailure {}

struct PreparedMutation {
    request: Arc<OperationRequest>,
    reservations: Vec<PreparedReservation>,
    final_fences: MutationFenceSpec,
}

#[derive(Clone)]
pub struct CancelToken(Arc<AtomicBool>);

#[derive(Clone)]
pub struct MutationContext {
    pub state_root: Arc<StateRoot>,
    pub ownership_policy: Arc<OwnershipPolicy>,
}

#[derive(Clone)]
pub struct ProgressSink(Arc<Mutex<Option<OperationEvent>>>);

impl CancelToken {
    pub fn new() -> Self;
    pub fn request(&self) -> bool;
    pub fn is_requested(&self) -> bool;
}

impl ProgressSink {
    pub fn new() -> Self;
    pub fn publish(&self, progress: OperationEvent);
    pub fn take(&self) -> Option<OperationEvent>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MutationWorkerErrorKind {
    ChannelClosed,
    ObserverUnavailable,
    Protocol,
    Shutdown,
    ThreadSpawn,
    Internal,
}

#[derive(Debug)]
pub struct MutationWorkerError {
    kind: MutationWorkerErrorKind,
    escaped_detail: String,
}

impl MutationWorkerError {
    pub fn kind(&self) -> MutationWorkerErrorKind;
    pub fn escaped_detail(&self) -> &str;
    pub(crate) fn new(kind: MutationWorkerErrorKind, escaped_detail: String) -> Self;
}

impl std::fmt::Display for MutationWorkerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl std::error::Error for MutationWorkerError {}

pub struct MutationWorker {
    context: MutationContext,
    command_tx: SyncSender<MutationCommand>,
    fence_ack_tx: SyncSender<FenceInstalled>,
    observation_rx: Receiver<WorkerObservation>,
    progress: ProgressSink,
    active: Option<ActiveMutation>,
    fallback_events: VecDeque<OperationEvent>,
    join: Option<JoinHandle<()>>,
}

pub enum StartMutationError { Busy(SubmissionId), WorkerUnavailable }
pub enum FenceAckError { NotPrepared, WrongSubmission, WouldBlock, WorkerUnavailable }
pub enum CancelAck { Requested, AlreadyRequested, NotActive }
struct FenceInstalled { submission_id: SubmissionId }
pub enum FenceAckDecision {
    NotPrepared,
    Installed(SubmissionId),
}
pub enum WorkerObservation {
    StartupRecovery(StartupRecoveryNotice),
    Prepared(PreparedNotice),
    Event(OperationEvent),
    ObserverStarted(SubmissionId),
}

impl MutationWorker {
    pub fn start(context: MutationContext) -> Result<Self, MutationWorkerError>;
    pub fn try_start(&mut self, intent: MutationIntent) -> Result<(), StartMutationError>;
    pub fn acknowledge_fences(&self, submission_id: SubmissionId) -> Result<(), FenceAckError>;
    pub fn request_cancel(&self, submission_id: SubmissionId) -> CancelAck;
    pub fn take_progress(&self) -> Option<OperationEvent>;
    pub fn try_recv(&mut self) -> Result<Option<WorkerObservation>, MutationWorkerError>;
    pub fn active_submission(&self) -> Option<SubmissionId>;
    pub fn active_operation(&self) -> Option<OperationId>;
    pub fn shutdown_after_active(
        self,
        on_observation: impl FnMut(WorkerObservation) -> FenceAckDecision,
    ) -> Result<(), MutationWorkerError>;
}
```

`MutationIntent` contains no `OperationId`, `ItemId`, captured final identity, or persistent marker name. Plan 2 initially accepts only `MutationIntentBody::Paths`; Plan 3 adds its typed restore body and Plan 4 adds the typed retained-source cleanup body without changing `try_start(MutationIntent)`. App/CLI own a crate-private `SubmissionIdAllocator` and can obtain IDs only through its checked monotonic `allocate`; they cannot construct the private tuple field, reuse zero, or wrap at exhaustion. External integration tests use only the hidden checked `SubmissionId::from_nonzero_for_test`, which rejects zero and is forbidden in production call sites. Coordinators perform bounded syntax/count/confirmation checks and install conservative provisional fences from the raw intent before `try_start`. The single worker performs filesystem preflight, captures final identities, generates final IDs, and reserves every applicable fixed/adjacent marker create-new. On collision it never opens the pre-existing candidate and regenerates only that candidate ID. The request becomes immutable only after all reservations and final fence roots are known.

`PreparationError` is the stable, non-authorizing cause shared with Plans 3 and 4. Its fields are private, its escaped detail is bounded to 512 bytes at construction, and it contains no path capability, file descriptor, lock, receipt, reservation, or recoverable ownership. `PreparationFailureKind` is exhaustive for this release. A host preparer must wrap this cause in its own owning error whenever it has acquired any capability or performed any durable effect; returning a naked `PreparationError` is legal only while the unchanged draft intent is still the complete authority. Public `ItemPreparationFailure` is a private-field diagnostic wrapper, so `prepare_item` has a valid public result type while integration callers can inspect only kind/bounded detail. Its crate-private `ItemPreparationFailureInner` is the worker's exact closed, non-generic dispatch error: Plan 2 begins with `Plain`, Plan 3 adds concrete `Trash`/`Restore`, and Plan 4 adds concrete `Exdev`/`ExdevCleanup`. The worker passes the whole wrapper to `mutation_ops::recover_preparation_failure`; that closed dispatcher consumes `into_inner` and calls the matching owner-module recovery routine, so cancellation, fallback observation, and cleanup never reconstruct a capability from an ID. Plans 3 and 4 may inspect `kind()` and render `escaped_detail()`, but cannot construct or use the value as a transition proof.

`MutationWorkerError` is likewise a public, private-field, bounded diagnostic so every public worker signature is valid under strict `private_interfaces`/`-D warnings`. It never contains an active intent, prepared reservation, recovery seed, channel endpoint, raw path, or filesystem handle; those remain in the worker/observer journal. Its crate-private constructor truncates escaped detail to 512 bytes, and `Display`/`Error` cannot authorize recovery.

`StartupRecoveryNotice` is non-authorizing session UI truth. Its optional IDs/protocol come only from a verified fixed header; inspect-only children have neither. Its escaped detail is capped at 512 bytes and contains no raw path, receipt bytes, file descriptor, or recovery capability. The closed `mutation_ops::recover_startup_control(context, claimed) -> StartupRecoveryNotice` match dispatches solely on the typed outer `ControlState`: Plan 2 calls `recover_source_claim_startup`, Plan 3 adds Trash/Restore owner entries, and Plan 4 adds EXDEV. It never manufactures an `ItemPlan`, `OperationRequest`, source path, or working root and never calls `open_control_bundle(ItemId)`.

The worker keeps the private `PreparedMutation` above and emits a non-droppable `PreparedNotice`; reservation capabilities never cross to `App`. The UI atomically replaces provisional fences with the final fence set and calls `acknowledge_fences`; `WouldBlock` keeps those final fences installed and retries from a later event-loop turn, while a stale/wrong acknowledgement never unlocks execution. Only a matching accepted acknowledgement may let the worker emit `Started` and enter a publish/rename/unlink path. Cancellation, missing acknowledgement, panic, command disconnect, or shutdown before acknowledgement has zero user-visible effect. The worker moves every still-opaque `PreparedReservation` through `mutation_ops::abort_prepared_item(context, plan, reservation)`; that closed dispatcher calls only the owning module's consuming abort routine. Each routine removes only its verified internal reservations in reverse order and syncs their parents, except Plan 4's cleanup-only reservation, which releases the unchanged original transaction without unlinking it. `PreparedAbortOutcome::ReleasedNoEffect` permits `FailedNoEffect`; `CleanupRequired`/`Indeterminate` means durable residue was retained/classified and forbids a no-effect claim. Plan 3 adds concrete Trash/Restore intent and reservation variants; Plan 4 adds the concrete EXDEV preparation and Recovery handoff. All use this same worker state machine and acknowledgement.

New-operation commands, fence acknowledgements, and non-progress `WorkerObservation`s use three separate bounded `sync_channel(1)` channels; `Prepared`, every concrete item-side observation, every item outcome, and the single `Finished` are non-droppable. `try_start` and `acknowledge_fences` use their respective nonblocking senders; logical shutdown closes/rejects new starts but retains the dedicated fence-ack sender until every prepared submission is terminal. Cancellation sets the active submission's retained `CancelToken` directly, so a full command or observation slot cannot delay the visible cancellation acknowledgement. Progress uses one replaceable slot. Each preparation and item boundary runs under `catch_unwind`. Both ordinary execution and `recover_execution_panic` return `ItemExecutionResult`; the worker consumes it and, when its closed observation is present, sends that observation exactly once before the matching terminal `OperationEvent`, then sends the outcome, and only later the unique `Finished`. A non-observation outcome cannot synthesize one. An outer disconnect starts one concrete observer thread using the retained intent, preparation journal, immutable request when available, fixed receipts, and read-only filesystem facts, then uses the same ordered result emitter to publish one synthesized final report. During `shutdown_after_active`, each drained `Prepared` is first delivered to `on_observation`; only `FenceAckDecision::Installed` with the exact submission authorizes the method to send the dedicated acknowledgement. `NotPrepared` or a mismatched ID keeps mutation effects locked, drives verified reservation cleanup, and still yields the terminal no-effect/cleanup report. The method drains progress plus non-progress observations until the unique `Finished`, holds no coordinator mutex, then drops receivers and joins worker/observer. It never joins first while a sender may be blocked on the capacity-one channel.

## Task-by-task TDD plan

### Task 1: Add raw-path, identity, and fd-relative Tier-1 primitives

**Files:**
- Create: `src/trusted_fs.rs`
- Create: `tests/trusted_fs.rs`
- Modify: `src/lib.rs:1-9`

- [ ] **Step 1: Write red codec and no-follow tests**

Add public-surface integration tests named `raw_unix_path_round_trips_non_utf8`, `raw_unix_path_capture_and_deserialize_reject_relative_empty_nul_dot_and_dotdot`, `raw_unix_path_checks_original_bytes_before_component_normalization`, `raw_unix_name_rejects_empty_dot_dotdot_slash_and_nul`, `observed_raw_name_has_no_serde_or_reverse_conversion`, `validated_name_downgrades_consumingly_to_observed`, `every_child_api_rejects_unvalidated_name_before_syscall`, `child_escape_matrix_preserves_outside_sentinel`, `enumeration_returns_validated_or_inspect_only_without_raw_fd`, `claim_lock_is_no_follow_owning_and_exclusive`, `failed_owned_receipt_advance_returns_same_locked_authority_without_revision_change`, `post_write_owned_receipt_advance_failure_retains_lock_and_rereads_truth`, `owned_receipt_remove_failure_retains_capability_stage_and_reread_truth`, `path_identity_same_snapshot_detects_ctime_mode_and_owner_drift`, `path_identity_same_object_rejects_inode_swap`, `path_identity_key_orders_every_field_without_display_text`, `trusted_dir_rejects_symlink_component`, `rename_child_no_replace_never_overwrites`, and `unlink_verified_child_rejects_identity_swap` in `tests/trusted_fs.rs`. Put the runtime behavior portions of the two owned-advance cases, the owning-remove case, `atomic_receipt_advance_rejects_wrong_revision_edge_or_facts`, and exact `trusted_fs::tests::adjacent_receipt_facts_require_live_bound_lock_and_actual_synced_bytes` in `trusted_fs.rs`'s `#[cfg(test)]` module so the crate-private raw receipt primitive never becomes an integration-test API. The public owned-error cases remain compile/API contract fixtures. Private tests prove the retained no-follow snapshot bytes, identity, revision/hash, receipt sync, and parent sync came from the actual locked receipt; `adjacent_receipt_facts_require_live_bound_lock_and_actual_synced_bytes` also acquires a genuine lock from bundle B and proves it cannot verify or own bundle A's receipt despite the same lock filename. The same gate injects receipt-create failures after create/write/file-sync/parent-sync and proves `AtomicReceiptCreation::CreateFailed` retains the exact partial fd/identity/truth until consuming abort/retry. The pre-write failure case must retain the same locked authority and opaque proof with the original revision, while its recovery method consumes rather than returns the proof; the post-publish or post-sync case must retain the same lock/authority and re-read `Advanced` or `Unreadable` truth, never assert that the old revision survived. The removal test injects before unlink, after unlink, and at parent sync and requires the exact receipt capability plus present/absent/unreadable truth on every error.

The escape matrix passes empty, `.`, `..`, `../sentinel`, `a/b`, an embedded NUL, and non-UTF-8 bytes through every child open/create/stat/lock/rename/unlink and receipt-create entry. Put a sentinel outside the trusted directory and assert every invalid name is rejected before a syscall and the sentinel's identity/content never changes. Enumeration must skip `.`/`..`, return valid raw names as capabilities, surface malformed/stat-failed observations with `ObservedRawName`, and never expose a directory fd. Add `inspect_only_observed_name_has_no_mutation_conversion` as a compile/API source-contract test proving no public conversion to `RawUnixName`/`OsString` and no child API accepts it. Use two independent processes to prove only one owner obtains the same verified `claim.lock`.

The same private gate also supplies a 64-KiB boundary receipt, a 64-KiB-plus-one receipt, and a file that grows after its first stat. Only the exact boundary is accepted; both oversized cases fail without allocating or retaining more than `MAX_RECEIPT_BYTES + 1` scratch bytes and cannot mint `AdjacentReceiptFacts`.

Also add unit test `atomic_receipt_open_existing_is_nofollow_owned_and_identity_bound` in `trusted_fs.rs`. It creates a valid receipt, reopens it through the crate-private capability factory, checks the captured stable identity, and proves symlink, directory, wrong-owner, wrong-mode, replaced-after-observation, and invalid-name inputs fail before mutation. It is covered by the locked `cargo test --lib` gate rather than exposing the substrate merely to make an integration test compile.

Also add exact `child_enumerator_midstream_io_error_is_not_end_of_stream_or_absence`. Inject an error after several valid entries and assert the iterator yields `Err`, does not yield `None`, and every catalog/uniqueness consumer must fail the operation rather than treat the unseen suffix as absent.

Also add `raw_unix_path_deserialize_rejects_noncanonical_base64`, `raw_unix_name_deserialize_revalidates_every_forbidden_component`, and `receipt_deserialize_cannot_fabricate_child_capability`. These tests must exercise actual `serde_json::from_slice`, not merely the public constructors. The observed-name source-contract test must also try to serialize a `ChildObservation::InspectOnly` and fail at compile/API level because `ObservedRawName` implements neither serde trait.

- [ ] **Step 2: Run the red tests**

Run:

```bash
python3 scripts/run_exact_test.py --test trusted_fs --name raw_unix_path_capture_and_deserialize_reject_relative_empty_nul_dot_and_dotdot
python3 scripts/run_exact_test.py --lib --name trusted_fs::tests::adjacent_receipt_facts_require_live_bound_lock_and_actual_synced_bytes
python3 scripts/run_exact_test.py --test trusted_fs --name failed_owned_receipt_advance_returns_same_locked_authority_without_revision_change
python3 scripts/run_exact_test.py --test trusted_fs --name post_write_owned_receipt_advance_failure_retains_lock_and_rereads_truth
python3 scripts/run_exact_test.py --test trusted_fs --name owned_receipt_remove_failure_retains_capability_stage_and_reread_truth
python3 scripts/run_exact_test.py --test trusted_fs --name child_enumerator_midstream_io_error_is_not_end_of_stream_or_absence
cargo test --locked --test trusted_fs -- --nocapture
```

Expected: FAIL with unresolved import `tersh::trusted_fs`.

- [ ] **Step 3: Add the locked `trusted_fs.rs` interfaces**

Use the signatures above. Encode raw paths with RFC 4648 URL-safe Base64 without padding and reject non-canonical encodings. Convert a `RawUnixName` to `OsString` only inside `trusted_fs.rs` immediately before the fd-relative syscall. Capture identity with `fstatat(..., AT_SYMLINK_NOFOLLOW)` and implement `PathIdentityKey` as the declared field-by-field total-order projection. Make unsupported no-replace syscalls return `TrustedFsError::UnsupportedNoReplace`; never call plain `rename` as fallback. Implement streaming child enumeration and an owning verified lock capability; do not expose `AsRawFd` outside this module. Implement `DurableReceipt::validate_next` plus atomic `advance(expected_revision,next,proved_facts)` and keep raw replacement private.

- [ ] **Step 4: Run focused and regression tests**

Run:

```bash
python3 scripts/run_exact_test.py --test trusted_fs --name observed_raw_name_has_no_serde_or_reverse_conversion
python3 scripts/run_exact_test.py --lib --name trusted_fs::tests::adjacent_receipt_facts_require_live_bound_lock_and_actual_synced_bytes
python3 scripts/run_exact_test.py --test trusted_fs --name failed_owned_receipt_advance_returns_same_locked_authority_without_revision_change
python3 scripts/run_exact_test.py --test trusted_fs --name post_write_owned_receipt_advance_failure_retains_lock_and_rereads_truth
python3 scripts/run_exact_test.py --test trusted_fs --name owned_receipt_remove_failure_retains_capability_stage_and_reread_truth
python3 scripts/run_exact_test.py --test trusted_fs --name child_enumerator_midstream_io_error_is_not_end_of_stream_or_absence
cargo test --locked --test trusted_fs
cargo test --locked --lib
cargo test --locked --test fs_ops
```

Expected: the trusted-fs integration target, library unit tests including the private receipt transition, and legacy fs-ops target PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/trusted_fs.rs tests/trusted_fs.rs
git commit -m "feat: add trusted fd-relative filesystem primitives"
```

### Task 2: Add concrete latest-wins read lanes

**Files:**
- Create: `src/read_lane.rs`
- Create: `tests/read_lane.rs`
- Modify: `src/lib.rs:1-10`

- [ ] **Step 1: Write red lane tests**

Add `scan_directory_key_replaces_only_pending_directory`, `preview_lane_replaces_pending_request`, `scan_mailbox_is_concrete_not_fifo`, `result_channel_never_exceeds_four`, `closing_lanes_releases_blocked_sender`, and `worker_panic_becomes_worker_lost`. Use barriers so directory request 1 is running while directory requests 2 and 3 are submitted; assert only 1 and 3 execute. Assert the private scan mailbox has a named directory slot and next-kind cursor rather than `VecDeque`, unkeyed `Option<ScanRequest>`, or generic payload storage.

- [ ] **Step 2: Verify RED**

Run:

```bash
python3 scripts/run_exact_test.py --test read_lane --name scan_directory_key_replaces_only_pending_directory
cargo test --locked --test read_lane -- --nocapture
```

Expected: FAIL with unresolved import `tersh::read_lane`.

- [ ] **Step 3: Add the concrete lanes**

Add the locked interfaces above, a private concrete `ScanMailbox { directory: Option<ScanRequest>, next_kind: ScanWorkKey, closed: bool }` behind one mutex/condvar, and a private `PreviewSlot { value: Option<PreviewRequest>, closed: bool }` behind its own mutex/condvar. Do not use a generic work payload or FIFO. Wrap each backend invocation in `catch_unwind`, send one worker-lost event, and close that lane after panic. Leave the exact extension seam documented in the stable handoff: Plan 4 adds a second typed recovery field and fair alternation without replacing the scan worker.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
python3 scripts/run_exact_test.py --test read_lane --name scan_directory_key_replaces_only_pending_directory
cargo test --locked --test read_lane -- --nocapture
```

Expected: all six tests PASS and no test creates more than two worker threads.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/read_lane.rs tests/read_lane.rs
git commit -m "feat: add bounded latest-wins read lanes"
```

### Task 3: Move directory scans off the UI event path

**Files:**
- Modify: `src/app.rs:101-132,301-417,451-583,913-933,1219-1263,1791-1839`
- Modify: `tests/app_keys.rs:27-42,437-448`
- Create: `tests/app_async.rs`

- [ ] **Step 1: Reverse the unsafe reload expectation**

Rename `reload_failure_clears_previous_selection` to `reload_failure_keeps_last_good_entries_and_selection_stale`. Assert the old entry and selection remain, `app.scan_status()` is `ReadStatus::Stale`, and the escaped error is visible.

- [ ] **Step 2: Add red generation tests**

Add `late_scan_for_old_cwd_is_discarded`, `rapid_refresh_keeps_only_latest_scan_request`, `slow_scan_does_not_block_navigation_command`, and `file_argument_slow_scan_focuses_and_previews_only_after_matching_generation` to `tests/app_async.rs` with injected read backends. The last test constructs Tersh with a `README.md` file path, holds the scan behind a barrier, proves the initial loading frame is renderable, releases the matching scan, and then asserts raw-name focus plus preview dispatch.

- [ ] **Step 3: Verify RED**

Run:

```bash
python3 scripts/run_exact_test.py --test app_async --name late_scan_for_old_cwd_is_discarded
cargo test --locked --test app_async
```

Expected: FAIL because `App` has no scan generation or injected read-lane constructor.

- [ ] **Step 4: Add App scan state and polling**

Add `cwd_generation`, `fs_epoch`, `scan_status`, and `ReadLanes` fields. Preserve file-argument startup as `InitialFocusIntent { raw_name: OsString, open_preview: bool, cwd_generation }`; never search the empty pre-scan list. Replace `reload` with `request_scan`, which increments `cwd_generation`, updates any still-pending initial intent to that generation, marks loading, and replaces the scan slot. Add `pub fn poll_background(&mut self) -> bool`, returning true only when visible state changes, plus `#[doc(hidden)] pub fn new_for_test(path: PathBuf) -> Result<Self>` and `#[doc(hidden)] pub fn settle_background_for_test(&mut self, timeout: Duration) -> Result<()>`. Production `App::new` schedules exactly one initial scan and returns immediately with loading state; `run_tui` renders that state and begins polling without scheduling a duplicate. Only `new_for_test` waits. After accepting a matching scan result, consume the intent, focus by exact raw `OsString`, and dispatch preview if requested. A stale generation never consumes the intent. Update existing integration tests that require populated entries to call `new_for_test`.

- [ ] **Step 5: Drain background events every event-loop turn**

At `run_tui`, start with `dirty = true`, call `dirty |= poll_background()` before the draw decision and after terminal input, draw only when dirty, then set dirty false. Use `event::poll(Duration::from_millis(25))`; a timeout never marks dirty and never redraws. Apply a scan only when cwd path, generation, and epoch match. On error, retain last-good entries/selection and set stale.

- [ ] **Step 6: Verify GREEN and existing keys**

Run:

```bash
python3 scripts/run_exact_test.py --test app_async --name late_scan_for_old_cwd_is_discarded
cargo test --locked --test app_async --test app_keys
```

Expected: all async scan and existing key tests PASS.

- [ ] **Step 7: Commit**

```bash
git add src/app.rs tests/app_async.rs tests/app_keys.rs
git commit -m "feat: make directory scans latest-wins and nonblocking"
```

### Task 4: Move preview generation off the UI event path

**Files:**
- Modify: `src/app.rs:242-258,451-495,935-999,1219-1263`
- Modify: `tests/app_async.rs`
- Modify: `tests/preview.rs:1-117`

- [ ] **Step 1: Write red preview races**

Add `late_preview_for_old_focus_is_discarded`, `preview_result_with_old_epoch_is_discarded`, `preview_failure_keeps_last_good_preview_stale`, and `rapid_focus_changes_keep_one_pending_preview`.

- [ ] **Step 2: Verify RED**

Run:

```bash
python3 scripts/run_exact_test.py --test app_async --name late_preview_for_old_focus_is_discarded
cargo test --locked --test app_async
```

Expected: FAIL because preview still executes inside `update_preview`.

- [ ] **Step 3: Add preview token application**

Add `preview_generation` and `preview_status`. Replace `preview_for_path` execution with cache lookup followed by `request_preview`. Apply results only when cwd generation, preview generation, epoch, and focused path match; cache only accepted successful results. Preserve the last-good preview on error and show stale state.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
python3 scripts/run_exact_test.py --test app_async --name late_preview_for_old_focus_is_discarded
cargo test --locked --test app_async --test preview --test app_keys
```

Expected: all tests PASS; injected 1,000 ms preview never blocks `handle_command`.

- [ ] **Step 5: Commit**

```bash
git add src/app.rs tests/app_async.rs tests/preview.rs
git commit -m "feat: make previews latest-wins and nonblocking"
```

### Task 5: Add `fs_epoch` read invalidation and freeze the G1a candidate

**Files:**
- Modify: `src/app.rs:101-132,913-980,1219-1263`
- Modify: `tests/app_async.rs`
- Create: `src/bin/tersh-plan2-read-bench.rs`
- Create: `tests/plan2_read_acceptance.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Write red read-invalidation races**

Add `scan_result_with_old_fs_epoch_is_discarded`, `preview_result_with_old_fs_epoch_is_discarded`, `filesystem_view_invalidation_advances_epoch_before_rescan`, `accepted_read_result_requires_current_generation_epoch_and_path`, and `late_read_after_invalidation_keeps_last_good_state_stale`.

- [ ] **Step 2: Verify RED**

Run:

```bash
python3 scripts/run_exact_test.py --test app_async --name scan_result_with_old_fs_epoch_is_discarded
cargo test --locked --test app_async
```

Expected: FAIL because `App` does not own/apply `fs_epoch`.

- [ ] **Step 3: Add the read invalidation epoch**

Add `fs_epoch: FsEpoch` and one private `invalidate_filesystem_view()` that increments before requesting replacement scan/preview work. Include the current epoch in every Directory/Preview request and require cwd generation, preview generation, epoch, and authoritative path/focus to match before applying a result. An old result never clears or overwrites the last good view; mark it stale until the replacement result arrives. Task 5 introduces no `MutationIntent`, worker, reservation, operation report, or mutation fence.

- [ ] **Step 4: Enforce result-application ordering**

For every event, compare all token fields before mutating App state. `invalidate_filesystem_view` first bumps epoch, then marks current scan/preview stale, then submits replacement work. Directory and Preview latest-wins slots remain independent and bounded. The future Task 11 mutation reducer calls this same method only after it has stored a truthful final report; it adds provisional/final mutation fences there, after the worker types exist.

Normal recovery uses the source-parent entry. EXDEV retained-source cleanup instead calls `reconcile_private_cleanup` with the verified private tombstone parent and may only continue an already persisted `ExdevSourceCleanup` delete phase. A `RestoreRequired` or persistent retry fault is consumed through nonmutating `release_classified`; this mode never opens or restores the original user path.

- [ ] **Step 5: Verify GREEN**

Run:

```bash
python3 scripts/run_exact_test.py --test app_async --name scan_result_with_old_fs_epoch_is_discarded
cargo test --locked --test app_async
```

Expected: all epoch/generation/path ordering tests PASS without importing mutation code.

- [ ] **Step 6: Commit the G1a implementation boundary**

```bash
git add src/app.rs tests/app_async.rs
git commit -m "feat: invalidate stale background reads by epoch"
```

- [ ] **Step 7: Write the frozen G1a candidate contract before its benchmark**

Add exact tests `g1a_candidate_reports_initial_frame_and_first_result`, `g1a_candidate_measures_exactly_200_keys_with_slow_read_backends`, `g1a_candidate_rejects_stale_and_superseded_results`, and `g1a_candidate_idle_window_has_zero_redraw_and_reports_cpu_rss`. The fixture injects separate 1,000 ms Directory and Preview backends, ten fixed warmups, a fixed 200-key input trace, cwd/preview generation supersession, 30 seconds of idle event-loop time, and frozen reference-profile metadata. These tests import no mutation worker, operation executor, control receipt, or cancellation type.

- [ ] **Step 8: Verify the G1a candidate is RED**

Run:

```bash
python3 scripts/run_exact_test.py --test plan2_read_acceptance --name g1a_candidate_reports_initial_frame_and_first_result
python3 scripts/run_exact_test.py --test plan2_read_acceptance --name g1a_candidate_measures_exactly_200_keys_with_slow_read_backends
python3 scripts/run_exact_test.py --test plan2_read_acceptance --name g1a_candidate_rejects_stale_and_superseded_results
python3 scripts/run_exact_test.py --test plan2_read_acceptance --name g1a_candidate_idle_window_has_zero_redraw_and_reports_cpu_rss
cargo test --locked --test plan2_read_acceptance
```

Expected: FAIL because `tersh-plan2-read-bench` and its frozen JSON schema do not exist.

- [ ] **Step 9: Implement and run the read-only candidate gate**

Create `tersh-plan2-read-bench` with no mutation dependency. It uses the fixed seed/profile and prints one canonical JSON document containing build/profile identity, fixture manifest, initial-frame and first-directory-result times, exactly 200 raw key-to-render samples plus p50/p95/max, longest event-loop stall, Directory/Preview submitted/superseded/stale-applied counts, 30-second idle redraw count, and raw idle CPU/RSS samples. Required `--output PATH` writes those exact bytes through a same-directory create-new temp, file sync, rename over only a previously verified same-user regular benchmark artifact, and parent sync, then reports their SHA-256; a symlink, wrong owner/mode, non-regular file, or unrelated JSON fails closed. `--require-reference-profile` fails closed unless the exact frozen host/OS/build/architecture/filesystem/toolchain profile matches. Exit nonzero unless p95 and max stall are each <=100 ms, stale-applied is zero, pending Directory and Preview work are each <=1, all 200 inputs are present, and idle redraws are zero. Do not emit mutation, copy, cancel, receipt, or recovery fields.

Run:

```bash
python3 scripts/run_exact_test.py --test plan2_read_acceptance --name g1a_candidate_reports_initial_frame_and_first_result
python3 scripts/run_exact_test.py --test plan2_read_acceptance --name g1a_candidate_measures_exactly_200_keys_with_slow_read_backends
python3 scripts/run_exact_test.py --test plan2_read_acceptance --name g1a_candidate_rejects_stale_and_superseded_results
python3 scripts/run_exact_test.py --test plan2_read_acceptance --name g1a_candidate_idle_window_has_zero_redraw_and_reports_cpu_rss
cargo test --locked --test plan2_read_acceptance --test app_async --test read_lane --test preview
cargo run --locked --release --bin tersh-plan2-read-bench -- --require-reference-profile --output target/tersh-plan2-read-candidate.json --fixture-root "$(mktemp -d /tmp/tersh-plan2-read.XXXXXX)"
```

Expected: tests PASS and JSON proves the exact Tier-1 reference profile, initial-frame/result timing, 200 inputs, p95/max-stall <=100 ms, zero stale applications, per-key pending bounds, zero idle redraws, and raw CPU/RSS. This JSON is the required G1a candidate evidence for the Tasks 1-5 implementation boundary; Tasks 6-13 may not retroactively substitute a mutation benchmark for it.

- [ ] **Step 10: Commit the frozen G1a component candidate**

```bash
git add Cargo.toml src/bin/tersh-plan2-read-bench.rs tests/plan2_read_acceptance.rs
git commit -m "test: freeze responsive read candidate evidence"
```

### Task 6: Create the trusted state root and atomic control bundles

**Files:**
- Create: `src/state_root.rs`
- Create: `src/source_claim.rs` with serialized state/proof types only; Task 7 adds execution
- Create: `tests/state_root.rs`
- Modify: `src/lib.rs`
- Modify: `src/operation.rs` to derive canonical serde for `OperationId` and `ItemId`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

- [ ] **Step 1: Write red trust and initialization tests**

Add `detects_exact_macos_and_linux_state_paths`, `rejects_relative_xdg_state_home`, `rejects_writable_or_wrong_owner_component`, `rejects_symlink_component`, `two_process_initializers_adopt_one_installation_id`, `loser_syncs_root_before_authorization`, `corrupt_winner_fails_closed`, `reserve_collision_never_opens_or_overwrites_existing_bundle`, `unclaimed_control_is_read_only`, `claim_consumes_handle_and_holds_lock_across_transition`, `second_process_observes_claim_in_use`, `transition_rejects_wrong_header_revision_edge_or_facts`, `serialized_or_boolean_facts_cannot_construct_transition_proof`, `terminal_verification_consumes_original_claim_and_remove_is_single_use`, `one_item_id_cannot_reserve_second_outer_protocol`, `fixed_mirror_intent_precedes_adjacent_confirmation`, `adjacent_ahead_without_fixed_intent_is_indeterminate`, `receipt_advance_syncs_file_then_parent`, `pending_controls_streams_without_collecting`, `startup_lists_pending_controls_from_different_cwd`, and `corrupt_pending_control_is_inspect_only_without_aborting_other_entries`.

Add `source_claim_proof_is_bound_to_bundle_revision_and_edge`, `mirror_proof_is_bound_to_bundle_revision_and_edge`, `mirror_factory_requires_verifier_issued_adjacent_facts`, `dropping_lock_before_confirm_is_unrepresentable`, `mirror_genuine_token_rejects_cross_bundle_replay`, `mirror_genuine_token_rejects_cross_revision_replay`, `mirror_genuine_token_rejects_cross_edge_replay`, `consumed_transition_token_cannot_be_used_twice`, `terminal_verification_consumes_original_claim_and_remove_is_single_use`, and `terminal_typestate_is_bound_to_bundle_revision_and_expectation`; obtain each genuine transition token from bundle A and prove bundle B rejects it without advancing anything. The mirror-factory test proves neither a caller hash nor serialized facts can become `AdjacentReceiptFacts` or `MirrorConfirmationProof`; compile/API tests prove neither the lock-borrowing facts nor any consumed transition token can outlive or be reused after its owning call. The terminal tests prove verification moves the original claim/lock into `VerifiedTerminalControl`, verification error owns both that same claim and moved expectation behind consuming retry, and neither a second lock nor a detachable/replayable terminal fact exists.

Add exact `terminal_remove_failure_retains_owning_typestate_and_reread_truth`. Inject failure before unlink, after unlink, and at pending-parent sync; assert every error retains `VerifiedTerminalControl` plus the exact stage, reports present/absent/unreadable from a no-follow re-read, and can enter reconciliation without reopening by `ItemId`.

Add exact `terminal_remove_revalidates_retained_owner_facts_immediately_before_unlink`. Mint genuine owner terminal facts, mutate the required external/adjacent fact after the first verification but before `remove`, and prove removal is refused while the same owning typestate remains recoverable. Source-check that `VerifiedTerminalControl` retains the concrete `TerminalExpectation`, not merely its hash, and that no serialized final-state booleans can enter the terminal enum.

Add exact `control_claim_error_returns_original_bundle_without_item_id_reopen`; inject lock/open failure and prove the same `ControlBundle` capability is returned and remains read-only/claimable after the fault clears.

After publishing and syncing the control receipt and its parent, refresh the already-open bundle directory with `TrustedDir::refresh_identity` and store that post-child snapshot in `ControlBundle::identity`; never reuse the snapshot captured before `claim.lock`/`receipt.json` creation as the durable bundle identity.

Add exact `control_bundle_identity_survives_consuming_claim`. Capture `ControlBundle::identity`, consume the bundle through both `Claimed` and injected-error paths, and require the claimed/original returned capability to report the same stable identity without a pathname or ItemId reopen.

Add crate-unit exact `state_root::tests::read_only_control_recovery_ref_claim_rejects_replacement_without_item_id_fallback`. Mint a `ControlRecoveryRef` from the actual read-only observation, replace or rename the observed bundle before claim, and prove `StateRoot::claim_recovery_ref` rejects the stale selector while returning exact reread truth; instrument `open_control_bundle(ItemId)` and require zero calls. The reference is Clone and nonauthorizing, retains the trusted pending parent, validated raw child name, full observed identity, and header, and never derives authority from its display `ItemId`. Keep both APIs crate-private; this test lives inside `src/state_root.rs` rather than an integration target.

Add crate-unit exact `state_root::tests::reserve_control_post_effect_failure_retains_partial_ownership_stage_and_disk_truth`. Inject every failure after bundle creation through claim verification; each error must own the exact optional bundle/lock/receipt handles, stage, no-follow disk truth, intended envelope, and parent capability. Its consuming abort either proves reverse-order removal plus parent sync or returns the same remaining ownership; only a pre-effect failure may be `NoEffect`. It lives beside the owner because the injected post-effect fault hook and partial owning typestate are private and never exist in an integration dependency or release build.

Add exact `prepared_control_abort_requires_owner_verified_pre_effect_state_and_retains_failure_ownership`. Reserve a complete plain SourceClaim control, prove that raw booleans/serialized envelopes and a mismatched host/revision cannot construct the owning typestate, then consume the genuine claim through `verify_prepared_abort(SourceClaimInitial)` and inject failure before unlink, after unlink, and at parent sync. Verification failure retains both the original claim and moved expectation in `PreparedControlAbortVerifyFailure`; every remove failure retains `VerifiedPreparedControlAbort`, the exact stage, and no-follow disk truth; consuming `retry` completes without an `ItemId` reopen. A compile test proves the verified authority is non-Clone/non-serde and cannot be detached from or outlive its original claim/lock. Plans 3/4 extend this same test contract for their subordinate-reservation absence/unchanged facts.

Add exact `fixed_mirror_intent_factory_requires_locked_current_adjacent_and_legal_host_edge`. Starting from a confirmed current adjacent receipt, prove the factory binds its actual live lock facts, current fixed identity/revision, exact concrete host edge, next intent, and canonical next state. A serialized edge/hash, stale receipt, wrong bundle, illegal edge, changed unrelated host field, or dropped lock cannot authorize `advance`.

Task 6 must GREEN before SourceClaim execution and the Plan 3/4 hosts exist. Therefore `src/state_root.rs` supplies one lint-clean `#[cfg(test)]`-only concrete harness: `ControlState::TestMirror`, `HostMirrorEdge::TestAdvance`, `TerminalExpectation::TestMirror`, and `PreparedControlAbortExpectation::TestMirror`, with owner validators and fact factories private to `state_root::tests`. These variants and hidden public field-private payload types exist only in the lib-test build, are never present in a normal library/release or accepted as production protocol state, and are not a generic/`Any` extension point. All tests that directly call a crate-private factory, obtain a genuine proof, or mint owner facts live under `state_root::tests`; `tests/state_root.rs` is limited to public initialization, streaming, and process-concurrency behavior. The unit harness exercises the same production reducers and owning typestates, while Plans 3/4 later replace it with their exhaustive owner validators.

Three substrate seams intentionally have no normal-library caller at the Plan 2 commit boundary: read-only `ControlBundle::recovery_ref`, mirror `confirm_mirror`/`verify_host_mirror_intent`, and the retained-source private-cleanup methods added in Task 7. Each declaration carries a narrow `#[cfg_attr(not(test), expect(dead_code, reason = "..."))]`, never `allow(dead_code)`: the expectation is active in the normal/integration dependency build where the seam is intentionally unused and absent in the lib-test build where the private harness exercises it. Plan 3 removes the entire two mirror attributes in the same source edits that add the first Trash host callers. Plan 4 removes the read-only-selector and private-cleanup attributes in the same source edits that add `cleanup_action` and EXDEV retained-source cleanup. With `-D warnings`, leaving an expectation after its first normal caller becomes `unfulfilled_lint_expectations`, so both the independent Plan 2 commit and every evolved consumer commit remain lint-clean without widening a capability to `pub`.

- [ ] **Step 2: Add deterministic crash schedules**

In `state_root.rs` unit tests, inject failure after temp create, write, file sync, no-replace publish, root sync, receipt temp sync, receipt advance, mirror-intent sync, adjacent-confirmation sync, and bundle sync. Add the paused-winner-after-rename/loser-before-sync interleaving. In `tests/state_root.rs`, make a hidden `state_root_child_process` test entry read a root/barrier/result-file configuration from environment; the parent integration test first invokes that binary with `--list` and requires the child entry exactly once, then launches two copies of `std::env::current_exe()` with `--exact state_root_child_process --nocapture`, releases them simultaneously, waits for both OS processes, and asserts both wrote the same validated ID. A two-thread substitute does not satisfy this gate.

- [ ] **Step 3: Verify RED**

Run:

```bash
python3 scripts/run_exact_test.py --lib --name state_root::tests::mirror_factory_requires_verifier_issued_adjacent_facts
python3 scripts/run_exact_test.py --lib --name state_root::tests::dropping_lock_before_confirm_is_unrepresentable
python3 scripts/run_exact_test.py --lib --name state_root::tests::consumed_transition_token_cannot_be_used_twice
python3 scripts/run_exact_test.py --lib --name state_root::tests::terminal_verification_consumes_original_claim_and_remove_is_single_use
python3 scripts/run_exact_test.py --lib --name state_root::tests::terminal_remove_failure_retains_owning_typestate_and_reread_truth
python3 scripts/run_exact_test.py --lib --name state_root::tests::terminal_remove_revalidates_retained_owner_facts_immediately_before_unlink
python3 scripts/run_exact_test.py --test state_root --name control_claim_error_returns_original_bundle_without_item_id_reopen
python3 scripts/run_exact_test.py --test state_root --name control_bundle_identity_survives_consuming_claim
python3 scripts/run_exact_test.py --lib --name state_root::tests::read_only_control_recovery_ref_claim_rejects_replacement_without_item_id_fallback
python3 scripts/run_exact_test.py --lib --name state_root::tests::reserve_control_post_effect_failure_retains_partial_ownership_stage_and_disk_truth
python3 scripts/run_exact_test.py --lib --name state_root::tests::fixed_mirror_intent_factory_requires_locked_current_adjacent_and_legal_host_edge
python3 scripts/run_exact_test.py --lib --name state_root::tests::prepared_control_abort_requires_owner_verified_pre_effect_state_and_retains_failure_ownership
cargo test --locked --lib state_root::tests -- --nocapture
cargo test --locked --test state_root -- --nocapture
```

Expected: FAIL with unresolved import `tersh::state_root`.

- [ ] **Step 4: Add the locked state-root interfaces**

Create user-owned mode-0700 directories fd-relatively, mode-0600 identity/receipt files, a 32-lowercase-hex installation ID, `transactions/pending/<item-id>`, and create-new/no-replace behavior. Add `sha2 = "0.10"` for canonical adjacent-receipt SHA-256 and regenerate the locked dependency graph. Every winner and loser validates the final identity file and syncs the state-root directory itself before returning success. `reserve_control` returns `ReserveControlError::Collision(candidate)` on the pre-effect `EEXIST` without opening the candidate, `NoEffect` only before any child exists, and `Owned(ControlReservationFailure)` after the first possible effect. The owning failure carries `PartialControlReservation`, exact stage and re-read truth; its `AtomicReceiptCreation` slot retains a completed receipt or the complete `AtomicReceiptCreateFailure`. Its consuming abort first consumes nested receipt-create abort/retry, then removes only the captured verified lock/bundle children in reverse order and syncs the parent, or returns `ControlReservationAbortFailure` with the same remaining authority. Host preparation errors in Plans 3/4 must retain the entire `ReserveControlError::Owned` until their owner recovery routine consumes it; they may not pretend it is already a `ClaimedControlBundle`. `pending_controls` returns `PendingControlStream` and yields every child independently: a valid strict envelope becomes `Verified`; a malformed name, receipt, owner, state, or header becomes `InspectOnly` with an escaped bounded error and never aborts enumeration or enables mutation/cleanup. It retains only its fd-relative enumerator and current observation, never a `Vec` proportional to stale controls.

Implement the exact non-generic `ControlEnvelope`, `ControlState`, owning `ClaimedControlBundle`, mirror intent, opaque transition-proof, and owning terminal typestate interfaces above. Put serialized `ClaimAction`/`SourceClaimState` data in `src/source_claim.rs` now, but keep every proof token non-serialized with private fields; do not add source rename/publish execution until Task 7. Unclaimed handles expose only `read` and consuming `try_claim`. Only the owning claimed handle may call `advance` or consume itself through `verify_terminal`; only the resulting `VerifiedTerminalControl` may call `remove`. A fixed host records mirror intent, the adjacent host later writes the exact next receipt, and fixed confirmation verifies revision plus canonical SHA-256 before constructing its opaque proof. `verify_terminal` retains the original claim/lock plus reopened no-follow snapshot and exact sync facts; it never acquires a second advisory lock or accepts caller booleans as evidence.

- [ ] **Step 5: Verify GREEN**

Run:

```bash
python3 scripts/run_exact_test.py --lib --name state_root::tests::mirror_factory_requires_verifier_issued_adjacent_facts
python3 scripts/run_exact_test.py --lib --name state_root::tests::dropping_lock_before_confirm_is_unrepresentable
python3 scripts/run_exact_test.py --lib --name state_root::tests::consumed_transition_token_cannot_be_used_twice
python3 scripts/run_exact_test.py --lib --name state_root::tests::terminal_verification_consumes_original_claim_and_remove_is_single_use
python3 scripts/run_exact_test.py --lib --name state_root::tests::terminal_remove_failure_retains_owning_typestate_and_reread_truth
python3 scripts/run_exact_test.py --lib --name state_root::tests::terminal_remove_revalidates_retained_owner_facts_immediately_before_unlink
python3 scripts/run_exact_test.py --test state_root --name control_claim_error_returns_original_bundle_without_item_id_reopen
python3 scripts/run_exact_test.py --test state_root --name control_bundle_identity_survives_consuming_claim
python3 scripts/run_exact_test.py --lib --name state_root::tests::read_only_control_recovery_ref_claim_rejects_replacement_without_item_id_fallback
python3 scripts/run_exact_test.py --lib --name state_root::tests::reserve_control_post_effect_failure_retains_partial_ownership_stage_and_disk_truth
python3 scripts/run_exact_test.py --lib --name state_root::tests::fixed_mirror_intent_factory_requires_locked_current_adjacent_and_legal_host_edge
python3 scripts/run_exact_test.py --lib --name state_root::tests::prepared_control_abort_requires_owner_verified_pre_effect_state_and_retains_failure_ownership
cargo test --locked --lib state_root::tests -- --nocapture
cargo test --locked --test state_root -- --nocapture
```

Expected: all trust, concurrency, sync-order, crash, and different-cwd tests PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/lib.rs src/operation.rs src/state_root.rs src/source_claim.rs tests/state_root.rs
git commit -m "feat: add durable trusted state root"
```

### Task 7: Add durable `SourceClaim`

**Files:**
- Modify: `src/source_claim.rs`
- Create: `tests/source_claim.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write red claim tests**

Add `claim_uses_prepared_outer_control_without_reserving_a_second_bundle`, `host_protocol_embeds_one_source_claim_substate`, `source_claim_proof_has_no_public_constructor_or_deserializer`, `fixed_control_is_claimed_before_adjacent_lock`, `adjacent_root_cannot_outlive_or_swap_its_claimed_control`, `adjacent_root_derives_item_id_and_rejects_cross_item_reservation`, `claim_records_intent_before_source_rename`, `claim_rename_ctime_change_still_recognizes_same_object`, `claim_rejects_true_inode_swap`, `source_swap_before_claim_never_moves_replacement`, `claimed_identity_mismatch_restores_no_clobber`, `restore_conflict_retains_tombstone_and_receipt`, `publish_never_overwrites_competing_target`, `delete_rechecks_private_tombstone`, `adjacent_root_requires_matching_installation_id`, `unsupported_no_replace_has_no_effect`, `different_cwd_discovers_unfinished_claim`, `restart_reconciles_precommit_claim_to_original_path`, and `two_reconcilers_cannot_claim_one_control_bundle`.

Add exact crate-unit gates `source_claim::tests::adjacent_root_open_failure_returns_fixed_borrow_and_opened_root`, `source_claim::tests::source_claim_consuming_failure_retains_control_adjacent_lock_and_payload`, and `source_claim::tests::source_claim_recovery_methods_consume_actual_handles_without_field_access`. Inject failures after adjacent root open, claim rename, publish intent, and cleanup intent; every post-effect outcome must carry `SourceClaimRecovery` and no caller may reopen by ItemId to continue. The owner-module tests inspect the private partial typestate and prove the public/sibling surface can call only `reconcile`/`reconcile_owned`, cannot inspect fields, and that those consuming calls retain the original fixed borrow, adjacent root, receipt lock, trusted source/destination/private parents, and payload through a second injected failure.

The same `source_claim_recovery_methods_consume_actual_handles_without_field_access` gate covers the tombstone-only cleanup entry, a persistent second reconcile failure, `RestoreRequired`, and consuming `release_classified`. It asserts zero syscall against the original source parent/name, no receipt advance during release, every adjacent handle/borrow released, and the fixed durable state still discoverable.

- [ ] **Step 2: Add crash-boundary tests**

For each numbered SourceClaim step, terminate the injected executor, reopen `StateRoot`, and assert the unique source is either at the original path or discoverable through the fixed receipt and verified tombstone. Never accept an unknown object as owned cleanup.

- [ ] **Step 3: Verify RED**

Run:

```bash
python3 scripts/run_exact_test.py --test source_claim --name adjacent_root_derives_item_id_and_rejects_cross_item_reservation
python3 scripts/run_exact_test.py --lib --name source_claim::tests::adjacent_root_open_failure_returns_fixed_borrow_and_opened_root
python3 scripts/run_exact_test.py --lib --name source_claim::tests::source_claim_consuming_failure_retains_control_adjacent_lock_and_payload
python3 scripts/run_exact_test.py --lib --name source_claim::tests::source_claim_recovery_methods_consume_actual_handles_without_field_access
cargo test --locked --test source_claim -- --nocapture
```

Expected: FAIL with unresolved import `tersh::source_claim`.

- [ ] **Step 4: Add `TrustedAdjacentRoot` and `SourceClaim`**

Use exact `.tersh-txn-v1`, `owner.json`, `source-claims/<item-id>/payload`, and control states from design lines 385-435 plus normative lines 1359-1383. `SourceClaim::acquire` requires `&StateRoot`, a no-follow `TrustedDir` reopened from the immutable plan and checked against its prepared source-parent identity, plus the preparation-owned `&mut ClaimedControlBundle`; it constructs `TrustedAdjacentRoot` from those exact inputs, verifies operation/item IDs against the final `ItemPlan`, and advances only the outer host's embedded `SourceClaimState`. It never calls `StateRoot::reserve_control`. Hold the fixed claim before opening or locking `.tersh-txn-v1`. Record `SourceClaimIntent` durably before adjacent-root creation or user-path rename. After the atomic rename, compare the tombstone to the pre-claim identity with `same_object`, then capture and persist a fresh post-claim snapshot; every later private-path cleanup requires `same_snapshot` against that fresh identity. Delete or publish only that verified private payload.

Reconciliation consumes a read-only `ControlBundle` into `ClaimedControlBundle` before opening any adjacent path; a live owner returns `in use`. It restores a pre-commit verified payload no-clobber, accepts a published/deleted result only from recorded matching identities, and retains contradictory state as `Indeterminate` without deleting anything. Plans 3 and 4 must call the same API with their host envelope; no host may create a sibling SourceClaim control.

- [ ] **Step 5: Verify GREEN**

Run:

```bash
python3 scripts/run_exact_test.py --test source_claim --name adjacent_root_derives_item_id_and_rejects_cross_item_reservation
python3 scripts/run_exact_test.py --lib --name source_claim::tests::adjacent_root_open_failure_returns_fixed_borrow_and_opened_root
python3 scripts/run_exact_test.py --lib --name source_claim::tests::source_claim_consuming_failure_retains_control_adjacent_lock_and_payload
python3 scripts/run_exact_test.py --lib --name source_claim::tests::source_claim_recovery_methods_consume_actual_handles_without_field_access
cargo test --locked --test source_claim -- --nocapture
```

Expected: all identity-swap, crash, ownership, and no-clobber tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/lib.rs src/source_claim.rs tests/source_claim.rs
git commit -m "feat: add durable source claim protocol"
```

### Task 8: Add cancellable no-clobber copy executors

**Files:**
- Create: `src/mutation.rs`
- Create: `src/mutation_ops.rs`
- Create: `tests/mutation_ops.rs`
- Modify: `src/lib.rs`
- Modify: `src/fs_ops.rs:14-82,207-297,375-445`

- [ ] **Step 1: Write red copy protocol tests**

Add public worker/filesystem tests `regular_copy_uses_bounded_chunks_and_checks_cancel`, `regular_copy_syncs_payload_before_publish_and_parent_after`, `symlink_copy_preserves_raw_target_without_following`, `directory_copy_publishes_only_complete_root`, `directory_copy_applies_mode_and_mtime_bottom_up`, `source_drift_before_publish_has_no_visible_target`, `destination_parent_replacement_has_no_effect`, `target_race_never_overwrites`, `enospc_before_publish_is_failed_no_effect_after_verified_cleanup`, `eacces_before_publish_is_failed_no_effect`, `payload_file_sync_failure_never_publishes`, `destination_parent_sync_failure_after_publish_is_not_failed_no_effect`, `directory_sync_failure_preserves_truth`, `cancel_at_each_copy_safe_point_has_exact_outcome`, `failed_owned_cleanup_is_cleanup_required`, and `copy_progress_is_bounded_top_level_counters` in `tests/mutation_ops.rs`. Put the three direct ownership/dispatch gates `preparation_error_is_non_authorizing_and_host_errors_retain_ownership`, `preparation_failure_dispatch_retains_concrete_host_ownership`, and `plain_prepared_abort_and_failure_recovery_consume_all_owned_reservations` in `mutation_ops::tests`; they exercise crate-private preparers and opaque owning values without widening production visibility.

- [ ] **Step 2: Verify RED**

Run:

```bash
python3 scripts/run_exact_test.py --test mutation_ops --name regular_copy_uses_bounded_chunks_and_checks_cancel
python3 scripts/run_exact_test.py --lib --name mutation_ops::tests::preparation_error_is_non_authorizing_and_host_errors_retain_ownership
python3 scripts/run_exact_test.py --lib --name mutation_ops::tests::preparation_failure_dispatch_retains_concrete_host_ownership
python3 scripts/run_exact_test.py --lib --name mutation_ops::tests::plain_prepared_abort_and_failure_recovery_consume_all_owned_reservations
cargo test --locked --test mutation_ops
```

Expected: FAIL with unresolved import `tersh::mutation_ops`.

- [ ] **Step 3: Add copy execution**

First add the locked intent/reservation, `CancelToken`, `MutationContext`, and `ProgressSink` definitions to `mutation.rs`; do not add the worker thread yet. Then add crate-private `prepare_item(context: &MutationContext, operation_id: OperationId, item_id: ItemId, draft: ItemDraft) -> Result<(ItemPlan, PreparedReservation, MutationFenceSpec), ItemPreparationFailure>` and crate-private `execute_item(context: &MutationContext, request: &OperationRequest, plan: &ItemPlan, reservation: PreparedReservation, cancel: &CancelToken, progress: &ProgressSink) -> ItemExecutionResult`. In the same module define `abort_prepared_item(context: &MutationContext, plan: &ItemPlan, reservation: PreparedReservation) -> PreparedAbortOutcome`, `recover_preparation_failure(context: &MutationContext, failure: ItemPreparationFailure) -> PreparedAbortOutcome`, and `recover_execution_panic(context: &MutationContext, plan: &ItemPlan, seed: PreparedRecoverySeed) -> ItemExecutionResult`. At the independent Plan 2 boundary `ItemExecutionObservation` has no production variant and every existing arm returns `ItemExecutionResult::outcome_only`; Plan 4 adds only the concrete `Exdev(CleanupAvailable)` variant, not a generic payload. The closed matches cover `None`, `FixedControl`, `OwnedStaging`, and every concrete host variant added by Plans 3/4; each arm transfers the complete opaque value to its owner-module abort/recovery free function. There is no wildcard arm, public field extraction, bare-ID reopen, or generic destructor. Before moving a reservation by value into `execute_item`, the worker derives and retains its non-authorizing `PreparedRecoverySeed`. An owned-staging seed keeps the trusted parent plus exact raw name/full identity; every fixed-host seed is a `ControlRecoveryRef` minted from the actual claim. On unwind, `recover_execution_panic` fd-relatively revalidates that exact selector and consumes `StateRoot::claim_recovery_ref`; it never searches by `ItemId` or display path. Missing/replaced/unreadable truth is `Indeterminate`. A verified control is dispatched exhaustively by `(plan.kind, claimed.read().state)`, not by state alone: `CleanupRetainedSource + Exdev` selects cleanup reconciliation, a normal move plan + Exdev selects normal reconciliation, and every kind/state mismatch is `Indeterminate`. The draft is passed by value so every preparation error can return it. Task 8's `mutation_ops.rs` unit tests call the preparer synchronously; public integration tests exercise the same behavior only through `MutationWorker`, so no external caller can obtain or execute a reservation outside the fence protocol. Task 10 moves the unchanged functions onto the worker. The worker stores an `ItemPreparationFailure` in its private preparation journal until `recover_preparation_failure` reaches verified cleanup/terminal classification; it never reduces a host failure to a naked cause.

The `FixedControl` pre-ack arm does not reach through `ClaimedControlBundle` fields or misuse terminal removal. It consumes the unchanged plain control through `verify_prepared_abort(PreparedControlAbortExpectation::SourceClaimInitial)` and then `VerifiedPreparedControlAbort::remove`; verification/remove failure retains the same authority and maps only to `CleanupRequired`/`Indeterminate`. Host arms in Plans 3/4 first clean or release their subordinate reservations, construct their owner-issued expectation facts, and use the same owning typestate.

For copy, preparation captures source/destination-parent identities and reserves the owned staging capability create-new before returning the final plan/fences. Execution copies regular files in 1 MiB chunks. Build file, symlink, or full directory topology under that prepared staging capability, apply the exact mode/mtime contract, sync payload, recheck source and destination parent, no-replace publish, then sync destination parent. Put named fault points for open, reservation create/collision/cleanup, chunk write, metadata, payload sync, no-replace publish, parent sync, and owned cleanup behind a deterministic `#[cfg(test)] pub(crate)` controller in `mutation_ops.rs`. It is compiled only into the library unit-test build; sibling owner unit modules such as Plan 4 `exdev::tests` may drive the same controller, but integration crates, normal libraries, feature builds, and release binaries cannot name or contain it. Keep the base injected matrix in `mutation_ops::tests`, while `tests/mutation_ops.rs` exercises real filesystem permissions and races. Do not expose a production filesystem trait, Cargo feature, or generic fault API.

- [ ] **Step 4: Preserve legacy callers while adding safe code**

Do not route `App` yet. Keep legacy `copy_path` compiling for old tests, but move shared name validation and read-only helpers behind `pub(crate)` functions. No new code may call `replace_path`, `remove_existing`, or `remove_existing_if_identity_matches`.

- [ ] **Step 5: Verify GREEN**

Run:

```bash
python3 scripts/run_exact_test.py --test mutation_ops --name regular_copy_uses_bounded_chunks_and_checks_cancel
python3 scripts/run_exact_test.py --lib --name mutation_ops::tests::preparation_error_is_non_authorizing_and_host_errors_retain_ownership
python3 scripts/run_exact_test.py --lib --name mutation_ops::tests::preparation_failure_dispatch_retains_concrete_host_ownership
python3 scripts/run_exact_test.py --lib --name mutation_ops::tests::plain_prepared_abort_and_failure_recovery_consume_all_owned_reservations
cargo test --locked --test mutation_ops --test fs_ops
```

Expected: new protocol tests and all legacy copy regressions PASS.

- [ ] **Step 6: Commit**

```bash
git add src/lib.rs src/fs_ops.rs src/mutation.rs src/mutation_ops.rs tests/mutation_ops.rs
git commit -m "feat: add cancellable staged copy operations"
```

### Task 9: Route same-filesystem destructive items through `SourceClaim`

**Files:**
- Modify: `src/mutation_ops.rs`
- Modify: `tests/mutation_ops.rs`
- Modify: `tests/fs_ops.rs:55-267`

- [ ] **Step 1: Write red destructive-operation tests**

Add `destructive_prepare_reserves_one_fixed_control_before_final_plan`, `destructive_prepare_collision_regenerates_item_id_without_opening_existing`, `partial_reservation_cleanup_unknown_is_cleanup_required`, `move_claims_then_publishes_no_replace`, `rename_claims_then_publishes_no_replace`, `trash_claims_then_publishes_no_replace`, `permanent_delete_unlinks_only_private_claim`, `permanent_delete_rejects_nonempty_directory_before_claim`, `empty_directory_delete_is_supported`, `replace_policy_is_rejected`, `exdev_move_restores_source_and_reports_no_effect`, `claim_eacces_preserves_source_and_reports_no_effect`, `publish_parent_replacement_never_redirects_target`, `publish_directory_sync_failure_is_cleanup_or_indeterminate`, `cancel_before_claim_is_cancelled_before_commit`, `cancel_after_claim_restores_or_reports_recovery_state`, and `cancel_during_publish_finishes_critical_section_before_reporting`.

- [ ] **Step 2: Verify RED**

Run:

```bash
python3 scripts/run_exact_test.py --test mutation_ops --name permanent_delete_unlinks_only_private_claim
cargo test --locked --test mutation_ops
```

Expected: FAIL because destructive dispatch is absent.

- [ ] **Step 3: Add operation dispatch**

Extend `prepare_item` so move, rename, current trash, and permanent delete reserve exactly one `ControlState::SourceClaim` fixed control create-new before the final `ItemPlan` exists. A collision regenerates the candidate `ItemId` and rebuilds only that item's final plan after every reservation created for the rejected candidate is proved removed and parent-synced. It never opens the collided object. If cleanup of an owned partial reservation is not proved, stop preparation and retain/report that candidate as receipt `CleanupRequired` or `Indeterminate`; do not silently try another ID.

Execution takes the prepared `ClaimedControlBundle` from `PreparedReservation::FixedControl`, passes it by mutable borrow to `SourceClaim`, then no-clobber publishes. For permanent delete, preparation accepts regular files, symlinks, and empty directories only; execution claims then unlinks only the verified private payload with `unlinkat`. If publish returns EXDEV, restore no-clobber and return `FailedNoEffect` only when restoration is proved; otherwise return the exact cleanup/indeterminate outcome. Plan 3 replaces current trash preparation with a concrete `TrashIngestV1` host/reservation; Plan 4 extends move preparation with its EXDEV host/adjacent reservation before final `Prepared`, so neither later path invents an ID or marker after acknowledgement.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
python3 scripts/run_exact_test.py --test mutation_ops --name permanent_delete_unlinks_only_private_claim
cargo test --locked --test mutation_ops --test source_claim --test fs_ops
```

Expected: all tests PASS and no non-empty directory traversal begins.

- [ ] **Step 5: Commit**

```bash
git add src/mutation_ops.rs tests/mutation_ops.rs tests/fs_ops.rs
git commit -m "feat: claim sources before destructive mutations"
```

### Task 10: Add the serial mutation worker and non-droppable truth events

**Files:**
- Modify: `src/mutation.rs`
- Modify: `src/mutation_ops.rs`
- Modify: `src/source_claim.rs`
- Create: `tests/mutation_worker.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write red worker tests**

Add `startup_reconciles_pending_claims_before_accepting_mutation`, `intent_has_only_ephemeral_submission_id`, `worker_preflight_generates_final_operation_and_item_ids`, `prepared_contains_immutable_request_reservations_and_final_fences`, `prepared_and_fence_ack_are_never_dropped`, `preflight_failure_prepares_terminal_report_without_started`, `full_ack_slot_returns_would_block_without_losing_prepared`, `dedicated_fence_ack_survives_logical_command_close`, `cancel_ack_bypasses_full_command_and_observation_slots`, exact `cancel_wins_if_ack_is_queued_before_worker_accepts_it`, `worker_emits_no_started_or_visible_effect_before_fence_ack`, `missing_ack_cleans_or_reports_every_reservation`, `cancel_during_preflight_is_terminal_and_no_effect`, `preflight_panic_or_disconnect_is_observed`, `second_intent_is_rejected_while_preparing_or_active`, `progress_coalesces_under_pressure`, `item_outcomes_and_finished_are_never_dropped`, `cancel_marks_queued_items_not_started`, `cancel_waits_through_noninterruptible_section`, `item_panic_becomes_indeterminate_or_observed_fact`, crate-unit exact `mutation::tests::panic_after_reservation_move_recovers_from_exact_seed_without_item_id_reopen`, `disconnect_synthesizes_one_finished`, and `first_final_wins_against_late_cancel`. The panic-after-move gate stays in the owner module because it drives the lib-test-only fault controller and inspects the private recovery seed journal; public worker behavior remains in `tests/mutation_worker.rs`.

- [ ] **Step 2: Verify RED**

Run:

```bash
python3 scripts/run_exact_test.py --test mutation_worker --name dedicated_fence_ack_survives_logical_command_close
python3 scripts/run_exact_test.py --test mutation_worker --name cancel_wins_if_ack_is_queued_before_worker_accepts_it
python3 scripts/run_exact_test.py --lib --name mutation::tests::panic_after_reservation_move_recovers_from_exact_seed_without_item_id_reopen
cargo test --locked --test mutation_worker -- --nocapture
```

Expected: FAIL with unresolved import `tersh::mutation`.

- [ ] **Step 3: Add worker channels and cancellation**

Before receiving a new command, add `source_claim::recover_source_claim_startup(context: &MutationContext, control: ClaimedControlBundle) -> StartupRecoveryNotice` and the closed `mutation_ops::recover_startup_control(context: &MutationContext, control: ClaimedControlBundle) -> StartupRecoveryNotice` now that all referenced types exist. Stream fixed-root pending bundles and route each item without constructing a fake request or plan. A `PendingControl::Verified` is consumed into its exact owning claim and passed to that dispatcher; its exhaustive match uses the typed outer `ControlState`, with Plan 2 handling standalone SourceClaim and later plans adding their owner-only Trash/Restore/EXDEV arms. `PendingControl::InspectOnly` produces a bounded inspect-only notice with no invented `ItemId` or protocol. Send every notice through the same non-droppable observation channel and remain unavailable until the stream reaches a clean terminal EOF; a mid-stream error keeps startup unavailable and surfaces a bounded failure rather than treating the unseen suffix as absent. Corrupt/unknown controls never authorize mutation. The existing `startup_reconciles_pending_claims_before_accepting_mutation` gate must prove different-cwd startup, typed owner dispatch without an `ItemPlan`, notice-before-availability ordering, exact retained cleanup/indeterminate truth, and no `open_control_bundle(ItemId)` fallback.

Use one new-start `sync_channel(1)`, one dedicated `sync_channel::<FenceInstalled>(1)`, one non-progress `sync_channel::<WorkerObservation>(1)`, and one `Mutex<Option<OperationEvent>>` progress slot. `try_start` and `acknowledge_fences` use distinct `try_send` paths and return typed `Busy`, `WouldBlock`, or unavailable errors rather than blocking the UI. Logical shutdown rejects new starts without dropping the dedicated acknowledgement sender. `request_cancel` release-stores the retained token and returns the visible acknowledgement without entering either full channel. `try_start` accepts `MutationIntent` by value, marks the `SubmissionId` busy immediately, and moves final preflight plus the 10,000-target walk onto the worker. For each item, generate candidate IDs, invoke the concrete preparer, and retry an `EEXIST` candidate only after `recover_preparation_failure` returns `ReleasedNoEffect`; `CleanupRequired`/`Indeterminate` stops candidate regeneration. Freeze `Arc<OperationRequest>`, `PreparedReservation`s, and the deduplicated final fences only when the entire request is prepared. Send `PreparedNotice`, then wait for matching `FenceInstalled`, cancellation, shutdown, or start-channel closure. Cancellation is the linearization winner whenever its release-store completes before the worker accepts the acknowledgement: the worker acquire-loads the token immediately before dequeuing/accepting an ack and again immediately after the matching dequeue, rejects the ack if either load observes cancellation, emits no `Started`, and consumes every reservation through `abort_prepared_item`. Thus an ack already queued when `request_cancel` returns `Requested` cannot race into execution. Only a matching ack bracketed by two non-cancelled loads permits `Started`; stale/wrong ack is rejected. After acknowledgement the worker checks cancellation before every item and between 1 MiB copy chunks but does not interrupt claim/verify, publish, or cleanup critical sections. The exact barrier test queues the ack, completes `request_cancel`, then releases the paused worker and proves zero `Started`/visible effects and no orphan reservation.

- [ ] **Step 4: Add panic/disconnect observation**

Wrap preparation and each item in `catch_unwind`. Preserve the active intent, candidate-ID/reservation journal, prepared immutable request when available, and current safe-point state in the `MutationWorker` owner. If preparation fails, is cancelled, panics, or disconnects before the normal `Prepared`, start one concrete observer that proves cleanup of every recorded reservation and assigns final report IDs to drafts that never received a durable candidate. It freezes one terminal-only immutable request, emits its non-droppable `PreparedNotice`, and waits for the same final-fence acknowledgement; after acknowledgement it emits exactly one outcome per draft plus `Finished`, but never `Started` and never executes an effect. It may use `FailedNoEffect` only when cleanup and no-effect are proved; unknown residue is `CleanupRequired`/`Indeterminate`. If acknowledgement never arrives, retain the same terminal state and perform the no-visible-effect cleanup path rather than inventing a second operation.

If an executing item panics after its reservation moved into the closure, the outer journal still owns the seed produced immediately before that move. After unwind has released any lock, the observer consumes the exact seed, revalidates raw name/full identity/header under the trusted parent, and claims only that object; fixed host state then routes to its owner reconciliation. The exact test injects panic immediately after move, after a durable intent, and after publish, and proves no reservation is silently dropped, no bare `ItemId`/display lookup occurs, and unknown post-terminal absence remains `Indeterminate`. If an executing worker channel disconnects, `MutationWorker::try_recv` starts one concrete observer thread, swaps in that observer's new bounded event receiver, and returns `ObserverStarted`; the observer inspects fixed receipts and final identities read-only, uses uncertainty=`Indeterminate`, marks remaining items `NotStarted(ExecutorUnavailable)`, and emits one `Finished` through the same reducer. If spawning the observer itself fails, enqueue bounded fallback outcomes of `Indeterminate` for any item with an unproved reservation/effect and `NotStarted(ExecutorUnavailable)` for untouched items plus one `Finished`; do not leave the submission running or run filesystem observation on the UI thread.

- [ ] **Step 5: Verify GREEN**

Run:

```bash
python3 scripts/run_exact_test.py --test mutation_worker --name dedicated_fence_ack_survives_logical_command_close
python3 scripts/run_exact_test.py --test mutation_worker --name cancel_wins_if_ack_is_queued_before_worker_accepts_it
python3 scripts/run_exact_test.py --lib --name mutation::tests::panic_after_reservation_move_recovers_from_exact_seed_without_item_id_reopen
cargo test --locked --test mutation_worker -- --nocapture
```

Expected: all prepare/ack, ID reservation, event, cancellation, pressure, panic, and disconnect tests PASS; the worker cannot produce a user-visible effect before the matching fence acknowledgement.

- [ ] **Step 6: Commit**

```bash
git add src/lib.rs src/mutation.rs tests/mutation_worker.rs
git commit -m "feat: add serial truthful mutation worker"
```

### Task 11: Cut App over to intents, prepared reports, and the worker

**Files:**
- Modify: `src/app.rs:101-174,1274-1704,1783-1839`
- Modify: `src/operation.rs`
- Modify: `src/ui.rs:368-494,539-614`
- Modify: `tests/app_keys.rs:125-340,437-685`
- Modify: `tests/render.rs:21-165,201-351`
- Create: `tests/app_mutation.rs`

- [ ] **Step 1: Write red App integration tests**

Add `dispatch_installs_provisional_fence_before_intent_submission`, `ui_intent_contains_no_final_operation_or_item_ids`, `prepared_swaps_final_fences_before_ack_and_started`, `busy_submission_removes_only_its_provisional_fence`, `worker_preflight_collision_surfaces_regenerated_final_ids`, `second_mutation_shows_busy_without_queueing`, `scan_committed_before_mutation_event_cannot_resurrect_entry`, `preview_intersecting_live_fence_is_held`, `no_effect_finished_clears_fence_without_epoch_bump`, `possible_effect_finished_bumps_epoch_before_rescan_and_fence_clear`, `slow_mutation_keeps_navigation_filter_help_and_inspection_responsive`, `cancel_request_is_visible_within_100ms_during_slow_copy`, `mixed_result_keeps_only_retry_candidates`, `retry_builds_new_ids_and_recaptures_identity`, `invalid_destination_keeps_mode_input_and_targets`, `replace_is_unreachable_from_conflict_ui`, `nonempty_directory_delete_is_rejected_during_worker_preflight`, and `finished_report_drives_selection_refresh_and_logs`.

- [ ] **Step 2: Verify RED**

Run:

```bash
python3 scripts/run_exact_test.py --test app_mutation --name dispatch_installs_provisional_fence_before_intent_submission
cargo test --locked --test app_mutation -- --nocapture
```

Expected: FAIL because App still calls synchronous `fs_ops` functions.

- [ ] **Step 3: Replace private duplicate drafts**

Remove `BufferedPath`, App's private `PathIdentity`, and `PendingFileOperation`. Modal validation and confirmation build only `MutationIntent::paths(submission_id, PathMutationIntent { kind, conflict_policy, protected_work_root, items: Vec<ItemDraft> })`; every `RawUnixPath::capture` failure retains the modal/input/targets and submits nothing. App rejects empty or more than 10,000 top-level drafts and performs no final path-identity capture, persistent-ID allocation, marker creation, or `OperationRequest` construction. Preserve captured modal targets and input on validation or busy failure. Final identity capture, `OperationId`/`ItemId` generation, create-new reservation, and immutable `OperationRequest`/`ItemPlan` construction remain worker-only.

- [ ] **Step 4: Replace synchronous App calls with the prepare/ack handshake**

Replace `execute_file_operation`, `submit_rename`, `submit_trash`, and `submit_delete` execution with this exact order: allocate ephemeral `SubmissionId`; derive and install conservative provisional fences; call bounded `MutationWorker::try_start(intent)`; on `Busy`/unavailable remove only those provisional fences and keep the modal/captured targets; on matching `PreparedNotice`, atomically swap to its final fences, create the active report from its immutable request, and call `acknowledge_fences`; on `WouldBlock`, retain the final fences and retry the acknowledgement from a later `poll_background` turn. Only a successful matching acknowledgement permits `Started`. A mismatched, stale, duplicate, or post-cancel `PreparedNotice` is never acknowledged and cannot mutate.

Add private `MutationFence { owner: FenceOwner, roots: Vec<FenceRoot> }`, with `FenceOwner::{Provisional(SubmissionId), Final(OperationId)}` and authoritative raw recursive roots. Deduplicate roots and collapse descendants under recursive ancestors. Add `OperationReport::did_or_may_have_effect()` for completed/warning/partial/cleanup/indeterminate/destination-committed truth. Any scan/preview intersecting a live provisional/final fence is held even if its epoch otherwise matches. On `Finished`, store the report first; if it did/may have effect call Task 5's `invalidate_filesystem_view` before requesting rescans; clear only that operation's fence last. No-effect completion clears without bumping epoch.

Drain `Prepared`, progress, item outcomes, observer notices, and `Finished` inside `poll_background`. Derive selection, retry candidates, epoch, rescans, log summary, active report, latest full report, and 20 summaries only from the final request and reducer output. Retry selects top-level candidates, then returns through modal validation and constructs a fresh `MutationIntent`; there is no replay API and App never reuses final IDs or reservations.

- [ ] **Step 5: Remove reachable replace and recursive delete**

Conflict UI accepts only `skip` or `abort`; remove all `replace` help/footer/modal text. Delete production calls to `copy_path(..., true)`, `replace_path`, and `remove_dir_all` for permanent delete. Keep a regression test that those paths are unreachable.

- [ ] **Step 6: Render truthful state**

At 40x10 render loading/stale, active operation count, `cancel requested`/`stopping after current item`, terminal summary, and a route to scrollable detail. At wider sizes reuse Inspector for active/latest full report. Do not expose fake percentages for unknown totals.

- [ ] **Step 7: Verify GREEN**

Run:

```bash
python3 scripts/run_exact_test.py --test app_mutation --name dispatch_installs_provisional_fence_before_intent_submission
cargo test --locked --test app_mutation --test app_keys --test render
```

Expected: all integration and 40x10 tests PASS.

- [ ] **Step 8: Commit**

```bash
git add src/app.rs src/operation.rs src/ui.rs tests/app_mutation.rs tests/app_keys.rs tests/render.rs
git commit -m "feat: integrate truthful background mutations"
```

### Task 12: Enforce mutation-aware shutdown and terminal failure ordering

**Files:**
- Modify: `src/app.rs:420-449,586-718,1791-1839,2180-2254`
- Modify: `src/main.rs:38-49`
- Modify: `tests/cli.rs`
- Create: `tests/shutdown.rs`

- [ ] **Step 1: Write red shutdown tests**

Add `q_does_not_commit_while_mutation_unresolved`, `ctrl_g_requests_cancel_without_exit`, `Q_requests_cancel_drains_finished_then_aborts`, `sigterm_drains_safe_point_then_returns_143`, `shutdown_before_fence_ack_cleans_or_reports_prepared_reservations`, `shutdown_prepared_callback_installs_fence_then_dedicated_ack_reaches_finished`, `shutdown_mismatched_fence_decision_never_starts_effect`, `full_nonprogress_channel_is_drained_before_join`, `ten_thousand_outcomes_do_not_deadlock_shutdown`, `render_failure_restores_terminal_then_drains_noninteractive`, `render_failure_with_blocked_finished_sender_still_terminates`, `worker_panic_or_disconnect_during_shutdown_gets_one_finished`, `shutdown_callback_holds_no_app_or_report_mutex`, `worker_failure_never_writes_cwd`, and `only_restored_commit_writes_stdout`.

- [ ] **Step 2: Verify RED**

Run:

```bash
python3 scripts/run_exact_test.py --test shutdown --name shutdown_prepared_callback_installs_fence_then_dedicated_ack_reaches_finished
cargo test --locked --test shutdown -- --nocapture
```

Expected: FAIL because `should_quit` exits immediately and `run_tui` returns only `PathBuf`.

- [ ] **Step 3: Add exact shutdown progression**

Use Plan 1 `RunOutcome`. Transition `Running -> ShutdownRequested(intent) -> logically close new mutation starts and read-lane submissions while retaining the dedicated fence-ack sender -> request cancellation -> continuously drain replaceable progress plus the capacity-one non-progress channel -> for Prepared, install final fences in the interactive/noninteractive reducer and return FenceAckDecision::Installed(exact submission) -> send dedicated acknowledgement -> observe the unique Finished -> drop event/ack receivers -> join worker/observer -> restore terminal -> return RunOutcome`. Never join while a sender can be blocked on `Prepared`, an item outcome, or `Finished`; never hold an App, report, terminal, or channel mutex while invoking `shutdown_after_active`'s observation callback. A shutdown before fence acknowledgement follows the verified reservation cleanup/`CleanupRequired` path and produces no user-visible filesystem effect. A callback that returns `NotPrepared` or a mismatched submission cannot unlock execution; the worker cleans or retains the reservation truth and still emits one terminal report.

After render/terminal failure, stop drawing and accepting input, attempt restoration immediately, then use a noninteractive local reducer to keep draining observations through the same exactly-one-final rule. Even with a full non-progress channel, 10,000 top-level outcomes, item panic, observer handoff, or worker disconnect, restore/return occurs only after the receiver is dropped and every real worker/observer thread is joined. Return failure with empty stdout; only a restored `CommitCwd` may write the cwd.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
python3 scripts/run_exact_test.py --test shutdown --name shutdown_prepared_callback_installs_fence_then_dedicated_ack_reaches_finished
cargo test --locked --test shutdown --test cli
```

Expected: exact exit codes and stdout rules PASS.

- [ ] **Step 5: Commit**

```bash
git add src/app.rs src/main.rs tests/cli.rs tests/shutdown.rs
git commit -m "fix: drain mutations before terminal shutdown"
```

### Task 13: Add frozen responsiveness/fault evidence and freeze the G1b component candidate

**Files:**
- Create: `src/bin/tersh-plan2-mutation-bench.rs`
- Create: `tests/plan2_acceptance.rs`
- Modify: `Cargo.toml`
- Modify: `README.md`

- [ ] **Step 1: Add deterministic mutation/integration acceptance tests**

Add `slow_mutation_keeps_navigation_filter_inspection_and_cancel_responsive`, `cancel_ack_latency_is_at_most_100ms_and_safe_stop_is_measured`, `ten_thousand_item_report_is_bounded`, `ten_thousand_and_one_targets_are_rejected_before_worker`, `final_ids_exist_only_after_worker_preflight`, `prepared_ack_precedes_every_visible_effect`, `reservation_collision_regenerates_id_without_opening_existing`, `late_read_after_mutation_never_applies`, `source_swap_never_deletes_replacement`, `enospc_eacces_parent_replace_and_sync_failures_keep_exact_outcomes`, `full_event_channel_shutdown_drains_before_join`, and `worker_loss_never_leaves_operation_running`. Keep Task 5's read candidate tests unchanged and require their committed JSON schema/profile hash as a prerequisite; this task does not re-label an integrated result as the G1a candidate.

- [ ] **Step 2: Add the frozen mutation/integration benchmark executable**

`tersh-plan2-mutation-bench` creates the named mutation fixtures with a fixed seed and prints one JSON document containing: the exact Task 5 read-candidate schema/profile/commit hash it consumed, raw navigation/filter/inspection samples while a mutation is active, 1 GiB copy throughput, processed top-level items/bytes, every raw cancel-request-to-visible-ack sample, every cancel-ack-to-safe-stop sample, full-channel drain/join timing, OS/architecture/filesystem/toolchain/git revision, and fixture manifest. `--require-reference-profile` verifies the exact frozen host/OS/build/architecture/filesystem/toolchain profile from the design before measuring and fails closed on any mismatch. Exit nonzero when foreground interaction or cancel acknowledgement exceeds 100 ms, the referenced read-candidate identity does not match Task 5, a deterministic stale/effect invariant is nonzero, or shutdown fails to reach one final report and joined worker. Safe-stop latency is reported raw and is not disguised as acknowledgement latency. This executable emits no new initial-frame, idle, or standalone scan/preview candidate claim.

- [ ] **Step 3: Run focused acceptance**

Run:

```bash
python3 scripts/run_exact_test.py --test plan2_acceptance --name prepared_ack_precedes_every_visible_effect
cargo test --locked --test plan2_acceptance -- --nocapture
```

Expected: all deterministic Plan 2 invariants PASS.

- [ ] **Step 4: Run frozen reference benchmark**

Run: `cargo run --locked --release --bin tersh-plan2-mutation-bench -- --require-reference-profile --read-candidate target/tersh-plan2-read-candidate.json --fixture-root "$(mktemp -d /tmp/tersh-plan2-mutation.XXXXXX)"`

Expected: JSON proves the exact reference profile and Task 5 read-candidate identity, foreground interaction and cancel acknowledgement <=100 ms with separate raw safe-stop times, raw copy throughput/items/bytes, no stale/effect invariant violation, and complete drain/join truth. Standalone initial-frame/result, 200-key read latency, idle redraw/CPU/RSS, and read supersession evidence remain solely Task 5's gate. Plan 4's additional RecoveryCatalog scan key is not fabricated here.

- [ ] **Step 5: Run formatting, lint, and full regression**

Run: `cargo fmt --all -- --check`

Expected: PASS.

Run: `cargo clippy --locked --all-targets --all-features -- -D warnings`

Expected: PASS.

Run: `cargo test --locked --all-targets`

Expected: PASS with no ignored safety test and every pre-Plan-2 regression retained or intentionally inverted by the trusted-core contract.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml README.md src/bin/tersh-plan2-mutation-bench.rs tests/plan2_acceptance.rs
git commit -m "test: lock responsive mutation core acceptance"
```

## Specification requirement to task map

| Design requirement | Task(s) | Acceptance evidence |
| --- | --- | --- |
| Dedicated latest-wins scan and preview workers; concrete keyed scan mailbox | 2-4 | directory-key replacement and App stale-generation tests |
| Normative prepare/ack boundary: UI owns only `SubmissionId`; worker creates final IDs/request after reservations | 8-11, 13 | no-ID intent, collision-regeneration, `Prepared`, fence-ack-before-effect tests (design 1324-1357) |
| File-path startup survives asynchronous initial scan | 3 | delayed `README.md` initial-focus/preview generation test |
| cwd generation, preview generation, `fs_epoch` | 3-5 | old cwd/focus/epoch results discarded |
| UI provisional fence before submission; final fence installed and acknowledged before `Started`; epoch/rescan/clear ordering | 5, 11-13 | prepare/ack and commit-before-event resurrection races stay closed (design 1324-1357) |
| Failed read preserves last-known-good list/selection/preview and marks stale | 3-4, 11 | inverted reload test and render tests |
| One serial active mutation; second rejected | 10-11 | worker and App busy tests |
| Worker-side filesystem preflight; final immutable request; 10,000 top-level cap; retry is a new intent/preflight | 8-11, 13 | final-ID/identity recapture, reservation, and 10,001 rejection |
| Progress may coalesce; item outcomes and `Finished` never drop | 10 | pressure tests on separate channels |
| Exactly one item outcome and one `Finished`; deterministic report truth | Plan 1, 10-11 | reducer plus worker race tests |
| Panic/disconnect observation and UI cannot remain running | 10, 12-13 | panic/disconnect and worker-loss acceptance tests |
| Fixed trusted state root and concurrent installation-ID winner durability | 1, 6 | trust, sync-fault, and paused-winner interleaving tests |
| One claimed typed outer control per `ItemId`; nested `SourceClaim`; fixed-before-adjacent mirror protocol | 6-10 | duplicate-protocol, owning-claim, lock-order, mirror crash tests (design 1359-1383) |
| Byte-safe paths and `RawUnixName`-only fd-relative child capabilities | 1, 6-9 | full escape/sentinel, invalid UTF-8, symlink, target-race tests (design 1385-1393) |
| Directory and Recovery scan keys share one fair latest-wins worker after Plan 4 | 2-4 plus stable handoff | directory replacement now; typed recovery/non-starvation in Plan 4 (design 1395-1402) |
| Existing-bundle discovery yields read-only observation then owning atomic claim | 1, 6-7 plus stable handoff | streaming inspect-only enumeration and exclusive `try_claim` tests (design 1404-1419) |
| Claim rename distinguishes stable object identity from mutable snapshot metadata | 1, 7 | ctime-change acceptance plus true inode-swap rejection |
| Durable fixed receipt and trusted adjacent root | 6-7 | crash-point and different-cwd discovery tests |
| Every destructive source action uses `SourceClaim` | 7, 9, 11 | source-swap/private-tombstone tests and no legacy production call |
| Restart reconciles durable same-filesystem claims without cwd dependence | 7, 10 | pre-commit restore, exclusive reconciler, and startup-recovery tests |
| Bounded cancellable staged regular-file copy | 8, 10 | 1 MiB safe-point and cancellation tests |
| Complete directory/symlink staging and no partial visible target | 8 | topology/raw-link/publish-only-root tests |
| Replace unreachable; non-empty directory permanent delete disabled | 9, 11 | UI and executor rejection tests |
| Cancel stops queued/new work, does not roll back committed work | 9-12 | safe-point and late-cancel first-final-wins tests |
| Slow mutations preserve navigation/filter/inspection/cancel responsiveness | 10-13 | App responsiveness and <=100 ms cancel acknowledgement evidence |
| G1a candidate is independently reproducible before mutation substrate exists | 1-5 | `tersh-plan2-read-bench` artifact with initial frame/result, 200 keys, slow scan/preview, stale/supersession, idle redraw/CPU/RSS and frozen profile |
| G1b mutation/integration evidence cannot impersonate the G1a candidate | 8-13 | `tersh-plan2-mutation-bench` consumes the exact read-candidate identity and reports mutation/cancel/copy/drain metrics separately |
| ENOSPC/EACCES/parent replacement/file+directory sync failures retain exact truth | 8-9, 13 | named fault matrix and aggregate acceptance test |
| Close/cancel/drain/drop/join shutdown ordering and stdout/exit truth | 10, 12-13 | full-channel, 10k-outcome, panic/disconnect, PTY/signal/terminal-failure tests (design 1421-1427) |
| Focused test gates cannot pass with zero tests | prerequisite and 1-13 | every RED/GREEN slice first uses `run_exact_test.py`, then runs its complete target regression (design 1471-1474) |
| Custom raw-path/name deserialization and inspect-only one-way downgrade | 1, 6-7 | canonical Base64, original-byte pre-normalization absolute-path checks, forbidden component, no-serde observed-name, and consuming downgrade tests (design 1488-1497) |
| Opaque verifier-issued transition facts cannot be forged, unlocked, dropped on error, or replayed | 1, 6-7 | live-lock adjacent factory, owning receipt/error and terminal typestates, plus genuine-token cross-bundle/revision/edge/single-use tests (design 1499-1505) |
| p95 key acknowledgement and max event-loop stall <= 100 ms | 5 | frozen standalone read-candidate JSON; Task 13 only verifies its identity while measuring active-mutation interaction |

## Stable handoff to Plans 3 and 4

After Task 13, later plans may depend only on these library-visible contracts. Cross-module production types are `pub` only where another library module requires them; integration-test constructors are `pub` plus `#[doc(hidden)]`; receipt mutation and unit fault hooks stay `pub(crate)` or private under `cfg(test)`:

The Tasks 1-5 G1a boundary is eligible as the `impl-03` component candidate only after `cargo test --locked --test plan2_read_acceptance` plus `tersh-plan2-read-bench --require-reference-profile --output target/tersh-plan2-read-candidate.json` pass. The Task 13 mutation benchmark may consume and verify that artifact identity but may neither regenerate nor replace the G1a claim. Only the shared implementation-evidence plan may close `impl-03` or `impl-04` after cumulative same-candidate gates and five-role review.

- `operation.rs`: IDs, immutable plans/requests, events, outcomes, reports;
- `trusted_fs.rs`: `PathIdentity`/explicitly ordered `PathIdentityKey`, fallible absolute `RawUnixPath`, capability `RawUnixName`, non-serde non-capability `ObservedRawName`, consuming `RawUnixName::into_observed`, `TrustedDir`, streaming `ChildEnumerator`/`ChildObservation`, parent-bound owning `ClaimedChildLock`, opaque lifetime-bound `AdjacentReceiptFacts`, owning `AtomicReceiptCreation`/create-abort recovery, owning `OwnedLockedReceipt<T>` plus owning error, and substrate-private `DurableReceipt`/`AtomicReceiptFile` transition helpers. Every child API takes `&RawUnixName`; Plans 3/4 may not add an `OsStr` or `ObservedRawName` child escape hatch;
- `state_root.rs`: `InstallationId`, public private-field `StateRootError`, concrete `ControlProtocol`/`ControlState`/`ControlEnvelope`, `PendingControl`/streaming `PendingControlStream`, `StateRoot`, read-only `ControlBundle`, owning `ClaimedControlBundle`, Clone nonauthorizing `ControlRecoveryRef` minted from either read-only or claimed exact capabilities, owning reservation/claim errors, typed transition proofs, `confirm_mirror(AdjacentReceiptFacts, next_state)`, owning `PreparedControlAbortVerifyFailure`/`VerifiedPreparedControlAbort`, owning `TerminalVerifyFailure`/`VerifiedTerminalControl`, and non-serde owner terminal/prepared-abort fact wrappers. One `ItemId` has one fixed typed outer bundle, and later host protocols add variants to `ControlState` rather than allocate sibling controls;
- `source_claim.rs`: serialized nested `SourceClaimState` with an immutable full source/tombstone/destination selector record preserved across every phase, control-borrowing `TrustedAdjacentRoot<'a>` that owns its actual bundle/lock/verified receipt, `SourceClaim`, `ClaimAction`, `ClaimResult`, source-parent `reconcile_pending`, tombstone-only `reconcile_private_cleanup`, consuming non-I/O `SourceClaimRecovery::release_classified`, and owner-only `recover_source_claim_startup`. `SourceClaim::acquire` transfers the already prepared `ClaimedControlBundle` borrow into the adjacent capability and never calls `reserve_control`; adjacent methods derive the item from that header;
- `mutation.rs`: `SubmissionId`, `ItemDraft`, `MutationIntent`, `PreparedNotice`, concrete `PreparedReservation`, crate-private `PreparedAbortOutcome`/`PreparedRecoverySeed`, closed crate-private `ItemExecutionObservation`/`ItemExecutionResult`, public non-authorizing `StartupRecoveryNotice`/`StartupRecoveryDisposition`, `MutationFenceSpec`, public/private-field `PreparationError` plus exhaustive `PreparationFailureKind`, public/private-field `ItemPreparationFailure` over a crate-private closed concrete inner enum with owning host variants, public/private-field `MutationWorkerError`, `MutationContext`, `MutationWorker`, and `CancelToken`; and
- `mutation_ops.rs`: worker-side `prepare_item`, single by-value `execute_item -> ItemExecutionResult`, closed `abort_prepared_item`, closed `recover_preparation_failure`, closed `(plan.kind, control state)` `recover_execution_panic -> ItemExecutionResult`, and closed no-`ItemPlan` `recover_startup_control -> StartupRecoveryNotice` dispatches extended by later operation kinds. Every later opaque reservation/error owner must supply consuming abort/recovery/panic/startup-reconciliation free functions; the worker never destructures host-private fields, and its one ordered emitter publishes a concrete item-side observation before the matching terminal outcome.

Plan 4 is allowed to evolve the concrete directory-only `read_lane::{ScanMailbox, ScanWorkKey, ScanRequestKind, ReadEvent, ScanBackendSet}` on the same feature branch by adding typed `RecoveryScanRequest`, `RecoveryPage`, and RecoveryCatalog backend/finished variants. It must keep the same scan thread and result channel, replace pending work per key, alternate fairly when both Directory and RecoveryCatalog are pending, and cap the scan mailbox at two. That planned source edit is the extension mechanism; Plan 2 exports no G2 record, stringly recovery state, generic payload, or `Any` hook.

Plans 3 and 4 extend preparation before `Prepared`: Plan 3 adds `MutationIntentBody::Restore(RestoreIntent)`, Trash/Restore host and reservation variants, and a matching worker-side `prepare_restore`; Plan 4 adds its EXDEV/cleanup body and reservations. UI and CLI coordinators still submit only a typed intent plus ephemeral `SubmissionId`; no later plan may call `MutationWorker::try_start` with an `OperationRequest` or construct final IDs outside worker preflight. Trash ingest, restore, EXDEV, and Recovery handoff may not create durable markers or final IDs after `FenceInstalled`; collision regeneration and unproved-reservation cleanup use the same worker preparation journal. The fixed control is acquired first; an adjacent receipt is written only after fixed mirror intent and is confirmed through verifier-issued `AdjacentReceiptFacts` before enabling source cleanup.

Catalog/recovery code keeps verified `RawUnixName` capabilities separate from inspect-only `ObservedRawName` keys. Only a private-field `VerifiedBundleRef` may contain a capability and enter `TrashStore::claim_existing`; downgrading a validated-but-invalid bundle consumes its name through `into_observed`. An inspect-only reference contains only that non-serde observed key, location, and identity for ordering/display and cannot enter a child API. Duplicate `ItemId`s across staging/items/claims/quarantine are contradictory inspect-only rows. `TrashStore::claim_existing(self, ClaimedControlBundle, VerifiedBundleRef)` atomically no-clobber renames the exact observed bundle from `items` to `claims`, verifies the fixed header/host binding and `same_object`, captures a fresh private snapshot, and returns one private owning transaction containing the store, fixed claim, adjacent capability, bundle and lock. It refuses action on stale or contradictory observations, but every pre-effect failure returns the original store/claim/observation and every post-rename failure returns an owning recovery typestate; consuming errors may not discard the fixed claim or adjacent lock. Fixed controls use consuming `ControlBundle::try_claim`; adjacent trash bundles use this claimed-store API. Neither path may reopen by display path or bare `ItemId`. Plan 3 provides the typed catalog domain and CLI engine; Plan 4 routes its typed page through the added RecoveryCatalog scan key.

Plans 3 and 4 must not copy receipt-writing, state-root discovery, path-identity, no-replace, source-claim, or worker logic. They must not read operational truth from `App`, log strings, cwd, or display paths.

## Plan 2 exit condition

The Plan 2 component recipes are complete only when Tasks 1-13 are committed in order, every focused gate is run through the exact-test runner, targeted RED/GREEN evidence is recorded, `cargo fmt`, strict locked Clippy, and `cargo test --locked --all-targets` pass, the frozen responsiveness gate passes on the named reference environment, and adversarial review finds no ID-collision, pre-ack effect, source-swap, crash-window, worker-loss, stale-read, cancellation, full-channel shutdown, or join-before-drain trace that violates the design. This produces the `impl-03` and `impl-04` candidates but does not accept either iteration; their evidence-only closure belongs to `2026-08-10-tersh-implementation-iteration-evidence.md`. It also does not claim G1c, recoverable trash/G2, G3, or the full Workbench Trusted Core release label.
