# Tersh Implementation Iteration Evidence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Execute and prove the seven Trusted Core feature iterations as seven ordered, independently reviewable candidate commits with exact gates, append-only five-role provenance, and evidence-only closure commits.

**Architecture:** Build one Python-standard-library evidence core before feature work, then treat Plans 1–5 as component recipe catalogs rather than an execution order. Each iteration freezes a clean committed candidate, resolves a committed cumulative gate catalog, reruns its slice plus every prior accepted special gate, and closes five independent review roles on that same candidate. One shared external-candidate CLI inventories required online custom runners, creates a unique `codex/evidence/**` push ref, binds the resulting CI/release `push` runs by numeric ID, and writes append-only attempt evidence; manual dispatch remains an operator recovery path but is never the first acceptance bootstrap.

**Tech Stack:** Python 3 standard library, Git, GitHub CLI, Rust/Cargo locked tests, canonical JSON, SHA-256, append-only reviewer records, and the component plans under `docs/superpowers/plans/`.

---

## Authority, Order, And Non-Goals

This plan is the sole execution order for the seven implementation iterations required by design lines 1244–1261 and 1476–1505. The numbered tasks inside Plans 1–5 remain test-driven component recipes. A component commit is not a slice acceptance event, and a component plan's local GREEN state is not an implementation-cycle manifest.

The immutable order is:

1. `impl-01`: G0a release/install truth.
2. `impl-02`: G0b existing interaction/result truth.
3. `impl-03`: G1a latest-wins read responsiveness.
4. `impl-04`: G1b serial mutation truth, fixed state root, and source-claim substrate.
5. `impl-05`: G2 CLI trash/restore engine.
6. `impl-06`: G1c limited EXDEV plus G2 Recovery TUI and integrated G2 acceptance.
7. `impl-07`: G3 bounded cluster companion correctness and integrated feature review.

This plan does not implement product behavior, create a generic job runtime, replace component TDD, publish a public release, or perform the seven later hardening cycles. It records implementation evidence only. A public release or tag remains a separate explicitly authorized action.

## Locked Component Slices

| Iteration | Component recipes, in execution order | Candidate boundary |
| --- | --- | --- |
| `impl-01` | Plan1 Task1; Task6a; Task7a; Task8; Task9 local/tooling steps; Task10a | Task10a documentation commit |
| `impl-02` | Plan1 Tasks2–5; Task6b; Task7b; Task10b | Task10b documentation commit |
| `impl-03` | Plan2 Tasks1–5, including Task5's frozen `tersh-plan2-read-bench` reference gate; using Task1 only as the fd-relative/read-identity substrate required by G1a | Task5 read-candidate component commit plus its unchanged benchmark artifact |
| `impl-04` | Plan2 Tasks6–13 | Plan2 G1b acceptance-candidate commit |
| `impl-05` | Plan3 Tasks1–7 | Plan3 CLI recovery candidate commit |
| `impl-06` | Plan4 Tasks1–6 | Plan4 integrated G1c/G2 candidate commit |
| `impl-07` | Plan5 Tasks1–6, including any production correction exposed by its final matrix before documentation | Plan5 G3 candidate commit |

No executor may run Plan1 Task6b or Task7b before `impl-01` is closed. Plan1 Task9's downloaded-binary gate occurs only after Task10a is committed and is run solely by this plan's external helper. No executor may run Plan1 Task10b as part of `impl-01`. Plan2 Task13's mutation benchmark remains in `impl-04`; it consumes but never replaces the frozen Task5 G1a read artifact. Plan5 Task6 does not close `impl-07` by itself; only this plan's manifest does.

## Shared Files And Responsibilities

| File | Single responsibility |
| --- | --- |
| `scripts/evidence_core.py` | Shared canonical JSON, bounded drain/hash, exact candidate/run/job validation, append-only review parsing, and closure primitives used by implementation and later hardening wrappers |
| `scripts/run_exact_test.py` | List one exact Rust integration or private library test, require one discovery and one execution, and validate an optional frozen parameter-case ID list |
| `scripts/tests/test_run_exact_test.py` | Exact discovery/execution, ignored/serial, malformed summary, and frozen case-matrix tests |
| `scripts/implementation_evidence/run_gate.py` | Run one argv without a shell, drain/hash bounded output, and atomically write one canonical gate record |
| `scripts/implementation_evidence/host_envelope_adapter.py` | Host-only context/invocation/response ingress: require a distinct-principal peer-credential-authenticated `AF_UNIX/SOCK_STREAM` FD, create-new store the closed envelope outside the agent sandbox, and return only an opaque single-use handle |
| `scripts/implementation_evidence/record_orchestration.py` | Consume an orchestrator-fixed context plus single-use host-owned invocation/response handles, or record an explicitly labeled model/effort attestation only when the immutable host identity/lifecycle response still exists |
| `scripts/implementation_evidence/finalize_iteration.py` | Enumerate every attempt and per-commit subtree for one evidence ID, validate orchestration/reviews/gates and the accepting candidate, and emit one canonical iteration manifest |
| `scripts/implementation_evidence/run_external_candidate.py` | Perform one runner-inventory/repository-wide run-ID snapshot/unique-push bootstrap, bind all requested workflow runs, enforce one remaining-deadline budget, cancel bound nonterminal runs on every failure, and record exact artifacts/results |
| `scripts/implementation_evidence/verify_ci_evidence.py` | Verify exact committed candidate, workflow attempt, and unique successful CI job IDs |
| `scripts/implementation_evidence/verify_release_candidate.py` | Verify exact release job IDs, AssetDescriptor/SmokeEvidence/manifest chain, downloaded artifact hashes, and downloaded-binary READY identity |
| `scripts/implementation_evidence/gate_catalog.json` | Frozen per-iteration cumulative local/external gate manifest, including matrices, ignored/serial/native tests and reference benchmarks |
| `scripts/implementation_evidence/run_cumulative_gates.py` | Resolve catalog inheritance through one iteration and execute every locked local gate create-new; reject missing or changed prior gates |
| `scripts/tests/test_implementation_evidence.py` | Canonicality, output bounds, append-only attempt layout, cumulative catalogs, unique push selection, deadlines/cancel, and closure tests |
| `target/implementation-evidence/impl-{01,02,03,04,05,06,07}/attempt-NNN/candidate-SHA/` | Ignored create-new per-commit root: gates/REST/artifacts live under `run-BINDING/`, while shared-schema per-file `orchestration/` and `reviews/` live directly under the candidate; failed attempts and superseded commits are never overwritten |
| `docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-{01,02,03,04,05,06,07}.json` | Seven committed canonical closure manifests, one per fixed iteration |

## Locked Evidence Schemas And Five-Role Waves

`run_gate.py` writes schema `tersh-implementation-gate-v1` with iteration,
three-character string evidence attempt, run binding, gate name, argv array, cwd, exact 40-hex
candidate, UTC start/end, duration milliseconds, exit code, stdout/stderr byte
counts and SHA-256 hashes, retained-log paths, OS, architecture, rustc/cargo
versions, and optional exact-test inventory. It retains at most 1 MiB per
stream while continuing to drain and hash excess bytes; each stream records its
complete byte count/hash and retained-prefix byte count/hash. Without
`--allow-failure`, it exits with the child's status. A final path is published
create-new: write/fsync a mode-0600 temporary in the same directory, hard-link
it to the absent final name, fsync the parent, then unlink the temporary. An
existing final path is an error; no entrypoint uses replace semantics.

Every raw path follows this closed layout. `attempt-NNN/` is independent of a
candidate; a baseline, an intermediate Wave B commit, and the final candidate
receive distinct immutable `candidate-SHA/` subtrees:

```text
target/implementation-evidence/EVIDENCE-ID/
  attempt-NNN/
    runner-inventory.json
    before-repo-runs.json
    candidate-SHA/
      run-local/gates/GATE.{json,stdout,stderr}
      run-unregistered-KIND/bootstrap.json
      run-RUN_ID/KIND/{selected-run.json,jobs.json,artifacts/,artifact-index.json}
      run-set-KIND-BINDING[-KIND-BINDING...]/external-candidate.json
      orchestration/ROLE.WAVE.REVIEW_ATTEMPT.json
      reviews/ROLE.WAVE.REVIEW_ATTEMPT.json
```

`EVIDENCE-ID` matches exactly `^(?:impl|hardening)-0[1-7]$`; implementation
finalizers accept only `impl-01` through `impl-07`, while the same helper also
serves `hardening-01` through `hardening-07`. `NNN` matches exactly
`^(?:00[1-9]|0[1-9][0-9]|[1-9][0-9]{2})$` (`001` through `999`);
`SHA` is a full lowercase commit; and every numeric `RUN_ID` is positive.
Every canonical schema names the NNN value `evidence_attempt` and stores it as
that three-character JSON string; no record exposes an ambiguous numeric
`attempt` field. GitHub's positive JSON integer remains separately named
`run_attempt`, and review records separately use their three-character
`review_attempt` string.
`BINDING` is either the kind's positive run ID or literal `unregistered`, so a
partial-registration failure still has one deterministic sorted combined
manifest, for example
`run-set-ci-123-release-unregistered/external-candidate.json`.
The shared review/orchestration `RUN-BINDING` matches exactly
`^(?:run-local|run-cumulative|run-set-(?:ci-(?:[1-9][0-9]*|unregistered)(?:-release-(?:[1-9][0-9]*|unregistered))?|release-(?:[1-9][0-9]*|unregistered)))$`.
This is the complete lexicographically ordered `ci`/`release` vocabulary used by
these plans; it rejects an empty component, doubled/trailing hyphen, duplicate or
reordered kind, zero run ID, and unknown kind.
`run-unregistered-KIND` may contain only pre-registration bootstrap/failure
evidence. The repository-wide pre-push ID snapshot is stored once per attempt
and referenced by hash from each kind's bootstrap record; no before snapshot
requires a workflow-path endpoint. `run-local` contains no external assertion.
Directories and files are create-new. A retry allocates the next attempt and
preserves the complete failed attempt; multiple commits in an attempt preserve
separate candidate subtrees and never remove, rename, truncate, or rewrite an
earlier record.

`GATE` matches `^[a-z][a-z0-9-]{0,63}$`, and every gate is exactly one closed
three-file set `GATE.json`, `GATE.stdout`, and `GATE.stderr`; a missing member,
fourth sibling, symlink, or mismatched basename fails. Both log files are always
created, including for an empty stream, and retain only the first 1 MiB. The
JSON stream object records `total_bytes`, the SHA-256 of the complete drained
stream, `retained_bytes`, the SHA-256 of the retained prefix, and the matching
candidate-relative retained-log path. Finalization validates each triplet and
embeds the canonical `GATE.json` body/hash, but deliberately does not embed raw
stdout/stderr bytes in the committed closure manifest.

