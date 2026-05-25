# Tersh

[![简体中文](https://img.shields.io/badge/语言-简体中文-1677ff)](#中文)
[![English](https://img.shields.io/badge/Language-English-24292f)](#english)

[![Rust](https://img.shields.io/badge/Rust-2024-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Terminal UI](https://img.shields.io/badge/TUI-ratatui-1f2937)](https://ratatui.rs/)
[![Shell Workflow](https://img.shields.io/badge/Workflow-Local%20%7C%20SSH%20Shell-0f766e)](#local-and-ssh-shell-sessions)
[![Mobile Friendly](https://img.shields.io/badge/Focus-Mobile--Friendly-2563eb)](#mobile-first-remote-workflow)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux-475569)](#installation)
[![Status](https://img.shields.io/badge/Status-Early%20Product-7c3aed)](#project-status)
[![License](https://img.shields.io/badge/License-MIT-16a34a)](#license)

<a id="english"></a>

**Tersh** is a lightweight, mobile-friendly terminal file workbench for local and SSH shell sessions.

It is built for the moment when you are already inside a terminal, locally or after SSH, and need a fast way to inspect files, preview content, move around directories, and perform basic file operations without bouncing between `ls`, `cd`, `cat`, `cp`, and `rm`.

## Why Tersh

- Built for terminal-first workflows, locally or inside an SSH shell
- Comfortable on narrow terminals, remote shells, and lightweight setups
- Lightweight enough for multi-terminal and low-friction setups
- Keyboard-first, with safe file operations and inline preview
- Useful on tablets and phones where a full desktop file manager is unavailable

## Local And SSH Shell Sessions

Tersh is not an SSH client and it is not a multi-host session manager.

Its role is narrower and more practical: run the `tersh` CLI in any local directory, or connect to a server through your preferred SSH app and launch `tersh` there for remote file inspection and lightweight operations.

That makes it a good fit for:

- remote content checks
- quick server-side file triage
- path copying during ops workflows
- working across multiple hosts from lightweight terminal clients

## Mobile-Friendly Terminal Workflow

The product direction is intentionally simple:

- open a local terminal or enter a host over SSH
- launch one command
- browse, filter, preview, copy, move, rename, and clean up files
- stay inside the terminal the whole time

This matters when your working environment is:

- an iPad with an SSH app
- a phone handling urgent remote checks
- a lightweight laptop in a multi-terminal setup
- a server environment where GUI tools are irrelevant

## Features

- Full-screen terminal file workbench
- Directory navigation with keyboard-first controls
- Optional shell wrapper for visual `cd` from the terminal
- Inline file preview
- Enter fullscreen preview for files with full-content scrolling/search/jump
- Filter the current directory
- Quick file edit with `nano` (`e`)
- Copy, cut, paste, rename, and move workflows
- Safe trash flow before permanent deletion
- Copy file name, relative path, and absolute path
- Compact info pane and operation log
- Hidden file toggle

## Installation

Product name: **Tersh**. CLI tool name and crate name: `tersh`.

### Fastest GitHub Install

```bash
# macOS / Linux
# Installs the tersh CLI from GitHub, then starts it in the current directory.
set -eu

if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  . "$HOME/.cargo/env"
fi

export PATH="$HOME/.cargo/bin:$PATH"
cargo install --git https://github.com/QiushanHuang/Tersh.git --bin tersh --force

tersh --help
tersh
```

Run this same block on a remote server after SSH if you want to use Tersh there.

### Download The Source

Use this when you want the local source repository too:

```bash
git clone https://github.com/QiushanHuang/Tersh.git
cd Tersh
./scripts/install.sh
tersh
```

The install script builds `target/release/tersh` and installs it as the `tersh` command. Set `TERSH_INSTALL_DIR` to choose another install directory.

### Local Development Build

```bash
cargo build --release --bin tersh
./target/release/tersh
```

## Quick Start

Run `tersh` in the current directory:

```bash
tersh
```

Open a specific path:

```bash
tersh /var/www
```

Typical remote flow:

```bash
ssh user@host
cd /srv/app
tersh
```

Use Tersh as a visual `cd` by defining a shell function:

```bash
tersh-cd() {
  local target_dir
  target_dir="$(tersh --print-cwd "$@")" || return
  [ -n "$target_dir" ] && cd -- "$target_dir"
}
alias tcd=tersh-cd
```

Then run `tcd`, browse from directory A to directory B, and quit. Your shell will return in directory B. This works through a shell wrapper because a child process cannot directly change its parent shell directory.

## Keybindings

### Navigation

- `j` / `k` or arrow keys: move
- `PageUp` / `PageDown`: move by page
- `Home` / `End` or `gg` / `G`: jump to first / last item
- `h`: parent directory
- `l` or `Enter`: open
- `/`: filter current directory
- `:`: go to directory
- `.`: toggle hidden files
- `r`: refresh

### Preview Mode

- `Enter` on a file: open fullscreen preview
- `j` / `k`: scroll preview by page
- `↑` / `↓` / `Ctrl+F` / `Ctrl+B`: scroll preview line by line
- `PageUp` / `PageDown`: scroll preview by page
- `Home` or `gg`: jump to top
- `End` or `G`: jump to bottom
- `/`: find in preview
- `n` / `N`: next / previous match
- `e`: open current file in `nano`

### File Operations

- `Space`: mark selection
- `yy`: copy
- `x`: cut
- `p`: paste
- `c`: copy to directory
- `m`: move to directory
- `n`: rename focused item
- `d`: move to `.tersh-trash`
- `D`: permanently delete

### Copy Helpers

- `yf`: copy file name
- `yr`: copy relative path
- `ya`: copy absolute path

### Exit and Help

- `?`: help
- `Ctrl+G`: cancel
- `q`: quit
- `Q` or `Ctrl+C`: force quit

## Typical Use Cases

- Review deployment artifacts on a remote machine
- Inspect logs, config files, and generated output from a phone or tablet
- Clean up directories during lightweight ops work
- Copy paths and move files across multiple SSH-connected environments
- Use one consistent terminal workflow across many hosts

## Product Positioning

Tersh sits between raw shell commands and a full remote file manager.

It aims to keep the speed and portability of terminal work while removing repetitive friction from everyday remote file handling.

## Project Status

This project is in an early product stage.

The current implementation focuses on a stable core workflow:

- browse
- preview
- filter
- copy and move
- rename
- trash and delete
- path handling

The current scope does not try to be:

- an SSH client
- a background sync tool
- a desktop GUI file manager
- a remote host dashboard

## Architecture

The codebase is organized around a small Rust TUI core:

- `src/app.rs`: application state and interaction flow
- `src/ui.rs`: terminal layout and rendering
- `src/fs_core.rs`: file listing and metadata helpers
- `src/fs_ops.rs`: copy, rename, trash, delete, and path operations
- `src/preview.rs`: file preview logic
- `src/clipboard.rs`: clipboard integration helpers

## Roadmap

- Better preview coverage for more file types
- Tighter small-screen behavior for narrow mobile terminals
- More remote-friendly copy and batch workflows
- Configurable keymaps and behavior
- Packaging for easier install beyond local builds

## License

MIT

---

<a id="中文"></a>

# Tersh

[![简体中文](https://img.shields.io/badge/语言-简体中文-1677ff)](#中文)
[![English](https://img.shields.io/badge/Language-English-24292f)](#english)

**Tersh** 是一个面向本地目录和 SSH shell 会话的轻量、移动端友好的终端文件工作台。

它适合这样的场景：你已经在本地终端里，或者已经通过 SSH 连上远程主机，这时你需要一种比反复敲 `ls`、`cd`、`cat`、`cp`、`rm` 更顺手的方式来查阅文件、预览内容、切换目录并完成基础文件操作。

## 为什么是 Tersh

- 面向终端场景，既能本地用，也能在 SSH shell 里用
- 更适合窄终端、远程 shell 和轻量环境
- 足够轻，适合多终端办公和低负担接入
- 键盘优先，带安全删除与内联预览
- 在平板和手机上也能提供可用的远程文件工作流

## 本地与 SSH Shell 会话

Tersh 不是 SSH 客户端，也不是多主机会话管理器。

它的角色更聚焦，也更实用：你可以在本地目录直接运行 `tersh`，也可以先通过自己习惯的 SSH 工具进入服务器，然后在服务器目录里启动 `tersh`，完成文件查阅和轻量操作。

它适合这些工作：

- 远程内容检查
- 服务器侧文件快速排查
- 运维流程里的路径复制
- 在多个主机之间保持一致的轻量终端工作方式

## 面向移动端的终端工作流

这个产品方向刻意保持简单：

- 打开本地终端，或通过 SSH 进入主机
- 启动一个命令
- 浏览、筛选、预览、复制、移动、重命名和清理文件
- 全程停留在终端里完成操作

这对以下环境尤其重要：

- 在 iPad 上通过 SSH App 远程工作
- 用手机做紧急检查和快速处理
- 在轻量笔记本上同时管理多个终端
- 运行在不需要 GUI 的服务器环境里

## 功能特性

- 全屏终端文件工作台
- 键盘优先的目录浏览
- 可选 shell 包装函数，让 `tersh` 作为可视化 `cd` 使用
- 文件内联预览
- 回车进入全文预览，可快速滚动、跳转与查找
- 当前目录筛选
- 可在预览中用 `e` 快速调用 `nano` 编辑
- 复制、剪切、粘贴、重命名与移动
- 先入回收站再永久删除的安全流程
- 复制文件名、相对路径和绝对路径
- 紧凑的信息面板和操作日志
- 隐藏文件开关

## 安装

产品展示名：**Tersh**。CLI 命令名和 crate 名：`tersh`。

### 最快 GitHub 安装

```bash
# macOS / Linux
# 从 GitHub 安装 tersh CLI，然后直接在当前目录启动。
set -eu

if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  . "$HOME/.cargo/env"
fi

export PATH="$HOME/.cargo/bin:$PATH"
cargo install --git https://github.com/QiushanHuang/Tersh.git --bin tersh --force

tersh --help
tersh
```

如果你已经 SSH 到服务器上，就在服务器终端里运行同一段命令；这样 `tersh` 会安装到服务器本地。

### 下载源码

需要把源码仓库也下载到本地时，用这一段：

```bash
git clone https://github.com/QiushanHuang/Tersh.git
cd Tersh
./scripts/install.sh
tersh
```

安装脚本会构建 `target/release/tersh`，并把它安装成 `tersh` 命令。你也可以设置 `TERSH_INSTALL_DIR` 指定安装目录。

### 本地开发构建

```bash
cargo build --release --bin tersh
./target/release/tersh
```

## 快速开始

在当前目录启动 `tersh`：

```bash
tersh
```

打开指定目录：

```bash
tersh /var/www
```

典型远程使用方式：

```bash
ssh user@host
cd /srv/app
tersh
```

把 Tersh 当作可视化 `cd` 使用时，先在 shell 里定义函数：

```bash
tersh-cd() {
  local target_dir
  target_dir="$(tersh --print-cwd "$@")" || return
  [ -n "$target_dir" ] && cd -- "$target_dir"
}
alias tcd=tersh-cd
```

之后运行 `tcd`，从 A 目录浏览到 B 目录并退出，回到终端后当前目录就是 B。这里必须通过 shell 函数实现，因为子进程不能直接修改父 shell 的当前目录。

## 快捷键

### 导航

- `j` / `k` 或方向键：移动
- `PageUp` / `PageDown`：按页移动
- `Home` / `End` 或 `gg` / `G`：跳到第一项 / 最后一项
- `h`：返回上级目录
- `l` 或 `Enter`：打开
- `/`：筛选当前目录
- `:`：跳转目录
- `.`：切换隐藏文件显示
- `r`：刷新

### 全屏预览

- `Enter`：对当前文件进入全文预览
- `j` / `k`：按页滚动
- `↑` / `↓` / `Ctrl+F` / `Ctrl+B`：按行上下滚动
- `PageUp` / `PageDown`：按页滚动
- `Home` 或 `gg`：跳到顶部
- `End` 或 `G`：跳到底部
- `/`：在预览中查找
- `n` / `N`：下一个 / 上一个匹配
- `e`：使用 `nano` 打开当前文件编辑

### 文件操作

- `Space`：标记选择
- `yy`：复制
- `x`：剪切
- `p`：粘贴
- `c`：复制到目标目录
- `m`：移动到目标目录
- `n`：重命名当前项
- `d`：移动到 `.tersh-trash`
- `D`：永久删除

### 复制辅助

- `yf`：复制文件名
- `yr`：复制相对路径
- `ya`：复制绝对路径

### 退出与帮助

- `?`：帮助
- `Ctrl+G`：取消
- `q`：退出
- `Q` 或 `Ctrl+C`：强制退出

## 典型使用场景

- 在远程主机上检查部署产物
- 用平板或手机查看日志、配置和生成文件
- 在轻量运维工作中清理目录
- 在多个 SSH 环境之间复制路径和整理文件
- 在不同主机上保持一致的终端文件工作流

## 产品定位

Tersh 处在原始 shell 命令和完整远程文件管理器之间。

它希望保留终端工作的速度和便携性，同时去掉日常远程文件处理中的重复摩擦。

## 项目状态

项目目前处于早期产品阶段。

当前实现聚焦在一条稳定核心链路上：

- 浏览
- 预览
- 筛选
- 复制与移动
- 重命名
- 回收站与删除
- 路径处理

当前范围不包含：

- SSH 客户端
- 后台同步工具
- 桌面 GUI 文件管理器
- 远程主机控制面板

## 架构说明

代码目前围绕一个小而清晰的 Rust TUI 核心组织：

- `src/app.rs`：应用状态和交互流程
- `src/ui.rs`：终端布局和渲染
- `src/fs_core.rs`：文件枚举与元数据辅助
- `src/fs_ops.rs`：复制、重命名、回收站、删除与路径操作
- `src/preview.rs`：文件预览逻辑
- `src/clipboard.rs`：剪贴板集成辅助

## 路线图

- 扩展更多文件类型的预览能力
- 进一步优化窄屏和移动端终端体验
- 增强面向远程环境的复制与批量工作流
- 支持可配置快捷键和行为选项
- 提供更容易安装的分发方式

## 许可证

MIT
