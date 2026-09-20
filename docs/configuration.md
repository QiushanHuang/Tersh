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
remain synchronous. Cancellation is cooperative and preserves completed work;
permanent deletion cannot be undone. Replacing existing directories is refused.

Trash receipts belong to the work root selected at startup. Restore validates
the receipt, stored object and original parent, then performs a no-clobber
same-filesystem rename. Malformed records are reported and excluded. Receipts
use filesystem permissions and identity checks; they are not cryptographic
protection against their OS owner. Legacy trash and non-UTF-8 original paths
are not silently assigned invented recovery metadata.
