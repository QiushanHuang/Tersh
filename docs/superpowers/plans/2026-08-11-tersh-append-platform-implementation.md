# Tersh Append-Platform Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the approved five-BODY `append-platform` recorder client, context-v2 parent binding, exact session/record/receipt validation, and a fault-injectable reference Host model that proves the specified lineage, atomicity, recovery, barrier, and projection invariants without claiming an unavailable privileged Host deployment.

**Architecture:** Keep production authority-free client logic in `scripts/evidence_core.py` and a new isolated `record_orchestration.py` entrypoint. Put Host-owned attempt selection, lineage state, ledger mutation, raw ancestry assertions, and projection capabilities only in a test-only reference model. The Host model independently constructs BODY 5; the production client independently derives and validates it. Reconcile the two older evidence plans before implementation so no worker follows their obsolete three-BODY or attestation prose.

**Tech Stack:** Python 3 standard library, canonical length-prefixed JSON over connected `AF_UNIX/SOCK_STREAM`, `unittest`, socketpair scripted peers, copy-on-write in-memory reference ledger, dirfd-relative no-follow filesystem helpers, Git/Rust repository verification commands.

---

## Task Identity And Frozen Inputs

- Task ID: `T-TERSH-PRODUCT-OPT-20260810-001`
- Repository: `/Users/joshua/.config/superpowers/worktrees/Studio/codex-tersh-trusted-core`
- Branch: `codex/tersh-trusted-core`
- Plan baseline: `69976c00288e187cea4fd6fecb31ad06bffa1184`
- Approved design: `docs/superpowers/specs/2026-08-11-tersh-append-platform-design.md`
- Required design SHA-256: `fc761d1ee4550e14aac10e70211f2b8cd87eab1d2ac3b9ace32aefdb224dade9`
- ResearchOS project, question, experiment, run, artifact, and registry IDs: not applicable; this is a personal Studio code repository.

At the start of every task, run:

```bash
cd /Users/joshua/.config/superpowers/worktrees/Studio/codex-tersh-trusted-core
test "$(shasum -a 256 docs/superpowers/specs/2026-08-11-tersh-append-platform-design.md | awk '{print $1}')" = "fc761d1ee4550e14aac10e70211f2b8cd87eab1d2ac3b9ace32aefdb224dade9"
git status --short
```

Expected: the design hash matches. Before editing, status is clean except for changes explicitly assigned to the current task. Stop on any unrelated tracked or untracked file instead of deleting or absorbing it.

## Trust Boundary And Deliverable Split

Production repository code in this plan may implement only:

- exact canonical parsers, schemas, source-map joins, and digest checks;
- the bounded five-BODY client transaction and absolute deadline;
- the closed four-option nonroot recorder CLI;
- validation of the complete Host-returned producer receipt.

Test-only code may model:

- attempt bindings, policy and worktree observations;
- generation-bearing handles and recovery;
- recorder leases, durable ledger transitions, receipts, report authorities;
- failure/supersede barriers and successor ordering;
- Host-selected projection publication and repair.

This plan does **not** create a production Host daemon. Formal acceptance remains fail-closed until the actual UID-0 Host Envelope Supervisor, private durable ledger/WAL, registered bundle/policy installation, Host-exclusive projection root, and required custom GitHub runners exist. A socketpair or reference-model pass is never privileged evidence.

## File Map

- Modify `scripts/evidence_core.py`: context v2, pure session/record/receipt validators, shared frame helpers, and public append client.
- Modify `scripts/tests/test_implementation_evidence.py`: migrate existing capture/provenance fixtures and assertions from exact context v1 to exact context v2.
- Create `scripts/tests/append_platform_host_model.py`: test-only independent Host record constructor, generation-bearing lineage store, copy-on-write ledger, fault hooks, deterministic barriers, enumeration, and projection model.
- Create `scripts/tests/test_append_platform.py`: focused schema, protocol, state, recovery, barrier, CLI, diagnostics, and projection tests.
- Create `scripts/implementation_evidence/record_orchestration.py`: isolated thin `append-platform` client entrypoint.
- Modify `docs/superpowers/plans/2026-08-10-tersh-implementation-iteration-evidence.md`: make the approved design authoritative and remove obsolete three-BODY/attest/client-projection claims.
- Modify `docs/superpowers/plans/2026-08-10-tersh-seven-cycle-hardening-implementation.md`: import the same authoritative append contract and remove formal attestation invocations until separately designed.

Do not create or modify a production Host service, `repair_projections.py`, `seal-agent-record`, attestation arm, manifest/finalizer protocol, Git closure, Rust product module, ResearchVault note, or registry in this plan.

## Production API Map

Add these public functions to `scripts/evidence_core.py`:

```text
validate_dispatch_context_v2(value: Any) -> dict[str, Any]
validate_orchestration_recorder_session(
    value: Any,
    *,
    context: Any,
    invocation: Any,
    response: Any,
) -> dict[str, Any]
derive_platform_orchestration_record(
    context: Any,
    invocation: Any,
    response: Any,
    recorder_session: Any,
) -> dict[str, Any]
validate_producer_receipt(value: Any) -> dict[str, Any]
append_platform_on_authenticated_socket(
    sock: Any,
    context_handle: Any,
    invocation_handle: Any,
    response_handle: Any,
) -> dict[str, Any]
```

Keep schema-specific helpers private. The append function accepts no operation string, body, policy, candidate, destination, nonce, receipt selector, filesystem path, Git option, or environment authority. It returns one detached exact `tersh-host-record-result-v1` object after REPLY-END and EOF.

## Test-Only Host API Map

Create the following test-only seam in `scripts/tests/append_platform_host_model.py`:

```text
FixtureClock.__call__() -> float
FixtureClock.advance(seconds: float) -> None

AppendPlatformHostModel.open_attempt(request, *, fault_at=None) -> dict
AppendPlatformHostModel.register_receipted_finding_source(receipt, body) -> None
AppendPlatformHostModel.validate_context_parent_sources(context) -> list[str]
AppendPlatformHostModel.seed_lineage(*, context) -> dict
AppendPlatformHostModel.capture_invocation(context_handle, invocation, *, lose_reply=False) -> dict
AppendPlatformHostModel.capture_response(context_handle, response, *, lose_reply=False) -> dict
AppendPlatformHostModel.bind_append_connection(handles, *, connection) -> dict
AppendPlatformHostModel.serve_append_platform(sock, lease, *, fault_at=None) -> None
AppendPlatformHostModel.recover_dispatch_lineage(request, *, fault_at=None) -> dict
AppendPlatformHostModel.abandon_dispatch_lineage(request, *, fault_at=None) -> dict
AppendPlatformHostModel.fail_responded_dispatch(request, *, fault_at=None) -> dict
AppendPlatformHostModel.fail_agent_report_authority(request, *, fault_at=None) -> dict
AppendPlatformHostModel.close_failed_attempt(request, *, fault_at=None) -> dict
AppendPlatformHostModel.close_superseded_attempt(request, *, fault_at=None) -> dict
AppendPlatformHostModel.open_successor_attempt(request, *, fault_at=None) -> dict
AppendPlatformHostModel.enumerate_attempt(attempt_binding_id) -> dict
AppendPlatformHostModel.install_projection_capability(root_fd, staging_fd, policy_sha256) -> None
AppendPlatformHostModel.publish_pending_projections(*, fault_at=None) -> dict
AppendPlatformHostModel.repair_missing_projections(evidence_id, through_attempt) -> dict
AppendPlatformHostModel.snapshot() -> dict
AppendPlatformHostModel.invoke_root_internal(operation, request, *, principal, fault_at=None) -> dict
```

