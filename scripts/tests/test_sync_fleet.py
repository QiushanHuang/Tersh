import base64
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest
from types import SimpleNamespace
from unittest import mock


SCRIPTS = Path(__file__).resolve().parents[1]


def module(name):
    spec = importlib.util.spec_from_file_location(name, SCRIPTS / (name + ".py"))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


fleet = module("sync-fleet")
remote = module("fleet_remote")
inventory = {"main_machine": {"alias": "console", "ssh_user": "owner"},
             "jump_host": {"alias": "gateway", "ssh_user": "member", "device_name": "gw.invalid"},
             "servers": [{"alias": "worker", "proxy_jump": "gateway", "campus_ip": "worker.invalid"}]}
binary = b"""#!/usr/bin/env python3
import json, sys
if sys.argv[1:] == ['--version']:
    print('tersh 1.4.0')
else:
    with open(sys.argv[sys.argv.index('--cluster-config')+1]) as f:
        inventory = json.load(f)
    devices = [inventory['main_machine'], inventory['jump_host']] + inventory['servers']
    print(json.dumps({'hosts': devices, 'local_alias': inventory.get('local_alias')}))
"""


class FleetTests(unittest.TestCase):
    def setUp(self):
        root = tempfile.TemporaryDirectory()
        self.addCleanup(root.cleanup)
        self.root = Path(root.name)
        self.home = self.root / "home"
        self.bin = self.home / ".local/bin/tersh"
        self.cfg = self.home / ".config/tersh/servers.json"
        self.release = self.root / "release"
        self.release.mkdir()
        self.archive = self.release / "tersh-v1.4.0-linux-x86_64.tar.gz"
        with tarfile.open(self.archive, "w:gz") as t:
            member = tarfile.TarInfo("tersh")
            member.size = len(binary)
            t.addfile(member, io.BytesIO(binary))
        self.archive.with_suffix(".gz.sha256").write_text(
            hashlib.sha256(self.archive.read_bytes()).hexdigest() + "  " + self.archive.name + "\n"
        )

    def request(self, operation="apply"):
        return {"operation": operation, "binary_sha256": remote.digest(binary), "version": "1.4.0",
                "binary": base64.b64encode(binary).decode(),
                "inventory": base64.b64encode(fleet.bound(inventory, "worker")).decode(),
                "inspected": remote.inspect(self.home, [self.bin])}

    def create_old(self):
        self.bin.parent.mkdir(parents=True)
        self.bin.write_bytes(b"original binary")
        self.bin.chmod(0o755)
        self.cfg.parent.mkdir(parents=True)
        self.cfg.write_bytes(b'{"servers": []}')
        (self.cfg.parent / "keymap.json").write_text("keep")
        (self.cfg.parent / "places.json").write_text("keep pins")

    def test_dry_run_has_no_writes_and_uses_existing_proxy(self):
        result = remote.install(self.home, [self.bin], self.request("plan"))
        self.assertEqual(result["changed"], [str(self.bin), str(self.cfg), str(self.cfg.with_name(".fleet-inventory.sha256"))])
        self.assertFalse(self.home.exists())
        args = fleet.endpoint_command(inventory, "worker", inventory["servers"][0])
        self.assertTrue(any("ProxyCommand=ssh " in arg and "member@gw.invalid" in arg for arg in args))
        self.assertIn("StrictHostKeyChecking=yes", args)
        self.assertIn("ForwardAgent=no", args)
        self.assertEqual(fleet.endpoint_command(inventory, "console", inventory["main_machine"])[0], sys.executable)

    def test_apply_backs_up_both_files_and_is_idempotent(self):
        self.create_old()
        req = self.request()
        with self.assertRaisesRegex(ValueError, "Different inventory"):
            remote.install(self.home, [self.bin], req)
        self.assertEqual(self.bin.read_bytes(), b"original binary")
        req["reconcile"] = remote.file_sha(self.cfg)
        done = remote.install(self.home, [self.bin], req)
        self.assertEqual(Path(done["backups"][str(self.bin)]).read_bytes(), b"original binary")
        self.assertEqual(Path(done["backups"][str(self.cfg)]).read_bytes(), b'{"servers": []}')
        self.assertEqual(self.bin.read_bytes(), binary)
        self.assertEqual(json.loads(self.cfg.read_text())["local_alias"], "worker")
        self.assertEqual(self.cfg.stat().st_mode & 0o777, 0o600)
        self.assertEqual((self.cfg.parent / "keymap.json").read_text(), "keep")
        self.assertEqual((self.cfg.parent / "places.json").read_text(), "keep pins")
        self.assertEqual(remote.install(self.home, [self.bin], self.request())["changed"], [])

    def test_managed_inventory_can_update_until_a_local_edit_is_seen(self):
        remote.install(self.home, [self.bin], self.request())
        update = self.request()
        revised = json.loads(base64.b64decode(update["inventory"]).decode())
        revised["servers"][0]["role"] = "new role"
        update["inventory"] = base64.b64encode(json.dumps(revised).encode()).decode()
        remote.install(self.home, [self.bin], update)
        self.assertEqual(json.loads(self.cfg.read_text()), revised)
        self.cfg.write_text(self.cfg.read_text() + " ")
        with self.assertRaisesRegex(ValueError, "Different inventory"):
            remote.install(self.home, [self.bin], self.request())

    def test_partial_activation_is_rolled_back(self):
        self.create_old()
        req = self.request()
        req["reconcile"] = remote.file_sha(self.cfg)
        replace = os.replace

        def fail_second(source, dest):
            if dest == str(self.cfg):
                raise OSError("simulated config rename failure")
            return replace(source, dest)

        with mock.patch.object(remote.os, "replace", side_effect=fail_second):
            with self.assertRaisesRegex(OSError, "simulated"):
                remote.install(self.home, [self.bin], req)
        self.assertEqual(self.bin.read_bytes(), b"original binary")
        self.assertEqual(self.cfg.read_bytes(), b'{"servers": []}')

    def test_refuses_unexpected_version_changed_source_and_symlink(self):
        self.create_old()
        req = self.request()
        req["reconcile"] = remote.file_sha(self.cfg)
        req["version"] = "1.4.2"
        with self.assertRaisesRegex(ValueError, "unexpected version"):
            remote.install(self.home, [self.bin], req)
        self.assertEqual(self.bin.read_bytes(), b"original binary")
        self.bin.write_bytes(b"changed while connecting")
        with self.assertRaisesRegex(ValueError, "since preflight"):
            remote.install(self.home, [self.bin], req)
        owner = self.root / "master.json"
        self.cfg.rename(owner)
        self.cfg.symlink_to(owner)
        with self.assertRaisesRegex(ValueError, "symlink"):
            remote.install(self.home, [self.bin], self.request())
        local = self.request()
        local["inventory"] = None
        remote.install(self.home, [self.bin], local)
        self.assertTrue(self.cfg.is_symlink())
        self.assertEqual(owner.read_bytes(), b'{"servers": []}')

    def test_offline_package_preserves_ssh_roles_and_rejects_tampering(self):
        config = SimpleNamespace(offline_dir=self.root / "offline", release_dir=self.release,
                                 version="1.4.0", platform="linux-x86_64", host=[], observer_only="Studio")
        manifest = fleet.offline_package(config, inventory)
        data = json.loads((config.offline_dir / "servers.json").read_text())
        self.assertTrue(data["observer_only"])
        self.assertNotIn("local_alias", data)
        self.assertEqual(data["main_machine"]["ssh_user"], "owner")
        self.assertEqual((config.offline_dir / "servers.json").stat().st_mode & 0o777, 0o600)
        self.assertEqual((config.offline_dir / "install.sh").stat().st_mode & 0o777, 0o700)
        self.assertEqual(manifest["binary_sha256"], remote.digest(binary))
        fleet.artifact.cache_clear()
        self.archive.write_bytes(self.archive.read_bytes() + b"corruption")
        with self.assertRaisesRegex(ValueError, "checksum mismatch"):
            fleet.artifact(self.release, "1.4.0", config.platform)

    def test_host_must_exist_and_ssh_target_is_data(self):
        with self.assertRaisesRegex(ValueError, "unknown"):
            fleet.bound(inventory, "not-registered")
        for value in ["-oProxyCommand=x", "a;other", "machine name", "x%h"]:
            with self.assertRaisesRegex(ValueError, "Invalid SSH"):
                fleet.ssh_target({"alias": "demo", "campus_ip": value})


if __name__ == "__main__":
    unittest.main()
