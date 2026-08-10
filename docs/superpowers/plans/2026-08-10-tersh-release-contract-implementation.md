# Tersh Release And Existing Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver Plan 1 of the Trusted Core design: truthful, reproducible G0a release/install evidence and a synchronous G0b interaction/result contract that reports every current mutation and exit without claiming that later async, durable trash, EXDEV, or cluster-refresh work already exists.

**Architecture:** This file is a catalog of independently committable component recipes, not their execution order. `2026-08-10-tersh-implementation-iteration-evidence.md` is the sole orchestration authority: it executes G0a as `impl-01` and only then G0b as `impl-02`, with separate candidate and evidence commits. The components add one concrete operation domain module, one process-outcome module, one terminal-session module, and one remote-launch protocol module; they do not add a generic executor, async runtime, database, plugin API, or durable mutation layer. Release automation is fail-closed: capability labels are emitted only after the exact native runner, immutable source, manifest, checksum, downloaded-binary READY-identity, and PTY gates pass.

**Tech Stack:** Rust 2024 with MSRV 1.88, clap, crossterm, ratatui, serde/serde_json, base64, getrandom, signal-hook, libc, GitHub Actions, POSIX shell, and Python 3 standard library for release-manifest assembly and validation.

---

## Scope And Existing Anchors

This plan implements design slices G0a and G0b only. It deliberately leaves latest-wins reads, the serial mutation worker, durable source claims, EXDEV moves, durable trash/restore, and cluster refresh queues to Plans 2–5.

Before any component recipe, complete and commit Task1 of
`docs/superpowers/plans/2026-08-10-tersh-implementation-iteration-evidence.md`.
That prerequisite owns `scripts/run_exact_test.py`, its tests, the append-only
review/finalization harness, cumulative gate catalog, and the single
`run_external_candidate.py` CI/release push bootstrap/verifier. Do not create or
redefine those files here.

The orchestration order is locked even though component numbers remain grouped
by domain: `impl-01` runs Task1, Task6a, Task7a, Task8, Task9's local/tooling
steps, and Task10a; only after its evidence-only commit may `impl-02` run
Tasks2–5, Task6b, Task7b, and Task10b. Task9 never launches an external workflow
from this file. Component GREEN means only that recipe may be committed; slice
acceptance requires the corresponding append-only implementation manifest.

Current anchors at commit `799cf08`:

- `Cargo.toml:1-23` declares development version `1.1.1`, Rust 2024, MSRV 1.88, runtime dependencies, and only `tempfile` as a dev dependency. `Cargo.lock:715-729` records the `tersh` 1.1.1 root package.
- `README.md:11`, `README.md:109-157`, and `README.md:323-365` incorrectly present the development tree as released v1.1.1 and install an unpinned Git head. The Chinese equivalents are `README.md:473-521` and `README.md:687-729`.
- `CHANGELOG.md:3-70` correctly has an Unreleased section, but line 21 needs explicit development-source wording.
- `src/main.rs:5-36` owns the current clap parser; `src/main.rs:38-49` returns `anyhow::Result`, inherits clap's exit 2 for usage errors, and has no remote-launch entry.
- `src/app.rs:34-91` defines `Mode` and `Command`; `src/app.rs:101-166` mixes app state with pending mutation state; `src/app.rs:420-431` collapses q and Q into one Boolean; `src/app.rs:1324-1624` performs copy, move, rename, trash, and delete synchronously and records only aggregate log strings; `src/app.rs:1783-1787` retains six logs; `src/app.rs:1795-1839` always returns a cwd after the TUI loop; `src/app.rs:2194-2213` restores the terminal only from `Drop`.
- `src/app.rs:1324-1333`, `src/app.rs:1381-1406`, `src/app.rs:1536-1579` discard prompt mode/input/targets on destination, conflict, rename, and goto validation failures.
- `src/ui.rs:18-65` is the workbench draw router; `src/ui.rs:370-428` is the current Inspector/log panel; `src/ui.rs:450-535` owns footer/help copy; `src/ui.rs:539-615` renders prompt modals without an attached validation error. `tests/render.rs:283-329` contains the existing 40x10 survival checks.
- `scripts/install.sh:4-50` builds the current checkout with `--locked`; `scripts/tersh-cd.sh:1-13` changes cwd after any successful `--print-cwd` stdout.
- `src/cluster.rs:453-575` owns cluster state and commands; `src/cluster.rs:909-970` owns its event loop; `src/cluster.rs:1087-1171` builds session commands; `src/cluster.rs:1189-1213` labels every returned status as closed; `src/cluster.rs:1222-1239` performs a shell lookup followed by `exec tersh`; `src/cluster.rs:1529-1590` duplicates terminal suspension/restoration.
- `tests/cli.rs:3-106`, `tests/app_keys.rs:43-135`, `tests/app_keys.rs:261-340`, `tests/app_keys.rs:450-685`, and `tests/cluster.rs:137-275` are the nearest existing contract tests. There are no PTY, signal, shell-wrapper, release-manifest, install-asset, advisory/license, or CI tests.
- The repository contains no `.github` directory, CI workflow, `deny.toml`, release schema, or manifest generator. The immutable stable tag is `v1.1.0` at commit `207d0b76aee4fa9dfcb6dd3f6aed1fc44cc6fa4c`; there is no v1.1.1 tag.

## Locked Interfaces For Later Plans

Plan 2 may depend on these names and invariants and must not redefine them:

- `src/operation.rs`: `OperationId`, `ItemId`, `OperationKind`, `ConflictPolicy`, `OperationRequest`, `ItemPlan`, `OperationEvent`, `ItemOutcome`, `NotStartedReason`, `EffectRole`, `CompletionState`, `OperationReport`, `OperationSummary`, and `ReportStore`.
- `src/process_outcome.rs`: `RunOutcome`, `InterruptSignal`, and `ExitCode`.
- `src/remote_launch.rs`: `RemoteLaunchRequest`, `ReadyFrame`, `LaunchIdentity`, `CompatibilityRegistry`, `RemoteProxySession`, and `ChildOutcome`.
- `OperationRequest` and `ItemPlan` are immutable after preflight; one item is one top-level target; descendants contribute bounded counters and at most 20 escaped error summaries.
- Item outcome is first-final-wins, every accepted item receives exactly one terminal outcome, a report receives exactly one `Finished`, and `CompletionState` is derived in the priority order in design lines 261–281.
- Preflight rejects zero targets and more than 10,000 top-level targets. `ReportStore` holds at most active plus latest completed full reports and 20 older summaries.
- Retry exposes candidate top-level items only. It never replays an `OperationRequest`; every retry recaptures identity/conflict state and creates new operation and item IDs.
- G0b adds no executor trait. Plan 2 adds the concrete serial `MutationWorker` and must preserve the model above.

## New File Responsibilities

| File | Single responsibility |
| --- | --- |
| `src/operation.rs` | In-memory G0b intent/event/outcome types, deterministic reducer, report bounds, and retry-candidate selection; no executor or persistence |
| `src/process_outcome.rs` | Workbench run intent, interrupt names, and exact local exit-code mapping |
| `src/terminal_session.rs` | Explicit enter/suspend/restore lifecycle and signal observation shared by workbench and cluster |
| `src/diagnostics.rs` | Opt-in current-session report serialization with default path/host redaction; no telemetry |
| `src/build_identity.rs` | Read-only embedded semver/source-commit/Cargo.lock/delivery/build-ID fields, keeping diagnostic build ID out of compatibility |
| `src/remote_launch.rs` | Remote request/READY codec, offline compatibility check, bounded pre-ready proxy, and child-outcome classification |
| `build.rs` | Re-derive clean-checkout commit and Cargo.lock identity, validate provenance inputs against those facts, and never embed a self-declared pair |
| `tests/support/pty.rs` | Unix PTY spawn/input/signal/timeout/termios and escape-sequence assertions reused by process and release smoke tests |
| `tests/release_contract.rs` | Repository wording, workflow, version, target-matrix, and no-overclaim contract tests |
| `tests/operation.rs` | Pure ID, bound, reducer, report-store, and retry-selection tests |
| `tests/process_outcome.rs` | End-to-end q/Q/signal/runtime/terminal/usage exit matrix |
| `tests/shell_wrapper.rs` | Sourced `tcd` commit-only cwd behavior in supported shells |
| `tests/remote_launch.rs` | READY parsing, identity binding, reserved exit classification, and replacement attacks |
| `tests/release_smoke.rs` | Native downloaded-binary version/help/first-frame/q/restoration smoke harness |
| `.github/workflows/ci.yml` | Locked ordinary quality, MSRV, advisory, and license gates |
| `.github/workflows/release.yml` | Exact Tier-1 artifact and Tier-2 source evidence, candidate assembly, re-download, and native smoke |
| `scripts/run_exact_test.py` | Shared prerequisite: list, uniquely discover, execute, and count one named Cargo test and validate optional frozen parameter-case IDs |
| `scripts/tests/test_run_exact_test.py` | Shared prerequisite: missing, duplicate, exact, ignored, serial, and parameter-case runner contracts |
| `scripts/validate_evidence_ref.py` | Validate the closed impl/hardening 01–07 evidence-ref grammar and exact candidate suffix for both workflows |
| `scripts/release_manifest.py` | Canonical descriptor/smoke/release and non-self-referential artifact-manifest generation/validation from explicit evidence |
| `scripts/record_artifact_producer.py` | Emit one canonical runtime join record from pinned upload Action outputs and GitHub job/run identity |
| `scripts/verified-build.sh` | Dirty-tree, source SHA, and lock-hash validation before an official asset/source build |

### Task 1: Make Stable And Development Version Claims Truthful

- [ ] Complete Task 1 and its focused regression gates.

**Files:**

- Create: `tests/release_contract.rs`
- Modify: `README.md:11,83-157,323-365,447-521,687-729`
- Modify: `CHANGELOG.md:3-35`
- Modify: `src/cluster.rs:1222-1229`

- [ ] **Step 1: Write the failing contract tests**

Create `tests/release_contract.rs` with these tests:

- `cargo_and_lock_versions_match_development_version`
- `stable_install_instructions_pin_v1_1_0`
- `development_source_is_not_labeled_as_a_published_release`
- `github_install_examples_are_immutable_and_locked`

The tests read files beneath `env!("CARGO_MANIFEST_DIR")`. They must assert:

- `Cargo.toml` and the `tersh` root package in `Cargo.lock` both say 1.1.1;
- English and Chinese stable install commands are exactly
  `cargo install --locked --git https://github.com/QiushanHuang/Tersh.git --tag v1.1.0 --bin tersh --force`;
- README labels v1.1.0 as latest stable and 1.1.1 as unreleased development source;
- every user-facing `cargo install --git https://github.com/QiushanHuang/Tersh.git`
  command in README and launcher diagnostics contains `--locked` and either the
  literal tag `v1.1.0` or a literal full 40-lowercase-hex `--rev`; development
  instructions use clone-and-build instead of a mutable cargo-install example.

- [ ] **Step 2: Run the RED test**

Run:

```bash
python3 scripts/run_exact_test.py --test release_contract --name cargo_and_lock_versions_match_development_version
python3 scripts/run_exact_test.py --test release_contract --name stable_install_instructions_pin_v1_1_0
python3 scripts/run_exact_test.py --test release_contract --name development_source_is_not_labeled_as_a_published_release
python3 scripts/run_exact_test.py --test release_contract --name github_install_examples_are_immutable_and_locked
```

Expected: each intended test is discovered exactly once; the documentation/install
tests FAIL because `README.md:11`, `README.md:129`,
`README.md:325`, the Chinese copies, and `src/cluster.rs:1227` claim v1.1.1 or
follow Git head.

- [ ] **Step 3: Implement the minimum truthful wording**

- Keep `Cargo.toml` and `Cargo.lock` at development version 1.1.1 and MSRV 1.88.
- Change the status badge and project-status prose to “latest stable v1.1.0; current checkout is unreleased 1.1.1 development source.”
- Restore the exact immutable stable command
  `cargo install --locked --git https://github.com/QiushanHuang/Tersh.git --tag v1.1.0 --bin tersh --force`
  in English, Chinese, and the remote missing-binary diagnostic.
- State that stable v1.1.0 requires Rust 1.85 and current development source requires Rust 1.88; do not imply that the stable tag contains Plan 1 behavior.
- Keep `CHANGELOG.md` changes beneath `Unreleased`; do not create a v1.1.1 release heading or release note.
- Label clone-and-build instructions as development-source installation, distinct from stable installation.
- Use the prerequisite exact runner unchanged; this recipe must not add a second
  test-filter or evidence helper.

- [ ] **Step 4: Run the GREEN and regression tests**

Run:

```bash
python3 scripts/run_exact_test.py --test release_contract --name cargo_and_lock_versions_match_development_version
python3 scripts/run_exact_test.py --test release_contract --name stable_install_instructions_pin_v1_1_0
python3 scripts/run_exact_test.py --test release_contract --name development_source_is_not_labeled_as_a_published_release
python3 scripts/run_exact_test.py --test release_contract --name github_install_examples_are_immutable_and_locked
cargo test --locked --test release_contract
cargo test --locked --test cli
git diff --check
```

Expected: every exact test executes once, both regression targets PASS, and
`git diff --check` reports no whitespace errors.

- [ ] **Step 5: Commit the truthful baseline**

```bash
git add README.md CHANGELOG.md src/cluster.rs tests/release_contract.rs
git commit -m "test: establish truthful release baseline"
```

Commit boundary: documentation and diagnostics become truthful without introducing any unproven artifact or platform claim.

### Task 2: Add The Pure Operation Model And Reducer

- [ ] Complete Task 2 and its focused regression gates.

**Files:**

- Create: `src/operation.rs`
- Create: `tests/operation.rs`
- Modify: `src/lib.rs:1-9`
- Modify: `Cargo.toml:11-23`
- Modify: `Cargo.lock:715-729` and dependency records generated by Cargo

- [ ] **Step 1: Write exhaustive failing model tests**

Create `tests/operation.rs` with these tests:

- `operation_and_item_ids_are_32_lowercase_hex_and_round_trip`
- `id_reservation_retries_an_injected_collision`
- `preflight_rejects_zero_targets_and_10001_targets`
- `top_level_item_progress_caps_error_summaries_at_twenty`
- `completion_state_uses_the_declared_seven_state_precedence`
- `item_outcome_is_first_final_wins_and_finished_is_emitted_once`
- `worker_loss_never_defaults_to_failed_no_effect`
- `retry_candidates_exclude_completed_cleanup_and_indeterminate_items`
- `report_store_retains_active_latest_full_and_twenty_summaries`

The precedence test must cover every branch in design lines 270–281 and mixtures
that would otherwise look successful. It emits frozen matrix
`completion-state-precedence-v1` with the ordered case IDs
`indeterminate-wins`, `cleanup-wins`, `partial-wins`,
`destination-retained-wins`, `failed-no-effect-wins`,
`cancel-before-commit-wins`, `success-only`, `warning-success`,
`empty-rejected`, `late-final-ignored`, `duplicate-final-ignored`, and
`worker-loss-indeterminate`—exactly 12 executed cases. The retry test permits
only `FailedNoEffect`, `CancelledBeforeCommit`, and retryable `NotStarted`
states; it excludes `CleanupRequired`, `PartialEffect`, `Indeterminate`, and
`DestinationCommittedSourceRetained` from a normal retry.

