# Tersh

[English](README.md) · [简体中文](README.zh-CN.md)

[![Release](https://img.shields.io/github/v/release/QiushanHuang/Tersh)](https://github.com/QiushanHuang/Tersh/releases)
[![CI](https://github.com/QiushanHuang/Tersh/actions/workflows/ci.yml/badge.svg)](https://github.com/QiushanHuang/Tersh/actions/workflows/ci.yml)
[![Rust 1.88+](https://img.shields.io/badge/Rust-1.88%2B-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**A lightweight terminal file workbench for local and SSH sessions.**
Browse, preview and organize files where you already work. Open `tersh --c` to
check host health and launch a shell or Tersh on a selected machine.

![Tersh file workbench with preview, selection and contextual shortcuts](docs/images/workbench.png)

*Screenshots use synthetic demo data and a representative terminal color palette.*

## Why Tersh

| What you need | What Tersh provides |
| --- | --- |
| Quick file inspection | Filterable, sortable files; inline/fullscreen preview; search; editor handoff |
| Long-running file work | One cancellable background worker, progress and explicit partial results |
| Recoverable cleanup | Managed trash receipts, original-location confirmation and no-overwrite recovery |
| Several remote machines | SSH health probes, jump-host routes, host filtering/sorting and bounded trends |
| A phone, laptop or SSH client | Adaptive layouts, searchable actions, configurable keys and ASCII fallback |
| A small tool that stays quiet | One binary, bounded caches/history, no resident agent, redraws driven by changes |

## Install

Download a binary for your platform from [Releases](https://github.com/QiushanHuang/Tersh/releases),
or build from source with **Rust 1.88 or newer**:

```sh
cargo install --locked --git https://github.com/QiushanHuang/Tersh.git --tag v1.2.0 --bin tersh
```

To update an existing Cargo installation, add `--force`. For a local checkout:

```sh
git clone https://github.com/QiushanHuang/Tersh.git
cd Tersh
./scripts/install.sh
```

The installer uses an existing Tersh location or a writable user bin directory.
Set `TERSH_INSTALL_DIR` to choose a location. For remote file work, install Tersh
on the remote host and run it after connecting through your SSH client.

## Start here

```sh
tersh                                  # Browse the current directory
tersh /path/to/project                 # Browse another directory
tersh README.md                        # Preview a file
tersh --c                              # Host health dashboard
tersh --ui-profile desktop             # Rounded borders and Unicode trends
tersh --ui-profile mobile --theme contrast
tersh --ui-profile ssh                 # ASCII, adaptive hints, no animation
```

Press **`o`** for searchable actions or **`?`** for the current keymap.
Type an action name, select it with arrows/Tab, and press Enter. Escape returns
to the previous view. The following are defaults; custom bindings also update
menus, help and footer hints.

| Task | Keys |
| --- | --- |
| Move / open / parent | `j`/`k` or arrows · Enter · `h` or Backspace |
| Filter / hidden files / sort | `/` · `.` · `s`/`S` |
| Select / copy / cut / paste | Space · `yy` · `x` · `p` |
| Copy to / move to / rename | `c` · `m` · `n` |
| Jobs / request cancellation | `J` · Ctrl+X |
| Trash / recover / permanent delete | `d` · `u` · `D` |
| Preview search / next match / edit | `/` · `n`/`N` · `e` |
| Copy name / relative path / absolute path | `yf` · `yr` · `ya` |
| Quit / cancel / safe emergency exit | `q` · Esc or Ctrl+G · Ctrl+C |

Tersh uses `$VISUAL`, `$EDITOR`, then `nano` for editing. For a visual `cd`, source
[scripts/tersh-cd.sh](scripts/tersh-cd.sh) in your shell and use `tersh-cd`.

## File jobs and recovery

Copy, move, trash, permanent delete and restore run in one background worker.
Browsing remains available while a job runs; overlapping writes are refused.
`J` shows progress, completed items, failures, skips and remaining work.

Cancellation keeps completed items and removes this copy's incomplete output.
Replacement copies are staged before commit; existing directories cannot be
replaced. Delayed operations recheck the source and approved target identities.
Ctrl+C during a job requests cancellation and waits for cleanup before exiting.
A blocked filesystem syscall cannot be interrupted, and completed deletion
cannot be undone.

`u` opens the current work root's managed trash. Enter shows the original
location before restoring. Existing destinations are never overwritten. Bad
receipts are reported while other valid records remain available. Recovery
works across restarts, requires same-filesystem rename and recorded UTF-8 paths;
legacy unrecorded trash remains untouched.

## Hosts and trends

![Cluster detail with current metrics and bounded observation trends](docs/images/cluster.png)

```sh
tersh --c --cluster-config examples/servers.json
```

Edit a copy of [the example inventory](examples/servers.json) with your own
hosts. Its `.example` addresses are placeholders. Health probes use
non-interactive SSH with already trusted host keys and existing credentials.

| Task | Keys |
| --- | --- |
| Filter alias, address or role / clear filter | `/` · Backspace |
| Cycle sort / reverse | `v` · `V` |
| Refresh all / refresh selected | `r` · Enter |
| Expand detail / scroll detail | `l` · PageUp/PageDown |
| Open shell or SSH / open Tersh | `s` · `t` |

Filtering and sorting change the view, not the probe scope. Selection follows
the host alias; unknown metrics stay last in either sort direction. Remote `t`
requires Tersh on that host. Returning from the session restores the dashboard.

History keeps at most 60 observations per host in memory. Gaps mean missing or
failed observations. Load is the 1-minute load average, memory is used percent,
and probe duration covers the full collection operation. Samples are spaced by
observation, with the actual elapsed span shown. No extra probe is run for a graph.

## Make it yours

```sh
tersh --theme aurora
tersh --theme mono
tersh --no-motion
tersh --dump-keymap > keymap.json
tersh --keymap keymap.json
```

Themes: `btop`, `aurora`, `contrast`, `mono`. `NO_COLOR` disables colors.
Device presets choose border, graph glyphs, footer density and motion without
changing bindings or writing configuration files.

Custom bindings use a partial JSON map:

```json
{
  "files": {
    "copy": ["Ctrl+y"],
    "open_jobs": ["F5", "J"],
    "open_trash": ["F6", "u"]
  }
}
```

This replaces `yy` with Ctrl+Y and keeps single-letter alternatives for devices
without function keys. See [configuration](docs/configuration.md) for all ten
contexts, file precedence, chords, environment variables and inventory fields.

## Develop and contribute

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
```

[Contributing](CONTRIBUTING.md) · [Changelog](CHANGELOG.md) ·
[v1.2.0 release notes](docs/releases/v1.2.0.md)

Maintained by [QiushanHuang](https://github.com/QiushanHuang).
See [contributors and attribution](CONTRIBUTORS.md). Tersh is licensed under the
[MIT License](LICENSE); the existing copyright notice is preserved.
