# Terminal File Workbench Design

Date: 2026-05-12

## Goal

Build a standalone full-screen terminal file workbench for SSH sessions. The app should feel closer to a lightweight VS Code Explorer inside the terminal than to a shell prompt extension: users launch one command, browse directories, preview text files, inspect file status, and perform safe copy, paste, rename, and delete operations without repeatedly typing `cd`, `ls`, `cat`, `cp`, and `rm`.

The MVP is an explorer, not an editor, synchronizer, shell, or full remote IDE.

## Product Direction

Tersh launches as an independent command named `tersh` and enters the terminal alternate screen. It is keyboard-first and must work well in local shells and after `ssh user@host` on ordinary Linux/macOS servers.

The interface uses dense btop/lazygit-style terminal information hierarchy, but the workflow is file-oriented:

- Browse current directory and parent/child directories quickly.
- Preview text files without opening an editor.
- Inspect file metadata: type, size, mode, modified time, symlink target, and operation state.
- Copy, paste, rename, and delete selected files.
- Show recent operations and failure messages clearly.

The first version deliberately excludes file editing, global full-text search, directory size recursion, Git auto-scanning, local-to-remote transfer, multi-host tabs, image preview, binary preview, diff views, and shell embedding.

## Recommended Technical Approach

Use Rust with Ratatui and Crossterm.

Rationale:

- A single static-ish binary is easier to deploy to remote machines than Python or Node dependency trees.
- Rust gives strong path and filesystem control for a tool where data loss is the highest risk.
- Ratatui supports rich terminal layouts and test backends.
- Crossterm handles raw mode, alternate screen, keyboard events, terminal size, and cross-platform terminal control.

Alternatives considered:

- Go + Bubble Tea: strong candidate with simple deployment and a clear update model, but complex multi-pane layouts and low-level filesystem safety are less direct than Rust/Ratatui for this product.
- Python + Textual: fastest prototype path and good component model, but the runtime and dependency footprint are worse for SSH servers.
- Node Ink/Blessed: not recommended for the core app because the runtime footprint and maintenance profile are weaker for this use case.

## Architecture

The app is a single-process TUI with a foreground event/render loop and background worker jobs for filesystem I/O. The UI must remain responsive while directories load, files preview, or copy/delete tasks run.

Core modules:

- `cli`: parses startup arguments such as initial path, theme, hidden-file mode, safety settings, and compact mode.
- `app`: owns global state, focused panel, selection, cursor position, copy buffer, modal state, and command dispatch.
- `fs_core`: defines file entries, metadata, operation plans, path normalization, display escaping, and conflict decisions.
- `fs_backend`: wraps local POSIX-style filesystem operations using Rust path APIs, not shell command strings.
- `preview`: detects text/binary files, reads bounded previews, handles invalid encodings, truncates long lines, and escapes control characters.
- `jobs`: runs directory scans, preview reads, copy, paste, rename, and trash/delete jobs off the UI loop, with progress and cancellation events.
- `safety`: applies destructive-operation guards, trash behavior, root/home/current-working-directory blocks, symlink rules, and confirmation requirements.
- `ui`: renders the layout, list panes, preview pane, compact inspector, status bar, help overlay, confirmation modals, progress, and operation log.
- `tests`: covers path handling, filesystem operations, terminal rendering snapshots, and terminal recovery behavior.

Data flow:

1. Terminal input is converted into app commands.
2. App commands update state immediately when safe, or create a job request for filesystem work.
3. Background jobs send progress, success, or failure events back to the app.
4. The app updates visible state and triggers a render.
5. Render code is pure with respect to filesystem side effects.

## Layout

The right sidebar should be compact because its value is secondary to navigation and preview. It should not take equal weight with the main panes.

Default wide layout at 120 columns and above:

- Left navigation/list pane: 32%.
- Center preview pane: 50%.
- Right inspector/log pane: 18%.
- Bottom command/status bar: 2 rows.

Medium layout from 80 to 119 columns:

- Left navigation/list pane: 42%.
- Center preview pane: 58%.
- Inspector collapses into the bottom status area and a toggleable overlay.