- [ ] **Step 2: Run the RED test**

Run:

```bash
for TERSH_TEST_NAME in \
  operation_and_item_ids_are_32_lowercase_hex_and_round_trip \
  id_reservation_retries_an_injected_collision \
  preflight_rejects_zero_targets_and_10001_targets \
  top_level_item_progress_caps_error_summaries_at_twenty \
  completion_state_uses_the_declared_seven_state_precedence \
  item_outcome_is_first_final_wins_and_finished_is_emitted_once \
  worker_loss_never_defaults_to_failed_no_effect \
  retry_candidates_exclude_completed_cleanup_and_indeterminate_items \
  report_store_retains_active_latest_full_and_twenty_summaries
do
  python3 scripts/run_exact_test.py --test operation --name "$TERSH_TEST_NAME"
done
```

Expected: FAIL to compile because `tersh::operation` and all locked types are absent.

- [ ] **Step 3: Implement the concrete model**

- Add direct dependency `getrandom = "0.3"`; use the operating-system CSPRNG for 128-bit IDs. Implement `Display`/`FromStr` as exactly 32 lowercase hexadecimal characters and retry collisions within the current store.
- Name the entry points `OperationId::generate`, `ItemId::generate`, `OperationRequest::new`, `OperationReport::apply`, `CompletionState::from_outcomes`, `OperationReport::retry_candidates`, and `ReportStore::{start,apply,finish}`. Keep the collision-injection seam private and test-only.
- Define the locked types listed above. Keep paths and identities in memory in G0b; do not create a database, ledger, generic transaction, or async abstraction.
- Define `OperationEvent::{Started, Progress, ItemOutcome, Finished}`. Progress is coalescible; item outcomes and the one finished event are not.
- Represent `PartialEffect` with bounded `EffectRole` entries (`Source`, `Destination`, `Staging`, `Receipt`, `Payload`) and verified identities, never descendant path lists.
- Store per-item processed-item/byte counters and at most 20 escaped error summaries.
- Implement `CompletionState::from_outcomes` exactly in the seven-step design order. Reject an empty report instead of inventing success.
- Implement a reducer that ignores duplicate/late final events, makes uncertainty `Indeterminate`, and maps missing active-worker outcomes to an observed terminal state rather than `FailedNoEffect` by assumption.
- Implement `ReportStore` with one active full report, one latest completed full report, and a `VecDeque` capped at 20 summaries.
- Expose retry candidates, not a replay method.

- [ ] **Step 4: Run GREEN and dependency checks**

Run:

```bash
for TERSH_TEST_NAME in \
  operation_and_item_ids_are_32_lowercase_hex_and_round_trip \
  id_reservation_retries_an_injected_collision \
  preflight_rejects_zero_targets_and_10001_targets \
  top_level_item_progress_caps_error_summaries_at_twenty \
  item_outcome_is_first_final_wins_and_finished_is_emitted_once \
  worker_loss_never_defaults_to_failed_no_effect \
  retry_candidates_exclude_completed_cleanup_and_indeterminate_items \
  report_store_retains_active_latest_full_and_twenty_summaries
do
  python3 scripts/run_exact_test.py --test operation --name "$TERSH_TEST_NAME"
done
python3 scripts/run_exact_test.py --test operation --name completion_state_uses_the_declared_seven_state_precedence --case-matrix completion-state-precedence-v1 --expect-case indeterminate-wins --expect-case cleanup-wins --expect-case partial-wins --expect-case destination-retained-wins --expect-case failed-no-effect-wins --expect-case cancel-before-commit-wins --expect-case success-only --expect-case warning-success --expect-case empty-rejected --expect-case late-final-ignored --expect-case duplicate-final-ignored --expect-case worker-loss-indeterminate
cargo test --locked --test operation
cargo test --locked --all-targets
cargo tree --locked -d
```

Expected: operation tests and the existing suite PASS; `cargo tree` completes and any duplicate versions are reviewed rather than silently treated as a failure.

- [ ] **Step 5: Commit the model contract**

```bash
git add Cargo.toml Cargo.lock src/lib.rs src/operation.rs tests/operation.rs
git commit -m "feat: define synchronous operation outcome contract"
```

Commit boundary: pure types, reduction, bounds, and retry selection compile and are tested, but existing mutations still use their old synchronous call sites.

### Task 3: Adapt Every Existing Mutation To Reports

- [ ] Complete Task 3 and its focused regression gates.

**Files:**

- Modify: `src/app.rs:101-239,301-418,1274-1624,1670-1704,1783-1787`
- Modify: `tests/app_keys.rs:261-340,450-685`
- Modify: `tests/fs_ops.rs:1-330` only where existing assertions need report verification

- [ ] **Step 1: Add failing synchronous-adapter tests**

Add these tests to `tests/app_keys.rs`:

- `copy_batch_records_one_outcome_for_each_top_level_target`
- `mixed_success_batch_reports_partial_and_keeps_only_retryable_targets`
- `trash_and_delete_reports_do_not_summarize_partial_as_success`
- `rename_report_preserves_the_captured_source_identity`
- `selection_of_10001_targets_is_rejected_before_executor_entry`
- `a_recursive_directory_is_one_item_plan_with_bounded_descendant_progress`
- `active_then_completed_reports_move_through_the_bounded_store`

Use temporary directories and current public keyboard commands. For the 10,001 test, create 10,001 zero-byte top-level files, select all, capture a before snapshot, submit copy-to, and assert the destination stays empty and the exact count is visible in the rejection report.

- [ ] **Step 2: Run the RED tests**

Run:

```bash
for TERSH_TEST_NAME in \
  copy_batch_records_one_outcome_for_each_top_level_target \
  mixed_success_batch_reports_partial_and_keeps_only_retryable_targets \
  trash_and_delete_reports_do_not_summarize_partial_as_success \
  rename_report_preserves_the_captured_source_identity \
  selection_of_10001_targets_is_rejected_before_executor_entry \
  a_recursive_directory_is_one_item_plan_with_bounded_descendant_progress \
  active_then_completed_reports_move_through_the_bounded_store
do
  python3 scripts/run_exact_test.py --test app_keys --name "$TERSH_TEST_NAME"
done
```

Expected: FAIL because `App` has no report accessors or operation preflight cap.

- [ ] **Step 3: Replace private pending-state duplicates with the model**

- Move the reusable path-identity capture represented by `BufferedPath`/`PathIdentity` at `src/app.rs:169-239` behind `ItemPlan`; do not keep a second Plan-2-visible identity type.
- Add `ReportStore` and the current retry candidate set to `App`.
- Build immutable `OperationRequest` values after prompt/conflict preflight and before calling `copy_path`, `rename_path`, `trash_path`, or `permanent_delete`.
- Make the edits at the existing `capture_operation_targets`, `start_file_operation`, `execute_file_operation`, `submit_rename`, `submit_trash`, and `submit_delete` boundaries; do not add a parallel mutation dispatcher.
- Emit `Started`, one final `ItemOutcome` per accepted top-level target, and exactly one `Finished` through the reducer while execution remains synchronous.
- Translate verified no-effect errors to `FailedNoEffect`; use `PartialEffect`, `CleanupRequired`, or `Indeterminate` whenever the current filesystem result cannot prove no effect. Never infer `FailedNoEffect` from an arbitrary `anyhow::Error`.
- Remove only `Completed`/`CompletedWithWarnings` cut targets from retry context. Preserve retryable targets, and require any retry command to construct a new preflight and new IDs.
- Reject 0 and 10,001+ target requests before any filesystem operation.
- Keep six-line logs as summaries derived from the report; do not parse logs to reconstruct state.
- Add read-only accessors for active/latest reports, summaries, and retry candidates for UI/tests.

- [ ] **Step 4: Run focused and full GREEN tests**

Run:

```bash
for TERSH_TEST_NAME in \
  copy_batch_records_one_outcome_for_each_top_level_target \
  mixed_success_batch_reports_partial_and_keeps_only_retryable_targets \
  trash_and_delete_reports_do_not_summarize_partial_as_success \
  rename_report_preserves_the_captured_source_identity \
  selection_of_10001_targets_is_rejected_before_executor_entry \
  a_recursive_directory_is_one_item_plan_with_bounded_descendant_progress \
  active_then_completed_reports_move_through_the_bounded_store
do
  python3 scripts/run_exact_test.py --test app_keys --name "$TERSH_TEST_NAME"
done
cargo test --locked --test app_keys
cargo test --locked --test fs_ops
cargo test --locked --test operation
```

Expected: all commands PASS; mixed batches expose every top-level item outcome and the 10,001 request has no destination effect.

- [ ] **Step 5: Commit the synchronous adapter**

```bash
git add src/app.rs tests/app_keys.rs tests/fs_ops.rs
git commit -m "feat: report every synchronous file operation"
```

Commit boundary: all current mutations tell item-level truth through the locked model; execution is intentionally still synchronous.

### Task 4: Preserve Modal Context On Validation Failure

- [ ] Complete Task 4 and its focused regression gates.

**Files:**

- Modify: `src/app.rs:34-91,101-132,1324-1406,1503-1579,1626-1665`
- Modify: `src/ui.rs:539-615`
- Modify: `tests/app_keys.rs:43-135,450-685`
- Modify: `tests/render.rs:110-144,155-198,200-240,320-329`

- [ ] **Step 1: Write failing mode-retention tests**

Add these tests:

- `goto_validation_failure_preserves_mode_and_input`
- `rename_validation_failure_preserves_mode_input_and_captured_target`
- `copy_to_validation_failure_preserves_mode_input_and_captured_targets`
- `move_to_validation_failure_preserves_mode_input_and_captured_targets`
- `conflict_validation_failure_preserves_mode_input_and_pending_operation`
- `modal_renders_escaped_inline_validation_error`

Each test must submit invalid input twice and prove that the mode, exact input text, and captured target identity remain unchanged until Esc/Ctrl+G explicitly cancels.

- [ ] **Step 2: Run the RED tests**

Run:

```bash
for TERSH_TEST_NAME in \
  goto_validation_failure_preserves_mode_and_input \
  rename_validation_failure_preserves_mode_input_and_captured_target \
  copy_to_validation_failure_preserves_mode_input_and_captured_targets \
  move_to_validation_failure_preserves_mode_input_and_captured_targets \
  conflict_validation_failure_preserves_mode_input_and_pending_operation
do
  python3 scripts/run_exact_test.py --test app_keys --name "$TERSH_TEST_NAME"
done
python3 scripts/run_exact_test.py --test render --name modal_renders_escaped_inline_validation_error
```

Expected: FAIL because current submit paths switch to `Mode::Normal`, clear input, drop targets, or expose validation only in the six-line log.

- [ ] **Step 3: Implement attached validation state**

- Add `validation_error: Option<String>` to `App` and clear it only on a successful edit, successful submit, or explicit cancel.
- Apply this rule inside `copy_to_destination`, `submit_conflict`, `submit_rename`, `submit_goto`, `submit`, and `cancel`; expose `App::validation_error` for rendering and tests.
- On invalid goto, rename, copy-to, move-to, or conflict input, retain current mode, input, and pending immutable preflight/captured targets.
- Render the escaped validation error inside the current modal, visually distinct without relying on color.
- Keep destructive confirmation mismatches in their mode with input and targets retained.
- Ensure `Esc` and Ctrl+G still clear the prompt state deliberately; Q/Ctrl+C follow Task6b's abort/interruption outcome.

- [ ] **Step 4: Run GREEN tests**

Run:

```bash
for TERSH_TEST_NAME in \
  goto_validation_failure_preserves_mode_and_input \
  rename_validation_failure_preserves_mode_input_and_captured_target \
  copy_to_validation_failure_preserves_mode_input_and_captured_targets \
  move_to_validation_failure_preserves_mode_input_and_captured_targets \
  conflict_validation_failure_preserves_mode_input_and_pending_operation
do
  python3 scripts/run_exact_test.py --test app_keys --name "$TERSH_TEST_NAME"
done
python3 scripts/run_exact_test.py --test render --name modal_renders_escaped_inline_validation_error
cargo test --locked --test app_keys
cargo test --locked --test render
```

Expected: both suites PASS, including all existing compact modal checks.

- [ ] **Step 5: Commit the validation contract**

```bash
git add src/app.rs src/ui.rs tests/app_keys.rs tests/render.rs
git commit -m "fix: retain prompt context after validation errors"
```

Commit boundary: modal failures no longer discard recovery context; operation execution and release automation are unchanged.

### Task 5: Show And Export Bounded Operation Truth

- [ ] Complete Task 5 and its focused regression gates.

**Files:**

- Create: `src/diagnostics.rs`
- Modify: `src/lib.rs:1-9`
- Modify: `src/app.rs:34-132,420-720,720-908,1783-1883`
- Modify: `src/ui.rs:18-65,370-535`
- Modify: `tests/app_keys.rs`
- Modify: `tests/render.rs`
- Create: `tests/diagnostics.rs`
- Modify: `README.md:256-307,620-671`

- [ ] **Step 1: Write failing report UI/export tests**

Add these tests:

- `normal_inspector_shows_latest_operation_without_hiding_files_at_80x24`
- `operation_detail_at_40x10_shows_state_back_help_quit_and_scroll_route`
- `operation_detail_scrolls_active_or_latest_full_report`
- `operation_detail_distinguishes_retry_cleanup_partial_and_indeterminate_without_color`
- `diagnostic_export_contains_active_latest_and_twenty_summaries_only`
- `diagnostic_export_redacts_paths_and_host_like_dynamic_text`
- `diagnostic_export_never_claims_cross_restart_history`

Use `TestBackend` at 40x10 and 80x24. The diagnostic tests serialize to an in-memory buffer or a caller-selected temporary file; they must assert that raw absolute paths and host/inventory strings do not appear.

- [ ] **Step 2: Run the RED tests**

Run:

```bash
for TERSH_TEST_NAME in \
  normal_inspector_shows_latest_operation_without_hiding_files_at_80x24 \
  operation_detail_at_40x10_shows_state_back_help_quit_and_scroll_route \
  operation_detail_scrolls_active_or_latest_full_report \
  operation_detail_distinguishes_retry_cleanup_partial_and_indeterminate_without_color
do
  python3 scripts/run_exact_test.py --test render --name "$TERSH_TEST_NAME"
done
for TERSH_TEST_NAME in \
  diagnostic_export_contains_active_latest_and_twenty_summaries_only \
  diagnostic_export_redacts_paths_and_host_like_dynamic_text \
  diagnostic_export_never_claims_cross_restart_history
do
  python3 scripts/run_exact_test.py --test diagnostics --name "$TERSH_TEST_NAME"
done
```

Expected: FAIL because the Inspector displays only five logs and no report/detail/export API exists.

- [ ] **Step 3: Implement the smallest report route**

