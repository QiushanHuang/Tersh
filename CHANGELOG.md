# Changelog

## Unreleased

### Added

- Added copy-conflict handling for existing targets, with explicit `replace` or `skip` confirmation before overwriting.
- Added `TERSH_CLIPBOARD=off` to disable OSC52 terminal clipboard writes while keeping in-app copy state and logs.
- Added `TERSH_THEME=btop|aurora|contrast|mono` for color-rich, aurora, high-contrast, or no-color terminal rendering.
- Added `TERSH_BORDER=ascii|rounded|thick` so Unicode-capable terminals can opt into rounded or heavier btop-style chrome while ASCII remains the remote-safe default.
- Added `TERSH_FOOTER=auto|compact|full` so mobile or narrow-device users can force shortcut footer density.
- Added `--cluster` as the clearer long-form entry for the read-only cluster health dashboard, while keeping `--c` as a compatibility alias.
- Added context-aware workbench footer recommendations for focused directories, focused files, active selections, and copy/cut buffers.
- Added cluster footer recommendations and detail hints so host status screens suggest the next action for online, stale, auth-failed, timeout, and unknown hosts.
- Added direct file startup support so `tersh <file>` opens the file's parent directory, focuses the file, and enters preview for regular files.
- Added remote workbench diagnostics for `tersh --c` so remote `t` sessions report missing `tersh` binaries and invalid `workdir` paths with actionable messages.
- Added compact workbench help and confirmation layouts for very narrow terminal screens.

### Changed

- Changed workbench and cluster event loops to dirty-driven rendering, reducing idle redraws while still repainting on input, resize, refresh, and probe updates.
- Changed workbench and cluster chrome to share themed status chips, footer highlighting, selected-row styling, and warning/error emphasis.
- Changed workbench and cluster rendering to use semantic panel-title, key/value, inactive, copy, cut, and search-match colors for stronger visual hierarchy without changing layout density.
- Changed cluster resource bars to style filled and empty segments separately, making high-load metrics stand out while keeping ASCII bar characters for lightweight remote terminals.
- Changed the TUI theme layer to use shared semantic `Tone` / `ChipTone` primitives for panel chrome, footer styling, key-value rows, and resource bars.
- Changed workbench and cluster panels to distinguish active and inactive chrome, making the current work surface easier to scan without adding layout weight.
- Changed the workbench status header to use plain semantic key-value text instead of filled background chips, avoiding unreadable `hidden OFF` combinations on light terminal palettes.
- Changed compact workbench status to show the full copy/cut buffer label instead of a generic copy count.
- Changed file preview caching from a single entry to a bounded LRU cache for faster adjacent-file navigation without retaining stale same-path previews.
- Changed directory entries to cache lowercase names, reducing repeated allocation during filtering and sorting.
- Changed file rows to show cursor, selection, and copy/cut buffer state in a fixed row marker so the active operation target is easier to scan.
- Changed trash/delete confirmation prompts to show required and typed confirmation text, with stronger visual severity styling.
- Changed filter input to use the current in-memory directory listing while typing, avoiding a full directory scan on every character.
- Changed compact workbench and cluster chrome to keep selection/buffer state and the cluster detail action visible on narrow terminals.
- Changed the install script to build with `--locked`, and enabled release stripping plus thin LTO for smaller optimized local rebuilds.

### Fixed