Each file beneath `orchestration/` uses the shared implementation/hardening
schema `tersh-evidence-orchestration-v1` and contains exactly one append-only
entry: orchestrator-issued agent ID, canonical task path, agent run ID, model,
reasoning effort, dispatch/start/end RFC 3339 timestamps, role, wave,
three-character review attempt, evidence attempt, run binding, baseline commit, reviewed commit,
parent finding IDs, and the closed provenance object below. `evidence_attempt` is always the three-character JSON
string `001` through `999`; GitHub `run_attempt` is a separate positive JSON
integer and the two fields are never coerced or compared as one namespace.
`canonical_task_path` matches
`^/root(?:/[a-z][a-z0-9_]{0,63})+$`; orchestrator-issued `agent_id` and
`agent_run_id` each match `^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$` and are
byte-for-byte equal to the orchestration envelope rather than reviewer input.
Orchestration and review filenames share the closed
grammar `ROLE.WAVE.REVIEW_ATTEMPT.json`, where ROLE is one of
`product|architecture|implementation|safety|verification`, WAVE is one of
`wave-a|wave-b|wave-c|closure-a|closure-b`, and REVIEW_ATTEMPT matches exactly
`^(?:00[1-9]|0[1-9][0-9]|[1-9][0-9]{2})$`. Every entry and matching review
must state model exactly `gpt-5.6-sol` and reasoning effort exactly `xhigh`;
this includes the Wave B implementation writer as well as every read-only
reviewer. Files are created with create-new semantics; no aggregate
orchestration file is rewritten.

Each review uses the shared schema `tersh-evidence-review-v1` and repeats the
matching orchestration identity plus verdict, checked requirement IDs,
findings, parent finding references, resolution references, direct gate-record
hashes, and commands. It is limited to 256 KiB. Each finding is the closed
object `{finding_id,severity,requirement,file,line,counterexample,required_correction}`
with no extra keys, a stable evidence-ID-scoped ID, and exact severity
`P0|P1|P2`. Parent references are only IDs under the shared union grammar below;
resolutions are only the closed objects defined below. Review types have no overwrite path.
Wave A binds its baseline as the reviewed commit; Wave B binds each concrete
implementation commit it produces, with both records stored in the matching
per-commit subtree. Only Wave C, Closure A/B, local/external gates, and
finalization must share the final candidate SHA. Historical Wave A/B records
remain append-only and are never relabeled as reviews of a future commit.

Each iteration executes these waves, never exceeding three concurrent reviewers:

1. Wave A: product, architecture, and implementation diagnosis on one baseline/candidate.
2. Wave B: one implementation writer applies the smallest correction and appends an execution report for every attempt.
3. Wave C: independent safety and verification reviews on the corrected candidate.
4. Closure A: product, architecture, and implementation final reports on the identical candidate.
5. Closure B: safety and verification final reports on that same candidate.

Every dispatch in all five waves explicitly requests model `gpt-5.6-sol` with
reasoning effort `xhigh`; inheritance, a default model, or a self-declared report
field is insufficient. Provenance is honest about what the host exposes. Before
dispatch, the platform-owned supervisor creates an immutable context; its host
adapter stores the actual terminal spawn result in a private mode-0600,
create-new response-envelope store outside the agent/operator sandbox and under
a distinct OS principal. That store contains the host-issued agent ID,
canonical task path, agent run ID, start/end timestamps, terminal status, and a
nullable host-returned reported-result-commit field. The recorder, not that
field, observes the clean worktree commit and compares it when present. Neither
the agent nor an operator can address, write, or edit the store. Context,
invocation, and response handles are single-consumption; immutable receipts are
read-only/queryable through closure so a failed finalizer retry cannot erase
provenance. If this supervisor boundary is unavailable, neither provenance mode
is evidence-bearing.

When the platform also supplies immutable requested/selected model metadata, the
adapter stores one spawn-invocation envelope and the recorder compares its
requested/selected model and effort while joining the separate response identity,
hashes all three canonical
envelopes, embeds each complete canonical body plus its hash, and appends
provenance mode `platform-envelope`. In that mode there is no model, effort,
identity, timestamp, output-path, or override value supplied through CLI or
environment. If and only if the host-owned response envelope has trustworthy
identity/lifecycle fields but the platform omits trustworthy model/effort
metadata, the recorder consumes that response handle and appends provenance mode
`operator-attestation` with the operator identity, exact requested
`gpt-5.6-sol`/`xhigh` from the fixed context, attestation timestamp, and reason
`platform-model-metadata-unavailable`. The finalizer may use that explicit
attestation to enforce the requested dispatch contract, but must label it as
operator-attested and must not claim the platform independently verified the
model. If the host cannot provide the immutable identity/lifecycle response
envelope, the dispatch is not evidence-bearing and finalization fails; the
recorder never fabricates or claims to observe absent data. Reviewer report JSON
is never an envelope or attestation source in either mode.

The shared schema's `provenance` is an exact tagged union with no extra fields.
Both variants contain `mode`, 64-lowercase-hex `host_receipt_id`,
`context: {body, sha256}`, and
`response: {body, sha256}`; each `body` is the complete canonical host envelope
and `sha256` is its 64-lowercase-hex digest. `platform-envelope` additionally
contains `invocation: {body, sha256}` and nothing attestation-specific.
`operator-attestation` instead contains `operator_id`, `attested_at`, fixed
`reason: "platform-model-metadata-unavailable"`,
`requested_model: "gpt-5.6-sol"`, and `requested_reasoning_effort: "xhigh"`,
and contains no invocation or platform-verified-model claim. `operator_id` uses
the shared agent-ID grammar and `attested_at` uses the timestamp grammar below.

The three embedded canonical bodies have these exact closed field sets and no
aliases or extra keys:

- Context: `{schema,context_nonce,evidence_id,evidence_attempt,role,wave,review_attempt,run_binding,baseline_commit,review_target,canonical_task_path,worktree_handle,requested_model,requested_reasoning_effort,created_at}` with schema `tersh-host-dispatch-context-v1`.
- Invocation: `{schema,context_nonce,dispatch_id,requested_model,requested_reasoning_effort,selected_model,selected_reasoning_effort,dispatched_at}` with schema `tersh-host-spawn-invocation-v1`.
- Response: `{schema,context_nonce,dispatch_id,agent_id,canonical_task_path,agent_run_id,started_at,ended_at,terminal_status,reported_result_commit}` with schema `tersh-host-spawn-response-v1`.

`context_nonce` and `dispatch_id` are 64 lowercase hex; evidence/review attempts,
IDs, roles, waves, run bindings, commits, task path, and agent IDs use the shared
grammars in this section. `worktree_handle` matches
`^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$`; `review_target` and
`reported_result_commit` are JSON null or a full lowercase commit.
`requested_model` and `selected_model` are exactly `gpt-5.6-sol`, and both effort
fields are exactly `xhigh`. Every timestamp matches UTC RFC 3339
`^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{1,9}Z$`.
Platform mode requires
`created_at <= dispatched_at <= started_at <= ended_at`; attestation mode
requires `created_at <= started_at <= ended_at <= attested_at`. `terminal_status` is
exactly `completed|failed|cancelled|interrupted`. All three bodies repeat one
nonce, invocation/response repeat one dispatch ID, response task identity equals
context task identity, and every duplicated requested field is byte-identical.
A missing body/hash, mismatched digest/nonce/dispatch ID, unknown key/mode,
invalid ordering, or both union arms in one object fails. Because the full
context body is embedded, `--verify-only` recomputes all three available hashes
without the host store.

Finding IDs match exactly
`^(?:impl|hardening)-0[1-7]-F(?:00[1-9]|0[1-9][0-9]|[1-9][0-9]{2})$`;
the captured evidence ID must equal the containing evidence tree, and IDs are
unique and monotonically allocated within it. A
`parent_finding_id` must match that same grammar and evidence ID and precede the
child numerically. A resolution is the closed object `{finding_id,
correcting_commit, verifying_review_ref}`. `correcting_commit` is a full
lowercase commit and `verifying_review_ref` is the closed object
`{evidence_attempt, candidate, run_binding, review_file,
review_body_sha256}`. Its `evidence_attempt` is the three-character string
grammar above, `candidate` is a full lowercase commit, `run_binding` matches
the exact shared `RUN-BINDING` grammar above, `review_file`
matches exactly the shared `ROLE.WAVE.REVIEW_ATTEMPT.json` grammar, and
`review_body_sha256` is 64 lowercase hexadecimal characters. The finalizer
resolves exactly
`attempt-EVIDENCE_ATTEMPT/candidate-CANDIDATE/reviews/REVIEW_FILE` beneath the
same evidence-ID root, opens it no-follow, requires its canonical body hash and
body `run_binding` to equal the reference, and requires that body to name the
same finding and record the correction as verified. A referenced review cannot
be the file containing the resolution, so its body hash is never
self-referential. The referenced attempt/candidate must precede or equal the
resolving report in finalizer history. Free-form finding, parent, resolution, or
review-reference strings are rejected.

Any Closure A/B finding returns to Wave B. A source, test, script, workflow, or product-documentation change after candidate freeze closes the current evidence attempt as failed, creates a new candidate commit in the next three-digit attempt, and invalidates prior external runs and closure reports without overwriting them. Pre-freeze Wave A/B commits remain separate subtrees in the current attempt. Wave A/B reports bind `run-local`; Wave C and closure bind the exact final `run-set`. The finalizer starts at the evidence-ID root, enumerates every attempt from `001` through the accepting attempt and every canonical candidate subtree inside each, and embeds in order the complete canonical body and SHA-256 of every present attempt-level `runner-inventory.json` and `before-repo-runs.json`, and every orchestration, review, gate JSON, bootstrap, selected-run, `jobs.json`, artifact-index, and combined external manifest. Runner inventory is required once external preflight starts; the before-run snapshot is required after that inventory passes and forbidden when inventory fails before snapshot, so state-justified absence is not silently filled. It validates but never embeds the bounded raw gate logs. It rejects an attempt gap, a noncanonical extra path, a missing state-required or superseded record, reused path, task/run identity mismatch, wrong model/reasoning or dishonest provenance claim, accepting-candidate drift, missing cumulative/catalog/direct-gate hashes, any final FAIL, or any unresolved P0/P1; it never stores only hashes and discards canonical raw history.

The candidate worktree must be clean before any exact-SHA gate or final review:

```bash
test -z "$(git status --porcelain=v1 --untracked-files=all)"
TERSH_IMPL_CANDIDATE="$(git rev-parse HEAD)"
test "$(printf '%s' "$TERSH_IMPL_CANDIDATE" | wc -c | tr -d ' ')" = 40
git cat-file -e "$TERSH_IMPL_CANDIDATE^{commit}"
```

Raw evidence lives below ignored `target/`; it does not dirty the candidate.
After finalization, exactly one new evidence manifest may be present. Tasks2–8
each contain their literal accepted path and evidence-only staged-file check; no
free-form output path is accepted.

### Task 1: Build And Commit The Shared Exact-Test And Evidence Harness

**Files:**

- Create: `scripts/run_exact_test.py`
- Create: `scripts/tests/test_run_exact_test.py`
- Create: `scripts/evidence_core.py`
- Create: `scripts/implementation_evidence/run_gate.py`
- Create: `scripts/implementation_evidence/host_envelope_adapter.py`
- Create: `scripts/implementation_evidence/record_orchestration.py`
- Create: `scripts/implementation_evidence/finalize_iteration.py`
- Create: `scripts/implementation_evidence/run_external_candidate.py`
- Create: `scripts/implementation_evidence/verify_ci_evidence.py`
- Create: `scripts/implementation_evidence/verify_release_candidate.py`
- Create: `scripts/implementation_evidence/gate_catalog.json`
- Create: `scripts/implementation_evidence/run_cumulative_gates.py`
- Create: `scripts/tests/test_implementation_evidence.py`
- Modify: `.gitignore:2` — retain `/target/` and add the explicit anchored
  `/target/implementation-evidence/` rule used by the clean-candidate checks

- [ ] **Step 1: Write the failing exact-runner and evidence tests**

Add exact Python unittest methods:

- `test_exact_runner_requires_one_discovered_and_one_executed`
- `test_exact_runner_rejects_missing_duplicate_ignored_or_zero_without_explicit_flags`
- `test_exact_runner_passes_argv_without_a_shell_and_serializes_when_requested`
- `test_exact_runner_supports_private_lib_tests_and_rejects_mixed_selectors`
- `test_exact_runner_lib_lists_then_exactly_executes_one_crate_private_test`
- `test_exact_runner_rejects_both_or_neither_test_and_lib_selector`
- `test_exact_runner_lib_rejects_zero_discovered_or_zero_executed`
- `test_exact_runner_always_uses_nocapture`
- `test_exact_runner_validates_frozen_parameter_case_ids_and_count`
- `test_exact_runner_requires_exactly_one_case_record_and_rejects_it_without_matrix`
- `test_exact_runner_rejects_missing_duplicate_extra_or_reordered_case_ids`
- `test_run_gate_drains_hashes_and_caps_both_streams`
- `test_run_gate_preserves_child_status_candidate_attempt_and_run_binding`
- `test_every_raw_record_is_create_new_beneath_attempt_candidate_and_run_binding`
- `test_evidence_id_union_accepts_only_impl_or_hardening_01_through_07`
- `test_attempt_root_is_candidate_independent_and_per_commit_records_are_immutable`
- `test_external_preflight_rejects_zero_or_missing_online_custom_runners_before_push`
- `test_external_bootstrap_pages_repository_run_ids_without_workflow_path_lookup`
- `test_external_first_push_succeeds_when_candidate_workflow_path_is_absent_on_default_branch`
- `test_external_bootstrap_requires_absent_unique_ref_and_one_exact_push_run_per_workflow`
- `test_external_selector_rejects_dispatch_wrong_head_branch_path_pre_snapshot_or_preexisting_run`
- `test_external_selector_requires_exact_bare_workflow_path_and_candidate_blob`
- `test_external_selector_rejects_path_at_ref_qualified_or_other_workflow_path`
- `test_external_registration_and_completion_deadlines_never_pass`
- `test_external_timeout_cancels_only_each_bound_numeric_run_id`
- `test_partial_registration_failure_cancels_each_bound_nonterminal_run_once`
- `test_interrupt_cancels_bound_nonterminal_runs_and_records_combined_failure`
- `test_cancel_records_request_then_drains_each_bound_run_to_terminal_observation`
- `test_cancel_fails_when_reserved_cleanup_budget_cannot_observe_terminal_state`
- `test_git_gh_download_and_verifier_subprocesses_use_remaining_global_deadline`
- `test_hung_download_is_terminated_reaped_recorded_and_never_passes`
- `test_external_result_embeds_run_attempt_jobs_artifacts_and_cancel_truth`
- `test_external_artifacts_require_exact_producer_template_schema_and_nonempty_set`
- `test_external_artifacts_reject_missing_empty_renamed_wrong_run_attempt_schema_or_extra`
- `test_artifact_index_hashes_manifest_that_excludes_itself_and_rejects_unlisted_payload`
- `test_artifact_producer_join_binds_unique_pinned_upload_step_artifact_id_name_and_job_log`
- `test_artifact_producer_log_unwraps_only_optional_initial_bom_and_rfc3339z_prefix`
- `test_artifact_producer_digest_normalizes_bare_upload_output_to_rest_sha256`
- `test_self_declared_or_expected_producer_without_runtime_join_never_passes`
- `test_shared_artifact_caller_map_requires_native_pair_for_hardening_04`
- `test_cumulative_catalog_replays_prior_matrix_ignored_serial_native_and_benchmark_gates`
- `test_cumulative_catalog_rejects_removed_renamed_or_zero_inventory_gate`
- `test_cumulative_catalog_substitutes_only_closed_whole_token_placeholders`
- `test_cumulative_catalog_rejects_embedded_unknown_dollar_or_shell_placeholders`
- `test_ci_verifier_requires_unique_exact_successful_job_ids`
- `test_release_verifier_requires_descriptor_smoke_manifest_and_ready_identity`
- `test_finalize_requires_wave_a_b_c_and_five_role_closure`
- `test_finalize_crosschecks_orchestrator_agent_task_and_run_ids`
- `test_orchestration_attempt_is_three_character_string_and_run_attempt_is_positive_integer`
- `test_orchestration_rejects_noncanonical_task_agent_finding_parent_or_resolution_refs`
- `test_run_binding_rejects_empty_doubled_trailing_duplicate_reordered_zero_or_unknown_components`
- `test_resolution_ref_binds_attempt_candidate_run_file_and_canonical_body_hash`
- `test_host_envelope_adapter_requires_distinct_peer_credential_unix_stream_socket`
- `test_host_envelope_adapter_rejects_same_principal_fifo_stdin_regular_file_trailing_bytes_and_reuse`
- `test_host_provenance_preflight_fails_without_distinct_supervisor`
- `test_context_invocation_response_envelope_schemas_are_closed_typed_and_nonce_bound`
- `test_finalizer_requires_host_receipt_body_hashes_and_destination_binding`
- `test_verify_only_rehashes_embedded_context_invocation_and_response_without_host_store`
- `test_platform_recorder_consumes_one_host_owned_invocation_and_response_handle_pair`
- `test_operator_attestation_requires_host_response_handle_when_platform_model_metadata_is_unavailable`
- `test_operator_attestation_rejects_identity_timestamp_model_effort_and_output_inputs`
- `test_provenance_tagged_union_rejects_extra_fields_mixed_arms_or_unhashed_bodies`
- `test_orchestration_fails_when_host_identity_response_is_unavailable`
- `test_reviewer_report_cannot_self_attest_model_or_reasoning_effort`
- `test_finalize_embeds_every_append_only_review_body_and_hash`
- `test_finalize_embeds_runner_before_jobs_and_all_canonical_json_bodies`
- `test_gate_record_requires_exact_json_stdout_stderr_triplet_but_closure_omits_raw_logs`
- `test_finalize_enumerates_every_attempt_and_per_commit_subtree_from_evidence_root`
- `test_finalize_rejects_attempt_gaps_unindexed_extras_or_missing_superseded_records`
- `test_finalize_requires_gpt_5_6_sol_xhigh_for_every_role_and_wave`
- `test_finalize_rejects_candidate_drift_unresolved_p0_p1_and_missing_gate`
- `test_finalize_rejects_zero_test_inventory_and_non_evidence_tree_changes`

The fake Cargo fixture independently emulates integration `--test TARGET` and
crate-private `--lib` list/execution argv, emits libtest summaries, and records
the literal argv so tests can require the exact selector, full-name filter,
`--exact`, and `--nocapture`; both/neither selectors and zero list/execution
counts are negative fixtures. A parameterized fixture
emits exactly one line beginning `tersh-case-count-v1 ` followed by canonical
JSON with `matrix`, `expected_ids`, `executed_ids`, `expected_count`, and
`executed_count`; zero or two records fail. External tests use temporary bare
remotes plus synthetic paginated repository-run, runner, run, job, log, and
artifact responses and a fake `gh` argv endpoint. Repository-run fixtures use
the real REST shape
`"path":".github/workflows/ci.yml"` together with the exact push
`head_branch`, `head_sha`, and `event`; they include negative `@main`,
`@refs/heads/...`, tag/SHA-qualified, other-path, wrong-blob, and near-path
cases. Job-log fixtures use the platform bytes actually returned by the logs
endpoint: an optional UTF-8 BOM only at byte zero, then one RFC 3339 UTC
timestamp, one ASCII space, and the payload on every line. The producer fixture
uses a bare 64-lowercase-hex `upload-artifact` output and a
`sha256:`-prefixed REST digest for the same bytes. One fixture returns 404 for both
workflow-path endpoints to prove the helper never calls them before the first
candidate push. Host-envelope fixtures use an `AF_UNIX/SOCK_STREAM` socketpair and a fake
supervisor/store adapter whose internal API pins a distinct synthetic peer
principal; production exposes no expected-principal override. Same-principal,
FIFO/plain-pipe, regular-file, stdin, wrong-nonce, trailing-byte, replay,
caller-JSON, closed-schema, receipt-binding, and missing-supervisor cases all
fail. Hung child
fixtures cover Git, `gh`, artifact download, and
verifier calls. Tests never contact GitHub and prove no push occurs when the
online inventory is empty, partial registration cancels only already bound
nonterminal IDs once, cancellation drains every bound run to a terminal
observation within the reserved cleanup budget, and every failure still
publishes one combined manifest.

- [ ] **Step 2: Run the harness tests and confirm RED**

Run:

```bash
python3 -m unittest scripts.tests.test_run_exact_test scripts.tests.test_implementation_evidence -v
```

Expected: FAIL because the shared scripts do not exist.

