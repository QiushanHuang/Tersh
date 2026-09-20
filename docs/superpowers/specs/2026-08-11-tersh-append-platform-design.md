# Tersh Append-Platform Evidence Design

- Date: 2026-08-11
- Task ID: `T-TERSH-PRODUCT-OPT-20260810-001`
- Product scope: Tersh personal software, outside ResearchOS
- Repository worktree:
  `/Users/joshua/.config/superpowers/worktrees/Studio/codex-tersh-trusted-core`
- Design baseline: `codex/tersh-trusted-core@e10c91d5130123dd9741dd004586b48d05187f6d`
- Experiment, run, and ResearchOS artifact IDs: not applicable

## Decision Summary

The `append-platform` evidence path will use four mutually reinforcing
boundaries:

1. dispatch context schema `tersh-host-dispatch-context-v2` binds the complete
   pre-spawn parent-finding set;
2. a root-owned Host Envelope service binds every append connection to one
   immutable recorder session, attempt, candidate/tree, policy route, and
   handle lineage;
3. the Host constructs and freezes one exact canonical orchestration record,
   then sends that record as the final BODY before the wire COMMIT for independent client
   validation; and
4. one Host-ledger transaction consumes the context/member handles and session,
   stores those exact frozen bytes, appends one receipt, and, when applicable,
   creates one pending agent-report authority.

The protocol does not accept a record body, candidate, destination, parent
finding, identity, timestamp, model, receipt, or policy override from argv or
the environment. It also does not preserve the current hash-only record design:
a SHA-256 value cannot provide bytes that the Host never received or built.

The user delegated detailed product and protocol decisions to the agent team.
Three independent design reviews compared context versioning, an added Host
session, client record upload, and Host record construction. Their shared
recommendation is the design above because it closes both the authority gap and
the missing record-data path without creating two independent record
serializers.

## Established Problem

At baseline, the repository implements authenticated capture for context,
invocation, and response envelopes plus pure validation of the three-body
`platform-envelope` provenance arm. The stateful append operation remains only
plan prose.

The previous append description is not executable:

- its Host BODY sequence contains only context, invocation, and response;
- those bodies do not contain the attempt-bound candidate tree, registered
  policy route, derived destination, or parent finding set;
- the recorder is required to construct a complete orchestration record and
  send only its SHA-256 in COMMIT; and
- the Host is then required to persist the complete record even though no
  client-to-Host frame carried those bytes and no Host-construction rule was
  defined.

The existing capture store also lacks a lineage typestate capable of proving
that `H2`, `HI`, and `HR` belong to one dispatch generation. Equal body values,
matching nonces, or individually valid handles are insufficient because a
caller could mix live handles from different dispatches.

## Goals

This design makes the following statements falsifiable:

- every orchestration record has one exact closed schema and an authoritative
  source for every field;
- parent finding IDs are frozen before the agent starts and cannot be relabeled
  after its response;
- the Host-selected attempt binding, candidate/tree, policy, destination, and
  handle lineage are all joined to the same recorder session;
- the client validates the exact bytes that the Host later commits;
- a successful transition produces all handle, blob, receipt, sequence, and
  report-authority effects, never a subset;
- pre-linearization failure is retryable with live handles and no evidence
  mutation;
- post-linearization reply failure cannot revive handles or create a second
  receipt;
  and
- local projections are recoverable views of Host-authoritative bytes rather
  than an alternative evidence source.

## Non-Goals

This design does not implement:

- `append-producer-batch`, manifest sealing, formal-lineage query, or Git
  closure;
- the later `seal-agent-record` record validator or review schema;
- a candidate-visible Host daemon, root principal, or test-only principal
  override;
- an offline substitute for root-peer-authenticated formal verification;
- client-side Git inspection during append; or
- a general transaction, workflow, plugin, or job framework.

The operator-attestation arm is not specified here. Context v2 does not prevent
a later attestation design, but that design needs its own provenance and record
source map because it has no invocation BODY. It must not reuse
`tersh-evidence-orchestration-v1` by inventing selected-model or dispatch-time
facts.

## Alternatives Considered

### Selected: Host-built record with client validation

The Host already owns the captured envelopes, attempt binding, producer
session, policy registration, and clean worktree observation. It constructs one
canonical record buffer, sends that exact object and digest to the registered
recorder, and persists the same buffer only after the recorder validates every
field and completes COMMIT.

This keeps one record constructor and a separate validator. A defect in either
side is detectable because the client derives every expected field from earlier
Host BODYs and rejects a mismatching final record.

### Rejected for this checkpoint: client RECORD upload

A client-built single RECORD stream can be secure, but it adds
RECORD-BEGIN/CHUNK/END phases, record-count and chunk aggregates, and a second
construction path that the Host must independently parse and rederive. That
machinery is appropriate for the later generic multi-record producer, not for
one bounded orchestration record that the Host can already derive from private
state.

### Rejected: fixed-empty or post-response parent findings

Always emitting an empty parent set would weaken the corrective-review model.
Allowing a recorder session to choose parents after response would permit
post-hoc relabeling of what the agent was asked to address. Parent IDs therefore
belong in a new closed context version created before spawn.

### Rejected: independent Host and client constructors

Two constructors that merely compare a digest can drift in field selection,
canonicalization, or version handling. This design uses one Host constructor
and one client validator over the exact Host-frozen body.

## Trust Boundary

Formal execution still requires:

