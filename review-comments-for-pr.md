# Tersh 审查记录状态

> 该文件曾保存早期可贴入 PR 的审查评论。当前分支已完成后续修复，旧行号和旧 P1 结论不再适合作为待办清单使用。

## Resolved In Current Branch

- Probe 输出不再通过管道等待，当前实现写入临时文件并设置输出大小上限。
- Timed-out active probe 不再导致自动刷新循环持续刷 `no eligible hosts`。
- `selected_host()` / `selected_snapshot()` 已使用 `Option` 路径处理空 host 列表。
- Windows symlink copy 会根据目标类型选择文件/目录 symlink 创建方式。
- `$VISUAL` / `$EDITOR` 已支持简单带参数命令解析。
- `--print-cwd` 模式下 TUI、OSC52 和编辑器挂起恢复使用一致输出通道。
- Preview cache 已包含内容采样 hash，不只依赖长度和 mtime。

## Current Follow-up Source

后续审查结论以新的代码审查报告和 `CHANGELOG.md` 的 `Unreleased` 条目为准。不要再直接复制本文件早期评论到 GitHub review。
