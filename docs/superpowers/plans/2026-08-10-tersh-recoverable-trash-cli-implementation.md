# Recoverable Trash CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the Plan 2 durable filesystem substrate with crash-consistent per-item trash receipts, conservative reconciliation, and no-clobber CLI list/restore.

**Architecture:** Keep receipt protocol and restore decisions out of `App`: `trash.rs` owns the trash state machine and bundle layout, while `recovery.rs` owns cataloging, reconciliation, and restore requests. Both call the concrete Plan 2 `trusted_fs`, `state_root`, `source_claim`, `operation`, and `mutation` interfaces; neither reimplements path identity, atomic receipt replacement, source claiming, or worker scheduling. This plan proves the engine and CLI slice only; the Recovery TUI and final G2 claim belong to Plan 4.

**Tech Stack:** Rust 1.88, `serde`/`serde_json`, `base64`, `clap`, Unix fd-relative filesystem primitives from Plan 2, Cargo integration tests, `tempfile` test fixtures.

---

## Locked boundaries and existing anchors

- Baseline used for these source anchors: `799cf08f1abd9b546133ed419bf6d4341714e292`.
- Existing unsafe direct trash entry point to replace: `src/fs_ops.rs:97-169` (`trash_path`, `prepare_trash_dir`).
- Existing workbench caller that Plan 2 must already have routed through the mutation lane: `src/app.rs:1582-1602` (`submit_trash`).
- Existing CLI root to extend: `src/main.rs:5-49`.
- Existing trash tests to retire or rewrite: `tests/fs_ops.rs:55-206`.
- Design authority: `docs/superpowers/specs/2026-08-10-tersh-trusted-core-design.md:851-977`, UI deferral at `:1037-1044`, gate row at `:1102`, planning boundary at `:1219-1221`, and controlling adversarial-review clarifications at `:1318-1357` (prepare/reserve before immutable identity), `:1359-1383` (one claimed control), `:1385-1393` (raw child-name capabilities), `:1404-1419` (exact catalog claim), `:1471-1474` (zero-test gate), and `:1488-1505` (custom deserialization and non-replayable verifier-issued proof tokens).

Plan 2 must already provide its exact stable handoff before Task 1 starts; Plan 3 imports those symbols but does not redeclare their impl blocks or change visibility. The required imports are `mutation::{ItemDraft, ItemPreparationFailure, ItemPreparationFailureInner, MutationContext, MutationFenceSpec, MutationIntent, MutationIntentBody, PreparationError, PreparationFailureKind, PreparedAbortOutcome, PreparedReservation, StartupRecoveryDisposition, StartupRecoveryNotice, SubmissionId}`, `operation::{ItemId, ItemOutcome, ItemPlan, OperationId, OperationRequest}`, `source_claim::{ClaimAction, ClaimError, ClaimResult, SourceClaim, SourceClaimAcquireFailure, SourceClaimState, TrustedAdjacentRoot}`, `state_root::{ClaimedControlBundle, ControlBundle, ControlClaimAttempt, ControlEnvelope, ControlHeader, ControlProtocol, ControlRecoveryRef, ControlReservationAbortFailure, ControlReservationFailure, ControlState, ControlTransitionProof, FixedMirrorIntentProof, HostMirrorEdge, InstallationId, MirrorConfirmationProof, MirrorIntent, PendingControl, PendingControlStream, PreparedControlAbortExpectation, PreparedControlAbortFailure, PreparedControlAbortVerifyFailure, ReserveControlError, StateRoot, TerminalExpectation, TerminalVerifyFailure, VerifiedPreparedControlAbort, VerifiedTerminalControl}`, and `trusted_fs::{AdjacentReceiptFacts, AtomicReceiptCreateAbortFailure, AtomicReceiptCreateFailure, AtomicReceiptCreation, AtomicReceiptFile, ChildEnumerator, ChildObservation, ClaimedChildLock, DurableReceipt, ObjectKind, ObservedRawName, OwnedLockedReceipt, OwnedLockedReceiptAdvanceError, OwnedLockedReceiptVerifyError, PathIdentity, PathIdentityKey, RawUnixName, RawUnixPath, ReceiptAdvanceStage, ReceiptAfterFailure, ReceiptAfterFailureBytes, TrustedDir, TrustedFsError}`.

`RawUnixPath::capture` is fallible and returns only an absolute, non-empty, NUL-free path with no lexical `.`/`..`; callers propagate the error and retain UI/CLI input. `RawUnixName` alone is a child mutation capability. `ObservedRawName` has no serde, raw-byte, path, name, or reverse-conversion API; corrupt validated entries are downgraded only through consuming `RawUnixName::into_observed`. `pending_controls` returns `PendingControlStream`, not `Vec`.

`ControlEnvelope` is the concrete Plan 2 envelope. Plan 3 extends `MutationIntentBody` with `Restore(RestoreIntent)`, `ControlState` with typed `Trash { host: TrashControlState, source_claim: Option<SourceClaimState>, mirror_intent: Option<MirrorIntent> }` and `Restore { host: RestoreControlState, source_claim: Option<SourceClaimState>, mirror_intent: Option<MirrorIntent> }`, and `PreparedReservation` with private Trash/Restore reservations; it does not introduce generic parallel families. `SourceClaim<'a>` advances only the nested field through `TrustedAdjacentRoot<'a>`, which retains the already claimed fixed control and derives its item from the fixed header.

`SourceClaimProof`, `AdjacentReceiptFacts`, and `MirrorConfirmationProof` are opaque, non-`Clone`, non-serde tokens with private fields; terminal authority is the owning `VerifiedTerminalControl` typestate. An owning trash transaction obtains `AdjacentReceiptFacts` only through `AtomicReceiptFile::verify_current_locked_synced` while retaining its bundle lock, builds the exact owner-approved confirmation next state, then calls its own fixed control's `confirm_mirror(adjacent_facts, &next_state)`; no adjacent wrapper constructs a proof directly from caller hash/bool input. A transition that must leave the borrow scope first consumes its actual receipt plus lock into `OwnedLockedReceipt`. Plan 3's receipt proof owns that locked receipt and the rest of the one transaction, adds exact bundle/revision/edge/fact bindings, and is itself consumed by `advance_receipt`. `PathIdentityKey` remains Plan 2's total-order projection; Plan 3 compares it rather than display/debug text.

Plan 1 must already provide `python3 scripts/run_exact_test.py (--test <target> | --lib) --name <full-name> [--serial] [--case-matrix <id> --expect-case <case> ...]`. The mutually exclusive selector first runs Cargo's list mode with `--locked`, fails unless the exact full name is discovered once, executes only that name, and fails if the libtest result reports zero executed tests or the frozen case matrix differs. Every focused command below uses this helper; complete target/repository regressions continue to call `cargo test --locked` directly. Parameterized tests additionally assert their exact table length inside the named test before iterating cases.

If any symbol or invariant above is absent, stop before Task 1 and finish Plan 2. Each item has exactly one fixed-root typed outer `ControlEnvelope`; Plan 3 uses `ControlProtocol::TrashIngestV1`, and later restore transitions use `ControlProtocol::RestoreV1` in a newly preflighted operation rather than nesting a second fixed control under the same item. Do not add a second identity type, receipt writer, source-claim algorithm, fixed control, or mutation queue in this plan. The only per-trash-bundle lock is Plan 2's `ClaimedChildLock`, owned by the private transaction or its consuming transition proof.

### Task 1: Versioned receipt model and raw Unix paths

**Files:**
- Create: `src/trash.rs`
- Modify: `src/lib.rs`
- Create: `tests/trash_receipt.rs`

- [ ] **Step 1: Write the failing receipt round-trip tests**

Add tests named exactly:

```rust
#[test]
fn receipt_roundtrips_non_utf8_original_path() {
    let raw = RawUnixPath::from_bytes(b"/tmp/a\xff".to_vec()).unwrap();
    let receipt = receipt_fixture(raw.clone());
    let decoded = TrashReceipt::from_json_bytes(&receipt.to_json_bytes().unwrap()).unwrap();
    assert_eq!(decoded.original_path, raw);
}

#[test]
fn receipt_deserializer_rejects_unknown_schema_from_raw_json() {
    let receipt = receipt_fixture(RawUnixPath::from_bytes(b"/tmp/a".to_vec()).unwrap());
    let mut raw: serde_json::Value = serde_json::from_slice(&receipt.to_json_bytes().unwrap()).unwrap();
    raw["schema"] = serde_json::json!(2);
    let bytes = serde_json::to_vec(&raw).unwrap();
    assert!(TrashReceipt::from_json_bytes(&bytes).is_err());
}

#[test]
fn receipt_serializer_rejects_unknown_schema() {
    let mut receipt = receipt_fixture(RawUnixPath::from_bytes(b"/tmp/a".to_vec()).unwrap());
    receipt.schema = 2;
    assert!(receipt.to_json_bytes().is_err());
}

#[test]
fn receipt_rejects_invalid_state_transition() {
    let committed = receipt_fixture_at(
        RawUnixPath::from_bytes(b"/tmp/a".to_vec()).unwrap(),
        TrashState::Committed,
        5,
    );
    assert!(committed.transition(TrashState::Prepared).is_err());
}

#[test]
fn receipt_uses_top_level_item_id_as_bundle_name() {
    let receipt = receipt_fixture(RawUnixPath::from_bytes(b"/tmp/a".to_vec()).unwrap());
    let name = receipt.item_id.to_string();
    assert_eq!(name.len(), 32);
    assert!(name.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
}
```

Add concrete `receipt_fixture(RawUnixPath) -> TrashReceipt` and `receipt_fixture_at(RawUnixPath, TrashState, u64) -> TrashReceipt` helpers at the top of the test file using Plan 2's deterministic test identity helpers and fixed `OperationId`/`ItemId` values. Assert the serialized path object is exactly `platform=unix`, `encoding=base64-raw-os-bytes`, and `bytes_b64=<RFC-4648 URL-safe base64 without padding>`, matching `RawUnixPath::encoded()`; assert no display string is accepted as an authoritative path. The raw-JSON deserialization test must mutate a valid `serde_json::Value` and serialize that value with `serde_json::to_vec`; it must not call the validating `TrashReceipt::to_json_bytes` after setting `schema = 2`.

- [ ] **Step 2: Run the focused tests and confirm RED**

Run each target exactly:

```bash
python3 scripts/run_exact_test.py --test trash_receipt --name receipt_roundtrips_non_utf8_original_path
python3 scripts/run_exact_test.py --test trash_receipt --name receipt_deserializer_rejects_unknown_schema_from_raw_json
python3 scripts/run_exact_test.py --test trash_receipt --name receipt_serializer_rejects_unknown_schema
python3 scripts/run_exact_test.py --test trash_receipt --name receipt_rejects_invalid_state_transition
python3 scripts/run_exact_test.py --test trash_receipt --name receipt_uses_top_level_item_id_as_bundle_name
cargo test --locked --test trash_receipt
```

Expected: FAIL because `tersh::trash::{TrashReceipt, TrashState, TransferMethod}` and its transition validator do not exist.

- [ ] **Step 3: Implement the receipt types and transition validator**

Define these exact types in `src/trash.rs` and export the module from `src/lib.rs`:

```rust
pub const TRASH_SCHEMA_V1: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TrashState {
    Prepared,
    PayloadReady,
    PayloadPublished,
    SourceRemovalPending,
    Committed,
    RestoreClaimed,
    RestorePublishIntent,
    RestoreDestinationPublished,
    RestorePayloadRemovalPending,
    Restored,
    CleanupRequired,
    Indeterminate,
    Quarantined,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransferMethod { Rename, CopyRegular, CopySymlink }

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RestorePublication { Rename, StagedCopy }

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RestoreReceiptIntent {
    #[serde(with = "receipt_path_serde")]
    pub destination: RawUnixPath,
    pub destination_parent: PathIdentity,
    pub payload_identity: PathIdentity,
    pub expected_published_identity: PathIdentity,
    pub publication: RestorePublication,
    pub staged_identity: Option<PathIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrashReceipt {
    pub schema: u32,
    pub item_id: ItemId,
    pub operation_id: OperationId,
    pub revision: u64,
    pub state: TrashState,
    #[serde(with = "receipt_path_serde")]
    pub original_path: RawUnixPath,
    pub original_parent: PathIdentity,
    pub object_kind: ObjectKind,
    pub object_identity: PathIdentity,
    pub trashed_at_unix_ns: u128,
    pub transfer: TransferMethod,
    pub payload_identity: Option<PathIdentity>,
    pub restore: Option<RestoreReceiptIntent>,
}

impl TrashReceipt {
    pub fn transition(&self, next: TrashState) -> Result<Self>;
    pub fn to_json_bytes(&self) -> Result<Vec<u8>>;
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self>;
}
```

Add private module `receipt_path_serde` in `src/trash.rs`. Its serializer emits exactly `{ "platform": "unix", "encoding": "base64-raw-os-bytes", "bytes_b64": raw.encoded() }`; its deserializer rejects every other platform/encoding and calls `RawUnixPath::from_bytes` only after URL-safe-no-padding base64 validation. The encoded field remains private; do not use derived `RawUnixPath` JSON inside a receipt.

`RestoreReceiptIntent` records raw destination, destination-parent identity, payload identity, expected published identity, publication method, and staged identity. Reject unknown schema, duplicate/non-increasing revision, impossible state edges, invalid base64, and non-Unix authoritative paths.

- [ ] **Step 4: Run the focused tests and confirm GREEN**

Run:

```bash
python3 scripts/run_exact_test.py --test trash_receipt --name receipt_roundtrips_non_utf8_original_path
python3 scripts/run_exact_test.py --test trash_receipt --name receipt_deserializer_rejects_unknown_schema_from_raw_json
python3 scripts/run_exact_test.py --test trash_receipt --name receipt_serializer_rejects_unknown_schema
python3 scripts/run_exact_test.py --test trash_receipt --name receipt_rejects_invalid_state_transition
python3 scripts/run_exact_test.py --test trash_receipt --name receipt_uses_top_level_item_id_as_bundle_name
cargo test --locked --test trash_receipt
```

Expected: PASS; all five receipt tests pass, the raw deserializer and validating serializer are exercised independently, and no other test runs.

- [ ] **Step 5: Commit the receipt contract**

```bash
git add src/trash.rs src/lib.rs tests/trash_receipt.rs
git commit -m "feat: define versioned trash receipts"
```

Attribute handoff rule for the next two tasks: the shorthand “remove Plan 2's temporary `#[expect(dead_code)]`” means remove the entire exact `#[cfg_attr(not(test), expect(dead_code, reason = "..."))]`. Plan 2 needs that conditional expectation only in its normal library build; its private lib-test harness compiles without it. Task 2 removes the `confirm_mirror` attribute with the first Trash confirmation caller, and Task 3 removes the `verify_host_mirror_intent` attribute with the first post-initial Trash edge.

### Task 2: Trusted trash root and bundle reservation

**Files:**
- Modify: `src/trash.rs`
- Modify: `src/state_root.rs`
- Modify: `tests/trash_receipt.rs`

- [ ] **Step 1: Write the failing trust, streaming, and claim-capability tests**

Add exact tests `trash_root_initializes_complete_v1_no_replace`, `trash_root_rejects_symlink_wrong_owner_mode_or_installation_id`, `trash_root_concurrent_initializers_share_verified_winner`, `trash_root_crash_before_parent_sync_disables_mutation`, `trash_store_cannot_initialize_or_reserve_without_claimed_outer_control`, `trash_store_init_and_reserve_crash_order_is_fixed_intent_first`, `trash_store_streams_child_names_fd_relative_without_accumulating`, `trash_transaction_owns_fixed_bundle_and_lock_until_terminal`, `trash_consuming_entry_errors_retain_owning_authority`, `claim_existing_requires_consumed_matching_control`, `claim_existing_pre_effect_failure_returns_original_claim_for_terminal_cleanup`, `claim_existing_post_rename_failure_remains_discoverable_and_owned`, `transaction_item_id_comes_only_from_fixed_header`, `claim_existing_rechecks_observed_identity_under_lock`, and `claim_existing_moves_whole_bundle_no_replace_and_syncs_parents`. Put the fourteen tests that call or inspect crate-private mutation/claim ownership in `trash::tests`; keep only `trash_store_streams_child_names_fd_relative_without_accumulating`, which uses the public catalog surface, in `tests/trash_receipt.rs`. The fixture must create `.tersh-trash/v1.init.<item-id>` and inject failures before/after fixed init intent, owner write, owner sync, init-dir sync, no-replace root publish, fixed adjacent mirror intent, bundle create, receipt sync, and both claim-parent syncs. `open_for_catalog` on an absent root must return an empty iterator without creating `.tersh-trash`; both mutation entry points must consume an owning `ClaimedControlBundle`, and success or failure must retain that original claim plus every created/opened adjacent handle until a consuming terminal/reconciliation outcome. The streaming test creates 10,000 bundle names and asserts the iterator holds at most the current decoded name plus its directory cursor, not a `Vec` of observations. The API tests prove there is no caller-supplied adjacent `ItemId`, borrowed-control overload, public handle field, way to pair an observation with a different fixed header, or consuming `Result::Err` that discards the claim. No production item is made public for these gates.

- [ ] **Step 2: Run the trust tests and confirm RED**

Run each target exactly:

```bash
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_root_initializes_complete_v1_no_replace
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_root_rejects_symlink_wrong_owner_mode_or_installation_id
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_root_concurrent_initializers_share_verified_winner --serial
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_root_crash_before_parent_sync_disables_mutation
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_store_cannot_initialize_or_reserve_without_claimed_outer_control
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_store_init_and_reserve_crash_order_is_fixed_intent_first
python3 scripts/run_exact_test.py --test trash_receipt --name trash_store_streams_child_names_fd_relative_without_accumulating
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_transaction_owns_fixed_bundle_and_lock_until_terminal
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_consuming_entry_errors_retain_owning_authority
python3 scripts/run_exact_test.py --lib --name trash::tests::claim_existing_requires_consumed_matching_control
python3 scripts/run_exact_test.py --lib --name trash::tests::claim_existing_pre_effect_failure_returns_original_claim_for_terminal_cleanup
python3 scripts/run_exact_test.py --lib --name trash::tests::claim_existing_post_rename_failure_remains_discoverable_and_owned
python3 scripts/run_exact_test.py --lib --name trash::tests::transaction_item_id_comes_only_from_fixed_header
python3 scripts/run_exact_test.py --lib --name trash::tests::claim_existing_rechecks_observed_identity_under_lock
python3 scripts/run_exact_test.py --lib --name trash::tests::claim_existing_moves_whole_bundle_no_replace_and_syncs_parents
cargo test --locked --test trash_receipt
```

Expected: FAIL because the split catalog/mutation open APIs, fixed-intent-gated reservation, and exact claim capability do not exist.

- [ ] **Step 3: Implement the trusted root and bundle API**

Add this API:

