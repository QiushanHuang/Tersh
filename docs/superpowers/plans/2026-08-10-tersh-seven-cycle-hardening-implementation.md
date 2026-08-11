# Tersh Seven-Cycle Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Execute and evidence the seven post-feature performance, stability, and security cycles required by the Trusted Core design without adding product scope or claiming full-task completion before the final integrated audit passes.

**Architecture:** Treat hardening as seven ordered gates over the already-implemented Plans 1-5, not as a new runtime layer. A small standard-library evidence harness records commands, immutable committed-candidate SHAs, append-only five-role provenance, and canonical cycle manifests; production changes are allowed only when a cycle reproduces a concrete invariant failure. Each cycle runs a three-agent diagnosis wave, a single-writer repair wave, a two-agent adversarial/verification wave, and two final closure waves of three then two agents, so concurrency never exceeds three.

**Tech Stack:** Rust 1.88 and current stable, Cargo locked builds/tests, Python 3 standard library, Bash 3.2-compatible shell, GitHub Actions, Unix PTY/process groups, `tmux`, GNU `screen`, native APFS/Linux filesystems, JSON evidence manifests, and recurring `gpt-5.6-sol` reviewers at `xhigh` reasoning.

---

## Entry Gate And Non-Goals

Before this entry gate is evaluated, the implementation-iteration evidence plan must have committed its shared harness and `impl-01` through `impl-07` must each have a valid manifest. The clean entry `HEAD` must be exactly the verified `impl-07` evidence-only closure commit, not merely a descendant of it. Cycle 1 records that immutable entry commit plus all seven manifest hashes; every later hardening candidate must descend from it. Hardening reuses the already tested `scripts/evidence_core.py` and exact-test parser; it must not recreate or fork their canonicalization, bounded-drain, candidate/job/run selection, case-inventory, or append-only review validation logic.

Start the full seven-cycle sequence only after implementation Plans 1-5 are committed and their G0a-G3 slice gates pass. Record that starting revision in the first cycle manifest. This sequencing is not a dependency of the earlier Workbench Trusted Core milestone: its owner may evaluate G0a-G2 release acceptance as soon as Plans 1-4 and their integrated workbench gates pass. Cycle 6 revalidates and records that same workbench-only scope; it does not use G3 evidence. G3 remains the frozen companion. This plan adds no database, plugin system, generic runtime, telemetry service, transfer/synchronization feature, remote agent, recursive permanent delete, overwrite path, or unsupported EXDEV directory move.

No cycle is accepted because its ordinal exists or because one narrow test passes. A cycle is accepted only when its focused gate, the complete local prior-gate suite, its applicable native/CI gate, all five final reviews, and its canonical evidence manifest agree. A reproduced P0 or P1 remains blocking until fixed and re-reviewed. If a cycle reproduces no defect, the implementation role records `no_reproduced_defect` and makes no speculative production change; tests and evidence still form the cycle commit.

Tasks 1-6 must not describe the full optimization objective as complete. Cycle 6 hardens the release path and rechecks the G0a-G2 boundary, but does not redefine or delay the independently evaluated Workbench Trusted Core milestone. Task 7 may update full-objective completion wording only after its final requirement audit and all earlier cycle manifests validate in the declared Git ancestry and its own final gates bind one committed candidate SHA.

## Locked Evidence Files And Responsibilities

| File | Responsibility |
| --- | --- |
| `scripts/evidence_core.py` | Repository source and diagnostic copy of the shared primitives; formal execution uses only the reviewed exact byte from the registered root-owned bundle. |
| `scripts/implementation_evidence/host_envelope_adapter.py` | Already committed host-only context/invocation/terminal-response ingress: run the exact shared BEGIN/BODY/COMMIT/REPLY transaction over one root-owned, root-peer-authenticated `AF_UNIX/SOCK_STREAM` FD and rotate opaque single-use context capabilities while the closed bodies remain in the private store. |
| `scripts/implementation_evidence/record_orchestration.py` | Shared single-record writer that uses the same transaction to atomically consume the final rotated context capability and host-owned invocation/response handles, validate a receipt bound to the derived record facts, and then create-new publish implementation or hardening provenance. |
| `scripts/hardening/run_gate.py` | Repository source/diagnostic copy; the registered `run-gate` entrypoint executes one argv without a shell, host-spools the canonical body, appends its producer receipt, and emits only an untrusted projection. |
| `scripts/hardening/finalize_cycle.py` | Repository source/diagnostic copy; the registered `finalize-cycle` entrypoint enumerates the full host ledger/spool, proves the projection bijection and five-role closure, obtains a preimage receipt, and emits one manifest envelope without running Git. |
| `scripts/hardening/gate_catalog.json` | Closed, ordered local gate and frozen case-matrix policy registered in the root bundle; the candidate copy is never acceptance authority. |
| `scripts/hardening/run_cumulative_gates.py` | Repository source/diagnostic copy of the registered cumulative producer, which executes the pinned catalog prefix and receipts every result. |
| `scripts/hardening/run_prior_gates.sh` | Run the complete locked local regression, MSRV, policy, and build union used after every cycle. |
| `scripts/hardening/verify_implementation_entry.py` | Revalidate the seven fixed implementation manifests through `scripts/evidence_core.py`, prove candidate/evidence lineage to the clean hardening start commit, and emit the canonical entry record embedded by Cycle 1. |
| `scripts/implementation_evidence/run_external_candidate.py` | Already committed shared bootstrap helper: preflight exact candidate and online runners, push one create-new `codex/evidence/**` ref, select only fresh exact-head `push` runs, enforce deadlines, jobs, and artifact policy, and emit append-only external evidence. Hardening invokes it directly. |
| `scripts/hardening/run_exact_tests.py` | List one primary integration-test target, require an exact named inventory, run it plus declared regression targets, and emit discovered/executed counts that cannot pass at zero. |
| `scripts/hardening/audit_requirements.py` | Repository source/diagnostic copy of the root-bundle audit entrypoint frozen in Task 1; it validates the 269-ID policy and host-seals Cycle 7 requirements/audit records. |
| `scripts/tests/test_hardening_evidence.py` | Unit-test command recording, bounds, canonical output, committed-candidate binding, role validation, and fail-closed finalization. |
| `scripts/tests/test_requirement_audit.py` | Unit-test missing, duplicate, stale, waived, and non-passing requirement evidence. |
| `target/hardening/hardening-0N/attempt-NNN/candidate-SHA/` | Shared `local` root class: untrusted create-new projections for markers, local/cumulative gates, orchestration, reviews, and Cycle 7 requirements/completion-audit/failure results; it is reviewable but never the authority or an omission selector. |
| `target/hardening-diagnostic/hardening-0N/attempt-NNN/candidate-SHA/` and shared `target/evidence-agent-drafts/EVIDENCE_ID/attempt-NNN/DISPATCH_ID.json` | Repo-local dirty/baseline diagnostics and unreceipted agent/requirements drafts only; these disjoint trees are never host-bound, enumerated, projected into a formal tree, or accepted by a finalizer. |
| `target/hardening-external/hardening-0N/attempt-NNN/candidate-SHA/` | Shared `external` root class: separate create-new runner/bootstrap/run/job/artifact/external-result and `run-set-*` wrapper-gate projections. The attempt-bound policy fixes both classes; only the local class contains the Host marker pair, and finalization/repair enumerate their exact union. |
| `docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-0N.json` | The sole committed canonical envelope for one cycle; formal acceptance additionally requires its detached online closure receipt. |
| Cycle 7 requirements/completion audit | Canonical host-spooled producer records embedded by `cycle-07.json`; any `target/` copy is an untrusted diagnostic projection and no separate copy is committed. |

The `scripts/hardening/*` entrypoints are thin hardening-policy adapters over
`scripts/evidence_core.py`. Gate and cycle-manifest schemas remain
hardening-specific, but orchestration, review, provenance, attempt typing,
finding IDs, parent links, and resolution-reference validation use the exact
shared `tersh-evidence-orchestration-v1` and `tersh-evidence-review-v1`
contracts. Cycle 7 alone uses the explicitly versioned closed
`tersh-evidence-review-v2` extension defined below; there is no hardening
projection or legacy alias for either version.
Tests source-check that bounded draining, canonical JSON, exact candidate
binding, report provenance, and finding closure are imported from the shared
core and have no second implementation. Hardening has no external
selector/watch/verifier entrypoint: each native/release task invokes
the registered `run-external` producer directly as its own top-level operation,
never under `run_gate.py`. `run_exact_tests.py` likewise calls the list/result/case-matrix
parser exported by `scripts/run_exact_test.py`; it only adds multi-target
orchestration and may not copy libtest parsing.

Hardening and implementation evidence share one closed identity grammar.
`EVIDENCE-ID` must match `^(?:impl|hardening)-0[1-7]$`; CLI `--attempt` and
review-attempt inputs match
`^(?:00[1-9]|0[1-9][0-9]|[1-9][0-9]{2})$`, and every canonical JSON body
stores them only as the three-character strings `evidence_attempt` and
`review_attempt`. No hardening record exposes numeric or ambiguous `attempt` or
`review-attempt` fields. A push ref must match
`^codex/evidence/(?:impl|hardening)-0[1-7]/attempt-(?:00[1-9]|0[1-9][0-9]|[1-9][0-9]{2})/[0-9a-f]{40}$`,
and its final component must equal `--candidate`. The hardening adapter
additionally requires that `--evidence-id hardening-0N` agree with `--cycle
0N`. The hardening gate CLI accepts only `run-local|run-cumulative`; shared
orchestration/review bodies use the unmodified shared run-binding grammar
`^(?:run-local|run-cumulative|run-set-(?:ci-(?:[1-9][0-9]*|unregistered)(?:-release-(?:[1-9][0-9]*|unregistered))?|release-(?:[1-9][0-9]*|unregistered)))$`
so external Wave C and closure records can bind their exact combined run set;
empty, doubled/trailing, duplicate, reordered, zero-ID, or unknown components
are not lexical matches.

The registered bundle's formal `run-gate` entrypoint has this exact CLI; the
repo-local same-name script may use the ordinary `python3` spelling only with
`--allow-dirty-diagnostic`, and its output is never accepted:

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_gate.py \
  --evidence-id hardening-01 \
  --cycle 01 \
  --attempt 001 \
  --candidate 0123456789abcdef0123456789abcdef01234567 \
  --run-binding run-local \
  --name example-test \
  --output-root target/hardening \
  --host-store-fd FRESH_HOST_FD \
  [--allow-failure] \
  -- SUPERVISOR_PINNED_CARGO test --locked --test hardening_performance
```

Before the first command, the root supervisor's shared `open-attempt`
linearization creates the attempt binding plus host-spooled, receipted
`attempt-marker` and `candidate-marker` bodies with the exact shared schemas and
fields, and derives `target/hardening/EVIDENCE-ID/attempt-NNN/attempt.json` plus
`candidate-SHA/candidate.json`. A trusted command only validates those immutable
markers or create-new repairs a missing projection from the host spool; it never
authors, races, or weakens them. Later commands may append distinct create-new
gate, orchestration, or review files only after validating both markers; a
different candidate in the same hardening attempt, a
missing/invalid marker, or an existing output file is an error and is never
adopted, replaced, or truncated. Candidate-specific facts live only at and below
`candidate-SHA`; every committed retry increments `evidence_attempt` even though
the root schema itself is candidate-independent. A gate lives at
`candidate-SHA/run-BINDING/gates/NAME.{stdout,stderr,json}`. The JSON schema is
`tersh-hardening-gate-v1` and contains evidence ID, cycle, string
`evidence_attempt`, candidate, run binding, name, argv, cwd, observed 40-hex
HEAD, clean/dirty status, UTC start/end, duration milliseconds, exit code,
whether failure was allowed, stdout/stderr byte counts and SHA-256 hashes, OS,
architecture, filesystem, rustc/cargo versions, and a maximum 1 MiB retained
log per stream. It requires observed `HEAD == --candidate`. An accepting gate
additionally requires a clean worktree; `--allow-dirty-diagnostic` labels a
record non-accepting, and `finalize_cycle.py` may embed but never count it. It
continues draining after 1 MiB and hashes discarded bytes, so output bounds
cannot deadlock a child. Without `--allow-failure`, it exits exactly as the
child; with the flag, it exits 0 but records the real status.

Each normal gate projection has the closed three-file set `NAME.json`,
`NAME.stdout`, and `NAME.stderr`. Both bounded logs are attempted even when
empty; a fourth sibling, symlink, basename mismatch, or present stream body/hash
mismatch fails finalization. The host-receipted canonical JSON remains formal
authority: if a post-COMMIT/lost-reply projection failure leaves a bounded log
missing, finalization labels that log diagnostic-unavailable rather than
discarding the gate. The cycle manifest embeds the canonical JSON body/hash but
not the bounded raw log bytes.

`run_gate.py` never accepts another evidence producer as its child and never
passes its transaction FD to a descendant. The registered top-level
`run-external` entrypoint owns a distinct fresh FD and emits, in one atomic
producer batch, the canonical external records plus the policy-named wrapper
`gate` record that finalization requires. That wrapper binds the external body
hash, push ref, workflow kinds/run IDs, exact artifact expectations, and overall
result without reselecting or re-verifying them. `finalize_cycle.py` rejects an
external required gate unless its host-receipted
`tersh-external-candidate-v1` hashes identically, names the same evidence
ID/string attempt/candidate, records `event=push`, has overall `PASS`, and proves
the exact required artifact set with no extras. A successful external child
without the wrapper and all underlying receipt-backed bodies is not evidence.
The committed cycle manifest embeds the complete canonical body and SHA-256 of
every required gate record. For an external gate it additionally embeds, in attempt/run/kind
order, every create-new attempt manifest for that evidence ID through the
accepting attempt and the bounded runner-inventory, before-runs, selected-run,
jobs, and artifact-index JSON bodies plus hashes referenced by those manifests.
Potentially large downloaded artifact bytes remain outside Git, but their exact
indexes/hashes are durable, so `--verify-only` does not depend on ignored
`target/` metadata or silently erase an earlier failed external attempt. The
helper requires each exact candidate workflow and evidence-producer helper blob
digest to equal the independently registered bundle policy before any push; an
approved workflow/helper change requires a new bundle registration and evidence
attempt. It then snapshots repo-wide paginated run IDs and selects only an exact bare REST path plus matching
`head_sha`, `head_branch`, and `event`; it never fabricates a `PATH@REF` value or
trusts a workflow-path endpoint. Artifact producer joins use the pinned Action's
bare 64-hex output normalized to the REST `sha256:` form and unwrap only the
optional-initial-BOM/RFC3339Z-prefixed job-log grammar. All subprocesses consume
one remaining global deadline. Partial registration, interrupt, or failure
cancels only already-bound nonterminal numeric run IDs, drains them to terminal
evidence, and still emits the single failure result object.

The shared helper's exact artifact CLI repeats `--require-artifact KIND=PRODUCER_JOB:EXACT_TEMPLATE:SCHEMA` and `--reject-extra-artifacts KIND` while retaining `--artifacts KIND=none|all`. Templates admit only complete hyphen-delimited `{candidate}`, `{run_id}`, and `{run_attempt}` fields—no glob, regex, slash, partial field, or unknown brace. Every downloaded artifact has one root `artifact-manifest.json` that binds its declared schema, producer job, candidate, run ID, run attempt, and a nonempty canonical payload file/hash inventory. That inventory excludes `artifact-manifest.json` itself; the downloaded regular-file set must equal exactly the listed payload union plus that one root manifest, with no symlink or unlisted file. The helper-owned outer `artifact-index.json` separately records the root manifest's own size/SHA-256, GitHub artifact ID/name and normalized REST digest, the exact producer/template/schema expectation, and every validated payload entry. The frozen logical artifact set is:

| Workflow | Producer job | Exact template | Manifest schema |
| --- | --- | --- | --- |
| `ci` | `native-exdev-linux` | `native-exdev-linux-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-native-exdev-evidence-v1` |
| `ci` | `native-exdev-macos` | `native-exdev-macos-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-native-exdev-evidence-v1` |
| `ci` | `terminal-multiplexer-linux` | `terminal-multiplexer-linux-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-terminal-multiplexer-evidence-v1` |
| `ci` | `terminal-multiplexer-macos` | `terminal-multiplexer-macos-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-terminal-multiplexer-evidence-v1` |
| `release` | `tier1-macos-arm64` | `tier1-macos-arm64-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-tier1-release-evidence-v1` |
| `release` | `tier1-linux-x86_64` | `tier1-linux-x86_64-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-tier1-release-evidence-v1` |
| `release` | `tier2-macos-x86_64-source` | `tier2-macos-x86_64-source-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-tier2-source-evidence-v1` |
| `release` | `tier2-linux-arm64-source` | `tier2-linux-arm64-source-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-tier2-source-evidence-v1` |
| `release` | `install-msrv-1-88` | `install-msrv-1-88-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-install-evidence-v1` |
| `release` | `install-current-stable` | `install-current-stable-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-install-evidence-v1` |
| `release` | `assemble-manifest` | `release-manifest-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-release-manifest-evidence-v1` |
| `release` | `verify-release-candidate` | `verified-release-candidate-{candidate}-run-{run_id}-attempt-{run_attempt}` | `tersh-release-verification-evidence-v1` |

Cycles 3 and 4 require exactly the first two `ci` artifacts; Cycle 5 and every later external `ci` gate require exactly all four `ci` artifacts. Cycles 6 and 7 additionally require exactly all eight `release` artifacts. `--reject-extra-artifacts` makes an unexpected artifact a failure even when every required artifact is present.

Every formal orchestration and review projection is one append-only root-owned
file under the matching
`target/hardening/hardening-0N/attempt-NNN/candidate-SHA/orchestration/` or
`reviews/` directory. Both use the exact filename grammar
`ROLE.WAVE.REVIEW_ATTEMPT.json`, where ROLE is
`product|architecture|implementation|safety|verification`, WAVE is
`wave-a|wave-b|wave-c|closure-a|closure-b`, and REVIEW_ATTEMPT is the string
`001` through `999`. For example, the first safety Wave C pair is
`safety.wave-c.001.json` in both directories. The root-owned
`record-orchestration` producer constructs and publishes the orchestration body
from its context/invocation/response facts. The agent writes only the review's
policy-derived diagnostic draft
`target/evidence-agent-drafts/EVIDENCE_ID/attempt-NNN/DISPATCH_ID.json`; after
the terminal callback binds that draft's digest, `seal-agent-record` no-follow
rehashes it and alone publishes the byte-identical receipt-backed formal review
projection. Each formal file is create-new; there is
no aggregate `orchestration.json`, append-by-replacement prefix, hardening-only
identity schema, or numeric review attempt.

Thus the only two complete record paths are exactly
`target/hardening/EVIDENCE-ID/attempt-NNN/candidate-SHA/orchestration/ROLE.WAVE.REVIEW_ATTEMPT.json`
and
`target/hardening/EVIDENCE-ID/attempt-NNN/candidate-SHA/reviews/ROLE.WAVE.REVIEW_ATTEMPT.json`;
no `run-*` directory occurs between the candidate and either record class.

Wave A/B records use `run-local`. On a cycle with required external evidence,
Wave C and both closure waves use the exact shared `run-set-*` binding emitted by
that candidate's external helper; otherwise they use `run-cumulative`.
Finalization resolves a `run-set-*` only to the same evidence attempt/candidate's
embedded shared external manifest and rejects an absent, differently ordered, or
cross-attempt run set. A body may not merely name an unverified run binding.

The orchestration file uses `tersh-evidence-orchestration-v1` and contains the
host-issued agent ID, canonical task path, agent run ID, exact model
`gpt-5.6-sol`, exact reasoning effort `xhigh`, dispatch/start/end RFC 3339
timestamps, role, wave, string `review_attempt`, string `evidence_attempt`, run
binding, baseline commit, reviewed candidate, parent finding IDs, and the exact
shared tagged provenance union. The matching review normally uses
`tersh-evidence-review-v1`, repeats that
identity exactly, and adds verdict, checked design requirements, findings,
parent finding references, resolution references, direct gate hashes, and
commands. Each finding is the exact shared closed object
`{finding_id,severity,requirement,file,line,counterexample,required_correction}`;
parents are only shared-union IDs and resolutions are only the shared closed
objects below. It is limited to 256 KiB. Cycle 7's pre-audit Wave A and Wave B
reviews use v1 because no audit pair exists yet; they reject v2. Every
post-audit Cycle 7 Wave C and closure review instead uses
`tersh-evidence-review-v2`, whose exact top-level set is the complete v1 field
set plus only `audit_revision` and `evidence_package_sha256`. The former is a
three-character string under the attempt grammar; the latter is the exact
64-lowercase-hex digest returned by the trusted audit entrypoint: the SHA-256 of
the canonical ordered pair containing the host-spooled requirements-body and
completion-audit-body digests. Earlier cycles reject v2, and post-audit Cycle 7
Wave C/closure rejects v1; neither schema permits null or undeclared extension
fields. The digest never
derives from staged files and never replaces or alters the candidate commit.

Every dispatch uses the already committed shared recorder plus Host Envelope
Supervisor. Before spawn the fixed UID-0 supervisor fixes the context;
its store is outside the agent/operator sandbox, and only its peer-credential-
authenticated `AF_UNIX/SOCK_STREAM` FD can create/read the mode-0600 entries. It
uses exactly one pre-opened connected `FD > 2` for each transaction and the
shared `tersh-host-transaction-*-v1` BEGIN/BODY/BODY-END/COMMIT/REQUEST-END/
REPLY/REPLY-END protocol locked by the implementation-evidence plan. Every
frame field set, operation arm, body order, transaction nonce, digest, end
marker, final half-close, peer-credential check, atomic store transition,
private-orphan rule, and typed result is byte-for-byte the shared contract;
hardening defines no projection, alternate transport, second FD, expected-UID
override, or reusable-handle alias. Production requires kernel peer UID `0`,
`fstat(FD).st_uid == 0`, and nonroot client effective UID; matching arbitrary
nonroot peer/socket UIDs, a root client, unsupported peer authentication, or any
missing host boundary fails closed.

Hardening inherits the shared isolated Python launch boundary without change.
`SUPERVISOR_HARNESS_ROOT`, `SUPERVISOR_PINNED_PYTHON`,
`SUPERVISOR_PINNED_BASH`, `SUPERVISOR_PINNED_GIT`,
`SUPERVISOR_PINNED_CARGO`, `SUPERVISOR_PINNED_CARGO_MSRV`,
`SUPERVISOR_PINNED_CARGO_DENY`, and the 40-hex
`SUPERVISOR_POLICY_IMPL07_EVIDENCE_COMMIT` below are root-TCB protocol
metavariables rendered
into an argv by the supervisor, never environment variables, PATH results, shell
substitutions, or caller inputs. `SUPERVISOR_HARNESS_ROOT` names the separately
installed root-owned harness bundle. The bundle is non-agent-writable, not
operator-writable, and covered by one canonical digest-pinned manifest whose
revision and SHA-256 bind every evidence entrypoint, policy, transitive import,
gate catalog, schema, case matrix, allowlist, and acceptance threshold inherited
from the implementation-evidence plan. The independently registered runtime
profile resolves and binds every `SUPERVISOR_PINNED_*` executable and its closed
transitive toolchain; the attempt binding jointly pins bundle and runtime profile
rather than placing host paths in the portable bundle manifest. Candidate code, a
candidate tree, or candidate scripts must never be executed as an evidence
producer and never receive the Host Envelope FD; they are untrusted subjects
whose exact Git blobs, argv, cwd, and toolchain the trusted harness records.
Every `text` fence containing a `SUPERVISOR_*` or `FRESH_HOST_FD` token is
ordered execution-plan notation, not one shell program or one argv. Each logical
command whose first element is `SUPERVISOR_*` is a separate closed argv template
(backslash-wrapped physical lines are one template); preceding Git/setup/check
lines are separate unprivileged preparation steps and must finish before the
supervisor launch. Within a formal argv, quoted `$TERSH_*` spellings are symbolic
typed slots populated from the supervisor's already validated candidate/attempt
state—not shell expansion—and `FRESH_HOST_FD` is a newly created numeric FD.
The closed resolver obtains every `*_CANDIDATE`, `*_ATTEMPT`, and
hardening-start commit from immutable host/Git state; evidence, completion,
draft, projection, and output roots from the registered policy; fixture roots
only from a trusted producer's mode-0700 create-new directory with retained
identity; `AUDIT_REVISION` from the host's create-new `reserve-audit-draft`
result; and `REQUIREMENTS_DRAFT` from that reservation's policy-derived path.
None is
a caller path or environment value. The two native-EXDEV variables are not
renderer slots at all: the trusted cumulative producer derives them solely from
the authenticated `distinct-writable-filesystems-v1` session capability and
injects them only into that one child.
The renderer rejects an unknown slot, state mismatch, command substitution, or
shell execution, then performs one direct `execve` per formal command. The
policy commit token comes from the registered policy row, must be exactly 40
lowercase hexadecimal characters, and is rendered as one argv element just like
the absolute runtime and bundle tokens.

Before each formal launch the supervisor verifies every absolute parent,
manifest, entrypoint, policy, and import no-follow and non-group/world-writable,
then uses direct `execve` with the literal flags `-I -S -B`, a minimal fixed
environment containing no `PYTHON*`, `LD_*`, `DYLD_*`, startup/user-site/search-
path inputs, and no inherited descriptors other than fixed stdio plus the one
transaction FD. Each host-bound `__main__` also rejects a missing isolated/no-
site/no-bytecode runtime before opening the FD. A direct or ordinary `python3`
launch is diagnostic-only and can never form hardening evidence; every capture,
record, producer, cumulative/external/audit, and create-mode finalization below
uses the pinned bundle. Every accepted hardening record has the shared immutable
host producer receipt, and every final manifest requires the shared manifest-
preimage receipt plus post-commit closure receipt; an agent-writable projection
or offline structural verification alone is never formal evidence.

Hardening imports the implementation plan's exact `bundle.json`, runtime
profile, bundle-registration receipt, attempt-binding predecessor chain,
producer-receipt hash chain, response-v2 report digest, host-only spool,
RECORD/SPOOL chunk streams, `append-producer-batch`,
`seal-manifest-preimage`, `commit-and-close`, `query-formal-lineage`, and
`audit-and-append-pair` transactions without renaming or projecting a field. The
first formal host COMMIT for each
evidence ID starts at attempt `001`; dirty or baseline diagnostics use only
`target/hardening-diagnostic` and allocate no host attempt binding. The first host COMMIT for
`hardening-0N/attempt-NNN` pins candidate/tree/worktree/bundle/runtime/policy;
the host ledger—not `target/hardening`—enumerates every receipt through the
accepting attempt. The finalizer requires a three-way bijection among that full
ledger, the host spool, and raw projections, then obtains the preimage receipt
inside that same `seal-manifest-preimage` transaction and fresh FD.
Only `commit-and-close` may create the evidence-only Git commit and detached
closure. Formal verification is an online root-registry query; offline
`--verify-only` emits only `STRUCTURAL_PASS/HOST_UNVERIFIED`.

The platform-envelope capture sequence is:

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/host_envelope_adapter.py capture-context --host-store-fd FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/host_envelope_adapter.py capture-invocation --context-handle CONTEXT_HANDLE_H0 --host-store-fd FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/host_envelope_adapter.py capture-response --context-handle CONTEXT_HANDLE_H1 --host-store-fd FD
```

The three closed stdout results yield `H0`, then `(H1, HI)`, then `(H2, HR)`
under schemas `tersh-host-capture-context-result-v1`,
`tersh-host-capture-invocation-result-v1`, and
`tersh-host-capture-response-result-v1`. Each capture invalidates its predecessor
context capability at the host's atomic COMMIT point, before REPLY; a failed
reply therefore exposes no successor but cannot revive the predecessor. When
model metadata exists, the executor then runs this exact command with only the
final context generation:

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/record_orchestration.py append-platform --context-handle CONTEXT_HANDLE_H2 --invocation-handle INVOCATION_HANDLE_HI --response-handle RESPONSE_HANDLE_HR --host-store-fd FD
```

It records provenance mode `platform-envelope`. When the host response still
contains trustworthy agent/task/run identity and lifecycle but model/effort
metadata is unavailable, it skips invocation capture, consumes `H0` during
response capture to obtain `(H1, HR)`, and runs this exact command:

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/record_orchestration.py attest --context-handle CONTEXT_HANDLE_H1 --response-handle RESPONSE_HANDLE_HR --operator-id ID --host-store-fd FD
```

