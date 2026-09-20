# EXDEV Move And Recovery UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete limited regular-file/symlink EXDEV moves without unique-source loss, then expose the already-proven trash restore engine through a thin no-overwrite Recovery overlay.

**Architecture:** `exdev.rs` is a concrete durable move state machine prepared and executed by Plan 2's one serial mutation worker. Background preflight creates the one claimed typed fixed control plus owned adjacent reservation, freezes the final request, and waits for `FenceInstalled`; execution then advances only typed receipts and the nested `SourceClaim`. `recovery_ui.rs` owns pure overlay state while the existing keyed scan worker loads typed Plan 3 pages and the mutation worker revalidates a private `VerifiedBundleRef`, so neither UI path constructs final IDs nor duplicates recovery logic.

**Tech Stack:** Rust 1.88, Plan 2 trusted-filesystem and mutation modules, Plan 3 recovery engine, `ratatui` TestBackend, `crossterm`, Cargo integration tests, real-process and fault-injected filesystem fixtures.

---

## Locked boundaries and existing anchors

- Baseline used for source anchors: `799cf08f1abd9b546133ed419bf6d4341714e292`.
- Current cross-device failure path: `src/fs_ops.rs:84-95` (`rename_path`).
- Current synchronous move/copy dispatcher to replace with the Plan 2 mutation lane: `src/app.rs:1307-1492` (`paste_buffer`, `copy_to_destination`, `start_file_operation`, `submit_conflict`, `execute_file_operation`).
- Current modal and mode definitions: `src/app.rs:34-91`, `src/app.rs:158-239`, `src/app.rs:1648-1668`.
- Current overlay renderer: `src/ui.rs:18-77`, `src/ui.rs:450-615`, `src/ui.rs:748-760`.
- Existing copy/move/conflict tests to update: `tests/fs_ops.rs:239-373`, `tests/app_keys.rs:284-685`, `tests/render.rs:111-321`.
- Design authority: G1c `docs/superpowers/specs/2026-08-10-tersh-trusted-core-design.md:771-849`; G2 overlay `:970-977` and `:1037-1044`; gate rows `:1101-1102`; planning boundary `:1221-1222`; normative prepare/claim/read/catalog clauses `:1324-1419`; canonical raw deserialization and proof binding/replay clauses `:1488-1505`.

Required completed inputs:

```rust
// Plan 2
use crate::mutation::{CancelToken, ItemDraft, ItemExecutionObservation, ItemExecutionResult, ItemPreparationFailure, ItemPreparationFailureInner, MutationContext, MutationFenceSpec, MutationIntent, MutationWorker, PreparationError, PreparedAbortOutcome, PreparedNotice, PreparedReservation, ProgressSink, StartMutationError, StartupRecoveryNotice, SubmissionId};
use crate::operation::{ConflictPolicy, ItemId, ItemOutcome, ItemPlan, OperationId, OperationKind, OperationRequest};
use crate::read_lane::{ReadEvent, ScanRequest, ScanRequestKind, ScanToken};
use crate::source_claim::{ClaimAction, ClaimError, ClaimResult, SourceClaim, SourceClaimState};
use crate::state_root::{ClaimedControlBundle, ControlBundle, ControlClaimAttempt, ControlEnvelope, ControlHeader, ControlProtocol, ControlRecoveryClaimDiskTruth, ControlRecoveryClaimFailure, ControlRecoveryRef, ControlReservationAbortFailure, ControlReservationFailure, ControlState, ControlTransitionProof, HostMirrorEdge, InstallationId, MirrorIntent, PendingControl, PendingControlStream, PreparedControlAbortExpectation, PreparedControlAbortFailure, PreparedControlAbortVerifyFailure, ReserveControlError, StateRoot, TerminalExpectation, TerminalRemoveFailure, TerminalVerifyFailure, VerifiedPreparedControlAbort, VerifiedTerminalControl};
use crate::trusted_fs::{AdjacentReceiptFacts, AtomicReceiptCreation, AtomicReceiptFile, ClaimedChildLock, DurableReceipt, ObjectKind, ObservedRawName, OwnedLockedReceipt, OwnedLockedReceiptAdvanceError, PathIdentity, PathIdentityKey, RawUnixName, RawUnixPath, TrustedDir, TrustedFsError};

// Plan 3
use crate::recovery::{PreparedRestoreReservation, RecoveryCatalogCursor, RecoveryPage, RecoveryRecord, RecoveryService, RestoreIntent, RestoreRequest, RestoreSelector, VerifiedBundleRef};
use crate::trash::{TrashReceipt, TrashState};
```

Before Task 1, run `cargo test --locked --test trusted_fs --test state_root --test source_claim --test mutation_ops --test mutation_worker --test trash_receipt --test recovery_cli`. All tests must pass. `AtomicReceiptFile`/`DurableReceipt` remain `pub(crate)` substrate; `ControlBundle` remains read-only; capability-bearing `VerifiedBundleRef` has private fields and is distinct from Plan 3's non-operational inspect-only reference. This plan must not add an identity type, raw receipt writer, source-claim algorithm, trash/restore state transition, CLI subcommand, second mutation worker, generic work payload, or conversion from inspect-only catalog data to `RawUnixName`.

Every EXDEV call to `StateRoot::reserve_control` exhaustively matches the stable Plan 2 result: `ReserveControlError::Collision(collided_item_id)` returns the unchanged `ItemDraft` and never opens the colliding fixed child; `NoEffect(error)` returns the draft plus diagnostic with no capability; `Owned(failure)` moves the entire `ControlReservationFailure` into `FixedReservationOwned`. Owner recovery consumes `failure.abort()` and any returned `ControlReservationAbortFailure::retry()` without detaching stage or disk truth. A destination-adjacent `EEXIST` is a separate typed foreign collision after fixed reserve; only a pre-effect control collision or a post-fixed collision whose exact fixed cleanup is proved may regenerate the candidate ID.

Plan 1's exact runner contract is `python3 scripts/run_exact_test.py (--test <target> | --lib) --name <full-name> [--ignored] [--serial] [--case-matrix <id> --expect-case <case> ...]`. The selector is mutually exclusive, list/discovery and execution must each find exactly one test, and a zero-executed or mismatched frozen matrix is a failure. Private owner/proof seams below use `--lib` with the fully qualified module name; public behavior uses `--test`.

Plan 3 already makes the concrete `Restore(RestoreIntent)` evolution to Plan 2's intent. Plan 4 adds only the cleanup-specific body; it does not redefine the restore request, reservation, or worker API:

```rust
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
    // Added by Plan 3.
    Restore(RestoreIntent),
    // Added by Plan 4.
    CleanupRetainedSource(CleanupSourceAction),
}

#[derive(Clone, Debug)]
pub struct MutationIntent {
    pub submission_id: SubmissionId,
    pub body: MutationIntentBody,
}

```

Normal move remains `Paths`; worker preflight detects differing devices and selects the EXDEV preparer. The UI and CLI allocate only `SubmissionId` and one concrete body. Plan 3's private `RestoreWorkRoot` binds raw authoritative root plus its submission-time identity, not display text: TUI construction derives it from the selected `VerifiedBundleRef`, while the CLI-only constructor observes current-dir identity inside recovery.rs. Worker preflight reopens and compares it no-follow and never trusts it as an already verified directory. Final `OperationId`, final `ItemId`, final captured identities, fixed/adjacent reservations, immutable `OperationRequest`, final fences, and every `PreparedReservation` remain worker-owned.

### Task 1: Prepare one durable EXDEV reservation inside the mutation worker

**Files:**
- Create: `src/exdev.rs`
- Modify: `src/lib.rs`
- Modify: `src/state_root.rs`
- Modify: `src/mutation.rs`
- Modify: `src/mutation_ops.rs`
- Create: `tests/exdev.rs`
- Modify: `tests/mutation_worker.rs`

- [ ] **Step 1: Write failing schema, preparation, and discovery tests**

Add exact tests `exdev_transition_rejects_backward_revision_or_unproved_fact`, `exdev_prepare_reserves_one_claimed_outer_before_adjacent`, `exdev_prepare_collision_regenerates_item_id_without_opening_existing`, `exdev_adjacent_collision_cleans_fixed_before_regenerating`, `exdev_prepare_failure_returns_draft_fixed_partial_or_complete_adjacent_ownership`, `exdev_adjacent_open_failure_retains_partial_owned_capabilities_and_reread_truth`, `exdev_unproved_reservation_cleanup_is_cleanup_required`, `exdev_prepared_waits_for_fence_ack_before_source_or_destination_effect`, `exdev_pre_ack_cancel_removes_only_verified_reservations`, `exdev_source_claim_is_nested_in_the_same_outer_control`, `exdev_cannot_fabricate_mirror_or_terminal_authority`, `exdev_transition_rejects_genuine_token_from_other_bundle_revision_or_edge`, `exdev_authorizing_facts_cannot_outlive_claim_or_locked_receipt_snapshot`, `exdev_consumed_transition_token_cannot_be_used_twice`, `exdev_transition_failure_retains_lock_authorization_and_reread_truth`, `exdev_terminal_remove_consumes_original_claim_and_retains_failure_typestate`, `exdev_adjacent_terminal_remove_failure_retains_fixed_claim_lock_stage_and_disk_truth`, `exdev_receipt_rejects_forged_raw_path_capability`, `exdev_raw_receipt_and_control_handles_are_not_public`, `exdev_unfinished_bundle_is_discovered_from_different_cwd`, and `exdev_pending_stream_error_is_incomplete_not_absent`.

Also add exact `exdev_prepared_abort_consumes_owner_verified_adjacent_absence_and_fixed_typestate`, proving the central Plan 2 abort dispatcher reaches the EXDEV owner, adjacent absence is revalidated immediately before fixed unlink, and every failure retains the owning typestate.

`exdev_prepare_collision_regenerates_item_id_without_opening_existing` precreates a valid-looking fixed candidate and an adjacent sentinel with identities/content snapshots; the worker must leave both untouched, prove removal of only its own partial reservations, and emit a final `PreparedNotice` whose regenerated `ItemId` differs. `exdev_prepared_waits_for_fence_ack_before_source_or_destination_effect` blocks the acknowledgement and proves the only new objects are fixed/adjacent reservations named by the preparation journal. The discovery test starts a second process in an unrelated cwd, finds the unfinished item through `StateRoot::pending_controls`, passes its exact claimed control through the Plan 2 startup dispatcher into `recover_exdev_startup` without an `ItemPlan`, and observes the bounded startup notice before new mutations become available.

Also add exact `exdev_preparation_failure_enters_worker_dispatch_with_ownership`, covering Draft, Fixed, PartialAdjacentOwned, and AdjacentOwned through the redacted public wrapper and `ItemPreparationFailureInner::Exdev` consuming recovery path.

Keep owner/private tests in the library modules that own the capabilities. The fully qualified crate-unit gates under `exdev::tests` cover receipt/proof/terminal and preparation ownership; `mutation_ops::tests` covers the closed abort/preparation dispatch without destructuring the error outside its owner. `tests/exdev.rs` retains only public worker/observation/serde/source-contract behavior. No production visibility is widened to make a test compile.

The proof/API tests are source-contract tests plus runtime wrong-token cases: there is no public constructor, `Clone`, or serde implementation for `ExdevTransitionProof`, `ExdevTransitionAuthorization`, Plan 2 `MirrorConfirmationProof`, `VerifiedTerminalControl`, or `SourceClaimProof`. The authorizing object is the whole `ExdevTransitionProof`: it owns `OwnedLockedReceipt<ExdevReceipt>` (the actual no-follow lock, receipt snapshot, and substrate sync witnesses) together with one subordinate authorization binding installation/operation/item/revision/identity/edge and verifier-issued edge facts. `ExdevTransitionAuthorization` alone is never accepted by an EXDEV API and cannot be extracted through a public/crate sibling accessor; only consuming `OwnedLockedReceipt::advance(self, ..., authorization)` sees it after the proof destructures. A genuine token from another bundle, revision, or edge is rejected, and a genuine token cannot be applied twice. The replay test publishes the frozen ordered case manifest `other-bundle`, `other-revision`, `other-edge`; the single-use test is a separate compile/API gate. A transition error returns the same owning lock/receipt/authorization plus exact pre-write unchanged or post-write reread truth. No caller-supplied bool/hash can advance a fixed or adjacent receipt; `ClaimedExdevTransaction` exposes no `AtomicReceiptFile`, `ClaimedChildLock`, or control handle, and `PreparedExdevReservation` exposes no `ControlBundle`/`ClaimedControlBundle` to App. `ExdevReceipt` derives serde only through Plan 2's custom `RawUnixPath`/`RawUnixName` deserializers: noncanonical Base64 and invalid path components are rejected, while child names additionally reject `/`, NUL, empty, `.` and `..`; Plan 4 adds no raw-type serde implementation or unchecked constructor.

- [ ] **Step 2: Run the exact RED gates**

```bash
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_transition_rejects_backward_revision_or_unproved_fact
python3 scripts/run_exact_test.py --test exdev --name exdev_prepare_reserves_one_claimed_outer_before_adjacent
python3 scripts/run_exact_test.py --test exdev --name exdev_prepare_collision_regenerates_item_id_without_opening_existing
python3 scripts/run_exact_test.py --test exdev --name exdev_adjacent_collision_cleans_fixed_before_regenerating
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_prepare_failure_returns_draft_fixed_partial_or_complete_adjacent_ownership
python3 scripts/run_exact_test.py --lib --name mutation_ops::tests::exdev_preparation_failure_enters_worker_dispatch_with_ownership
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_adjacent_open_failure_retains_partial_owned_capabilities_and_reread_truth
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_unproved_reservation_cleanup_is_cleanup_required
python3 scripts/run_exact_test.py --test exdev --name exdev_prepared_waits_for_fence_ack_before_source_or_destination_effect
python3 scripts/run_exact_test.py --test exdev --name exdev_pre_ack_cancel_removes_only_verified_reservations
python3 scripts/run_exact_test.py --lib --name mutation_ops::tests::exdev_prepared_abort_consumes_owner_verified_adjacent_absence_and_fixed_typestate
python3 scripts/run_exact_test.py --test exdev --name exdev_source_claim_is_nested_in_the_same_outer_control
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_cannot_fabricate_mirror_or_terminal_authority
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_transition_rejects_genuine_token_from_other_bundle_revision_or_edge --serial --case-matrix exdev-transition-replay --expect-case other-bundle --expect-case other-revision --expect-case other-edge
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_authorizing_facts_cannot_outlive_claim_or_locked_receipt_snapshot
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_consumed_transition_token_cannot_be_used_twice
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_transition_failure_retains_lock_authorization_and_reread_truth
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_terminal_remove_consumes_original_claim_and_retains_failure_typestate
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_adjacent_terminal_remove_failure_retains_fixed_claim_lock_stage_and_disk_truth
python3 scripts/run_exact_test.py --test exdev --name exdev_receipt_rejects_forged_raw_path_capability
python3 scripts/run_exact_test.py --test exdev --name exdev_raw_receipt_and_control_handles_are_not_public
python3 scripts/run_exact_test.py --test exdev --name exdev_unfinished_bundle_is_discovered_from_different_cwd --serial
python3 scripts/run_exact_test.py --test exdev --name exdev_pending_stream_error_is_incomplete_not_absent
```

Expected: every command discovers exactly one test and FAILS because the EXDEV host state, typed bundle, and worker preparer do not exist.

- [ ] **Step 3: Implement the private receipt and claimed preparation types**