The model uses lock order `attempt-global -> lineage`, copy-on-write shadow state, and a single `ledger.commit` visibility point. Its `build_host_orchestration_record` implementation must not call `derive_platform_orchestration_record`; otherwise one shared bug could make Host construction and client validation agree incorrectly.

## Test Discipline For Every Task

- Import the target module successfully before asserting a required callable;
  a missing API must produce a focused assertion failure, never an ImportError
  that could hide a broken fixture.
- Run one valid positive control before every mutation matrix so an
  always-raise implementation cannot pass.
- Mutate one authority or protocol fact at a time. When mutating a body field,
  recompute that body's canonical digest unless digest drift itself is the
  case under test.
- For rejection before any lease is bound, require complete Host-model snapshot
  equality. After a connection/session/nonce lease is bound but before ledger
  linearization, require byte/value equality for handles, attempt/lineage facts,
  chain, blob, receipt, authority, and projection state while allowing exactly
  one transient delta: that failed lease becomes absent or permanently invalid.
  Keep transient leases visible in `snapshot()` so old-session replay rejection
  remains provable. Post-linearization reply faults use the separately frozen
  committed snapshot.
- Propagate scripted-Host assertion, EOF, reset, and broken-pipe defects to the
  test thread; never suppress them merely because the client also raised
  `EvidenceError`.
- Require exact exception class `EvidenceError` for rejected client values and
  exact fixture assertions for a malformed Host script. Do not accept generic
  nonzero exit or any exception.
- Keep `append_platform_host_model.py` test-only. Production modules and the
  registered recorder must have no import path to `scripts.tests`.

## Task 1: Freeze The Approved Contract And Remove Legacy Contradictions

**Files:**

- Create: `scripts/tests/test_append_platform.py`
- Modify: `docs/superpowers/plans/2026-08-10-tersh-implementation-iteration-evidence.md`
- Modify: `docs/superpowers/plans/2026-08-10-tersh-seven-cycle-hardening-implementation.md`

- [ ] **Step 1: Add a semantic source-contract RED**

Create `AppendPlatformContractTests.test_approved_append_contract_has_five_bodies_no_record_upload_and_no_attest_arm`. It must:

1. recompute and assert the approved design SHA-256;
2. assert the design contains the ordered five BODY kinds `context`, `invocation`, `response`, `recorder-session`, `orchestration-record`;
3. assert the design rejects `RECORD-BEGIN`, `RECORD-CHUNK`, and `RECORD-END` for this operation;
4. assert both older plans contain a single authoritative reference to this design and SHA;
5. reject normative three-BODY append wording, a formal `record_orchestration.py attest` command, or a claim that the nonroot recorder writes the formal projection.

Use a frozen list of forbidden regular expressions, not a broad keyword ban that would reject a clearly labeled historical explanation:

```python
FORBIDDEN_NORMATIVE_APPEND_PATTERNS = (
    r"append-platform`\s*=\s*`context, invocation, response`",
    r"record_orchestration\.py attest\b",
    r"recorder .*create-new publish.*formal projection",
)
```

- [ ] **Step 2: Run the exact RED**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformContractTests.test_approved_append_contract_has_five_bodies_no_record_upload_and_no_attest_arm
```

Expected: `FAIL`, naming the obsolete three-BODY/attest/client-projection contract in at least one older plan. It must not fail with ImportError.

- [ ] **Step 3: Reconcile both older plans**

Add a short normative section to each older plan with the exact design path and SHA. Replace obsolete append details with:

```text
append-platform imports the approved 2026-08-11 design verbatim: exact five BODYs,
Host-built frozen BODY 5, client validation, no client RECORD stream, Host-ledger
linearization, and Host-exclusive formal projection. Operator attestation is deferred
and no formal attest CLI exists in this checkpoint.
```

Do not duplicate the 996-line design. Preserve unrelated tasks and explicitly label any retained historical three-BODY discussion as superseded non-normative context.

- [ ] **Step 4: Verify GREEN and commit**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformContractTests
git diff --check
git add -- \
  scripts/tests/test_append_platform.py \
  docs/superpowers/plans/2026-08-10-tersh-implementation-iteration-evidence.md \
  docs/superpowers/plans/2026-08-10-tersh-seven-cycle-hardening-implementation.md
git commit -m "docs(evidence): freeze append platform contract"
```

Expected: contract test passes and the commit contains only the three listed files.

## Task 2: Require Context V2 And Structural Parent Binding

**Files:**

- Modify: `scripts/evidence_core.py`
- Modify: `scripts/tests/test_implementation_evidence.py`
- Modify: `scripts/tests/test_append_platform.py`

- [ ] **Step 1: Add context-v2 RED tests**

Add these exact methods to `AppendPlatformSchemaTests`:

- `test_context_v2_binds_bounded_existing_parent_finding_ids`
- `test_context_v2_structurally_rejects_legacy_alias_duplicate_reordered_or_cross_evidence_parents`

The positive matrix covers zero parents, one parent, and 128 strictly ascending same-evidence parents. The negative matrix has stable case IDs and covers:

```text
context-v1
v1-plus-optional-parents
missing-parents
null-parents
bool-parents
129-parents
non-string-member
finding-000
finding-outside-001..999
wrong-evidence-prefix
duplicate
descending
same-sequence-alias
```

Every mutation starts from a valid v2 body and only accepts `EvidenceError`. The input object and nested parent list must remain unchanged.

- [ ] **Step 2: Run RED**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests.test_context_v2_binds_bounded_existing_parent_finding_ids \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests.test_context_v2_structurally_rejects_legacy_alias_duplicate_reordered_or_cross_evidence_parents
```

Expected: `FAIL` because `validate_dispatch_context_v2` is absent and current `_validate_context_body` accepts v1 only.

- [ ] **Step 3: Implement the minimal v2 validator**

Add:

```python
FINDING_ID_RE = re.compile(
    r"^(?P<evidence>(?:impl|hardening)-0[1-7])-F"
    r"(?P<sequence>(?:00[1-9]|0[1-9][0-9]|[1-9][0-9]{2}))$"
)

MAX_PARENT_FINDING_IDS = 128
```

Implement `_validate_parent_finding_ids(value, evidence_id)` with exact-list and exact-string checks, same evidence prefix, strictly increasing integer sequence, and detached-list return. Change the exact context key set to include required `parent_finding_ids`, require schema `tersh-host-dispatch-context-v2`, and expose `validate_dispatch_context_v2` as a deep-copying public wrapper. Make capture and `validate_platform_envelope_provenance` use v2 only.

