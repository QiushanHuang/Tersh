#!/usr/bin/env python3
"""Run one argv gate and publish one immutable bounded evidence triplet."""

from __future__ import annotations

import argparse
import hashlib
import os
import platform
import subprocess
import sys
import threading
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Sequence


SCRIPTS_DIR = Path(__file__).resolve().parents[1]
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import evidence_core as core  # noqa: E402


RETAIN_LIMIT = 1024 * 1024
DIAGNOSTIC_LIMIT = 4096


class GateError(core.EvidenceError):
    pass


class ClosedParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:
        raise GateError(f"arguments: {message}")


class SingleUseAction(argparse.Action):
    def __call__(
        self,
        parser: argparse.ArgumentParser,
        namespace: argparse.Namespace,
        values: Any,
        option_string: str | None = None,
    ) -> None:
        seen = getattr(namespace, "_single_use", set())
        if self.dest in seen:
            raise GateError(f"arguments: {option_string} may be supplied only once")
        seen.add(self.dest)
        namespace._single_use = seen
        setattr(namespace, self.dest, self.const if self.nargs == 0 else values)


@dataclass
class DrainedStream:
    total_bytes: int
    sha256: str
    retained: bytes
    retained_sha256: str


def _utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="microseconds").replace("+00:00", "Z")


def _bounded_diagnostic(value: object) -> str:
    text = str(value).replace("\r", "\\r").replace("\n", "\\n")
    if len(text) > DIAGNOSTIC_LIMIT:
        return text[:DIAGNOSTIC_LIMIT] + "...[truncated]"
    return text


def parse_arguments(argv: Sequence[str] | None) -> tuple[argparse.Namespace, list[str]]:
    raw = list(sys.argv[1:] if argv is None else argv)
    if "--" not in raw:
        raise GateError("arguments: require -- before the child argv")
    separator = raw.index("--")
    child_argv = raw[separator + 1 :]
    if not child_argv:
        raise GateError("arguments: child argv must not be empty")

    parser = ClosedParser(prog="run_gate.py", allow_abbrev=False, add_help=True)
    parser.add_argument("--iteration", action=SingleUseAction, required=True)
    parser.add_argument("--attempt", action=SingleUseAction, required=True)
    parser.add_argument("--run-binding", action=SingleUseAction, required=True)
    parser.add_argument("--name", action=SingleUseAction, required=True)
    parser.add_argument("--candidate", action=SingleUseAction, required=True)
    parser.add_argument("--output-root", action=SingleUseAction, required=True)
    parser.add_argument(
        "--allow-failure",
        action=SingleUseAction,
        nargs=0,
        const=True,
        default=False,
    )
    arguments = parser.parse_args(raw[:separator])
    core.validate_evidence_id(arguments.iteration)
    core.validate_attempt(arguments.attempt)
    core.validate_run_binding(arguments.run_binding)
    core.validate_gate_name(arguments.name)
    core.validate_candidate(arguments.candidate)
    core.parse_output_root(arguments.output_root)
    if any(type(item) is not str or "\x00" in item for item in child_argv):
        raise GateError("arguments: child argv contains a non-string or NUL")
    return arguments, child_argv