Define exactly these module boundaries; every field shown without `pub` remains inaccessible outside `exdev.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ExdevState {
    Prepared,
    PayloadReady,
    PublishIntent,
    DestinationPublishedSourceRemovalPending,
    Committed,
    CleanupRequired,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExdevErrorKind {
    InvalidReceipt,
    UnsupportedObject,
    StaleObservation,
    InUse,
    Collision,
    PermissionDenied,
    OutOfSpace,
    Filesystem,
    CleanupRequired,
    Indeterminate,
}

#[derive(Debug)]
pub struct ExdevError {
    kind: ExdevErrorKind,
    escaped_detail: String,
}

impl ExdevError {
    pub fn kind(&self) -> ExdevErrorKind;
    pub fn escaped_detail(&self) -> &str;
    pub(crate) fn new(kind: ExdevErrorKind, escaped_detail: String) -> Self;
}

impl std::fmt::Display for ExdevError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl std::error::Error for ExdevError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExdevReceipt {
    schema: u32,
    item_id: ItemId,
    operation_id: OperationId,
    revision: u64,
    state: ExdevState,
    source: RawUnixPath,
    source_parent: PathIdentity,
    source_identity: PathIdentity,
    destination: RawUnixPath,
    destination_parent: PathIdentity,
    adjacent_bundle: RawUnixPath,
    staged_identity: Option<PathIdentity>,
    published_identity: Option<PathIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum ExdevPersistedObjectFact {
    Unobserved,
    Present(PathIdentity),
    Absent { parent: PathIdentityKey },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum ExdevAdjacentRootFact {
    Planned { destination_parent: PathIdentityKey },
    Present(PathIdentityKey),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExdevControlState {
    phase: ExdevState,
    source: RawUnixPath,
    source_parent: PathIdentity,
    expected_source: PathIdentity,
    destination: RawUnixPath,
    destination_parent: PathIdentity,
    adjacent_bundle: RawUnixPath,
    adjacent_root: RawUnixPath,
    adjacent_root_fact: ExdevAdjacentRootFact,
    adjacent_bundle_name: RawUnixName,
    adjacent_bundle_identity: Option<PathIdentityKey>,
    adjacent_receipt_identity: Option<PathIdentityKey>,
    adjacent_lock_identity: Option<PathIdentityKey>,
    source_fact: ExdevPersistedObjectFact,
    staged_fact: ExdevPersistedObjectFact,
    published_fact: ExdevPersistedObjectFact,
    private_tombstone_fact: ExdevPersistedObjectFact,
    source_claim_revision: Option<u64>,
    confirmed_adjacent_revision: Option<u64>,
    confirmed_adjacent_sha256: Option<[u8; 32]>,
    confirmed_adjacent_identity: Option<PathIdentityKey>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExdevEdge {
    PreparedToPayloadReady,
    PayloadReadyToPublishIntent,
    PublishIntentToDestinationPublished,
    DestinationPublishedToCommitted,
    PreparedToCleanupRequired,
    PayloadReadyToCleanupRequired,
    PublishIntentToCleanupRequired,
    DestinationPublishedToCleanupRequired,
    CleanupRequiredToCommitted,
    PreparedToIndeterminate,
    PayloadReadyToIndeterminate,
    PublishIntentToIndeterminate,
    DestinationPublishedToIndeterminate,
    CleanupRequiredToIndeterminate,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExdevFixedMirrorEdge {
    PreparedToPayloadReady,
    PayloadReadyToPublishIntent,
    PublishIntentToDestinationPublished,
    DestinationPublishedToCommitted,
    PreparedToCleanupRequired,
    PayloadReadyToCleanupRequired,
    PublishIntentToCleanupRequired,
    DestinationPublishedToCleanupRequired,
    CleanupRequiredToCommitted,
    PreparedToIndeterminate,
    PayloadReadyToIndeterminate,
    PublishIntentToIndeterminate,
    DestinationPublishedToIndeterminate,
    CleanupRequiredToIndeterminate,
}

pub(crate) fn validate_exdev_fixed_mirror_intent_edge(
    current: &ExdevControlState,
    next: &ExdevControlState,
    edge: ExdevFixedMirrorEdge,
    current_adjacent: &crate::trusted_fs::AdjacentReceiptFacts<'_>,
    next_intent: &MirrorIntent,
) -> Result<(), ExdevError>;
pub(crate) fn validate_exdev_mirror_confirmation(
    current: &ExdevControlState,
    next: &ExdevControlState,
    expected_intent: &MirrorIntent,
    confirmed_adjacent: &AdjacentReceiptFacts<'_>,
) -> Result<(), ExdevError>;
struct ExdevTransitionBinding {
    installation_id: InstallationId,
    operation_id: OperationId,
    item_id: ItemId,
    current_revision: u64,
    edge: ExdevEdge,
    bundle_identity: PathIdentityKey,
    receipt_identity: PathIdentityKey,
}
struct ExdevReceiptSyncWitness;
struct ExdevParentSyncWitness;
enum VerifiedPathFact {
    Present(PathIdentityKey),
    Absent { parent: PathIdentityKey },
    NotApplicable,
}
struct VerifiedExdevDiskFacts {
    source: VerifiedPathFact,
    destination_parent_snapshot: PathIdentityKey,
    staged: VerifiedPathFact,
    published: VerifiedPathFact,
    private_tombstone: VerifiedPathFact,
    source_claim_revision: Option<u64>,
    receipt_sync: ExdevReceiptSyncWitness,
    parent_sync: ExdevParentSyncWitness,
}
pub(crate) struct ExdevIndeterminateExpectation {
    expected_source: Option<PathIdentityKey>,
    expected_staged: Option<PathIdentityKey>,
    expected_published: Option<PathIdentityKey>,
    expected_private_tombstone: Option<PathIdentityKey>,
}
pub(crate) enum ExdevTransitionExpectation {
    PayloadReady { staged: PathIdentity },
    PublishIntent { staged: PathIdentity, destination_parent: PathIdentity },
    DestinationPublished { destination: PathIdentity },
    Committed { destination: PathIdentity, private_source_absent: RawUnixPath },
    CleanupRequired { destination: Option<PathIdentity>, retained_private_source: Option<PathIdentity> },
    Indeterminate(ExdevIndeterminateExpectation),
}
pub(crate) struct ExdevTransitionAuthorization {
    binding: ExdevTransitionBinding,
    verified_disk_facts: VerifiedExdevDiskFacts,
}
struct ClaimedExdevAdjacent {
    root: TrustedDir,
    bundle_dir: TrustedDir,
    transition: AtomicReceiptFile,
    transition_lock: ClaimedChildLock,
    binding: ExdevAdjacentBinding,
}
struct ExdevAdjacentRemainder {
    root: TrustedDir,
    bundle_dir: TrustedDir,
    binding: ExdevAdjacentBinding,
}
pub(crate) struct ExdevAdjacentBinding {
    installation_id: InstallationId,
    operation_id: OperationId,
    item_id: ItemId,
    bundle_name: RawUnixName,
    root_identity: PathIdentityKey,
    bundle_identity: PathIdentityKey,
}
struct ExdevAdjacentObservation {
    root_path: RawUnixPath,
    root_identity: PathIdentityKey,
    bundle_name: RawUnixName,
    bundle_path: RawUnixPath,
    bundle_identity: PathIdentityKey,
    receipt_identity: PathIdentityKey,
}
enum ExdevAdjacentOpenStage { Root, Bundle, Lock, Receipt, Binding }
pub(crate) struct PartialClaimedExdevAdjacent {
    root: Option<TrustedDir>,
    bundle_dir: Option<TrustedDir>,
    transition: AtomicReceiptCreation,
    transition_lock: Option<ClaimedChildLock>,
    created_name: Option<RawUnixName>,
    binding: Option<ExdevAdjacentBinding>,
}
enum ExdevAdjacentChildDiskTruth {
    NotObserved,
    Absent,
    Present(PathIdentityKey),
    Unreadable { escaped_error: String },
}
struct ExdevAdjacentLocationDiskTruth {
    bundle: PathIdentityKey,
    lock: ExdevAdjacentChildDiskTruth,
    receipt: ExdevAdjacentChildDiskTruth,
}
enum ExdevAdjacentDiskTruth {
    Missing,
    InStaging(ExdevAdjacentLocationDiskTruth),
    InClaims(ExdevAdjacentLocationDiskTruth),
    InBoth {
        staging: ExdevAdjacentLocationDiskTruth,
        claims: ExdevAdjacentLocationDiskTruth,
    },
    StreamUnreadable { escaped_error: String },
}
enum ExdevAdjacentOpenFailure {
    NoAdjacentEffect { stage: ExdevAdjacentOpenStage, source: ExdevError },
    AdjacentOwned {
        recovery: PartialClaimedExdevAdjacent,
        stage: ExdevAdjacentOpenStage,
        disk_truth: ExdevAdjacentDiskTruth,
        source: ExdevError,
    },
}
pub(crate) struct ClaimedExdevTransaction {
    control: ClaimedControlBundle,
    adjacent: ClaimedExdevAdjacent,
}
pub(crate) struct ExdevTransactionRemainder {
    control: ClaimedControlBundle,
    adjacent: ExdevAdjacentRemainder,
}
pub(crate) struct ExdevTransitionProof {
    transaction: ExdevTransactionRemainder,
    locked_receipt: OwnedLockedReceipt<ExdevReceipt>,
    authorization: ExdevTransitionAuthorization,
}
pub(crate) struct ExdevAdvanceError {
    transaction: ExdevTransactionRemainder,
    locked_failure: OwnedLockedReceiptAdvanceError<ExdevReceipt>,
}
pub(crate) struct ExdevVerifyError {
    transaction: ClaimedExdevTransaction,
    source: ExdevError,
}
enum ExdevPreparationOwnership {
    Draft(ItemDraft),
    FixedReservationOwned { draft: ItemDraft, failure: ControlReservationFailure },
    Fixed { draft: ItemDraft, control: ClaimedControlBundle },
    PartialAdjacentOwned {
        draft: ItemDraft,
        control: ClaimedControlBundle,
        recovery: PartialClaimedExdevAdjacent,
    },
    AdjacentOwned { draft: ItemDraft, transaction: ClaimedExdevTransaction },
}
pub(crate) struct PrepareExdevError {
    ownership: ExdevPreparationOwnership,
    source: PreparationError,
}
#[doc(hidden)]
pub struct PreparedExdevReservation {
    transaction: ClaimedExdevTransaction,
    source_identity: PathIdentity,
    destination_parent_identity: PathIdentity,
}
pub enum ExdevObservationState { Recoverable, NeedsCleanup, InUse, Indeterminate, InspectOnly }
pub enum ExdevObservation {
    Verified(VerifiedExdevObservation),
    InspectOnly { name: ObservedRawName, escaped_error: String },
}
pub struct VerifiedExdevObservation {
    control: ControlBundle,
    facts: ExdevObservedFacts,
}
enum ExdevAdjacentObservedTruth {
    Verified { receipt: ExdevReceipt, observed: ExdevAdjacentObservation },
    Missing,
    ReceiptAbsent { bundle: PathIdentityKey, lock: PathIdentityKey },
    ReceiptAndLockAbsent { bundle: PathIdentityKey },
    BundleAbsent,
    InBoth { staging: PathIdentityKey, claims: PathIdentityKey },
    Unreadable { escaped_error: String },
}
pub(crate) struct ExdevObservedFacts {
    envelope: ControlEnvelope,
    control_identity: PathIdentityKey,
    adjacent_truth: ExdevAdjacentObservedTruth,
    state: ExdevObservationState,
}
pub struct ExdevObservationStream {
    installation_id: InstallationId,
    controls: PendingControlStream,
}
pub struct ExdevStore<'a> { state_root: &'a StateRoot }
enum ExdevReconcileOwnership {
    PreClaim(VerifiedExdevObservation),
    Fixed {
        facts: ExdevObservedFacts,
        control: ClaimedControlBundle,
    },
    AdjacentOwned {
        facts: ExdevObservedFacts,
        transaction: ClaimedExdevTransaction,
    },
    PartialAdjacentOwned {
        facts: ExdevObservedFacts,
        control: ClaimedControlBundle,
        recovery: PartialClaimedExdevAdjacent,
    },
}
enum ExdevReconcileOutcome {
    Unchanged(VerifiedExdevObservation),
    StillPending(ExdevReconcileOwnership),
    Completed,
}
struct ExdevReconcileFailure {
    ownership: ExdevReconcileOwnership,
    stage: ExdevReconcileStage,
    disk_truth: ExdevReconcileDiskTruth,
    source: ExdevError,
}
enum ExdevReconcileStage { Claim, AdjacentOpen, BundleClaimRename, ParentSync, ReceiptVerify, Classify }
enum ExdevReconcileDiskTruth {
    PreClaimUnchanged { adjacent: ExdevAdjacentDiskTruth },
    FixedClaimed { adjacent: ExdevAdjacentDiskTruth },
}

pub(crate) enum ExdevAdjacentRemovalRecovery {
    ReceiptPresent {
        control: ClaimedControlBundle,
        root: TrustedDir,
        bundle_dir: TrustedDir,
        transition: AtomicReceiptFile,
        transition_lock: ClaimedChildLock,
        binding: ExdevAdjacentBinding,
    },
    ReceiptAbsent {
        control: ClaimedControlBundle,
        root: TrustedDir,
        bundle_dir: TrustedDir,
        transition_lock: ClaimedChildLock,
        binding: ExdevAdjacentBinding,
    },
    LockAbsent {
        control: ClaimedControlBundle,
        root: TrustedDir,
        bundle_dir: TrustedDir,
        binding: ExdevAdjacentBinding,
    },
    BundleAbsent {
        control: ClaimedControlBundle,
        root: TrustedDir,
        binding: ExdevAdjacentBinding,
    },
}
pub(crate) enum ExdevAdjacentTerminalRemoveStage {
    VerifyCommitted,
    ReceiptRemove,
    LockRemove,
    BundleRemove,
    ParentSync,
}
pub(crate) enum ExdevAdjacentTerminalDiskTruth {
    ReceiptAndLockPresent {
        bundle: PathIdentityKey,
        receipt: PathIdentityKey,
        lock: PathIdentityKey,
    },
    ReceiptAbsentLockPresent {
        bundle: PathIdentityKey,
        lock: PathIdentityKey,
    },
    ReceiptAndLockAbsentBundlePresent { bundle: PathIdentityKey },
    BundleAbsent,
    Unreadable { escaped_error: String },
}
pub(crate) struct ExdevAdjacentTerminalRemoveFailure {
    recovery: ExdevAdjacentRemovalRecovery,
    preconditions: ExdevTerminalPreconditions,
    stage: ExdevAdjacentTerminalRemoveStage,
    disk_truth: ExdevAdjacentTerminalDiskTruth,
    source: ExdevError,
}

struct ExdevPreparedAbortParentSyncWitness;
enum ExdevPreparedAbortAdjacentFact {
    AbsentAfterOwnedRemoval {
        removed_bundle_identity: PathIdentityKey,
        parent_sync: ExdevPreparedAbortParentSyncWitness,
    },
    ForeignCollisionUnchanged {
        observed_bundle_identity: PathIdentity,
    },
}
#[doc(hidden)]
pub struct ExdevPreparedAbortFacts {
    trusted_adjacent_root: TrustedDir,
    bundle_name: RawUnixName,
    adjacent_root_identity: PathIdentityKey,
    adjacent: ExdevPreparedAbortAdjacentFact,
}

struct ExdevTerminalSyncWitness;
pub(crate) struct ExdevTerminalPreconditions {
    trusted_source_parent: TrustedDir,
    source_name: RawUnixName,
    trusted_destination_parent: TrustedDir,
    destination_name: RawUnixName,
    expected_destination: PathIdentity,
    trusted_private_parent: TrustedDir,
    private_tombstone_name: RawUnixName,
    committed_adjacent_revision: u64,
    committed_adjacent_sha256: [u8; 32],
    committed_adjacent_identity: PathIdentityKey,
}
#[doc(hidden)]
pub struct ExdevTerminalExpectation {
    preconditions: ExdevTerminalPreconditions,
    trusted_adjacent_root: TrustedDir,
    adjacent_bundle_name: RawUnixName,
    removed_adjacent_identity: PathIdentityKey,
    affected_parents_synced: ExdevTerminalSyncWitness,
}

pub(crate) struct ExdevFixedTerminalReady {
    control: ClaimedControlBundle,
    terminal: ExdevTerminalExpectation,
}

pub(crate) struct ExdevFixedTerminalVerifyFailure {
    failure: TerminalVerifyFailure,
}

impl DurableReceipt for ExdevReceipt {
    type Proof = ExdevTransitionAuthorization;
    fn revision(&self) -> u64;
    fn validate_next(&self, next: &Self, proof: &ExdevTransitionAuthorization) -> Result<(), TrustedFsError>;
}

impl ClaimedExdevTransaction {
    pub(crate) fn verify_transition(
        self,
        next: &ExdevReceipt,
        expectation: &ExdevTransitionExpectation,
    ) -> Result<ExdevTransitionProof, ExdevVerifyError>;
    pub(crate) fn confirm_pending_mirror(&mut self) -> Result<(), ExdevError>;
    pub(crate) fn remove_adjacent_terminal(
        self,
        preconditions: ExdevTerminalPreconditions,
    ) -> Result<ExdevFixedTerminalReady, ExdevAdjacentTerminalRemoveFailure>;
}

impl ExdevFixedTerminalReady {
    pub(crate) fn verify_fixed_terminal(
        self,
    ) -> Result<VerifiedTerminalControl, ExdevFixedTerminalVerifyFailure>;
}

impl ExdevFixedTerminalVerifyFailure {
    pub(crate) fn source(&self) -> &crate::state_root::StateRootError;
    pub(crate) fn retry(self) -> Result<VerifiedTerminalControl, ExdevFixedTerminalVerifyFailure>;
}

pub(crate) fn validate_exdev_prepared_abort(
    current: &ExdevControlState,
    facts: &ExdevPreparedAbortFacts,
) -> Result<(), ExdevError>;

pub(crate) fn validate_exdev_terminal(
    current: &ExdevControlState,
    facts: &ExdevTerminalExpectation,
) -> Result<(), ExdevError>;

impl ClaimedExdevAdjacent {
    fn create_new_bound(
        state: &StateRoot,
        parent: &TrustedDir,
        control: &mut ClaimedControlBundle,
        initial: &ExdevReceipt,
    ) -> Result<Self, ExdevAdjacentOpenFailure>;
    fn open_existing_bound(
        state: &StateRoot,
        parent: &TrustedDir,
        control: &mut ClaimedControlBundle,
        observed: &ExdevAdjacentObservation,
    ) -> Result<Self, ExdevAdjacentOpenFailure>;
}

impl ExdevTransitionProof {
    pub(crate) fn advance_receipt(
        self,
        expected_revision: u64,
        next: &ExdevReceipt,
    ) -> Result<ClaimedExdevTransaction, ExdevAdvanceError>;
}

impl<'a> ExdevStore<'a> {
    pub fn new(state_root: &'a StateRoot) -> Self;
    pub fn observations(&self) -> Result<ExdevObservationStream, ExdevError>;
    pub(crate) fn reconcile_for_worker(
        &self,
        observed: VerifiedExdevObservation,
    ) -> ItemExecutionResult;
    fn reconcile(
        &self,
        observed: VerifiedExdevObservation,
    ) -> Result<ExdevReconcileOutcome, ExdevReconcileFailure>;
}

impl PrepareExdevError {
    pub(crate) fn source(&self) -> &PreparationError;
}

pub(crate) fn abort_prepared_exdev(
    context: &MutationContext,
    plan: &ItemPlan,
    prepared: PreparedExdevReservation,
) -> PreparedAbortOutcome;

impl PreparedExdevReservation {
    pub(crate) fn recovery_ref(&self) -> ControlRecoveryRef;
}

pub(crate) fn recover_exdev_execution_panic(
    context: &MutationContext,
    plan: &ItemPlan,
    claimed: ClaimedControlBundle,
) -> ItemExecutionResult;

pub(crate) fn recover_exdev_startup(
    context: &MutationContext,
    claimed: ClaimedControlBundle,
) -> StartupRecoveryNotice;

pub(crate) fn recover_exdev_preparation_failure(
    context: &MutationContext,
    failure: PrepareExdevError,
) -> PreparedAbortOutcome;

impl ExdevReconcileFailure {
    fn stage(&self) -> ExdevReconcileStage;
    fn disk_truth(&self) -> &ExdevReconcileDiskTruth;
    fn source(&self) -> &ExdevError;
}

impl ExdevStore<'_> {
    fn retry_reconcile(
        &self,
        failure: ExdevReconcileFailure,
    ) -> Result<ExdevReconcileOutcome, ExdevReconcileFailure>;
}

impl ExdevAdjacentTerminalRemoveFailure {
    pub(crate) fn stage(&self) -> ExdevAdjacentTerminalRemoveStage;
    pub(crate) fn disk_truth(&self) -> &ExdevAdjacentTerminalDiskTruth;
    pub(crate) fn source(&self) -> &ExdevError;
    pub(crate) fn retry(self) -> Result<ExdevFixedTerminalReady, ExdevAdjacentTerminalRemoveFailure>;
}

pub(crate) fn resume_exdev_terminal_from_fixed(
    context: &MutationContext,
    claimed: ClaimedControlBundle,
) -> ItemExecutionResult;

impl Iterator for ExdevObservationStream {
    type Item = Result<ExdevObservation, ExdevError>;
}
```