```rust
use std::sync::Arc;

pub struct TrashStore {
    root: Option<TrustedDir>,
    staging: Option<TrustedDir>,
    items: Option<TrustedDir>,
    claims: Option<TrustedDir>,
    quarantine: Option<TrustedDir>,
    legacy_root: Option<TrustedDir>,
    installation_id: InstallationId,
}
pub struct TrashBundle {
    name: RawUnixName,
    location: BundleLocation,
    dir: Arc<TrustedDir>,
    identity: PathIdentity,
    claim_lock: ClaimedChildLock,
    receipt_file: AtomicReceiptFile,
}
pub(crate) struct ClaimedTrashRootTransaction {
    control: ClaimedControlBundle,
    store: TrashStore,
}
pub(crate) struct ClaimedTrashTransaction {
    control: ClaimedControlBundle,
    store: TrashStore,
    bundle: TrashBundle,
}
pub(crate) struct PartialTrashBundle {
    name: Option<RawUnixName>,
    location: BundleLocation,
    dir: Option<Arc<TrustedDir>>,
    identity: Option<PathIdentity>,
    claim_lock: Option<ClaimedChildLock>,
    receipt: AtomicReceiptCreation,
}
impl PartialTrashBundle {
    fn record_receipt_create_failure(self, failure: AtomicReceiptCreateFailure) -> Self;
}
pub(crate) enum TrashAdjacentOwnership {
    None,
    Partial(PartialTrashBundle),
    ReceiptCreateAbortFailed {
        partial: PartialTrashBundle,
        failure: AtomicReceiptCreateAbortFailure,
    },
    Complete(TrashBundle),
}
pub(crate) enum TrashEntryDiskTruth {
    NoAdjacentObject,
    Partial {
        bundle: Option<PathIdentityKey>,
        lock: Option<PathIdentityKey>,
        receipt: Option<PathIdentityKey>,
    },
    InLocation { location: BundleLocation, identity: PathIdentityKey },
    Unreadable { escaped_error: String },
}
pub(crate) struct ClaimedTrashRecoveryState {
    control: ClaimedControlBundle,
    store: Option<TrashStore>,
    adjacent: TrashAdjacentOwnership,
    observed: Option<VerifiedBundleRef>,
    stage: TrashEntryStage,
    disk_truth: TrashEntryDiskTruth,
}
pub(crate) struct ClaimedTrashForeignCollision {
    control: ClaimedControlBundle,
    store: TrashStore,
    trusted_location_parent: Arc<TrustedDir>,
    bundle_name: RawUnixName,
    observed_foreign_identity: PathIdentity,
}
pub(crate) enum TrashEntryStage {
    FixedValidatedNoAdjacentEffect,
    RootInitializationMayExist,
    BundleCreated,
    ClaimLockCreated,
    ReceiptCreateFailed,
    ReceiptCreated,
    ReceiptSynced,
    BundleClaimRenamed,
    ClaimSourceParentSync,
    ClaimDestinationParentSync,
    ClaimedBundleReopenFailed,
}
pub(crate) enum TrashMutationEntryFailure {
    PreAdjacent {
        control: ClaimedControlBundle,
        store: Option<TrashStore>,
        observed: Option<VerifiedBundleRef>,
        kind: ClaimExistingFailureKind,
    },
    AdjacentOwned {
        recovery: ClaimedTrashRecoveryState,
        kind: ClaimExistingFailureKind,
    },
    ForeignCollision(ClaimedTrashForeignCollision),
}
pub(crate) enum ClaimExistingFailureKind {
    InvalidRoot,
    InUse,
    StaleObservation,
    Contradictory,
    InspectOnly,
    Io,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrashRootIntent {
    work_root: RawUnixPath,
    work_root_identity: PathIdentity,
    trash_parent_name: RawUnixName,
    expected_trash_parent_identity: Option<PathIdentity>,
    init_root_name: RawUnixName,
    expected_init_root_identity: Option<PathIdentity>,
    final_root_name: RawUnixName,
    expected_final_root_identity: Option<PathIdentity>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrashBundleSelector {
    work_root: RawUnixPath,
    work_root_identity: PathIdentity,
    trash_parent_name: RawUnixName,
    trash_parent_identity: PathIdentity,
    v1_root_name: RawUnixName,
    v1_root_identity: PathIdentity,
    location: BundleLocation,
    location_parent_name: RawUnixName,
    location_parent_identity: PathIdentity,
    raw_name: RawUnixName,
    bundle_identity: PathIdentity,
    receipt_identity: PathIdentity,
    claim_lock_identity: PathIdentity,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrashBundleIntent {
    location: BundleLocation,
    location_parent_name: RawUnixName,
    expected_location_parent_identity: Option<PathIdentity>,
    raw_name: RawUnixName,
    expected_bundle_identity: Option<PathIdentity>,
    expected_receipt_identity: Option<PathIdentity>,
    expected_claim_lock_identity: Option<PathIdentity>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrashControlState {
    original_path: RawUnixPath,
    expected_source: PathIdentity,
    root_intent: TrashRootIntent,
    bundle_intent: TrashBundleIntent,
    mirror: ReceiptMirror,
    phase: TrashState,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReceiptMirror {
    bundle: Option<VerifiedBundleFact>,
    confirmed_revision: Option<u64>,
    confirmed_receipt_sha256: Option<[u8; 32]>,
    pending_phase: Option<TrashState>,
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum BundleLocation { Staging, Items, Claims, Quarantine, LegacyUnmanaged }
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum CatalogClass { Verified, InspectOnly }
#[derive(Clone, Eq, PartialEq)]
pub struct CatalogOpaqueName {
    observed: ObservedRawName,
    escaped_display: String,
}
#[derive(Clone, Eq, PartialEq)]
pub enum CatalogIdentity { Item(ItemId), Opaque(CatalogOpaqueName) }
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CatalogObservedIdentity { Captured(PathIdentityKey), Unavailable }
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogKey {
    pub class: CatalogClass,
    pub id_or_raw_name: CatalogIdentity,
    pub location: BundleLocation,
    pub observed_identity: CatalogObservedIdentity,
}
#[derive(Clone, Eq, PartialEq)]
pub struct VerifiedBundleRef {
    class: CatalogClass,
    item_id: ItemId,
    raw_name: RawUnixName,
    location: BundleLocation,
    observed_identity: PathIdentity,
    observed_work_root: RawUnixPath,
    observed_work_root_identity: PathIdentity,
    observed_location_parent_identity: PathIdentity,
}
impl std::fmt::Debug for CatalogOpaqueName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}
impl std::fmt::Debug for CatalogIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}
impl std::fmt::Debug for VerifiedBundleRef {
    // Emit only complete non-authorizing key plus item ID; never raw name bytes.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VerifiedBundleFact {
    item_id: ItemId,
    location: BundleLocation,
    selector: TrashBundleSelector,
}

pub(crate) fn validate_trash_mirror_confirmation(
    current: &TrashControlState,
    next: &TrashControlState,
    expected_intent: &MirrorIntent,
    confirmed_adjacent: &AdjacentReceiptFacts<'_>,
) -> Result<(), TrustedFsError>;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectOnlyRef {
    class: CatalogClass,
    id_or_raw_name: CatalogIdentity,
    location: BundleLocation,
    observed_identity: CatalogObservedIdentity,
}
pub enum BundleObservation {
    VerifiedCandidate { bundle_ref: VerifiedBundleRef, receipt: TrashReceipt, payload_identity: Option<PathIdentity> },
    InspectOnly { inspect_ref: InspectOnlyRef, escaped_error: String },
}
pub struct BundleObservationStream<'a> {
    store: &'a TrashStore,
    locations: [BundleLocation; 5],
    location_index: usize,
    active_names: Option<ChildEnumerator>,
}

impl VerifiedBundleRef {
    pub fn key(&self) -> CatalogKey;
    pub fn item_id(&self) -> ItemId;
}

impl InspectOnlyRef {
    pub fn key(&self) -> CatalogKey;
    pub fn escaped_display(&self) -> &str;
}

impl Ord for CatalogIdentity {
    fn cmp(&self, other: &Self) -> Ordering;
}

impl PartialOrd for CatalogIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>;
}

impl Ord for CatalogKey {
    fn cmp(&self, other: &Self) -> Ordering;
}

impl PartialOrd for CatalogKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>;
}

impl Iterator for BundleObservationStream<'_> {
    type Item = Result<BundleObservation>;
}

impl TrashStore {
    pub fn open_for_catalog(work_root: &Path, state_root: &StateRoot) -> Result<Self>;
    pub(crate) fn open_or_initialize_for_mutation(work_root: &Path, state_root: &StateRoot, control: ClaimedControlBundle, intent: &TrashRootIntent) -> Result<ClaimedTrashRootTransaction, TrashMutationEntryFailure>;
    pub fn observations(&self) -> Result<BundleObservationStream<'_>>;
    pub(crate) fn claim_existing(self, control: ClaimedControlBundle, observed: VerifiedBundleRef) -> Result<ClaimedTrashTransaction, TrashMutationEntryFailure>;
}

impl ClaimedTrashRootTransaction {
    pub(crate) fn create_initial_bundle_matching_intent(self, receipt: TrashReceipt) -> Result<ClaimedTrashTransaction, TrashMutationEntryFailure>;
}

fn trash_receipt_name() -> RawUnixName;
fn trash_claim_lock_name() -> RawUnixName;

impl ClaimedTrashTransaction {
    pub(crate) fn publish_to_items(&mut self) -> Result<()>;
}
```

Use Plan 2 fd-relative/no-follow/create-new/no-replace/sync primitives. `open_for_catalog` is existing-only: an absent `.tersh-trash/v1` is an empty catalog and causes no mkdir, owner write, temp path, or sync. In this task, before either mutation entry point exists, extend the concrete Plan 2 state enum with `ControlState::Trash { host: TrashControlState, source_claim: Option<SourceClaimState>, mirror_intent: Option<MirrorIntent> }` and teach `state_root.rs` to delegate an initial Trash confirmation to `validate_trash_mirror_confirmation`; no Task 3 type is needed to compile or run Task 2. This same source edit removes Plan 2's temporary `#[expect(dead_code)]` from `ClaimedControlBundle::confirm_mirror`, because the initial Trash confirmation is its first normal-library caller; retaining the expectation would fail `unfulfilled_lint_expectations` under `-D warnings`. Mutation preparation first records and syncs the complete `TrashRootIntent { work_root, work_root_identity, trash_parent_name, expected_trash_parent_identity, init_root_name, expected_init_root_identity, final_root_name, expected_final_root_identity }` and `TrashBundleIntent { location: Staging, location_parent_name, expected_location_parent_identity, raw_name, expected_bundle_identity: None, expected_receipt_identity: None, expected_claim_lock_identity: None }` inside `TrashControlState`, then calls `open_or_initialize_for_mutation`. All root and location components are captured as `RawUnixName` before fixed reservation; the raw bundle name is also chosen and persisted there, so no restart derives any child from the fixed header's `ItemId` or parses it back out of an absolute path. The only absolute selector is the existing work root paired with its full identity. The method rereads the control and rejects a wrong protocol/header/revision, replaced work root/trash parent/location parent, different raw name, or any identity/name mismatch before touching the work root. A parent not yet created is recorded with `expected_*_identity: None`; that value authorizes only create-new/no-replace under its already trusted parent and never authorizes opening, replacing, or deleting a pre-existing child. Only then may it initialize `v1`: build the complete private `init_root_name`, write mode-0600 `owner.json` with schema and `StateRoot::installation_id()`, sync it, publish to `final_root_name` no-replace, and sync the trusted `trash_parent_name` directory. Initial confirmation records every freshly observed full parent/root/location/bundle/receipt/lock identity; no later validator treats `None` as wildcard authority. Existing unversioned children are never adopted, moved, or deleted, but they do not block safe creation/opening of the independent trusted `v1`; cataloging reports each as `InspectOnly`/`LegacyUnmanaged`. Every v1 bundle owns an exclusive no-follow `claim.lock`.

All Plan 3 receipt creation/opening uses only the private infallible helpers `trash_receipt_name()` and `trash_claim_lock_name()`, whose bodies respectively call `RawUnixName::from_bytes(b"receipt.json".to_vec()).expect("fixed receipt name is valid")` and `RawUnixName::from_bytes(b"claim.lock".to_vec()).expect("fixed claim-lock name is valid")`. The two allowed substrate calls are exactly `AtomicReceiptFile::create_new_json(Arc::clone(&bundle_dir), trash_receipt_name(), trash_claim_lock_name(), &receipt)` and `AtomicReceiptFile::open_existing(Arc::clone(&bundle_dir), trash_receipt_name(), trash_claim_lock_name())`; every Trash and Restore path supplies that lock name, and no overload or caller-provided name exists. Bundle-directory ownership is therefore `Arc<TrustedDir>` everywhere it coexists with an `AtomicReceiptFile`, so passing the substrate its owning parent cannot strand the host on either success or error.

`BundleLocation` has the locked rank `Staging < Items < Claims < Quarantine < LegacyUnmanaged`; `CatalogClass` has `Verified < InspectOnly`. Implement `CatalogIdentity::cmp` explicitly: `Item` sorts before `Opaque`, item IDs compare their canonical 32 lowercase hexadecimal bytes, and `CatalogOpaqueName` delegates to its sole `ObservedRawName` field's private raw-byte order. There is no `CatalogOpaqueNameInner::Validated` branch. Do not rely on an incidental `ItemId` derive. `CatalogKey` contains and orders exactly `(class, id_or_raw_name, location, observed_identity)`, with `Captured(identity.stable_key()) < Unavailable`; because each page is scoped to one work root, root identity is not part of the row ordering key. `VerifiedBundleRef` privately carries the validated raw name plus the same class/item/location/captured identity and also the full observed work-root and location-parent identities. It is the only live observation capability accepted by `claim_existing`; its only public projections are `key()` and `item_id()`. `VerifiedBundleFact` is explicitly non-authorizing serialized recovery data and retains the full `TrashBundleSelector`: raw absolute work root plus full identity, raw child names plus full identities for trash parent/v1/location parent, the explicit location enum, validated raw bundle name, and full bundle/receipt/lock identities. It never derives a child name from `ItemId` or parses one from a stored path. On mutation or restart, the owner starts from the trusted work-root selector, reopens every recorded child fd-relatively in order, compares every full identity, then mints a fresh private `VerifiedBundleRef`; a work-root, trash-parent, v1-root, location-parent, bundle, receipt, or lock replacement fails closed. `CatalogOpaqueName`, `CatalogIdentity`, and `VerifiedBundleRef` implement manual escaped/non-authorizing `Debug` so Plan 2's derived `Clone + Debug` mutation intent remains compilable without leaking raw child bytes. `InspectOnlyRef` carries only non-operational catalog identity and escaped display. Neither it nor `CatalogKey` can yield or be converted to `RawUnixName`.

`TrashStore::observations` walks each verified directory through `TrustedDir::enumerate_children`; it never reconstructs an absolute child path and never accumulates a `Vec`. For each `ChildObservation::Validated { name, identity }` it privately retains the validated name and already captured no-follow whole-bundle identity; a valid bundle/receipt becomes `VerifiedCandidate`, while invalid receipt/header/content must consume `name.into_observed()` before constructing `CatalogOpaqueName` and emits an `InspectOnlyRef` with `CatalogObservedIdentity::Captured(identity.stable_key())`. `ChildObservation::InspectOnly { name: ObservedRawName, escaped_error }` moves that already non-capability value directly into `CatalogOpaqueName`, always uses `CatalogObservedIdentity::Unavailable`, and never becomes a `RawUnixName`; it does not stop later siblings. A name-less `ChildEnumerator::Err` terminates the observation stream as incomplete and is never converted to `None`. The stream retains no page or catalog collection.

`open_or_initialize_for_mutation` consumes the claimed control, validates the root intent, and returns private `ClaimedTrashRootTransaction { control, store }`; it never returns an unbound store or pretends a bundle exists before reservation. Its error is also owning: before any adjacent effect, `TrashMutationEntryFailure::PreAdjacent` returns the original claim unchanged plus any already opened `TrashStore`; after root initialization may have changed disk, `AdjacentOwned` retains that claim plus every opened trusted handle and a durable/re-read stage for reconciliation. Before `StateRoot::reserve_control`, preparation canonicalizes the complete initial `TrashReceipt` and embeds its revision/hash as `mirror_intent: Some(...)` plus the matching pending phase in the create-new fixed envelope. `ClaimedTrashRootTransaction::create_initial_bundle_matching_intent(self, receipt)` derives operation/item IDs from that fixed header, verifies the passed canonical bytes exactly match the already durable initial intent, then creates/locks/syncs the adjacent bundle/receipt without another fixed advance. If create-new of the persisted bundle name returns `EEXIST`, it does not open, lock, rename, delete, or adopt that child: it captures the no-follow full identity under the retained trusted location parent and returns `ForeignCollision(ClaimedTrashForeignCollision)` with the original fixed claim, store, parent `Arc`, raw name, and identity. An unreadable or identity-unstable child is not a retryable collision and remains owning `Indeterminate`. Receipt creation starts as `AtomicReceiptCreation::NotStarted`; success installs `Created(file)`, while every `AtomicReceiptCreateFailure` is moved into `CreateFailed(failure)` through `record_receipt_create_failure` before returning `AdjacentOwned`. Owner reconciliation uses `std::mem::replace(&mut partial.receipt, AtomicReceiptCreation::NotStarted)` and matches the old slot by value: `CreateFailed(failure)` is consumed through `failure.abort()`; success continues reverse cleanup, while `AtomicReceiptCreateAbortFailure` is retained beside that now-`NotStarted` partial in `TrashAdjacentOwnership::ReceiptCreateAbortFailed`, and only its consuming `retry` may continue. A repeated abort failure returns the same owning variant with the updated failure; it is never reduced to `TrustedFsError`, a bool, or an observed receipt identity. `Created(file)` remains owning and is removed only by Plan 2's consuming verified receipt removal path while the exact `claim.lock` is held. With the live new receipt facts the successful path constructs the complete `TrashBundleSelector`, proposes the exact next host with `expected_final_root_identity` and `mirror.bundle` filled, and calls `confirm_mirror`; `state_root.rs` accepts only `validate_trash_mirror_confirmation` and clears the matching pending mirror. Any reservation failure returns the owning pre/post-adjacent error rather than a partially initialized success type. A crash before fixed creation creates nothing; a crash after fixed intent remains fixed-root discoverable through the retained root selectors. The negative API test source-checks that no mutation/open/reserve overload omits or merely borrows the claimed control, that no adjacent method accepts a caller `ItemId`, that all transaction/plan handle fields are private, that every receipt open/create supplies the exact lock name, and that every consuming capability entry returns either success ownership or an owning failure/recovery typestate. `trash_consuming_entry_errors_retain_owning_authority` injects every Plan 2 receipt-create stage and proves the create/abort/retry chain retains the parent `Arc`, receipt and lock names, any created fd/identity, fixed claim, remaining bundle handles, monotonic stage, and re-read truth.

`claim_existing` is the only observation-to-mutation bridge and consumes the `TrashStore`, an unforgeable non-`Deserialize` `VerifiedBundleRef`, and the corresponding owning `ClaimedControlBundle`. It first verifies the fixed protocol/header/host expected recovery item and root, then rejects `Quarantine`, checks the same valid item name in all four v1 locations, and returns `PreAdjacent { control, store: Some(store), observed, kind: Contradictory }` before adjacent mutation if more than one exists. It opens the observed bundle and `claim.lock` no-follow, takes the nonblocking exclusive lock while the fixed claim remains owned, rechecks location/raw name/full identity under both locks, then no-replace renames the whole bundle to `claims` and syncs both location parents. If the observation already names `Claims`, it reopens and revalidates without a second rename. After rename it accepts only `same_object`, captures a fresh claimed snapshot, reopens and strictly validates the receipt/header while still holding both locks, and returns `ClaimedTrashTransaction`; no raw `TrashBundle` escapes. Missing/stale identity, wrong fixed binding, `InUse`, or contradiction before rename returns the original claim, store, and exact observation in `PreAdjacent`, so caller cleanup needs no ItemId reopen. Any failure after rename/sync returns `AdjacentOwned { recovery }` retaining the original claim, bundle directory, lock/receipt if opened, store, observation, stage, and re-read disk truth; it cannot degrade to a naked error or silently drop ownership.