- Fixed parallel test instability in probe temporary-file cleanup checks by isolating the probe-output tests from each other.
- Hardened file operation race windows with source identity rechecks, no-follow regular-file opens, safer trash/delete target checks, no-clobber copy targets, and no-replace rename APIs where supported.
- Fixed copy failure cleanup so failed regular-file and recursive-directory copies do not leave partial targets behind.
- Fixed replace-copy behavior so an invalid source no longer deletes an existing target before source validation succeeds.
- Fixed edit and preview safety by revalidating regular files with no-follow opens before editor launch or preview cache hashing.
- Fixed cut/paste retry behavior so failed cut items remain in the transfer buffer instead of being discarded.
- Fixed cluster refresh generations so timed-out probe results cannot overwrite newer timeout/stale state, while real in-flight probes continue to count toward the concurrency cap.
- Fixed cluster refresh retry throttling so timed-out active probes do not cause repeated no-eligible-host refresh attempts.
- Fixed cluster inventory parsing to reject unknown JSON fields and store trimmed aliases, SSH fields, roles, and work directories.
- Fixed cluster inventory validation to reject whitespace inside SSH target fields.
- Changed cluster probes to require known SSH host keys instead of auto-accepting new keys during read-only health checks.
- Fixed cluster probe cleanup by capping captured stdout/stderr and killing timed-out Unix probe process groups.
- Fixed cluster probe output handling so non-UTF-8 stdout/stderr is preserved lossily instead of being reported as a read failure.
- Fixed terminal suspension recovery paths to avoid half-restored alternate-screen/raw-mode states.
- Fixed narrow file-list rendering by truncating long file names in-row.
- Fixed file-list truncation to use terminal display width for wide Unicode characters.
- Fixed OSC52 clipboard writes to reject oversized payloads before emitting terminal escape sequences.
- Fixed install script failures for unwritable install directories with an explicit remediation message and atomic temporary-file installation.
- Fixed cluster snapshots so known host updates are applied even when injected outside an active refresh, restoring summary counts, stale metrics, and detail rendering.
- Fixed cluster refresh scheduling so empty or fully saturated refresh attempts do not reset the automatic refresh timer.
- Fixed cluster probe command execution to preserve stdout/stderr in timeout/error paths and report probe execution errors with context instead of silently dropping them.
- Fixed timeout handling for remote/local probes so output is cleaned up deterministically and timeout errors remain visible in refresh logs.
- Fixed directory listings so transient failures for individual entries are skipped instead of clearing the entire view.
- Fixed named home expansion for `~user` paths, including the no-trailing-slash form.
- Fixed preview cache invalidation to hash the visible preview range instead of only the first 4 KiB.
- Fixed preview handling for size-zero virtual regular files by attempting a bounded read before reporting an empty file.
- Fixed editor launch recovery so alternate screen and raw mode are restored consistently after editor spawn/status failures.
- Fixed resource percentage parsing for fullwidth percent signs and rounded percentage values.

## v1.1.0 - 2026-05-31

v1.1.0 is a small product-quality update that makes Tersh denser, more inspectable, and safer for remote terminal work.

### Added

- Added a btop-inspired file workbench header showing cwd, item count, selected size, copy/cut buffer state, hidden-file state, filter text, and sort mode.
- Added sortable file browsing with `s` to cycle sort modes and `S` to reverse the current sort.
- Added an Inspector panel that separates target metadata, buffer state, search/sort context, and logs.
- Added cluster dashboard health tokens (`OK`, `OLD`, `FAIL`, `CHK`) plus per-host latency, memory, and disk columns.
- Added `Esc` as a cancel key alongside `Ctrl+G`.
- Added release documentation in `docs/releases/v1.1.0.md`.

### Changed

- Changed file rows into a denser operational table with selection, kind, permission, size, and name columns.
- Changed edit behavior to respect `$VISUAL` and `$EDITOR` before falling back to `nano`.
- Changed multi-target trash/delete confirmation dialogs to show operation source and the first affected paths.
- Updated crate metadata to version `1.1.0` and declared `rust-version = "1.85"`.

### Fixed

- Fixed terminal display safety gaps by escaping rendered paths and prompt input.
- Fixed edit safety so symlinks and special files are rejected instead of being handed to the editor.
- Fixed directory reload behavior so transient metadata failures for one entry do not clear the whole view.

### Verification

- `cargo fmt --check`
- `cargo clippy --locked --all-targets -- -D warnings`
- `cargo test --locked --all-targets`
- `cargo build --locked --release --bin tersh`
- `./target/release/tersh --help`
- `./target/release/tersh --version`

## V1 - 2026-05-31

V1 turns Tersh into a safer and clearer product baseline for terminal file work and read-only multi-host checks.

### Added

