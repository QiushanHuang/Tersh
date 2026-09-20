# Contributing to Tersh

Small, focused changes that improve local and SSH terminal work are welcome.
For a new workflow or a change to file-operation semantics, describe the use
case and failure behavior in an issue before implementing a large change.

## Development

Use Rust 1.88 or newer. The Python reference tests use only the standard
library; Python 3.12 is used in CI.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
```

For layout changes, inspect wide and 40-column views, no-color mode, and the
effective keymap. The offline gallery needs Pillow only for PNG rendering:

```sh
cargo run --example ui_gallery -- target/ui-gallery
python3 scripts/render-gallery.py target/ui-gallery
```

`scripts/verify-workflow-pty.py` exercises real terminal workflows on local
macOS using temporary fixtures. It must not be pointed at real project data.

## What to preserve

- No-clobber defaults, explicit destructive confirmations and identity checks.
- Cooperative cancellation with accurate partial results and ownership-aware
  cleanup. Never treat cancellation as undo.
- Bounded worker concurrency, preview caches, history, probe output and timeouts.
- ASCII/no-color fallbacks and usable narrow-terminal controls.
- Keybinding hints generated from actual bindings, not a second key list.
- Existing user configuration and SSH trust decisions.

Include a regression test for a bug and explain how it fails before the fix.
Tests should check behavior and rendered cells rather than incidental paths,
timing, test counts or exact narrative wording.

## Pull requests and attribution

Keep the PR scoped, describe the user-visible change, record relevant checks,
and update the changelog for behavior changes. Use synthetic examples; do not
commit personal host inventories, private deployment receipts, credentials,
build output or unrelated local notes.

Use **your own** Git name and a verified email or your GitHub noreply email.
Do not copy the maintainer's identity. Add co-author trailers only for actual
contributors who should receive credit. Historical authorship is preserved;
`.mailmap` only normalizes verified aliases for display.

Contributor identities are recorded in Git and on the repository's
[contributors page](https://github.com/QiushanHuang/Tersh/graphs/contributors).
The existing [MIT copyright notice](LICENSE) must remain intact.