- Reuse the Inspector and overlays; do not add a top-level Operation Center.
- Add `Mode::OperationDetail` and `Command::{OpenOperationDetail,OperationDetailUp,OperationDetailDown,OpenDiagnosticExport}`.
- Bind `o` in normal mode to the active/latest report overlay. Within the overlay, bind arrows/j/k/PageUp/PageDown to scroll, `e` to a caller-selected redacted export path prompt, and q/Esc/Ctrl+G to back. Q and Ctrl+C retain their process meanings.
- At 80x24+, show active or latest completion state and counts in the existing Inspector while leaving files and preview visible.
- At 40x10, show mode, primary state/error, `o`/detail route or current scroll position, back, help, q, and interrupt/abort controls.
- Render explicit words for retryable, cleanup-required, partial, and indeterminate states; color is supplementary only.
- `src/diagnostics.rs` creates an opt-in, local JSON export from the active full report, latest completed full report, and 20 summaries. It marks scope as current session only and redacts path, hostname, and inventory-derived text by default.
- Name the pure/export boundary `DiagnosticExport::from_store` and `DiagnosticExport::write_redacted`; keep file selection and validation in `App`.
- The export path prompt uses the Task 4 validation rules and never writes on invalid input.

- [ ] **Step 4: Run GREEN tests**

Run:

```bash
for TERSH_TEST_NAME in \
  normal_inspector_shows_latest_operation_without_hiding_files_at_80x24 \
  operation_detail_at_40x10_shows_state_back_help_quit_and_scroll_route \
  operation_detail_scrolls_active_or_latest_full_report \
  operation_detail_distinguishes_retry_cleanup_partial_and_indeterminate_without_color
do
  python3 scripts/run_exact_test.py --test render --name "$TERSH_TEST_NAME"
done
for TERSH_TEST_NAME in \
  diagnostic_export_contains_active_latest_and_twenty_summaries_only \
  diagnostic_export_redacts_paths_and_host_like_dynamic_text \
  diagnostic_export_never_claims_cross_restart_history
do
  python3 scripts/run_exact_test.py --test diagnostics --name "$TERSH_TEST_NAME"
done
cargo test --locked --test diagnostics
cargo test --locked --test render
cargo test --locked --test app_keys
```

Expected: all suites PASS; 40x10 retains survival controls and report detail is scrollable.

- [ ] **Step 5: Commit report visibility**

```bash
git add src/diagnostics.rs src/lib.rs src/app.rs src/ui.rs tests/app_keys.rs tests/render.rs tests/diagnostics.rs README.md
git commit -m "feat: expose bounded operation reports"
```

Commit boundary: users can inspect and explicitly export current-session truth; no persistent history or telemetry is introduced.

### Task 6a: Add The Reusable Native PTY Smoke Harness

- [ ] Complete Task 6a during `impl-01`; it contains test infrastructure only.

**Files:**

- Create: `tests/support/mod.rs`
- Create: `tests/support/pty.rs`
- Create: `tests/pty_support.rs`

- [ ] **Step 1: Write failing PTY harness contract tests**

Create exact tests `pty_harness_captures_first_frame_and_restoration_bytes`,
`pty_harness_timeout_kills_and_reaps_child`, and
`pty_harness_drains_beyond_capture_cap`. The harness opens a Unix PTY with the
existing `libc` dependency, gives the child a controlling terminal, records the
pre-launch termios, captures bounded bytes while draining excess, sends input or
POSIX signals, enforces a five-second deadline, reaps the child, and exposes
post-exit termios plus alternate-screen/cursor restoration observations.

- [ ] **Step 2: Run the exact RED gates**

```bash
for TERSH_TEST_NAME in \
  pty_harness_captures_first_frame_and_restoration_bytes \
  pty_harness_timeout_kills_and_reaps_child \
  pty_harness_drains_beyond_capture_cap
do
  python3 scripts/run_exact_test.py --test pty_support --name "$TERSH_TEST_NAME" --serial
done
```

Expected: FAIL because `tests/support/pty.rs` does not exist.

- [ ] **Step 3: Implement only the reusable test harness**

Keep all process creation, file-descriptor ownership, bounded drain, timeout,
signal, reap, termios, and escape-sequence assertions inside
`tests/support/pty.rs`. Do not change `src/main.rs`, `src/app.rs`, terminal exit
semantics, or the cluster launcher in this task.

- [ ] **Step 4: Run GREEN exactly**

```bash
for TERSH_TEST_NAME in \
  pty_harness_captures_first_frame_and_restoration_bytes \
  pty_harness_timeout_kills_and_reaps_child \
  pty_harness_drains_beyond_capture_cap
do
  python3 scripts/run_exact_test.py --test pty_support --name "$TERSH_TEST_NAME" --serial
done
cargo test --locked --test pty_support -- --test-threads=1
```

Expected: all three tests are discovered and executed once; the full PTY target
passes serially with no remaining child.

- [ ] **Step 5: Commit the PTY fixture**

```bash
git add tests/support tests/pty_support.rs
git commit -m "test: add reusable native pty harness"
```

Commit boundary: G0a release smoke can exercise a native TUI without importing
any G0b exit behavior.

### Task 6b: Implement Exact Workbench Exit, Signal, Terminal, And `tcd` Semantics

- [ ] Complete Task 6b only during `impl-02`, after `impl-01` evidence is committed.

**Files:**

- Create: `src/process_outcome.rs`
- Create: `src/terminal_session.rs`
- Create: `tests/process_outcome.rs`
- Create: `tests/shell_wrapper.rs`
- Modify: `src/lib.rs:1-9`
- Modify: `src/main.rs:1-49`
- Modify: `src/app.rs:420-431,586-720,1791-1839,2179-2250`
- Modify: `src/cluster.rs:909-970,1173-1213,1529-1590`
- Modify: `scripts/tersh-cd.sh:1-13`
- Modify: `README.md:217-234,302-307,581-598,666-671`
- Modify: `Cargo.toml:11-23`
- Modify: `Cargo.lock`

- [ ] **Step 1: Use Task6a's PTY harness and write RED tests**

Use Task6a's already committed `tests/support/pty.rs` unchanged for all terminal
observations in this component.

Create these tests in `tests/process_outcome.rs`:

- `q_commits_cwd_after_terminal_restore_and_exits_zero`
- `capital_q_aborts_without_commit_stdout_and_exits_two`
- `ctrl_c_interrupts_without_commit_stdout_and_exits_130`
- `sigint_interrupts_without_commit_stdout_and_exits_130`
- `sigterm_interrupts_without_commit_stdout_and_exits_143`
- `sighup_interrupts_without_commit_stdout_and_exits_129`
- `runtime_failure_restores_terminal_emits_no_cwd_and_exits_one`
- `terminal_restore_failure_emits_no_cwd_and_exits_one`
- `usage_error_emits_no_cwd_and_exits_64`

Run all cwd/stdout assertions with `--print-cwd`, so TUI bytes use stderr and stdout is reserved for the committed cwd.

Create these tests in `tests/shell_wrapper.rs` using a temporary fake `tersh` executable and both `/bin/sh` and `zsh` when available:

- `shell_wrapper_changes_directory_only_after_exit_zero_with_one_path`
- `shell_wrapper_preserves_cwd_and_exit_two_on_user_abort`
- `shell_wrapper_preserves_cwd_and_failure_status_on_runtime_error`
- `shell_wrapper_rejects_empty_or_multiline_commit_output`

- [ ] **Step 2: Run the RED tests serially**

Run:

```bash
for TERSH_TEST_NAME in \
  q_commits_cwd_after_terminal_restore_and_exits_zero \
  capital_q_aborts_without_commit_stdout_and_exits_two \
  ctrl_c_interrupts_without_commit_stdout_and_exits_130 \
  sigint_interrupts_without_commit_stdout_and_exits_130 \
  sigterm_interrupts_without_commit_stdout_and_exits_143 \
  sighup_interrupts_without_commit_stdout_and_exits_129 \
  runtime_failure_restores_terminal_emits_no_cwd_and_exits_one \
  terminal_restore_failure_emits_no_cwd_and_exits_one \
  usage_error_emits_no_cwd_and_exits_64
do
  python3 scripts/run_exact_test.py --test process_outcome --name "$TERSH_TEST_NAME" --serial
done
for TERSH_TEST_NAME in \
  shell_wrapper_changes_directory_only_after_exit_zero_with_one_path \
  shell_wrapper_preserves_cwd_and_exit_two_on_user_abort \
  shell_wrapper_preserves_cwd_and_failure_status_on_runtime_error \
  shell_wrapper_rejects_empty_or_multiline_commit_output
do
  python3 scripts/run_exact_test.py --test shell_wrapper --name "$TERSH_TEST_NAME" --serial
done
```

Expected: FAIL because q/Q/Ctrl+C share `should_quit`, signals have no controlled result, usage errors exit 2, and the shell helper trusts any successful stdout.

- [ ] **Step 3: Implement explicit process and terminal state**

- Define `RunOutcome::{CommitCwd(PathBuf),AbortByUser,Interrupted(InterruptSignal),Failed(String)}` and exact `ExitCode` mapping 0/2/129/130/143/1/64.
- Replace `should_quit: bool` with a pending run outcome. q commits; Q aborts; Ctrl+C is `Interrupted(SigInt)`. External HUP/INT/TERM are observed through direct dependency `signal-hook = "0.3"` and converted to the same outcome.
- Make `TerminalSession::restore` explicit and fallible. `run_tui` must finish/safely stop the current synchronous operation, restore the terminal, and only then return `CommitCwd`. `Drop` remains best-effort fallback, not proof of restoration.
- Define `TerminalSession::suspend(&mut self) -> Result<SuspendedTerminal<'_>,
  TerminalError>`. `SuspendedTerminal` has private fields, borrows the only
  `TerminalSession`, exposes `resume(self) -> Result<(), TerminalError>`, and
  restores once from `Drop` only as a fallback. While it exists, the outer
  dashboard cannot read stdin, render, or create a second suspension.
- `TerminalSession::enter` installs the one process signal broker up front through the pinned `signal-hook = "0.3"` dependency. The broker observes `SIGWINCH`, `SIGHUP`, `SIGINT`, and `SIGTERM` without a background control thread. Define crate-private `TerminalSignalEvent::{Resize,Interrupt(InterruptSignal)}` plus `TerminalSession::poll_signal_event` and the equivalent method on `SuspendedTerminal`; the borrow ensures only the active terminal owner can drain it. `SuspendedTerminal::current_size` returns the live rows/columns used after `SIGWINCH`. Suspending transfers exclusive signal/resize consumption with the guard, and `resume` returns it to the dashboard; no second signal registration, test-only sender, or outer event loop remains responsible while the proxy owns the guard.
- If render or terminal control fails, stop rendering, attempt restoration, finish observing the synchronous operation, return `Failed`, and emit no cwd.
- Move the duplicated cluster terminal suspend/restore logic to the same concrete module without introducing a generic terminal framework.
- Change `main` to parse with `Cli::try_parse`; map clap help/version to 0 and every usage/invocation error to 64. Return `std::process::ExitCode` instead of `anyhow::Result`.
- Print cwd only for `RunOutcome::CommitCwd` after successful restoration and only when `--print-cwd` is set.
- Harden `scripts/tersh-cd.sh`: capture exactly one non-empty line, preserve the child's nonzero status, reject multiline output, and call `cd` only on exit 0.
- Keep the concrete entry points `run_with_options`, `run_tui`, `TerminalSession::{enter,suspend,restore}`, and a `run_cli` function called by `main`; no generic application runtime is introduced.

- [ ] **Step 4: Run GREEN and regression tests**

Run:

```bash
for TERSH_TEST_NAME in \
  q_commits_cwd_after_terminal_restore_and_exits_zero \
  capital_q_aborts_without_commit_stdout_and_exits_two \
  ctrl_c_interrupts_without_commit_stdout_and_exits_130 \
  sigint_interrupts_without_commit_stdout_and_exits_130 \
  sigterm_interrupts_without_commit_stdout_and_exits_143 \
  sighup_interrupts_without_commit_stdout_and_exits_129 \
  runtime_failure_restores_terminal_emits_no_cwd_and_exits_one \
  terminal_restore_failure_emits_no_cwd_and_exits_one \
  usage_error_emits_no_cwd_and_exits_64
do
  python3 scripts/run_exact_test.py --test process_outcome --name "$TERSH_TEST_NAME" --serial
done
for TERSH_TEST_NAME in \
  shell_wrapper_changes_directory_only_after_exit_zero_with_one_path \
  shell_wrapper_preserves_cwd_and_exit_two_on_user_abort \
  shell_wrapper_preserves_cwd_and_failure_status_on_runtime_error \
  shell_wrapper_rejects_empty_or_multiline_commit_output
do
  python3 scripts/run_exact_test.py --test shell_wrapper --name "$TERSH_TEST_NAME" --serial
done
cargo test --locked --test process_outcome -- --test-threads=1
cargo test --locked --test shell_wrapper -- --test-threads=1
cargo test --locked --test cli
cargo test --locked --test app_keys
cargo test --locked --test cluster
```

Expected: all commands PASS; every tested exit restores termios/alternate screen, and non-commit wrapper runs leave cwd unchanged.

- [ ] **Step 5: Commit the process contract**

```bash
git add Cargo.toml Cargo.lock src/lib.rs src/process_outcome.rs src/terminal_session.rs src/main.rs src/app.rs src/cluster.rs scripts/tersh-cd.sh tests/process_outcome.rs tests/shell_wrapper.rs README.md
git commit -m "feat: make exit and shell cwd intent explicit"
```

Commit boundary: local workbench and shell wrapper outcomes are exact. Remote launcher status remains the next isolated commit.

### Task 7a: Bind Official Build Identity And READY Emission

- [ ] Complete Task 7a during `impl-01`; do not integrate the cluster proxy here.

**Files:**

- Create: `build.rs`
- Create: `src/build_identity.rs`
- Create: `src/remote_launch.rs` with build identity, request/READY codec, and same-process emitter only
- Create: `release/accepted-compatibility.json`
- Create: `tests/remote_launch.rs`
- Modify: `src/lib.rs:1-9`
- Modify: `src/main.rs:5-49` only for the hidden same-process READY entry
- Modify: `Cargo.toml:1-23` (add pinned `sha2` build dependency)
- Modify: `Cargo.lock`

- [ ] **Step 1: Write exact build-identity and READY RED tests**

Add these exact tests:

- `ready_frame_round_trips_at_or_below_512_bytes`
- `ready_frame_rejects_wrong_nonce_protocol_source_pair_and_non_ascii_fields`
- `asset_build_id_is_diagnostic_not_a_compatibility_key`
- `source_build_without_commit_and_lock_identity_cannot_emit_ready`
- `remote_request_encodes_raw_unix_workdir_bytes_without_padding`
- `same_process_ready_emitter_uses_embedded_commit_and_lock`
- `forged_identity_environment_cannot_change_embedded_commit_or_lock`
- `dirty_tracked_or_untracked_checkout_cannot_emit_ready`
- `lock_mismatch_cannot_emit_ready`
- `build_script_declares_every_identity_rerun_input`
- `linked_worktree_build_watches_actual_gitdir_head_ref_index_and_common_packed_refs`
- `linked_worktree_head_change_rebuilds_embedded_identity`
- `same_target_untracked_change_cannot_reuse_clean_embedded_identity`
- `production_compatibility_registry_has_no_runtime_or_environment_injection`
- `clean_official_build_accepts_its_embedded_identity_when_historical_manifest_is_empty`

