"""Package a built executable without rebuilding or embedding local metadata."""
import hashlib
import io
from pathlib import Path
import re
import sys
import tarfile

binary = Path(sys.argv[1])
platform = sys.argv[2]
assert platform in {"macos-arm64", "linux-x86_64", "linux-arm64"}
version = re.search(r'^version = "([^"]+)"', Path("Cargo.toml").read_text(), re.M)[1]
assert binary.is_file()
output = Path("dist")
output.mkdir(exist_ok=True)
archive = output / f"tersh-v{version}-{platform}.tar.gz"
install = b'''Install the executable and apply applicable local WSL lifecycle patches:
  ./install.sh
  tersh --version

The installer reuses the current tersh directory or a writable user directory
on PATH. Set TERSH_INSTALL_DIR to choose a directory explicitly.
Run it on each SSH client and WSL endpoint that uses the Tersh WSL route.
Only recognized WSL routes and an existing dedicated Tersh SSH endpoint are
patched. A failed patch is reported as an incomplete installation.

If copying the executable manually, run this afterward:
  tersh --install-patches
Cargo installs do not run a post-install hook; after cargo install, also run:
  tersh --install-patches
To inspect applicable lifecycle patches without writing:
  tersh --check-install-patches

Documentation: https://github.com/QiushanHuang/Tersh
macOS builds are not Developer ID signed or notarized.
'''
with tarfile.open(archive, "w:gz") as bundle:
    for path, name, mode in [
        (binary, "tersh", 0o755),
        (Path("scripts/install-binary.sh"), "install.sh", 0o755),
        (Path("LICENSE"), "LICENSE", 0o644),
    ]:
        data = path.read_bytes()
        info = tarfile.TarInfo(name)
        info.mode = mode
        info.size = len(data)
        bundle.addfile(info, io.BytesIO(data))
    info = tarfile.TarInfo("INSTALL.txt")
    info.mode = 0o644
    info.size = len(install)
    bundle.addfile(info, io.BytesIO(install))
digest = hashlib.sha256(archive.read_bytes()).hexdigest()
archive.with_suffix(archive.suffix + ".sha256").write_text(f"{digest}  {archive.name}\n")
print(archive)