Extend the concrete Plan 2 state with `ControlState::Exdev { host: ExdevControlState, source_claim: Option<SourceClaimState>, mirror_intent: Option<MirrorIntent> }`, `TerminalExpectation::Exdev(ExdevTerminalExpectation)`, `PreparedControlAbortExpectation::Exdev(ExdevPreparedAbortFacts)`, `PreparedReservation::Exdev(PreparedExdevReservation)`, and `HostMirrorEdge::Exdev(ExdevFixedMirrorEdge)`. The public abort/terminal fact wrappers are `#[doc(hidden)]` only to satisfy the public enum surface; fields/constructors remain owner-private, non-Clone, and non-serde. `state_root.rs` keeps exhaustive concrete matches and delegates private host-field validation to `exdev::validate_exdev_fixed_mirror_intent_edge`, `validate_exdev_mirror_confirmation`, `validate_exdev_prepared_abort`, and `validate_exdev_terminal`; only those hard-coded owner-module successes can authorize fixed transitions or removal. `ExdevStore` is discovery/reconciliation only: it has no `reserve`, raw receipt mutation, or constructor returning a control handle.

The fixed `ExdevControlState` is the recovery authority, not merely an adjacent-receipt pointer. It persists immutable raw source/destination/adjacent selectors and parent/object identities, the validated adjacent child name/root fact, current source/staged/published/private-tombstone facts, source-claim revision, and confirmed adjacent revision/hash/bundle identity. The initial fixed envelope uses `ExdevAdjacentRootFact::Planned { destination_parent }`; the first locked confirmation alone may change it to `Present(identity)`. The adjacent `ExdevReceipt` mirrors local execution detail but is never the sole copy of source-loss or terminal truth. `recover_exdev_startup(context, claimed)` is the Plan 2 closed-startup-dispatch owner entry: it receives no `ItemPlan`, reopens only immutable selectors stored in this fixed state, routes through the same fixed-only/adjacent reconcile and terminal-resume core, and emits a bounded non-authorizing `StartupRecoveryNotice` after durable truth is terminal or conservatively retained. Startup can therefore classify/reconcile an absent, corrupt, partially removed, or replaced adjacent receipt from the claimed fixed control without inventing a path or accepting same-bytes content from a different bundle. Durable `Indeterminate` is written only through one of the five exact per-source mirrored edges when both fixed and adjacent receipts remain writable/verified; if the adjacent receipt is unreadable/missing, Indeterminate remains an observation/outcome classification without a fabricated write. Cleanup-required receipt transitions use the four exact source-phase-to-cleanup edges; cleanup retry uses the exact `CleanupRequiredToCommitted` edge. Each has a matching `ExdevFixedMirrorEdge` and owner validator, so no `AnyTo*` or proofless fixed transition exists.

`ExdevError` is the public, private-field diagnostic returned by public observation methods; construction is crate-private, its escaped detail is bounded to 512 bytes, and it contains no authority. Owner-module consuming errors keep capability state opaque: sibling callers may borrow only bounded public diagnostics and must pass the whole error to an exdev-owned recovery free function; no `into_parts`, public/crate-visible ownership enum, proof, control, adjacent lock, precondition, or terminal fact crosses the owner boundary. Extend Plan 2's crate-private `ItemPreparationFailureInner` with concrete `Exdev(PrepareExdevError)` and `ExdevCleanup(PrepareExdevCleanupError)` variants; the public `ItemPreparationFailure` wrapper continues to expose only kind/detail, while the central dispatch transfers the opaque whole value to `recover_exdev_preparation_failure` or `recover_exdev_cleanup_preparation_failure`. `ExdevStore::new` is the only constructor and merely borrows an existing `StateRoot`; it does not initialize, reserve, or claim anything.

`VerifiedPathFact` distinguishes an observed object, a no-follow proven absence whose parent identity was also revalidated/synced, and a fact that the particular edge does not require. `Option<PathIdentityKey>` and caller booleans are forbidden because they conflate absence with not-applicable. The Committed verifier accepts only destination/published `Present`, original source `Absent`, private tombstone `Absent`, the matching source-claim revision, and actual receipt/parent sync witnesses; earlier edges name their exact Present/Absent/NotApplicable matrix. The whole fact set remains private inside the owning transition proof.

- [ ] **Step 4: Implement worker-side create-new reservation and collision handling**

Define `pub(crate) exdev::prepare_exdev(context, operation_id, candidate_item_id, draft) -> Result<(ItemPlan, PreparedExdevReservation, MutationFenceSpec), PrepareExdevError>` in `src/exdev.rs`; `mutation_ops.rs` only dispatches to that factory and wraps its returned reservation in the concrete enum. This keeps construction/destruction of private transaction fields inside their owning module. It runs only during the Plan 2 worker's `MutationIntentBody::Paths` preflight after no-follow source/destination-parent capture proves EXDEV and a supported file/symlink kind. Every error owns the capability-bearing state reached so far: before the fixed reservation it returns `Draft`; a post-effect error inside `reserve_control` returns `FixedReservationOwned { draft, failure: ControlReservationFailure }`; only after reserve succeeds can a later error return `Fixed`; after any adjacent root/bundle/receipt/lock effect it returns `PartialAdjacentOwned { control, recovery }`; only a fully verified adjacent capability returns `AdjacentOwned`. `ExdevAdjacentOpenFailure::NoAdjacentEffect` is legal only before any adjacent creation/rename/lock effect; otherwise `AdjacentOwned` retains optional root/bundle/receipt/lock, created name, binding, stage, and no-follow reread truth. The preparer may collapse a partial failure back to `Fixed` only after it proves removal plus every parent sync. No error returns only a naked cause or asks its caller to reopen an `ItemId` to recover ownership. Plan 2's `recover_preparation_failure` passes the whole EXDEV variant to `recover_exdev_preparation_failure`, and `abort_prepared_item` passes the opaque prepared variant to `abort_prepared_exdev`; these owner-module routines are the only pre-ack destruction path. `FixedReservationOwned` calls `ControlReservationFailure::abort` without separating its ownership from stage/truth; complete reservations are reversed and parents synced before `ReleasedNoEffect`, otherwise durable residue is retained/classified. The preparation-failure and pre-ack-cancel exact tests invoke those central dispatchers rather than read private fields.

