"""Local-only POSIX terminal smoke tests; all mutations use fresh temp files.

python3 scripts/verify-workflow-pty.py /absolute/path/to/tersh output.json
"""
import fcntl
import codecs
import json
import os
from pathlib import Path
import pty
import re
import select
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time

BINARY = str(Path(sys.argv[1]).resolve())


class Screen:
    """Decode the ASCII cursor-addressed output of the SSH preset for assertions.

    This is a small QA decoder, not a general-purpose terminal emulator.
    """
    def __init__(self, width=100, height=30):
        self.decoder = codecs.getincrementaldecoder("utf-8")("replace")
        self.pending = ""
        self.resize(width, height)

    def resize(self, width, height):
        self.width, self.height = width, height
        self.cells = [[" "] * width for _ in range(height)]
        self.x = self.y = 0

    @property
    def text(self):
        return "\n".join("".join(row) for row in self.cells)

    def feed(self, data):
        text = self.pending + self.decoder.decode(data)
        self.pending = ""
        index = 0
        while index < len(text):
            ch = text[index]
            if ch == "\x1b":
                match = re.match(r"\x1b\[([0-9;?]*)([@-~])", text[index:])
                if match is None:
                    self.pending = text[index:]
                    break
                params, code = match.groups()
                nums = [int(p or "0") for p in params.lstrip("?").split(";")]
                amount = nums[0] or 1
                if code in "Hf":
                    self.y = max(0, min(self.height-1, (nums[0] or 1)-1))
                    self.x = max(0, min(self.width-1, ((nums[1] if len(nums)>1 else 1) or 1)-1))
                elif code == "C": self.x = min(self.width-1, self.x+amount)
                elif code == "D": self.x = max(0, self.x-amount)
                elif code == "A": self.y = max(0, self.y-amount)
                elif code == "B": self.y = min(self.height-1, self.y+amount)
                elif code == "G": self.x = min(self.width-1, amount-1)
                elif code == "J" and nums[0] == 2:
                    self.cells = [[" "] * self.width for _ in range(self.height)]
                elif code == "K":
                    start = 0 if nums[0] in (1, 2) else self.x
                    end = self.width if nums[0] in (0, 2) else self.x+1
                    self.cells[self.y][start:end] = [" "]*(end-start)
                index += len(match.group(0))
                continue
            if ch == "\r": self.x = 0
            elif ch == "\n": self.y = min(self.height-1, self.y+1)
            elif ch == "\b": self.x = max(0, self.x-1)
            elif ord(ch) >= 32:
                if self.x >= self.width:
                    self.x = 0
                    self.y = min(self.height-1, self.y+1)
                self.cells[self.y][self.x] = ch
                self.x += 1
            index += 1


class Session:
    def __init__(self, *args):
        self.screen = Screen()
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
        self.original = termios.tcgetattr(self.slave)
        env = os.environ.copy()
        env.pop("NO_COLOR", None)
        env.pop("TERSH_KEYMAP", None)
        env["TERSH_CLIPBOARD"] = "off"
        # Do not inherit a real user's config into the fixture.
        env["XDG_CONFIG_HOME"] = str(Path(tempdir) / "empty-config")
        self.process = subprocess.Popen([BINARY, "--ui-profile", "ssh", *map(str, args)],
                                        stdin=self.slave, stdout=self.slave, stderr=self.slave,
                                        env=env, start_new_session=True)
        self.read(.2)

    def read(self, seconds):
        data = bytearray()
        until = time.monotonic() + seconds
        while time.monotonic() < until:
            ready, _, _ = select.select([self.master], [], [], max(0, until-time.monotonic()))
            if ready:
                try:
                    data.extend(os.read(self.master, 65536))
                except OSError:
                    break
        self.screen.feed(bytes(data))
        return bytes(data)

    def send(self, data, settle=.1):
        os.write(self.master, data)
        return self.read(settle) if settle else b""

    def close(self):
        try:
            if self.process.poll() is None:
                self.send(b"\x03", .3)
            self.process.wait(timeout=5)
            assert self.process.returncode == 0
            assert termios.tcgetattr(self.slave) == self.original
        finally:
            if self.process.poll() is None:
                self.process.kill()
                self.process.wait()
            os.close(self.master)
            os.close(self.slave)