- the kernel's peer-credential and no-follow filesystem semantics;
- a UID-0 Host Envelope service with a private append-only ledger;
- a root-owned, exact-tree, digest-pinned harness bundle and policy;
- the attempt-bound runtime profile and registered entrypoint; and
- a nonroot client launched with the one connected Host FD and no inherited
  authority descriptors.

The candidate checkout, environment, CLI strings, agent report, diagnostic
drafts, local projection tree, and embedded receipt-looking JSON are not trust
anchors. The connected FD is useful only because the Host has already bound it
to a recorder session before launch. Root peer credentials alone do not select
an attempt, policy, worktree, or handle generation.

The formal projection root named by the Host's `local` root-map entry is
physically outside every candidate/operator-writable directory. It and every
ancestor are root-owned, no-follow opened, and not group/world writable; the
Host retains the opened directory identity for the attempt. Nonroot processes
may receive read access but cannot add, rename, replace, or unlink entries.
Any candidate-visible copy beneath a worktree `target/` directory is a disjoint
diagnostic mirror and is never scanned as a formal projection.

The registered recorder performs no subprocess launch. It does not run Git,
read `HEAD`, resolve a destination from the filesystem, inspect a user config,
or accept a pathname. Candidate and route facts arrive only in the Host session
BODY and are revalidated by the Host at COMMIT.

## Context V2

`tersh-host-dispatch-context-v2` is the exact v1 field set plus required
`parent_finding_ids`:

```text
{
  schema,
  context_nonce,
  harness_bundle_revision,
  harness_bundle_sha256,
  evidence_id,
  evidence_attempt,
  role,
  wave,
  review_attempt,
  run_binding,
  baseline_commit,
  review_target,
  parent_finding_ids,
  canonical_task_path,
  worktree_handle,
  requested_model,
  requested_reasoning_effort,
  created_at
}
```

The schema literal is `tersh-host-dispatch-context-v2`. Context v1 is rejected
by a v2 bundle; the new field is not an optional extension under the old schema.

`parent_finding_ids` is an array with these exact rules:

- length `0..128`;
- every member is an exact string under the shared finding-ID grammar;
- every member carries the context's `evidence_id`;
- members are strictly ascending by their three-digit finding sequence;
- duplicates, aliases, missing values, null, and booleans are rejected; and
- at context creation, the Host requires every member to resolve to a finding
  in Host-receipted review or audit-failure history whose evidence attempt is
  strictly less than the context's `evidence_attempt`.

An empty array is valid for a dispatch with no parent finding. The UID-0
supervisor freezes the array before spawn. The agent, recorder CLI, response,
review draft, and environment cannot add, delete, or reorder it.

The context canonical body and its digest remain embedded in provenance, so an
offline structural verifier can recompute the parent binding even though it
cannot prove Host origin without the online receipt lineage.

## Exact Recorder Session

Before launching the recorder, the Host creates one connection-bound body with
schema `tersh-host-orchestration-recorder-session-v1` and exact fields:

```text
{
  schema,
  producer_session_id,
  attempt_binding_id,
  predecessor_attempt_binding_id,
  entrypoint,
  producer_mode,
  operation,
  context_nonce,
  dispatch_id,
  evidence_id,
  evidence_attempt,
  run_binding,
  candidate,
  candidate_tree,
  worktree_handle,
  worktree_observed_at,
  baseline_commit,
  candidate_relation,
  bundle_id,
  runtime_profile_id,
  policy_sha256,
  policy_entry_id,
  policy_entry_sha256,
  projection_root_class,
  record_class,
  record_schema,
  destination,
  parent_finding_ids_sha256,
  next_receipt_sequence,
  previous_receipt_id
}
```

For `append-platform`, fixed values are:

```text
entrypoint              = "record-orchestration"
producer_mode           = "harness"
operation               = "append-platform"
projection_root_class   = "local"
record_class            = "orchestration"
record_schema           = "tersh-evidence-orchestration-v1"
```

`producer_session_id`, `attempt_binding_id`, `context_nonce`, `dispatch_id`,
`worktree_handle`, `bundle_id`, `runtime_profile_id`, `policy_sha256`,
`policy_entry_sha256`, and `parent_finding_ids_sha256` are exact
64-lowercase-hex strings. Candidate, candidate tree, and `baseline_commit` are
exact 40-lowercase-hex Git object IDs. `evidence_id`, three-digit
`evidence_attempt`, `run_binding`, and `policy_entry_id` use their shared closed
grammars. `predecessor_attempt_binding_id` is null only for attempt `001` and
otherwise is an exact 64-lowercase-hex ID equal to the immutable binding's
predecessor. Timestamps use the shared RFC 3339 UTC grammar.
`candidate_relation` is exactly `equal|descendant`.
`next_receipt_sequence` is a positive integer; `previous_receipt_id` is null
exactly when that sequence is one and otherwise is an exact 64-lowercase-hex
receipt ID. `parent_finding_ids_sha256` hashes the canonical JSON array
including its trailing LF.

The Host derives the session from one immutable attempt binding and registered
policy. It requires:

- session `attempt_binding_id`, predecessor, evidence, attempt, candidate/tree,
  worktree, bundle, runtime, and policy to equal that binding and its
  registrations;
- session `context_nonce == context.context_nonce`;
- session `evidence_id`, `evidence_attempt`, `run_binding`, `worktree_handle`,
  and `baseline_commit` to equal those exact context fields;
- session `dispatch_id == invocation.dispatch_id == response.dispatch_id` and
  all three bodies to select one captured lineage;
- the context's bundle manifest SHA to equal `bundle_id`;
- the installed bundle revision to equal the context revision;
- the parent digest to equal the exact context-v2 array; and
- `candidate_relation` to be `equal` if and only if candidate equals baseline,
  otherwise `descendant`, with raw-object ancestry proved by the Host; and
