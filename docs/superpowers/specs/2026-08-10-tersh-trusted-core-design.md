# Tersh Trusted Core Product Design

- Date: 2026-08-10
- Task ID: `T-TERSH-PRODUCT-OPT-20260810-001`
- Product scope: Tersh personal software, outside ResearchOS
- Repository: `/Users/joshua/Desktop/Qiushan_Studio/6_Personal/Studio`
- Implementation baseline:
  `codex/review@a7e79611c9f8b771b3d2a4ad69c818a26a86774a`
- Implementation branch: `codex/tersh-trusted-core`
- Experiment, run, and ResearchOS artifact IDs: not applicable

## Decision Summary

Tersh will focus on one product promise:

> When a user is already in a local or SSH shell, Tersh helps them locate,
> understand, and safely act on files without a frozen interface, silent data
> loss, or an unexplained partial result.

The file workbench is the primary product. The cluster dashboard remains a
secondary, explicitly bounded companion. This design does not turn Tersh into
an SSH client, SFTP client, synchronization system, fleet monitor, or general
remote-operations platform.

The selected strategy is **Trust-first Workbench**. Two alternatives were
rejected for this release:

- Feature-parity expansion would add previews, plugins, tabs, configurable
  keymaps, mouse support, and Git integrations before the core operation model
  is responsive or recoverable.
- Cluster-led expansion would enlarge the SSH, installation, polling, security,
  and support surfaces without evidence that the cluster dashboard is an
  independent retention driver.

The user delegated detailed product decisions to the agent team. Five
GPT-5.6-sol xhigh agents, working in sequential concurrent groups, completed
three cross-reviewed design iterations. Their shared conclusion was to keep
the cluster surface, freeze its scope, and first make the workbench installable,
responsive, truthful, and recoverable.

## Evidence Behind The Decision

The baseline already has meaningful safety controls: path identity rechecks,
no-follow opens, root/home/work-root delete guards, bounded previews, bounded
cluster probes, known-host enforcement, and terminal restoration guards. Those
controls remain product assets.

The following gaps are established by the current source and tests:

- Directory reload, preview, copy, move, trash, and permanent delete execute on
  the UI event path. Slow storage or a large operation can stop input and
  rendering.
- A destructive batch clears its selection after execution even when some
  targets fail. The six-line log cannot preserve a complete failure set.
- Copy conflicts are decided for the whole batch. Move conflicts are skipped.
  There is no durable, per-item operation outcome.
- Cross-filesystem moves have no `EXDEV` protocol and therefore fail even for a
  regular file that could be moved safely through staged copy and source
  cleanup.
- `.tersh-trash` has no byte-safe origin record, crash reconciliation, list
  command, restore engine, or in-product recovery entry.
- `--print-cwd` does not distinguish a normal commit from `Q` or `Ctrl+C`, so a
  shell wrapper can change the parent shell directory after an abort.
- A remote workbench process that exits nonzero can be reported as normally
  closed.
- Cluster refresh is capped but a single "refresh all" request does not keep
  filling freed slots until every requested host reaches a terminal state.
- The repository has no automated CI, dependency advisory gate, benchmark
  fixture, fault-injection gate, or reproducible multi-platform release flow.
- The development source calls itself 1.1.1 while the latest published release
  and tag are 1.1.0. Installation documentation can follow Git HEAD instead of
  an immutable version.

External product comparison supports focus rather than breadth. Mature terminal
file managers already provide broad preview, search, task, customization, and
packaging systems. Tersh's credible opportunity is a smaller, conservative,
low-friction workflow whose state and failures are easier to trust.

## Product Users And Jobs

### Primary job

A local or SSH-shell user needs to:

1. start Tersh in a known directory;
2. locate and preview a target without losing control of the terminal;
3. perform a small or batch file operation with an explicit target and safe
   default;
4. understand every completed, skipped, failed, or retained item;
5. recover trashed content or retry a failed plan safely; and
6. exit normally, abort, or commit a visual `cd` exactly as intended.

### Secondary job

A multi-host user may open the cluster companion to:

1. inspect a bounded, read-only health snapshot;
2. choose the intended host;
3. confirm whether a Tersh workbench can be launched there;
4. enter a shell or workbench; and
5. return to an honestly refreshed dashboard.

The secondary job must not distort the architecture or release schedule of the
primary job.

## Product Outcomes And Measurement

No default network telemetry will be added. Product evidence comes from:

- deterministic CI fixtures and fault injection;
- explicit local benchmark and doctor commands;
- opt-in, locally generated, path- and host-redacted diagnostic reports; and
- small-sample task tests that report raw numerator, denominator, and failure
  reasons rather than a statistically unsupported retention percentage.

There are two north-star tasks, because an inspection session and a mutation
session do not have the same risk or denominator:

- **Trusted Inspection Completion**: locate a target, render the intended
  preview or an explicit preview failure, and exit according to intent without
  a stale result or unintended parent-shell cwd change.
- **Trusted Mutation Completion**: confirm an immutable preflight, act, verify
  every item, use retry or restore when that path is exercised, and exit with
  no unexplained or misclassified filesystem effect.

Task tests report each outcome as a raw `completed / attempted` count plus the
reason for every non-completion. Inspection and mutation counts are never
combined into one flattering percentage.

Activation is reported as the raw count of participants who install from an
immutable supported input and complete an unassisted Trusted Inspection in
their first session, with median elapsed time and every failure reason. Early
retention is the raw count of activated participants who independently complete
a real Tersh task on at least two distinct days during days 8 through 14. No
percentage target or population claim is made until the sampling method and
denominator are credible.

The frozen initial reference environment is `Qiushanmbp.local`, macOS 26.6
build 25G72, Apple M4 Max arm64, 128 GiB RAM, APFS, with rustc and cargo 1.95.0.
The benchmark harness creates its data in a temporary directory on the same
APFS data volume with a fixed seed. Its named fixtures are: empty directory;
10,000-entry directory; 1 GiB regular file; 10,000 top-level zero-byte files;
long Unicode and invalid-UTF-8 names; permission failures; and injected
1,000 ms scan and preview backends. The harness records its own revision,
fixture manifest, OS, architecture, filesystem, toolchain, and raw samples.

Before implementation changes, the baseline is recorded on those fixtures.
For the injected 1,000 ms scan and preview case, 10 warm-up inputs are excluded
and the next 200 navigation/filter/preview inputs must have key acknowledgement
p95 at or below 100 ms and no event-loop stall above 100 ms. Any later threshold
change requires an explicit design amendment with before/after evidence; a
failing implementation may not redefine its gate.

Deterministic invariants do not wait for a baseline:

- stale read results applied after a newer generation: `0`;
- protected-path escapes: `0`;
- unique source objects lost after a failed or cancelled supported operation:
  `0`;
- item outcomes that disagree with verified final filesystem state: `0`;
- aborts that change the parent shell cwd: `0`;
- supported trash items that cannot be enumerated and restored in fault-free
  conditions: `0`;
- release assets without a source revision and SHA-256 digest: `0`.

Performance reporting must include, without inventing percentages for unknown
work:

- key-to-render p50/p95/max and longest event-loop stall;
- time to initial TUI frame and time to first directory result;
- scan and preview supersession counts;
- processed items and bytes for mutations;
- cancellation acknowledgement latency and time to the next safe stop;
- idle CPU and resident memory;
- copy throughput on fixed local fixtures; and
- cluster sweep completion and maximum real probe concurrency.

## Architecture Principles

### Keep the runtime concrete

The implementation will use dedicated components for this product, not a
general async runtime, plugin job API, generic thread pool, database, global
transaction manager, or path-lock framework.

The workbench will have three execution paths:

1. **UI coordinator**: owns `App`, terminal input, rendered state, generation
   counters, and the sole authority to publish terminal operation states.
2. **Latest-wins read workers**: one bounded scan slot and one bounded preview
   slot. Replaced requests do not accumulate. Results carry generation and
   filesystem epoch values.
3. **Serial mutation worker**: accepts at most one active batch in the first
   implementation. The UI remains available for browsing, help, inspection,
   and cancellation, but a second mutation is rejected with an accurate
   message rather than silently queued.

These are implementation boundaries, not user-facing product features.

### Separate intent, progress, and truth

The minimum operation model is concrete:

```text
OperationId
OperationKind
OperationRequest
ItemPlan
OperationEvent
ItemOutcome
CompletionState
OperationReport
```

An `OperationId` is 128 cryptographically random bits from the operating
system CSPRNG, serialized everywhere as exactly 32 lowercase hexadecimal
characters. In-memory collisions are retried. Every persisted bundle or marker
is reserved with a no-replace/create-new operation; a collision generates a
new ID and never opens or overwrites the existing object.

Every persisted top-level item also receives an independent `ItemId` with the
same 128-bit random and 32-lowercase-hex contract. It is not derived from a path
or user-visible item index.

`OperationRequest` and `ItemPlan` become immutable after preflight. They capture
source identity, destination-parent identity when relevant, the protected work
root, conflict policy, and requested operation.