def until(predicate, session, timeout=5):
    end = time.monotonic() + timeout
    while not predicate():
        assert time.monotonic() < end, "timed out waiting for filesystem state"
        assert session.process.poll() is None, "TUI exited unexpectedly"
        session.read(.005)


results = {}
with tempfile.TemporaryDirectory(prefix="tersh-workflow-pty-") as tempdir:
    root = Path(tempdir)
    source_dir, target_dir = root/"source", root/"target"
    source_dir.mkdir()
    target_dir.mkdir()
    source = source_dir/"large.bin"
    with source.open("wb") as stream:
        stream.truncate(1024*1024*1024)
    target = target_dir/source.name
    session = Session(source_dir)
    try:
        session.send(b"c")
        session.send(str(target_dir).encode()+b"\r", settle=0)
        until(lambda: target.exists() and target.stat().st_size >= 128*1024, session)
        observed = target.stat().st_size
        session.send(b"\x18", .5)  # Ctrl+x: cancel job, keep TUI alive.
        until(lambda: not target.exists(), session)
        session.send(b"J", .2)
        assert "File jobs" in session.screen.text
        assert source.stat().st_size == 1024*1024*1024
        session.send(b"q")
        # Repeat, then Ctrl+c must cancel, clean up and restore the terminal.
        session.send(b"c")
        session.send(str(target_dir).encode()+b"\r", settle=0)
        until(lambda: target.exists() and target.stat().st_size >= 128*1024, session)
        session.send(b"\x03", .5)
        session.process.wait(timeout=5)
        assert not target.exists()
        results["cancel_copy"] = {"bytes_observed_before_cancel": observed,
                                  "partial_target_removed": True, "source_preserved": True,
                                  "jobs_view_opened": True, "exit_waited_for_cleanup": True}
    finally:
        session.close()

    recovery = root/"recovery"
    recovery.mkdir()
    original = recovery/"keep.txt"
    original.write_text("restore across restart")
    session = Session(recovery)
    try:
        session.send(b"d")
        session.send(b"trash\r", .3)
        until(lambda: not original.exists(), session)
        session.send(b"q", .3)
    finally:
        session.close()
    session = Session(recovery)
    try:
        session.send(b"u")
        assert "Trash recovery" in session.screen.text
        session.send(b"\r")
        assert "Restore confirmation" in session.screen.text
        session.send(b"\r", .3)
        until(original.exists, session)
        assert original.read_text() == "restore across restart"
        results["persistent_restore"] = True
    finally:
        session.close()

    keys = root/"keymap.json"
    keys.write_text(json.dumps({"files": {"open_jobs": ["F5"], "open_trash": ["F6"]}}))
    session = Session("--keymap", keys, recovery)
    try:
        session.send(b"\x1b[15~")
        assert "File jobs" in session.screen.text
        session.send(b"q")
        session.send(b"\x1b[17~")
        assert "Trash recovery" in session.screen.text
        results["custom_function_keys"] = True
    finally:
        session.close()

    inventory = root/"local.json"
    inventory.write_text(json.dumps({"main_machine": {"alias": "local-qa", "tailscale_ip": "127.0.0.1", "role": "Local QA"}}))
    session = Session("--c", "--cluster-config", inventory)
    try:
        session.read(6.5)
        session.send(b"/")
        session.send(b"unmatched\r", .2)
        assert "No matching hosts" in session.screen.text
        session.send(b"\x7f")  # Backspace clears committed host filter.
        session.send(b"v", .2)
        assert "alias asc" in session.screen.text
        session.screen.resize(40, 18)
        fcntl.ioctl(session.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 18, 40, 0, 0))
        os.kill(session.process.pid, signal.SIGWINCH)
        assert session.read(.2)
        rss = int(subprocess.check_output(["/bin/ps", "-o", "rss=", "-p", str(session.process.pid)]).strip())
        idle = session.read(2)
        assert not idle
        results["cluster_local"] = {"filter_clear_sort": True, "resize_40_columns": True,
                                    "idle_seconds": 2, "idle_output_bytes": len(idle), "rss_kib": rss,
                                    "remote_hosts_contacted": False}
    finally:
        session.close()

receipt = {"binary_bytes": Path(BINARY).stat().st_size, "checks": results,
           "terminal_modes_restored": True,
           "limits": "Local macOS PTY smoke checks; RSS is a point sample. No remote or Linux runtime tested."}
Path(sys.argv[2]).write_text(json.dumps(receipt, indent=2))
print(json.dumps(receipt, indent=2))
