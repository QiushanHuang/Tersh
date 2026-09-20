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
install = b'''Install the executable in a user directory on PATH:
  mkdir -p "$HOME/.local/bin"
  install -m 755 tersh "$HOME/.local/bin/tersh"
  tersh --version

Documentation: https://github.com/QiushanHuang/Tersh
macOS builds are not Developer ID signed or notarized.
'''
with tarfile.open(archive, "w:gz") as bundle:
    for path, name, mode in [(binary, "tersh", 0o755), (Path("LICENSE"), "LICENSE", 0o644)]:
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