The provenance tests build isolated clones with independent
`CARGO_TARGET_DIR`s. The linked-worktree fixture uses `git worktree add`, checks
out one loose-ref HEAD and one packed-ref HEAD, touches the per-worktree index,
and asserts Cargo sees every canonical `rerun-if-changed` path rather than the
text `.git` marker. The production-registry test source-checks that the field is
private, no public constructor accepts pairs/JSON/path/environment data, and the
only production constructor is
`CompatibilityRegistry::from_embedded_build_identity`. A private
`#[cfg(test)]` fixture constructor may accept explicit pairs in module unit tests.
The clean-build acceptance test reuses the isolated provenance-repository
fixture, commits the canonical empty historical manifest, builds with a separate
target directory, and invokes the production constructor rather than
`CompatibilityFixture`. It requires the registry to accept that binary's exact
embedded source/Cargo.lock pair. The outer test may select its fixture-child
mode, but no environment value may supply or alter an identity or accepted pair.

- [ ] **Step 2: Run every exact RED gate**

```bash
for TERSH_TEST_NAME in \
  ready_frame_round_trips_at_or_below_512_bytes \
  ready_frame_rejects_wrong_nonce_protocol_source_pair_and_non_ascii_fields \
  asset_build_id_is_diagnostic_not_a_compatibility_key \
  source_build_without_commit_and_lock_identity_cannot_emit_ready \
  remote_request_encodes_raw_unix_workdir_bytes_without_padding \
  same_process_ready_emitter_uses_embedded_commit_and_lock \
  forged_identity_environment_cannot_change_embedded_commit_or_lock \
  dirty_tracked_or_untracked_checkout_cannot_emit_ready \
  lock_mismatch_cannot_emit_ready \
  build_script_declares_every_identity_rerun_input \
  linked_worktree_build_watches_actual_gitdir_head_ref_index_and_common_packed_refs \
  linked_worktree_head_change_rebuilds_embedded_identity \
  same_target_untracked_change_cannot_reuse_clean_embedded_identity \
  production_compatibility_registry_has_no_runtime_or_environment_injection \
  clean_official_build_accepts_its_embedded_identity_when_historical_manifest_is_empty
do
  python3 scripts/run_exact_test.py --test remote_launch --name "$TERSH_TEST_NAME" --serial
done
```

Expected: FAIL because build identity, the hidden request, and the READY codec do
not exist.

- [ ] **Step 3: Implement re-derived identity and the same-process frame**

- `build.rs` invokes Git with argument vectors, resolves a normal repository or
  linked-worktree `.git` marker to the actual gitdir and common dir, resolves
  the full HEAD and any referenced loose ref, rejects tracked and untracked
  dirty state, and computes the actual `Cargo.lock` SHA-256. Missing Git, dirty
  state, or a hash mismatch leaves a standalone binary runnable but unable to
  emit READY.
- Optional `TERSH_BUILD_PROVENANCE_PATH` may supply delivery kind and diagnostic
  build ID only after its commit/lock equal the re-derived facts. Raw source,
  lock, or compatibility environment variables are never authoritative.
- `release/accepted-compatibility.json` begins as canonical schema 1 with an
  empty manifest array. Historical pairs are accepted only from a committed
  manifest path plus its verified SHA-256. Malformed or partially valid input
  yields no historical pairs.
- `CompatibilityRegistry` keeps its pair set private. Production code constructs
  it only through `from_embedded_build_identity`; no CLI, config, environment,
  public field, or serde input can inject a pair.
- `from_embedded_build_identity` always inserts the verified current embedded
  source/Cargo.lock pair before unioning verified historical pairs. An empty
  historical manifest is the normal first-release case, not an empty production
  registry; unavailable or internally inconsistent current identity still fails
  closed instead of accepting a guessed pair.
- Lock the concrete Plan 5 handoff rather than leaving constructors implicit:

```rust
pub struct BuildIdentity {
    launch_identity: Option<LaunchIdentity>,
    delivery_kind: DeliveryKind,
    diagnostic_build_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryKind { Asset, Source, Development }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchIdentity {
    source_commit: [u8; 20],
    cargo_lock_sha256: [u8; 32],
}

#[derive(Debug)]
pub struct CompatibilityError {
    kind: CompatibilityErrorKind,
    escaped_detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompatibilityErrorKind {
    IdentityUnavailable,
    EmbeddedRegistryInvalid,
    FixtureInvalid,
}

#[cfg(test)]
pub(crate) struct CompatibilityFixture {
    accepted_pairs: Vec<([u8; 20], [u8; 32])>,
}

pub struct CompatibilityRegistry {
    accepted_pairs: BTreeSet<([u8; 20], [u8; 32])>,
}

pub struct RemoteLaunchRequest {
    nonce: [u8; 16],
    workdir_raw: Vec<u8>,
}

#[derive(Debug)]
pub struct RemoteLaunchRequestError {
    kind: RemoteLaunchRequestErrorKind,
    escaped_detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteLaunchRequestErrorKind { Protocol, Nonce, Workdir }

impl BuildIdentity {
    pub fn embedded() -> Self;
    pub fn launch_identity(&self) -> Option<&LaunchIdentity>;
}

impl CompatibilityRegistry {
    pub fn from_embedded_build_identity(
        identity: &BuildIdentity,
    ) -> Result<Self, CompatibilityError>;
    pub fn accepts(&self, identity: &LaunchIdentity) -> bool;

    #[cfg(test)]
    pub(crate) fn from_fixture(
        fixture: CompatibilityFixture,
    ) -> Result<Self, CompatibilityError>;
}

#[cfg(test)]
impl CompatibilityFixture {
    pub(crate) fn from_hex_pairs(
        pairs: &[(&str, &str)],
    ) -> Result<Self, CompatibilityError>;
}

impl RemoteLaunchRequest {
    pub fn new(nonce: [u8; 16], workdir: &OsStr)
        -> Result<Self, RemoteLaunchRequestError>;
    pub fn parse(protocol: &str, nonce: &str, workdir_b64url: &str)
        -> Result<Self, RemoteLaunchRequestError>;
    pub fn nonce(&self) -> [u8; 16];
    pub fn remote_exec_command(&self) -> OsString;
}

impl CompatibilityError {
    pub fn kind(&self) -> CompatibilityErrorKind;
    pub fn escaped_detail(&self) -> &str;
}

impl RemoteLaunchRequestError {
    pub fn kind(&self) -> RemoteLaunchRequestErrorKind;
    pub fn escaped_detail(&self) -> &str;
}
```

  Both error types implement bounded `Display`/`Error`. Their kind enums are public and exhaustive; their fields and constructors stay private. `remote_exec_command` returns the exact printable-ASCII `exec tersh --remote-launch=tersh-exit-v1 --nonce=... --workdir-b64url=...` string assembled only from the validated fixed protocol, lowercase nonce, and canonical unpadded Base64. It contains no caller display/path bytes or shell metacharacters. `CompatibilityFixture` and `from_fixture` do not exist in a production build.
- Implement `RemoteLaunchRequest::parse` and `ReadyFrame::{encode,decode}`.
  Requests accept only `tersh-exit-v1`, one 32-lowercase-hex nonce, and canonical
  URL-safe base64 without padding. Emit exactly one printable-ASCII frame of at
  most 512 bytes before the same process enters the existing TUI; do not perform
  a second lookup or exec.
- Emit canonical Cargo rerun directives for provenance, `Cargo.lock`, registry,
  the actual gitdir `HEAD`, the resolved loose ref when present, the worktree's
  own `index`, and the common-dir `packed-refs`. Also watch a normal checkout's
  corresponding HEAD/ref/index/packed-refs paths. A linked worktree must never
  watch only the checkout's `.git` text file.
- In `linked_worktree_head_change_rebuilds_embedded_identity`, reuse one linked
  worktree and one `CARGO_TARGET_DIR`, build and read identity, switch to a second
  commit whose only change is documentation, build again without touching Rust,
  and require embedded commit/lock identity and READY to reflect the second
  commit. A source-check of directives alone is insufficient.
- Embed a development-worktree guard separately from compatibility identity.
  When that canonical worktree still exists, READY rechecks no-follow that HEAD,
  lock hash, and tracked/untracked cleanliness still equal the embedded facts;
  it rejects stale identity on any mismatch. Official asset/source builds must
  additionally use a newly created external `CARGO_TARGET_DIR`. The exact
  untracked test reuses one target, adds a nested untracked file without a Rust
  change, runs `cargo build` again, and requires READY rejection even if Cargo
  reused the prior binary. Missing source identity remains runnable locally but
  cannot emit READY.

- [ ] **Step 4: Run GREEN for the G0a identity component**

```bash
for TERSH_TEST_NAME in \
  ready_frame_round_trips_at_or_below_512_bytes \
  ready_frame_rejects_wrong_nonce_protocol_source_pair_and_non_ascii_fields \
  asset_build_id_is_diagnostic_not_a_compatibility_key \
  source_build_without_commit_and_lock_identity_cannot_emit_ready \
  remote_request_encodes_raw_unix_workdir_bytes_without_padding \
  same_process_ready_emitter_uses_embedded_commit_and_lock \
  forged_identity_environment_cannot_change_embedded_commit_or_lock \
  dirty_tracked_or_untracked_checkout_cannot_emit_ready \
  lock_mismatch_cannot_emit_ready \
  build_script_declares_every_identity_rerun_input \
  linked_worktree_build_watches_actual_gitdir_head_ref_index_and_common_packed_refs \
  linked_worktree_head_change_rebuilds_embedded_identity \
  same_target_untracked_change_cannot_reuse_clean_embedded_identity \
  production_compatibility_registry_has_no_runtime_or_environment_injection \
  clean_official_build_accepts_its_embedded_identity_when_historical_manifest_is_empty
do
  python3 scripts/run_exact_test.py --test remote_launch --name "$TERSH_TEST_NAME" --serial
done
cargo test --locked --test remote_launch -- --test-threads=1
cargo build --locked --release --bin tersh
```

Expected: every identity/codec test executes once, the target passes serially,
and a release binary builds without introducing cluster proxy behavior.

- [ ] **Step 5: Commit the G0a identity component**

```bash
git add Cargo.toml Cargo.lock build.rs release/accepted-compatibility.json src/lib.rs src/build_identity.rs src/remote_launch.rs src/main.rs tests/remote_launch.rs
git commit -m "feat: bind official build and ready identity"
```

Commit boundary: an official candidate can prove its embedded source/lock pair;
no cluster child classification or proxy ownership exists yet.

### Task 7b: Bind Remote Launch Identity And Classify Child Outcomes

- [ ] Complete Task 7b only during `impl-02`, after `impl-01` evidence is committed.

**Files:**

- Modify: `src/remote_launch.rs`
- Modify: `tests/remote_launch.rs`
- Create: `tests/fixtures/fake_remote_launcher.sh`
- Modify: `src/main.rs:5-49` only for exact child outcome mapping
- Modify: `src/cluster.rs:1087-1247`
- Modify: `src/cluster_ui.rs:357-423,599-691`
- Modify: `tests/cluster.rs:137-275,576-636,838-880`

- [ ] **Step 1: Write protocol and integration RED tests**

Add these exact tests, split between the public integration target and the owner module's crate-unit target as specified below:

- `ready_frame_rejects_premature_output_truncation_timeout_eof_and_513_bytes`
- `remote_command_is_one_exec_of_tersh_with_protocol_nonce_and_workdir`
- `valid_ready_classifies_zero_two_129_130_and_143_by_intent`
- `unbound_remote_codes_zero_two_127_129_130_and_143_are_launch_failed`
- `ssh_255_is_transport_failed_and_local_child_signal_is_signaled`
- `path_and_executable_replacement_after_ready_do_not_change_bound_child`
- `every_child_outcome_preserves_code_and_bounded_escaped_diagnostics`
- `pre_ready_failure_reaps_child_drains_streams_and_resumes_dashboard`
- `proxy_forwards_os_resize_and_has_one_stdin_owner`
- `post_ready_direct_child_exit_with_descendant_pipe_terminates_group_and_joins_reader`
- `post_ready_user_interrupt_terminates_reaps_joins_and_resumes_once`
- `post_ready_reader_panic_terminates_reaps_joins_and_resumes_once`
- `blocked_stdin_reader_is_woken_and_joined_on_child_exit_or_drop`
- `blocked_local_output_writer_is_woken_and_joined_on_child_exit_signal_or_drop`
- `remote_proxy_drop_is_a_bounded_cleanup_guard`
- `remote_proxy_session_owns_nonce_deadline_limits_signal_broker_and_pty_events`
- `pty_reader_drains_under_chunk_backpressure_without_losing_terminal_bytes`
- `pty_critical_ready_error_and_eof_events_are_non_droppable`
- `terminal_signal_broker_is_polled_by_run_without_a_control_thread`
- `post_ready_bytes_are_never_forwarded_before_identity_acceptance`
- `stdin_is_not_forwarded_before_ready_identity_acceptance`

Keep only `ready_frame_rejects_premature_output_truncation_timeout_eof_and_513_bytes`, `remote_command_is_one_exec_of_tersh_with_protocol_nonce_and_workdir`, `unbound_remote_codes_zero_two_127_129_130_and_143_are_launch_failed`, `ssh_255_is_transport_failed_and_local_child_signal_is_signaled`, and `every_child_outcome_preserves_code_and_bounded_escaped_diagnostics` in `tests/remote_launch.rs`. Put `valid_ready_classifies_zero_two_129_130_and_143_by_intent` and every remaining accepted-READY/proxy-lifecycle test in `src/remote_launch.rs` as `remote_launch::tests::<name>` and invoke them with `--lib`. Only that lib-test module can use `CompatibilityFixture`/`from_fixture`; the integration dependency is deliberately built without `cfg(test)` and a dirty implementation checkout has no official launch identity. The unit tests still spawn the real process group, PTY, readers, signal broker, and cleanup path. Do not expose the fixture, add a test feature, or weaken production identity merely to make an integration test pass.

`valid_ready_classifies_zero_two_129_130_and_143_by_intent` emits frozen matrix
`remote-bound-child-outcomes-v1` with ordered IDs `exit-0-closed`,
`exit-2-user-aborted`, `exit-129-remote-interrupted`,
`exit-130-remote-interrupted`, and `exit-143-remote-interrupted`, exactly five
executed cases.

`blocked_stdin_reader_is_woken_and_joined_on_child_exit_or_drop` keeps the
pseudo-terminal input side open without writing a byte, exercises both ordinary
child exit and an unconsumed-session `Drop`, and requires each path to wake and
join the stdin reader, reap the child group, and restore the dashboard within
700 ms. A timeout is a test failure, not an accepted detached cleanup.
`blocked_local_output_writer_is_woken_and_joined_on_child_exit_signal_or_drop`
fills and freezes a nonblocking local-terminal output sink after READY, then
exercises ordinary child exit, a real local interrupt, and an unconsumed-session
`Drop`. Each path must wake the PTY reader out of output-writability polling,
discard only bytes that cannot be delivered during cleanup, join it, reap the
child group, and restore the dashboard within the same 700 ms bound; Ctrl-S or a
saturated output sink can never turn terminal restoration into an unbounded
write.

