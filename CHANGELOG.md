# Changelog

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