- [ ] **Step 4: Migrate existing fixtures without weakening old tests**

Change `host_envelope_bodies()` in `test_implementation_evidence.py` to exact context v2 with `parent_finding_ids: []`. Update schema literals in existing positive assertions. Keep an explicit legacy-v1 negative rather than deleting legacy coverage.

- [ ] **Step 5: Verify focused and regression GREEN**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests.test_context_v2_binds_bounded_existing_parent_finding_ids \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests.test_context_v2_structurally_rejects_legacy_alias_duplicate_reordered_or_cross_evidence_parents \
  scripts.tests.test_implementation_evidence
```

Expected: all selected tests pass; capture framing remains unchanged except the captured context schema.

- [ ] **Step 6: Commit**

```bash
git diff --check
git add -- scripts/evidence_core.py scripts/tests/test_implementation_evidence.py scripts/tests/test_append_platform.py
git commit -m "feat(evidence): require dispatch context v2"
```

Boundary after this commit: structural context validation only; no Host history-origin claim.

## Task 3: Validate Host-Receipted Parent Sources

**Files:**

- Create: `scripts/tests/append_platform_host_model.py`
- Modify: `scripts/tests/test_append_platform.py`

- [ ] **Step 1: Add the Host-source RED**

Add both approved source-aware tests:

- `test_context_v2_rejects_v1_optional_alias_duplicate_reordered_cross_evidence_or_future_parents`
- `test_context_v2_rejects_projection_only_mismatched_ambiguous_or_same_attempt_parent_sources`

Define each normalized, already body-schema-validated Host source as the exact object:

```text
{
  finding_id,
  evidence_id,
  evidence_attempt,
  source_kind,
  receipt_id,
  body,
  body_sha256
}
```

`source_kind` is exactly `review|audit-failure`; `receipt_id` and `body_sha256` are 64-lowercase hex. The source body is a canonical closed fixture body whose findings array contains the exact ID once. `register_receipted_finding_source` accepts only sources already admitted by the corresponding closed review/audit-failure fixture validator; it cannot accept projection-only bodies. Cover prior valid review and audit-failure sources, then reject projection-only omission, missing receipt ID, canonical digest mismatch, wrong evidence, same/future attempt, absent ID, duplicate ID in one body, and the same ID in two receipted bodies.

- [ ] **Step 2: Run RED**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests.test_context_v2_rejects_v1_optional_alias_duplicate_reordered_cross_evidence_or_future_parents \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests.test_context_v2_rejects_projection_only_mismatched_ambiguous_or_same_attempt_parent_sources
```

Expected: `FAIL` because the reference Host source registry is absent.

- [ ] **Step 3: Implement exact source resolution**

The model method must first call `validate_dispatch_context_v2`, validate the exact normalized source object and canonical body digest, build a one-to-one private index by finding ID, and require every context parent to resolve exactly once to a prior attempt of the same evidence ID. It returns a detached copy of the validated parent array, not source objects or receipt selectors.

This logic stays test-only because the production UID-0 registry/component is unidentified. Do not call it client-side or add a source-list wire/CLI argument. A nonroot caller never supplies the source list over the append wire.

- [ ] **Step 4: Verify and commit**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests
git diff --check
git add -- scripts/tests/append_platform_host_model.py scripts/tests/test_append_platform.py
git commit -m "test(evidence): model parent finding sources"
```

Boundary after this commit: test-only reference source resolution; receipt origin still depends on the unavailable Host ledger.

## Task 4: Add Closed Recorder-Session Validation

**Files:**

- Modify: `scripts/evidence_core.py`
- Modify: `scripts/tests/test_append_platform.py`

- [ ] **Step 1: Add session RED tests**

Add:

- `test_recorder_session_schema_rejects_wrong_schema_extra_missing_alias_null_bool_and_wrong_type_fields`
- `test_append_platform_rejects_each_context_session_and_dispatch_identity_drift`
- `test_append_platform_rejects_candidate_tree_policy_destination_or_parent_join_drift`

The exact session key set is:

```python
SESSION_KEYS = {
    "schema", "producer_session_id", "attempt_binding_id",
    "predecessor_attempt_binding_id", "entrypoint", "producer_mode",
    "operation", "context_nonce", "dispatch_id", "evidence_id",
    "evidence_attempt", "run_binding", "candidate", "candidate_tree",
    "worktree_handle", "worktree_observed_at", "baseline_commit",
    "candidate_relation", "bundle_id", "runtime_profile_id",
    "policy_sha256", "policy_entry_id", "policy_entry_sha256",
    "projection_root_class", "record_class", "record_schema",
    "destination", "parent_finding_ids_sha256",
    "next_receipt_sequence", "previous_receipt_id",
}
```

Use one-field mutations for every fixed literal, hash/OID/ID/timestamp/integer/nullability field. Include sequence `1/null`, sequence `2/non-null`, and the two invalid crossed pairs. Join tests mutate each duplicated context/invocation/response fact after recomputing its local digest so they cannot pass merely because of a stale hash.

- [ ] **Step 2: Run RED**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests.test_recorder_session_schema_rejects_wrong_schema_extra_missing_alias_null_bool_and_wrong_type_fields \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests.test_append_platform_rejects_each_context_session_and_dispatch_identity_drift \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests.test_append_platform_rejects_candidate_tree_policy_destination_or_parent_join_drift
```

Expected: `FAIL` because `validate_orchestration_recorder_session` is absent.

- [ ] **Step 3: Implement schema and join validation**

Require fixed literals:

```text
schema                = tersh-host-orchestration-recorder-session-v1
entrypoint            = record-orchestration
producer_mode         = harness
operation             = append-platform
projection_root_class = local
record_class          = orchestration
record_schema         = tersh-evidence-orchestration-v1
```

Validate the session against already validated context/invocation/response bodies. Derive the exact destination:

```python
f"attempt-{context['evidence_attempt']}/candidate-{session['candidate']}/" \
f"orchestration/{context['role']}.{context['wave']}.{context['review_attempt']}.json"
```

Require `bundle_id == context.harness_bundle_sha256`, matching nonce/dispatch/evidence/attempt/run/worktree/baseline, exact canonical parent-array hash, exact `equal|descendant` tag, and a detached deep-copy return. Do not run Git or claim ancestry here; raw ancestry is a Host binding assertion tested in the model.

- [ ] **Step 4: Verify and commit**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests
git diff --check
git add -- scripts/evidence_core.py scripts/tests/test_append_platform.py
git commit -m "feat(evidence): validate recorder sessions"
```

## Task 5: Derive The Exact Orchestration Record And Validate Receipts

**Files:**

- Modify: `scripts/evidence_core.py`
- Modify: `scripts/tests/test_append_platform.py`

- [ ] **Step 1: Add record and receipt RED tests**

Add:

- `test_orchestration_record_schema_has_one_authoritative_source_per_field`
- `test_orchestration_record_rejects_extra_missing_mixed_or_overbound_fields`
- `test_append_platform_candidate_rules_cover_wave_a_wave_b_changed_and_unchanged`
- `test_producer_receipt_schema_is_closed_typed_and_detached`

The test's expected-record factory must construct records manually from the approved source-map table. Before constructing the record, bind `validated_platform_provenance` to the exact return of `validate_platform_envelope_provenance` over detached context/invocation/response `{body,sha256}` entries. It must not call the production derivation function.

- [ ] **Step 2: Run RED**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests.test_orchestration_record_schema_has_one_authoritative_source_per_field \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests.test_orchestration_record_rejects_extra_missing_mixed_or_overbound_fields \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests.test_append_platform_candidate_rules_cover_wave_a_wave_b_changed_and_unchanged \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests.test_producer_receipt_schema_is_closed_typed_and_detached
```