Canonicalize the complete initial adjacent receipt before creating either object. Create and sync exactly one fixed `ControlEnvelope { protocol: ExdevMoveV1, state: ControlState::Exdev { .. } }` first; its host records the exact expected adjacent raw path, validated bundle name, destination-parent identity, `ExdevAdjacentRootFact::Planned`, and `MirrorIntent { adjacent_revision, canonical_receipt_sha256 }` before any destination-adjacent object is created. Retain the returned owning `ClaimedControlBundle`, then call private `ClaimedExdevAdjacent::create_new_bound(&mut control, ...)` to create-new exactly those canonical bytes. That factory borrows the control only for header/path/identity binding and returns a fully owning adjacent capability; Plan 2's borrowing `TrustedAdjacentRoot<'a>` remains exclusive to temporary `SourceClaim` operations and is never stored beside the control it borrows. Move the independent owning control and adjacent capability into one safe-Rust `ClaimedExdevTransaction`. While that transaction retains its lock, re-read revision/hash/identity, prove the file and parent syncs, build the owner-approved next fixed state that changes the planned root to `Present` and records confirmed bundle/receipt/lock identities, pass its lifetime-bound `AdjacentReceiptFacts` plus that next state to `ClaimedControlBundle::confirm_mirror`, and consume the returned fixed proof in the matching confirmation transition before considering the reservation prepared. Put the one transaction into `PreparedReservation::Exdev`; only after every target is reserved and initially mirror-confirmed may the worker freeze `OperationRequest`/`ItemPlan`, emit non-droppable `Prepared`, and wait for `FenceInstalled`.

`ExdevStore::observations` streams Plan 2 `PendingControlStream` without collecting. The returned stream copies the current `InstallationId` from its borrowed `StateRoot` at construction, so every adjacent owner/binding validation is tied to the same installation even after the `ExdevStore` method returns; it never obtains installation identity from a receipt or caller string. A verified EXDEV record retains the exact read-only `ControlBundle`, envelope revision, stable no-follow fixed identity, and a closed `ExdevAdjacentObservedTruth`: verified receipt+observation, wholly missing, receipt-absent with lock, receipt-and-lock-absent, bundle-absent, two-location contradiction, or unreadable. Thus a fixed control whose adjacent receipt is corrupt/missing or whose terminal deletion was interrupted remains discoverable and can route to owner recovery; discovery never fabricates a verified receipt. Reconciliation consumes that exact observation and then calls `try_claim` on that exact handle. An inspect-only item never enables mutation. A name-less `readdir`/stream failure marks the overall scan incomplete, so no caller may convert a partial scan into global absence, uniqueness, completeness, or a count. A verified observation already yielded before that error remains only a non-authorizing exact selector: `cleanup_action`/`reconcile` must independently consume its exact control handle, reacquire every still-existing lock, and revalidate protocol/revision/identity/destination/tombstone facts before any effect. They never rely on the suffix being absent. There is no `pending() -> Vec<_>`, `reconcile(ItemId)`, or display-path reopen API.

The private `ExdevStore::reconcile`/`retry_reconcile` engine returns the owner-only consuming `ExdevReconcileOutcome`, never a newly fabricated read-only observation after it has claimed or advanced state. A pre-claim no-effect path may return `Unchanged` with the original exact observation; a claimed but nonterminal path returns `StillPending` with the complete owning fixed/adjacent recovery; a terminal path returns `Completed` after consuming/removing authority. Any failure retains the exact preclaim/claimed/partial/transaction ownership plus stage and reread truth inside exdev.rs. The only sibling-facing seam is `reconcile_for_worker -> ItemExecutionResult`, which drives the private engine/retry to a truthful outcome and optional owner-minted cleanup observation; it does not expose capability types, reopen by item ID, or require an impossible `ClaimedControlBundle -> ControlBundle` downgrade.

An `EEXIST` fixed or adjacent candidate is never opened. Regenerate that candidate `ItemId` only after every reservation created for it is verified removed and both parents are synced. If cleanup is not proved, keep the fixed control discoverable and terminalize the submission as `CleanupRequired`/`Indeterminate`; do not try a new ID. Cancellation, panic, disconnect, or shutdown before acknowledgement follows the same journal cleanup. No pre-ack path may copy user payload, publish a destination, claim a source, or unlink a source/destination/user payload; abort cleanup may unlink only its own verified fixed/adjacent reservations in reverse order and must sync every affected parent.

`ExdevPreparedAbortFacts` has two closed adjacent facts. `AbsentAfterOwnedRemoval` is minted only after reverse-order removal and parent sync of a bundle created by this preparation. `ForeignCollisionUnchanged` is minted only for an adjacent create-new `EEXIST` that was never opened for ownership or mutated: it retains the trusted parent/name plus the full no-follow observed identity, and the owner validator freshly requires that same snapshot while the fixed host is still its initial `Planned`, unconfirmed state. Either a freshly proved child absence or that exact unchanged foreign collision authorizes removal of only this attempt's fixed control. Missing trusted parent/name/observation capability, a replaced child, or unreadable reread truth fails closed and keeps the fixed control discoverable. The existing `exdev_adjacent_collision_cleans_fixed_before_regenerating` gate exercises this second path and asserts the foreign sentinel identity/content is unchanged.

For a fully prepared normal EXDEV reservation, `abort_prepared_exdev` first removes the exact transition receipt/lock/bundle in reverse order and syncs the trusted adjacent parent. It then constructs `ExdevPreparedAbortFacts` with `AbsentAfterOwnedRemoval`, consumes the still-initial fixed claim through `verify_prepared_abort(PreparedControlAbortExpectation::Exdev(facts))`, and consumes `VerifiedPreparedControlAbort::remove`. Verification and remove revalidate the exact closed adjacent fact while the original fixed lock remains owned. Failure keeps the owning typestate for retry/classification. Cleanup-only pre-ack abort is different: it releases the original unchanged fixed/adjacent transaction and does not remove either authority.

Before moving either `PreparedReservation::Exdev` or `PreparedReservation::ExdevCleanup` into execution, Plan 2 maps it to `PreparedRecoverySeed::Fixed(prepared.recovery_ref())`. An unwind consumes the exact seed through `StateRoot::claim_recovery_ref`, then the exhaustive `(plan.kind, ControlState::Exdev)` match calls `recover_exdev_execution_panic` for a normal move or `recover_exdev_cleanup_execution_panic` for `CleanupRetainedSource`; mismatches are `Indeterminate`. Each owner hook reopens only the fixed-state raw selectors/identities, claims fixed-before-adjacent, and reconciles the durable phase. The cleanup hook deliberately does not require the fresh cleanup report's `ItemPlan` IDs to equal the original fixed-control header IDs: fresh IDs validate only the new report/request, while the original IDs are checked against `CleanupSourceAction` and the claimed fixed state. It never searches by display path or ItemId.

- [ ] **Step 5: Verify GREEN and Plan 2 regressions**

Run: `cargo test --locked --test exdev --test mutation_worker --test state_root --test source_claim`

Expected: all preparation, proof opacity, ID-collision, pre-ack, different-cwd, and existing Plan 2 tests PASS.

- [ ] **Step 6: Commit the prepared EXDEV schema**

```bash
git add src/exdev.rs src/lib.rs src/state_root.rs src/mutation.rs src/mutation_ops.rs tests/exdev.rs tests/mutation_worker.rs
git commit -m "feat: prepare discoverable EXDEV reservations"
```

### Task 2: Regular-file EXDEV move

**Files:**
- Modify: `src/exdev.rs`
- Modify: `src/fs_ops.rs:84-95`
- Modify: `src/mutation_ops.rs`
- Modify: `tests/exdev.rs`
- Modify: `tests/fs_ops.rs:239-373`

- [ ] **Step 1: Write the failing regular-file and mirror protocol tests**

Add exact tests `exdev_regular_copies_chunks_and_preserves_mode_and_mtime`, `exdev_execution_consumes_prepared_reservation_without_new_control`, `same_fs_unexpected_exdev_after_fence_fails_without_late_reservation`, `unexpected_exdev_failed_no_effect_requires_source_restore_and_reservation_cleanup`, `exdev_regular_cancel_before_publish_removes_only_owned_staging`, `exdev_regular_target_competition_never_overwrites`, `exdev_regular_enospc_retains_source`, `exdev_regular_sync_failure_retains_source`, `exdev_fixed_mirror_intent_precedes_adjacent_advance`, `exdev_adjacent_wrapper_mints_confirmation_only_after_locked_reread_and_sync`, `exdev_mirror_hash_revision_or_identity_mismatch_blocks_source_claim`, `exdev_source_claim_advances_nested_same_outer_state`, `exdev_terminal_expectation_binds_original_authority_facts`, `exdev_terminal_verification_consumes_original_control_after_adjacent_cleanup`, and crate-unit `exdev::tests::exdev_regular_crash_after_each_transition_preserves_a_valid_copy`. The crash test exposes a frozen 12-case manifest: `payload-ready-fixed-intent`, `payload-ready-adjacent-write`, `payload-ready-fixed-confirm`, `publish-intent-fixed-intent`, `publish-intent-adjacent-write`, `publish-intent-fixed-confirm`, `destination-published-fixed-intent`, `destination-published-adjacent-write`, `destination-published-fixed-confirm`, `committed-fixed-intent`, `committed-adjacent-write`, `committed-fixed-confirm`. It lives in `src/exdev.rs` so it can reuse Plan 2's private deterministic fault controller without adding a production/test-feature surface; native different-device evidence is Task 6 only.

Also add exact `exdev_executor_panic_reclaims_exact_fixed_ref_and_reconciles` and `exdev_restart_resumes_terminal_remove_after_each_monotonic_phase` as public worker/restart integration tests. Add crate-unit exact `exdev::tests::exdev_mirror_confirmation_rejects_unapproved_private_host_delta` and `exdev::tests::exdev_terminal_facts_are_minted_after_adjacent_sync_and_revalidated_before_fixed_unlink`, because they exercise private owner validators/typestates and must not widen production visibility.

- [ ] **Step 2: Run the regular-file tests and confirm RED**

Run each target through the exact-test runner:

```bash
python3 scripts/run_exact_test.py --test exdev --name exdev_regular_copies_chunks_and_preserves_mode_and_mtime
python3 scripts/run_exact_test.py --test exdev --name exdev_execution_consumes_prepared_reservation_without_new_control
python3 scripts/run_exact_test.py --test exdev --name same_fs_unexpected_exdev_after_fence_fails_without_late_reservation
python3 scripts/run_exact_test.py --test exdev --name unexpected_exdev_failed_no_effect_requires_source_restore_and_reservation_cleanup
python3 scripts/run_exact_test.py --test exdev --name exdev_regular_cancel_before_publish_removes_only_owned_staging
python3 scripts/run_exact_test.py --test exdev --name exdev_regular_target_competition_never_overwrites
python3 scripts/run_exact_test.py --test exdev --name exdev_regular_enospc_retains_source
python3 scripts/run_exact_test.py --test exdev --name exdev_regular_sync_failure_retains_source
python3 scripts/run_exact_test.py --test exdev --name exdev_fixed_mirror_intent_precedes_adjacent_advance
python3 scripts/run_exact_test.py --test exdev --name exdev_adjacent_wrapper_mints_confirmation_only_after_locked_reread_and_sync
python3 scripts/run_exact_test.py --test exdev --name exdev_mirror_hash_revision_or_identity_mismatch_blocks_source_claim
python3 scripts/run_exact_test.py --test exdev --name exdev_source_claim_advances_nested_same_outer_state
python3 scripts/run_exact_test.py --test exdev --name exdev_terminal_expectation_binds_original_authority_facts
python3 scripts/run_exact_test.py --test exdev --name exdev_terminal_verification_consumes_original_control_after_adjacent_cleanup
python3 scripts/run_exact_test.py --test exdev --name exdev_executor_panic_reclaims_exact_fixed_ref_and_reconciles
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_mirror_confirmation_rejects_unapproved_private_host_delta
python3 scripts/run_exact_test.py --test exdev --name exdev_restart_resumes_terminal_remove_after_each_monotonic_phase --serial
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_terminal_facts_are_minted_after_adjacent_sync_and_revalidated_before_fixed_unlink
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_regular_crash_after_each_transition_preserves_a_valid_copy --serial --case-matrix exdev-transition-crash --expect-case payload-ready-fixed-intent --expect-case payload-ready-adjacent-write --expect-case payload-ready-fixed-confirm --expect-case publish-intent-fixed-intent --expect-case publish-intent-adjacent-write --expect-case publish-intent-fixed-confirm --expect-case destination-published-fixed-intent --expect-case destination-published-adjacent-write --expect-case destination-published-fixed-confirm --expect-case committed-fixed-intent --expect-case committed-adjacent-write --expect-case committed-fixed-confirm
```

Expected: FAIL because `execute_exdev_move` is not implemented.

- [ ] **Step 3: Implement the regular-file protocol**

Add the crate-private free dispatcher owned by `exdev.rs`; `mutation_ops.rs` never instantiates an engine with private fields:

```rust
pub(crate) fn execute_exdev_move(
    context: &MutationContext,
    request: &OperationRequest,
    plan: &ItemPlan,
    prepared: PreparedExdevReservation,
    cancel: &CancelToken,
    progress: &ProgressSink,
) -> ItemExecutionResult;
```

`move_item` consumes the prepared reservation and may not call `StateRoot::reserve_control`, initialize another adjacent root, or generate an ID. Only after `FenceInstalled`, copy regular files in bounded chunks, check `CancelToken` between chunks, preserve the declared mode/mtime contract, verify bytes/metadata, and sync payload and bundle.

Every adjacent transition consumes and reconstructs the one `ClaimedExdevTransaction` in this exact sequence: verify the currently confirmed adjacent receipt under its actual lock, call `control.verify_host_mirror_intent(..., HostMirrorEdge::Exdev(exact_edge), ...)`, and consume `ControlTransitionProof::MirrorIntentInstalled(proof)` to advance/sync fixed `ControlState::Exdev` with the next `MirrorIntent`; consume the transaction through `verify_transition` so the actual adjacent receipt and no-follow lock enter `OwnedLockedReceipt<ExdevReceipt>` together; consume the resulting `ExdevTransitionProof` through `advance_receipt`; then, while the reconstructed transaction still owns that same lock, call `confirm_pending_mirror`. That last method re-reads the locked adjacent receipt, verifies revision/hash/bundle identity plus receipt/parent sync, obtains Plan 2's lifetime-bound `AdjacentReceiptFacts`, constructs the concrete confirmation next state, passes both to `ClaimedControlBundle::confirm_mirror(adjacent_facts, &next_state)`, and immediately consumes the returned fixed proof in the exact confirmation transition. State root delegates the private host delta to `validate_exdev_mirror_confirmation`; no proof or raw bool/hash leaves the owning transaction. `DestinationPublishedSourceRemovalPending` must be confirmed in both places before source cleanup.

Then verify the published destination, no-follow reopen and full-identity-check the source parent recorded in the fixed state, and borrow only the transaction's original fixed control for `SourceClaim::acquire(context.state_root.as_ref(), &trusted_source_parent, control, plan, ClaimAction::ExdevSourceCleanup)`. Advance only `ControlState::Exdev.source_claim` in that same outer envelope, hold the claimed fixed control before every adjacent lock, and delete only the verified private tombstone. After the `Committed` adjacent transition is mirror-confirmed, verify source absence/destination identity and construct private `ExdevTerminalPreconditions` retaining the trusted source/destination/private parents, exact raw names/identities, and committed mirror. `ExdevAdjacentObservation::bundle_name` captures the validated `RawUnixName` directly from exact fd-relative enumeration/create; `ExdevAdjacentBinding::bundle_name` receives that capability without parsing `bundle_path`, revalidates it across staging-to-claims, and carries it through every remainder/removal typestate. Terminal retry never parses a path, enumerates by ID, or fabricates a child name. `remove_adjacent_terminal(preconditions)` verifies the committed receipt under its actual lock, uses Plan 2's owning `AtomicReceiptFile::remove_verified`, removes `claim.lock` through its retained name/identity while the fd remains held, removes the bundle through `root.unlink_verified_child(&binding.bundle_name, bundle_dir.identity(), true)`, and syncs the adjacent parent. Only after that actual sync does it mint the non-detachable `ExdevTerminalExpectation` containing the retained trusted adjacent parent/name/removed identity plus all preconditions, and return `ExdevFixedTerminalReady`. The key in `binding.bundle_identity` is for binding/equality, not a substitute for the full `PathIdentity` required by unlink. `ExdevAdjacentRemovalRecovery` is an exact monotonic typestate—`ReceiptPresent`, `ReceiptAbsent`, `LockAbsent`, or `BundleAbsent`—so no post-unlink error claims to own a consumed receipt/lock. Any failure returns `ExdevAdjacentTerminalRemoveFailure` with the matching typestate, original preconditions, exact stage, and reread truth; `retry` continues without rebuilding a consumed handle. Process restart uses `resume_exdev_terminal_from_fixed`, which claims the fixed control, uses its private adjacent raw name/full identities to reconstruct the exact monotonic deletion stage even when the receipt is absent/corrupt, and resumes without catalog or bare-ID lookup. Success consumes `ExdevFixedTerminalReady::verify_fixed_terminal`, which delegates owner validation and moves the original claim plus complete facts into `VerifiedTerminalControl`; only its consuming `remove` removes/syncs the fixed bundle and immediately revalidates those facts. There is no detachable terminal token, second fixed-control lock, or naked adjacent remove error. On cleanup failure return `DestinationCommittedSourceRetained`; a cleanup-specific intent never copies again.

Worker preflight selects `PreparedReservation::Exdev` only when the freshly captured source and destination-parent device identities differ, and only for `ObjectKind::RegularFile` or `ObjectKind::Symlink`; directory and special-file cross-device requests return `FailedNoEffect` before any fixed/adjacent reservation. A same-device request uses the already prepared same-filesystem path. If its post-ack atomic publish unexpectedly returns OS `EXDEV`, it creates no late EXDEV control, adjacent bundle, copy, or ID. It may return `FailedNoEffect` only after Plan 2's owning same-filesystem recovery proves the private claim was restored to the original source no-clobber, destination has no effect, and every prepared reservation was removed/parent-synced. A restore collision, cleanup/sync failure, or unreadable truth returns the retained owning recovery state as `CleanupRequired` or `Indeterminate`; it never drops the fixed claim or guesses no effect. The user may submit a fresh intent only after that truthful terminal report so background preflight can prepare the changed topology. No execution-time dispatch may call `execute_exdev_move` without an existing `PreparedReservation::Exdev`.

- [ ] **Step 4: Run regular-file and filesystem regression tests**

Run: `cargo test --locked --test exdev --test fs_ops --test mutation_ops --test mutation_worker`

Expected: PASS; cancellation/ENOSPC/sync failure retain the source, target competition does not overwrite, and legacy filesystem tests remain green.

- [ ] **Step 5: Commit regular-file EXDEV**

```bash
git add src/exdev.rs src/fs_ops.rs src/mutation_ops.rs tests/exdev.rs tests/fs_ops.rs
git commit -m "feat: move regular files safely across filesystems"
```

The “temporary `#[expect(dead_code)]`” shorthand in Task 3 means Plan 2's exact `#[cfg_attr(not(test), expect(dead_code, reason = "..."))]`. Task 3 removes each whole conditional attribute from read-only `ControlBundle::recovery_ref` and all three private-cleanup methods in the same edit that adds their first normal-library callers; the unit-test build never carried those expectations.