- [ ] **Step 3: Implement the exact CLI contracts**

`scripts/run_exact_test.py` accepts:

```text
python3 scripts/run_exact_test.py (--test TARGET | --lib) --name FULL_NAME [--ignored] [--serial] [--case-matrix MATRIX --expect-case CASE_ID ...] [--cargo-bin PATH]
```

It first invokes exactly one of
`cargo test --locked --test TARGET -- --list` and
`cargo test --locked --lib -- --list`, requires exactly one complete-name match,
then invokes exactly one of
`cargo test --locked --test TARGET FULL_NAME -- --exact --nocapture` and
`cargo test --locked --lib FULL_NAME -- --exact --nocapture`. Supplying both or
neither selector fails; the existing `--test TARGET` interface and behavior are
unchanged. `--ignored` and `--test-threads=1` are appended after `--nocapture`
only when requested. The list and execution phases each must exit zero, and the
execution summary must report a positive count with the exact named test
executed once; zero discovery or zero execution fails identically for `--lib`
and `--test`. With
`--case-matrix`, it requires exactly one canonical `tersh-case-count-v1` record
whose matrix and ordered expected/executed IDs exactly equal the repeated
`--expect-case` arguments. Without `--case-matrix`, any such record is an error.
It prints one canonical `tersh-exact-test-v1` JSON line whose selector is exactly
`{"kind":"integration","target":"TARGET"}` or
`{"kind":"lib","target":null}`. Missing, duplicate,
ignored-without-flag, zero, malformed, signal, nonzero, extra case record, or
case drift fails closed.

`scripts/evidence_core.py` owns canonical JSON, create-new publish/sync,
bounded drain-and-hash, exact 40-hex candidate validation, REST run selection,
unique job/artifact validation, remaining-deadline subprocess execution,
append-only path parsing, finding-resolution closure, fixed reviewer identity,
and canonical body-plus-hash embedding. It has no CLI and no iteration-specific
paths. Every script under `scripts/implementation_evidence/` is a thin CLI over
this module; it must not duplicate JSON, REST/run/job, timeout/cancel, output,
or review validation. The later hardening plan imports the same module and calls
`run_external_candidate.py`; it may add thin policy wrappers but cannot copy
selector/watch logic.

`record_orchestration.py` is an orchestrator-facing consumer, not a
reviewer-facing provenance form and not a host substitute. Evidence-bearing
dispatch requires a platform-owned Host Envelope Supervisor outside the
worktree/agent/operator sandbox, running as a distinct OS principal. Its private
mode-0700 store and mode-0600 create-new entries are not mounted or
pathname-addressable in the agent/operator namespace. The repository does not
create that principal, store, socket, or platform metadata; a preflight fails
closed when the current host does not expose this supervisor.

The supervisor is the sole context creator. Before spawn it fixes the exact
`tersh-host-dispatch-context-v1` body above, generates the 256-bit context nonce,
stores the body, and starts the agent only after this exact host-side command
returns successfully:

```text
python3 scripts/implementation_evidence/host_envelope_adapter.py capture-context --host-store-fd FD
```

The supervisor injects `FD` as a pre-opened connected
`AF_UNIX/SOCK_STREAM` socket; it is not inherited by the agent and is not accepted from
stdin, a pathname, environment, or a caller-owned pipe. The adapter verifies
address family `AF_UNIX`, `SO_TYPE == SOCK_STREAM`, plus the platform peer UID.
On macOS it calls `getsockopt(SOL_LOCAL=0, LOCAL_PEERCRED, 76)` and unpacks the
returned `xucred` as native `=III16I`; on Linux it calls
`getsockopt(SOL_SOCKET, SO_PEERCRED, 12)` and unpacks native `=iii` pid/uid/gid.
It requires exactly 76 bytes, `xucred` version `0`, and group count `0..16` on
macOS, and exactly 12 bytes plus nonnegative pid/uid/gid on Linux. It requires the parsed UID to
equal the supervisor/store owner and differ from the worktree process UID.
There is no `--expected-uid`, test-mode, body, nonce, model, or identity override.
The adapter reads the same four-byte big-endian 1..65536 length plus exact body
and requires a no-trailing-byte peer write-half close, validates the closed
schema, sends its digest/create-new request back over that same socket,
and accepts exactly one store reply containing a random 64-lowercase-hex
`CONTEXT_HANDLE`; the nonce remains inside the host store until the final record
embeds the body.

Immediately before spawn and at its terminal callback, the same supervisor
stores the exact invocation/response bodies with these host-side commands:

```text
python3 scripts/implementation_evidence/host_envelope_adapter.py capture-invocation --context-handle CONTEXT_HANDLE --host-store-fd FD
python3 scripts/implementation_evidence/host_envelope_adapter.py capture-response --context-handle CONTEXT_HANDLE --host-store-fd FD
```

Each command re-authenticates the `AF_UNIX/SOCK_STREAM` peer, reads one
four-byte big-endian unsigned length in the closed range 1..65536, then exactly
that many canonical-body bytes, then requires the peer to half-close its write
side with no trailing byte. It verifies the
context nonce/dispatch ID and closed field set, create-new stores it, and prints
one random 64-lowercase-hex handle. It rejects same-principal/fake peers, FIFO or
plain pipes, regular files, TTY/stdin, caller JSON, wrong nonce, replay, trailing
bytes, and unknown options. The supervisor records only callback fields it
actually exposes; if invocation model metadata is absent there is no invocation
handle, and if terminal identity/lifecycle is absent there is no response handle.

In `platform-envelope` mode the supervisor launches the recorder with a fresh
authenticated store connection:

```text
python3 scripts/implementation_evidence/record_orchestration.py append-platform --context-handle CONTEXT_HANDLE --invocation-handle INVOCATION_HANDLE --response-handle RESPONSE_HANDLE --host-store-fd FD
```

The recorder retrieves all three bodies from the authenticated peer, consumes
the handles once, verifies every digest/nonce/dispatch ID/duplicated field,
derives the destination, observes the clean worktree HEAD, compares the nullable
reported commit, and create-new publishes the record. The supervisor returns a
64-lowercase-hex `host_receipt_id` for the exact closed receipt body
`{schema,receipt_id,mode,context_sha256,invocation_sha256,response_sha256,destination,created_at}`
with schema `tersh-host-record-receipt-v1`; `mode` is one provenance tag,
`invocation_sha256` is 64 lowercase hex for platform mode and JSON null for
attestation, and `destination` is the derived candidate-relative orchestration
record path. The record embeds only the receipt ID; finalization retrieves and
cross-checks the full receipt from the private store. No model, effort, identity,
timestamp, response body, destination, or receipt override exists on CLI or
environment.

If and only if the authenticated response exists but invocation model metadata
does not, the operator requests this supervisor-launched fallback:

```text
python3 scripts/implementation_evidence/record_orchestration.py attest --context-handle CONTEXT_HANDLE --response-handle RESPONSE_HANDLE --operator-id ID --host-store-fd FD
```

It consumes context/response through the authenticated peer, derives identity
and lifecycle only from the response, independently observes the worktree commit,
and writes the exact `operator-attestation` arm plus fixed reason
`platform-model-metadata-unavailable`; only the context's model/effort request is
operator-attested. The CLI rejects identity, task, run, timestamp, commit, model,
effort, output, receipt, or free-form reason arguments. `finalize_iteration.py`
uses its own supervisor-injected store FD to require every `host_receipt_id` to
bind the embedded body hashes and destination before closure; fake/local handles
or records cannot satisfy that query. After the evidence commit, `--verify-only`
rehashes the embedded canonical bodies and no longer needs the private store. A
missing supervisor, context, response, receipt, or host FD means no evidence
entry; no mode invents metadata the callback did not expose.

The evidence entrypoints accept only argparse argument vectors. They write
sorted-key compact UTF-8 JSON plus trailing newline using the create-new
hard-link publication protocol above. The cumulative catalog uses schema
`tersh-cumulative-gates-v1`: each iteration names one predecessor and adds a
closed list of `{gate_id, kind, argv, exact_test, serial, ignored, case_matrix,
expected_cases, required_online_labels, required_external_jobs}`. Gate IDs are
globally unique. The initial committed catalog contains the Plan1 G0a/G0b
exact runners, all three downloaded-asset ignored serial smokes, the frozen outcome
matrices, format/Clippy/full/MSRV/policy commands, and CI/release/native job
requirements. Its entries for later iterations are populated from the exact
commands already named by Plans2–5, including Task5's G1a read benchmark,
Task13's mutation benchmark, trash/restore and EXDEV matrices, 40x10 tests,
native EXDEV jobs, and G3 process matrices. `run_cumulative_gates.py --through
impl-NN` resolves every predecessor, rejects a changed/removed/duplicate/zero
inventory entry, and passes each local argv to `run_gate.py`; it never replaces
the catalog with a generic full-suite script. Thus each later candidate reruns
old matrices with their ordered case IDs, ignored and serial flags, reference
benchmarks, and externally verified native requirements as well as the broad
locked regression.

Catalog argv is an array, never a shell string. Substitution occurs only when an
argv element equals one entire closed token: `{candidate}`, `{evidence_root}`,
`{attempt_root}`, `{candidate_root}`, `{fixture_root}`, or
`{artifact:ARTIFACT_ID}`. Intrinsic roots resolve to canonical absolute paths;
an artifact token resolves to the exact candidate-relative path declared for
that ID in the catalog's `outputs` map. `{fixture_root}` is valid only for a gate
declaring `fixture: "temp-dir"`; the runner creates that fresh mode-0700 directory
and removes it after records/artifacts are durably published. A token embedded
in a larger string, `$NAME`, `${NAME}`, command substitution, an unknown token,
an undeclared artifact, or a literal shell metacharacter is rejected before any
child starts. Declared outputs use create-new paths beneath
`{candidate_root}/run-local/artifacts/`; a missing, duplicate, overwritten, or
path-escaping artifact fails the gate.

Both external verifiers reject missing/duplicate/skipped/cancelled/non-success
jobs, run-attempt mismatch, cross-run artifacts, and exact-head mismatch. Their
`--require-job` values are source-checked workflow IDs and must also equal each
job's explicit `name:` returned by the REST/`gh run view` record; aliases are
rejected. `verify_release_candidate.py` additionally rehashes every artifact,
validates descriptor-to-smoke-to-manifest ordering, requires release tag and
asset names to carry the same numeric `run_id`/`run_attempt`, and requires each
downloaded Tier-1 binary's READY protocol/source-commit/Cargo.lock pair to match
its descriptor from that same run.

- [ ] **Step 4: Run GREEN and self-test a nonzero child**

