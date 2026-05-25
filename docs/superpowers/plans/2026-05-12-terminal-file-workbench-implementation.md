# Terminal File Workbench Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build Tersh, a standalone terminal file workbench for local and SSH shell sessions with directory browsing, text preview, compact status/log inspector, and safe copy/paste/rename/delete operations.

**Architecture:** Rust binary using Ratatui/Crossterm. Keep filesystem side effects out of rendering; model app state in testable pure modules; run preview and file operations through bounded helpers that can later move behind a background job queue.

**Tech Stack:** Rust, Ratatui, Crossterm, Clap, Anyhow, Tempfile for integration tests, and the standard library filesystem APIs.

---

## File Structure

- `Cargo.toml`: package metadata and dependencies.
- `src/main.rs`: CLI parsing, terminal setup, panic-safe cleanup, and app launch.
- `src/lib.rs`: module exports for tests.
- `src/app.rs`: app state, focus, selection, copy buffer, key command handling.
- `src/fs_core.rs`: file entry model, metadata formatting, safe display escaping, path helpers.
- `src/fs_ops.rs`: copy, paste, rename, trash, permanent delete guards, conflict decisions.
- `src/preview.rs`: bounded text/binary preview.
- `src/ui.rs`: Ratatui rendering for wide, medium, and compact layouts.
- `tests/fs_ops.rs`: integration tests for filesystem operations.
- `tests/preview.rs`: integration tests for preview safety.
- `tests/app_keys.rs`: key behavior tests for mature TUI shortcuts.
- `tests/render.rs`: Ratatui snapshot-style assertions using `TestBackend`.

## Task 1: Scaffold Rust Project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`

- [ ] **Step 1: Write the package manifest**

```toml
[package]
name = "tersh"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
crossterm = "0.29"
ratatui = "0.29"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Add module exports**

```rust
pub mod app;
pub mod fs_core;
pub mod fs_ops;
pub mod preview;
pub mod ui;
```

- [ ] **Step 3: Add a minimal main that compiles**

```rust
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "tersh", version, about = "Tersh is a terminal file workbench")]
struct Cli {
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    tersh::app::run(cli.path)
}
```

- [ ] **Step 4: Run compile check**

Run: `cargo check`
Expected: fails until later modules exist, then passes.

## Task 2: Core File Model and Display Safety

**Files:**
- Create: `src/fs_core.rs`
- Test: `tests/fs_core.rs`

- [ ] **Step 1: Write failing tests**

```rust
use tersh::fs_core::{escape_display, format_size};

#[test]
fn escapes_control_characters_for_terminal_display() {
    assert_eq!(escape_display("a\nb\t\u{1b}[31m"), "a\\nb\\t\\u{1b}[31m");
}

#[test]
fn formats_sizes_for_compact_terminal_rows() {
    assert_eq!(format_size(42), "42 B");
    assert_eq!(format_size(2048), "2.0 KiB");
}
```

- [ ] **Step 2: Verify red**

Run: `cargo test --test fs_core`
Expected: fail because `fs_core` functions do not exist.

- [ ] **Step 3: Implement file model helpers**

Create `FileKind`, `FileEntry`, `escape_display`, `format_size`, and metadata collection helpers using `std::fs::symlink_metadata`.

- [ ] **Step 4: Verify green**

Run: `cargo test --test fs_core`
Expected: pass.

## Task 3: Preview Engine

**Files:**
- Create: `src/preview.rs`
- Test: `tests/preview.rs`

- [ ] **Step 1: Write failing tests**

```rust
use std::io::Write;
use tersh::preview::{preview_file, PreviewKind};

#[test]
fn previews_utf8_text_with_line_numbers() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.txt");
    std::fs::write(&path, "alpha\nbeta\n").unwrap();
    let preview = preview_file(&path).unwrap();
    assert_eq!(preview.kind, PreviewKind::Text);
    assert!(preview.lines.iter().any(|line| line.contains("1  alpha")));
}

#[test]
fn binary_preview_never_emits_raw_control_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bin.dat");
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(&[0, 159, 146, 150, 27]).unwrap();
    let preview = preview_file(&path).unwrap();
    assert_eq!(preview.kind, PreviewKind::Binary);
    assert!(preview.lines.join("\n").contains("Binary file"));
}
```

- [ ] **Step 2: Verify red**

Run: `cargo test --test preview`
Expected: fail because preview API does not exist.

- [ ] **Step 3: Implement bounded preview**

Implement `Preview`, `PreviewKind`, `preview_file`, text/binary detection from the first 64 KiB, a 2 MiB read cap, escaped display lines, and binary metadata output.

- [ ] **Step 4: Verify green**

Run: `cargo test --test preview`
Expected: pass.

## Task 4: File Operations and Safety Guards

**Files:**
- Create: `src/fs_ops.rs`
- Test: `tests/fs_ops.rs`

- [ ] **Step 1: Write failing tests**

```rust
use tersh::fs_ops::{copy_path, rename_path, trash_path, DeleteDecision};