- the sequence/predecessor pair to equal the current attempt-global Host ledger
  head when the session is created; and
- destination to equal
  `attempt-NNN/candidate-SHA/orchestration/ROLE.WAVE.REVIEW_ATTEMPT.json`
  beneath the policy-fixed local class base.

More precisely, the filename is
`ROLE.WAVE.REVIEW_ATTEMPT.json`, using the context's exact role, wave, and
three-digit review attempt; for example, `safety.wave-c.001.json`. No caller
provides any path component.

Before launch, a session is bound to one connected FD, operation, and lineage.
When the Host receives the first valid BEGIN, it binds that same session to the
fresh transaction nonce before sending any BODY. It cannot move to another FD
or survive a durable ledger linearization. A pre-linearization disconnect
discards the session lease without consuming the handles; the root supervisor
may create a new session over the same still-live lineage.

The connection handoff has an internal five-second monotonic launch lease. A
valid BEGIN accepted within that lease starts one fresh absolute five-second
monotonic transaction deadline shared by every subsequent prefix, frame body,
half-close, and EOF check. Neither deadline is reset per frame, serialized on
the wire, read from the environment, or caller-configurable. Expiry before
ledger linearization discards only the session lease and follows the
pre-linearization retry rule.

## Exact Orchestration Record

The Host constructs schema `tersh-evidence-orchestration-v1` with exactly:

```text
{
  schema,
  evidence_id,
  evidence_attempt,
  run_binding,
  role,
  wave,
  review_attempt,
  baseline_commit,
  reviewed_commit,
  parent_finding_ids,
  dispatch_id,
  agent_id,
  canonical_task_path,
  agent_run_id,
  model,
  reasoning_effort,
  dispatched_at,
  started_at,
  ended_at,
  terminal_status,
  provenance
}
```

The canonical record is nonempty and at most 61,440 bytes. The Host also
requires the complete canonical BODY wrapper to remain within the shared
65,536-byte frame limit before sending it. Its source map is closed:

| Record field | Sole source |
| --- | --- |
| `schema` | Host literal |
| `evidence_id`, `evidence_attempt`, `run_binding`, `role`, `wave`, `review_attempt`, `baseline_commit`, `parent_finding_ids` | validated context-v2 |
| `reviewed_commit` | recorder session candidate |
| `dispatch_id`, `dispatched_at` | invocation |
| `agent_id`, `agent_run_id`, `started_at`, `ended_at`, `terminal_status` | response-v2 |
| `canonical_task_path` | context, byte-equal to response |
| `model`, `reasoning_effort` | invocation selected values, after all requested/selected joins |
| `provenance` | exact validated platform-envelope arm containing detached context, invocation, and response bodies plus their canonical hashes |

The record deliberately does not duplicate `created_at`, `review_target`,
`worktree_handle`, `reported_result_commit`, or `reported_record_sha256` at the
top level. Those facts remain available in the complete provenance bodies.

`reviewed_commit` always equals the attempt-bound session candidate. The record
does not treat the nullable response-reported commit as authority.

When the later sealer accepts a formal review, that review's
`parent_finding_ids` must be byte-for-byte equal to this orchestration record and
its embedded context-v2 body. A report cannot expand or relabel its assigned
parent set after dispatch.

## Candidate Rules

The Host applies these rules before sending the final record and again under the
ledger linearization lock:

- the session candidate/tree equals the immutable attempt binding and current
  clean worktree observation;
- the observed index, HEAD, tree, and worktree identity do not drift between
  session creation and linearization;
- for every wave, `candidate_relation` is `equal` exactly when candidate equals
  baseline and otherwise is `descendant`; in the latter case the Host proves
  from raw commit objects that baseline is an ancestor of candidate, with
  replacement refs, grafts, shallow/alternate object views, and caller Git
  configuration disabled;
- `wave == "wave-a"` requires
  `baseline_commit == review_target == session.candidate`;
- a nonnull `response.reported_result_commit` equals the session candidate;
- outside Wave B, `context.review_target` is nonnull and equals the session
  candidate, in addition to the stricter Wave-A equality above;
- in Wave B, role is exactly `implementation` and
  `context.review_target == context.baseline_commit == session.baseline_commit`;
- in Wave B, that baseline is derived from and equals the immutable immediate
  predecessor attempt binding's candidate and the clean pre-spawn HEAD; an
  older ancestor supplied by context cannot replace this Host-ledger value;
- if a Wave B candidate differs from its baseline, the reported result commit
  is nonnull and equals the candidate; and
- if Wave B makes no candidate change, candidate equals baseline and the
  reported result commit is either null or that same candidate.

Changed-plus-null, unrelated/replaced history, reported/binding/HEAD mismatch,
dirty worktree, wrong tree, unborn HEAD, missing object, or a BODY-to-COMMIT ref
change fails before any handle or evidence mutation. A policy-authorized
history rewrite starts a distinct evidence lineage; it cannot use ordinary
Wave B.

## Handle Lineage Typestate

The Host stores a private random `lineage_id` for each dispatch. Every handle row
binds `{lineage_id,kind,transition_index,body_sha256,live}`. Body equality never
substitutes for this relation.

The platform path is:

```text
CREATED(H0)
  -> INVOKED(H1, HI)
  -> RESPONDED_PLATFORM(H2, HI, HR)
  -> APPENDED_PLATFORM
```

