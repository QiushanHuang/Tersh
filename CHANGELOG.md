# Changelog

## Unreleased

### Added

- Added `tersh --c`, a multi-host cluster status dashboard for local, jump, and remote servers.
- Added JSON inventory loading from `--cluster-config`, `TERSH_SERVERS_JSON`, `./ssh/servers.json`, and `~/.config/tersh/servers.json`.
- Added non-interactive SSH health probes for connection status, system info, uptime, CPU load, memory, storage, task count, and GPU availability.
- Added `s` in the cluster dashboard to open a local shell or interactive SSH session for the selected host.
- Added `t` in the cluster dashboard to open the Tersh file workbench on the selected host; remote hosts run through `ssh -t`.
- Added `workdir` / `directory` / `tersh_dir` inventory support so `t` can start Tersh in a specific directory.
- Added cluster dashboard tests for inventory parsing, probe parsing, SSH argument construction, key handling, stale metrics, and rendering.

### Notes

- Remote `t` mode expects `tersh` to already be installed and available in `PATH` on the remote host.
- `s` exits back to the dashboard when the shell or SSH session exits with `exit` or `Ctrl+D`.
- `t` exits back to the dashboard when the nested Tersh workbench exits with `q`.

## 中文更新说明

### 新增

- 新增 `tersh --c` 多主机状态面板，可查看本机、跳板机和远端服务器。
- 支持从 `--cluster-config`、`TERSH_SERVERS_JSON`、`./ssh/servers.json`、`~/.config/tersh/servers.json` 读取 JSON 主机清单。
- 新增非交互式 SSH 健康探测，展示连接状态、系统信息、运行时间、CPU load、内存、存储、进程数量和 GPU 可用性。
- 在状态面板中按 `s` 可进入选中主机的本地 shell 或交互式 SSH 会话。
- 在状态面板中按 `t` 可在选中主机上打开 Tersh 文件工作台；远端主机会通过 `ssh -t` 运行。
- 主机清单支持 `workdir` / `directory` / `tersh_dir`，用于指定 `t` 打开 Tersh 时进入的目录。
- 增加了主机清单解析、探测解析、SSH 参数、快捷键、stale 指标和渲染测试。

### 注意

- 远端 `t` 模式要求目标主机上已经安装 `tersh`，并且 `tersh` 在远端 `PATH` 中可用。
- `s` 进入后，用 `exit` 或 `Ctrl+D` 回到状态面板。
- `t` 进入后，用 `q` 退出内层 Tersh 并回到状态面板。
