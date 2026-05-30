# Terminal Product Polish And Safety Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Tersh's shortcut contract, responsive terminal UI, preview/delete safety, and cluster inventory/probe behavior match the approved v1 design.

**Architecture:** Keep the change local to existing Rust modules. Add small pure helper functions where they reduce repeated footer/layout/safety logic, but avoid a configurable keymap module or async job system in this pass.

**Tech Stack:** Rust 2024, ratatui 0.29, crossterm 0.28.1, serde/serde_json, existing integration tests.

---

## File Map

- Modify `src/app.rs`: preview key handling and destructive modal state accessors.
- Modify `src/ui.rs`: mode-aware footer, compact status context, clamped modal sizing, destructive modal context.
- Modify `src/preview.rs`: no-follow preview behavior, unsupported special-file messages, line-count cap.
- Modify `src/fs_ops.rs`: canonical destructive guards and trash-directory protection.
- Modify `src/cluster.rs`: inventory validation and refresh concurrency cap.
- Modify `src/cluster_ui.rs`: priority footer copy, tiny layout handling, no-wrap host rows.
- Modify `tests/app_keys.rs`: preview key regression and prompt mode behavior.
- Modify `tests/render.rs`: mode-aware footer and compact/tiny render assertions.
- Modify `tests/preview.rs`: symlink/special/no-follow and line-cap regressions.
- Modify `tests/fs_ops.rs`: canonical guard regressions.
- Modify `tests/cluster.rs`: inventory validation, concurrency cap, and cluster footer/tiny render checks.

## Task 1: Key And Footer Contract

**Files:**
- Modify: `tests/app_keys.rs`
- Modify: `tests/render.rs`
- Modify: `src/app.rs`
- Modify: `src/ui.rs`
- Modify: `src/cluster_ui.rs`

- [ ] **Step 1: Write failing tests**

Add tests asserting preview `PageDown/PageUp` changes `preview_offset`, prompt footers do not advertise normal shortcuts, preview search footer is prompt-specific, and cluster narrow footer keeps quit/help visible.

- [ ] **Step 2: Run tests to verify red**

Run:

```bash
cargo test --test app_keys preview_page_keys_scroll_fullscreen_preview
cargo test --test render prompt_footer_is_mode_specific
```

Expected: failures because preview page keys and prompt footers are not implemented correctly.

- [ ] **Step 3: Implement minimal behavior**

Update preview-mode key handling in `src/app.rs` and mode-aware footer rendering in `src/ui.rs` and `src/cluster_ui.rs`.

- [ ] **Step 4: Verify green**

Run:

```bash
cargo test --test app_keys
cargo test --test render
cargo test --test cluster cluster_render
```

Expected: all selected tests pass.

## Task 2: Preview Safety

**Files:**
- Modify: `tests/preview.rs`
- Modify: `src/preview.rs`

- [ ] **Step 1: Write failing tests**

Add tests for symlink preview no-follow and line-count capped text preview.

- [ ] **Step 2: Run tests to verify red**

Run:

```bash
cargo test --test preview symlink_preview_does_not_follow_target
cargo test --test preview many_short_lines_are_capped
```

Expected: failures because symlink targets are followed and line count is uncapped.

- [ ] **Step 3: Implement minimal behavior**

Use `symlink_metadata` file type checks before opening. Return message previews for symlinks and unsupported file types. Add a constant line cap.

- [ ] **Step 4: Verify green**

Run:

```bash
cargo test --test preview
```

Expected: preview tests pass.

## Task 3: Destructive Operation Guards And Context

**Files:**
- Modify: `tests/fs_ops.rs`
- Modify: `tests/render.rs`
- Modify: `src/fs_ops.rs`
- Modify: `src/app.rs`
- Modify: `src/ui.rs`

- [ ] **Step 1: Write failing tests**

Add tests that deletion/trash reject the work root through canonical paths and that confirmation modals render target count plus first target path.

- [ ] **Step 2: Run tests to verify red**

Run:

```bash
cargo test --test fs_ops delete_rejects_canonical_work_root
cargo test --test render delete_confirmation_names_target
```

Expected: failures because canonical variants and modal context are not covered.

- [ ] **Step 3: Implement minimal behavior**

Canonicalize protected paths in `fs_ops`, reject `.tersh-trash`, and expose operation target summary from `App` for modal rendering.

- [ ] **Step 4: Verify green**

Run:

```bash
cargo test --test fs_ops
cargo test --test render
```

Expected: tests pass.

## Task 4: Cluster Validation And Probe Cap

**Files:**
- Modify: `tests/cluster.rs`
- Modify: `src/cluster.rs`
- Modify: `src/cluster_ui.rs`

- [ ] **Step 1: Write failing tests**

Add tests that duplicate aliases, unresolved proxy jumps, control characters, and option-like SSH fields are rejected. Add a probe cap test through `begin_refresh`.

- [ ] **Step 2: Run tests to verify red**

Run:

```bash
cargo test --test cluster inventory_rejects_duplicate_aliases
cargo test --test cluster begin_refresh_caps_concurrent_hosts
```

Expected: failures because validation and cap are missing.

- [ ] **Step 3: Implement minimal behavior**

Validate inventory during `from_json`, cap newly-started refresh aliases, and adjust footer copy for narrow cluster views.

- [ ] **Step 4: Verify green**

Run:

```bash
cargo test --test cluster
```

Expected: cluster tests pass.

## Task 5: Responsive Render Polish

**Files:**
- Modify: `tests/render.rs`
- Modify: `tests/cluster.rs`
- Modify: `src/ui.rs`
- Modify: `src/cluster_ui.rs`

- [ ] **Step 1: Write failing tests**

Add render tests at compact and tiny sizes that assert survival controls remain visible and selected context remains present.

- [ ] **Step 2: Run tests to verify red**

Run:

```bash
cargo test --test render compact_layout_keeps_survival_controls_visible
cargo test --test cluster cluster_render_tiny_keeps_exit_visible
```

Expected: failures where long footer strings clip important controls.

- [ ] **Step 3: Implement minimal behavior**

Use width-tier footer text and compact detail/status lines. Prefer clipping stable one-line rows over wrapping.

- [ ] **Step 4: Verify green**

Run:

```bash
cargo test --test render
cargo test --test cluster cluster_render
```

Expected: render-focused tests pass.

## Task 6: Full Verification And Review

**Files:**
- All changed files.

- [ ] **Step 1: Format and full test**

Run:

```bash
cargo fmt --check
cargo test --all-targets
```

Expected: both commands pass.

- [ ] **Step 2: Request code review**

Dispatch subagents focused on UI/key behavior, safety/performance, and test adequacy. Fix critical and important findings.

- [ ] **Step 3: Repeat verification**

Run:

```bash
cargo fmt --check
cargo test --all-targets
```

Expected: both commands pass after review fixes.

## Self-Review

Spec coverage:

- Shortcut contract covered by Task 1.
- Preview safety covered by Task 2.
- Destructive guards and context covered by Task 3.
- Cluster validation/probe cap covered by Task 4.
- Responsive/tiny rendering covered by Task 5.
- Full verification and review covered by Task 6.

Placeholder scan: no deferred implementation placeholders remain inside v1 task steps.

Type consistency: all named functions and modules refer to existing repo surfaces or helpers to be introduced in the listed task.
