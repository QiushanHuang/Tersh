# Cluster Route Dashboard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add route-map and bounded resource-bar visualizations to the existing `tersh --c` cluster status TUI.

**Architecture:** Keep cluster probing and app state unchanged. Add pure rendering helpers in `src/cluster_ui.rs`, driven by existing `HostConfig`, `HostSnapshot`, and `ProbeReport` data. Expand `tests/cluster.rs` render assertions before implementation.

**Tech Stack:** Rust, Ratatui, Crossterm test backend, existing cluster module.

---

## File Structure

- Modify `tests/cluster.rs`: add render tests for route map, resource bars, direct/local route behavior, and narrow detail mode.
- Modify `src/cluster_ui.rs`: split selected-host dashboard rendering into route and detail sections; add resource bar parsing/render helpers.
- Do not modify `src/cluster.rs` unless tests reveal an unavoidable public accessor gap.

## Task 1: Add Render Tests For Route And Metrics

**Files:**
- Modify: `tests/cluster.rs`

- [ ] **Step 1: Add failing wide-layout assertions**

Add assertions to `cluster_render_shows_host_list_detail_metrics_and_footer_keys` so the rendered buffer must contain:

```rust
assert!(buffer.contains("Route"));
assert!(buffer.contains("LOCAL"));
assert!(buffer.contains("JUMP"));
assert!(buffer.contains("SERVER"));
assert!(buffer.contains("ssh -J"));
assert!(buffer.contains("joshua@100.90.116.54"));
assert!(buffer.contains("512/1024 MB (50%)"));
assert!(buffer.contains("8G/20G 40% used"));
```

- [ ] **Step 2: Add a direct/local route render test**

Add a test that selects the local host and asserts the buffer contains `LOCAL ONLY` and does not require `JUMP`.

- [ ] **Step 3: Add narrow detail route assertions**

Extend `cluster_render_narrow_detail_mode_shows_metrics` so it asserts `Route`, `LOCAL`, `JUMP`, `SERVER`, and existing metric labels.

- [ ] **Step 4: Run red tests**

Run:

```bash
cargo test --test cluster cluster_render
```

Expected: tests fail because the current UI does not render `Route`, route nodes, or route command text.

## Task 2: Implement Route Panel And Dashboard Split

**Files:**
- Modify: `src/cluster_ui.rs`

- [ ] **Step 1: Add a selected dashboard renderer**

Create `draw_dashboard(frame, area, app)` that splits enough vertical space into a route section and detail section, then calls `draw_route` and the existing detail renderer content.

- [ ] **Step 2: Wire layouts**

Use `draw_dashboard` for the right side of the wide layout and the lower area of the medium layout. In `Detail` mode, render the same dashboard instead of only raw detail.

- [ ] **Step 3: Implement route lines**

Create route helper functions that produce lines for:

```text
LOCAL ONLY
LOCAL => SERVER
LOCAL => JUMP => SERVER
ssh -J jump-target ssh-target
ssh ssh-target
```

Use `host.kind()`, `host.proxy_jump_target()`, and `host.ssh_target()`.

- [ ] **Step 4: Run route tests**

Run:

```bash
cargo test --test cluster cluster_render
```

Expected: route assertions pass; resource bar-specific assertions may still fail if not yet rendered.

## Task 3: Add Resource Bar Rendering

**Files:**
- Modify: `src/cluster_ui.rs`

- [ ] **Step 1: Add parsing helpers**

Implement pure helpers:

```rust
fn percent_from_token(input: &str) -> Option<u16>
fn clamp_percent(value: u16) -> u16
```

- [ ] **Step 2: Add bar line helper**

Implement a helper that renders label, bar, and raw value as one `Line`, using ASCII blocks such as `########----` so tests and terminals do not depend on Unicode glyphs.

- [ ] **Step 3: Use bars in detail metrics**

Replace plain `Memory`, `Storage`, and bounded `GPU` lines with bar lines that retain the raw text. Keep `CPU load` as text-only because the probe reports load average, not CPU utilization.

- [ ] **Step 4: Run focused tests**

Run:

```bash
cargo test --test cluster cluster_render
```

Expected: all cluster render tests pass.

## Task 4: Full Verification And Documentation Check

**Files:**
- Modify if needed: `README.md`

- [ ] **Step 1: Run formatting**

Run:

```bash
cargo fmt --check
```

Expected: passes.

- [ ] **Step 2: Run full tests**

Run:

```bash
cargo test
```

Expected: passes.

- [ ] **Step 3: Run build check**

Run:

```bash
cargo check
```

Expected: passes.

- [ ] **Step 4: Inspect docs**

Review the README `tersh --c` section. Only edit it if it now contradicts the UI behavior.