- [ ] **Step 4: Run trust tests and the Plan 2 trust regression tests**

Run:

```bash
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_root_initializes_complete_v1_no_replace
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_root_rejects_symlink_wrong_owner_mode_or_installation_id
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_root_concurrent_initializers_share_verified_winner --serial
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_root_crash_before_parent_sync_disables_mutation
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_store_cannot_initialize_or_reserve_without_claimed_outer_control
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_store_init_and_reserve_crash_order_is_fixed_intent_first
python3 scripts/run_exact_test.py --test trash_receipt --name trash_store_streams_child_names_fd_relative_without_accumulating
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_transaction_owns_fixed_bundle_and_lock_until_terminal
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_consuming_entry_errors_retain_owning_authority
python3 scripts/run_exact_test.py --lib --name trash::tests::claim_existing_requires_consumed_matching_control
python3 scripts/run_exact_test.py --lib --name trash::tests::claim_existing_pre_effect_failure_returns_original_claim_for_terminal_cleanup
python3 scripts/run_exact_test.py --lib --name trash::tests::claim_existing_post_rename_failure_remains_discoverable_and_owned
python3 scripts/run_exact_test.py --lib --name trash::tests::transaction_item_id_comes_only_from_fixed_header
python3 scripts/run_exact_test.py --lib --name trash::tests::claim_existing_rechecks_observed_identity_under_lock
python3 scripts/run_exact_test.py --lib --name trash::tests::claim_existing_moves_whole_bundle_no_replace_and_syncs_parents
cargo test --locked --test trash_receipt
cargo test --locked --test trusted_fs --test state_root
```

Expected: PASS; all fifteen trash-root/stream/claim tests pass, catalog open is side-effect-free, every adjacent creation is preceded by durable fixed intent, transaction/error capabilities never escape or drop ownership, observation memory is independent of catalog size, and Plan 2 trust tests remain green.

- [ ] **Step 5: Commit trusted root creation**

```bash
git add src/trash.rs src/state_root.rs tests/trash_receipt.rs
git commit -m "feat: initialize trusted trash receipt store"
```

### Task 3: Crash-consistent trash execution

**Files:**
- Modify: `src/trash.rs`
- Modify: `src/state_root.rs`
- Modify: `src/fs_ops.rs:97-169`
- Modify: `src/mutation.rs`
- Modify: `src/mutation_ops.rs`
- Modify: `tests/trash_receipt.rs`
- Modify: `tests/fs_ops.rs:55-206`

- [ ] **Step 1: Write the failing protocol and legacy-removal matrix**

Add exact tests `trash_same_fs_claims_then_publishes_payload`, `trash_same_fs_nonempty_directory_publishes_whole_tree`, `trash_cross_fs_regular_retains_source_until_payload_committed`, `trash_cross_fs_symlink_never_follows_target`, `trash_cross_fs_directory_is_rejected_before_control_or_bundle_reservation`, `trash_prepared_reservation_waits_for_fence_and_cleans_on_abort`, `trash_prepared_abort_consumes_owner_verified_adjacent_absence_and_fixed_typestate`, `trash_adjacent_bundle_collision_preserves_foreign_and_regenerates_worker_item_id`, `trash_executor_panic_reclaims_exact_fixed_ref_and_reconciles`, `trash_source_swap_is_restored_or_retained`, `trash_uses_one_outer_control_with_nested_source_claim`, `trash_mirror_transition_orders_intent_adjacent_confirmation`, `trash_proof_tokens_cannot_be_fabricated_or_deserialized`, `trash_authorizing_facts_cannot_outlive_claim_or_locked_receipt_snapshot`, `trash_raw_receipt_cannot_forge_transition_proof`, `trash_genuine_token_rejects_cross_bundle_replay`, `trash_genuine_token_rejects_cross_revision_replay`, `trash_genuine_token_rejects_cross_edge_replay`, `trash_consumed_token_cannot_be_used_twice`, `trash_committed_terminal_preserves_bundle_while_consuming_fixed_control`, `legacy_trash_path_api_and_direct_rename_are_removed`, and `trash_fault_after_each_state_preserves_a_discoverable_copy`. Parameterize the last test over the frozen ordered 24-case matrix `trash_durable_edge_v1`: for each lowercase phase ID `prepared`, `payload_ready`, `payload_published`, and `source_removal_pending`, include `.before_fixed_intent`, `.after_fixed_intent`, `.before_adjacent_replace`, `.after_adjacent_replace`, `.before_fixed_confirmation`, and `.after_fixed_confirmation`, in that phase/edge order. Define those exact strings in `const TRASH_DURABLE_EDGE_CASES: [&str; 24]`, assert `len() == 24` before iterating, and emit every case ID for the runner; assert each restart has either the original source or a verified private payload and never reports `Completed` early. The directory rejection test injects different device IDs, snapshots state-root and trash-root directory entries before execution, and proves the non-empty source tree is byte-for-byte unchanged with no fixed control or staging bundle created. The preparation test proves background preflight create-new-reserves the sole control and staging bundle before final request freeze, emits a non-droppable `Prepared`, performs no source-visible operation before `FenceInstalled`, and either consumes both reservations after the acknowledgement or proves their removal and parent sync on cancellation/disconnect. The prepared-abort test reaches the owner only through Plan 2's central dispatcher and proves each closed fact branch is revalidated immediately before fixed unlink. The collision test precreates a same-name foreign staging child, captures its full identity/content, forces fixed reservation to succeed and adjacent create-new to return `EEXIST`, then proves the foreign child is never opened/renamed/deleted, the exact fixed control is removed and parent-synced through the foreign-unchanged abort fact, and only then does the worker regenerate an execution `ItemId`. The panic test injects unwind immediately after the reservation move and at every durable mirror edge, then proves recovery claims only the retained exact raw-name/identity reference. The proof tests source-check that proof fields/constructors and serde/clone impls are unavailable, feed forged raw receipts/JSON/bools/hashes, obtain one genuine token, and prove it cannot cross bundle/revision/edge or be used twice; only live locked adjacent verification plus the owning `VerifiedTerminalControl` typestate can authorize transitions/removal. The committed-terminal test proves trash ingestion releases its adjacent lock but leaves the synced `receipt.json`, payload, bundle name, and bundle identity discoverable for a later restore while consuming only the fixed control.

Also add exact `trash_transition_verify_failure_retains_locked_receipt_and_fixed_claim`, injecting decode/identity verification failure before any write and asserting the owning verify error retains both the actual receipt lock and fixed-control transaction.

Also add exact `trash_preparation_failure_enters_worker_dispatch_with_ownership`, proving the public diagnostic wrapper redacts capabilities while `ItemPreparationFailureInner::Trash` returns the exact draft/entry/reservation ownership to the worker.

Keep public end-to-end filesystem/worker behavior in `tests/trash_receipt.rs`. Put `trash_prepared_abort_consumes_owner_verified_adjacent_absence_and_fixed_typestate`, `trash_adjacent_bundle_collision_preserves_foreign_and_regenerates_worker_item_id`, `trash_executor_panic_reclaims_exact_fixed_ref_and_reconciles`, `trash_mirror_transition_orders_intent_adjacent_confirmation`, every proof/authorization gate from `trash_proof_tokens_cannot_be_fabricated_or_deserialized` through `trash_consumed_token_cannot_be_used_twice`, `trash_transition_verify_failure_retains_locked_receipt_and_fixed_claim`, `trash_preparation_failure_enters_worker_dispatch_with_ownership`, and `trash_fault_after_each_state_preserves_a_discoverable_copy` in `trash::tests`. Those tests use only `#[cfg(test)] pub(crate)` sibling fault hooks and private ownership; they run via `--lib` and do not widen a production item or expose the controller to integration, normal-library, or release builds.

- [ ] **Step 2: Run the protocol tests and confirm RED**

Run each target exactly:

```bash
python3 scripts/run_exact_test.py --test trash_receipt --name trash_same_fs_claims_then_publishes_payload
python3 scripts/run_exact_test.py --test trash_receipt --name trash_same_fs_nonempty_directory_publishes_whole_tree
python3 scripts/run_exact_test.py --test trash_receipt --name trash_cross_fs_regular_retains_source_until_payload_committed
python3 scripts/run_exact_test.py --test trash_receipt --name trash_cross_fs_symlink_never_follows_target
python3 scripts/run_exact_test.py --test trash_receipt --name trash_cross_fs_directory_is_rejected_before_control_or_bundle_reservation
python3 scripts/run_exact_test.py --test trash_receipt --name trash_prepared_reservation_waits_for_fence_and_cleans_on_abort
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_prepared_abort_consumes_owner_verified_adjacent_absence_and_fixed_typestate
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_adjacent_bundle_collision_preserves_foreign_and_regenerates_worker_item_id
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_executor_panic_reclaims_exact_fixed_ref_and_reconciles
python3 scripts/run_exact_test.py --test trash_receipt --name trash_source_swap_is_restored_or_retained
python3 scripts/run_exact_test.py --test trash_receipt --name trash_uses_one_outer_control_with_nested_source_claim
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_mirror_transition_orders_intent_adjacent_confirmation
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_proof_tokens_cannot_be_fabricated_or_deserialized
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_authorizing_facts_cannot_outlive_claim_or_locked_receipt_snapshot
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_raw_receipt_cannot_forge_transition_proof
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_genuine_token_rejects_cross_bundle_replay
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_genuine_token_rejects_cross_revision_replay
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_genuine_token_rejects_cross_edge_replay
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_consumed_token_cannot_be_used_twice
python3 scripts/run_exact_test.py --test trash_receipt --name trash_committed_terminal_preserves_bundle_while_consuming_fixed_control
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_transition_verify_failure_retains_locked_receipt_and_fixed_claim
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_preparation_failure_enters_worker_dispatch_with_ownership
python3 scripts/run_exact_test.py --test fs_ops --name legacy_trash_path_api_and_direct_rename_are_removed
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_fault_after_each_state_preserves_a_discoverable_copy \
  --case-matrix trash_durable_edge_v1 \
  --expect-case prepared.before_fixed_intent --expect-case prepared.after_fixed_intent \
  --expect-case prepared.before_adjacent_replace --expect-case prepared.after_adjacent_replace \
  --expect-case prepared.before_fixed_confirmation --expect-case prepared.after_fixed_confirmation \
  --expect-case payload_ready.before_fixed_intent --expect-case payload_ready.after_fixed_intent \
  --expect-case payload_ready.before_adjacent_replace --expect-case payload_ready.after_adjacent_replace \
  --expect-case payload_ready.before_fixed_confirmation --expect-case payload_ready.after_fixed_confirmation \
  --expect-case payload_published.before_fixed_intent --expect-case payload_published.after_fixed_intent \
  --expect-case payload_published.before_adjacent_replace --expect-case payload_published.after_adjacent_replace \
  --expect-case payload_published.before_fixed_confirmation --expect-case payload_published.after_fixed_confirmation \
  --expect-case source_removal_pending.before_fixed_intent --expect-case source_removal_pending.after_fixed_intent \
  --expect-case source_removal_pending.before_adjacent_replace --expect-case source_removal_pending.after_adjacent_replace \
  --expect-case source_removal_pending.before_fixed_confirmation --expect-case source_removal_pending.after_fixed_confirmation
cargo test --locked --test trash_receipt --test fs_ops
```

Expected: FAIL because `prepare_trash`/`execute_trash` are absent and `fs_ops::trash_path` still performs a direct rename.

- [ ] **Step 3: Implement the trash state machine**

Add:

```rust
#[doc(hidden)]
pub struct PreparedTrashReservation {
    transaction: ClaimedTrashTransaction,
}

pub(crate) enum TrashPreparationOwnership {
    Draft(ItemDraft),
    FixedReservationOwned {
        draft: ItemDraft,
        failure: ControlReservationFailure,
    },
    Entry {
        draft: ItemDraft,
        failure: TrashMutationEntryFailure,
    },
    Prepared {
        draft: ItemDraft,
        reservation: PreparedTrashReservation,
    },
}

pub(crate) struct TrashPreparationFailure {
    ownership: TrashPreparationOwnership,
    source: PreparationError,
}
pub(crate) enum PrepareTrashError {
    Invalid(TrashPreparationFailure),
    ControlNameCollision {
        candidate_item_id: ItemId,
        draft: ItemDraft,
        collided_item_id: ItemId,
        source: PreparationError,
    },
    AdjacentBundleCollision {
        candidate_item_id: ItemId,
        draft: ItemDraft,
        collision: ClaimedTrashForeignCollision,
        source: PreparationError,
    },
}

pub(crate) struct TrashTransactionRemainder {
    control: ClaimedControlBundle,
    store: TrashStore,
    bundle_name: RawUnixName,
    bundle_location: BundleLocation,
    bundle_dir: Arc<TrustedDir>,
    bundle_identity: PathIdentity,
}

pub(crate) struct TrashReceiptProof {
    transaction: TrashTransactionRemainder,
    locked_receipt: OwnedLockedReceipt<TrashReceipt>,
    authorization: TrashTransitionAuthorization,
}
pub(crate) struct TrashTransitionAuthorization {
    installation_id: InstallationId,
    operation_id: OperationId,
    item_id: ItemId,
    bundle_identity: PathIdentity,
    current_revision: u64,
    next_revision: u64,
    next_canonical_sha256: [u8; 32],
    exact_edge: TrashTransitionEdge,
    verified_expectation: TrashTransitionExpectation,
}
pub(crate) struct TrashAdvanceError {
    transaction: TrashTransactionRemainder,
    locked_failure: OwnedLockedReceiptAdvanceError<TrashReceipt>,
}
pub(crate) struct TrashTransitionVerifyFailure {
    transaction: TrashTransactionRemainder,
    locked_failure: OwnedLockedReceiptVerifyError,
}
pub(crate) enum TrashTransitionEdge {
    PreparedToPayloadReady,
    PayloadReadyToPayloadPublished,
    PayloadPublishedToSourceRemovalPending,
    SourceRemovalPendingToCommitted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrashFixedMirrorEdge {
    PreparedToPayloadReady,
    PayloadReadyToPayloadPublished,
    PayloadPublishedToSourceRemovalPending,
    SourceRemovalPendingToCommitted,
}
struct TrashPreparedAbortParentSyncWitness;
struct TrashPreparedAbortAbsenceWitness;
struct TrashPreparedAbortForeignUnchangedWitness;
enum TrashPreparedAbortFactInner {
    OwnedRemovedAndSynced {
        trusted_location_parent: TrustedDir,
        bundle_name: RawUnixName,
        removed_bundle_identity: PathIdentityKey,
        parent_identity: PathIdentityKey,
        parent_sync: TrashPreparedAbortParentSyncWitness,
    },
    FixedOnlyAdjacentAbsent {
        trusted_location_parent: TrustedDir,
        bundle_name: RawUnixName,
        parent_identity: PathIdentityKey,
        absence: TrashPreparedAbortAbsenceWitness,
    },
    ObservedForeignUnchanged {
        trusted_location_parent: Arc<TrustedDir>,
        bundle_name: RawUnixName,
        observed_foreign_identity: PathIdentity,
        unchanged: TrashPreparedAbortForeignUnchangedWitness,
    },
}
#[doc(hidden)]
pub struct TrashPreparedAbortFacts {
    inner: TrashPreparedAbortFactInner,
}
pub(crate) enum TrashTransitionExpectation {
    PayloadReady { expected_payload: PathIdentity },
    BundlePublished { expected_bundle: PathIdentity },
    SourceRemovalPending { expected_payload: PathIdentity },
    Committed { expected_payload: PathIdentity, source_absent: RawUnixPath },
}

pub(crate) fn validate_trash_fixed_mirror_intent_edge(
    current: &TrashControlState,
    next: &TrashControlState,
    edge: TrashFixedMirrorEdge,
    current_adjacent: &AdjacentReceiptFacts<'_>,
    next_intent: &MirrorIntent,
) -> Result<(), TrustedFsError>;

pub(crate) fn validate_trash_prepared_abort(
    current: &TrashControlState,
    facts: &TrashPreparedAbortFacts,
) -> Result<(), TrustedFsError>;

struct TrashTerminalSyncWitness;
#[doc(hidden)]
pub struct TrashTerminalExpectation {
    trusted_location_parent: TrustedDir,
    bundle_name: RawUnixName,
    expected_bundle: VerifiedBundleFact,
    trusted_payload_parent: TrustedDir,
    payload_name: RawUnixName,
    expected_payload: PathIdentity,
    trusted_original_parent: TrustedDir,
    original_name: RawUnixName,
    affected_parents_synced: TrashTerminalSyncWitness,
}

pub(crate) fn validate_trash_terminal(
    current: &TrashControlState,
    facts: &TrashTerminalExpectation,
) -> Result<(), TrustedFsError>;

pub(crate) fn prepare_trash(
    context: &MutationContext,
    operation_id: OperationId,
    candidate_item_id: ItemId,
    draft: ItemDraft,
) -> Result<(ItemPlan, PreparedTrashReservation, MutationFenceSpec), PrepareTrashError>;

pub(crate) fn execute_trash(
    context: &MutationContext,
    request: &OperationRequest,
    plan: &ItemPlan,
    reservation: PreparedTrashReservation,
) -> ItemOutcome;

impl PreparedTrashReservation {
    pub(crate) fn recovery_ref(&self) -> ControlRecoveryRef;
}

impl TrashPreparationFailure {
    pub(crate) fn source(&self) -> &PreparationError;
    pub(crate) fn into_parts(self) -> (TrashPreparationOwnership, PreparationError);
}

impl PrepareTrashError {
    pub(crate) fn diagnostic(&self) -> (PreparationFailureKind, &str);
}

impl TrashAdvanceError {
    pub(crate) fn into_recovery_discarding_proof(
        self,
    ) -> (
        ClaimedTrashTransaction,
        ReceiptAdvanceStage,
        ReceiptAfterFailure<TrashReceipt>,
        TrustedFsError,
    );
}

impl TrashTransitionVerifyFailure {
    pub(crate) fn into_recovery(
        self,
    ) -> (ClaimedTrashTransaction, ReceiptAfterFailureBytes, TrustedFsError);
}

pub(crate) fn abort_prepared_trash(
    context: &MutationContext,
    plan: &ItemPlan,
    prepared: PreparedTrashReservation,
) -> PreparedAbortOutcome;

pub(crate) fn recover_trash_preparation_failure(
    context: &MutationContext,
    failure: PrepareTrashError,
) -> PreparedAbortOutcome;

pub(crate) fn recover_trash_execution_panic(
    context: &MutationContext,
    plan: &ItemPlan,
    claimed: ClaimedControlBundle,
) -> ItemOutcome;

pub(crate) fn recover_trash_pending_control(
    context: &MutationContext,
    claimed: ClaimedControlBundle,
) -> StartupRecoveryNotice;

impl ClaimedTrashTransaction {
    pub(crate) fn verify_transition(
        self,
        next: &TrashReceipt,
        expectation: &TrashTransitionExpectation,
    ) -> Result<TrashReceiptProof, TrashTransitionVerifyFailure>;
    pub(crate) fn confirm_pending_mirror(&mut self) -> Result<()>;
    pub(crate) fn into_fixed_terminal_preserving_committed_bundle(
        self,
        expectation: &TrashTerminalExpectation,
    ) -> Result<ClaimedControlBundle, TrashTransitionVerifyFailure>;
}

impl TrashReceiptProof {
    pub(crate) fn advance_receipt(
        self,
        expected_revision: u64,
        next: &TrashReceipt,
    ) -> Result<ClaimedTrashTransaction, TrashAdvanceError>;
}

impl DurableReceipt for TrashReceipt {
    type Proof = TrashTransitionAuthorization;
    fn revision(&self) -> u64;
    fn validate_next(
        &self,
        next: &Self,
        proof: &TrashTransitionAuthorization,
    ) -> Result<(), TrustedFsError>;
}
```

