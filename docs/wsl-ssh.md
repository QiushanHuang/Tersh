# WSL SSH lifecycle installation patch

The patch is included in development version **1.3.0-dev.2**. It prevents a known
failure mode of SSH → Windows → WSL → netcat routes: disconnected sessions can
leave WSL relay processes alive until repeated probes exhaust WSL resources.
It does not promise that every network or WSL startup failure is recoverable.

## Installation

Use `./scripts/install.sh` from a source checkout, or `./install.sh` from an
archive built from this revision. Both install the binary and invoke the same
compiled patch command. An incomplete patch makes the installer fail explicitly,
even if the executable has already been installed. Fix the reported issue and
rerun the patch; do not assume a copied binary means the connection is protected.

Cargo does not run post-install hooks. After installing **this revision or a
later build containing the patch**, run:

```sh
tersh --install-patches
tersh --check-install-patches
```

Do this under the SSH configuration's user account on each client/gateway, and
inside the WSL distribution hosting the dedicated endpoint. An endpoint owned
by root requires appropriate write and service-control permissions; for example,
after patching your user's client configuration, run `sudo /path/to/tersh
--install-patches` inside WSL. The command never invokes sudo itself or connects
to other devices. Installing on a Mac alone does not patch an endpoint in WSL.

Ordinary file-workbench and cluster startup never applies installation patches.
`--check-install-patches` is headless and read-only and exits nonzero when a
required change or an unsupported recognized route is found.

## Client and gateway policy

The command inspects the current user's `~/.ssh/config`. For flat `Host` stanzas
with a recognizable direct WSL netcat invocation, it preserves aliases, relay
hosts, WSL distribution/user options and loopback ports while applying:

```sshconfig
Host home-wsl
  # Existing identity, host-key, user and routing configuration remains in place.
  ProxyCommand ssh -T gateway ssh -T windows-host wsl.exe -d Ubuntu -- /usr/bin/nc -q 0 -w 30 127.0.0.1 2222
  ServerAliveInterval 10
  ServerAliveCountMax 2
```

The example names and port are illustrative, not built into detection. The
client heartbeat is required as well as the relay deadline: it keeps a healthy
idle session active even if the server has not yet been upgraded. The explicit
WSL invocation is retained so the route can start a stopped WSL instance.

Custom shell wrappers, unsupported netcat flags, conditional `Match` blocks and
conflicting earlier/shared defaults are rejected rather than guessed. Files
referenced by `Include` are outside the selected file's scope; use
`--patch-ssh-config /real/path/to/config` to select a concrete configuration, and
resolve cross-file precedence before applying. A selected file combining WSL
routes and `Include` requires manual review. Symlinked configuration files are
not replaced: select the real file explicitly.

## Existing Tersh endpoint in WSL

On WSL only, the patch inspects the existing
`/etc/ssh/tersh_sshd_config`, applying:

```sshconfig
ClientAliveInterval 10
ClientAliveCountMax 2
LoginGraceTime 30
```

The installer does **not** create SSH services, authorized keys, users, open
ports, or change `/etc/ssh/sshd_config`. When provisioning a new Tersh endpoint,
include these limits from the beginning and use the dedicated file and
`tersh-sshd.service` names. A missing dedicated endpoint is reported as absent;
installing a file manager is not treated as permission to enable remote login.

Before replacing an endpoint file, Tersh validates its staged candidate with
`sshd -t`. If the dedicated service is active, it verifies that its main process
uses this configuration, then sends HUP only to that listener. An inactive
endpoint reads the protected configuration at its next start. No global SSH
service is restarted. `Include` and `Match` in the dedicated endpoint require
manual resolution, since they can override the intended limits.

## Repeat installation, recovery and scope

Already protected files remain byte-identical and are not rewritten. Changed
files receive one sibling backup named `<file>.tersh-before-wsl-lifecycle`;
existing backups are retained. Candidates and backups are written completely
before publication, permission modes are preserved, and a validation failure
leaves the original untouched. A reload failure attempts to restore all files
changed in that invocation and reports any rollback failure. Concurrent edits
are detected before replacement rather than silently overwritten.

The end-to-end heartbeat and relay deadline cover loss of client traffic,
normal disconnection and login stalls. In the original two-device deployment,
fault injection showed stalled relays returning in about 31–63 seconds with the
combined policies; healthy idle sessions remained connected. The precise time
includes both SSH and relay timers and depends on implementation and scheduling.
These are connection lifetimes, not hard deadlines for arbitrary remote jobs
that still have a responsive SSH client, nor a watchdog for a WSL service hung
before netcat starts.

References: [OpenSSH client configuration](https://man.openbsd.org/ssh_config),
[OpenSSH server configuration](https://man.openbsd.org/sshd_config),
[Debian netcat](https://manpages.debian.org/unstable/netcat-openbsd/nc_openbsd.1.en.html).