`post_ready_bytes_are_never_forwarded_before_identity_acceptance` injects a
READY frame and sentinel bytes in the same PTY read and covers compatible,
incompatible, malformed, validation-timeout, reader-panic, and unconsumed-`Drop`
paths. Only the compatible accepted path may expose the sentinel. The paired
stdin test writes a sentinel keystroke before the decision and proves the remote
PTY receives it only after Accept; Reject, timeout, panic, and Drop receive none
and join both readers within the same 700 ms bound.

The three signal-broker tests run the proxy entry in an isolated helper process with a real controlling PTY. The harness changes that PTY through `TIOCSWINSZ`, sends the process `SIGWINCH`, and observes the remote PTY's resulting size; it then sends real HUP/INT/TERM cases. Each case must retain `local_interrupt`, complete child reap/readers join/terminal restoration, persist the attempt, exit the outer process with 129/130/143, and keep stdout empty. A separate protocol-bound fixture exits the same three numeric codes without a local signal and proves `local_interrupt = None` while `ChildOutcome::RemoteInterrupted` remains visible. The tests never construct a channel sender or call a test-only control hook, proving the synchronous production `run` path remains controllable while the outer dashboard loop is suspended without conflating local and remote intent.

Update cluster tests to replace current assertions for `command -v tersh`/`sh -c` with the single-process protocol and to assert visible `Closed`, `UserAborted`, `RemoteInterrupted`, `LaunchFailed`, `TransportFailed`, and `Signaled` labels.
Retain every Task7a codec, provenance, and private-registry test unchanged.

- [ ] **Step 2: Run the RED tests**

Run:

```bash
for TERSH_TEST_NAME in \
  ready_frame_rejects_premature_output_truncation_timeout_eof_and_513_bytes \
  remote_command_is_one_exec_of_tersh_with_protocol_nonce_and_workdir \
  unbound_remote_codes_zero_two_127_129_130_and_143_are_launch_failed \
  ssh_255_is_transport_failed_and_local_child_signal_is_signaled \
  every_child_outcome_preserves_code_and_bounded_escaped_diagnostics
do
  python3 scripts/run_exact_test.py --test remote_launch --name "$TERSH_TEST_NAME" --serial
done
for TERSH_TEST_NAME in \
  path_and_executable_replacement_after_ready_do_not_change_bound_child \
  pre_ready_failure_reaps_child_drains_streams_and_resumes_dashboard \
  proxy_forwards_os_resize_and_has_one_stdin_owner \
  post_ready_direct_child_exit_with_descendant_pipe_terminates_group_and_joins_reader \
  post_ready_user_interrupt_terminates_reaps_joins_and_resumes_once \
  post_ready_reader_panic_terminates_reaps_joins_and_resumes_once \
  blocked_stdin_reader_is_woken_and_joined_on_child_exit_or_drop \
  blocked_local_output_writer_is_woken_and_joined_on_child_exit_signal_or_drop \
  remote_proxy_drop_is_a_bounded_cleanup_guard \
  remote_proxy_session_owns_nonce_deadline_limits_signal_broker_and_pty_events \
  pty_reader_drains_under_chunk_backpressure_without_losing_terminal_bytes \
  pty_critical_ready_error_and_eof_events_are_non_droppable \
  terminal_signal_broker_is_polled_by_run_without_a_control_thread \
  post_ready_bytes_are_never_forwarded_before_identity_acceptance \
  stdin_is_not_forwarded_before_ready_identity_acceptance
do
  python3 scripts/run_exact_test.py --lib --name "remote_launch::tests::$TERSH_TEST_NAME" --serial
done
python3 scripts/run_exact_test.py --lib --name remote_launch::tests::valid_ready_classifies_zero_two_129_130_and_143_by_intent --serial --case-matrix remote-bound-child-outcomes-v1 --expect-case exit-0-closed --expect-case exit-2-user-aborted --expect-case exit-129-remote-interrupted --expect-case exit-130-remote-interrupted --expect-case exit-143-remote-interrupted
python3 scripts/run_exact_test.py --test cluster --name host_workbench_command_opens_local_or_remote_tersh
```

Expected: the new proxy/integration tests FAIL because Task7a contains only the
already-exported codec, build identity, and hidden same-process READY entry;
the cluster launcher still performs a shell lookup/exec without READY
validation or owned proxy cleanup.

- [ ] **Step 3: Implement the single-owner proxy and child classification**

- Reuse Task7a's `ReadyFrame`, `RemoteLaunchRequest`, embedded identity, and
  private production registry unchanged. Name the new boundaries
  `RemoteProxySession::{spawn,run}`, `proxy_until_ready`, and `classify_child`;
  adapt `host_workbench_command`, `ssh_workbench_args`,
  `open_selected_workbench`, and `run_session_command` rather than creating a
  second launcher.
- Build the remote SSH command as `exec tersh --remote-launch=tersh-exit-v1 --nonce=... --workdir-b64url=...` with no login shell and no second executable lookup.
- Buffer at most 512 pre-ready bytes for five seconds, consume a valid frame, validate exact protocol and source-commit/Cargo.lock pair against `CompatibilityRegistry`, then proxy subsequent terminal bytes.
- Keep the outer cluster loop as the sole `TerminalSession` owner. Task6b's
  `TerminalSession::suspend(&mut self) -> Result<SuspendedTerminal<'_>,
  TerminalError>` returns the only transferable terminal-I/O guard.
- Define the concrete API:

```rust
pub struct RemoteProxySpec {
    program: OsString,
    args: Vec<OsString>,
    expected_nonce: [u8; 16],
    ready_timeout: Duration,
    diagnostic_limit: usize,
}

pub enum PtyEvent {
    Ready(ReadyFrame),
    ChunkObserved { bytes: u64, sha256: [u8; 32], escaped_tail: String },
    Error { kind: io::ErrorKind, escaped_tail: String },
    Eof { bytes: u64, sha256: [u8; 32], escaped_tail: String },
}

pub struct PtyEventReceiver {
    critical_rx: Receiver<PtyEvent>,
    chunk_rx: Receiver<PtyEvent>,
}

pub enum ReadyDecision { Accept, Reject }

struct ReadyGateController {
    state: Arc<AtomicU8>,
    stdin_decision_wake: OwnedFd,
    pty_decision_wake: OwnedFd,
}

pub struct RemoteProxySession<'terminal> {
    dashboard: Option<SuspendedTerminal<'terminal>>,
    child: Option<Child>,
    process_group: libc::pid_t,
    expected_nonce: [u8; 16],
    ready_deadline: Instant,
    diagnostic_limit: usize,
    pty_resize_fd: Option<OwnedFd>,
    pty_events: PtyEventReceiver,
    ready_gate: ReadyGateController,
    stdin_wake_write: Option<OwnedFd>,
    pty_wake_write: Option<OwnedFd>,
    stdin_reader: Option<JoinHandle<io::Result<()>>>,
    pty_reader: Option<JoinHandle<DrainResult>>,
    cleanup_complete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProxyTerminalOutcome {
    Resumed,
    RestoreFailed { escaped_diagnostic: String },
}

pub struct RemoteProxyCompletion {
    ready_identity: Option<LaunchIdentity>,
    child_outcome: ChildOutcome,
    escaped_diagnostic: Option<String>,
    terminal_outcome: ProxyTerminalOutcome,
    local_interrupt: Option<InterruptSignal>,
}

#[derive(Debug)]
pub struct RemoteProxyStartError {
    child_outcome: Option<ChildOutcome>,
    escaped_diagnostic: String,
    terminal_outcome: ProxyTerminalOutcome,
    local_interrupt: Option<InterruptSignal>,
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

impl std::fmt::Display for RemoteProxyStartError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl std::error::Error for RemoteProxyStartError {}

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
    pub fn run(mut self, registry: &CompatibilityRegistry) -> RemoteProxyCompletion;
}
```

  `RemoteProxySpec::for_request` copies only the validated request nonce and fixes the READY timeout/diagnostic cap at five seconds/512 bytes; callers cannot substitute either bound or inject a receiver. `RemoteProxySession` moves the nonce, absolute deadline, and diagnostic cap out of that private-field spec and is the sole child/process-group/PTY/reader owner. On each bounded event-loop turn, `run` polls `TerminalSignalEvent` through its owned `SuspendedTerminal`: `Resize` queries `current_size` and applies it through the resize-only PTY fd, while HUP/INT/TERM becomes the exact remote interrupt/containment path. There is no Plan 5 control thread, second signal registration, caller-supplied production receiver, or second control consumer.
- `spawn` creates separate private close-on-exec nonblocking decision and
  cleanup wake pipes for each reader. The sole
  stdin thread owns the stdin-read capability plus one duplicated nonblocking
  PTY writer. Before READY identity is accepted it polls only its decision and
  cleanup wake fds—never terminal input—and therefore cannot send a keystroke to
  an unbound peer. After acceptance it polls terminal input, PTY writability,
  and cleanup wake; it never performs an uninterruptible terminal read or PTY
  write. The sole PTY
  reader owns the PTY read fd and one duplicated local-terminal output fd. The
  output duplicate is close-on-exec and nonblocking; every forward write polls
  output writability and the reader's cleanup wake together, handles partial
  writes, and never enters a blocking terminal write. It parses and consumes the pre-READY frame,
  emits `PtyEvent::Ready`, and stops reading further PTY bytes until the
  session-owned gate decides. Bytes already co-read after the frame remain in
  one fixed-size read buffer and are not displayed. Only `Accept` flushes those
  bytes and begins raw terminal forwarding; `Reject`, timeout, reader panic, or
  cleanup discards them and enters terminate/drain-without-display cleanup.
- `ReadyGateController` is private, session-owned, and first-decision-wins. It
  shares one atomic `Pending|Accepted|Rejected` state with distinct stdin and PTY
  waiters and owns a nonblocking wake writer for each. `run` validates nonce,
  protocol, source commit, and Cargo.lock pair before calling
  `decide(Accept)`; every malformed/incompatible/timeout/error path calls
  `decide(Reject)`. `Drop`/cleanup changes Pending to Rejected and wakes both
  readers; after Accept it still wakes both for bounded cleanup. No reader may
  infer acceptance merely from receiving or parsing READY.
- `PtyEventReceiver` is one session-owned facade over a capacity-three critical
  channel and a capacity-one coalesced chunk channel. The PTY reader emits at
  most `Ready` plus exactly one terminal `Error` or `Eof`, so those critical
  events cannot be dropped or displaced by chunks. `ChunkObserved` is bounded,
  coalesced with cumulative bytes/hash/tail, and may replace only an older chunk
  observation; it is not the raw proxy transport. Slow UI/event consumption
  therefore cannot lose terminal bytes, stop PTY drain, or grow memory.
- The session retains `stdin_wake_write`, `pty_wake_write`, the sole event
  receiver, reader handles, and a duplicated resize-only PTY fd. The optional
  dashboard guard permits the
  single cleanup path (including `Drop`) to `take()` and consume it exactly once
  through `resume`; no other owner may restore the terminal. Cleanup writes one
  byte and closes both wake ends before joining stdin and PTY, including
  ordinary child exit and `Drop`.
  `run` is total after successful construction and always reaches one bounded cleanup path: TERM
  group, wait 500 ms, KILL if needed, reap direct child, drain/close PTY, join
  both readers, then resume/restore `SuspendedTerminal`. A restore failure is
  retained in `ProxyTerminalOutcome::RestoreFailed` beside (rather than instead
  of) the exact `ChildOutcome`; it is never rendered as a successful terminal
  restoration. The total `RemoteProxyCompletion` also retains the accepted
  `LaunchIdentity` (or `None` before compatibility acceptance) and bounded
  diagnostic. No runtime failure may short-circuit past that record. `spawn`
  returns `RemoteProxyStartError` only after its own zero-effect or owning
  startup guard has performed the same cleanup; that error retains any child
  outcome already created and the terminal restoration truth.
  `Drop` runs the same idempotent bounded cleanup when `run` unwinds or is not
  called. It never reports completion, but guarantees no owned child or reader
  is silently detached.
- The first HUP/INT/TERM drained from the transferred terminal broker is retained as `local_interrupt` before proxy containment begins. It is never inferred from a remote numeric exit code and never overwritten by later child/cleanup outcomes. The session still records the exact `ChildOutcome`, completes group termination/reap, reader drain/join, and terminal restoration, then returns both truths. A signal already pending when `spawn` takes the guard follows the same rule and is returned by `RemoteProxyStartError` after zero-effect/owning cleanup. Conversely, a protocol-bound remote child that independently exits 129/130/143 sets only `ChildOutcome::RemoteInterrupted` and leaves `local_interrupt = None`. The caller must persist the complete launch attempt first, then translate a present local interrupt back to the outer `RunOutcome::Interrupted`/129/130/143 path with empty stdout.
- Implement `ChildOutcome::{Closed,UserAborted,RemoteInterrupted,LaunchFailed,TransportFailed,Signaled}` exactly as design lines 532–538. No valid frame means raw remote code 0 is still launch failure; 255 remains transport failure.
- Bound and escape diagnostics before adding them to cluster logs/UI.
- Keep cluster refresh behavior unchanged. Footer/help may say only `r` refreshes all configured hosts and Enter refreshes the selected host; do not mention a future queue/readiness sweep.

- [ ] **Step 4: Run GREEN tests**

Run:

```bash
for TERSH_TEST_NAME in \
  ready_frame_rejects_premature_output_truncation_timeout_eof_and_513_bytes \
  remote_command_is_one_exec_of_tersh_with_protocol_nonce_and_workdir \
  unbound_remote_codes_zero_two_127_129_130_and_143_are_launch_failed \
  ssh_255_is_transport_failed_and_local_child_signal_is_signaled \
  every_child_outcome_preserves_code_and_bounded_escaped_diagnostics
do
  python3 scripts/run_exact_test.py --test remote_launch --name "$TERSH_TEST_NAME" --serial
done
for TERSH_TEST_NAME in \
  path_and_executable_replacement_after_ready_do_not_change_bound_child \
  pre_ready_failure_reaps_child_drains_streams_and_resumes_dashboard \
  proxy_forwards_os_resize_and_has_one_stdin_owner \
  post_ready_direct_child_exit_with_descendant_pipe_terminates_group_and_joins_reader \
  post_ready_user_interrupt_terminates_reaps_joins_and_resumes_once \
  post_ready_reader_panic_terminates_reaps_joins_and_resumes_once \
  blocked_stdin_reader_is_woken_and_joined_on_child_exit_or_drop \
  blocked_local_output_writer_is_woken_and_joined_on_child_exit_signal_or_drop \
  remote_proxy_drop_is_a_bounded_cleanup_guard \
  remote_proxy_session_owns_nonce_deadline_limits_signal_broker_and_pty_events \
  pty_reader_drains_under_chunk_backpressure_without_losing_terminal_bytes \
  pty_critical_ready_error_and_eof_events_are_non_droppable \
  terminal_signal_broker_is_polled_by_run_without_a_control_thread \
  post_ready_bytes_are_never_forwarded_before_identity_acceptance \
  stdin_is_not_forwarded_before_ready_identity_acceptance
do
  python3 scripts/run_exact_test.py --lib --name "remote_launch::tests::$TERSH_TEST_NAME" --serial
done
python3 scripts/run_exact_test.py --lib --name remote_launch::tests::valid_ready_classifies_zero_two_129_130_and_143_by_intent --serial --case-matrix remote-bound-child-outcomes-v1 --expect-case exit-0-closed --expect-case exit-2-user-aborted --expect-case exit-129-remote-interrupted --expect-case exit-130-remote-interrupted --expect-case exit-143-remote-interrupted
cargo test --locked --test remote_launch -- --test-threads=1
cargo test --locked --test cluster
cargo test --locked --test cli
```