Expected: `FAIL` because record derivation and receipt validation are absent.

- [ ] **Step 3: Implement record derivation**

Return exactly:

```python
{
    "schema": "tersh-evidence-orchestration-v1",
    "evidence_id": context["evidence_id"],
    "evidence_attempt": context["evidence_attempt"],
    "run_binding": context["run_binding"],
    "role": context["role"],
    "wave": context["wave"],
    "review_attempt": context["review_attempt"],
    "baseline_commit": context["baseline_commit"],
    "reviewed_commit": recorder_session["candidate"],
    "parent_finding_ids": copy.deepcopy(context["parent_finding_ids"]),
    "dispatch_id": invocation["dispatch_id"],
    "agent_id": response["agent_id"],
    "canonical_task_path": context["canonical_task_path"],
    "agent_run_id": response["agent_run_id"],
    "model": invocation["selected_model"],
    "reasoning_effort": invocation["selected_reasoning_effort"],
    "dispatched_at": invocation["dispatched_at"],
    "started_at": response["started_at"],
    "ended_at": response["ended_at"],
    "terminal_status": response["terminal_status"],
    "provenance": validated_platform_provenance,
}
```

Require canonical record length `1..61440`. Enforce every Wave A/Wave B equality and reported-result rule derivable from context/response/session. For `descendant`, the client validates only the exact relation tag and the cross-body values available to it; Task 6 makes the Host model prove raw ancestry from the immediate predecessor. The production client never invokes Git and never treats `reported_result_commit` as authority.

- [ ] **Step 4: Implement complete producer-receipt validation**

Validate exact `tersh-host-producer-receipt-v1` fields from the shared plan:

```text
schema,receipt_id,attempt_binding_id,producer_session_id,sequence,
previous_receipt_id,producer_mode,entrypoint,bundle_id,runtime_profile_id,
policy_entry_id,policy_entry_sha256,environment_capability,
projection_root_class,record_class,record_schema,destination,body_sha256,
byte_count,dispatch_id,reported_record_sha256,created_at
```

Support this exact record-class enum:

```text
attempt-marker | candidate-marker | gate | cumulative-gates |
runner-inventory | before-repo-runs | bootstrap | selected-run | jobs |
artifact-index | external-candidate | orchestration | review |
implementation-entry | requirements | completion-audit |
audit-reservation-failure | orchestration-failure | agent-report-failure
```

The append-specific private join requires the orchestration receipt to match the session, exact record bytes/hash/count and sequence/predecessor, with null detached `dispatch_id`, `reported_record_sha256`, and environment fields. Add one-field negatives for either detached agent-report field being populated; do not compare a null harness-receipt dispatch field to the captured dispatch.

Validate `environment_capability` as either null or the exact closed
`tersh-host-environment-capability-v1`/two `tersh-host-opened-directory-v1`
objects imported from the shared plan. Require `producer_mode="harness"` to
carry null `dispatch_id` and `reported_record_sha256`; require
`producer_mode="agent-report"`, `entrypoint="seal-agent-record"`, and two
64-hex agent-report fields together. Use the existing gate-name grammar for
`policy_entry_id`, positive exact integers for sequence/byte count, the
null/predecessor sequence rule, canonical RFC 3339 UTC `created_at`, and a
canonical relative destination that is later required to equal the session.

- [ ] **Step 5: Verify and commit**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformSchemaTests
git diff --check
git add -- scripts/evidence_core.py scripts/tests/test_append_platform.py
git commit -m "feat(evidence): derive append platform records"
```

Boundary after this commit: pure values only; no socket transaction, handles, or durability.

## Task 6: Build The Generation-Bearing Reference Host Model

**Files:**

- Modify: `scripts/tests/append_platform_host_model.py`
- Modify: `scripts/tests/test_append_platform.py`

- [ ] **Step 1: Add state-model RED tests**

Add `AppendPlatformStateTests` with:

- `test_append_platform_rejects_cross_lineage_generation_alias_duplicate_or_mode_mixed_handles`
- `test_root_peer_with_wrong_recorder_session_fails_before_handle_lookup`
- `test_recorder_session_launch_lease_replay_and_fd_generation_reuse_fail_before_handle_lookup`
- `test_root_internal_authentication_precedes_nonce_session_or_handle_lookup`
- `test_append_platform_rejects_wave_a_baseline_drift_and_unrelated_wave_b_candidate`
- `test_candidate_relation_matches_raw_ancestry_in_wave_c_and_closure`
- `test_wave_b_baseline_is_immediate_predecessor_not_older_context_ancestor`

The valid platform typestate is exactly:

```text
CREATED(H0)
  -> INVOKED(H1, HI)
  -> RESPONDED_PLATFORM(H2, HI, HR)
  -> APPENDED_PLATFORM
```

- [ ] **Step 2: Run RED**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformStateTests
```

Expected: `FAIL` because the parent-source-only model has no lineage, attempt, session, or root-call implementation. The test module itself imports successfully.

- [ ] **Step 3: Implement the minimal state skeleton**

Represent every handle row as:

```python
{
    "lineage_id": HEX64,
    "kind": "context|invocation|response",
    "transition_index": NONNEGATIVE_INT,
    "body": CLOSED_BODY,
    "body_sha256": HEX64,
    "live": BOOL,
}
```

Represent lineage rows with context nonce, exact state, transition index, recovery generation, attempt binding ID, registered session IDs, and terminal state. `seed_lineage`, `capture_invocation`, and `capture_response` must commit complete tuple rotations atomically using copy-on-write state replacement. Cross-lineage, alias, old-generation, duplicate invocation/response, and wrong state fail without changing `snapshot()`. Add the Host-only `build_host_orchestration_record` from the frozen source map without importing or calling the production derivation function.

- [ ] **Step 4: Add attempt/session/lease authority**

`open_attempt` persists an immutable candidate/tree/baseline/predecessor/bundle/runtime/policy binding plus marker receipt chain. `bind_append_connection` authenticates the root-internal connection/session before handle lookup, assigns a Host-generated monotonic connection generation to the actual socket object plus kernel-visible identity, binds that generation and lineage, and creates a five-second launch lease using `FixtureClock`; no caller-provided numeric FD or connection ID is authoritative. The first valid BEGIN binds one fresh transaction nonce and one absolute deadline; pre-linearization expiry discards only the lease. Tests expire the lease before BEGIN, close the socket, reuse its numeric FD for a new socket generation, and require rejection before handle lookup while a separately Host-bound fresh session succeeds. Identical session/nonce replay on the old connection also fails. `invoke_root_internal` rejects any principal other than the model's root-supervisor identity before nonce/session/handle lookup and dispatches only the exact closed root-internal operations; no production CLI or wire arm is added.