Only the transition shown for the current state is legal. A second invocation,
second response, old predecessor, duplicate member, aliased handle, mixed
lineage, a tuple not in `RESPONDED_PLATFORM`, or a consumed handle fails without
changing any row.

`append-platform` requires live `H2`, `HI`, and `HR` from one
`RESPONDED_PLATFORM` state. A later attestation design must define its own
typestate and cannot silently extend this one.

An adapter reply failure after a capture COMMIT may leave private successor
handles that were never exposed to the client. The Host must not discard a
captured response or candidate merely because delivery of those handles
failed. A root-internal recovery operation rotates every live unpublished
handle for that lineage to a fresh replacement and returns the replacements
only to the UID-0 supervisor.

Recovery accepts exact request
`tersh-host-recover-dispatch-lineage-request-v1`
`{schema,context_nonce,expected_state,transition_index,recovery_generation,reason}`.
`expected_state` is exactly `created|invoked|responded-platform`; both indexes
are nonnegative integers; and reason is exactly
`capture-reply-unrecoverable`. Its exact result is
`tersh-host-recover-dispatch-lineage-result-v1`
`{schema,context_nonce,state,transition_index,recovery_generation,context_handle,invocation_handle,response_handle}`.
The state and transition index equal the request, while result
`recovery_generation == request.recovery_generation + 1`. `created` returns
only a context handle; `invoked` returns context and invocation handles;
`responded-platform` returns all three; every inapplicable handle is null.
Identical replay before any replacement is used returns the same stored result;
wrong state/index/generation, cross-lineage access, a later transition, or
replay after use conflicts. A recovered `responded-platform` tuple must proceed
through the ordinary recorder session and append path, so its response and
candidate remain enumerable.

Recovery is one durable compare-and-swap ledger transaction. It validates the
exact state/generation and absence of append, failure, abandonment, or a
competing recovery of the current tuple;
invalidates the complete old live tuple; creates the complete replacement
tuple; and stores the canonical request digest plus replay result before any
result byte is returned. A crash exposes either the entire old tuple or the
entire new tuple plus stored result, never a mix. A replacement becomes used
only when a later capture or append transition durably consumes it; presenting
it in BEGIN or a failed pre-linearization session does not consume it.
When such a transition advances `transition_index`, a later lost reply may
recover that new tuple with its own generation; an earlier recovery row does
not block it.

Zero-record abandonment is allowed only in `created` or `invoked`, where no
response/result can exist. That internal operation accepts exact request
`tersh-host-abandon-dispatch-lineage-request-v1`
`{schema,context_nonce,expected_state,transition_index,recovery_generation,reason}`,
where `expected_state` is `created|invoked`, both indexes match the current
tuple, and reason is exactly `capture-reply-unrecoverable`. The root
supervisor obtains `context_nonce` from the dispatch it created; no nonroot CLI
exposes this request. The exact result is
`tersh-host-abandon-dispatch-lineage-result-v1`
`{schema,context_nonce,state}`, with state exactly `abandoned`. Identical replay
returns the stored result; a lineage with committed append evidence conflicts.
The operation creates no evidence blob, producer receipt, or report authority.
If `open-attempt` already persisted the attempt binding, that attempt remains
an enumerable marker-only failure and its number is never released or reused;
only a wholly tentative reservation with no binding may be released. A
responded lineage rejects abandonment and requires recovery.

Abandonment is also one durable compare-and-swap transaction: it validates the
exact state and absence of another terminal transition, invalidates every live
handle in the tuple, marks the lineage terminal-abandoned, records the request
digest and replay result, and atomically either releases the no-binding
reservation or retains the existing marker binding. A crash exposes only the
unchanged live state or the complete abandoned state. Concurrent recovery,
abandonment, capture, and append race under the same lineage lock and exactly
one transition can win.

When a binding is retained, that same abandonment commit changes attempt
`ACTIVE` to `CLOSING_FAILED` and persists the lineage's terminal `abandoned`
row for the close barrier. It is an accepted failed-attempt trigger even though
no response body or producer receipt exists. Host attempt enumeration exposes
the terminal row alongside the marker-only binding, and the finalizer treats it
as non-accepting history. Only no-binding abandonment keeps the no-record,
release-and-reuse behavior.

If a recovered `responded-platform` lineage cannot ever append because a
nonretryable candidate, worktree, policy, or record invariant fails, it uses a
different root-internal terminal transition; it is never zero-record abandoned.
The exact request is `tersh-host-fail-responded-dispatch-request-v1`
`{schema,context_nonce,transition_index,recovery_generation,reason}`. The two
indexes must match the current responded tuple. Reason is exactly
`candidate-object-missing|candidate-relation-invalid|worktree-identity-drift|attempt-policy-drift|record-construction-invalid`.
The exact result is `tersh-host-fail-responded-dispatch-result-v1`
`{schema,context_nonce,state,failure_receipt_id}`, where state is `failed` and
the receipt ID is exact 64-lowercase hex.

Under one durable ledger transaction that operation validates the root-observed
failure, consumes the full `H2+HI+HR` tuple, stores one closed
`tersh-host-dispatch-failure-v1` body, appends one consecutive harness receipt
with `entrypoint="fail-dispatch-lineage"` and
`record_class="orchestration-failure"`, marks that lineage terminal-failed and
the attempt `CLOSING_FAILED`, and creates no report authority. The failure body is exactly
`{schema,evidence_id,evidence_attempt,candidate,context_nonce,dispatch_id,reason,context,invocation,response,created_at}`;
each envelope member is the exact captured `{body,sha256}` entry. Its
policy-derived destination is
`attempt-NNN/candidate-SHA/orchestration-failures/DISPATCH_ID.json`. The shared
receipt record-class enum and finalizer history must include this class, and any
such body makes the attempt non-accepting. It does not by itself permit the next
attempt.
The operation is keyed by `context_nonce`: identical lost-result replay returns
the stored result, while a different reason, any partial body, or a competing
transition conflicts. Its body, receipt, terminal state, and idempotency row
have the same all-or-none crash guarantee as append.