One `ItemPlan` and one `ItemOutcome` correspond to one top-level selected target.
Recursive descendants contribute bounded counters for visited items, bytes, and
the first 20 escaped error summaries; they never create persistent per-descendant
plans or outcomes. `PartialEffect` enumerates only the bounded top-level effect
roles (`source`, `destination`, `staging`, `receipt`, and `payload`) and their
verified identities, never an unbounded descendant path list.

Progress events may be coalesced. `ItemOutcome` and the single terminal
`Finished` event may never be dropped. The aggregate completion state is
derived from item outcomes instead of being maintained separately.

The minimum item outcomes are:

- `Completed`;
- `CompletedWithWarnings`;
- `FailedNoEffect`;
- `PartialEffect`;
- `CleanupRequired`;
- `Indeterminate`;
- `CancelledBeforeCommit`;
- `NotStarted(reason)`, where reason is `Cancelled`, `AbortBatch`, or
  `ExecutorUnavailable`; and
- `DestinationCommittedSourceRetained`.

`FailedNoEffect` is legal only after Tersh proves that no user-visible effect
was committed. `PartialEffect` enumerates the exact committed and uncommitted
parts. `CleanupRequired` identifies an owned duplicate or residue and the
identity-checked cleanup action. `Indeterminate` is used when final state cannot
be proved; it is never reduced to success, failure-with-no-effect, or retry.
`DestinationCommittedSourceRetained` is the explicit cross-filesystem move
case whose cleanup path must not copy again.

`CompletionState` is exactly `Succeeded`, `SucceededWithWarnings`, `Cancelled`,
`Failed`, `Partial`, `NeedsCleanup`, or `Indeterminate`. A zero-item operation is
rejected before execution. Non-empty reports reduce deterministically in this
order:

For this reduction, a committed item is `Completed` or
`CompletedWithWarnings`; the higher-priority rules explicitly handle
`PartialEffect` and `DestinationCommittedSourceRetained`.

1. any `Indeterminate` item -> `Indeterminate`;
2. any `CleanupRequired` or `DestinationCommittedSourceRetained` item ->
   `NeedsCleanup`;
3. any `PartialEffect`, or any mixture of a committed item and a failed,
   cancelled, or not-started item -> `Partial`;
4. with no committed effect, any `FailedNoEffect` or
   `NotStarted(ExecutorUnavailable)` -> `Failed`;
5. only `CancelledBeforeCommit`, `NotStarted(Cancelled)`, or
   `NotStarted(AbortBatch)` -> `Cancelled`;
6. all items completed and at least one `CompletedWithWarnings` ->
   `SucceededWithWarnings`; and
7. all items `Completed` -> `Succeeded`.

The report always retains counts for every item outcome, so a higher-priority
aggregate does not hide lower-priority facts.

The final state uses first-final-wins. A late cancel event cannot relabel an
already committed item as cancelled. Each item executor boundary contains
panics. If an active worker panics or disconnects before its outcome arrives,
the coordinator uses the immutable plan, durable receipts, and read-only
filesystem observation to synthesize `CompletedWithWarnings`, `PartialEffect`,
`CleanupRequired`, `FailedNoEffect`, or `Indeterminate`; uncertainty always
chooses `Indeterminate`. Remaining items become
`NotStarted(ExecutorUnavailable)`. The coordinator emits the single `Finished`,
bumps `fs_epoch` on any effect or possible effect, requests authoritative
rescans, and then clears mutation fences. Disconnect is never assumed to mean
no effect and can never leave the UI permanently running.

### Make stale reads impossible to apply

Read results carry:

- cwd generation;
- focus or preview generation; and
- `fs_epoch`.

Accepting a mutation installs a UI-owned `MutationFence` for every affected
source and destination directory before the request is dispatched. While a
fence exists, scan or preview results intersecting those directories are held
or rejected even if their generation and `fs_epoch` still match. This closes
the channel race in which the filesystem commit happens before the terminal
mutation event reaches the UI.

Every mutation terminal event that committed or may have committed an effect
increments `fs_epoch`, schedules authoritative rescans, and only then clears
the corresponding fence. A read result is applied only if every generation,
epoch, and fence check succeeds. A failed reload keeps the last-known-good list
and selection, labels them stale, and exposes the error. It does not clear the
screen and destroy retry context.

Mutation staging and trash-internal directories are hidden from normal scans by
exact internal-name rules, not merely by the user's hidden-file preference.

### Make durability claims exact

Tier-1 mutation and recovery protocols require the applicable data or receipt
file sync followed by sync of every parent directory whose entry was created,
renamed, or removed. Initial receipt creation uses create-new, file sync, then
bundle-directory sync. Receipt updates write a same-directory create-new temp
file, sync it, atomically rename it over the Tersh-owned receipt, and sync the
bundle directory. Bundle publication and removal sync both affected parent
directories.

The supported regular-file metadata contract is byte content, Unix permission
mode, and modification time. A symlink preserves its raw link-target bytes and
is never followed. A directory copy preserves entry topology, directory mode,
and modification time after its children complete. Owner, group, ACL, xattr,
resource-fork, birth-time, and sparse-layout fidelity are explicitly outside
the first-release contract.

On both Tier-1 targets, a required sync failure is not silently downgraded. A
failure before publication is `FailedNoEffect` after owned staging cleanup is
verified. A failure after a visible effect is `CleanupRequired` or
`Indeterminate`, depending on whether final identity can be proved; source
cleanup is forbidden while durability is unproved.

### Claim a pathname before destructive action

An identity check followed by `rename` or `unlink` is not conditional on that
identity. Therefore permanent delete, same-filesystem move/rename/trash, EXDEV
source cleanup, cross-filesystem trash source cleanup, and restore-payload
cleanup all use one durable `SourceClaim` protocol. No destructive operation
acts directly on a rechecked user pathname.

The fixed private state root is
`~/Library/Application Support/Tersh/state/v1` on macOS. On Linux it is
`$XDG_STATE_HOME/tersh/v1` only when `XDG_STATE_HOME` is absolute, otherwise it
is `~/.local/state/tersh/v1`. It contains one
create-new control bundle per item under `transactions/pending/<item-id>/`; it
is not a shared append-only ledger. Every component is opened fd-relative with
no-follow semantics. System ancestors such as `/`, `/Users`, or `/home` may be
root-owned but must not be group- or world-writable. Starting at the user's home
directory on macOS, or at the selected absolute XDG state directory/default
home state directory on Linux, each component must be owned by the effective
user and must not be group- or world-writable. The Tersh state root and its
directory descendants are user-owned and mode 0700; receipt and identity files
are mode 0600. It stores a 128-bit installation ID in a mode-0600 file.

The installation ID file contains exactly 32 lowercase hexadecimal bytes with
no newline. First initialization creates `installation-id.tmp.<item-id>` with
create-new mode 0600, writes and syncs all 32 bytes, atomically no-replace
renames it to `installation-id`, and syncs the state root. Concurrent losers on
the final no-replace publish delete only their own verified temp file, then
open the winner no-follow and require a regular file, effective-user ownership,
mode 0600, exact length, and valid hex. Before using the winner, every process,
including a concurrent loser, must itself successfully sync the state-root
directory; validation without that sync never authorizes receipt or adjacent-root
creation. A missing winner after the race retries from the beginning. A
truncated, malformed, wrongly owned, wrongly permissioned, or non-regular winner
fails closed: it is never overwritten, repaired, or regenerated automatically,
and all destructive operations remain disabled. Crash injection covers temp
create/write/sync, publish, root sync, two-process first initialization, loser
cleanup, winner validation, and the paused-winner-after-rename/loser-before-sync
power-loss schedule.

Adjacent transaction roots use the exact name `.tersh-txn-v1`, mode 0700, and
an `owner.json` containing schema version and that installation ID. Every path
component is opened relative to a verified directory descriptor with no-follow
semantics. A symlink, wrong owner, unsafe mode, missing/mismatched owner record,
or untrusted pre-existing root causes a no-effect error; Tersh never adopts or
cleans it.

First creation builds `.tersh-txn-v1.init.<item-id>` with create-new semantics,
writes/syncs `owner.json`, syncs the init directory, then atomically no-replace
renames the complete directory to `.tersh-txn-v1` and syncs the parent. The
fixed control receipt records the init and final raw paths before creation. A
crash cannot publish a half-initialized trusted root; an owned init orphan is
reconciled only through that receipt.

No-replace rename means `renameatx_np(..., RENAME_EXCL)` on macOS and
`renameat2(..., RENAME_NOREPLACE)` on Linux. If the OS or mounted filesystem
returns unsupported, the item is rejected before source claim or publication;
there is no check-then-plain-rename fallback.

Before creating or changing an adjacent bundle, Tersh writes and syncs the
fixed-root control receipt. It contains operation/item IDs, action, expected
identity, authoritative raw source path, source-parent identity, authoritative
raw adjacent-bundle path, and any destination/payload identities. The control
states are `SourceClaimIntent`, `SourceClaimed`, `PublishOrCleanupIntent`,
`Committed`, `RestoreRequired`, `CleanupRequired`, and `Indeterminate`. The
control receipt is removed only after final facts and adjacent-bundle removal
are verified and both parent directories are synced. Startup scans this fixed
pending root, so recovery does not depend on the current directory.

