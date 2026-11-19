"""POSIX endpoint helper for sync-fleet.py. Compatible with Python 3.6+."""
import base64
import fcntl
import hashlib
import json
import os
import platform
import re
import shutil
import stat
import subprocess
import sys
import tempfile
from pathlib import Path


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_sha(path):
    if not path.is_file():
        if path.is_symlink() or path.exists():
            raise ValueError("Expected a regular file: {}".format(path))
        return None
    h = hashlib.sha256()
    with path.open("rb") as src:
        for block in iter(lambda: src.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def locations(home):
    conventional = [home / ".cargo/bin/tersh", home / ".local/bin/tersh", home / "bin/tersh",
                    Path("/opt/homebrew/bin/tersh"), Path("/usr/local/bin/tersh")]
    active = shutil.which("tersh")
    paths = ([Path(active).absolute()] if active else []) + conventional
    found = []
    for path in paths:
        if path not in found and path.exists():
            found.append(path)
    return found or [home / ".local/bin/tersh"]


def inspect(home, binaries):
    config = home / ".config/tersh/servers.json"
    marker = config.with_name(".fleet-inventory.sha256")
    managed = None
    if marker.is_symlink():
        raise ValueError("A managed-inventory digest cannot be a symlink")
    if marker.exists():
        info = marker.stat()
        if not stat.S_ISREG(info.st_mode) or info.st_uid != os.geteuid() or info.st_mode & 0o077 or info.st_size > 80:
            raise ValueError("The managed-inventory digest must be a private, owner-controlled regular file")
        text = marker.read_bytes().strip()
        if re.fullmatch(b"[0-9a-f]{64}", text):
            managed = text.decode("ascii")
    return {"os": platform.system(), "arch": platform.machine(),
            "binaries": {str(p): file_sha(p) for p in binaries},
            "config": file_sha(config), "config_link": config.is_symlink(),
            "managed_config": managed, "marker_file": file_sha(marker)}


def plan(state, request, config_path):
    changes = []
    for path, previous in state["binaries"].items():
        if previous != request["binary_sha256"]:
            if Path(path).is_symlink():
                raise ValueError("Managed binary symlink needs an explicit install location: {}".format(path))
            changes.append((Path(path), "binary", previous))
    incoming = request.get("inventory")
    if incoming is not None:
        data = base64.b64decode(incoming, validate=True)
        if len(data) > 65536:
            raise ValueError("Invalid inventory")
        incoming_doc = json.loads(data.decode("utf-8"))
        if not isinstance(incoming_doc, dict):
            raise ValueError("Invalid inventory")
        previous = state["config"]
        wanted = digest(data)
        if previous != wanted:
            if state["config_link"]:
                raise ValueError("Will not replace a private inventory symlink")
            if previous is not None and request.get("reconcile") != previous:
                if previous != state["managed_config"]:
                    raise ValueError("Different inventory already exists. Reconcile its exact SHA-256: " + previous)
                # Multiple SSH hosts can share one home directory. A clean managed
                # snapshot authorizes refreshing that receiver, not rebinding it.
                current_doc = json.loads(config_path.read_bytes().decode("utf-8"))
                if (current_doc.get("local_alias") != incoming_doc.get("local_alias")
                        or current_doc.get("observer_only", False) != incoming_doc.get("observer_only", False)):
                    raise ValueError("Different device identity. Reconcile its exact SHA-256: " + previous)
            changes.append((config_path, "inventory", previous))
        if wanted != state["managed_config"]:
            if state["config_link"]:
                raise ValueError("A symlinked authority cannot be enrolled as a managed copy")
            changes.append((config_path.with_name(".fleet-inventory.sha256"), "marker", state["marker_file"]))
    return changes


def stage_file(path, data, mode):
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    fd, name = tempfile.mkstemp(prefix=".tersh-stage-", dir=str(path.parent))
    try:
        with os.fdopen(fd, "wb") as out:
            os.fchmod(out.fileno(), mode)
            out.write(data)
            out.flush()
            os.fsync(out.fileno())
        return Path(name)
    except BaseException:
        os.unlink(name)
        raise


def install(home, binaries, request):
    cfg = home / ".config/tersh/servers.json"
    state = inspect(home, binaries)
    changes = plan(state, request, cfg)
    report = {"changed": [str(path) for path, _, _ in changes], "backups": {}}
    if not changes or request["operation"] == "plan":
        return report

    binary = base64.b64decode(request["binary"], validate=True)
    if digest(binary) != request["binary_sha256"]:
        raise ValueError("Binary SHA-256 mismatch")
    if not (request.get("inspected") == state):
        raise ValueError("Device or files changed since preflight")
    private = home / ".config/tersh"
    private.mkdir(parents=True, exist_ok=True, mode=0o700)
    lock_fd = os.open(str(private / ".fleet.lock"), os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
    with os.fdopen(lock_fd, "r+b") as lock:
        if not stat.S_ISREG(os.fstat(lock.fileno()).st_mode):
            raise ValueError("Fleet lock must be regular")
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        if inspect(home, binaries) != state:
            raise ValueError("Target changed while waiting for fleet lock")
        staged = {}
        backups = {}
        published = []
        try:
            for path, kind, previous in changes:
                is_bin = kind == "binary"
                data = binary if is_bin else base64.b64decode(request["inventory"], validate=True)
                if kind == "marker":
                    data = (digest(data) + "\n").encode("ascii")
                staged[path] = stage_file(path, data, 0o755 if is_bin else 0o600)
                if is_bin:
                    version = subprocess.check_output([str(staged[path]), "--version"], timeout=10).decode().strip()
                    if version != "tersh " + request["version"]:
                        raise ValueError("Staged executable has unexpected version: " + version)
                if previous is not None:
                    fd, name = tempfile.mkstemp(prefix="." + path.name + ".pre-fleet-", dir=str(path.parent))
                    with os.fdopen(fd, "wb") as out, path.open("rb") as src:
                        if is_bin:
                            os.fchmod(out.fileno(), 0o700)
                        shutil.copyfileobj(src, out)
                        out.flush()
                        os.fsync(out.fileno())
                    backup = Path(name)
                    if file_sha(backup) != previous:
                        raise ValueError("Source changed while backing up " + str(path))
                    backups[path] = (backup, stat.S_IMODE(path.stat().st_mode))
                    report["backups"][str(path)] = str(backup)
            if request.get("inventory") is not None:
                source = staged.get(cfg, cfg)
                executable = staged.get(binaries[0], binaries[0])
                actual = json.loads(subprocess.check_output(
                    [str(executable), "--c", "--cluster-config", str(source), "--list-hosts"], timeout=10
                ).decode("utf-8"))
                intended = json.loads(base64.b64decode(request["inventory"]).decode("utf-8"))
                entries = [intended.get("main_machine"), intended.get("jump_host")] + intended.get("servers", [])
                expected = [entry["alias"] for entry in entries if entry]
                if ([h["alias"] for h in actual["hosts"]] != expected
                        or actual.get("local_alias") != intended.get("local_alias")):
                    raise ValueError("The staged inventory did not reproduce the intended device list and local binding")
            for path, _, previous in changes:
                if path.is_symlink() or file_sha(path) != previous:
                    raise ValueError("Target changed before activation: " + str(path))
                os.replace(str(staged[path]), str(path))
                published.append(path)
            if any(file_sha(p) != request["binary_sha256"] for p in binaries):
                raise ValueError("Installed binary checksum mismatch")
            if request.get("inventory") is not None and file_sha(cfg) != digest(base64.b64decode(request["inventory"])):
                raise ValueError("Installed inventory checksum mismatch")
        except BaseException:
            for path in reversed(published):
                if path in backups:
                    backup, mode = backups[path]
                    restore = stage_file(path, backup.read_bytes(), mode)
                    os.replace(str(restore), str(path))
                else:
                    path.unlink()
            raise
        finally:
            for file in staged.values():
                if file.exists():
                    file.unlink()
    return report


def main():
    if sys.argv[1:2] == ["--bundle"]:
        import argparse
        parser = argparse.ArgumentParser(description="Install and bind this offline Tersh package on the current device")
        parser.add_argument("--dry-run", action="store_true")
        parser.add_argument("--reconcile", metavar="SHA256", help="accept the existing inventory after reviewing this SHA-256")
        args = parser.parse_args(sys.argv[2:])
        bundle = Path(__file__).resolve().parent
        meta = json.loads((bundle / "manifest.json").read_text())
        platforms = {("Darwin", "arm64"): "macos-arm64", ("Linux", "arm64"): "linux-arm64",
                     ("Linux", "aarch64"): "linux-arm64", ("Linux", "x86_64"): "linux-x86_64"}
        if platforms.get((platform.system(), platform.machine())) != meta["platform"]:
            raise ValueError("This package targets " + meta["platform"])
        binary = (bundle / "tersh").read_bytes()
        inventory = (bundle / "servers.json").read_bytes()
        if digest(binary) != meta["binary_sha256"] or digest(inventory) != meta["inventory_sha256"]:
            raise ValueError("Offline package checksum mismatch")
        home = Path.home()
        paths = locations(home)
        request = {"operation": "plan" if args.dry_run else "apply", "inspected": inspect(home, paths),
                   "binary_sha256": meta["binary_sha256"], "binary": base64.b64encode(binary).decode(),
                   "inventory": base64.b64encode(inventory).decode(), "version": meta["version"],
                   "reconcile": args.reconcile}
        print(json.dumps(install(home, paths, request)))
        return
    request = json.load(sys.stdin)
    home = Path.home()
    binaries = locations(home)
    if request["operation"] == "inspect":
        answer = inspect(home, binaries)
    elif request["operation"] in ("plan", "apply"):
        answer = install(home, binaries, request)
    else:
        raise ValueError("Unexpected operation")
    print(json.dumps(answer))


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        sys.exit(str(exc))
