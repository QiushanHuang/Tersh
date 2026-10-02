# Configuration

Tersh runs without a configuration file. CLI options and environment variables
affect the current process; presentation presets do not write user settings.

## Presentation

| Option | Values / effect |
| --- | --- |
| `--ui-profile desktop` | Full hints, rounded borders, Unicode trends, activity animation |
| `--ui-profile mobile` | Compact hints, ASCII, no animation |
| `--ui-profile ssh` | Adaptive hints, ASCII, no animation |
| `--theme` / `TERSH_THEME` | `btop`, `aurora`, `contrast`, `mono` |
| `TERSH_BORDER` | `ascii` (default), `rounded`, `thick` |
| `TERSH_FOOTER` | `auto` (default), `compact`, `full` |
| `TERSH_GLYPHS` | `ascii` (default), `unicode` |
| `--no-motion` / `TERSH_MOTION=off` | Disable probe activity animation |
| `TERSH_CLIPBOARD=off` | Disable OSC52 terminal clipboard writes |
| `NO_COLOR` / `TERSH_COLOR=off` | Disable colors |

A preset overrides the corresponding environment variables. Explicit `--theme`
and `--no-motion` apply afterwards; no-color settings remain respected. Motion
is limited to active probes at at most four frames/second. Graph history comes
from completed probes, not an extra collector.

## Keys

Lookup order:

1. `--keymap FILE`
2. `TERSH_KEYMAP`
3. `$XDG_CONFIG_HOME/tersh/keymap.json`, or `~/.config/tersh/keymap.json`
4. Built-in defaults if the default file is absent

An explicitly selected missing/invalid file is an error. Files must be regular,
UTF-8 JSON, at most 64 KiB, and not a final-component symlink.

```sh
tersh --dump-keymap > keymap.json
tersh --keymap keymap.json
tersh --keymap keymap.json --c
```

`--dump-keymap` prints the complete effective map and exits without starting the
TUI. Partial maps override only the named actions. Each array replaces that
action's previous bindings; `[]` removes its shortcuts, while the action may
remain available through its menu.

```json
{
  "files": { "copy": ["Ctrl+y"], "open_jobs": ["F5", "J"] },
  "actions": { "down": ["Down", "Tab", "Ctrl+n"] }
}
```

Supported keys include characters, `F1`–`F24`, `Enter`, `Esc`, `Space`, `Tab`,
`BackTab`, `Backspace`, arrows, `Home`, `End`, `PageUp`, `PageDown`, plus
`Ctrl+`, `Alt+`, `Shift+` and `Super+` modifiers. Space-separated sequences such
as `g g` form chords of up to four keys. Terminal clients may intercept some
combinations; keep a simple alternative on mobile keyboards.

| Context | Used in |
| --- | --- |
| `files` | Normal browsing |
| `preview` | Fullscreen preview |
| `input` | File prompts and confirmations |
| `help` | Scrollable binding help |
| `actions` | Searchable action picker |
| `cluster` | Host list |
| `cluster_detail` | Expanded host detail |
| `cluster_filter` | Host filter input |
| `trash` | Recovery list |
| `jobs` | File-job progress/results |
| `places` | Searchable recent and pinned directories |
| `log` | Bounded log tail/follow, pause and search |

Unknown contexts/actions, duplicate JSON fields, duplicate keys and ambiguous
chord prefixes are rejected. For example, `g` and `g g` cannot coexist as
separate actions in the same context. Modal contexts must retain a cancellation
key. **Ctrl+C is reserved**; during a file job it cancels and waits for cleanup
before exit. Failed printable chords replay their text in input contexts.

## Host inventory

Lookup order: `--cluster-config FILE`, `TERSH_SERVERS_JSON`, `./ssh/servers.json`,
then `~/.config/tersh/servers.json`. If no inventory exists, the dashboard uses
a local-host fallback. See [examples/servers.json](../examples/servers.json).

The top-level keys are `main_machine`, `jump_host`, and `servers`. All are
optional. A server accepts `alias`, `campus_ip` (hostname or IP), `ssh_user`,
`proxy_jump`, `role`, and `workdir`. A jump host accepts `device_name` or
`tailscale_ip` for its address. `directory` and `tersh_dir` are accepted aliases
for `workdir`. A server's `proxy_jump` must identify the configured jump host.

Aliases must be unique; unknown fields and invalid SSH values are rejected.
Use existing SSH credentials and trusted host keys. `t` requires Tersh on the
selected remote host, and `workdir` controls where that workbench starts.

Filters affect visible hosts only. `r` refreshes the configured inventory;
automatic probing remains capped and rotated. No history is written to disk.

## File-operation boundaries

Only one mutation job runs at a time. Directory listing and preview loading
use bounded background readers in the interactive runtime. A newer queued read
supersedes an older one; stale results cannot replace the current directory or
preview. Returning to a parent restores the departed child, and a failed path
input stays editable. Cancellation is cooperative and preserves completed work;
permanent deletion cannot be undone. Replacing existing directories is refused.