The failure body's source map is closed. Evidence ID, attempt, and candidate
come from the immutable attempt binding and must equal context; context nonce
comes from that context; dispatch ID is byte-equal across invocation and
response; the three envelope entries are the exact already validated captured
bodies and hashes; reason is the root-observed failure validated by the Host;
and Host `created_at` is not earlier than response `ended_at`. Unknown,
duplicated, aliased, or caller-supplied values are rejected.

The failure receipt is also closed: `producer_mode="harness"`,
`entrypoint="fail-dispatch-lineage"`, the same attempt binding and its
bundle/runtime/registered failure-policy row,
`projection_root_class="local"`, `record_class="orchestration-failure"`,
`record_schema="tersh-host-dispatch-failure-v1"`, and the exact derived
destination, canonical body size/hash, and current sequence/predecessor. Its
agent-report fields and environment capability are null. Every one of those
fields is revalidated under the ledger lock.

An orchestration append with a nonnull reported draft digest is not terminal
until its private report authority is either sealed or failed. If the Host can
prove that authority permanently unsealable, the root-internal exact request is
`tersh-host-fail-agent-report-authority-request-v1`
`{schema,context_nonce,reason}`, where reason is exactly
`draft-missing|draft-digest-mismatch|draft-schema-invalid|draft-path-invalid|sealer-policy-invalid`.
The exact result is `tersh-host-fail-agent-report-authority-result-v1`
`{schema,context_nonce,state,failure_receipt_id}`, with state `failed` and one
exact 64-lowercase-hex receipt ID.

One durable transaction revalidates the Host-selected authority and root-
observed reason, consumes that authority, stores exact body
`tersh-host-agent-report-failure-v1`
`{schema,evidence_id,evidence_attempt,candidate,context_nonce,dispatch_id,reason,reported_record_sha256,orchestration_receipt_id,created_at}`,
appends one consecutive harness receipt, marks the lineage `report-failed`, and
moves an `ACTIVE` attempt to `CLOSING_FAILED`. The body sources are the immutable
attempt binding, captured context/response, pending authority, and Host clock;
`created_at >= response.ended_at`. The receipt is fixed to the same binding,
bundle/runtime and registered failure-policy row,
`entrypoint="fail-agent-report-authority"`,
`projection_root_class="local"`, `record_class="agent-report-failure"`,
`record_schema="tersh-host-agent-report-failure-v1"`, policy-derived
`attempt-NNN/candidate-SHA/review-failures/DISPATCH_ID.json`, exact body
size/hash/current chain, null agent-report fields, and null environment
capability. The shared record-class enum and finalizer history must include it.

This transition is idempotent by private authority/context nonce: an identical
lost result returns the stored body/receipt result; changed reason, wrong
authority, sealer race, or reuse conflicts. Authority consumption, body,
receipt, lineage state, attempt state, and replay row are one all-or-none
commit. It creates no formal review and permanently makes the attempt
non-accepting, while retaining the callback digest and orchestration join in
Host-enumerable history.

Attempt state is exactly
`ACTIVE|CLOSING_FAILED|TERMINAL_FAILED|TERMINAL_SUPERSEDED|FORMALLY_CLOSED`.
The first lineage-failure commit changes `ACTIVE` to `CLOSING_FAILED`. That
state forbids creating a new dispatch lineage but permits every already
registered sibling to finish capture, recover, append, fail, or abandon, and
permits an already created report authority to seal or enter its own terminal
failure path. Every such transition joins the attempt-global lock and rejects
all three terminal states.

The root-internal close barrier accepts exact request
`tersh-host-close-failed-attempt-request-v1`
`{schema,attempt_binding_id}` and returns exact result
`tersh-host-close-failed-attempt-result-v1`
`{schema,attempt_binding_id,state,terminal_lineage_count,ordered_lineage_states_sha256}`,
where state is exactly `terminal-failed`. In one durable transaction it requires
at least one dispatch-failure receipt, agent-report-failure receipt, or
binding-retaining abandonment row; every registered lineage in an accounted
terminal state; and zero live handle, recorder session, recovery result capable
of further transition, or pending report authority. It then seals the attempt
`TERMINAL_FAILED` and stores the replay result. Identical replay returns that
result; any changed lineage set conflicts.

The barrier hashes the exact array of
`tersh-host-terminal-lineage-state-v1` rows. Each row is exactly
`{schema,lineage_id,context_nonce,transition_index,terminal_state,terminal_receipt_id}`.
IDs are exact 64-lowercase hex, transition index is a nonnegative integer, and
terminal state is exactly
`appended-no-report|review-sealed|report-failed|dispatch-failed|abandoned`.
`terminal_receipt_id` is null only for `abandoned`; otherwise it is the exact
orchestration, review, agent-report-failure, or dispatch-failure receipt that
caused the named state. Rows are strictly sorted by `lineage_id`, with no
duplicate or omitted registry lineage. `terminal_lineage_count` is the positive
array length and equals the frozen attempt registration count;
`ordered_lineage_states_sha256` hashes the canonical ordered array plus LF.
The Host rederives every row under the barrier lock rather than accepting this
array from a caller.

