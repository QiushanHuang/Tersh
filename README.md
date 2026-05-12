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
- Inline file preview
- Filter the current directory
- Copy, cut, paste, rename, and move workflows
- Safe trash flow before permanent deletion
- Copy file name, relative path, and absolute path
- Compact info pane and operation log
- Hidden file toggle

## Installation

### Cargo

```bash
cargo install --path .
```

This installs the CLI tool as `tersh`.

### Install script

```bash
./scripts/install.sh
```

By default this installs `target/release/tersh` as `tersh` into a writable common directory on your `PATH`. Set `TERSH_INSTALL_DIR` to choose another install directory.

### Build locally

```bash
cargo build --release --bin tersh
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

## Keybindings

### Navigation

- `j` / `k` or arrow keys: move
- `h`: parent directory
- `l` or `Enter`: open
- `/`: filter current directory
- `:`: go to directory
- `.`: toggle hidden files
- `r`: refresh

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
- `Esc`: cancel
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
- 文件内联预览
- 当前目录筛选
- 复制、剪切、粘贴、重命名与移动
- 先入回收站再永久删除的安全流程
- 复制文件名、相对路径和绝对路径
- 紧凑的信息面板和操作日志
- 隐藏文件开关

## 安装

### Cargo

```bash
cargo install --path .
```

这会把 CLI 命令安装为 `tersh`。

### 安装脚本

```bash
./scripts/install.sh
```

默认会把 `target/release/tersh` 作为 `tersh` 安装到一个常见且可写的 `PATH` 目录。你也可以设置 `TERSH_INSTALL_DIR` 指定安装目录。

### 本地构建

```bash
cargo build --release --bin tersh
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

## 快捷键

### 导航

- `j` / `k` 或方向键：移动
- `h`：返回上级目录
- `l` 或 `Enter`：打开
- `/`：筛选当前目录
- `:`：跳转目录
- `.`：切换隐藏文件显示
- `r`：刷新

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
- `Esc`：取消
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