### Task 3: Symlink handling, reconciliation, and cleanup-only retry

**Files:**
- Modify: `src/exdev.rs`
- Modify: `src/operation.rs`
- Modify: `src/source_claim.rs`
- Modify: `src/state_root.rs`
- Modify: `src/mutation.rs`
- Modify: `src/mutation_ops.rs`
- Modify: `src/app.rs:1274-1704`
- Modify: `src/cli.rs`
- Modify: `tests/operation.rs`
- Modify: `tests/exdev.rs`
- Modify: `tests/mutation_worker.rs`
- Modify: `tests/app_mutation.rs`
- Modify: `tests/recovery_cli.rs`

- [ ] **Step 1: Write failing symlink, source-race, and reconciler tests**

Add exact tests `exdev_symlink_copies_raw_target_without_following`, `exdev_source_swap_before_claim_is_restored_or_retained`, `exdev_publish_intent_reconcile_matches_identity_not_path_existence`, `exdev_reconciler_consumes_exact_observed_control_before_adjacent`, `exdev_reconcile_rejects_replaced_observation_without_item_id_fallback`, `exdev_reconcile_preclaim_failure_returns_exact_observation`, `exdev_reconcile_posteffect_failure_retains_transaction_and_reread_truth`, `exdev_reconcile_crash_after_claim_rename_reopens_exact_claims_bundle`, `exdev_reconcile_staging_and_claims_duplicate_is_indeterminate_without_mutation`, `exdev_two_reconcilers_have_one_mutating_winner`, `exdev_cleanup_intent_contains_submission_and_selector_but_no_final_ids`, `cleanup_intent_exposes_complete_provisional_fence_projection_without_io`, `cleanup_payload_satisfies_clone_debug_without_raw_path_leak`, `exdev_cleanup_worker_generates_fresh_report_ids_without_new_control`, `exdev_cleanup_preflight_claims_original_fixed_then_adjacent`, `exdev_cleanup_prepare_failure_returns_action_or_original_claim`, `exdev_cleanup_execution_consumes_opaque_prepared_reservation_post_ack`, `exdev_cleanup_pre_ack_abort_releases_original_unchanged`, `exdev_cleanup_continues_original_private_source_claim`, `exdev_cleanup_retry_never_copies_destination_again`, `exdev_cleanup_action_is_delivered_non_droppable_without_ui_filesystem_io`, `exdev_cleanup_busy_preserves_cached_opaque_action`, `exdev_cleanup_action_is_visible_but_not_a_normal_retry_candidate`, `exdev_cleanup_action_replacement_fails_without_bare_id_reopen`, `exdev_cleanup_action_claims_exact_control_ref_without_item_id_reopen`, `normal_app_and_cli_callers_use_concrete_intent_bodies`, and `exdev_directory_is_rejected_before_effect`.

`exdev_cleanup_continues_original_private_source_claim` is a crate-unit `exdev::tests` gate because it drives private owning variants, the deterministic second-failure seam, and the non-I/O release path; it is never weakened into an integration assertion over public diagnostics.

Also add crate-unit exact `exdev::tests::exdev_reconcile_retry_uses_original_store_context_for_preclaim_fixed_and_partial_rootless`, injecting failures in the preclaim, fixed-before-root-open, and partial-adjacent-without-root states and proving only private `ExdevStore::retry_reconcile` supplies the original trusted state-root context; no error method opens a global root or searches by ID.

Also add crate-unit exact `mutation_ops::tests::exdev_cleanup_preparation_failure_enters_worker_dispatch_with_ownership`, proving the closed `ItemPreparationFailureInner::ExdevCleanup` arm transfers the opaque whole error to its owner recovery routine and returns either the exact action or original claimed transaction without exposing either through public Debug.

Also add exact `exdev_cleanup_executor_panic_reclaims_exact_fixed_ref_and_reconciles` as a public worker integration test and crate-unit exact `exdev::tests::exdev_reconcile_outcome_never_fabricates_read_only_observation_after_claim`; the latter uses the private consuming reconcile outcome without exposing it publicly.

Add public worker exact tests `exdev_retained_source_emits_cleanup_observation_once_before_terminal`, `exdev_panic_recovery_emits_cleanup_observation_once_before_terminal`, and `exdev_non_cleanup_outcomes_emit_no_cleanup_observation`. The first two assert one cleanup observation precedes the matching item terminal event and `Finished` on both normal and fallback/panic paths; the third covers all other outcome classes and rejects a fabricated action.

- [ ] **Step 2: Run the focused tests and confirm RED**

Run each target exactly:

```bash
python3 scripts/run_exact_test.py --test exdev --name exdev_symlink_copies_raw_target_without_following
python3 scripts/run_exact_test.py --test exdev --name exdev_source_swap_before_claim_is_restored_or_retained
python3 scripts/run_exact_test.py --test exdev --name exdev_publish_intent_reconcile_matches_identity_not_path_existence
python3 scripts/run_exact_test.py --test exdev --name exdev_reconciler_consumes_exact_observed_control_before_adjacent
python3 scripts/run_exact_test.py --test exdev --name exdev_reconcile_rejects_replaced_observation_without_item_id_fallback
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_reconcile_preclaim_failure_returns_exact_observation
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_reconcile_posteffect_failure_retains_transaction_and_reread_truth
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_reconcile_retry_uses_original_store_context_for_preclaim_fixed_and_partial_rootless
python3 scripts/run_exact_test.py --test exdev --name exdev_reconcile_crash_after_claim_rename_reopens_exact_claims_bundle
python3 scripts/run_exact_test.py --test exdev --name exdev_reconcile_staging_and_claims_duplicate_is_indeterminate_without_mutation
python3 scripts/run_exact_test.py --test exdev --name exdev_two_reconcilers_have_one_mutating_winner --serial
python3 scripts/run_exact_test.py --test exdev --name exdev_cleanup_intent_contains_submission_and_selector_but_no_final_ids
python3 scripts/run_exact_test.py --test exdev --name cleanup_intent_exposes_complete_provisional_fence_projection_without_io
python3 scripts/run_exact_test.py --test exdev --name cleanup_payload_satisfies_clone_debug_without_raw_path_leak
python3 scripts/run_exact_test.py --test exdev --name exdev_cleanup_worker_generates_fresh_report_ids_without_new_control
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_cleanup_preflight_claims_original_fixed_then_adjacent
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_cleanup_prepare_failure_returns_action_or_original_claim
python3 scripts/run_exact_test.py --lib --name mutation_ops::tests::exdev_cleanup_preparation_failure_enters_worker_dispatch_with_ownership
python3 scripts/run_exact_test.py --test exdev --name exdev_cleanup_executor_panic_reclaims_exact_fixed_ref_and_reconciles
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_reconcile_outcome_never_fabricates_read_only_observation_after_claim
python3 scripts/run_exact_test.py --lib --name mutation_ops::tests::exdev_cleanup_execution_consumes_opaque_prepared_reservation_post_ack
python3 scripts/run_exact_test.py --lib --name mutation_ops::tests::exdev_cleanup_pre_ack_abort_releases_original_unchanged
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_cleanup_continues_original_private_source_claim
python3 scripts/run_exact_test.py --test mutation_worker --name exdev_cleanup_action_is_delivered_non_droppable_without_ui_filesystem_io
python3 scripts/run_exact_test.py --test mutation_worker --name exdev_retained_source_emits_cleanup_observation_once_before_terminal
python3 scripts/run_exact_test.py --test mutation_worker --name exdev_panic_recovery_emits_cleanup_observation_once_before_terminal
python3 scripts/run_exact_test.py --test mutation_worker --name exdev_non_cleanup_outcomes_emit_no_cleanup_observation
python3 scripts/run_exact_test.py --test app_mutation --name exdev_cleanup_busy_preserves_cached_opaque_action
python3 scripts/run_exact_test.py --test exdev --name exdev_cleanup_retry_never_copies_destination_again
python3 scripts/run_exact_test.py --test operation --name exdev_cleanup_action_is_visible_but_not_a_normal_retry_candidate
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_cleanup_action_replacement_fails_without_bare_id_reopen
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_cleanup_action_claims_exact_control_ref_without_item_id_reopen
python3 scripts/run_exact_test.py --test recovery_cli --name normal_app_and_cli_callers_use_concrete_intent_bodies
python3 scripts/run_exact_test.py --test exdev --name exdev_directory_is_rejected_before_effect
```

Expected: FAIL in the new cases because symlink publication and cleanup-only reconciliation are absent.

- [ ] **Step 3: Implement raw symlink copy and conservative reconciliation**

Read the link with a no-follow capability, copy its raw target bytes into owned staging, reopen/readlink to verify, sync the bundle, and publish no-clobber. The private `ExdevStore::reconcile` engine consumes `VerifiedExdevObservation`, not an `ItemId`: before claiming, it checks the exact recorded item in both adjacent `staging` and `claims`. Zero/stale matches returns the original observation; two matches is contradictory inspect-only `Indeterminate` and mutates neither. Exactly one staging match permits consuming `try_claim` on the observation's exact read-only fixed `ControlBundle`, then fixed-before-adjacent lock, no-clobber rename to claims, both-parent sync, same-object validation, and a fresh post-rename snapshot. Exactly one claims match is the crash-after-rename path: while holding the fixed claim, reopen/lock/revalidate that exact claims object and continue without another rename. In all paths it revalidates protocol, revision, stable identity, and every recorded path/identity before classification. A race after the precheck is re-read into the closed `ExdevAdjacentDiskTruth`: deletion is `Missing`, a competitor-created second name is `InBoth { staging, claims }`, and neither may be mislabeled as merely fixed-claimed. Both retain the claimed fixed control and exact post-effect recovery state as `Indeterminate`; `exdev_reconcile_posteffect_failure_retains_transaction_and_reread_truth` injects and checks both cases. `PublishIntent` advances only when either the owned staged payload matches or the destination matches `published_identity`; otherwise retain the owning claim/adjacent recovery state as `Indeterminate`. The private `ExdevReconcileFailure` returns exact preclaim/claimed/partial/transaction ownership plus stage and no-follow reread truth after claim/open/rename/sync/verify failure. In particular, `open_existing_bound` maps every post-open/lock effect failure to `PartialAdjacentOwned`; it never collapses an adjacent capability to a naked error or reopens by ID. Retrying stays inside `ExdevStore::retry_reconcile(&self, failure)` so even `PreClaim`, `Fixed` before root open, and `PartialAdjacentOwned { root: None }` use the same injected `StateRoot`; the error alone has no global-open method. `reconcile_for_worker` converts only the terminal owner decision into `ItemExecutionResult`. A replaced observation or incomplete stream fails closed without a bare-ID or display-path fallback.

Implement the final concrete intent boundary declared above and private cleanup preparation:

```rust
#[derive(Clone)]
pub struct CleanupSourceAction {
    control_ref: ControlRecoveryRef,
    original_exdev_item_id: ItemId,
    expected_control_identity: PathIdentityKey,
    expected_revision: u64,
    expected_adjacent_identity: PathIdentityKey,
    expected_private_tombstone: PathIdentityKey,
    expected_destination: PathIdentityKey,
    provisional_fences: MutationFenceSpec,
}

impl std::fmt::Debug for CleanupSourceAction {
    // Emit only item/revision and root count; never raw private paths.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

// Concrete Plan 4 payload for WorkerObservation::Exdev.
pub struct CleanupAvailable {
    operation_id: OperationId,
    item_id: ItemId,
    action: CleanupSourceAction,
}

// Concrete Plan 4 evolution of Plan 2's closed, initially uninhabited enum.
pub(crate) enum ItemExecutionObservation {
    Exdev(CleanupAvailable),
}

impl ItemExecutionResult {
    // Added together with the first inhabited observation variant, so Plan 2
    // has no impossible/dead constructor under strict Clippy.
    pub(crate) fn with_observation(
        outcome: ItemOutcome,
        observation: ItemExecutionObservation,
    ) -> Self;
}
#[doc(hidden)]
pub struct PreparedExdevCleanup {
    original: ClaimedExdevTransaction,
    action: CleanupSourceAction,
    report_operation_id: OperationId,
    report_item_id: ItemId,
}
enum ExdevCleanupPreparationOwnership {
    Action(CleanupSourceAction),
    FixedClaimFailed {
        action: CleanupSourceAction,
        failure: ControlRecoveryClaimFailure,
    },
    OriginalClaimed {
        action: CleanupSourceAction,
        original: ClaimedExdevTransaction,
    },
}
pub(crate) struct PrepareExdevCleanupError {
    ownership: ExdevCleanupPreparationOwnership,
    source: PreparationError,
}

impl ExdevStore<'_> {
    pub fn cleanup_action(&self, observed: &VerifiedExdevObservation) -> Result<CleanupSourceAction, ExdevError>;
}

pub(crate) fn prepare_exdev_cleanup(
    context: &MutationContext,
    operation_id: OperationId,
    candidate_item_id: ItemId,
    action: CleanupSourceAction,
) -> Result<(ItemPlan, PreparedExdevCleanup, MutationFenceSpec), PrepareExdevCleanupError>;

pub(crate) fn execute_exdev_cleanup(
    context: &MutationContext,
    request: &OperationRequest,
    plan: &ItemPlan,
    prepared: PreparedExdevCleanup,
    cancel: &CancelToken,
    progress: &ProgressSink,
) -> ItemExecutionResult;

pub(crate) fn abort_prepared_exdev_cleanup(
    context: &MutationContext,
    plan: &ItemPlan,
    prepared: PreparedExdevCleanup,
) -> PreparedAbortOutcome;

impl PreparedExdevCleanup {
    pub(crate) fn recovery_ref(&self) -> ControlRecoveryRef;
}

pub(crate) fn recover_exdev_cleanup_execution_panic(
    context: &MutationContext,
    plan: &ItemPlan,
    claimed: ClaimedControlBundle,
) -> ItemExecutionResult;

pub(crate) fn recover_exdev_cleanup_preparation_failure(
    context: &MutationContext,
    failure: PrepareExdevCleanupError,
) -> PreparedAbortOutcome;

impl PrepareExdevCleanupError {
    pub(crate) fn source(&self) -> &PreparationError;
    pub(crate) fn claim_disk_truth(&self) -> Option<&ControlRecoveryClaimDiskTruth>;
}

impl MutationIntent {
    pub fn cleanup_retained_source(submission_id: SubmissionId, action: CleanupSourceAction) -> Self;
    pub(crate) fn provisional_fences(&self) -> MutationFenceSpec;
}

impl CleanupSourceAction {
    pub(crate) fn provisional_fences(&self) -> &MutationFenceSpec;
}

impl CleanupAvailable {
    pub(crate) fn report_key(&self) -> (OperationId, ItemId);
    pub(crate) fn clone_action(&self) -> CleanupSourceAction;
}
```

