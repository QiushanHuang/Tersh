# Tersh

[English](README.md) · [简体中文](README.zh-CN.md)

**面向本地终端和 SSH 会话的轻量文件工作台。**
在当前终端里浏览、预览和整理文件，用 `tersh --c` 查看多台主机的健康状态，
并进入选中主机的 shell 或文件工作台。

![文件工作台、预览与上下文快捷键](docs/images/workbench.png)

*截图使用模拟数据；实际颜色由终端配色决定。*

## 能做什么

| 使用场景 | 功能 |
| --- | --- |
| 快速检查文件 | 筛选、排序、内嵌/全屏预览、全文查找、外部编辑器 |
| 批量文件操作 | 单个后台工作线程，进度、取消与明确的部分完成结果 |
| 清理后恢复 | 持久化回收站记录、原位置确认、同名文件不覆盖 |
| 多台服务器 | SSH 探测、跳板路由、主机筛选排序、有限长度的趋势历史 |
| 手机与不同终端 | 自适应布局、可搜索操作菜单、自定义键位、ASCII 降级 |
| 保持轻量 | 单个可执行文件，有界缓存与历史，无常驻代理，按变化重绘 |

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
