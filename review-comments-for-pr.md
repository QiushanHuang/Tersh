# Tersh 审查结果（可直接贴入 PR review comments）

> 该文件整理了最近几轮交叉审查中的问题清单，按可直接贴到 GitHub PR review 的形式给出。未修复，仅记录。

## High

- **[P1] `run_command_with_timeout` 存在输出管道阻塞风险，可能导致探测卡死/超时失效**  
  文件: `src/cluster.rs`  
  行: `src/cluster.rs#1048` `src/cluster.rs#1050` `src/cluster.rs#1060`
  
  建议评论：
  > 在 `run_command_with_timeout` 中先用 `child.try_wait()` 检查退出态，直到结束前从不持续读取 `stdout/stderr`。如果被探测命令持续输出，子进程可能阻塞在满管道上，`try_wait` 永远返回 `None`，造成刷新永远卡住。请改为异步/线程读取输出（或直接 `wait_with_output()`）并设置超时，避免管道阻塞。

- **[P1] `refreshing` 集合若 worker 未回传 snapshot 会长期阻塞轮询**  
  文件: `src/cluster.rs`  
  行: `src/cluster.rs#540` `src/cluster.rs#576` `src/cluster.rs#689`

  建议评论：
  > `begin_refresh` 仅在 `apply_snapshot` 中清理 `refreshing`。若 `collect_host_snapshot` 被卡住/线程异常未回传，`is_refreshing()` 一直为真，`refresh_due()` 会长期返回 false，导致后续手动/自动刷新全部停滞。应补充超时与清理机制。

- **[P1] `selected_host()` 与 `selected_snapshot()` 缺少边界保护，`hosts` 空时可越界 panic**  
  文件: `src/cluster.rs`  
  行: `src/cluster.rs#636` `src/cluster.rs#641`

  建议评论：
  > 这两个方法直接 `&self.hosts[self.cursor]` 和 `self.snapshots.get(self.selected_host().alias())`，未对 `hosts.is_empty()` 或 `cursor` 有界性做保护。若初始化/读取失败后 `hosts` 为空且未阻挡路径，可能触发 panic。

- **[P1] Windows 下 symlink 目录复制时目录类型参数未传递，可能创建错误类型链接**  
  文件: `src/fs_ops.rs`  
  行: `src/fs_ops.rs#26` `src/fs_ops.rs#247`

  建议评论：
  > `copy_path` 复制符号链接时始终调用 `create_symlink(&link_target, target, false)`，`false` 在 Windows 代表文件链接。若原链接指向目录会创建错误类型。请根据 `metadata.is_dir()`/`file_type` 决定 `is_dir`。

- **[P1] 编辑器配置变量不支持参数（`VISUAL`/`EDITOR`）**  
  文件: `src/app.rs`  
  行: `src/app.rs#858` `src/app.rs#861`

  建议评论：
  > `launch_editor` 使用 `ProcessCommand::new(editor)`，把 `VISUAL/EDITOR` 当单一可执行文件，不解析参数。用户设置 `EDITOR="vim -u NONE"` 会执行失败。建议按 shell/分词策略处理命令字符串。

- **[P1] `print-cwd` 与编辑器/挂起恢复通道选择不一致，输出恢复路径可能异常**  
  文件: `src/app.rs`  
  行: `src/app.rs#1275` `src/app.rs#1279` `src/app.rs#1288` `src/app.rs#855`

  建议评论：
  > `run_with_options(print_cwd)` 时 `TerminalOutput::Stderr` 创建 TUI guard；但 `launch_editor` 强制写回 `io::stdout()`，即使当前 TUI 在 stderr 输出模式。与退出/恢复通道不一致，可能导致界面与外部 shell 恢复行为不稳。

- **[P1] 探测命令完全依赖 `sh -lc`，在非 POSIX Shell/Windows 环境下兼容性脆弱**  
  文件: `src/cluster.rs`  
  行: `src/cluster.rs#865` `src/cluster.rs#899` `src/cluster.rs#1037`

  建议评论：
  > 本地探测与 SSH remote probe/workbench 执行均硬编码 `sh -lc`。在非类 Unix 环境或未安装 sh 的 shell 下会直接失败，不利于跨平台。建议按平台选择 shell 或改为 `env` 指令列表。

## Medium

- **[P2] `path_exists_no_follow` 误将非 NotFound 错误当作“存在”处理，权限问题会被掩盖**  
  文件: `src/fs_ops.rs`  
  行: `src/fs_ops.rs#177` `src/fs_ops.rs#181`

  建议评论：
  > 当前实现把任何非 `NotFound` 错误都返回 `true`，例如权限不足将导致误判目标存在，进而走到错误分支（如拒绝覆盖提示不准确）。应区分 `Err` 场景：权限等异常应可透传或给出明确报错。

- **[P2] `read_dir_entries` 对单项元数据失败静默跳过，用户无法感知文件列表缺失**  
  文件: `src/fs_core.rs`  
  行: `src/fs_core.rs#79` `src/fs_core.rs#88`

  建议评论：
  > `if let Ok(entry) = FileEntry::from_path(...) { entries.push(entry); }` 直接忽略错误，目录中存在权限/损坏项时会悄悄丢失条目。建议累计失败并上报、或至少在列表中以错误项展示。

- **[P2] `expand_path` 只识别 `~` 与 `~/`，不处理 `~user` 等常见写法**  
  文件: `src/app.rs`  
  行: `src/app.rs#1380`

  建议评论：
  > 路径展开只处理当前用户主目录前缀，`~other_user/path` 目前不会展开。建议补齐 `~user`（可选）或改用 shell 风格展开策略以与用户预期一致。

- **[P2] 预览签名只包含 `len/modified/kind`，对同名短内容变更无法识别**  
  文件: `src/app.rs`  
  行: `src/app.rs#1353` `src/app.rs#1364`  

  建议评论：
  > `preview_signature` 仅看文件长度/mtime/type。某些情况下 mtime 或长度不变但内容变更会误用旧缓存导致预览不刷新。若依赖 `checksum`（如前 N 字节哈希）可避免该类脏读。

## Low

- **[P3] Symlink preview 策略缺少目录目标语义测试（与 copy 逻辑同源）**  
  文件: `src/preview.rs`  
  行: `src/preview.rs#70`

  建议评论：
  > 预览阶段对 `is_file` 做了硬性限制，当前对目录/特殊类型直接返回 Unsupported。若是针对 symlink 目标目录，结果可能与实际预期不一致；建议与文件系统语义对齐并补充测试。

- **[P3] `run_local_probe` 与 `run_ssh_probe` 共享同一超时与脚本字符串，错误信息可观测性较弱**  
  文件: `src/cluster.rs`  
  行: `src/cluster.rs#865` `src/cluster.rs#871` `src/cluster.rs#1040`

  建议评论：
  > 两条通路都在超时后统一返回类似 timeout/shell 输出，缺少区分“连接超时/执行脚本卡住/命令退出码失败”的结构化信息。建议在失败路径上扩充错误上下文。