Raw ancestry is modeled as Host-owned immutable facts: equal or a raw-object descendant of the immediate predecessor candidate. Include negative fixtures for older-ancestor side branches, replacement/graft/shallow/alternate views, wrong tree, dirty observation, missing/unborn object, and BODY-to-COMMIT drift. Do not launch Git from production client code.

- [ ] **Step 5: Verify and commit**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformStateTests
git diff --check
git add -- scripts/tests/append_platform_host_model.py scripts/tests/test_append_platform.py
git commit -m "test(evidence): model append platform Host state"
```

Boundary: this is a deterministic test oracle, not a production or privileged Host.

## Task 7: Implement The Exact Five-BODY Append Client

**Files:**

- Modify: `scripts/evidence_core.py`
- Modify: `scripts/tests/append_platform_host_model.py`
- Modify: `scripts/tests/test_append_platform.py`

- [ ] **Step 1: Add wire RED tests**

Add `AppendPlatformWireTests`:

- `test_append_platform_exact_session_and_host_built_record_body_order`
- `test_host_record_constructor_is_independent_from_client_derivation`
- `test_append_platform_rejects_record_frame_or_hash_only_commit`
- `test_host_spools_exact_record_body_sent_before_commit`
- `test_record_reply_receipt_joins_session_route_body_and_chain`
- `test_record_reply_receipt_joins_session_route_body_and_chain`
- `test_append_platform_one_absolute_deadline_covers_begin_bodies_commit_request_end_eof_and_reply`
- `test_append_platform_deadline_is_not_reset_or_caller_configurable`

The exact transcript is:

```text
client BEGIN
Host BODY context
Host BODY invocation
Host BODY response
Host BODY recorder-session
Host BODY orchestration-record
Host BODY-END
client COMMIT
client REQUEST-END
client SHUT_WR
Host ledger linearization
Host REPLY
Host REPLY-END
Host SHUT_WR / client EOF
```

The constructor-independence test patches
`derive_platform_orchestration_record` to raise while the Host constructor
still succeeds, asserts the Host-model AST contains no import/call reference to
that production function, then mutates one Host-built source field and proves
the unpatched client derivation rejects it.

- [ ] **Step 2: Run RED**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformWireTests
```

Expected: `FAIL` because `append_platform_on_authenticated_socket` is absent.

- [ ] **Step 3: Implement shared receive/request/reply helpers**

Extract only reusable mechanics from the existing capture transaction:

```text
_receive_host_body_sequence(
    sock: socket.socket,
    transaction_nonce: str,
    operation: str,
    body_order: Sequence[str],
    validators: dict[str, Callable[[Any], dict[str, Any]]],
    deadline: float,
) -> tuple[dict[str, dict[str, Any]], list[str]]

_send_host_request_end(
    sock: socket.socket,
    transaction_nonce: str,
    operation: str,
    commit: dict[str, Any],
    deadline: float,
) -> None
```

Preserve capture behavior. All prefix/body/half-close/EOF operations share one absolute monotonic deadline.

- [ ] **Step 4: Implement the public append call**

Validate three distinct 64-hex handles. Generate a fresh 64-hex transaction nonce. The public function has no deadline parameter: it computes exactly `time.monotonic() + HOST_TRANSACTION_TIMEOUT_SECONDS` once and uses that absolute value through final EOF. Tests patch the module clock/socket behavior rather than pass a duration or deadline. Receive exactly the five BODY kinds, validate context/session/provenance, derive BODY 5 independently, and require exact equality and exact canonical digest. Send exactly:

```python
commit = {
    "schema": "tersh-host-transaction-commit-v1",
    "transaction_nonce": transaction_nonce,
    "operation": "append-platform",
    "body_sha256s": body_hashes,
    "record_facts": {
        "evidence_id": context["evidence_id"],
        "evidence_attempt": context["evidence_attempt"],
        "run_binding": context["run_binding"],
        "candidate": session["candidate"],
        "destination": session["destination"],
        "record_sha256": body_hashes[4],
    },
}
```

Reject any extra/missing/reordered BODY, upload frame, hash-only COMMIT behavior, wrong BODY-END, wrong nonce/operation, trailing frame, early/late EOF, reply/result/receipt mismatch, or private-field leak. Return only the detached record result after reply EOF.

- [ ] **Step 5: Implement the reference Host happy path**

Implement `serve_append_platform` just far enough to make the non-faulted
transcript executable: validate the prebound socket generation and launch lease
before handle lookup; freeze and send the independently constructed five BODYs
plus BODY-END; receive and validate exact COMMIT and REQUEST-END; observe client
EOF; perform one successful all-or-none in-memory transition consuming the
session/H2/HI/HR and creating the exact blob/receipt/conditional authority;
then send the joined REPLY, REPLY-END, and EOF. This version has no fault hooks,
crash replay, concurrency barrier, or chain-drift injection. Task 8 replaces
the happy-path transition internals with the copy-on-write fault-atomic
implementation while retaining this wire surface.

- [ ] **Step 6: Verify GREEN and capture regression**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformWireTests \
  scripts.tests.test_implementation_evidence
```

Expected: wire and existing capture suites pass.

- [ ] **Step 7: Commit**

```bash
git diff --check
git add -- scripts/evidence_core.py scripts/tests/append_platform_host_model.py scripts/tests/test_append_platform.py
git commit -m "feat(evidence): add append platform wire"
```

Boundary: a successful socketpair transcript proves client behavior only, not Host durability.

## Task 8: Prove Atomic Linearization, Chain Drift, And Reply Loss

**Files:**

- Modify: `scripts/tests/append_platform_host_model.py`
- Modify: `scripts/tests/test_append_platform.py`

- [ ] **Step 1: Add atomicity RED tests**

Add `AppendPlatformAtomicityTests`:

- `test_append_platform_atomic_handle_blob_receipt_authority_fault_matrix`
- `test_append_platform_deadline_after_commit_before_request_end_or_eof_is_prelinearization`
- `test_append_platform_concurrent_calls_create_one_receipt_and_authority`
- `test_append_platform_chain_head_drift_fails_before_linearization_and_retries`
- `test_every_prelinearization_failure_preserves_exact_state_vector`
- `test_every_postcommit_reply_fault_preserves_one_durable_state_vector`

- [ ] **Step 2: Freeze the fault-point matrix**

Use these exact IDs:

```text
wire.after-begin
wire.after-body-context
wire.after-body-invocation
wire.after-body-response
wire.after-body-session
wire.after-body-record
wire.after-body-end
wire.after-commit
wire.after-request-end
wire.after-client-eof
linearize.after-session-consume
linearize.after-handle-consume
linearize.after-blob-insert
linearize.after-receipt-append
linearize.after-authority-insert
linearize.before-commit
linearize.after-commit
reply.after-reply
reply.after-reply-end
reply.before-eof
```

Before `linearize.after-commit`, reopening the model must show live handles, unchanged chain, and no blob/receipt/authority/projection, but the failed connection/nonce-bound recorder session lease must be absent or permanently invalid. For every such fault, prove replay through the old FD/session/nonce fails before handle lookup and a fresh Host-bound session over the same live tuple can retry. At and after `linearize.after-commit`: consumed session/handles, one exact blob/receipt, chain advanced once, conditional authority present exactly when `reported_record_sha256` is nonnull, and no second append.

- [ ] **Step 3: Run RED**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformAtomicityTests
```

