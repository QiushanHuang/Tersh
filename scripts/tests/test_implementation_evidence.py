import hashlib
import importlib
import json
import os
import pathlib
import signal
import stat
import subprocess
import sys
import tempfile
import textwrap
import time
import unittest
from unittest import mock


ROOT = pathlib.Path(__file__).resolve().parents[2]
RUN_GATE = ROOT / "scripts" / "implementation_evidence" / "run_gate.py"
MIB = 1024 * 1024


FALLBACK_RUN_GATE = r'''#!/usr/bin/env python3
"""Deliberately incomplete fixture used only to make pre-implementation RED non-vacuous."""
import subprocess
import sys

args = sys.argv[1:]
try:
    separator = args.index("--")
except ValueError:
    print("fixture-run-gate-stub: missing child argv", file=sys.stderr)
    raise SystemExit(2)
child = subprocess.run(args[separator + 1 :], check=False)
print("fixture-run-gate-stub: evidence contract unavailable", file=sys.stderr)
if "--allow-failure" in args:
    raise SystemExit(0)
raise SystemExit(child.returncode if child.returncode >= 0 else 128 - child.returncode)
'''


class ImplementationEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        # Normalize only this fixture-owned macOS /var alias; production must
        # never resolve or follow a caller-supplied output-root component.
        self.root = pathlib.Path(self.temp.name).resolve(strict=True)
        self.stub = self.root / "run_gate_stub.py"
        self.stub.write_text(textwrap.dedent(FALLBACK_RUN_GATE), encoding="utf-8")
        self.repo, self.candidate_a, self.candidate_b = self.make_repo()

    def make_repo(self):
        repo = self.root / "repo"
        repo.mkdir()
        self.git(repo, "init", "-q")
        self.git(repo, "config", "user.name", "Evidence Fixture")
        self.git(repo, "config", "user.email", "evidence@example.invalid")
        tracked = repo / "tracked.txt"
        tracked.write_text("a\n", encoding="utf-8")
        self.git(repo, "add", "tracked.txt")
        self.git(repo, "commit", "-q", "-m", "candidate a")
        candidate_a = self.git(repo, "rev-parse", "HEAD").stdout.strip()
        tracked.write_text("b\n", encoding="utf-8")
        self.git(repo, "commit", "-q", "-am", "candidate b")
        candidate_b = self.git(repo, "rev-parse", "HEAD").stdout.strip()
        return repo, candidate_a, candidate_b

    def git(self, cwd, *args):
        return subprocess.run(
            ["git", *args], cwd=cwd, check=True, text=True,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )

    @property
    def entrypoint(self):
        return RUN_GATE if RUN_GATE.is_file() else self.stub

    def run_gate(
        self,
        *,
        evidence_id="impl-01",
        attempt="001",
        candidate=None,
        run_binding="run-local",
        name="gate",
        output_root=None,
        child=None,
        allow_failure=False,
        cwd=None,
    ):
        candidate = candidate or self.candidate_b
        output_root = os.fspath(self.root / "evidence") if output_root is None else os.fspath(output_root)
        child = child or [sys.executable, "-c", "raise SystemExit(0)"]
        command = [
            sys.executable,
            str(self.entrypoint),
            "--iteration", evidence_id,
            "--attempt", attempt,
            "--run-binding", run_binding,
            "--name", name,
            "--candidate", candidate,
            "--output-root", output_root,
        ]
        if allow_failure:
            command.append("--allow-failure")
        command += ["--", *child]
        env = os.environ.copy()
        env["PYTHONDONTWRITEBYTECODE"] = "1"
        return subprocess.run(
            command,
            cwd=cwd or self.repo,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

    def gate_base(
        self,
        *,
        output_root=None,
        evidence_id="impl-01",
        attempt="001",
        candidate=None,
        run_binding="run-local",
        name="gate",
    ):
        output_root = pathlib.Path(self.root / "evidence") if output_root is None else pathlib.Path(output_root)
        candidate = candidate or self.candidate_b
        return (
            output_root
            / evidence_id
            / f"attempt-{attempt}"
            / f"candidate-{candidate}"
            / run_binding
            / "gates"
            / name
        )

    def load_record(self, base):
        path = pathlib.Path(f"{base}.json")
        self.assertTrue(
            path.is_file(),
            f"run_gate contract did not create canonical record {path}; "
            f"entrypoint={self.entrypoint}",
        )
        raw = path.read_bytes()
        parsed = json.loads(raw)
        self.assertEqual(
            raw,
            json.dumps(parsed, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
            + b"\n",
        )
        return parsed

    def assert_child_not_run(self, result, marker):
        self.assertFalse(
            marker.exists(),
            f"rejected input reached child; stderr={result.stderr[:512]!r}",
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertLessEqual(len(result.stderr), 8192)

    def marker_child(self, marker):
        return [
            sys.executable,
            "-c",
            "import pathlib,sys; pathlib.Path(sys.argv[1]).write_text('ran',encoding='utf-8')",
            str(marker),
        ]

    def test_run_gate_drains_hashes_and_caps_both_streams(self):
        stdout_tail = b"stdout-tail-after-cap"
        stderr_tail = b"stderr-tail-after-cap"
        child = [
            sys.executable,
            "-c",
            textwrap.dedent(
                """
                import sys, threading
                def emit(stream, byte, tail):
                    for _ in range(1024):
                        stream.write(byte * 1024)
                        stream.flush()
                    stream.write(tail)
                    stream.flush()
                threads = [
                    threading.Thread(target=emit, args=(sys.stdout.buffer, b'O', b'stdout-tail-after-cap')),
                    threading.Thread(target=emit, args=(sys.stderr.buffer, b'E', b'stderr-tail-after-cap')),
                ]
                for thread in threads: thread.start()
                for thread in threads: thread.join()
                """
            ),
        ]
        result = self.run_gate(name="dual-stream", child=child)
        self.assertEqual(result.returncode, 0, result.stderr[:1024])
        base = self.gate_base(name="dual-stream")
        record = self.load_record(base)
        expected = {
            "stdout": b"O" * MIB + stdout_tail,
            "stderr": b"E" * MIB + stderr_tail,
        }
        for stream_name, complete in expected.items():
            stream = record[stream_name]
            retained = complete[:MIB]
            self.assertEqual(stream["total_bytes"], len(complete))
            self.assertEqual(stream["sha256"], hashlib.sha256(complete).hexdigest())
            self.assertEqual(stream["retained_bytes"], MIB)
            self.assertEqual(stream["retained_sha256"], hashlib.sha256(retained).hexdigest())
            log_path = pathlib.Path(f"{base}.{stream_name}")
            self.assertEqual(log_path.read_bytes(), retained)

    def test_run_gate_preserves_child_status_candidate_attempt_and_run_binding(self):
        cases = (
            ("exit-seven", [sys.executable, "-c", "raise SystemExit(7)"], False, 7, 7),
            ("exit-seven-allowed", [sys.executable, "-c", "raise SystemExit(7)"], True, 0, 7),
            (
                "sigterm",
                [sys.executable, "-c", "import os,signal; os.kill(os.getpid(),signal.SIGTERM)"],
                False,
                None,
                -signal.SIGTERM,
            ),
            (
                "sigterm-allowed",
                [sys.executable, "-c", "import os,signal; os.kill(os.getpid(),signal.SIGTERM)"],
                True,
                0,
                -signal.SIGTERM,
            ),
        )
        for name, child, allowed, wrapper_status, child_status in cases:
            with self.subTest(name=name):
                result = self.run_gate(name=name, child=child, allow_failure=allowed)
                if wrapper_status is None:
                    self.assertNotEqual(result.returncode, 0)
                else:
                    self.assertEqual(result.returncode, wrapper_status, result.stderr[:512])
                record = self.load_record(self.gate_base(name=name))
                self.assertEqual(record["evidence_attempt"], "001")
                self.assertEqual(record["candidate"], self.candidate_b)
                self.assertEqual(record["run_binding"], "run-local")
                self.assertEqual(record["exit_code"], child_status)

    def test_every_raw_record_is_create_new_beneath_attempt_candidate_and_run_binding(self):
        for suffix in ("json", "stdout", "stderr"):
            with self.subTest(suffix=suffix):
                output_root = self.root / f"collision-{suffix}"
                base = self.gate_base(output_root=output_root, name="collision")
                base.parent.mkdir(parents=True)
                existing = pathlib.Path(f"{base}.{suffix}")
                existing.write_bytes(b"immutable-existing")
                before = (existing.stat().st_ino, existing.read_bytes())
                marker = self.root / f"child-{suffix}.marker"
                result = self.run_gate(
                    output_root=output_root,
                    name="collision",
                    child=self.marker_child(marker),
                )
                self.assert_child_not_run(result, marker)
                self.assertEqual((existing.stat().st_ino, existing.read_bytes()), before)
                for other in {"json", "stdout", "stderr"} - {suffix}:
                    self.assertFalse(pathlib.Path(f"{base}.{other}").exists())

    def test_evidence_id_union_accepts_only_impl_or_hardening_01_through_07(self):
        valid_ids = [f"{family}-0{number}" for family in ("impl", "hardening") for number in range(1, 8)]
        self.assertEqual(len(valid_ids), 14)
        for index, evidence_id in enumerate(valid_ids, start=1):
            with self.subTest(valid=evidence_id):
                output_root = self.root / f"valid-id-{index}"
                result = self.run_gate(
                    evidence_id=evidence_id,
                    output_root=output_root,
                    name="valid-id",
                )
                self.assertEqual(result.returncode, 0, result.stderr[:512])
                self.load_record(
                    self.gate_base(
                        output_root=output_root,
                        evidence_id=evidence_id,
                        name="valid-id",
                    )
                )

        invalid_ids = (
            "impl-00", "impl-08", "hardening-00", "hardening-08", "Impl-01",
            "impl-1", "impl-001", "impl-01x", "ximpl-01", "impl-01\n",
            "impl/01", "implementation-01",
        )
        for index, evidence_id in enumerate(invalid_ids, start=1):
            with self.subTest(invalid=evidence_id):
                marker = self.root / f"invalid-id-{index}.marker"
                result = self.run_gate(
                    evidence_id=evidence_id,
                    output_root=self.root / f"invalid-id-root-{index}",
                    child=self.marker_child(marker),
                )
                self.assert_child_not_run(result, marker)

    def test_attempt_root_is_candidate_independent_and_per_commit_records_are_immutable(self):
        output_root = self.root / "candidate-history"
        snapshots = {}
        for candidate in (self.candidate_a, self.candidate_b):
            self.git(self.repo, "checkout", "-q", "--detach", candidate)
            result = self.run_gate(
                output_root=output_root,
                candidate=candidate,
                name="coexist",
            )
            self.assertEqual(result.returncode, 0, result.stderr[:512])
            base = self.gate_base(
                output_root=output_root,
                candidate=candidate,
                name="coexist",
            )
            self.load_record(base)
            snapshots[candidate] = {
                suffix: (
                    pathlib.Path(f"{base}.{suffix}").stat().st_ino,
                    pathlib.Path(f"{base}.{suffix}").read_bytes(),
                )
                for suffix in ("json", "stdout", "stderr")
            }

        self.assertNotEqual(self.candidate_a, self.candidate_b)
        self.git(self.repo, "checkout", "-q", "--detach", self.candidate_a)
        marker = self.root / "candidate-rerun.marker"
        rerun = self.run_gate(
            output_root=output_root,
            candidate=self.candidate_a,
            name="coexist",
            child=self.marker_child(marker),
        )
        self.assert_child_not_run(rerun, marker)
        for candidate, snapshot in snapshots.items():
            base = self.gate_base(
                output_root=output_root,
                candidate=candidate,
                name="coexist",
            )
            for suffix, expected in snapshot.items():
                path = pathlib.Path(f"{base}.{suffix}")
                self.assertEqual((path.stat().st_ino, path.read_bytes()), expected)

    def test_run_binding_rejects_empty_doubled_trailing_duplicate_reordered_zero_or_unknown_components(self):
        valid = (
            "run-local",
            "run-cumulative",
            "run-set-ci-1",
            "run-set-ci-unregistered",
            "run-set-release-1",
            "run-set-release-unregistered",
            "run-set-ci-1-release-2",
            "run-set-ci-unregistered-release-unregistered",
        )
        for index, binding in enumerate(valid, start=1):
            with self.subTest(valid=binding):
                output_root = self.root / f"valid-binding-{index}"
                result = self.run_gate(
                    output_root=output_root,
                    run_binding=binding,
                    name="binding",
                )
                self.assertEqual(result.returncode, 0, result.stderr[:512])
                self.load_record(
                    self.gate_base(
                        output_root=output_root,
                        run_binding=binding,
                        name="binding",
                    )
                )

        invalid = (
            "", "run-set", "run-set-ci", "run-set-release", "run-set-ci-0",
            "run-set-ci-01", "run-set-ci--1", "run-set-ci-1-", "run-set-ci-1-ci-2",
            "run-set-release-2-ci-1", "run-set-deploy-1", "run-set-ci-1-release-2-extra",
            "RUN-LOCAL", "run/local", "run-local ", "run-local\n", "run-unregistered-ci",
        )
        for index, binding in enumerate(invalid, start=1):
            with self.subTest(invalid=repr(binding)):
                marker = self.root / f"invalid-binding-{index}.marker"
                result = self.run_gate(
                    output_root=self.root / f"invalid-binding-root-{index}",
                    run_binding=binding,
                    child=self.marker_child(marker),
                )
                self.assert_child_not_run(result, marker)

    def test_run_gate_rejects_ancestor_and_final_symlinks_without_running_child(self):
        for level in ("output-root", "evidence-id", "attempt", "candidate", "run", "gates"):
            with self.subTest(ancestor=level):
                sandbox = self.root / f"symlink-{level}"
                outside = sandbox / "outside"
                outside.mkdir(parents=True)
                output_root = sandbox / "evidence"
                components = [
                    output_root,
                    output_root / "impl-01",
                    output_root / "impl-01" / "attempt-001",
                    output_root / "impl-01" / "attempt-001" / f"candidate-{self.candidate_b}",
                    output_root / "impl-01" / "attempt-001" / f"candidate-{self.candidate_b}" / "run-local",
                    output_root / "impl-01" / "attempt-001" / f"candidate-{self.candidate_b}" / "run-local" / "gates",
                ]
                target_index = ("output-root", "evidence-id", "attempt", "candidate", "run", "gates").index(level)
                if target_index:
                    components[target_index - 1].mkdir(parents=True)
                components[target_index].symlink_to(outside, target_is_directory=True)
                marker = sandbox / "child.marker"
                result = self.run_gate(
                    output_root=output_root,
                    name="symlinked",
                    child=self.marker_child(marker),
                )
                self.assert_child_not_run(result, marker)
                self.assertEqual(list(outside.iterdir()), [])

        for suffix in ("json", "stdout", "stderr"):
            with self.subTest(final_symlink=suffix):
                output_root = self.root / f"final-symlink-{suffix}"
                outside = self.root / f"outside-{suffix}.txt"
                outside.write_bytes(b"outside-immutable")
                base = self.gate_base(output_root=output_root, name="symlinked")
                base.parent.mkdir(parents=True)
                pathlib.Path(f"{base}.{suffix}").symlink_to(outside)
                marker = self.root / f"final-symlink-{suffix}.marker"
                result = self.run_gate(
                    output_root=output_root,
                    name="symlinked",
                    child=self.marker_child(marker),
                )
                self.assert_child_not_run(result, marker)
                self.assertEqual(outside.read_bytes(), b"outside-immutable")

    def test_canonical_json_rejects_duplicate_nonfinite_bool_and_noncanonical_inputs(self):
        try:
            core = importlib.import_module("scripts.evidence_core")
        except ImportError as error:
            self.fail(f"missing evidence_core canonical validators: {error}")
        dumps = getattr(core, "canonical_json_bytes", None)
        loads = getattr(core, "load_canonical_json_bytes", None)
        exact_int = getattr(core, "require_exact_int", None)
        self.assertTrue(callable(dumps), "evidence_core.canonical_json_bytes is required")
        self.assertTrue(callable(loads), "evidence_core.load_canonical_json_bytes is required")
        self.assertTrue(callable(exact_int), "evidence_core.require_exact_int is required")

        canonical = b'{"a":1,"b":["x"]}\n'
        self.assertEqual(dumps({"b": ["x"], "a": 1}), canonical)
        self.assertEqual(loads(canonical), {"a": 1, "b": ["x"]})
        malformed = (
            b'{"a":1,"a":2}\n',
            b'{"a":{"x":1,"x":2}}\n',
            b'{"a":NaN}\n',
            b'{"a":Infinity}\n',
            b'{"a":-Infinity}\n',
            b'{ "a":1,"b":["x"]}\n',
            b'{"b":["x"],"a":1}\n',
            b'{"a":1,"b":["x"]}',
            b'{"a":1,"b":["x"]}\n\n',
            b'\xef\xbb\xbf{"a":1,"b":["x"]}\n',
            b'\xff',
        )
        for raw in malformed:
            with self.subTest(raw=raw):
                with self.assertRaises(Exception):
                    loads(raw)
        self.assertEqual(exact_int(1, "count"), 1)
        for value in (True, False, 1.0, "1", None):
            with self.subTest(value=value):
                with self.assertRaises(Exception):
                    exact_int(value, "count")
        for value in (float("nan"), float("inf"), float("-inf")):
            with self.subTest(nonfinite=value):
                with self.assertRaises(Exception):
                    dumps({"value": value})

    def test_core_requires_nofollow_and_detects_directory_identity_replacement(self):
        core = importlib.import_module("scripts.evidence_core")
        with mock.patch.object(core.os, "O_NOFOLLOW", 0):
            with self.assertRaises(Exception):
                core._directory_flags()

        parent = self.root / "identity-parent"
        parent.mkdir(mode=0o700)
        (parent / "victim").mkdir(mode=0o700)
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        real_open = core.os.open
        swapped = False

        def swap_before_open(path, flags, *args, dir_fd=None, **kwargs):
            nonlocal swapped
            if path == "victim" and dir_fd is not None and not swapped:
                swapped = True
                os.rename("victim", "displaced", src_dir_fd=dir_fd, dst_dir_fd=dir_fd)
                os.mkdir("victim", 0o700, dir_fd=dir_fd)
            return real_open(path, flags, *args, dir_fd=dir_fd, **kwargs)

        opened_fd = None
        try:
            with mock.patch.object(core.os, "open", side_effect=swap_before_open):
                try:
                    opened_fd = core.open_internal_tree(parent_fd, ("victim",))
                except Exception:
                    pass
                else:
                    self.fail("directory replacement between validation and open was accepted")
            self.assertTrue(swapped, "identity test never reached the validation/open boundary")
        finally:
            if opened_fd is not None:
                os.close(opened_fd)
            os.close(parent_fd)

    def test_run_gate_passes_child_argv_without_a_shell(self):
        captured = self.root / "literal-argv.txt"
        injected = self.root / "shell-injected.txt"
        payload = f"$(touch {injected}) ; touch {injected}"
        child = [
            sys.executable,
            "-c",
            "import pathlib,sys; pathlib.Path(sys.argv[1]).write_text(sys.argv[2],encoding='utf-8')",
            str(captured),
            payload,
        ]
        result = self.run_gate(name="argv-literal", child=child)
        self.assertEqual(result.returncode, 0, result.stderr[:512])
        self.assertEqual(captured.read_text(encoding="utf-8"), payload)
        self.assertFalse(injected.exists())
        record = self.load_record(self.gate_base(name="argv-literal"))
        self.assertEqual(record["argv"], child)

        captured_separator = self.root / "literal-child-separator.json"
        child_with_separator = [
            sys.executable,
            "-c",
            "import json,pathlib,sys; pathlib.Path(sys.argv[1]).write_text(json.dumps(sys.argv[2:]),encoding='utf-8')",
            str(captured_separator),
            "--",
            "--exact",
        ]
        separator_result = self.run_gate(name="argv-child-separator", child=child_with_separator)
        self.assertEqual(separator_result.returncode, 0, separator_result.stderr[:512])
        self.assertEqual(json.loads(captured_separator.read_text(encoding="utf-8")), ["--", "--exact"])
        separator_record = self.load_record(self.gate_base(name="argv-child-separator"))
        self.assertEqual(separator_record["argv"], child_with_separator)

    def test_run_gate_reserves_one_concurrent_triplet_before_starting_child(self):
        output_root = self.root / "concurrent"
        entered = self.root / "concurrent-entered.txt"
        release = self.root / "concurrent-release"
        child_program = textwrap.dedent(
            """
            import pathlib, sys, time
            entered, release, token = map(pathlib.Path, sys.argv[1:])
            with entered.open("ab", buffering=0) as stream:
                stream.write(bytes(token.name + "\\n", "utf-8"))
            deadline = time.monotonic() + 5
            while not release.exists():
                if time.monotonic() >= deadline:
                    raise SystemExit(91)
                time.sleep(0.01)
            print(token.name)
            """
        )

        def command(token):
            return [
                sys.executable, str(RUN_GATE),
                "--iteration", "impl-01",
                "--attempt", "001",
                "--run-binding", "run-local",
                "--name", "concurrent-gate",
                "--candidate", self.candidate_b,
                "--output-root", str(output_root),
                "--", sys.executable, "-c", child_program,
                str(entered), str(release), token,
            ]

        env = os.environ.copy()
        env["PYTHONDONTWRITEBYTECODE"] = "1"
        first = subprocess.Popen(
            command("first"), cwd=self.repo, env=env,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        second = subprocess.Popen(
            command("second"), cwd=self.repo, env=env,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        deadline = time.monotonic() + 5
        while not entered.exists() and time.monotonic() < deadline:
            time.sleep(0.01)
        self.assertTrue(entered.exists(), "no reserved child reached the barrier")
        release.write_bytes(b"release")
        first_output = first.communicate(timeout=10)
        second_output = second.communicate(timeout=10)
        statuses = (first.returncode, second.returncode)
        self.assertEqual(sum(status == 0 for status in statuses), 1, (statuses, first_output, second_output))
        entered_tokens = entered.read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(entered_tokens), 1, entered_tokens)

        base = self.gate_base(output_root=output_root, name="concurrent-gate")
        record = self.load_record(base)
        winner = entered_tokens[0]
        self.assertEqual(record["argv"][-1], winner)
        self.assertEqual(pathlib.Path(f"{base}.stdout").read_text(encoding="utf-8"), winner + "\n")
        self.assertEqual(
            sorted(path.name for path in base.parent.iterdir()),
            ["concurrent-gate.json", "concurrent-gate.stderr", "concurrent-gate.stdout"],
        )

    def test_run_gate_record_and_stream_schemas_paths_versions_and_modes_are_closed(self):
        output_root = self.root / "closed-record"
        child = [
            sys.executable,
            "-c",
            "import sys; sys.stdout.write('out'); sys.stderr.write('ordinary diagnostic')",
        ]
        before = time.time()
        result = self.run_gate(output_root=output_root, name="record-contract", child=child)
        after = time.time()
        self.assertEqual(result.returncode, 0, result.stderr[:1024])
        base = self.gate_base(output_root=output_root, name="record-contract")
        record = self.load_record(base)
        self.assertEqual(
            set(record),
            {
                "schema", "iteration", "evidence_attempt", "run_binding", "name",
                "argv", "cwd", "candidate", "started_at", "ended_at", "duration_ms",
                "exit_code", "stdout", "stderr", "os", "architecture", "rustc_version",
                "cargo_version", "exact_test_inventory",
            },
        )
        self.assertEqual(record["schema"], "tersh-implementation-gate-v1")
        self.assertEqual(record["iteration"], "impl-01")
        self.assertEqual(record["evidence_attempt"], "001")
        self.assertEqual(record["run_binding"], "run-local")
        self.assertEqual(record["name"], "record-contract")
        self.assertEqual(record["argv"], child)
        self.assertEqual(record["cwd"], str(self.repo.resolve()))
        self.assertEqual(record["candidate"], self.candidate_b)
        self.assertEqual(record["exit_code"], 0)
        self.assertIsNone(record["exact_test_inventory"])
        self.assertIs(type(record["duration_ms"]), int)
        self.assertGreaterEqual(record["duration_ms"], 0)
        timestamp = r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{1,9}Z$"
        self.assertRegex(record["started_at"], timestamp)
        self.assertRegex(record["ended_at"], timestamp)
        self.assertLessEqual(record["started_at"], record["ended_at"])
        self.assertLessEqual(before - 2, after)
        self.assertIs(type(record["os"]), str)
        self.assertTrue(record["os"])
        self.assertIs(type(record["architecture"]), str)
        self.assertTrue(record["architecture"])
        self.assertRegex(record["rustc_version"], r"^rustc .+")
        self.assertRegex(record["cargo_version"], r"^cargo .+")

        for stream_name, payload in (("stdout", b"out"), ("stderr", b"ordinary diagnostic")):
            stream = record[stream_name]
            self.assertEqual(
                set(stream),
                {"total_bytes", "sha256", "retained_bytes", "retained_sha256", "retained_log"},
            )
            self.assertEqual(stream["total_bytes"], len(payload))
            self.assertEqual(stream["sha256"], hashlib.sha256(payload).hexdigest())
            self.assertEqual(stream["retained_bytes"], len(payload))
            self.assertEqual(stream["retained_sha256"], hashlib.sha256(payload).hexdigest())
            self.assertEqual(stream["retained_log"], f"run-local/gates/record-contract.{stream_name}")
            log_path = pathlib.Path(f"{base}.{stream_name}")
            self.assertEqual(log_path.read_bytes(), payload)

        for suffix in ("json", "stdout", "stderr"):
            path = pathlib.Path(f"{base}.{suffix}")
            self.assertTrue(stat.S_ISREG(path.stat().st_mode))
            self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o600)
        current = output_root
        while True:
            self.assertEqual(stat.S_IMODE(current.stat().st_mode), 0o700)
            if current == base.parent:
                break
            relative_parts = base.parent.relative_to(output_root).parts
            current = output_root.joinpath(*relative_parts[: len(current.relative_to(output_root).parts) + 1])

    def test_run_gate_rejects_raw_path_and_basic_component_contracts_before_child(self):
        unsafe_roots = (
            "",
            f"{self.root}/unsafe/../escape",
            f"{self.root}/unsafe/./dot",
            f"{self.root}/unsafe//repeated",
            f"{self.root}/unsafe/trailing/",
        )
        for index, raw_root in enumerate(unsafe_roots, start=1):
            with self.subTest(output_root=repr(raw_root)):
                marker = self.root / f"unsafe-root-{index}.marker"
                result = self.run_gate(output_root=raw_root, child=self.marker_child(marker))
                self.assert_child_not_run(result, marker)

        invalid_components = (
            {"attempt": "000"}, {"attempt": "01"}, {"attempt": "1000"},
            {"candidate": "A" * 40}, {"candidate": "a" * 39}, {"candidate": "a" * 41},
            {"name": "Gate"}, {"name": "-gate"}, {"name": "gate_underscore"},
            {"name": "g" * 65},
        )
        for index, overrides in enumerate(invalid_components, start=1):
            with self.subTest(overrides=overrides):
                marker = self.root / f"invalid-component-{index}.marker"
                result = self.run_gate(
                    output_root=self.root / f"invalid-component-root-{index}",
                    child=self.marker_child(marker),
                    **overrides,
                )
                self.assert_child_not_run(result, marker)


if __name__ == "__main__":
    unittest.main()