The claim protocol is:

1. durably record `SourceClaimIntent(expected_identity, source_parent,
   tombstone_raw_path, action)` in the fixed root;
2. reserve an owned random tombstone path under the verified source parent's
   `.tersh-txn-v1/source-claims/<item-id>/` and sync its parents;
3. atomically no-replace rename the current source pathname to the tombstone
   `payload` and sync both affected parent directories;
4. reopen the tombstone no-follow and verify its identity against the intent;
5. on mismatch, no-clobber restore it to the raw original path and sync both
   parents. If restore cannot be proved, retain the tombstone and control receipt
   as `RestoreRequired`, `CleanupRequired`, or `Indeterminate`; never delete it;
6. on match, write/sync `SourceClaimed`, then perform only the recorded publish
   or cleanup action against the random path inside the private root; and
7. immediately before a tombstone unlink, re-open/no-follow and re-verify it.
   After the action, verify final source/destination/payload facts, write/sync
   `Committed`, remove owned markers, and sync their parents.

The private random tombstone prevents unrelated path replacement from becoming
the deletion target. A crash at every numbered boundary leaves the fixed control
receipt before any unique source can become undiscoverable. Failure to create or
sync the fixed receipt disables that destructive item before the claim rename.

### Define cancellation at safe points

An item moves through these conceptual states:

```text
queued -> running/pre-commit -> commit point -> committed -> reported
```

- Queued items can be cancelled with no side effect.
- Pre-commit file copy checks cancellation between bounded chunks and removes
  only a staging object whose identity it owns.
- A source-to-private-claim rename enters a non-interruptible claim-and-verify
  section but is not final publication. Cancellation afterward performs the
  recorded no-clobber restore when possible. A verified private-payload rename
  into the final no-clobber destination is at its commit point once issued.
- First-release permanent delete accepts regular files, symlinks, and empty
  directories only. Non-empty directory permanent delete is disabled; the UI
  directs the user to recoverable trash. This prevents a recursively deleted
  item from being misreported as `FailedNoEffect` after partial traversal.
- Once publish or source cleanup enters a non-interruptible critical section,
  cancellation is acknowledged but the critical section finishes before exit.

`q` may commit cwd only when no mutation is unresolved. `Q`, `Ctrl+C`, SIGINT,
SIGTERM, and SIGHUP express non-commit intent. Normal shutdown ordering is
exactly `Running -> ShutdownRequested(intent) -> request cancellation while
draining events and rendering status -> mutation terminal outcome -> restore
terminal -> return RunOutcome`. The application waits for the current safe
point instead of killing a mutation worker and guessing the disk state.

If rendering or terminal control has already failed, Tersh stops rendering,
immediately attempts terminal restoration, continues draining the mutation
non-interactively to a terminal outcome, and returns `Failed`; it emits no cwd.
Only a successful restoration followed by `CommitCwd` may write stdout. SIGKILL
and power loss are excluded from graceful terminal restoration, but durable
mutation protocols must reconcile their on-disk states after restart.

### Define process and launcher outcomes

The workbench returns exactly one of:

```text
RunOutcome::CommitCwd(path)
RunOutcome::AbortByUser
RunOutcome::Interrupted(signal)
RunOutcome::Failed(error)
```

Only `CommitCwd(path)`, after successful terminal restoration, writes the path
to stdout. The local process exit map is: `q`/commit `0`; `Q`/user abort `2`;
SIGHUP `129`; Ctrl+C or SIGINT `130`; SIGTERM `143`; and runtime, worker, or
terminal failure `1`. CLI usage or invocation validation errors use `64`, so
protocol-compatible exit `2` is not shared with argument parsing. Stdout remains
empty on every non-commit outcome.

The cluster launcher records `Closed`, `UserAborted`, `RemoteInterrupted`,
`LaunchFailed`, `TransportFailed`, or `Signaled`. The launcher generates a
128-bit nonce and performs one remote executable lookup only:

```text
exec tersh --remote-launch=tersh-exit-v1 \
  --nonce=<32-lowercase-hex> \
  --workdir-b64url=<RFC-4648-URL-safe-Base64-without-padding>
```

The workdir argument encodes the raw Unix path bytes, not a display string. That
single Tersh process validates the protocol, nonce, and decoded workdir,
changes directory, and, before emitting any terminal escape, writes one bounded
ASCII control frame:

```text
0x1e TERSH-LAUNCH-V1 READY <nonce> tersh-exit-v1 <semver>
  <source-commit-40hex> <cargo-lock-sha256> <asset-or-source>
  <diagnostic-build-id-or-dash> \n
```

The frame begins with the single byte `0x1e`, ends with one LF, contains only
printable ASCII between them, and is at most 512 bytes including delimiters. The
local PTY proxy buffers no more than 512 pre-ready bytes, requires and consumes
the complete frame within 5 seconds, checks the nonce and
exact protocol plus source commit/Cargo.lock pair against its offline
release-compatibility registry, then forwards all subsequent terminal bytes.
The registry is generated from release manifests and includes supported Tier-1
asset and Tier-2 pinned-source builds. Asset build ID is diagnostic only and is
never a compatibility key. An official source build must embed the verified
checkout commit and computed Cargo.lock hash; a build without both may run the
standalone workbench but cannot emit READY or be classified as a supported
remote launch.

Oversize, premature terminal output, timeout, EOF, malformed fields, or an
unknown source pair fail launch without interpreting the exit code as user
intent. The same already-running Tersh process enters the workbench directly;
it does not perform another `tersh` lookup or `exec`. A
binary or PATH replacement after process start therefore cannot change the exit
protocol being classified.

Only an SSH child that produced the valid frame is protocol-bound: its exit `2`
is `UserAborted`, and `129`, `130`, or `143` is
`RemoteInterrupted(code)`. Without the frame, every raw remote program code,
including `0`, `2`, `129`, `130`, and `143`, is `LaunchFailed(code)`; shell
command-not-found `127` is also `LaunchFailed`. SSH transport exit `255` is
`TransportFailed`, and a signal in the local OS child status is `Signaled`.
Every result preserves the original code plus bounded escaped diagnostic bytes.
These child classifications do not rewrite the local Tersh exit map.

## Seven Independently Reviewed Delivery Slices

The slices are ordered by dependency. They form one Trusted Core product line,
not seven unrelated releases.

### G0a: Release And Installation Truth

User outcome: a user can identify and install the version the documentation
claims.

Design:

- Keep v1.1.0 labeled as the latest stable release until a later release is
  actually published.
- Label development source honestly and do not describe v1.1.1 as a published
  release without matching tag, release note, and asset evidence.
- Pin stable installation to an immutable tag or revision. Prefer checksummed
  prebuilt artifacts after the release workflow is proven.
- Add locked format, lint, test, release-build, MSRV, dependency-advisory,
  installation, version, and help gates.
- Declare MSRV 1.88 and test it independently from the current stable toolchain.
- The Tier-1 prebuilt/native-smoke matrix is exactly
  `aarch64-apple-darwin` and `x86_64-unknown-linux-gnu`. A native smoke means
  executing the downloaded release binary on that operating system and
  architecture. `x86_64-apple-darwin` and `aarch64-unknown-linux-gnu` remain
  Tier-2 source-install targets only after the pinned tag builds natively and
  that native source-built binary passes the same PTY smoke. Without that
  evidence a target is `unverified`, not supported. Tier 2 publishes no prebuilt
  claim. Windows and every unlisted target are unsupported in this product line.
- Tier-1 macOS arm64 support is exactly macOS 14.5 build 23F79 or later on
  Apple Silicon. Release builds set `MACOSX_DEPLOYMENT_TARGET=14.5` and run on an
  arm64 macOS 14.5 build 23F79 snapshot with Xcode 16.2 build 16C5032a, macOS
  15.2 SDK, and rustc 1.95.0. `otool` must report minimum OS 14.5. The downloaded
  asset's native PTY smoke runs on the same minimum OS snapshot, not only a newer
  runner. The snapshot ID, disk-image SHA-256, Xcode/SDK build, Apple clang and
  linker versions are release inputs and manifest fields.
- Tier-1 Linux x86_64 support is exactly x86-64-v1, Linux kernel 4.18 or later,
  and glibc 2.28 or later. Release builds use rustc 1.95.0 and GCC 14 inside
  `quay.io/pypa/manylinux_2_28_x86_64:2026.05.07-2@sha256:443eabd378e140996780a772e12c1a1ef10551da933fe76d74a1bab61f68a7b7`
  on a native x86_64 runner with `-C target-cpu=x86-64`. Symbol-version
  inspection must show no GLIBC requirement above 2.28. The downloaded asset's
  minimum-environment PTY smoke runs natively in AlmaLinux 8.10 amd64 image
  `sha256:f043b7ac550015e1ed0b5a55a420c61d178bff4357ab9663fe0fbdcf1e6e2d86`.