Migrate normal App callers to `MutationIntent::paths`. The Plan 3 CLI restore path evaluates `let request = RestoreRequest { selector: RestoreSelector::UniqueItem(id), to }; let intent = RestoreIntent::capture_cli(cli_work_root, request)?;`, then calls `MutationIntent::restore(submission_id, intent)`. `UniqueItem` is accepted only by this CLI constructor; App has no constructor or command that creates it. The worker no-follow reopens and compares the submitted root identity before resolving the unique item, so changing cwd or replacing the path after submission cannot redirect the restore.

Extend Plan 2's closed execution/result path with `ItemExecutionObservation::Exdev(CleanupAvailable)` and its non-progress channel with `WorkerObservation::Exdev(CleanupAvailable)`. `execute_exdev_move`, `execute_exdev_cleanup`, both EXDEV panic recovery hooks, and fixed-only terminal resume return `ItemExecutionResult`, never a naked `ItemOutcome`; the closed `mutation_ops` dispatcher preserves the optional concrete observation. The worker's one ordered emitter maps the EXDEV item-side observation to `WorkerObservation::Exdev` exactly once before the matching terminal event and `Finished`, on normal and fallback paths alike. Every non-cleanup outcome carries `None`, so the worker cannot construct an action from an outcome enum after the owner transaction is gone.

The cleanup action exists only for a verified `DestinationPublishedSourceRemovalPending`/`CleanupRequired` outer whose mirror is confirmed and whose destination plus already-private SourceClaim tombstone freshly match the fixed receipt. Its fields are private, it is non-serde, and only factories in `exdev.rs` may construct it while they still own or exactly observe the original fixed/adjacent transaction: they call `observed.control.recovery_ref()` on the actual read-only fixed capability and capture exact fixed/adjacent/tombstone/destination identities, revision, and a conservative raw fence projection for the original source parent, destination parent, private tombstone parent, and adjacent bundle root. `control_ref` retains the trusted pending parent, validated raw child name, full observed fixed identity, and header; `original_exdev_item_id` remains display/key metadata only and is never an open selector. App uses only `CleanupAvailable::{report_key,clone_action}` and `MutationIntent::provisional_fences`; it cannot read raw action fields or construct one from a displayed/report `ItemId`. The values are an observed selector, not authorization: worker preflight must consume a clone of the exact `control_ref` through `StateRoot::claim_recovery_ref`, then acquire the adjacent lock and revalidate every binding. A replaced, missing, or unreadable fixed child fails closed through `FixedClaimFailed { action, failure }`; the nested owning failure retains the exact ref, `ControlRecoveryClaimDiskTruth::{Present,Absent,Unreadable}`, and diagnostic source until the host recovery routine consumes it. No branch calls `open_control_bundle(ItemId)` or enumerates by ID, and no layer restates typed truth from a string. App caches the delivered observation by its report key with the active/latest full report. The key handler performs no filesystem lookup. Missing/disconnected action disables `c` and leaves the truthful cleanup-required outcome visible rather than synthesizing a selector. `recover_exdev_cleanup_preparation_failure` consumes either the unchanged action, the nested exact claim failure, or the original claimed transaction; `abort_prepared_exdev_cleanup` consumes the opaque prepared cleanup and releases its fixed/adjacent locks unchanged. Neither routine unlinks, creates, copies, publishes, advances a receipt, or removes the original durable control. The existing cleanup preparation/pre-ack tests call the Plan 2 central dispatch and prove those invariants.

Add `OperationKind::CleanupRetainedSource`; selecting `c` clones the cached action, allocates only a new `SubmissionId`, creates the intent, derives its stored conservative fences without I/O, installs `FenceOwner::Provisional(submission_id)`, and only then calls `try_start`. `Busy`/unavailable removes only that provisional fence and retains the action/page. Retain the original cache entry on preflight rejection or pre-ack cancellation; disable duplicate submission while that `SubmissionId` is active, and remove the cache entry only after the cleanup operation proves `Completed` or an authoritative rescan says the original is no longer actionable. Worker preflight generates fresh final report `OperationId`/`ItemId` values and calls only `exdev::prepare_exdev_cleanup`, whose private factory owns all field construction. That function consumes the action's cloned `ControlRecoveryRef` through `StateRoot::claim_recovery_ref`, rechecks identity/revision/protocol/mirror/destination/private-tombstone facts, and locks the original adjacent bundle in fixed-before-adjacent order. It never calls `open_control_bundle`, searches by the action's display `ItemId`, or creates a new fixed control/adjacent bundle. This same edit supplies the first normal-library caller of read-only `ControlBundle::recovery_ref` inside `cleanup_action` and removes Plan 2's narrow `#[expect(dead_code)]` from that method; `-D warnings` would reject retaining the now-fulfilled expectation. Store the original owning transaction and the fresh report IDs in `PreparedReservation::ExdevCleanup`; stale/replaced actions fail with zero effect and no ItemId fallback. A failure before attempting the exact claim returns private `Action(action)`; an exact-claim rejection returns private `FixedClaimFailed { action, failure }` with typed reread truth; only a failure after a successful claim returns `OriginalClaimed { action, original }`. The opaque wrapper crosses the central dispatch but none of those variants does; no error drops ownership or reopens by ID. Before `FenceInstalled`, cancellation releases the unchanged original transaction and creates/removes nothing. `exdev_cleanup_action_claims_exact_control_ref_without_item_id_reopen` pauses after action creation, replaces the fixed child, and requires fail-closed preflight plus a zero-call assertion on the ItemId-opening seam.

After acknowledgement, mutation dispatch calls only `exdev::execute_exdev_cleanup` with `PreparedReservation::ExdevCleanup`; the opaque prepared fields never leave `exdev.rs`. Cleanup never calls `SourceClaim::acquire`/source-parent `reconcile_pending` and never opens, stats, restores, renames, or unlinks the original user path. It no-follow reopens and full-identity-checks the private tombstone parent recorded in the claimed fixed state, then calls only `SourceClaim::reconcile_private_cleanup(context.state_root.as_ref(), &trusted_private_parent, control)`. These calls are the first normal-library consumers of `reconcile_private_cleanup` and `release_classified`; this same `src/source_claim.rs` edit removes all three temporary Plan 2 `#[expect(dead_code)]` attributes, so an unfulfilled expectation cannot survive the consumer commit. The owner runs one exhaustive by-value loop with `MAX_PRIVATE_CLEANUP_RECONCILE_ATTEMPTS = 2`. `Deleted` is the only success. `Published` and `RestoredNoEffect` are action/state contradictions and become conservative `Indeterminate`. `RestoreRequired(recovery)` is immediately consumed through non-I/O `release_classified` and is never reconciled/restored in cleanup mode. `CleanupRequired` and `Indeterminate` may call only `recovery.reconcile_private_cleanup`; after the bounded budget, any owning result is consumed through `release_classified`. `SourceClaimAcquireFailure::NoAdjacentEffect` retains unchanged fixed authority for conservative classification; `AdjacentOwned { recovery, .. }` enters the same bounded private-cleanup/release loop. No `ItemExecutionResult` or startup notice is emitted until no `SourceClaimRecovery`/adjacent handle/borrow remains, yet a persistent I/O/permission fault cannot monopolize the serial mutation worker. Cleanup then deletes only the freshly same-snapshot private tombstone, mirror-advances the same original fixed/adjacent receipts through the exact `CleanupRequiredToCommitted` edge, and consumes the original control through the owning terminal typestate. The fresh report IDs identify this cleanup attempt but are not written into or substituted for the original durable authority. `OperationReport::retry_candidates` continues to exclude `DestinationCommittedSourceRetained`; this explicit action is a fresh intent whose final IDs are generated by the worker, never App. Two fixed controls, a sibling cleanup receipt, a second source claim, and a copy/publish re-entry are all forbidden by source-contract tests. `exdev_cleanup_continues_original_private_source_claim` covers all six `ClaimResult` variants, a second injected reconcile failure, budget exhaustion, release, zero original-parent syscalls, unchanged original identity, and zero dropped authority.

For avoidance of doubt, neither `ClaimResult::Published` nor `ClaimResult::RestoredNoEffect` is a cleanup success under `ClaimAction::ExdevSourceCleanup`. Only verified `ClaimResult::Deleted` may enter the committed-terminal path.

- [ ] **Step 4: Run the complete EXDEV test file**

Run: `cargo test --locked --test exdev --test operation --test mutation_worker --test app_mutation --test recovery_cli`

Expected: PASS; regular-file, symlink, crash, race, and cleanup-only tests all pass.

- [ ] **Step 5: Commit EXDEV reconciliation**

```bash
git add src/exdev.rs src/operation.rs src/mutation.rs src/mutation_ops.rs src/app.rs src/cli.rs tests/operation.rs tests/exdev.rs tests/mutation_worker.rs tests/app_mutation.rs tests/recovery_cli.rs
git commit -m "feat: reconcile EXDEV moves without source loss"
```

### Task 4: Pure Recovery overlay state and reducer

**Files:**
- Create: `src/recovery_ui.rs`
- Modify: `src/recovery.rs`
- Modify: `src/lib.rs`
- Create: `tests/recovery_ui.rs`

- [ ] **Step 1: Write failing reducer tests**

Add exact tests `recovery_overlay_opens_loading_then_ready`, `recovery_overlay_navigation_is_bounded_by_full_catalog_key`, `recovery_overlay_next_and_previous_emit_complete_cursor_anchors`, `recovery_overlay_forward_backward_pages_have_no_gap_or_repeat`, `recovery_previous_page_keeps_o_page_memory`, `recovery_overlay_verified_restore_emits_observed_bundle_ref`, `recovery_overlay_never_emits_unique_item_selector`, `recovery_overlay_inspect_only_ref_has_no_raw_name_or_restore_capability`, `recovery_overlay_restore_has_no_replace_or_rename_action`, `recovery_overlay_restore_to_preserves_input_on_invalid_parent`, `recovery_overlay_corrupt_item_is_inspect_only`, `recovery_overlay_r_reloads_and_question_mark_toggles_help`, and `recovery_overlay_close_does_not_change_workbench_selection`.

- [ ] **Step 2: Run reducer tests and confirm RED**

Run each target exactly:

```bash
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_overlay_opens_loading_then_ready
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_overlay_navigation_is_bounded_by_full_catalog_key
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_overlay_next_and_previous_emit_complete_cursor_anchors
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_overlay_forward_backward_pages_have_no_gap_or_repeat
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_previous_page_keeps_o_page_memory
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_overlay_verified_restore_emits_observed_bundle_ref
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_overlay_never_emits_unique_item_selector
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_overlay_inspect_only_ref_has_no_raw_name_or_restore_capability
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_overlay_restore_has_no_replace_or_rename_action
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_overlay_restore_to_preserves_input_on_invalid_parent
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_overlay_corrupt_item_is_inspect_only
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_overlay_r_reloads_and_question_mark_toggles_help
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_overlay_close_does_not_change_workbench_selection
```

Expected: FAIL because `RecoveryOverlayState` and `RecoveryAction` do not exist.

- [ ] **Step 3: Implement a pure overlay reducer**

Define:

```rust
pub enum RecoveryMode { List, Inspect, RestoreToInput, ConfirmRestore }
pub enum RecoveryPhase { Loading, Ready, Stale, Active, Cancelling, Help { return_to: Box<RecoveryPhase> } }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryPageAnchor {
    First,
    After(RecoveryCatalogCursor),
    Before(RecoveryCatalogCursor),
}
pub struct RecoveryWindow {
    pub page: RecoveryPage,
    pub anchor: RecoveryPageAnchor,
    pub has_previous: bool,
}
pub enum RecoveryAction {
    Move(i32), Inspect, Restore, BeginRestoreTo, EditRestoreTo(String),
    SubmitRestoreTo, Confirm, Reload, NextPage, PreviousPage,
    ToggleHelp, CancelActive, Back, Close,
}
pub enum RecoveryEffect {
    None,
    LoadPage(RecoveryPageAnchor),
    StartRestore(RestoreRequest),
    Close,
}

pub struct RecoveryOverlayState {
    pub mode: RecoveryMode,
    pub phase: RecoveryPhase,
    pub window: Option<RecoveryWindow>,
    pub selected_key: Option<RecoveryCatalogCursor>,
    pub restore_to_input: String,
    pub validation_error: Option<String>,
}

impl RecoveryOverlayState {
    pub fn reduce(&mut self, action: RecoveryAction) -> RecoveryEffect;
    pub fn selected_record(&self) -> Option<&RecoveryRecord>;
}

impl RecoveryRecord {
    pub fn key(&self) -> RecoveryCatalogCursor;
    pub fn observed_restore_selector(&self) -> Option<RestoreSelector>;
}
```

Extend Plan 3's `RecoveryService` in `src/recovery.rs` without changing its CLI-facing `list_page` contract:

```rust
impl RecoveryService<'_> {
    pub fn list_window(
        &self,
        work_root: &Path,
        anchor: &RecoveryPageAnchor,
        limit: usize,
    ) -> Result<RecoveryWindow>;
}
```

`First` and `After` delegate to the same side-effect-free streaming selection semantics as Plan 3. `Before(cursor)` performs one full streaming pass, retains only the largest `limit` complete keys strictly before the cursor in a bounded min-heap, sorts that bounded result ascending, and records whether any still-earlier key exists. All three variants clamp the limit to `1..=200`, count the same authoritative `total`, preserve complete `CatalogKey` ordering, and fail the whole window on a midstream enumeration error. `RecoveryWindow::has_previous` is true only when at least one key precedes the first returned record. `RecoveryPage::next` remains the last returned full key only when a later key exists, including when a `Before` window can advance back toward its anchor. Neither the service nor UI keeps an unbounded cursor stack or materializes the catalog; forward and backward navigation are both `O(catalog)` time and `O(page)` memory.

Only `RecoveryRecord::Verified { receipt: Committed, state: Recoverable, .. }` exposes Restore/Restore-to. Confirming it clones the record's private-field `VerifiedBundleRef` and emits `RestoreRequest { selector: RestoreSelector::Observed(bundle_ref), to }`. The reducer has no `UniqueItem` branch; that selector is reserved for Plan 3 CLI parsing. A verified observation may become stale or contradictory later, but the UI never falls back to an ID lookup.

There is no replace or rename action. Invalid restore-to input retains mode, input, the complete `RecoveryCatalogCursor`, and window. `RecoveryRecord::InspectOnly` carries only `InspectOnlyRef` plus escaped display/state; neither reducer nor renderer imports `RawUnixName`, accesses raw bytes, or constructs a selector from it. Inspect-only and verified non-recoverable rows allow Inspect/Reload/page navigation/Help/Back/Close only. Lock `R` to Reload, `[` to Previous page, `]` to Next page, `?` to Help, Esc to Back/Close, and Ctrl+G to cancel the active restore. Reload emits `LoadPage` with the current window's complete anchor, or `First` before the first result. Next emits `After(last_returned_key)` only when `page.next` proves a later record. Previous emits `Before(first_returned_key)` only when `has_previous` is true. An empty/stale window never fabricates an anchor. `Help { return_to }` stores the exact prior phase and `?`/Esc restores it, including Stale/Active/Cancelling. Preserve selection by the complete key including class/name-or-ID/location/identity; if it disappeared, clamp to the nearest row without converting its display identity into a capability.