Expected: all commands PASS; exit 127 and every unbound code render as failure, never normal close.

- [ ] **Step 5: Commit the launcher contract**

```bash
git add src/remote_launch.rs src/main.rs src/cluster.rs src/cluster_ui.rs tests/remote_launch.rs tests/fixtures/fake_remote_launcher.sh tests/cluster.rs
git commit -m "fix: bind remote workbench launch outcomes"
```

Commit boundary: launcher classification is truthful and source-bound; cluster refresh scheduling remains explicitly out of scope.

### Task 8: Establish Locked CI, MSRV, Advisory, And License Gates

- [ ] Complete Task 8 and its focused regression gates.

**Files:**

- Create: `.github/workflows/ci.yml`
- Create: `deny.toml`
- Create: `scripts/validate_evidence_ref.py`
- Modify: `tests/release_contract.rs`

- [ ] **Step 1: Add failing repository-policy tests**

Add these tests to `tests/release_contract.rs`:

- `ci_runs_locked_quality_on_msrv_and_current_stable`
- `ci_runs_release_version_help_and_startup_smoke`
- `ci_runs_advisory_and_license_policy`
- `ci_actions_are_pinned_to_full_commit_shas`
- `ci_job_ids_match_external_verifier_contract`
- `ci_evidence_push_trigger_is_restricted_and_dispatch_input_is_stable`
- `ci_evidence_ref_accepts_only_closed_impl_hardening_union`
- `declared_msrv_is_exactly_1_88`

The tests inspect `.github/workflows/ci.yml` and `deny.toml`, asserting presence of `--locked`, Rust 1.88.0, current stable, format, clippy with warnings denied, all-target tests, release build, version/help, startup smoke, and advisory/license gates. Reject floating action tags such as `@v4`.
`ci_job_ids_match_external_verifier_contract` also requires the
restricted `push` event for `codex/evidence/**`, the retained
`workflow_dispatch` event with required input spelled exactly `candidate_sha`,
and the three immutable base verifier-facing job IDs named below. It rejects a
broad push trigger and requires push jobs to bind `github.sha` plus the exact
evidence-ref grammar. Later native
EXDEV jobs may be added under their separately locked IDs; they cannot rename,
replace, or conditionally skip these three base jobs.

- [ ] **Step 2: Run the RED test**

Run:

```bash
for TERSH_TEST_NAME in \
  ci_runs_locked_quality_on_msrv_and_current_stable \
  ci_runs_release_version_help_and_startup_smoke \
  ci_runs_advisory_and_license_policy \
  ci_actions_are_pinned_to_full_commit_shas \
  ci_job_ids_match_external_verifier_contract \
  ci_evidence_push_trigger_is_restricted_and_dispatch_input_is_stable \
  ci_evidence_ref_accepts_only_closed_impl_hardening_union \
  declared_msrv_is_exactly_1_88
do
  python3 scripts/run_exact_test.py --test release_contract --name "$TERSH_TEST_NAME"
done
```

Expected: FAIL because `.github/workflows/ci.yml` and `deny.toml` do not exist.

- [ ] **Step 3: Implement the fail-closed CI policy**

- Create a PR, restricted evidence-push, and `workflow_dispatch` workflow.
  `push.branches` contains only `codex/evidence/**`; on that event the workflow
  validates `github.ref_name` against
  `^codex/evidence/(?:impl|hardening)-0[1-7]/attempt-(?:00[1-9]|0[1-9][0-9]|[1-9][0-9]{2})/[0-9a-f]{40}$`
  and requires the suffix to equal `github.sha`. It rejects `impl-00`,
  `impl-08`, `hardening-00`, `hardening-08`, any other prefix, attempt `000`,
  uppercase/short SHA, or extra path segment. Dispatch retains a required
  `candidate_sha` input of 40 lowercase hexadecimal characters. Dispatch is a
  recovery path only when the operator selects that existing evidence ref and
  its suffix, `github.sha`, and `candidate_sha` are byte-identical. Both modes
  check out and verify that exact commit; pull requests remain ordinary
  non-acceptance quality runs. Lock these exact job IDs:
  `quality-stable`, `msrv-1-88`, and `policy`; later verifiers reject renamed,
  missing, duplicate, skipped, cancelled, or non-success jobs. Each job's
  explicit `name:` is identical to its ID so `gh run view --json jobs` evidence
  can be joined to the source-checked ID without an alias.
- Implement the shared workflow check as
  `python3 scripts/validate_evidence_ref.py --ref "$GITHUB_REF_NAME" --candidate "$TERSH_WORKFLOW_CANDIDATE"`,
  where push sets `TERSH_WORKFLOW_CANDIDATE=$GITHUB_SHA` and dispatch sets it
  from `candidate_sha` only after requiring it to equal `$GITHUB_SHA`.
  The standard-library script uses `re.fullmatch` with the exact expression
  above, requires the candidate to be 40 lowercase hex, and compares the final
  path component byte-for-byte; it accepts no environment fallback or unknown
  option. Both CI and release call this same file rather than maintaining two
  regexes.
- The `quality-stable` job runs:
  - `cargo fmt --all --check`
  - `cargo clippy --locked --all-targets --all-features -- -D warnings`
  - `cargo test --locked --all-targets --all-features`
  - `cargo build --locked --release --bin tersh`
  - `./target/release/tersh --version`
  - `./target/release/tersh --help`
  - the native PTY startup/q restoration smoke from Task6a.
- Add independent `msrv-1-88` running Rust 1.88.0 locked check/tests without
  relying on the stable cache.
- In `policy`, install exactly `cargo-deny 0.20.2` with
  `cargo install cargo-deny --version 0.20.2 --locked` and run
  `cargo deny check advisories licenses` against committed inputs.
- In `deny.toml`, explicitly allow only licenses actually present in `cargo deny list`; keep unmaintained/yanked/vulnerability handling deny-by-default with documented exceptions containing advisory ID, reason, and expiry date. Do not add an empty wildcard exception.
- Use only these pinned third-party Actions, with their readable release in a
  comment: `actions/checkout@11d5960a326750d5838078e36cf38b85af677262`,
  `actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02`,
  `actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093`, and
  `dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c`.
  CI uses only checkout and rust-toolchain; release may additionally use the two
  artifact actions. No floating action tag is allowed.
- Cache keys include OS, toolchain, target triple, and `Cargo.lock` hash.

- [ ] **Step 4: Run local equivalents**

Run:

```bash
for TERSH_TEST_NAME in \
  ci_runs_locked_quality_on_msrv_and_current_stable \
  ci_runs_release_version_help_and_startup_smoke \
  ci_runs_advisory_and_license_policy \
  ci_actions_are_pinned_to_full_commit_shas \
  ci_job_ids_match_external_verifier_contract \
  ci_evidence_push_trigger_is_restricted_and_dispatch_input_is_stable \
  ci_evidence_ref_accepts_only_closed_impl_hardening_union \
  declared_msrv_is_exactly_1_88
do
  python3 scripts/run_exact_test.py --test release_contract --name "$TERSH_TEST_NAME"
done
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo build --locked --release --bin tersh
./target/release/tersh --version
./target/release/tersh --help
test "$(cargo deny --version 2>/dev/null)" = "cargo-deny 0.20.2" || cargo install cargo-deny --version 0.20.2 --locked --force
test "$(cargo deny --version)" = "cargo-deny 0.20.2"
cargo deny check advisories licenses
cargo +1.88.0 test --locked --all-targets --all-features
```

Expected: every command PASS. Rust 1.88.0 must be installed before rerunning;
the command installs or verifies exactly cargo-deny 0.20.2. Missing tools never
waive the gate.

- [ ] **Step 5: Commit the quality gate**

```bash
git add .github/workflows/ci.yml deny.toml scripts/validate_evidence_ref.py tests/release_contract.rs
git commit -m "ci: add locked quality and supply chain gates"
```

Commit boundary: ordinary CI and policy are enforceable independently of release asset production.

### Task 9: Produce And Re-Verify Exact Release Evidence

- [ ] Complete Task 9 and its focused regression gates.

**Files:**

- Create: `.github/workflows/release.yml`
- Create: `release/asset-descriptor.schema.json`
- Create: `release/smoke-evidence.schema.json`
- Create: `release/release-manifest.schema.json`
- Create: `scripts/release_manifest.py`
- Create: `scripts/record_artifact_producer.py`
- Create: `scripts/verified-build.sh`
- Create: `scripts/tests/test_release_manifest.py`
- Create: `tests/release_smoke.rs`
- Create: `tests/fixtures/fake_release_asset.sh`
- Modify: `tests/release_contract.rs`
- Modify: `tests/support/pty.rs`
- Modify: `README.md:109-157,473-521` only to describe available evidence without changing stable status

- [ ] **Step 1: Write failing manifest and workflow tests**

Create Python standard-library tests:

- `test_manifest_rejects_missing_required_asset_field`
- `test_manifest_rejects_unknown_delivery_tier`
- `test_manifest_rejects_bad_commit_lock_and_asset_hashes`
- `test_manifest_rejects_skipped_native_smoke_for_supported_target`
- `test_manifest_keeps_build_id_out_of_remote_compatibility_key`
- `test_manifest_round_trip_is_deterministic`
- `test_asset_descriptor_contains_no_smoke_or_support_claim`
- `test_smoke_evidence_binds_exact_descriptor_hash_and_candidate_commit`
- `test_final_manifest_rejects_missing_or_mismatched_smoke_evidence`
- `test_final_manifest_cannot_be_used_as_its_own_smoke_input`
- `test_run_evidence_rejects_wrong_head_missing_job_and_non_success`
- `test_manifest_rejects_cross_run_or_cross_attempt_descriptor_smoke_and_asset`
- `test_create_new_release_paths_reject_existing_tag_or_asset`
- `test_artifact_manifest_excludes_itself_and_rejects_unlisted_payload`
- `test_artifact_producer_record_normalizes_bare_upload_digest_and_rejects_wrong_job_run_attempt_id_name_or_digest`

Create `tests/release_smoke.rs` with ignored native-asset tests driven by
`TERSH_SMOKE_BINARY`, `TERSH_SMOKE_EXPECTED_CWD`, and
`TERSH_SMOKE_ASSET_DESCRIPTOR`:

- `downloaded_asset_reports_expected_version_and_help`
- `downloaded_asset_renders_first_frame_and_q_restores_terminal`
- `downloaded_asset_ready_identity_matches_descriptor`
- `release_smoke_harness_accepts_matching_fixture_identity`
- `release_smoke_harness_rejects_mismatched_fixture_identity`

Extend `tests/release_contract.rs` with:

- `release_matrix_names_exact_tier_one_and_tier_two_targets`
- `release_matrix_pins_compilers_images_and_os_floors`
- `release_jobs_validate_asset_descriptor_before_smoke_and_final_manifest_after`
- `release_workflow_has_explicit_write_permission_and_independent_final_verify`
- `release_actions_and_job_ids_match_external_verifier_contract`
- `release_evidence_push_trigger_is_restricted_and_dispatch_input_is_stable`
- `release_evidence_ref_accepts_only_closed_impl_hardening_union`
- `release_candidate_concurrency_never_cancels_prior_attempt`
- `release_tag_and_assets_are_create_new_and_bound_to_run_attempt`
- `release_artifact_names_producers_and_content_schemas_match_shared_verifier`
- `release_artifact_upload_steps_are_unique_pinned_and_runtime_joinable`
- `minimum_environment_checks_require_exact_macos_build_and_native_kernel`
- `unsupported_targets_are_never_labeled_supported`

`release_matrix_names_exact_tier_one_and_tier_two_targets` emits frozen matrix
`release-targets-v1` with ordered case IDs `tier1-macos-arm64`,
`tier1-linux-x86_64`, `tier2-macos-x86_64-source`, and
`tier2-linux-arm64-source`, exactly four cases.
`release_actions_and_job_ids_match_external_verifier_contract` also requires
the restricted `codex/evidence/**` push contract, the retained
`workflow_dispatch` event with required input spelled exactly `candidate_sha`,
all eight verifier-facing job IDs below, and the four exact Action commit pins
inherited from Task8. The create-new test requires run ID and run attempt in the
draft tag, every uploaded artifact name, every descriptor/smoke/manifest body,
and the independent verifier inputs.
`release_artifact_upload_steps_are_unique_pinned_and_runtime_joinable` parses
the committed workflow and requires in each of those eight exact job mappings
one and only one `upload-evidence` step using the pinned upload Action, one exact
artifact-name variable, and one immediately following recorder call wired to
that step's artifact ID/digest outputs; it rejects another marker/call anywhere
in the workflow. The Python producer-record test supplies mismatched job/run/
attempt/artifact ID/name/digest cases, accepts only the pinned Action's bare
64-lowercase-hex digest output, requires the canonical record to retain that
bare value and derive exactly one `sha256:`-prefixed REST value, and rejects an
already-prefixed, uppercase, short, or nonhex Action value. No identity field is
silently normalized.

- [ ] **Step 2: Run the RED tests**

Run:

```bash
python3 -m unittest scripts.tests.test_release_manifest -v
python3 scripts/run_exact_test.py --test release_contract --name release_matrix_names_exact_tier_one_and_tier_two_targets --case-matrix release-targets-v1 --expect-case tier1-macos-arm64 --expect-case tier1-linux-x86_64 --expect-case tier2-macos-x86_64-source --expect-case tier2-linux-arm64-source
for TERSH_TEST_NAME in \
  release_matrix_pins_compilers_images_and_os_floors \
  release_jobs_validate_asset_descriptor_before_smoke_and_final_manifest_after \
  release_workflow_has_explicit_write_permission_and_independent_final_verify \
  release_actions_and_job_ids_match_external_verifier_contract \
  release_evidence_push_trigger_is_restricted_and_dispatch_input_is_stable \
  release_evidence_ref_accepts_only_closed_impl_hardening_union \
  release_candidate_concurrency_never_cancels_prior_attempt \
  release_tag_and_assets_are_create_new_and_bound_to_run_attempt \
  release_artifact_names_producers_and_content_schemas_match_shared_verifier \
  release_artifact_upload_steps_are_unique_pinned_and_runtime_joinable \
  minimum_environment_checks_require_exact_macos_build_and_native_kernel \
  unsupported_targets_are_never_labeled_supported
do
  python3 scripts/run_exact_test.py --test release_contract --name "$TERSH_TEST_NAME"
done
python3 scripts/run_exact_test.py --test release_smoke --name release_smoke_harness_accepts_matching_fixture_identity --serial
python3 scripts/run_exact_test.py --test release_smoke --name release_smoke_harness_rejects_mismatched_fixture_identity --serial
```