The record operation first retrieves and validates the fixed BODY sequence,
derives the clean candidate, destination, and preparatory-record hash, and
streams the proposed canonical body without mutating the host store. Every
pre-linearization failure consumes no host capability and creates no spool body
or receipt. Only REQUEST-END plus EOF permits the host, at COMMIT, to atomically
create-new publish and fsync the spool body, consume every final
context/member handle and append the corresponding producer receipt; partial
consumption is impossible. Only a validated REPLY, REPLY-END, and EOF permits a
matching agent-writable projection. A post-COMMIT reply or projection failure
leaves the old handles invalid and stdout empty, but the complete receipt and
spool body remain discoverable through host enumeration; it is not silently
garbage-collected or omitted from finalization. Capture operations that create
only successor capabilities may still leave the shared contract's private,
agent-unaddressable orphan on a post-COMMIT reply failure. The fallback records
provenance mode
`operator-attestation` with reason `platform-model-metadata-unavailable`. The
latter attests only the fixed
`gpt-5.6-sol`/`xhigh` request; identity and lifecycle come only from the
  host-owned response. If that response does not exist, the dispatch cannot
produce evidence. The provenance object is byte-for-byte the shared closed
union: both arms contain only `mode`, `context: {body, sha256}`, and
`response: {body, sha256}`; the platform arm adds only
`invocation: {body, sha256}`, while the attestation arm adds only `operator_id`,
`attested_at`, fixed reason and the exact requested model/effort. Both modes
reject missing/unhashed bodies, mixed arms, extra keys, agent/reviewer JSON, and
CLI/environment identity, timestamp, model, effort, destination, or free-form
reason overrides. The detached producer receipt binds the final record by
destination, byte count, body digest, and (for an agent report) the shared
`dispatch_id`; no host receipt ID is embedded in the provenance or record body.
The available embedded bodies use the exact shared context,
invocation, and response-v2 field sets/types/grammars (attestation has no
  invocation body). Create-mode finalization uses its one injected FD for one
  shared `seal-manifest-preimage` transaction. Its BEGIN selects only the
  evidence ID, terminal three-character attempt, and manifest kind; it contains
  no receipt IDs, candidate, count, page, digest, or omission filter. The host
  returns the complete bounded snapshot plus every receipt-joined spool stream.
  The finalizer validates the predecessor and receipt chains and every aggregate,
  repairs only a missing projection create-new from host bytes, rejects an
  existing mismatch or extra, constructs and uploads the exact payload RECORD,
  and sends the snapshot/payload COMMIT on that same still-open FD. Only the
  sealed preimage REPLY/REPLY-END/EOF permits envelope publication; the operation
  never raises the frame bound or truncates history. Committed
`--verify-only` rejects a host FD, rehashes only the embedded envelope and
preimage receipt, and reports `STRUCTURAL_PASS/HOST_UNVERIFIED`; it cannot
authenticate host origin or claim formal PASS.

Every create-mode `finalize_cycle.py` block below is launched by that supervisor,
which substitutes each `FRESH_HOST_FD` token with a distinct already-open numeric
descriptor. It is never an environment variable, and an operator must not
export, synthesize, or reuse it. Peer-credential validation is the authority.
Committed-manifest `--verify-only`
commands intentionally have no FD because they are structural diagnostics only;
formal acceptance always performs a fresh root-peer-authenticated query for the
bundle registration, preimage receipt, and detached closure receipt.

Finding and parent IDs both match the exact shared union
`^(?:impl|hardening)-0[1-7]-F(?:00[1-9]|0[1-9][0-9]|[1-9][0-9]{2})$`.
For a hardening record, the captured evidence ID must equal the containing tree,
each finding and parent must name that same evidence ID, and IDs are unique and
monotonically allocated; a parent also precedes its child numerically. A resolution is
`{finding_id, correcting_commit, verifying_review_ref}` where
`verifying_review_ref` is exactly `{evidence_attempt, candidate, run_binding,
review_file, review_body_sha256}`. The attempt is a three-character string, the
candidate is 40 lowercase hex, `run_binding` uses the full shared grammar above,
`review_file` uses the shared filename grammar, and
the body hash is 64 lowercase hex. The finalizer resolves
`attempt-EVIDENCE_ATTEMPT/candidate-CANDIDATE/reviews/REVIEW_FILE` no-follow,
requires its body hash and `run_binding` to match, and requires that earlier or
same-history review to name the finding and verify the correction. It rejects a
self-referential review body hash, free-form reference, or cross-tree ID.

A Wave B implementation report cites every parent finding and correcting commit
it addresses; a later independent verifying review supplies the non-circular
resolution reference. `finalize_cycle.py` enumerates every per-file orchestration
and review record plus raw gate records across every attempt. It rejects
missing/duplicate IDs, task/run/provenance mismatch, reused paths, another
model/reasoning setting, invalid timestamps, stale baseline/candidate, a direct
gate hash that does not match the named raw record, any final `FAIL`, any P0/P1
without a later resolution and closure confirmation, a nonzero required gate,
or a missing wave. The committed cycle manifest embeds the complete canonical
JSON body and SHA-256 of every required gate, shared external manifest,
orchestration, planning, execution, adversarial, verification, and closure
record in deterministic evidence-attempt/candidate/run-binding/review-attempt
order. It enumerates every host-bound evidence attempt from `001` through
`--accepting-attempt`, including failed, interrupted, superseded-candidate, and
host-receipted unregistered-external attempts; a missing intermediate attempt or
an unembedded create-new formal record fails. The disjoint diagnostic root is
neither scanned nor copied into a formal projection. Raw logs and downloaded
artifacts may remain ignored, but finalization preserves their byte
counts/indexes/hashes and every canonical discussion or failed review attempt
needed for audit.

Each accepted cycle has exactly two acceptance commits: its final candidate and
its evidence-only closure. Superseded candidate commits may remain in Git after
a review-driven retry, but no superseded SHA or its gates/reviews count toward
acceptance. First stage only the candidate paths, assert the exact staged-name
set, and commit code/tests/workflows/documentation without the current cycle
manifest. Then prove a clean committed candidate:

```bash
git diff --exit-code
test -z "$(git ls-files --others --exclude-standard)"
test -n "$(git diff --cached --name-only)"
git commit -m "test: freeze hardening cycle candidate"
test -z "$(git status --porcelain=v1 --untracked-files=all)"
TERSH_HARDENING_CANDIDATE="$(git rev-parse HEAD)"
case "$TERSH_HARDENING_CANDIDATE" in ''|*[!0-9a-f]*) exit 1 ;; esac
test "$(printf '%s' "$TERSH_HARDENING_CANDIDATE" | wc -c | tr -d ' ')" = 40
git cat-file -e "$TERSH_HARDENING_CANDIDATE^{commit}"
```

All required focused, prior, native, CI, and release gates are rerun after that
commit under a fresh three-digit evidence attempt and record the same candidate.
Every retry, including an environmental retry of the same candidate, increments
the evidence attempt; a product source/test/workflow/docs change also increments
it and supplies the new candidate. Wave C and both closure waves bind that exact
attempt and commit. Any subsequent product source, test, workflow, or user-facing
documentation change invalidates every accepting gate and review for the prior
candidate. The registered acceptance policy cannot be changed by a candidate;
an independently approved bundle update likewise forces a new attempt.
Finalization creates only the current cycle envelope and embeds all earlier
attempts. `commit-and-close` then requires the index/worktree to contain exactly
that one destination before it creates the evidence commit and host closure;
the evidence-only commit cannot retroactively change the candidate it attests.

## Locked Five-Role Wave Order

Every task below repeats this order with cycle-specific prompts:

1. **Wave A, maximum three concurrent:** product outcome/scope, architecture/state model, and implementation/focused-diagnosis roles run read-only against the same baseline; all three append reports.
2. **Wave B, one writer:** only the implementation role adds the named tests and applies the smallest fix for reproduced failures; every execution attempt appends a report with parent finding IDs and its correcting commit. A later independent review adds the body-hash-bound resolution reference. It records `no_reproduced_defect` when the new matrix already passes.
3. **Wave C, maximum two concurrent:** adversarial safety/failure and independent verification/regression roles inspect the clean committed candidate and append independent reports.
4. **Closure A, maximum three concurrent:** product, architecture, and implementation/focused-diagnosis re-review the final candidate and append closure reports.
5. **Closure B, maximum two concurrent:** safety and verification re-review that identical candidate and append closure reports. Any new finding returns to Wave B, increments attempt numbers, and repeats Wave C plus both closure waves.

Finalization proves at least one valid Wave A report for its three roles, one Wave B execution report, one Wave C report for safety and verification, and five same-hash closure reports. Earlier attempts remain in the manifest as canonical body-plus-hash history; only the latest complete closure set determines acceptance.

No reviewer may silently convert a missing environment, skipped native test, unavailable multiplexer, absent CI artifact, or 0-test filter into PASS.

`run_exact_tests.py` has this exact CLI shape: `python3 -B scripts/hardening/run_exact_tests.py --target TARGET --require-test NAME [--require-test NAME ...] [--regression-target TARGET ...] [--matrix MATRIX --expect-case MATRIX:CASE ...] [--expect-case-range MATRIX:PREFIX:START:END:WIDTH] [--cargo-bin ABSOLUTE_CARGO]`. Repo-local diagnostic mode may omit `--cargo-bin` and reports itself nonformal. Every registered formal invocation must receive `--cargo-bin` from the authenticated parent producer's attempt-bound runtime-profile pair; it must equal that pair's absolute Cargo path, be rehashed immediately, and cannot come from PATH, cwd, candidate, or environment. The parent catalog row expresses the value only as whole token `{runtime:cargo}`. It first runs that exact Cargo executable with `test --locked --test TARGET -- --list` for the primary target and the corresponding `--list` command for every regression target, parses exact test names, requires every named primary test to be discovered exactly once, and requires every listed target to discover at least one test. It then runs the full primary and every regression target unfiltered with `--nocapture --test-threads=1`, parses Cargo's executed-test summaries, requires a positive executed count for every target, and requires every named primary test to have executed. A parameterized test emits one `tersh-case-count-v1` line containing its matrix name, immutable expected case-ID list, executed ID list, and counts. The wrapper expands any numeric range, passes the externally supplied ordered IDs to the shared parser, and rejects a missing declared primary matrix, missing/duplicate/extra/reordered emitted ID, unequal count, or a test that changes its own expected and executed lists together. A regression matrix named by `--matrix` is equally external and mandatory; other regression matrices retain shared-parser structural validation without becoming a new hardening acceptance claim. It prints one `tersh-exact-test-v1` JSON object containing required/discovered/executed names, per-target discovered/executed/ignored counts, and validated external case matrices. `run_gate.py` recognizes that object and copies it into the gate record. Every gate whose name starts with `focused-` must contain this inventory; `finalize_cycle.py` rejects a missing inventory, zero discovered or executed tests, an invalid declared matrix, or a required name that did not execute. This implements the normative focused-gate contract in the design addendum at lines 1470-1474.

### Locked Hardening Case Matrices

The following are the only parameter-matrix grammars accepted by the hardening wrapper. The implementation stores these literal ordered lists in test code only as the emitted execution record; the CLI below remains the independent expectation. `cancel-success-schedules` is the exact numeric expansion `schedule-000` through `schedule-255`, inclusive, with width three and count 256.

| Matrix | Exact ordered case IDs |
| --- | --- |
| `hardening-add-009-serde` | `empty-name`, `dot-name`, `dotdot-name`, `slash-name`, `nul-name`, `padded-base64`, `aliased-base64`, `malformed-base64`, `invalid-path-component` |
| `hardening-add-010-proof-replay` | `other-bundle`, `other-revision`, `other-edge`, `same-token-second-use` |
| `hardening-add-010-lock-liveness` | `lock-dropped-before-transition`, `snapshot-replaced-while-unlocked` |
| `hardening-fs-operation-faults` | `copy-enospc`, `copy-eacces`, `samefs-move-eacces`, `rename-parent-swap`, `trash-eacces`, `empty-delete-eacces`, `exdev-regular-enospc`, `exdev-regular-eacces`, `exdev-checksum`, `exdev-metadata`, `exdev-file-sync`, `exdev-directory-sync`, `exdev-source-swap`, `exdev-target-race`, `exdev-cancel-prepublish`, `exdev-cancel-postpublish`, `exdev-symlink`, `exdev-directory-reject`, `exdev-special-reject`, `source-claim-two-process` |
| `hardening-recovery-crash-seams` | `source-claim-before-write`, `source-claim-after-write`, `source-claim-after-file-sync`, `source-claim-after-rename`, `source-claim-after-directory-sync`, `trash-before-mirror-intent`, `trash-after-mirror-intent`, `trash-after-adjacent-replace`, `trash-after-fixed-confirm`, `restore-before-claim`, `restore-after-claim`, `restore-after-publish`, `restore-before-payload-cleanup`, `restore-after-payload-cleanup`, `exdev-before-copy`, `exdev-after-copy`, `exdev-after-file-sync`, `exdev-after-publish`, `exdev-after-directory-sync`, `exdev-before-source-claim`, `exdev-after-source-claim`, `exdev-before-source-delete`, `exdev-after-source-delete`, `exdev-after-terminal-sync` |
| `hardening-recovery-invalid-state` | `corrupt`, `truncated`, `duplicate-id`, `unknown-schema`, `mirror-mismatch`, `orphan-payload`, `orphan-staging`, `contradictory-location` |
| `hardening-recovery-add-009-serde` | `empty-name`, `dot-name`, `dotdot-name`, `slash-name`, `nul-name`, `padded-base64`, `aliased-base64`, `malformed-base64`, `invalid-path-component` |
| `hardening-recovery-add-010-proof-replay` | `other-bundle`, `other-revision`, `other-edge`, `same-token-second-use` |
| `hardening-recovery-add-010-lock-liveness` | `lock-dropped-before-transition`, `snapshot-replaced-while-unlocked` |
| `hardening-terminal-outcomes` | `q`, `shift-q`, `ctrl-c`, `sigterm`, `sighup`, `write-failure`, `restore-failure`, `panic` |
| `hardening-terminal-layouts` | `40x10`, `60x16`, `80x24` |
| `hardening-proxy-lifecycle` | `ready`, `pre-ready-timeout`, `malformed-ready`, `user-interrupt`, `reader-eof`, `reader-panic`, `descendant-pipe` |
| `release-targets-v1` | `tier1-macos-arm64`, `tier1-linux-x86_64`, `tier2-macos-x86_64-source`, `tier2-linux-arm64-source` |
| `hardening-rollback-policy` | `plan-default`, `manifest-mismatch`, `execute-missing-env`, `execute-wrong-tag`, `argv-no-shell`, `retains-forensic-assets` |
| `g3-sweeps` | `hosts-1`, `hosts-16`, `hosts-17`, `hosts-40` |
| `g3-process-count` | `live-1`, `live-16`, `refill-17`, `refill-40` |
| `g3-shutdown` | `queued-quit`, `active-quit-term`, `active-quit-kill`, `timeout-term`, `timeout-kill`, `grandchild-pipe`, `reader-eof`, `reader-panic` |
| `g3-refresh` | `one-followup`, `latest-followup-wins`, `late-token`, `late-generation` |
| `g3-launch` | `ready-valid`, `ready-malformed`, `ready-oversize`, `ready-timeout`, `source-pair-unknown`, `exit-0`, `exit-2`, `exit-127`, `exit-129`, `exit-130`, `exit-143`, `exit-255`, `local-signal` |
| `add-009-exdev-serde` | `empty-name`, `dot-name`, `dotdot-name`, `slash-name`, `nul-name`, `padded-base64`, `aliased-base64`, `malformed-base64`, `invalid-path-component` |
| `exdev-transition-replay` | `other-bundle`, `other-revision`, `other-edge` |

The hardening proof matrices use genuine verifier-issued tokens. The `same-token-second-use` case is a compile-fail/API ownership fixture proving a by-value token cannot be supplied twice; it is not an `Option::take` or fabricated-token simulation. The two lock-liveness cases hold a deterministic competitor behind a barrier: one proves an authorizing fact cannot outlive its actual no-follow lock and verified snapshot, and the other replaces the object after an intentionally dropped non-authorizing observation and proves no fixed or adjacent revision advances. Tasks 3 and 4 run both forms for core/EXDEV and trash/restore respectively as crate-unit tests, because their proof constructors and lock-bearing typestates are intentionally unavailable to integration targets.

The inherited ADD-010 catalog has two entries: the three-case `exdev-transition-replay` matrix invokes `exdev::tests::exdev_transition_rejects_genuine_token_from_other_bundle_revision_or_edge`, and the non-matrix exact gate invokes `exdev::tests::exdev_consumed_transition_token_cannot_be_used_twice` with `--lib`. They are never collapsed into the hardening-specific four-case matrix.

Cycle 3 adds two more distinct crate-unit catalog entries: `hardening-add-010-replay` invokes `exdev::tests::hardening_add_010_proof_replay_matrix_has_exact_cases` with the four frozen `hardening-add-010-proof-replay` cases, and `hardening-add-010-lock-liveness` invokes `exdev::tests::hardening_add_010_lock_liveness_matrix_has_exact_cases` with the two frozen liveness cases. Cycle 4 likewise adds `hardening-recovery-add-010-replay` and `hardening-recovery-add-010-lock-liveness`, invoking the corresponding `recovery::tests::hardening_recovery_add_010_proof_replay_matrix_has_exact_cases` and `recovery::tests::hardening_recovery_add_010_lock_liveness_matrix_has_exact_cases` crate-unit tests. Each is a separate `--lib` gate and separate catalog/finalizer requirement; neither `hardening_fs_faults` nor `hardening_recovery` may claim or emit those private matrices.

The inherited `g3-launch` matrix is likewise a standalone crate-unit gate named `inherited-g3-launch`, invoking `cluster_launch::tests::g3_launch_frame_and_child_outcome_matrix` with `--lib`. Its accepted READY/local-signal rows need the crate-unit-only compatibility fixture, so `run_exact_tests.py` must not try to discover it through an integration `--regression-target cluster`. The inherited `g3-shutdown` matrix is a second standalone crate-unit gate named `inherited-g3-shutdown`, invoking `cluster_probe::tests::g3_timeout_and_quit_term_wait_kill_reap_and_join` with `--lib`; its reader-panic and signal-fault seams remain private. `gate_catalog.json`, the hardening-05 cumulative replay, and the cycle-05 finalizer require both separate inherited records in addition to `focused-terminal`.

The seven-case `hardening-proxy-lifecycle` matrix has the same reachability rule: it is a standalone crate-unit gate named `focused-proxy-lifecycle`, invoking `cluster_launch::tests::hardening_proxy_lifecycle_matrix_has_exact_cases` with `--lib`. Its accepted READY row uses the crate-unit-only compatibility fixture, so `hardening_terminal` must not declare or emit this matrix. The Cycle 5 catalog prefix and finalizer require `focused-terminal`, `focused-proxy-lifecycle`, `inherited-g3-shutdown`, and `inherited-g3-launch` as four distinct records. Only `g3-sweeps`, `g3-process-count`, and `g3-refresh`, whose rows are observable exclusively through public behavior, remain in the integration wrapper.

### Cumulative Gate Catalog Contract

`gate_catalog.json` is the sole local acceptance inventory. Each entry has a stable gate ID, introducing evidence ID, argv array, exact-test target/names, independently frozen matrix names/case IDs, environment capability, and whether it is required locally or only on a named native runner. Formal catalog argv permits two whole-element token forms only: `{runtime:NAME}`, where NAME is exactly `python|bash|git|gh|cargo|cargo-msrv|cargo-deny`, resolves to the attempt-bound runtime profile's absolute executable; and `{bundle-entrypoint:NAME}` resolves to an exact entrypoint in the attempt-bound bundle manifest. The trusted runner verifies the executable/bundle digest immediately before direct `execve` and supplies only the runtime profile's fixed toolchain environment; either token embedded inside another string, an unknown name, a bare/PATH executable, or any candidate-tree fallback fails before launch. The `prior-gates` entry is exactly `["{runtime:bash}","{bundle-entrypoint:prior-gates}","{runtime:cargo}","{runtime:cargo-msrv}","{runtime:python}","{runtime:cargo-deny}","{runtime:git}","{bundle-entrypoint:candidate-unittest}"]`. The prefix for `hardening-0N` is the ordered union of entries introduced by `hardening-01` through `hardening-0N`; entries cannot be deleted, renamed, reordered, weakened, or silently skipped. The broad prior-gate script below remains useful regression coverage but never substitutes for a catalog entry or its frozen matrix.

Every formal catalog row whose bundle entrypoint is `run-exact-tests` contains
the adjacent exact pair `--cargo-bin`, `{runtime:cargo}`. A missing, duplicate,
reordered, embedded, candidate, environment, or different-runtime Cargo value
fails before any test child starts.

The Cycle 3 `native-exdev` entry has argv exactly
`["{bundle-entrypoint:native-exdev}"]` and environment capability
`distinct-writable-filesystems-v1`, whose only variable names are
`TERSH_H3_EXDEV_ROOT_A` and `TERSH_H3_EXDEV_ROOT_B`. The trusted cumulative
runner accepts that capability only from the authenticated producer-session
BODY imported from the shared contract. Before launch, the UID-0 supervisor
allocates two mode-0700 directories for the fixed nonroot producer on registered
writable filesystems, no-follow opens and retains both directory capabilities,
requires distinct `st_dev`, and binds the exact
`tersh-host-environment-capability-v1` body to the attempt, policy row, producer
session, and every resulting gate receipt. The runner reopens each path
no-follow, matches the retained device/inode/owner/mode identity, and injects
only those two path values into the child alongside the fixed runtime
environment. No shell export, inherited caller environment, argv root, or
candidate-selected path participates. The runner resolves it only to the exact
root-owned `native-exdev` entrypoint in the attempt's registered
bundle manifest and verifies that entrypoint's digest immediately before
`execve`. It never resolves to `scripts/test-exdev-native.sh` in the candidate.
The registered entrypoint accepts zero positional arguments, reads only those
two host-injected controlled variables, and fails unless each still matches its
live no-follow capability and remains writable with different `st_dev` values.
Cycle 3 therefore requires an independently reviewed bundle upgrade and
a new evidence attempt before this entry may run. `run_cumulative_gates.py`
validates the closed capability before launching the gate and passes the two
values without interpolating them into catalog argv. Each clean retry creates
its own `native-exdev` receipt; a dirty diagnostic record can never satisfy the
cumulative manifest or finalizer.

Cycle 5 adds two policy rows whose argv are exactly
`["{bundle-entrypoint:terminal-multiplexer}","tmux"]` and
`["{bundle-entrypoint:terminal-multiplexer}","screen"]`. The entrypoint is a
reviewed root-bundle copy with a closed one-argument mode and never resolves to
the candidate's `scripts/test-terminal-multiplexer.sh`. Before either row or the
Cycle 5 cumulative prefix may run, an independently approved successor bundle
must register this entrypoint plus the updated CI/helper policy and start a new
evidence attempt.

Every clean committed candidate runs:

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_cumulative_gates.py \
  --catalog SUPERVISOR_HARNESS_ROOT/scripts/hardening/gate_catalog.json \
  --through hardening-0N \
  --attempt "$TERSH_HARDENING_ATTEMPT" \
  --candidate "$TERSH_HARDENING_CANDIDATE" \
  --output-root target/hardening \
  --host-store-fd FRESH_HOST_FD
