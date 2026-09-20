# Tersh

[English](README.md#english) · [简体中文](README.md#中文)

**文件、主机状态与可恢复的操作，都在你正在使用的终端里。**

一次简单检查，经常变成 `ls` → `cd` → `cat` → `cp` → 切换另一台服务器。
反复输入路径、确认操作对象、核对任务结果，才是这些日常工作中容易累积的成本。
Tersh 把它们组织成一个可见、可搜索、可用键盘完成的操作流程。

## 为什么选 Tersh？

**如果你经常在本地或 SSH 中检查和整理文件，希望少输路径、少切工具、看清操作结果，
Tersh 就是为这类工作准备的。** 它的优势在于把文件工作台、SSH 主机概览和明确的任务反馈
放在一起，并让这套操作能适应笔记本终端与手机 SSH 会话。

| 现有工作方式容易遇到的问题 | Tersh 的做法 | 直接收益 |
| --- | --- | --- |
| 用一串命令交互式浏览文件，要反复输入路径、自己记住操作对象。 | 在同一视图中浏览、筛选、预览、选择并操作。 | 减少重复输入，让当前目标更直观。 |
| 已经连上 SSH，却还要为桌面文件管理器切换连接或另起一套操作流程。 | 在文件所在的机器上运行，沿用现有终端连接。 | 留在当前会话中完成可视化文件操作。 |
| 看主机状态和处理文件往往分散在不同工具、不同步骤里。 | `tersh --c` 查看主机与路由，按 `t` 进入选中主机的文件工作台。 | 从选机器到看文件衔接起来。 |
| 临时复制、清理后，还要重新核对哪些完成了、删掉的文件原来在哪里。 | 后台任务报告部分完成结果；受管理的回收站记录原位置。 | 结果可见，任务可取消，恢复不覆盖同名目标。 |
| 手机 SSH 屏幕窄、键盘不全，长快捷键列表难记又难用。 | `o` 搜索操作；底部提示跟随页面和实际键位变化。 | 更容易找到操作，适配不同设备。 |

### 什么时候特别有用？

- **检查开发或科研服务器**：筛选主机、进入工作台、查看输出目录、搜索日志，不必先把文件下载回来。
- **用手机临时处理问题**：通过常用 SSH 客户端连接，在服务器上运行 `tersh`，用紧凑提示或操作菜单找到文件。
- **整理本地项目或远端工作目录**：先预览再操作，多选复制时继续浏览，需要时查看进度或恢复之前移入回收站的项目。

### 半分钟了解主要功能

| 你想做什么 | 从这里开始 |
| --- | --- |
| 打开编辑器前先确认文件内容 | `tersh`，Enter 预览，`/` 查找 |
| 复制时继续浏览，并掌握进度 | 复制/粘贴，`J` 查看任务，Ctrl+X 请求取消 |
| 恢复用 Tersh 移入回收站的项目 | `u`，查看原路径后确认恢复 |
| 找到目标服务器并开始处理文件 | `tersh --c`，`/` 筛选，`l` 查看详情，`t` 进入工作台 |
| 不记得某项操作的快捷键 | `o` 搜索名称，再选择执行 |

**轻量有具体设计支撑**：单个可执行文件、有界缓存与主机历史、按变化重绘，没有常驻监控代理。
预编译的 macOS/Linux 程序无需 Rust 工具链即可运行。健康检查需要 SSH；打开远端文件工作台还需要在目标机器安装 Tersh。
文件操作发生在该工作台所在的主机上。

<details>
<summary>与其他工具如何搭配、各自适合什么</summary>

脚本和重复自动化仍适合直接用 shell 命令。
[Yazi](https://yazi-rs.github.io/features/) 提供更丰富的文件预览与插件生态，
[btop](https://github.com/aristocratos/btop) 专注于详细的系统资源监控。
Tersh 聚焦“选主机 → 检查文件 → 有明确反馈的操作 → 必要时恢复”这条工作路径。
这里比较的是使用侧重点，没有宣称 Tersh 在速度或功能数量上全面领先。
目录枚举和预览读取目前仍是同步的；取消保留已完成操作，不等于撤销。

</details>

![文件工作台、预览与上下文快捷键](docs/images/workbench.png)

*截图使用模拟数据；实际颜色由终端配色决定。*

## 安装

从 [Releases](https://github.com/QiushanHuang/Tersh/releases) 下载对应平台的程序，
或使用 **Rust 1.88 及以上版本**从源码安装：

```sh
cargo install --locked --git https://github.com/QiushanHuang/Tersh.git --tag v1.2.0 --bin tersh
```

更新已有 Cargo 安装时加 `--force`。也可以克隆后运行安装脚本：

```sh
git clone https://github.com/QiushanHuang/Tersh.git
cd Tersh
./scripts/install.sh
```

脚本优先使用已有安装位置，否则选择可写的用户 bin 目录。
可用 `TERSH_INSTALL_DIR` 指定位置。远程文件操作需在服务器上安装 Tersh，
然后通过常用 SSH 客户端登录运行。

## 快速开始

```sh
tersh
tersh /path/to/project
tersh README.md
tersh --c
tersh --ui-profile desktop
tersh --ui-profile mobile --theme contrast
tersh --ui-profile ssh
```

按 **`o`** 搜索操作，按 **`?`** 查看当前键位。操作菜单中输入名称，
用方向键/Tab 选择、Enter 执行、Esc 返回。以下为默认键位；自定义后，
菜单、帮助和底部提示会同步更新。

| 操作 | 默认键位 |
| --- | --- |
| 移动 / 打开 / 上级目录 | `j`/`k` 或方向键 · Enter · `h` 或 Backspace |
| 筛选 / 隐藏文件 / 排序 | `/` · `.` · `s`/`S` |
| 选择 / 复制 / 剪切 / 粘贴 | Space · `yy` · `x` · `p` |
| 复制到 / 移动到 / 重命名 | `c` · `m` · `n` |
| 任务与结果 / 请求取消 | `J` · Ctrl+X |
| 移入回收站 / 恢复 / 永久删除 | `d` · `u` · `D` |
| 预览查找 / 下一个匹配 / 编辑 | `/` · `n`/`N` · `e` |
| 复制文件名 / 相对路径 / 绝对路径 | `yf` · `yr` · `ya` |
| 退出 / 取消 / 安全紧急退出 | `q` · Esc 或 Ctrl+G · Ctrl+C |

编辑器依次使用 `$VISUAL`、`$EDITOR`、`nano`。
需要可视化 `cd` 时，在 shell 中加载 [tersh-cd.sh](scripts/tersh-cd.sh)，
然后使用 `tersh-cd`。

## 后台任务与恢复

复制、移动、回收站、永久删除和恢复由单个后台线程执行。运行期间仍能浏览，
新的写操作会被拒绝，直到当前任务结束。`J` 显示进度及完成、失败、跳过、剩余项目。

取消保留已经完成的操作，并清理本次未完成的复制。替换先完成暂存再提交，
不允许覆盖已有目录；延迟执行前会重新检查源文件和已确认目标的身份。
任务期间 Ctrl+C 会请求取消、等待清理后退出。协作取消无法中断阻塞的系统调用，
也不能撤销已经完成的永久删除。

`u` 打开当前工作根目录的回收站记录，Enter 在显示原位置后确认恢复。
同名目标不会覆盖；损坏记录会跳过并提示，其他有效记录仍可使用。
恢复记录跨重启保留，要求同一文件系统内重命名和可记录的 UTF-8 路径。
旧版没有原位置记录的垃圾文件不会被猜测或改动。

## 主机与趋势

![集群详情与观测趋势](docs/images/cluster.png)

```sh
tersh --c --cluster-config examples/servers.json
```

先复制并编辑[清单示例](examples/servers.json)，填入自己的主机。
示例中的 `.example` 地址仅作占位。健康探测使用非交互 SSH、已有凭据和已信任的主机密钥。

| 操作 | 默认键位 |
| --- | --- |
| 按别名、地址、角色筛选 / 清除 | `/` · Backspace |
| 切换排序 / 反转 | `v` · `V` |
| 刷新全部 / 刷新选中主机 | `r` · Enter |
| 展开详情 / 滚动详情 | `l` · PageUp/PageDown |
| 打开 shell/SSH / 打开远端 Tersh | `s` · `t` |

筛选排序只改变视图，不改变完整清单的探测范围。选择按主机别名保留；
未知指标在正反排序中都放在末尾。远端 `t` 要求目标已安装 Tersh，退出后返回主机面板。

每台主机最多在内存保留 60 次观测。缺口表示失败或缺失；CPU load 是 1 分钟负载，
内存统一为已用百分比，Probe 包含整个探测过程的耗时。横向按观测顺序排列并标明实际时间跨度，
不会为了画图增加探测。

## 配色与键位

```sh
tersh --theme aurora
tersh --theme mono
tersh --no-motion
tersh --dump-keymap > keymap.json
tersh --keymap keymap.json
```

主题包括 `btop`、`aurora`、`contrast`、`mono`；`NO_COLOR` 关闭颜色。
桌面、移动端、SSH 预设调整边框、图形字符、提示密度和动效，不改按键，也不写入配置。

自定义键位只需覆盖需要的部分：

```json
{
  "files": {
    "copy": ["Ctrl+y"],
    "open_jobs": ["F5", "J"],
    "open_trash": ["F6", "u"]
  }
}
```

这会用 Ctrl+Y 替换原来的 `yy`，同时为缺少功能键的设备保留单字母入口。
完整上下文、配置优先级、组合键、环境变量和主机字段见[配置说明](docs/configuration.md)。

## 开发与贡献

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
```

[贡献指南](CONTRIBUTING.md) · [更新记录](CHANGELOG.md) ·
[v1.2.0 说明](docs/releases/v1.2.0.md)

维护者：[QiushanHuang](https://github.com/QiushanHuang)。
贡献署名见 [CONTRIBUTORS.md](CONTRIBUTORS.md)。采用 [MIT 许可](LICENSE)，保留原有版权声明。