Small layout below 80 columns:

- Single-pane list mode by default.
- `Tab` toggles list, preview, and inspector overlays.
- The app remains usable enough to exit, navigate, and inspect, but shows a compact-mode notice.

Minimum target:

- Fully usable at 80x24.
- Graceful degraded mode below 80x24.
- No core workflow depends on mouse support, color, Nerd Font, or Alt/Option key combinations.

## Keyboard Model

Keyboard behavior should copy mature terminal products where possible: vim-like navigation, lazygit/ranger-style panels, predictable `Ctrl+G` cancel behavior, and obvious quit commands.

Navigation:

- `j` / `Down`: next item.
- `k` / `Up`: previous item.
- `Ctrl+d`: half-page down.
- `Ctrl+u`: half-page up.
- `gg`: first item.
- `G`: last item.
- `h` / `Backspace`: parent directory.
- `l` / `Enter`: enter directory or open focused file in preview.
- `/`: filter current directory.
- `Ctrl+G`: close filter, close modal, clear transient state, or return focus to the list.
- `r`: refresh current directory.
- `.`: toggle hidden files.

Selection and panels:

- `Space`: toggle selection.
- `a`: select all visible items.
- `A`: clear selection.
- `Tab`: next panel or overlay.
- `Shift+Tab`: previous panel or overlay.

File operations:

- `y`: copy selected items, or focused item if nothing is selected.
- `p`: paste copy buffer into current directory.
- `n`: rename focused item.
- `d`: move selected items to app trash.
- `Shift+D`: permanent delete, guarded by typed confirmation.

Exit and help:

- `q`: quit when no modal/filter is open; otherwise closes the active modal/filter first.
- `Q`: force quit after terminal state restoration, without running pending destructive actions.
- `Ctrl+c`: emergency quit with terminal state restoration.
- `Ctrl+G`: universal cancel/back action.
- `?`: help overlay with keybindings and safety rules.

No destructive modal may default to a destructive action on `Enter`. The default focused action is always `Cancel`, `Skip`, or another non-destructive choice.

## Preview Behavior

Preview must be safe and bounded.

- Read the first 64 KiB to detect binary/text and validate UTF-8 enough for display.
- Text preview limit: 2 MiB or 20,000 lines, whichever comes first.
- Long lines are soft-wrapped in the UI but hard-truncated after 4,096 bytes with a visible truncation marker.
- Control characters and embedded ANSI escapes are escaped before display.
- Binary files show metadata and a short hex summary, not raw content.
- Preview jobs are asynchronous; if a preview takes more than about 50 ms, the UI shows a loading state and remains responsive.
- The MVP does not edit files.

## File Operations

All operations use filesystem APIs and path objects, never shell string concatenation.

Copy buffer:

- Internal to the app, independent from the system clipboard.
- Stores absolute source paths, operation type, and basic source metadata captured at copy time.
- `p` pastes into the currently displayed directory.

Paste:

- Directories copy recursively.
- Symlinks copy as symlinks by default; the app does not follow symlink targets during recursive copy.
- Preserve file mode and modified time in MVP.
- Owner, group, ACLs, and xattrs are not guaranteed in MVP and are reported as out of scope.
- Name conflicts open a modal: `Skip`, `Rename`, `Replace`, `Abort`.
- Default conflict focus is `Skip`.
- `Replace All` requires typing `replace`.

Rename:

- Uses atomic rename where the filesystem supports it.
- Refuses to overwrite by default.
- Shows the full old and new paths before applying.

Delete:

- `d` moves items to an app trash directory when possible.
- Trash path is `.tersh-trash` under the current work root or a configured safe trash directory.
- Trash entries include enough metadata to make recent manual recovery understandable.
- If trash is unavailable, the app asks before falling back to permanent delete.
- `Shift+D` is permanent delete and requires stronger confirmation.

Hard blocks:

- Refuse to delete filesystem root.
- Refuse to delete the user's home directory.
- Refuse to delete the active work root without typing the full absolute path.
- Refuse recursive deletion through symlink targets.

Typed confirmation is required for:

- Permanent delete.
- Non-empty directory delete.
- More than 10 selected objects.
- More than 100 MiB estimated total size when known.
- Replacing existing files in bulk.

## Status and Inspector

The compact right pane is secondary. It contains:

- Focused path.
- File type, size, mode, modified time.
- Symlink target when applicable.
- Current copy buffer summary.
- Current background job and progress.
- Recent operation log.
- Latest error.

Git status is not part of MVP auto-scan. A post-MVP version may add lazy Git status for the current repository only.

## Error Handling

Errors must be visible, specific, and non-destructive.

- Permission errors show the path and operation that failed.
- Partial batch operations show completed, skipped, failed, and pending counts.
- Cancelled jobs do not roll back completed file copies in MVP, but the app reports exactly what completed.
- The app never hides filesystem errors in the status bar only; the inspector/log must retain recent failures.
- Terminal cleanup runs on normal quit, `Ctrl+c`, panic, and common termination signals where practical.

## Terminal Compatibility

Baseline:

- xterm-compatible terminal.
- raw mode and alternate screen.
- keyboard-first operation.

Compatibility targets:

- Direct SSH.
- tmux and screen.
- iTerm2, macOS Terminal, WezTerm, Windows Terminal, VS Code terminal.
- 80x24 minimum.
- CJK-width text and non-ASCII paths.

Degradation:

- ASCII fallback for icons.
- Color is not the only state indicator.
- Mouse is optional and not required for any operation.
- High-frequency animations are disabled; rendering is event-driven and throttled to avoid wasting SSH bandwidth.

## Testing Strategy

Unit tests:

- Path normalization and display escaping.
- File names with spaces, quotes, newlines, leading dash, CJK, invalid UTF-8, and control characters.
- Selection state through refresh, filter, delete, and sorting.
- Conflict decision logic.
- Symlink behavior.
- Trash and destructive-operation guard decisions.

Integration tests:

- Copy file.
- Copy directory recursively.
- Copy symlink as link.
- Rename without overwrite.
- Paste conflict skip/rename/replace/abort.
- Trash delete.
- Permanent delete confirmation.
- Partial batch failure.
- Permission denied.
- Large directory listing.

Rendering tests:

- Ratatui `TestBackend` snapshots for 80x24, 100x30, and 160x48.
- Focus, selection, modal, compact mode, help overlay, and operation log states.

Terminal tests:

- Raw mode and alternate screen restoration.
- `q`, `Q`, `Ctrl+G`, and `Ctrl+c` behavior.
- Resize handling.
- Panic cleanup path.

Manual test matrix:

- Direct SSH.
- tmux.
- screen.
- iTerm2.
- macOS Terminal.
- WezTerm.
- Windows Terminal.
- VS Code integrated terminal.

## Internal Agent Workflow

Implementation should use a multi-agent workflow in two passes.

Initial implementation pass:

1. Code architecture agent proposes module layout and implementation slices.
2. Parameter-selection agent validates dependencies, keybindings, layout ratios, preview limits, safety thresholds, and test matrix.
3. Design-review agent checks scope creep, safety risks, terminal compatibility, and performance traps.
4. Code-review agent reviews the produced diff for filesystem safety, path handling, symlink behavior, terminal cleanup, and missing tests.

One internal optimization iteration after the initial pass:

1. Optimization architecture agent identifies focused refactors or missing boundaries from the first diff.
2. Parameter agent retunes thresholds, layout proportions, and keyboard behavior based on the first implementation.
3. Review agent checks whether the iteration improves safety and usability without expanding scope.
4. Code-review agent reviews the second diff and verifies regressions are covered by tests.

The implementation is not complete until this single internal iteration has run and the final verification commands pass.

## References

- Ratatui: https://ratatui.rs/
- Ratatui backends: https://ratatui.rs/concepts/backends/
- Crossterm: https://docs.rs/crossterm/latest/crossterm/
- Bubble Tea: https://github.com/charmbracelet/bubbletea
- Textual: https://www.textualize.io/
- Ink: https://github.com/vadimdemedes/ink
- Blessed: https://www.npmjs.com/package/blessed