```

The cumulative runner never spawns the host-bound `run_gate.py` entrypoint. In
one trusted process it calls the shared non-CLI `collect_gate` primitive for each
catalog row with `run-binding=run-cumulative`, closes the sole Host Envelope FD
in every untrusted child, and buffers the closed gate bodies plus
`tersh-hardening-cumulative-v1` catalog/result body. It then uses its own one
fresh FD for one all-or-none `append-producer-batch`; no nested producer or child
inherits or reuses that FD. Hand-written cycle-specific top-level gates use
`run-local`; the separate binding avoids filename collisions while the
cumulative manifest remains the acceptance authority. It rejects a candidate
that did not replay an earlier cycle's focused, native-local, exact-test,
ADD-009, ADD-010, live-lock, or snapshot matrix. External CI/release requirements
are cumulative separately: Cycles 3 and 4 require five CI jobs and the exact two
native EXDEV artifacts; Cycle 5 adds both terminal jobs/artifacts; Cycles 6 and 7
retain all seven CI jobs and all four CI artifacts and add all eight release
jobs/artifacts. Tests mutate each earlier gate and case in turn and prove every
later prefix fails, including `hardening-07` when any Cycle 1-6 matrix is absent.

## Complete Local Prior-Gate Command

Task 1 creates `scripts/hardening/run_prior_gates.sh` with exactly this fail-fast
body. Formal execution passes the five profile-resolved absolute executables and
the root-bundle candidate-unittest launcher shown in the catalog row; direct diagnostic execution may supply local absolute
paths but can never become evidence:

```bash
#!/usr/bin/env bash
set -euo pipefail
test "$#" -eq 6
readonly CARGO_BIN="$1" CARGO_MSRV_BIN="$2" PYTHON_BIN="$3" CARGO_DENY_BIN="$4" GIT_BIN="$5" UNITTEST_LAUNCHER="$6"
for TOOL_BIN in "$CARGO_BIN" "$CARGO_MSRV_BIN" "$PYTHON_BIN" "$CARGO_DENY_BIN" "$GIT_BIN" "$UNITTEST_LAUNCHER"; do
  case "$TOOL_BIN" in /*) ;; *) exit 2 ;; esac
done
"$CARGO_BIN" fmt --all -- --check
"$CARGO_BIN" clippy --locked --all-targets --all-features -- -D warnings
"$CARGO_BIN" test --locked --all-targets --all-features
"$CARGO_BIN" build --locked --release --bin tersh
"$CARGO_MSRV_BIN" test --locked --all-targets --all-features
"$PYTHON_BIN" -I -S -B "$UNITTEST_LAUNCHER"
"$CARGO_DENY_BIN" check advisories licenses
"$GIT_BIN" diff --check
```

The candidate-unittest launcher is a root-bundle entrypoint, not candidate code.
It runs under `-I -S -B`, preloads unittest from the pinned stdlib, never adds the
candidate root to ordinary `sys.path`, and installs a closed importer that loads
only the policy-listed `scripts/tests/test_*.py` and their policy-listed
`scripts.*` dependencies by no-follow Git-blob-verified paths under the candidate
root. It rejects candidate `unittest.py`, `sitecustomize.py`, package initializers
or import shadowing, and any unlisted import. The producer closes the Host
Envelope FD in this test child; the surrounding trusted producer alone parses
the result and appends evidence.

Expected after every cycle repair: every command exits 0. The script is the local union only; native EXDEV, multiplexer, and release target matrices retain their additional named gates below.

### Task 1: Cycle 1 — Event-Loop Latency, Supersession, Bounds, CPU, And Memory

**Files:**
- Create: `scripts/hardening/run_gate.py`
- Create: `scripts/hardening/finalize_cycle.py`
- Create: `scripts/hardening/gate_catalog.json`
- Create: `scripts/hardening/run_cumulative_gates.py`
- Create: `scripts/hardening/run_prior_gates.sh`
- Create: `scripts/hardening/candidate_unittest.py`
- Create: `scripts/hardening/verify_implementation_entry.py`
- Create: `scripts/hardening/run_exact_tests.py`
- Create: `scripts/hardening/audit_requirements.py`
- Create: `scripts/hardening/verify_fixed14.py`
- Create: `scripts/tests/test_hardening_evidence.py`
- Create: `scripts/tests/test_requirement_audit.py`
- Create: `docs/superpowers/evidence/2026-08-10-tersh-hardening/expected-requirements.json`
- Create: `tests/hardening_performance.rs`
- Modify: `src/bin/tersh-read-bench.rs` (`main`, benchmark sample schema, environment validation)
- Modify only after a reproduced failure: `src/read_lane.rs` (`ReplaceSlot`, keyed result delivery, close), `src/mutation.rs` (background preflight and `Prepared`/`FenceInstalled` coordination), `src/app.rs` (`poll_background`, read-result application, fence acknowledgement, dirty rendering), `src/ui.rs` (render invalidation)
- Create after all checks pass: `docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-01.json`

- [ ] **Step 1: Write failing evidence-harness tests**

Before creating the first file, capture the clean post-implementation starting commit exactly once:

```bash
test -z "$(git status --porcelain=v1 --untracked-files=all)"
mkdir -p target/hardening-diagnostic/cycle-01
git rev-parse HEAD > target/hardening-diagnostic/cycle-01/hardening-start.txt
```

Before creating or modifying any Task 1 file, use the implementation bundle's
registered generic recorder/sealer to run formal Wave A product, architecture,
and implementation diagnosis concurrently against this clean start commit. The
root supervisor opens the first hardening-01 attempt and host-seals all three
reports; repo-local diagnostic copies are not substitutes. Only after their
terminal responses are captured may the writer begin the failing harness tests.

Add exact Python tests `test_run_gate_preserves_child_exit`, `test_allow_failure_records_nonzero`, `test_output_is_drained_hashed_and_capped`, `test_gate_json_is_canonical_and_atomic`, `test_gate_requires_exact_json_stdout_stderr_triplet`, `test_run_gate_requires_create_new_attempt_candidate_root`, `test_run_gate_rejects_reused_attempt_or_existing_output`, `test_concurrent_attempt_marker_has_one_valid_winner`, `test_run_gate_rejects_head_or_evidence_id_drift`, `test_gate_and_review_attempts_are_three_character_strings`, `test_shared_run_binding_rejects_empty_doubled_trailing_duplicate_reordered_zero_or_unknown_components`, `test_hardening_uses_shared_per_file_orchestration_and_review_schemas`, `test_hardening_rejects_aggregate_orchestration_legacy_schema_and_numeric_attempts`, `test_hardening_host_protocol_reuses_shared_exact_frame_state_machine`, `test_hardening_provenance_uses_rotating_context_capabilities`, `test_hardening_finalizer_enumerates_complete_host_ledger_without_client_ids`, `test_platform_provenance_consumes_host_owned_invocation_and_response_handles`, `test_operator_attestation_requires_host_response_and_rejects_identity_overrides`, `test_shared_provenance_union_rejects_extra_fields_mixed_arms_or_unhashed_bodies`, `test_missing_host_response_cannot_produce_hardening_evidence`, `test_finalize_requires_supervisor_receipt_hashes_and_destination`, `test_verify_only_rehashes_embedded_envelopes_and_rejects_host_fd`, `test_finalize_embeds_every_failed_superseded_and_accepting_attempt`, `test_finalize_rejects_missing_intermediate_or_unembedded_attempt`, `test_finalize_requires_append_only_wave_a_b_c_and_five_role_closure`, `test_finalize_crosschecks_orchestrator_agent_task_and_run_ids`, `test_finalize_embeds_every_review_canonical_body_and_hash`, `test_finalize_embeds_required_gate_and_external_manifest_bodies`, `test_finalize_rejects_unbound_direct_gate_hash`, `test_finalize_requires_parent_finding_resolution_chain`, `test_resolution_ref_binds_attempt_candidate_run_file_and_body_hash`, `test_hardening_rejects_legacy_finding_ids_and_ambiguous_resolution_refs`, `test_finalize_rejects_unresolved_p0_or_p1`, `test_finalize_rejects_candidate_drift`, `test_verify_only_rejects_tampered_manifest`, `test_external_gate_requires_shared_result_manifest`, `test_external_gate_rejects_manifest_path_escape_or_symlink`, `test_external_gate_requires_exact_artifact_templates_and_rejects_extra`, `test_external_artifact_manifest_excludes_itself_and_outer_index_hashes_it`, `test_exact_runner_lists_then_executes_every_required_test`, `test_exact_runner_rejects_zero_discovered_or_executed`, `test_exact_runner_rejects_parameter_case_id_or_count_mismatch`, and `test_prior_gate_script_contains_locked_commands`. Use temporary directories, a child that writes 2 MiB per stream, fixed environment metadata, append-only multi-attempt per-file orchestration/review fixtures, and synthetic shared `tersh-external-candidate-result-v1` plus referenced-manifest fixtures. Host framing uses local `AF_UNIX/SOCK_STREAM` socketpairs and a scripted shared host state machine: first reproduce `EPIPE` for the obsolete body-then-host-half-close sequence, then exercise the exact shared frame order, end markers, final half-closes, rotating `H0 -> H1 -> H2` capabilities, atomic member consumption, and host-selected ledger enumeration. Reject client-supplied receipt IDs, early half-close, reply before COMMIT, missing/duplicate/extra/reordered frames or ledger results, wrong nonce/digest/count, trailing bytes, replay, cross-generation mixing, and partial consumption, always with empty client stdout. Socketpairs test framing only; a pure internal parser accepts synthetic peer/store/current UID triples, while a real production CLI socketpair remains same-UID and fails because production has no expected-UID, environment, or test override. Include negative fixtures for missing response handles, caller-supplied identity/lifecycle/model fields, numeric attempts, legacy hardening schema names, aggregate `orchestration.json`, path-only or hashless resolution references, self-listed artifact manifests, and unlisted payload files.

Also add exact methods
`test_candidate_unittest_launcher_rejects_unittest_sitecustomize_and_scripts_package_shadowing`,
`test_run_exact_tests_formal_mode_requires_attempt_bound_pinned_cargo`,
`test_finalize_requires_five_distinct_agent_runs_and_rejects_cross_role_dispatch_reuse`,
`test_receipted_gate_json_survives_missing_projection_logs_but_present_logs_must_match`,
`test_append_producer_batch_is_all_or_none_and_rejects_chunk_hash_end_or_partial_commit`,
`test_each_formal_cli_uses_distinct_fresh_fd_and_nested_producer_never_inherits_it`,
`test_seal_manifest_preimage_snapshots_streams_payload_and_retries_idempotently_on_one_fd`,
and
`test_commit_close_formal_query_and_audit_pair_have_closed_ordered_replies`.
The distinct-identity fixture independently reuses `agent_id`, `agent_run_id`,
`canonical_task_path`, and `dispatch_id` across two closure roles and between
each closure role and the Wave B writer; every mutation must fail.
They reuse the shared scripted host but independently kill truncated/reordered
RECORD/SPOOL chunks, partial batch visibility, missing/extra phase frames,
same-FD reuse, descendant FD inheritance, snapshot drift, lost-reply retry, and
noncanonical aggregate/result fields.

Also freeze and test the Cycle 7 acceptance policy now, before any hardening
candidate can influence it. Add the exact requirement-catalog and audit tests
listed in Cycle 7 Step 5, including all 269 IDs, ten addendum ranges, logical
reference grammar, host-ledger enumeration, audit revisioning, producer-receipt
bijection, and structural-only offline result. The reviewed hardening catalog,
cycle finalizer, implementation-entry/requirement auditors, schemas, and runtime
dependencies are installed together in Task 1 as the first registered
hardening successor to the implementation-only bundle. The successor starts a
new formal attempt and its closed exact tree contains only the members available
at that boundary; the unit tests remain review/build evidence and need not be
runtime bundle members. Cycle 7 uses that immutable policy and does not add or
approve its own verifier. Freeze exact method
`test_cycle7_review_v2_requires_same_latest_audit_revision_and_package_digest_for_wave_c_and_five_roles`:
it requires v1 and rejects v2 for pre-audit Wave A/B, requires v2 and rejects v1
for post-audit Wave C plus all five closure roles, and rejects missing/extra v2
fields, an old or wrong Wave C revision/package, mixed revision or package
digest across either Wave C report or the five closure roles, an older complete pair when a newer complete
host-receipted audit pair exists, or any repeated agent ID, dispatch, agent run,
or canonical task path across roles. Safety and verification must also be
distinct from each other, and every one of the five accepted closure roles must
differ from the Wave B writer in all four identity dimensions.
Freeze exact method
`test_audit_draft_reservation_is_host_allocated_idempotent_and_blocks_unpaired_or_skipped_revision`:
it races two root-internal reservations, replays a lost result, attempts a gap,
reuse, changed candidate, wrong predecessor, and occupied draft path, and proves
that only the contiguous winner is returned while any newer unpaired reservation
blocks a later `open-attempt`, an older audit pair, and finalizer. It also rejects an invalid draft
without consuming the reservation, permits only no-follow delete/create-new
correction at that same diagnostic path, and then pairs it without allocating a
gap revision.
Freeze exact method
`test_substantive_audit_failure_is_receipted_and_allows_only_next_attempt`:
it binds a closed trusted-audit non-PASS callback to one Host-held failure
authority, loses and replays the callback result under the exact
producer-session/reservation key, rejects changed callback facts without minting
a second authority, races pair versus failure, and fault-injects every durable
transition point. It proves the authority consume, terminal-attempt bit, failure spool,
receipt sequence/link, and idempotency row appear all-or-none. Exact lost-result
replay returns the one stored result; wrong, changed, stale, reused,
cross-session/reservation/attempt authority, a second same-attempt reservation,
or any same/different-revision pair fails. Missing projection repair creates no
second receipt. Only the next candidate attempt and next contiguous revision may
continue, and every later finalizer must embed and resolve the failed attempt
rather than fall back. It rejects an unresolved tombstone finding, a duplicate
finding ID across review/tombstone sources, a same-attempt, circular,
cross-source, or wrong-candidate resolution, and any attempt to omit the
tombstone after its correction.

Before that successor exists, Task 1 Wave A/B reports are still host-receipted,
not repo-local claims: the implementation bundle's already registered generic
`record-orchestration` and `seal-agent-record` rows accept the closed hardening
evidence/role/destination grammar but cannot run or approve a hardening gate or
finalizer. After the Task 1 candidate installs the full hardening successor, the
bundle change forces a new attempt for every Wave C/closure/gate record. The
finalizer embeds both generations and never upgrades the earlier generic reports
into a gate or PASS.

The host tests additionally lock the shared trust and linearization boundaries.
The synthetic credential parser succeeds only for `(peer_uid, fd_st_uid,
client_euid) == (0, 0, nonzero)`. It rejects matching nonroot `(501, 501, 502)`,
nonroot peer `(501, 0, 502)`, nonroot FD owner `(0, 501, 502)`, and root client
`(0, 0, 0)`. The ordinary nonroot repository suite runs a real production-CLI
same-UID negative socketpair but never requires root, changes UID, or skips a
privileged positive while reporting success. The UID-0 TCB-created socketpair
followed by a dropped-nonroot adapter belongs only to privileged host
preflight/acceptance when the platform supplies that supervisor; otherwise
formal evidence fails closed. Pre-COMMIT failures leave every handle retryable
and mutate no host capability, spool, receipt, or local candidate state; the
shared external-producer contract separately governs its already bound unique
remote ref/run effects. Post-COMMIT missing, malformed, or trailing replies keep
stdout empty and reject old-handle replay. A capture successor capability remains
an unaddressable private orphan, while receipt-backed spool objects remain
discoverable only by Host-selected enumeration or their idempotent replay key;
neither class is ever a partial or absent transition. Query fixtures use
`128`, `129`, `325`, and `999` receipts and reject an oversized page, index gap
or reorder, cross-page duplicate, wrong total/page count/order digest, BODY-ID
mismatch, and aggregate body-digest mismatch.

Also add `test_hardening_wrappers_delegate_to_shared_evidence_core`, `test_hardening_exact_runner_reuses_shared_exact_parser`, `test_evidence_id_and_push_ref_use_shared_union_grammar`, `test_entry_requires_impl01_through_impl07`, `test_entry_requires_head_equal_impl07_evidence_closure`, `test_entry_rejects_schema_hash_candidate_or_lineage_drift`, `test_entry_records_start_and_manifest_hashes`, `test_hardening_has_no_external_selector_or_verifier`, `test_exact_runner_requires_external_frozen_case_inventory`, `test_exact_runner_rejects_self_declared_case_drift`, `test_cumulative_catalog_replays_every_prior_cycle_matrix`, `test_cumulative_catalog_rejects_removed_reordered_or_extra_cases`, `test_native_exdev_capability_is_closed_and_not_argv_interpolated`, `test_fixed14_evaluator_requires_policy_set_distinct_fds_and_current_closed_results`, and `test_finalize_requires_committed_candidate_and_evidence_only_output`. Source-contract fixtures fail if a hardening wrapper defines a second drain loop, canonical writer, libtest summary parser, case-record parser, run selector, job verifier, release verifier, or review-closure algorithm. The native-capability test rejects positional root interpolation, unknown environment keys, relative/missing/equal-device roots, and any cumulative attempt that tries to reuse an earlier gate record. The fixed-14 test mutates each fixed key/path, removes or duplicates a result, replays an old result, aliases two descriptors, returns structural-only status, and proves that only fourteen current online results on fourteen distinct fresh FDs produce the one closed aggregate PASS.

- [ ] **Step 2: Verify the harness RED state**

Run: `python3 -B -m unittest scripts.tests.test_hardening_evidence scripts.tests.test_requirement_audit -v`

Expected: FAIL because `run_gate.py`, `finalize_cycle.py`, and `run_prior_gates.sh` do not exist.

- [ ] **Step 3: Implement and verify the evidence harness**

The repository copies in this task are TDD/build inputs and diagnostic tools;
they are never formal producers. Install their reviewed bytes, exact transitive
imports, catalogs, schemas, and policies into the separately registered
root-owned bundle, then execute formal `run-gate`, `run-cumulative`,
`verify-implementation-entry`, `finalize-cycle`, `commit-and-close`, and
`verify-fixed14` only by
the pinned absolute runtime and bundle paths. Hardening-specific gate,
cumulative-catalog, and cycle-manifest policy may call the shared core, but must
not project, rename, aggregate, or copy its canonicalization, bounded drain,
run/job/candidate selection, exact-test parsing, orchestration/review,
host-envelope, receipt-ledger, or finding-closure contracts.

In create mode `finalize_cycle.py` accepts `--evidence-id`,
`--accepting-attempt`, repeated `--required-gate`, exact 40-hex `--candidate`,
the fixed projection roots `target/hardening` and
`target/hardening-external`, the supervisor-injected `--host-store-fd FD`,
the Cycle 1 implementation-entry path assertion, and the one fixed `--output`.
There is no Cycle 7 audit revision, receipt, completion-audit, or pair selector:
the host selects the maximum receipted `audit_revision` for the accepting
attempt, and that revision must be complete, valid, and bound by both accepting
Wave C bodies plus all five latest closure review-v2 bodies—seven reports total—to
the same package digest. It never falls back to an older
complete pair. Every path, selector, required
gate, order, and destination is only an assertion and must equal the registered
policy row exactly; no caller may weaken or omit policy. On its one fresh FD the
finalizer performs exactly one `seal-manifest-preimage` transaction. The host
streams the complete attempt-binding/producer-receipt ledger and every joined
host-only spool body. The finalizer validates committed candidates, predecessor
and receipt chains, every audit-reservation failure tombstone, required gates and external facts, every
orchestration/review relationship, resolution reference, Cycle 7's exact
pre-audit Wave A/B v1 versus post-audit Wave C/closure v2 phase split, both
latest Wave C reports bound to the current audit pair, and the latest complete
five-role closure bound to that same pair. It repairs only a missing projection create-new from exact
host bytes; an existing mismatch, extra, replaced, unreceipted, or
projection-only record fails. It then uploads the canonical payload RECORD and
echoes every snapshot/payload aggregate in COMMIT on that same connection. Only
the unique validated `tersh-host-manifest-preimage-receipt-v1` REPLY permits
create-new publication of the single `tersh-evidence-manifest-envelope-v1`
destination. It never runs Git or reuses the retired FD.
The accepting maximum attempt may not contain a failed or unpaired audit
reservation; failure tombstones from prior attempts are mandatory history and
can never authorize fallback to an older pair.

The separate root-owned `commit-and-close` entrypoint verifies the index and
worktree change set contains only that envelope destination, creates the
evidence-only Git commit,
requires `HEAD^ == candidate` and exact blob bytes, and idempotently appends or
returns the detached closure receipt. `--verify-only MANIFEST` rejects all
create-mode options and the host FD, rehashes only embedded structures, and
emits exactly `STRUCTURAL_PASS/HOST_UNVERIFIED`; only an online authenticated
query that matches bundle registration, attempt binding, preimage, and closure
may return formal PASS. Cycle 1 embeds the exact implementation-entry record and
requires its `entry_head` to equal the host-verified impl-07 closure commit.

The hardening successor bundle also registers root-owned
`verify-fixed14`. Its policy contains exactly the ordered keys and immutable
manifest destinations `impl-01..impl-07,hardening-01..hardening-07`; argv cannot
add, remove, reorder, or substitute a manifest. Its only arguments are exactly
fourteen repetitions of the three argv elements `--host-store-fd KEY FD`, in
policy order. The UID-0
supervisor injects fourteen distinct already authenticated descriptors, one per
key. The entrypoint invokes the shared non-CLI formal-lineage verifier fourteen
times in the same order, first rejecting any equal numeric FD or duplicated
open-file/socket identity and then giving
each call ownership of only its keyed connection. Unused connections remain
CLOEXEC-owned by the evaluator and are closed on the first failure; it never
spawns candidate code, shares a descriptor, or accepts a prior stdout/file as
proof. It emits exactly one
`tersh-fixed14-formal-result-v1`
`{schema,manifest_count:14,ordered_evidence_ids,ordered_manifest_sha256s_sha256,ordered_lineage_result_sha256s_sha256,formal_status:"PASS"}`
only after all fourteen current envelope/Git/root-ledger joins independently
return their closed online PASS. Missing/duplicate/unknown keys, wrong order,
FD alias/reuse, old result, absent manifest, structural-only result, query
failure, or any extra argument yields empty stdout and nonzero status.

The root-bundle `verify-implementation-entry` append policy has a host-side
prerequisite: before accepting its one producer batch, the root registry
internally resolves and validates the fixed impl-01..impl-07 envelope/preimage/
closure/Git lineages. The client supplies no receipt IDs and does not multiplex
seven query transactions onto its append FD. Only after all seven host joins
pass may the entrypoint atomically append the implementation-entry body plus its
policy-named wrapper gate. Shared orchestration recording and external execution remain wholly in
their registered shared entrypoints.

Run: `python3 -B -m unittest scripts.tests.test_hardening_evidence scripts.tests.test_requirement_audit -v`

Expected: the full named target passes, including shared-core delegation, the 2 MiB drain case, create-new attempt/candidate roots, three-character attempt fields, shared per-file orchestration/review schemas, both host-envelope provenance modes, distinct-peer supervisor/receipt validation, rejection when the host response is unavailable, all-attempt embedding, cumulative matrix replay, exact self-excluding artifact inventory plus outer index, implementation-entry equality, external path confinement, latest-closure proof, gate-hash provenance, body-hash-bound finding closure, committed-candidate enforcement, 0-test and externally frozen parameter-case rejection, tampered manifest, and the source-contract proof that Hardening contains no second external selector/verifier or legacy orchestration/review projection.

- [ ] **Step 4: Capture the immutable Cycle 1 baseline**

Run:

```bash
mkdir -p target/hardening-diagnostic/cycle-01
TERSH_H1_FIXTURE="$(mktemp -d /tmp/tersh-hardening-01.XXXXXX)"
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-01 --cycle 01 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name baseline-implementation-entry --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/hardening/verify_implementation_entry.py --entry-head "$(cat target/hardening-diagnostic/cycle-01/hardening-start.txt)" --candidate "$(cat target/hardening-diagnostic/cycle-01/hardening-start.txt)" --manifest-root docs/superpowers/evidence/2026-08-10-tersh-implementation --output "target/hardening-diagnostic/hardening-01/attempt-001/candidate-$(cat target/hardening-diagnostic/cycle-01/hardening-start.txt)/run-local/baseline-implementation-entry.json"
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-01 --cycle 01 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name baseline-read --allow-failure --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- cargo run --release --locked --bin tersh-read-bench -- --fixture-root "$TERSH_H1_FIXTURE"
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-01 --cycle 01 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name baseline-tests --allow-failure --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- cargo test --locked --test read_lane --test app_async --test plan2_acceptance -- --nocapture --test-threads=1
```

Expected: the implementation-entry gate must pass before hardening continues; the two allow-failure baseline records still exist if a current performance or read-lane invariant fails. The benchmark contains 10 warmups plus 200 measured inputs, key-to-render p50/p95/max, longest stall, first-frame and first-directory-result times, scan/preview supersession counts, stale-applied count, pending-slot maxima, idle CPU/RSS, fixture manifest, and reference-environment identity. It refuses reference-gate status unless the host matches the design's named M4 Max/APFS profile.

- [ ] **Step 5: Review the already sealed Wave A reports**

Use the three formal reports captured before Step 1 edited the tree. Product checks that the inspection north star remains usable and metrics are local evidence rather than telemetry. Architecture checks one replaceable slot per lane, result-channel bounds, mutation fences, generation/epoch application, and dirty-render ownership. Implementation diagnoses the raw baseline and names only reproducible bottlenecks. Any additional repo-local discussion while the harness worktree is dirty belongs only under `target/hardening-diagnostic`; it cannot replace or amend those append-only Wave A records.

- [ ] **Step 6: Add the focused fault/load matrix before fixing production code**

Create exact tests `ten_thousand_scan_requests_leave_one_pending`, `ten_thousand_preview_requests_leave_one_pending`, `directory_and_recovery_catalog_requests_never_supersede_each_other`, `ten_thousand_target_preflight_never_blocks_event_loop`, `superseded_results_never_apply`, `worker_output_pressure_does_not_block_input`, `unchanged_tick_emits_no_dirty_render`, `slow_scan_and_preview_keep_two_hundred_inputs_under_frozen_gate`, and `benchmark_rejects_wrong_reference_profile`. Use barriers and injected 1,000 ms read/preflight backends; never use wall-clock sleep to order races.

Run: `python3 -B scripts/hardening/run_exact_tests.py --target hardening_performance --require-test ten_thousand_scan_requests_leave_one_pending --require-test ten_thousand_preview_requests_leave_one_pending --require-test directory_and_recovery_catalog_requests_never_supersede_each_other --require-test ten_thousand_target_preflight_never_blocks_event_loop --require-test superseded_results_never_apply --require-test worker_output_pressure_does_not_block_input --require-test unchanged_tick_emits_no_dirty_render --require-test slow_scan_and_preview_keep_two_hundred_inputs_under_frozen_gate --require-test benchmark_rejects_wrong_reference_profile`

Expected: every named case runs. A violated bound or frozen 100 ms threshold FAILS with observed/expected values; a full pass is recorded as `no_reproduced_defect` and does not justify production refactoring.

- [ ] **Step 7: Apply the smallest Cycle 1 repair and rerun focused evidence**

Change only the named read-lane, background mutation-preflight, App polling/application/fence-acknowledgement, or render-invalidation functions implicated by a failing case. Preserve two dedicated read workers, keyed `Directory`/`RecoveryCatalog` replacement with fair alternation, the shared bounded result path, conservative provisional fences until immutable preparation, stale-last-good UI, and the existing 100 ms threshold; do not add a pool, cache service, or new metric surface.

Run:

```bash
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-01 --cycle 01 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name focused-performance --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/hardening/run_exact_tests.py --target hardening_performance --require-test ten_thousand_scan_requests_leave_one_pending --require-test ten_thousand_preview_requests_leave_one_pending --require-test directory_and_recovery_catalog_requests_never_supersede_each_other --require-test ten_thousand_target_preflight_never_blocks_event_loop --require-test superseded_results_never_apply --require-test worker_output_pressure_does_not_block_input --require-test unchanged_tick_emits_no_dirty_render --require-test slow_scan_and_preview_keep_two_hundred_inputs_under_frozen_gate --require-test benchmark_rejects_wrong_reference_profile --regression-target read_lane --regression-target app_async --regression-target plan2_acceptance
TERSH_H1_FINAL_FIXTURE="$(mktemp -d /tmp/tersh-hardening-01-final.XXXXXX)"
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-01 --cycle 01 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name reference-performance --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- cargo run --release --locked --bin tersh-read-bench -- --fixture-root "$TERSH_H1_FINAL_FIXTURE" --require-reference-profile
```

Expected: stale applied results are 0, pending maxima are 1 per lane, 200 measured inputs meet p95 and longest-stall <=100 ms on the named reference machine, and CPU/RSS plus non-threshold timing fields are reported without invented pass percentages.

- [ ] **Step 8: Run all prior gates**

Run: `python3 -B scripts/hardening/run_gate.py --evidence-id hardening-01 --cycle 01 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name prior-gates --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- scripts/hardening/run_prior_gates.sh`

Expected: exit 0 with every locked local gate passing.

- [ ] **Step 9: Commit the candidate, rerun every gate, and run Wave C plus both closure waves**

Stage exactly the Task 1 scripts, policies, tests, and any reproduced minimal
source fix. After the candidate commit, the implementation role finishes its
Wave B execution report; the still-current implementation bundle's generic
sealer opens a new attempt for this clean candidate and host-seals that report.
Only then, and before the first hardening-gate command carrying a
`SUPERVISOR_*` metavariable, stop for the distinct UID-0 supervisor to build the
exact-tree bundle, independently approve and register its manifest/policies and
pinned runtime profile, and make its registration receipt queryable. The
bundle change opens another attempt for the same candidate; it cannot rewrite
the Wave A/B receipts. The candidate process cannot perform, approve, or skip that installation. If the
privileged boundary is unavailable, diagnostics may continue but formal Cycle 1
fails closed. Then run:

```text
git add scripts/hardening/run_gate.py scripts/hardening/finalize_cycle.py scripts/hardening/gate_catalog.json scripts/hardening/run_cumulative_gates.py scripts/hardening/run_prior_gates.sh scripts/hardening/candidate_unittest.py scripts/hardening/verify_implementation_entry.py scripts/hardening/run_exact_tests.py scripts/hardening/audit_requirements.py scripts/hardening/verify_fixed14.py scripts/tests/test_hardening_evidence.py scripts/tests/test_requirement_audit.py docs/superpowers/evidence/2026-08-10-tersh-hardening/expected-requirements.json tests/hardening_performance.rs src/bin/tersh-read-bench.rs src/read_lane.rs src/mutation.rs src/app.rs src/ui.rs
git diff --exit-code
test -z "$(git ls-files --others --exclude-standard)"
python3 -B -c 'import subprocess; required=set("scripts/hardening/run_gate.py scripts/hardening/finalize_cycle.py scripts/hardening/gate_catalog.json scripts/hardening/run_cumulative_gates.py scripts/hardening/run_prior_gates.sh scripts/hardening/candidate_unittest.py scripts/hardening/verify_implementation_entry.py scripts/hardening/run_exact_tests.py scripts/hardening/audit_requirements.py scripts/hardening/verify_fixed14.py scripts/tests/test_hardening_evidence.py scripts/tests/test_requirement_audit.py docs/superpowers/evidence/2026-08-10-tersh-hardening/expected-requirements.json tests/hardening_performance.rs src/bin/tersh-read-bench.rs".split()); allowed=required|set("src/read_lane.rs src/mutation.rs src/app.rs src/ui.rs".split()); actual=set(subprocess.check_output(["git","diff","--cached","--name-only"], text=True).splitlines()); assert required <= actual <= allowed, (actual, required, allowed)'
git commit -m "test: freeze event-loop hardening candidate"
TERSH_H1_CANDIDATE="$(git rev-parse HEAD)"
test -z "$(git status --porcelain=v1 --untracked-files=all)"
case "$TERSH_H1_CANDIDATE" in ''|*[!0-9a-f]*) exit 1 ;; esac
test "$(printf '%s' "$TERSH_H1_CANDIDATE" | wc -c | tr -d ' ')" = 40
: "${TERSH_H1_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_cumulative_gates.py --catalog SUPERVISOR_HARNESS_ROOT/scripts/hardening/gate_catalog.json --through hardening-01 --attempt "$TERSH_H1_ATTEMPT" --candidate "$TERSH_H1_CANDIDATE" --output-root target/hardening --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/verify_implementation_entry.py --evidence-id hardening-01 --attempt "$TERSH_H1_ATTEMPT" --entry-head SUPERVISOR_POLICY_IMPL07_EVIDENCE_COMMIT --candidate "$TERSH_H1_CANDIDATE" --manifest-root docs/superpowers/evidence/2026-08-10-tersh-implementation --output "target/hardening/hardening-01/attempt-$TERSH_H1_ATTEMPT/candidate-$TERSH_H1_CANDIDATE/run-local/implementation-entry.json" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_gate.py --evidence-id hardening-01 --cycle 01 --attempt "$TERSH_H1_ATTEMPT" --candidate "$TERSH_H1_CANDIDATE" --run-binding run-local --name focused-performance --output-root target/hardening --host-store-fd FRESH_HOST_FD -- SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_exact_tests.py --cargo-bin SUPERVISOR_PINNED_CARGO --target hardening_performance --require-test ten_thousand_scan_requests_leave_one_pending --require-test ten_thousand_preview_requests_leave_one_pending --require-test directory_and_recovery_catalog_requests_never_supersede_each_other --require-test ten_thousand_target_preflight_never_blocks_event_loop --require-test superseded_results_never_apply --require-test worker_output_pressure_does_not_block_input --require-test unchanged_tick_emits_no_dirty_render --require-test slow_scan_and_preview_keep_two_hundred_inputs_under_frozen_gate --require-test benchmark_rejects_wrong_reference_profile --regression-target read_lane --regression-target app_async --regression-target plan2_acceptance
TERSH_H1_FINAL_FIXTURE="$(mktemp -d /tmp/tersh-hardening-01-committed.XXXXXX)"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_gate.py --evidence-id hardening-01 --cycle 01 --attempt "$TERSH_H1_ATTEMPT" --candidate "$TERSH_H1_CANDIDATE" --run-binding run-local --name reference-performance --output-root target/hardening --host-store-fd FRESH_HOST_FD -- SUPERVISOR_PINNED_CARGO run --release --locked --bin tersh-read-bench -- --fixture-root "$TERSH_H1_FINAL_FIXTURE" --require-reference-profile
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_gate.py --evidence-id hardening-01 --cycle 01 --attempt "$TERSH_H1_ATTEMPT" --candidate "$TERSH_H1_CANDIDATE" --run-binding run-local --name prior-gates --output-root target/hardening --host-store-fd FRESH_HOST_FD -- SUPERVISOR_PINNED_BASH SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_prior_gates.sh SUPERVISOR_PINNED_CARGO SUPERVISOR_PINNED_CARGO_MSRV SUPERVISOR_PINNED_PYTHON SUPERVISOR_PINNED_CARGO_DENY SUPERVISOR_PINNED_GIT SUPERVISOR_HARNESS_ROOT/scripts/hardening/candidate_unittest.py
```

Run Wave C safety and verification concurrently on `TERSH_H1_CANDIDATE`: safety attacks stale application, channel pressure, panic/close, and resource exhaustion; verification independently repeats `implementation-entry`, `focused-performance`, `reference-performance`, and `prior-gates` and checks that no command ran zero tests. Run Closure A product, architecture, and implementation diagnosis concurrently, followed by Closure B safety and verification, on that identical commit. All five reports must say `PASS`. Any finding returns to Wave B; commit a new candidate, rerun every command in this step, and discard the prior closure set from acceptance while preserving it append-only.

- [ ] **Step 10: Finalize evidence and commit Cycle 1**

Run:

```text
TERSH_H1_CANDIDATE="$(git rev-parse HEAD)"
: "${TERSH_H1_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/finalize_cycle.py --cycle 01 --evidence-id hardening-01 --accepting-attempt "$TERSH_H1_ATTEMPT" --candidate "$TERSH_H1_CANDIDATE" --raw-root target/hardening --external-root target/hardening-external --implementation-entry "target/hardening/hardening-01/attempt-$TERSH_H1_ATTEMPT/candidate-$TERSH_H1_CANDIDATE/run-local/implementation-entry.json" --required-gate implementation-entry --required-gate focused-performance --required-gate reference-performance --required-gate prior-gates --required-gate cumulative-gates --host-store-fd FRESH_HOST_FD --output docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-01.json
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/commit_and_close.py --manifest docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-01.json --candidate "$TERSH_H1_CANDIDATE" --commit-message "docs: record event-loop hardening evidence" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/verify_formal_lineage.py --manifest docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-01.json --host-store-fd FRESH_HOST_FD
```

Expected: finalization exits 0, the manifest embeds the exact implementation-entry record and candidate commit, and the closure commit contains only Cycle 1 evidence. Cycle 1 is accepted; no full-task claim is made.

### Task 2: Cycle 2 — Cancellation, Terminal Races, Worker Loss, And Shutdown

**Files:**
- Create: `tests/hardening_shutdown.rs`
- Modify only after a reproduced failure: `src/mutation.rs` (`request_cancel`, terminal emission, disconnect observer), `src/mutation_ops.rs` (safe-point checks), `src/operation.rs` (`first-final-wins` reduction), `src/app.rs` (`ShutdownRequested`, background drain), `src/terminal_session.rs` (restore ordering), `src/main.rs` (`RunOutcome` handoff)
- Modify: `tests/support/pty.rs` (deterministic signal/barrier helpers)
- Create after all checks pass: `docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-02.json`

- [ ] **Step 1: Capture baseline shutdown and cancellation evidence**

Run:

```bash
mkdir -p target/hardening-diagnostic/cycle-02
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-02 --cycle 02 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name baseline-shutdown --allow-failure --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- cargo test --locked --test mutation_worker --test shutdown --test cli -- --nocapture --test-threads=1
```

Expected: the record contains every existing cancel, worker-loss, exit-code, stdout, and terminal-restoration result, including any failure.

- [ ] **Step 2: Run Wave A with three `gpt-5.6-sol` `xhigh` roles**

Product checks that cancel acknowledgement is truthful, completed items remain completed, unresolved mutation prevents cwd commit, and failures remain actionable. Architecture checks first-final-wins, exactly one `Finished`, non-droppable outcomes, safe-point boundaries, and shutdown ordering. Implementation diagnosis searches deterministic schedules for cancel/success, panic/disconnect, quit/signal, and terminal-error races. Each agent create-new writes its exact draft only at the shared policy-derived agent-draft path; after the callback digest, the root sealer alone publishes the receipt-backed formal review projection. Wave A is read-only.

- [ ] **Step 3: Add deterministic race and fault-reproduction tests**

Add exact tests `cancel_before_start_marks_only_queued_not_started`, `cancel_before_fence_ack_proves_no_effect_or_cleanup`, `cancel_during_chunked_copy_stops_at_next_safe_point`, `cancel_after_publish_never_relabels_committed`, `cancel_success_race_emits_one_finished_for_256_schedules`, `item_panic_reports_observed_or_indeterminate`, `worker_disconnect_cannot_leave_running_report`, `full_event_channel_is_drained_before_worker_join`, `shift_q_waits_for_safe_point_and_never_commits_cwd`, `sigterm_returns_143_after_drain`, `render_failure_restores_then_drains_noninteractive`, and `terminal_restore_failure_never_writes_stdout`. Drive order with barriers/fault hooks, not sleeps.

Run: `python3 -B scripts/hardening/run_exact_tests.py --target hardening_shutdown --require-test cancel_before_start_marks_only_queued_not_started --require-test cancel_before_fence_ack_proves_no_effect_or_cleanup --require-test cancel_during_chunked_copy_stops_at_next_safe_point --require-test cancel_after_publish_never_relabels_committed --require-test cancel_success_race_emits_one_finished_for_256_schedules --require-test item_panic_reports_observed_or_indeterminate --require-test worker_disconnect_cannot_leave_running_report --require-test full_event_channel_is_drained_before_worker_join --require-test shift_q_waits_for_safe_point_and_never_commits_cwd --require-test sigterm_returns_143_after_drain --require-test render_failure_restores_then_drains_noninteractive --require-test terminal_restore_failure_never_writes_stdout --matrix cancel-success-schedules --expect-case-range cancel-success-schedules:schedule-:0:255:3`

Expected: all twelve cases execute; any duplicate/missing terminal event, incorrect cwd output, unsafe interruption, pre-fence residue, join-before-drain, or post-commit relabel FAILS. An all-pass result records `no_reproduced_defect`.

- [ ] **Step 4: Apply only reproduced Cycle 2 fixes**

Keep cancellation cooperative: queued work stops immediately, copies stop between bounded chunks, and claim/publish/source-cleanup critical sections finish before terminal outcome. Preserve the exact `Running -> ShutdownRequested -> cancel/drain -> mutation terminal -> terminal restore -> RunOutcome` order. Do not add rollback of completed items, force-kill a mutation worker, or create resumable jobs.

- [ ] **Step 5: Run focused and prior gates**

Run:

```bash
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-02 --cycle 02 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name focused-shutdown --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/hardening/run_exact_tests.py --target hardening_shutdown --require-test cancel_before_start_marks_only_queued_not_started --require-test cancel_before_fence_ack_proves_no_effect_or_cleanup --require-test cancel_during_chunked_copy_stops_at_next_safe_point --require-test cancel_after_publish_never_relabels_committed --require-test cancel_success_race_emits_one_finished_for_256_schedules --require-test item_panic_reports_observed_or_indeterminate --require-test worker_disconnect_cannot_leave_running_report --require-test full_event_channel_is_drained_before_worker_join --require-test shift_q_waits_for_safe_point_and_never_commits_cwd --require-test sigterm_returns_143_after_drain --require-test render_failure_restores_then_drains_noninteractive --require-test terminal_restore_failure_never_writes_stdout --regression-target mutation_worker --regression-target shutdown --regression-target cli --matrix cancel-success-schedules --expect-case-range cancel-success-schedules:schedule-:0:255:3
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-02 --cycle 02 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name prior-gates --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- scripts/hardening/run_prior_gates.sh
```

Expected: both records exit 0; every operation is terminal exactly once, abort paths emit no cwd, and all Cycle 1/prior slice regressions remain green.

- [ ] **Step 6: Commit the Cycle 2 candidate, rerun gates, and obtain final five-role sign-off**

Run:

```text
git add tests/hardening_shutdown.rs tests/support/pty.rs src/mutation.rs src/mutation_ops.rs src/operation.rs src/app.rs src/terminal_session.rs src/main.rs
git diff --exit-code
test -z "$(git ls-files --others --exclude-standard)"
python3 -B -c 'import subprocess; required={"tests/hardening_shutdown.rs","tests/support/pty.rs"}; allowed=required|set("src/mutation.rs src/mutation_ops.rs src/operation.rs src/app.rs src/terminal_session.rs src/main.rs".split()); actual=set(subprocess.check_output(["git","diff","--cached","--name-only"], text=True).splitlines()); assert required <= actual <= allowed, (actual, required, allowed)'
git commit -m "test: freeze cancellation hardening candidate"
TERSH_H2_CANDIDATE="$(git rev-parse HEAD)"
test -z "$(git status --porcelain=v1 --untracked-files=all)"
case "$TERSH_H2_CANDIDATE" in ''|*[!0-9a-f]*) exit 1 ;; esac
test "$(printf '%s' "$TERSH_H2_CANDIDATE" | wc -c | tr -d ' ')" = 40
: "${TERSH_H2_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_cumulative_gates.py --catalog SUPERVISOR_HARNESS_ROOT/scripts/hardening/gate_catalog.json --through hardening-02 --attempt "$TERSH_H2_ATTEMPT" --candidate "$TERSH_H2_CANDIDATE" --output-root target/hardening --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_gate.py --evidence-id hardening-02 --cycle 02 --attempt "$TERSH_H2_ATTEMPT" --candidate "$TERSH_H2_CANDIDATE" --run-binding run-local --name focused-shutdown --output-root target/hardening --host-store-fd FRESH_HOST_FD -- SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_exact_tests.py --cargo-bin SUPERVISOR_PINNED_CARGO --target hardening_shutdown --require-test cancel_before_start_marks_only_queued_not_started --require-test cancel_before_fence_ack_proves_no_effect_or_cleanup --require-test cancel_during_chunked_copy_stops_at_next_safe_point --require-test cancel_after_publish_never_relabels_committed --require-test cancel_success_race_emits_one_finished_for_256_schedules --require-test item_panic_reports_observed_or_indeterminate --require-test worker_disconnect_cannot_leave_running_report --require-test full_event_channel_is_drained_before_worker_join --require-test shift_q_waits_for_safe_point_and_never_commits_cwd --require-test sigterm_returns_143_after_drain --require-test render_failure_restores_then_drains_noninteractive --require-test terminal_restore_failure_never_writes_stdout --regression-target mutation_worker --regression-target shutdown --regression-target cli --matrix cancel-success-schedules --expect-case-range cancel-success-schedules:schedule-:0:255:3
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_gate.py --evidence-id hardening-02 --cycle 02 --attempt "$TERSH_H2_ATTEMPT" --candidate "$TERSH_H2_CANDIDATE" --run-binding run-local --name prior-gates --output-root target/hardening --host-store-fd FRESH_HOST_FD -- SUPERVISOR_PINNED_BASH SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_prior_gates.sh SUPERVISOR_PINNED_CARGO SUPERVISOR_PINNED_CARGO_MSRV SUPERVISOR_PINNED_PYTHON SUPERVISOR_PINNED_CARGO_DENY SUPERVISOR_PINNED_GIT SUPERVISOR_HARNESS_ROOT/scripts/hardening/candidate_unittest.py
```

Run Wave C safety and verification concurrently on `TERSH_H2_CANDIDATE`: safety injects every cancellation boundary plus panic/disconnect and terminal errors; verification repeats focused/prior commands, validates exact exit codes 0/1/64/129/130/143, and rejects 0-test filters. Run Closure A product, architecture, and implementation diagnosis, then Closure B safety and verification, on that same committed candidate. A finding creates a new candidate commit and reruns both gates plus all closure waves; all attempts remain append-only.

- [ ] **Step 7: Finalize evidence and commit Cycle 2**

Run:

```text
TERSH_H2_CANDIDATE="$(git rev-parse HEAD)"
: "${TERSH_H2_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/finalize_cycle.py --cycle 02 --evidence-id hardening-02 --accepting-attempt "$TERSH_H2_ATTEMPT" --candidate "$TERSH_H2_CANDIDATE" --raw-root target/hardening --external-root target/hardening-external --required-gate focused-shutdown --required-gate prior-gates --required-gate cumulative-gates --host-store-fd FRESH_HOST_FD --output docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-02.json
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/commit_and_close.py --manifest docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-02.json --candidate "$TERSH_H2_CANDIDATE" --commit-message "docs: record cancellation hardening evidence" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/verify_formal_lineage.py --manifest docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-02.json --host-store-fd FRESH_HOST_FD
```

Expected: the evidence-only commit contains exactly the Cycle 2 manifest bound to the committed candidate; passing does not claim recovery, release, or full-task completion.

### Task 3: Cycle 3 — Filesystem Permission, Identity, Symlink, Target, And EXDEV Fault Matrix

**Files:**
- Create: `tests/hardening_fs_faults.rs`
- Modify: `tests/exdev.rs`, `src/exdev.rs` (crate-unit hardening ADD-010 matrices), `scripts/test-exdev-native.sh`, `.github/workflows/ci.yml` (`native-exdev` gate only)
- Modify only after a reproduced failure: `src/trusted_fs.rs`, `src/source_claim.rs`, `src/mutation_ops.rs`, `src/state_root.rs`
- Create after all checks pass: `docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-03.json`

- [ ] **Step 1: Capture the existing injected and native-capability baseline**

Run:

```bash
mkdir -p target/hardening-diagnostic/cycle-03
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-03 --cycle 03 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name baseline-fs-faults --allow-failure --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- cargo test --locked --test trusted_fs --test source_claim --test mutation_ops --test exdev -- --nocapture --test-threads=1
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-03 --cycle 03 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name baseline-native-script --allow-failure --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- scripts/test-exdev-native.sh
```

Expected: injected tests always run. The zero-argument native command reads only `TERSH_H3_EXDEV_ROOT_A/B` and exits nonzero rather than skipping when either variable is missing, non-absolute, non-canonical, absent, unwritable, or on equal `st_dev`; supplied roots on different devices run both exact ignored native tests. Any positional argument also fails closed.

- [ ] **Step 2: Run Wave A with three `gpt-5.6-sol` `xhigh` roles**

Product checks that supported failures never lose the unique source, never overwrite a target, and produce actionable exact outcomes. Architecture checks fd-relative no-follow/no-replace operations, fixed-root trust, source claims, private staging, and EXDEV transition ownership. Implementation diagnosis builds the operation-by-fault matrix and reproduces only concrete gaps. Each agent create-new writes its read-only draft only at the shared policy-derived agent-draft path; after the callback digest, the root sealer alone publishes the receipt-backed formal review projection.

- [ ] **Step 3: Add the complete table-driven filesystem fault matrix**

Add integration tests `complete_operation_fault_matrix_has_exact_cases`, `enospc_matrix_preserves_unique_source`, `eacces_matrix_has_no_unowned_cleanup`, `source_identity_drift_never_moves_replacement`, `destination_parent_swap_never_publishes_outside_verified_parent`, `symlink_component_is_never_followed`, `raw_unix_name_rejects_empty_dot_dot_slash_and_nul`, `raw_unix_path_and_name_deserialize_reject_forged_capabilities`, `one_item_uses_one_fixed_control_bundle`, `native_script_rejects_arguments_and_invalid_environment`, `target_competition_never_overwrites`, `file_and_directory_sync_failures_are_not_failed_no_effect_after_publish`, `exdev_regular_and_symlink_fault_matrix_is_truthful`, `exdev_directory_and_special_file_are_rejected_before_effect`, `two_process_source_claim_has_one_winner`, and `cancel_at_every_exdev_transition_preserves_a_valid_copy`. In `src/exdev.rs`, add crate-unit parameter tests `hardening_add_010_proof_replay_matrix_has_exact_cases` and `hardening_add_010_lock_liveness_matrix_has_exact_cases`; only these owning-module tests may construct genuine private proofs and lock-bearing typestates. Together they cover copy, same-FS move/rename/trash, empty permanent delete, regular/symlink EXDEV, checksum, metadata, file-sync, directory-sync, cleanup failures, deserialization forgery, the native script's closed zero-argv environment contract, cross-bundle/revision/edge replay of genuine verifier-issued tokens, a compile-fail second use of an already-consumed token, and the live-lock/snapshot swap schedule from design line 1505.

Run:

```bash
TERSH_H3_MATRIX_ARGS=(
  --matrix hardening-fs-operation-faults
  --expect-case hardening-fs-operation-faults:copy-enospc --expect-case hardening-fs-operation-faults:copy-eacces --expect-case hardening-fs-operation-faults:samefs-move-eacces --expect-case hardening-fs-operation-faults:rename-parent-swap --expect-case hardening-fs-operation-faults:trash-eacces --expect-case hardening-fs-operation-faults:empty-delete-eacces --expect-case hardening-fs-operation-faults:exdev-regular-enospc --expect-case hardening-fs-operation-faults:exdev-regular-eacces --expect-case hardening-fs-operation-faults:exdev-checksum --expect-case hardening-fs-operation-faults:exdev-metadata --expect-case hardening-fs-operation-faults:exdev-file-sync --expect-case hardening-fs-operation-faults:exdev-directory-sync --expect-case hardening-fs-operation-faults:exdev-source-swap --expect-case hardening-fs-operation-faults:exdev-target-race --expect-case hardening-fs-operation-faults:exdev-cancel-prepublish --expect-case hardening-fs-operation-faults:exdev-cancel-postpublish --expect-case hardening-fs-operation-faults:exdev-symlink --expect-case hardening-fs-operation-faults:exdev-directory-reject --expect-case hardening-fs-operation-faults:exdev-special-reject --expect-case hardening-fs-operation-faults:source-claim-two-process
  --matrix hardening-add-009-serde
  --expect-case hardening-add-009-serde:empty-name --expect-case hardening-add-009-serde:dot-name --expect-case hardening-add-009-serde:dotdot-name --expect-case hardening-add-009-serde:slash-name --expect-case hardening-add-009-serde:nul-name --expect-case hardening-add-009-serde:padded-base64 --expect-case hardening-add-009-serde:aliased-base64 --expect-case hardening-add-009-serde:malformed-base64 --expect-case hardening-add-009-serde:invalid-path-component
)
python3 -B scripts/hardening/run_exact_tests.py --target hardening_fs_faults --require-test complete_operation_fault_matrix_has_exact_cases --require-test enospc_matrix_preserves_unique_source --require-test eacces_matrix_has_no_unowned_cleanup --require-test source_identity_drift_never_moves_replacement --require-test destination_parent_swap_never_publishes_outside_verified_parent --require-test symlink_component_is_never_followed --require-test raw_unix_name_rejects_empty_dot_dot_slash_and_nul --require-test raw_unix_path_and_name_deserialize_reject_forged_capabilities --require-test one_item_uses_one_fixed_control_bundle --require-test native_script_rejects_arguments_and_invalid_environment --require-test target_competition_never_overwrites --require-test file_and_directory_sync_failures_are_not_failed_no_effect_after_publish --require-test exdev_regular_and_symlink_fault_matrix_is_truthful --require-test exdev_directory_and_special_file_are_rejected_before_effect --require-test two_process_source_claim_has_one_winner --require-test cancel_at_every_exdev_transition_preserves_a_valid_copy "${TERSH_H3_MATRIX_ARGS[@]}"
python3 -B scripts/run_exact_test.py --lib --name exdev::tests::hardening_add_010_proof_replay_matrix_has_exact_cases --serial --case-matrix hardening-add-010-proof-replay --expect-case other-bundle --expect-case other-revision --expect-case other-edge --expect-case same-token-second-use
python3 -B scripts/run_exact_test.py --lib --name exdev::tests::hardening_add_010_lock_liveness_matrix_has_exact_cases --serial --case-matrix hardening-add-010-lock-liveness --expect-case lock-dropped-before-transition --expect-case snapshot-replaced-while-unlocked
```

Expected: all sixteen integration tests and all 29 integration-matrix cases execute; the two private `--lib` tests independently execute all four proof-replay and both lock-liveness cases. Any source loss, clobber, followed symlink, forged raw capability, open positional/native environment contract, replayed or reused proof, proof surviving a dropped lock/snapshot, duplicate fixed control, wrong parent, fabricated `FailedNoEffect`, or unsupported-object mutation FAILS; a fully passing matrix records `no_reproduced_defect`.

- [ ] **Step 4: Apply the minimum filesystem correction**

The genuine-proof matrix includes the four frozen ADD-010 replay cases. `same-token-second-use` compiles a negative ownership fixture and requires the compiler to reject the second by-value use. The lock-liveness matrix additionally proves that every accepted authorizing fact borrows or owns the actual no-follow lock and verified receipt snapshot until the transition returns; a prior observation whose lock was dropped cannot authorize either fixed or adjacent advancement after a deterministic object swap.

Repair only the lowest shared failing primitive or state transition. Preserve no-replace publication, raw Unix paths, source-claim/tombstone ownership, per-item receipts, file/symlink-only EXDEV, and disabled non-empty recursive delete/replace. Keep the shared external-evidence trigger contract on `.github/workflows/ci.yml` so the two named native jobs run against and verify the exact candidate SHA. Each native job uploads exactly its frozen logical artifact name with a root `artifact-manifest.json` in schema `tersh-native-exdev-evidence-v1`; the manifest binds producer job, candidate, run ID/attempt, device identities, executed case IDs, and nonempty file hashes. Do not add portability fallbacks that weaken Tier-1 invariants.

- [ ] **Step 5: Run focused, local-native, and all prior gates**

Run:

```bash
test -n "${TERSH_H3_EXDEV_ROOT_A:-}" || exit 1
test -n "${TERSH_H3_EXDEV_ROOT_B:-}" || exit 1
export TERSH_H3_EXDEV_ROOT_A TERSH_H3_EXDEV_ROOT_B
TERSH_H3_MATRIX_ARGS=(
  --matrix hardening-fs-operation-faults
  --expect-case hardening-fs-operation-faults:copy-enospc --expect-case hardening-fs-operation-faults:copy-eacces --expect-case hardening-fs-operation-faults:samefs-move-eacces --expect-case hardening-fs-operation-faults:rename-parent-swap --expect-case hardening-fs-operation-faults:trash-eacces --expect-case hardening-fs-operation-faults:empty-delete-eacces --expect-case hardening-fs-operation-faults:exdev-regular-enospc --expect-case hardening-fs-operation-faults:exdev-regular-eacces --expect-case hardening-fs-operation-faults:exdev-checksum --expect-case hardening-fs-operation-faults:exdev-metadata --expect-case hardening-fs-operation-faults:exdev-file-sync --expect-case hardening-fs-operation-faults:exdev-directory-sync --expect-case hardening-fs-operation-faults:exdev-source-swap --expect-case hardening-fs-operation-faults:exdev-target-race --expect-case hardening-fs-operation-faults:exdev-cancel-prepublish --expect-case hardening-fs-operation-faults:exdev-cancel-postpublish --expect-case hardening-fs-operation-faults:exdev-symlink --expect-case hardening-fs-operation-faults:exdev-directory-reject --expect-case hardening-fs-operation-faults:exdev-special-reject --expect-case hardening-fs-operation-faults:source-claim-two-process
  --matrix hardening-add-009-serde
  --expect-case hardening-add-009-serde:empty-name --expect-case hardening-add-009-serde:dot-name --expect-case hardening-add-009-serde:dotdot-name --expect-case hardening-add-009-serde:slash-name --expect-case hardening-add-009-serde:nul-name --expect-case hardening-add-009-serde:padded-base64 --expect-case hardening-add-009-serde:aliased-base64 --expect-case hardening-add-009-serde:malformed-base64 --expect-case hardening-add-009-serde:invalid-path-component
  --matrix add-009-exdev-serde
  --expect-case add-009-exdev-serde:empty-name --expect-case add-009-exdev-serde:dot-name --expect-case add-009-exdev-serde:dotdot-name --expect-case add-009-exdev-serde:slash-name --expect-case add-009-exdev-serde:nul-name --expect-case add-009-exdev-serde:padded-base64 --expect-case add-009-exdev-serde:aliased-base64 --expect-case add-009-exdev-serde:malformed-base64 --expect-case add-009-exdev-serde:invalid-path-component
)
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-03 --cycle 03 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name focused-fs-faults --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/hardening/run_exact_tests.py --target hardening_fs_faults --require-test complete_operation_fault_matrix_has_exact_cases --require-test enospc_matrix_preserves_unique_source --require-test eacces_matrix_has_no_unowned_cleanup --require-test source_identity_drift_never_moves_replacement --require-test destination_parent_swap_never_publishes_outside_verified_parent --require-test symlink_component_is_never_followed --require-test raw_unix_name_rejects_empty_dot_dot_slash_and_nul --require-test raw_unix_path_and_name_deserialize_reject_forged_capabilities --require-test one_item_uses_one_fixed_control_bundle --require-test native_script_rejects_arguments_and_invalid_environment --require-test target_competition_never_overwrites --require-test file_and_directory_sync_failures_are_not_failed_no_effect_after_publish --require-test exdev_regular_and_symlink_fault_matrix_is_truthful --require-test exdev_directory_and_special_file_are_rejected_before_effect --require-test two_process_source_claim_has_one_winner --require-test cancel_at_every_exdev_transition_preserves_a_valid_copy --regression-target trusted_fs --regression-target source_claim --regression-target mutation_ops --regression-target exdev "${TERSH_H3_MATRIX_ARGS[@]}"
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-03 --cycle 03 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name hardening-add-010-replay --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/run_exact_test.py --lib --name exdev::tests::hardening_add_010_proof_replay_matrix_has_exact_cases --serial --case-matrix hardening-add-010-proof-replay --expect-case other-bundle --expect-case other-revision --expect-case other-edge --expect-case same-token-second-use
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-03 --cycle 03 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name hardening-add-010-lock-liveness --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/run_exact_test.py --lib --name exdev::tests::hardening_add_010_lock_liveness_matrix_has_exact_cases --serial --case-matrix hardening-add-010-lock-liveness --expect-case lock-dropped-before-transition --expect-case snapshot-replaced-while-unlocked
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-03 --cycle 03 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name inherited-add-010-replay --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/run_exact_test.py --lib --name exdev::tests::exdev_transition_rejects_genuine_token_from_other_bundle_revision_or_edge --serial --case-matrix exdev-transition-replay --expect-case other-bundle --expect-case other-revision --expect-case other-edge
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-03 --cycle 03 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name inherited-add-010-single-use --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/run_exact_test.py --lib --name exdev::tests::exdev_consumed_transition_token_cannot_be_used_twice
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-03 --cycle 03 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name native-exdev --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- scripts/test-exdev-native.sh
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-03 --cycle 03 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name prior-gates --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- scripts/hardening/run_prior_gates.sh
```

Expected: all seven gates exit 0 on the declared local roots. The two hardening-specific private gates, inherited three-case verifier replay, and inherited by-value single-use gate are four distinct catalog entries; no integration wrapper substitutes for them. `.github/workflows/ci.yml` exposes required push-bootstrap jobs named `native-exdev-linux` using tmpfs and `native-exdev-macos` using temporary APFS on `codex/evidence/**`; it may also retain future manual dispatch. Local success alone is not cross-platform acceptance.

- [ ] **Step 6: Commit the native-CI candidate before any accepting review**

Stage Cycle 3 files after local RED/GREEN diagnosis. No Wave C or closure report is accepting evidence until after this commit:

```bash
git add tests/hardening_fs_faults.rs tests/exdev.rs scripts/test-exdev-native.sh .github/workflows/ci.yml src/trusted_fs.rs src/source_claim.rs src/mutation_ops.rs src/exdev.rs src/state_root.rs
git diff --exit-code
test -z "$(git ls-files --others --exclude-standard)"
python3 -B -c 'import subprocess; required=set("tests/hardening_fs_faults.rs tests/exdev.rs scripts/test-exdev-native.sh .github/workflows/ci.yml src/exdev.rs".split()); allowed=required|set("src/trusted_fs.rs src/source_claim.rs src/mutation_ops.rs src/state_root.rs".split()); actual=set(subprocess.check_output(["git","diff","--cached","--name-only"], text=True).splitlines()); assert required <= actual <= allowed, (actual, required, allowed)'
git commit -m "test: harden filesystem and EXDEV fault candidate"
TERSH_H3_CANDIDATE="$(git rev-parse HEAD)"
test -z "$(git status --porcelain=v1 --untracked-files=all)"
case "$TERSH_H3_CANDIDATE" in ''|*[!0-9a-f]*) exit 1 ;; esac
test "$(printf '%s' "$TERSH_H3_CANDIDATE" | wc -c | tr -d ' ')" = 40
```

Stop here. The distinct UID-0 operator reviews the committed native helper and
workflow policy, builds and registers a successor exact-tree bundle containing
the root-owned `native-exdev` entrypoint and updated policy rows, and makes its
bundle/runtime/policy registration receipts queryable. This upgrade is mandatory
even if the product fix happened not to change the helper bytes, because the
entrypoint first becomes formal in Cycle 3. No formal hardening-03 command may
bind an older bundle. After registration the supervisor reserves a new attempt
for this candidate, provisions and session-binds the closed distinct-filesystem
capability above, and renders the following argv template. The operator shell
does not export either root:

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_cumulative_gates.py --catalog SUPERVISOR_HARNESS_ROOT/scripts/hardening/gate_catalog.json --through hardening-03 --attempt "$TERSH_H3_ATTEMPT" --candidate "$TERSH_H3_CANDIDATE" --output-root target/hardening --host-store-fd FRESH_HOST_FD
```

Expected: the candidate commit contains no evidence JSON and is the only SHA eligible for Cycle 3 native CI. The `hardening-03` catalog prefix reruns `focused-fs-faults`, both hardening-specific crate-unit ADD-010 gates, both inherited ADD-010 gates, zero-argument `native-exdev`, and `prior-gates` as separate clean-candidate records. Its native capability check rejects missing/equal-device roots before execution and the accepting attempt cannot reuse any dirty diagnostic record.

- [ ] **Step 7: Run both Tier-1 native EXDEV jobs on the exact candidate**

An authorized CI operator uses the same never-used three-digit formal attempt
already reserved by this clean candidate's cumulative local run. Diagnostics do
not allocate host attempts, but the earlier host-sealed Wave A/B records and the
mandatory bundle upgrade already consumed their own attempts; this step never
assumes `001`. Increment only after a host-bound formal failure or candidate
change. The operator runs the
registered external producer directly, never beneath the gate recorder:

```text
TERSH_H3_CANDIDATE="$(git rev-parse HEAD)"
: "${TERSH_H3_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
case "$TERSH_H3_ATTEMPT" in 00[1-9]|0[1-9][0-9]|[1-9][0-9][0-9]) ;; *) exit 1 ;; esac
test -z "$(git status --porcelain=v1 --untracked-files=all)"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/run_external_candidate.py --evidence-id hardening-03 --attempt "$TERSH_H3_ATTEMPT" --candidate "$TERSH_H3_CANDIDATE" --repository QiushanHuang/Tersh --remote origin --push-ref "codex/evidence/hardening-03/attempt-$TERSH_H3_ATTEMPT/$TERSH_H3_CANDIDATE" --output-root target/hardening-external \
  --workflow ci=.github/workflows/ci.yml --require-job ci=quality-stable --require-job ci=msrv-1-88 --require-job ci=policy --require-job ci=native-exdev-linux --require-job ci=native-exdev-macos \
  --require-online-label ci=tersh-macos-14.5-23F79-arm64 --require-online-label ci=tersh-almalinux-8.10-kernel-4.18-x86_64 --artifacts ci=all \
  --require-artifact 'ci=native-exdev-linux:native-exdev-linux-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-native-exdev-evidence-v1' \
  --require-artifact 'ci=native-exdev-macos:native-exdev-macos-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-native-exdev-evidence-v1' --reject-extra-artifacts ci \
  --registration-timeout-seconds 180 --completion-timeout-seconds ci=5400 --overall-timeout-seconds 14400 --poll-seconds 5 --host-store-fd FRESH_HOST_FD
```

Expected: before any push the helper proves both exact native labels online, snapshots CI runs, and proves the exact `codex/evidence/**` ref absent. It then performs one no-force push of exactly `TERSH_H3_CANDIDATE`; the workflow validates `github.sha`, and the helper accepts only one new numeric `event=push` run with the exact head, branch, and workflow path. All five cumulative CI jobs are successful/non-skipped; exactly the Linux and macOS native-EXDEV artifact templates exist, each root manifest matches `tersh-native-exdev-evidence-v1`, and any extra artifact fails. The helper writes append-only `tersh-external-candidate-v1` evidence and prints exactly one canonical `tersh-external-candidate-result-v1`; missing authorization, label, registration, job, artifact, or timeout fails after recording evidence. It never uses `workflow_dispatch` for first registration and never selects or cancels a run by branch query. The workflow may retain manual dispatch for future use, but that path is not acceptance evidence here.

- [ ] **Step 8: Rerun local gates and obtain final five-role reports on the committed candidate**

Do not rerun the attempt-001 diagnostic commands. The `hardening-03` cumulative runner has already recorded the exact focused/native/prior matrices under the clean `TERSH_H3_CANDIDATE` attempt, and Step 7 adds its external record under that same attempt. Run Wave C safety and verification concurrently: safety races two processes, identity replacement, corrupt ownership markers, permission changes, live-lock/snapshot replacement, and every EXDEV transition; verification independently runs every externally frozen matrix, the native script on distinct devices, the exact CI record, and prior gates. Then run Closure A product, architecture, and implementation diagnosis concurrently, followed by Closure B safety and verification. All five latest reports bind `TERSH_H3_CANDIDATE` and the same direct gate hashes. Any source change creates a new candidate commit and repeats Steps 7-8; prior reports remain append-only but cannot close the new candidate.

- [ ] **Step 9: Finalize evidence and commit Cycle 3 closure**

Run:

```text
TERSH_H3_CANDIDATE="$(git rev-parse HEAD)"
: "${TERSH_H3_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/finalize_cycle.py --cycle 03 --evidence-id hardening-03 --accepting-attempt "$TERSH_H3_ATTEMPT" --candidate "$TERSH_H3_CANDIDATE" --raw-root target/hardening --external-root target/hardening-external --required-gate focused-fs-faults --required-gate hardening-add-010-replay --required-gate hardening-add-010-lock-liveness --required-gate inherited-add-010-replay --required-gate inherited-add-010-single-use --required-gate native-exdev --required-gate native-exdev-ci --required-gate prior-gates --required-gate cumulative-gates --host-store-fd FRESH_HOST_FD --output docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-03.json
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/commit_and_close.py --manifest docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-03.json --candidate "$TERSH_H3_CANDIDATE" --commit-message "docs: record filesystem hardening evidence" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/verify_formal_lineage.py --manifest docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-03.json --host-store-fd FRESH_HOST_FD
```

Expected: the closure commit changes only Cycle 3 evidence, which names both real filesystems and the exact candidate CI run. No broader EXDEV support is claimed.

### Task 4: Cycle 4 — Crash Discovery, Receipt Reconciliation, Restore, And Orphan Isolation

**Files:**
- Create: `tests/hardening_recovery.rs`
- Modify: `src/recovery.rs` (crate-unit hardening recovery ADD-010 matrices)
- Modify only after a reproduced failure: `src/state_root.rs`, `src/source_claim.rs`, `src/trash.rs`, `src/exdev.rs`, `src/recovery_ui.rs`
- Modify: `tests/trash_receipt.rs`, `tests/recovery_cli.rs`, `tests/recovery_ui.rs`
- Create after all checks pass: `docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-04.json`

- [ ] **Step 1: Capture the existing recovery baseline**

Run:

```bash
mkdir -p target/hardening-diagnostic/cycle-04
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-04 --cycle 04 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name baseline-recovery --allow-failure --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- cargo test --locked --test state_root --test source_claim --test trash_receipt --test recovery_cli --test recovery_ui --test exdev -- --nocapture --test-threads=1
```

Expected: the record includes every existing crash-point, reconciliation, list/restore, corrupt receipt, and competing reconciler result.

- [ ] **Step 2: Run Wave A with three `gpt-5.6-sol` `xhigh` roles**

Product checks that recovery is discoverable by ID, restore conflicts preserve user choice, corrupt state is inspect-only, and no recovery action silently deletes. Architecture checks durable receipt phases, fixed-root enumeration, exclusive bundle claims, publish/source-cleanup ordering, and reconciliation idempotence. Implementation diagnosis enumerates every protocol transition and crash seam for source claim, trash, restore, and EXDEV. Each agent create-new writes its read-only draft only at the shared policy-derived agent-draft path; after the callback digest, the root sealer alone publishes the receipt-backed formal review projection.

- [ ] **Step 3: Add the crash/reconciliation matrix**

Add integration tests `crash_after_every_source_claim_transition_is_discoverable`, `crash_after_every_trash_transition_is_listable_or_quarantined`, `crash_after_every_restore_transition_is_truthful`, `crash_after_every_exdev_transition_preserves_source_or_verified_copy`, `corrupt_truncated_duplicate_and_unknown_receipts_are_inspect_only`, `trash_restore_receipts_reject_forged_raw_path_capabilities`, `complete_recovery_crash_matrix_has_exact_cases`, `complete_recovery_invalid_state_matrix_has_exact_cases`, `adjacent_receipt_mismatch_never_authorizes_cleanup`, `orphan_payload_and_staging_are_isolated_without_delete`, `restore_conflict_skip_preserves_record_and_explicit_restore_to_never_overwrites`, `catalog_claim_rechecks_observed_bundle_identity`, `duplicate_item_across_locations_is_inspect_only`, `two_process_reconcile_has_one_owner`, `different_cwd_startup_finds_every_pending_bundle`, `pagination_memory_is_o_page_size`, `ten_thousand_recovery_records_are_bounded_and_deterministic`, and `repeated_reconcile_is_idempotent`. In `src/recovery.rs`, add crate-unit parameter tests `hardening_recovery_add_010_proof_replay_matrix_has_exact_cases` and `hardening_recovery_add_010_lock_liveness_matrix_has_exact_cases`; only these owning-module tests may construct the private trash/restore proofs and lock-bearing typestates. Fault immediately before and after each durable write, file sync, atomic rename, directory sync, claim, publish, and cleanup boundary; feed crafted serde payloads and genuine verifier-issued tokens from the wrong bundle/revision/edge, plus a consumed token used a second time, into their exact rejection paths.

Run:

```bash
TERSH_H4_MATRIX_ARGS=(
  --matrix hardening-recovery-crash-seams
  --expect-case hardening-recovery-crash-seams:source-claim-before-write --expect-case hardening-recovery-crash-seams:source-claim-after-write --expect-case hardening-recovery-crash-seams:source-claim-after-file-sync --expect-case hardening-recovery-crash-seams:source-claim-after-rename --expect-case hardening-recovery-crash-seams:source-claim-after-directory-sync
  --expect-case hardening-recovery-crash-seams:trash-before-mirror-intent --expect-case hardening-recovery-crash-seams:trash-after-mirror-intent --expect-case hardening-recovery-crash-seams:trash-after-adjacent-replace --expect-case hardening-recovery-crash-seams:trash-after-fixed-confirm
  --expect-case hardening-recovery-crash-seams:restore-before-claim --expect-case hardening-recovery-crash-seams:restore-after-claim --expect-case hardening-recovery-crash-seams:restore-after-publish --expect-case hardening-recovery-crash-seams:restore-before-payload-cleanup --expect-case hardening-recovery-crash-seams:restore-after-payload-cleanup
  --expect-case hardening-recovery-crash-seams:exdev-before-copy --expect-case hardening-recovery-crash-seams:exdev-after-copy --expect-case hardening-recovery-crash-seams:exdev-after-file-sync --expect-case hardening-recovery-crash-seams:exdev-after-publish --expect-case hardening-recovery-crash-seams:exdev-after-directory-sync --expect-case hardening-recovery-crash-seams:exdev-before-source-claim --expect-case hardening-recovery-crash-seams:exdev-after-source-claim --expect-case hardening-recovery-crash-seams:exdev-before-source-delete --expect-case hardening-recovery-crash-seams:exdev-after-source-delete --expect-case hardening-recovery-crash-seams:exdev-after-terminal-sync
  --matrix hardening-recovery-invalid-state
  --expect-case hardening-recovery-invalid-state:corrupt --expect-case hardening-recovery-invalid-state:truncated --expect-case hardening-recovery-invalid-state:duplicate-id --expect-case hardening-recovery-invalid-state:unknown-schema --expect-case hardening-recovery-invalid-state:mirror-mismatch --expect-case hardening-recovery-invalid-state:orphan-payload --expect-case hardening-recovery-invalid-state:orphan-staging --expect-case hardening-recovery-invalid-state:contradictory-location
  --matrix hardening-recovery-add-009-serde
  --expect-case hardening-recovery-add-009-serde:empty-name --expect-case hardening-recovery-add-009-serde:dot-name --expect-case hardening-recovery-add-009-serde:dotdot-name --expect-case hardening-recovery-add-009-serde:slash-name --expect-case hardening-recovery-add-009-serde:nul-name --expect-case hardening-recovery-add-009-serde:padded-base64 --expect-case hardening-recovery-add-009-serde:aliased-base64 --expect-case hardening-recovery-add-009-serde:malformed-base64 --expect-case hardening-recovery-add-009-serde:invalid-path-component
)
python3 -B scripts/hardening/run_exact_tests.py --target hardening_recovery --require-test crash_after_every_source_claim_transition_is_discoverable --require-test crash_after_every_trash_transition_is_listable_or_quarantined --require-test crash_after_every_restore_transition_is_truthful --require-test crash_after_every_exdev_transition_preserves_source_or_verified_copy --require-test corrupt_truncated_duplicate_and_unknown_receipts_are_inspect_only --require-test trash_restore_receipts_reject_forged_raw_path_capabilities --require-test complete_recovery_crash_matrix_has_exact_cases --require-test complete_recovery_invalid_state_matrix_has_exact_cases --require-test adjacent_receipt_mismatch_never_authorizes_cleanup --require-test orphan_payload_and_staging_are_isolated_without_delete --require-test restore_conflict_skip_preserves_record_and_explicit_restore_to_never_overwrites --require-test catalog_claim_rechecks_observed_bundle_identity --require-test duplicate_item_across_locations_is_inspect_only --require-test two_process_reconcile_has_one_owner --require-test different_cwd_startup_finds_every_pending_bundle --require-test pagination_memory_is_o_page_size --require-test ten_thousand_recovery_records_are_bounded_and_deterministic --require-test repeated_reconcile_is_idempotent "${TERSH_H4_MATRIX_ARGS[@]}"
python3 -B scripts/run_exact_test.py --lib --name recovery::tests::hardening_recovery_add_010_proof_replay_matrix_has_exact_cases --serial --case-matrix hardening-recovery-add-010-proof-replay --expect-case other-bundle --expect-case other-revision --expect-case other-edge --expect-case same-token-second-use
python3 -B scripts/run_exact_test.py --lib --name recovery::tests::hardening_recovery_add_010_lock_liveness_matrix_has_exact_cases --serial --case-matrix hardening-recovery-add-010-lock-liveness --expect-case lock-dropped-before-transition --expect-case snapshot-replaced-while-unlocked
```

Expected: all eighteen integration tests and all 41 integration-matrix cases run; the two private `--lib` tests independently execute all four recovery proof-replay and both lock-liveness cases. Missing discovery, destructive handling of ambiguous or mirrored state, forged path capability, replayed genuine proof, an authorizing fact surviving a dropped lock or replaced snapshot, stale-bundle mutation, unbounded pagination, non-idempotence, clobber, or an unverifiable success outcome FAILS. A pass records `no_reproduced_defect`.

- [ ] **Step 4: Apply only the lowest failing recovery-state correction**

The trash/restore genuine-proof matrix uses a compile-fail ownership fixture to prove a verifier-issued token cannot be consumed twice. Its runtime cases prove that a proof token borrows or owns the actual no-follow lock plus the verified receipt snapshot until the consuming transition returns; an observation retained after dropping that lock is never authorizing, and replacing the snapshot while unlocked cannot advance either mirror.

Keep receipts per item and crash-consistent rather than claiming global atomicity. Unknown/corrupt/contradictory state goes to inspect-only quarantine; it never triggers deletion. A conflict defaults to `Skip` and preserves the record; choosing the existing explicit `Restore To` flow creates a new preflight for a user-entered destination. No third implicit conflict mode is introduced. Any cleanup-specific retry creates a new preflight rather than replaying copy/restore.

- [ ] **Step 5: Run focused and prior gates**

Run:

```bash
TERSH_H4_MATRIX_ARGS=(
  --matrix hardening-recovery-crash-seams
  --expect-case hardening-recovery-crash-seams:source-claim-before-write --expect-case hardening-recovery-crash-seams:source-claim-after-write --expect-case hardening-recovery-crash-seams:source-claim-after-file-sync --expect-case hardening-recovery-crash-seams:source-claim-after-rename --expect-case hardening-recovery-crash-seams:source-claim-after-directory-sync
  --expect-case hardening-recovery-crash-seams:trash-before-mirror-intent --expect-case hardening-recovery-crash-seams:trash-after-mirror-intent --expect-case hardening-recovery-crash-seams:trash-after-adjacent-replace --expect-case hardening-recovery-crash-seams:trash-after-fixed-confirm
  --expect-case hardening-recovery-crash-seams:restore-before-claim --expect-case hardening-recovery-crash-seams:restore-after-claim --expect-case hardening-recovery-crash-seams:restore-after-publish --expect-case hardening-recovery-crash-seams:restore-before-payload-cleanup --expect-case hardening-recovery-crash-seams:restore-after-payload-cleanup
  --expect-case hardening-recovery-crash-seams:exdev-before-copy --expect-case hardening-recovery-crash-seams:exdev-after-copy --expect-case hardening-recovery-crash-seams:exdev-after-file-sync --expect-case hardening-recovery-crash-seams:exdev-after-publish --expect-case hardening-recovery-crash-seams:exdev-after-directory-sync --expect-case hardening-recovery-crash-seams:exdev-before-source-claim --expect-case hardening-recovery-crash-seams:exdev-after-source-claim --expect-case hardening-recovery-crash-seams:exdev-before-source-delete --expect-case hardening-recovery-crash-seams:exdev-after-source-delete --expect-case hardening-recovery-crash-seams:exdev-after-terminal-sync
  --matrix hardening-recovery-invalid-state
  --expect-case hardening-recovery-invalid-state:corrupt --expect-case hardening-recovery-invalid-state:truncated --expect-case hardening-recovery-invalid-state:duplicate-id --expect-case hardening-recovery-invalid-state:unknown-schema --expect-case hardening-recovery-invalid-state:mirror-mismatch --expect-case hardening-recovery-invalid-state:orphan-payload --expect-case hardening-recovery-invalid-state:orphan-staging --expect-case hardening-recovery-invalid-state:contradictory-location
  --matrix hardening-recovery-add-009-serde
  --expect-case hardening-recovery-add-009-serde:empty-name --expect-case hardening-recovery-add-009-serde:dot-name --expect-case hardening-recovery-add-009-serde:dotdot-name --expect-case hardening-recovery-add-009-serde:slash-name --expect-case hardening-recovery-add-009-serde:nul-name --expect-case hardening-recovery-add-009-serde:padded-base64 --expect-case hardening-recovery-add-009-serde:aliased-base64 --expect-case hardening-recovery-add-009-serde:malformed-base64 --expect-case hardening-recovery-add-009-serde:invalid-path-component
)
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-04 --cycle 04 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name focused-recovery --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/hardening/run_exact_tests.py --target hardening_recovery --require-test crash_after_every_source_claim_transition_is_discoverable --require-test crash_after_every_trash_transition_is_listable_or_quarantined --require-test crash_after_every_restore_transition_is_truthful --require-test crash_after_every_exdev_transition_preserves_source_or_verified_copy --require-test corrupt_truncated_duplicate_and_unknown_receipts_are_inspect_only --require-test trash_restore_receipts_reject_forged_raw_path_capabilities --require-test complete_recovery_crash_matrix_has_exact_cases --require-test complete_recovery_invalid_state_matrix_has_exact_cases --require-test adjacent_receipt_mismatch_never_authorizes_cleanup --require-test orphan_payload_and_staging_are_isolated_without_delete --require-test restore_conflict_skip_preserves_record_and_explicit_restore_to_never_overwrites --require-test catalog_claim_rechecks_observed_bundle_identity --require-test duplicate_item_across_locations_is_inspect_only --require-test two_process_reconcile_has_one_owner --require-test different_cwd_startup_finds_every_pending_bundle --require-test pagination_memory_is_o_page_size --require-test ten_thousand_recovery_records_are_bounded_and_deterministic --require-test repeated_reconcile_is_idempotent "${TERSH_H4_MATRIX_ARGS[@]}" --regression-target trash_receipt --regression-target recovery_cli --regression-target recovery_ui --regression-target exdev
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-04 --cycle 04 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name hardening-recovery-add-010-replay --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/run_exact_test.py --lib --name recovery::tests::hardening_recovery_add_010_proof_replay_matrix_has_exact_cases --serial --case-matrix hardening-recovery-add-010-proof-replay --expect-case other-bundle --expect-case other-revision --expect-case other-edge --expect-case same-token-second-use
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-04 --cycle 04 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name hardening-recovery-add-010-lock-liveness --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/run_exact_test.py --lib --name recovery::tests::hardening_recovery_add_010_lock_liveness_matrix_has_exact_cases --serial --case-matrix hardening-recovery-add-010-lock-liveness --expect-case lock-dropped-before-transition --expect-case snapshot-replaced-while-unlocked
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-04 --cycle 04 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name prior-gates --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- scripts/hardening/run_prior_gates.sh
```

Expected: all four gates exit 0; the two private ADD-010 gates are individually cataloged, every supported trash item is enumerable/restorable in fault-free conditions, ambiguous state is retained, and all earlier gates pass.

- [ ] **Step 6: Commit the Cycle 4 candidate, rerun gates, and obtain final five-role sign-off**

Run:

```text
git add tests/hardening_recovery.rs tests/trash_receipt.rs tests/recovery_cli.rs tests/recovery_ui.rs src/state_root.rs src/source_claim.rs src/trash.rs src/recovery.rs src/exdev.rs src/recovery_ui.rs
git diff --exit-code
test -z "$(git ls-files --others --exclude-standard)"
python3 -B - <<'PY'
import subprocess
required = {"tests/hardening_recovery.rs", "tests/trash_receipt.rs", "tests/recovery_cli.rs", "tests/recovery_ui.rs", "src/recovery.rs"}
allowed = required | {"src/state_root.rs", "src/source_claim.rs", "src/trash.rs", "src/recovery.rs", "src/exdev.rs", "src/recovery_ui.rs"}
actual = set(subprocess.check_output(["git", "diff", "--cached", "--name-only"], text=True).splitlines())
assert required <= actual <= allowed, (required - actual, actual - allowed)
PY
git commit -m "test: harden crash recovery and orphan isolation"
TERSH_H4_CANDIDATE="$(git rev-parse HEAD)"
test "${#TERSH_H4_CANDIDATE}" -eq 40
case "$TERSH_H4_CANDIDATE" in ''|*[!0-9a-f]*) exit 1 ;; esac
test -z "$(git status --porcelain=v1 --untracked-files=all)"
: "${TERSH_H4_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_cumulative_gates.py --catalog SUPERVISOR_HARNESS_ROOT/scripts/hardening/gate_catalog.json --through hardening-04 --attempt "$TERSH_H4_ATTEMPT" --candidate "$TERSH_H4_CANDIDATE" --output-root target/hardening --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/run_external_candidate.py --evidence-id hardening-04 --attempt "$TERSH_H4_ATTEMPT" --candidate "$TERSH_H4_CANDIDATE" --repository QiushanHuang/Tersh --remote origin --push-ref "codex/evidence/hardening-04/attempt-$TERSH_H4_ATTEMPT/$TERSH_H4_CANDIDATE" --output-root target/hardening-external \
  --workflow ci=.github/workflows/ci.yml --require-job ci=quality-stable --require-job ci=msrv-1-88 --require-job ci=policy --require-job ci=native-exdev-linux --require-job ci=native-exdev-macos \
  --require-online-label ci=tersh-macos-14.5-23F79-arm64 --require-online-label ci=tersh-almalinux-8.10-kernel-4.18-x86_64 --artifacts ci=all \
  --require-artifact 'ci=native-exdev-linux:native-exdev-linux-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-native-exdev-evidence-v1' \
  --require-artifact 'ci=native-exdev-macos:native-exdev-macos-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-native-exdev-evidence-v1' --reject-extra-artifacts ci \
  --registration-timeout-seconds 180 --completion-timeout-seconds ci=5400 --overall-timeout-seconds 14400 --poll-seconds 5 --host-store-fd FRESH_HOST_FD
```

The cumulative runner has rerun every Cycle 1-4 local matrix, including the two separate Cycle 4 `recovery::tests` ADD-010 `--lib` records, and `native-exdev-ci` has rerun the inherited Cycle 3 two-runner floor at the Cycle 4 candidate. Wave C safety independently kills/restarts at every durable boundary and attacks corrupt/orphan/duplicate receipts, dropped-lock and replaced-snapshot authorization, plus two-process races; verification recreates the state root from a different cwd, runs focused/prior gates, checks both private ADD-010 records, checks the exact native artifact pair, checks 10,000-record bounds, and confirms no recovery test or frozen matrix ran zero cases. Then run Closure A product, architecture, and implementation diagnosis, followed by Closure B safety and verification. All five latest reports and their direct gate hashes name `TERSH_H4_CANDIDATE`. Any source correction creates a new candidate commit and repeats Step 6; prior attempts remain append-only but cannot close the new candidate.

- [ ] **Step 7: Finalize evidence and commit Cycle 4**

Run:

```text
TERSH_H4_CANDIDATE="$(git rev-parse HEAD)"
: "${TERSH_H4_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/finalize_cycle.py --cycle 04 --evidence-id hardening-04 --accepting-attempt "$TERSH_H4_ATTEMPT" --candidate "$TERSH_H4_CANDIDATE" --raw-root target/hardening --external-root target/hardening-external --required-gate focused-recovery --required-gate hardening-recovery-add-010-replay --required-gate hardening-recovery-add-010-lock-liveness --required-gate native-exdev-ci --required-gate prior-gates --required-gate cumulative-gates --host-store-fd FRESH_HOST_FD --output docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-04.json
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/commit_and_close.py --manifest docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-04.json --candidate "$TERSH_H4_CANDIDATE" --commit-message "docs: record crash recovery hardening evidence" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/verify_formal_lineage.py --manifest docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-04.json --host-store-fd FRESH_HOST_FD
```

Expected: the evidence-only closure commit changes exactly the Cycle 4 manifest. The candidate contains no auto-purge, global ledger, compaction, or cross-OS receipt promise.

### Task 5: Cycle 5 — PTY, Signals, Multiplexers, Narrow Layouts, And Low-Bandwidth Rendering

**Files:**
- Create: `tests/hardening_terminal.rs`
- Create: `scripts/test-terminal-multiplexer.sh`
- Create: `scripts/tests/test_terminal_multiplexer.py`
- Modify: `tests/support/pty.rs`, `tests/render.rs`, `tests/cluster.rs`, `src/cluster_launch.rs` (private proxy matrix), `.github/workflows/ci.yml` (`terminal-multiplexer` jobs only)
- Reuse unchanged: `src/cluster_probe.rs` crate-unit `g3-shutdown` matrix from implementation Iteration 5
- Modify only after a reproduced failure: `src/terminal_session.rs`, `src/app.rs`, `src/ui.rs`, `src/cluster.rs`, `src/cluster_ui.rs`
- Create after all checks pass: `docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-05.json`

- [ ] **Step 1: Capture PTY, signal, layout, and renderer baseline**

Run:

```bash
mkdir -p target/hardening-diagnostic/cycle-05
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-05 --cycle 05 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name baseline-terminal --allow-failure --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- cargo test --locked --test shutdown --test shell_wrapper --test render --test recovery_ui --test cluster_scheduler --test cluster -- --nocapture --test-threads=1
```

Expected: the record preserves exact q/Q/signal/terminal restoration, 40x10/60x16/80x24 rendering, Recovery, operation-report, and Cluster results.

- [ ] **Step 2: Run Wave A with three `gpt-5.6-sol` `xhigh` roles**

Product checks that narrow users can always see mode, primary state/error, cancel/back/help/quit and that low bandwidth does not hide truth. Architecture checks terminal guard ownership, suspend/resume state, process signal mapping, dirty-render invalidation, and cluster child reaping before restoration. Implementation diagnosis exercises raw PTY, tmux, screen, resizes, throttled output, and repeated signals. Each agent create-new writes its read-only draft only at the shared policy-derived agent-draft path; after the callback digest, the root sealer alone publishes the receipt-backed formal review projection.

- [ ] **Step 3: Add deterministic terminal and rendering tests**

Add integration tests `pty_q_shift_q_ctrl_c_term_hup_restore_exactly_once`, `suspend_resume_reenters_raw_and_alternate_screen`, `resize_40x10_60x16_80x24_preserves_survival_controls`, `operation_partial_cleanup_and_indeterminate_fit_narrow`, `recovery_loading_conflict_active_and_failure_fit_narrow`, `cluster_refresh_stopping_and_launch_failure_fit_narrow`, `unchanged_frame_writes_zero_cells`, `single_navigation_under_throttled_sink_acknowledges_within_frozen_gate`, `terminal_write_failure_restores_before_noninteractive_drain`, `remote_proxy_has_one_stdin_owner_and_drains_bounded_streams`, `probe_descendant_pipe_is_bounded_terminated_reaped`, `one_hundred_start_quit_cycles_leave_termios_unchanged`, `complete_terminal_outcome_matrix_has_exact_cases`, and `complete_terminal_layout_matrix_has_exact_cases`. Add crate-unit parameter test `cluster_launch::tests::hardening_proxy_lifecycle_matrix_has_exact_cases` for the private accepted-READY/proxy lifecycle, and retain the inherited `cluster_probe::tests::g3_timeout_and_quit_term_wait_kill_reap_and_join` for the private reader-panic/signal-fault shutdown lifecycle. Do not expose either fixture seam to integration tests.

Create `scripts/test-terminal-multiplexer.sh` so its only accepted first argument is the literal `tmux` or `screen`. It starts a private session, runs the built Tersh PTY smoke, sends q/Q/Ctrl+C/SIGTERM/SIGHUP and resize sequences, captures exit/stdout/termios/cursor/alternate-screen facts plus `tmux -V` or `screen --version`, kills the private session, and exits nonzero on a missing binary or any mismatch. Add Python tests proving missing tools and any other argument fail closed and arguments cannot escape into a shell. CI enables the shared `codex/evidence/**` push-bootstrap contract, may retain future manual dispatch, runs both modes in jobs named `terminal-multiplexer-linux` and `terminal-multiplexer-macos`, and uploads canonical per-job artifacts containing the installed versions plus PTY/signal/termios/cursor/alternate-screen facts; no version is silently inferred as supported merely because a package manager installed it.

Run:

```bash
python3 -B -m unittest scripts.tests.test_terminal_multiplexer -v
TERSH_H5_MATRIX_ARGS=(
  --matrix hardening-terminal-outcomes
  --expect-case hardening-terminal-outcomes:q --expect-case hardening-terminal-outcomes:shift-q --expect-case hardening-terminal-outcomes:ctrl-c --expect-case hardening-terminal-outcomes:sigterm --expect-case hardening-terminal-outcomes:sighup --expect-case hardening-terminal-outcomes:write-failure --expect-case hardening-terminal-outcomes:restore-failure --expect-case hardening-terminal-outcomes:panic
  --matrix hardening-terminal-layouts
  --expect-case hardening-terminal-layouts:40x10 --expect-case hardening-terminal-layouts:60x16 --expect-case hardening-terminal-layouts:80x24
  --matrix g3-sweeps
  --expect-case g3-sweeps:hosts-1 --expect-case g3-sweeps:hosts-16 --expect-case g3-sweeps:hosts-17 --expect-case g3-sweeps:hosts-40
  --matrix g3-process-count
  --expect-case g3-process-count:live-1 --expect-case g3-process-count:live-16 --expect-case g3-process-count:refill-17 --expect-case g3-process-count:refill-40
  --matrix g3-refresh
  --expect-case g3-refresh:one-followup --expect-case g3-refresh:latest-followup-wins --expect-case g3-refresh:late-token --expect-case g3-refresh:late-generation
)
python3 -B scripts/hardening/run_exact_tests.py --target hardening_terminal --require-test pty_q_shift_q_ctrl_c_term_hup_restore_exactly_once --require-test suspend_resume_reenters_raw_and_alternate_screen --require-test resize_40x10_60x16_80x24_preserves_survival_controls --require-test operation_partial_cleanup_and_indeterminate_fit_narrow --require-test recovery_loading_conflict_active_and_failure_fit_narrow --require-test cluster_refresh_stopping_and_launch_failure_fit_narrow --require-test unchanged_frame_writes_zero_cells --require-test single_navigation_under_throttled_sink_acknowledges_within_frozen_gate --require-test terminal_write_failure_restores_before_noninteractive_drain --require-test remote_proxy_has_one_stdin_owner_and_drains_bounded_streams --require-test probe_descendant_pipe_is_bounded_terminated_reaped --require-test one_hundred_start_quit_cycles_leave_termios_unchanged --require-test complete_terminal_outcome_matrix_has_exact_cases --require-test complete_terminal_layout_matrix_has_exact_cases --regression-target cluster_scheduler --regression-target cluster "${TERSH_H5_MATRIX_ARGS[@]}"
python3 -B scripts/run_exact_test.py --lib --name cluster_launch::tests::hardening_proxy_lifecycle_matrix_has_exact_cases --serial --case-matrix hardening-proxy-lifecycle --expect-case ready --expect-case pre-ready-timeout --expect-case malformed-ready --expect-case user-interrupt --expect-case reader-eof --expect-case reader-panic --expect-case descendant-pipe
python3 -B scripts/run_exact_test.py --lib --name cluster_probe::tests::g3_timeout_and_quit_term_wait_kill_reap_and_join --serial --case-matrix g3-shutdown --expect-case queued-quit --expect-case active-quit-term --expect-case active-quit-kill --expect-case timeout-term --expect-case timeout-kill --expect-case grandchild-pipe --expect-case reader-eof --expect-case reader-panic
python3 -B scripts/run_exact_test.py --lib --name cluster_launch::tests::g3_launch_frame_and_child_outcome_matrix --serial --case-matrix g3-launch --expect-case ready-valid --expect-case ready-malformed --expect-case ready-oversize --expect-case ready-timeout --expect-case source-pair-unknown --expect-case exit-0 --expect-case exit-2 --expect-case exit-127 --expect-case exit-129 --expect-case exit-130 --expect-case exit-143 --expect-case exit-255 --expect-case local-signal
```

Expected: all fourteen named hardening tests and all 23 externally frozen terminal/public-G3 matrix cases run in the focused integration wrapper. Three separate private `--lib` tests run all seven proxy, eight `g3-shutdown`, and thirteen `g3-launch` cases through their owning lifecycles. Hidden controls, terminal drift, redraw on unchanged state, slow-input breach, signal misclassification, proxy/descendant leak, cluster lifecycle drift, or multiplexer skip FAILS. A fully passing matrix records `no_reproduced_defect`.

- [ ] **Step 4: Apply the smallest terminal/UI correction**

Change only the failing terminal guard, signal transition, dirty-render condition, or compact renderer. Preserve immediate restoration on terminal failure, mutation drain after restoration attempt, bounded escaped details, and no fake percentage. Do not add a new top-level Operation Center or periodic cluster surface.

- [ ] **Step 5: Run focused, multiplexer, and prior gates**

Run:

```bash
TERSH_H5_MATRIX_ARGS=(
  --matrix hardening-terminal-outcomes
  --expect-case hardening-terminal-outcomes:q --expect-case hardening-terminal-outcomes:shift-q --expect-case hardening-terminal-outcomes:ctrl-c --expect-case hardening-terminal-outcomes:sigterm --expect-case hardening-terminal-outcomes:sighup --expect-case hardening-terminal-outcomes:write-failure --expect-case hardening-terminal-outcomes:restore-failure --expect-case hardening-terminal-outcomes:panic
  --matrix hardening-terminal-layouts
  --expect-case hardening-terminal-layouts:40x10 --expect-case hardening-terminal-layouts:60x16 --expect-case hardening-terminal-layouts:80x24
  --matrix g3-sweeps
  --expect-case g3-sweeps:hosts-1 --expect-case g3-sweeps:hosts-16 --expect-case g3-sweeps:hosts-17 --expect-case g3-sweeps:hosts-40
  --matrix g3-process-count
  --expect-case g3-process-count:live-1 --expect-case g3-process-count:live-16 --expect-case g3-process-count:refill-17 --expect-case g3-process-count:refill-40
  --matrix g3-refresh
  --expect-case g3-refresh:one-followup --expect-case g3-refresh:latest-followup-wins --expect-case g3-refresh:late-token --expect-case g3-refresh:late-generation
)
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-05 --cycle 05 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name focused-terminal --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/hardening/run_exact_tests.py --target hardening_terminal --require-test pty_q_shift_q_ctrl_c_term_hup_restore_exactly_once --require-test suspend_resume_reenters_raw_and_alternate_screen --require-test resize_40x10_60x16_80x24_preserves_survival_controls --require-test operation_partial_cleanup_and_indeterminate_fit_narrow --require-test recovery_loading_conflict_active_and_failure_fit_narrow --require-test cluster_refresh_stopping_and_launch_failure_fit_narrow --require-test unchanged_frame_writes_zero_cells --require-test single_navigation_under_throttled_sink_acknowledges_within_frozen_gate --require-test terminal_write_failure_restores_before_noninteractive_drain --require-test remote_proxy_has_one_stdin_owner_and_drains_bounded_streams --require-test probe_descendant_pipe_is_bounded_terminated_reaped --require-test one_hundred_start_quit_cycles_leave_termios_unchanged --require-test complete_terminal_outcome_matrix_has_exact_cases --require-test complete_terminal_layout_matrix_has_exact_cases "${TERSH_H5_MATRIX_ARGS[@]}" --regression-target shutdown --regression-target render --regression-target recovery_ui --regression-target cluster_scheduler --regression-target cluster
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-05 --cycle 05 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name focused-proxy-lifecycle --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/run_exact_test.py --lib --name cluster_launch::tests::hardening_proxy_lifecycle_matrix_has_exact_cases --serial --case-matrix hardening-proxy-lifecycle --expect-case ready --expect-case pre-ready-timeout --expect-case malformed-ready --expect-case user-interrupt --expect-case reader-eof --expect-case reader-panic --expect-case descendant-pipe
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-05 --cycle 05 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name inherited-g3-shutdown --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/run_exact_test.py --lib --name cluster_probe::tests::g3_timeout_and_quit_term_wait_kill_reap_and_join --serial --case-matrix g3-shutdown --expect-case queued-quit --expect-case active-quit-term --expect-case active-quit-kill --expect-case timeout-term --expect-case timeout-kill --expect-case grandchild-pipe --expect-case reader-eof --expect-case reader-panic
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-05 --cycle 05 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name inherited-g3-launch --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/run_exact_test.py --lib --name cluster_launch::tests::g3_launch_frame_and_child_outcome_matrix --serial --case-matrix g3-launch --expect-case ready-valid --expect-case ready-malformed --expect-case ready-oversize --expect-case ready-timeout --expect-case source-pair-unknown --expect-case exit-0 --expect-case exit-2 --expect-case exit-127 --expect-case exit-129 --expect-case exit-130 --expect-case exit-143 --expect-case exit-255 --expect-case local-signal
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-05 --cycle 05 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name tmux-terminal --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- scripts/test-terminal-multiplexer.sh tmux
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-05 --cycle 05 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name screen-terminal --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- scripts/test-terminal-multiplexer.sh screen
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-05 --cycle 05 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name prior-gates --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- scripts/hardening/run_prior_gates.sh
```

Expected: all seven local gates (`focused-terminal`, `focused-proxy-lifecycle`, `inherited-g3-shutdown`, `inherited-g3-launch`, both multiplexers, and `prior-gates`) exit 0 on the declared hardening host. Cross-platform acceptance still requires the two named CI jobs; no unavailable-tool result is accepted.

- [ ] **Step 6: Commit the multiplexer-CI candidate**

Stage only the declared Cycle 5 paths and create the candidate before any acceptance review:

```bash
git add tests/hardening_terminal.rs scripts/test-terminal-multiplexer.sh scripts/tests/test_terminal_multiplexer.py tests/support/pty.rs tests/render.rs tests/cluster.rs .github/workflows/ci.yml src/cluster_launch.rs src/terminal_session.rs src/app.rs src/ui.rs src/cluster.rs src/cluster_ui.rs
git diff --exit-code
test -z "$(git ls-files --others --exclude-standard)"
python3 -B - <<'PY'
import subprocess
required = {"tests/hardening_terminal.rs", "scripts/test-terminal-multiplexer.sh", "scripts/tests/test_terminal_multiplexer.py", "tests/support/pty.rs", "tests/render.rs", "tests/cluster.rs", ".github/workflows/ci.yml", "src/cluster_launch.rs"}
allowed = required | {"src/terminal_session.rs", "src/app.rs", "src/ui.rs", "src/cluster.rs", "src/cluster_ui.rs"}
actual = set(subprocess.check_output(["git", "diff", "--cached", "--name-only"], text=True).splitlines())
assert required <= actual <= allowed, (required - actual, actual - allowed)
PY
git commit -m "test: harden terminal and multiplexer candidate"
TERSH_H5_CANDIDATE="$(git rev-parse HEAD)"
test "${#TERSH_H5_CANDIDATE}" -eq 40
case "$TERSH_H5_CANDIDATE" in ''|*[!0-9a-f]*) exit 1 ;; esac
test -z "$(git status --porcelain=v1 --untracked-files=all)"
```

Stop here. The distinct UID-0 operator independently reviews the committed
multiplexer helper and workflow policy, registers a successor bundle containing
the root-owned `terminal-multiplexer` entrypoint and its two exact policy rows,
and makes its registration receipts queryable. No formal hardening-05 command
may bind the prior bundle. The supervisor then reserves a new attempt for this
candidate and renders:

```text
: "${TERSH_H5_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_cumulative_gates.py --catalog SUPERVISOR_HARNESS_ROOT/scripts/hardening/gate_catalog.json --through hardening-05 --attempt "$TERSH_H5_ATTEMPT" --candidate "$TERSH_H5_CANDIDATE" --output-root target/hardening --host-store-fd FRESH_HOST_FD
```

Expected: the clean committed candidate contains no Cycle 5 evidence. The cumulative catalog individually reruns the focused integration gate, all three private crate-unit lifecycle gates, and the two root-bundle tmux/screen gates. A source correction always creates a new candidate commit; no review of staged bytes can close a cycle.

- [ ] **Step 7: Run Linux and macOS terminal-multiplexer CI on the exact candidate**

An authorized CI operator uses the same never-used three-digit attempt already reserved by this clean candidate's cumulative local run and invokes the shared bootstrap helper:

```text
TERSH_H5_CANDIDATE="$(git rev-parse HEAD)"
: "${TERSH_H5_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
case "$TERSH_H5_ATTEMPT" in 00[1-9]|0[1-9][0-9]|[1-9][0-9][0-9]) ;; *) exit 1 ;; esac
test -z "$(git status --porcelain=v1 --untracked-files=all)"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/run_external_candidate.py --evidence-id hardening-05 --attempt "$TERSH_H5_ATTEMPT" --candidate "$TERSH_H5_CANDIDATE" --repository QiushanHuang/Tersh --remote origin --push-ref "codex/evidence/hardening-05/attempt-$TERSH_H5_ATTEMPT/$TERSH_H5_CANDIDATE" --output-root target/hardening-external \
  --workflow ci=.github/workflows/ci.yml --require-job ci=quality-stable --require-job ci=msrv-1-88 --require-job ci=policy --require-job ci=native-exdev-linux --require-job ci=native-exdev-macos --require-job ci=terminal-multiplexer-linux --require-job ci=terminal-multiplexer-macos \
  --require-online-label ci=tersh-macos-14.5-23F79-arm64 --require-online-label ci=tersh-almalinux-8.10-kernel-4.18-x86_64 --artifacts ci=all \
  --require-artifact 'ci=native-exdev-linux:native-exdev-linux-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-native-exdev-evidence-v1' \
  --require-artifact 'ci=native-exdev-macos:native-exdev-macos-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-native-exdev-evidence-v1' \
  --require-artifact 'ci=terminal-multiplexer-linux:terminal-multiplexer-linux-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-terminal-multiplexer-evidence-v1' \
  --require-artifact 'ci=terminal-multiplexer-macos:terminal-multiplexer-macos-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-terminal-multiplexer-evidence-v1' --reject-extra-artifacts ci \
  --registration-timeout-seconds 180 --completion-timeout-seconds ci=5400 --overall-timeout-seconds 14400 --poll-seconds 5 --host-store-fd FRESH_HOST_FD
```

Expected: runner inventory succeeds before any mutation; one create-new evidence ref triggers exactly one selected fresh `push` CI run at the candidate. All seven cumulative CI jobs, including both native EXDEV and both multiplexer jobs, are successful/non-skipped; exactly the four frozen CI artifact templates and schemas exist and any extra artifact fails. The append-only external manifest/result contract and every fail-closed registration/completion/cancellation rule are identical to Cycle 3; the first-run path never depends on `workflow_dispatch`.

- [ ] **Step 8: Rerun local gates and obtain final five-role reports**

Do not reuse the attempt-001 diagnostic commands. The `hardening-05` cumulative runner has already replayed the focused integration record, the separate proxy, `g3-shutdown`, and `g3-launch` crate-unit records, and every earlier catalog entry under the clean `TERSH_H5_CANDIDATE` attempt; Step 7 adds the external record under that same attempt. Run Wave C safety and verification concurrently: safety attacks signal ordering, terminal write/restore failures, suspend/resume, resize storms, throttled sinks, proxy reader panic/descendant pipes, and cluster quit with live descendants; verification independently runs raw PTY, both multiplexers, all frozen cases through their owning public or private gate, inspects 40x10/60x16/80x24 snapshots, and checks the bound CI artifact indexes/bodies for exact tmux/screen versions plus PTY/signal/termios/cursor/alternate-screen facts. Then run Closure A product, architecture, and implementation diagnosis concurrently, followed by Closure B safety and verification, against that same candidate and exact CI record. All five latest reports and direct gate hashes must name `TERSH_H5_CANDIDATE`. Any source correction creates a new candidate commit and repeats Steps 7-8; prior attempts remain append-only but cannot close the replacement candidate.

- [ ] **Step 9: Finalize evidence and commit Cycle 5 closure**

Run:

```text
TERSH_H5_CANDIDATE="$(git rev-parse HEAD)"
: "${TERSH_H5_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/finalize_cycle.py --cycle 05 --evidence-id hardening-05 --accepting-attempt "$TERSH_H5_ATTEMPT" --candidate "$TERSH_H5_CANDIDATE" --raw-root target/hardening --external-root target/hardening-external --required-gate focused-terminal --required-gate focused-proxy-lifecycle --required-gate inherited-g3-shutdown --required-gate inherited-g3-launch --required-gate tmux-terminal --required-gate screen-terminal --required-gate terminal-multiplexer-ci --required-gate prior-gates --required-gate cumulative-gates --host-store-fd FRESH_HOST_FD --output docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-05.json
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/commit_and_close.py --manifest docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-05.json --candidate "$TERSH_H5_CANDIDATE" --commit-message "docs: record terminal hardening evidence" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/verify_formal_lineage.py --manifest docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-05.json --host-store-fd FRESH_HOST_FD
```

Expected: the closure commit changes only Cycle 5 evidence. Cycle 5 is evidenced without claiming SIGKILL restoration or a mobile-specific UI.

### Task 6: Cycle 6 — MSRV, Supply Chain, Install Matrix, Artifacts, Checksums, And Rollback

**Files:**
- Create: `tests/hardening_release.rs`
- Create: `scripts/release_rollback.py`
- Create: `scripts/tests/test_release_rollback.py`
- Create: `docs/releases/ROLLBACK.md`
- Modify: `scripts/release_manifest.py`, `scripts/verified-build.sh`, `scripts/install.sh`, `release/release-manifest.schema.json`
- Modify: `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `tests/release_contract.rs`, `tests/release_smoke.rs`
- Modify only after a reproduced documentation mismatch: `README.md`, `CHANGELOG.md`, `src/ui.rs` (help/footer release wording)
- Modify only after a reproduced mismatch: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `build.rs`, `src/build_identity.rs`
- Create after all checks pass: `docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-06.json`

- [ ] **Step 1: Capture locked release and policy baseline**

Run:

```bash
mkdir -p target/hardening-diagnostic/cycle-06
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-06 --cycle 06 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name baseline-release --allow-failure --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- cargo test --locked --test release_contract --test release_smoke -- --nocapture --test-threads=1
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-06 --cycle 06 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name baseline-manifest --allow-failure --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B -m unittest scripts.tests.test_release_manifest -v
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-06 --cycle 06 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name baseline-policy --allow-failure --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- cargo deny check advisories licenses
```

Expected: records expose exact MSRV/stable, advisory/license, workflow pin, target, manifest, asset, smoke, and source-build status without treating workflow presence as native evidence.

- [ ] **Step 2: Run Wave A with three `gpt-5.6-sol` `xhigh` roles**

Product checks install instructions, supported-target labels, stable/unreleased wording, and a recoverable rollback story. Architecture checks source/lock compatibility identity, installation-ID behavior, manifest schema, immutable inputs, and workflow trust boundaries. Implementation diagnosis reruns clean source installs, asset re-downloads, checksums, PTY smokes, MSRV/current stable, advisory/license policy, and action/image pins. Each agent create-new writes its draft only at the shared policy-derived agent-draft path; after the callback digest, the root sealer alone publishes the receipt-backed formal review projection. Wave A makes no release or repository mutation.

- [ ] **Step 3: Add release-hardening and rollback tests**

Add exact Rust tests `all_release_commands_are_locked`, `tier_matrix_has_exact_runner_image_and_os_floors`, `macos_23f79_and_linux_4_18_native_floor_are_exact`, `supported_asset_requires_redownload_checksum_and_native_pty`, `tier2_ready_requires_exact_source_commit_lock_pair`, `official_ready_identity_rejects_dirty_or_environment_only_identity`, `workflow_actions_and_images_are_immutable`, `install_script_rejects_manifest_or_hash_mismatch`, `candidate_mode_cannot_publish_public_release`, `workbench_readme_help_changelog_match_g0a_through_g2`, `g3_evidence_is_not_an_input_to_workbench_release_acceptance`, `complete_release_target_matrix_has_exact_cases`, and `complete_rollback_policy_matrix_has_exact_cases`.

Implement the concrete dry-run command `python3 -B scripts/release_rollback.py plan --current-manifest target/hardening-diagnostic/cycle-06/rollback/current/release-manifest.json --previous-manifest target/hardening-diagnostic/cycle-06/rollback/previous/release-manifest.json --output target/hardening-diagnostic/cycle-06/rollback/plan.json`. It verifies both manifests/assets and emits canonical actions: keep current assets/tag for forensics, remove the bad candidate from latest/promotion, restore the previous verified install link, publish an explicit advisory, and rerun its smoke. The separate `execute` subcommand additionally requires `--confirm-tag` to equal the current manifest's exact tag and `TERSH_ALLOW_RELEASE_ROLLBACK=1`; it may edit release metadata but never delete a tag or asset. Add Python tests `test_plan_is_default_and_never_calls_gh`, `test_manifest_mismatch_fails_closed`, `test_execute_requires_env_and_exact_tag`, `test_execute_uses_argument_vector_not_shell`, and `test_plan_retains_forensic_assets` with a fake `gh` executable.

Do not create a hardening-local release verifier. The candidate check invokes `scripts/implementation_evidence/verify_release_candidate.py`, whose shared tests already require the final manifest to bind the expected clean-checkout source commit/computed Cargo.lock pair, recompute every asset size/SHA-256, bind native smoke to the immutable descriptor, validate exact OS/ABI facts and complete OCI digest references, and reject any publication indication.

Run:

```bash
TERSH_H6_MATRIX_ARGS=(
  --matrix release-targets-v1
  --expect-case release-targets-v1:tier1-macos-arm64 --expect-case release-targets-v1:tier1-linux-x86_64 --expect-case release-targets-v1:tier2-macos-x86_64-source --expect-case release-targets-v1:tier2-linux-arm64-source
  --matrix hardening-rollback-policy
  --expect-case hardening-rollback-policy:plan-default --expect-case hardening-rollback-policy:manifest-mismatch --expect-case hardening-rollback-policy:execute-missing-env --expect-case hardening-rollback-policy:execute-wrong-tag --expect-case hardening-rollback-policy:argv-no-shell --expect-case hardening-rollback-policy:retains-forensic-assets
)
python3 -B scripts/hardening/run_exact_tests.py --target hardening_release --require-test all_release_commands_are_locked --require-test tier_matrix_has_exact_runner_image_and_os_floors --require-test macos_23f79_and_linux_4_18_native_floor_are_exact --require-test supported_asset_requires_redownload_checksum_and_native_pty --require-test tier2_ready_requires_exact_source_commit_lock_pair --require-test official_ready_identity_rejects_dirty_or_environment_only_identity --require-test workflow_actions_and_images_are_immutable --require-test install_script_rejects_manifest_or_hash_mismatch --require-test candidate_mode_cannot_publish_public_release --require-test workbench_readme_help_changelog_match_g0a_through_g2 --require-test g3_evidence_is_not_an_input_to_workbench_release_acceptance --require-test complete_release_target_matrix_has_exact_cases --require-test complete_rollback_policy_matrix_has_exact_cases "${TERSH_H6_MATRIX_ARGS[@]}"
python3 -B -m unittest scripts.tests.test_release_rollback -v
```

Expected: all eighteen named tests plus all ten externally frozen target/rollback cases execute. Floating inputs, mislabeled support, dirty/environment-only official identity, imprecise OS/kernel floors, missing native evidence, checksum/source drift, implicit publish, destructive rollback, false workbench wording, or a G3 dependency in the workbench release contract FAILS. An all-pass baseline records `no_reproduced_defect`.

- [ ] **Step 4: Apply the minimum release correction and document rollback**

Repair only the failing manifest/workflow/install/build-identity/policy contract. Break the smoke/manifest dependency cycle explicitly: build an immutable `AssetDescriptor` with source/lock/toolchain/size/hash, smoke the re-downloaded asset against that descriptor, bind a `SmokeEvidence` record to its hash, and only then assemble/validate the supported final manifest. CI and release workflows accept the unique `codex/evidence/**` push-bootstrap path, verify `github.sha` as the candidate, and force non-publishing artifact-only behavior for that event; a future manual `workflow_dispatch` mode may remain after the workflow is registered on the default branch, but it is not this plan's first-run evidence prerequisite. Keep workbench release wording scoped to G0a-G2 and make the boundary test fail if any G3 evidence path or status becomes an input; this full-sequence hardening cycle does not move the earlier milestone behind G3. `docs/releases/ROLLBACK.md` gives exact detection, freeze, evidence preservation, previous-manifest verification, dry-run, authorized execute, smoke, communication, and reopen criteria. Do not perform a public release or rollback while implementing this task.

- [ ] **Step 5: Run local MSRV, policy, manifest, install, and prior gates**

Run:

```bash
TERSH_H6_MATRIX_ARGS=(
  --matrix release-targets-v1
  --expect-case release-targets-v1:tier1-macos-arm64 --expect-case release-targets-v1:tier1-linux-x86_64 --expect-case release-targets-v1:tier2-macos-x86_64-source --expect-case release-targets-v1:tier2-linux-arm64-source
  --matrix hardening-rollback-policy
  --expect-case hardening-rollback-policy:plan-default --expect-case hardening-rollback-policy:manifest-mismatch --expect-case hardening-rollback-policy:execute-missing-env --expect-case hardening-rollback-policy:execute-wrong-tag --expect-case hardening-rollback-policy:argv-no-shell --expect-case hardening-rollback-policy:retains-forensic-assets
)
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-06 --cycle 06 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name focused-release --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/hardening/run_exact_tests.py --target hardening_release --require-test all_release_commands_are_locked --require-test tier_matrix_has_exact_runner_image_and_os_floors --require-test macos_23f79_and_linux_4_18_native_floor_are_exact --require-test supported_asset_requires_redownload_checksum_and_native_pty --require-test tier2_ready_requires_exact_source_commit_lock_pair --require-test official_ready_identity_rejects_dirty_or_environment_only_identity --require-test workflow_actions_and_images_are_immutable --require-test install_script_rejects_manifest_or_hash_mismatch --require-test candidate_mode_cannot_publish_public_release --require-test workbench_readme_help_changelog_match_g0a_through_g2 --require-test g3_evidence_is_not_an_input_to_workbench_release_acceptance --require-test complete_release_target_matrix_has_exact_cases --require-test complete_rollback_policy_matrix_has_exact_cases "${TERSH_H6_MATRIX_ARGS[@]}" --regression-target release_contract --regression-target release_smoke --regression-target operation --regression-target cli --regression-target shutdown --regression-target plan2_acceptance --regression-target exdev --regression-target trash_receipt --regression-target recovery_cli --regression-target recovery_ui
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-06 --cycle 06 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name manifest-policy --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B -m unittest scripts.tests.test_release_manifest scripts.tests.test_release_rollback scripts.tests.test_implementation_evidence -v
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-06 --cycle 06 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name msrv --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- cargo +1.88.0 test --locked --all-targets
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-06 --cycle 06 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name prior-gates --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- scripts/hardening/run_prior_gates.sh
```

Expected: all four exit 0; `Cargo.lock` is unchanged except an explicitly reviewed dependency correction, every advisory/license exception has ID/reason/expiry, and rollback remains dry-run by default.

- [ ] **Step 6: Commit the release-CI candidate**

Stage only the declared Cycle 6 code/test/workflow/runbook paths and create the candidate before any acceptance review:

```bash
git add tests/hardening_release.rs scripts/release_rollback.py scripts/tests/test_release_rollback.py docs/releases/ROLLBACK.md scripts/release_manifest.py scripts/verified-build.sh scripts/install.sh release/release-manifest.schema.json .github/workflows/ci.yml .github/workflows/release.yml tests/release_contract.rs tests/release_smoke.rs README.md CHANGELOG.md src/ui.rs Cargo.toml Cargo.lock deny.toml build.rs src/build_identity.rs
git diff --exit-code
test -z "$(git ls-files --others --exclude-standard)"
python3 -B - <<'PY'
import subprocess
required = {"tests/hardening_release.rs", "scripts/release_rollback.py", "scripts/tests/test_release_rollback.py", "docs/releases/ROLLBACK.md", "scripts/release_manifest.py", "scripts/verified-build.sh", "scripts/install.sh", "release/release-manifest.schema.json", ".github/workflows/ci.yml", ".github/workflows/release.yml", "tests/release_contract.rs", "tests/release_smoke.rs"}
allowed = required | {"README.md", "CHANGELOG.md", "src/ui.rs", "Cargo.toml", "Cargo.lock", "deny.toml", "build.rs", "src/build_identity.rs"}
actual = set(subprocess.check_output(["git", "diff", "--cached", "--name-only"], text=True).splitlines())
assert required <= actual <= allowed, (required - actual, actual - allowed)
PY
git commit -m "test: harden release candidate and rollback"
TERSH_H6_CANDIDATE="$(git rev-parse HEAD)"
test "${#TERSH_H6_CANDIDATE}" -eq 40
case "$TERSH_H6_CANDIDATE" in ''|*[!0-9a-f]*) exit 1 ;; esac
test -z "$(git status --porcelain=v1 --untracked-files=all)"
```

Stop here because this candidate changes policy-pinned workflows and release
helpers. The distinct UID-0 operator independently reviews those exact blobs,
registers a successor bundle/policy generation, and makes its receipts
queryable. No hardening-06 producer may bind the prior generation. The
supervisor then reserves the candidate's new attempt and renders:

```text
: "${TERSH_H6_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_cumulative_gates.py --catalog SUPERVISOR_HARNESS_ROOT/scripts/hardening/gate_catalog.json --through hardening-06 --attempt "$TERSH_H6_ATTEMPT" --candidate "$TERSH_H6_CANDIDATE" --output-root target/hardening --host-store-fd FRESH_HOST_FD
```

Expected: the committed candidate contains no Cycle 6 evidence and no public release mutation.

- [ ] **Step 7: Produce native CI and non-publishing release matrices at the exact candidate SHA**

An authorized release operator uses the same never-used three-digit attempt already reserved by this clean candidate's cumulative local run and invokes one shared helper process for both workflows, so both before snapshots precede the single push:

```text
TERSH_H6_CANDIDATE="$(git rev-parse HEAD)"
: "${TERSH_H6_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
case "$TERSH_H6_ATTEMPT" in 00[1-9]|0[1-9][0-9]|[1-9][0-9][0-9]) ;; *) exit 1 ;; esac
test -z "$(git status --porcelain=v1 --untracked-files=all)"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/run_external_candidate.py --evidence-id hardening-06 --attempt "$TERSH_H6_ATTEMPT" --candidate "$TERSH_H6_CANDIDATE" --repository QiushanHuang/Tersh --remote origin --push-ref "codex/evidence/hardening-06/attempt-$TERSH_H6_ATTEMPT/$TERSH_H6_CANDIDATE" --output-root target/hardening-external \
  --workflow ci=.github/workflows/ci.yml --workflow release=.github/workflows/release.yml \
  --require-job ci=quality-stable --require-job ci=msrv-1-88 --require-job ci=policy --require-job ci=native-exdev-linux --require-job ci=native-exdev-macos --require-job ci=terminal-multiplexer-linux --require-job ci=terminal-multiplexer-macos \
  --require-job release=tier1-macos-arm64 --require-job release=tier1-linux-x86_64 --require-job release=tier2-macos-x86_64-source --require-job release=tier2-linux-arm64-source --require-job release=install-msrv-1-88 --require-job release=install-current-stable --require-job release=assemble-manifest --require-job release=verify-release-candidate \
  --require-online-label ci=tersh-macos-14.5-23F79-arm64 --require-online-label ci=tersh-almalinux-8.10-kernel-4.18-x86_64 --require-online-label release=tersh-macos-14.5-23F79-arm64 --require-online-label release=tersh-almalinux-8.10-kernel-4.18-x86_64 --require-online-label release=tersh-macos-14.5-23F79-x86_64 --require-online-label release=tersh-almalinux-8.10-kernel-4.18-aarch64 \
  --artifacts ci=all --require-artifact 'ci=native-exdev-linux:native-exdev-linux-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-native-exdev-evidence-v1' --require-artifact 'ci=native-exdev-macos:native-exdev-macos-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-native-exdev-evidence-v1' --require-artifact 'ci=terminal-multiplexer-linux:terminal-multiplexer-linux-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-terminal-multiplexer-evidence-v1' --require-artifact 'ci=terminal-multiplexer-macos:terminal-multiplexer-macos-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-terminal-multiplexer-evidence-v1' --reject-extra-artifacts ci \
  --artifacts release=all --require-artifact 'release=tier1-macos-arm64:tier1-macos-arm64-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-tier1-release-evidence-v1' --require-artifact 'release=tier1-linux-x86_64:tier1-linux-x86_64-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-tier1-release-evidence-v1' --require-artifact 'release=tier2-macos-x86_64-source:tier2-macos-x86_64-source-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-tier2-source-evidence-v1' --require-artifact 'release=tier2-linux-arm64-source:tier2-linux-arm64-source-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-tier2-source-evidence-v1' \
  --require-artifact 'release=install-msrv-1-88:install-msrv-1-88-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-install-evidence-v1' --require-artifact 'release=install-current-stable:install-current-stable-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-install-evidence-v1' --require-artifact 'release=assemble-manifest:release-manifest-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-release-manifest-evidence-v1' --require-artifact 'release=verify-release-candidate:verified-release-candidate-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-release-verification-evidence-v1' --reject-extra-artifacts release \
  --registration-timeout-seconds 180 --completion-timeout-seconds ci=5400 --completion-timeout-seconds release=10800 --overall-timeout-seconds 14400 --poll-seconds 5 --host-store-fd FRESH_HOST_FD
```

Expected: all six workflow-scoped native-label requirements (four unique labels) are observed online before the helper snapshots both workflow run sets and proves the exact ref absent. One no-force push triggers uniquely selected exact-head `push` runs for both paths. The CI run passes all seven cumulative jobs and exactly four frozen artifacts; the release run passes all eight exact jobs and exactly eight frozen artifacts, rejects extras, and the shared verifier proves every root artifact manifest plus descriptor→smoke→final-manifest ordering, hashes, source/lock identity, OS/ABI floors, downloaded-binary READY identity, and non-publication. The combined append-only manifest is PASS only if both workflows, all jobs, and both artifact policies pass; a timeout cancels only already bound numeric run IDs and still records FAIL evidence.

- [ ] **Step 8: Rerun local gates and obtain final five-role sign-off**

Do not reuse the attempt-001 diagnostic commands. The `hardening-06` cumulative runner has already replayed their exact matrices and every earlier matrix under the clean `TERSH_H6_CANDIDATE` attempt; Step 7 adds both external records under that same attempt. Run Wave C safety and verification concurrently: safety attacks poisoned manifest fields, asset/source swaps, dirty builds, dependency drift, action/image drift, skipped jobs, smoke/manifest ordering, and rollback authorization; verification independently rebuilds from immutable source, recomputes SHA-256/size, reruns local PTY smoke, invokes the single shared release verifier, and performs rollback dry-run with fake `gh`. Then run Closure A product, architecture, and implementation diagnosis concurrently, followed by Closure B safety and verification, against that same `TERSH_H6_CANDIDATE` plus both real workflow records/artifacts. Each role separately verifies that release/docs wording preserves the five Workbench Trusted Core Release Acceptance bullets at design lines 1283-1297 as a G0a-G2-only milestone and does not turn G3 into a prerequisite. Any code/workflow/runbook change creates a new candidate commit and repeats Steps 7-8; prior attempts remain append-only but cannot close the new candidate.

- [ ] **Step 9: Finalize evidence and commit Cycle 6 closure**

```text
TERSH_H6_CANDIDATE="$(git rev-parse HEAD)"
: "${TERSH_H6_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/finalize_cycle.py --cycle 06 --evidence-id hardening-06 --accepting-attempt "$TERSH_H6_ATTEMPT" --candidate "$TERSH_H6_CANDIDATE" --raw-root target/hardening --external-root target/hardening-external --required-gate focused-release --required-gate manifest-policy --required-gate msrv --required-gate external-ci-release-candidate --required-gate prior-gates --required-gate cumulative-gates --host-store-fd FRESH_HOST_FD --output docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-06.json
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/commit_and_close.py --manifest docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-06.json --candidate "$TERSH_H6_CANDIDATE" --commit-message "docs: record release hardening evidence" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/verify_formal_lineage.py --manifest docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-06.json --host-store-fd FRESH_HOST_FD
```

Expected: the closure commit changes only Cycle 6 evidence. It proves the exact release candidate and rollback procedure, and proves that current release wording preserves the independent G0a-G2 Workbench milestone without making G3 a prerequisite; it neither publishes a release nor claims the full task complete.

### Task 7: Cycle 7 — Integrated Scale, Soak, Product Contract, And Completion Audit

**Files:**
- Create: `tests/hardening_scale.rs`
- Create: `examples/tersh_hardening_soak.rs`
- Use without modifying: the root-bundle `audit-requirements` entrypoint and its
  registered `expected-requirements.json` policy frozen in Task 1
- Create after the final candidate's external gates pass: the unreceipted
  requirement draft only beneath
  `target/hardening-diagnostic/hardening-07/attempt-NNN/candidate-SHA/completion/`;
  an accepting reservation creates only the two receipt-backed projections
  `requirements-revision-RRR.json` and
  `completion-audit-revision-RRR.json` beneath
  `target/hardening/hardening-07/attempt-NNN/candidate-SHA/completion/`;
  a substantively failed reservation instead creates only the root-receipted
  `audit-reservation-revision-RRR-failed.json` tombstone there and can never
  coexist with a pair for that revision;
  `RRR` must equal each body and host transaction's `audit_revision`, and prior
  revisions coexist create-new;
  the host-only spool and producer receipts are authoritative
- Create only after every audit check passes: `docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-07.json`
- Modify only after evidenced mismatch: `README.md`, `CHANGELOG.md`, `src/ui.rs`
  (help/footer copy), and `src/cluster_ui.rs` (companion copy). A scale failure
  implicating any other source/test path stops the cycle for an independently
  reviewed plan amendment before that path may be edited or staged.

- [ ] **Step 1: Capture integrated baseline and verify Cycles 1-6**

Run:

```bash
mkdir -p target/hardening-diagnostic/cycle-07
for TERSH_H7_CYCLE in 01 02 03 04 05 06; do test "$(python3 -B scripts/hardening/finalize_cycle.py --verify-only docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-${TERSH_H7_CYCLE}.json)" = "STRUCTURAL_PASS/HOST_UNVERIFIED"; done
TERSH_H7_HARDENING_START="$(python3 -B -c 'import json; print(json.load(open("docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-01.json"))["implementation_entry"]["entry_head"])')"
case "$TERSH_H7_HARDENING_START" in ''|*[!0-9a-f]*) exit 1 ;; esac
test "${#TERSH_H7_HARDENING_START}" -eq 40
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-07 --cycle 07 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name baseline-implementation-entry --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/hardening/verify_implementation_entry.py --entry-head "$TERSH_H7_HARDENING_START" --candidate "$(git rev-parse HEAD)" --manifest-root docs/superpowers/evidence/2026-08-10-tersh-implementation --output "target/hardening-diagnostic/hardening-07/attempt-001/candidate-$(git rev-parse HEAD)/run-local/baseline-implementation-entry.json"
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-07 --cycle 07 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name baseline-integrated --allow-failure --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- scripts/hardening/run_prior_gates.sh
```

Expected: all six envelopes pass the offline structural diagnostic and therefore
report `STRUCTURAL_PASS/HOST_UNVERIFIED`; this step does not authenticate their
root origin. Cycle 1's embedded `entry_head` still equals the recorded impl-07
evidence commit. Formal Cycle 7 later queries the registered prior closures
online; any missing/stale cycle, closure, entry mismatch, or baseline regression
blocks acceptance.

- [ ] **Step 2: Run Wave A with three `gpt-5.6-sol` `xhigh` roles**

Product audits both north-star tasks, discoverability, truthful wording, retry/recovery paths, 40x10 survival, and the frozen Cluster boundary. Architecture audits all intent/event/outcome, read-lane, mutation, claim, trash/restore, EXDEV, terminal, remote-launch, and cluster invariants as one system. Implementation diagnosis runs the scale/soak baseline and maps regressions to the smallest owning module. Each agent create-new writes its draft only at the shared policy-derived agent-draft path; after the callback digest, the root sealer alone publishes the receipt-backed formal review projection. No role may infer completion from earlier cycle counts.

- [ ] **Step 3: Add integrated scale and soak evidence**

Add exact tests `ten_thousand_entry_navigation_keeps_bounds`, `ten_thousand_item_batch_has_exact_outcomes_and_report_bound`, `one_gib_copy_cancel_and_complete_preserve_truth`, `repeated_trash_restore_exdev_cycles_leave_no_unowned_state`, `forty_host_refresh_completes_with_real_active_max_sixteen`, `repeated_refresh_launch_quit_reaps_every_child`, `two_process_mutation_and_recovery_races_never_clobber`, and `all_user_visible_statuses_match_terminal_reports`.

Create `examples/tersh_hardening_soak.rs` with exact arguments `--seed`, `--duration-seconds`, `--fixture-root`, `--cluster-hosts`, and `--json-output`. With seed `20260810`, duration `900`, and 40 scripted hosts it repeats scan/preview supersession, large copy/cancel, trash/restore, EXDEV cleanup, terminal start/quit, cluster refresh/quit, and recovery enumeration. It emits raw iteration counts, key latency, stalls, CPU/RSS, copy throughput, cancel timings, queue maxima, stale applications, source-loss count, mismatched outcomes, unreaped children, and unowned receipts. It exits nonzero on any deterministic invariant or the frozen 100 ms event-loop gate; other performance fields are reported without a newly invented threshold.

Run:

```bash
python3 -B scripts/hardening/run_exact_tests.py --target hardening_scale --require-test ten_thousand_entry_navigation_keeps_bounds --require-test ten_thousand_item_batch_has_exact_outcomes_and_report_bound --require-test one_gib_copy_cancel_and_complete_preserve_truth --require-test repeated_trash_restore_exdev_cycles_leave_no_unowned_state --require-test forty_host_refresh_completes_with_real_active_max_sixteen --require-test repeated_refresh_launch_quit_reaps_every_child --require-test two_process_mutation_and_recovery_races_never_clobber --require-test all_user_visible_statuses_match_terminal_reports
TERSH_H7_FIXTURE="$(mktemp -d /tmp/tersh-hardening-07.XXXXXX)"
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-07 --cycle 07 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name diagnostic-integrated-soak --allow-failure --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- cargo run --release --locked --example tersh_hardening_soak -- --seed 20260810 --duration-seconds 900 --fixture-root "$TERSH_H7_FIXTURE" --cluster-hosts 40 --json-output target/hardening-diagnostic/cycle-07/soak.json
```

Expected: all eight tests run. The soak either records a precise failing invariant for repair or exits 0 with stale/source-loss/mismatched-outcome/unreaped-child/unowned-receipt counts all 0, real probe maximum <=16, queue bounds intact, and the frozen latency gate met.

- [ ] **Step 4: Apply only reproduced integrated fixes**

Change the lowest owning module for a reproduced failure and add its exact regression to `hardening_scale.rs`. Do not tune away frozen thresholds, suppress fault cases, increase queue/report bounds, broaden supported operations, or label a partial run complete. If the matrix already passes, record `no_reproduced_defect`.

- [ ] **Step 5: Revalidate the independently frozen requirement policy**

Task 1 already froze `expected-requirements.json` into the registered root-owned
bundle with schema `tersh-trusted-core-expected-requirements-v1`, the exact
SHA-256 of the approved design spec, and exactly 269 top-level normative entries
in these fixed families: `OUT-001..021` (21), `ARCH-001..034` (34),
`G0A-001..014` (14), `G0B-001..014` (14), `G1A-001..010` (10),
`G1B-001..017` (17), `G1C-001..009` (9), `G2-001..012` (12),
`G3-001..010` (10), `UI-001..005` (5), `ERR-001..010` (10),
`SEC-001..009` (9), `TEST-001..037` (37), `ROLL-001..006` (6),
`NONGOAL-001..015` (15), `PROCESS-001..024` (24), `WBACC-001..005` (5),
`FULLACC-001..007` (7), and adversarial addendum `ADD-001..010` (10). The ten
addendum entries bind exact controlling ranges `1324-1357`, `1359-1383`,
`1385-1393`, `1395-1402`, `1404-1419`, `1421-1441`, `1443-1474`,
`1476-1486`, `1488-1497`, and `1499-1505`; each has explicit `subchecks` for
every normative sentence, state edge, and code protocol in its range. Every
other entry likewise binds one top-level bullet/numbered item to an exact
start/end line with explicit subchecks. Cycle 7 must not edit, replace, or load
this policy from the candidate checkout; any policy change requires an
independent bundle registration and a new evidence attempt.

After Step 9 has fixed the final candidate, evidence attempt, and external run
set, Step 10 will author a create-new untrusted requirements draft beneath the fixed diagnostic
attempt/candidate completion tree, never the formal projection tree. Its body uses schema
`tersh-trusted-core-requirements-v1` and contains exactly the same 269 IDs and
design ranges. Each entry adds scope (`workbench`, `cluster`, or `full`), one or
more logical evidence references, exact test/workflow names, status `pass`, and
required reviewer roles. Logical references use only the closed forms
`implementation/impl-0N/manifest`, `hardening/hardening-0N/manifest`,
`hardening/hardening-07/attempt/NNN/gate/NAME`, and
`hardening/hardening-07/attempt/NNN/external/KIND/RUN_ID/OBJECT`; every
component is grammar-checked. Final Cycle 7 review files are not direct
requirement references: their role set is a closure constraint resolved by the
later finalizer. The draft is input to the trusted validator, never authority.
It permits no pathname reference, `target`, `raw`, `file:`, dot component,
symlink, glob, arbitrary URI, `waived`, `unknown`, `not_applicable`, merged ID,
extra ID, or cycle count standing in for direct evidence.

The trusted validator canonicalizes that draft into the closed body
`{schema,evidence_id,evidence_attempt,candidate,audit_revision,expected_requirements_sha256,entries}`
with schema `tersh-trusted-core-requirements-v1`. Its completion audit is the
closed body
`{schema,evidence_id,evidence_attempt,candidate,audit_revision,bundle_id,runtime_profile_id,policy_sha256,requirements,resolver_version,prior_envelopes,current_objects,required_final_roles,requires_cycle_07_manifest}`
with schema `tersh-trusted-core-completion-audit-v1`. `requirements` is exactly
`{body,sha256}`. Every `prior_envelopes` member is exactly
`{evidence_id,envelope,sha256}`; every
`current_objects` member is exactly
`{logical_reference,record_schema,body,sha256,producer_receipt:{body,sha256}}`.
The lists are canonically ordered and duplicate-free; `required_final_roles` is
exactly `product,architecture,implementation,safety,verification`; and the last
field is literal true. The audit does not contain its own hash, producer receipt,
the later package digest, Cycle 7 envelope, or closure, so the chain has no
self-reference.

The registered `audit-requirements` entrypoint has two modes. Formal
`prepare-current` accepts only current evidence ID, three-character attempt,
candidate, a three-character audit revision, the one fixed draft projection,
and a supervisor FD. The spec hash, expected catalog, resolver grammar, prior
manifest set, required five roles, and output destinations come only from the
bundle policy. Revision and draft path are assertions against the current
`reserve-audit-draft` result. The create branch requires it to be unconsumed; an
exact post-COMMIT lost-reply retry may name the same already-paired reservation
and replay only its frozen pair/result. Wrong, missing, skipped, differently
paired, or caller-invented values fail before any body is streamed. It
enumerates the complete current host ledger, streams the
host-only spool, resolves every logical reference, and structurally validates
every prior envelope and embedded preimage. It then create-new spools both the canonical
requirements body and a completion-audit body, appending producer receipts for
each. The audit embeds every referenced canonical body/hash, requires a later
Cycle 7 manifest, and persists no resolved filesystem path. A closed stdout
result returns only the two body digests, their ordered package digest, audit
revision, and projection destinations; it is not a receipt selector. Pair COMMIT
atomically marks the reservation paired. An invalid unreceipted draft leaves the
reservation unconsumed and may be no-follow deleted/re-created only at that same
diagnostic path; it cannot allocate a later revision until the pair succeeds.
If the registered auditor instead proves a substantive gate/evidence/product
defect requiring a candidate change, it emits a closed non-PASS diagnostic to
the Host callback and performs no pair COMMIT. The Host atomically creates the
single-use failure authority and returns only its opaque ID to the UID-0
supervisor; the auditor CLI has empty stdout. The supervisor may then invoke
only the shared internal `fail-audit-reservation` transition with that authority
ID and no caller diagnostic/session/path selector. Its receipted tombstone permanently fails the attempt/revision and is
embedded by all later finalizers; only after that atomic transition may the next
candidate attempt open. No caller can abandon a reservation, erase the failure,
or select an older audit pair. Formal finalization later enumerates all
revisions and accepts exactly the pair on the current maximum accepting attempt,
whose ordered package digest is bound by all five final reviews. A prior failed
attempt remains mandatory history but can never supply that pair. The
final online acceptance evaluator separately queries the fixed impl-01..07 and
hardening-01..07 registry keys and matches every detached closure; neither the
audit draft nor a caller supplies that set or any receipt ID.
Offline `verify-only` accepts only the finalized Cycle 7 envelope, rehashes the
embedded requirements/audit bodies and logical joins, reads no mutable source,
and emits `STRUCTURAL_PASS/HOST_UNVERIFIED` rather than formal PASS.

Rerun the exact Python tests frozen in Task 1:
`test_expected_catalog_has_fixed_269_ids_and_family_counts`,
`test_addendum_has_exact_ten_controlling_ranges`,
`test_catalog_spec_hash_or_line_drift_fails`,
`test_missing_normative_requirement_fails`,
`test_duplicate_or_bad_line_range_fails`,
`test_missing_test_or_workflow_name_fails`,
`test_waived_unknown_or_glob_evidence_fails`,
`test_requirement_rejects_target_raw_absolute_relative_or_file_reference`,
`test_logical_reference_rejects_unknown_kind_or_component`,
`test_cycle_count_without_direct_evidence_fails`,
`test_prepare_current_enumerates_full_host_ledger_and_spools_two_receipted_records`,
`test_prepare_current_never_persists_resolved_paths`,
`test_audit_revision_is_create_new_and_preserves_prior_records`,
`test_audit_revision_two_uses_distinct_receipted_paths_without_overwrite`,
`test_audit_draft_reservation_is_host_allocated_idempotent_and_blocks_unpaired_or_skipped_revision`,
`test_substantive_audit_failure_is_receipted_and_allows_only_next_attempt`,
`test_cycle7_review_v2_requires_same_latest_audit_revision_and_package_digest_for_wave_c_and_five_roles`,
`test_verify_only_requires_embedded_matching_cycle07_audit`,
`test_verify_only_rejects_embedded_body_or_hash_swap`,
`test_verify_only_emits_structural_host_unverified_after_projections_are_removed`,
`test_unresolved_p0_p1_fails`, and
`test_complete_fixture_emits_canonical_audit`.

Run: `python3 -B -m unittest scripts.tests.test_requirement_audit -v`

Expected: the full named validator target passes; negative fixtures exit nonzero with exact missing/drifted IDs, forged logical references, or embedded body/hash mismatches. Product, architecture, safety, implementation, and verification roles must each inspect the expected catalog against the spec and controlling addendum; agreement among the mapping and tool alone is not completeness evidence.

- [ ] **Step 6: Run focused soak, complete prior gates, and documentation truth tests**

Add Rust tests `readme_help_footer_changelog_match_workbench_capabilities`, `cluster_copy_remains_companion_scoped`, `no_document_claims_unsupported_overwrite_recursive_delete_or_directory_exdev`, and `completion_wording_requires_passing_audit` to `tests/hardening_scale.rs`.

Run:

```bash
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-07 --cycle 07 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name focused-scale --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/hardening/run_exact_tests.py --target hardening_scale --require-test ten_thousand_entry_navigation_keeps_bounds --require-test ten_thousand_item_batch_has_exact_outcomes_and_report_bound --require-test one_gib_copy_cancel_and_complete_preserve_truth --require-test repeated_trash_restore_exdev_cycles_leave_no_unowned_state --require-test forty_host_refresh_completes_with_real_active_max_sixteen --require-test repeated_refresh_launch_quit_reaps_every_child --require-test two_process_mutation_and_recovery_races_never_clobber --require-test all_user_visible_statuses_match_terminal_reports --require-test readme_help_footer_changelog_match_workbench_capabilities --require-test cluster_copy_remains_companion_scoped --require-test no_document_claims_unsupported_overwrite_recursive_delete_or_directory_exdev --require-test completion_wording_requires_passing_audit
TERSH_H7_FINAL_FIXTURE="$(mktemp -d /tmp/tersh-hardening-07-final.XXXXXX)"
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-07 --cycle 07 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name integrated-soak --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- cargo run --release --locked --example tersh_hardening_soak -- --seed 20260810 --duration-seconds 900 --fixture-root "$TERSH_H7_FINAL_FIXTURE" --cluster-hosts 40 --json-output target/hardening-diagnostic/cycle-07/soak-final.json
TERSH_H7_HARDENING_START="$(python3 -B -c 'import json; print(json.load(open("docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-01.json"))["implementation_entry"]["entry_head"])')"
case "$TERSH_H7_HARDENING_START" in ''|*[!0-9a-f]*) exit 1 ;; esac
test "${#TERSH_H7_HARDENING_START}" -eq 40
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-07 --cycle 07 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name implementation-entry --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- python3 -B scripts/hardening/verify_implementation_entry.py --entry-head "$TERSH_H7_HARDENING_START" --candidate "$(git rev-parse HEAD)" --manifest-root docs/superpowers/evidence/2026-08-10-tersh-implementation --output "target/hardening-diagnostic/hardening-07/attempt-001/candidate-$(git rev-parse HEAD)/run-local/implementation-entry.json"
python3 -B scripts/hardening/run_gate.py --evidence-id hardening-07 --cycle 07 --attempt 001 --candidate "$(git rev-parse HEAD)" --run-binding run-local --name prior-gates --allow-dirty-diagnostic --output-root target/hardening-diagnostic -- scripts/hardening/run_prior_gates.sh
```

Expected: all four exit 0. The implementation-entry record validates exactly impl-01 through impl-07, their canonical bodies/hashes, evidence-only commits, and ancestry to the recorded hardening start. Earlier native artifacts remain historical evidence only; the final candidate must rerun the complete native CI and release matrices below.

- [ ] **Step 7: Run non-accepting diagnosis and add conditional completion wording**

Before staging or committing, run diagnostic safety plus verification concurrently. Safety composes mutation plus stale read, cancel plus terminal error, crash plus corrupt receipt, EXDEV plus target race, cluster quit plus child-output pressure, and release/source drift. Verification reruns focused scale, the 15-minute soak, all prior gates, Cycles 1-6 validation, and the fixed catalog tests. These reports are advisory only and cannot satisfy Wave C or either closure. Resolve P0/P1 through the single implementation role and repeat both.

Only after that preliminary review passes, update README/help/footer/CHANGELOG
with this fail-closed meaning: completion exists only when the committed
`cycle-07.json` envelope embeds the matching completion audit and an online
root-peer-authenticated query matches its bundle registration, preimage receipt,
and detached post-commit closure receipt. A present file or offline structural
check alone is `HOST_UNVERIFIED` and the candidate remains explicitly
incomplete. Document implemented workbench behavior and G3 companion scope,
but do not state that a public release was published or that explicit non-goals
became supported.

- [ ] **Step 8: Commit the final code/documentation candidate**

Run:

```text
git add tests/hardening_scale.rs examples/tersh_hardening_soak.rs README.md CHANGELOG.md src/ui.rs src/cluster_ui.rs
git diff --exit-code
test -z "$(git ls-files --others --exclude-standard)"
python3 -B - <<'PY'
import subprocess
required = {"tests/hardening_scale.rs", "examples/tersh_hardening_soak.rs"}
allowed = required | {"README.md", "CHANGELOG.md", "src/ui.rs", "src/cluster_ui.rs"}
actual = set(subprocess.check_output(["git", "diff", "--cached", "--name-only"], text=True).splitlines())
assert required <= actual <= allowed, (required - actual, actual - allowed)
PY
git commit -m "test: prepare final trusted core audit candidate"
TERSH_H7_CANDIDATE="$(git rev-parse HEAD)"
test "${#TERSH_H7_CANDIDATE}" -eq 40
case "$TERSH_H7_CANDIDATE" in ''|*[!0-9a-f]*) exit 1 ;; esac
test -z "$(git status --porcelain=v1 --untracked-files=all)"
: "${TERSH_H7_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/run_cumulative_gates.py --catalog SUPERVISOR_HARNESS_ROOT/scripts/hardening/gate_catalog.json --through hardening-07 --attempt "$TERSH_H7_ATTEMPT" --candidate "$TERSH_H7_CANDIDATE" --output-root target/hardening --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/verify_implementation_entry.py --evidence-id hardening-07 --attempt "$TERSH_H7_ATTEMPT" --entry-head SUPERVISOR_POLICY_IMPL07_EVIDENCE_COMMIT --candidate "$TERSH_H7_CANDIDATE" --manifest-root docs/superpowers/evidence/2026-08-10-tersh-implementation --output "target/hardening/hardening-07/attempt-$TERSH_H7_ATTEMPT/candidate-$TERSH_H7_CANDIDATE/run-local/implementation-entry.json" --host-store-fd FRESH_HOST_FD
```

Expected: this is the one final candidate SHA for every remaining
local/native/release gate. The registered implementation-entry policy resolves
and verifies all seven prior lineages host-side, atomically appends the entry
record plus wrapper gate on its own fresh FD, and treats the displayed entry-head
as an exact policy assertion rather than caller authority. No acceptance-policy or evidence-control-plane file
is changed by this candidate. Its wording is conditional and therefore does not
claim completion before the host-closed evidence commit.

- [ ] **Step 9: Run the complete native CI and non-publishing release matrix at the final candidate**

An authorized operator uses the same never-used three-digit attempt already reserved by the final candidate's cumulative local run and runs the shared two-workflow bootstrap:

```text
TERSH_H7_CANDIDATE="$(git rev-parse HEAD)"
: "${TERSH_H7_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
case "$TERSH_H7_ATTEMPT" in 00[1-9]|0[1-9][0-9]|[1-9][0-9][0-9]) ;; *) exit 1 ;; esac
test -z "$(git status --porcelain=v1 --untracked-files=all)"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/run_external_candidate.py --evidence-id hardening-07 --attempt "$TERSH_H7_ATTEMPT" --candidate "$TERSH_H7_CANDIDATE" --repository QiushanHuang/Tersh --remote origin --push-ref "codex/evidence/hardening-07/attempt-$TERSH_H7_ATTEMPT/$TERSH_H7_CANDIDATE" --output-root target/hardening-external \
  --workflow ci=.github/workflows/ci.yml --workflow release=.github/workflows/release.yml \
  --require-job ci=quality-stable --require-job ci=msrv-1-88 --require-job ci=policy --require-job ci=native-exdev-linux --require-job ci=native-exdev-macos --require-job ci=terminal-multiplexer-linux --require-job ci=terminal-multiplexer-macos \
  --require-job release=tier1-macos-arm64 --require-job release=tier1-linux-x86_64 --require-job release=tier2-macos-x86_64-source --require-job release=tier2-linux-arm64-source --require-job release=install-msrv-1-88 --require-job release=install-current-stable --require-job release=assemble-manifest --require-job release=verify-release-candidate \
  --require-online-label ci=tersh-macos-14.5-23F79-arm64 --require-online-label ci=tersh-almalinux-8.10-kernel-4.18-x86_64 --require-online-label release=tersh-macos-14.5-23F79-arm64 --require-online-label release=tersh-almalinux-8.10-kernel-4.18-x86_64 --require-online-label release=tersh-macos-14.5-23F79-x86_64 --require-online-label release=tersh-almalinux-8.10-kernel-4.18-aarch64 \
  --artifacts ci=all --require-artifact 'ci=native-exdev-linux:native-exdev-linux-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-native-exdev-evidence-v1' --require-artifact 'ci=native-exdev-macos:native-exdev-macos-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-native-exdev-evidence-v1' --require-artifact 'ci=terminal-multiplexer-linux:terminal-multiplexer-linux-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-terminal-multiplexer-evidence-v1' --require-artifact 'ci=terminal-multiplexer-macos:terminal-multiplexer-macos-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-terminal-multiplexer-evidence-v1' --reject-extra-artifacts ci \
  --artifacts release=all --require-artifact 'release=tier1-macos-arm64:tier1-macos-arm64-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-tier1-release-evidence-v1' --require-artifact 'release=tier1-linux-x86_64:tier1-linux-x86_64-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-tier1-release-evidence-v1' --require-artifact 'release=tier2-macos-x86_64-source:tier2-macos-x86_64-source-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-tier2-source-evidence-v1' --require-artifact 'release=tier2-linux-arm64-source:tier2-linux-arm64-source-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-tier2-source-evidence-v1' \
  --require-artifact 'release=install-msrv-1-88:install-msrv-1-88-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-install-evidence-v1' --require-artifact 'release=install-current-stable:install-current-stable-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-install-evidence-v1' --require-artifact 'release=assemble-manifest:release-manifest-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-release-manifest-evidence-v1' --require-artifact 'release=verify-release-candidate:verified-release-candidate-{candidate}-run-{run_id}-attempt-{run_attempt}:tersh-release-verification-evidence-v1' --reject-extra-artifacts release \
  --registration-timeout-seconds 180 --completion-timeout-seconds ci=5400 --completion-timeout-seconds release=10800 --overall-timeout-seconds 14400 --poll-seconds 5 --host-store-fd FRESH_HOST_FD
```

Expected: the exact runner inventory, before snapshots, absent ref, single no-force push, unique fresh `event=push` selection, numeric run IDs, cumulative seven CI jobs, eight release jobs, the exact four CI plus eight release artifact templates/root schemas with extras rejected, and non-publication validate under the shared append-only manifest. Any missing/ambiguous registration, timeout, artifact/job drift, or candidate mismatch records FAIL and exits nonzero; there is no inline selector/watch and no first-run dispatch dependency.

- [ ] **Step 10: Rerun local gates and host-seal the non-self-referential completion audit**

The `hardening-07` cumulative runner from Step 8 is authoritative; rerun it
under a new evidence attempt if any accepting local record failed, and use that
same attempt for the external gate and reviews. The untrusted requirements draft
is never staged or a formal record by itself and cannot be authored before its
Host reservation supplies the path. Only now, after Step 9 has completed on the
final current attempt/candidate, the UID-0
supervisor calls the shared internal `reserve-audit-draft` operation. Its closed
result is the sole source of the typed `TERSH_H7_AUDIT_REVISION` and
`TERSH_H7_REQUIREMENTS_DRAFT` slots; a lost result replays the same request ID.
The author then create-new writes exactly the 269 expected IDs and logical
references only at that returned diagnostic path. The caller cannot choose a revision/path, and no later attempt
may open while this reservation is unpaired. A mapping-only retry first corrects
the same unpaired diagnostic draft or, after a pair has committed, obtains the
next contiguous reservation. Then run:

```text
case "$TERSH_H7_AUDIT_REVISION" in 00[1-9]|0[1-9][0-9]|[1-9][0-9][0-9]) ;; *) exit 1 ;; esac
TERSH_H7_COMPLETION_ROOT="target/hardening/hardening-07/attempt-$TERSH_H7_ATTEMPT/candidate-$TERSH_H7_CANDIDATE/completion"
test -f "$TERSH_H7_REQUIREMENTS_DRAFT"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/audit_requirements.py prepare-current --evidence-id hardening-07 --attempt "$TERSH_H7_ATTEMPT" --candidate "$TERSH_H7_CANDIDATE" --audit-revision "$TERSH_H7_AUDIT_REVISION" --requirements-draft "$TERSH_H7_REQUIREMENTS_DRAFT" --projection-root "$TERSH_H7_COMPLETION_ROOT" --host-store-fd FRESH_HOST_FD
test -z "$(git status --porcelain=v1 --untracked-files=all)"
```

Expected: the registered entrypoint loads the spec hash, 269-ID catalog,
resolver, prior-envelope policy, and destination grammar only from the bundle.
It validates Cycles 1-6 plus the current ledger, create-new spools canonical
`requirements` and `completion-audit` records, atomically appends one producer
receipt for each, and emits one closed result containing their SHA-256 values
and the SHA-256 of their canonical ordered digest pair. The completion audit
embeds every referenced body/hash and `requires_cycle_07_manifest: true`; it
does not look for or fabricate `cycle-07.json`. Projection bytes may be reviewed
but never substitute for the host spool/receipts or `TERSH_H7_CANDIDATE`.
If this trusted audit instead identifies a defect requiring code, workflow,
gate, or product repair, do not edit under the reserved attempt and do not leave
an invisible reservation. After the Host callback returns the opaque authority,
the UID-0 supervisor atomically runs `fail-audit-reservation` with only that
authority ID and verifies the
receipted failure projection, then returns to the owning repair step. The next
candidate uses the next host-opened evidence attempt and repeats Steps 8-10;
the failed attempt remains in the ledger and later manifest.

- [ ] **Step 11: Obtain five final reports bound to code, docs, requirements, and audit**

Run Wave C safety and verification concurrently against the exact committed
`TERSH_H7_CANDIDATE` and the ordered package digest returned by the trusted
audit entrypoint. Then run Closure A product, architecture, and implementation
diagnosis concurrently, including a line-by-line external review of the fixed
269-ID policy and all ten addendum ranges, followed by Closure B safety and
verification. Every report binds the same candidate, audit revision, ordered
package digest, and direct gate hashes; safety and verification independently
inspect the two projections, real CI/release bodies, and final soak. The root
sealer rehashes each report and the finalizer requires those reported digests
to equal the host-spooled bodies. Every latest role report must be `PASS`, use
`gpt-5.6-sol` with `xhigh`, and contain no unresolved P0/P1.

If code, user-facing docs, product tests, or workflows change, create a new
candidate commit and repeat external plus local Steps 8-11 under a new evidence
attempt. The registered policy cannot change inside this sequence. If only the
untrusted mapping or audit references change, request the next host reservation,
create its new draft and new spool/receipt pair, append distinct review-attempt
files, and repeat Wave C plus both closure waves. Prior revisions remain in the
ledger and projections and cannot be overwritten or omitted. Do not rerun an
existing gate name in the same evidence attempt. Step 13 commits only the final
Cycle 7 envelope; requirements, audit, raw reports, and receipts are embedded
or detached according to the host-ledger contract, never separately staged.

- [ ] **Step 12: Finalize Cycle 7 and structurally inspect the immutable envelope**

Run:

```text
TERSH_H7_CANDIDATE="$(git rev-parse HEAD)"
: "${TERSH_H7_ATTEMPT:?set a never-used three-digit attempt and increment it on every retry}"
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/finalize_cycle.py --cycle 07 --evidence-id hardening-07 --accepting-attempt "$TERSH_H7_ATTEMPT" --candidate "$TERSH_H7_CANDIDATE" --raw-root target/hardening --external-root target/hardening-external --required-gate focused-scale --required-gate integrated-soak --required-gate implementation-entry --required-gate final-external-candidate --required-gate prior-gates --required-gate cumulative-gates --host-store-fd FRESH_HOST_FD --output docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-07.json
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/audit_requirements.py verify-only --cycle-07-manifest docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-07.json
git diff --check
```

Expected: finalization enumerates every attempt, audit revision, producer receipt,
and spool body; it embeds the uniquely review-bound requirements/audit pair,
obtains the preimage receipt, and publishes one envelope. `verify-only` prints
exactly `STRUCTURAL_PASS/HOST_UNVERIFIED`, rehashes only envelope-embedded
bodies, and performs no `target/`, host-store, or network read. Formal PASS does
not exist until Step 13's detached closure receipt is online-verifiable.

- [ ] **Step 13: Commit the evidence-only completion closure**

Run:

```text
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/implementation_evidence/commit_and_close.py --manifest docs/superpowers/evidence/2026-08-10-tersh-hardening/cycle-07.json --candidate "$TERSH_H7_CANDIDATE" --commit-message "docs: record trusted core completion evidence" --host-store-fd FRESH_HOST_FD
SUPERVISOR_PINNED_PYTHON -I -S -B SUPERVISOR_HARNESS_ROOT/scripts/hardening/verify_fixed14.py --host-store-fd impl-01 FRESH_HOST_FD --host-store-fd impl-02 FRESH_HOST_FD --host-store-fd impl-03 FRESH_HOST_FD --host-store-fd impl-04 FRESH_HOST_FD --host-store-fd impl-05 FRESH_HOST_FD --host-store-fd impl-06 FRESH_HOST_FD --host-store-fd impl-07 FRESH_HOST_FD --host-store-fd hardening-01 FRESH_HOST_FD --host-store-fd hardening-02 FRESH_HOST_FD --host-store-fd hardening-03 FRESH_HOST_FD --host-store-fd hardening-04 FRESH_HOST_FD --host-store-fd hardening-05 FRESH_HOST_FD --host-store-fd hardening-06 FRESH_HOST_FD --host-store-fd hardening-07 FRESH_HOST_FD
```

Expected: the trusted entrypoint proves the index/worktree change set contains
only the single Cycle 7 envelope destination, creates the evidence-only commit with parent exactly
`TERSH_H7_CANDIDATE`, verifies identical blob bytes, and appends or returns the
detached closure receipt. The next line performs that cycle's online root query
as part of the fixed evaluator; it is not optional. The evaluator consumes the
fourteen distinct supervisor injections as fourteen independent transactions—
one each for the fixed manifest set `impl-01..impl-07` and
`cycle-01..cycle-07`. The shared verifier derives selectors from each
envelope/Git state, and the platform may report program `PASS` only when this
exact command emits its closed 14-of-14 formal-PASS result. An old output,
missing/duplicate key, reused FD, missing manifest, or offline structural result
is not counted. The commit
changes no code, workflow, tests, policy, raw audit, or user-facing copy. This
is not authorization to publish a release or perform another external mutation.

## Requirement-To-Cycle Map

| Design requirement | Hardening task | Direct evidence |
| --- | --- | --- |
| Event-loop latency, supersession, queue bounds, CPU, memory (`:1265`) | Task 1 | `hardening_performance`, reference benchmark, Cycle 1 manifest |
| Cancellation, terminal races, worker panic/disconnect, shutdown (`:1266`) | Task 2 | `hardening_shutdown`, exact PTY/exit outcomes, Cycle 2 manifest |
| ENOSPC/EACCES/identity/symlink/target/EXDEV matrix (`:1267`) | Task 3 | injected matrix, two-device native gate, Cycle 3 manifest |
| Crash discovery, receipts, restore, orphan isolation (`:1268-1269`) | Task 4 | transition crash matrix, two-process reconciliation, Cycle 4 manifest |
| PTY/signals/restoration/multiplexers/layout/bandwidth (`:1270-1271`) | Task 5 | raw PTY, tmux/screen, narrow/dirty-render gates, Cycle 5 manifest |
| MSRV/advisory/locked install/artifacts/checksums/rollback (`:1272-1273`) | Task 6 | local policy tests, real non-publishing candidate matrix, rollback dry-run, Cycle 6 manifest |
| Scale/soak/regression/product/docs/final audit (`:1274-1275`) | Task 7 | 15-minute soak, locked regression, five final reviews, completion audit |
| A cycle count alone is not evidence (`:1277-1281`) | Tasks 1-7 | per-cycle focused/prior/native gates plus direct findings and role reports |
| Workbench release acceptance (`:1283-1297`) | Independent implementation milestone; Task 6 rechecks its boundary | G0a-G2 direct evidence and truthful release/docs tests; this full-sequence plan does not add G3 as an input |
| Full task acceptance (`:1299-1316`) | Task 7 only | all slice/cycle evidence, final reviews, truthful docs, zero-gap completion audit |
| Prepared mutation/fence handshake (`:1324-1357`) | Tasks 1, 2, 7 | event-loop/preflight bounds, cancel-before-effect schedules, integrated batch test |
| One claimed fixed control and mirrored receipt authority (`:1359-1383`) | Tasks 3, 4 | source-claim race matrix, receipt transition/crash matrix, inspect-only disagreement |
| `RawUnixName` capability boundary (`:1385-1393`) | Task 3 | table-driven invalid-component and fd-relative no-follow tests |
| Keyed directory/recovery read bounds (`:1395-1402`) | Tasks 1, 4 | keyed supersession/fairness and 10,000-record bounded discovery |
| Exact observed recovery-bundle claim (`:1404-1419`) | Task 4 | identity recheck, duplicate-location quarantine, O(page-size) pagination evidence |
| Drain-before-join mutation/remote/probe lifecycle (`:1421-1441`) | Tasks 2, 5, 7 | full-channel shutdown, proxy stream/child reaping, integrated quit soak |
| Non-circular release/native evidence and zero-test rejection (`:1443-1474`) | Tasks 1, 3, 5, 6, 7 | exact-test records, fresh exact-SHA CI JSON, descriptor/smoke/final-manifest chain, native floor jobs |
| Append-only fourteen-cycle review provenance (`:1476-1486`) | Tasks 1-7 | orchestration cross-check, all attempt reports, five same-hash closure roles, candidate/evidence commit split |
| Raw path/name deserialization cannot forge a filesystem capability (`:1488-1497`) | Tasks 3, 4, 7 | canonical serde rejection matrices for core, trash/restore, and EXDEV receipts |
| Genuine transition proofs are bundle/revision/edge bound, single-use, and non-replayable (`:1499-1505`) | Tasks 3, 4, 7 | cross-bundle/revision/edge replay and same-token second-use rejection with genuine verifier-issued tokens |

## Final Self-Review Checklist

- [ ] Exactly seven hardening tasks exist and correspond one-to-one with design lines 1265-1275; the controlling addendum at lines 1318-1505 is mapped into those same tasks.
- [ ] Every task includes baseline/fault reproduction, the five-phase role protocol, minimal repair, focused gate, all-prior gate, adversarial/independent review, evidence finalization, and commit boundary.
- [ ] Every cycle commits its code/test/workflow candidate before accepting gates or reviews; Wave C and all five latest closure reports bind that exact clean 40-hex SHA, and the following commit contains only the allowed evidence path(s).
- [ ] Every formal gate/external record lives under a validated create-new evidence-ID/attempt/candidate/run binding; each orchestration/review is a candidate-root create-new per-file record with string attempt fields and an exact body `run_binding`. Retries increment attempts, finalization embeds every host-bound failed, superseded, interrupted, and accepting attempt rather than only the winner, and unreceipted diagnostics remain exclusively under `target/hardening-diagnostic`.
- [ ] Every clean candidate runs the cumulative catalog prefix through its own cycle; Cycle 4 reruns Cycle 3 native CI, and Cycles 5-7 retain the cumulative external job/artifact floor.
- [ ] No wave exceeds three concurrent agents; all role reports use shared `tersh-evidence-orchestration-v1` with `tersh-evidence-review-v1`, except Cycle 7's post-audit Wave C and closure reports use exact audit-bound `tersh-evidence-review-v2` while its pre-audit Wave A/B remain v1; all require `gpt-5.6-sol` and `xhigh` proven by a root-owned, root-peer-authenticated supervisor envelope or response-bound operator attestation. Create-mode finalization authenticates the host receipt over bounded `AF_UNIX/SOCK_STREAM`; a missing supervisor, receipt, or host identity/lifecycle response fails closed.
- [ ] Finding and parent IDs use the shared closed union grammar, and every resolution binds its evidence attempt, candidate, run binding, review filename, and canonical review-body SHA-256 without self-reference.
- [ ] Every Cargo build/test/run command that resolves dependencies uses `--locked`.
- [ ] Filtered test commands name an owning test target; final gates run the complete target and all targets.
- [ ] Every frozen parameter matrix is supplied independently on the exact-runner CLI: inherited `exdev-transition-replay` has exactly three IDs plus its separate single-use exact gate; each hardening-specific ADD-010 replay matrix has four IDs and each private live-lock matrix has two; ADD-009 has nine; `hardening-proxy-lifecycle`, `g3-shutdown`, and `g3-launch` have separate owning-module `--lib` gates with seven, eight, and thirteen IDs; all remaining ordered counts match the locked table.
- [ ] Missing native roots, multiplexers, target runners, CI artifacts, or review reports fail closed.
- [ ] Every external gate calls only the shared bootstrap helper, requires each candidate workflow and evidence-producer helper blob digest to match the independently registered bundle policy before one create-new `codex/evidence/**` push, snapshots repo-wide runs first, and selects only a fresh response whose bare REST `path`, exact `head_sha`, exact `head_branch`, and `event=push` all match. It uses only legal cumulative job IDs and embeds all attempt metadata bodies/hashes in the cycle manifest; no `PATH@REF`, workflow-path selector/watch, or first-run dispatch remains.
- [ ] Every artifact-bearing workflow requires the frozen producer/template/schema triples and rejects extras; each artifact root manifest binds the exact candidate, numeric run ID/attempt, producer, and nonempty payload hashes while excluding itself. The outer index hashes that manifest, joins the pinned upload Action's bare 64-hex digest to REST `sha256:<hex>`, and accepts only the optional-byte-zero-BOM plus per-line RFC3339Z job-log fixture.
- [ ] `hardening-04` retains the exact two native EXDEV producer/template/schema artifacts, rejects extras, and uses the same manifest self-exclusion, outer-index, runtime producer-join, and terminal cancellation-drain contracts as every later cumulative external gate.
- [ ] Thresholds are limited to frozen design gates and deterministic invariants; other measurements are reported without invented promises.
- [ ] Tasks 1-6 contain no full-task completion wording, and Task 7 changes it only after a passing audit.
- [ ] Hardening entry `HEAD` equals the verified impl-07 evidence-only closure commit; later candidates descend from that exact entry.
- [ ] Cycle 7 revalidates impl-01 through impl-07 and the six prior envelopes, audits exactly the root-bundle-pinned 269 requirement IDs using logical references only, embeds every canonical host-spooled evidence body/hash, proves `verify-only` is only `STRUCTURAL_PASS/HOST_UNVERIFIED` and needs no raw/`target` state, binds the host-derived ordered requirements/audit package digest without using it as a candidate substitute, and obtains the detached post-commit closure receipt before formal PASS.
- [ ] Every command, path, role, gate, expected count, and external job is concrete and owned.
- [ ] Type, script, schema, role, gate, evidence path, and command names remain consistent across all seven tasks.