Expected: every exact test is discovered once and FAILS because the schemas,
workflow, fixture, and harness do not exist. No downloaded-asset test runs
locally at RED, so the recipe never depends on a nonexistent descriptor or
commits a known failing gate.

- [ ] **Step 3: Implement deterministic manifest tooling**

- `scripts/verified-build.sh` fails on tracked or untracked dirty state,
  resolves the actual full commit, computes the actual `Cargo.lock` SHA-256,
  verifies optional expected values, writes a canonical temporary build
  provenance JSON, creates a fresh external mode-0700 `CARGO_TARGET_DIR`, exports
  only that path and `TERSH_BUILD_PROVENANCE_PATH`, and executes
  `cargo build --locked --release --bin tersh`. It copies the finished binary to
  the explicit create-new output path and removes the temporary target.
  Its closed interface is
  `scripts/verified-build.sh --expected-commit SHA --expected-lock SHA256 --output ABSENT_PATH`;
  unknown/missing arguments or an existing output fail. `build.rs` independently
  re-derives and compares those facts; the wrapper is not the trust boundary.
- `scripts/release_manifest.py` uses only Python 3 standard library and exposes
  separate `asset-descriptor`, `smoke-evidence`, `artifact-manifest`, `assemble`,
  `verify`, and `verify-run`
  subcommands. `AssetDescriptor` contains immutable source/lock/filename/
  size/hash/build facts and explicitly contains no smoke result or support
  label. Every phase also carries the numeric `run_id` and positive numeric
  `run_attempt`. `SmokeEvidence` contains those same-run fields, the exact
  descriptor SHA-256, candidate commit, runner/snapshot/image/kernel facts, test
  result, and raw evidence hashes. Only `assemble` may combine matching passed
  evidence from one run/attempt into a supported final manifest. `verify-run`
  consumes saved `gh run view --json
  databaseId,attempt,headSha,conclusion,jobs` output plus the expected candidate
  and required job names; it rejects a wrong head, missing/duplicate job,
  skipped/cancelled/non-success conclusion, malformed numeric run identity, or
  evidence/artifacts from a different run or attempt.
  `artifact-manifest` receives an absent output path plus an explicit artifact
  root, producer/schema/candidate/run identity, enumerates every payload regular
  file before publication, rejects symlinks and an existing/unlisted manifest,
  and create-new publishes `artifact-manifest.json` whose ordered file list
  excludes itself. `verify` requires the on-disk set to equal the manifest plus
  its listed payload and never asks the manifest to hash its own bytes.
- `scripts/record_artifact_producer.py` uses only Python 3 standard library and
  has the closed interface `--producer-job JOB --run-id POSITIVE --run-attempt
  POSITIVE --artifact-id POSITIVE --artifact-name NAME --artifact-digest
  HEX`, where `HEX` is exactly 64 lowercase hexadecimal characters as emitted by
  the pinned `upload-artifact` Action. It validates closed job/name/hash grammars
  and prints exactly one compact canonical line prefixed
  `tersh-artifact-producer-join-v1 `. The JSON repeats the five identity/name
  values, stores `upload_output_digest_hex: HEX`, derives
  `rest_artifact_digest: sha256:HEX`, and names schema
  `tersh-artifact-producer-join-v1`; it has
  no environment fallback, file output, free-form metadata, or unknown option.
  Workflow source wires every value explicitly from `GITHUB_JOB`,
  `GITHUB_RUN_ID`, `GITHUB_RUN_ATTEMPT`, the exact artifact-name expression, and
  the unique pinned upload step's `artifact-id`/bare `artifact-digest` outputs. This
  log record is runtime join material for the external evidence helper; it does
  not by itself make a support or producer claim.
- Final validation covers every field from design lines 592–605 and normative
  addendum lines 1443–1474. It fails if a supported target lacks matching passed
  smoke evidence, an asset does not match its descriptor, a compatibility pair
  differs from embedded binary identity, or the final manifest appears in any
  smoke input/evidence chain.
- The three schemas mirror those disjoint phases. The final schema permits only
  Tier 1 `prebuilt`, Tier 2 `source`, or `unverified`; Windows and unlisted
  targets cannot be `supported`.
- `tests/fixtures/fake_release_asset.sh` is a test-only deterministic executable
  that implements version/help and one nonce-bound READY frame. The two
  non-ignored fixture tests generate a matching or deliberately mismatched
  descriptor in `tempfile`, prove the local validator's GREEN/negative paths,
  and never label the fixture a supported asset.

- [ ] **Step 4: Implement the exact target workflow**

Create a candidate workflow with explicit `permissions: contents: write`, the
same restricted `push.branches: ["codex/evidence/**"]` grammar as Task8, and the
retained required `workflow_dispatch` input `candidate_sha`. Push mode verifies
the identical closed
`^codex/evidence/(?:impl|hardening)-0[1-7]/attempt-(?:00[1-9]|0[1-9][0-9]|[1-9][0-9]{2})/[0-9a-f]{40}$`
union, the 40-hex suffix, and `github.sha`; dispatch mode verifies its input.
Both modes invoke the already committed
`python3 scripts/validate_evidence_ref.py --ref "$GITHUB_REF_NAME" --candidate "$TERSH_WORKFLOW_CANDIDATE"`
for the ref/candidate check; release does not embed or fork the regular
expression. Push sets that variable from `github.sha`. Dispatch is a recovery
path only when the operator selects the existing evidence ref and its suffix,
`github.sha`, and validated `candidate_sha` are byte-identical; it then exports
that value as `TERSH_WORKFLOW_CANDIDATE`. Set
candidate-scoped concurrency to
`group: tersh-release-candidate-${{ github.sha }}` and
`cancel-in-progress: false`, so a later attempt cannot erase the run whose
evidence is under review. Verify and check out that exact 40-hex commit. Use only
the four pinned Actions from Task8. The fail-closed
phases are: build each candidate; emit canonical asset descriptors; create an
absent draft tag named
`tersh-candidate-${{ github.sha }}-run-${{ github.run_id }}-attempt-${{ github.run_attempt }}`; upload
create-new assets/descriptors whose filenames contain the same run/attempt; run
native jobs against re-downloaded draft assets or immutable source; emit smoke
evidence; assemble the final manifest; attach it; then run an independent final
job that re-downloads the manifest and assets from only that numeric run and
revalidates all hashes. Existing tags, artifact names, draft assets, descriptor
paths, smoke paths, or manifest paths fail; neither `--clobber` nor an artifact
overwrite option is permitted.

Each of the eight verifier-facing jobs uploads exactly one nonempty GitHub
artifact. Its exact basename, producer, and root content schema are:

| Producer job | Exact artifact template | `artifact-manifest.json.schema` |
| --- | --- | --- |
| `tier1-macos-arm64` | `tier1-macos-arm64-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-tier1-release-evidence-v1` |
| `tier1-linux-x86_64` | `tier1-linux-x86_64-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-tier1-release-evidence-v1` |
| `tier2-macos-x86_64-source` | `tier2-macos-x86_64-source-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-tier2-source-evidence-v1` |
| `tier2-linux-arm64-source` | `tier2-linux-arm64-source-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-tier2-source-evidence-v1` |
| `install-msrv-1-88` | `install-msrv-1-88-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-install-evidence-v1` |
| `install-current-stable` | `install-current-stable-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-install-evidence-v1` |
| `assemble-manifest` | `release-manifest-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-release-manifest-evidence-v1` |
| `verify-release-candidate` | `verified-release-candidate-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-release-verification-evidence-v1` |

The root `artifact-manifest.json` repeats its self-declared producer job, full
candidate, numeric run ID/attempt, and every other payload regular file's
size/SHA-256 in canonical relative-path order. It excludes itself, so no
self-referential hash is required. A downloaded artifact is valid only when its
regular-file set is exactly that listed payload union plus the one root
manifest, with no symlink or unlisted file. The external helper's separate
`artifact-index.json` records the root manifest's own size/SHA-256, GitHub
artifact ID/name/digest, and each validated payload entry. Workflow contract
tests reject a missing, empty, renamed, duplicate, wrong-producer/run/attempt/
schema, self-listed manifest, unlisted payload, or extra artifact; the shared
helper passes the same eight `--require-artifact` values and
`--reject-extra-artifacts release`.

In each producer job the exact artifact name is assigned once to
`TERSH_EVIDENCE_ARTIFACT_NAME`. Exactly one step with ID `upload-evidence` uses
the pinned `actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02`
and that name, followed immediately by exactly one producer-record step that
invokes:

```text
python3 scripts/record_artifact_producer.py --producer-job "$GITHUB_JOB" --run-id "$GITHUB_RUN_ID" --run-attempt "$GITHUB_RUN_ATTEMPT" --artifact-id "${{ steps.upload-evidence.outputs.artifact-id }}" --artifact-name "$TERSH_EVIDENCE_ARTIFACT_NAME" --artifact-digest "${{ steps.upload-evidence.outputs.artifact-digest }}"
```

The record step runs under `if: success()` and no other workflow step may invoke
the recorder or contain its schema marker. The external helper fetches the log
for the unique numeric job in `jobs.json`, hashes the complete raw bytes, and
unwraps the real platform format: an optional UTF-8 BOM only at byte zero,
followed on every line by an RFC 3339 UTC timestamp with one to nine fractional
digits, one ASCII space, and the payload. It requires exactly one canonical
producer payload and rejects a bare marker line, malformed timestamp prefix,
later BOM, or duplicate payload. The record's bare upload digest is normalized
to exactly `sha256:<same-lowercase-hex>` and must equal the artifact REST
`digest`; the helper joins its workflow job ID, numeric run ID/attempt, artifact
ID/name and both digest forms to GitHub's job and artifact REST bodies. The
manifest's producer field remains
self-declared until this join succeeds; an expected job name plus matching
manifest text is never accepted as runtime producer proof.

The workflow job IDs are exact and immutable:
`tier1-macos-arm64`, `tier1-linux-x86_64`,
`tier2-macos-x86_64-source`, `tier2-linux-arm64-source`,
`install-msrv-1-88`, `install-current-stable`, `assemble-manifest`, and
`verify-release-candidate`. Every job's explicit `name:` is identical to its ID
so external job JSON and the source workflow have one unambiguous join key. The
target jobs are:

- Tier-1 `aarch64-apple-darwin`: native runner label
  `tersh-macos-14.5-23F79-arm64`; require `sw_vers -buildVersion` to equal
  `23F79` exactly, Xcode 16.2 build 16C5032a, macOS 15.2 SDK, recorded Apple
  clang/linker versions, rustc 1.95.0, and
  `MACOSX_DEPLOYMENT_TARGET=14.5`; reject any `otool` minimum other than 14.5.
  Require and record valid `TERSH_SNAPSHOT_ID` and
  `TERSH_DISK_IMAGE_SHA256` values.
- Tier-1 `x86_64-unknown-linux-gnu`: native runner label
  `tersh-almalinux-8.10-kernel-4.18-x86_64`; require the host/VM
  `uname -r` to match `^4\.18\.` before starting containers. Build with rustc
  1.95.0 and GCC 14 inside
  `quay.io/pypa/manylinux_2_28_x86_64:2026.05.07-2@sha256:443eabd378e140996780a772e12c1a1ef10551da933fe76d74a1bab61f68a7b7`,
  with `-C target-cpu=x86-64`, and reject GLIBC symbols above 2.28. Smoke the
  re-downloaded asset in
  `docker.io/library/almalinux:8.10@sha256:f043b7ac550015e1ed0b5a55a420c61d178bff4357ab9663fe0fbdcf1e6e2d86`;
  the container's inherited kernel is accepted only because the runner already
  proved the exact 4.18.x floor.
- Tier-2 `x86_64-apple-darwin`: native Intel label `tersh-macos-14.5-23F79-x86_64`, Xcode 16.2/macOS 15.2 SDK, and two clean pinned-source builds/smokes under Rust 1.88.0 and 1.95.0. Publish no prebuilt asset.
- Tier-2 `aarch64-unknown-linux-gnu`: native runner label
  `tersh-almalinux-8.10-kernel-4.18-aarch64`; require host/VM `uname -r` to
  match `^4\.18\.` and ARMv8-A. Build under Rust 1.88.0 and 1.95.0 in
  `quay.io/pypa/manylinux_2_28_aarch64:2026.05.07-2@sha256:a435288af93def166dc59b5d052fa20ce59d76c6f38e8ad105767262d36843f0`
  and smoke in
  `docker.io/library/almalinux:8.10@sha256:058da2bf381d460db9121940fbd035190ffbf28caec923cb9ba06c6e990da274`.
  Publish no prebuilt asset.
- Each Tier-1 smoke downloads the draft candidate artifact and its
  `AssetDescriptor` into a clean directory, validates filename/size/SHA-256,
  runs version/help, invokes that downloaded binary's hidden same-process launch
  entry with a fresh nonce, parses READY, and requires its protocol,
  source-commit, and Cargo.lock hash to equal the descriptor. It then runs the
  five-second native PTY first-frame/q/termios/cursor/alternate-screen test. It outputs
  `SmokeEvidence`; it does not read the not-yet-created final manifest.
- Each Tier-2 smoke checks out the recorded immutable full SHA in a clean environment, recomputes the lock hash, builds with `scripts/verified-build.sh`, validates READY identity, and runs the identical native PTY test under both compilers.
- Separate clean install jobs run `cargo +1.88.0 install --locked --git https://github.com/QiushanHuang/Tersh.git --rev "$SOURCE_COMMIT" --root "$RUNNER_TEMP/tersh-msrv" --bin tersh` and the same command with `+1.95.0` and root `tersh-stable`, then execute each installed binary's version/help and native PTY smoke.
- Candidate assembly fails unless all four target jobs pass and every
  `SmokeEvidence` matches its descriptor and candidate commit. It attaches the
  final `release-manifest.json` to the draft candidate, then the independent
  verifier re-downloads all evidence and both Tier-1 assets. This plan does not
  publish a public release.

- [ ] **Step 5: Run local tooling GREEN tests**

Run:

```bash
python3 -m unittest scripts.tests.test_release_manifest -v
python3 scripts/run_exact_test.py --test release_contract --name release_matrix_names_exact_tier_one_and_tier_two_targets --case-matrix release-targets-v1 --expect-case tier1-macos-arm64 --expect-case tier1-linux-x86_64 --expect-case tier2-macos-x86_64-source --expect-case tier2-linux-arm64-source
for TERSH_TEST_NAME in \
  release_matrix_pins_compilers_images_and_os_floors \
  release_jobs_validate_asset_descriptor_before_smoke_and_final_manifest_after \
  release_workflow_has_explicit_write_permission_and_independent_final_verify \
  release_actions_and_job_ids_match_external_verifier_contract \
  release_evidence_push_trigger_is_restricted_and_dispatch_input_is_stable \
  release_evidence_ref_accepts_only_closed_impl_hardening_union \
  release_candidate_concurrency_never_cancels_prior_attempt \
  release_tag_and_assets_are_create_new_and_bound_to_run_attempt \
  release_artifact_names_producers_and_content_schemas_match_shared_verifier \
  release_artifact_upload_steps_are_unique_pinned_and_runtime_joinable \
  minimum_environment_checks_require_exact_macos_build_and_native_kernel \
  unsupported_targets_are_never_labeled_supported
do
  python3 scripts/run_exact_test.py --test release_contract --name "$TERSH_TEST_NAME"
done
cargo test --locked --test release_contract
cargo build --locked --release --bin tersh
python3 scripts/run_exact_test.py --test release_smoke --name release_smoke_harness_accepts_matching_fixture_identity --serial
python3 scripts/run_exact_test.py --test release_smoke --name release_smoke_harness_rejects_mismatched_fixture_identity --serial
```

