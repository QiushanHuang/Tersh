#!/usr/bin/env python3
"""Roll out a verified Tersh release and bound private inventory over existing SSH routes.

Active installations, credentials, SSH config and known-hosts files are never
copied between hosts. Use --apply to update reachable devices. Without it,
online mode reads the selected targets and reports what would change.
"""
import argparse
import base64
import concurrent.futures
import functools
import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

WORKER = Path(__file__).with_name("fleet_remote.py")
PLATFORMS = {("Darwin", "arm64"): "macos-arm64", ("Linux", "x86_64"): "linux-x86_64",
             ("Linux", "aarch64"): "linux-arm64", ("Linux", "arm64"): "linux-arm64"}
SSH_OPTIONS = ["-o", "BatchMode=yes", "-o", "ConnectTimeout=10", "-o", "ConnectionAttempts=1",
               "-o", "StrictHostKeyChecking=yes", "-o", "UpdateHostKeys=no", "-o", "ForwardAgent=no",
               "-o", "PermitLocalCommand=no", "-o", "ClearAllForwardings=yes",
               "-o", "ServerAliveInterval=10", "-o", "ServerAliveCountMax=2"]


def positive_version(value):
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.-]+)?", value):
        raise argparse.ArgumentTypeError("use a plain release version, for example 1.4.0")
    return value


def ssh_target(host):
    addr = host.get("campus_ip") or host.get("tailscale_ip") or host.get("device_name") or host.get("alias")
    user = host.get("ssh_user")
    for part in (addr, user) if user is not None else (addr,):
        if not isinstance(part, str) or not re.fullmatch(r"[A-Za-z0-9_.:\[\]-]+", part) or part.startswith("-"):
            raise ValueError("Invalid SSH account or target for " + repr(host.get("alias")))
    return user + "@" + addr if user else addr


def devices(inventory):
    main = inventory.get("main_machine")
    jump = inventory.get("jump_host")
    hosts = ([main] if main else []) + ([jump] if jump else []) + inventory.get("servers", [])
    by_name = {}
    for host in hosts:
        name = host.get("alias")
        if not isinstance(name, str) or not name.strip() or name in by_name:
            raise ValueError("Fleet entries require unique explicit aliases")
        ssh_target(host)
        if host.get("proxy_jump") and (not jump or host["proxy_jump"] != jump["alias"]):
            raise ValueError("Unresolved proxy_jump for " + name)
        by_name[name] = host
    return by_name


def bound(inventory, alias=None, observer=False):
    document = dict(inventory)
    document.pop("local_alias", None)
    document.pop("observer_only", None)
    if observer:
        document["observer_only"] = True
    else:
        if alias not in devices(inventory):
            raise ValueError("Cannot bind an unknown device")
        document["local_alias"] = alias
    data = (json.dumps(document, indent=2, ensure_ascii=False) + "\n").encode()
    if len(data) > 65536:
        raise ValueError("Private inventory exceeds 64 KiB")
    return data


@functools.lru_cache(maxsize=None)
def artifact(directory, version, platform):
    archive = directory / ("tersh-v{}-{}.tar.gz".format(version, platform))
    sidecar = archive.with_suffix(".gz.sha256")
    record = sidecar.read_text().strip().split()
    if len(record) != 2 or record[1].lstrip("*") != archive.name or not re.fullmatch(r"[\da-fA-F]{64}", record[0]):
        raise ValueError("Invalid release checksum file: " + sidecar.name)
    archive_digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    if archive_digest != record[0].lower():
        raise ValueError("Release archive checksum mismatch: " + archive.name)
    with tarfile.open(archive, "r:gz") as bundle:
        members = [entry for entry in bundle if entry.name == "tersh"]
        if len(members) != 1 or not members[0].isfile() or not 0 < members[0].size <= 64 * 1024 * 1024:
            raise ValueError("Expected one regular tersh executable in " + archive.name)
        binary = bundle.extractfile(members[0]).read()
    return binary, hashlib.sha256(binary).hexdigest()


def endpoint_command(inventory, name, host):
    main = inventory.get("main_machine") or {}
    controller = None if inventory.get("observer_only") else inventory.get("local_alias", main.get("alias"))
    if name == controller:
        return [sys.executable, str(WORKER)]
    args = ["ssh", *SSH_OPTIONS]
    jump = inventory.get("jump_host") or {}
    if host.get("proxy_jump") and controller != jump.get("alias"):
        # Quote the proxy helper separately so the inner hop also enforces the same host-key policy.
        proxy = shlex.join(["ssh", *SSH_OPTIONS, "-W", "%h:%p", "--", ssh_target(jump)])
        args.extend(["-o", "ProxyCommand=" + proxy])
    elif controller == jump.get("alias") and host.get("proxy_jump"):
        args.extend(["-o", "ProxyJump=none", "-o", "ProxyCommand=none"])
    args.extend(["--", ssh_target(host), "python3 -c " + shlex.quote(WORKER.read_text())])
    return args


def endpoint(argv, request):
    r = subprocess.run(argv, input=json.dumps(request), text=True, stdout=subprocess.PIPE,
                       stderr=subprocess.PIPE, timeout=120)
    if r.returncode:
        message = r.stderr.strip().splitlines()
        raise ValueError(message[-1] if message else "Endpoint command failed")
    return json.loads(r.stdout)