In the existing Plan 2 mutation worker's background preparation phase, reject cross-filesystem directories and unsupported objects with `FailedNoEffect` before allocating a durable ID or reservation; first-release cross-filesystem trash supports only regular files and symlinks. Same-filesystem trash supports regular files, symlinks, empty directories, and non-empty directories because the whole source object is atomically claimed and renamed without traversal or recursive deletion.

Extend Plan 2's initially empty `HostMirrorEdge` in this task only with `Trash(TrashFixedMirrorEdge)`. This source edit also removes Plan 2's temporary `#[expect(dead_code)]` from `verify_host_mirror_intent`, because the Trash post-initial edge is its first production caller. `state_root.rs` does not inspect the private host fields itself: its fixed-intent match passes current/next Trash host references, the concrete edge, live current-adjacent facts, and next intent to `validate_trash_fixed_mirror_intent_edge`; its confirmation match passes current/next host, expected intent, and newly verified adjacent facts to the Task 2 `validate_trash_mirror_confirmation`. Only those hard-coded owner-module successes let state root mint/accept the corresponding opaque proofs; caller-provided booleans or a generic trait implementation are not accepted. Every post-initial Trash transition first obtains the fixed-intent proof and consumes `ControlTransitionProof::MirrorIntentInstalled(proof)` in the fixed `advance`, then advances adjacent, then calls `confirm_mirror(adjacent_facts, &owner_approved_next_state)` and consumes the separately bound confirmation proof. Task 2 already implemented the create-new-envelope exception and initial confirmation. Task 5 adds the Restore edge, validators, and terminal-removal methods only after their concrete types exist.

Also extend the concrete public `PreparedControlAbortExpectation` with `Trash(TrashPreparedAbortFacts)`. The fact wrapper is `#[doc(hidden)] pub` only so the public enum does not violate `private_interfaces`; its sole field wraps the closed private `TrashPreparedAbortFactInner`, every field/constructor remains private/crate-private, and the wrapper is non-Clone/non-serde. Only `trash.rs` can mint one of three exact facts: `OwnedRemovedAndSynced` after removing its verified initial adjacent reservation and syncing the trusted location parent; `FixedOnlyAdjacentAbsent` when no adjacent create succeeded and the persisted raw name is freshly absent; or `ObservedForeignUnchanged` after create-new returned `EEXIST`, retaining the same trusted parent/raw name/full foreign identity without ever opening or mutating the child. `state_root.rs` delegates both initial verification and the immediately-before-fixed-unlink revalidation to `validate_trash_prepared_abort`. The owner validator binds each branch to the initial Trash host and exact `bundle_intent`: the owned branch requires the confirmed initial mirror plus matching removed identity and re-proves absence; the fixed-only branch requires no confirmed adjacent identity and re-proves absence; the collision branch requires no confirmed adjacent identity and re-captures `same_snapshot` for that exact foreign child. A hash, bool, displayed path, stale absence, or an EEXIST code without the live parent/name/identity cannot authorize fixed removal.

`ControlState::Trash` and its initial mirror validator already exist from Task 2. In this task extend `TerminalExpectation` with `Trash(TrashTerminalExpectation)` and extend crate-private `ItemPreparationFailureInner` only with `Trash(PrepareTrashError)` rather than flattening it to `PreparationError`; Task 5 adds `Restore(PrepareRestoreError)` only after that type exists. `TrashTerminalExpectation` is an owner-minted, non-Clone/non-serde fact wrapper with private fields retaining the trusted adjacent/payload/original parents, exact raw names/identities, and sync witness. `state_root.rs` delegates both terminal verification and the immediate-before-fixed-unlink revalidation to `validate_trash_terminal`; the serialized committed receipt or a caller-provided path/hash cannot substitute. For each accepted item, background preparation generates the final IDs and complete raw root/location/bundle-name selectors, canonicalizes the full initial receipt, and constructs `ControlEnvelope { state: ControlState::Trash { host: TrashControlState { root_intent, bundle_intent: TrashBundleIntent { location: BundleLocation::Staging, location_parent_name, expected_location_parent_identity, raw_name, expected_bundle_identity: None, expected_receipt_identity: None, expected_claim_lock_identity: None }, mirror: ReceiptMirror { bundle: None, confirmed_revision: None, confirmed_receipt_sha256: None, pending_phase: Some(TrashState::Prepared) }, .. }, source_claim: None, mirror_intent: Some(MirrorIntent { adjacent_revision: initial.revision, canonical_receipt_sha256: initial_sha256 }) }, .. }` with `ControlProtocol::TrashIngestV1`. It create-new-reserves/syncs that fixed intent through `StateRoot::reserve_control` before any trash-root initialization, moves the claim into `open_or_initialize_for_mutation`, and calls `create_initial_bundle_matching_intent` to create/sync exactly that persisted raw name and those adjacent bytes and confirm the mirror. The `reserve_control` result is matched exhaustively and immediately: `Collision(collided_item_id)` becomes `PrepareTrashError::ControlNameCollision` without opening the colliding fixed bundle, `NoEffect(error)` becomes `Invalid` with `TrashPreparationOwnership::Draft`, and `Owned(failure)` becomes `Invalid` with `FixedReservationOwned`; the last branch retains the complete `ControlReservationFailure` until owner recovery consumes `abort`, and a `ControlReservationAbortFailure` remains owning and is consumed through `retry`. A later adjacent create-new `EEXIST` is a different `AdjacentBundleCollision` carrying `ClaimedTrashForeignCollision`; neither collision kind may be collapsed into the other. Confirmation fills the optional root/location and all three adjacent identities in the fixed host and installs the full `TrashBundleSelector`; later transitions may move the bundle only by owner-validated location-name/full-identity replacement while preserving the same immutable work/trash/v1 root chain. There is no proofless initial fixed advance. The resulting single owning transaction enters private `PreparedReservation::Trash(PreparedTrashReservation)` before the final `OperationRequest`/`ItemPlan` become immutable. Ordinary errors use `PrepareTrashError::Invalid(TrashPreparationFailure)` and retain the draft plus exact owning fixed/entry/reservation state. Both collision diagnostics use `PreparationFailureKind::ReservationCollision`, and the worker retains the typed draft/candidate journal while passing the whole owning error through the central recovery dispatcher. Reservation capabilities never enter `PreparedNotice` or `App`; the worker retains them privately while the non-droppable notice coordinates `FenceInstalled`. Only then does `mutation_ops` call crate-private `execute_trash`, which consumes rather than recreates them. Cancellation, panic, or disconnect before that acknowledgement must prove removal and parent sync of both reservations or report the discoverable residue as `CleanupRequired`/`Indeterminate`.

The worker never destructures those private ownership states. `mutation_ops::recover_preparation_failure` passes `ItemPreparationFailureInner::Trash` whole to `recover_trash_preparation_failure`, and `abort_prepared_item` passes `PreparedReservation::Trash` whole to `abort_prepared_trash`. Both routines live in `trash.rs` and consume the exact transaction/error. `FixedReservationOwned` calls `ControlReservationFailure::abort` without detaching its stage/truth; `ControlReservationAbortFailure::retry` is the only continuation after an abort error. Complete preparation removes only the verified internal staging receipt/lock/bundle, syncs the location parent, and uses `OwnedRemovedAndSynced`. A fixed-only cancellation uses `FixedOnlyAdjacentAbsent`. `ControlNameCollision` has no fixed or adjacent capability and may regenerate only because `ReserveControlError::Collision` proves the attempted candidate had no effect. `AdjacentBundleCollision` consumes `ClaimedTrashForeignCollision`, revalidates the retained parent's identity and the exact foreign child's `same_snapshot`, and uses `ObservedForeignUnchanged`; it never removes or renames that child. Each post-fixed branch consumes the still-original fixed claim through `verify_prepared_abort(PreparedControlAbortExpectation::Trash(facts))` and then consumes `VerifiedPreparedControlAbort::remove`; failure retains the owning typestate and is retried/classified without reopening. They return `ReleasedNoEffect` only after zero user payload effect, the exact applicable adjacent fact, and fixed removal parent sync are proved. Only a pre-effect control-name collision or a post-fixed adjacent collision that reaches `ReleasedNoEffect` lets the worker use its retained draft, discard the collided candidate, generate a fresh execution `ItemId`, and call `prepare_trash` again. Any cleanup uncertainty leaves the fixed residue discoverable and returns `CleanupRequired`/`Indeterminate`; it never retries the ID, touches the sentinel, or reopens by `ItemId`. The existing collision test executes both distinct branches and asserts that neither opens the colliding fixed bundle nor touches the adjacent sentinel; the remaining preparation/pre-ack tests invoke these central Plan 2 dispatchers rather than reach through private fields.

`PreparedReservation::recovery_seed` maps `Trash` to `PreparedRecoverySeed::Fixed(prepared.recovery_ref())` before execution moves the opaque reservation. After an unwind, the Plan 2 closed `(plan.kind, ControlState)` dispatcher passes the newly reclaimed exact fixed control whole to `recover_trash_execution_panic`. The panic wrapper verifies that the retained immutable plan kind/IDs match the claimed header, then transfers the control to one private `reconcile_trash_claimed_control(context, claimed)` core. The startup wrapper `recover_trash_pending_control(context, claimed)` transfers only the already claimed control to that same core; it accepts no `ItemPlan`, `ItemId`, work root, cwd, or display path argument. The core reads the complete `TrashRootIntent`/`TrashBundleSelector` from the claimed state, no-follow reopens and fully revalidates every recorded parent/name/identity, acquires fixed-before-adjacent, and reconciles the current mirror/source-claim state. If the nested `SourceClaimState` is pending, it calls the locked Plan 2 `SourceClaim::reconcile_pending(...) -> Result<ClaimResult<'_>, SourceClaimAcquireFailure<'_>>` and exhaustively consumes both layers. `Ok(result)` is matched by value: terminal `Published`/`Deleted`/`RestoredNoEffect` may advance the host, while `RestoreRequired(recovery)`, `CleanupRequired(recovery)`, and `Indeterminate(recovery)` are each consumed through `SourceClaimRecovery::reconcile`; any returned owning recovery is fed back through the same consuming match, never converted into a notice. `Err(SourceClaimAcquireFailure::NoAdjacentEffect(error))` ends the borrow while the core still owns `claimed` and can classify no-effect; `Err(SourceClaimAcquireFailure::AdjacentOwned { recovery, .. })` consumes that recovery directly through `recovery.reconcile()` and exhaustively matches its owning `ClaimResult`. The host does not flatten an adjacent-owned failure through the observation-only `ClaimError` branch of `reconcile_owned`. No panic/startup wrapper returns an `ItemOutcome`/`StartupRecoveryNotice` while a `SourceClaimRecovery` is still live; a persistently unresolved owning recovery keeps that owner reconciliation and startup availability pending. No arm converts an owning result to `ClaimObservation`, drops a recovery, or reopens by ID. Once ownership is fully consumed into fixed durable truth, the panic wrapper maps the final decision to `ItemOutcome` and the startup wrapper maps it to Plan 2's bounded `StartupRecoveryNotice`. Missing, contradictory, or unreadable non-owning truth is `StartupRecoveryDisposition::Indeterminate`; neither wrapper reconstructs a transaction from a catalog `ItemId` or displayed path.

Plan 2 already defines public `WorkerObservation::StartupRecovery(StartupRecoveryNotice)` and the non-authorizing optional-ID/protocol notice shape. Extend only its closed `mutation_ops::recover_startup_control(context, claimed) -> StartupRecoveryNotice` match so `ControlState::Trash` is consumed by `recover_trash_pending_control`; do not redeclare the notice, disposition, worker enum, or dispatcher signature. The existing worker sends the returned notice through the non-progress channel before scanning the next pending control. A full channel may delay but never drop or reorder this handoff; the worker remains unavailable for new commands until every verified pending control is terminally classified. `PendingControl::InspectOnly` remains the Plan 2 inspect-only path without an owning call. Task 5 adds the `ControlState::Restore` arm using the same handoff, so no synthetic `ItemPlan` is ever fabricated at startup.

Keep the `ClaimedTrashTransaction` through the item transition. Execution no-follow reopens the source parent from the immutable plan and verifies its prepared full identity before lending `&trusted_source_parent` and the scoped control borrow. Same-filesystem trash calls `SourceClaim::acquire(context.state_root.as_ref(), &trusted_source_parent, control, plan, ClaimAction::TrashPublish { .. })` so the source is claimed and the complete object is published into the reserved trash payload no-clobber. Cross-filesystem trash bounded-copies a regular file or raw symlink target, verifies metadata and syncs before `PayloadReady`, publishes the complete bundle no-clobber into `items`, writes `PayloadPublished` and `SourceRemovalPending`, re-verifies the published payload, then calls the same signature with `ClaimAction::ExdevSourceCleanup` and deletes only the verified private tombstone. Write `Committed` only after the required source/payload facts are proved.

The bundle `receipt.json` is the user-facing local payload record; the claimed fixed control is the only transition authority and cross-cwd ownership record. Implement `DurableReceipt<Proof = TrashTransitionAuthorization>` for `TrashReceipt`; the private authorization contains exact bound facts but never contains or aliases the owning receipt. `ClaimedTrashTransaction::verify_transition(self, ...)` consumes the transaction, splits its actual `AtomicReceiptFile + ClaimedChildLock` into Plan 2's `OwnedLockedReceipt<TrashReceipt>`, verifies the current bytes and named disk facts, and returns outer `TrashReceiptProof { transaction, locked_receipt, authorization }`. `TrashReceiptProof::advance_receipt(self, ...)` destructures itself exactly once and passes only `locked_receipt + authorization` to the substrate. Success reconstructs `ClaimedTrashTransaction`; failure returns `TrashAdvanceError { transaction, locked_failure }`, where `locked_failure` retains the same actual lock, receipt, authorization, failure stage, and re-read disk truth. Pre-write validation failure proves the original revision; post-write/sync failure is `Advanced` or `Unreadable` and reconciles as durable truth/`Indeterminate`. The outer proof is non-Clone/non-serde and non-circular; no early lock drop, second use, or error-path ownership loss is expressible. The serialized host stores `VerifiedBundleFact { item_id, location, selector: TrashBundleSelector }`, never a serialized `VerifiedBundleRef`; restart recovery opens the recorded absolute work root, verifies its full identity, then reopens the stored raw trash-parent/v1/location/bundle/receipt/lock child names fd-relatively in order and checks every full identity before minting a fresh private capability. It never derives a name or root from `ItemId`, cwd, a displayed path, or an absolute child path. Every mirrored transition uses exactly: (1) canonicalize the next adjacent receipt and compute SHA-256; (2) advance the fixed `ControlState::Trash` so `host.pending_phase` names the next phase and the standard `mirror_intent` is `MirrorIntent { adjacent_revision, canonical_receipt_sha256 }`; (3) verify and consume the owning trash proof to advance/sync those exact adjacent bytes; (4) `confirm_pending_mirror` obtains lifetime-bound `AdjacentReceiptFacts<'lock>` from the locked receipt, clones the verifier-refreshed full bundle snapshot returned by `AdjacentReceiptFacts::bundle_identity`, constructs the exact owner-approved next state with that snapshot in `TrashBundleSelector`, and passes the facts plus next state to `control.confirm_mirror`; (5) after consuming `ControlTransitionProof::MirrorConfirmed`, it writes that same refreshed snapshot into the live `TrashBundle.identity`/`TrashTransactionRemainder.bundle_identity` before the lock can end. No later transition or cleanup reuses a directory snapshot captured before a receipt/lock child mutation. Terminal removal consumes the transaction's original fixed claim into `VerifiedTerminalControl` and then consumes `VerifiedTerminalControl::remove`; it never acquires a second lock or detaches terminal facts. The existing mirror-order exact test asserts the in-memory transaction, fixed selector, and adjacent verifier all carry the identical post-sync full snapshot after every edge. An adjacent receipt ahead without a matching fixed intent, any revision/hash disagreement, a missing fixed control, or a different bundle identity is inspect-only `Indeterminate`/`Contradictory` and never authorizes source or payload cleanup. A crash with matching intent plus adjacent bytes may only finish fixed confirmation after re-verifying the named disk fact. The standardized nested `SourceClaimState` in the same `ControlState::Trash` is the only source-claim receipt. `SourceClaim` must not call `StateRoot::reserve_control`.

The cross-module error seams do not expose substrate proof objects. `TrashAdvanceError::into_recovery_discarding_proof` consumes Plan 2's owning error, destroys the failed authorization/stale snapshot, reinserts the returned receipt plus actual lock into its `TrashTransactionRemainder`, and returns one reconstructed `ClaimedTrashTransaction` with stage and reread truth. `TrashTransitionVerifyFailure::into_recovery` similarly reconstructs the transaction from the retained receipt/lock and returns only observed bytes plus diagnostic. `recovery.rs` must use these consuming methods on every `Err`; it cannot destructure private fields, drop authority, extract `TrashTransitionAuthorization`, or call the substrate advance directly.

Source removal is forbidden until `ReceiptMirror.confirmed_revision` and `confirmed_receipt_sha256` equal the re-read adjacent `PayloadPublished`/`SourceRemovalPending` receipt. After the final `Committed` adjacent receipt and disk facts are mirror-confirmed, call `ClaimedTrashTransaction::into_fixed_terminal_preserving_committed_bundle`: under the actual receipt lock it re-verifies the committed receipt, synced payload, bundle name/identity, and source absence, then releases only `claim.lock` without unlinking `receipt.json`, the payload, or the bundle. It returns the still-owning original fixed claim, which is consumed through `verify_terminal(Trash(...))` and then `VerifiedTerminalControl::remove`. A failed verification returns the same fixed claim and a failed fixed removal remains owning/discoverable cleanup truth rather than creating another control. Trash ingestion must never use the restore-only bundle-removal path because the committed bundle is the user-visible recovery asset.

If `into_verified_owned_locked` cannot verify/decode the current receipt, `TrashTransitionVerifyFailure` returns `TrashTransactionRemainder + OwnedLockedReceiptVerifyError`; the actual receipt and `claim.lock` remain owned and the fixed claim is never reopened by ID. Map any unproved post-effect fact to `CleanupRequired` or `Indeterminate`. Delete `fs_ops::trash_path`, `prepare_trash_dir`, every direct-rename trash implementation, and all callers; route production trash only through `mutation_ops` background `prepare_trash`, `Prepared`/`FenceInstalled`, then `execute_trash`. Do not retain a compatibility wrapper lacking `StateRoot`, `MutationContext`, or the claimed typed control. The source-inspection regression reads `src/fs_ops.rs` and `src/mutation_ops.rs`, rejects definitions/calls of `trash_path` or `prepare_trash_dir`, and requires the two prepared free-function dispatches.

- [ ] **Step 4: Run the trash protocol and existing filesystem tests**

Run:

```bash
python3 scripts/run_exact_test.py --test trash_receipt --name trash_same_fs_claims_then_publishes_payload
python3 scripts/run_exact_test.py --test trash_receipt --name trash_same_fs_nonempty_directory_publishes_whole_tree
python3 scripts/run_exact_test.py --test trash_receipt --name trash_cross_fs_regular_retains_source_until_payload_committed
python3 scripts/run_exact_test.py --test trash_receipt --name trash_cross_fs_symlink_never_follows_target
python3 scripts/run_exact_test.py --test trash_receipt --name trash_cross_fs_directory_is_rejected_before_control_or_bundle_reservation
python3 scripts/run_exact_test.py --test trash_receipt --name trash_prepared_reservation_waits_for_fence_and_cleans_on_abort
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_prepared_abort_consumes_owner_verified_adjacent_absence_and_fixed_typestate
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_adjacent_bundle_collision_preserves_foreign_and_regenerates_worker_item_id
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_executor_panic_reclaims_exact_fixed_ref_and_reconciles
python3 scripts/run_exact_test.py --test trash_receipt --name trash_source_swap_is_restored_or_retained
python3 scripts/run_exact_test.py --test trash_receipt --name trash_uses_one_outer_control_with_nested_source_claim
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_mirror_transition_orders_intent_adjacent_confirmation
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_proof_tokens_cannot_be_fabricated_or_deserialized
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_authorizing_facts_cannot_outlive_claim_or_locked_receipt_snapshot
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_raw_receipt_cannot_forge_transition_proof
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_genuine_token_rejects_cross_bundle_replay
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_genuine_token_rejects_cross_revision_replay
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_genuine_token_rejects_cross_edge_replay
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_consumed_token_cannot_be_used_twice
python3 scripts/run_exact_test.py --test trash_receipt --name trash_committed_terminal_preserves_bundle_while_consuming_fixed_control
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_transition_verify_failure_retains_locked_receipt_and_fixed_claim
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_preparation_failure_enters_worker_dispatch_with_ownership
python3 scripts/run_exact_test.py --test fs_ops --name legacy_trash_path_api_and_direct_rename_are_removed
python3 scripts/run_exact_test.py --lib --name trash::tests::trash_fault_after_each_state_preserves_a_discoverable_copy \
  --case-matrix trash_durable_edge_v1 \
  --expect-case prepared.before_fixed_intent --expect-case prepared.after_fixed_intent \
  --expect-case prepared.before_adjacent_replace --expect-case prepared.after_adjacent_replace \
  --expect-case prepared.before_fixed_confirmation --expect-case prepared.after_fixed_confirmation \
  --expect-case payload_ready.before_fixed_intent --expect-case payload_ready.after_fixed_intent \
  --expect-case payload_ready.before_adjacent_replace --expect-case payload_ready.after_adjacent_replace \
  --expect-case payload_ready.before_fixed_confirmation --expect-case payload_ready.after_fixed_confirmation \
  --expect-case payload_published.before_fixed_intent --expect-case payload_published.after_fixed_intent \
  --expect-case payload_published.before_adjacent_replace --expect-case payload_published.after_adjacent_replace \
  --expect-case payload_published.before_fixed_confirmation --expect-case payload_published.after_fixed_confirmation \
  --expect-case source_removal_pending.before_fixed_intent --expect-case source_removal_pending.after_fixed_intent \
  --expect-case source_removal_pending.before_adjacent_replace --expect-case source_removal_pending.after_adjacent_replace \
  --expect-case source_removal_pending.before_fixed_confirmation --expect-case source_removal_pending.after_fixed_confirmation