Expected: `FAIL` because the Host model has no copy-on-write ledger commit/fault replay.

- [ ] **Step 4: Implement one durable reference transaction**

Build the entire successor state in a shadow copy. Under lock, revalidate attempt/session/handles/worktree/route/frozen bytes/deadline/current chain. Apply session and H2/HI/HR consumption, create-new blob, consecutive receipt, optional authority, and replay row only to the shadow. Publish the shadow at one `ledger.commit`. Fault injection before commit discards the shadow, then invalidates/removes the transient connection/session/nonce lease while retaining the handles; after commit cannot roll the durable state back.

Use deterministic `threading.Barrier` hooks for two concurrent appends. Timing-only sleeps are not acceptance evidence. Force one session's `next_receipt_sequence/previous_receipt_id` to drift and prove it retries from a fresh session.

- [ ] **Step 5: Verify and commit**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformAtomicityTests
git diff --check
git add -- scripts/tests/append_platform_host_model.py scripts/tests/test_append_platform.py
git commit -m "test(evidence): prove append atomicity"
```

## Task 9: Add Crash-Atomic Recovery And Binding-Aware Abandonment

**Files:**

- Modify: `scripts/tests/append_platform_host_model.py`
- Modify: `scripts/tests/test_append_platform.py`

- [ ] **Step 1: Add recovery/abandonment RED tests**

Add `AppendPlatformRecoveryTests`:

- `test_capture_reply_orphan_recovery_rotates_private_handles_and_preserves_response`
- `test_recovery_generation_allows_invoked_then_responded_lost_reply_recovery`
- `test_recovery_result_state_generation_and_nullable_handles_are_exact`
- `test_capture_orphan_abandon_rejects_responded_state_and_never_reuses_persisted_attempt`
- `test_recover_and_abandon_closed_schemas_and_atomic_fault_matrix`
- `test_recover_abandon_capture_and_append_race_has_exactly_one_durable_winner`
- `test_binding_retaining_abandon_enters_closing_failed_without_reusing_binding`

- [ ] **Step 2: Freeze exact fault and race cases**

```text
recover.after-old-tuple-invalidate
recover.after-new-tuple-insert
recover.after-replay-row
recover.before-commit
recover.after-commit-before-result
abandon.after-handle-invalidate
abandon.after-terminal-row
abandon.after-attempt-state
abandon.after-replay-row
abandon.before-commit
abandon.after-commit-before-result
```

Race recovery, abandonment, capture, and append at the lineage lock; exactly one transition wins. Prove a later state/transition may recover with a new generation even after an earlier recovery was durably used.

- [ ] **Step 3: Run RED**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformRecoveryTests
```

Expected: `FAIL` because root-internal recovery/abandon transitions are absent.

- [ ] **Step 4: Implement exact CAS operations**

Freeze the exact recovery request/result schemas:

```text
tersh-host-recover-dispatch-lineage-request-v1
{schema,context_nonce,expected_state,transition_index,recovery_generation,reason}

tersh-host-recover-dispatch-lineage-result-v1
{schema,context_nonce,state,transition_index,recovery_generation,
 context_handle,invocation_handle,response_handle}
```

`expected_state` is exactly `created|invoked|responded-platform`; reason is
exactly `capture-reply-unrecoverable`; transition/recovery indexes are exact
nonnegative integers. Authenticate the root-internal call first, resolve the
private lineage ID from `context_nonce` inside the Host, and use the internal
key `(resolved_lineage_id, transition_index, recovery_generation)` for the CAS.
Reject caller `lineage_id`, handle, receipt, authority, or any extra selector.
Atomically invalidate the complete old tuple, create the complete replacement
tuple, and store canonical request digest plus replay result. Identical unused
replay returns the same result; wrong generation/state or replay after durable
use conflicts.

The result `state` and `transition_index` equal the request, and result
`recovery_generation == request.recovery_generation + 1`. State `created`
returns only `context_handle`; `invoked` returns context and invocation handles;
`responded-platform` returns all three. Every inapplicable handle is exactly
null. Add one-field negatives for old/same/skipped generation, wrong state or
index, leaked stale/sibling handle, nonnull inapplicable handle, null applicable
handle, and aliased returned handles.

Freeze abandonment as exact request
`tersh-host-abandon-dispatch-lineage-request-v1`
`{schema,context_nonce,expected_state,transition_index,recovery_generation,reason}`
and exact result `tersh-host-abandon-dispatch-lineage-result-v1`
`{schema,context_nonce,state}`, where expected state is only
`created|invoked`, reason is `capture-reply-unrecoverable`, and result state is
`abandoned`. It atomically invalidates every live tuple handle and stores
terminal/replay state. A no-binding reservation may be released; a persisted
binding is retained, the attempt moves to `CLOSING_FAILED`, and an `abandoned`
terminal row becomes a valid close-barrier trigger. Responded lineages reject
abandonment.

- [ ] **Step 5: Verify and commit**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformRecoveryTests
git diff --check
git add -- scripts/tests/append_platform_host_model.py scripts/tests/test_append_platform.py
git commit -m "test(evidence): prove dispatch recovery"
```

## Task 10: Preserve Responded And Report-Authority Failures

**Files:**

- Modify: `scripts/tests/append_platform_host_model.py`
- Modify: `scripts/tests/test_append_platform.py`

- [ ] **Step 1: Add failure-transition RED tests**

Add `AppendPlatformFailureTests`:

- `test_irrecoverable_responded_dispatch_atomically_records_failure`
- `test_dispatch_failure_lost_reply_replays_one_receipt_and_finalizer_rejects_attempt`
- `test_dispatch_failure_body_receipt_and_route_source_map_is_closed`
- `test_unsealable_report_authority_atomically_records_callback_failure`
- `test_report_authority_failure_lost_result_and_sealer_race_have_one_winner`
- `test_dispatch_and_report_failure_reasons_require_current_host_observation`

Use exact reason enums, failure-body field sets, policy-derived destinations, receipt joins, and null environment/agent-report fields from the design.

- [ ] **Step 2: Run RED**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformFailureTests
```

Expected: `FAIL` because durable failure bodies/receipts and authority consumption do not exist.

- [ ] **Step 3: Implement all-or-none failure transitions**

For each operation, inject faults after consume, body insert, receipt append, lineage state, attempt state, replay row, before commit, and after commit-before-result. Reopening must show exactly the unchanged prior state or the complete terminal failure with one replayable receipt. `fail-agent-report-authority` and the test-only successful sealer race under the same authority lock; one and only one wins.