def reconcile_hashes(values, hosts):
    result = {}
    for value in values:
        name, sep, sha = value.partition("=")
        if not sep or name not in hosts or not re.fullmatch("[0-9a-f]{64}", sha):
            raise ValueError("--reconcile must be a selected alias and a reviewed SHA-256")
        result[name] = sha
    return result


def sync(args, inventory, name, host, accepted):
    command = endpoint_command(inventory, name, host)
    state = endpoint(command, {"operation": "inspect"})
    platform = PLATFORMS.get((state["os"], state["arch"]))
    if not platform:
        raise ValueError("Unsupported endpoint platform: {} {}".format(state["os"], state["arch"]))
    binary, digest = artifact(args.release_dir, args.version, platform)
    # The controller's source inventory (often a symlink) is authoritative. Only imported copies are bound.
    config = None if len(command) == 2 else base64.b64encode(bound(inventory, name)).decode()
    request = {"operation": "plan", "version": args.version, "binary_sha256": digest, "inventory": config,
               "inspected": state, "reconcile": accepted}
    result = endpoint(command, request)
    if args.apply and result["changed"]:
        request.update(operation="apply", binary=base64.b64encode(binary).decode())
        result = endpoint(command, request)
    return {"host": name, "platform": platform, "mode": "apply" if args.apply else "plan",
            "executable_sha256": digest, **result}


def offline_package(args, inventory):
    if args.offline_dir.exists():
        raise ValueError("Offline package destination must be new: " + str(args.offline_dir))
    alias = args.host[0] if args.host else None
    if bool(alias) == bool(args.observer_only) or len(args.host) > 1 or not args.platform:
        raise ValueError("Offline mode requires --platform and either one --host or --observer-only NAME")
    data = bound(inventory, alias, observer=bool(args.observer_only))
    binary, digest = artifact(args.release_dir, args.version, args.platform)
    manifest = {"platform": args.platform, "version": args.version, "binary_sha256": digest,
                "inventory_sha256": hashlib.sha256(data).hexdigest(), "alias": alias,
                "observer_label": args.observer_only}
    args.offline_dir.parent.mkdir(parents=True, exist_ok=True)
    tmp = Path(tempfile.mkdtemp(prefix=".tersh-private-", dir=str(args.offline_dir.parent)))
    try:
        for name, content, mode in [
            ("tersh", binary, 0o755), ("servers.json", data, 0o600),
            ("manifest.json", (json.dumps(manifest, indent=2) + "\n").encode(), 0o600),
            ("fleet_remote.py", WORKER.read_bytes(), 0o600),
            ("install.sh", b'#!/bin/sh\nset -eu\ncd -- "$(dirname -- "$0")"\nexec python3 ./fleet_remote.py --bundle "$@"\n', 0o700),
        ]:
            destination = tmp / name
            destination.write_bytes(content)
            destination.chmod(mode)
        tmp.rename(args.offline_dir)
    finally:
        if tmp.exists():
            shutil.rmtree(tmp)
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", type=Path, default=Path.home() / ".config/tersh/servers.json")
    parser.add_argument("--release-dir", type=Path, required=True, help="official archives and matching .sha256 files")
    parser.add_argument("--version", type=positive_version, required=True)
    parser.add_argument("--host", action="append", default=[], help="exact inventory alias (repeatable); default all")
    parser.add_argument("--main-user", help="verified SSH account for observers connecting to main_machine")
    parser.add_argument("--apply", action="store_true", help="activate staged files; default is read-only remote preflight")
    parser.add_argument("--reconcile", action="append", default=[], metavar="ALIAS=SHA256",
                        help="replace the reviewed private inventory with this exact current digest, with backup")
    parser.add_argument("--offline-dir", type=Path, help="prepare a private offline installation directory instead of connecting")
    parser.add_argument("--platform", choices=sorted(set(PLATFORMS.values())))
    parser.add_argument("--observer-only", metavar="LABEL", help="offline package for an unregistered observer")
    args = parser.parse_args()
    inventory = json.loads(args.inventory.read_text())
    main_device = inventory.get("main_machine")
    if main_device and args.main_user:
        if main_device.get("ssh_user") not in (None, args.main_user):
            parser.error("Existing main_machine.ssh_user disagrees with --main-user")
        main_device["ssh_user"] = args.main_user
    hosts = devices(inventory)
    if any(alias not in hosts for alias in args.host):
        parser.error("Each --host must be an exact registered alias")
    if args.offline_dir:
        if args.apply or args.reconcile:
            parser.error("Offline preparation does not activate remote changes")
        print(json.dumps(offline_package(args, inventory), indent=2))
        return 0
    if args.observer_only or args.platform:
        parser.error("--observer-only and --platform require --offline-dir")
    selected = {name: hosts[name] for name in (args.host or hosts)}
    accepted = reconcile_hashes(args.reconcile, selected)
    failures = 0
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as workers:
        tasks = {workers.submit(sync, args, inventory, name, host, accepted.get(name)): name for name, host in selected.items()}
        for task in concurrent.futures.as_completed(tasks):
            try:
                record = task.result()
            except (ValueError, OSError, subprocess.SubprocessError, tarfile.TarError) as exc:
                record = {"host": tasks[task], "error": str(exc)}
                failures += 1
            print(json.dumps(record, ensure_ascii=False), flush=True)
    return int(bool(failures))


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, OSError, tarfile.TarError) as exc:
        sys.exit(str(exc))