cargo test --locked --test trash_receipt --test fs_ops
if rg -n '\b(trash_path|prepare_trash_dir)\b' src; then exit 1; fi
```

Expected: PASS; protocol tests pass, same-filesystem non-empty directories publish as one complete payload, unsupported cross-filesystem directories have zero receipt/source effect, the source scan prints nothing, and all remaining filesystem tests pass.

- [ ] **Step 5: Commit trash execution**

```bash
git add src/trash.rs src/state_root.rs src/fs_ops.rs src/mutation.rs src/mutation_ops.rs tests/trash_receipt.rs tests/fs_ops.rs
git commit -m "feat: make trash operations crash consistent"
```

### Task 4: Conservative reconciliation and quarantine

**Files:**
- Create: `src/recovery.rs`
- Modify: `src/lib.rs`
- Modify: `src/trash.rs`
- Modify: `tests/trash_receipt.rs`

- [ ] **Step 1: Write failing reconciliation race tests**

Add exact tests `reconcile_finishes_only_proven_monotonic_transition`, `reconcile_corrupt_receipt_quarantines_without_deletion`, `reconcile_unknown_payload_never_purges`, `reconcile_loser_reports_in_use_without_mutation`, `reconcile_discovers_fixed_control_from_different_cwd`, `reconcile_corrupt_pending_control_is_inspect_only_and_does_not_hide_verified_controls`, `catalog_corrupt_sibling_does_not_hide_verified_items`, `catalog_unknown_bundle_has_no_fabricated_item_id`, `catalog_legacy_content_is_inspect_only_unmanaged`, `catalog_10000_items_is_deterministic_and_paged`, `catalog_limit_one_has_no_gap_or_repeat_across_all_locations`, `catalog_duplicate_item_id_is_contradictory_in_every_location_and_not_restorable`, `catalog_duplicate_raw_name_is_ordered_by_location`, `catalog_restart_preserves_total_order_and_cursor`, `catalog_same_name_location_replacement_has_distinct_cursor_key`, `catalog_midstream_enumeration_error_never_returns_false_unique_or_restorable`, `claim_existing_rejects_identity_swap_after_observation`, and `claim_existing_two_process_race_has_one_mutating_winner`. Spawn two real processes for the claim race; both must use the same state root and trash root.

The `limit=1` fixture places consecutive keys in `staging`, `items`, `claims`, and `quarantine`, traverses every returned cursor until `next=None`, and asserts the concatenated full `CatalogKey`s equal one independently sorted expected vector with no gap or duplicate. The duplicate-ID fixture places the same valid 32-hex name in all four locations with different directory identities and otherwise valid receipts; all four records must be `InspectOnly { state: Contradictory }`, `resolve_unique_item` must fail ambiguous, and no bundle may move. The duplicate-raw-name fixture uses the same non-ItemId non-UTF-8 name in multiple locations, exposes it only as `CatalogOpaqueName`, proves no `RawUnixName` conversion/accessor exists, and proves location breaks the tie. The restart test destroys/reopens `TrashStore` between every page and requires identical ordering. The replacement test swaps the object under the same validated name/location between pages and proves `CatalogObservedIdentity::Captured(PathIdentityKey)` makes the new key distinct and a stale observed selector non-restorable. The midstream-error fixture injects name-less enumeration failure before a hidden duplicate; page/unique lookup must return an incomplete-scan error and never a verified/restorable row.

- [ ] **Step 2: Run reconciliation tests and confirm RED**

Run each target exactly:

```bash
python3 scripts/run_exact_test.py --test trash_receipt --name reconcile_finishes_only_proven_monotonic_transition
python3 scripts/run_exact_test.py --test trash_receipt --name reconcile_corrupt_receipt_quarantines_without_deletion
python3 scripts/run_exact_test.py --test trash_receipt --name reconcile_unknown_payload_never_purges
python3 scripts/run_exact_test.py --test trash_receipt --name reconcile_loser_reports_in_use_without_mutation --serial
python3 scripts/run_exact_test.py --test trash_receipt --name reconcile_discovers_fixed_control_from_different_cwd
python3 scripts/run_exact_test.py --test trash_receipt --name reconcile_corrupt_pending_control_is_inspect_only_and_does_not_hide_verified_controls
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_corrupt_sibling_does_not_hide_verified_items
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_unknown_bundle_has_no_fabricated_item_id
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_legacy_content_is_inspect_only_unmanaged
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_10000_items_is_deterministic_and_paged
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_limit_one_has_no_gap_or_repeat_across_all_locations
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_duplicate_item_id_is_contradictory_in_every_location_and_not_restorable
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_duplicate_raw_name_is_ordered_by_location
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_restart_preserves_total_order_and_cursor
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_same_name_location_replacement_has_distinct_cursor_key
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_midstream_enumeration_error_never_returns_false_unique_or_restorable
python3 scripts/run_exact_test.py --test trash_receipt --name claim_existing_rejects_identity_swap_after_observation
python3 scripts/run_exact_test.py --test trash_receipt --name claim_existing_two_process_race_has_one_mutating_winner --serial
cargo test --locked --test trash_receipt
```

Expected: FAIL because `RecoveryService::reconcile` and recovery classifications do not exist.

- [ ] **Step 3: Implement observation, claim, and reconciliation**

Define:

```rust
pub enum RecoveryState { Recoverable, NeedsCleanup, Incomplete, Orphaned, Quarantined, Contradictory, InUse, LegacyUnmanaged }
pub type RecoveryCatalogCursor = CatalogKey;
pub enum RecoveryRecord {
    Verified { bundle_ref: VerifiedBundleRef, receipt: TrashReceipt, state: RecoveryState },
    InspectOnly { inspect_ref: InspectOnlyRef, state: RecoveryState, escaped_error: String },
}
pub struct RecoveryPage {
    pub records: Vec<RecoveryRecord>,
    pub loaded: usize,
    pub total: usize,
    pub next: Option<RecoveryCatalogCursor>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemLookupErrorKind {
    Missing,
    Ambiguous,
    ScanIncomplete,
    InspectOnly,
    Filesystem,
}
#[derive(Debug)]
pub struct ItemLookupError {
    kind: ItemLookupErrorKind,
    escaped_detail: String,
    observed_matches: usize,
}
impl ItemLookupError {
    pub fn kind(&self) -> ItemLookupErrorKind;
    pub fn escaped_detail(&self) -> &str;
    pub fn observed_matches(&self) -> usize;
    pub(crate) fn new(
        kind: ItemLookupErrorKind,
        escaped_detail: String,
        observed_matches: usize,
    ) -> Self;
}
impl std::fmt::Display for ItemLookupError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}
impl std::error::Error for ItemLookupError {}
pub struct RecoveryService<'a> { pub state_root: &'a StateRoot }

impl RecoveryService<'_> {
    pub fn list_page(&self, work_root: &Path, after: Option<&RecoveryCatalogCursor>, limit: usize) -> Result<RecoveryPage>;
    pub fn resolve_unique_item(&self, work_root: &Path, id: ItemId) -> Result<VerifiedBundleRef, ItemLookupError>;
    pub fn reconcile(&self, work_root: &Path, observed: VerifiedBundleRef) -> Result<RecoveryRecord>;
}

```

Clamp `limit` to 1..=200. `list_page` calls side-effect-free `TrashStore::open_for_catalog`, treats an absent trusted/legacy root as `loaded=0,total=0,next=None`, consumes `TrashStore::observations()` directly, and retains only the smallest `limit` final `CatalogKey`s strictly after the cursor in a bounded max-heap. Neither `TrashStore` nor `RecoveryService` exposes a collecting catalog-scan API. Instrumentation must prove heap length never exceeds the clamped limit for the 10,000-record fixture. It makes one streaming pass to count `total`, detect whether a later key exists, and select the page. Set `loaded = records.len()` and `next` to the last returned full key only when a later record exists. A `ChildEnumerator`/observation-stream `Err` aborts the page with `CatalogScanIncomplete`; no partial page or count is returned as complete.

`ItemLookupError` is the public, private-field, non-authorizing diagnostic for `resolve_unique_item`; its exact inspectable kinds distinguish `Missing`, `Ambiguous`, `ScanIncomplete`, `InspectOnly`, and filesystem failure. Construction escapes and truncates detail to 512 bytes, `observed_matches` is bounded by the streaming scan counter, and `Debug`/`Display` contain no `RawUnixName`, `VerifiedBundleRef`, directory handle, receipt bytes, or path capability. A midstream error is always `ScanIncomplete`, never rewritten to `Missing` or a unique match.

Before assigning a final class/key to any `VerifiedCandidate` whose private validated name parses as an `ItemId`, check that exact name fd-relatively in each of the other three v1 locations. If any sibling exists, consume every affected `VerifiedBundleRef` and emit `InspectOnlyRef` plus `RecoveryState::Contradictory`, regardless of which receipt parses; no occurrence retains a claim capability. This per-observation four-location check occurs before heap insertion, so discovering a contradiction never changes a key after pagination. Non-ItemId, malformed, corrupt, and legacy names are inspect-only and remain distinct through `CatalogOpaqueName + location`; the opaque name exposes ordering and escaped display only. A malformed sibling does not abort the page. Unknown names never receive a fabricated `ItemId`.

Only `Verified { receipt: Committed, state: Recoverable }` can enter restore preflight. `resolve_unique_item` streams all four v1 locations and returns a capability only when exactly one matching valid item exists and it remains verified/recoverable; zero matches is `Missing`, two or more is `Ambiguous`, and either result is non-mutating. Any midstream enumeration error is `ScanIncomplete`, never `Missing` or unique. Every non-committed, contradictory, damaged, legacy, inspect-only, or incompletely scanned record is rejected.

Every mutating reconciliation first obtains the matching `PendingControl::Verified` from `StateRoot::pending_controls()`'s streaming iterator, consumes that read-only bundle through `try_claim`, and then consumes the catalog store through `store.claim_existing(claimed_control, verified_bundle_ref)`. No overload accepts `InspectOnlyRef`, `CatalogKey`, bare `ItemId`, opaque/display text, a caller-built tuple, or a borrowed/unclaimed control. The method acquires `claim.lock`, rechecks location/name/observed identity under both retained locks, and moves the whole bundle to `claims` no-clobber before returning its owning transaction. A lost lock, identity drift, competing destination, or rename race returns `InUse`, `StaleObservation`, or `Contradictory` without cleanup while returning the original fixed claim, store, and exact observation. Advance only when the receipt's named disk fact is proven. Keep corrupt, contradictory, and unknown content in place as logical quarantine; never unlink it. `PendingControl::InspectOnly` is surfaced as non-actionable inspection; one corrupt/unknown stream entry neither stops later entries nor authorizes mutation or a second fixed control.

- [ ] **Step 4: Run reconciliation and trust regressions**

Run:

```bash
python3 scripts/run_exact_test.py --test trash_receipt --name reconcile_finishes_only_proven_monotonic_transition
python3 scripts/run_exact_test.py --test trash_receipt --name reconcile_corrupt_receipt_quarantines_without_deletion
python3 scripts/run_exact_test.py --test trash_receipt --name reconcile_unknown_payload_never_purges
python3 scripts/run_exact_test.py --test trash_receipt --name reconcile_loser_reports_in_use_without_mutation --serial
python3 scripts/run_exact_test.py --test trash_receipt --name reconcile_discovers_fixed_control_from_different_cwd
python3 scripts/run_exact_test.py --test trash_receipt --name reconcile_corrupt_pending_control_is_inspect_only_and_does_not_hide_verified_controls
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_corrupt_sibling_does_not_hide_verified_items
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_unknown_bundle_has_no_fabricated_item_id
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_legacy_content_is_inspect_only_unmanaged
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_10000_items_is_deterministic_and_paged
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_limit_one_has_no_gap_or_repeat_across_all_locations
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_duplicate_item_id_is_contradictory_in_every_location_and_not_restorable
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_duplicate_raw_name_is_ordered_by_location
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_restart_preserves_total_order_and_cursor
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_same_name_location_replacement_has_distinct_cursor_key
python3 scripts/run_exact_test.py --test trash_receipt --name catalog_midstream_enumeration_error_never_returns_false_unique_or_restorable
python3 scripts/run_exact_test.py --test trash_receipt --name claim_existing_rejects_identity_swap_after_observation
python3 scripts/run_exact_test.py --test trash_receipt --name claim_existing_two_process_race_has_one_mutating_winner --serial
cargo test --locked --test trash_receipt
cargo test --locked --test source_claim --test state_root
```

Expected: PASS; every full cursor is stable, all duplicate-ID locations are contradictory/non-restorable, the claim race has one mutating winner, corrupt data remains present, and Plan 2 recovery discovery tests pass.

- [ ] **Step 5: Commit reconciliation**

```bash
git add src/recovery.rs src/lib.rs src/trash.rs tests/trash_receipt.rs
git commit -m "feat: reconcile interrupted trash receipts"
```

### Task 5: No-clobber restore engine

**Files:**
- Modify: `src/recovery.rs`
- Modify: `src/trash.rs`
- Modify: `src/state_root.rs`
- Modify: `src/mutation.rs`
- Modify: `src/mutation_ops.rs`
- Modify: `tests/trash_receipt.rs`

- [ ] **Step 1: Write the failing restore matrix**

Add exact tests `restore_defaults_to_skip_on_conflict`, `restore_requires_to_when_original_parent_identity_changed`, `restore_intent_contains_only_submission_draft_not_final_ids`, `restore_payload_chain_is_clone_debug_without_raw_capability_leak`, `restore_worker_allocates_final_ids_and_reserves_before_freeze`, `restore_plan_and_reservation_handles_are_worker_private`, `restore_reservation_collision_regenerates_worker_item_id`, `restore_prepared_control_waits_for_fence_and_cleans_on_abort`, `restore_stale_observed_ref_fails_without_id_fallback`, `restore_observed_ref_duplicate_after_page_is_fail_closed`, `restore_same_fs_publishes_before_restored`, `same_fs_nonempty_directory_trash_and_restore_round_trip`, `restore_cross_fs_retains_payload_until_destination_verified`, `restore_cross_fs_directory_publishes_complete_tree_before_payload_cleanup`, `restore_source_swap_cleanup_deletes_only_private_claim`, `restore_uses_one_outer_control_with_nested_source_claim`, `restore_mirror_mismatch_is_inspect_only_and_never_cleans_payload`, `restore_crash_after_each_state_is_reconcilable`, `restore_two_process_race_has_one_mutating_winner`, and `restore_bundle_terminal_remove_failure_retains_exact_monotonic_ownership`. Cover raw non-UTF-8 names, destination-parent replacement, nested directories, empty files, symlinks inside a directory, directory mode/mtime, competing destination creation, and every fixed-intent/adjacent-replace/fixed-confirm mirror boundary. Freeze `restore_crash_after_each_state_is_reconcilable` as ordered matrix `restore_durable_edge_v1`: for each phase ID `restore_claimed`, `restore_publish_intent`, `restore_destination_published`, `restore_payload_removal_pending`, and `restored`, include `.before_fixed_intent`, `.after_fixed_intent`, `.before_adjacent_replace`, `.after_adjacent_replace`, `.before_fixed_confirmation`, and `.after_fixed_confirmation`, in that phase/edge order. Define `const RESTORE_DURABLE_EDGE_CASES: [&str; 30]`, assert `len() == 30`, and emit every case ID. Freeze `restore_bundle_terminal_remove_failure_retains_exact_monotonic_ownership` as ordered matrix `restore_terminal_remove_v1` with `receipt.before`, `receipt.after`, `lock.before`, `lock.after`, `bundle.before`, `bundle.after`, `parent_sync.before`, and `parent_sync.after`; define `const RESTORE_TERMINAL_REMOVE_CASES: [&str; 8]`, assert `len() == 8`, and reuse that exact table for the restart-resume test below. The draft/ownership tests prove the submitted restore body contains no final IDs or handles, only the worker passes generated IDs into preflight, all reservation/plan fields remain private, and collision retry returns the draft before a new worker ID is chosen. The Clone/Debug contract test embeds the payload in Plan 2's derived `MutationIntentBody`/`MutationIntent`, proves clone compiles and preserves the exact private selector, and proves custom Debug exposes only the complete non-authorizing catalog key/escaped display rather than raw `RawUnixName` bytes. The prepared-control test proves the restore operation's new fixed control is create-new-reserved before the final request freezes, the existing trash bundle remains in its observed location until `FenceInstalled`, and pre-ack cancellation removes/syncs the new reservation without moving or mutating the observed bundle. The observed-selector tests replace the object and insert a duplicate after the catalog page respectively; both must fail closed without retrying lookup by `ItemId`. The terminal-removal test proves each case returns exactly one monotonic `TrashBundleRemovalRecovery` variant with the original restore control and no already-consumed handle.

The same-filesystem end-to-end test creates a non-empty tree, dispatches normal trash through `mutation_ops`, lists the committed record, destroys and reopens `StateRoot`/`TrashStore`, restores through `RestoreSelector::Observed` with the returned `VerifiedBundleRef`, and proves the exact tree and metadata are back while the trash payload and terminal fixed controls are absent. The cross-filesystem directory restore test starts from a directory that was legally trashed by same-filesystem rename, injects a destination with a different device ID, and proves no destination root becomes visible before the complete staged topology is verified/synced; at every failure point either the original trash payload remains or the committed destination is a verified complete copy.

Also add exact `restore_prepare_failure_returns_draft_or_fixed_claim_ownership`, injecting one failure before reservation and one after it. The former returns the original `RestoreIntent`; the latter returns the same claimed control and draft for terminal cleanup, with no ItemId reopen.

Also add exact `restore_preparation_failure_enters_worker_dispatch_with_ownership`, covering both invalid and reservation-collision branches through `ItemPreparationFailureInner::Restore` without flattening or losing the intent/claim.

Also add exact `restore_prepared_abort_preserves_observed_bundle_and_consumes_fixed_typestate`, `restore_executor_panic_reclaims_exact_fixed_ref_and_reconciles`, `restore_restart_resumes_terminal_remove_after_each_monotonic_phase`, and `restore_terminal_facts_are_revalidated_immediately_before_fixed_unlink`. The abort test enters only through Plan 2's central dispatcher; the panic test injects unwind after the opaque reservation move and at each durable Restore edge and reuses the exact 30 ordered `RESTORE_DURABLE_EDGE_CASES`; the restart test reuses the exact eight ordered `RESTORE_TERMINAL_REMOVE_CASES`, kills/reopens at each named boundary, and must continue without a catalog receipt; the terminal test races subordinate recreation/drift between initial verification and fixed removal and requires refusal with the owning typestate retained.

Also add exact `restore_rejects_replaced_observed_work_root_and_location_parent`, replacing each parent after the page/intent is built and proving worker preflight fails closed without opening the same item ID under the replacement.

Also add exact `restore_authorizing_facts_cannot_outlive_claim_or_locked_receipt_snapshot`, now that Restore types exist. Put it, `restore_prepare_failure_returns_draft_or_fixed_claim_ownership`, `restore_preparation_failure_enters_worker_dispatch_with_ownership`, `restore_crash_after_each_state_is_reconcilable`, `restore_bundle_terminal_remove_failure_retains_exact_monotonic_ownership`, `restore_prepared_abort_preserves_observed_bundle_and_consumes_fixed_typestate`, `restore_executor_panic_reclaims_exact_fixed_ref_and_reconciles`, `restore_restart_resumes_terminal_remove_after_each_monotonic_phase`, and `restore_terminal_facts_are_revalidated_immediately_before_fixed_unlink` in `recovery::tests` and run them with `--lib`. They inspect private ownership or use only `#[cfg(test)] pub(crate)` fault hooks. Keep the remaining public worker/catalog/filesystem tests in `tests/trash_receipt.rs`; do not expose a production reservation, proof, or deterministic fault controller.

