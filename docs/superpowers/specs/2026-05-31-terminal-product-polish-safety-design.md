# Terminal Product Polish And Safety Design

Date: 2026-05-31

## Goal

Tighten Tersh's product logic, terminal UI, and safety baseline without turning this pass into a large architecture rewrite. The v1 outcome should make the visible shortcut contract truthful, keep narrow terminal workflows usable, prevent unsafe preview/delete behavior, and make the advertised cluster dashboard safer for real inventories.

## Product Principles

Tersh is a terminal-first file workbench plus a lightweight cluster status and launcher surface. The file workbench remains the primary product. The cluster screen is user-facing because it is documented in the README and CLI help, so it must have basic onboarding, validation, and safe scaling behavior.

The target environments are local shells, SSH sessions, tmux/screen, and phone or tablet SSH clients. Core workflows must remain keyboard-first, ASCII-safe, and useful at 80x24. Below that size the interface can degrade, but it must still show a way to navigate, preview, cancel, get help, and quit.

## V1 Scope

In scope:

- Mode-aware footer copy for the file workbench and cluster dashboard.
- Preview-mode key fixes for shortcuts already advertised by README/help/footer.
- Responsive footer/layout adjustments that keep survival actions visible on narrow and tiny terminals.
- Safer preview behavior: do not follow symlinks or open non-regular files during preview.
- Preview line-count cap in addition to the existing byte and line-length caps.
- Canonical destructive-operation guards for root, home, active work root, and trash directory.
- Destructive confirmation modals that show operation context: count and first target path.
- Cluster inventory validation for duplicate aliases, empty/control-character fields, option-like SSH fields, and unresolved proxy jumps.
- A practical cap on concurrent cluster probes.
- Regression tests for key, render, preview, filesystem, cluster, and CLI contracts.

Out of scope for this pass:

- Full async preview, directory scan, copy, or delete job manager.
- Configurable keymaps.
- Tab panel focus or overlay navigation.
- Esc-as-cancel behavior.
- Replace-all, rename-all, resumable copy jobs, or full conflict-resolution UI.
- Trash restore UI.
- Mouse support, themes, image preview, diff view, or deeper editor integration.

## Shortcut Contract

Normal file mode:

- `j/k` and arrows move.
- `Enter/l` opens a directory or fullscreen preview.
- `h` and `Backspace` go to parent directory.
- `/` filters, `:` goes to directory.
- `Space` marks, `a` selects visible items, `A` clears selection.
- `yy`, `yf`, `yr`, `ya` copy file/buffer/text variants.
- `x`, `p`, `c`, `m`, `n`, `d`, `D`, `e` keep their current meanings.
- `?` opens help, `q` quits from normal mode, `Q` and `Ctrl+C` force quit from non-input surfaces.

Text prompt and confirmation modes:

- Printable keys edit the input.
- `Backspace` edits.
- `Enter` submits.
- `Ctrl+G` cancels.
- `Ctrl+C` force quits.
- Normal shortcuts are not active and must not be advertised in the footer.

Preview mode:

- `j/k`, `Space`, and `PageDown` page down/up style navigation where existing behavior already did page-sized movement.
- `Up/Down` and `Ctrl+B/Ctrl+F` provide line movement.
- `PageUp/PageDown` must work because README/help/footer already advertise them.
- `gg/G`, `/`, `n/N`, `e`, `Enter`, `q`, `Ctrl+G`, and `Ctrl+C` keep documented meanings.

Cluster mode:

- Preserve `r`, `Enter`, `s`, `t`, `l`, `?`, `q`, `Ctrl+G`, and `Ctrl+C`.
- Do not remap cluster launcher habits in this pass.

## Layout Contract

File workbench:

- Wide `>=120`: file list, preview, and info panes.
- Medium `80..119`: file list and preview panes.
- Compact `60..79`: file list stays primary and includes compact focused-item/status context. Fullscreen preview remains reachable with `Enter`.
- Tiny `<60` or very short height: survival UI. Prefer visible mode and exit/cancel/help controls over dense shortcut lists.

Cluster dashboard:

- Wide `>=100`: host list and dashboard columns.
- Medium `72..99`: stacked host list and dashboard.
- Narrow `<72`: host list primary; `l` opens full-width detail/dashboard.
- Tiny or very short detail mode: compact route/status only; long commands and logs are trimmed first.

Footer rendering should prioritize exit, cancel/help, and current-mode primary action before lower-frequency commands. Long host/file rows should clip or truncate rather than wrap unpredictably into multiple rows.

## Safety Contract

Preview:

- `preview_file` must use `symlink_metadata` as the authority for whether a path is a regular file.
- Symlinks are shown as symlinks and are not followed by preview.
- Directories and special files show safe messages and are not opened.
- Text preview remains byte-capped and line-length-capped, with an additional line-count cap.

Destructive operations:

- `trash_path` and `permanent_delete` must canonicalize existing targets and protected roots before comparing.
- Refuse root, home, active work root, and `.tersh-trash`.
- Symlink deletion deletes only the link, not the target.
- Confirmation UI must show the operation type, target count, and first target path.

Cluster:

- Inventory aliases must be non-empty, unique, and free of control characters.
- SSH user/address/proxy fields must be non-empty when used, free of control characters, and not start with `-`.
- Proxy aliases must resolve to a configured jump host.
- Refresh must not spawn an unbounded number of concurrent probe threads.

## Acceptance

Commands:

- `cargo fmt --check`
- `cargo test --all-targets`

Focused checks:

- `cargo test --test app_keys`
- `cargo test --test render`
- `cargo test --test preview`
- `cargo test --test fs_ops`
- `cargo test --test cluster`
- `cargo test --test cli`

Tests should use semantic assertions against app state and rendered buffer landmarks, not full terminal snapshots.