- Tier-2 source-install evidence uses the same declared floors: macOS 14.5+
  x86_64 on a native Intel minimum-OS runner with Xcode 16.2/macOS 15.2 SDK,
  and Linux kernel 4.18+/glibc 2.28+ ARMv8-A built in
  `quay.io/pypa/manylinux_2_28_aarch64:2026.05.07-2@sha256:a435288af93def166dc59b5d052fa20ce59d76c6f38e8ad105767262d36843f0`
  and smoked natively in AlmaLinux 8.10 arm64 image
  `sha256:058da2bf381d460db9121940fbd035190ffbf28caec923cb9ba06c6e990da274`.
  Each pinned source tag must build and pass the PTY smoke under both MSRV 1.88
  and release rustc 1.95.0 on that target before the source-install claim exists.
- Generate and attach `release-manifest.json`. For each asset it records schema
  version, version, tag, full commit SHA, Cargo.lock SHA-256, rustc/toolchain,
  build ID (`ci-run-id.run-attempt.target`), target triple, asset filename,
  byte size, asset SHA-256, delivery tier, OS/kernel/libc/CPU floor, deployment
  target, build/smoke image digest or macOS snapshot hash, SDK, compiler, linker,
  and native-smoke status. A target with a skipped or failed minimum-environment
  native smoke cannot be labeled prebuilt- or source-install-supported.
- The manifest separately records `remote_protocol: tersh-exit-v1` and supported
  `(source_commit, Cargo.lock_SHA256)` pairs for every Tier-1 asset and Tier-2
  pinned-source build. Tier-2 native build gates check out the full recorded SHA,
  compute the lock hash, embed both into the binary, and exercise the READY
  frame. Build scripts fail remote-launch identity generation on a dirty checkout,
  missing commit, or lock mismatch. CI/asset build ID remains separate diagnostic
  provenance and is not required for a source-built remote.

