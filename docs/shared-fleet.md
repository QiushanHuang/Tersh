# Shared device inventories and updates

A Tersh dashboard reads a local inventory. Installing the executable alone
gives a new computer its local fallback. To see the same roster on multiple
machines, keep a private inventory on a controller and distribute a bound copy
to each receiver. Run the following on a receiver to check the source file,
local identity, aliases and effective jumps without probing any device:

```sh
tersh --c --list-hosts
```

## Device identity and routes

Use the existing `main_machine`, `jump_host` and `servers` records. Give
every fleet entry an explicit, unique alias and an address resolvable from
the other participants. `main_machine.ssh_user` specifies the login account
when another device connects to the controller.

```json
{
  "local_alias": "studio",
  "main_machine": {"alias": "desk", "tailscale_ip": "desk.example", "ssh_user": "operator"},
  "jump_host": {"alias": "gateway", "device_name": "gateway.example", "ssh_user": "operator"},
  "servers": [
    {"alias": "studio", "campus_ip": "studio.example", "ssh_user": "operator", "proxy_jump": "gateway"},
    {"alias": "compute", "campus_ip": "compute.example", "ssh_user": "operator", "proxy_jump": "gateway"}
  ]
}
```

On the receiver named by `local_alias`, only that machine is sampled locally.
A gateway reaches its own downstream devices without jumping through itself.
All other targets use their effective SSH routes. An unregistered viewer uses
`"observer_only": true` in place of `local_alias`; it sees the registered
devices and treats them as remote. Legacy inventories keep their existing
controller-local behavior.

Status is measured from the terminal you opened. Sharing the roster does not
grant login permissions or replicate private keys, SSH aliases, known-host
records or live measurements. A listed device can be unreachable from a given
receiver. The dashboard shows the corresponding probe result.

## Deploy from a controller

Use a v1.4.0 or newer source checkout and Python 3.8+ on the controller.
Endpoints need Python 3.6+ and an existing trusted SSH route. The controller
downloads the public release packages once. Keep the actual inventory and any
offline bundles private.

```sh
mkdir -p release-assets
gh release download v1.4.0 -R QiushanHuang/Tersh -D release-assets
python3 scripts/sync-fleet.py --inventory ~/.config/tersh/servers.json \
  --version 1.4.0 --release-dir release-assets
```

The first run reports a plan. Add `--apply` to install, or add `--host ALIAS`
to target a subset. If the controller login is missing from the inventory,
pass `--main-user USER`. Run this from the controller named by the source
inventory, or use a source with an explicitly bound `local_alias`.

Each receiver gets the executable for its OS/architecture and its own private
inventory. A differing hand-maintained inventory requires review; the plan
reports its current SHA-256. After checking the contents, pass
`--reconcile ALIAS=SHA256` to authorize replacing that exact revision.
A later sync can refresh an unchanged managed copy. Rebinding an existing copy
to a different observer still requires the reviewed digest.
The tool verifies the staged files, creates backups for changed existing files
and reports their paths. Rerun after changing the authoritative roster or to
retry an unavailable endpoint. Devices sharing a home and configuration path
must use separate inventory files with `TERSH_SERVERS_JSON` if their local
aliases differ.

Fleet deployment changes the program and inventory. On a new WSL route, use
`tersh --check-install-patches` to inspect the separate [SSH lifecycle settings](wsl-ssh.md).

## Offline or unregistered computer

Build a private bundle on the controller using verified release assets:

```sh
python3 scripts/sync-fleet.py --inventory ~/.config/tersh/servers.json \
  --version 1.4.0 --release-dir release-assets --platform macos-arm64 \
  --observer-only Studio --offline-dir ./studio-private
```

Transfer the directory privately, open it on the Mac and run `sh install.sh`.
For a registered device use one `--host ALIAS` instead of `--observer-only NAME`.
The name in observer mode labels the bundle. To monitor that computer from
other dashboards, add its verified address, account and route to the controller
inventory and sync again. Check the result with `tersh --c --list-hosts`.

## Software updates

`tersh` and `tersh --c` check the official GitHub stable release in the
background on each interactive launch. A newer compatible release adds a
notice and a yes/no prompt after the terminal is restored on exit. Press
Enter to keep the installed binary. To make the decision immediately:

```sh
tersh --check-update
tersh --update
```

`--update --yes` explicitly accepts a scripted update. Archive name, platform,
SHA-256 checksum, GitHub asset digest (when available) and executable version
are checked before same-directory replacement. Self-update changes the running
executable's installation path. The fleet distributor also handles multiple
local installs. The updated version takes effect at the next launch.

The metadata request times out after three seconds. An offline dashboard can
still start; `--check-update` reports connection errors explicitly. If a node
cannot download the asset, use controller deployment or an offline bundle.
`TERSH_UPDATE_CHECK=off` disables automatic checks. Use the
[unified installer](../scripts/install-release.sh) once to bring older versions
without an updater onto this release line.

## 中文

**在 Studio 看到同一批设备**：把主控清单的私有副本安装到 Studio。已登记的
Studio 在副本顶层设置 `local_alias` 为它自己的别名；未登记的 Studio 用
`observer_only: true`。在 Studio 运行 `tersh --c --list-hosts`，可以直接
核对实际读取的清单、全部别名和本机身份。要让其他设备反过来也能监测
Studio，需要先把它的真实地址、登录账号和跳板关系登记进主控清单。
每台机器使用自己的 SSH 凭据进行探测，因此同一目标在不同观察端上的状态可能不同。

**一次分发多机安装**：在主控上下载本版本的三个平台包和配套摘要，按上方
`sync-fleet.py` 示例先预演，再加 `--apply` 执行。可以使用 `--host` 只重试某台
机器。不同的已有清单会显示待核对的摘要；确认内容后使用
`--reconcile 别名=摘要`。程序会备份被替换的文件。清单增删设备后重新同步即可。
控制端无法连接的机器可使用上方离线包，私下传输到目标设备后运行 `sh install.sh`。

**以后更新**：每次交互启动都后台检查 GitHub 稳定版。有新版本时，界面显示
提示，退出时输入 `y` 才安装；直接回车会保留当前程序。运行 `tersh --update`
可立即检查和选择，`tersh --check-update` 只查版本。无法访问 GitHub 的节点
可以继续使用，并通过主控或离线包更新。