- [ ] **Step 4: Run reducer tests and confirm GREEN**

Run: `cargo test --locked --test recovery_ui -- --nocapture`

Expected: PASS; all pure reducer tests pass without constructing `App` or a terminal, and no inspect-only record can produce a restore selector.

- [ ] **Step 5: Commit Recovery overlay state**

```bash
git add src/recovery_ui.rs src/recovery.rs src/lib.rs tests/recovery_ui.rs
git commit -m "feat: define recovery overlay interactions"
```

### Task 5: Render and wire Recovery through the serial mutation lane

**Files:**
- Modify: `src/read_lane.rs`
- Modify: `src/operation.rs`
- Modify: `src/mutation_ops.rs`
- Modify: `src/app.rs:34-91`
- Modify: `src/app.rs:101-132`
- Modify: `src/app.rs:1503-1668`
- Modify: `src/ui.rs:18-77`
- Modify: `src/ui.rs:450-615`
- Modify: `src/ui.rs:748-760`
- Modify: `tests/app_keys.rs:284-685`
- Modify: `tests/render.rs:111-321`
- Modify: `tests/read_lane.rs`
- Modify: `tests/mutation_ops.rs`
- Modify: `tests/recovery_ui.rs`
- Modify: `tests/app_mutation.rs`

- [ ] **Step 1: Write failing App and rendering tests**

Add exact tests `directory_and_recovery_pending_do_not_replace_each_other`, `directory_latest_wins_only_within_directory_key`, `recovery_latest_wins_only_within_recovery_key`, `scan_mailbox_never_exceeds_two_pending`, `continuous_directory_requests_do_not_starve_recovery`, `continuous_recovery_requests_do_not_starve_directory`, `slow_running_directory_preserves_latest_recovery_pending`, `slow_running_recovery_preserves_latest_directory_pending`, `closing_keyed_scan_worker_releases_full_result_sender_and_joins`, `recovery_key_opens_loading_without_changing_selection`, `recovery_scan_1000ms_keeps_input_responsive`, `recovery_result_carries_and_matches_authoritative_work_root`, `recovery_late_generation_or_fs_epoch_result_is_rejected`, `recovery_close_ignores_late_catalog_result`, `recovery_tui_submits_observed_selector_not_unique_item`, `recovery_stale_observed_ref_after_page_fails_without_id_fallback`, `recovery_duplicate_after_page_is_fail_closed`, `recovery_inspect_only_row_cannot_submit_mutation`, `recovery_restore_uses_plan3_typed_advance_and_opaque_proofs`, `recovery_restore_preflight_and_execution_run_on_mutation_worker`, `recovery_second_restore_is_rejected_while_mutation_active`, `recovery_success_removes_row_and_selects_nearest`, `recovery_skip_or_failure_keeps_selected_row`, `recovery_busy_keeps_selected_row_and_shows_busy`, `recovery_completion_reloads_catalog_and_files`, `recovery_tui_reaches_record_beyond_first_200`, `recovery_tui_forward_backward_pages_have_no_gap_or_repeat`, `operation_detail_40x10_shows_cleanup_source_action`, `recovery_40x10_keeps_phase_page_r_help_cancel_back_and_quit_visible`, and `recovery_80x24_shows_id_path_time_kind_and_state`.

- [ ] **Step 2: Run integration/render tests and confirm RED**

Run each target exactly in its owning test binary:

```bash
python3 scripts/run_exact_test.py --test read_lane --name directory_and_recovery_pending_do_not_replace_each_other
python3 scripts/run_exact_test.py --test read_lane --name directory_latest_wins_only_within_directory_key
python3 scripts/run_exact_test.py --test read_lane --name recovery_latest_wins_only_within_recovery_key
python3 scripts/run_exact_test.py --test read_lane --name scan_mailbox_never_exceeds_two_pending
python3 scripts/run_exact_test.py --test read_lane --name continuous_directory_requests_do_not_starve_recovery
python3 scripts/run_exact_test.py --test read_lane --name continuous_recovery_requests_do_not_starve_directory
python3 scripts/run_exact_test.py --test read_lane --name slow_running_directory_preserves_latest_recovery_pending
python3 scripts/run_exact_test.py --test read_lane --name slow_running_recovery_preserves_latest_directory_pending
python3 scripts/run_exact_test.py --test read_lane --name closing_keyed_scan_worker_releases_full_result_sender_and_joins
python3 scripts/run_exact_test.py --test app_keys --name recovery_key_opens_loading_without_changing_selection
python3 scripts/run_exact_test.py --test read_lane --name recovery_scan_1000ms_keeps_input_responsive
python3 scripts/run_exact_test.py --test read_lane --name recovery_result_carries_and_matches_authoritative_work_root
python3 scripts/run_exact_test.py --test app_keys --name recovery_late_generation_or_fs_epoch_result_is_rejected
python3 scripts/run_exact_test.py --test app_keys --name recovery_close_ignores_late_catalog_result
python3 scripts/run_exact_test.py --test app_keys --name recovery_tui_reaches_record_beyond_first_200
python3 scripts/run_exact_test.py --test app_keys --name recovery_tui_forward_backward_pages_have_no_gap_or_repeat
python3 scripts/run_exact_test.py --test app_mutation --name recovery_tui_submits_observed_selector_not_unique_item
python3 scripts/run_exact_test.py --test mutation_ops --name recovery_stale_observed_ref_after_page_fails_without_id_fallback
python3 scripts/run_exact_test.py --test mutation_ops --name recovery_duplicate_after_page_is_fail_closed
python3 scripts/run_exact_test.py --test app_mutation --name recovery_inspect_only_row_cannot_submit_mutation
python3 scripts/run_exact_test.py --test mutation_ops --name recovery_restore_uses_plan3_typed_advance_and_opaque_proofs
python3 scripts/run_exact_test.py --test mutation_ops --name recovery_restore_preflight_and_execution_run_on_mutation_worker
python3 scripts/run_exact_test.py --test app_mutation --name recovery_second_restore_is_rejected_while_mutation_active
python3 scripts/run_exact_test.py --test app_mutation --name recovery_success_removes_row_and_selects_nearest
python3 scripts/run_exact_test.py --test app_mutation --name recovery_skip_or_failure_keeps_selected_row
python3 scripts/run_exact_test.py --test app_mutation --name recovery_busy_keeps_selected_row_and_shows_busy
python3 scripts/run_exact_test.py --test app_mutation --name recovery_completion_reloads_catalog_and_files
python3 scripts/run_exact_test.py --test render --name operation_detail_40x10_shows_cleanup_source_action
python3 scripts/run_exact_test.py --test render --name recovery_40x10_keeps_phase_page_r_help_cancel_back_and_quit_visible
python3 scripts/run_exact_test.py --test render --name recovery_80x24_shows_id_path_time_kind_and_state
```

Expected: FAIL because `Mode::Recovery`, its command routing, and its renderer are absent.

- [ ] **Step 3: Wire the overlay without duplicating recovery logic**

Extend the exact Plan 2 directory-only scan worker with a second concrete keyed slot; the existing scan thread and result channel remain the sole owner:

```rust
pub type RecoveryCatalogGeneration = u64;
pub struct RecoveryScanRequest {
    pub work_root: PathBuf,
    pub generation: RecoveryCatalogGeneration,
    pub anchor: RecoveryPageAnchor,
    pub limit: usize,
}
pub enum ScanRequestKind {
    Directory { cwd: PathBuf, show_hidden: bool },
    RecoveryCatalog(RecoveryScanRequest),
}
pub enum ReadEvent {
    ScanFinished { token: ScanToken, cwd: PathBuf, result: Result<DirectoryEntries, ReadFailure> },
    RecoveryCatalogFinished {
        token: ScanToken,
        work_root: PathBuf,
        generation: RecoveryCatalogGeneration,
        result: Result<RecoveryWindow, ReadFailure>,
    },
    PreviewFinished { token: PreviewToken, path: PathBuf, result: Result<Preview, ReadFailure> },
    ScanWorkerLost,
    PreviewWorkerLost,
}
pub type RecoveryCatalogBackend = Arc<dyn Fn(&RecoveryScanRequest) -> Result<RecoveryWindow, ReadFailure> + Send + Sync>;
pub struct ScanBackendSet {
    pub directory: DirectoryScanBackend,
    pub recovery_catalog: RecoveryCatalogBackend,
}

enum ScanWorkKey { Directory, RecoveryCatalog }
struct ScanMailbox {
    directory: Option<ScanRequest>,
    recovery_catalog: Option<ScanRequest>,
    next: ScanWorkKey,
    closed: bool,
}

let request = ScanRequest {
    token: ScanToken { cwd_generation, fs_epoch },
    kind: ScanRequestKind::RecoveryCatalog(RecoveryScanRequest {
        work_root: cwd.clone(),
        generation: recovery_generation,
        anchor: requested_anchor,
        limit: 200,
    }),
};
read_lanes.request_scan(request)?;
```

Implement `ScanBackendSet::recovery_catalog` with an injected `Arc<StateRoot>`: inside the background closure construct `let recovery = RecoveryService { state_root: state_root.as_ref() };`, then call `recovery.list_window(&request.work_root, &request.anchor, request.limit)` and preserve typed `RecoveryRecord::{Verified,InspectOnly}`, the exact requested anchor, complete cursors, `has_previous`, `loaded`, and `total`. No call uses an associated/static `RecoveryService` form or omits the required receiver. Submission pattern-matches the request kind and replaces only that field. `take_next` alternates its cursor after every selected key whenever both fields are occupied; a stream of one kind cannot starve a pending other kind. The bound is exactly two pending scan requests. Close sets `closed`, clears both fields, wakes the worker, continues draining/drops the bounded result receiver to release a blocked sender, and joins the one scan thread. There is no third worker, FIFO, second catalog queue, `Any`, or generic job interface.

Normal mode binds uppercase `R` to `Command::OpenRecovery`; lowercase `r` remains directory reload. Inside the overlay uppercase `R` is Reload and `[`/`]` are Previous/Next page. Add `Mode::Recovery`, `Command::OpenRecovery`, and `App::recovery_overlay`. `OpenRecovery` increments `RecoveryCatalogGeneration`, records `ScanToken { cwd_generation, fs_epoch }`, sets `Loading`, and submits `RecoveryPageAnchor::First`. Every reducer `LoadPage(anchor)` increments the generation and submits that exact complete anchor through the same RecoveryCatalog slot. Accept only a result whose scan token, catalog generation, work root, requested anchor, and still-open mode match. Failure retains the last-good window and anchor as `Stale`; close increments generation so a late 1,000-ms result cannot reopen the overlay. The App stores only the current bounded `RecoveryWindow`, never a cursor-history vector.

Route keys through the pure reducer. `RecoveryEffect::StartRestore` already contains `RestoreRequest { selector: RestoreSelector::Observed(VerifiedBundleRef), .. }`. App allocates a `SubmissionId`, completes the only fallible non-authorizing construction with `let intent = RestoreIntent::from_observed(request)?;`—which consumes the private selected ref's already observed root/path identities and performs no I/O—derives the conservative work-root/explicit-destination provisional fence through an opaque intent projection, installs `FenceOwner::Provisional(submission_id)`, and only then calls `MutationWorker::try_start(MutationIntent::restore(submission_id, intent))`. App performs no stat, open, ID lookup, list, claim, reconciliation, final-ID creation, `OperationRequest` construction, or restore. The worker first generates the final `operation_id` and candidate execution `item_id`, constructs `let recovery = RecoveryService { state_root: context.state_root.as_ref() };`, then calls the exact receiver method `recovery.prepare_restore(context, operation_id, candidate_item_id, intent)`. That method no-follow reopens and compares the private `RestoreWorkRoot`, reopens the exact location-parent/name/identity, rechecks every duplicate location, and returns only `(ItemPlan, PreparedRestoreReservation, MutationFenceSpec)`. A work-root or location-parent identity change, stale/replaced/newly duplicated/non-recoverable/in-use observation fails closed without `UniqueItem` fallback; `UniqueItem` remains CLI-only. Only after the worker freezes the request, sends `Prepared`, and receives `FenceInstalled` does execution consume `PreparedRestoreReservation` and claim the exact `VerifiedBundleRef`. It calls only Plan 3's consuming Trash/Restore transaction APIs and owning proof/terminal typestates; App and `recovery_ui.rs` cannot import `AtomicReceiptFile`, construct mirror/source-claim authorization, hold a locked receipt snapshot, or access a raw control handle. Plan 3's `RestorePlan` and `PreparedRestoreReservation` fields, including the claimed control, are private/crate-private to the worker and never enter `PreparedNotice`.

On `StartMutationError::Busy` or worker unavailable, remove only `FenceOwner::Provisional(submission_id)` and retain the window, complete selected key, input, and mode. `recovery_busy_keeps_selected_row_and_shows_busy` also instruments the filesystem boundary and asserts that `RestoreIntent::from_observed` projects exactly the observed work-root recursive fence plus any explicit destination fence, contains no final IDs, performs zero I/O before `try_start`, and leaves no orphan provisional fence after Busy/unavailable. Active/Cancelling derive only from operation observations. Success removes the restored row after authoritative reload of the current anchor and selects the nearest complete key; skip/failure retains the same key. Terminal effect/maybe-effect events update the report, bump `fs_epoch`, and request both file and RecoveryCatalog keys.

In normal operation detail, `DestinationCommittedSourceRetained` renders `c cleanup source` at 40x10 and wider. Pressing `c` submits `MutationIntent::cleanup_retained_source(new_submission_id, action)`; App does not generate operation/item IDs and the action never enters normal retry or copy/publish.

Add `draw_recovery_overlay(frame, area, state)` in `src/ui.rs`. At 40x10 render `Loading`/`Ready`/`Stale`/`Active`/`Cancelling`/`Help`, one escaped path line, `[` Prev and `]` Next when applicable, `R` Reload, `?` Help, Cancel when active, Back, and Quit; abbreviate labels but never hide phase or active cancellation. At 80x24 render a page of receipt ID, escaped original path, time, kind, state, page position, and inspect detail. Do not place overwrite or rename on any key or label.

- [ ] **Step 4: Run TUI and existing modal regressions**

Run: `cargo test --locked --test read_lane --test recovery_ui --test app_keys --test app_mutation --test mutation_ops --test render -- --nocapture`

Expected: PASS; Recovery tests pass and existing confirmation/input/modal behavior remains green.

- [ ] **Step 5: Commit Recovery TUI integration**

```bash
git add src/read_lane.rs src/operation.rs src/mutation_ops.rs src/app.rs src/ui.rs tests/read_lane.rs tests/mutation_ops.rs tests/app_keys.rs tests/app_mutation.rs tests/render.rs tests/recovery_ui.rs
git commit -m "feat: add trash recovery overlay"
```

### Task 6: Freeze the integrated G1c/G2 component candidate

**Files:**
- Create: `scripts/test-exdev-native.sh`
- Modify: `src/exdev.rs`
- Modify: `.github/workflows/ci.yml`
- Modify: `.github/workflows/release.yml`
- Modify: `README.md`
- Modify: `CHANGELOG.md`
- Modify: `tests/exdev.rs`
- Modify: `tests/recovery_ui.rs`
- Modify: `tests/release_contract.rs`

- [ ] **Step 1: Add failing fault and native-workflow contract tests**