Expected: all local validators and fixture tests PASS; no known RED is committed.
The three ignored `downloaded_asset_*` tests are deliberately not invoked with a
local build. Each Tier-1 workflow job invokes all three against its downloaded
candidate asset and descriptor, and the implementation-iteration verifier
requires their same-run raw successful evidence. Local fixture tests prove only
the harness; G0a target acceptance remains pending until every exact native job
passes after Task10a.

- [ ] **Step 6: Commit release tooling without running external acceptance**

```bash
git add .github/workflows/release.yml release/asset-descriptor.schema.json release/smoke-evidence.schema.json release/release-manifest.schema.json scripts/release_manifest.py scripts/record_artifact_producer.py scripts/verified-build.sh scripts/tests/test_release_manifest.py tests/release_smoke.rs tests/fixtures/fake_release_asset.sh tests/release_contract.rs tests/support/pty.rs README.md
git commit -m "ci: verify reproducible release candidates"
```

Commit boundary: deterministic tooling and exact fail-closed jobs are reviewable; public release publication remains an explicitly authorized external action.

The real external run is deliberately absent here. The implementation-iteration
plan runs it only after Task10a's G0a documentation commit has frozen a clean
`impl-01` candidate. A local smoke, a workflow from another SHA, or an external
run before Task10a is never G0a evidence.

### Task 10a: Close G0a Documentation Before The External Candidate Gate

- [ ] Complete Task10a last in `impl-01`; its commit is the G0a candidate SHA.

**Files:**

- Modify: `README.md:11,83-157,323-365,447-521,687-729`
- Modify: `CHANGELOG.md:3-35`
- Modify: `tests/release_contract.rs`

- [ ] **Step 1: Add exact G0a documentation RED tests**

Add `readme_separates_stable_v1_1_0_from_unreleased_development`,
`readme_support_labels_require_the_exact_manifest_and_native_smoke`,
`changelog_keeps_g0a_changes_under_unreleased`, and
`historical_release_notes_remain_version_scoped`. The tests reject any wording
that calls the current tree released, labels an unverified target supported, or
claims a public release was published by Task9.

- [ ] **Step 2: Run every exact RED test**

```bash
for TERSH_TEST_NAME in \
  readme_separates_stable_v1_1_0_from_unreleased_development \
  readme_support_labels_require_the_exact_manifest_and_native_smoke \
  changelog_keeps_g0a_changes_under_unreleased \
  historical_release_notes_remain_version_scoped
do
  python3 scripts/run_exact_test.py --test release_contract --name "$TERSH_TEST_NAME"
done
```

Expected: FAIL until the complete G0a component behavior and exact evidence
boundary are documented.

- [ ] **Step 3: Write only G0a user-facing truth**

Document immutable stable/source install commands, the unreleased development
status, exact Tier-1/Tier-2 scope, descriptor→smoke→manifest ordering,
downloaded-binary READY identity verification, and the fact that Task9 creates
an unverified draft candidate rather than publishing a release. Keep operation
reports, q/Q/tcd semantics, remote child labels, and Cluster behavior out of
Task10a; Task10b owns those G0b claims.

- [ ] **Step 4: Run G0a local gates**

```bash
for TERSH_TEST_NAME in \
  readme_separates_stable_v1_1_0_from_unreleased_development \
  readme_support_labels_require_the_exact_manifest_and_native_smoke \
  changelog_keeps_g0a_changes_under_unreleased \
  historical_release_notes_remain_version_scoped
do
  python3 scripts/run_exact_test.py --test release_contract --name "$TERSH_TEST_NAME"
done
cargo test --locked --test release_contract --test remote_launch -- --test-threads=1
python3 -m unittest scripts.tests.test_release_manifest -v
git diff --check
```

Expected: every exact/local G0a gate passes. No external workflow has run yet.

- [ ] **Step 5: Commit the G0a candidate boundary**

```bash
git add README.md CHANGELOG.md tests/release_contract.rs
git commit -m "docs: close g0a release candidate truth"
```

Commit boundary: this exact SHA contains every G0a source, workflow, test, and
user-facing claim. The orchestration plan now runs CI/release, five-role closure,
and the evidence-only `impl-01` commit. No G0b component may begin first.

### Task 10b: Close G0b Documentation And Integrated Local Evidence

- [ ] Complete Task10b last in `impl-02`, after Task6b and Task7b.

**Files:**

- Modify: `README.md:56-81,109-157,179-234,256-365,420-445,473-521,543-598,620-729`
- Modify: `CHANGELOG.md:3-70`
- Modify: `tests/release_contract.rs`

- [ ] **Step 1: Add final failing consistency tests**

Add these tests:

- `readme_documents_operation_reports_modal_retention_and_exact_exit_intent`
- `readme_calls_cluster_a_frozen_read_only_companion`
- `readme_does_not_claim_async_jobs_restore_exdev_or_refresh_queue`
- `changelog_keeps_g0b_changes_under_unreleased`

- [ ] **Step 2: Run the RED test**

Run:

```bash
for TERSH_TEST_NAME in \
  readme_documents_operation_reports_modal_retention_and_exact_exit_intent \
  readme_calls_cluster_a_frozen_read_only_companion \
  readme_does_not_claim_async_jobs_restore_exdev_or_refresh_queue \
  changelog_keeps_g0b_changes_under_unreleased
do
  python3 scripts/run_exact_test.py --test release_contract --name "$TERSH_TEST_NAME"
done
```

Expected: FAIL until the completed Plan 1 behavior and non-goals are documented.

- [ ] **Step 3: Update user-facing documentation without overclaiming**

- Document active/latest full reports, 20 summaries, retry-by-new-preflight, modal retention, q commit, Q abort, Ctrl+C interrupt, `tcd` commit-only cwd, and redacted local diagnostic export.
- Describe remote READY compatibility and truthful child statuses without exposing internal implementation as a product feature.
- State that cluster remains a read-only companion with current all/selected refresh only.
- Keep async jobs, durable retry, EXDEV, durable trash/restore, refresh queue/readiness, database, plugins, and automatic updates out of current capability claims.
- Keep stable v1.1.0 separate from unreleased Plan 1 development behavior.
- Add only G0b changes to `CHANGELOG.md` under `Unreleased`; do not revise the
  already committed G0a support scope or create a release heading.

- [ ] **Step 4: Run complete Plan 1 verification**

Run:

```bash
for TERSH_TEST_NAME in \
  readme_documents_operation_reports_modal_retention_and_exact_exit_intent \
  readme_calls_cluster_a_frozen_read_only_companion \
  readme_does_not_claim_async_jobs_restore_exdev_or_refresh_queue \
  changelog_keeps_g0b_changes_under_unreleased
do
  python3 scripts/run_exact_test.py --test release_contract --name "$TERSH_TEST_NAME"
done
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo build --locked --release --bin tersh
./target/release/tersh --version
./target/release/tersh --help
cargo deny check advisories licenses
cargo +1.88.0 test --locked --all-targets --all-features
python3 -m unittest scripts.tests.test_release_manifest -v
python3 scripts/run_exact_test.py --test release_smoke --name release_smoke_harness_accepts_matching_fixture_identity --serial
python3 scripts/run_exact_test.py --test release_smoke --name release_smoke_harness_rejects_mismatched_fixture_identity --serial
git diff --check
git status --short
```

Expected: every executable check PASS. `git status --short` lists only the
intended G0b documentation/test changes before commit. Slice acceptance still
belongs to `impl-02`; this component does not reuse `impl-01` external evidence.

- [ ] **Step 5: Commit Plan 1 documentation**

```bash
git add README.md CHANGELOG.md tests/release_contract.rs
git commit -m "docs: close g0b interaction and result truth"
```

Commit boundary: user-facing claims match implemented and evidenced Plan 1 behavior, with later slices explicitly unclaimed.

## Requirement-To-Task Map

| Design requirement | Implementation task | Acceptance evidence |
| --- | --- | --- |
| Stable v1.1.0 vs unreleased 1.1.1 truth; immutable source install (`§G0a`, lines 553–561, 616–617) | Tasks1, 10a | `release_contract` wording/install tests; README/Changelog review |
| Exact Tier-1/Tier-2 target, OS/ABI/compiler/image floors (lines 562–591) | Task 9 | Workflow contract tests plus real native job records |
| Release manifest fields, source/lock pair, diagnostic build-ID separation (lines 592–605) | Tasks7a, 9 | Python manifest tests, downloaded-binary READY tests, attached candidate manifest |
| Re-derived build identity and non-circular descriptor/smoke/manifest evidence (normative lines 1443–1470) | Tasks7a, 9 | forged-environment tests, phased schema tests, exact native workflow evidence |
| Evidence bootstrap binds the closed impl/hardening ref union, exact workflow/run attempt, non-self-referential artifact inventory, and source-checked/runtime-joined producer | Tasks8, 9; implementation-evidence Task1 | restricted-push, manifest/index, unique pinned upload, and job-log/artifact-ID join contract tests plus same-run verifier manifest |
| Re-download, checksum/size, version/help, native PTY, terminal restoration, source-tag install (lines 614–628) | Tasks6a, 7a, 9 | `release_smoke`, READY identity, PTY tests, real target jobs |
| Concrete operation model and ID contract (lines 202–239) | Task 2 | ID, bound, reducer, first-final-wins tests |
| Exact item outcomes and deterministic `CompletionState` (lines 240–296) | Tasks 2, 3 | exhaustive reducer and mixed-batch report tests |
| Active/latest full reports, 20 summaries, diagnostic export (lines 640–648) | Tasks 2, 3, 5 | report-store, UI, and redacted export tests |
| 10,000 top-level cap and bounded descendants (lines 649–652) | Tasks 2, 3 | 10,001 no-effect and bounded recursive-progress tests |
| Retry creates a new preflight; completed/cleanup items are not replayed (lines 653–654; UI rules 1055–1057) | Tasks 2, 3 | retry-candidate and mixed-success tests |
| Invalid goto/rename/copy-to/move-to/conflict retains mode/input/targets/error (lines 655–656, 1048–1050) | Task 4 | five modal retention tests and render test |
| Exact `RunOutcome`, exit, signal, stdout, terminal restoration, shell wrapper (lines 463–489, 657, 664–667) | Task6b | serial PTY and shell-wrapper matrix |
| Source-bound single-process `tersh-exit-v1` and child classification (lines 491–539, 672–677) | Tasks7a, 7b | codec, replacement, reserved-code, cluster render tests |
| One terminal/proxy owner with child reap, stream drain, and dashboard resume (normative lines 1429–1435) | Tasks6b, 7b | pre/post-READY lifecycle, descendant-pipe, interrupt, panic, Drop, resize, and PTY restoration tests |
| Cluster copy describes current refresh only (lines 659–660) | Tasks7b, 10b | cluster footer/help tests and README claim test |
| 40x10 summary/control/detail route and 80x24 Inspector (lines 678–679, 1022–1044) | Task 5 | TestBackend 40x10 and 80x24 tests |
| Locked format/lint/test/build/MSRV/advisory/license gates (lines 559–561, 1097, 1142–1155) | Tasks8, 9, 10a, 10b | exact job/action policy tests, local equivalents, native workflow |
| No default telemetry and redacted opt-in diagnostics (lines 108–114, 1083–1085) | Task 5 | diagnostics scope/redaction tests |
| Focused gates reject zero discovered/executed tests (normative lines 1471–1474) | Task 1 and every later task | exact-test-runner Python suite plus all named focused gates |
| No generic runtime/DB/plugin/updates/package-manager expansion (lines 180–198, 630–631) | All tasks | code review; absence of those abstractions and claims |

## Component Boundaries And Slice Authority

- Task6a and Task7a are G0a foundations; Task6b and Task7b are G0b behavior.
  Mixing either pair into one candidate violates the iteration contract.
- Task8 is ordinary repository policy; Task9 commits deterministic release
  tooling only. Neither independently accepts G0a.
- Task10a is the final `impl-01` component commit. Only the orchestration plan may
  run its exact-SHA CI/release gates, five-role closure, and evidence-only commit.
- Tasks2–5, 6b, 7b, and 10b form `impl-02`. Task10b is its candidate boundary;
  old `impl-01` external evidence cannot attest that newer SHA.
- No statement in this file independently accepts G0a, G0b, Plan1, Workbench
  Trusted Core, or the full task. Acceptance authority is exclusively
  `docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-01.json` and
  `impl-02.json`, followed by the later slice and hardening requirements.

## Final Review Checklist

- [ ] No task adds async jobs, a generic executor, durable receipts, EXDEV copy/delete, restore, a cluster refresh queue, a database, or a plugin system.
- [ ] No success state is derived from a log string or raw child exit code without its required binding.
- [ ] Every test is introduced RED before implementation and rerun GREEN with previous gates.
- [ ] Every focused test is uniquely discovered and reports one executed test through `scripts/run_exact_test.py`; Cargo's zero-test exit is never accepted.
- [ ] Every parameterized focused test emits and validates its frozen ordered case IDs and exact count.
- [ ] Every commit boundary compiles and passes the focused tests named in that task.
- [ ] README, Changelog, Cargo metadata, tags, manifest, binary identity, and capability labels describe the same version and evidence.
- [ ] Asset descriptors precede smoke, smoke evidence binds the exact candidate, and only the later final manifest carries support claims.
- [ ] Every downloaded Tier-1 binary's READY source/Cargo.lock pair equals its AssetDescriptor before support assembly.
- [ ] Linked-worktree builds watch the actual gitdir HEAD/ref/index and common packed-refs inputs, not only the `.git` marker.
- [ ] `RemoteProxySession` owns nonce/deadline/limits/control/event/reader state; blocked input and PTY readers are explicitly woken and joined on every exit and Drop path.
- [ ] CI/release accept only `impl-01..07` or `hardening-01..07` under the locked `codex/evidence/**` ref grammar, retain `candidate_sha` dispatch for recovery, and keep verifier-facing job IDs immutable.
- [ ] Release tag/assets/descriptors/smokes/manifest are create-new and carry one selected `run_id`/`run_attempt`; every artifact manifest excludes itself, the outer index hashes it, all eight exact producer/template/schema artifacts are nonempty with no extras, every producer is source-checked and runtime-joined through upload ID/name plus bare-digest-to-REST-digest normalization and the timestamp-unwrapped numeric job log, and candidate concurrency never cancels an earlier attempt.
- [ ] Task9 commits only GREEN local fixture/validator gates and launches no external workflow; all downloaded-binary tests run after Task10a through the orchestration plan's exact selected push run.
- [ ] Task10a/10b candidate execution and evidence-only commits follow the orchestration plan.
- [ ] Files outside those named by a task are not modified without amending this plan and its requirement map.
