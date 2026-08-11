# Tersh Implementation Iteration Evidence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Execute and prove the seven Trusted Core feature iterations as seven ordered, independently reviewable candidate commits with exact gates, append-only five-role provenance, and evidence-only closure commits.

**Architecture:** Build and independently register one root-owned,
digest-pinned Python-standard-library evidence bundle before feature work, then
treat Plans 1–5 as untrusted component recipe catalogs rather than an execution
order. Each iteration freezes a clean committed candidate, runs the registered
cumulative policy, reruns its slice plus every prior accepted special gate, and
closes five independently sealed review roles on that same candidate. The
registered external-candidate producer inventories required online custom
runners, creates a unique `codex/evidence/**` push ref, binds the resulting
CI/release `push` runs by numeric ID, and appends host-receipted attempt evidence;
manual dispatch remains an operator recovery path but is never the first
acceptance bootstrap.

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
| `scripts/evidence_core.py` | Repository source/diagnostic copy of the shared primitives; formal execution uses only the exact reviewed byte in the registered root bundle |
| `scripts/run_exact_test.py` | Repository source/diagnostic copy of the registered exact-test producer; the root-bundle policy fixes discovery/execution and case inventory |
| `scripts/tests/test_run_exact_test.py` | Exact discovery/execution, ignored/serial, malformed summary, and frozen case-matrix tests |
| `scripts/implementation_evidence/run_gate.py` | Repository source/diagnostic copy; registered `run-gate` host-spools and receipts one bounded command observation before emitting a projection |
| `scripts/implementation_evidence/host_envelope_adapter.py` | Repository source plus registered transport entrypoint for the closed root-peer-authenticated single-FD transaction and opaque capability rotation |
| `scripts/implementation_evidence/record_orchestration.py` | Repository source/diagnostic copy of the registered recorder and agent-record sealer; formal records live first in host spool/ledger |
| `scripts/implementation_evidence/finalize_iteration.py` | Repository source/diagnostic copy; registered finalizer host-enumerates all attempts/receipts, proves spool/projection bijection, obtains a preimage receipt, and emits one envelope without Git |
| `scripts/implementation_evidence/commit_and_close.py` | Registered root-owned entrypoint that commits the sole envelope and appends the detached closure receipt; the repository copy is never formal authority |
| `scripts/implementation_evidence/run_external_candidate.py` | Repository source/diagnostic copy of the registered runner selector/collector; the candidate version cannot select or bless its own run evidence |
| `scripts/implementation_evidence/verify_ci_evidence.py` | Repository source/diagnostic copy of the root-bundle CI policy |
| `scripts/implementation_evidence/verify_release_candidate.py` | Repository source/diagnostic copy of the root-bundle release policy |
| `scripts/implementation_evidence/gate_catalog.json` | Frozen registered per-iteration gate policy; a candidate copy is never acceptance authority |
| `scripts/implementation_evidence/run_cumulative_gates.py` | Repository source/diagnostic copy of the registered cumulative producer |
| `scripts/tests/test_implementation_evidence.py` | Canonicality, output bounds, append-only attempt layout, cumulative catalogs, unique push selection, deadlines/cancel, and closure tests |
| `target/implementation-evidence/impl-{01,02,03,04,05,06,07}/attempt-NNN/candidate-SHA/` | Policy-fixed shared physical root for both implementation `local` and `external` projection classes; receipt class disambiguates ownership, the host ledger/spool is authoritative, and every projected record must bijectively match it |
| `docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-{01,02,03,04,05,06,07}.json` | Seven sole-destination canonical envelopes; each is formal only with its detached online closure receipt |

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

Every receipted raw destination follows the closed relative layout below. The
attempt-bound policy supplies one immutable
`tersh-evidence-projection-root-map-v1`
`{schema,evidence_family,local_root,external_root}`. For `impl`, both roots are
the same canonical repository-relative `target/implementation-evidence`; for
`hardening`, `local_root` is `target/hardening` and `external_root` is
`target/hardening-external`. No caller, environment, or receipt body may choose
a root. A receipt destination is relative to the evidence-specific class base,
not directly to the family-level class root:
`physical_path = root_map[projection_root_class] / attempt_binding.evidence_id / receipt.destination`.
The closed `destination` grammar therefore begins with `attempt-NNN/`; the Host
derives and validates `evidence_id` only through the joined attempt binding. An
evidence attempt binds exactly one candidate commit and therefore one
immutable candidate-SHA namespace. The `local` class always contains the Host
marker pair and its `candidate-SHA/` directory. The `external` class is absent
until external preflight; once populated it may contain one directory with the
same SHA but never a second marker or different candidate. When both classes map
to one physical root their path sets overlay without duplicate files; when they
map to distinct roots the finalizer treats the matching directories as one
logical candidate namespace. The baseline, every Wave B result commit, and the
final candidate use successive three-digit attempts rather than sharing an
attempt:

```text
LOCAL_ROOT/EVIDENCE-ID/
  attempt-NNN/
    attempt.json
    candidate-SHA/
      candidate.json
      run-local/gates/GATE.{json,stdout,stderr}
      run-cumulative/gates/GATE.{json,stdout,stderr}
      run-cumulative/cumulative-gates.json
      run-cumulative/artifacts/ARTIFACT
      orchestration/ROLE.WAVE.REVIEW_ATTEMPT.json
      reviews/ROLE.WAVE.REVIEW_ATTEMPT.json
      completion/requirements-revision-RRR.json
      completion/completion-audit-revision-RRR.json
      completion/audit-reservation-revision-RRR-failed.json

EXTERNAL_ROOT/EVIDENCE-ID/
  attempt-NNN/
    runner-inventory.json
    before-repo-runs.json
    candidate-SHA/
      run-unregistered-KIND/bootstrap.json
      run-RUN_ID/KIND/{selected-run.json,jobs.json,artifacts/,artifact-index.json}
      run-set-KIND-BINDING[-KIND-BINDING...]/external-candidate.json
      run-set-KIND-BINDING[-KIND-BINDING...]/gates/GATE.{json,stdout,stderr}
```

`EVIDENCE-ID` matches exactly `^(?:impl|hardening)-0[1-7]$`; implementation
finalizers accept only `impl-01` through `impl-07`, while the same helper also
serves `hardening-01` through `hardening-07`. `NNN` matches exactly
`^(?:00[1-9]|0[1-9][0-9]|[1-9][0-9]{2})$` (`001` through `999`);
`SHA` is a full lowercase commit; and every numeric `RUN_ID` is positive.
`RRR` has the same `001` through `999` grammar as `NNN` but names the
host-allocated Cycle 7 `audit_revision`; the three `completion/` forms are
permitted only for `hardening-07`, and one revision contains either the first
two accepting-pair members or the single failed-reservation member, never both.
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
`run-local` is owned only by one direct registered `run-gate` session.
`run-cumulative` is owned only by one registered cumulative session: it projects
each collected gate triplet beneath `run-cumulative/gates/` and its one
receipt-backed aggregate at `run-cumulative/cumulative-gates.json`. Neither
producer may write the other's binding. `run-unregistered-KIND` may contain only pre-registration bootstrap/failure
evidence. The repository-wide pre-push ID snapshot is stored once per attempt
and referenced by hash from each kind's bootstrap record; no before snapshot
requires a workflow-path endpoint. `run-local` contains no external assertion.
Directories and files are create-new. A retry or candidate change allocates the
next attempt and preserves the complete failed attempt. A second candidate SHA
in either root class of one attempt is noncanonical; no record is removed, renamed,
truncated, rewritten, or relabeled into a later attempt.

Each producer receipt carries the policy-derived root class. `local` contains
attempt/candidate markers; direct and cumulative gates/aggregates;
orchestration/reviews; implementation-entry; requirements, completion audit,
and audit-reservation failure. `external` contains runner inventory,
before-repo-runs, bootstrap, selected run, jobs, artifact index/payload,
external-candidate, and the wrapper gate whose run binding is `run-set-*`.
For implementation evidence the two classes deliberately resolve to the same
physical root; for hardening they resolve to the two disjoint roots above.
Relative destinations remain unique within the class, and the finalizer and
projection repair enumerate the exact two-class union. A missing class, wrong
class, third root, duplicate body across classes, or policy/root drift fails.

`GATE` matches `^[a-z][a-z0-9-]{0,63}$`. A normal projection is the closed
three-file set `GATE.json`, `GATE.stdout`, and `GATE.stderr`; a fourth sibling,
symlink, or mismatched basename fails. Both bounded log files are attempted even
for an empty stream and retain only the first 1 MiB. The receipt-backed JSON
stream object records `total_bytes`, the SHA-256 of the complete drained stream,
`retained_bytes`, the SHA-256 of the retained prefix, and the matching
candidate-relative retained-log path, while the trusted producer session binds
the observed stream hashes. If a bounded log projection exists, finalization
must validate it; a missing log after a post-COMMIT/lost-reply projection failure
is diagnostic-unavailable rather than loss of the authoritative gate record.
Finalization requires and embeds the host-receipted canonical `GATE.json`
body/hash but deliberately does not embed raw stdout/stderr bytes.

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
Wave A binds its baseline as the reviewed commit. Before each Wave B dispatch the
supervisor reserves the next evidence-attempt number but does not create its
attempt binding; after the response, `seal-agent-record` binds that attempt to
the one reported result commit (or to the unchanged baseline for a failed/no-
change report) and seals exactly one execution report. A Wave B dispatch may
produce at most one evidence candidate boundary; another correction uses the
next attempt. Only Wave C, Closure A/B, local/external gates, and finalization
share the final attempt and candidate SHA. Historical Wave A/B records remain
append-only in their own attempts and are never relabeled as reviews of a future
commit.

Each iteration executes these waves, never exceeding three concurrent reviewers:

1. Wave A: product, architecture, and implementation diagnosis on one baseline/candidate.
2. Wave B: one implementation writer applies the smallest correction and appends an execution report for every attempt.
3. Wave C: independent safety and verification reviews on the corrected candidate.
4. Closure A: product, architecture, and implementation final reports on the identical candidate.
5. Closure B: safety and verification final reports on that same candidate.

The five accepted closure roles must bind five distinct `agent_id` values, five
distinct `agent_run_id` values, five role-matching canonical task paths, and five
distinct `dispatch_id` values. No identity dimension can serve more than one
role; all five accepted closure roles must also differ from the Wave B writer in
all four dimensions. The finalizer rejects cross-role or writer/reviewer identity reuse
even when every body otherwise says PASS.

Every dispatch in all five waves explicitly requests model `gpt-5.6-sol` with
reasoning effort `xhigh`; inheritance, a default model, or a self-declared report
field is insufficient. Provenance is honest about what the host exposes. Before
dispatch, the platform-owned supervisor creates an immutable context; its host
adapter stores the actual terminal spawn result in a private mode-0600,
create-new response-envelope store outside the agent/operator sandbox and under
the fixed OS root principal (UID `0`). That store contains the host-issued agent ID,
canonical task path, agent run ID, start/end timestamps, terminal status, and a
nullable host-returned reported-result-commit field. The recorder, not that
field, observes the clean worktree commit and compares it when present. Neither
the agent nor an operator can address, write, or edit the store. The first
context capability `H0` is single-consumption. Capturing an invocation consumes
`H0` and atomically returns successor context capability `H1` plus invocation
handle `HI`; capturing a response consumes the current context capability and
atomically returns its successor plus response handle `HR`. Thus the platform
path is `H0 -> (H1, HI) -> (H2, HR)`, while the no-invocation attestation path is
`H0 -> (H1, HR)`. A predecessor capability becomes invalid as soon as its
successful transaction commits. The recorder atomically consumes the final
context capability and its member handles; no handle can be replayed or mixed
across context generations. Immutable receipt IDs are read-only/queryable
through closure so a failed finalizer retry cannot erase provenance. If this
supervisor boundary is unavailable, neither provenance mode is evidence-bearing.

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
Both variants contain `mode`, `context: {body, sha256}`, and
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

- Context: `{schema,context_nonce,harness_bundle_revision,harness_bundle_sha256,evidence_id,evidence_attempt,role,wave,review_attempt,run_binding,baseline_commit,review_target,canonical_task_path,worktree_handle,requested_model,requested_reasoning_effort,created_at}` with schema `tersh-host-dispatch-context-v1`.
- Invocation: `{schema,context_nonce,harness_bundle_revision,harness_bundle_sha256,dispatch_id,requested_model,requested_reasoning_effort,selected_model,selected_reasoning_effort,dispatched_at}` with schema `tersh-host-spawn-invocation-v1`.
- Response: `{schema,context_nonce,harness_bundle_revision,harness_bundle_sha256,dispatch_id,agent_id,canonical_task_path,agent_run_id,started_at,ended_at,terminal_status,reported_result_commit,reported_record_sha256}` with schema `tersh-host-spawn-response-v2`.

`context_nonce` and `dispatch_id` are 64 lowercase hex; evidence/review attempts,
IDs, roles, waves, run bindings, commits, task path, and agent IDs use the shared
grammars in this section. `worktree_handle` matches
`^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$`; `review_target` and
`reported_result_commit` are JSON null or a full lowercase commit.
`reported_record_sha256` is null only when the callback exposes no agent record;
every required wave and closure response carries its 64-lowercase-hex canonical
record digest. This is the closed response-v2 schema and supersedes the legacy
response shape; no parser accepts the new field as an optional extension.
`harness_bundle_revision` is the independently approved 40-lowercase-hex source
revision of the installed evidence harness and `harness_bundle_sha256` is the
64-lowercase-hex digest of its canonical root-owned bundle manifest. All bodies
in one context generation repeat both values byte-for-byte; a bundle upgrade
starts a new evidence attempt and no accepting attempt may mix bundle identities.
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

The global finding-source union is exactly every review body's `findings` plus
every `audit-reservation-failure` diagnostic body's `findings`. Finding IDs are
unique and monotonically allocated across both source classes; a duplicate,
cross-source alias, or omission is invalid. Finding IDs match exactly
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

Every finding in an audit-failure tombstone, including every P0/P1, must be
closed by a resolution in a canonical review from a strictly later
attempt/candidate. Its `correcting_commit` must equal that later candidate and
its `verifying_review_ref` must resolve to an independent review in that later
history that names and verifies the correction. A tombstone cannot resolve
itself, reuse a review from the failed attempt, form a parent/resolution cycle,
or borrow another source's finding ID. Preimage finalization and offline
structural verification both reject an unresolved, duplicate, circular,
cross-source, or wrong-attempt tombstone finding even though the tombstone
itself remains mandatory history.

Any Closure A/B finding returns to Wave B. A source, test, script, workflow, or product-documentation change after candidate freeze closes the current evidence attempt as failed, creates a new candidate commit in the next three-digit attempt, and invalidates prior external runs and closure reports without overwriting them. Every pre-freeze Wave A/B candidate already occupies its own earlier attempt. Wave A/B reports bind `run-local`; Wave C and closure bind the exact final `run-set`. The finalizer starts from the evidence family's policy-fixed `local`/`external` projection-root union, enumerates every attempt from `001` through the accepting attempt and its one canonical candidate-SHA namespace, and embeds in order the complete canonical body and SHA-256 of every present attempt-level `runner-inventory.json` and `before-repo-runs.json`, and every orchestration, review, gate JSON, bootstrap, selected-run, `jobs.json`, artifact-index, combined external manifest, Cycle 7 audit pair, and audit-reservation-failure tombstone. Runner inventory is required once external preflight starts; the before-run snapshot is required after that inventory passes and forbidden when inventory fails before snapshot, so state-justified absence is not silently filled. It validates but never embeds the bounded raw gate logs. It rejects an attempt gap, a missing/multiple/different candidate namespace, a wrong root class, a noncanonical extra path in either root, a missing state-required or superseded record, reused path, task/run identity mismatch, wrong model/reasoning or dishonest provenance claim, accepting-candidate drift, missing cumulative/catalog/direct-gate hashes, any final FAIL, or any unresolved review/tombstone P0/P1; it never stores only hashes and discards canonical raw history.

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