- [ ] **Step 2: Run restore tests and confirm RED**

Run each target exactly:

```bash
restore_durable_case_args=(
  --expect-case restore_claimed.before_fixed_intent --expect-case restore_claimed.after_fixed_intent
  --expect-case restore_claimed.before_adjacent_replace --expect-case restore_claimed.after_adjacent_replace
  --expect-case restore_claimed.before_fixed_confirmation --expect-case restore_claimed.after_fixed_confirmation
  --expect-case restore_publish_intent.before_fixed_intent --expect-case restore_publish_intent.after_fixed_intent
  --expect-case restore_publish_intent.before_adjacent_replace --expect-case restore_publish_intent.after_adjacent_replace
  --expect-case restore_publish_intent.before_fixed_confirmation --expect-case restore_publish_intent.after_fixed_confirmation
  --expect-case restore_destination_published.before_fixed_intent --expect-case restore_destination_published.after_fixed_intent
  --expect-case restore_destination_published.before_adjacent_replace --expect-case restore_destination_published.after_adjacent_replace
  --expect-case restore_destination_published.before_fixed_confirmation --expect-case restore_destination_published.after_fixed_confirmation
  --expect-case restore_payload_removal_pending.before_fixed_intent --expect-case restore_payload_removal_pending.after_fixed_intent
  --expect-case restore_payload_removal_pending.before_adjacent_replace --expect-case restore_payload_removal_pending.after_adjacent_replace
  --expect-case restore_payload_removal_pending.before_fixed_confirmation --expect-case restore_payload_removal_pending.after_fixed_confirmation
  --expect-case restored.before_fixed_intent --expect-case restored.after_fixed_intent
  --expect-case restored.before_adjacent_replace --expect-case restored.after_adjacent_replace
  --expect-case restored.before_fixed_confirmation --expect-case restored.after_fixed_confirmation
)
restore_terminal_case_args=(
  --expect-case receipt.before --expect-case receipt.after
  --expect-case lock.before --expect-case lock.after
  --expect-case bundle.before --expect-case bundle.after
  --expect-case parent_sync.before --expect-case parent_sync.after
)
python3 scripts/run_exact_test.py --test trash_receipt --name restore_defaults_to_skip_on_conflict
python3 scripts/run_exact_test.py --test trash_receipt --name restore_requires_to_when_original_parent_identity_changed
python3 scripts/run_exact_test.py --test trash_receipt --name restore_intent_contains_only_submission_draft_not_final_ids
python3 scripts/run_exact_test.py --test trash_receipt --name restore_payload_chain_is_clone_debug_without_raw_capability_leak
python3 scripts/run_exact_test.py --test trash_receipt --name restore_worker_allocates_final_ids_and_reserves_before_freeze
python3 scripts/run_exact_test.py --test trash_receipt --name restore_plan_and_reservation_handles_are_worker_private
python3 scripts/run_exact_test.py --test trash_receipt --name restore_reservation_collision_regenerates_worker_item_id
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_prepare_failure_returns_draft_or_fixed_claim_ownership
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_preparation_failure_enters_worker_dispatch_with_ownership
python3 scripts/run_exact_test.py --test trash_receipt --name restore_prepared_control_waits_for_fence_and_cleans_on_abort
python3 scripts/run_exact_test.py --test trash_receipt --name restore_stale_observed_ref_fails_without_id_fallback
python3 scripts/run_exact_test.py --test trash_receipt --name restore_observed_ref_duplicate_after_page_is_fail_closed
python3 scripts/run_exact_test.py --test trash_receipt --name restore_same_fs_publishes_before_restored
python3 scripts/run_exact_test.py --test trash_receipt --name same_fs_nonempty_directory_trash_and_restore_round_trip
python3 scripts/run_exact_test.py --test trash_receipt --name restore_cross_fs_retains_payload_until_destination_verified
python3 scripts/run_exact_test.py --test trash_receipt --name restore_cross_fs_directory_publishes_complete_tree_before_payload_cleanup
python3 scripts/run_exact_test.py --test trash_receipt --name restore_source_swap_cleanup_deletes_only_private_claim
python3 scripts/run_exact_test.py --test trash_receipt --name restore_uses_one_outer_control_with_nested_source_claim
python3 scripts/run_exact_test.py --test trash_receipt --name restore_mirror_mismatch_is_inspect_only_and_never_cleans_payload
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_crash_after_each_state_is_reconcilable --case-matrix restore_durable_edge_v1 "${restore_durable_case_args[@]}"
python3 scripts/run_exact_test.py --test trash_receipt --name restore_two_process_race_has_one_mutating_winner --serial
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_bundle_terminal_remove_failure_retains_exact_monotonic_ownership --case-matrix restore_terminal_remove_v1 "${restore_terminal_case_args[@]}"
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_prepared_abort_preserves_observed_bundle_and_consumes_fixed_typestate
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_executor_panic_reclaims_exact_fixed_ref_and_reconciles --case-matrix restore_durable_edge_v1 "${restore_durable_case_args[@]}"
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_restart_resumes_terminal_remove_after_each_monotonic_phase --serial --case-matrix restore_terminal_remove_v1 "${restore_terminal_case_args[@]}"
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_terminal_facts_are_revalidated_immediately_before_fixed_unlink
python3 scripts/run_exact_test.py --test trash_receipt --name restore_rejects_replaced_observed_work_root_and_location_parent
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_authorizing_facts_cannot_outlive_claim_or_locked_receipt_snapshot
cargo test --locked --test trash_receipt
```

Expected: FAIL because restore request/preflight/execution types do not exist.

- [ ] **Step 3: Implement restore preflight and execution**

Add:

```rust
#[derive(Clone, Debug)]
pub enum RestoreSelector {
    Observed(VerifiedBundleRef),
    UniqueItem(ItemId),
}
#[derive(Clone, Debug)]
pub struct RestoreRequest { pub selector: RestoreSelector, pub to: Option<RawUnixPath> }
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RestoreWorkRoot {
    raw: RawUnixPath,
    identity: PathIdentity,
}
#[derive(Clone, Debug)]
pub struct RestoreIntent {
    work_root: RestoreWorkRoot,
    request: RestoreRequest,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RestorePublicationRecord {
    Rename {
        expected_published_identity: PathIdentity,
    },
    StagedCopy {
        staged_parent: RawUnixPath,
        staged_parent_identity: PathIdentity,
        staged_name: RawUnixName,
        staged_identity: PathIdentity,
        expected_published_identity: PathIdentity,
    },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RestoreTerminalRecord {
    destination_parent: RawUnixPath,
    destination_parent_identity: PathIdentity,
    destination_name: RawUnixName,
    expected_destination: PathIdentity,
    payload_parent: RawUnixPath,
    payload_parent_identity: PathIdentity,
    payload_name: RawUnixName,
    expected_payload_before_cleanup: Option<PathIdentity>,
    publication: RestorePublicationRecord,
    restored_receipt_revision: u64,
    restored_receipt_sha256: [u8; 32],
}
pub(crate) struct RestoreTerminalPreconditions {
    record: RestoreTerminalRecord,
    trusted_destination_parent: TrustedDir,
    trusted_payload_parent: TrustedDir,
    expected_bundle_identity: PathIdentity,
}
pub(crate) enum TrashBundleRemovalRecovery {
    ReceiptPresent {
        control: ClaimedControlBundle,
        store: TrashStore,
        location: BundleLocation,
        name: RawUnixName,
        bundle_dir: Arc<TrustedDir>,
        receipt_file: AtomicReceiptFile,
        claim_lock: ClaimedChildLock,
        bundle_identity: PathIdentity,
    },
    ReceiptAbsent {
        control: ClaimedControlBundle,
        store: TrashStore,
        location: BundleLocation,
        name: RawUnixName,
        bundle_dir: Arc<TrustedDir>,
        claim_lock: ClaimedChildLock,
        bundle_identity: PathIdentity,
    },
    LockAbsent {
        control: ClaimedControlBundle,
        store: TrashStore,
        location: BundleLocation,
        name: RawUnixName,
        bundle_dir: Arc<TrustedDir>,
        bundle_identity: PathIdentity,
    },
    BundleAbsent {
        control: ClaimedControlBundle,
        store: TrashStore,
        location: BundleLocation,
        name: RawUnixName,
    },
}
pub(crate) enum TrashBundleTerminalRemoveStage {
    VerifyRestored,
    ReceiptRemove,
    LockRemove,
    BundleRemove,
    ParentSync,
}
pub(crate) enum TrashBundleTerminalDiskTruth {
    ReceiptAndLockPresent(PathIdentityKey),
    ReceiptAbsentBundlePresent(PathIdentityKey),
    ReceiptAndLockAbsentBundlePresent(PathIdentityKey),
    BundleAbsent,
    Unreadable { escaped_error: String },
}
pub(crate) struct TrashBundleTerminalRemoveFailure {
    recovery: TrashBundleRemovalRecovery,
    preconditions: RestoreTerminalPreconditions,
    stage: TrashBundleTerminalRemoveStage,
    disk_truth: TrashBundleTerminalDiskTruth,
    source: TrustedFsError,
}
pub(crate) struct RestorePlan {
    store: TrashStore,
    bundle_ref: VerifiedBundleRef,
    receipt: TrashReceipt,
    payload_identity: PathIdentity,
    destination: RawUnixPath,
    destination_parent: PathIdentity,
    control: ClaimedControlBundle,
}
#[doc(hidden)]
pub struct PreparedRestoreReservation {
    plan: RestorePlan,
}
pub(crate) struct RestorePreparationFailure {
    ownership: RestorePreparationOwnership,
    source: PreparationError,
}
pub(crate) enum RestorePreparationOwnership {
    Draft(RestoreIntent),
    FixedReservationOwned {
        intent: RestoreIntent,
        failure: ControlReservationFailure,
    },
    Fixed {
        intent: RestoreIntent,
        control: ClaimedControlBundle,
    },
}
pub(crate) enum PrepareRestoreError {
    Invalid(RestorePreparationFailure),
    ControlNameCollision {
        candidate_item_id: ItemId,
        collided_item_id: ItemId,
        intent: RestoreIntent,
    },
}

impl RestorePreparationFailure {
    pub(crate) fn source(&self) -> &PreparationError;
    pub(crate) fn into_parts(self) -> (RestorePreparationOwnership, PreparationError);
}

impl PrepareRestoreError {
    pub(crate) fn diagnostic(&self) -> (PreparationFailureKind, &str);
}

pub(crate) fn abort_prepared_restore(
    context: &MutationContext,
    plan: &ItemPlan,
    prepared: PreparedRestoreReservation,
) -> PreparedAbortOutcome;

pub(crate) fn recover_restore_preparation_failure(
    context: &MutationContext,
    failure: PrepareRestoreError,
) -> PreparedAbortOutcome;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RestoreControlState {
    recovery_item_id: ItemId,
    work_root: RestoreWorkRoot,
    bundle: TrashBundleSelector,
    mirror: ReceiptMirror,
    phase: TrashState,
    terminal: Option<RestoreTerminalRecord>,
}
struct RestoreTerminalSyncWitness;
#[doc(hidden)]
pub struct RestoreTerminalExpectation {
    terminal_record: RestoreTerminalRecord,
    trusted_destination_parent: TrustedDir,
    trusted_location_parent: TrustedDir,
    bundle_name: RawUnixName,
    removed_bundle_identity: PathIdentity,
    affected_parents_synced: RestoreTerminalSyncWitness,
}
pub(crate) struct RestoreFixedTerminalReady {
    control: ClaimedControlBundle,
    terminal: RestoreTerminalExpectation,
}
pub(crate) struct RestoreFixedTerminalVerifyFailure {
    failure: TerminalVerifyFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreFixedMirrorEdge {
    RestoreClaimedToRestorePublishIntent,
    RestorePublishIntentToRestoreDestinationPublished,
    RestoreDestinationPublishedToRestorePayloadRemovalPending,
    RestorePayloadRemovalPendingToRestored,
}

// In Task 5, replace the Task 3 definitions with these complete final enums.
pub(crate) enum TrashTransitionEdge {
    PreparedToPayloadReady,
    PayloadReadyToPayloadPublished,
    PayloadPublishedToSourceRemovalPending,
    SourceRemovalPendingToCommitted,
    RestoreClaimedToRestorePublishIntent,
    RestorePublishIntentToRestoreDestinationPublished,
    RestoreDestinationPublishedToRestorePayloadRemovalPending,
    RestorePayloadRemovalPendingToRestored,
}
pub(crate) enum TrashTransitionExpectation {
    PayloadReady { expected_payload: PathIdentity },
    BundlePublished { expected_bundle: PathIdentity },
    SourceRemovalPending { expected_payload: PathIdentity },
    Committed { expected_payload: PathIdentity, source_absent: RawUnixPath },
    RestorePublished { expected_destination: PathIdentity },
    Restored { expected_destination: PathIdentity, payload_absent: RawUnixPath },
}

struct RestorePreparedAbortUnchangedWitness;
#[doc(hidden)]
pub struct RestorePreparedAbortFacts {
    trusted_location_parent: TrustedDir,
    bundle_name: RawUnixName,
    bundle_identity: PathIdentity,
    receipt_identity: PathIdentity,
    receipt_revision: u64,
    receipt_sha256: [u8; 32],
    parent_identity: PathIdentityKey,
    unchanged: RestorePreparedAbortUnchangedWitness,
}

pub(crate) fn validate_restore_fixed_mirror_intent_edge(
    current: &RestoreControlState,
    next: &RestoreControlState,
    edge: RestoreFixedMirrorEdge,
    current_adjacent: &AdjacentReceiptFacts<'_>,
    next_intent: &MirrorIntent,
) -> Result<(), TrustedFsError>;

pub(crate) fn validate_restore_mirror_confirmation(
    current: &RestoreControlState,
    next: &RestoreControlState,
    expected_intent: &MirrorIntent,
    confirmed_adjacent: &AdjacentReceiptFacts<'_>,
) -> Result<(), TrustedFsError>;

pub(crate) fn validate_restore_prepared_abort(
    current: &RestoreControlState,
    facts: &RestorePreparedAbortFacts,
) -> Result<(), TrustedFsError>;

pub(crate) fn validate_restore_terminal(
    current: &RestoreControlState,
    facts: &RestoreTerminalExpectation,
) -> Result<(), TrustedFsError>;

impl PreparedRestoreReservation {
    pub(crate) fn recovery_ref(&self) -> ControlRecoveryRef;
}

pub(crate) fn recover_restore_execution_panic(
    context: &MutationContext,
    plan: &ItemPlan,
    claimed: ClaimedControlBundle,
) -> ItemOutcome;

pub(crate) fn recover_restore_pending_control(
    context: &MutationContext,
    claimed: ClaimedControlBundle,
) -> StartupRecoveryNotice;

impl ClaimedTrashTransaction {
    pub(crate) fn remove_restored_bundle_terminal(
        self,
        preconditions: RestoreTerminalPreconditions,
    ) -> Result<RestoreFixedTerminalReady, TrashBundleTerminalRemoveFailure>;
}

impl TrashBundleRemovalRecovery {
    pub(crate) fn resume_terminal_remove(
        self,
        preconditions: RestoreTerminalPreconditions,
    ) -> Result<RestoreFixedTerminalReady, TrashBundleTerminalRemoveFailure>;
}

impl TrashBundleTerminalRemoveFailure {
    pub(crate) fn stage(&self) -> TrashBundleTerminalRemoveStage;
    pub(crate) fn disk_truth(&self) -> &TrashBundleTerminalDiskTruth;
    pub(crate) fn source(&self) -> &TrustedFsError;
    pub(crate) fn retry(self) -> Result<RestoreFixedTerminalReady, TrashBundleTerminalRemoveFailure>;
}

impl RestoreFixedTerminalReady {
    pub(crate) fn verify_fixed_terminal(
        self,
    ) -> Result<VerifiedTerminalControl, RestoreFixedTerminalVerifyFailure>;
}

impl RestoreFixedTerminalVerifyFailure {
    pub(crate) fn source(&self) -> &crate::state_root::StateRootError;
    pub(crate) fn retry(self) -> Result<VerifiedTerminalControl, RestoreFixedTerminalVerifyFailure>;
}

pub(crate) fn resume_restore_terminal_from_fixed(
    context: &MutationContext,
    claimed: ClaimedControlBundle,
) -> ItemOutcome;

impl RecoveryService<'_> {
    pub(crate) fn prepare_restore(
        &self,
        context: &MutationContext,
        operation_id: OperationId,
        candidate_item_id: ItemId,
        intent: RestoreIntent,
    ) -> Result<(ItemPlan, PreparedRestoreReservation, MutationFenceSpec), PrepareRestoreError>;
    pub(crate) fn execute_restore(
        &self,
        request: &OperationRequest,
        plan: &ItemPlan,
        reservation: PreparedRestoreReservation,
        mutation: &MutationContext,
    ) -> ItemOutcome;
}

impl MutationIntent {
    pub fn restore(submission_id: SubmissionId, restore: RestoreIntent) -> Self;
}

impl RestoreIntent {
    pub(crate) fn from_observed(request: RestoreRequest) -> Result<Self, ItemLookupError>;
    pub fn capture_cli(work_root: &Path, request: RestoreRequest) -> Result<Self, ItemLookupError>;
    pub(crate) fn work_root(&self) -> &RawUnixPath;
    pub(crate) fn work_root_identity(&self) -> &PathIdentity;
    pub(crate) fn request(&self) -> &RestoreRequest;
}
```