Run:

```bash
python3 -m unittest scripts.tests.test_run_exact_test scripts.tests.test_implementation_evidence -v
TERSH_HARNESS_PARENT="${TMPDIR:-/tmp}"
TERSH_HARNESS_RAW="$(mktemp -d "${TERSH_HARNESS_PARENT%/}/tersh-evidence-harness.XXXXXX")"
TERSH_HARNESS_ROOT="$(cd "$TERSH_HARNESS_RAW" && pwd -P)"
trap 'rm -rf -- "$TERSH_HARNESS_ROOT"' EXIT
python3 scripts/implementation_evidence/run_gate.py --iteration impl-01 --attempt 001 --run-binding run-local --name expected-failure --candidate "$(git rev-parse HEAD)" --output-root "$TERSH_HARNESS_ROOT" --allow-failure -- sh -c 'exit 7'
python3 -c 'import json,pathlib,sys; p=list(pathlib.Path(sys.argv[1]).glob("impl-01/attempt-001/candidate-*/run-local/gates/expected-failure.json")); assert len(p)==1; d=json.loads(p[0].read_text()); assert d["exit_code"] == 7 and d["evidence_attempt"] == "001"; assert p[0].with_suffix(".stdout").is_file() and p[0].with_suffix(".stderr").is_file()' "$TERSH_HARNESS_ROOT"
python3 scripts/implementation_evidence/run_cumulative_gates.py --catalog scripts/implementation_evidence/gate_catalog.json --through impl-01 --attempt 001 --candidate "$(git rev-parse HEAD)" --output-root "$TERSH_HARNESS_ROOT" --self-test-only
git diff --check
```

Expected: all commands exit 0; the recorded child status is exactly 7.
`sh -c 'exit 7'` is an inert test child used only to prove exit-status capture;
the harness itself still passes argv arrays and never constructs shell commands.

- [ ] **Step 5: Commit the process-only prerequisite**

```bash
git add .gitignore scripts/evidence_core.py scripts/run_exact_test.py scripts/tests/test_run_exact_test.py scripts/implementation_evidence scripts/tests/test_implementation_evidence.py
git commit -m "test: add implementation iteration evidence harness"
```

Commit boundary: exact tests and cycle evidence can now fail closed before any feature iteration starts. This commit is process infrastructure, not one of the seven feature candidates.

## Exact External Candidate Procedure

Tasks2–8 call one shared CLI once per candidate attempt. The authorized operator
must have `gh` REST access and permission to create a non-protected
`codex/evidence/**` branch. Neither this helper nor its caller publishes a
public release. The exact impl-01 invocation is:

```bash
python3 scripts/implementation_evidence/run_external_candidate.py \
  --evidence-id impl-01 \
  --attempt 001 \
  --candidate "$TERSH_IMPL_CANDIDATE" \
  --repository QiushanHuang/Tersh \
  --remote origin \
  --push-ref "codex/evidence/impl-01/attempt-001/$TERSH_IMPL_CANDIDATE" \
  --output-root target/implementation-evidence \
  --workflow ci=.github/workflows/ci.yml \
  --workflow release=.github/workflows/release.yml \
  --require-job ci=quality-stable \
  --require-job ci=msrv-1-88 \
  --require-job ci=policy \
  --require-job release=tier1-macos-arm64 \
  --require-job release=tier1-linux-x86_64 \
  --require-job release=tier2-macos-x86_64-source \
  --require-job release=tier2-linux-arm64-source \
  --require-job release=install-msrv-1-88 \
  --require-job release=install-current-stable \
  --require-job release=assemble-manifest \
  --require-job release=verify-release-candidate \
  --require-online-label release=tersh-macos-14.5-23F79-arm64 \
  --require-online-label release=tersh-almalinux-8.10-kernel-4.18-x86_64 \
  --require-online-label release=tersh-macos-14.5-23F79-x86_64 \
  --require-online-label release=tersh-almalinux-8.10-kernel-4.18-aarch64 \
  --artifacts ci=none \
  --artifacts release=all \
  --require-artifact release=tier1-macos-arm64:tier1-macos-arm64-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-tier1-release-evidence-v1 \
  --require-artifact release=tier1-linux-x86_64:tier1-linux-x86_64-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-tier1-release-evidence-v1 \
  --require-artifact release=tier2-macos-x86_64-source:tier2-macos-x86_64-source-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-tier2-source-evidence-v1 \
  --require-artifact release=tier2-linux-arm64-source:tier2-linux-arm64-source-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-tier2-source-evidence-v1 \
  --require-artifact release=install-msrv-1-88:install-msrv-1-88-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-install-evidence-v1 \
  --require-artifact release=install-current-stable:install-current-stable-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-install-evidence-v1 \
  --require-artifact release=assemble-manifest:release-manifest-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-release-manifest-evidence-v1 \
  --require-artifact release=verify-release-candidate:verified-release-candidate-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-release-verification-evidence-v1 \
  --reject-extra-artifacts release \
  --registration-timeout-seconds 180 \
  --completion-timeout-seconds ci=5400 \
  --completion-timeout-seconds release=10800 \
  --overall-timeout-seconds 14400 \
  --poll-seconds 5
```

`--workflow KIND=PATH`, `--require-job KIND=ID`,
`--require-online-label KIND=LABEL`, `--artifacts KIND=none|all`,
`--require-artifact KIND=PRODUCER_JOB:EXACT_TEMPLATE:SCHEMA`, and
`--completion-timeout-seconds KIND=SECONDS` repeat;
`--reject-extra-artifacts KIND` is a repeated flag. A kind matches
`[a-z][a-z0-9-]*` and is unique. `PRODUCER_JOB` must be a required job for that
kind. `EXACT_TEMPLATE` is a safe artifact basename whose only substitutions are
the complete hyphen-delimited fields `{candidate}`, `{run_id}`, and
`{run_attempt}`; no glob, regex, slash, partial-field placeholder, or unknown
brace is accepted. `SCHEMA` matches `[a-z][a-z0-9-]*-v[1-9][0-9]*` and is the
required `schema` in the artifact's canonical root `artifact-manifest.json`,
which also repeats producer job, candidate, run ID/attempt, nonempty file list,
and payload file hashes. That list excludes `artifact-manifest.json` itself;
the helper's outer `artifact-index.json` hashes the manifest separately and
requires the downloaded regular-file set to equal exactly the manifest plus its
listed payload. `--reject-extra-artifacts` requires exact set equality and is
valid only with `--artifacts KIND=all` plus at least one required artifact.

`--attempt` is parsed and persisted as a string matching exactly
`^(?:00[1-9]|0[1-9][0-9]|[1-9][0-9]{2})$`; it is never JSON-number-coerced.
GitHub `run_attempt` remains a positive JSON integer. `--evidence-id` matches the shared closed
union `^(?:impl|hardening)-0[1-7]$`. `--push-ref` must equal
`^codex/evidence/(?:impl|hardening)-0[1-7]/attempt-(?:00[1-9]|0[1-9][0-9]|[1-9][0-9]{2})/[0-9a-f]{40}$`
exactly and its captured evidence ID, attempt, and SHA must equal the three
separate arguments. The helper rejects a different candidate suffix, force
option, protected/current branch, or existing remote ref.

The shared artifact vocabulary is also closed. Implementation `impl-06` and
`impl-07`, plus hardening `hardening-03` and `hardening-04`, require the first
two CI entries. `hardening-05`, `hardening-06`, and `hardening-07` require all
four entries (the same first two plus the two terminal entries). Every such
caller passes `--artifacts ci=all` and
`--reject-extra-artifacts ci`:

| Producer job | Exact artifact template | Root schema |
| --- | --- | --- |
| `native-exdev-linux` | `native-exdev-linux-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-native-exdev-evidence-v1` |
| `native-exdev-macos` | `native-exdev-macos-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-native-exdev-evidence-v1` |
| `terminal-multiplexer-linux` | `terminal-multiplexer-linux-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-terminal-multiplexer-evidence-v1` |
| `terminal-multiplexer-macos` | `terminal-multiplexer-macos-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-terminal-multiplexer-evidence-v1` |

Hardening `hardening-06` and `hardening-07` also require the same eight release
artifact arguments shown in the `impl-01` invocation and pass
`--reject-extra-artifacts release`. `hardening-03` and `hardening-04` each
require exactly the two native CI entries, and `hardening-05` requires exactly
all four CI entries. This
is one helper contract: a hardening wrapper may forward these literal arguments
but must not broaden `none|all`, rename a producer/schema/template, or implement
a second artifact selector.

The helper performs this order without an inline shell selector:

1. Require a clean worktree, exact candidate commit, canonical argument maps,
   and committed workflow contracts. For every requested workflow, require the
   exact safe repository-relative path to exist as a regular blob at
   `CANDIDATE:PATH`; read it through Git argument vectors, then record its Git
   object ID, byte count, and SHA-256 as `candidate_workflow_blob`. A missing,
   symlink-like, different, or post-hash-changing blob fails before network or
   push. The first network operation queries the
   repository runner REST API and records complete relevant name/status/busy/
   label inventory. Every required custom label must have an online runner;
   zero visible/matching runners fails before run snapshot, ref lookup/push,
   workflow, tag, or asset side effect.
2. Page the repository-wide `/actions/runs` endpoint before push until an empty
   page, extract every positive numeric run ID, and create
   `attempt-NNN/before-repo-runs.json` with sorted IDs, count, page/body hashes,
   snapshot start/end timestamps, and `selection_not_before` equal to the UTC
   whole-second floor of snapshot start. Do not call either workflow-path endpoint:
   it may correctly return 404 because the candidate commit introduces the
   repository's first workflow. Then require the exact remote ref to return 404
   and push `SHA:refs/heads/PUSH_REF` once without force; a race/existing ref
   fails and is never adopted.
3. Until the registration deadline, poll repository-wide runs at most every
   five seconds. For each requested kind select exactly one run whose positive
   ID is absent from the pre-push set, `created_at` is no earlier than saved
   `selection_not_before`, event is `push`, `head_sha` is SHA,
   `head_branch` is PUSH_REF, and the repository-wide REST workflow `path`
   exactly equals the bare `REQUESTED_WORKFLOW_PATH` byte-for-byte. The helper
   never constructs or accepts `PATH@REF`, `PATH@refs/heads/REF`, a tag/SHA
   suffix, a different path, or a near path. Exact `head_sha`, `head_branch`,
   `event`, and the pre-push candidate blob record together bind the source;
   REST `path` is not treated as a source-ref field. Only after selection
   may the helper query that numeric run/jobs/artifacts. A dispatch run,
   ambiguity, earlier creation, pre-existing ID, or near path fails.