- Modify: `scripts/run_exact_test.py`
- Modify: `scripts/tests/test_run_exact_test.py`
- Modify: `scripts/evidence_core.py`
- Modify: `scripts/implementation_evidence/run_gate.py`
- Modify: `scripts/implementation_evidence/host_envelope_adapter.py`
- Create: `scripts/implementation_evidence/record_orchestration.py`
- Create: `scripts/implementation_evidence/finalize_iteration.py`
- Create: `scripts/implementation_evidence/commit_and_close.py`
- Create: `scripts/implementation_evidence/verify_formal_lineage.py`
- Create: `scripts/implementation_evidence/repair_projections.py`
- Create: `scripts/implementation_evidence/run_external_candidate.py`
- Create: `scripts/implementation_evidence/verify_ci_evidence.py`
- Create: `scripts/implementation_evidence/verify_release_candidate.py`
- Create: `scripts/implementation_evidence/gate_catalog.json`
- Create: `scripts/implementation_evidence/run_cumulative_gates.py`
- Modify: `scripts/tests/test_implementation_evidence.py`
- Modify: `.gitignore:2` — retain `/target/`, add the explicit anchored
  `/target/implementation-evidence/` rule used by the clean-candidate checks,
  and ignore Python `__pycache__/` plus `*.py[cod]` diagnostic bytecode

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
- `test_each_attempt_binds_exactly_one_candidate_and_candidate_change_allocates_next_attempt`
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
- `test_cumulative_catalog_resolves_only_pinned_runtime_tokens_and_rejects_bare_path_or_unknown_executables`
- `test_cumulative_catalog_resolves_only_registered_bundle_entrypoints_and_ignores_candidate_script`
- `test_cumulative_catalog_rejects_embedded_unknown_dollar_or_shell_placeholders`
- `test_ci_verifier_requires_unique_exact_successful_job_ids`
- `test_release_verifier_requires_descriptor_smoke_manifest_and_ready_identity`
- `test_finalize_requires_wave_a_b_c_and_five_role_closure`
- `test_finalize_requires_five_distinct_agent_runs_and_rejects_cross_role_dispatch_reuse`
- `test_finalize_crosschecks_orchestrator_agent_task_and_run_ids`
- `test_orchestration_attempt_is_three_character_string_and_run_attempt_is_positive_integer`
- `test_orchestration_rejects_noncanonical_task_agent_finding_parent_or_resolution_refs`
- `test_run_binding_rejects_empty_doubled_trailing_duplicate_reordered_zero_or_unknown_components`
- `test_resolution_ref_binds_attempt_candidate_run_file_and_canonical_body_hash`
- `test_host_envelope_adapter_requires_distinct_peer_credential_unix_stream_socket`
- `test_host_envelope_adapter_rejects_same_principal_fifo_stdin_regular_file_trailing_bytes_and_reuse`
- `test_host_envelope_adapter_requires_root_peer_root_owned_socket_and_nonroot_client`
- `test_host_transaction_old_early_half_close_sequence_reproduces_epipe`
- `test_host_transaction_rejects_frame_order_end_trailing_nonce_digest_and_reply_before_commit`
- `test_host_transaction_linearizes_consumption_before_reply_without_partial_consume`
- `test_context_capability_rotates_and_rejects_replay_cross_generation_or_partial_consume`
- `test_host_context_binds_root_owned_harness_bundle_revision_and_digest`
- `test_bundle_manifest_exact_tree_registration_and_runtime_profile_are_root_owned`
- `test_formal_evidence_producers_execute_only_from_registered_digest_pinned_bundle`
- `test_attempt_binding_pins_bundle_runtime_policy_candidate_tree_and_predecessor`
- `test_open_attempt_internal_receipts_are_closed_idempotent_and_reject_post_preimage_or_closure_reopen`
- `test_finalizer_enumerates_host_receipt_ledger_without_client_selected_ids`
- `test_later_host_only_attempt_blocks_earlier_enumerate_audit_or_seal`
- `test_receipt_query_pages_bind_complete_host_order_without_frame_overflow`
- `test_host_spool_receipts_and_projected_raw_tree_require_exact_bijection`
- `test_projection_root_classes_are_policy_fixed_and_repair_exact_union`
- `test_agent_report_sealer_binds_dispatch_and_terminal_report_digest`
- `test_agent_report_sealer_uses_host_selected_authority_after_orchestration_consumes_handles`
- `test_manifest_preimage_and_idempotent_post_commit_closure_bind_formal_pass`
- `test_verify_formal_lineage_derives_selectors_from_fixed_manifest_and_requires_fresh_authenticated_fd`
- `test_repair_projections_host_enumerates_without_client_ids_before_reviews_after_lost_reply`
- `test_append_producer_batch_is_all_or_none_and_rejects_chunk_hash_end_or_partial_commit`
- `test_each_formal_cli_uses_distinct_fresh_fd_and_nested_producer_never_inherits_it`
- `test_seal_manifest_preimage_snapshots_streams_payload_and_retries_idempotently_on_one_fd`
- `test_commit_close_formal_query_and_audit_pair_have_closed_ordered_replies`
- `test_commit_close_sanitizes_git_hooks_filters_config_and_never_inherits_host_fd`
- `test_commit_close_and_formal_query_reject_replace_graft_shallow_alternate_or_ref_race`
- `test_seal_commit_query_require_current_max_attempt_and_replay_only_exact_sealed_lineage`
- `test_audit_pair_lost_reply_replays_frozen_snapshot_without_self_inclusion`
- `test_fail_audit_reservation_atomically_receipts_tombstone_and_unblocks_next_attempt`
- `test_audit_failure_findings_require_later_unique_resolution`
- `test_offline_verify_only_reports_structural_pass_host_unverified`
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
- `test_gate_logs_validate_when_present_but_receipted_json_survives_missing_projection_logs`
- `test_finalize_enumerates_every_attempt_and_its_single_candidate_subtree_from_policy_root_union`
- `test_finalize_rejects_attempt_gaps_unindexed_extras_or_missing_superseded_records`
- `test_finalize_requires_gpt_5_6_sol_xhigh_for_every_role_and_wave`
- `test_finalize_rejects_candidate_drift_unresolved_p0_p1_and_missing_gate`
- `test_finalize_rejects_zero_test_inventory_and_non_evidence_tree_changes`

Replace (do not retain alongside it) the already-implemented
`test_attempt_root_is_candidate_independent_and_per_commit_records_are_immutable`.
That legacy test permits two candidates in one attempt and is mutually exclusive
with the host binding invariant above. The replacement proves candidate A binds
one attempt, candidate B requires the next host-opened attempt, and neither
create-new history can be overwritten or replayed.

The agent-report fixture first completes `append-platform`/`attest` and proves
the final context/member handles are consumed before opening the sealer session.
It then drives the zero-selector `seal-agent-record` CLI through the exact
Host-selected authority, wrong/missing callback digest, wrong orchestration
receipt, draft mutation/path swap, pre-COMMIT retry, and post-COMMIT lost-reply
plus projection-repair cases. No fixture is allowed to hand the sealer a context
handle or authority body. The Git fixture uses real repositories to install
replace refs, grafts, shallow state, alternates, hooks, filters, and a ref/Git-dir
swap and asserts both close and formal-query paths inspect the raw unchanged
object graph with no inherited Host FD.

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
candidate push. Host-envelope framing fixtures use an `AF_UNIX/SOCK_STREAM`
socketpair and a scripted host state machine. A regression fixture first runs
the obsolete sequence in which the host shuts down writes before the
request/reply exchange, and requires its later reply to fail with `EPIPE`; the
accepted fixture follows the exact BEGIN/BODY/BODY-END/COMMIT/REQUEST-END/
REPLY/REPLY-END order below; enumeration fixtures additionally require the
host-generated SUMMARY/PAGE/PAGE-END stream before BODY. Negative transcripts cover an early half-close,
missing, duplicate, extra, or reordered frames, missing or duplicate end
markers, bytes or frames after an end marker, early or late EOF, reply before
commit, wrong frame type/count/transaction nonce/body digest, noncanonical or
open JSON and host-enumeration page gaps, reorder, duplicate IDs, count/digest
drift, a client-supplied receipt selector, or a page over 128 IDs. Receipt
fixtures cover totals `128`, `129`, `325`, and `999` without raising the frame
bound or truncating history; deletion of a failed spool/projection record and an
unreceipted local extra both fail the three-way bijection. Projection fixtures
exercise the implementation shared-root and hardening split-root maps, every
class/record/run-binding row, wrong/third/missing roots, cross-class duplicates,
and exact two-class repair with no caller-selected root. They create two
different evidence IDs at `attempt-001` under the same family root, require
distinct `CLASS_ROOT/EVIDENCE-ID/attempt-001` paths, and reject any receipt or
repair join that omits, duplicates, or embeds a conflicting evidence ID.
Dedicated extended
operation fixtures truncate, duplicate, reorder, and add extra RECORD/SPOOL
chunks and end markers; corrupt chunk, record, descriptor, ordered-body, and
snapshot digests; inject a failure at every item of a multi-record batch; and
prove `append-producer-batch` exposes either every spool body and consecutive
receipt or none. Separate fixtures give each formal top-level producer a distinct
FD, prove its descendant cannot inherit or reuse that FD, exercise lost-reply
idempotence for `seal-manifest-preimage` and `commit-and-close`, and require the
closed phase order and exact aggregate fields for formal-lineage query and the
two-record audit transaction. They create a later host-only marker attempt
before sealing and require every attempt to finalize only at the registry
maximum; fault-injected inconsistent later-binding state makes close and formal
query fail defensively. Once a preimage exists, `open-attempt` fails before
creating any marker, and the identical seal/close retry remains the sole path to
formal closure. Audit
lost-reply replay streams the frozen pre-pair prefix, excludes its own receipts,
and cannot append the pair twice. The root-internal audit-failure fixture races
pair versus failure, requires the trusted non-PASS callback join, loses and
replays the callback result under the exact session/reservation key, and rejects
a changed body/digest or cross-session replay without minting another authority.
It fault-injects every authority-consume, terminal-bit, spool/fsync, receipt-link,
and idempotency-row boundary, proves the failure body/spool/receipt appear
atomically and replay read-only, rejects a changed diagnostic or any same-attempt
reservation/pair, and permits only the next attempt while preserving the failed
attempt as mandatory history. It then
injects unresolved, duplicate cross-source, circular, same-attempt, and
wrong-candidate finding resolutions and accepts only the unique later correction
plus independent verifying review. Bundle fixtures reject
an unregistered ID, extra/missing/symlink/writable bundle member, wrong runtime
profile, mixed-attempt bundle, and candidate-root executable. Agent-report
fixtures require response-v2's terminal digest and reject a changed draft,
wrong dispatch/generation/destination, null required digest, or operator
attestation substituted for byte binding. Pre-COMMIT
failures prove the same input handles remain retryable and no store mutation
occurs. Post-COMMIT missing, malformed, or trailing REPLY fixtures prove stdout
is empty, all predecessor/member handles are already invalid, their replay is
rejected, and the atomic transition produced every successor/receipt—never a
partial or absent transition. A capture successor capability whose reply is
rejected remains private and unreachable and never becomes evidence. A
receipt-backed spool/preimage/closure/audit object exposes no caller capability,
but remains discoverable only through its host-selected snapshot or idempotent
key; it becomes evidence only after the unique projection/lineage is repaired
and all later validation succeeds. The sealer lost-reply fixture therefore finds
exactly one enumerable review receipt, proves its authority is already consumed,
repairs the byte-identical projection, and never appends a second receipt.
Socketpair tests
exercise framing only. The internal credential-parser seam accepts synthetic
peer/store/current UID triples, and its only positive class is `(0, 0,
nonzero)`. It rejects matching nonroot `(501, 501, 502)`, nonroot peer
`(501, 0, 502)`, nonroot FD owner `(0, 501, 502)`, and root client `(0, 0, 0)`.
Production always obtains family, type, owner, peer credentials, and effective
UID from the kernel and exposes no expected-principal, environment, or test
override. The ordinary nonroot repository suite runs one real production-CLI
same-UID negative socketpair but never requires root, changes UID, or skips a
privileged positive while reporting success. The UID-0 TCB-created socketpair
followed by a dropped-nonroot adapter is tested only by privileged host
preflight/acceptance when the platform actually supplies that supervisor; when
it does not, formal evidence fails closed. FIFO/plain-pipe, regular-file, stdin,
wrong-nonce, replay, caller-JSON, closed-schema, receipt-binding, and
missing-supervisor cases all fail. Hung child
fixtures cover Git, `gh`, artifact download, and
verifier calls. Tests never contact GitHub and prove no push occurs when the
online inventory is empty, partial registration cancels only already bound
nonterminal IDs once, cancellation drains every bound run to a terminal
observation within the reserved cleanup budget, and every failure still
publishes one combined manifest.

- [ ] **Step 2: Run the harness tests and confirm RED**

Run:

```bash
python3 -B -m unittest scripts.tests.test_run_exact_test scripts.tests.test_implementation_evidence -v
```

Expected: FAIL because the shared scripts do not exist.

- [ ] **Step 3: Implement the exact CLI contracts**

`scripts/run_exact_test.py` accepts:

```text
python3 -B scripts/run_exact_test.py (--test TARGET | --lib) --name FULL_NAME [--ignored] [--serial] [--case-matrix MATRIX --expect-case CASE_ID ...] [--cargo-bin PATH]
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
worktree/agent/operator sandbox, running as the fixed OS root principal UID `0`. Its private
mode-0700 store and mode-0600 create-new entries are not mounted or
pathname-addressable in the agent/operator namespace. The repository does not
create that principal, store, socket, or platform metadata; a preflight fails
closed when the current host does not expose this supervisor.

The supervisor also owns the complete evidence-harness launch boundary.
`SUPERVISOR_PINNED_PYTHON` and `SUPERVISOR_HARNESS_ROOT` below are protocol
metavariables resolved from root-owned supervisor configuration, never
environment variables, PATH lookups, caller arguments, or shell substitutions.
Every `text` fence containing a `SUPERVISOR_*` or `FRESH_HOST_FD` token is
ordered execution-plan notation, not one shell program or one argv. Each logical
command whose first element is `SUPERVISOR_*` is a separate closed argv template
(backslash-wrapped physical lines are one template); preceding Git/setup/check
lines are separate unprivileged preparation steps and must finish before the
supervisor launch. Within a formal argv, quoted `$TERSH_*` spellings are symbolic
typed slots populated from the supervisor's already validated candidate/attempt
state—not shell expansion—and `FRESH_HOST_FD` is a newly created numeric FD.
The closed slot resolver obtains candidate/tree/attempt values from the immutable
attempt binding; evidence, attempt, projection, and output roots and fixed
destinations from the registered policy row; iteration IDs and review/draft
paths from the dispatch plus policy grammar; and fixture roots only from a
trusted producer's mode-0700 create-new directory whose opened identity is
retained for the session. It never accepts any such path or identifier from an
environment or operator string.
The renderer rejects an unknown slot, a value different from the validated
state, command substitution, or an attempt to send a formal argv through a
shell, then performs one direct `execve` per formal command.
`SUPERVISOR_HARNESS_ROOT` is a separately installed, root-owned harness bundle:
every parent directory, manifest, entrypoint, policy file, and transitive import
is regular/no-follow, is non-agent-writable and not operator-writable, and is covered by one
canonical digest-pinned manifest. That closed manifest binds the approved
40-hex harness revision and relative entrypoint/policy paths plus their SHA-256
values. The separately registered runtime profile binds pinned Python/stdlib,
Bash, Git, `gh`, Cargo stable and Cargo 1.88.0, rustc/rustdoc/rustfmt/Clippy,
`cargo-deny`, the platform linker/toolchain roots, and every verifier runtime
path, exact-tree digest, version, and executable hash;
the gate catalogs and policies that exist in that bundle generation; every
schema, case matrix, job/artifact allowlist, and acceptance threshold used by
those entrypoints; and no unlisted executable or import.

The manifest file is exactly `bundle.json` with schema
`tersh-evidence-harness-bundle-v1` and closed body
`{schema,bundle_revision,protocol_version,files,entrypoints,policies}`. Every
`files` member is exactly `{path,kind,byte_count,sha256}`, where `kind` is
`entrypoint|python-module|policy|schema-data`; every entrypoint is exactly
`{name,path,operation}`; every policy is exactly `{name,path,sha256}`. Paths are
unique canonical relative paths. The bundle directory's no-follow regular-file
set must equal `bundle.json` plus the manifest's nonempty file set, with no
symlink, hard-link alias, writable parent, or extra member. `bundle_id` is the
SHA-256 of canonical `bundle.json` bytes including LF; `bundle.json` never
contains its own ID. A separate root-registry `tersh-host-runtime-profile-v1`
binds the absolute pinned Python/stdlib root and the Bash, Git, `gh`, both Cargo
toolchains, rustc/rustdoc/rustfmt/Clippy, `cargo-deny`, linker/toolchain roots,
and verifier paths, versions, exact-tree identities, and executable digests; its canonical hash is
`runtime_profile_id`, so machine paths never enter the portable bundle digest.
The root registry exposes three exact closed registration bodies:
`tersh-host-bundle-registration-v1`
`{schema,bundle_registration_receipt_id,bundle_id,bundle_revision,manifest_byte_count,manifest_sha256,created_at}`;
`tersh-host-runtime-profile-registration-v1`
`{schema,runtime_profile_registration_receipt_id,runtime_profile_id,profile_byte_count,profile_sha256,created_at}`;
and `tersh-host-policy-registration-v1`
`{schema,policy_registration_receipt_id,bundle_registration_receipt_id,policy_sha256,policy_byte_count,created_at}`.
These authenticated registration receipts contain only bounded size/digest
joins, not copies of possibly large bundle or policy objects, so each fits one
65,536-byte frame. The runtime profile itself has schema
`tersh-host-runtime-profile-body-v1`, contains the closed tool-name map and
absolute path, version, executable/tree digest for every runtime named above,
is at most 60 KiB, and hashes to `runtime_profile_id` without containing that ID.
Whenever a client needs a tool, the host sends one closed
`tersh-host-runtime-profile-pair-v1` `{schema,registration,profile}` whose
registration size/hash exactly joins the profile. Before any
trusted helper launches a subprocess, its authenticated transaction supplies
that exact runtime-profile pair BODY, the helper matches its ID to the
attempt binding, rehashes the selected absolute executable/tree, and calls it by
absolute argv. No helper accepts a runtime path from environment, PATH, candidate
content, or a caller-controlled free-form flag.

Across its registered generations, the evidence TCB includes `evidence_core.py`,
every capture/record/finalize entrypoint, both exact and cumulative runners,
local gate wrappers, external runner/verifiers, implementation-entry and
requirement auditors, and every transitive helper whose structured output is
accepted as proof. The closed TCB also includes the UID-0 supervisor and its
typed argv renderer, the Host Envelope daemon and closed-frame parser, the
root-owned bundle/runtime/policy installer and registration verifier, the
append-only registry/ledger and host-only spool, and the kernel peer-credential,
no-follow/create-new/fsync, and Git-object primitives those components invoke
through the pinned runtime profile. Candidate
product tests, binaries, workflows, and test scripts remain untrusted subjects:
the trusted harness records their exact Git blobs, argv, cwd, and toolchain, but
candidate code or a candidate tree must never be executed as an evidence
producer and never receives a Host Envelope FD. A clean worktree is data to
verify, not executable trust. Agent drafts and local projections are likewise
untrusted inputs until joined to host spool bytes and detached receipts; GitHub,
Actions, TLS, and runner/job/artifact observations remain external assertions
that only the pinned verifiers may accept under their closed policies. Installing
or upgrading a harness bundle is an
independent root/operator approval outside the candidate; it invalidates the
current evidence attempt, and no bundle may approve or install itself.
The initial bundle installed by Task 1 contains only the implementation files,
implementation catalog, schemas, and policies created by Task 1. Its formal
entrypoints are `host-envelope-adapter` with the three capture operations,
`run-gate`, `run-cumulative`, `run-external`, `verify-ci`, `verify-release`,
`record-orchestration`, `seal-agent-record`, `finalize-iteration`,
`commit-and-close`, `verify-formal-lineage`, `repair-projections`, and
`run-exact-test`; it neither names
nor requires a hardening catalog, `finalize-cycle`, an implementation-entry
auditor, or a requirements auditor that does not yet exist. Hardening Task 1
may nevertheless use this generation's already registered generic
`record-orchestration` and `seal-agent-record` policy rows to host-seal Wave A/B
reports under the closed `hardening-0N` evidence/role/destination grammar. Those
receipts are formal provenance history but cannot execute or approve a hardening
gate, policy, finalizer, or PASS. Hardening Task 1
later builds and independently registers a successor bundle generation that
adds exactly those reviewed hardening files and policies. That upgrade starts a
new hardening evidence attempt, never rewrites an implementation receipt, and
cannot retroactively bless an earlier attempt. Repository files with the same
names are development or diagnostic inputs only and never formal producers.

Before every launch the supervisor verifies the pinned manifest and absolute
root-owned, non-group/world-writable interpreter and harness component chain,
then calls `execve` directly without a shell. A trusted producer resolves every
policy command's executable from the attempt-bound runtime profile, rechecks its
digest immediately before `execve`, and supplies only the profile's minimal
fixed environment. No formal argv contains a PATH-resolved executable; Cargo's
transitive rustc/rustdoc/rustfmt/Clippy/linker resolution is confined to the
profile's exact root-owned toolchain trees with wrapper/plugin variables absent.
Every evidence-bearing entrypoint
is launched from the harness bundle with the exact prefix
`SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/...`. The sole shell
wrapper is launched as
`SUPERVISOR_PINNED_BASH SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_prior_gates.sh`;
neither its shebang nor `/usr/bin/env` participates in formal resolution. `-I` ignores
all `PYTHON*` inputs and removes caller import paths, `-S` prevents site and
`sitecustomize` startup, and `-B` prevents bytecode writes to the bundle.
The supervisor supplies a minimal fixed environment with no `PYTHON*`,
`LD_*`, `DYLD_*`, startup, user-site, or caller search-path entries and closes
every inherited descriptor except the transaction socket and fixed stdio.
Repository tests use the current absolute `sys.executable` only as an
unprivileged stand-in for that pinned interpreter. A direct/non-isolated script
launch is never formal evidence; each host-bound `__main__` fails before opening
the transaction FD unless isolated, no-site, and no-bytecode flags are all set.

Pinned execution alone is not a record credential. Independent root/operator
installation first appends a bundle-registration receipt in the private host
registry; `bundle_id` is the SHA-256 of the canonical bundle manifest including
LF and equals the context bodies' `harness_bundle_sha256`. Dispatch capture may
reserve an attempt number and store provisional host-only context before the
result commit exists, but it does not create an evidence attempt binding. Once
the candidate is known, and before launching any formal producer for that
attempt, the root supervisor performs one internal `open-attempt` transaction.
Wave A uses its known baseline; Wave B waits for the response's reported result
commit or the proven unchanged baseline after a failed/no-change response.
`open-attempt` atomically creates exactly one closed
`tersh-host-attempt-binding-v1`
`{schema,attempt_binding_id,evidence_id,evidence_attempt,candidate,candidate_tree,worktree_handle,bundle_id,bundle_registration_receipt_id,runtime_profile_id,runtime_profile_registration_receipt_id,policy_sha256,policy_registration_receipt_id,predecessor_attempt_binding_id,created_at}`.
All receipt/binding/runtime/policy/bundle IDs are 64 lowercase hex; Git commits,
trees, and the installed harness revision are 40 lowercase hex. Attempt `001`
has null predecessor; every later three-digit attempt must name the host-registry
binding for the immediately preceding attempt of the same evidence ID. Bundle,
runtime, policy, candidate, tree, worktree, and predecessor are immutable; any
change requires the next attempt rather than rebinding the current one. In that
same host linearization point, `open-attempt` create-new spools and appends the
sequence-1/2 receipts for policy-derived `attempt-marker` and `candidate-marker`
records. Their bodies are exactly `tersh-evidence-attempt-marker-v1`
`{schema,evidence_id,evidence_attempt,attempt_binding_id,candidate,created_at}` and
`tersh-evidence-candidate-marker-v1`
`{schema,evidence_id,evidence_attempt,attempt_binding_id,candidate,candidate_tree,worktree_handle,created_at}`;
their destinations are exactly `attempt-NNN/attempt.json` and
`attempt-NNN/candidate-SHA/candidate.json`. Missing projections can be repaired
from those host bodies, but neither marker can be omitted or client-authored.
The host's unique idempotency key is `(evidence_id,evidence_attempt)`: retrying
the same complete binding/marker tuple returns the existing binding and two
receipts, while any candidate/tree/worktree/bundle/runtime/policy/predecessor
drift conflicts. A lost post-COMMIT reply therefore cannot mint another binding
or another sequence-1/2 marker pair. This is a UID-0 supervisor-to-host internal
operation, never a candidate-visible Host Envelope FD. Its closed request is
`tersh-host-open-attempt-request-v1`
`{schema,evidence_id,evidence_attempt,candidate,candidate_tree,worktree_handle,bundle_id,runtime_profile_id,policy_sha256,predecessor_attempt_binding_id}`;
the host resolves and revalidates all three registration receipts rather than
accepting them from the request. It assigns one internal producer session with
literal `entrypoint="open-attempt"`, `producer_mode="harness"`, and the
root-registry `open-attempt` policy row, then returns exactly
`tersh-host-open-attempt-result-v1`
`{schema,binding:{body,sha256},attempt_marker_receipt:{body,sha256},candidate_marker_receipt:{body,sha256}}`.
Those two receipts use that session and policy identity and are the complete
sequence-1/2 receipt objects described below with
`projection_root_class="local"`. An identical replay of an existing
binding remains read-only and idempotent. An unpaired `reserve-audit-draft`
result for the current attempt blocks every later `open-attempt`; the audit pair
or the receipted `fail-audit-reservation` tombstone must atomically consume it
first. Once any manifest-preimage or detached
manifest-closure receipt exists for an evidence ID, creation of every later
attempt permanently conflicts; a sealed or closed lineage cannot be reopened by
appending a marker-only attempt. All finding-driven returns to Wave B and new
candidates therefore occur before `seal-manifest-preimage`; after that terminal
freeze, only an exact seal/close retry may progress the evidence ID.
The internal operation linearizes at one root-database atomic commit that either
creates the binding, both spool markers, and both receipts or creates nothing;
the result is emitted only after that commit. It is not one of the framed Host
Envelope transactions below and has no transaction nonce, COMMIT frame, or
REQUEST-END.
Every later producer receives an already-persistent binding and starts its
receipt chain after the two markers. A Wave B number is only a host-local
tentative reservation until the response establishes the result candidate; a
failure before `open-attempt` may release and reuse that number only
after the supervisor terminally invalidates every context/member capability and
proves no host receipt or external effect exists. If a result commit may exist,
recovery resolves and binds it before any retry. No irreversible external
effect, including a push, may start before `open-attempt` commits. A crash
thereafter leaves a failed but enumerable marker-only attempt,
so the next attempt never creates a ledger gap. Exactly one candidate SHA
namespace must match every persisted binding, including every populated root
class.

Every accepted orchestration, review, gate, cumulative, external, audit, and
finalization-input record is produced in a root-launched producer session. The
trusted producer streams canonical bytes to the host-only spool protocol; that
spool is never mounted or pathname-addressable to the agent. At the single host
COMMIT, the host atomically create-new publishes and fsyncs one or more complete
spool bodies and appends one closed receipt for each body. It never makes only
one side visible. Each receipt uses
`tersh-host-producer-receipt-v1`
`{schema,receipt_id,attempt_binding_id,producer_session_id,sequence,previous_receipt_id,producer_mode,entrypoint,bundle_id,runtime_profile_id,policy_entry_id,policy_entry_sha256,environment_capability,projection_root_class,record_class,record_schema,destination,body_sha256,byte_count,dispatch_id,reported_record_sha256,created_at}`.
`sequence` starts at one and is consecutive; `previous_receipt_id` is null only
for sequence one and otherwise equals the preceding host receipt. `record_class`
is exactly `attempt-marker|candidate-marker|gate|cumulative-gates|runner-inventory|before-repo-runs|bootstrap|selected-run|jobs|artifact-index|external-candidate|orchestration|review|implementation-entry|requirements|completion-audit|audit-reservation-failure`.
`projection_root_class` is exactly `local|external`, agrees with the immutable
root map and record/run-binding classification above, and is derived by policy.
`destination` is the unique canonical path relative to that class root's joined
`attempt_binding.evidence_id` directory, never a caller absolute/free-form root
and never a second embedded evidence ID. In `producer_mode=harness`, both
agent-report fields are null. In `producer_mode=agent-report`, `entrypoint` is
exactly `seal-agent-record` and both `dispatch_id` and
`reported_record_sha256` are 64 lowercase hex. `dispatch_id` equals the joined
invocation/response dispatch ID, `reported_record_sha256` binds the pre-seal
draft, and `body_sha256` binds the exact final body. `policy_entry_id` names one closed row in the
attempt-bound bundle policy and `policy_entry_sha256` hashes that canonical row;
the receipt, session, and finalizer must agree. `environment_capability` is null
unless the bound policy row requires one. Its only nonnull arm is the closed
`tersh-host-environment-capability-v1`
`{schema,capability_id,kind,attempt_binding_id,root_a,root_b,created_at,expires_at}`
with `kind="distinct-writable-filesystems-v1"`; each root is exactly
`tersh-host-opened-directory-v1`
`{schema,path,device,inode,owner_uid,mode}`. Paths are canonical absolute,
device/inode are positive integers, owner is the fixed nonroot producer UID,
mode is exactly decimal `448` (`0700`), both opened directories remain live for
the session, and their devices differ. Marker and agent-report receipts always
carry null. A producer session in the root registry also
binds its exact entrypoint/policy/runtime digests, argv, cwd identity,
timestamps, exit/signal truth, and stdout/stderr/API observation hashes. The
host rejects a producer-session ID reused by another transaction, duplicate
`(producer_session_id,record_ordinal)`,
`(projection_root_class,attempt_binding.evidence_id,destination)`, sequence, or record body;
all records within one batch intentionally share that session ID.

The agent-writable evidence tree is only a create-new projection and never the
authority or receipt selector. A crash before producer COMMIT leaves neither a
spool record nor a receipt. A lost reply after COMMIT can leave the atomic
receipt-backed spool body without a projection; the next host snapshot exposes
it without relying on a client-held receipt ID. The finalizer requires an exact
bijection between the complete host receipt chain, host-only spool bodies, and
every projected canonical raw record. Before checking that bijection, the
root-owned finalizer may repair only a missing projection by create-new writing
the exact host-spooled bytes to the receipt-derived no-follow destination and
fsyncing its parent. An existing projection is never replaced; an existing
mismatch, local extra, body/destination mismatch, sequence gap, or receipt/spool
disagreement invalidates the attempt.

Formal CLI policy flags are assertions, never authority. The registered policy
row derives the gate name and argv, catalog prefix, repository/workflows,
jobs/artifacts/labels/timeouts, record classes/destinations, finalizer required
gates and sole manifest destination, and commit message. If an illustrative CLI
below retains any of those flags for operator readability, the trusted
entrypoint requires byte-for-byte, order-preserving equality with its bound
policy row and rejects a missing, extra, duplicated, weakened, or reordered
value before launching a child or writing the spool. Identity selectors are
limited to evidence ID, attempt, candidate, and the fresh authenticated FD.

After that bijection the finalizer constructs a canonical manifest payload with
no preimage, closure, or self-referential host-binding member and asks the host
to create exactly one closed
`tersh-host-manifest-preimage-receipt-v1`
`{schema,receipt_id,attempt_binding_id,finalizer_session_id,bundle_id,runtime_profile_id,manifest_kind,evidence_id,accepting_attempt,candidate,destination,payload_sha256,payload_byte_count,attempt_binding_count,ordered_attempt_binding_ids_sha256,producer_receipt_count,ordered_producer_receipt_ids_sha256,spool_record_count,spool_byte_count,ordered_body_sha256s_sha256,ordered_spool_body_sha256s_sha256,ordered_spool_join_sha256s_sha256,created_at}`.
`manifest_kind` is exactly `implementation-iteration|hardening-cycle`. At this
COMMIT the host re-enumerates its complete evidence-ID history through the
accepting attempt, requires every count and digest aggregate to match, and atomically seals
those attempts against later producer appends. The final file is exactly
`{schema:"tersh-evidence-manifest-envelope-v1",payload,payload_sha256,preimage_receipt:{body,sha256}}`.
Every implementation-iteration and hardening-cycle payload schema contains one
shared exact `host_history` field and never repeats those bodies elsewhere:
`host_history` is the ordered array of
`{attempt_binding:{body,sha256},records:[{producer_receipt:{body,sha256},record:{body,sha256}}]}`.
Attempt rows and each nested receipt list are in host ledger order; all other
payload fields are the closed policy-required semantic summaries and may refer
to history entries only by their canonical digest. Offline structural
verification can therefore recompute all
predecessor, receipt, body, spool, and join aggregates without `target/`. The
embedded receipt body/hash must match payload bytes, size, destination,
identity, and the host enumeration, avoiding a self-hash.

Root-owned `commit-and-close` independently verifies the index/worktree contain
only the envelope, creates the evidence-only Git commit, then requires clean
HEAD, `HEAD^ == candidate`, a diff containing only `destination`, and committed
blob bytes identical to that envelope. Its host COMMIT appends a
detached closed `tersh-host-manifest-closure-receipt-v1`
`{schema,receipt_id,preimage_receipt_id,attempt_binding_id,bundle_id,runtime_profile_id,manifest_kind,evidence_id,accepting_attempt,candidate,destination,manifest_sha256,manifest_byte_count,evidence_commit,manifest_blob_oid,created_at}`.
The host keys that closure uniquely by `preimage_receipt_id`: an identical
evidence commit retry returns it, while a different commit conflicts. That is
the final formal-PASS linearization point. Online verification must
query and match the bundle registration, attempt binding, preimage receipt, and
closure receipt. Offline `--verify-only` may return only
`STRUCTURAL_PASS/HOST_UNVERIFIED`; it can never upgrade an absent or unreachable
closure receipt into formal `PASS`.

The supervisor is the sole context creator. Before spawn it fixes the exact
`tersh-host-dispatch-context-v1` body above, injects the installed manifest's
`harness_bundle_revision` and `harness_bundle_sha256`, generates the 256-bit
context nonce, stores the body, and starts the agent only after this exact
host-side command returns successfully:

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/host_envelope_adapter.py capture-context --host-store-fd FD
```