- Added `tersh --c`, a multi-host cluster status dashboard for local, jump, and remote servers.
- Added JSON inventory loading from `--cluster-config`, `TERSH_SERVERS_JSON`, `./ssh/servers.json`, and `~/.config/tersh/servers.json`.
- Added non-interactive SSH health probes for connection status, system info, uptime, CPU load, memory, storage, task count, and GPU availability.
- Added `s` in the cluster dashboard to open a local shell or interactive SSH session for the selected host.
- Added `t` in the cluster dashboard to open the Tersh file workbench on the selected host; remote hosts run through `ssh -t`.
- Added `workdir` / `directory` / `tersh_dir` inventory support so `t` can start Tersh in a specific directory.
- Added cluster dashboard tests for inventory parsing, probe parsing, SSH argument construction, key handling, stale metrics, and rendering.
- Added mode-aware footer copy for normal browsing, fullscreen preview, preview search, filter, goto, rename, copy-to, move-to, trash confirmation, delete confirmation, help, cluster list, and cluster detail modes.
- Added compact footer and status variants for narrow/mobile terminal widths.
- Added destructive-operation confirmation context showing target count and the first affected path.
- Added release documentation in `docs/releases/V1.md`.

### Changed

- Changed fullscreen preview controls so `j` / `k` and `PageUp` / `PageDown` scroll by page, while arrow keys and `Ctrl+F` / `Ctrl+B` scroll by line.
- Changed help text to describe the preview scroll model accurately.
- Changed cluster refresh behavior to cap concurrent probes and rotate through host aliases, preventing large inventories from starting every probe at once.
- Updated crate metadata to version `1.0.0`.

### Fixed

- Fixed misleading footers that advertised shortcuts unavailable in the current mode.
- Fixed cramped footer/status rendering on very small terminal widths.
- Fixed cluster help behavior so `?` and `Enter` close help consistently with the displayed footer.
- Fixed clippy findings in nested key/event handling paths.

### Security And Stability

- Preview now avoids following symlinks and rejects non-regular files after opening with a no-follow path on Unix.
- Preview now reports safe messages for directories, symlinks, and unsupported special files instead of trying to read them as normal files.
- Preview content is capped by bytes, line length, and total line count to keep rendering predictable.
- Trash/delete guards now reject filesystem roots, `$HOME`, the active work root, `.tersh-trash`, relative paths, and symlinked trash directories.
- Symlink deletion now targets the link object itself, including dangling symlinks.
- Cluster inventory validation now rejects duplicate aliases, empty/control-character SSH fields, option-like SSH fields starting with `-`, unresolved jump hosts, and invalid role/workdir text.