4. Poll only bound numeric run IDs until per-kind and global operational
   deadlines. On registration timeout after partial binding, completion timeout,
   validation/download/verifier failure, signal, `KeyboardInterrupt`, or any
   other exception, query each bound run and issue exactly one cancel request to
   every still-nonterminal numeric ID; terminal IDs receive a `cancel_request`
   record with `needed: false`. Cancellation never uses branch, workflow query, candidate, or
   unrelated ID. A missing kind does not excuse cancelling a different kind
   that already registered. After that decision, continue polling each bound
   numeric ID until it yields a terminal REST status/conclusion and record that
   separate observation. A cancel HTTP response is not terminal evidence; if
   the reserved cleanup budget expires before every bound run has a terminal
   observation, the attempt remains FAIL and records the nonconvergence.
5. Establish one monotonic global deadline and reserve its final 60 seconds for
   cancellation/record publication. Every Git, `gh` REST/watch/download, and
   verifier subprocess receives a timeout no greater than the remaining
   applicable phase and operational-global budget. On timeout the helper
   terminates its child process group, waits boundedly, kills if necessary,
   reaps/drains it, enters the cancellation path, and records failure. Cleanup
   calls and terminal-observation polls use only the reserved remaining global
   budget. No subprocess, retry, sleep, output drain, cancel request, or terminal
   poll may reset or exceed the original deadline.
6. Download artifacts only from the bound numeric run into a newly created run
   directory. Require each exact expanded artifact once and validate its
   run/attempt/candidate and content schema from `artifact-manifest.json`. The
   manifest lists and hashes every nonempty payload regular file but excludes
   itself. The helper rejects any downloaded file outside that exact payload
   union plus the one root manifest, then writes the outer `artifact-index.json`
   containing the manifest's own size/SHA-256 and every validated payload entry.
   Reject empty, missing, renamed, duplicate, wrong-run/attempt/schema, symlink,
   path escape, and—when requested—extra artifacts. Never reuse bytes or metadata
   from another run or attempt.
7. Do not treat the manifest's producer field as proof. Source-check the exact
   candidate workflow so the expected producer job has exactly one pinned
   `actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02`
   step with ID `upload-evidence`, the exact required artifact name, and exactly
   one following call to the committed artifact-producer recorder wired to that
   step's `artifact-id` and bare-hex `artifact-digest` outputs. The recorder
   accepts exactly 64 lowercase hexadecimal digest characters, stores that
   upload output separately, and derives the normalized REST form by prefixing
   exactly `sha256:`; it rejects an already-prefixed, uppercase, short, or
   nonhex Action output. Fetch the log for the unique numeric job from
   `jobs.json`. Decode UTF-8 and unwrap each platform log line using exactly
   `^(?:\ufeff)?[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{1,9}Z (?P<payload>.*)$`,
   permitting the BOM only at byte zero of the first line. Require exactly one
   payload beginning `tersh-artifact-producer-join-v1 ` followed by canonical
   JSON; a bare marker line, malformed/missing timestamp prefix, later BOM, or
   second marker fails. Join its source workflow job ID, numeric run ID/attempt,
   artifact ID/name, bare upload digest, and derived `sha256:` digest to the jobs
   and artifacts REST bodies. `artifact-index.json` embeds the normalized unique upload/recorder
   source mapping and candidate workflow blob SHA-256, the extracted canonical
   producer record, numeric job/artifact identities, and the complete job-JSON
   body hash plus complete job-log byte count/SHA-256. The source checker also
   rejects that marker or recorder call outside the unique post-upload step. If
   the platform cannot expose a runtime join,
   record `expected_producer_job` and `manifest_declared_producer_job` only as
   `expected-and-self-declared`; strict required-artifact acceptance never treats
   that fallback as producer proof and therefore cannot PASS.

`run_external_candidate.py` writes schema `tersh-external-candidate-v1`. It
contains `evidence_id`, three-character string `evidence_attempt`, candidate,
repository, remote,
`push_ref`, fixed event `push`, runner-inventory/body hash, paginated
repository-wide before-ID snapshot body/hash, registration/per-kind/operational/
cleanup/global deadlines, subprocess timeout/final-state records, and a workflow
map. Each workflow entry contains requested path, exact bare REST `rest_path`,
the pre-push `candidate_workflow_blob` object, nullable numeric `run_id` for a failed registration,
positive numeric `run_attempt` once registered,
head/event/branch, registration/completion timestamps, required and observed
jobs, exact artifact requirements, manifest-excluding-itself indexes, outer
manifest hashes, and producer joins with source workflow, jobs, artifacts, and
log body hashes,
and, once failure handling begins, exactly one `cancel_request`
`{needed,requested_at,response_status,reason}` followed by exactly one
`terminal_observation` `{observed_at,status,conclusion,run_attempt,head_sha}` for
every bound run. An unbound kind has neither object and records its registration
failure instead. `cleanup_converged` is true only when every bound run has a
terminal observation; it is never inferred from a successful cancel response.
Top-level result is PASS only if every requested workflow, job, and artifact
policy passes. Stdout is exactly one canonical
`tersh-external-candidate-result-v1` JSON object with `manifest_path` and
`result`; failure writes and prints that object before returning nonzero.

Before a numeric run exists, bootstrap evidence goes under
`run-unregistered-KIND`. Each selected run is copied create-new into
`run-RUN_ID/KIND`; every invocation, including partial registration, writes its
explicit combined `external-candidate.json` under the sorted
`run-set-KIND-BINDING[-KIND-BINDING...]`. Stdout's `manifest_path` names that
file. No failed attempt is retried in place. `impl-06` and `impl-07` add
`ci=native-exdev-linux` and
`ci=native-exdev-macos` job requirements from the cumulative catalog; all other
arguments remain literal and unchanged.

## Locked Per-Iteration Gate And Closure Commands

For each Tasks2–8 attempt, set the six variables below before any gate. A
retry increments `TERSH_IMPL_ATTEMPT` and therefore creates a new root; it never
reuses `001` or any earlier attempt directory.

```bash
export TERSH_IMPL_ITERATION=impl-01
export TERSH_IMPL_ATTEMPT=001
export TERSH_IMPL_CANDIDATE="$(git rev-parse HEAD)"
export TERSH_IMPL_EVIDENCE_ROOT="target/implementation-evidence/$TERSH_IMPL_ITERATION"
export TERSH_IMPL_ATTEMPT_ROOT="$TERSH_IMPL_EVIDENCE_ROOT/attempt-$TERSH_IMPL_ATTEMPT"
export TERSH_IMPL_CANDIDATE_ROOT="$TERSH_IMPL_ATTEMPT_ROOT/candidate-$TERSH_IMPL_CANDIDATE"
test -z "$(git status --porcelain=v1 --untracked-files=all)"
python3 scripts/implementation_evidence/run_cumulative_gates.py \
  --catalog scripts/implementation_evidence/gate_catalog.json \
  --through "$TERSH_IMPL_ITERATION" \
  --attempt "$TERSH_IMPL_ATTEMPT" \
  --candidate "$TERSH_IMPL_CANDIDATE" \
  --output-root target/implementation-evidence
```

Then run the external CLI above with `--evidence-id`, `--attempt`, `--candidate`,
and `--push-ref` equal to these variables. Use the complete fixed CI/release job
and label arguments shown above. For `impl-06` and `impl-07`, additionally pass
`--require-job ci=native-exdev-linux` and
`--require-job ci=native-exdev-macos`, change `--artifacts ci=none` to
`--artifacts ci=all`, and append these literal arguments:

```text
--require-artifact ci=native-exdev-linux:native-exdev-linux-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-native-exdev-evidence-v1
--require-artifact ci=native-exdev-macos:native-exdev-macos-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-native-exdev-evidence-v1
--reject-extra-artifacts ci
```

Do not run a second selector or verifier:
the single helper invocation writes the `ci` and `release` records from the
bound run IDs into the resulting `run-set`.

After Wave C and closure reviews have been created append-only beneath the final
candidate subtree and bind its exact `run-set`, finalize from the evidence-ID
root. The Host Envelope Supervisor launches this command and injects an already
open numeric `AF_UNIX/SOCK_STREAM` descriptor as `TERSH_HOST_STORE_FD`; an operator
must not export or synthesize that variable:

```bash
python3 scripts/implementation_evidence/finalize_iteration.py \
  --iteration "$TERSH_IMPL_ITERATION" \
  --accepting-attempt "$TERSH_IMPL_ATTEMPT" \
  --candidate "$TERSH_IMPL_CANDIDATE" \
  --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" \
  --require-cumulative-through "$TERSH_IMPL_ITERATION" \
  --host-store-fd "$TERSH_HOST_STORE_FD" \
  --output "docs/superpowers/evidence/2026-08-10-tersh-implementation/$TERSH_IMPL_ITERATION.json"
```

`finalize_iteration.py` enumerates every attempt from `001` through
`--accepting-attempt`, every per-commit subtree, and exactly one complete final
`run-set` for the accepting candidate. It rejects attempt gaps, later attempt
directories, noncanonical extras, missing historical reports, external results
from another run/attempt, or a local gate that did not record one exact
discovered/executed test where required. It also requires the iteration-specific
gate IDs enumerated below and exact `gpt-5.6-sol`/`xhigh` identity for every
orchestration/review entry in every attempt, while preserving whether that
identity is platform-envelope-verified or operator-attested. It requires
`--host-store-fd` in create mode to validate every host receipt; the
committed-manifest `--verify-only` mode rejects that option and uses only the
embedded bodies/hashes. Its history array
embeds canonical body/hash pairs for attempt-level runner inventory and before
run snapshots and candidate-level gates, orchestration, reviews, bootstrap,
selected runs, jobs, artifact indexes, and combined manifests; raw bounded gate
logs remain external. It constructs the committed
`--output` directly from that immutable history and publishes that one file
create-new; there is no self-referential raw `finalization.json` beneath the
attempt tree.

### Task 2: Close `impl-01` — G0a Release And Installation Truth

**Files:**

- Execute component recipes in Plan1: Task1, Task6a, Task7a, Task8, Task9 local/tooling steps, Task10a
- Create after closure: `docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-01.json`

- [ ] **Step 1: Run Wave A, then execute component TDD in Wave B and commit Task10a last**

Run product, architecture, and implementation Wave A on the same baseline. Then
the single implementation writer runs every RED/GREEN command and component
commit in Wave B, appending its execution report. Do not execute Task6b, Task7b,
Task10b, or any Task9 external candidate run. Task10a's documentation commit is the
candidate boundary.