For each command the supervisor injects `FD > 2` as the one pre-opened connected
`AF_UNIX/SOCK_STREAM` socket for that transaction; it is not inherited by the
agent and is not accepted from stdin, a pathname, environment, or a caller-owned
pipe. Before sending any byte, the client obtains the address family and
connected state from the socket, requires `SO_TYPE == SOCK_STREAM`, calls
`fstat(FD)`, and obtains the platform peer credential. On macOS it calls
`getsockopt(SOL_LOCAL=0, LOCAL_PEERCRED, 76)` and unpacks the exact 76 returned
bytes as native `=III16I`; `xucred` version is exactly `0` and group count is
`0..16`. On Linux it calls `getsockopt(SOL_SOCKET, SO_PEERCRED, 12)`, requires
exactly 12 bytes, and unpacks native `=iii` pid/uid/gid, all nonnegative. The
production trust anchor is the fixed OS root principal: the parsed peer UID must
equal `0`, `fstat(FD).st_uid` must independently equal `0`, and the client's
effective UID must be nonzero. Equality between an arbitrary nonroot peer UID
and socket owner is never sufficient. The UID-0 TCB creates the connected socket
pair and launches the adapter only after dropping the adapter process to its
nonroot worktree UID. Any unsupported production platform or unavailable exact
kernel check fails before protocol I/O. There is no configured supervisor UID,
`--expected-uid`, environment, test-mode, pathname, stdin, body, nonce, model,
identity, or same-principal/root-client override.

Every protocol frame is four-byte big-endian unsigned length `1..65536`
followed by exactly that many compact, sorted-key, closed-schema canonical UTF-8
JSON bytes including one trailing LF. Duplicate keys, non-finite numbers,
noncanonical bytes, unknown keys, and a boolean where an integer is required
fail. A transaction nonce, context/invocation/response handle, receipt ID, every
SHA-256 field, and every digest-array member is exactly 64 lowercase hex. The
client generates a fresh transaction nonce and sends one BEGIN frame. For every
capture or record operation, the host then sends the fixed BODY sequence and
BODY-END. For `enumerate-evidence`, BEGIN selects only an evidence ID and
terminal attempt; the host sends its bounded SUMMARY, receipt-ID PAGE sequence,
PAGE-END, then every attempt-binding and producer-receipt BODY in canonical
ledger order, with the matching host-spool stream immediately after each
receipt, plus aggregate BODY-END. The agent-writable tree supplies no ID,
count, page, digest, or omission filter. The host keeps its write side open in
all cases. After validating all bodies and relationships, the client
sends COMMIT, REQUEST-END, and then `shutdown(SHUT_WR)`. The host must accept
that exact end frame and EOF before the transaction reaches its single atomic
commit point; it then sends REPLY, REPLY-END, and `shutdown(SHUT_WR)`. The client
accepts a result only after the exact reply, end frame, and EOF. Missing, extra,
duplicated, or reordered frames, wrong type/count/nonce/digest, early or late
EOF, bytes after an end marker, or a reply before COMMIT fails with empty stdout.

The frame schemas and field sets are closed as follows:

- BEGIN has schema `tersh-host-transaction-begin-v1`. The `capture-context` arm
  is exactly `{schema,transaction_nonce,operation}`. The `capture-invocation`
  and `capture-response` arms are exactly
  `{schema,transaction_nonce,operation,context_handle}`. The `append-platform`
  arm is exactly
  `{schema,transaction_nonce,operation,context_handle,invocation_handle,response_handle}`.
  The `attest` arm is exactly
  `{schema,transaction_nonce,operation,context_handle,response_handle}`.
  `enumerate-evidence` instead uses schema
  `tersh-host-enumerate-evidence-begin-v1` and exactly
  `{schema,transaction_nonce,operation,evidence_id,through_evidence_attempt}`,
  where operation is `enumerate-evidence`. The attempt is an assertion only: before
  SUMMARY the host requires it to equal the maximum registered attempt for that
  evidence ID, including a later marker-only or failed attempt whose projection
  is absent. Any caller-supplied receipt/binding ID, count, page, digest,
  candidate, omission selector, or earlier attempt is an extra/mismatch and
  fails.
- Only `enumerate-evidence` has host enumeration frames before BODY. SUMMARY is
  exactly
  `{schema,transaction_nonce,operation,evidence_id,through_evidence_attempt,attempt_binding_count,producer_receipt_count,spool_record_count,spool_byte_count,page_count,ordered_attempt_binding_ids_sha256,ordered_producer_receipt_ids_sha256,ordered_spool_body_sha256s_sha256,ordered_spool_join_sha256s_sha256}`
  with schema `tersh-host-enumerate-evidence-summary-v1`. Counts are positive,
  attempts are exactly `001..through_evidence_attempt`, and `page_count` equals
  `ceil(producer_receipt_count / 128)`. Each host PAGE is exactly
  `{schema,transaction_nonce,operation,page_index,page_count,total_receipt_count,ordered_producer_receipt_ids_sha256,receipt_ids}`
  with schema `tersh-host-enumerate-evidence-page-v1`; pages are one-based and
  consecutive, every nonfinal page has 128 globally unique IDs, and the final
  page has `1..128`. PAGE-END is exactly SUMMARY's aggregate fields plus
  `{schema,query_pages_sha256}` with schema
  `tersh-host-enumerate-evidence-pages-end-v1`, where `query_pages_sha256`
  hashes the canonical ordered PAGE-frame digest array including LF. The client
  sends no frame between BEGIN and the host's SUMMARY/PAGE/PAGE-END/BODY stream.
  `spool_record_count` equals `producer_receipt_count`. A spool join item is
  exactly `{receipt_id,destination,body_sha256,byte_count}`; its aggregate digest
  hashes the canonical ordered join-item digest array including LF.
- Every BODY wrapper is exactly
  `{schema,transaction_nonce,operation,body_kind,ordinal,total,body,body_sha256}`
  with schema `tersh-host-transaction-body-v1`; `ordinal` is one-based, `total`
  is the operation's exact positive body count, and `body_sha256` hashes the
  nested body's canonical bytes including its LF. For capture and record
  operations BODY-END is exactly
  `{schema,transaction_nonce,operation,total,body_sha256s}` with schema
  `tersh-host-transaction-body-end-v1`; its ordered digest array equals the BODY
  sequence byte-for-byte. Immediately after every producer-receipt BODY, the
  host emits exactly one spool stream. SPOOL-BEGIN is
  `{schema,transaction_nonce,operation,record_ordinal,record_count,receipt_id,record_class,record_schema,destination,byte_count,body_sha256,chunk_count}`
  with schema `tersh-host-spool-begin-v1`. It is followed by exactly
  `chunk_count` SPOOL-CHUNK frames
  `{schema,transaction_nonce,operation,record_ordinal,chunk_ordinal,chunk_count,encoding,data,chunk_sha256}`
  with schema `tersh-host-spool-chunk-v1` and `encoding="base64"`; data is RFC
  4648 canonical padded base64, `chunk_sha256` hashes decoded bytes,
  `chunk_count == ceil(byte_count / 32768)`, and decoded bytes are exactly
  32,768 bytes except the final nonempty `1..32768` bytes. SPOOL-END
  is
  `{schema,transaction_nonce,operation,record_ordinal,record_count,receipt_id,byte_count,body_sha256,chunk_count,ordered_chunk_sha256s_sha256}`
  with schema `tersh-host-spool-end-v1`. Reconstructed bytes are one closed-
  schema canonical JSON body including LF and match the adjacent receipt's
  destination, byte count, and hash. Enumeration BODY-END is exactly
  `{schema,transaction_nonce,operation,total,attempt_binding_count,producer_receipt_count,spool_record_count,spool_byte_count,ordered_attempt_binding_ids_sha256,ordered_producer_receipt_ids_sha256,ordered_body_sha256s_sha256,ordered_spool_body_sha256s_sha256,ordered_spool_join_sha256s_sha256}`
  with the same schema. The host emits each attempt binding in attempt order,
  followed by that attempt's producer receipts in sequence order; `total` is
  the two positive counts' sum. IDs match SUMMARY/PAGE/PAGE-END, every receipt
  chain is consecutive with the exact previous ID, and the body digest hashes
  the canonical complete ordered BODY-digest array including LF without placing
  any unbounded digest array in one frame.
- COMMIT has schema `tersh-host-transaction-commit-v1`. Capture arms are exactly
  `{schema,transaction_nonce,operation,body_sha256s}`. The two record arms are
  exactly
  `{schema,transaction_nonce,operation,body_sha256s,record_facts}`.
  `append-platform` record facts are exactly
  `{evidence_id,evidence_attempt,run_binding,candidate,destination,record_sha256}`.
  `attest` record facts have those fields plus exactly
  `{operator_id,attested_at,reason,requested_model,requested_reasoning_effort}`.
  The candidate is the clean observed HEAD, destination is the derived
  candidate-relative orchestration path, reason is exactly
  `platform-model-metadata-unavailable`, and requested model/effort are exactly
  `gpt-5.6-sol`/`xhigh`. `record_sha256` hashes the complete canonical record
  exactly as written to the host spool and later projected. The detached receipt
  joins it by destination, byte count, and digest; no receipt ID is inserted into
  or omitted from the record body.
  The enumeration arm is exactly
  `{schema,transaction_nonce,operation,evidence_id,through_evidence_attempt,attempt_binding_count,producer_receipt_count,spool_record_count,spool_byte_count,ordered_attempt_binding_ids_sha256,ordered_producer_receipt_ids_sha256,ordered_body_sha256s_sha256,ordered_spool_body_sha256s_sha256,ordered_spool_join_sha256s_sha256}`
  and every value equals SUMMARY, PAGE-END, and the validated BODY sequence.
  At its read-only COMMIT linearization point the host again requires
  `through_evidence_attempt` to be the registry maximum and every aggregate to
  match the current ledger; a concurrently opened later attempt is snapshot
  drift and yields no result.
  REQUEST-END is exactly
  `{schema,transaction_nonce,operation,commit_sha256}` with schema
  `tersh-host-transaction-request-end-v1`, where the digest covers the canonical
  COMMIT frame.
- Each capture or record REPLY is exactly
  `{schema,transaction_nonce,operation,body_sha256s,result}` with schema
  `tersh-host-transaction-reply-v1`. For `capture-context`, result is exactly
  `{schema,context_handle}` with schema `tersh-host-capture-context-result-v1`.
  For `capture-invocation`, it is exactly
  `{schema,context_handle,invocation_handle}` with schema
  `tersh-host-capture-invocation-result-v1`. For `capture-response`, it is
  exactly `{schema,context_handle,response_handle}` with schema
  `tersh-host-capture-response-result-v1`. For either record arm, it is exactly
  `{schema,receipt}` with schema `tersh-host-record-result-v1`, and `receipt` is
  the exact closed receipt body below. An `enumerate-evidence` REPLY is instead
  exactly
  `{schema,transaction_nonce,operation,evidence_id,through_evidence_attempt,attempt_binding_count,producer_receipt_count,spool_record_count,spool_byte_count,ordered_attempt_binding_ids_sha256,ordered_producer_receipt_ids_sha256,ordered_body_sha256s_sha256,ordered_spool_body_sha256s_sha256,ordered_spool_join_sha256s_sha256,result}`
  with schema `tersh-host-transaction-reply-v1`; its result is exactly
  `{schema,evidence_id,through_evidence_attempt,attempt_binding_count,producer_receipt_count,spool_record_count,spool_byte_count,ordered_attempt_binding_ids_sha256,ordered_producer_receipt_ids_sha256,ordered_body_sha256s_sha256,ordered_spool_body_sha256s_sha256,ordered_spool_join_sha256s_sha256}`
  with schema `tersh-host-enumerate-evidence-result-v1`, and every aggregate
  matches BEGIN, SUMMARY, PAGE-END, BODY-END, and COMMIT. No reply repeats the
  receipt-ID or body-hash arrays. REPLY-END is exactly
  `{schema,transaction_nonce,operation,reply_sha256}` with schema
  `tersh-host-transaction-reply-end-v1`, where the digest covers the canonical
  REPLY frame.