Trash receipts belong to the work root selected at startup. Restore validates
the receipt, stored object and original parent, then performs a no-clobber
same-filesystem rename. Malformed records are reported and excluded. Receipts
use filesystem permissions and identity checks; they are not cryptographic
protection against their OS owner. Legacy trash and non-UTF-8 original paths
are not silently assigned invented recovery metadata.

## Locations

`b` opens recent and pinned locations; `B` pins/unpins the current work directory.
Type to filter, use arrows/Tab and Enter to open. In the locations view, Ctrl+B
toggles a pin, Ctrl+D forgets the selected saved record, and Ctrl+L clears recent
records for the highlighted place's host while retaining pins (`--c`), or the current host in the file workbench. In `--c`, `B` pins the configured work directory. These actions change location metadata, never the
directory itself. Failed navigation retains the selection and explains the error.

The file workbench lists locations for its current host identity. `--c` can search
saved host/path pairs and configured `workdir` entries. Saved remote locations
must still match the current alias, SSH target, resolved jump target and host
kind before they can launch. Remote histories stay on their host; no history
sync or additional SSH discovery runs in the background.

State is limited to 128 entries, 64 pins and 64 KiB:

- `TERSH_PLACES=off` disables history and pins.
- `TERSH_PLACES_FILE` selects a state file.
- Otherwise use `$XDG_STATE_HOME/tersh/places.json` or
  `~/.local/state/tersh/places.json`.

State files are private and written atomically. A kernel lock protects saves;
an existing lock sidecar is normal. A concurrent state change is reported rather
than overwritten; reopen the view/application as directed to reload it. Invalid
or symbolic-link state files are not replaced. Saved paths must be absolute UTF-8.

## Recent task results

`J` opens the active task and the latest 20 results from this session. Use `[`/`]`
for newer/older results, `f` for failed/remaining items, `r` to propose a retry,
and `e` to export the selected result as JSON to a new file.

History has a 1 MiB retained-data budget and 64 KiB per entry. Full outcome counts
remain available when details are omitted; the view and export state the omitted
counts. Retrying includes failed and remaining items only, clears previous
overwrite/skip decisions, and revalidates the sources and destination through
the normal confirmation path. If retry paths were omitted, retry is refused
rather than silently operating on a subset. Export never overwrites an existing
file. General task history is in-memory; persistent trash recovery remains separate.

Progress distinguishes processed top-level items from bytes copied. The displayed
mean rate is an estimate based on elapsed task time; no ETA is invented when a
total byte count is unknown. Cancelling preserves already completed operations.

## Logs and structured previews

`L` opens a bounded log tail on the focused regular file. Space pauses/resumes;
scrolling or `/` search pauses following so the reading position stays stable.
`n`/`N` move between matches. Escape closes the log view. Each background poll
reads at most 64 KiB, with automatic polling at most four times per second. The retained buffer is
at most 2,000 lines / 256 KiB; older data and truncated lines are identified.

The reader reports observed file rotation, shortening and read errors. A file
that is truncated and regrows past the old offset entirely between two polls may
not be recognized as rotated. Pausing stops further automatic scheduling and freezes the displayed buffer; an already in-flight read may finish.
Symbolic links and special files are refused for following.

`f` switches raw/structured preview for JSON, CSV and unified diff. JSON formatting
preserves original numeric tokens, duplicate fields, order and string escapes.
CSV supports quoted commas/newlines. Structured input/output is capped at
256 KiB, 2,000 lines and 512 display columns; CSV is limited to 32 columns.
Invalid or oversized input has an explicit bounded raw fallback. These are data
limits, not a claim about peak process memory. No external renderer is launched.

## Host attention and refresh

In `--c`, `a` toggles hosts needing attention, `E` opens the latest 100 state
changes, and `P` pauses automatic refresh. Manual refresh remains available.
`TERSH_DISK_WARN` sets the used-storage warning threshold (0–100, default 90).
Unknown, old and failed observations remain distinguishable from fresh success.

Refreshing retains the last valid values with a separate checking/error state
and data age. Normal refresh scheduling is per host; repeated failure backs off
to 30/60/120 seconds. Existing concurrency and connection-time budgets remain
bounded. Closing the dashboard cancels active probes and reaps their workers.
Attention filters and charts reuse existing snapshots and never add collectors.

Trend positions represent observation order with equal visual spacing, not equal
time intervals. Gaps remain missing. Memory/storage use fixed percentage scales;
load is not CPU utilization, and probe duration is not pure network latency.

## Interface behavior

`I` toggles the wide-screen inspector. Directory sizes are shown as unscanned
rather than presenting directory metadata as recursive storage use. At 40 columns,
primary actions and the action menu retain priority, including custom keybindings.
Action menus group tasks, support search aliases, and explain unavailable actions.
Pending chord hints follow the effective keymap. Existing no-color, ASCII and
no-motion presets continue to work without font extensions or idle animation.
