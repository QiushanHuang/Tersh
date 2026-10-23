"""Exercise installation and packaging with an inert, locally built fake binary."""

import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
INSTALL = ROOT / "scripts" / "install-binary.sh"


class InstallPackageTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="tersh install tests ")
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.log = self.directory / "patch invocation.txt"
        self.binary = self.directory / "built executable" / "tersh"
        self.binary.parent.mkdir()
        self.binary.write_text(
            '#!/bin/sh\n'
            'printf "%s\\n" "$0" "$@" >> "$TERSH_TEST_PATCH_LOG"\n'
            'exit "${TERSH_TEST_PATCH_EXIT:-0}"\n'
        )
        self.binary.chmod(0o755)
        self.destination = self.directory / "installed tools with spaces"
        self.env = os.environ.copy()
        self.env.update(
            HOME=str(self.directory / "home"),
            PATH="/usr/bin:/bin",
            TERSH_INSTALL_DIR=str(self.destination),
            TERSH_TEST_PATCH_LOG=str(self.log),
        )

    def run_script(self, script, *args, cwd=None):
        return subprocess.run(
            ["/bin/sh", str(script), *map(str, args)],
            env=self.env,
            cwd=cwd or self.directory,
            text=True,
            capture_output=True,
            check=False,
        )

    def assert_patch_invoked_from_installation(self):
        installed = self.destination / "tersh"
        self.assertTrue(self.log.is_file(), "Installed binary did not receive --install-patches")
        self.assertEqual(self.log.read_text().splitlines(), [str(installed), "--install-patches"])
        self.assertEqual(installed.read_bytes(), self.binary.read_bytes())
        self.assertEqual(installed.stat().st_mode & 0o777, 0o755)
        self.assertFalse(list(self.destination.glob(".tersh.*")))

    def test_installs_in_explicit_directory_with_spaces_and_patches_installed_binary(self):
        result = self.run_script(INSTALL, self.binary)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assert_patch_invoked_from_installation()

    def test_patch_failure_propagates_and_explains_already_installed_binary(self):
        self.env["TERSH_TEST_PATCH_EXIT"] = "23"
        result = self.run_script(INSTALL, self.binary)
        self.assertEqual(result.returncode, 23, result.stdout + result.stderr)
        self.assert_patch_invoked_from_installation()
        self.assertIn("binary", result.stderr.lower())
        self.assertIn("installed", result.stderr.lower())
        self.assertIn("incomplete", result.stderr.lower())
        self.assertIn("--install-patches", result.stderr)
        self.assertNotIn("Installation complete", result.stdout)

    def test_packaged_installer_defaults_to_sibling_executable(self):
        self.assertTrue(INSTALL.is_file(), "Packaged binary installer is missing")
        archive_root = self.directory / "unpacked archive"
        archive_root.mkdir()
        script = archive_root / "install.sh"
        shutil.copyfile(INSTALL, script)
        shutil.copyfile(self.binary, archive_root / "tersh")
        result = self.run_script(script)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assert_patch_invoked_from_installation()

    def test_existing_tersh_path_is_reused_without_an_override(self):
        self.destination.mkdir()
        existing = self.destination / "tersh"
        existing.write_text("#!/bin/sh\nexit 0\n")
        existing.chmod(0o755)
        self.env.pop("TERSH_INSTALL_DIR")
        self.env["PATH"] = str(self.destination) + ":/usr/bin:/bin"
        result = self.run_script(INSTALL, self.binary)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assert_patch_invoked_from_installation()

    def test_source_installer_builds_and_delegates_to_the_same_patch_path(self):
        project = self.directory / "source checkout"
        scripts = project / "scripts"
        scripts.mkdir(parents=True)
        shutil.copyfile(ROOT / "scripts" / "install.sh", scripts / "install.sh")
        if INSTALL.exists():
            shutil.copyfile(INSTALL, scripts / "install-binary.sh")
        fake_tools = self.directory / "fake build tools"
        fake_tools.mkdir()
        cargo = fake_tools / "cargo"
        cargo.write_text(
            '#!/bin/sh\n'
            '[ "$*" = "build --locked --release --bin tersh" ] || exit 44\n'
            'mkdir -p target/release\n'
            'cp "$TERSH_TEST_BUILT_SOURCE" target/release/tersh\n'
        )
        cargo.chmod(0o755)
        self.env["TERSH_TEST_BUILT_SOURCE"] = str(self.binary)
        self.env["PATH"] = str(fake_tools) + ":/usr/bin:/bin"
        result = self.run_script(scripts / "install.sh")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assert_patch_invoked_from_installation()

    def test_archive_contains_executable_installer_license_guidance_and_valid_checksum(self):
        project = self.directory / "package workspace"
        scripts = project / "scripts"
        scripts.mkdir(parents=True)
        (project / "Cargo.toml").write_text('[package]\nname = "tersh"\nversion = "1.3.0-dev.1"\n')
        license_bytes = b"Test license preserved exactly.\n"
        (project / "LICENSE").write_bytes(license_bytes)
        if INSTALL.exists():
            shutil.copyfile(INSTALL, scripts / "install-binary.sh")
        result = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "package-release.py"), str(self.binary), "linux-x86_64"],
            cwd=project,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        archive = project / result.stdout.strip()
        self.assertTrue(archive.is_file())
        with tarfile.open(archive) as bundle:
            self.assertEqual(set(bundle.getnames()), {"tersh", "install.sh", "LICENSE", "INSTALL.txt"})
            self.assertEqual(bundle.getmember("install.sh").mode & 0o777, 0o755)
            self.assertEqual(bundle.getmember("tersh").mode & 0o777, 0o755)
            self.assertEqual(bundle.extractfile("tersh").read(), self.binary.read_bytes())
            self.assertEqual(bundle.extractfile("install.sh").read(), INSTALL.read_bytes())
            self.assertEqual(bundle.extractfile("LICENSE").read(), license_bytes)
            guide = bundle.extractfile("INSTALL.txt").read().decode()
            self.assertIn("./install.sh", guide)
            self.assertIn("tersh --install-patches", guide)
            self.assertIn("Cargo", guide)
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        self.assertEqual(archive.with_suffix(".gz.sha256").read_text(), f"{digest}  {archive.name}\n")


if __name__ == "__main__":
    unittest.main()