Five state-changing/verification operations extend that transport without
raising the frame bound or reusing a half-closed connection. Whenever the client
uploads a canonical record, RECORD-BEGIN is exactly
`{schema,transaction_nonce,operation,record_ordinal,record_count,record_class,record_schema,destination,byte_count,body_sha256,chunk_count,dispatch_id,reported_record_sha256}`
with schema `tersh-host-record-begin-v1`. It is followed by `chunk_count`
RECORD-CHUNK frames exactly
`{schema,transaction_nonce,operation,record_ordinal,chunk_ordinal,chunk_count,encoding,data,chunk_sha256}`
with schema `tersh-host-record-chunk-v1`, `encoding="base64"`, and RFC 4648
canonical padded base64. `chunk_sha256` hashes decoded bytes; `chunk_count`
equals `ceil(byte_count / 32768)`, and the stream follows the same 32,768-byte
decoded chunk rule as SPOOL-CHUNK. RECORD-END is exactly
`{schema,transaction_nonce,operation,record_ordinal,record_count,byte_count,body_sha256,chunk_count,ordered_chunk_sha256s_sha256}`
with schema `tersh-host-record-end-v1`. A record descriptor is exactly
`{record_ordinal,record_class,record_schema,destination,byte_count,body_sha256,chunk_count,dispatch_id,reported_record_sha256}`;
descriptor and ordered-aggregate hashes cover canonical bytes including LF.
Each reconstructed body must itself be one canonical closed-schema JSON object
including LF. An ordinary record is `1..16,777,216` decoded bytes, one producer
batch contains `1..128` records and at most `268,435,456` decoded bytes, and the
single manifest-payload upload is `1..268,435,456` bytes. The declared byte and
chunk counts must satisfy these limits before allocation or reads; audit records
use the ordinary limit. `dispatch_id` and `reported_record_sha256` are both null
for harness records and both 64-hex for an agent-report record.

- `append-producer-batch` uses BEGIN schema
  `tersh-host-append-producer-batch-begin-v1` and exact fields
  `{schema,transaction_nonce,operation}`. The authenticated
  producer session, not the caller, already binds evidence ID, attempt,
  candidate, bundle/runtime, entrypoint, and policy row. The host first sends
  exactly three BODYs: `attempt-binding`, `producer-session`, then the exact
  `runtime-profile` pair, where the session
  is the closed
  `{schema,producer_session_id,attempt_binding_id,entrypoint,bundle_id,runtime_profile_id,policy_entry_id,policy_entry_sha256,environment_capability,agent_report_authority}`
  body with schema `tersh-host-producer-session-v1`. The binding must be the
  existing immutable result of `open-attempt`; a missing binding or either marker
  fails before any child launch. The pair's
  `registration.runtime_profile_id` must equal both the binding and session
  `runtime_profile_id`, and
  `registration.runtime_profile_registration_receipt_id` must equal the
  binding's `runtime_profile_registration_receipt_id`. `environment_capability`
  is null or the exact object above and must equal every resulting receipt;
  `agent_report_authority` is null except for `entrypoint="seal-agent-record"`.
  In that arm it is exactly `tersh-host-agent-report-authority-v1`
  `{schema,dispatch_id,attempt_binding_id,agent_id,canonical_task_path,agent_run_id,context_body_sha256,invocation_body_sha256,response_body_sha256,reported_result_commit,reported_record_sha256,orchestration_receipt_id,draft_path,review_destination}`.
  The invocation hash and result commit alone may be null; all other identities
  and digests are nonnull, the existing orchestration receipt must join the same
  immutable captured context/response, and both paths are derived by policy.
  Those three BODYs are followed by BODY-END
  exactly `{schema,transaction_nonce,operation,total:3,body_sha256s}` under
  `tersh-host-transaction-body-end-v1`. Only after validating those bodies does
  the producer launch its policy-bound child and learn the state-justified result
  set. The client then sends exactly the same actual `record_count` in every
  RECORD stream, followed by COMMIT exactly
  `{schema,transaction_nonce,operation,record_count,ordered_host_body_sha256s_sha256,ordered_record_descriptor_sha256s_sha256,ordered_record_body_sha256s_sha256}`.
  Under one lock the host revalidates every closed policy/destination/schema
  join. For a sealer session it additionally requires every record descriptor
  and review body's dispatch ID, reported digest, attempt, identities, and
  destination to equal the Host-selected `agent_report_authority`; for a
  capability session it requires the session/receipt capability objects to be
  identical and still live. It then create-new writes and fsyncs all spool bodies, and appends the same number
  of consecutive attempt-global receipts. Batch sequence values are consecutive
  and the first receipt's `previous_receipt_id` links to the preceding receipt
  from any earlier producer session. REPLY is exactly
  `{schema,transaction_nonce,operation,attempt_binding_id,producer_session_id,record_count,first_sequence,last_sequence,ordered_host_body_sha256s_sha256,ordered_record_body_sha256s_sha256,ordered_receipt_ids_sha256,result}`
  with schema `tersh-host-transaction-reply-v1`;
  result is exactly
  `{schema,record_count,first_sequence,last_sequence,ordered_record_body_sha256s_sha256,ordered_receipt_ids_sha256}`
  with schema `tersh-host-append-producer-batch-result-v1`. It never returns a
  caller-selectable receipt array. `run-gate`, cumulative/external collectors,
  and the agent-record sealer use this operation; a missing reply is recovered
  only from a later host-selected snapshot.

- `seal-manifest-preimage` uses BEGIN schema
  `tersh-host-seal-manifest-preimage-begin-v1` and exact fields
  `{schema,transaction_nonce,operation,evidence_id,through_evidence_attempt,manifest_kind}`.
  The host sends the same bounded SUMMARY/PAGE/PAGE-END, attempt-binding/
  producer-receipt BODYs, SPOOL streams, and aggregate BODY-END defined above.
  Before SUMMARY, the host requires `through_evidence_attempt` to equal the
  registry's maximum attempt for that evidence ID. There are exactly two legal
  states. In create state, that maximum binding is current and unsealed and the
  unique `(terminal_attempt_binding_id,manifest_kind,destination)` preimage key
  is absent. In replay state, that same maximum binding is already sealed by the
  one preimage at that key; the host streams the immutable snapshot frozen by
  that preimage and accepts only the identical selector, aggregates, destination,
  and payload bytes. A sealed binding without that exact preimage, a second key,
  or any later binding fails. A later marker-only/failed host attempt therefore
  blocks sealing or replaying an earlier apparently passing projection. The
  stream uses schemas `tersh-host-evidence-snapshot-summary-v1`,
  `tersh-host-evidence-snapshot-page-v1`,
  `tersh-host-evidence-snapshot-pages-end-v1`, and
  `tersh-host-evidence-snapshot-body-end-v1`. The finalizer validates the full
  predecessor/receipt/spool chain and constructs the payload only from those
  host bytes. It then sends exactly one RECORD stream with
  `record_class="manifest-payload"`, the manifest-kind payload schema, and the
  policy-derived destination. This is the sole non-receipt RECORD upload class;
  it proposes preimage content and is outside the producer-receipt record-class
  enum. COMMIT is exactly
  `{schema,transaction_nonce,operation,evidence_id,through_evidence_attempt,manifest_kind,attempt_binding_count,producer_receipt_count,spool_record_count,spool_byte_count,ordered_attempt_binding_ids_sha256,ordered_producer_receipt_ids_sha256,ordered_body_sha256s_sha256,ordered_spool_body_sha256s_sha256,ordered_spool_join_sha256s_sha256,payload_sha256,payload_byte_count,destination}`.
  Under the COMMIT lock the host revalidates that the selected attempt is still
  the registry maximum. In create state it re-enumerates, requires every
  aggregate still equal, validates the payload bytes/schema, seals all included
  attempts against later appends, and appends the unique preimage keyed only by
  `(terminal_attempt_binding_id,manifest_kind,destination)`. An identical
  `payload_sha256` retry in replay state returns that receipt; a different
  selector, aggregate, destination, or payload at the same lineage key conflicts
  and leaves that evidence ID terminally non-PASS rather than authorizing a new
  attempt, so
  `commit-and-close` can resolve exactly one preimage without a client receipt
  selector.
  REPLY is exactly the COMMIT aggregate fields plus
  `{schema,transaction_nonce,operation,preimage_receipt:{body,sha256}}` under
  schema `tersh-host-transaction-reply-v1`. Only after validating REPLY-END and
  EOF may the finalizer create-new publish the manifest envelope. A lost reply is
  retried on a fresh FD with the identical selector/payload and returns the same
  preimage; it never creates a second seal.

- `commit-and-close` uses BEGIN schema `tersh-host-commit-and-close-begin-v1`
  and exact fields
  `{schema,transaction_nonce,operation,manifest_kind,evidence_id,accepting_attempt,candidate,destination}`;
  no receipt ID is a selector. Before sending a BODY, the host requires
  `accepting_attempt` to equal the registry maximum for the evidence ID, requires
  that binding to be sealed by the exact uniquely resolved preimage, and rejects
  any later binding. The host sends exactly `terminal-attempt-binding`,
  its `runtime-profile` pair, then `manifest-preimage-receipt` BODY and
  the exact three-digest BODY-END. While that one FD remains open, the
  trusted entrypoint validates the envelope and clean index, creates the sole
  evidence commit (or recognizes an identical prior commit after a lost reply),
  and sends COMMIT exactly
  `{schema,transaction_nonce,operation,manifest_kind,evidence_id,accepting_attempt,candidate,destination,preimage_receipt_id,manifest_sha256,manifest_byte_count,evidence_commit,manifest_blob_oid,ordered_host_body_sha256s_sha256}`.
  Under the COMMIT lock the host rechecks the same maximum-attempt and exact-seal
  invariants, independently resolves the unique preimage, and append-or-returns the
  closure keyed only by `preimage_receipt_id`; the same evidence commit is an
  idempotent retry and any different commit conflicts. REPLY is exactly
  `{schema,transaction_nonce,operation,ordered_host_body_sha256s_sha256,result}`
  with schema `tersh-host-transaction-reply-v1`; result is exactly
  `{schema,closure_receipt:{body,sha256}}` with schema
  `tersh-host-commit-and-close-result-v1`. A retry
  accepts only clean `HEAD^ == candidate`, the same sole changed destination/blob, and
  the same commit; it never creates a second Git commit.

- `query-formal-lineage` uses BEGIN schema
  `tersh-host-query-formal-lineage-begin-v1` and exact fields
  `{schema,transaction_nonce,operation,manifest_kind,evidence_id,accepting_attempt,candidate,destination,payload_sha256,manifest_sha256}`.
  The host resolves, rather than accepts IDs for, the unique lineage. Before
  SUMMARY and again under the COMMIT lock it requires `accepting_attempt` to
  equal the registry maximum, requires the unique closure to close that exact
  terminal binding/preimage, and rejects any later binding or alternate closure.
  It sends
  SUMMARY exactly
  `{schema,transaction_nonce,operation,manifest_kind,evidence_id,accepting_attempt,candidate,destination,attempt_binding_count,bundle_registration_count,runtime_profile_count,policy_registration_count,total_body_count}`
  with schema `tersh-host-formal-lineage-summary-v1`, followed by BODYs in this fixed order: all attempt bindings; distinct
  bundle registrations sorted by ID; distinct runtime-profile pairs sorted by ID;
  distinct policy registrations sorted by ID; the preimage receipt; and the
  closure receipt. `total_body_count` is the four counts plus two. BODY-END is exactly
  `{schema,transaction_nonce,operation,attempt_binding_count,bundle_registration_count,runtime_profile_count,policy_registration_count,total_body_count,ordered_body_sha256s_sha256}`
  with schema `tersh-host-formal-lineage-body-end-v1`.
  The verifier validates every distinct runtime-profile registration/pair in the
  lineage. For Git execution it must select exactly one pair whose
  `runtime_profile_id` equals the terminal accepting binding's ID; a missing,
  duplicate, or older-attempt pair cannot supply the executable. It then
  rehashes and invokes that pair's absolute pinned Git to derive the evidence
  commit and manifest blob OID.
  COMMIT is exactly
  `{schema,transaction_nonce,operation,manifest_kind,evidence_id,accepting_attempt,candidate,destination,payload_sha256,manifest_sha256,evidence_commit,manifest_blob_oid,attempt_binding_count,bundle_registration_count,runtime_profile_count,policy_registration_count,total_body_count,ordered_body_sha256s_sha256}`.
  REPLY is exactly the COMMIT fields plus `result`, under schema
  `tersh-host-transaction-reply-v1`; result is
  `{schema,formal_status:"PASS"}` under schema
  `tersh-host-formal-lineage-result-v1`. The trusted verifier reports formal
  PASS only after independently matching local envelope/blob/parent truth and
  validating this root-peer-authenticated reply plus EOF.

Cycle 7 audit authoring starts with a separate UID-0 supervisor/host internal
`reserve-audit-draft` operation; it is not a candidate-facing Host Envelope
transaction and receives no agent-selected path or revision. Its exact request
is `tersh-host-reserve-audit-draft-request-v1`
`{schema,audit_request_id,evidence_id,evidence_attempt,candidate,attempt_binding_id}`.
The host requires `evidence_id="hardening-07"`, the current maximum unsealed,
non-`audit_failed_terminal` attempt binding and candidate, a fresh 64-hex
`audit_request_id`, and no existing unpaired reservation. Under one root-ledger lock it allocates the next contiguous
three-character `audit_revision`, derives the diagnostic
`requirements-draft-revision-RRR.json` path from registered policy, and appends
exactly one result
`tersh-host-audit-draft-reservation-v1`
`{schema,audit_reservation_id,audit_request_id,evidence_id,evidence_attempt,candidate,attempt_binding_id,audit_revision,draft_path,predecessor_audit_reservation_id,created_at}`.
`audit_reservation_id` is 64 lowercase hex;
`predecessor_audit_reservation_id` is null only for revision `001` and otherwise
the immediately preceding 64-hex reservation ID. `created_at` uses the shared
UTC RFC 3339 grammar.
The same request tuple is read-only idempotent after a lost result; any changed
field conflicts. A second request while the first reservation is unpaired,
skipped/reused revision, preoccupied path, wrong predecessor, or concurrent
winner/loser mismatch fails. Once `audit-and-append-pair` atomically consumes a
reservation, a later mapping-only retry uses a new request and the next revision.
Because the draft is explicitly untrusted and unreceipted, validation failure
does not consume its reservation: the operator may no-follow delete only that
exact reserved diagnostic file and create-new corrected bytes at the same path.
It cannot allocate another revision until the reserved pair commits or the
failure transition below consumes it. Once paired, the path is immutable and
any mapping change requires the next reservation.
The current accepting attempt's maximum reservation must be paired before that
attempt can finalize. A failed reservation makes its attempt permanently
ineligible but, through the transition below, permits the next attempt; an
outstanding unpaired reservation makes every older audit pair ineligible, so the
allocator can never be bypassed by falling back.