These floors are intentionally narrower than the Rust target's theoretical
minimum. The evidence basis is the official
[Rust Apple target contract](https://doc.rust-lang.org/rustc/platform-support/apple-darwin.html),
[PyPA manylinux 2.28 image contract](https://github.com/pypa/manylinux), and
[Xcode 16.2 SDK/host requirements](https://developer.apple.com/documentation/xcode-release-notes/xcode-16_2-release-notes/);
Tersh's release claim remains governed by its own minimum-environment smoke.

Exit evidence:

- README, Cargo metadata, Changelog, release notes, and published-status wording
  are consistent.
- Each Tier-1 native job downloads its published candidate asset again, verifies
  filename, byte size, and SHA-256 against `release-manifest.json`, then executes
  that downloaded file for `tersh --version` and `tersh --help`. It also launches
  the file in a native PTY against a temporary directory, waits at most 5 seconds
  for a first frame containing the Tersh title and that directory, sends `q`,
  verifies exit `0`, and compares termios flags, cursor visibility, and
  alternate-screen exit against the pre-launch state. Tier-2 native jobs run
  this identical PTY smoke on the binary built from the immutable tag. A
  separate clean environment installs the immutable source tag under MSRV and
  current stable.
- CI success is necessary but does not substitute for the install smoke.

Out of scope: automatic updates, every package manager, and a full provenance
service.

### G0b: Existing Interaction And Result Truth

User outcome: current synchronous operations and exits stop lying or discarding
the information needed to recover.

Design:

- Introduce the concrete operation outcome/report model while the executor is
  still synchronous.
- Preserve the active operation and most recent completed mutation as full
  item-level reports in a scrollable Inspector or overlay. Retain at most 20
  older completed operations as bounded summaries containing operation ID,
  kind, counts by outcome, start/end time, and final state. Six-line logs remain
  summaries, not the source of truth. Explicit diagnostic export contains the
  active and most recent full reports plus those summaries; it never claims to
  be complete cross-restart history.
- Cap a preflight at 10,000 top-level targets. A larger selection is rejected
  before mutation with an exact count and no side effect. Descendants traversed
  by a supported copy or trash operation are streamed and do not expand an
  in-memory `ItemPlan` without bound.
- Partial completion removes only completed targets from retry context. A retry
  creates a new preflight; it never blindly replays an old action.
- Invalid goto, rename, copy-to, move-to, and conflict input preserves
  mode, text, captured targets, and an inline validation error.
- Implement the exact `RunOutcome`, stdout, exit-code, signal, and cluster child
  classification contract above.
- Make cluster footer and help text describe only the refresh behavior that is
  currently implemented.

Exit evidence:

- State-machine and PTY/shell-wrapper tests prove commit and abort behavior.
- PTY tests cover q, Q, Ctrl+C/SIGINT, SIGTERM, SIGHUP, runtime failure, terminal
  restoration failure, usage error 64, stdout emptiness, and every declared exit
  code.
- A mixed-success batch exposes every item outcome and keeps failed targets
  available for a new preflight.
- A 10,001-target request is rejected before executor entry; report retention
  never exceeds two full reports and 20 older summaries.
- A remote launcher exit 127 is displayed as failure, never normal close.
- Remote-launch tests cover Tier-1 asset and Tier-2 pinned-source identities,
  replace PATH and the executable after READY, replay a wrong nonce/source pair,
  vary diagnostic build ID, omit/truncate/oversize the control frame, and return
  every reserved code; only the source-compatible already-bound process receives
  user-intent classification.
- A 40x10 terminal can see an operation summary, exit/cancel controls, and a
  route into scrollable detail.

### G1a: Read Responsiveness

User outcome: navigation, filtering, and preview remain controllable while the
filesystem is slow.

Design:

- Move directory scan and preview off the event path into separate dedicated
  latest-wins workers.
- Keep one replaceable pending request per worker. Do not accumulate threads or
  an unbounded queue.
- Render a loading or stale-last-good state immediately.
- Apply only results whose cwd/focus generation and `fs_epoch` match and whose
  directories are not covered by a `MutationFence`.
- Install mutation fences before worker dispatch. Terminal mutation processing
  increments the epoch when an effect did or may have occurred, schedules
  relevant refreshes, and then clears the fences.
- Unknown totals use processed counts or indeterminate activity, never a fake
  percentage.

Exit evidence:

- The frozen 200-input, injected-1,000-ms benchmark meets key acknowledgement
  p95 <= 100 ms and longest event-loop stall <= 100 ms on the named reference
  environment.
- Rapid cursor and cwd changes never show a stale preview or directory result.
- A mutation followed by a late scan cannot visually resurrect removed or moved
  entries.
- The pending queues remain bounded under repeated input.

### G1b: Mutation Truth

User outcome: a long or partial operation is observable, stoppable at honest
safe points, and completely explained.

Design:

- Move copy, same-filesystem move, rename, trash, and permanent delete into the
  serial mutation worker.
- Reject a second mutation while one is active in the first release; browsing
  and read-only inspection remain available.
- Recheck source, target, target-parent, work-root, and trash-root identities at
  the final safe point.
- Use no-follow opens and filesystem-level no-replace primitives. Single-process
  serialization is not treated as multi-instance protection.
- Same-filesystem move, rename, and trash first acquire the source through the
  durable `SourceClaim` protocol, verify the private claimed payload, and only
  then no-clobber publish that payload. Permanent delete likewise claims first
  and deletes only the verified private tombstone.
- Regular-file copy uses a private staging target and bounded chunks. The target
  is published only after successful byte copy, the exact mode/mtime contract,
  source identity recheck, and file sync; publication is immediately followed
  by destination-parent sync before the item can be reported complete.
- A supported directory copy builds the complete topology in an owned private
  sibling staging directory, applies final directory mode/mtime bottom-up, and
  publishes only the root with a no-clobber rename. Cancellation or failure
  before publish cannot expose a partial target; failed owned-staging cleanup is
  `CleanupRequired`.
- A symlink copy reads raw link-target bytes without following them, creates the
  link in an owned private sibling staging path, and publishes it with the same
  no-clobber rule. Recursive descendant progress remains bounded as defined by
  the one-top-level-plan contract; it is not an item-level history.
- First-release conflict policy is exactly `Skip` or `AbortBatch`. Replace is
  disabled for copy, move, rename, trash restore, and recovery cleanup. Existing
  backup-rename or overwrite paths are removed from the reachable product UI
  because an identity check followed by rename is not compare-and-swap and
  cannot exclude an external replacement race.
- Permanent delete accepts regular files, symlinks, and empty directories only.
  A non-empty directory request is rejected during preflight and points to
  recoverable trash; no recursive deletion begins.
- Retry is always a new preflight. Trash and permanent delete require renewed
  confirmation.
- Exiting with an active mutation follows the cancellation and safe-point model
  above.

Exit evidence:

- Each item has exactly one terminal outcome and each operation exactly one
  `Finished` event.
- Progress may be coalesced; item outcomes and `Finished` survive channel
  pressure.
- ENOSPC, EACCES, source drift, destination-parent replacement, target races,
  symlinks, worker panic, channel disconnect, terminal error, and cancellation
  all produce verified disk-consistent reports.
- Fault tests prove replace is unreachable, non-empty-directory permanent
  delete has no effect, and post-publication sync failures become
  `CleanupRequired` or `Indeterminate` rather than `FailedNoEffect`.
- Selection, retry drafts, Inspector summaries, and refresh decisions derive
  from the operation report rather than log strings.

### G1c: Limited Cross-Filesystem Move

User outcome: a supported cross-device move never deletes the only valid source
or reports a duplicate as a complete move.

First-release scope: regular files and symlinks. Cross-filesystem directory
moves, special files, ACL/xattr fidelity, and resumable transfer remain
unsupported until an independent fault matrix proves them.

G1c depends on the same durable receipt writer, parent-sync, per-bundle claim,
and reconciliation primitives used by G2. Those primitives are implemented
before G1c even though the complete trash UI is accepted later. A move uses an
owned bundle adjacent to the destination so staging and publication share a
filesystem:

```text
<destination-parent>/.tersh-txn-v1/
  staging/<item-id>/
    transition.json
    claim.lock
    payload
  claims/<item-id>/
  quarantine/<item-id>/
```

Before that adjacent bundle is created, the fixed state root receives and syncs
the per-item control receipt with byte-safe source, destination, and exact
adjacent-bundle paths plus their captured parent identities. Every adjacent
transition is mirrored into that control receipt before the operation can
advance to source removal. Startup discovery scans the fixed root from any cwd;
opening a destination directory is not required to find an unfinished move.

The active process holds an exclusive no-follow lock on `claim.lock`. A
reconciler must first acquire that lock and atomically rename the whole bundle
from `staging` to `claims`; failure of either step means `in use`, with no
cleanup. The exact internal directory is hidden from normal scans but remains
visible to doctor/recovery tooling.

The durable states are `Prepared`, `PayloadReady`, `PublishIntent`,
`DestinationPublishedSourceRemovalPending`, `Committed`, `CleanupRequired`,
and `Indeterminate`. Every transition uses the exact receipt update and sync
sequence defined above.

Protocol:

1. preflight source and destination-parent identities;
2. reserve and sync the fixed control receipt, then reserve the owned adjacent
   bundle and sync a `Prepared` transition;
3. copy in bounded chunks without following symlinks, verify bytes and the
   exact metadata contract, sync payload and bundle, then record
   `PayloadReady`;
4. recheck source and destination-parent identities and check cancellation;
5. durably record `PublishIntent`, enter the non-interruptible section, publish
   payload with a no-clobber atomic rename, and sync the destination parent;
6. durably record `DestinationPublishedSourceRemovalPending` in both adjacent
   and fixed control receipts and verify the published-destination identity;
7. execute the shared durable `SourceClaim` protocol against the original raw
   source path and expected identity. A mismatch is restored no-clobber or
   retained as recovery-required; it is never deleted;
8. delete only the matched private claim tombstone after its final no-follow
   identity check and sync the tombstone parent; and
9. record `Committed`, verify source absence plus destination identity, then
   remove the owned adjacent and fixed control bundles and sync their parents.

If source cleanup fails, the outcome is
`DestinationCommittedSourceRetained`. The product creates a cleanup-specific
preflight that rechecks both objects. It does not copy again. A crash may leave
two copies or an identifiable private staging object, but it must not remove the
only valid source. Reconciliation of `PublishIntent` distinguishes payload still
in staging from a matching published destination; it never guesses from path
existence alone. Source cleanup is available only after the durable record and
destination identity prove publication, and it always acts on a private claimed
tombstone rather than the original source pathname.

Exit evidence covers EXDEV, ENOSPC, permissions, cancellation at each boundary,
identity changes, target competition, symlinks, durability calls, cleanup
failure, crash/restart at every state transition, competing reconcilers, and
startup discovery of staging orphans from a different cwd through the fixed
control root.

### G2: Recoverable Trash

User outcome: a supported trashed item can be found, understood, and restored;
damage to recovery metadata never triggers automatic deletion.

The design is a **per-item crash-consistent receipt bundle**, not a claim of a
global atomic transaction:

```text
.tersh-trash/v1/
  staging/<unique-id>/
    receipt.json
    claim.lock
    payload
  items/<unique-id>/
    receipt.json
    claim.lock
    payload
  claims/<unique-id>/
    receipt.json
    claim.lock
    payload
  quarantine/<unique-id>/
```

The `v1` directory is user-owned mode 0700 with a mode-0600 `owner.json`
containing schema version and the same installation ID. It is initialized by
complete private temp-directory creation plus no-replace publish, and every
later open is fd-relative/no-follow. An untrusted, symlinked, mismatched, or
group/world-writable `v1` is rejected without moving or deleting legacy content.

Each receipt includes schema version, unique ID, operation ID, monotonic receipt
revision, state, original-parent identity, object kind and identity, timestamp,
transfer method, payload identity, and the authoritative original path. On the
supported Unix platforms the authoritative path object is exactly
`{"platform":"unix","encoding":"base64-raw-os-bytes","bytes_b64":"..."}`.
`display_path` is lossy presentation only and is never an input to restore or
cleanup. Windows path encoding is outside this product line.

Trash states are `Prepared`, `PayloadReady`, `PayloadPublished`,
`SourceRemovalPending`, `Committed`, `CleanupRequired`, `Indeterminate`, and
`Quarantined`. Every receipt create/update, bundle rename, and bundle removal
uses the exact sync rules above. A state may advance only after its named disk
fact is verified; reconciliation may never move a receipt backwards.

Protocol:

1. reserve a unique staging directory with create-new semantics, acquire its
   exclusive `claim.lock`, write/sync `Prepared`, and sync the staging parent;
2. same-filesystem trash obtains the expected source through `SourceClaim`, then
   no-clobber publishes the verified private tombstone into bundle `payload`.
   Cross-filesystem regular-file or symlink trash copies without following links
   while retaining the original source pathname;
3. verify/sync payload and metadata, then write/sync `PayloadReady`;
4. atomically rename the complete bundle from `staging/<id>` to `items/<id>` and
   sync both parents, then write/sync `PayloadPublished`;
5. if a copied source remains, write/sync `SourceRemovalPending`, immediately
   verify the published payload identity, then obtain the original source via
   the durable `SourceClaim` protocol. Delete only the matched private tombstone
   after its final no-follow identity check, and sync its parent; and
6. write/sync `Committed`. Any cleanup ambiguity becomes `CleanupRequired` or
   `Indeterminate`; no ambiguous object is deleted.

Startup reconciliation classifies bundles as recoverable, needs-cleanup,
incomplete, orphaned, or quarantined. It never automatically purges a corrupt
receipt or unrecognized payload. Reconciliation that can change disk first
acquires the no-follow lock and atomically renames the bundle into `claims`; if
another process wins either claim step, the loser reports `in use` and makes no
change. Damaged or contradictory bundles are logically quarantined in place and
excluded from list/restore actions. A physical move into `quarantine` requires
an explicit rescue action after exclusive claim; quarantine never deletes data.
Only a receipt in verified `Committed` state is offered by normal list/restore;
all other states route through reconciliation or rescue.

Restore rechecks the payload, destination parent, and original-parent identity.
The default conflict decision is skip. A safe rename requires an explicit
choice. Overwrite is not a first-release restore option. When the original
parent is missing or its identity changed, restore requires an explicit
destination. Restore uses the durable states `RestoreClaimed`,
`RestorePublishIntent`, `RestoreDestinationPublished`,
`RestorePayloadRemovalPending`, `Restored`, `CleanupRequired`, and
`Indeterminate`:

1. acquire `claim.lock`, atomically rename `items/<id>` to `claims/<id>`, sync
   both parents, and write/sync `RestoreClaimed`;
2. validate the raw-byte path, payload identity, chosen destination parent, and
   no-clobber target. `RestorePublishIntent` durably records the authoritative
   raw-byte destination path, destination-parent identity, payload identity,
   expected published identity, and whether publication is rename or staged
   copy before any publication begins;
3. publish by same-filesystem no-clobber rename when possible; otherwise copy
   into an owned sibling under the destination's `.tersh-txn-v1`, record the raw
   target path plus expected staged identity in `RestorePublishIntent`, verify
   and sync it, no-clobber publish it, and retain the trash payload;
4. sync the destination parent and write/sync
   `RestoreDestinationPublished`;
5. when a copied trash payload remains, write/sync
   `RestorePayloadRemovalPending`, verify both identities, then use the cleanup
   form of `SourceClaim` inside the owned claims bundle: no-replace rename
   `payload` to a random private deletion tombstone, verify it no-follow, delete
   only a match, and sync its parent; and
6. verify the destination identity and the protocol-required payload state
   (absent after same-filesystem rename; present until copied-payload cleanup,
   then absent). Only after those facts pass, write/sync `Restored`; afterward
   remove the owned claimed bundle and sync `claims`.

A crash at any numbered boundary leaves a state that reconciliation can inspect
without overwriting a destination or deleting the only valid copy. A
`RestorePublishIntent` with a missing payload is resolved only by matching the
recorded destination identity; otherwise it is `Indeterminate` and quarantined.

The CLI rescue surface is implemented first so the engine is testable:

```text
tersh trash list
tersh trash restore <id>
tersh trash restore <id> --to <directory>
```

The same slice is not complete until a thin TUI Recovery overlay can discover,
inspect, and restore items. Only then may documentation call trash recoverable.

Exit evidence fault-injects every receipt, payload, publish, and cleanup
boundary, including non-UTF-8 paths, ENOSPC, EACCES, parent replacement,
identity drift immediately before cleanup, ID collision, damaged/truncated
JSON, file-sync and directory-sync failure, process death after every durable
state, and multiple independent Tersh instances racing to restore or reconcile.

### G3: Cluster Companion Correctness

User outcome: an explicitly requested cluster refresh finishes the declared
scope, and workbench launch readiness and failure are truthful.

Design:

- A refresh sweep captures a fixed alias set and generation.
- A pending queue feeds at most 16 real active probes.
- Every probe starts in its own process group/session. Timeout and quit signal
  the entire group, not only the direct shell or SSH child.
- A slot is released only after its child is actually exited or killed and
  reaped. The probe deadline is 6 seconds. A timeout sends TERM, waits 500 ms,
  sends KILL to the group if needed, reaps the direct child, closes and joins its
  bounded output readers, and only then releases the slot; therefore real child
  concurrency can never exceed 16.
- A reaped completion or timeout immediately fills the next free slot until the
  sweep is terminal for every captured alias.
- Late results from an older token or generation cannot overwrite newer state.
- Pressing refresh during a sweep coalesces to one `refresh_again` request; it
  does not create an unbounded queue. That next generation starts only after
  every alias in the current generation is terminal and every child is reaped.
- Quit marks queued and active aliases `Cancelled`, prevents new probes from
  starting, terminates and reaps active children with the same TERM/KILL
  protocol, and closes the generation before the dashboard restores its
  terminal. Late messages are ignored by generation token.
- Tersh version and workdir readiness are checked only when the user attempts a
  workbench launch. Readiness is `Unknown` before that attempt. The immediate
  single-process remote-launch mode verifies protocol and workdir, emits the
  nonce/protocol/source-bound READY frame, and directly enters the TUI without a
  second lookup.
  Its result is stamped with launch-attempt ID and time, authorizes only that
  launch, and is not displayed later as current fleet state. It does not become
  a new periodic fleet-monitoring surface.
- Command-not-found, invalid workdir, SSH nonzero exit, and signal termination
  use the exact `Closed`/`UserAborted`/`RemoteInterrupted`/`LaunchFailed`/
  `TransportFailed`/`Signaled` classification and preserve bounded escaped
  stderr.

G3 does not add remote transfer, synchronization, credentials, agents, metrics,
topology, or session management. G3 does not block release of the workbench
Trusted Core; the G0b launcher truth fix does.

## UI Contract

The first release reuses the current Inspector and overlays. It does not add a
new top-level Operation Center.

Normal workbench UI adds only the states needed to tell the truth:

- `loading` or `stale` for read work;
- one active mutation summary with processed count/bytes;
- `cancel requested` or `stopping after current item`;
- terminal completion summary; and
- scrollable full reports for the active and most recently completed mutation,
  plus the bounded 20-summary session list, with retryable,
  cleanup-required, partial, and indeterminate items visibly distinct.

The Recovery overlay lists receipt ID, original display path, trashed time,
payload kind, and recovery state. It exposes inspect, restore, restore-to, and
close. It never puts overwrite on Enter.

At 40x10, survival behavior takes priority: current mode, the primary state or
error, cancel/back, help, and quit must remain visible. Detailed reports and
recovery records may require scrolling. At 80x24 and above, the Inspector shows
the active or most recent operation without hiding file navigation.

## Error And Recovery Rules

- Errors remain attached to the mode or operation that produced them.
- A validation error does not discard the user's text or captured target.
- A failed read preserves the last-known-good view and marks it stale.
- A `FailedNoEffect` or `CancelledBeforeCommit` item preserves its unique source.
  Any intentional permanent-delete commit or other visible effect is verified
  and reported as completed, partial, cleanup-required, or indeterminate rather
  than being relabeled as a no-effect failure.
- A partial mutation is not summarized as success.
- A cleanup-required duplicate is not offered as a normal retry.
- Unknown, corrupt, or contradictory recovery state is quarantined and reported.
- No automatic cleanup follows a lossy path decode, identity mismatch, corrupt
  receipt, or unknown staging object.
- Logs are useful summaries but never the only audit trail for an operation.
- Color is never the only carrier of running, partial, failed, stale, or
  recoverable state.

## Security And Data-Safety Invariants

Existing protected-path, path-identity, no-follow, and terminal escaping rules
remain mandatory.

New work must also prove:

- no target publication through a symlinked parent captured after preflight;
- no user-target overwrite in the first Trusted Core release; conflict can only
  skip the item or abort the batch;
- no deletion of the source before the destination is committed and verified;
- no destructive rename or unlink of a rechecked user pathname; all destructive
  source actions require a durable fixed-root intent, no-replace move into a
  trusted private tombstone, post-claim no-follow identity verification, and a
  final verification on that private path;
- no cleanup of staging, receipt, payload, or source without ownership and
  identity proof;
- no assumption that one Tersh process controls all filesystem actors;
- no unbounded request, progress, probe, or report channel;
- no secret, absolute path, hostname, or inventory content in an opt-in
  diagnostic report unless the user explicitly requests unredacted output; and
- no new network call in the workbench product path.

## Testing Strategy

### Slice-to-gate matrix

Every slice must pass all previously accepted slice gates plus its row below;
the seven post-feature cycles repeat and combine these gates but are not the
first time a safety or performance case is tested.

| Slice | Mandatory evidence before the slice is accepted |
| --- | --- |
| G0a | locked format, lint, unit/integration tests, release build, MSRV 1.88, current stable, advisory/license policy, exact target matrix, immutable-source install, re-downloaded Tier-1 assets, manifest checksum/size match, Tier-1 and Tier-2 native PTY smoke with terminal restoration |
| G0b | outcome/reducer and report-bound tests; 10,001-target rejection; mixed-result retry context; 40x10 UI; PTY q/Q/SIGINT/SIGTERM/SIGHUP/runtime/terminal/usage-64 cases; exact stdout and exit codes; nonce/protocol/source-bound single-process remote launch for asset and source builds; missing/malformed/oversize frame and PATH/binary replacement; cluster child classifications |
| G1a | generation/epoch/fence race tests; bounded latest-wins slots; slow scan/preview disconnect and panic; frozen 200-input latency gate; stale-last-good UI; rapid cwd/focus mutation race; idle CPU/memory baseline |
| G1b | exactly-one terminal outcome and reducer; all conflict choices; replace-unreachable and non-empty-directory-delete rejection; two-process installation-ID initialization plus corrupt winner; fixed-root and adjacent-root trust checks; source swap between preflight and claim; claim mismatch restore/retention; crash at every claim boundary; ENOSPC, EACCES, parent/symlink/target races, file/directory sync failure, cancellation at every safe point, worker panic/disconnect observation, terminal failure, 10,000-item report bound, mutation-fence integration |
| G1c | regular-file and symlink EXDEV; every durable state crash point; ENOSPC/EACCES; source swap before claim; target competition; cancellation boundaries; checksum/metadata/sync failure; duplicate cleanup; from-different-cwd fixed-root marker discovery; two-process claim race; unique-source preservation |
| G2 | trash and restore state transitions; every receipt/payload/publish/source-claim/cleanup crash point; raw non-UTF-8 paths; corrupted/truncated receipt; ID collision; source swap and parent replacement; file/directory sync failure; CLI list/restore; 40x10 Recovery overlay; two-process restore/reconcile races; quarantine with no deletion |
| G3 | 1/16/17/40-host sweeps; real process-group count <=16; timeout/quit group TERM/500-ms/KILL, child reap, and reader join; queued/active quit cancellation; coalesced refresh generation order; stale-result rejection; same-process nonce/protocol/source-bound READY/TUI and Unknown/attempt stamp; exact exit/signal/transport classifications; narrow and normal dashboard rendering |

### Contract and state tests

- exit intent and shell-wrapper commit/abort;
- modal validation retention;
- exhaustive item-outcome reduction, aggregate precedence, and worker-loss
  observation;
- exactly-one final event and first-final-wins races;
- selection and retry-draft reduction;
- scan/preview generation, `fs_epoch`, and dispatch-time mutation fences;
- bounded pending slots and worker disconnect; and
- cluster sweep, generation, coalescing, and shutdown.

### Filesystem integration tests

- regular, directory, and symlink copy within one filesystem;
- skip/abort conflict behavior and proof that target replace is unreachable;
- fault-injected EXDEV regular-file and symlink move;
- ENOSPC, EACCES, source drift, parent replacement, target race, cancellation,
  and cleanup failure;
- fixed-root control receipt, private adjacent-root validation, source-claim
  mismatch restore, retained tombstone, and different-cwd startup discovery;
- stale and orphan staging detection;
- trash receipt prepare, payload, publish, cleanup, reconcile, list, and restore;
- invalid UTF-8 filenames on Unix; and
- concurrent independent process attempts using no-replace boundaries.

### Terminal and UI tests

- PTY tests for q, Q, Ctrl+C/SIGINT, SIGTERM, SIGHUP, panic/error restoration,
  exact exit codes, empty non-commit stdout, `--print-cwd`, and the installed
  shell helper;
- TestBackend rendering at 40x10, 60x16, 80x24, 100x30, and 160x48;
- active, cancelling, partial, failed, cleanup-required, stale, recovery, and
  corrupt-receipt states;
- slow-worker input responsiveness; and
- low-bandwidth dirty-render behavior.

### Release and supply-chain tests

- locked format, lint, unit, integration, and release build;
- current stable and declared MSRV;
- Tier-1 native-smoke and Tier-2 source-install evidence without capability
  overclaim;
- macOS deployment-target/SDK/snapshot verification and Linux
  kernel/glibc/CPU floor, pinned build-image digest, symbol-version, and
  minimum-environment PTY verification;
- dependency advisory and license policy;
- clean install from immutable input;
- machine-readable manifest, Cargo.lock/source revision, asset size, re-download,
  and SHA-256 verification; and
- version/help/startup smoke.

### Performance and stability fixtures

- empty directory, 10k-entry directory, long Unicode names, and metadata errors;
- bounded small, large, binary, virtual, and slow previews;
- large regular-file copy, multi-item batch, recursive same-filesystem copy, and
  cancellation;
- 1, 16, 17, and 40-host cluster sweeps;
- worker panic/disconnect and repeated supersession;
- repeated terminal suspend/resume and resize; and
- bounded soak runs that record event-loop stalls, memory, CPU, and channel
  depth.

## Rollout And Compatibility

- The implementation begins from `codex/review@a7e7961` because it contains the
  v1.1.1 safety and UI hardening not present on main.
- Existing keybindings and default ASCII-compatible rendering remain unless a
  contract fix above explicitly changes semantics.
- Existing `.tersh-trash` directories without v1 receipts remain visible as
  legacy, non-indexed content. Tersh must not guess their original paths or
  auto-import/delete them. Documentation explains manual recovery.
- New recovery data uses a versioned directory so future schema evolution does
  not reinterpret legacy entries.
- Cluster inventory schema remains backward compatible; readiness data is
  runtime state, not a required new inventory field.
- No public release is called **Workbench Trusted Core** until G0a, G0b, G1a,
  G1b, G1c, and G2 plus their integrated workbench gates pass. G3 is not a
  prerequisite for that workbench label, but the full product-optimization task
  remains incomplete until G3 and the complete iteration protocol pass.

## Explicit Non-Goals

This design does not include:

- image, PDF, archive, or media preview expansion;
- content search, tabs, bookmarks, Git status, mouse, or configurable keymaps;
- plugin or theme package systems;
- target overwrite, full per-item interactive conflict workflows, or Replace
  All;
- permanent deletion of a non-empty directory;
- rollback of already completed items;
- resumable copy or cross-restart operation history;
- first-release cross-filesystem directory moves;
- ACL, xattr, owner, or group fidelity guarantees;
- default overwrite during restore;
- automatic trash purge or corrupt-record repair;
- default online telemetry;
- SFTP, remote transfer, synchronization, remote agents, or fleet monitoring;
- more cluster metrics, topology, launchers, or credentials; or
- a database, generic job runtime, global transaction system, or automatic
  updater.

## Implementation Planning Boundaries

This file is one product contract, but implementation is split into five
reviewable plans so a partially correct safety system is not hidden inside one
oversized diff:

1. G0a release/install truth plus G0b existing interaction/result truth.
2. G1a read responsiveness plus G1b serial mutation truth, including the fixed
   state root and durable `SourceClaim`/tombstone substrate required by every
   destructive G1b action.
3. Extend that durable receipt/claim/reconciliation substrate with G2 CLI trash
   and restore engine states.
4. G1c limited EXDEV move on that substrate plus the G2 TUI Recovery overlay
   and final G2 acceptance.
5. G3 cluster correctness plus repository-wide integration and documentation.

Each plan has its own requirement map, red tests, implementation checkpoints,
adversarial review, and acceptance evidence. Completing a plan does not imply
that a later delivery slice or the full user objective is complete.

## Multi-Agent Iteration And Evidence Protocol

The work uses five recurring review roles, scheduled in sequential waves because
the environment allows at most three subagents concurrently:

1. product outcome and scope;
2. architecture and state model;
3. implementation or focused diagnosis;
4. adversarial safety and failure analysis; and
5. independent verification and regression review.

Each iteration must produce evidence for planning, execution, focused tests,
cross-review, and integrated verification. A repeated review with no new finding
is recorded as such; it is not represented as an implementation change.

### Seven implementation iterations

1. G0a release and installation truth.
2. G0b existing interaction and result truth.
3. G1a read responsiveness.
4. G1b mutation truth, fixed control root, and durable source-claim/tombstone
   safety.
5. Extend the durable substrate with the G2 CLI recovery engine.
6. G1c limited EXDEV move, then the G2 TUI surface and complete G2 acceptance.
7. G3 cluster companion correctness plus full integrated product review.

Each iteration uses test-driven implementation, at least one adversarial review,
its exact slice-to-gate row, every previously accepted gate, and the complete
locked regression suite before the next iteration starts. ENOSPC, crash, signal,
concurrency, responsiveness, and durability cases therefore enter in the slice
that introduces their risk; none is deferred to post-feature hardening. G3 does
not delay release-ready workbench evidence; it is the seventh maintenance and
integration iteration because the public companion still requires verification.

### Seven post-feature performance, stability, and security iterations

1. Event-loop latency, read supersession, queue bounds, CPU, and memory.
2. Cancellation, terminal-event races, worker panic/disconnect, and shutdown.
3. ENOSPC, EACCES, identity drift, symlink, target race, and EXDEV fault matrix.
4. Crash points, staging discovery, receipt reconciliation, restore, and orphan
   isolation.
5. PTY, signal, terminal restoration, tmux/screen assumptions, narrow/mobile
   layouts, and low-bandwidth rendering.
6. MSRV, dependency advisory, locked reproducibility, install matrix, artifacts,
   checksums, and release rollback procedure.
7. Integrated scale/soak, regression, product-contract audit, documentation
   truth, and final requirement-by-requirement completion audit.

No iteration is accepted solely because a narrow test passes. Its declared user
outcome, safety invariants, failure cases, and integration surface must all have
authoritative evidence. The requested count of seven implementation and seven
post-feature cycles is mandatory process evidence, but a cycle count by itself
is never completion evidence.

## Workbench Trusted Core Release Acceptance

The workbench may be labeled release-ready only when:

- G0a, G0b, G1a, G1b, G1c, and G2 each have their exit evidence;
- their locked test, lint, release-build, fault, performance, PTY, install, and
  security gates pass together on the declared Tier-1 scope;
- README, help, Changelog, release wording, and actual workbench behavior agree;
- no unresolved P0/P1 data-safety, exit-intent, responsiveness, recovery, or
  installation issue remains; and
- independent product, architecture, safety, and code reviewers accept the
  integrated workbench diff.

This milestone does not claim that G3 or the user's full optimization objective
is complete.

## Full Task Acceptance

The full product-optimization objective is complete only when:

- every delivery slice has its exit evidence;
- all seven implementation iterations and all seven post-feature hardening
  iterations have recorded planning, review, execution, and verification;
- the complete locked test, lint, release-build, fault, performance, PTY,
  install, and security gates pass on their declared scope;
- README, help, footer, Changelog, release wording, and actual behavior agree;
- no unresolved P0/P1 data-safety, exit-intent, responsiveness, recovery,
  installation, or public-cluster-truth issue remains;
- the final diff receives independent product, architecture, safety, and code
  review; and
- a completion audit maps every requirement in this design to direct evidence.

Until then, the task remains active and no partial milestone is described as the
full product optimization objective.

## Normative Clarifications From Adversarial Review

The clauses below close implementation ambiguities found during the independent
plan review. They are part of the product contract. Where an earlier paragraph
can be read more than one way, these clauses control.

### Prepare mutations before freezing their durable identity

G0b may adapt the existing synchronous executor to the immutable report model,
but G1b moves filesystem preflight and durable reservation off the UI thread.
The G1b coordination sequence is exactly:

```text
MutationIntent + ephemeral SubmissionId
  -> background filesystem preflight and create-new reservations
  -> Prepared(final OperationRequest, ItemPlans, reservations, fence set)
  -> UI installs the final fences and returns FenceInstalled
  -> Started
  -> filesystem mutation
```

The UI performs only bounded syntax, selection-count, and confirmation checks
before submission. It installs conservative provisional fences from the raw
intent before placing the intent in the bounded worker slot. The worker then
captures current identities, generates the final `OperationId` and `ItemId`
values, and reserves every applicable fixed marker or bundle with create-new
semantics. A pre-existing candidate is never opened or overwritten; only that
candidate ID is regenerated. The request and item plans become immutable only
after all required reservations and final fence roots are known.

`Prepared` and its acknowledgement are non-droppable coordination events. The
worker cannot publish, rename, unlink, or otherwise create a user-visible effect
before `FenceInstalled`. A missing acknowledgement, cancellation, panic, or
disconnect before that point produces a terminal no-effect report only after
all created reservations are proved removed and their parents synced. Any
reservation whose cleanup cannot be proved remains discoverable and is reported
as `CleanupRequired` or `Indeterminate`; it is not silently abandoned while a
new ID is tried. Restore, trash, same-filesystem mutation, permanent delete, and
the EXDEV path all use this preparation boundary. The 10,000-target preflight
therefore never runs on the event/render thread.

### Use one claimed fixed control per item

There is at most one fixed
`transactions/pending/<item-id>/` control bundle for an `ItemId`. Its typed
outer protocol is `SourceClaimV1`, `TrashIngestV1`, `ExdevMoveV1`, or
`RestoreV1`. Trash, EXDEV, and restore embed their `SourceClaimState` as a
substate of that same outer envelope; `SourceClaim` never creates a second
fixed bundle for the same item.

An unclaimed control handle is read-only. Exclusive mutation consumes the
handle and returns an owning claimed handle that holds the no-follow lock for
the entire read/transition/remove sequence. Only that claimed handle may
advance a receipt or remove a terminal bundle. A transition validates the same
schema, protocol, operation and item IDs, the expected current revision,
revision `+1`, the legal edge, and the disk facts required by that edge. Raw
receipt replacement is not exposed. Lock acquisition order is fixed control
first, adjacent bundle second.

An adjacent trash or EXDEV receipt records local payload facts, but it is not a
second independent authority. Mirrored transitions use fixed mirror intent,
then the adjacent next revision, then fixed confirmation of that exact revision
and hash; source removal is forbidden unless both match. An adjacent-ahead
state, disagreement, or missing fixed control is inspect-only `Indeterminate`.
A cleanup-specific EXDEV attempt gets fresh report IDs but must reacquire and
continue the original fixed control's private tombstone; it creates no sibling control, source claim, or copy/publish path.

### Make fd-relative names capabilities, not unchecked strings

Every fd-relative API that accepts a single child component accepts
`RawUnixName`, never `OsStr`, `Path`, or a display string. This includes child
open/create/stat/lock/rename/unlink and receipt-file creation. Construction
rejects empty, `.`, `..`, slash, and NUL before a syscall. Directory enumeration
returns validated names or an inspect-only escaped observation; it never turns
an invalid entry into a mutation capability. No module exposes a raw directory
descriptor merely to bypass these APIs.

### Keep bounded read work without cross-kind supersession

G1a has one directory slot in the scan worker and one preview slot in the
preview worker. When G2 adds recovery discovery, the same scan worker owns two
concrete keyed replace slots: `Directory` and `RecoveryCatalog`. Requests are
latest-wins within their own kind, never replace the other kind, and are drained
with fair alternation. The bound is therefore two pending scan requests, not an
open-ended queue or generic job runtime.

### Identify and claim the exact observed recovery bundle

A recovery catalog key and action reference contain the record class, raw ID or
raw name, `BundleLocation`, and the no-follow observed bundle identity. Verified
records carry that reference. Pagination uses the complete key, a stable
location order, streaming fd-relative enumeration, and O(page-size) retained
memory. The scanner must not first accumulate all observations.

Before mutation, recovery opens the referenced object, verifies its identity,
acquires its no-follow claim lock, re-verifies the location/name/identity, moves
the whole bundle no-replace into `claims`, syncs both parents, and reopens and
validates the receipt/header. The catalog observation alone never authorizes a
mutation. If the same `ItemId` appears in more than one of staging, items,
claims, or quarantine, every occurrence is contradictory inspect-only and no
restore is offered. CLI lookup by ID succeeds only for exactly one verified
recoverable bundle.

### Drain truth before joining workers or child processes

Mutation shutdown closes new commands, requests cancellation, and drains
progress plus non-progress events without holding coordinator mutexes until the
single `Finished` is observed. Only then may it drop the receiver and join the
worker or observer. Render or terminal failure uses the same noninteractive
drain. A full bounded channel can never be joined before it is drained.

The outer cluster event loop remains the sole owner of `TerminalSession`.
Launching a remote workbench suspends the dashboard, transfers terminal I/O to
one proxy owner, and always terminates/waits/reaps the child, drains and joins
all stream readers, then resumes or restores the dashboard. Pre-READY timeout,
malformed frame, spawn/read failure, user interrupt, and panic use this same
guard. There is one stdin reader; PTY resize is forwarded; retained diagnostics
are bounded while excess output continues to be drained. `TerminalSession`
installs the process's sole signal broker before worker threads exist; suspension
transfers exclusive consumption of `SIGWINCH` and HUP/INT/TERM to the proxy, so
the synchronous proxy remains resizable and interruptible without a second
control thread, a caller-supplied receiver, or a still-running dashboard loop.

Probe completion is likewise per-record and fail-contained. A direct child exit
with descendants still holding pipes enters a bounded draining phase, followed
by TERM, 500 ms, KILL, reap, and reader join as needed. One probe or reader error
becomes that probe's terminal failure and cannot abandon other active records.
The first terminal intent is monotonic: startup failure or timeout cannot later
be weakened to cancellation by quit. Containment errors accumulate on the same
owning record until reap/join, and any such error makes the emitted terminal
result failed without erasing the original reason cleanup began. Bulk shutdown
attempts every record before returning an aggregate error.

### Build release evidence without circular trust

Remote READY identity for an official build is derived from the actual clean
checkout and computed `Cargo.lock`, not accepted from format-checked environment
variables alone. The build verifies the full Git commit, tracked and untracked
dirty state, lock hash, and compatibility-registry provenance; a mismatch may
build a standalone binary only without official READY identity. All identity
inputs participate in the build system's rerun rules.

Release assembly is staged:

```text
build
  -> canonical AssetDescriptor(commit/lock/name/size/hash)
  -> upload unverified draft assets
  -> native jobs re-download and validate that descriptor
  -> SmokeEvidence(descriptor hash, exact environment facts, result)
  -> assemble the final supported manifest
  -> independently re-download and validate manifest plus assets
```

The final manifest cannot be an input to the smoke result it is meant to attest.
macOS minimum evidence requires exact build `23F79`, not a `23F79+` comparison.
Linux kernel-floor evidence requires a native runner or VM whose recorded
`uname -r` is in the declared 4.18.x floor; a container that merely reuses a
newer host kernel is not minimum-kernel evidence. OCI images are recorded as
complete immutable `registry/repository@sha256:...` references.

Every focused test gate first lists tests and proves the intended exact test is
discovered exactly once before executing it. Zero discovered or zero executed
tests is a gate failure even when Cargo exits zero. Parameterized gates record
and validate their expected case count.

### Preserve review provenance across all fourteen cycles

Cycle evidence is append-only by role, wave, and attempt. Each record carries
the orchestrator-issued task/agent/run identity, start and end time, baseline
and candidate commit, parent finding references, resolutions, and direct gate
hashes. A cycle closes only with planning records, an execution record,
independent safety and verification records, and final reports from all five
roles on the same candidate. Later reports never overwrite earlier discussion
or failed attempts. External native, CI, or release evidence binds the exact
committed candidate; an evidence-only follow-up commit cannot retroactively
change the candidate it attests.

### Prevent deserialization and proof-token capability forgery

Private fields alone are not a capability boundary because derived
deserialization can populate them without invoking a constructor. `RawUnixName`
and `RawUnixPath` therefore use custom deserializers that decode, re-encode, and
require the exact canonical URL-safe-no-pad spelling, then rerun the same
component/path validation as live capture. A receipt cannot deserialize an
empty, dot, dot-dot, slash-containing, NUL-containing, padded, aliased, or
malformed child capability. Inspect-only observed names expose escaped display
and ordering only, never raw bytes or a conversion accepted by a child syscall.
Canonical receipt serialization and every no-follow receipt read/re-read are
bounded to 64 KiB; an oversized or concurrently growing receipt is inspect-only
or unreadable and can never allocate without bound or mint transition facts.

Likewise, a transition's disk facts are opaque verifier-issued tokens, not
serialized fact structs or caller-supplied booleans. They have private fields,
no public constructor, and no `Serialize`, `Deserialize`, or `Clone`
implementation. Each transition token binds installation, operation/item,
revision, retained identity, exact edge, and sync observations. Terminal
authority instead moves the original claim/lock/snapshot into a consuming typestate. The claimed control rechecks every binding, so a real token from a different bundle, revision, or transition cannot be replayed.
Every authorizing fact borrows or owns the actual no-follow lock and verified receipt snapshot until the consuming transition returns; a prior unlocked observation is not authorization. A transition consumes its token, and terminal verify/remove consumes its typestate, so neither can be applied twice even to the same bundle.
