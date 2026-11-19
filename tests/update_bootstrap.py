"""Exercise the release installer with a fixture archive and no network."""
import hashlib
import os
from pathlib import Path
import platform
import subprocess
import tarfile
import tempfile

repo = Path(__file__).resolve().parents[1]
target = {("Darwin", "arm64"): "macos-arm64", ("Linux", "x86_64"): "linux-x86_64",
          ("Linux", "aarch64"): "linux-arm64", ("Linux", "arm64"): "linux-arm64"}[
              platform.system(), platform.machine()]
with tempfile.TemporaryDirectory() as temp:
    root = Path(temp)
    pkg = root / "tersh-v1.4.0"
    pkg.mkdir()
    binary = pkg / "tersh"
    binary.write_text('#!/bin/sh\ncase "$1" in --version) echo "tersh 1.4.0";; --help) echo "--check-update";; --update) [ "$2" = "--yes" ]; echo delegated >> "$FIXTURE/log";; --install-patches) exit 0;; *) exit 2;; esac\n')
    binary.chmod(0o755)
    archive = root / f"tersh-v1.4.0-{target}.tar.gz"
    with tarfile.open(archive, "w:gz") as bundle:
        bundle.add(binary, "tersh")
        bundle.add(repo / "scripts/install-binary.sh", "install.sh")
    sidecar = root / (archive.name + ".sha256")
    sidecar.write_text(hashlib.sha256(archive.read_bytes()).hexdigest() + "  " + archive.name + "\n")
    curl = root / "curl"
    curl.write_text('''#!/bin/sh
set -eu
while [ "$#" -gt 1 ]; do
    case "$1" in --output) dest=$2; shift;; esac
    shift
done
echo "$1" >> "$FIXTURE/log"
case "$1" in
https://github.com/QiushanHuang/Tersh/releases/latest) printf '%s' "https://github.com/QiushanHuang/Tersh/releases/tag/v1.4.0";;
https://github.com/QiushanHuang/Tersh/releases/download/v1.4.0/*) cp "$FIXTURE/${1##*/}" "$dest";;
*) exit 1;;
esac
''')
    curl.chmod(0o755)
    destination = root / "install target"
    env = dict(os.environ, PATH=f"{root}:/usr/bin:/bin:/usr/sbin:/sbin", HOME=str(root),
               TERSH_INSTALL_DIR=str(destination), FIXTURE=str(root))
    def run(ok=True):
        result = subprocess.run(["sh", str(repo / "scripts/install-release.sh")], env=env,
                                capture_output=True, text=True, timeout=10)
        assert (result.returncode == 0) == ok, result.stderr
    run()
    assert (destination / "tersh").read_bytes() == binary.read_bytes()
    run()
    assert (root / "log").read_text().splitlines()[-1] == "delegated"
    assert len((root / "log").read_text().splitlines()) == 4
    # A legacy installation with a newer major version is kept.
    (destination / "tersh").write_text('#!/bin/sh\n[ "$1" != --version ] || echo "tersh 20.0.0"\n')
    run()
    assert "20.0.0" in (destination / "tersh").read_text()
    # A bad checksum must leave the existing legacy device install intact.
    (destination / "tersh").write_text('#!/bin/sh\n[ "$1" != --version ] || echo "tersh 1.3.0"\n')
    archive.write_bytes(b"corrupt")
    run(False)
    assert "1.3.0" in (destination / "tersh").read_text()