A substantive trusted-audit failure that requires a new candidate first creates
one Host-held, single-use failure authority. At the registered
`audit-requirements` session's non-PASS terminal callback, before any pair
COMMIT, the host requires the session's attempt binding, evidence ID, candidate,
audit revision, bundle/runtime identity, and exact policy row to equal the
unpaired reservation. It validates the closed, at-most-256-KiB diagnostic body
`tersh-audit-requirements-failure-v1`
`{schema,evidence_id,evidence_attempt,candidate,audit_revision,failure_class,findings}`,
where `failure_class` is exactly
`candidate-repair-required|external-evidence-invalid|trusted-audit-input-missing`
and `findings` is a nonempty canonical list of the shared closed finding objects.
Under one durable host transaction keyed by
`(producer_session_id,audit_reservation_id)` it create-once stores the pending
`tersh-host-audit-failure-authority-v1`
`{schema,audit_failure_authority_id,audit_reservation_id,producer_session_id,attempt_binding_id,evidence_id,evidence_attempt,candidate,audit_revision,bundle_id,runtime_profile_id,policy_entry_id,policy_entry_sha256,diagnostic,created_at}`,
where `diagnostic` is exactly `{body,sha256}`, then returns to the UID-0
supervisor only the closed internal callback result
`tersh-host-audit-failure-callback-result-v1`
`{schema,audit_failure_authority_id,diagnostic_sha256}`. The auditor CLI itself
emits empty stdout and a bounded nonzero diagnostic. An identical replay of the
same registered terminal callback returns the stored authority/result read-only;
a changed diagnostic or callback tuple conflicts, and a lost callback result is
recovered only through that Host-held replay key rather than a caller-supplied
authority. A PASS callback, a paired
or failed reservation, a second authority for one reservation, or any missing,
wrong, stale, cross-session, cross-reservation, cross-attempt, bundle, runtime,
policy, body, or digest join creates no authority. A pending authority blocks
`audit-and-append-pair` for that reservation.
`audit_failure_authority_id` and every exposed diagnostic digest are 64
lowercase hex, and the stored callback timestamp uses the shared UTC grammar.

The UID-0 supervisor then invokes the separate host-internal
`fail-audit-reservation` operation; deleting or silently forgetting the
reservation is forbidden. Its exact request is
`tersh-host-fail-audit-reservation-request-v1`
`{schema,audit_failure_authority_id}` and contains no diagnostic, reservation,
path, producer-session, or policy selector. In the absent-key create branch,
the host resolves that exact pending authority and requires its reservation to
remain current, unpaired, and unfailed. One crash-atomic durable transaction
consumes the authority, marks both the reservation failed and its attempt
`audit_failed_terminal`, create-new writes and fsyncs the exact
`tersh-host-audit-reservation-failure-v1`
`{schema,audit_failure_authority_id,audit_reservation_id,audit_producer_session_id,attempt_binding_id,evidence_id,evidence_attempt,candidate,audit_revision,diagnostic,created_at}`
spool body at policy destination
`attempt-NNN/candidate-SHA/completion/audit-reservation-revision-RRR-failed.json`,
appends exactly one attempt-global receipt with the correct sequence and
previous-receipt link, and records the idempotent result. That receipt uses a
root-internal `producer_mode="harness"` session bound to the failed attempt's
immutable bundle/runtime identity, literal `entrypoint="fail-audit-reservation"`,
the registered policy row, `projection_root_class="local"`, and
`record_class="audit-reservation-failure"`; its receipt's
`producer_session_id` is that new root-internal failure session, while the
tombstone's `audit_producer_session_id` equals the auditor session in the
consumed authority. The request cannot override either identity or any field.
The transaction's journal exposes either
all authority-consume/terminal-state/spool/receipt/result changes after durable
commit or none, and sends no result before that point. Its exact result is
`tersh-host-fail-audit-reservation-result-v1`
`{schema,reservation_failure:{body,sha256},producer_receipt:{body,sha256}}`.
In the existing-key lost-result branch, the same authority ID resolves its
already-consumed immutable result and returns that body/receipt read-only even
though the reservation is no longer unpaired; it never reruns the create
preconditions or appends another receipt. A different, stale, reused, or
cross-reservation authority conflicts.

An `audit_failed_terminal` attempt can never reserve another audit revision,
pair any revision, seal, close, or be accepted, but the consumed authority no
longer blocks the next `open-attempt`. The next attempt receives the next
contiguous global audit revision. The failed attempt, diagnostic, spool, and
receipt remain enumerable and embedded in every later manifest, and no
finalizer may fall back to any older audit pair. A lost internal result may be
replayed by authority ID to recover the stored result; a missing projection is
then recovered by host enumeration plus `repair-projections`. Neither path
appends another tombstone. Invalid draft syntax or mapping correctable without
changing the candidate stays on the same unpaired reservation and creates no
failure authority.

- `audit-and-append-pair` uses BEGIN schema
  `tersh-host-audit-and-append-pair-begin-v1` and exact fields
  `{schema,transaction_nonce,operation,evidence_id,through_evidence_attempt,audit_revision}`,
  with evidence ID fixed to `hardening-07`. The revision must name the Host
  reservation for this exact attempt/candidate, and both policy-derived RECORD
  destinations must join its revision; the caller cannot select or skip a
  revision. In the absent-key create branch that reservation must be current and
  unconsumed, and its attempt must not be `audit_failed_terminal` or contain any
  audit-failure authority/tombstone from another revision. In the existing-key lost-reply branch it must be the same
  already-paired reservation and can only replay the frozen result. Before
  streaming and again under the
  COMMIT lock, the host requires `through_evidence_attempt` to equal the maximum
  registered hardening-07 attempt and its binding to be current and unsealed; a
  later host-only failed attempt cannot be omitted by choosing an older value.
  If `(terminal_attempt_binding_id,audit_revision)` is absent, the host sends the
  same complete snapshot and spool stream as `seal-manifest-preimage`. If that
  key already exists after a lost reply, it instead replays the immutable
  pre-COMMIT snapshot prefix frozen for that pair—excluding the pair's own two
  receipts and every later receipt—and accepts only the identical two bodies,
  descriptors, package digest, and COMMIT. A different package at that revision
  conflicts; replay never snapshots its own receipts or appends them twice. The
  auditor then sends
  exactly two RECORD streams. For audit revision `RRR`, ordinal 1 is the
  host-derived policy destination
  `completion/requirements-revision-RRR.json` and ordinal 2 is
  `completion/completion-audit-revision-RRR.json`; the caller supplies neither
  destination. Both body revisions must equal BEGIN. COMMIT is exactly
  `{schema,transaction_nonce,operation,evidence_id,through_evidence_attempt,audit_revision,attempt_binding_count,producer_receipt_count,spool_record_count,spool_byte_count,ordered_attempt_binding_ids_sha256,ordered_producer_receipt_ids_sha256,ordered_body_sha256s_sha256,ordered_spool_body_sha256s_sha256,ordered_spool_join_sha256s_sha256,record_count,ordered_record_descriptor_sha256s_sha256,ordered_record_body_sha256s_sha256,evidence_package_sha256}`,
  where `record_count` is 2 and the package digest hashes canonical
  `[requirements_body_sha256,completion_audit_body_sha256]` including LF. Under
  one lock the host rechecks the snapshot and reservation. Only the absent-key branch create-new
  spools both bodies and appends two consecutive receipts or nothing, keyed
  idempotently only by `(terminal_attempt_binding_id,audit_revision)`, and marks
  that exact reservation paired in the same atomic transition. The
  existing-key branch validates the frozen prefix and identical pair/package/
  COMMIT, touches no destination or sequence, and returns the stored receipts.
  A different package digest at that key conflicts and must use the next audit
  revision. REPLY
  is exactly
  `{schema,transaction_nonce,operation,evidence_id,through_evidence_attempt,audit_revision,requirements:{destination,body_sha256},completion_audit:{destination,body_sha256},evidence_package_sha256,first_sequence,last_sequence,ordered_receipt_ids_sha256,result}`
  with schema `tersh-host-transaction-reply-v1`; result is exactly
  `{schema,audit_revision,requirements:{destination,body_sha256},completion_audit:{destination,body_sha256},evidence_package_sha256,first_sequence,last_sequence,ordered_receipt_ids_sha256}`
  with schema `tersh-host-audit-and-append-pair-result-v1`. It exposes no receipt
  selector. The returned destinations must be those exact revision-bearing
  paths. A lost-reply retry returns the same pair; revision `002` coexists with
  and cannot overwrite revision `001`; a later finalizer repairs
  either missing projection from the host spool without replacing an existing
  path. After snapshot BODY-END the only legal client frames are the fixed
  requirements RECORD stream, the fixed completion-audit RECORD stream, COMMIT,
  and REQUEST-END in that order; another SUMMARY/PAGE/BODY/SPOOL, a third or
  swapped record, or any frame between end markers fails before COMMIT.

Whenever an outer REPLY and its nested `result` repeat a field, every repeated
value must be byte-for-byte/value-equal after canonical decoding. In particular,
the append-batch count/sequence/body/receipt aggregates and the audit
revision/destinations/digests/sequence aggregates may not disagree across the
two closed objects; disagreement is a malformed post-COMMIT reply and yields no
client-visible success.

Every state-changing producer or single-lineage formal verifier CLI receives
exactly one newly created, independently peer-authenticated FD for exactly one
of these transactions. REQUEST-END,
half-close, and EOF permanently retire it. A nested producer never inherits a
parent FD: `run-external` is a top-level producer rather than a `run-gate` child,
and cumulative/external producers batch all of their records on their own fresh
FD. Every `FRESH_HOST_FD` token below denotes a different supervisor injection,
not a shell variable, environment value, or reusable descriptor.
The sole aggregate exception is the Hardening Cycle 7 registered root-bundle
fixed-14 evaluator defined there: it receives exactly fourteen distinct
supervisor-injected descriptors, performs fourteen independent read-only
`query-formal-lineage` transactions, one per descriptor, and aggregates only
their closed results. It never multiplexes two lineages on one descriptor,
passes a descriptor to a child, or reuses a retired FD.

The fixed BODY orders are: `capture-context` = `context`;
`capture-invocation` = `context, invocation`; `capture-response` = `context,
response`; `append-platform` = `context, invocation, response`; `attest` =
`context, response`; and `enumerate-evidence` = each `attempt-binding` in
attempt order followed immediately by its `producer-receipt` BODY and matching
SPOOL stream in sequence order. No operation permits a null, optional,
additional, omitted, or different-kind BODY or spool stream.

Each framed Host Envelope transaction's host-store linearization point is the host's atomic
application of the validated COMMIT after it has accepted exact REQUEST-END plus
EOF. Every failure before that point consumes no host capability and creates no
host binding, spool body, or receipt, leaving all input handles retryable. The
local candidate worktree, index, and HEAD remain unchanged except for
`commit-and-close`'s exact evidence-only Git commit after its BODY validation.
A policy-bound external producer may already have created its one unique remote
ref and associated CI runs before its producer COMMIT; `open-attempt` has
already made that attempt enumerable, and exact ref/run selection, cancel,
terminal drain, and later host-ledger recovery prevent substituting or silently
repeating another run. If the later closure host COMMIT or
reply fails, deterministic `HEAD^`, tree, destination, blob, and preimage checks
recover only that same commit, which remains nonformal until the detached
closure exists. At the host linearization point the daemon performs exactly one indivisible
transition: create `H0`; invalidate `H0` while creating `(H1, HI)`; invalidate
the current context capability while creating `(Hnext, HR)`; invalidate the
final context and every required member handle while atomically creating its
spool body and receipt; install a whole producer batch; seal one idempotent preimage; append
one idempotent closure; append the
two-record audit pair; or return a read-only host snapshot/lineage. No operation
installs only a subset. Once that point has occurred,
all predecessor/member handles are invalid and every successor or receipt
already exists even if REPLY is later missing, malformed, or followed by
trailing data. Such a post-COMMIT failure returns empty stdout and the old
handles cannot be replayed. A successor capability from a capture transaction
is an unaddressable private orphan unless a completely validated reply releases
its handle. Spool bodies, producer receipts, preimages, closures, and audit
records have no caller capability and remain discoverable through their
host-selected snapshot or idempotent retry keys even when the reply is lost.

Immediately before spawn and at its terminal callback, the same supervisor
stores the exact invocation/response bodies with these host-side commands:

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/host_envelope_adapter.py capture-invocation --context-handle CONTEXT_HANDLE_H0 --host-store-fd FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/host_envelope_adapter.py capture-response --context-handle CONTEXT_HANDLE_H1 --host-store-fd FD
```

`capture-context` prints exactly
`{schema,context_handle}` with schema `tersh-host-capture-context-result-v1` and
the returned handle is `H0`. `capture-invocation` consumes `H0` and prints
exactly `{schema,context_handle,invocation_handle}` with schema
`tersh-host-capture-invocation-result-v1`, yielding `H1` and `HI`.
`capture-response` consumes `H1` and prints exactly
`{schema,context_handle,response_handle}` with schema
`tersh-host-capture-response-result-v1`, yielding `H2` and `HR`. Each old context
handle becomes invalid at the host's atomic COMMIT point, before REPLY; the
successor/member handles are printed only after the client validates the entire
reply and EOF. If invocation metadata is absent, the invocation command is not
run; response capture instead consumes `H0` and yields `H1` plus `HR`. The
supervisor records only callback fields it actually exposes, and absence of
terminal identity/lifecycle means no response
handle. Every command uses a fresh authenticated FD and the exact transaction
above; same-principal/fake peers, FIFO or plain pipes, regular files, TTY/stdin,
caller JSON, wrong nonce, replay, trailing bytes, and unknown options fail.

In `platform-envelope` mode the supervisor launches the recorder with a fresh
authenticated store connection:

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/record_orchestration.py append-platform --context-handle CONTEXT_HANDLE_H2 --invocation-handle INVOCATION_HANDLE_HI --response-handle RESPONSE_HANDLE_HR --host-store-fd FD
```

The recorder sends BEGIN and retrieves the three ordered bodies without yet
consuming any handle. It verifies every closed envelope, digest, nonce, dispatch
ID, and duplicated field. The root session supplies and revalidates the bound
worktree candidate/tree directly; this record operation never launches Git or
any other subprocess and therefore needs no runtime-profile BODY. It compares the
nullable reported commit, derives the only permitted destination, and constructs
the complete final record. Its COMMIT carries the ordered body
hashes and exact derived record facts. Only after REQUEST-END plus EOF does the
host atomically consumes `H2`, `HI`, and `HR`, create-new fsyncs the complete
orchestration record in the host-only spool, and appends its consecutive
`producer_mode=harness`, `record_class=orchestration` producer receipt. The
recorder validates REPLY, REPLY-END, and EOF and then create-new publishes the
byte-identical local
projection. A projection failure or post-COMMIT reply failure cannot revive the
handles or erase the host record/receipt; host enumeration exposes the unmatched
side and invalidates the attempt unless an idempotent projection retry restores
the exact bytes. Consume, spool create, and receipt append are one indivisible
transition, never partial. If the captured response has a nonnull
`reported_record_sha256`, that same transition also installs exactly one pending
Host-selected `agent_report_authority` joined to the new orchestration receipt;
if it is null, no formal review authority exists.

That receipt uses the exact shared `tersh-host-producer-receipt-v1` schema.
Its harness-mode agent-report fields are null, its bundle/runtime/entrypoint and
attempt binding come from the authenticated root producer session, and its body
hash/size/destination bind the exact spooled/projected record. The host spool
retains the exact context/invocation/response bodies and
their hashes as part of the record's closed provenance arm. No model, effort,
identity, timestamp, response body, destination, producer, bundle, or receipt
override exists on CLI or environment.