A normally drained attempt whose Host-receipted reviews require correction uses
a separate supersede barrier. Its root-internal exact request is
`tersh-host-close-superseded-attempt-request-v1`
`{schema,attempt_binding_id}`. The exact result is
`tersh-host-close-superseded-attempt-result-v1`
`{schema,attempt_binding_id,state,superseding_finding_count,ordered_superseding_finding_ids_sha256,terminal_lineage_count,ordered_lineage_states_sha256}`,
where state is `terminal-superseded`. The Host derives a nonempty, strictly
sorted set of unresolved P0/P1 finding IDs from the attempt's exact
Host-receipted review bodies; the count is positive and the digest hashes the
canonical ordered ID array plus LF. No caller supplies that set.

Under the attempt-global lock, supersede requires state `ACTIVE`, the same
complete terminal-lineage array and zero-live/session/authority conditions as
the failed barrier, and no existing manifest preimage or closure. It atomically
stores the two aggregate digests, freezes the receipt/lineage set, changes state
to `TERMINAL_SUPERSEDED`, and persists the replay result. A late append either
commits before this validation and is included or loses after the terminal
state is visible; it can never mutate the frozen predecessor.

`open-attempt(N+1)` joins the same attempt-global lock and requires its
predecessor be `TERMINAL_FAILED` or `TERMINAL_SUPERSEDED`, with the zero-live
barrier and frozen lineage/receipt aggregates still true. `FORMALLY_CLOSED`
is set only by the broader manifest `commit-and-close` operation after its own
lineage checks and prohibits a successor. Thus a sibling append may finish while an attempt is
`CLOSING_FAILED`, but no receipt can arrive after either terminal barrier and no
next attempt can open early. A fail/supersede-vs-sibling-append-vs-open race has
one serializable history and never loses a captured response.

## Append-Platform Wire Protocol

The registered recorder accepts only:

```text
append-platform
--context-handle H2
--invocation-handle HI
--response-handle HR
--host-store-fd FD
```

Every scalar option is required exactly once. Abbreviations, duplicate options,
stdin/path sockets, body JSON, candidate, parent, model, identity, timestamp,
destination, receipt, nonce, policy, or test-mode arguments are rejected before
opening the FD.

BEGIN is the exact existing record arm:

```text
{
  schema: "tersh-host-transaction-begin-v1",
  transaction_nonce,
  operation: "append-platform",
  context_handle,
  invocation_handle,
  response_handle
}
```

The Host sends exactly five BODY wrappers in this order:

```text
1. context
2. invocation
3. response
4. recorder-session
5. orchestration-record
```

BODY-END has total `5` and the exact ordered digest array. Context is v2;
invocation and response retain their closed v1/v2 schemas; recorder-session and
orchestration-record use the exact schemas above.

Before sending the wire COMMIT, the client:

1. validates each BODY wrapper and canonical digest;
2. validates the platform provenance triplet;
3. validates the recorder session and every attempt/policy/route join available
   from the installed bundle and Host bodies;
4. independently derives every orchestration field from the first four BODYs;
5. requires the fifth BODY to equal that derived closed record exactly; and
6. requires the fifth BODY digest to hash the Host-frozen record bytes.

The Host freezes the canonical record buffer before sending BODY 5. It persists
that same buffer only at durable ledger linearization; receiving the wire COMMIT
does not itself mutate state. It does not reconstruct or reserialize the body.
The client sends no RECORD-BEGIN, RECORD-CHUNK, RECORD-END, body JSON, or upload
frame. Any such frame is a protocol error.

COMMIT is exactly:

```text
{
  schema: "tersh-host-transaction-commit-v1",
  transaction_nonce,
  operation: "append-platform",
  body_sha256s,
  record_facts: {
    evidence_id,
    evidence_attempt,
    run_binding,
    candidate,
    destination,
    record_sha256
  }
}
```

Every field is an assertion derived from the validated BODYs. In particular,
`record_sha256 == body_sha256s[4]`, candidate equals session candidate, and
destination equals session destination. REQUEST-END and both final half-closes
retain the shared exact transaction schemas and ordering.

The Host revalidates the current session, handle lineage, attempt binding,
worktree observation, route, receipt-chain head, frozen buffer, and COMMIT
assertions after exact REQUEST-END plus EOF. Under the same ledger lock it
checks that the absolute deadline has not expired and that session
`next_receipt_sequence` and `previous_receipt_id` still equal the current chain
head. A deadline or head drift is a pre-linearization failure with no state
mutation.

REPLY retains the exact record result:

```text
{
  schema: "tersh-host-transaction-reply-v1",
  transaction_nonce,
  operation: "append-platform",
  body_sha256s,
  result: {
    schema: "tersh-host-record-result-v1",
    receipt
  }
}
```

The client validates the complete closed producer receipt. It requires the
receipt's attempt binding, producer session, bundle/runtime/policy entry,
producer mode, entrypoint, projection root class, record class/schema,
destination, byte count, and body hash to equal the session and frozen record.
Its sequence must equal session `next_receipt_sequence`, and its predecessor
must equal session `previous_receipt_id`. For the harness-produced orchestration
receipt, detached agent-report fields are null. The private pending report
authority is not returned.

The recorder never receives a projection-root path or directory FD and never
writes the projection filesystem. After durable ledger commit, the Host may
use its policy-bound no-follow projection-root capability to create-new publish
the exact frozen bytes before REPLY. A publication failure leaves the receipt
and blob durable but the projection absent; the root-owned repair operation is
the only recovery path.

