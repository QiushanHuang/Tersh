"""Offline POSIX PTY product acceptance, using only disposable local fixtures.

Usage: python3 scripts/verify-product-pty.py target/release/tersh output.json
The receipt includes reconstructed complete terminal screens, not matches against
raw escape streams. No configured user state or remote inventory is loaded.
"""
import argparse
import ast
import codecs
import fcntl
import hashlib
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


def screen_class():
    """Reuse only the decoder definition; never import the old script's runner."""
    source = Path(__file__).with_name("verify-workflow-pty.py")
    tree = ast.parse(source.read_text(), filename=str(source))
    definition = next(node for node in tree.body if isinstance(node, ast.ClassDef) and node.name == "Screen")
    namespace = {"codecs": codecs, "re": re}
    exec(compile(ast.Module(body=[definition], type_ignores=[]), str(source), "exec"), namespace)
    return namespace["Screen"]


Screen = screen_class()


class Session:
    def __init__(self, runner, *args, width=120, height=30):
        self.runner = runner
        self.screen = Screen(width, height)
        self.master, self.slave = pty.openpty()
        self.closed = False
        self.resize_io(width, height)
        self.original = termios.tcgetattr(self.slave)
        env = os.environ.copy()
        for key in list(env):
            if key.startswith("TERSH_") or key == "NO_COLOR":
                env.pop(key, None)
        env.update({"TERM": "xterm-256color", "TERSH_CLIPBOARD": "off",
                    "XDG_CONFIG_HOME": str(runner.root / "empty-config"),
                    "XDG_STATE_HOME": str(runner.root / "state")})
        self.process = subprocess.Popen(
            [runner.binary, "--ui-profile", "ssh", *map(str, args)],
            stdin=self.slave, stdout=self.slave, stderr=self.slave,
            env=env, start_new_session=True,
        )
        runner.sessions.append(self)

    def __enter__(self):
        return self

    def __exit__(self, kind, error, traceback):
        self.close()

    def resize_io(self, width, height):
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", height, width, 0, 0))

    def resize(self, width, height=30):
        self.screen.resize(width, height)
        self.resize_io(width, height)
        os.kill(self.process.pid, signal.SIGWINCH)

    def read(self, seconds=.02):
        until = time.monotonic() + seconds
        data = bytearray()
        while time.monotonic() < until:
            ready, _, _ = select.select([self.master], [], [], max(0, until - time.monotonic()))
            if not ready:
                break
            try:
                chunk = os.read(self.master, 65536)
            except OSError:
                break
            if not chunk:
                break
            data.extend(chunk)
            self.screen.feed(chunk)
        return bytes(data)

    def wait(self, label, predicate, timeout=8):
        self.runner.current = label
        until = time.monotonic() + timeout
        while True:
            if predicate():
                return
            if time.monotonic() >= until or self.process.poll() is not None:
                raise AssertionError(f"{label}: terminal state was not reached\n{self.screen.text}")
            self.read(.02)

    def expect(self, label, *parts, absent=(), timeout=8):
        self.wait(label, lambda: all(part in self.screen.text for part in parts)
                  and all(part not in self.screen.text for part in absent), timeout)

    def send(self, data):
        if isinstance(data, str):
            data = data.encode()
        os.write(self.master, data)

    def capture(self, label):
        self.runner.screens[label] = self.screen.text

    def quit(self):
        self.send(b"q")
        self.runner.current = "quit restores terminal"
        end = time.monotonic() + 5
        while self.process.poll() is None and time.monotonic() < end:
            self.read(.02)
        assert self.process.poll() == 0, f"q did not exit the normal screen\n{self.screen.text}"
        self.close()

    def close(self):
        if self.closed:
            return
        try:
            if self.process.poll() is None:
                self.send(b"\x03")
            # Keep consuming the PTY while the child flushes its final frame and
            # terminal restoration. Waiting only on PID can fill the PTY buffer.
            deadline = time.monotonic() + 5
            while self.process.poll() is None and time.monotonic() < deadline:
                self.read(.02)
            if self.process.poll() is None:
                self.process.kill()
                self.process.wait()
            restored = termios.tcgetattr(self.slave) == self.original
            self.runner.restorations.append({"exit_code": self.process.returncode, "restored": restored})
            assert self.process.returncode == 0 and restored, "TUI exit or terminal restoration failed"
        finally:
            os.close(self.master)
            os.close(self.slave)
            self.closed = True