If and only if the authenticated response exists but invocation model metadata
does not, the operator requests this supervisor-launched fallback:

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/record_orchestration.py attest --context-handle CONTEXT_HANDLE_H1 --response-handle RESPONSE_HANDLE_HR --operator-id ID --host-store-fd FD
```

It uses the same retrieve-before-commit transaction, atomically consumes `H1`
and `HR`, derives identity and lifecycle only from the response, independently
matches the root-session-bound worktree commit without launching Git, and writes the exact `operator-attestation` arm
plus fixed reason `platform-model-metadata-unavailable`; only the context's
model/effort request is operator-attested. The CLI rejects identity, task, run,
timestamp, commit, model, effort, output, receipt, or free-form reason arguments.
Its atomic host transition applies the same nonnull-report-digest rule for
creating the one pending `agent_report_authority`.

Reviewer reports use a separate root-bundle `seal-agent-record` entrypoint. The
agent may create-new one draft only at the policy-derived disjoint diagnostic
path
`target/evidence-agent-drafts/EVIDENCE_ID/attempt-NNN/DISPATCH_ID.json`; that
tree is never a formal projection root and is not enumerated by a finalizer. The
response-v2 terminal callback binds the draft's complete canonical SHA-256.
After `append-platform` or `attest` atomically consumes the final context/member
handles and creates the orchestration receipt, the host retains those immutable
captured bodies and derives exactly one pending `agent_report_authority` from
that receipt plus the response callback. The already-consumed `H2`/`HI`/`HR`
are never passed to or reconstructed by the sealer. The root supervisor opens a
new producer session bound to that authority and direct-executes exactly:

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/record_orchestration.py seal-agent-record --host-store-fd FRESH_HOST_FD
```

There is no dispatch, draft, destination, identity, digest, context-handle, or
receipt selector on argv or environment. On that one fresh authenticated FD the
entrypoint performs one `append-producer-batch` transaction: BEGIN; the exact
attempt-binding, producer-session-with-`agent_report_authority`, and runtime
profile BODYs; exactly one review RECORD; COMMIT; REPLY/REPLY-END/EOF. Only
after validating the Host-selected authority does the sealer no-follow open its
`draft_path`, hold the descriptor, rehash it, and require the exact closed report
schema, context/task/attempt/candidate identities, `dispatch_id`, callback
digest, and `review_destination`. At COMMIT the host revalidates that authority,
the existing orchestration receipt and captured response, then indivisibly marks
the authority consumed, publishes the complete host-spool record, and appends a
`producer_mode=agent-report` receipt binding the response's `dispatch_id`, the
pre-seal `reported_record_sha256`, and the final `body_sha256`. The spool body and
formal projection are byte-identical to the complete draft, so the receipt's
`body_sha256` must equal the response's `reported_record_sha256`; the detached
receipt, not a field inserted into the review, supplies host origin. Only the
sealer publishes the formal projection. A pre-COMMIT disconnect leaves the
authority unused and retryable through a newly authenticated session. After the
atomic COMMIT, a lost/malformed reply cannot reuse the authority or append a
second receipt; host enumeration plus `repair-projections` recovers the one
spooled body and projection before any review consumer continues.
A missing callback digest makes the report advisory, never formal; operator
model attestation cannot substitute for report-byte binding. Cross-generation,
wrong-destination, modified-after-callback, duplicate, or replayed drafts fail
without a receipt.

In create mode `finalize_iteration.py` obtains one fresh supervisor-injected FD
and performs the single `seal-manifest-preimage` transaction. Its BEGIN selects
only evidence ID, through-attempt, and manifest kind. The host—not the manifest
or local tree—returns SUMMARY, its complete bounded PAGE sequence, PAGE-END,
every attempt-binding and producer-receipt BODY, every joined SPOOL stream, and
aggregate BODY-END. The finalizer validates the complete predecessor and receipt
chains, repairs only a missing projection from exact host bytes, rejects any
existing mismatch/extra, and requires the three-way bijection. It then streams
the exact proposed payload and sends the snapshot/payload COMMIT on that same
still-open FD. The host re-enumerates and atomically seals the preimage before
replying; only a validated REPLY, REPLY-END, and EOF permits create-new envelope
publication. The transaction never raises the 65536-byte frame limit or
truncates history. Gaps, local extras, duplicates, reorder, oversize, wrong
bundle/runtime/policy, or count/digest drift fail. A copied bundle/receipt string
cannot select or satisfy the host snapshot.

The separately registered,
root-owned `commit-and-close` entrypoint verifies the index/worktree change set
contains only that manifest, creates the evidence-only commit, requires its
parent to equal the candidate and its sole changed blob to equal the envelope,
then idempotently appends or returns the closure receipt under unique key
`preimage_receipt_id`. The same commit returns the existing receipt; a different
commit conflicts. A lost reply never creates a second Git
commit. A semantic failure can leave a nonformal evidence commit, but never a
formal PASS. `--verify-only` rejects a host FD, rehashes embedded structures,
and emits only `STRUCTURAL_PASS/HOST_UNVERIFIED`; formal verification must use a
root-peer-authenticated FD to query the exact bundle registration, preimage,
and detached closure. A missing supervisor, context, response/report digest,
producer receipt, preimage receipt, closure, or create-mode FD means no formal
evidence; no mode invents metadata the callback did not expose.

`commit_and_close.py` accepts exactly `--manifest FIXED_PATH --candidate SHA
--commit-message FIXED_MESSAGE --host-store-fd FD`; path and message are only
assertions that must equal the bound policy row, and no arbitrary output, path, author,
parent, extra staged file, amend, signing override, or Git environment is
accepted. It uses the pinned absolute Git runtime with argument vectors, holds
the no-follow manifest descriptor through blob comparison, requires a clean
index/worktree except for that one untracked manifest, stages and commits it
once, verifies the new tree/parent/blob, and performs the idempotent closure
COMMIT. If Git commit succeeds but closure fails, stdout/formal status remains
non-PASS; retry names the same preimage receipt and evidence commit and may only
query or return the same closure.

Every Git child uses `close_fds=True`, inherits no Host Envelope FD, and receives
no caller Git configuration. The trusted helper supplies a root-policy-fixed
author/committer identity, sets `GIT_NO_REPLACE_OBJECTS=1`, disables signing,
points `core.hooksPath` at a root-owned empty directory, and rejects or clears
caller `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, object-directory,
alternates, namespace, replace-ref, config, and signing environment. Before any
Git child it no-follow opens and freezes the worktree, Git directory, index,
HEAD/ref, object directory, and manifest identities; it rejects any identity or
ref drift at each later boundary. Repository/global/system hooks, aliases,
external diff/textconv, every filter/process/clean/smudge driver,
`refs/replace`, `.git/info/grafts`, shallow state, and configured or environment
object alternates are forbidden. The helper rehashes the manifest directly and
uses replacement-disabled low-level object reads to parse the raw commit bytes,
requiring exactly one parent equal to the candidate plus the independently
verified tree, sole changed path/blob, fixed message, and identities before
host COMMIT. `verify_formal_lineage.py` applies the identical raw-object,
replacement-disabled, frozen-Git-directory rules rather than trusting the view
of an ordinary `rev-parse`. Real malicious-repository fixtures install a replace
ref, graft, shallow file, alternate object directory, hook/filter/config, and a
concurrent ref/Git-directory swap in turn; none executes or changes the raw
parent/tree/blob interpretation, and every forbidden state fails without a host
closure or formal query PASS.

`repair_projections.py` accepts exactly `--evidence-id ID --through-attempt NNN
--host-store-fd FD`; it accepts no root or path selector. The authenticated
session supplies the attempt-bound two-class projection-root map. On its one fresh FD it performs
one read-only `enumerate-evidence` transaction with no caller receipt/binding
IDs, validates the complete ledger/spool join across both classes, and create-new repairs only
missing receipt-backed canonical JSON projections at policy-derived no-follow
destinations. It never synthesizes raw gate logs, overwrites an existing path,
or tolerates a mismatched/extra projection. Stdout is exactly one closed
`tersh-projection-repair-result-v1`
`{schema,evidence_id,through_evidence_attempt,expected_projection_count,existing_projection_count,repaired_projection_count,result}`;
it contains no receipt ID or selectable path. After a producer loses its reply,
the supervisor runs this entrypoint before any Wave C/closure reviewer or later
producer consumes that attempt's gate hashes, run-set, or external result. A
failed repair blocks those consumers but does not erase the host receipt.

`verify_formal_lineage.py` accepts exactly `--manifest FIXED_PATH
--host-store-fd FD`. The fixed manifest path is a policy assertion. Before BEGIN
it no-follow reads the canonical envelope and derives manifest kind, evidence ID,
accepting attempt, candidate, destination, payload digest, and manifest digest;
it accepts no receipt, binding, commit, blob, repository, or status selector.
The host resolves the unique registered lineage and supplies every authenticated
runtime-profile pair referenced by it. The verifier requires exactly one pair
matching the terminal accepting binding and only then rehashes and executes that
pair's pinned absolute Git to derive and validate the evidence commit, parent, and manifest
blob, echo them in `query-formal-lineage` COMMIT, and require the exact PASS
REPLY/REPLY-END/EOF. Stdout is exactly one closed
`tersh-formal-lineage-verification-v1`
`{schema,manifest_kind,evidence_id,accepting_attempt,candidate,evidence_commit,formal_status:"PASS"}`.
Any offline invocation, structural-only envelope, missing root FD, nonunique
lineage, local Git mismatch, or lost/malformed reply emits no PASS.

Before the host accepts REQUEST-END plus EOF, failure cannot create a host-store
orphan or consume a handle; the narrowly specified recoverable Git commit above
is not a host-store object and is never itself formal evidence. After the atomic
COMMIT point, however, a missing, malformed,
or trailing REPLY cannot roll the transition back. A capture-only successor may
become a private, agent-unaddressable capability orphan eligible for bounded
host garbage collection. A committed producer, preimage, or closure receipt is
different: it remains immutable and ledger-enumerable even when the reply or
projection is lost, and no finalizer may omit or garbage-collect it. The client
still emits no unverified handle, receipt, or formal status. This plan does not
require zero private-store effects after a valid COMMIT. Each operation uses one
connection, while later operations receive separate fresh authenticated FDs.

The evidence entrypoints accept only argparse argument vectors. They write
sorted-key compact UTF-8 JSON plus trailing newline using the create-new
hard-link publication protocol above. The cumulative catalog uses schema
`tersh-cumulative-gates-v1`: each iteration names one predecessor and adds a
closed list of `{gate_id,kind,argv,exact_test,serial,ignored,case_matrix,
expected_cases,required_online_labels,required_external_jobs,fixture,outputs}`.
Every field is present. `kind` is exactly `command|exact-test|benchmark`;
`exact_test` and `case_matrix` are null when unused; `serial`/`ignored` are JSON
booleans; every expected/required field is an ordered duplicate-free array;
`fixture` is null or literal `temp-dir`; and `outputs` is a canonical object from
artifact ID to candidate-relative output path, empty when unused. Gate IDs are
globally unique. The initial committed catalog contains the Plan1 G0a/G0b
exact runners, all three downloaded-asset ignored serial smokes, the frozen outcome
matrices, format/Clippy/full/MSRV/policy commands, and CI/release/native job
requirements. Its entries for later iterations are populated from the exact
commands already named by Plans2–5, including Task5's G1a read benchmark,
Task13's mutation benchmark, trash/restore and EXDEV matrices, 40x10 tests,
native EXDEV jobs, and G3 process matrices. `run_cumulative_gates.py --through
impl-NN` resolves every predecessor, rejects a changed/removed/duplicate/zero
inventory entry, and calls the shared non-CLI `collect_gate` primitive inside the
same trusted process; it never spawns the host-bound `run_gate.py` entrypoint or
passes its sole FD to a child. After collecting every closed gate body and the
cumulative body, it performs one all-or-none `append-producer-batch` on its own
fresh FD. It never replaces the catalog with a generic full-suite script. Thus each later candidate reruns
old matrices with their ordered case IDs, ignored and serial flags, reference
benchmarks, and externally verified native requirements as well as the broad
locked regression.

Catalog argv is an array, never a shell string. Substitution occurs only when an
argv element equals one entire closed token: `{runtime:NAME}`, where NAME is
exactly `python|bash|git|gh|cargo|cargo-msrv|cargo-deny`,
`{bundle-entrypoint:NAME}`, where NAME must be an entrypoint in the
attempt-bound registered bundle, `{candidate}`, `{evidence_root}`,
`{attempt_root}`, `{candidate_root}`, `{fixture_root}`, or
`{artifact:ARTIFACT_ID}`. Intrinsic roots resolve to canonical absolute paths;
an artifact token resolves to the exact candidate-relative path declared for
that ID in the catalog's `outputs` map. `{fixture_root}` is valid only for a gate
declaring `fixture: "temp-dir"`; the runner creates that fresh mode-0700 directory
and removes it after records/artifacts are durably published. A token embedded
in a larger string, `$NAME`, `${NAME}`, command substitution, an unknown token,
bare/PATH executable, an undeclared artifact, or a literal shell metacharacter
is rejected before any child starts. A runtime token resolves only to the
attempt-bound runtime profile's absolute executable and its digest is rechecked
immediately before direct `execve`. A bundle-entrypoint token resolves no-follow
only through the registered bundle manifest, rehashes that exact root-owned
entrypoint and its transitive imports, and never falls back to a same-named
candidate script. Every formal exact-test catalog row begins with
`{bundle-entrypoint:run-exact-test}`; that trusted entrypoint obtains Cargo only
from the authenticated runtime pair. Unknown entrypoints fail before a child.
Declared cumulative outputs use create-new paths beneath
`{candidate_root}/run-cumulative/artifacts/`; a missing, duplicate, overwritten, or
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

- [ ] **Step 4: Run GREEN and self-test a nonzero child diagnostically**

Run:

```bash
python3 -B -m unittest scripts.tests.test_run_exact_test scripts.tests.test_implementation_evidence -v
TERSH_HARNESS_PARENT="${TMPDIR:-/tmp}"
TERSH_HARNESS_RAW="$(mktemp -d "${TERSH_HARNESS_PARENT%/}/tersh-evidence-harness.XXXXXX")"
TERSH_HARNESS_ROOT="$(cd "$TERSH_HARNESS_RAW" && pwd -P)"
trap 'rm -rf -- "$TERSH_HARNESS_ROOT"' EXIT
python3 -B scripts/implementation_evidence/run_gate.py --iteration impl-01 --attempt 001 --run-binding run-local --name expected-failure --candidate "$(git rev-parse HEAD)" --output-root "$TERSH_HARNESS_ROOT" --allow-failure -- sh -c 'exit 7'
python3 -B -c 'import json,pathlib,sys; p=list(pathlib.Path(sys.argv[1]).glob("impl-01/attempt-001/candidate-*/run-local/gates/expected-failure.json")); assert len(p)==1; d=json.loads(p[0].read_text()); assert d["exit_code"] == 7 and d["evidence_attempt"] == "001"; assert p[0].with_suffix(".stdout").is_file() and p[0].with_suffix(".stderr").is_file()' "$TERSH_HARNESS_ROOT"
python3 -B scripts/implementation_evidence/run_cumulative_gates.py --catalog scripts/implementation_evidence/gate_catalog.json --through impl-01 --attempt 001 --candidate "$(git rev-parse HEAD)" --output-root "$TERSH_HARNESS_ROOT" --self-test-only
git diff --check
```

Expected: all commands exit 0; the recorded child status is exactly 7.
`sh -c 'exit 7'` is an inert test child used only to prove exit-status capture;
the harness itself still passes argv arrays and never constructs shell commands.
These repo-local commands are development diagnostics only: their records cannot
become formal evidence and must never be accepted by a later finalizer.

- [ ] **Step 5: Commit the process-only prerequisite**

```bash
git add -- .gitignore scripts/evidence_core.py scripts/run_exact_test.py scripts/tests/test_run_exact_test.py scripts/implementation_evidence/run_gate.py scripts/implementation_evidence/host_envelope_adapter.py scripts/implementation_evidence/record_orchestration.py scripts/implementation_evidence/finalize_iteration.py scripts/implementation_evidence/commit_and_close.py scripts/implementation_evidence/verify_formal_lineage.py scripts/implementation_evidence/repair_projections.py scripts/implementation_evidence/run_external_candidate.py scripts/implementation_evidence/verify_ci_evidence.py scripts/implementation_evidence/verify_release_candidate.py scripts/implementation_evidence/gate_catalog.json scripts/implementation_evidence/run_cumulative_gates.py scripts/tests/test_implementation_evidence.py
git diff --exit-code
test -z "$(git ls-files --others --exclude-standard)"
python3 -B - <<'PY'
import subprocess