Because that root has no candidate-writable ancestor, the create-new namespace
check cannot be raced by the worktree between ledger commit and publication.
An unexpected existing file, symlink, directory, changed root identity, or
permission drift therefore indicates Host storage/TCB corruption and fails
closed; it is not quarantined or replaced. Ordinary post-commit I/O failure may
leave only an absent leaf, which `repair-projections` can create from the exact
ledger blob. The broader implementation plan must treat its existing
candidate-writable `target/...` projection wording as diagnostic-only and bind
formal finalization to this Host-exclusive root.

## Atomic Commit And Durable Authority

The Host's private ledger blob is the authoritative host-only spool. The local
formal projection and any filesystem materialization of that blob are
repairable views, not independent sources of truth. This avoids claiming an
impossible atomic transaction across an unrelated filesystem file and a ledger
receipt row.

`PRE_LINEARIZATION` lasts through wire COMMIT, exact REQUEST-END, client EOF,
deadline validation, and all under-lock revalidation. `COMMITTED_DURABLE` starts
only when the following one Host-ledger transaction commits:

1. revalidate and consume the one recorder session;
2. change `H2`, `HI`, and `HR` from live to consumed;
3. insert the exact frozen canonical record bytes as one create-new blob;
4. append exactly one attempt-global consecutive producer receipt with the
   session's exact next sequence and previous-receipt link; and
5. if `response.reported_record_sha256` is nonnull, create exactly one pending
   `tersh-host-agent-report-authority-v1` joined to the response, orchestration
   receipt, draft path, and review destination.

The transaction result is made durable before any REPLY byte is sent. Faults
can expose only two states:

- before: session and handles are live; blob, receipt, and authority are absent;
- after: session and handles are consumed; blob and receipt exist; authority
  exists exactly when the response digest is nonnull.

A database/ledger implementation must provide this atomicity directly. An
implementation that writes an independent spool file first and later appends a
receipt must instead provide a reviewed WAL and crash-recovery proof; it cannot
claim the invariant from ordinary locking alone.

## Retry, Reply Loss, And Projection Repair

Any `PRE_LINEARIZATION` validation failure, timeout, disconnect, client
interruption, REQUEST-END/EOF defect, chain-head drift, or under-lock failure
discards only the transient recorder session. This includes a failure after the
wire COMMIT was received. All handles remain live and no blob, receipt,
sequence, authority, or projection exists. The supervisor may open a fresh
authenticated FD and retry the same lineage.

After `COMMITTED_DURABLE`, a missing, malformed, truncated, or trailing REPLY
returns no client success. Nevertheless, all handles and the session are
consumed and the unique blob/receipt/authority transition remains durable.
Replaying the old handles or beginning another append fails before mutation.

The root supervisor recovers only through Host-selected evidence enumeration.
Enumeration exposes the unique receipt and exact blob bytes without accepting a
caller receipt ID. `repair-projections` creates only a missing byte-identical
projection; an existing mismatch or extra remains a failure. Repair never
appends another receipt or creates another report authority.

When an authority exists, the root supervisor binds a fresh sealer FD to that
exact private authority. The zero-selector sealer does not choose among pending
authorities. Concurrent authorities require separate supervisor-selected FDs;
no global "latest" choice is allowed.

## Deferred Operator Attestation

This checkpoint defines no `attest` BEGIN, session, provenance arm, record
schema, BODY order, or source map. A later design must use actual Host-observed
dispatch facts and an attestation-specific tagged record; it may not substitute
context creation/start time for dispatch time or requested model/effort for
platform-selected metadata. `append-platform` rejects every non-platform
operation or handle state.

## Security And Failure Invariants

The implementation must reject, before state mutation:

- context v1, missing/null/duplicate/reordered/cross-evidence/future parent IDs,
  a same-attempt parent, a projection-only or receipt/body-mismatched parent,
  an ID ambiguously present in two Host-receipted source bodies, or a parent set
  not present in prior Host history;
- valid bodies from different nonces, dispatches, bundles, lineages,
  generations, attempts, worktrees, policies, or candidates;
- a second invocation/response, an old predecessor, duplicate/aliased member,
  or a handle state outside `RESPONDED_PLATFORM`;
- root-authenticated FDs bound to the wrong session, expired sessions, session
  replay, transaction-nonce drift, or FD reuse;
- dirty or drifting worktree/index/HEAD/tree facts;
- a Wave-A baseline/target/candidate mismatch, an unrelated Wave-B candidate,
  a wrong relation tag in any wave, changed-plus-null Wave-B result, or any
  reported/binding/HEAD mismatch;
- wrong record field, extra/missing key, wrong schema, body/digest mismatch,
  record over 60 KiB, record upload frame, or hash-only COMMIT without BODY 5;
- destination normalization, wrong candidate namespace, collision, symlink,
  path escape, or any caller-selected path component; and
- receipt/session/record/sequence mismatch.

No failure writes unbounded diagnostics or leaks private bodies, handles,
session facts, authority selectors, or arbitrary environment values.

## Test Strategy

Implementation proceeds in dependency order with genuine RED before each
production slice.

### Context and pure record validation

- `test_context_v2_binds_bounded_existing_parent_finding_ids`
- `test_context_v2_rejects_v1_optional_alias_duplicate_reordered_cross_evidence_or_future_parents`
- `test_context_v2_rejects_projection_only_mismatched_ambiguous_or_same_attempt_parent_sources`
- `test_orchestration_record_schema_has_one_authoritative_source_per_field`
- `test_orchestration_record_rejects_extra_missing_mixed_or_overbound_fields`