Extend the same concrete Plan 2 enums with `MutationIntentBody::Restore(RestoreIntent)`, `PreparedReservation::Restore(PreparedRestoreReservation)`, crate-private `ItemPreparationFailureInner::Restore(PrepareRestoreError)`, `ControlState::Restore { host, source_claim, mirror_intent }`, `HostMirrorEdge::Restore(RestoreFixedMirrorEdge)`, `PreparedControlAbortExpectation::Restore(RestorePreparedAbortFacts)`, and `TerminalExpectation::Restore(RestoreTerminalExpectation)`. Also replace Task 3's Trash-only `TrashTransitionEdge`/`TrashTransitionExpectation` definitions with the complete final enums shown above; no Restore variant exists before this task. The public abort and terminal fact wrappers are `#[doc(hidden)]` with private fields/constructors, non-Clone, and non-serde solely to keep the public enum visibility valid. `state_root.rs` delegates Restore fixed-edge, prepared-abort, and terminal validation/revalidation to the crate-private owner functions in this task; it never reads private Restore fields or trusts a caller boolean/hash. `RestoreIntent` is the concrete typed draft submitted by TUI/CLI and contains no `OperationId`, execution `ItemId`, claimed control, or reservation. Its private `RestoreWorkRoot` binds raw absolute path plus the full identity observed at submission: `from_observed` derives both only from the selected private `VerifiedBundleRef`, while `capture_cli` performs one no-follow identity observation inside recovery.rs for the CLI-only unique selector. Callers cannot supply an arbitrary identity or read raw capability fields. This observation is a stale-input guard, not final preflight; the worker reopens/rechecks it and captures all final identities. `prepare_restore` is crate-private and is called only by the Plan 2 mutation worker during background preparation after that worker has generated `operation_id` and `candidate_item_id`; it returns only the final `ItemPlan`, private owning reservation, and final fence spec. It never constructs an `OperationRequest`, publishes a `PreparedNotice`, or allocates IDs itself. The worker alone freezes the one-item immutable `OperationRequest` from that returned plan after every reservation succeeds. On `ReservationCollision`, the error returns the intact typed intent; the worker proves cleanup of any partial reservation, generates a new candidate execution item ID, and calls `prepare_restore` again. Neither the catalog recovery ID nor an existing colliding control name is opened or reused.

Every preparation error is owning. Validation before fixed reservation returns `RestorePreparationOwnership::Draft(intent)`; a post-effect error inside `reserve_control` returns `FixedReservationOwned { intent, failure }` and keeps the partial bundle/lock/receipt stage and truth attached to `ControlReservationFailure`; only a successfully completed reservation followed by a later error returns `Fixed { intent, control }`. The preceding shorthand “On `ReservationCollision`” refers to the diagnostic kind; the concrete typed branch is `PrepareRestoreError::ControlNameCollision`, which contains the unchanged draft and collided candidate ID but no opened colliding bundle. No `PrepareRestoreError` variant contains only a naked `PreparationError`, and the worker never reconstructs ownership by reopening a bare `ItemId`.

`mutation_ops::recover_preparation_failure` passes the whole Restore variant to `recover_restore_preparation_failure`; `abort_prepared_item` passes the whole opaque reservation to `abort_prepared_restore`. Because restore preparation has not yet claimed or moved the observed trash bundle, both owner-module routines may remove/sync only the new restore fixed reservation and must leave the observed bundle byte-for-byte unchanged. The owner no-follow reopens the exact observed location/name, verifies full bundle/receipt identities plus revision/hash, retains that trusted parent/name in `RestorePreparedAbortFacts`, and consumes the new fixed claim through `verify_prepared_abort(PreparedControlAbortExpectation::Restore(facts)) -> VerifiedPreparedControlAbort::remove`. The typestate revalidates the observed bundle unchanged immediately before fixed unlink; failure retains ownership for retry/classification. `ReleasedNoEffect` requires that removal and parent sync to be proved; otherwise they classify the still-discoverable control as `CleanupRequired`/`Indeterminate`. The restore preparation and pre-ack tests invoke the closed central dispatches and assert the observed bundle identity/receipt/payload never changes.

`RestoreSelector::Observed` reopens and revalidates the bound work root, location parent, exact private name/location/identity and rechecks the other three v1 locations for a newly appeared duplicate; stale/replaced root or parent, replaced child, duplicated, or non-recoverable observations fail without falling back to an ID search. `RestoreSelector::UniqueItem` is the CLI-only path and calls `resolve_unique_item`; zero or multiple matches fail without effect. Only a verified `Committed` receipt classified `Recoverable` proceeds. With worker-provided IDs, preparation constructs `ControlEnvelope { header: ControlHeader { operation_id, item_id: candidate_item_id, protocol: ControlProtocol::RestoreV1, .. }, state: ControlState::Restore { host, source_claim: None, mirror_intent: None }, .. }` and create-new-reserves it through `StateRoot::reserve_control`. It exhaustively maps `ReserveControlError::Collision(collided_item_id)` to `PrepareRestoreError::ControlNameCollision` without opening the colliding bundle, `NoEffect(error)` to `Invalid` with `RestorePreparationOwnership::Draft`, and `Owned(failure)` to `Invalid` with `FixedReservationOwned`; owner recovery consumes `ControlReservationFailure::abort` and any returned `ControlReservationAbortFailure::retry` before deciding whether a new candidate ID is safe. The private `RestorePlan` owns a successful original claim plus the catalog store and exact verified observation; it is nested in `PreparedRestoreReservation`, then `PreparedReservation::Restore`, and never crosses `PreparedNotice` or enters `App`. The catalog/recovery ID remains `TrashReceipt::item_id` and is distinct from the restore execution `ItemId`. A destination collision always skips/fails without overwrite; the only first-release alternate is an explicit `--to <directory>`, freshly identity-checked. The observed trash bundle is not claimed or moved during preparation. Plan 4 TUI passes `RestoreIntent::from_observed(RestoreRequest { selector: Observed(record.bundle_ref), .. })` from the selected latest catalog page; it never degrades selection to a bare ID or invents a root identity.

After the non-droppable `Prepared` event is acknowledged with `FenceInstalled`, worker dispatch consumes `PreparedRestoreReservation`. `execute_restore` destructures its private `RestorePlan`, calls `store.claim_existing(control, bundle_ref)`, and receives the sole `ClaimedTrashTransaction`; no bare control, receipt handle, or raw child capability is returned to the coordinator. A stale exact observation fails no-effect and goes through verified reservation cleanup before publication. The transaction uses the already claimed `ControlProtocol::RestoreV1` outer envelope and standardized nested source-claim field; it never creates a second control. Implement `RestoreClaimed -> RestorePublishIntent -> RestoreDestinationPublished -> RestorePayloadRemovalPending -> Restored`. The create-new fixed state contains the full `RestoreWorkRoot` and `TrashBundleSelector` before the observed bundle may move. Before destructive publication, construct and sync `RestoreTerminalRecord` with raw destination/payload parents and names, both full parent identities, expected destination/payload identities, exact rename-or-staged publication facts, staged parent/name/identity where applicable, and the receipt revision/hash. Store it in `RestoreControlState.terminal`; later phases may update only fields whose exact owner validator names the edge and may never clear it. Every adjacent restore-receipt update uses Task 3's owning `TrashReceiptProof::advance_receipt` followed by fixed mirror confirmation; payload cleanup is forbidden on an adjacent-ahead, hash-mismatched, unlocked-snapshot, or unconfirmed restore state. Cancellation, panic, or disconnect before `FenceInstalled` must prove removal and parent sync of the new restore control or surface its durable residue; it must not claim the observed trash bundle.

`PreparedReservation::recovery_seed` maps Restore to the prepared control's exact `ControlRecoveryRef`. After an unwind, Plan 2's closed `(plan.kind, ControlState::Restore)` arm calls `recover_restore_execution_panic` with the newly reclaimed fixed control. That wrapper validates the retained plan/header match, then consumes the control through private `reconcile_restore_claimed_control(context, claimed)`. `recover_restore_pending_control(context, claimed)` accepts only the claimed control and calls the same core; extend Plan 2's existing closed `recover_startup_control` with the `ControlState::Restore` arm and return its existing `StartupRecoveryNotice`. The core reopens only `RestoreControlState.work_root`, `bundle`, and `terminal` selectors and validates every full identity fixed-before-adjacent; it never asks the catalog for an ID match. If nested SourceClaim cleanup is pending, it uses the identical exhaustive owning match as the Trash core: direct-match every `ClaimResult`, retain `claimed` on `NoAdjacentEffect`, and consume `AdjacentOwned.recovery` through `SourceClaimRecovery::reconcile` rather than flattening it to `ClaimError`/`ClaimObservation`. Panic maps the decision to `ItemOutcome`; startup maps it to the non-authorizing notice without fabricating an `ItemPlan`.

Every transition failure in `execute_restore` is consumed through the Trash-owned recovery methods before the function returns an outcome. `recovery.rs` never matches a private field or drops `TrashAdvanceError`/`TrashTransitionVerifyFailure`; the reconstructed transaction remains available for reread reconciliation, terminal cleanup, or a truthful durable-residue outcome.

Same-filesystem restore supports regular files, symlinks, and complete non-empty directory trees through one no-clobber rename. Cross-filesystem restore supports regular files, symlinks, and a directory that previously entered trash by a supported same-filesystem rename. A directory restore builds the complete topology under the owned adjacent staging root, copies symlinks without following, applies file and directory metadata bottom-up, syncs every payload object and affected directory, rechecks payload/destination-parent identity, then publishes only the complete root no-clobber. It retains the trash payload until the destination is proven. For copied-payload cleanup, execution no-follow reopens and identity-verifies the payload parent from the private plan, then temporarily lends its fixed-control field to `SourceClaim::acquire(context.state_root.as_ref(), &trusted_payload_parent, control, plan, ClaimAction::RestorePayloadCleanup)` while retaining all trash bundle handles; the borrow never escapes `execute_restore`. `SourceClaim` advances the nested `ControlState::Restore.source_claim` and creates no second fixed control. Write `Restored` only after destination identity and required payload absence are verified, then no-follow reopen and fully revalidate `RestoreControlState.terminal` to build private `RestoreTerminalPreconditions`; live trusted parents, not serialized facts alone, authorize cleanup. `ClaimedTrashTransaction::remove_restored_bundle_terminal(preconditions)` verifies the `Restored` receipt under the live lock, carries the current post-confirmation bundle snapshot into `ReceiptPresent`, and consumes Plan 2's owning receipt removal. On successful receipt removal it immediately calls `bundle_dir.refresh_identity()`, requires `same_object` with the prior snapshot, and carries the refreshed full snapshot into `ReceiptAbsent`; after verified `claim.lock` removal it repeats that refresh/same-object check and carries the newest snapshot into `LockAbsent`. Only `LockAbsent` may call `location_parent.unlink_verified_child(bundle_name, bundle_dir.identity(), true)`, so the final bundle unlink never uses the snapshot from before either child deletion. After the actual location-parent sync it mints `RestoreTerminalExpectation` with the immutable terminal record, retained trusted destination/location parents, exact bundle name/removed identity, and sync witness, then returns `RestoreFixedTerminalReady`; it does not retain a payload-parent capability that cannot be reconstructed after a crash. A failure returns `ReceiptPresent`, `ReceiptAbsent`, `LockAbsent`, or `BundleAbsent` with the original fixed restore claim, original preconditions, remaining handles, exact stage, and reread disk truth; every present-bundle variant owns the full snapshot refreshed at its own monotonic stage and its consuming `retry` continues without reconstructing a handle. If the process dies and only the fixed Restore control remains, `resume_restore_terminal_from_fixed` uses the retained `RestoreWorkRoot`, `TrashBundleSelector`, and `RestoreTerminalRecord` to no-follow inspect and reconstruct exactly one monotonic phase, including receipt-absent/lock-absent/bundle-absent. Legitimate receipt/lock deletion by this owned cleanup may change the location-parent snapshot: restart first proves the same trusted parent object and the exact expected child/absence sequence, then captures a fresh full parent snapshot; it never requires the pre-cleanup location-parent `same_snapshot` after an authorized own child mutation. In a bundle-absent phase it reopens only the recorded destination and location parents, revalidates destination identity plus exact bundle absence, and mints the same terminal expectation without a vanished payload-parent handle. It resumes cleanup without a catalog receipt, bare ItemId, cwd, or displayed path. Success consumes `RestoreFixedTerminalReady::verify_fixed_terminal`; state root retains/revalidates the complete owner facts in `VerifiedTerminalControl`, whose consuming `remove` alone removes the fixed bundle. The eight-case terminal-removal and restart tests assert the full identity changes after each child unlink while `same_object` remains true and the final unlink uses the newest snapshot. There is no bare-ID reopen, detachable terminal token, or generic unlink retry.

First-release support boundary is therefore explicit: cross-filesystem **trash ingestion** accepts only regular files and symlinks; same-filesystem trash accepts non-empty directories; cross-filesystem **restore** accepts a verified directory payload using complete-tree staging. Unsupported special objects and a directory presented directly to cross-filesystem trash fail before fixed-control or bundle reservation. Every supported cross-filesystem failure preserves at least one verified complete copy and never publishes a partial directory root.

- [ ] **Step 4: Run restore and full receipt tests**

Run:

```bash
restore_durable_case_args=(
  --expect-case restore_claimed.before_fixed_intent --expect-case restore_claimed.after_fixed_intent
  --expect-case restore_claimed.before_adjacent_replace --expect-case restore_claimed.after_adjacent_replace
  --expect-case restore_claimed.before_fixed_confirmation --expect-case restore_claimed.after_fixed_confirmation
  --expect-case restore_publish_intent.before_fixed_intent --expect-case restore_publish_intent.after_fixed_intent
  --expect-case restore_publish_intent.before_adjacent_replace --expect-case restore_publish_intent.after_adjacent_replace
  --expect-case restore_publish_intent.before_fixed_confirmation --expect-case restore_publish_intent.after_fixed_confirmation
  --expect-case restore_destination_published.before_fixed_intent --expect-case restore_destination_published.after_fixed_intent
  --expect-case restore_destination_published.before_adjacent_replace --expect-case restore_destination_published.after_adjacent_replace
  --expect-case restore_destination_published.before_fixed_confirmation --expect-case restore_destination_published.after_fixed_confirmation
  --expect-case restore_payload_removal_pending.before_fixed_intent --expect-case restore_payload_removal_pending.after_fixed_intent
  --expect-case restore_payload_removal_pending.before_adjacent_replace --expect-case restore_payload_removal_pending.after_adjacent_replace
  --expect-case restore_payload_removal_pending.before_fixed_confirmation --expect-case restore_payload_removal_pending.after_fixed_confirmation
  --expect-case restored.before_fixed_intent --expect-case restored.after_fixed_intent
  --expect-case restored.before_adjacent_replace --expect-case restored.after_adjacent_replace
  --expect-case restored.before_fixed_confirmation --expect-case restored.after_fixed_confirmation
)
restore_terminal_case_args=(
  --expect-case receipt.before --expect-case receipt.after
  --expect-case lock.before --expect-case lock.after
  --expect-case bundle.before --expect-case bundle.after
  --expect-case parent_sync.before --expect-case parent_sync.after
)
python3 scripts/run_exact_test.py --test trash_receipt --name restore_defaults_to_skip_on_conflict
python3 scripts/run_exact_test.py --test trash_receipt --name restore_requires_to_when_original_parent_identity_changed
python3 scripts/run_exact_test.py --test trash_receipt --name restore_intent_contains_only_submission_draft_not_final_ids
python3 scripts/run_exact_test.py --test trash_receipt --name restore_payload_chain_is_clone_debug_without_raw_capability_leak
python3 scripts/run_exact_test.py --test trash_receipt --name restore_worker_allocates_final_ids_and_reserves_before_freeze
python3 scripts/run_exact_test.py --test trash_receipt --name restore_plan_and_reservation_handles_are_worker_private
python3 scripts/run_exact_test.py --test trash_receipt --name restore_reservation_collision_regenerates_worker_item_id
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_prepare_failure_returns_draft_or_fixed_claim_ownership
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_preparation_failure_enters_worker_dispatch_with_ownership
python3 scripts/run_exact_test.py --test trash_receipt --name restore_prepared_control_waits_for_fence_and_cleans_on_abort
python3 scripts/run_exact_test.py --test trash_receipt --name restore_stale_observed_ref_fails_without_id_fallback
python3 scripts/run_exact_test.py --test trash_receipt --name restore_observed_ref_duplicate_after_page_is_fail_closed
python3 scripts/run_exact_test.py --test trash_receipt --name restore_same_fs_publishes_before_restored
python3 scripts/run_exact_test.py --test trash_receipt --name same_fs_nonempty_directory_trash_and_restore_round_trip
python3 scripts/run_exact_test.py --test trash_receipt --name restore_cross_fs_retains_payload_until_destination_verified
python3 scripts/run_exact_test.py --test trash_receipt --name restore_cross_fs_directory_publishes_complete_tree_before_payload_cleanup
python3 scripts/run_exact_test.py --test trash_receipt --name restore_source_swap_cleanup_deletes_only_private_claim
python3 scripts/run_exact_test.py --test trash_receipt --name restore_uses_one_outer_control_with_nested_source_claim
python3 scripts/run_exact_test.py --test trash_receipt --name restore_mirror_mismatch_is_inspect_only_and_never_cleans_payload
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_crash_after_each_state_is_reconcilable --case-matrix restore_durable_edge_v1 "${restore_durable_case_args[@]}"
python3 scripts/run_exact_test.py --test trash_receipt --name restore_two_process_race_has_one_mutating_winner --serial
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_bundle_terminal_remove_failure_retains_exact_monotonic_ownership --case-matrix restore_terminal_remove_v1 "${restore_terminal_case_args[@]}"
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_prepared_abort_preserves_observed_bundle_and_consumes_fixed_typestate
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_executor_panic_reclaims_exact_fixed_ref_and_reconciles --case-matrix restore_durable_edge_v1 "${restore_durable_case_args[@]}"
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_restart_resumes_terminal_remove_after_each_monotonic_phase --serial --case-matrix restore_terminal_remove_v1 "${restore_terminal_case_args[@]}"
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_terminal_facts_are_revalidated_immediately_before_fixed_unlink
python3 scripts/run_exact_test.py --test trash_receipt --name restore_rejects_replaced_observed_work_root_and_location_parent
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_authorizing_facts_cannot_outlive_claim_or_locked_receipt_snapshot
cargo test --locked --test trash_receipt
```