### Verification

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets`
- `cargo build --release --bin tersh`

### Notes

- Remote `t` mode expects `tersh` to already be installed and available in `PATH` on the remote host.
- `s` exits back to the dashboard when the shell or SSH session exits with `exit` or `Ctrl+D`.
- `t` exits back to the dashboard when the nested Tersh workbench exits with `q`.

## 中文更新说明

### v1.1.0 - 2026-05-31

v1.1.0 是一次小版本产品质量更新，让 Tersh 的终端界面更高密度、更容易扫读，也更适合远程文件工作。

### 新增

- 新增参考 btop 的文件工作台顶部状态栏，展示 cwd、条目数量、已选大小、复制/剪切缓冲区、隐藏文件状态、筛选文本和排序模式。
- 新增 `s` 循环排序模式、`S` 反转当前排序。
- 新增 Inspector 面板，分块展示目标元数据、缓冲区状态、搜索/排序上下文和日志。
- 集群面板新增 `OK`、`OLD`、`FAIL`、`CHK` 健康指标，并在主机列表中显示延迟、内存和磁盘列。
- 新增 `Esc` 作为取消键，与 `Ctrl+G` 一起使用。
- 新增 `docs/releases/v1.1.0.md` 发行说明。

### 调整

- 文件行改为更紧凑的操作表格，展示选择、类型、权限、大小和名称。
- 编辑器优先使用 `$VISUAL` 和 `$EDITOR`，最后回退到 `nano`。
- 多目标删除/回收站确认会显示操作来源和前几个受影响路径。
- crate 版本更新为 `1.1.0`，并声明 `rust-version = "1.85"`。

### 修复

- 修复路径和输入内容中的控制字符可能污染终端显示的问题。
- 修复编辑功能会把符号链接和特殊文件交给编辑器的问题。
- 修复目录刷新时单个条目 metadata 临时失败会清空整个视图的问题。

### 验证

- `cargo fmt --check`
- `cargo clippy --locked --all-targets -- -D warnings`
- `cargo test --locked --all-targets`
- `cargo build --locked --release --bin tersh`
- `./target/release/tersh --help`
- `./target/release/tersh --version`

### V1 - 2026-05-31

V1 将 Tersh 收束成一个更安全、更清楚的终端文件工作和只读多主机检查产品基线。

### 新增

- 新增 `tersh --c` 多主机状态面板，可查看本机、跳板机和远端服务器。
- 支持从 `--cluster-config`、`TERSH_SERVERS_JSON`、`./ssh/servers.json`、`~/.config/tersh/servers.json` 读取 JSON 主机清单。
- 新增非交互式 SSH 健康探测，展示连接状态、系统信息、运行时间、CPU load、内存、存储、进程数量和 GPU 可用性。
- 在状态面板中按 `s` 可进入选中主机的本地 shell 或交互式 SSH 会话。
- 在状态面板中按 `t` 可在选中主机上打开 Tersh 文件工作台；远端主机会通过 `ssh -t` 运行。
- 主机清单支持 `workdir` / `directory` / `tersh_dir`，用于指定 `t` 打开 Tersh 时进入的目录。
- 增加了主机清单解析、探测解析、SSH 参数、快捷键、stale 指标和渲染测试。
- 新增随模式变化的底部快捷键说明，覆盖普通浏览、全文预览、预览查找、筛选、跳转、重命名、复制到、移动到、回收站确认、永久删除确认、帮助、集群列表和集群详情。
- 新增面向窄屏和移动端终端的紧凑 footer 与状态区。
- 新增删除/回收站确认上下文，显示目标数量和第一个受影响路径。
- 新增 `docs/releases/V1.md` 发行说明。

### 调整

- 全文预览中 `j` / `k` 和 `PageUp` / `PageDown` 改为按页滚动，方向键和 `Ctrl+F` / `Ctrl+B` 按行滚动。
- 帮助页同步更新预览滚动说明。
- 集群刷新增加并发上限和轮转策略，避免大清单一次性启动所有探测。
- crate 版本更新为 `1.0.0`。

### 修复

- 修复底部 footer 在部分模式中展示不可用快捷键的问题。
- 修复极窄终端下 footer 和状态区信息拥挤的问题。
- 修复集群帮助页中 `?` 和 `Enter` 与 footer 说明不一致的问题。
- 修复嵌套按键和事件处理路径中的 clippy 警告。

### 安全与稳定性

- 预览不再跟随符号链接，Unix 下使用 no-follow 打开并在打开后确认目标仍是普通文件。
- 目录、符号链接和特殊文件会展示安全提示，不再按普通文件读取。
- 预览内容同时限制字节数、单行长度和总行数，避免渲染失控。
- 回收站/删除防护会拒绝文件系统根目录、`$HOME`、当前工作根目录、`.tersh-trash`、相对路径和被替换成符号链接的回收站目录。
- 删除符号链接时只删除链接本身，包括悬空符号链接。
- 集群清单校验会拒绝重复 alias、空值或包含控制字符的 SSH 字段、以 `-` 开头的危险 SSH 字段、无法解析的跳板机和非法 role/workdir 文本。

### 验证

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets`
- `cargo build --release --bin tersh`

### 注意

- 远端 `t` 模式要求目标主机上已经安装 `tersh`，并且 `tersh` 在远端 `PATH` 中可用。
- `s` 进入后，用 `exit` 或 `Ctrl+D` 回到状态面板。
- `t` 进入后，用 `q` 退出内层 Tersh 并回到状态面板。