expected = {
    ".gitignore",
    "scripts/evidence_core.py",
    "scripts/run_exact_test.py",
    "scripts/tests/test_run_exact_test.py",
    "scripts/implementation_evidence/run_gate.py",
    "scripts/implementation_evidence/host_envelope_adapter.py",
    "scripts/implementation_evidence/record_orchestration.py",
    "scripts/implementation_evidence/finalize_iteration.py",
    "scripts/implementation_evidence/commit_and_close.py",
    "scripts/implementation_evidence/verify_formal_lineage.py",
    "scripts/implementation_evidence/repair_projections.py",
    "scripts/implementation_evidence/run_external_candidate.py",
    "scripts/implementation_evidence/verify_ci_evidence.py",
    "scripts/implementation_evidence/verify_release_candidate.py",
    "scripts/implementation_evidence/gate_catalog.json",
    "scripts/implementation_evidence/run_cumulative_gates.py",
    "scripts/tests/test_implementation_evidence.py",
}
actual = set(subprocess.check_output(
    ["git", "diff", "--cached", "--name-only"], text=True
).splitlines())
assert actual == expected, (actual - expected, expected - actual)
PY
git commit -m "test: add implementation iteration evidence harness"
```

Commit boundary: exact tests and cycle evidence can now fail closed before any feature iteration starts. This commit is process infrastructure, not one of the seven feature candidates.

- [ ] **Step 6: Independently install and register the formal harness bundle**

An authorized root/operator process outside this repository builds the exact
`bundle.json` tree from the committed source, reviews it, installs it beneath
`SUPERVISOR_HARNESS_ROOT`, registers its bundle and runtime profile in the
append-only host registry, and returns the bundle-registration receipt through
the privileged platform preflight. This repository neither implements nor
simulates that principal. Tasks2–8 may continue locally for diagnostics, but no
formal attempt may start until the real supervisor proves that receipt and a
nonroot adapter can authenticate the root-owned socket. A missing registry,
bundle, runtime, or distinct-principal FD is a hard formal-evidence blocker,
never a skipped/pass fixture.

## Exact External Candidate Procedure

Tasks2–8 call one shared CLI once per candidate attempt. The authorized operator
must have `gh` REST access and permission to create a non-protected
`codex/evidence/**` branch. Neither this helper nor its caller publishes a
public release. The exact per-attempt invocation (with the current locked policy
row asserted by the remaining flags) is:

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/run_external_candidate.py \
  --evidence-id "$TERSH_IMPL_ITERATION" \
  --attempt "$TERSH_IMPL_ATTEMPT" \
  --candidate "$TERSH_IMPL_CANDIDATE" \
  --repository QiushanHuang/Tersh \
  --remote origin \
  --push-ref "codex/evidence/$TERSH_IMPL_ITERATION/attempt-$TERSH_IMPL_ATTEMPT/$TERSH_IMPL_CANDIDATE" \
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
  --poll-seconds 5 \
  --host-store-fd FRESH_HOST_FD
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
7. Do not treat the manifest's producer field as proof. Before the push, require
   the exact candidate workflow blob SHA-256 and every evidence-producing helper
   blob SHA-256 to equal the independently approved digests in the registered
   bundle policy; a legitimate workflow/helper change requires a new bundle and
   evidence attempt. Then source-check those exact bytes so the expected
   producer job has exactly one pinned
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
As a top-level formal producer, `run-external` does not invoke `run-gate`. Its
single `append-producer-batch` atomically contains every policy-required
runner/before/bootstrap/selected-run/jobs/artifact-index/external-candidate body
plus one policy-named wrapper `gate` body. The wrapper binds the combined
external body hash, push ref, workflow/run identities, exact artifact policy,
and overall result without reselecting them. Thus implementation policies that
require `native-exdev-ci` receive that wrapper from the external batch itself;
no nested producer, second selector, or inherited Host FD is involved.

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

```text
export TERSH_IMPL_ITERATION=impl-01
: "${TERSH_IMPL_ATTEMPT:?supervisor must reserve the current three-digit attempt; do not hard-code 001 after Wave A or Wave B}"
case "$TERSH_IMPL_ATTEMPT" in 00[1-9]|0[1-9][0-9]|[1-9][0-9][0-9]) ;; *) exit 1 ;; esac
export TERSH_IMPL_CANDIDATE="$(git rev-parse HEAD)"
export TERSH_IMPL_EVIDENCE_ROOT="target/implementation-evidence/$TERSH_IMPL_ITERATION"
export TERSH_IMPL_ATTEMPT_ROOT="$TERSH_IMPL_EVIDENCE_ROOT/attempt-$TERSH_IMPL_ATTEMPT"
export TERSH_IMPL_CANDIDATE_ROOT="$TERSH_IMPL_ATTEMPT_ROOT/candidate-$TERSH_IMPL_CANDIDATE"
test -z "$(git status --porcelain=v1 --untracked-files=all)"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/run_cumulative_gates.py \
  --catalog SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/gate_catalog.json \
  --through "$TERSH_IMPL_ITERATION" \
  --attempt "$TERSH_IMPL_ATTEMPT" \
  --candidate "$TERSH_IMPL_CANDIDATE" \
  --output-root target/implementation-evidence \
  --host-store-fd FRESH_HOST_FD
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
root. The Host Envelope Supervisor launches this command and substitutes the
`FRESH_HOST_FD` metavariable with a newly created, already-open numeric
`AF_UNIX/SOCK_STREAM` descriptor. It is not an environment variable, and an
operator must not export, synthesize, or reuse it:

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/finalize_iteration.py \
  --iteration "$TERSH_IMPL_ITERATION" \
  --accepting-attempt "$TERSH_IMPL_ATTEMPT" \
  --candidate "$TERSH_IMPL_CANDIDATE" \
  --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" \
  --require-cumulative-through "$TERSH_IMPL_ITERATION" \
  --host-store-fd FRESH_HOST_FD \
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

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/finalize_iteration.py --iteration "$TERSH_IMPL_ITERATION" --accepting-attempt "$TERSH_IMPL_ATTEMPT" --candidate "$TERSH_IMPL_CANDIDATE" --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" --require-cumulative-through impl-01 --require-gate g0a-local --host-store-fd FRESH_HOST_FD --output docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-01.json
```

The committed `impl-01` catalog additionally requires `ci` and `release`.

Expected: PASS only with five final roles on the exact candidate and no unresolved P0/P1.

- [ ] **Step 4: Commit and host-close the evidence only**

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/commit_and_close.py --manifest docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-01.json --candidate "$TERSH_IMPL_CANDIDATE" --commit-message "test: record impl-01 g0a evidence" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/verify_formal_lineage.py --manifest docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-01.json --host-store-fd FRESH_HOST_FD
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

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/finalize_iteration.py --iteration "$TERSH_IMPL_ITERATION" --accepting-attempt "$TERSH_IMPL_ATTEMPT" --candidate "$TERSH_IMPL_CANDIDATE" --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" --require-cumulative-through impl-02 --host-store-fd FRESH_HOST_FD --output docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-02.json
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/commit_and_close.py --manifest docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-02.json --candidate "$TERSH_IMPL_CANDIDATE" --commit-message "test: record impl-02 g0b evidence" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/verify_formal_lineage.py --manifest docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-02.json --host-store-fd FRESH_HOST_FD
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
through `scripts/run_exact_test.py --test trusted_fs`. Run both crate-private
tests through the library target exactly as follows:

```bash
python3 -B scripts/run_exact_test.py --lib --name trusted_fs::tests::adjacent_receipt_facts_require_live_bound_lock_and_actual_synced_bytes
python3 -B scripts/run_exact_test.py --lib --name trusted_fs::tests::atomic_receipt_advance_rejects_wrong_revision_edge_or_facts
```

Together they prove the verified
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
    "kind": "command",
    "argv": ["{runtime:cargo}", "test", "--locked", "--test", "plan2_read_acceptance"],
    "exact_test": null,
    "serial": false,
    "ignored": false,
    "case_matrix": null,
    "expected_cases": [],
    "required_online_labels": [],
    "required_external_jobs": [],
    "fixture": null,
    "outputs": {}
  },
  {
    "gate_id": "g1a-reference",
    "kind": "benchmark",
    "fixture": "temp-dir",
    "outputs": {
      "g1a-read-candidate": "run-cumulative/artifacts/tersh-plan2-read-candidate.json"
    },
    "argv": ["{runtime:cargo}", "run", "--locked", "--release", "--bin", "tersh-plan2-read-bench", "--", "--require-reference-profile", "--output", "{artifact:g1a-read-candidate}", "--fixture-root", "{fixture_root}"],
    "exact_test": null,
    "serial": false,
    "ignored": false,
    "case_matrix": null,
    "expected_cases": [],
    "required_online_labels": [],
    "required_external_jobs": []
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

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/finalize_iteration.py --iteration "$TERSH_IMPL_ITERATION" --accepting-attempt "$TERSH_IMPL_ATTEMPT" --candidate "$TERSH_IMPL_CANDIDATE" --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" --require-cumulative-through impl-03 --require-gate g1a-reference --host-store-fd FRESH_HOST_FD --output docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-03.json
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/commit_and_close.py --manifest docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-03.json --candidate "$TERSH_IMPL_CANDIDATE" --commit-message "test: record impl-03 g1a evidence" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/verify_formal_lineage.py --manifest docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-03.json --host-store-fd FRESH_HOST_FD
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

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/finalize_iteration.py --iteration "$TERSH_IMPL_ITERATION" --accepting-attempt "$TERSH_IMPL_ATTEMPT" --candidate "$TERSH_IMPL_CANDIDATE" --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" --require-cumulative-through impl-04 --require-gate g1a-reference --require-gate g1b-mutation --host-store-fd FRESH_HOST_FD --output docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-04.json
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/commit_and_close.py --manifest docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-04.json --candidate "$TERSH_IMPL_CANDIDATE" --commit-message "test: record impl-04 g1b evidence" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/verify_formal_lineage.py --manifest docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-04.json --host-store-fd FRESH_HOST_FD
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
`trash_consumed_token_cannot_be_used_twice`, and these two distinct
owner-module gates:

```bash
python3 -B scripts/run_exact_test.py --lib --name trash::tests::trash_authorizing_facts_cannot_outlive_claim_or_locked_receipt_snapshot
python3 -B scripts/run_exact_test.py --lib --name recovery::tests::restore_authorizing_facts_cannot_outlive_claim_or_locked_receipt_snapshot
```
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

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/finalize_iteration.py --iteration "$TERSH_IMPL_ITERATION" --accepting-attempt "$TERSH_IMPL_ATTEMPT" --candidate "$TERSH_IMPL_CANDIDATE" --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" --require-cumulative-through impl-05 --require-gate g2-cli --host-store-fd FRESH_HOST_FD --output docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-05.json
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/commit_and_close.py --manifest docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-05.json --candidate "$TERSH_IMPL_CANDIDATE" --commit-message "test: record impl-05 g2 cli evidence" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/verify_formal_lineage.py --manifest docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-05.json --host-store-fd FRESH_HOST_FD
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

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/finalize_iteration.py --iteration "$TERSH_IMPL_ITERATION" --accepting-attempt "$TERSH_IMPL_ATTEMPT" --candidate "$TERSH_IMPL_CANDIDATE" --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" --require-cumulative-through impl-06 --require-gate g1c-g2 --require-gate native-exdev-ci --host-store-fd FRESH_HOST_FD --output docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-06.json
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/commit_and_close.py --manifest docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-06.json --candidate "$TERSH_IMPL_CANDIDATE" --commit-message "test: record impl-06 g1c g2 evidence" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/verify_formal_lineage.py --manifest docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-06.json --host-store-fd FRESH_HOST_FD
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

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/finalize_iteration.py --iteration "$TERSH_IMPL_ITERATION" --accepting-attempt "$TERSH_IMPL_ATTEMPT" --candidate "$TERSH_IMPL_CANDIDATE" --evidence-root "$TERSH_IMPL_EVIDENCE_ROOT" --require-cumulative-through impl-07 --require-gate g3-cluster --require-gate native-exdev-ci --host-store-fd FRESH_HOST_FD --output docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-07.json
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/commit_and_close.py --manifest docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-07.json --candidate "$TERSH_IMPL_CANDIDATE" --commit-message "test: record impl-07 g3 evidence" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/verify_formal_lineage.py --manifest docs/superpowers/evidence/2026-08-10-tersh-implementation/impl-07.json --host-store-fd FRESH_HOST_FD
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
- [ ] Every finalizer starts from the policy-fixed local/external root-class union and embeds canonical body/hash pairs for runner inventory, before-run snapshot, jobs, and all other canonical JSON across every attempt's one candidate-SHA namespace through the accepting attempt; bounded gate logs are validated but not embedded.
- [ ] External acceptance inventories required online custom runners before side effects, requires every candidate workflow and evidence-producer helper blob digest to match the independently registered bundle policy, pages repository-wide pre-push run IDs without workflow-path lookup, and binds only new creation-time push runs whose REST path is the exact bare workflow path and whose `head_sha`/`head_branch`/`event` match the pushed candidate.
- [ ] Every partial-registration, timeout, interrupt, download, verifier, or other failure issues at most one cancel request per bound nonterminal numeric ID and then observes every bound run terminal within the reserved budget; every subprocess/poll consumes the original remaining global deadline.
- [ ] Required artifacts match exact template/schema and nonempty contents; the root manifest excludes itself, the outer index hashes it, extras are rejected when requested, and producer acceptance requires unique pinned upload source plus artifact-ID/name, bare-upload-digest-to-REST-digest normalization, and timestamp-unwrapped job-log runtime join.
- [ ] Catalog argv uses only closed whole-token placeholders; no `$` variable, interpolation, or shell expansion survives into the committed catalog.
- [ ] Every source-changing correction creates a new candidate and reruns applicable external gates.
- [ ] Every iteration has Wave A, Wave B, Wave C, and five same-candidate closure reports with append-only bodies and hashes.
- [ ] Every role in every required wave is bound by the root-owned, root-peer-authenticated Host Envelope Supervisor to requested `gpt-5.6-sol` with `xhigh`; create-mode finalization authenticates every host receipt, rejects any other value or a missing supervisor/response, and distinguishes platform-envelope verification from explicit operator attestation when trustworthy platform model metadata is unavailable.
- [ ] Each closure commit contains exactly one of `impl-01.json` through `impl-07.json` and no code, test, workflow, or product-documentation change.
- [ ] `impl-01` and `impl-02` use the split Plan1 component recipes; Task9 external evidence is never run before Task10a.
- [ ] ADD-009 and ADD-010 map to exact Plan2/3/4 tests, including lock/snapshot lifetime and by-value single-use cases, and the final 269-ID requirement catalog.
- [ ] `impl-07` does not relabel G3 as a Workbench prerequisite or claim the seven later hardening cycles are complete.