#[test]
fn copies_symlink_as_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.txt");
    std::fs::write(&target, "target").unwrap();
    let link = dir.path().join("link.txt");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &link).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&target, &link).unwrap();
    let copied = dir.path().join("copied-link.txt");
    copy_path(&link, &copied, false).unwrap();
    assert!(std::fs::symlink_metadata(&copied).unwrap().file_type().is_symlink());
}

#[test]
fn trash_moves_file_into_tersh_trash() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("old.txt");
    std::fs::write(&file, "old").unwrap();
    let decision = trash_path(&file, dir.path()).unwrap();
    assert!(matches!(decision, DeleteDecision::MovedToTrash { .. }));
    assert!(!file.exists());
    assert!(dir.path().join(".tersh-trash").exists());
}
```

- [ ] **Step 2: Verify red**

Run: `cargo test --test fs_ops`
Expected: fail because filesystem operation API does not exist.

- [ ] **Step 3: Implement safe operations**

Implement copy without shell commands, symlink-as-link copy, recursive directory copy, rename without overwrite, app trash move, root/home guard helpers, and conflict error reporting.

- [ ] **Step 4: Verify green**

Run: `cargo test --test fs_ops`
Expected: pass.

## Task 5: App State and Mature Keybindings

**Files:**
- Create: `src/app.rs`
- Test: `tests/app_keys.rs`

- [ ] **Step 1: Write failing tests**

```rust
use tersh::app::{App, Command, Mode};

#[test]
fn ctrl_g_closes_filter_before_quitting() {
    let mut app = App::for_test();
    app.apply(Command::OpenFilter);
    app.apply(Command::Cancel);
    assert_eq!(app.mode(), Mode::Normal);
    assert!(!app.should_quit());
}

#[test]
fn q_quits_only_from_normal_mode() {
    let mut app = App::for_test();
    app.apply(Command::OpenHelp);
    app.apply(Command::Quit);
    assert_eq!(app.mode(), Mode::Normal);
    assert!(!app.should_quit());
    app.apply(Command::Quit);
    assert!(app.should_quit());
}
```

- [ ] **Step 2: Verify red**

Run: `cargo test --test app_keys`
Expected: fail because app API does not exist.

- [ ] **Step 3: Implement state and commands**

Implement `App`, `Mode`, `Command`, directory loading, selection, copy buffer, log messages, command dispatch, and key-to-command mapping for `q`, `Q`, `Ctrl+G`, `Ctrl+c`, `?`, `j/k/h/l`, `Space`, `y`, `p`, `d`, `D`, `/`, `.`, and `r`.

- [ ] **Step 4: Verify green**

Run: `cargo test --test app_keys`
Expected: pass.

## Task 6: Ratatui Rendering and Terminal Loop

**Files:**
- Create: `src/ui.rs`
- Modify: `src/main.rs`
- Modify: `src/app.rs`
- Test: `tests/render.rs`

- [ ] **Step 1: Write failing render tests**

```rust
use ratatui::{backend::TestBackend, Terminal};
use tersh::{app::App, ui::draw};

#[test]
fn wide_layout_contains_compact_info_pane_and_quit_keys() {
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::for_test();
    terminal.draw(|frame| draw(frame, &app)).unwrap();
    let buffer = terminal.backend().buffer().to_string();
    assert!(buffer.contains("Files"));
    assert!(buffer.contains("Preview"));
    assert!(buffer.contains("Info"));
    assert!(buffer.contains("q quit"));
    assert!(buffer.contains("Ctrl+C exit"));
}
```

- [ ] **Step 2: Verify red**

Run: `cargo test --test render`
Expected: fail because render API does not exist.

- [ ] **Step 3: Implement UI and terminal runner**

Implement wide/medium/compact layout, list pane, preview pane, compact info/log pane, footer keys, help overlay, confirm overlays, terminal raw mode setup, alternate screen setup, event loop, and cleanup guard.

- [ ] **Step 4: Verify green**

Run: `cargo test --test render`
Expected: pass.

## Task 7: Full Verification and One Internal Iteration

**Files:**
- Modify as needed based on reviews.

- [ ] **Step 1: Run full verification**

Run: `cargo fmt --check && cargo test && cargo check`
Expected: all pass.

- [ ] **Step 2: Dispatch GPT-5.5 xhigh review subagents**

Use separate agents for architecture review, parameter review, design/safety review, and code review. Provide the spec, plan, and current diff. Do not accept completion until they return actionable feedback.

- [ ] **Step 3: Implement one optimization iteration**

Apply focused improvements from subagent feedback without expanding MVP scope.

- [ ] **Step 4: Re-run full verification**

Run: `cargo fmt --check && cargo test && cargo check`
Expected: all pass after iteration.