- [ ] **Step 2: Freeze the candidate and run local, CI, and release gates**

Run the locked per-iteration commands with `TERSH_IMPL_ITERATION=impl-01`, then
the exact external invocation. The cumulative catalog must emit `g0a-local`,
must place `clean_official_build_accepts_its_embedded_identity_when_historical_manifest_is_empty`
in the `impl-01` slice (and therefore every later cumulative replay), and the
external manifest must contain unique `quality-stable`, `msrv-1-88`,
`policy`, all eight locked release jobs, the create-new descriptor/smoke/
manifest chain, and both Tier-1 downloaded-binary READY identities on the exact
candidate. Expected: every gate is PASS; a missing runner fails before push.

- [ ] **Step 3: Run Wave C and the five-role closure, then finalize**

Run safety and verification Wave C, then Closure A and Closure B with the locked
schemas. Any finding returns to Wave B; after correction, repeat Step2 with a new
commit and new external runs before repeating Wave C/closure. Then run:

Run:

```bash
python3 scripts/implementation_evidence/finalize_iteration.py --iteration "$TERSH_IMPL_ITERATION" --accepting-attempt "$TERSH_IMPL_ATTEMPT" --candidate "$TERSH_IMPL_CANDIDATE" --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" --require-cumulative-through impl-01 --require-gate g0a-local --host-store-fd "$TERSH_HOST_STORE_FD" --output docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-01.json
```

The committed `impl-01` catalog additionally requires `ci` and `release`.

Expected: PASS only with five final roles on the exact candidate and no unresolved P0/P1.

- [ ] **Step 4: Commit evidence only**

```bash
test "$(git status --porcelain=v1 --untracked-files=all)" = "?? docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-01.json"
git add docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-01.json
test "$(git diff --cached --name-only)" = "docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-01.json"
git commit -m "test: record impl-01 g0a evidence"
```

### Task 3: Close `impl-02` — G0b Existing Interaction And Result Truth

**Files:**

- Execute component recipes in Plan1: Tasks2–5, Task6b, Task7b, Task10b
- Create after closure: `docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-02.json`

- [ ] **Step 1: Run Wave A, then execute component TDD in Wave B and commit Task10b last**

Run the three-role Wave A on the post-`impl-01` baseline. The single Wave B
writer then runs all exact focused tests, regression commands, and component
commits and appends the execution report. Task10b's documentation commit is the
candidate. Do not edit G0a support labels after this boundary.

- [ ] **Step 2: Run the G0b gate and every prior G0a gate**

Run the locked commands with `TERSH_IMPL_ITERATION=impl-02`. The cumulative
catalog must rerun every `impl-01` matrix, ignored/serial smoke, policy and
external requirement plus emit `g0b-contract`; then run the exact external
helper. Expected: the new remote/terminal behavior does not invalidate G0a
install or downloaded-binary READY identity.

- [ ] **Step 3: Run Wave C/closure and commit only `impl-02.json`**

Run Wave C, Closure A, and Closure B. Any finding returns to Wave B and requires
a new candidate plus all Step2 gates before finalization.

```bash
python3 scripts/implementation_evidence/finalize_iteration.py --iteration "$TERSH_IMPL_ITERATION" --accepting-attempt "$TERSH_IMPL_ATTEMPT" --candidate "$TERSH_IMPL_CANDIDATE" --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" --require-cumulative-through impl-02 --host-store-fd "$TERSH_HOST_STORE_FD" --output docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-02.json
test "$(git status --porcelain=v1 --untracked-files=all)" = "?? docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-02.json"
git add docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-02.json
test "$(git diff --cached --name-only)" = "docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-02.json"
git commit -m "test: record impl-02 g0b evidence"
```

### Task 4: Close `impl-03` — G1a Read Responsiveness

**Files:**

- Execute Plan2 Tasks1–5
- Create after closure: `docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-03.json`

- [ ] **Step 1: Run Wave A, then execute the G1a recipes in Wave B and freeze the candidate**

Run Wave A on the `impl-02` evidence commit. In Wave B, Task1 contributes only
the raw-path/identity substrate needed by the read lanes; no mutation worker or
durable user-visible mutation begins. Execute Tasks2–5 and their frozen
latency/stale-result cases. The exact Task1 inventory must include
`raw_unix_path_deserialize_rejects_noncanonical_base64` and
`raw_unix_name_deserialize_revalidates_every_forbidden_component`; run each
through `scripts/run_exact_test.py --test trusted_fs`. Run the integration test
`adjacent_receipt_facts_require_live_bound_lock_and_actual_synced_bytes` through
that same target and the crate-private
`trusted_fs::tests::atomic_receipt_advance_rejects_wrong_revision_edge_or_facts`
through `scripts/run_exact_test.py --lib`; together they prove the verified
snapshot token retains the owning no-follow lock and cannot authorize a
different revision/edge. Append the execution report, then commit the final G1a
component change.

- [ ] **Step 2: Run exact slice, prior, CI, and release gates**

Run the locked commands with `TERSH_IMPL_ITERATION=impl-03`. Its cumulative
catalog entry must contain these exact array-valued gates in addition to every
prior gate:

```json
[
  {
    "gate_id": "g1a-read-acceptance",
    "argv": ["cargo", "test", "--locked", "--test", "plan2_read_acceptance"]
  },
  {
    "gate_id": "g1a-reference",
    "fixture": "temp-dir",
    "outputs": {
      "g1a-read-candidate": "run-local/artifacts/tersh-plan2-read-candidate.json"
    },
    "argv": ["cargo", "run", "--locked", "--release", "--bin", "tersh-plan2-read-bench", "--", "--require-reference-profile", "--output", "{artifact:g1a-read-candidate}", "--fixture-root", "{fixture_root}"]
  }
]
```

`run_cumulative_gates.py` resolves both placeholders as whole argv elements;
neither `$TERSH_IMPL_ATTEMPT_ROOT`, `$TERSH_READ_FIXTURE_ROOT`, interpolation,
nor a shell appears in the catalog. It records the benchmark as
`g1a-reference`. Run the external helper
so a changed binary cannot inherit old G0a evidence. Expected: the benchmark
matches Plan2's frozen reference profile and its artifact is create-new.

- [ ] **Step 3: Run Wave C/closure and commit evidence only**

Any review finding returns to Wave B and invalidates the candidate and Step2
external evidence.

```bash
python3 scripts/implementation_evidence/finalize_iteration.py --iteration "$TERSH_IMPL_ITERATION" --accepting-attempt "$TERSH_IMPL_ATTEMPT" --candidate "$TERSH_IMPL_CANDIDATE" --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" --require-cumulative-through impl-03 --require-gate g1a-reference --host-store-fd "$TERSH_HOST_STORE_FD" --output docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-03.json
test "$(git status --porcelain=v1 --untracked-files=all)" = "?? docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-03.json"
git add docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-03.json
test "$(git diff --cached --name-only)" = "docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-03.json"
git commit -m "test: record impl-03 g1a evidence"
```

### Task 5: Close `impl-04` — G1b Mutation And Durable Claim Truth

**Files:**

- Execute Plan2 Tasks6–13
- Create after closure: `docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-04.json`

- [ ] **Step 1: Run Wave A, then execute G1b TDD in Wave B including ADD-009/010**

Run Wave A on the `impl-03` evidence commit. The Wave B writer executes Plan2
Tasks6–13. The exact focused inventory must include
`receipt_deserialize_cannot_fabricate_child_capability`,
`source_claim_proof_is_bound_to_bundle_revision_and_edge`,
`mirror_proof_is_bound_to_bundle_revision_and_edge`, and
`terminal_typestate_is_bound_to_bundle_revision_and_expectation`. It also requires
`claim_consumes_handle_and_holds_lock_across_transition`,
`mirror_factory_requires_verifier_issued_adjacent_facts`, and
`dropping_lock_before_confirm_is_unrepresentable`, and
`consumed_transition_token_cannot_be_used_twice`; together these prove a
by-value proof cannot survive either its actual no-follow lock or the verified
receipt snapshot it authorizes and cannot be reused after consumption. Parameterized
fault matrices emit frozen case IDs/counts through `tersh-case-count-v1`; append
the Wave B execution report before candidate freeze.

- [ ] **Step 2: Run slice, prior, CI, and release gates**

Run the locked commands with `TERSH_IMPL_ITERATION=impl-04`, including the
catalog's `g1b-mutation` and mutation-only `tersh-plan2-mutation-bench` gates.
The mutation benchmark must consume the unchanged
`tersh-plan2-read-candidate.json` schema/reference regenerated by the cumulative
`g1a-reference` gate in the current candidate subtree. Its catalog argv passes
`{artifact:g1a-read-candidate}` as the whole value following
`--read-candidate`; it cannot interpolate a path, replace the artifact, or
retroactively pass `g1a-reference`. Then run the exact external helper.

- [ ] **Step 3: Run Wave C/closure and commit evidence only**

Any finding returns to Wave B and requires a new candidate and all Step2 gates.

```bash
python3 scripts/implementation_evidence/finalize_iteration.py --iteration "$TERSH_IMPL_ITERATION" --accepting-attempt "$TERSH_IMPL_ATTEMPT" --candidate "$TERSH_IMPL_CANDIDATE" --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" --require-cumulative-through impl-04 --require-gate g1a-reference --require-gate g1b-mutation --host-store-fd "$TERSH_HOST_STORE_FD" --output docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-04.json
test "$(git status --porcelain=v1 --untracked-files=all)" = "?? docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-04.json"
git add docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-04.json
test "$(git diff --cached --name-only)" = "docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-04.json"
git commit -m "test: record impl-04 g1b evidence"
```

### Task 6: Close `impl-05` — G2 CLI Recovery Engine

**Files:**

- Execute Plan3 Tasks1–7
- Create after closure: `docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-05.json`

- [ ] **Step 1: Run Wave A, then execute Plan3 in Wave B with ADD-009/010 cases**

Run Wave A on the `impl-04` evidence commit. The Wave B writer executes Plan3
and appends its execution report. Its inventory must include
`trash_raw_receipt_cannot_forge_transition_proof`,
`trash_genuine_token_rejects_cross_bundle_replay`,
`trash_genuine_token_rejects_cross_revision_replay`,
`trash_genuine_token_rejects_cross_edge_replay`,
`trash_consumed_token_cannot_be_used_twice`, and
`trash_and_restore_authorizing_facts_cannot_outlive_claim_or_locked_receipt_snapshot`.
Each deserialization case feeds mutated raw JSON through
`serde_json::from_slice`; each replay case obtains a real token from bundle A
and proves bundle B remains byte-for-byte unchanged; the second-use case proves
the original bundle/revision cannot advance twice with the same by-value token.

