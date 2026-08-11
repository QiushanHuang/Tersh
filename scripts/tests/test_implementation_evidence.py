import ast
import copy
import hashlib
import importlib
import io
import json
import os
import pathlib
import re
import signal
import socket
import stat
import struct
import subprocess
import sys
import tempfile
import textwrap
import threading
import time
import unittest
from unittest import mock


ROOT = pathlib.Path(__file__).resolve().parents[2]
RUN_GATE = ROOT / "scripts" / "implementation_evidence" / "run_gate.py"
MIB = 1024 * 1024


def fixture_canonical_json(value):
    return json.dumps(
        value,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8") + b"\n"


def fixture_send_frame(sock, value):
    body = fixture_canonical_json(value)
    sock.sendall(struct.pack(">I", len(body)) + body)


def fixture_recv_exact(sock, size):
    result = bytearray()
    while len(result) < size:
        chunk = sock.recv(size - len(result))
        if not chunk:
            raise EOFError("scripted host received early EOF")
        result.extend(chunk)
    return bytes(result)


def fixture_recv_frame(sock):
    size = struct.unpack(">I", fixture_recv_exact(sock, 4))[0]
    if not 1 <= size <= 65536:
        raise ValueError(f"scripted host received invalid frame size {size}")
    raw = fixture_recv_exact(sock, size)
    value = json.loads(raw)
    if fixture_canonical_json(value) != raw:
        raise ValueError("scripted host received noncanonical JSON frame")
    return value


class ScriptedCaptureStore:
    def __init__(self):
        self.contexts = {}
        self.invocations = set()
        self.responses = {}
        self.next_handle = 1

    def new_handle(self):
        value = f"{self.next_handle:064x}"
        self.next_handle += 1
        return value

    def valid_context(self, handle):
        return handle is not None and handle in self.contexts

    def prepare_bodies(self, operation, context_handle, context_body, invocation_body, response_body):
        if operation == "capture-context":
            return [("context", copy.deepcopy(context_body))]
        if not self.valid_context(context_handle):
            return None
        current = copy.deepcopy(self.contexts[context_handle])
        if operation == "capture-invocation":
            return [("context", current), ("invocation", copy.deepcopy(invocation_body))]
        if operation == "capture-response":
            return [("context", current), ("response", copy.deepcopy(response_body))]
        raise AssertionError(f"unexpected scripted capture operation {operation}")

    def commit(self, operation, context_handle, bodies):
        committed = {kind: copy.deepcopy(body) for kind, body in bodies}
        if operation == "capture-context":
            successor = self.new_handle()
            self.contexts[successor] = committed["context"]
            return {"schema": "tersh-host-capture-context-result-v1", "context_handle": successor}
        if context_handle not in self.contexts:
            raise AssertionError("scripted host committed an invalid context handle")
        current = self.contexts.pop(context_handle)
        successor = self.new_handle()
        self.contexts[successor] = current
        if operation == "capture-invocation":
            member = self.new_handle()
            self.invocations.add(member)
            return {
                "schema": "tersh-host-capture-invocation-result-v1",
                "context_handle": successor,
                "invocation_handle": member,
            }
        if operation == "capture-response":
            member = self.new_handle()
            self.responses[member] = committed["response"]
            return {
                "schema": "tersh-host-capture-response-result-v1",
                "context_handle": successor,
                "response_handle": member,
            }
        raise AssertionError(f"unexpected scripted commit operation {operation}")


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

    def host_envelope_bodies(self):
        context_nonce = "c" * 64
        dispatch_id = "d" * 64
        harness_bundle_revision = "7" * 40
        harness_bundle_sha256 = "8" * 64
        context = {
            "schema": "tersh-host-dispatch-context-v1",
            "context_nonce": context_nonce,
            "harness_bundle_revision": harness_bundle_revision,
            "harness_bundle_sha256": harness_bundle_sha256,
            "evidence_id": "impl-01",
            "evidence_attempt": "001",
            "role": "safety",
            "wave": "wave-c",
            "review_attempt": "001",
            "run_binding": "run-local",
            "baseline_commit": self.candidate_a,
            "review_target": self.candidate_b,
            "canonical_task_path": "/root/safety/reviewer",
            "worktree_handle": "fixture-worktree",
            "requested_model": "gpt-5.6-sol",
            "requested_reasoning_effort": "xhigh",
            "created_at": "2026-08-10T00:00:00.000000001Z",
        }
        invocation = {
            "schema": "tersh-host-spawn-invocation-v1",
            "context_nonce": context_nonce,
            "dispatch_id": dispatch_id,
            "harness_bundle_revision": harness_bundle_revision,
            "harness_bundle_sha256": harness_bundle_sha256,
            "requested_model": "gpt-5.6-sol",
            "requested_reasoning_effort": "xhigh",
            "selected_model": "gpt-5.6-sol",
            "selected_reasoning_effort": "xhigh",
            "dispatched_at": "2026-08-10T00:00:01.000000001Z",
        }
        response = {
            "schema": "tersh-host-spawn-response-v2",
            "context_nonce": context_nonce,
            "dispatch_id": dispatch_id,
            "harness_bundle_revision": harness_bundle_revision,
            "harness_bundle_sha256": harness_bundle_sha256,
            "agent_id": "fixture-agent",
            "canonical_task_path": "/root/safety/reviewer",
            "agent_run_id": "fixture-run",
            "started_at": "2026-08-10T00:00:02.000000001Z",
            "ended_at": "2026-08-10T00:00:03.000000001Z",
            "terminal_status": "completed",
            "reported_result_commit": self.candidate_b,
            "reported_record_sha256": None,
        }
        return context, invocation, response

    def run_scripted_capture(
        self,
        store,
        operation,
        context_handle=None,
        *,
        scenario="success",
        body_mutator=None,
    ):
        core = importlib.import_module("scripts.evidence_core")
        capture = getattr(core, "capture_host_envelope_on_authenticated_socket", None)
        self.assertTrue(callable(capture), "shared capture transaction seam is required")
        context_body, invocation_body, response_body = self.host_envelope_bodies()
        client, host = socket.socketpair(socket.AF_UNIX, socket.SOCK_STREAM)
        client.settimeout(2)
        host.settimeout(2)
        host_errors = []
        transcript = []

        def scripted_host():
            try:
                begin = fixture_recv_frame(host)
                transcript.append(("client", begin))
                expected_begin = {
                    "schema": "tersh-host-transaction-begin-v1",
                    "transaction_nonce": begin.get("transaction_nonce"),
                    "operation": operation,
                }
                if operation != "capture-context":
                    expected_begin["context_handle"] = context_handle
                if begin != expected_begin:
                    raise AssertionError(f"client BEGIN is not exact: {begin!r}")
                nonce = begin["transaction_nonce"]
                if (
                    type(nonce) is not str
                    or len(nonce) != 64
                    or any(character not in "0123456789abcdef" for character in nonce)
                ):
                    raise AssertionError("client transaction nonce is not 64 lowercase hex")
                if scenario == "reply-before-commit":
                    fixture_send_frame(
                        host,
                        {
                            "schema": "tersh-host-transaction-reply-v1",
                            "transaction_nonce": nonce,
                            "operation": operation,
                            "body_sha256s": [],
                            "result": {"schema": "unexpected-reply-v1"},
                        },
                    )
                    host.shutdown(socket.SHUT_WR)
                    return

                bodies = store.prepare_bodies(
                    operation,
                    context_handle,
                    context_body,
                    invocation_body,
                    response_body,
                )
                if bodies is None:
                    host.shutdown(socket.SHUT_WR)
                    return
                if body_mutator is not None:
                    bodies = body_mutator(copy.deepcopy(bodies))
                digests = [hashlib.sha256(fixture_canonical_json(body)).hexdigest() for _, body in bodies]
                wrappers = []
                for ordinal, ((kind, body), digest) in enumerate(zip(bodies, digests), start=1):
                    wrappers.append(
                        {
                            "schema": "tersh-host-transaction-body-v1",
                            "transaction_nonce": nonce,
                            "operation": operation,
                            "body_kind": kind,
                            "ordinal": ordinal,
                            "total": len(bodies),
                            "body": body,
                            "body_sha256": digest,
                        }
                    )
                body_end = {
                    "schema": "tersh-host-transaction-body-end-v1",
                    "transaction_nonce": nonce,
                    "operation": operation,
                    "total": len(bodies),
                    "body_sha256s": digests,
                }
                if scenario == "wrong-body-nonce":
                    wrappers[0]["transaction_nonce"] = "e" * 64
                if scenario == "wrong-body-digest":
                    wrappers[0]["body_sha256"] = "f" * 64
                if scenario == "bool-body-ordinal":
                    wrappers[0]["ordinal"] = True
                if scenario == "wrong-body-schema":
                    wrappers[0]["schema"] = "tersh-host-transaction-body-v0"
                if scenario == "wrong-body-operation":
                    wrappers[0]["operation"] = (
                        "capture-response" if operation != "capture-response" else "capture-context"
                    )
                if scenario == "wrong-body-kind":
                    wrappers[0]["body_kind"] = "response"
                if scenario == "wrong-body-total":
                    wrappers[0]["total"] = len(bodies) + 1
                if scenario == "bool-body-total":
                    wrappers[0]["total"] = True
                if scenario == "extra-body-key":
                    wrappers[0]["extra"] = None
                if scenario == "duplicate-body":
                    wrappers.insert(1, copy.deepcopy(wrappers[0]))
                if scenario == "reordered-bodies":
                    wrappers.reverse()
                if scenario == "extra-body":
                    extra = copy.deepcopy(wrappers[-1])
                    extra["ordinal"] = len(wrappers) + 1
                    wrappers.append(extra)
                if scenario == "wrong-body-end-nonce":
                    body_end["transaction_nonce"] = "e" * 64
                if scenario == "wrong-body-end-schema":
                    body_end["schema"] = "tersh-host-transaction-body-end-v0"
                if scenario == "wrong-body-end-operation":
                    body_end["operation"] = (
                        "capture-response" if operation != "capture-response" else "capture-context"
                    )
                if scenario == "wrong-body-end-total":
                    body_end["total"] = len(bodies) + 1
                if scenario == "bool-body-end-total":
                    body_end["total"] = True
                if scenario == "wrong-body-end-digests":
                    body_end["body_sha256s"] = ["f" * 64 for _ in digests]
                if scenario == "extra-body-end-key":
                    body_end["extra"] = None
                if scenario == "body-end-before-body":
                    fixture_send_frame(host, body_end)
                    for wrapper in wrappers:
                        fixture_send_frame(host, wrapper)
                    host.shutdown(socket.SHUT_WR)
                    return
                for wrapper in wrappers:
                    fixture_send_frame(host, wrapper)
                if scenario == "missing-body-end":
                    host.shutdown(socket.SHUT_WR)
                    return
                fixture_send_frame(host, body_end)
                if scenario == "duplicate-body-end":
                    fixture_send_frame(host, body_end)

                commit = fixture_recv_frame(host)
                request_end = fixture_recv_frame(host)
                transcript.extend((("client", commit), ("client", request_end)))
                expected_commit = {
                    "schema": "tersh-host-transaction-commit-v1",
                    "transaction_nonce": nonce,
                    "operation": operation,
                    "body_sha256s": digests,
                }
                if commit != expected_commit:
                    raise AssertionError(f"client COMMIT is not exact: {commit!r}")
                expected_request_end = {
                    "schema": "tersh-host-transaction-request-end-v1",
                    "transaction_nonce": nonce,
                    "operation": operation,
                    "commit_sha256": hashlib.sha256(fixture_canonical_json(commit)).hexdigest(),
                }
                if request_end != expected_request_end:
                    raise AssertionError(f"client REQUEST-END is not exact: {request_end!r}")
                if host.recv(1) != b"":
                    raise AssertionError("client did not half-close after REQUEST-END")
                result = store.commit(operation, context_handle, bodies)
                reply = {
                    "schema": "tersh-host-transaction-reply-v1",
                    "transaction_nonce": nonce,
                    "operation": operation,
                    "body_sha256s": digests,
                    "result": result,
                }
                if scenario == "wrong-reply-nonce":
                    reply["transaction_nonce"] = "e" * 64
                if scenario == "wrong-reply-digest":
                    reply["body_sha256s"] = ["f" * 64 for _ in digests]
                if scenario == "wrong-reply-schema":
                    reply["schema"] = "tersh-host-transaction-reply-v0"
                if scenario == "wrong-reply-operation":
                    reply["operation"] = (
                        "capture-response" if operation != "capture-response" else "capture-context"
                    )
                if scenario == "extra-reply-key":
                    reply["extra"] = None
                if scenario == "wrong-result-schema":
                    reply["result"]["schema"] = "tersh-host-capture-result-v0"
                if scenario == "extra-result-key":
                    reply["result"]["extra"] = None
                if scenario == "bad-result-context-handle":
                    reply["result"]["context_handle"] = "not-a-handle"
                if scenario == "bad-result-member-handle":
                    member_field = {
                        "capture-invocation": "invocation_handle",
                        "capture-response": "response_handle",
                    }.get(operation)
                    if member_field is None:
                        raise AssertionError(
                            "bad-result-member-handle requires a member-producing operation"
                        )
                    reply["result"][member_field] = "not-a-handle"
                if scenario == "reused-predecessor-context-handle":
                    if context_handle is None:
                        raise AssertionError(
                            "reused-predecessor-context-handle requires an input handle"
                        )
                    reply["result"]["context_handle"] = context_handle
                if scenario in (
                    "member-aliases-predecessor",
                    "member-aliases-successor",
                ):
                    member_field = {
                        "capture-invocation": "invocation_handle",
                        "capture-response": "response_handle",
                    }.get(operation)
                    if member_field is None or context_handle is None:
                        raise AssertionError(f"{scenario} requires a member-producing operation")
                    reply["result"][member_field] = (
                        context_handle
                        if scenario == "member-aliases-predecessor"
                        else reply["result"]["context_handle"]
                    )
                fixture_send_frame(host, reply)
                if scenario == "missing-reply-end":
                    host.shutdown(socket.SHUT_WR)
                    return
                reply_end = {
                    "schema": "tersh-host-transaction-reply-end-v1",
                    "transaction_nonce": nonce,
                    "operation": operation,
                    "reply_sha256": hashlib.sha256(fixture_canonical_json(reply)).hexdigest(),
                }
                if scenario == "wrong-reply-end-nonce":
                    reply_end["transaction_nonce"] = "e" * 64
                if scenario == "wrong-reply-end-schema":
                    reply_end["schema"] = "tersh-host-transaction-reply-end-v0"
                if scenario == "wrong-reply-end-operation":
                    reply_end["operation"] = (
                        "capture-response" if operation != "capture-response" else "capture-context"
                    )
                if scenario == "wrong-reply-end-digest":
                    reply_end["reply_sha256"] = "f" * 64
                if scenario == "extra-reply-end-key":
                    reply_end["extra"] = None
                fixture_send_frame(host, reply_end)
                if scenario == "duplicate-reply-end":
                    fixture_send_frame(host, reply_end)
                if scenario == "trailing":
                    host.sendall(b"trailing")
                if scenario == "late-eof":
                    time.sleep(1)
                host.shutdown(socket.SHUT_WR)
            except (OSError, EOFError) as error:
                if scenario == "success" and body_mutator is None:
                    host_errors.append(error)
            except BaseException as error:
                host_errors.append(error)
            finally:
                host.close()

        thread = threading.Thread(target=scripted_host)
        thread.start()
        result = None
        error = None
        try:
            result = capture(
                client,
                operation,
                context_handle,
                deadline=time.monotonic() + 0.7,
            )
        except BaseException as caught:
            error = caught
        finally:
            client.close()
            thread.join(timeout=2)
        self.assertFalse(thread.is_alive(), f"scripted host did not terminate for {scenario}")
        self.assertFalse(host_errors, f"scripted host errors for {scenario}: {host_errors!r}")
        return result, error, transcript

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

    def test_run_gate_delegates_bounded_drain_and_stream_record_to_evidence_core(self):
        tree = ast.parse(RUN_GATE.read_text(encoding="utf-8"), filename=str(RUN_GATE))
        direct_imports = {
            alias.name
            for node in ast.walk(tree)
            if isinstance(node, ast.Import)
            for alias in node.names
        }
        self.assertTrue(
            {"hashlib", "threading"}.isdisjoint(direct_imports),
            f"run_gate owns stream internals through imports: {direct_imports}",
        )
        self.assertFalse(
            any(
                isinstance(node, ast.Call)
                and isinstance(node.func, ast.Attribute)
                and node.func.attr == "Popen"
                for node in ast.walk(tree)
            ),
            "run_gate must not spawn the gate child itself",
        )
        definitions = {
            node.name
            for node in ast.walk(tree)
            if isinstance(node, (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef))
        }
        self.assertTrue(
            {"DrainedStream", "_drain", "run_and_drain", "_stream_record"}.isdisjoint(definitions),
            f"run_gate duplicates shared stream internals: {definitions}",
        )
        core_calls = {
            node.func.attr
            for node in ast.walk(tree)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and isinstance(node.func.value, ast.Name)
            and node.func.value.id == "core"
        }
        self.assertTrue({"run_and_drain", "gate_stream_record"}.issubset(core_calls), core_calls)

        core = importlib.import_module("scripts.evidence_core")
        run_and_drain = getattr(core, "run_and_drain", None)
        stream_record = getattr(core, "gate_stream_record", None)
        self.assertTrue(callable(run_and_drain), "evidence_core.run_and_drain is required")
        self.assertTrue(callable(stream_record), "evidence_core.gate_stream_record is required")
        exit_code, stdout, stderr = run_and_drain(
            [
                sys.executable,
                "-c",
                "import sys; sys.stdout.buffer.write(b'abc'); sys.stderr.buffer.write(b'defg'); raise SystemExit(9)",
            ],
            retain_limit=2,
        )
        self.assertEqual(exit_code, 9)
        for stream, complete in ((stdout, b"abc"), (stderr, b"defg")):
            self.assertEqual(stream.total_bytes, len(complete))
            self.assertEqual(stream.sha256, hashlib.sha256(complete).hexdigest())
            self.assertEqual(stream.retained, complete[:2])
            self.assertEqual(stream.retained_sha256, hashlib.sha256(complete[:2]).hexdigest())
        self.assertEqual(
            stream_record(stdout, "run-local", "delegated", "stdout"),
            {
                "total_bytes": 3,
                "sha256": hashlib.sha256(b"abc").hexdigest(),
                "retained_bytes": 2,
                "retained_sha256": hashlib.sha256(b"ab").hexdigest(),
                "retained_log": "run-local/gates/delegated.stdout",
            },
        )

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

        output_root = self.root / "collision-extra"
        base = self.gate_base(output_root=output_root, name="collision")
        base.parent.mkdir(parents=True)
        extra = pathlib.Path(f"{base}.extra")
        extra.write_bytes(b"immutable-extra")
        before = (extra.stat().st_ino, extra.read_bytes())
        marker = self.root / "child-extra.marker"
        result = self.run_gate(
            output_root=output_root,
            name="collision",
            child=self.marker_child(marker),
        )
        self.assert_child_not_run(result, marker)
        self.assertEqual((extra.stat().st_ino, extra.read_bytes()), before)
        for suffix in ("json", "stdout", "stderr"):
            self.assertFalse(pathlib.Path(f"{base}.{suffix}").exists())

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

    def test_publish_new_rejects_temp_path_replacement_before_hardlink(self):
        core = importlib.import_module("scripts.evidence_core")
        directory = self.root / "publish-temp-replacement"
        directory.mkdir(mode=0o700)
        directory_fd = os.open(directory, os.O_RDONLY | os.O_DIRECTORY)
        real_link = core.os.link
        swapped = False

        def replace_temp_then_link(
            source,
            destination,
            *,
            src_dir_fd=None,
            dst_dir_fd=None,
            follow_symlinks=True,
        ):
            nonlocal swapped
            swapped = True
            os.unlink(source, dir_fd=src_dir_fd)
            attacker_fd = os.open(
                source,
                os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                0o600,
                dir_fd=src_dir_fd,
            )
            try:
                os.write(attacker_fd, b"ATTACKER")
                os.fsync(attacker_fd)
            finally:
                os.close(attacker_fd)
            return real_link(
                source,
                destination,
                src_dir_fd=src_dir_fd,
                dst_dir_fd=dst_dir_fd,
                follow_symlinks=follow_symlinks,
            )

        try:
            with mock.patch.object(core.os, "link", side_effect=replace_temp_then_link):
                with self.assertRaises(Exception):
                    core.publish_new_at(directory_fd, "record.json", b"EXPECTED")
            self.assertTrue(swapped, "temp replacement hook was not reached")
            final = directory / "record.json"
            self.assertEqual(final.read_bytes(), b"ATTACKER")
            self.assertEqual([path.name for path in directory.iterdir()], ["record.json"])
        finally:
            os.close(directory_fd)

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

    def test_run_gate_rejects_namespace_entry_created_while_child_runs(self):
        output_root = self.root / "runtime-namespace"
        base = self.gate_base(output_root=output_root, name="runtime-entry")
        extra = pathlib.Path(f"{base}.extra")
        unrelated = base.parent / "other-gate.extra"
        child = [
            sys.executable,
            "-c",
            "import pathlib,sys; pathlib.Path(sys.argv[1]).write_bytes(b'ATTACKER')",
            str(extra),
        ]
        result = self.run_gate(output_root=output_root, name="runtime-entry", child=child)
        self.assertNotEqual(result.returncode, 0, result.stderr[:1024])
        self.assertEqual(extra.read_bytes(), b"ATTACKER")
        for suffix in ("json", "stdout", "stderr"):
            self.assertFalse(pathlib.Path(f"{base}.{suffix}").exists())
        self.assertFalse((base.parent / ".runtime-entry.reservation").exists())

        unrelated.parent.mkdir(parents=True, exist_ok=True)
        unrelated.write_bytes(b"OTHER-GATE")
        clean = self.run_gate(output_root=output_root, name="unaffected")
        self.assertEqual(clean.returncode, 0, clean.stderr[:1024])
        self.load_record(self.gate_base(output_root=output_root, name="unaffected"))
        self.assertEqual(unrelated.read_bytes(), b"OTHER-GATE")

    def test_run_gate_interrupt_kills_child_group_before_releasing_reservation(self):
        output_root = self.root / "interrupt-cleanup"
        base = self.gate_base(output_root=output_root, name="interrupt-cleanup")
        reservation = base.parent / ".interrupt-cleanup.reservation"
        pid_file = self.root / "interrupt-child.pid"
        term_marker = self.root / "interrupt-child.term"
        child_program = textwrap.dedent(
            """
            import os, pathlib, signal, sys, time
            pid_file, term_marker = map(pathlib.Path, sys.argv[1:])
            def ignore_term(signum, frame):
                term_marker.write_bytes(b"TERM")
            signal.signal(signal.SIGTERM, ignore_term)
            pid_file.write_text(str(os.getpid()), encoding="ascii")
            while True:
                time.sleep(1)
            """
        )
        command = [
            sys.executable,
            str(RUN_GATE),
            "--iteration", "impl-01",
            "--attempt", "001",
            "--run-binding", "run-local",
            "--name", "interrupt-cleanup",
            "--candidate", self.candidate_b,
            "--output-root", str(output_root),
            "--", sys.executable, "-c", child_program, str(pid_file), str(term_marker),
        ]
        env = os.environ.copy()
        env["PYTHONDONTWRITEBYTECODE"] = "1"
        runner = subprocess.Popen(
            command,
            cwd=self.repo,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        child_pid = None

        def process_exists(pid):
            try:
                os.kill(pid, 0)
            except ProcessLookupError:
                return False
            except PermissionError:
                return True
            return True

        try:
            deadline = time.monotonic() + 5
            while not pid_file.exists() and time.monotonic() < deadline:
                time.sleep(0.01)
            self.assertTrue(pid_file.exists(), "gate child never reached ready state")
            child_pid = int(pid_file.read_text(encoding="ascii"))
            self.assertTrue(reservation.is_file(), "reservation was absent before interrupt")

            interrupted_at = time.monotonic()
            os.kill(runner.pid, signal.SIGINT)
            term_deadline = time.monotonic() + 3
            while not term_marker.exists() and time.monotonic() < term_deadline:
                time.sleep(0.01)
            self.assertTrue(term_marker.exists(), "interrupt did not terminate the child process group")
            self.assertTrue(
                reservation.is_file(),
                "reservation was released before child-group cleanup completed",
            )

            output = runner.communicate(timeout=8)
            self.assertNotEqual(runner.returncode, 0, output)
            self.assertLess(time.monotonic() - interrupted_at, 6)
            gone_deadline = time.monotonic() + 2
            while process_exists(child_pid) and time.monotonic() < gone_deadline:
                time.sleep(0.01)
            self.assertFalse(process_exists(child_pid), f"child {child_pid} survived runner interrupt")
            self.assertFalse(reservation.exists())
            for suffix in ("json", "stdout", "stderr"):
                self.assertFalse(pathlib.Path(f"{base}.{suffix}").exists())
        finally:
            if runner.poll() is None:
                runner.kill()
                runner.communicate(timeout=5)
            if child_pid is not None and process_exists(child_pid):
                os.kill(child_pid, signal.SIGKILL)
                cleanup_deadline = time.monotonic() + 3
                while process_exists(child_pid) and time.monotonic() < cleanup_deadline:
                    time.sleep(0.01)

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

    def test_host_envelope_adapter_requires_distinct_peer_credential_unix_stream_socket(self):
        core = importlib.import_module("scripts.evidence_core")
        validator = getattr(core, "validate_host_peer_identity", None)
        mac_parser = getattr(core, "parse_macos_local_peercred", None)
        linux_parser = getattr(core, "parse_linux_so_peercred", None)
        opener = getattr(core, "open_authenticated_host_store_socket", None)
        self.assertTrue(callable(validator), "shared host peer validator is required")
        self.assertTrue(callable(mac_parser), "exact macOS xucred parser is required")
        self.assertTrue(callable(linux_parser), "exact Linux SO_PEERCRED parser is required")
        self.assertTrue(callable(opener), "production kernel-authenticated socket opener is required")

        self.assertEqual(
            mac_parser(struct.pack("=III16I", 0, 0, 1, 0, *([0] * 15))),
            (0, (0,)),
        )
        for raw in (
            b"\0" * 75,
            b"\0" * 77,
            struct.pack("=III16I", 1, 0, 1, 0, *([0] * 15)),
            struct.pack("=III16I", 0, 0, 17, *([0] * 16)),
        ):
            with self.subTest(mac_raw_len=len(raw), mac_raw=raw[:12]):
                with self.assertRaises(core.EvidenceError):
                    mac_parser(raw)

        self.assertEqual(linux_parser(struct.pack("=iii", 123, 0, 0)), (123, 0, 0))
        for raw in (
            b"\0" * 11,
            b"\0" * 13,
            struct.pack("=iii", -1, 0, 0),
            struct.pack("=iii", 123, -1, 0),
            struct.pack("=iii", 123, 0, -1),
        ):
            with self.subTest(linux_raw_len=len(raw), linux_raw=raw):
                with self.assertRaises(core.EvidenceError):
                    linux_parser(raw)

        self.assertIsNone(validator(0, 0, 501))
        rejected_identities = [
            (uid, uid, 502) for uid in (1, 2, 501, 65534)
        ] + [
            (uid, 0, 502) for uid in (1, 2, 501, 65534)
        ] + [
            (0, uid, 502) for uid in (1, 2, 501, 65534)
        ] + [
            (0, 0, 0),
            (True, 0, 501),
            (0, False, 501),
            (0, 0, True),
        ]
        for peer_uid, owner_uid, client_euid in rejected_identities:
            with self.subTest(
                peer_uid=peer_uid,
                owner_uid=owner_uid,
                client_euid=client_euid,
            ):
                with self.assertRaises(core.EvidenceError):
                    validator(peer_uid, owner_uid, client_euid)

        client, peer = socket.socketpair(socket.AF_UNIX, socket.SOCK_STREAM)
        peer.settimeout(1)
        try:
            with self.assertRaises(core.EvidenceError):
                opener(client.fileno())
            client.close()
            self.assertEqual(peer.recv(1), b"", "failed production authentication sent bytes")
        finally:
            client.close()
            peer.close()

    def test_host_envelope_adapter_rejects_same_principal_fifo_stdin_regular_file_trailing_bytes_and_reuse(self):
        adapter = ROOT / "scripts" / "implementation_evidence" / "host_envelope_adapter.py"
        self.assertTrue(adapter.is_file(), "host envelope adapter entrypoint is required")
        isolated_adapter_argv = [sys.executable, "-I", "-S", "-B", str(adapter)]
        hostile_root = self.root / "hostile-pythonpath"
        hostile_scripts = hostile_root / "scripts"
        hostile_scripts.mkdir(parents=True)
        hostile_marker = self.root / "hostile-core-imported.marker"
        hostile_site_marker = self.root / "hostile-sitecustomize.marker"
        (hostile_root / "sitecustomize.py").write_text(
            textwrap.dedent(
                """
                import os
                import pathlib

                pathlib.Path(os.environ["TERSH_HOSTILE_SITE_MARKER"]).write_text(
                    "hostile-sitecustomize", encoding="utf-8"
                )
                """
            ),
            encoding="utf-8",
        )
        (hostile_scripts / "__init__.py").write_text("", encoding="utf-8")
        (hostile_scripts / "evidence_core.py").write_text(
            textwrap.dedent(
                """
                import os
                import pathlib

                pathlib.Path(os.environ["TERSH_HOSTILE_IMPORT_MARKER"]).write_text(
                    "hostile-core-imported", encoding="utf-8"
                )

                class EvidenceError(Exception):
                    pass
                """
            ),
            encoding="utf-8",
        )
        hostile_environment = os.environ.copy()
        hostile_environment["PYTHONPATH"] = os.pathsep.join(
            filter(
                None,
                (
                    str(hostile_scripts.parent),
                    hostile_environment.get("PYTHONPATH", ""),
                ),
            )
        )
        hostile_environment["TERSH_HOSTILE_IMPORT_MARKER"] = str(hostile_marker)
        hostile_environment["TERSH_HOSTILE_SITE_MARKER"] = str(hostile_site_marker)
        hostile_probe = subprocess.run(
            [sys.executable, "-c", "pass"],
            env=hostile_environment,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=5,
        )
        self.assertEqual(hostile_probe.returncode, 0)
        self.assertTrue(
            hostile_site_marker.is_file(),
            "hostile sitecustomize fixture did not execute without isolation",
        )
        hostile_site_marker.unlink()
        hostile_help = subprocess.run(
            [*isolated_adapter_argv, "--help"],
            env=hostile_environment,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=5,
        )
        self.assertEqual(hostile_help.returncode, 0)
        self.assertFalse(
            hostile_marker.exists(),
            "adapter imported an attacker-controlled scripts.evidence_core",
        )
        self.assertFalse(
            hostile_site_marker.exists(),
            "isolated adapter executed attacker-controlled sitecustomize",
        )
        self.assertIn(b"capture-context", hostile_help.stdout)
        self.assertNotIn(b"Traceback", hostile_help.stderr)

        adapter_source = adapter.read_text(encoding="utf-8")
        adapter_tree = ast.parse(adapter_source, filename=str(adapter))
        core_call_names = {
            node.func.attr
            for node in ast.walk(adapter_tree)
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
        }
        self.assertIn("open_authenticated_host_store_socket", core_call_names)
        self.assertIn("capture_host_envelope_on_authenticated_socket", core_call_names)
        main_guards = [
            node
            for node in ast.walk(adapter_tree)
            if isinstance(node, ast.If)
            and isinstance(node.test, ast.Compare)
            and isinstance(node.test.left, ast.Name)
            and node.test.left.id == "__name__"
            and any(
                isinstance(comparator, ast.Constant) and comparator.value == "__main__"
                for comparator in node.test.comparators
            )
        ]
        self.assertEqual(len(main_guards), 1, "adapter requires one executable __main__ guard")
        guarded_calls = {
            node.func.id
            for statement in main_guards[0].body
            for node in ast.walk(statement)
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
        }
        self.assertIn(
            "_isolated_cli_main",
            guarded_calls,
            "adapter __main__ guard must execute the isolated CLI guard",
        )
        for forbidden in (
            "expected-uid",
            "expected_uid",
            "test-mode",
            "test_mode",
            "allow-same-principal",
            "HOST_STORE_PATH",
        ):
            self.assertNotIn(forbidden, adapter_source)

        adapter_module = importlib.import_module(
            "scripts.implementation_evidence.host_envelope_adapter"
        )
        self.assertIs(
            getattr(adapter_module, "core", None),
            importlib.import_module("scripts.evidence_core"),
            "adapter must call the shared evidence_core module",
        )
        unisolated_help = subprocess.run(
            [sys.executable, str(adapter), "--help"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=5,
        )
        self.assertNotEqual(unisolated_help.returncode, 0)
        self.assertEqual(unisolated_help.stdout, b"")
        self.assertGreater(len(unisolated_help.stderr), 0)
        self.assertLessEqual(len(unisolated_help.stderr), 8192)
        self.assertNotIn(b"Traceback", unisolated_help.stderr)

        help_result = subprocess.run(
            [*isolated_adapter_argv, "--help"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=5,
        )
        self.assertEqual(help_result.returncode, 0)
        self.assertNotIn(b"Traceback", help_result.stderr)
        for token in (b"capture-context", b"capture-invocation", b"capture-response"):
            self.assertIn(token, help_result.stdout)
        for operation in ("capture-context", "capture-invocation", "capture-response"):
            with self.subTest(help_operation=operation):
                operation_help = subprocess.run(
                    [*isolated_adapter_argv, operation, "--help"],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    check=False,
                    timeout=5,
                )
                self.assertEqual(operation_help.returncode, 0)
                self.assertNotIn(b"Traceback", operation_help.stderr)
                self.assertIn(b"--host-store-fd", operation_help.stdout)
                if operation == "capture-context":
                    self.assertNotIn(b"--context-handle", operation_help.stdout)
                else:
                    self.assertIn(b"--context-handle", operation_help.stdout)

        context_handle = "a" * 64
        operation_results = {
            "capture-context": {
                "schema": "tersh-host-capture-context-result-v1",
                "context_handle": "1" * 64,
            },
            "capture-invocation": {
                "schema": "tersh-host-capture-invocation-result-v1",
                "context_handle": "2" * 64,
                "invocation_handle": "3" * 64,
            },
            "capture-response": {
                "schema": "tersh-host-capture-response-result-v1",
                "context_handle": "4" * 64,
                "response_handle": "5" * 64,
            },
        }
        for operation, fake_result in operation_results.items():
            with self.subTest(adapter_operation=operation):
                fake_socket = mock.MagicMock(name=f"{operation}-authenticated-socket")
                fake_socket.__enter__.return_value = fake_socket
                stdout = io.StringIO()
                stderr = io.StringIO()
                argv = [operation, "--host-store-fd", "9"]
                expected_context_handle = None
                if operation != "capture-context":
                    argv.extend(("--context-handle", context_handle))
                    expected_context_handle = context_handle
                with (
                    mock.patch.object(
                        adapter_module.core,
                        "open_authenticated_host_store_socket",
                        return_value=fake_socket,
                    ) as authenticated_open,
                    mock.patch.object(
                        adapter_module.core,
                        "capture_host_envelope_on_authenticated_socket",
                        return_value=fake_result,
                    ) as capture,
                    mock.patch("sys.stdout", stdout),
                    mock.patch("sys.stderr", stderr),
                ):
                    status = adapter_module.main(argv)
                self.assertEqual(status, 0)
                authenticated_open.assert_called_once_with(9)
                capture.assert_called_once()
                capture_args = capture.call_args.args
                capture_kwargs = capture.call_args.kwargs
                self.assertLessEqual(
                    len(capture_args),
                    3,
                    "adapter passed undeclared positional capture overrides",
                )
                self.assertLessEqual(
                    set(capture_kwargs),
                    {"sock", "operation", "context_handle", "deadline"},
                    "adapter passed undeclared keyword capture overrides",
                )
                for index, name in enumerate(("sock", "operation", "context_handle")):
                    if len(capture_args) > index:
                        self.assertNotIn(
                            name,
                            capture_kwargs,
                            f"adapter passed duplicate capture parameter {name}",
                        )
                actual_socket = (
                    capture_args[0] if len(capture_args) > 0 else capture_kwargs.get("sock")
                )
                actual_operation = (
                    capture_args[1]
                    if len(capture_args) > 1
                    else capture_kwargs.get("operation")
                )
                actual_context_handle = (
                    capture_args[2]
                    if len(capture_args) > 2
                    else capture_kwargs.get("context_handle")
                )
                self.assertIs(actual_socket, fake_socket)
                self.assertEqual(actual_operation, operation)
                self.assertEqual(actual_context_handle, expected_context_handle)
                self.assertTrue(
                    fake_socket.close.called or fake_socket.__exit__.called,
                    "adapter must release its authenticated socket",
                )
                self.assertEqual(
                    stdout.getvalue().encode("utf-8"),
                    fixture_canonical_json(fake_result),
                )
                self.assertEqual(stderr.getvalue(), "")

        failed_socket = mock.MagicMock(name="failed-capture-authenticated-socket")
        failed_stdout = io.StringIO()
        failed_stderr = io.StringIO()
        with (
            mock.patch.object(
                adapter_module.core,
                "open_authenticated_host_store_socket",
                return_value=failed_socket,
            ) as failed_open,
            mock.patch.object(
                adapter_module.core,
                "capture_host_envelope_on_authenticated_socket",
                side_effect=adapter_module.core.EvidenceError(
                    "simulated capture failure"
                ),
            ) as failed_capture,
            mock.patch("sys.stdout", failed_stdout),
            mock.patch("sys.stderr", failed_stderr),
        ):
            failed_status = adapter_module.main(
                ["capture-context", "--host-store-fd", "9"]
            )
        self.assertNotEqual(failed_status, 0)
        failed_open.assert_called_once_with(9)
        failed_capture.assert_called_once()
        self.assertEqual(failed_socket.close.call_count, 1)
        self.assertEqual(failed_stdout.getvalue(), "")
        self.assertGreater(len(failed_stderr.getvalue()), 0)
        self.assertLessEqual(len(failed_stderr.getvalue().encode("utf-8")), 8192)
        self.assertNotIn("Traceback", failed_stderr.getvalue())

        oversized_fd = "9" * 1000
        invalid_argvs = (
            ("capture-context",),
            ("capture-context", "--host-store-fd"),
            ("capture-context", "--host-store-fd", oversized_fd),
            (
                "capture-invocation",
                "--host-store-fd",
                "9",
                "--context-handle",
                "not-a-handle",
            ),
            ("capture-context", "--host-store-fd", "9", "--body", "{}"),
            (
                "capture-context",
                "--host-store-fd",
                "9",
                "--transaction-nonce",
                "b" * 64,
            ),
            ("capture-context", "--host-store-fd", "9", "--model", "other"),
            ("capture-context", "--host-store-fd", "9", "--identity", "caller"),
            (
                "capture-context",
                "--host-store-fd",
                "9",
                "--host-store-fd",
                "10",
            ),
            (
                "capture-invocation",
                "--host-store-fd",
                "9",
                "--context-handle",
                context_handle,
                "--context-handle",
                "b" * 64,
            ),
            (
                "capture-context",
                "--host-store-fd",
                "9",
                "--context-handle",
                context_handle,
            ),
            ("capture-invocation", "--host-store-fd", "9"),
            ("capture-response", "--host-store-fd", "9"),
        )
        for invalid_argv in invalid_argvs:
            with self.subTest(invalid_argv=invalid_argv):
                invalid_stdout = io.StringIO()
                invalid_stderr = io.StringIO()
                with (
                    mock.patch.object(
                        adapter_module.core,
                        "open_authenticated_host_store_socket",
                        side_effect=adapter_module.core.EvidenceError(
                            "invalid argv reached authenticated opener"
                        ),
                    ) as authenticated_open,
                    mock.patch.object(
                        adapter_module.core,
                        "capture_host_envelope_on_authenticated_socket",
                    ) as capture,
                    mock.patch("sys.stdout", invalid_stdout),
                    mock.patch("sys.stderr", invalid_stderr),
                ):
                    invalid_status = adapter_module.main(list(invalid_argv))
                self.assertNotEqual(invalid_status, 0)
                authenticated_open.assert_not_called()
                capture.assert_not_called()
                self.assertEqual(invalid_stdout.getvalue(), "")
                self.assertGreater(len(invalid_stderr.getvalue()), 0)
                self.assertLessEqual(len(invalid_stderr.getvalue()), 8192)
                self.assertNotIn("Traceback", invalid_stderr.getvalue())

        oversized_fd_cli = subprocess.run(
            [
                *isolated_adapter_argv,
                "capture-context",
                "--host-store-fd",
                oversized_fd,
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=5,
        )
        self.assertNotEqual(oversized_fd_cli.returncode, 0)
        self.assertEqual(oversized_fd_cli.stdout, b"")
        self.assertGreater(len(oversized_fd_cli.stderr), 0)
        self.assertLessEqual(len(oversized_fd_cli.stderr), 8192)
        self.assertNotIn(b"Traceback", oversized_fd_cli.stderr)

        multibyte_unknown_cli = subprocess.run(
            [
                *isolated_adapter_argv,
                "capture-context",
                "--host-store-fd",
                "9",
                "--" + "😀" * 5000,
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=5,
        )
        self.assertNotEqual(multibyte_unknown_cli.returncode, 0)
        self.assertEqual(multibyte_unknown_cli.stdout, b"")
        self.assertGreater(len(multibyte_unknown_cli.stderr), 0)
        self.assertLessEqual(len(multibyte_unknown_cli.stderr), 8192)
        self.assertNotIn(b"Traceback", multibyte_unknown_cli.stderr)

        def run_with_fd(fd):
            return subprocess.run(
                [
                    *isolated_adapter_argv,
                    "capture-context",
                    "--host-store-fd",
                    str(fd),
                ],
                pass_fds=() if fd == 0 else (fd,),
                stdin=subprocess.DEVNULL if fd != 0 else None,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
                timeout=5,
            )

        client, peer = socket.socketpair(socket.AF_UNIX, socket.SOCK_STREAM)
        self.addCleanup(client.close)
        self.addCleanup(peer.close)
        peer.settimeout(1)
        same_uid = run_with_fd(client.fileno())
        self.assertNotEqual(same_uid.returncode, 0)
        self.assertEqual(same_uid.stdout, b"")
        self.assertGreater(len(same_uid.stderr), 0)
        self.assertLessEqual(len(same_uid.stderr), 8192)
        self.assertNotIn(b"Traceback", same_uid.stderr)
        client.close()
        self.assertEqual(peer.recv(1), b"", "rejected same-UID peer received protocol bytes")

        regular_path = self.root / "host-store.regular"
        regular_fd = os.open(regular_path, os.O_RDWR | os.O_CREAT | os.O_EXCL, 0o600)
        try:
            regular = run_with_fd(regular_fd)
        finally:
            os.close(regular_fd)
        self.assertNotEqual(regular.returncode, 0)
        self.assertEqual(regular.stdout, b"")
        self.assertGreater(len(regular.stderr), 0)
        self.assertNotIn(b"Traceback", regular.stderr)
        self.assertEqual(regular_path.stat().st_size, 0)

        fifo_path = self.root / "host-store.fifo"
        os.mkfifo(fifo_path, 0o600)
        fifo_fd = os.open(fifo_path, os.O_RDWR | os.O_NONBLOCK)
        try:
            fifo = run_with_fd(fifo_fd)
        finally:
            os.close(fifo_fd)
        self.assertNotEqual(fifo.returncode, 0)
        self.assertEqual(fifo.stdout, b"")
        self.assertGreater(len(fifo.stderr), 0)
        self.assertNotIn(b"Traceback", fifo.stderr)

        stdin_result = run_with_fd(0)
        self.assertNotEqual(stdin_result.returncode, 0)
        self.assertEqual(stdin_result.stdout, b"")
        self.assertGreater(len(stdin_result.stderr), 0)
        self.assertNotIn(b"Traceback", stdin_result.stderr)

        core = importlib.import_module("scripts.evidence_core")
        send_frame = getattr(core, "send_host_frame", None)
        recv_frame = getattr(core, "recv_host_frame", None)
        require_eof = getattr(core, "require_host_eof", None)
        self.assertTrue(callable(send_frame), "shared bounded frame writer is required")
        self.assertTrue(callable(recv_frame), "shared bounded frame reader is required")
        self.assertTrue(callable(require_eof), "shared exact EOF verifier is required")

        sender, receiver = socket.socketpair(socket.AF_UNIX, socket.SOCK_STREAM)
        self.addCleanup(sender.close)
        self.addCleanup(receiver.close)
        deadline = time.monotonic() + 2
        send_frame(sender, {"schema": "tersh-host-test-frame-v1"}, deadline)
        sender.sendall(b"trailing")
        sender.shutdown(socket.SHUT_WR)
        self.assertEqual(
            recv_frame(receiver, deadline),
            {"schema": "tersh-host-test-frame-v1"},
        )
        with self.assertRaises(core.EvidenceError):
            require_eof(receiver, deadline)

    def test_host_transaction_old_early_half_close_sequence_reproduces_epipe(self):
        host, adapter = socket.socketpair(socket.AF_UNIX, socket.SOCK_STREAM)
        self.addCleanup(host.close)
        self.addCleanup(adapter.close)
        body = b"obsolete-host-body"
        request = b"late-create-request"
        host.sendall(body)
        host.shutdown(socket.SHUT_WR)
        self.assertEqual(adapter.recv(len(body)), body)
        self.assertEqual(adapter.recv(1), b"")
        adapter.sendall(request)
        self.assertEqual(host.recv(len(request)), request)
        with self.assertRaises(BrokenPipeError):
            host.sendall(b"impossible-reply")

        core = importlib.import_module("scripts.evidence_core")
        capture = getattr(core, "capture_host_envelope_on_authenticated_socket", None)
        self.assertTrue(
            callable(capture),
            "new request/commit-first capture transaction seam is required",
        )

    def test_host_transaction_rejects_frame_order_end_trailing_nonce_digest_and_reply_before_commit(self):
        core = importlib.import_module("scripts.evidence_core")
        pre_error = core.EvidenceError
        post_error = core.EvidenceError
        recv_frame = getattr(core, "recv_host_frame", None)
        self.assertTrue(callable(recv_frame), "bounded host frame reader is required")

        malformed_frames = (
            struct.pack(">I", 0),
            struct.pack(">I", 65537),
            b"\x00\x00",
            struct.pack(">I", 8) + b"{}\n",
            struct.pack(">I", 14) + b'{"a":1,"a":2}\n',
            struct.pack(">I", 4) + b"\xff\xff\xff\n",
            struct.pack(">I", 14) + b'{"b":1,"a":2}\n',
            struct.pack(">I", 10) + b'{"a":NaN}\n',
        )
        for index, raw in enumerate(malformed_frames):
            with self.subTest(raw_frame=index):
                sender, receiver = socket.socketpair(socket.AF_UNIX, socket.SOCK_STREAM)
                try:
                    sender.sendall(raw)
                    sender.shutdown(socket.SHUT_WR)
                    with self.assertRaises(core.EvidenceError):
                        recv_frame(receiver, time.monotonic() + 0.5)
                finally:
                    sender.close()
                    receiver.close()

        success_store = ScriptedCaptureStore()
        result, error, transcript_a = self.run_scripted_capture(success_store, "capture-context")
        self.assertIsNone(error)
        self.assertEqual(set(result), {"schema", "context_handle"})
        self.assertEqual(result["schema"], "tersh-host-capture-context-result-v1")
        self.assertIn(result["context_handle"], success_store.contexts)
        second_store = ScriptedCaptureStore()
        second_result, second_error, transcript_b = self.run_scripted_capture(
            second_store,
            "capture-context",
        )
        self.assertIsNone(second_error)
        self.assertIn(second_result["context_handle"], second_store.contexts)
        self.assertNotEqual(
            transcript_a[0][1]["transaction_nonce"],
            transcript_b[0][1]["transaction_nonce"],
            "transaction nonces must be fresh per connection",
        )

        for scenario in (
            "wrong-body-nonce",
            "wrong-body-digest",
            "bool-body-ordinal",
            "wrong-body-schema",
            "wrong-body-operation",
            "wrong-body-kind",
            "wrong-body-total",
            "bool-body-total",
            "extra-body-key",
            "body-end-before-body",
            "missing-body-end",
            "wrong-body-end-nonce",
            "wrong-body-end-schema",
            "wrong-body-end-operation",
            "wrong-body-end-total",
            "bool-body-end-total",
            "wrong-body-end-digests",
            "extra-body-end-key",
            "reply-before-commit",
        ):
            with self.subTest(precommit=scenario):
                store = ScriptedCaptureStore()
                before = copy.deepcopy(store.contexts)
                result, error, _ = self.run_scripted_capture(
                    store,
                    "capture-context",
                    scenario=scenario,
                )
                self.assertIsNone(result)
                self.assertIsInstance(error, pre_error)
                self.assertEqual(store.contexts, before)

        for scenario in ("duplicate-body", "reordered-bodies", "extra-body"):
            with self.subTest(multi_body_precommit=scenario):
                store = ScriptedCaptureStore()
                context, error, _ = self.run_scripted_capture(store, "capture-context")
                self.assertIsNone(error)
                h0 = context["context_handle"]
                before = (copy.deepcopy(store.contexts), set(store.invocations))
                result, error, _ = self.run_scripted_capture(
                    store,
                    "capture-invocation",
                    h0,
                    scenario=scenario,
                )
                self.assertIsNone(result)
                self.assertIsInstance(error, pre_error)
                self.assertEqual((store.contexts, store.invocations), before)

        for scenario in (
            "wrong-reply-nonce",
            "wrong-reply-digest",
            "wrong-reply-schema",
            "wrong-reply-operation",
            "extra-reply-key",
            "missing-reply-end",
            "duplicate-body-end",
            "duplicate-reply-end",
            "wrong-reply-end-nonce",
            "wrong-reply-end-schema",
            "wrong-reply-end-operation",
            "wrong-reply-end-digest",
            "extra-reply-end-key",
            "trailing",
            "late-eof",
        ):
            with self.subTest(postcommit=scenario):
                store = ScriptedCaptureStore()
                result, error, _ = self.run_scripted_capture(
                    store,
                    "capture-context",
                    scenario=scenario,
                )
                self.assertIsNone(result)
                self.assertIsInstance(error, post_error)
                self.assertEqual(len(store.contexts), 1, "post-COMMIT successor was not atomic")

        for operation in ("capture-context", "capture-invocation", "capture-response"):
            scenarios = [
                "wrong-result-schema",
                "extra-result-key",
                "bad-result-context-handle",
            ]
            if operation != "capture-context":
                scenarios.extend(
                    (
                        "bad-result-member-handle",
                        "reused-predecessor-context-handle",
                        "member-aliases-predecessor",
                        "member-aliases-successor",
                    )
                )
            for scenario in scenarios:
                with self.subTest(result_operation=operation, result_attack=scenario):
                    store = ScriptedCaptureStore()
                    context_handle = None
                    if operation != "capture-context":
                        context_result, context_error, _ = self.run_scripted_capture(
                            store,
                            "capture-context",
                        )
                        self.assertIsNone(context_error)
                        context_handle = context_result["context_handle"]
                    result, error, _ = self.run_scripted_capture(
                        store,
                        operation,
                        context_handle,
                        scenario=scenario,
                    )
                    self.assertIsNone(result)
                    self.assertIsInstance(error, post_error)
                    self.assertEqual(
                        len(store.contexts),
                        1,
                        "malformed result did not preserve one atomic private successor",
                    )
                    if context_handle is not None:
                        self.assertNotIn(context_handle, store.contexts)
                    if operation == "capture-invocation":
                        self.assertEqual(len(store.invocations), 1)
                    if operation == "capture-response":
                        self.assertEqual(len(store.responses), 1)

    def test_context_capability_rotates_and_rejects_replay_cross_generation_or_partial_consume(self):
        core = importlib.import_module("scripts.evidence_core")
        pre_error = core.EvidenceError
        post_error = core.EvidenceError
        store = ScriptedCaptureStore()

        context_result, error, _ = self.run_scripted_capture(store, "capture-context")
        self.assertIsNone(error)
        h0 = context_result["context_handle"]
        invocation_result, error, _ = self.run_scripted_capture(
            store,
            "capture-invocation",
            h0,
        )
        self.assertIsNone(error)
        self.assertEqual(
            set(invocation_result),
            {"schema", "context_handle", "invocation_handle"},
        )
        self.assertEqual(
            invocation_result["schema"],
            "tersh-host-capture-invocation-result-v1",
        )
        h1 = invocation_result["context_handle"]
        hi = invocation_result["invocation_handle"]
        self.assertNotIn(h0, store.contexts)
        self.assertIn(h1, store.contexts)
        self.assertIn(hi, store.invocations)

        for operation in ("capture-invocation", "capture-response"):
            with self.subTest(replayed_operation=operation):
                before = (
                    copy.deepcopy(store.contexts),
                    set(store.invocations),
                    copy.deepcopy(store.responses),
                )
                result, replay_error, _ = self.run_scripted_capture(store, operation, h0)
                self.assertIsNone(result)
                self.assertIsInstance(replay_error, pre_error)
                self.assertEqual(
                    (store.contexts, store.invocations, store.responses),
                    before,
                    "replay or cross-generation failure partially consumed state",
                )

        response_result, error, _ = self.run_scripted_capture(
            store,
            "capture-response",
            h1,
        )
        self.assertIsNone(error)
        h2 = response_result["context_handle"]
        hr = response_result["response_handle"]
        self.assertEqual(
            response_result["schema"],
            "tersh-host-capture-response-result-v1",
        )
        self.assertNotIn(h1, store.contexts)
        self.assertIn(h2, store.contexts)
        self.assertIn(hr, store.responses)

        before_precommit = (
            copy.deepcopy(store.contexts),
            set(store.invocations),
            copy.deepcopy(store.responses),
        )
        result, error, _ = self.run_scripted_capture(
            store,
            "capture-response",
            h2,
            scenario="wrong-body-digest",
        )
        self.assertIsNone(result)
        self.assertIsInstance(error, pre_error)
        self.assertEqual((store.contexts, store.invocations, store.responses), before_precommit)

        orphan_context, error, _ = self.run_scripted_capture(store, "capture-context")
        self.assertIsNone(error)
        orphan_h0 = orphan_context["context_handle"]
        contexts_before = set(store.contexts)
        invocation_count = len(store.invocations)
        result, error, _ = self.run_scripted_capture(
            store,
            "capture-invocation",
            orphan_h0,
            scenario="wrong-reply-nonce",
        )
        self.assertIsNone(result)
        self.assertIsInstance(error, post_error)
        self.assertNotIn(orphan_h0, store.contexts)
        self.assertEqual(len(store.contexts), len(contexts_before))
        self.assertEqual(len(store.invocations), invocation_count + 1)

        result, replay_error, _ = self.run_scripted_capture(
            store,
            "capture-invocation",
            orphan_h0,
        )
        self.assertIsNone(result)
        self.assertIsInstance(replay_error, pre_error)

    def test_context_invocation_response_envelope_schemas_are_closed_typed_and_nonce_bound(self):
        core = importlib.import_module("scripts.evidence_core")
        pre_error = core.EvidenceError
        store = ScriptedCaptureStore()
        context_result, error, _ = self.run_scripted_capture(store, "capture-context")
        self.assertIsNone(error)
        h0 = context_result["context_handle"]

        def extra_context_key(bodies):
            bodies[0][1]["caller_override"] = True
            return bodies

        def wrong_context_schema(bodies):
            bodies[0][1]["schema"] = "tersh-host-dispatch-context-v0"
            return bodies

        def wrong_context_type(bodies):
            bodies[0][1]["evidence_attempt"] = True
            return bodies

        def invocation_nonce_mismatch(bodies):
            bodies[1][1]["context_nonce"] = "e" * 64
            return bodies

        def wrong_invocation_schema(bodies):
            bodies[1][1]["schema"] = "tersh-host-spawn-invocation-v0"
            return bodies

        def wrong_invocation_type(bodies):
            bodies[1][1]["dispatched_at"] = True
            return bodies

        def wrong_selected_model(bodies):
            bodies[1][1]["selected_model"] = "untrusted-model"
            return bodies

        def extra_invocation_key(bodies):
            bodies[1][1]["caller_override"] = True
            return bodies

        def response_task_mismatch(bodies):
            bodies[1][1]["canonical_task_path"] = "/root/other/reviewer"
            return bodies

        def response_nonce_mismatch(bodies):
            bodies[1][1]["context_nonce"] = "e" * 64
            return bodies

        def wrong_response_schema(bodies):
            bodies[1][1]["schema"] = "tersh-host-spawn-response-v1"
            return bodies

        def wrong_response_type(bodies):
            bodies[1][1]["terminal_status"] = True
            return bodies

        def response_time_reversed(bodies):
            bodies[1][1]["ended_at"] = "2026-08-09T23:59:59.000000001Z"
            return bodies

        def extra_response_key(bodies):
            bodies[1][1]["caller_override"] = True
            return bodies

        for operation, mutator in (
            ("capture-invocation", extra_context_key),
            ("capture-invocation", wrong_context_schema),
            ("capture-invocation", wrong_context_type),
            ("capture-invocation", invocation_nonce_mismatch),
            ("capture-invocation", wrong_invocation_schema),
            ("capture-invocation", wrong_invocation_type),
            ("capture-invocation", wrong_selected_model),
            ("capture-invocation", extra_invocation_key),
            ("capture-response", response_task_mismatch),
            ("capture-response", response_nonce_mismatch),
            ("capture-response", wrong_response_schema),
            ("capture-response", wrong_response_type),
            ("capture-response", response_time_reversed),
            ("capture-response", extra_response_key),
        ):
            with self.subTest(operation=operation, mutator=mutator.__name__):
                before = copy.deepcopy(store.contexts)
                result, validation_error, _ = self.run_scripted_capture(
                    store,
                    operation,
                    h0,
                    body_mutator=mutator,
                )
                self.assertIsNone(result)
                self.assertIsInstance(validation_error, pre_error)
                self.assertEqual(store.contexts, before)

        invocation_result, error, _ = self.run_scripted_capture(
            store,
            "capture-invocation",
            h0,
        )
        self.assertIsNone(error)
        h1 = invocation_result["context_handle"]
        response_result, error, _ = self.run_scripted_capture(
            store,
            "capture-response",
            h1,
        )
        self.assertIsNone(error)
        self.assertEqual(
            set(response_result),
            {"schema", "context_handle", "response_handle"},
        )
        self.assertEqual(
            response_result["schema"],
            "tersh-host-capture-response-result-v1",
        )
        self.assertIn(response_result["context_handle"], store.contexts)
        self.assertIn(response_result["response_handle"], store.responses)

    def test_capture_response_v2_accepts_null_or_exact_report_digest(self):
        for reported_record_sha256 in (None, "a" * 64):
            with self.subTest(reported_record_sha256=reported_record_sha256):
                store = ScriptedCaptureStore()
                context_result, error, _ = self.run_scripted_capture(store, "capture-context")
                self.assertIsNone(error)
                predecessor = context_result["context_handle"]

                def set_reported_record_sha256(bodies):
                    bodies[1][1]["reported_record_sha256"] = reported_record_sha256
                    return bodies

                response_result, error, _ = self.run_scripted_capture(
                    store,
                    "capture-response",
                    predecessor,
                    body_mutator=set_reported_record_sha256,
                )
                self.assertIsNone(error)
                self.assertNotIn(predecessor, store.contexts)
                self.assertIn(response_result["context_handle"], store.contexts)
                response_handle = response_result["response_handle"]
                expected_response = self.host_envelope_bodies()[2]
                expected_response["reported_record_sha256"] = reported_record_sha256
                self.assertEqual(store.responses[response_handle], expected_response)

    def test_scripted_capture_surfaces_host_connection_reset_before_commit(self):
        store = ScriptedCaptureStore()
        context_result, error, _ = self.run_scripted_capture(store, "capture-context")
        self.assertIsNone(error)
        predecessor = context_result["context_handle"]
        original_recv_frame = fixture_recv_frame
        host_recv_count = 0

        def inject_host_connection_reset(sock):
            nonlocal host_recv_count
            host_recv_count += 1
            if host_recv_count == 2:
                raise ConnectionResetError("injected Host reset before COMMIT")
            return original_recv_frame(sock)

        with mock.patch.object(
            sys.modules[__name__],
            "fixture_recv_frame",
            new=inject_host_connection_reset,
        ):
            with self.assertRaisesRegex(
                AssertionError,
                r"scripted host errors.*injected Host reset before COMMIT",
            ):
                self.run_scripted_capture(
                    store,
                    "capture-response",
                    predecessor,
                )
        self.assertEqual(host_recv_count, 2)

    def test_capture_response_v2_rejects_legacy_missing_malformed_or_extra_report_digest_without_consuming_context(self):
        core = importlib.import_module("scripts.evidence_core")

        def mutate_response(name, value=None):
            def mutator(bodies):
                response = bodies[1][1]
                if name == "missing":
                    response.pop("reported_record_sha256")
                elif name == "extra":
                    response["reported_record_sha256_alias"] = response[
                        "reported_record_sha256"
                    ]
                elif name == "legacy-v1-exact":
                    response["schema"] = "tersh-host-spawn-response-v1"
                    response.pop("reported_record_sha256")
                elif name == "legacy-v1-with-v2-field":
                    response["schema"] = "tersh-host-spawn-response-v1"
                else:
                    response["reported_record_sha256"] = value
                return bodies

            return mutator

        invalid_cases = (
            ("missing", None),
            ("extra", None),
            ("legacy-v1-exact", None),
            ("legacy-v1-with-v2-field", None),
            ("integer", 1),
            ("bool", True),
            ("list", []),
            ("dict", {}),
            ("empty", ""),
            ("uppercase", "A" * 64),
            ("lowercase-nonhex", "g" * 64),
            ("short", "a" * 63),
            ("overlong", "a" * 65),
        )
        for name, value in invalid_cases:
            with self.subTest(invalid_report_digest=name):
                store = ScriptedCaptureStore()
                context_result, error, _ = self.run_scripted_capture(
                    store,
                    "capture-context",
                )
                self.assertIsNone(error)
                predecessor = context_result["context_handle"]
                before = (
                    copy.deepcopy(store.contexts),
                    set(store.invocations),
                    copy.deepcopy(store.responses),
                )
                result, validation_error, _ = self.run_scripted_capture(
                    store,
                    "capture-response",
                    predecessor,
                    body_mutator=mutate_response(name, value),
                )
                self.assertIsNone(result)
                self.assertIsInstance(validation_error, core.EvidenceError)
                self.assertEqual(
                    (store.contexts, store.invocations, store.responses),
                    before,
                    f"invalid {name} report digest partially consumed the context",
                )

    def test_capture_response_v2_postcommit_reply_failure_consumes_context_and_retains_immutable_response(self):
        core = importlib.import_module("scripts.evidence_core")
        store = ScriptedCaptureStore()
        context_result, error, _ = self.run_scripted_capture(store, "capture-context")
        self.assertIsNone(error)
        h0 = context_result["context_handle"]
        invocation_result, error, _ = self.run_scripted_capture(
            store,
            "capture-invocation",
            h0,
        )
        self.assertIsNone(error)
        h1 = invocation_result["context_handle"]
        invocation_handle = invocation_result["invocation_handle"]
        self.assertNotIn(h0, store.contexts)
        self.assertIn(h1, store.contexts)
        self.assertEqual(store.invocations, {invocation_handle})
        reported_record_sha256 = "b" * 64

        def set_reported_record_sha256(bodies):
            bodies[1][1]["reported_record_sha256"] = reported_record_sha256
            return bodies

        result, error, _ = self.run_scripted_capture(
            store,
            "capture-response",
            h1,
            scenario="wrong-reply-nonce",
            body_mutator=set_reported_record_sha256,
        )
        self.assertIsNone(result)
        self.assertIsInstance(error, core.EvidenceError)
        self.assertNotIn(h1, store.contexts)
        self.assertEqual(len(store.contexts), 1)
        self.assertEqual(store.invocations, {invocation_handle})
        self.assertEqual(len(store.responses), 1)
        response_handle = next(iter(store.responses))
        expected_response = self.host_envelope_bodies()[2]
        expected_response["reported_record_sha256"] = reported_record_sha256
        self.assertEqual(store.responses[response_handle], expected_response)

        committed_snapshot = (
            copy.deepcopy(store.contexts),
            set(store.invocations),
            copy.deepcopy(store.responses),
        )
        replay_result, replay_error, _ = self.run_scripted_capture(
            store,
            "capture-response",
            h1,
            body_mutator=set_reported_record_sha256,
        )
        self.assertIsNone(replay_result)
        self.assertIsInstance(replay_error, core.EvidenceError)
        self.assertEqual(
            (store.contexts, store.invocations, store.responses),
            committed_snapshot,
            "replay changed the committed response body or capability generation",
        )

    def test_host_context_binds_root_owned_harness_bundle_revision_and_digest(self):
        core = importlib.import_module("scripts.evidence_core")
        adapter = ROOT / "scripts" / "implementation_evidence" / "host_envelope_adapter.py"
        adapter_source = adapter.read_text(encoding="utf-8")
        adapter_tree = ast.parse(adapter_source, filename=str(adapter))

        with self.subTest(source_contract="harness-root-only"):
            self.assertEqual(
                adapter_source.count("REPOSITORY_ROOT"),
                0,
                "adapter must not derive trusted tooling from the repository root",
            )
            self.assertEqual(
                adapter_source.count("CANDIDATE_ROOT"),
                0,
                "adapter must not derive trusted tooling from a candidate root",
            )
            tool_root_assignments = [
                node
                for node in ast.walk(adapter_tree)
                if isinstance(node, (ast.Assign, ast.AnnAssign))
                and any(
                    isinstance(target, ast.Name) and target.id == "HARNESS_ROOT"
                    for target in (
                        node.targets if isinstance(node, ast.Assign) else [node.target]
                    )
                )
            ]
            self.assertEqual(len(tool_root_assignments), 1)
            harness_root_value = tool_root_assignments[0].value
            self.assertEqual(
                {
                    node.id
                    for node in ast.walk(harness_root_value)
                    if isinstance(node, ast.Name)
                },
                {"Path", "__file__"},
                "HARNESS_ROOT must derive only from the exact adapter file",
            )
            self.assertIn(
                "resolve",
                {
                    node.attr
                    for node in ast.walk(harness_root_value)
                    if isinstance(node, ast.Attribute)
                },
            )

        with self.subTest(source_contract="exact-core-load-from-harness-root"):
            core_path_assignments = [
                node
                for node in ast.walk(adapter_tree)
                if isinstance(node, (ast.Assign, ast.AnnAssign))
                and any(
                    isinstance(target, ast.Name) and target.id == "CORE_PATH"
                    for target in (
                        node.targets if isinstance(node, ast.Assign) else [node.target]
                    )
                )
            ]
            self.assertEqual(len(core_path_assignments), 1)
            core_path_value = core_path_assignments[0].value
            self.assertEqual(
                {
                    node.id
                    for node in ast.walk(core_path_value)
                    if isinstance(node, ast.Name)
                },
                {"HARNESS_ROOT"},
                "CORE_PATH must derive only from HARNESS_ROOT",
            )
            exact_loads = [
                node
                for node in ast.walk(adapter_tree)
                if isinstance(node, ast.Call)
                and isinstance(node.func, ast.Attribute)
                and node.func.attr == "spec_from_file_location"
                and len(node.args) >= 2
                and isinstance(node.args[1], ast.Name)
                and node.args[1].id == "CORE_PATH"
            ]
            self.assertEqual(len(exact_loads), 1)

        plan_paths = (
            ROOT
            / "docs"
            / "superpowers"
            / "plans"
            / "2026-08-10-tersh-implementation-iteration-evidence.md",
            ROOT
            / "docs"
            / "superpowers"
            / "plans"
            / "2026-08-10-tersh-seven-cycle-hardening-implementation.md",
        )
        for plan_path in plan_paths:
            plan = plan_path.read_text(encoding="utf-8")
            with self.subTest(plan=plan_path.name, contract="harness-root-metavariable"):
                self.assertEqual(
                    plan.count("SUPERVISOR_CANDIDATE_ROOT"),
                    0,
                    "candidate-root execution metavariable remains in the plan",
                )
                self.assertIn("SUPERVISOR_HARNESS_ROOT", plan)
            for semantic, pattern in (
                (
                    "root-owned-harness-bundle",
                    r"(?is)(?:root-owned.{0,400}harness bundle|harness bundle.{0,400}root-owned)",
                ),
                (
                    "non-agent-writable-harness-bundle",
                    r"(?is)(?:harness bundle.{0,400}(?:non-agent-writable|not (?:agent|operator)[ -]writable|not writable by (?:the )?(?:agent|operator))|(?:non-agent-writable|not (?:agent|operator)[ -]writable).{0,400}harness bundle)",
                ),
                (
                    "digest-pinned-harness-bundle",
                    r"(?is)(?:harness bundle.{0,400}(?:digest-pinned|pinned.{0,120}(?:digest|sha-256)|(?:digest|sha-256).{0,120}pinned)|(?:digest-pinned|pinned.{0,120}(?:digest|sha-256)|(?:digest|sha-256).{0,120}pinned).{0,400}harness bundle)",
                ),
                (
                    "candidate-never-executable",
                    r"(?is)(?:candidate(?: code| tree| scripts?)?.{0,80}(?:must )?(?:never be|not be) execut|(?:never|must not) execut.{0,80}candidate)",
                ),
            ):
                with self.subTest(plan=plan_path.name, semantic=semantic):
                    self.assertIsNotNone(
                        re.search(pattern, plan),
                        f"plan lacks {semantic} semantics",
                    )

        context_body, invocation_body, response_body = self.host_envelope_bodies()
        expected_bundle_identity = {
            "harness_bundle_revision": "7" * 40,
            "harness_bundle_sha256": "8" * 64,
        }
        for body_kind, body in (
            ("context", context_body),
            ("invocation", invocation_body),
            ("response", response_body),
        ):
            with self.subTest(valid_fixture_body=body_kind):
                self.assertEqual(
                    {field: body[field] for field in expected_bundle_identity},
                    expected_bundle_identity,
                )

        def seeded_store(operation):
            store = ScriptedCaptureStore()
            if operation == "capture-context":
                return store, None
            predecessor = store.new_handle()
            store.contexts[predecessor] = copy.deepcopy(context_body)
            return store, predecessor

        valid_operations = {}
        for operation in (
            "capture-context",
            "capture-invocation",
            "capture-response",
        ):
            valid_operations[operation] = False
            with self.subTest(valid_bundle_transcript=operation):
                store, predecessor = seeded_store(operation)
                result, error, _ = self.run_scripted_capture(
                    store,
                    operation,
                    predecessor,
                )
                valid_operations[operation] = error is None and result is not None
                self.assertIsNone(error)
                self.assertIsNotNone(result)

        def mutate_body(body_index, mutation):
            def mutate(bodies):
                body = bodies[body_index][1]
                if mutation == "missing-revision":
                    del body["harness_bundle_revision"]
                elif mutation == "missing-digest":
                    del body["harness_bundle_sha256"]
                elif mutation == "extra":
                    body["harness_bundle_path"] = "/candidate-controlled"
                elif mutation == "wrong-revision":
                    body["harness_bundle_revision"] = "G" * 40
                elif mutation == "wrong-digest":
                    body["harness_bundle_sha256"] = "G" * 64
                elif mutation == "mismatched-revision":
                    body["harness_bundle_revision"] = "1" * 40
                elif mutation == "mismatched-digest":
                    body["harness_bundle_sha256"] = "2" * 64
                else:
                    raise AssertionError(f"unknown bundle mutation {mutation}")
                return bodies

            return mutate

        body_targets = (
            ("context", "capture-context", 0),
            ("invocation", "capture-invocation", 1),
            ("response", "capture-response", 1),
        )
        for body_kind, operation, body_index in body_targets:
            if not valid_operations[operation]:
                continue
            for mutation in (
                "missing-revision",
                "missing-digest",
                "extra",
                "wrong-revision",
                "wrong-digest",
            ):
                with self.subTest(
                    bundle_body=body_kind,
                    bundle_attack=mutation,
                ):
                    store, predecessor = seeded_store(operation)
                    before = (
                        copy.deepcopy(store.contexts),
                        set(store.invocations),
                        copy.deepcopy(store.responses),
                        store.next_handle,
                    )
                    result, error, _ = self.run_scripted_capture(
                        store,
                        operation,
                        predecessor,
                        body_mutator=mutate_body(body_index, mutation),
                    )
                    self.assertIsNone(result)
                    self.assertIsInstance(error, core.EvidenceError)
                    self.assertEqual(
                        (
                            store.contexts,
                            store.invocations,
                            store.responses,
                            store.next_handle,
                        ),
                        before,
                    )

        mismatch_targets = (
            ("context", "capture-invocation", 0),
            ("invocation", "capture-invocation", 1),
            ("response", "capture-response", 1),
        )
        for body_kind, operation, body_index in mismatch_targets:
            if not valid_operations[operation]:
                continue
            for mutation in ("mismatched-revision", "mismatched-digest"):
                with self.subTest(
                    bundle_body=body_kind,
                    bundle_attack=mutation,
                ):
                    store, predecessor = seeded_store(operation)
                    before = (
                        copy.deepcopy(store.contexts),
                        set(store.invocations),
                        copy.deepcopy(store.responses),
                        store.next_handle,
                    )
                    result, error, _ = self.run_scripted_capture(
                        store,
                        operation,
                        predecessor,
                        body_mutator=mutate_body(body_index, mutation),
                    )
                    self.assertIsNone(result)
                    self.assertIsInstance(error, core.EvidenceError)
                    self.assertEqual(
                        (
                            store.contexts,
                            store.invocations,
                            store.responses,
                            store.next_handle,
                        ),
                        before,
                    )


if __name__ == "__main__":
    unittest.main()