Expected: PASS; every restore crash state is classifiable, same-filesystem non-empty directories round-trip end to end, cross-filesystem directory restore never exposes a partial tree or loses the trash payload, and neither conflict nor identity drift overwrites a destination.

- [ ] **Step 5: Commit restore engine**

```bash
git add src/recovery.rs src/trash.rs src/state_root.rs src/mutation.rs src/mutation_ops.rs tests/trash_receipt.rs
git commit -m "feat: add no-clobber trash restore engine"
```

### Task 6: CLI list and restore

**Files:**
- Create: `src/cli.rs`
- Modify: `src/main.rs:5-49`
- Modify: `src/lib.rs`
- Modify: `tests/cli.rs`
- Create: `tests/recovery_cli.rs`

- [ ] **Step 1: Write failing CLI contract tests**

Add exact tests `trash_list_prints_stable_id_state_and_escaped_path`, `trash_list_paginates_duplicate_locations_without_gap_or_repeat`, `trash_restore_uses_original_parent_by_default`, `trash_restore_to_accepts_existing_trusted_directory`, `trash_restore_conflict_exits_nonzero_without_overwrite`, `trash_restore_duplicate_item_id_exits_nonzero_without_mutation`, `trash_restore_corrupt_receipt_exits_nonzero_without_mutation`, and `trash_usage_errors_exit_64`. Invoke the built binary and assert stdout/stderr separately. The duplicate-ID fixture places the same ID in `items` and `claims`, snapshots both directory identities and payloads, and asserts exit 1, bounded escaped ambiguity diagnostics naming both locations, empty stdout, and no mutation.

- [ ] **Step 2: Run CLI tests and confirm RED**

Run each target exactly:

```bash
python3 scripts/run_exact_test.py --test recovery_cli --name trash_list_prints_stable_id_state_and_escaped_path
python3 scripts/run_exact_test.py --test recovery_cli --name trash_list_paginates_duplicate_locations_without_gap_or_repeat
python3 scripts/run_exact_test.py --test recovery_cli --name trash_restore_uses_original_parent_by_default
python3 scripts/run_exact_test.py --test recovery_cli --name trash_restore_to_accepts_existing_trusted_directory
python3 scripts/run_exact_test.py --test recovery_cli --name trash_restore_conflict_exits_nonzero_without_overwrite
python3 scripts/run_exact_test.py --test recovery_cli --name trash_restore_duplicate_item_id_exits_nonzero_without_mutation
python3 scripts/run_exact_test.py --test recovery_cli --name trash_restore_corrupt_receipt_exits_nonzero_without_mutation
python3 scripts/run_exact_test.py --test cli --name trash_usage_errors_exit_64
cargo test --locked --test recovery_cli --test cli
```

Expected: FAIL because the `trash` subcommand is not recognized.

- [ ] **Step 3: Add the exact CLI shape and thin dispatch**

Define in `src/cli.rs`:

```rust
#[derive(clap::Subcommand)]
pub enum Command {
    Trash { #[command(subcommand)] command: TrashCommand },
}

#[derive(clap::Subcommand)]
pub enum TrashCommand {
    List,
    Restore { id: String, #[arg(long)] to: Option<PathBuf> },
}

pub fn run(command: Command) -> anyhow::Result<i32>;
```

Keep `src/main.rs` responsible only for parsing, dispatch, and the established local exit map. Both commands use `std::env::current_dir()` as the work root; this plan deliberately does not add `--root` because the design locks the CLI to the three forms above. `list` streams deterministic `RecoveryService::list_page` results without retaining more than 200 records and advances only with the returned full `CatalogKey`. `restore` parses the 32-hex `ItemId`, captures the optional destination, calls `RestoreIntent::capture_cli(&current_dir, RestoreRequest { selector: RestoreSelector::UniqueItem(id), to })`, then submits `MutationIntent::restore(submission_id, intent)` to the same Plan 2 `MutationWorker` used by Workbench; only this CLI shape performs unique lookup. The constructor records only the submission-time no-follow root identity used to reject replacement; all final filesystem preflight stays on the worker. The headless CLI coordinator receives the non-droppable prepared plan, installs its conservative fence set, returns `FenceInstalled`, then drains the single final report before shutdown; it does not call `RecoveryService::execute_restore` on the CLI thread and does not add another worker or queue. A duplicate ID in any combination of staging/items/claims/quarantine is an operational ambiguity: print bounded escaped locations to stderr, exit 1, and do not choose a first match. Print escaped display paths only; never round-trip them into a filesystem operation. Usage errors return 64, operational/recovery failures return 1, and successful list/restore returns 0.

- [ ] **Step 4: Run CLI and engine tests**

Run:

```bash
python3 scripts/run_exact_test.py --test recovery_cli --name trash_list_prints_stable_id_state_and_escaped_path
python3 scripts/run_exact_test.py --test recovery_cli --name trash_list_paginates_duplicate_locations_without_gap_or_repeat
python3 scripts/run_exact_test.py --test recovery_cli --name trash_restore_uses_original_parent_by_default
python3 scripts/run_exact_test.py --test recovery_cli --name trash_restore_to_accepts_existing_trusted_directory
python3 scripts/run_exact_test.py --test recovery_cli --name trash_restore_conflict_exits_nonzero_without_overwrite
python3 scripts/run_exact_test.py --test recovery_cli --name trash_restore_duplicate_item_id_exits_nonzero_without_mutation
python3 scripts/run_exact_test.py --test recovery_cli --name trash_restore_corrupt_receipt_exits_nonzero_without_mutation
python3 scripts/run_exact_test.py --test cli --name trash_usage_errors_exit_64
cargo test --locked --test recovery_cli --test cli
cargo test --locked --test trash_receipt
```

Expected: PASS; CLI exit codes and output streams match the assertions, and the engine matrix remains green.

- [ ] **Step 5: Commit the CLI rescue surface**

```bash
git add src/cli.rs src/main.rs src/lib.rs tests/cli.rs tests/recovery_cli.rs
git commit -m "feat: expose trash recovery CLI"
```

### Task 7: Freeze the Plan 3 component candidate

**Files:**
- Modify: `README.md`
- Modify: `CHANGELOG.md`
- Modify: `src/recovery.rs`
- Modify: `src/trash.rs`
- Modify: `tests/trash_receipt.rs`
- Modify: `tests/recovery_cli.rs`

- [ ] **Step 1: Add the missing fault-evidence cases before documentation**

Add exact parameterized tests `trash_fault_matrix_preserves_unique_source`, `restore_fault_matrix_never_clobbers_destination`, `directory_trash_restore_fault_matrix_preserves_one_complete_tree`, `catalog_duplicate_location_matrix_never_enables_restore`, and `damaged_bundle_matrix_never_deletes_content`. Freeze these ordered tables and use the strings verbatim in both the test constants and runner commands:

- `TRASH_FAULT_CASES: [&str; 24]` / matrix ID `trash_fault_matrix_v1`: `fault.enospc`, `fault.eacces`, `fault.file_sync`, `fault.directory_sync`, `fault.bundle_id_collision`, `fault.parent_replacement`, `fault.identity_drift_before_claim`, `fault.identity_drift_before_cleanup`, followed by `.before` then `.after` for each durable state ID `prepared`, `payload_ready`, `payload_published`, `source_removal_pending`, `committed`, `cleanup_required`, `indeterminate`, and `quarantined`.
- `RESTORE_FAULT_CASES: [&str; 23]` / matrix ID `restore_fault_matrix_v1`: `fault.enospc`, `fault.eacces`, `fault.file_sync`, `fault.directory_sync`, `fault.bundle_id_collision`, `fault.destination_collision`, `fault.parent_replacement`, `fault.identity_drift_before_claim`, `fault.identity_drift_before_cleanup`, followed by `.before` then `.after` for each durable state ID `restore_claimed`, `restore_publish_intent`, `restore_destination_published`, `restore_payload_removal_pending`, `restored`, `cleanup_required`, and `indeterminate`.
- `DIRECTORY_FAULT_CASES: [&str; 12]` / matrix ID `directory_fault_matrix_v1`: `same_fs_trash.before_claim`, `same_fs_trash.after_claim`, `same_fs_trash.after_bundle_publish`, `same_fs_restore.before_publish`, `same_fs_restore.after_publish`, `same_fs_restore.before_bundle_cleanup`, `cross_fs_restore_staged.before_stage_sync`, `cross_fs_restore_staged.after_stage_sync`, `cross_fs_restore_staged.after_destination_publish`, `cross_fs_trash_unsupported.before_validation`, `cross_fs_trash_unsupported.after_validation`, and `cross_fs_trash_unsupported.reservation_snapshot`.
- `CATALOG_DUPLICATE_CASES: [&str; 6]` / matrix ID `catalog_duplicate_pairs_v1`: `staging_items`, `staging_claims`, `staging_quarantine`, `items_claims`, `items_quarantine`, and `claims_quarantine`.
- `DAMAGED_BUNDLE_CASES: [&str; 10]` / matrix ID `damaged_bundle_matrix_v1`: `truncated_json`, `unknown_schema`, `malformed_header`, `missing_receipt`, `missing_payload`, `unexpected_child`, `symlinked_receipt`, `parent_replacement`, `identity_drift_before_claim`, and `identity_drift_before_cleanup`.

Place all five matrices in `recovery::tests` and run them with `--lib`: exact durable fault points use only the existing `#[cfg(test)] pub(crate)` sibling hooks and never expose those hooks to an integration, normal-library, or release build. Each named test asserts its exact constant length before iterating and emits each case ID exactly once for `run_exact_test.py`. The directory matrix covers supported same-filesystem trash, same-filesystem restore, supported staged cross-filesystem restore, and unsupported cross-filesystem directory trash; every case asserts one complete tree remains and zero partial destination roots are visible. Duplicate/damaged matrices never mint a restoration capability and never delete content. Keep `tests/trash_receipt.rs`/`tests/recovery_cli.rs` as public end-to-end regressions in the complete slice below.

- [ ] **Step 2: Run the G2 CLI gate, correct any RED case, then replay the exact block GREEN**

Run each newly added target first, then the complete slice. Before corrections at least one missing case must be RED. After corrections, run this same complete block again and require every exact matrix plus the slice to PASS before Step 3:

```bash
trash_fault_case_args=(
  --expect-case fault.enospc --expect-case fault.eacces --expect-case fault.file_sync --expect-case fault.directory_sync
  --expect-case fault.bundle_id_collision --expect-case fault.parent_replacement
  --expect-case fault.identity_drift_before_claim --expect-case fault.identity_drift_before_cleanup
  --expect-case prepared.before --expect-case prepared.after
  --expect-case payload_ready.before --expect-case payload_ready.after
  --expect-case payload_published.before --expect-case payload_published.after
  --expect-case source_removal_pending.before --expect-case source_removal_pending.after
  --expect-case committed.before --expect-case committed.after
  --expect-case cleanup_required.before --expect-case cleanup_required.after
  --expect-case indeterminate.before --expect-case indeterminate.after
  --expect-case quarantined.before --expect-case quarantined.after
)
restore_fault_case_args=(
  --expect-case fault.enospc --expect-case fault.eacces --expect-case fault.file_sync --expect-case fault.directory_sync
  --expect-case fault.bundle_id_collision --expect-case fault.destination_collision --expect-case fault.parent_replacement
  --expect-case fault.identity_drift_before_claim --expect-case fault.identity_drift_before_cleanup
  --expect-case restore_claimed.before --expect-case restore_claimed.after
  --expect-case restore_publish_intent.before --expect-case restore_publish_intent.after
  --expect-case restore_destination_published.before --expect-case restore_destination_published.after
  --expect-case restore_payload_removal_pending.before --expect-case restore_payload_removal_pending.after
  --expect-case restored.before --expect-case restored.after
  --expect-case cleanup_required.before --expect-case cleanup_required.after
  --expect-case indeterminate.before --expect-case indeterminate.after
)
directory_fault_case_args=(
  --expect-case same_fs_trash.before_claim --expect-case same_fs_trash.after_claim --expect-case same_fs_trash.after_bundle_publish
  --expect-case same_fs_restore.before_publish --expect-case same_fs_restore.after_publish --expect-case same_fs_restore.before_bundle_cleanup
  --expect-case cross_fs_restore_staged.before_stage_sync --expect-case cross_fs_restore_staged.after_stage_sync
  --expect-case cross_fs_restore_staged.after_destination_publish
  --expect-case cross_fs_trash_unsupported.before_validation --expect-case cross_fs_trash_unsupported.after_validation
  --expect-case cross_fs_trash_unsupported.reservation_snapshot
)
catalog_duplicate_case_args=(
  --expect-case staging_items --expect-case staging_claims --expect-case staging_quarantine
  --expect-case items_claims --expect-case items_quarantine --expect-case claims_quarantine
)
damaged_bundle_case_args=(
  --expect-case truncated_json --expect-case unknown_schema --expect-case malformed_header
  --expect-case missing_receipt --expect-case missing_payload --expect-case unexpected_child --expect-case symlinked_receipt
  --expect-case parent_replacement --expect-case identity_drift_before_claim --expect-case identity_drift_before_cleanup
)
python3 scripts/run_exact_test.py --lib --name recovery::tests::trash_fault_matrix_preserves_unique_source --case-matrix trash_fault_matrix_v1 "${trash_fault_case_args[@]}"
python3 scripts/run_exact_test.py --lib --name recovery::tests::restore_fault_matrix_never_clobbers_destination --case-matrix restore_fault_matrix_v1 "${restore_fault_case_args[@]}"
python3 scripts/run_exact_test.py --lib --name recovery::tests::directory_trash_restore_fault_matrix_preserves_one_complete_tree --case-matrix directory_fault_matrix_v1 "${directory_fault_case_args[@]}"
python3 scripts/run_exact_test.py --lib --name recovery::tests::catalog_duplicate_location_matrix_never_enables_restore --case-matrix catalog_duplicate_pairs_v1 "${catalog_duplicate_case_args[@]}"
python3 scripts/run_exact_test.py --lib --name recovery::tests::damaged_bundle_matrix_never_deletes_content --case-matrix damaged_bundle_matrix_v1 "${damaged_bundle_case_args[@]}"
cargo test --locked --test trash_receipt --test recovery_cli --test fs_ops --test cli
```

Expected before completing the matrix: at least one new parameterized case FAILS at its injected boundary. Expected after the mandatory identical replay: every listed exact test and frozen case ID PASSES, with no missing, duplicate, or extra case.

- [ ] **Step 3: Document the deliberately limited milestone**

Document exactly these claims: `tersh trash list/restore` is available; legacy `.tersh-trash` content is visible but never guessed, imported, or deleted; duplicate IDs are contradictory and never selected; restore never overwrites; damaged records are quarantined without deletion. State the object boundary exactly: same-filesystem trash/restore supports non-empty directories; cross-filesystem trash ingestion supports only regular files and symlinks; cross-filesystem restore of an already verified directory payload uses complete-tree staging. State that TUI recovery and the full “recoverable trash” product claim require Plan 4.

- [ ] **Step 4: Run repository quality gates**

Run:

```bash
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --all-targets --locked
cargo build --release --locked
```

Expected: all four commands exit 0; no test is ignored to make the matrix green.

- [ ] **Step 5: Commit the Plan 3 component candidate**

```bash
git add README.md CHANGELOG.md src/recovery.rs src/trash.rs tests/trash_receipt.rs tests/recovery_cli.rs
git commit -m "test: gate recoverable trash CLI faults"
```

## Spec-to-task map and acceptance boundary

| Design requirement | Implemented and proven by |
| --- | --- |
| G2 receipt schema, byte-safe path, catalog-only side-effect-free open, and fixed-intent-first trusted `v1` mutation root (`:851-899`, `:395-397`, `:1377-1383`) | Tasks 1-3 |
| Background preflight/reservation before immutable request and `FenceInstalled` before effects (`:1318-1357`) | Tasks 3, 5-6 |
| One owning claimed typed fixed control per item with nested `SourceClaimState`; no second control (`:1359-1383`) | Tasks 2-5 |
| `RawUnixName` fd-relative child capabilities and inspect-only invalid names (`:1385-1393`) | Tasks 1-2, 4 |
| Custom raw deserialization, absolute pre-normalization path validation, non-serde `ObservedRawName`, and consuming one-way downgrade (`:1488-1497`) | Tasks 1-2, 4-6 |
| Verifier-issued transition authorization retains the actual no-follow lock/receipt snapshot, is bundle/revision/edge-bound, and is consumed once; consuming-entry/advance errors retain owning recovery authority (`:1499-1505`) | Tasks 2-5 and exact forged/replay/lock-lifetime/failure tests |
| Trash protocol, complete same-filesystem directory claim, cross-filesystem regular/symlink boundary, and source cleanup (`:900-918`) | Task 3 |
| Streaming bounded catalog, complete identity-bearing key, opaque inspect-only reference, private verified capability, duplicate-ID contradiction, exact atomic `claim_existing`, multi-instance claim, quarantine without deletion (`:914-923`, `:1385-1393`, `:1404-1419`) | Tasks 2, 4 |
| Exact-observation TUI selector, unique-ID CLI selector, restore conflict/states, same-filesystem directory round trip, cross-filesystem complete-tree publication and cleanup (`:925-960`) | Tasks 5-6 |
| CLI list/restore (`:962-968`) | Task 6 |
| G2 fault matrix, exact-test discovery gate, duplicate locations, and directory unique-copy preservation (`:973-977`, `:1102`, `:1471-1474`) | Task 7 and every focused RED/GREEN command through `scripts/run_exact_test.py` |
| Recovery overlay (`:970-971`, `:1037-1044`) | Explicitly deferred to Plan 4 |

Tasks 1-7 produce the clean Plan 3 component candidate for `impl-05`; they cannot independently accept an implementation iteration. The candidate is eligible only after every prior Plan 1/2 gate passes, no `trash_path`/direct-rename production bypass remains, catalog open creates nothing, root/bundle creation is fixed-intent-first, inspect-only records expose no mutation name, only a private exact observed capability reaches `claim_existing`, pagination proves no gaps or repeats across every location, duplicate IDs are non-restorable, the directory unique-copy matrices pass, and every trash/restore item has one typed outer fixed control with only nested source-claim state. Only `2026-08-10-tersh-implementation-iteration-evidence.md` Task 6 may rerun the cumulative gates and same-candidate five-role closure and commit `impl-05.json`. Even then, this slice establishes only the crash-consistent trash/restore engine and CLI rescue surface; it does **not** complete G2 or authorize the Workbench Trusted Core label until Plan 4 supplies the Recovery overlay and `impl-06` closes.