Add crate-unit exact Rust tests `exdev::tests::exdev_gate_fault_matrix_preserves_unique_source` and `exdev::tests::exdev_gate_cancel_matrix_reports_exact_outcome`, plus public integration tests `exdev_gate_two_process_claim_matrix_has_one_winner`, `exdev_native_roots_have_different_devices`, `exdev_native_regular_and_symlink_cross_real_devices`, `recovery_gate_receipt_state_matrix_is_truthful`, `recovery_catalog_10000_fixture_is_bounded_and_deterministic`, `recovery_catalog_10000_forward_backward_is_bounded_and_complete`, `workflow_has_exact_native_exdev_jobs_and_tier1_steps`, and `native_exdev_script_uses_exact_runner_only`. The injected matrices live in `src/exdev.rs` and use the private deterministic fault controller; no production or feature-gated fault API is added. Mark only the two `exdev_native_*` tests `#[ignore = "requires two writable roots on different st_dev"]`. The 10,000-row forward/backward test drives the actual reducer and keyed background backend through every page to the final row and back to the first, requires the independently sorted complete keys with no gap/repeat, and asserts service/App memory never exceeds one 200-row window plus the two-slot scan mailbox.

Cover every EXDEV transition, fixed/adjacent mirror mismatch, opaque-proof rejection, ENOSPC, EACCES, checksum/metadata/file-sync/directory-sync failures, source swap, parent replacement, target race, before/after-publish cancellation, duplicate cleanup, and a competing reconciler. Freeze these external case manifests rather than letting each Rust test declare its own expected set:

- `g1c-unique-source-faults` (18): `enospc-copy`, `eacces-stage`, `payload-checksum`, `metadata-apply`, `payload-sync`, `bundle-sync`, `fixed-receipt-sync`, `adjacent-receipt-sync`, `destination-parent-replaced`, `source-swap`, `target-race`, `publish-parent-sync`, `source-claim-sync`, `private-tombstone-unlink`, `tombstone-parent-sync`, `fixed-terminal-remove`, `adjacent-terminal-remove`, `reconciler-contention`.
- `g1c-cancel-boundaries` (10): `before-prepared`, `waiting-fence-ack`, `copy-chunk`, `after-payload-ready`, `before-publish-intent`, `after-publish-intent`, `after-destination-publish`, `source-claim`, `private-cleanup`, `after-committed`.
- `g2-recovery-states` (12): `prepared`, `payload-ready`, `publish-intent-staged`, `publish-intent-published`, `destination-published-source-pending`, `committed`, `cleanup-required`, `indeterminate`, `corrupt`, `duplicate`, `in-use`, `legacy-unmanaged`.

The workflow contract test requires exact ordinary-CI job names `native-exdev-linux` and `native-exdev-macos`, exact release job names `tier1-linux-x86_64` and `tier1-macos-arm64`, and required steps named `native-exdev-linux`/`native-exdev-macos` inside their matching Tier-1 jobs. It rejects generic aliases such as `native-exdev`, `linux`, or `macos`. Each ordinary-CI native job must produce exactly one artifact from this frozen map:

| Producer job | Exact artifact name | Root schema |
| --- | --- | --- |
| `native-exdev-linux` | `native-exdev-linux-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-native-exdev-evidence-v1` |
| `native-exdev-macos` | `native-exdev-macos-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-native-exdev-evidence-v1` |

The root `artifact-manifest.json` excludes itself and lists a nonempty canonical payload inventory. The payload `native-exdev-evidence.json` binds the producer job, 40-hex candidate, numeric run ID/attempt, both distinct device identities, exact two executed test names, exact ignored/native flags, result, and SHA-256 of each captured test output. The source contract requires one pinned `actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02` step with ID `upload-evidence` and the exact name, followed immediately by exactly one `scripts/record_artifact_producer.py` invocation wired to that step's artifact ID and bare digest. Missing/empty/extra payload, self-listed manifest, second upload/recorder, wrong producer/schema/name, or an unjoined upload fails `workflow_has_exact_native_exdev_jobs_and_tier1_steps`. Exact candidate selection, runner inventory, timeout/cancel, REST download, and artifact validation belong only to the shared implementation-evidence harness; this component plan must not fork those trust rules.

- [ ] **Step 2: Run the integrated gate and expose missing cases**

Run every focused gate through Plan 1's exact runner:

```bash
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_gate_fault_matrix_preserves_unique_source --serial --case-matrix g1c-unique-source-faults --expect-case enospc-copy --expect-case eacces-stage --expect-case payload-checksum --expect-case metadata-apply --expect-case payload-sync --expect-case bundle-sync --expect-case fixed-receipt-sync --expect-case adjacent-receipt-sync --expect-case destination-parent-replaced --expect-case source-swap --expect-case target-race --expect-case publish-parent-sync --expect-case source-claim-sync --expect-case private-tombstone-unlink --expect-case tombstone-parent-sync --expect-case fixed-terminal-remove --expect-case adjacent-terminal-remove --expect-case reconciler-contention
python3 scripts/run_exact_test.py --lib --name exdev::tests::exdev_gate_cancel_matrix_reports_exact_outcome --serial --case-matrix g1c-cancel-boundaries --expect-case before-prepared --expect-case waiting-fence-ack --expect-case copy-chunk --expect-case after-payload-ready --expect-case before-publish-intent --expect-case after-publish-intent --expect-case after-destination-publish --expect-case source-claim --expect-case private-cleanup --expect-case after-committed
python3 scripts/run_exact_test.py --test exdev --name exdev_gate_two_process_claim_matrix_has_one_winner --serial
python3 scripts/run_exact_test.py --test exdev --name exdev_native_roots_have_different_devices --ignored --serial
python3 scripts/run_exact_test.py --test exdev --name exdev_native_regular_and_symlink_cross_real_devices --ignored --serial
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_gate_receipt_state_matrix_is_truthful --serial --case-matrix g2-recovery-states --expect-case prepared --expect-case payload-ready --expect-case publish-intent-staged --expect-case publish-intent-published --expect-case destination-published-source-pending --expect-case committed --expect-case cleanup-required --expect-case indeterminate --expect-case corrupt --expect-case duplicate --expect-case in-use --expect-case legacy-unmanaged
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_catalog_10000_fixture_is_bounded_and_deterministic
python3 scripts/run_exact_test.py --test recovery_ui --name recovery_catalog_10000_forward_backward_is_bounded_and_complete
python3 scripts/run_exact_test.py --test release_contract --name workflow_has_exact_native_exdev_jobs_and_tier1_steps
python3 scripts/run_exact_test.py --test release_contract --name native_exdev_script_uses_exact_runner_only
```

Expected: the gates FAIL on absent matrix/workflow/script behavior. Direct ignored native gates fail closed without two provisioned roots and are not acceptance evidence.

- [ ] **Step 3: Implement the native script and four required job/step contracts**

Create `scripts/test-exdev-native.sh <root-a> <root-b>` with `set -euo pipefail`. A Python `os.stat` preflight rejects absent, non-directory, non-writable, aliased, or equal-`st_dev` roots. Export canonical absolute `TERSH_EXDEV_ROOT_A/B`, then run only:

```bash
python3 scripts/run_exact_test.py --test exdev --name exdev_native_roots_have_different_devices --ignored --serial
python3 scripts/run_exact_test.py --test exdev --name exdev_native_regular_and_symlink_cross_real_devices --ignored --serial
```

The script contains no raw Cargo filter or direct `--exact`. Ordinary tests retain the injected seam and do not claim native evidence. When and only when `TERSH_NATIVE_EVIDENCE_DIR` is set, the script requires `GITHUB_JOB`, `GITHUB_SHA`, `GITHUB_RUN_ID`, and `GITHUB_RUN_ATTEMPT`, creates that previously absent directory, captures each exact-runner stream/exit separately, and create-new writes canonical `native-exdev-evidence.json` with both verified device identities, the two exact executed test names, result, and output hashes. It never writes `artifact-manifest.json`; the workflow creates that root only after the payload is complete.

In `.github/workflows/ci.yml`, add actual required jobs `native-exdev-linux` and `native-exdev-macos` with matching `name` values. Linux creates a private workspace root plus a separately mounted tmpfs root, runs the script, and unmounts under `if: always()`. macOS creates/attaches a temporary APFS image, runs the script against the workspace root and attached volume, and detaches under `if: always()`. Both jobs use locked toolchain/dependency setup inherited from Plan 1 and fail if cleanup or the script fails.

Each job sets `TERSH_EVIDENCE_ARTIFACT_NAME` to its exact frozen template expanded with `${GITHUB_SHA}`, `${GITHUB_RUN_ID}`, and `${GITHUB_RUN_ATTEMPT}`, sets a job-private absent `TERSH_NATIVE_EVIDENCE_DIR`, and passes it to the native script. After the script succeeds, run `python3 scripts/release_manifest.py artifact-manifest --artifact-root "$TERSH_NATIVE_EVIDENCE_DIR" --producer-job "$GITHUB_JOB" --schema tersh-native-exdev-evidence-v1 --candidate "$GITHUB_SHA" --run-id "$GITHUB_RUN_ID" --run-attempt "$GITHUB_RUN_ATTEMPT"`; this create-new root manifest enumerates the nonempty payload and excludes itself. Then use exactly one step with `id: upload-evidence` and `uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02`, name `$TERSH_EVIDENCE_ARTIFACT_NAME`, and path only that evidence directory. The immediately following step invokes exactly:

```text
python3 scripts/record_artifact_producer.py --producer-job "$GITHUB_JOB" --run-id "$GITHUB_RUN_ID" --run-attempt "$GITHUB_RUN_ATTEMPT" --artifact-id "${{ steps.upload-evidence.outputs.artifact-id }}" --artifact-name "$TERSH_EVIDENCE_ARTIFACT_NAME" --artifact-digest "${{ steps.upload-evidence.outputs.artifact-digest }}"
```

No other native job step uploads an artifact or emits a producer marker. Cleanup is `if: always()` but cannot overwrite the payload/manifest/result or convert a failed test into a passing artifact.

In `.github/workflows/release.yml`, add a required step named `native-exdev-linux` inside exact job `tier1-linux-x86_64` and `native-exdev-macos` inside exact job `tier1-macos-arm64`, using the same provision/run/always-cleanup protocol. These steps test the exact checked-out candidate source before that job emits passed smoke evidence; a skipped native step prevents the Tier-1 job and final manifest from succeeding. Do not create generic substitute job/step names. Preserve the Plan 1/meta workflow inputs, restricted `codex/evidence/**` push bootstrap, exact job IDs, and evidence artifacts; Plan 4 adds only its two native jobs/steps and does not change the shared candidate selector.

- [ ] **Step 4: Update scope, run local GREEN gates, and commit the complete candidate**

Document supported EXDEV regular files/symlinks, explicit rejection of EXDEV directories/special files, duplicate-not-loss cleanup semantics, CLI plus TUI recovery, default skip/no overwrite, and legacy trash limitations. Do not mark G1c/G2 accepted yet.

Run:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
```

Expected: every local command exits 0; only the two environment-bound `exdev_native_*` tests are ignored in the ordinary suite. Then require a clean candidate boundary:

```bash
git add src/exdev.rs scripts/test-exdev-native.sh .github/workflows/ci.yml .github/workflows/release.yml README.md CHANGELOG.md tests/exdev.rs tests/recovery_ui.rs tests/release_contract.rs
git commit -m "test: prepare G1c and Recovery UI candidate"
git status --porcelain=v1 --untracked-files=all
```

Expected: the commit contains every Plan 4 code/test/workflow/documentation change and status output is empty. This committed `HEAD` is the component candidate; Task 6 does not itself accept `impl-06`.

- [ ] **Step 5: Hand the clean candidate to the single implementation-evidence orchestrator**

```bash
test -z "$(git status --porcelain=v1 --untracked-files=all)"
git rev-parse --verify HEAD^{commit}
```

Record that exact 40-hex SHA in the Wave B execution report for `impl-06`, then return to `2026-08-10-tersh-implementation-iteration-evidence.md` Task 7. Only that plan may invoke the shared external-candidate helper, require exact CI jobs `quality-stable`, `msrv-1-88`, `policy`, `native-exdev-linux`, `native-exdev-macos`, require the eight locked release jobs plus matching native steps, run same-candidate five-role closure, and commit only `impl-06.json`. A local ignored test, workflow source text, generic job label, old run, inline `gh` selector, or a Plan 4-specific evidence verifier cannot accept G1c/G2.

## Spec-to-task map and acceptance boundary

| Design requirement | Implemented and proven by |
| --- | --- |
| Worker preflight creates final IDs, one fixed/adjacent reservation, immutable request, then waits for fence ack (`:1324-1357`) | Task 1 collision/pre-ack/panic cleanup tests |
| One claimed `ExdevMoveV1` outer with nested `SourceClaim`, fixed-before-adjacent lock order (`:1359-1383`) | Tasks 1-3 one-control and lock-order tests |
| Consuming typed receipt transition, lifetime-bound mirror/source-claim proofs, and owning `VerifiedTerminalControl` removal | Tasks 1-2 fabricated/replayed-proof, live-lock lifetime, retained-error, and terminal-remove tests |
| Fixed receipt, adjacent bundle, mirrored transitions (`:780-810`) | Tasks 1-2 fixed-intent → adjacent revision/hash → fixed-confirmation evidence |
| EXDEV copy/verify/publish/source-claim protocol (`:812-838`) | Tasks 2-3 prepared-reservation execution and cleanup-only intent tests |
| Duplicate cleanup reuses the original authority with fresh report IDs, plus exact-observation reconciliation and different-cwd discovery (`:840-849`) | Tasks 1 and 3 no-second-control, no-copy, stale-action, fixed-root discovery, and two-process tests |
| Recovery overlay product contract (`:970-977`, `:1037-1044`) | Tasks 4-5 pure reducer, 40x10, observed-selector, and stale-page tests |
| Verified bundle capability is distinct from inspect-only opaque observation; stale/duplicate observations fail closed (`:1404-1419`) | Tasks 4-5 no-capability and no-ID-fallback tests |
| Custom canonical raw deserialization plus live-lock/snapshot-bound, consuming transition authorization (`:1488-1505`) | Tasks 1-3 malformed-receipt, cross-bundle/revision/edge replay, second-use, lifetime, and retained-error-ownership tests |
| One keyed scan worker with Directory/Recovery latest-wins slots, fair alternation, maximum two (`:1395-1402`) | Task 5 replacement, slow-request, starvation, bound, and close tests |
| G1c fault matrix (`:1101`, `:1117-1129`) | Tasks 2-3 and 6 injected matrix plus real-device jobs |
| Exact candidate native evidence | Task 6 supplies the jobs/steps; the shared implementation-evidence Task 7 selects and verifies them against the one `impl-06` candidate SHA |
| Final G2 UI and multi-instance evidence (`:1102`, `:1131-1140`) | Tasks 5-6 observed restore, bounded catalog, and multi-instance evidence |

Tasks 1-6 produce one clean Plan 4 component candidate and cannot independently accept an iteration. `impl-06` closes only when the shared implementation-evidence plan reruns every prior gate, obtains its fail-closed exact-candidate CI/release evidence, completes the five-role closure on that same SHA, and commits only `impl-06.json`. Local ignored tests, generic job names, inline selectors, evidence from different commits, or a later source edit do not satisfy G1c/G2. Once `impl-06` closes, it completes G1c and the thin TUI half of G2, supplying the final filesystem/recovery prerequisites for the Workbench Trusted Core release boundary. It does not implement or claim G3 cluster correctness.