def _tool_version(argv: Sequence[str], label: str) -> str:
    try:
        completed = subprocess.run(
            list(argv),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
    except OSError as error:
        raise GateError(f"{label} version command could not start: {error}") from error
    if completed.returncode != 0:
        raise GateError(f"{label} version command exited {completed.returncode}")
    try:
        value = completed.stdout.decode("utf-8", errors="strict").strip()
    except UnicodeDecodeError as error:
        raise GateError(f"{label} version output is not UTF-8") from error
    if not value or "\n" in value or "\r" in value:
        raise GateError(f"{label} version output is not one nonempty line")
    return value


def _drain(pipe: Any, result: dict[str, DrainedStream], key: str, errors: list[BaseException]) -> None:
    digest = hashlib.sha256()
    retained = bytearray()
    total = 0
    try:
        while True:
            chunk = pipe.read(65536)
            if not chunk:
                break
            total += len(chunk)
            digest.update(chunk)
            remaining = RETAIN_LIMIT - len(retained)
            if remaining > 0:
                retained.extend(chunk[:remaining])
        retained_bytes = bytes(retained)
        result[key] = DrainedStream(
            total_bytes=total,
            sha256=digest.hexdigest(),
            retained=retained_bytes,
            retained_sha256=hashlib.sha256(retained_bytes).hexdigest(),
        )
    except BaseException as error:
        errors.append(error)
    finally:
        try:
            pipe.close()
        except BaseException as error:
            errors.append(error)


def run_and_drain(argv: Sequence[str]) -> tuple[int, DrainedStream, DrainedStream]:
    try:
        child = subprocess.Popen(
            list(argv),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            shell=False,
        )
    except OSError as error:
        raise GateError(f"child could not start: {error}") from error
    if child.stdout is None or child.stderr is None:
        child.kill()
        child.wait()
        raise GateError("child pipes were not created")

    drained: dict[str, DrainedStream] = {}
    errors: list[BaseException] = []
    readers = [
        threading.Thread(target=_drain, args=(child.stdout, drained, "stdout", errors)),
        threading.Thread(target=_drain, args=(child.stderr, drained, "stderr", errors)),
    ]
    for reader in readers:
        reader.start()
    returncode = child.wait()
    for reader in readers:
        reader.join()
    if errors:
        raise GateError(f"child output drain failed: {errors[0]}")
    if set(drained) != {"stdout", "stderr"}:
        raise GateError("child output drain did not produce both streams")
    return returncode, drained["stdout"], drained["stderr"]


def _stream_record(
    drained: DrainedStream,
    run_binding: str,
    gate_name: str,
    stream_name: str,
) -> dict[str, Any]:
    return {
        "total_bytes": drained.total_bytes,
        "sha256": drained.sha256,
        "retained_bytes": len(drained.retained),
        "retained_sha256": drained.retained_sha256,
        "retained_log": core.candidate_relative_log_path(run_binding, gate_name, stream_name),
    }


def _open_gate_directory(arguments: argparse.Namespace) -> tuple[int, core.GateReservation]:
    root_fd: int | None = None
    gates_fd: int | None = None
    try:
        root_fd = core.open_output_root(arguments.output_root)
        gates_fd = core.open_internal_tree(
            root_fd,
            (
                arguments.iteration,
                f"attempt-{arguments.attempt}",
                f"candidate-{arguments.candidate}",
                arguments.run_binding,
                "gates",
            ),
        )
        reservation = core.reserve_gate_triplet(gates_fd, arguments.name)
        return gates_fd, reservation
    except BaseException:
        core.close_fds(gates_fd)
        raise
    finally:
        core.close_fds(root_fd)


def execute(arguments: argparse.Namespace, child_argv: list[str]) -> int:
    gates_fd, reservation = _open_gate_directory(arguments)
    try:
        rustc_version = _tool_version(("rustc", "--version"), "rustc")
        cargo_version = _tool_version(("cargo", "--version"), "cargo")
        cwd = core.resolved_cwd()
        started_at = _utc_now()
        monotonic_start = time.monotonic_ns()
        exit_code, stdout, stderr = run_and_drain(child_argv)
        monotonic_end = time.monotonic_ns()
        ended_at = _utc_now()
        duration_ms = max(0, (monotonic_end - monotonic_start) // 1_000_000)

        record = {
            "schema": "tersh-implementation-gate-v1",
            "iteration": arguments.iteration,
            "evidence_attempt": arguments.attempt,
            "run_binding": arguments.run_binding,
            "name": arguments.name,
            "argv": child_argv,
            "cwd": cwd,
            "candidate": arguments.candidate,
            "started_at": started_at,
            "ended_at": ended_at,
            "duration_ms": duration_ms,
            "exit_code": exit_code,
            "stdout": _stream_record(stdout, arguments.run_binding, arguments.name, "stdout"),
            "stderr": _stream_record(stderr, arguments.run_binding, arguments.name, "stderr"),
            "os": platform.system(),
            "architecture": platform.machine(),
            "rustc_version": rustc_version,
            "cargo_version": cargo_version,
            "exact_test_inventory": None,
        }

        # The JSON is the completion marker, so publish both retained logs first.
        core.publish_new_at(gates_fd, f"{arguments.name}.stdout", stdout.retained)
        core.publish_new_at(gates_fd, f"{arguments.name}.stderr", stderr.retained)
        core.publish_new_at(
            gates_fd,
            f"{arguments.name}.json",
            core.canonical_json_bytes(record),
        )
        if arguments.allow_failure:
            return 0
        if exit_code >= 0:
            return min(exit_code, 255)
        return min(128 + (-exit_code), 255)
    finally:
        try:
            reservation.close()
        finally:
            core.close_fds(gates_fd)


def main(argv: Sequence[str] | None = None) -> int:
    try:
        arguments, child_argv = parse_arguments(argv)
        return execute(arguments, child_argv)
    except (GateError, core.EvidenceError, OSError, ValueError) as error:
        sys.stderr.write(f"run_gate: rejected: {_bounded_diagnostic(error)}\n")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
