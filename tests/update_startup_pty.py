"""Drive a disposable Tersh instance with a real controlling terminal; no SSH."""
import errno
import fcntl
import importlib.util
import os
from pathlib import Path
import select
import signal
import struct
import sys
import termios
import time

binary, mode, answer = sys.argv[1:]
decoder = Path(__file__).resolve().parents[1] / "scripts" / "verify-product-pty.py"
spec = importlib.util.spec_from_file_location("tersh_pty", decoder)
pty_qa = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pty_qa)
screen = pty_qa.Screen(120, 30)
root = Path(os.environ["RELEASE_MOCK"])
released = False
requests = root / "requests"
args = [binary, str(root / "contents")]
if mode == "cluster":
    inventory = root / "local.json"
    inventory.write_text('{"main_machine":{"alias":"local-update-fixture"}}')
    args = [binary, "--c", "--cluster-config", str(inventory)]

pid, master = os.forkpty()
if pid == 0:
    os.execv(binary, args)

try:
    fcntl.ioctl(master, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 120, 0, 0))
    os.kill(pid, signal.SIGWINCH)
    output = bytearray()

    def expect(needle):
        deadline = time.monotonic() + 12
        while needle not in output and needle.decode() not in screen.text:
            remaining = deadline - time.monotonic()
            if remaining <= 0 or not select.select([master], [], [], remaining)[0]:
                raise AssertionError(f"timed out waiting for {needle!r}: {bytes(output[-280:])!r}")
            chunk = os.read(master, 65536)
            if not chunk:
                raise AssertionError("Tersh exited before displaying update controls")
            output.extend(chunk)
            screen.feed(chunk)

    expect(b"Update available")
    assert len(requests.read_text().splitlines()) == 1
    os.write(master, b"q")
    expect(b"[y/N]")
    assert len(requests.read_text().splitlines()) == 1
    os.write(master, (answer + "\n").encode())
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        child, status = os.waitpid(pid, os.WNOHANG)
        if child:
            released = True
            assert os.waitstatus_to_exitcode(status) == 0, bytes(output[-250:])
            break
        if select.select([master], [], [], 0.05)[0]:
            try:
                output.extend(os.read(master, 65536))
            except OSError as ex:
                if ex.errno != errno.EIO:
                    raise
    assert released, "update confirmation did not finish"
    assert len(requests.read_text().splitlines()) == (3 if answer == "y" else 1)
finally:
    if not released:
        os.kill(pid, signal.SIGKILL)
        os.waitpid(pid, 0)
    os.close(master)