Before consuming anything and again under the commit lock, rederive the reason
from current Host-owned facts. Dispatch reasons are exactly
`candidate-object-missing|candidate-relation-invalid|worktree-identity-drift|attempt-policy-drift|record-construction-invalid`;
report-authority reasons are exactly
`draft-missing|draft-digest-mismatch|draft-schema-invalid|draft-path-invalid|sealer-policy-invalid`.
Pair every allowed reason with a fixture where that observation is false, and
mutate the observed fact between request and lock. False, stale, or changed
reasons leave handles/authority, body store, receipt chain, lineage/attempt
state, and replay table untouched.

The first lineage failure moves `ACTIVE -> CLOSING_FAILED`. It does not directly permit a successor.

- [ ] **Step 4: Verify and commit**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformFailureTests
git diff --check
git add -- scripts/tests/append_platform_host_model.py scripts/tests/test_append_platform.py
git commit -m "test(evidence): preserve terminal failures"
```

## Task 11: Freeze Failed And Superseded Attempts Before Successors

**Files:**

- Modify: `scripts/tests/append_platform_host_model.py`
- Modify: `scripts/tests/test_append_platform.py`

- [ ] **Step 1: Add barrier RED tests**

Add `AppendPlatformBarrierTests`:

- `test_root_internal_operations_reject_caller_selected_receipt_authority_finding_or_terminal_sets`
- `test_binding_retaining_abandon_closes_marker_only_attempt_before_next_attempt`
- `test_irrecoverable_responded_dispatch_atomically_records_failure_and_allows_next_attempt`
- `test_unsealable_report_authority_atomically_records_callback_failure_and_drains_barrier`
- `test_failed_lineage_drains_siblings_before_attempt_barrier_or_next_attempt`
- `test_terminal_lineage_array_rejects_omitted_duplicate_reordered_live_or_pending_rows`
- `test_closing_failed_freezes_lineage_registration_and_supersede_requires_active`
- `test_receipted_findings_supersede_drained_attempt_before_successor_opens`
- `test_supersede_vs_late_append_race_freezes_one_complete_predecessor_history`
- `test_fail_supersede_sibling_append_and_open_race_has_one_serializable_winner`

The exact terminal row is:

```text
{schema,lineage_id,context_nonce,transition_index,terminal_state,terminal_receipt_id}
```

Rows are strictly sorted by lineage ID; count equals frozen registration count; the digest hashes the canonical full array plus LF. Test every omission, duplicate, reorder, live handle/session/recovery result, pending authority, and mismatched terminal receipt.

- [ ] **Step 2: Run RED**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformBarrierTests
```

Expected: `FAIL` because close barriers and successor gating are absent.

- [ ] **Step 3: Implement attempt-global barriers**

Implement exact states:

```text
ACTIVE | CLOSING_FAILED | TERMINAL_FAILED | TERMINAL_SUPERSEDED | FORMALLY_CLOSED
```

The first transition to `CLOSING_FAILED` freezes the complete registered-lineage
set and rejects every later dispatch registration under the attempt lock.
`close_failed_attempt` requires a failure receipt or binding-retaining
abandonment, all frozen registered lineages terminal, and zero live
capability/session/recovery/authority. `close_superseded_attempt` starts only
from exactly `ACTIVE`; it rejects `CLOSING_FAILED` and every terminal state, and
requires a nonempty Host-derived sorted unresolved P0/P1 set, the same zero-live
terminal array, and no preimage/closure. Both persist idempotent result rows and
frozen aggregate hashes in one commit.

`open_successor_attempt` joins the same attempt lock and requires predecessor `TERMINAL_FAILED|TERMINAL_SUPERSEDED` with frozen aggregates unchanged. `FORMALLY_CLOSED` prohibits a successor. Race late sibling append, failure/supersede, and two different successor candidates using deterministic barriers.

- [ ] **Step 4: Verify and commit**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformBarrierTests
git diff --check
git add -- scripts/tests/append_platform_host_model.py scripts/tests/test_append_platform.py
git commit -m "test(evidence): freeze attempt predecessors"
```

## Task 12: Prove Host-Exclusive Projection And Repair Semantics

**Files:**

- Modify: `scripts/tests/append_platform_host_model.py`
- Modify: `scripts/tests/test_append_platform.py`

- [ ] **Step 1: Add projection RED tests**

Add `AppendPlatformProjectionTests`:

- `test_append_platform_prelinearization_retry_and_postlinearization_lost_reply_repair`
- `test_host_projects_exact_committed_blob_and_repairs_only_a_missing_projection`
- `test_formal_projection_root_has_no_candidate_writable_ancestor_or_collision_race`
- `test_projection_fault_matrix_yields_only_absent_or_exact_committed_leaf`
- `test_repair_never_replaces_mismatch_or_appends_receipt_or_authority`

The test formal root and a disjoint same-filesystem staging root must both be physically outside the candidate tree. The model installs both opened capabilities into immutable Host policy state before opening the attempt; publication/repair operations receive neither a receipt ID nor a root FD. Retain and compare every opened ancestor's device/inode/mode identity. Under the current nonroot test UID, simulate the required non-writable boundary with already opened capabilities; do not claim UID-0 installation success.

- [ ] **Step 2: Run RED**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformProjectionTests
```

Expected: `FAIL` because the model has no projection state or repair path.

- [ ] **Step 3: Implement Host-selected publication and repair**

Do not use `publish_new_at` inside the formal directory: its in-directory `.tmp-*` file can survive process kill and poison the exact namespace. The Host model writes/fsyncs a random mode-0600 file beneath its separate prebound staging capability, then create-new links that inode into the final prebound formal directory and fsyncs the final directory. A crash may leave a Host-private staging orphan, which restart cleanup may remove, but the formal namespace remains exactly absent-or-final. The nonroot recorder never receives either root/path/fd. Host enumeration selects every publish/repair receipt/blob; the repair API accepts only evidence/through-attempt identity plus prebound policy capabilities, never a caller receipt, destination, or root.

Only an absent leaf is repairable. Existing mismatch, symlink, directory, extra namespace entry, changed root identity, or permission drift fails closed and is never replaced or quarantined. Fault points include before staging create, after staging fsync, before final link, after final link-before-directory-fsync, after fsync, and before reply. Kill/reopen tests require the complete formal directory to contain exactly no leaf or the byte-identical final leaf, with no temporary/extra entry, and never append a second receipt/authority.