- [ ] **Step 2: Run slice, prior, CI, and release gates**

Run the locked commands with `TERSH_IMPL_ITERATION=impl-05`. The catalog must
emit `g2-cli`, rerun every prior read/mutation/reference gate with its frozen
case IDs, and then bind fresh CI/release push runs through the exact helper.

- [ ] **Step 3: Run Wave C/closure and commit evidence only**

Any finding returns to Wave B and requires a new candidate and all Step2 gates.

```bash
python3 scripts/implementation_evidence/finalize_iteration.py --iteration "$TERSH_IMPL_ITERATION" --accepting-attempt "$TERSH_IMPL_ATTEMPT" --candidate "$TERSH_IMPL_CANDIDATE" --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" --require-cumulative-through impl-05 --require-gate g2-cli --host-store-fd "$TERSH_HOST_STORE_FD" --output docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-05.json
test "$(git status --porcelain=v1 --untracked-files=all)" = "?? docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-05.json"
git add docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-05.json
test "$(git diff --cached --name-only)" = "docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-05.json"
git commit -m "test: record impl-05 g2 cli evidence"
```

### Task 7: Close `impl-06` — G1c EXDEV And G2 Recovery TUI

**Files:**

- Execute Plan4 Tasks1–6
- Create after closure: `docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-06.json`

- [ ] **Step 1: Run Wave A, then execute Plan4 in Wave B including unforgeable receipt/proof cases**

Run Wave A on the `impl-05` evidence commit. The Wave B writer executes Plan4,
appends its execution report, and includes exact
`exdev_receipt_rejects_forged_raw_path_capability` and
`exdev_transition_rejects_genuine_token_from_other_bundle_revision_or_edge`,
`exdev_consumed_transition_token_cannot_be_used_twice`, and
`exdev_authorizing_facts_cannot_outlive_claim_or_locked_receipt_snapshot` in
addition to source-swap, crash, native two-device, restore-conflict, retry, and
40x10 cases.

- [ ] **Step 2: Run local, native, prior, CI, and release gates**

Run the locked commands with `TERSH_IMPL_ITERATION=impl-06`. The catalog emits
`g1c-g2` and every prior gate. In the single external-helper invocation add
`--require-job ci=native-exdev-linux` and
`--require-job ci=native-exdev-macos`; both must be unique successes on the
same selected CI `run_id`/`run_attempt`. Do not run a second verifier against a
copied `ci-run.json`.

- [ ] **Step 3: Run Wave C/closure and commit evidence only**

Any finding returns to Wave B and requires a new candidate and all Step2 gates.

```bash
python3 scripts/implementation_evidence/finalize_iteration.py --iteration "$TERSH_IMPL_ITERATION" --accepting-attempt "$TERSH_IMPL_ATTEMPT" --candidate "$TERSH_IMPL_CANDIDATE" --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" --require-cumulative-through impl-06 --require-gate g1c-g2 --require-gate native-exdev-ci --host-store-fd "$TERSH_HOST_STORE_FD" --output docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-06.json
test "$(git status --porcelain=v1 --untracked-files=all)" = "?? docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-06.json"
git add docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-06.json
test "$(git diff --cached --name-only)" = "docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-06.json"
git commit -m "test: record impl-06 g1c g2 evidence"
```

### Task 8: Close `impl-07` — G3 Cluster Companion

**Files:**

- Execute Plan5 Tasks1–6
- Create after closure: `docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-07.json`

- [ ] **Step 1: Run Wave A, then complete every production correction in Wave B before the candidate**

Run Wave A on the `impl-06` evidence commit. The Wave B writer runs Plan5
Tasks1–5, then its Task6 matrix RED. If the matrix exposes a defect, modify only
the scheduler/probe/launch/UI production files named by the failing component
task, rerun the exact case and all prior G3 tests, and commit the correction
before Task6 documentation. Append the execution report. The final G3
documentation commit is the candidate boundary. No correction is permitted
after external evidence without creating a new candidate.

- [ ] **Step 2: Run G3, all prior local gates, CI, and release**

Run the locked commands with `TERSH_IMPL_ITERATION=impl-07`. The catalog emits
`g3-cluster`, its frozen scheduler/probe/launch process cases, and all prior
gates. Run one external-helper invocation with both workflows and the two native
EXDEV CI jobs. Expected: G3 is proven without changing the already accepted
Workbench Trusted Core boundary.

- [ ] **Step 3: Run Wave C and integrated five-role closure, then commit evidence only**

Product verifies G3 is still a bounded companion and not a Workbench prerequisite. Architecture verifies private reducer ownership and no generic runtime. Implementation verifies every frozen matrix case. Safety verifies no child/reader survives quit, timeout, poll error, or panic. Verification independently reruns G3 and every prior gate.

```bash
python3 scripts/implementation_evidence/finalize_iteration.py --iteration "$TERSH_IMPL_ITERATION" --accepting-attempt "$TERSH_IMPL_ATTEMPT" --candidate "$TERSH_IMPL_CANDIDATE" --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" --require-cumulative-through impl-07 --require-gate g3-cluster --require-gate native-exdev-ci --host-store-fd "$TERSH_HOST_STORE_FD" --output docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-07.json
test "$(git status --porcelain=v1 --untracked-files=all)" = "?? docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-07.json"
git add docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-07.json
test "$(git diff --cached --name-only)" = "docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-07.json"
git commit -m "test: record impl-07 g3 evidence"
```

## Requirement And Acceptance Map

| Requirement | Iteration evidence |
| --- | --- |
| Ordered seven implementation iterations, TDD, prior gates, full regression (design 1244–1261) | Tasks2–8 manifests |
| Append-only role/wave/attempt provenance and same-candidate five-role closure (1476–1486) | Task1 finalizer plus every manifest |
| Exact focused discovery/execution and frozen parameter cases (1471–1474) | Task1 exact runner and every component focused gate |
| G0a native/external evidence binds exact candidate (1443–1470) | `impl-01` and every later prior release gate |
| External proof uses exact online runners, unique push ref, string evidence attempt plus numeric GitHub run ID/attempt, bounded timeout, and create-new evidence | Task1 helper plus Tasks2–8 external manifests |
| Shared evidence IDs and workflow refs admit only impl/hardening 01–07 | Task1 parser plus Plan1 CI/release contract tests |
| Candidate-independent attempts preserve every per-commit record and finalization scans full history | Task1 layout/finalizer plus Tasks2–8 manifests |
| Every later iteration reruns frozen prior matrices, ignored/serial/native tests, and benchmarks | Task1 cumulative catalog plus Tasks2–8 local gate records |
| Raw path custom deserialization cannot forge capabilities (ADD-009, 1488–1497) | `impl-03` trusted-fs core tests, revalidated by `impl-04`, plus `impl-05/06` host receipt tests |
| Genuine proof tokens cannot replay across bundle/revision/edge, outlive their actual no-follow lock/verified receipt snapshot, or be reused after by-value consumption (ADD-010, 1499–1505) | `impl-03/04` substrate lifetime tests plus `impl-05/06` host transition lifetime/replay tests |
| Workbench release excludes G3; full task includes it | `impl-06` may support Workbench acceptance; `impl-07` closes only feature iteration 7 |

## Final Checklist

- [ ] The shared harness is committed before `impl-01` starts.
- [ ] Every focused test uses exactly one mutually exclusive selector (`--test TARGET` or crate-private `--lib`), is listed once, executes once under `--exact`, and every parameter matrix validates frozen ordered IDs/count; both/neither selectors and zero discovery/execution fail.
- [ ] Every candidate is a clean committed SHA before external execution or final review.
- [ ] Every focused execution includes `--nocapture` and exactly one case record when—and only when—a frozen matrix is declared.
- [ ] Every iteration reruns the committed cumulative gate catalog, including prior ignored/serial/native matrices and reference benchmarks; no generic script stands in for them.
- [ ] Every raw gate, bootstrap, selected run, artifact, orchestration, and review path is create-new beneath one attempt/candidate/run binding; retries allocate a new attempt, and the committed finalization manifest is also create-new.
- [ ] Every finalizer starts at the evidence-ID root and embeds canonical body/hash pairs for runner inventory, before-run snapshot, jobs, and all other canonical JSON across all attempts/per-commit subtrees through the accepting attempt; bounded gate logs are validated but not embedded.
- [ ] External acceptance inventories required online custom runners before side effects, hashes each exact candidate workflow blob, pages repository-wide pre-push run IDs without workflow-path lookup, and binds only new creation-time push runs whose REST path is the exact bare workflow path and whose `head_sha`/`head_branch`/`event` match the pushed candidate.
- [ ] Every partial-registration, timeout, interrupt, download, verifier, or other failure issues at most one cancel request per bound nonterminal numeric ID and then observes every bound run terminal within the reserved budget; every subprocess/poll consumes the original remaining global deadline.
- [ ] Required artifacts match exact template/schema and nonempty contents; the root manifest excludes itself, the outer index hashes it, extras are rejected when requested, and producer acceptance requires unique pinned upload source plus artifact-ID/name, bare-upload-digest-to-REST-digest normalization, and timestamp-unwrapped job-log runtime join.
- [ ] Catalog argv uses only closed whole-token placeholders; no `$` variable, interpolation, or shell expansion survives into the committed catalog.
- [ ] Every source-changing correction creates a new candidate and reruns applicable external gates.
- [ ] Every iteration has Wave A, Wave B, Wave C, and five same-candidate closure reports with append-only bodies and hashes.
- [ ] Every role in every required wave is bound by the distinct-principal Host Envelope Supervisor to requested `gpt-5.6-sol` with `xhigh`; create-mode finalization authenticates every host receipt, rejects any other value or a missing supervisor/response, and distinguishes platform-envelope verification from explicit operator attestation when trustworthy platform model metadata is unavailable.
- [ ] Each closure commit contains exactly one of `impl-01.json` through `impl-07.json` and no code, test, workflow, or product-documentation change.
- [ ] `impl-01` and `impl-02` use the split Plan1 component recipes; Task9 external evidence is never run before Task10a.
- [ ] ADD-009 and ADD-010 map to exact Plan2/3/4 tests, including lock/snapshot lifetime and by-value single-use cases, and the final 269-ID requirement catalog.
- [ ] `impl-07` does not relabel G3 as a Workbench prerequisite or claim the seven later hardening cycles are complete.