### Session and candidate binding

- `test_append_platform_exact_session_and_host_built_record_body_order`
- `test_recorder_session_schema_rejects_wrong_schema_extra_missing_alias_null_bool_and_wrong_type_fields`
- `test_append_platform_rejects_each_context_session_and_dispatch_identity_drift`
- `test_append_platform_rejects_candidate_tree_policy_destination_or_parent_join_drift`
- `test_append_platform_candidate_rules_cover_wave_a_wave_b_changed_and_unchanged`
- `test_append_platform_rejects_wave_a_baseline_drift_and_unrelated_wave_b_candidate`
- `test_candidate_relation_matches_raw_ancestry_in_wave_c_and_closure`
- `test_wave_b_baseline_is_immediate_predecessor_not_older_context_ancestor`
- `test_root_peer_with_wrong_recorder_session_fails_before_handle_lookup`

### Lineage and protocol closure

- `test_append_platform_rejects_cross_lineage_generation_alias_duplicate_or_mode_mixed_handles`
- `test_append_platform_rejects_record_frame_or_hash_only_commit`
- `test_host_spools_exact_record_body_sent_before_commit`
- `test_record_reply_receipt_joins_session_route_body_and_chain`
- `test_append_platform_chain_head_drift_fails_before_linearization_and_retries`

### Atomicity and recovery

- `test_append_platform_atomic_handle_blob_receipt_authority_fault_matrix`
- `test_append_platform_prelinearization_retry_and_postlinearization_lost_reply_repair`
- `test_append_platform_deadline_after_commit_before_request_end_or_eof_is_prelinearization`
- `test_append_platform_concurrent_calls_create_one_receipt_and_authority`
- `test_capture_reply_orphan_recovery_rotates_private_handles_and_preserves_response`
- `test_recovery_generation_allows_invoked_then_responded_lost_reply_recovery`
- `test_capture_orphan_abandon_rejects_responded_state_and_never_reuses_persisted_attempt`
- `test_recover_and_abandon_closed_schemas_and_atomic_fault_matrix`
- `test_recover_abandon_capture_and_append_race_has_exactly_one_durable_winner`
- `test_irrecoverable_responded_dispatch_atomically_records_failure_and_allows_next_attempt`
- `test_dispatch_failure_lost_reply_replays_one_receipt_and_finalizer_rejects_attempt`
- `test_dispatch_failure_body_receipt_and_route_source_map_is_closed`
- `test_failed_lineage_drains_siblings_before_attempt_barrier_or_next_attempt`
- `test_terminal_lineage_array_rejects_omitted_duplicate_reordered_live_or_pending_rows`
- `test_receipted_findings_supersede_drained_attempt_before_successor_opens`
- `test_supersede_vs_late_append_race_freezes_one_complete_predecessor_history`
- `test_binding_retaining_abandon_closes_marker_only_attempt_before_next_attempt`
- `test_unsealable_report_authority_atomically_records_callback_failure_and_drains_barrier`
- `test_report_authority_failure_lost_result_and_sealer_race_have_one_winner`
- `test_host_projects_exact_committed_blob_and_repairs_only_a_missing_projection`
- `test_formal_projection_root_has_no_candidate_writable_ancestor_or_collision_race`

### Thin CLI boundary

- `test_record_orchestration_append_platform_accepts_only_three_handles_and_host_fd`
- `test_record_orchestration_never_invokes_git_subprocess_or_reads_authority_env`
- `test_record_orchestration_never_opens_or_writes_a_projection_path`
- `test_record_orchestration_isolated_runtime_and_exact_harness_imports`

Every negative transcript asserts no handle, session, blob, receipt, sequence,
authority, or projection mutation in `PRE_LINEARIZATION`. Reply faults after
`COMMITTED_DURABLE` assert the exact opposite all-or-none state and one
repairable receipt.

## Implementation Checkpoints

1. **Context v2:** closed parent-array parser, capture migration, provenance
   validation, and legacy rejection.
2. **Pure schemas:** recorder-session and orchestration-record validators plus
   exact Host-construction/source-map fixtures.
3. **Lineage state:** generation-bearing context/member store and invalid
   transition tests.
4. **Append transaction:** five-BODY state machine, COMMIT/reply validation, and
   atomic Host-ledger transition.
5. **Projection recovery:** byte-identical publication and Host-selected repair
   after reply loss.
6. **Thin recorder CLI:** isolated registered entrypoint with the closed argv.

No checkpoint claims the next one's properties. In particular, pure context or
record validation does not claim handle consumption, and a passing socketpair
transaction does not claim the unavailable privileged production supervisor.

## Acceptance Criteria

The append-platform path is implemented only when:

- all exact tests above are present and falsifiable;
- the registered v2 bundle rejects every legacy or mixed context shape;
- one valid `H2+HI+HR` tuple yields one exact orchestration blob and receipt;
- every field and digest is joined to its sole authoritative source;
- fault injection proves the two allowed durable states at every transition;
- reply-loss enumeration and projection repair recover the one exact record
  without a second receipt or authority;
- the recorder CLI has no hidden authority input and launches no subprocess;
- all preexisting harness tests pass; and
- independent code, security, and test-quality reviews report no open P0/P1 or
  Critical/Important finding.

Append-platform completion makes no claim that operator attestation is already
designed or implemented.

Formal evidence remains fail-closed until the actual UID-0 Host Envelope
Supervisor and required custom GitHub runners exist. Repository socketpair and
synthetic state-machine tests are implementation evidence, not a substitute for
that privileged acceptance boundary.