- [ ] **Step 4: Verify and commit**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.AppendPlatformProjectionTests
git diff --check
git add -- scripts/tests/append_platform_host_model.py scripts/tests/test_append_platform.py
git commit -m "test(evidence): prove Host projection repair"
```

## Task 13: Add The Isolated Thin Recorder CLI

**Files:**

- Create: `scripts/implementation_evidence/record_orchestration.py`
- Modify: `scripts/tests/test_append_platform.py`

- [ ] **Step 1: Add CLI RED tests**

Add `RecordOrchestrationCliTests`:

- `test_root_internal_recovery_abandon_failure_and_barrier_have_no_nonroot_cli_or_wire_arm`
- `test_record_orchestration_append_platform_accepts_only_three_handles_and_host_fd`
- `test_record_orchestration_never_invokes_git_subprocess_or_reads_authority_env`
- `test_record_orchestration_never_opens_or_writes_a_projection_path`
- `test_record_orchestration_isolated_runtime_and_exact_harness_imports`
- `test_every_rejection_has_empty_stdout_bounded_redacted_stderr`
- `test_diagnostics_never_contain_handle_body_session_authority_path_or_environment_canaries`
- `test_frame_and_record_limits_fail_before_allocation_or_state_lookup`
- `test_invalid_utf8_duplicate_json_trailing_frames_and_slow_drip_are_bounded`

Probe missing/duplicate/misplaced options; capture-context/attest/seal/internal-operation names; body/candidate/parent/path/model/identity/nonce/policy/receipt/test-mode overrides; fd `<3`, bool, non-integer, and oversized integer; hostile `PYTHONPATH`, `PYTHONSTARTUP`, and `sitecustomize` markers; multi-byte diagnostic input; private canaries.

- [ ] **Step 2: Run RED**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.RecordOrchestrationCliTests
```

Expected: `FAIL` because `record_orchestration.py` is absent. It must not fail because the socket fixture is invalid.

- [ ] **Step 3: Implement the thin entrypoint**

Mirror the exact-path trusted-core loader, duplicate-option action, signed-int32 FD parser, UTF-8 byte-bounded diagnostics, socket cleanup, and `-I -S -B` runtime guard from `host_envelope_adapter.py`. Expose only:

```text
append-platform
--context-handle H2
--invocation-handle HI
--response-handle HR
--host-store-fd FD
```

`main(argv)` authenticates the FD, calls `append_platform_on_authenticated_socket`, closes the socket exactly once on success/failure, and writes one canonical result only after complete reply EOF. It imports no subprocess/Git module, reads no authority environment variable, opens no pathname, and writes no projection.

- [ ] **Step 4: Verify isolated and in-process paths**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_append_platform.RecordOrchestrationCliTests
TERSH_RECORD_HELP="$(/usr/bin/python3 -I -S -B scripts/implementation_evidence/record_orchestration.py --help)" || exit 1
test -n "$TERSH_RECORD_HELP" || exit 1
unset TERSH_RECORD_HELP
```

Expected: tests pass and isolated help exits zero without writing a predictable temporary path.

- [ ] **Step 5: Commit**

```bash
git diff --check
git add -- scripts/implementation_evidence/record_orchestration.py scripts/tests/test_append_platform.py
git commit -m "feat(evidence): add isolated append recorder"
```

## Task 14: Full Regression, Stress, Review, And Evidence Handoff

**Files:**

- Review all files changed by Tasks 1–13.
- Modify only a file whose verification exposes a real defect, with a fresh semantic RED before the fix.

- [ ] **Step 1: Run syntax and complete Python regression**

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B - <<'PY'
import ast
from pathlib import Path

for name in (
    "scripts/evidence_core.py",
    "scripts/implementation_evidence/host_envelope_adapter.py",
    "scripts/implementation_evidence/record_orchestration.py",
    "scripts/tests/append_platform_host_model.py",
    "scripts/tests/test_implementation_evidence.py",
    "scripts/tests/test_append_platform.py",
):
    ast.parse(Path(name).read_text(encoding="utf-8"), filename=name)
PY

PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -v \
  scripts.tests.test_run_exact_test \
  scripts.tests.test_implementation_evidence \
  scripts.tests.test_append_platform
```

Expected: syntax and all Python tests pass with no skip-as-pass privileged positive.

- [ ] **Step 2: Run repeated state/race/stability gates**

```bash
for iteration in $(jot 20 1); do
  PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest -q \
    scripts.tests.test_append_platform.AppendPlatformAtomicityTests \
    scripts.tests.test_append_platform.AppendPlatformRecoveryTests \
    scripts.tests.test_append_platform.AppendPlatformFailureTests \
    scripts.tests.test_append_platform.AppendPlatformBarrierTests || exit 1
done
```

Expected: 20/20 deterministic passes with no hang, timeout, flaky race, resource warning, or leaked thread/socket/fd.

- [ ] **Step 3: Run repository-wide Rust and diff hygiene gates**

```bash
cargo fmt --check
cargo test --locked --all-targets
git diff --check
find scripts -type d -name __pycache__ -print
find scripts -type f \( -name '*.pyc' -o -name '*.pyo' \) -print
git status --short
```

Expected: formatting and Rust tests pass; diff check passes; both cache searches print nothing; worktree is clean after the final task commit.

- [ ] **Step 4: Run three independent reviews**

Dispatch three read-only reviewers against one frozen diff/commit:

1. code/API correctness and regression review;
2. adversarial security/trust-boundary review;
3. test-quality/falsifiability/fault-model review.

Require each reviewer to recompute the same file/commit hashes before and after review and report no open P0/P1 or Critical/Important finding. For every valid finding, add a semantic RED, implement the minimum correction, rerun the focused and complete gates, and obtain fresh reviews of the new frozen bytes.

- [ ] **Step 5: Record the honest completion boundary**

The handoff summary must say:

- repository client, schemas, and reference Host model implemented;
- tests and exact commands/results;
- no ResearchOS experiment/run/artifact or registry changes;
- no production UID-0 Host, root projection installation, durable real ledger, or custom-runner acceptance;
- formal evidence remains fail-closed until those external components exist;
- operator attestation, producer batch, sealer, manifest/closure, formal query, and production projection-repair wire remain deferred.

Do not create a verification-only cleanup commit. Commit only if this task discovers and fixes a real defect through a new RED.

## Acceptance Checklist

- [ ] The approved design hash is unchanged.
- [ ] The two older plans contain no normative three-BODY append, formal attest arm, or client-written formal projection.
- [ ] Exact context v2 is required across capture, provenance, and append; v1 and mixed shapes fail.
- [ ] Parent arrays are bounded, sorted, same-evidence, pre-spawn, and resolvable only to unique prior Host-receipted sources.
- [ ] Session, record, BODY wrapper, COMMIT facts, result, and receipt are closed and independently joined.
- [ ] Production client invokes no Git/subprocess and accepts no authority/path/body selector.
- [ ] Test Host constructor is independent of the production record derivation function.
- [ ] One live H2+HI+HR generation yields at most one durable blob/receipt/conditional authority.
- [ ] Every fault exposes exactly PRE_LINEARIZATION or COMMITTED_DURABLE state.
- [ ] Recovery, abandonment, failure, supersede, sibling drain, and successor races have one serializable winner.
- [ ] Formal projection tests use a Host-exclusive capability and repair only an absent exact leaf.
- [ ] CLI is isolated, exact, bounded, redacted, and closes resources on all paths.
- [ ] All exact tests from the approved design are present and falsifiable.
- [ ] Full Python/Rust regression and 20x state stress pass.
- [ ] Three independent frozen-byte reviews have no open threshold finding.
- [ ] No socketpair/reference model is represented as privileged formal acceptance.