class Runner:
    def __init__(self, binary, root):
        self.binary, self.root = str(binary.resolve()), root
        self.checks, self.screens, self.restorations, self.sessions = {}, {}, [], []
        self.current = "initialization"

    def state(self):
        path = self.root / "state/tersh/places.json"
        return json.loads(path.read_text()) if path.exists() else []

    def places(self):
        base = self.root / "places-case"
        child = base / "bookmark-target"
        child.mkdir(parents=True)
        (child / "place-marker.txt").write_text("place marker")
        with Session(self, base) as session:
            session.expect("places initial directory", "bookmark-target", absent=("Loading directory",))
            session.send(b"\r")
            session.expect("entered bookmark directory", "place-marker.txt", absent=("Loading directory",))
            session.send(b"B")
            session.wait("pin persisted from B", lambda: any(p["path"] == str(child) and p["pinned"] for p in self.state()))
            session.send(b"b")
            session.expect("places b opens keyboard-searchable list", "Places", "Find:", "bookmark-target")
            session.capture("places_pinned")
            session.send(b"\x1b")
            session.expect("places escape returns to files", "place-marker.txt", absent=("Find:",))
            session.quit()
        with Session(self, base) as session:
            session.expect("restart initial directory", "bookmark-target", absent=("Loading directory",))
            session.send(b"bbookmark-target")
            session.expect("persistent place searchable after restart", "Find: bookmark-target", "Places")
            session.send(b"\r")
            session.expect("persistent place enter navigates", "place-marker.txt", absent=("Find:", "Loading directory"))
            session.capture("places_restart_navigation")
            session.quit()
        renamed = base / "renamed-fixture"
        child.rename(renamed)
        with Session(self, base) as session:
            session.expect("missing place fixture initial directory", "renamed-fixture", absent=("Loading directory",))
            session.send(b"bbookmark-target")
            session.expect("missing place filter", "Find: bookmark-target")
            session.send(b"\r")
            session.expect("missing place retains query and error", "Places", "Find: bookmark-target", "goto failed")
            assert not child.exists()
            session.capture("places_invalid_path")
            session.send(b"\x04")
            session.wait("Ctrl+d removes only saved metadata", lambda: not any(p["path"] == str(child) for p in self.state()))
            assert (renamed / "place-marker.txt").exists()
            session.send(b"\x1b")
            session.expect("return after removing saved metadata", "renamed-fixture", absent=("Find:",))
            session.quit()
        self.checks["places"] = {"pin_key_B": True, "open_key_b": True, "cross_restart": True,
                                  "invalid_path_retains_query": True, "remove_metadata_only": True}

    def structured(self):
        path = self.root / "structured-case.json"
        path.write_text('{"nested":{"value":3},"label":"raw-sentinel"}')
        with Session(self, path) as session:
            session.expect("raw JSON preview", "Preview | raw", "raw-sentinel", absent=("Loading preview",))
            session.capture("preview_raw")
            session.send(b"f")
            session.expect("structured JSON preview", "Preview | structured", "JSON", "formatted view", absent=("Loading / verifying",))
            session.capture("preview_structured")
            session.send(b"f")
            session.expect("return to raw JSON", "Preview | raw", "raw-sentinel", absent=("formatted view",))
            session.send(b"q")
            session.expect("leave preview", "Files", absent=("Preview | raw",))
            session.quit()
        self.checks["structured_preview"] = {"raw_structured_raw_keys": True}

    def log(self):
        path = self.root / "follow-case.log"
        path.write_text("".join(f"phase0-{n:02}\n" for n in range(24)))
        with Session(self, path) as session:
            session.expect("log source initial preview", "Preview | raw", "phase0-00")
            session.send(b"L")
            session.expect("L opens bounded following log", "Log | following", "phase0-23")
            with path.open("a") as stream:
                stream.write("phase1-tail\n")
            session.expect("append appears in follow", "phase1-tail")
            session.send(b" ")
            session.expect("Space pauses log", "Log | paused")
            with path.open("a") as stream:
                stream.write("phase2-paused\n")
            # This observation window verifies absence of a forbidden update;
            # positive readiness elsewhere is always screen-state driven.
            session.read(.4)
            assert "phase2-paused" not in session.screen.text
            session.capture("log_paused")
            session.send(b"/")
            session.expect("search prompt in paused log", "Find in paused log")
            session.send(b"phase0-05\r")
            session.expect("search finds retained line and stays paused", "Log | paused", "search: phase0-05", "phase0-05", absent=("Find in paused log",))
            session.capture("log_search")
            session.send(b" ")
            session.expect("resume catches up", "Log | following", "phase2-paused")
            path.rename(self.root / "follow-case.previous.log")
            path.write_text("rotation-marker\n")
            session.expect("rotation follows replacement file", "Log | following", "log rotated", "rotation-marker")
            for width in (40, 80, 120):
                session.resize(width)
                session.expect(f"log resize {width} columns", "Log | following", "rotation-marker")
                session.capture(f"log_{width}_columns")
            session.send(b"q")
            session.expect("q from log returns to regular preview", "Preview | raw", "rotation-marker", absent=("Log |",))
            session.send(b"q")
            session.expect("leave preview after log", "Files", absent=("Preview | raw",))
            session.quit()
        self.checks["log"] = {"append": True, "pause_freezes_visible_data": True, "search_pauses": True,
                               "resume": True, "rotation": True, "resize_widths": [40, 80, 120], "q_returns": True}

    def jobs(self):
        directory = self.root / "jobs-case"
        directory.mkdir()
        source = directory / "source.txt"
        source.write_text("first-version")
        destinations = [self.root / "copy-one", self.root / "copy-two"]
        for destination in destinations:
            destination.mkdir()
        with Session(self, source) as session:
            session.expect("job source preview", "Preview | raw", "first-version")
            session.send(b"q")
            session.expect("job source files", "Files", absent=("Preview | raw",))
            session.send(b"c")
            session.expect("first copy prompt", "Copy selected/focused")
            session.send(str(destinations[0]) + "\r")
            session.wait("first copy content", lambda: (destinations[0] / source.name).exists())
            session.send(b"J")
            session.expect("first retained task", "File jobs", "RECENT TASK 1/1", "1 done")
            session.send(b"q")
            session.expect("return to source before second task", "Files", absent=("RECENT TASK",))
            session.send(b"c")
            session.expect("second copy captures source identity", "Copy selected/focused")
            source.write_text("second-version-changed-after-capture")
            session.send(str(destinations[1]) + "\r")
            session.send(b"J")
            session.expect("second task reports identity failure", "RECENT TASK 1/2", "1 failed")
            session.capture("jobs_failed_task")
            session.send(b"]")
            session.expect("select previous completed task", "RECENT TASK 2/2", "1 done")
            session.capture("jobs_previous_result")
            session.send(b"[")
            session.expect("return to latest task", "RECENT TASK 1/2", "1 failed")
            session.send(b"f")
            session.expect("filter task details", "View: failed / remaining", "FAILED")
            conflict = destinations[1] / source.name
            conflict.write_text("retain-conflict-target")
            session.send(b"r")
            session.expect("retry requires fresh conflict decision", "Destination already exists", "CONFLICT")
            assert conflict.read_text() == "retain-conflict-target"
            session.capture("jobs_retry_conflict")
            session.send(b"\x1b")
            session.expect("cancel conflict returns to files", "Files", absent=("CONFLICT",))
            session.send(b"J")
            session.expect("jobs after cancelling conflict", "RECENT TASK 1/2")
            existing = self.root / "keep-export.json"
            existing.write_text("do-not-overwrite")
            session.send(b"e")
            session.expect("export prompts for new file", "Export selected task")
            session.send(str(existing) + "\r")
            session.expect("export refuses overwrite and retains prompt", "Export selected task", "export failed", "keep-export.json")
            assert existing.read_text() == "do-not-overwrite"
            session.capture("jobs_export_refused")
            session.send(b"\x1b")
            session.expect("cancel export returns to jobs", "RECENT TASK", absent=("Export selected task",))
            session.send(b"e")
            session.expect("new export prompt", "Export selected task")
            exported = self.root / "fresh-export.json"
            session.send(str(exported) + "\r")
            session.wait("export file exists", exported.exists)
            report = json.loads(exported.read_text())
            session.expect("successful export returns to jobs", "File jobs", absent=("Export selected task",))
            session.send(b"q")
            session.expect("files before delete identity fixture", "Files", absent=("RECENT TASK",))
            session.send(b"D")
            session.expect("initial delete confirmation", "Permanent delete", "DANGER")
            source.write_text("third-version-changed-before-delete-confirmation")
            session.send(b"delete\rJ")
            session.expect("delete failure retained", "RECENT TASK 1/3", "Delete", "1 failed")
            assert source.exists()
            session.send(b"r")
            session.expect("delete retry requires confirmation again", "Permanent delete", "DANGER")
            assert source.exists()
            session.capture("jobs_retry_delete_confirmation")
            session.send(b"\x1b")
            session.expect("cancel retry leaves files", "Files", absent=("DANGER",))
            session.quit()
        self.checks["jobs"] = {"history_selection": True, "failure_filter": True, "export_json_keys": sorted(report),
                                "export_no_overwrite": True, "retry_new_conflict": True,
                                "retry_delete_reconfirms": True, "delete_fixture_retained": source.exists()}

    def cluster(self):
        inventory = self.root / "local-only-inventory.json"
        inventory.write_text(json.dumps({"main_machine": {"alias": "local-qa", "tailscale_ip": "127.0.0.1", "role": "Local QA", "workdir": str(self.root)}}))
        with Session(self, "--c", "--cluster-config", inventory) as session:
            session.expect("local cluster completes probe", "local-qa", "CHK 0/1", timeout=15)
            session.send(b"a")
            session.expect("attention filter is visible", "attention")
            session.capture("cluster_attention")
            session.send(b"aE")
            session.expect("state-change events overlay", "Events")
            session.capture("cluster_events")
            session.send(b"q")
            session.expect("q returns from events", "local-qa", absent=("Events ",))
            session.send(b"P")
            session.expect("pause automatic probes", "PAUSED")
            session.send(b"b")
            session.expect("cluster host/path places", "Places", "Find host/path:", "[config] local")
            session.capture("cluster_places")
            session.send(b"\x1b")
            session.expect("places escape returns to paused cluster", "PAUSED", absent=("Find host/path:",))
            for width in (40, 80, 120):
                session.resize(width)
                session.expect(f"cluster resize {width} columns", "local-qa", "PAUSED")
                session.capture(f"cluster_{width}_columns")
            session.quit()
        self.checks["cluster_local"] = {"attention": True, "events_q_return": True, "pause": True,
                                         "places": True, "resize_widths": [40, 80, 120], "remote_hosts_contacted": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    assert args.binary.is_file(), f"binary missing: {args.binary}"
    receipt = {"status": "running", "binary": str(args.binary.resolve()), "binary_bytes": args.binary.stat().st_size,
               "binary_sha256": hashlib.sha256(args.binary.read_bytes()).hexdigest(),
               "platform": os.uname().sysname, "architecture": os.uname().machine,
               "python_version": sys.version.split()[0],
               "limits": f"{os.uname().sysname} native subprocess PTY using SSH ASCII profile and fresh fixture state; these scenarios do not exercise SSH transport, other operating systems or mobile/native-terminal rendering. Decoder is a bounded QA screen reconstruction, not a full terminal emulator."}
    with tempfile.TemporaryDirectory(prefix="tersh-product-pty-") as directory:
        runner = Runner(args.binary, Path(directory).resolve())
        try:
            for scenario in (runner.places, runner.structured, runner.log, runner.jobs, runner.cluster):
                scenario()
            receipt["status"] = "passed"
        except Exception as error:
            receipt.update(status="failed", failed_check=runner.current, error=str(error))
            if runner.sessions:
                runner.screens["failure"] = runner.sessions[-1].screen.text
            raise
        finally:
            for session in runner.sessions:
                if not session.closed:
                    try:
                        session.close()
                    except Exception as error:
                        receipt.setdefault("cleanup_errors", []).append(str(error))
            receipt.update(checks=runner.checks, screens=runner.screens, terminal_restorations=runner.restorations)
            receipt["terminal_modes_restored"] = bool(runner.restorations) and all(item["restored"] and item["exit_code"] == 0 for item in runner.restorations)
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps(receipt, indent=2, ensure_ascii=False))
            print(json.dumps({"status": receipt["status"], "checks_completed": list(runner.checks), "receipt": str(args.output.resolve()), "failed_check": receipt.get("failed_check")}))


if __name__ == "__main__":
    main()
