"""Shared fail-closed primitives for implementation and hardening evidence.

This module intentionally has no command-line entry point.  Path creation and
publication are dirfd-relative so validation is not followed by a pathname
re-open that could traverse a substituted symlink.
"""

from __future__ import annotations

import hashlib
import json
import math
import os
import re
import secrets
import selectors
import signal
import socket
import stat
import struct
import subprocess
import sys
import threading
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable, Sequence


EVIDENCE_ID_RE = re.compile(r"^(?:impl|hardening)-0[1-7]$")
ATTEMPT_RE = re.compile(r"^(?:00[1-9]|0[1-9][0-9]|[1-9][0-9]{2})$")
CANDIDATE_RE = re.compile(r"^[0-9a-f]{40}$")
GATE_RE = re.compile(r"^[a-z][a-z0-9-]{0,63}$")
RUN_BINDING_RE = re.compile(
    r"^(?:run-local|run-cumulative|"
    r"run-set-(?:"
    r"ci-(?:[1-9][0-9]*|unregistered)"
    r"(?:-release-(?:[1-9][0-9]*|unregistered))?"
    r"|release-(?:[1-9][0-9]*|unregistered)"
    r"))$"
)
HEX64_RE = re.compile(r"^[0-9a-f]{64}$")
TASK_PATH_RE = re.compile(r"^/root(?:/[a-z][a-z0-9_]{0,63})+$")
HOST_ID_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$")
RFC3339_NANO_RE = re.compile(
    r"^(?P<date>[0-9]{4}-[0-9]{2}-[0-9]{2})T"
    r"(?P<time>[0-9]{2}:[0-9]{2}:[0-9]{2})\."
    r"(?P<fraction>[0-9]{1,9})Z$"
)
DEFAULT_RETAIN_LIMIT = 1024 * 1024
HOST_FRAME_MAX_BYTES = 65536
HOST_TRANSACTION_TIMEOUT_SECONDS = 5.0
PROCESS_TERM_GRACE_SECONDS = 1.0
PROCESS_KILL_GRACE_SECONDS = 1.0
READER_CLEANUP_SECONDS = 1.0


class EvidenceError(Exception):
    """A bounded, user-facing fail-closed evidence error."""


@dataclass(frozen=True)
class DrainedStream:
    total_bytes: int
    sha256: str
    retained: bytes
    retained_sha256: str


def _drain_stream(
    pipe: Any,
    result: dict[str, DrainedStream],
    key: str,
    errors: list[BaseException],
    retain_limit: int,
    stop_event: threading.Event,
) -> None:
    digest = hashlib.sha256()
    retained = bytearray()
    total = 0
    selector: selectors.BaseSelector | None = None
    try:
        os.set_blocking(pipe.fileno(), False)
        selector = selectors.DefaultSelector()
        selector.register(pipe, selectors.EVENT_READ)
        while True:
            if stop_event.is_set():
                break
            if not selector.select(timeout=0.05):
                continue
            try:
                chunk = os.read(pipe.fileno(), 65536)
            except BlockingIOError:
                continue
            if not chunk:
                break
            total += len(chunk)
            digest.update(chunk)
            remaining = retain_limit - len(retained)
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
        if selector is not None:
            selector.close()
        try:
            pipe.close()
        except BaseException as error:
            errors.append(error)


def _require_posix_process_groups() -> None:
    if (
        os.name != "posix"
        or not callable(getattr(os, "killpg", None))
        or not callable(getattr(os, "set_blocking", None))
    ):
        raise EvidenceError("POSIX process-group cleanup is unavailable")


def _process_group_exists(process_group: int) -> bool:
    try:
        os.killpg(process_group, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def _wait_process_group(child: subprocess.Popen[bytes], deadline: float) -> bool:
    while True:
        child.poll()
        if not _process_group_exists(child.pid):
            return True
        if time.monotonic() >= deadline:
            return False
        time.sleep(0.01)


def _cleanup_child_process_group(
    child: subprocess.Popen[bytes],
    readers: Sequence[threading.Thread],
    stop_event: threading.Event | None,
) -> None:
    try:
        os.killpg(child.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    term_deadline = time.monotonic() + PROCESS_TERM_GRACE_SECONDS
    terminated = _wait_process_group(child, term_deadline)
    if not terminated:
        try:
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        _wait_process_group(child, time.monotonic() + PROCESS_KILL_GRACE_SECONDS)

    if child.poll() is None:
        try:
            child.wait(timeout=PROCESS_KILL_GRACE_SECONDS)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait(timeout=PROCESS_KILL_GRACE_SECONDS)

    reader_deadline = time.monotonic() + READER_CLEANUP_SECONDS
    for reader in readers:
        reader.join(timeout=max(0.0, reader_deadline - time.monotonic()))
    if any(reader.is_alive() for reader in readers):
        if stop_event is not None:
            stop_event.set()
        reader_deadline = time.monotonic() + READER_CLEANUP_SECONDS
        for reader in readers:
            reader.join(timeout=max(0.0, reader_deadline - time.monotonic()))
    for pipe in (child.stdout, child.stderr):
        if pipe is not None:
            try:
                pipe.close()
            except OSError:
                pass
    if any(reader.is_alive() for reader in readers):
        reader_deadline = time.monotonic() + READER_CLEANUP_SECONDS
        for reader in readers:
            reader.join(timeout=max(0.0, reader_deadline - time.monotonic()))
    if any(reader.is_alive() for reader in readers):
        raise EvidenceError("child output readers did not terminate during cleanup")


def run_and_drain(
    argv: Sequence[str],
    *,
    retain_limit: int = DEFAULT_RETAIN_LIMIT,
) -> tuple[int, DrainedStream, DrainedStream]:
    """Run one argv without a shell and concurrently drain both byte streams."""

    if isinstance(argv, (str, bytes)) or not argv:
        raise EvidenceError("child argv must be a nonempty sequence")
    command = list(argv)
    if any(type(item) is not str or "\x00" in item for item in command):
        raise EvidenceError("child argv contains a non-string or NUL")
    require_exact_int(retain_limit, "retain limit", minimum=0)
    _require_posix_process_groups()
    try:
        child = subprocess.Popen(
            command,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            shell=False,
            start_new_session=True,
        )
    except OSError as error:
        raise EvidenceError(f"child could not start: {error}") from error
    if child.stdout is None or child.stderr is None:
        child.kill()
        child.wait()
        raise EvidenceError("child pipes were not created")

    started_readers: list[threading.Thread] = []
    stop_event: threading.Event | None = None
    try:
        drained: dict[str, DrainedStream] = {}
        errors: list[BaseException] = []
        stop_event = threading.Event()
        readers = [
            threading.Thread(
                target=_drain_stream,
                args=(child.stdout, drained, "stdout", errors, retain_limit, stop_event),
            ),
            threading.Thread(
                target=_drain_stream,
                args=(child.stderr, drained, "stderr", errors, retain_limit, stop_event),
            ),
        ]
        for reader in readers:
            reader.start()
            started_readers.append(reader)
        returncode = child.wait()
        for reader in readers:
            reader.join()
        if errors:
            raise EvidenceError(f"child output drain failed: {errors[0]}")
        if set(drained) != {"stdout", "stderr"}:
            raise EvidenceError("child output drain did not produce both streams")
        return returncode, drained["stdout"], drained["stderr"]
    except BaseException:
        _cleanup_child_process_group(child, started_readers, stop_event)
        raise


def canonical_json_bytes(value: Any) -> bytes:
    """Return sorted compact UTF-8 JSON with one LF and no non-finite floats."""

    try:
        text = json.dumps(
            value,
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
            allow_nan=False,
        )
        return text.encode("utf-8", errors="strict") + b"\n"
    except (TypeError, ValueError, UnicodeError) as error:
        raise EvidenceError(f"value is not canonical JSON: {error}") from error


def _closed_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise EvidenceError(f"duplicate JSON key: {key!r}")
        result[key] = value
    return result


def _reject_constant(value: str) -> Any:
    raise EvidenceError(f"non-finite JSON number: {value}")


def load_canonical_json_bytes(raw: bytes) -> Any:
    """Load only byte-for-byte canonical JSON generated by this module."""

    if type(raw) is not bytes:
        raise EvidenceError("canonical JSON input must be bytes")
    try:
        text = raw.decode("utf-8", errors="strict")
        value = json.loads(
            text,
            object_pairs_hook=_closed_pairs,
            parse_constant=_reject_constant,
        )
    except EvidenceError:
        raise
    except (UnicodeError, json.JSONDecodeError, ValueError) as error:
        raise EvidenceError(f"invalid canonical JSON: {error}") from error
    if canonical_json_bytes(value) != raw:
        raise EvidenceError("JSON bytes are not sorted compact canonical UTF-8 plus LF")
    return value


def require_exact_int(value: Any, field: str, *, minimum: int | None = None) -> int:
    if type(value) is not int:
        raise EvidenceError(f"{field} must be an exact JSON integer")
    if minimum is not None and value < minimum:
        raise EvidenceError(f"{field} must be at least {minimum}")
    return value


def _require_match(value: Any, field: str, pattern: re.Pattern[str]) -> str:
    if type(value) is not str or pattern.fullmatch(value) is None:
        raise EvidenceError(f"invalid {field}: {value!r}")
    return value


def validate_evidence_id(value: Any) -> str:
    return _require_match(value, "evidence ID", EVIDENCE_ID_RE)


def validate_attempt(value: Any) -> str:
    return _require_match(value, "evidence attempt", ATTEMPT_RE)


def validate_candidate(value: Any) -> str:
    return _require_match(value, "candidate", CANDIDATE_RE)


def validate_gate_name(value: Any) -> str:
    return _require_match(value, "gate name", GATE_RE)


def validate_run_binding(value: Any) -> str:
    return _require_match(value, "run binding", RUN_BINDING_RE)


def validate_sha256(value: Any, field: str = "sha256") -> str:
    return _require_match(value, field, HEX64_RE)


def validate_host_peer_identity(
    peer_uid: Any,
    fd_owner_uid: Any,
    client_euid: Any,
) -> None:
    """Bind a host connection to the fixed root-owned supervisor TCB."""

    for value, field in (
        (peer_uid, "host peer UID"),
        (fd_owner_uid, "host socket owner UID"),
        (client_euid, "client effective UID"),
    ):
        require_exact_int(value, field, minimum=0)
    if peer_uid != 0 or fd_owner_uid != 0 or client_euid == 0:
        raise EvidenceError(
            "host connection requires root peer and owner with a nonroot client"
        )


def parse_macos_local_peercred(raw: Any) -> tuple[int, tuple[int, ...]]:
    """Parse one exact macOS xucred returned by LOCAL_PEERCRED."""

    if type(raw) is not bytes or len(raw) != 76:
        raise EvidenceError("macOS LOCAL_PEERCRED must be exactly 76 bytes")
    try:
        version, uid, group_count, *groups = struct.unpack("=III16I", raw)
    except struct.error as error:
        raise EvidenceError(f"invalid macOS LOCAL_PEERCRED: {error}") from error
    if version != 0:
        raise EvidenceError("macOS LOCAL_PEERCRED version must be zero")
    if group_count > 16:
        raise EvidenceError("macOS LOCAL_PEERCRED group count exceeds 16")
    return uid, tuple(groups[:group_count])


def parse_linux_so_peercred(raw: Any) -> tuple[int, int, int]:
    """Parse one exact Linux ucred returned by SO_PEERCRED."""

    if type(raw) is not bytes or len(raw) != 12:
        raise EvidenceError("Linux SO_PEERCRED must be exactly 12 bytes")
    try:
        pid, uid, gid = struct.unpack("=iii", raw)
    except struct.error as error:
        raise EvidenceError(f"invalid Linux SO_PEERCRED: {error}") from error
    if pid < 0 or uid < 0 or gid < 0:
        raise EvidenceError("Linux SO_PEERCRED fields must be nonnegative")
    return pid, uid, gid


def open_authenticated_host_store_socket(fd: Any) -> socket.socket:
    """Duplicate and authenticate one connected production host-store socket."""

    require_exact_int(fd, "host store FD", minimum=3)
    try:
        duplicate = os.dup(fd)
    except (OSError, OverflowError) as error:
        raise EvidenceError(f"cannot duplicate host store FD: {error}") from error
    try:
        metadata = os.fstat(duplicate)
    except OSError as error:
        os.close(duplicate)
        raise EvidenceError(f"cannot inspect duplicated host store FD: {error}") from error
    connection: socket.socket | None = None
    try:
        connection = socket.socket(fileno=duplicate)
        duplicate = -1
        if connection.family != socket.AF_UNIX:
            raise EvidenceError("host store FD is not AF_UNIX")
        if connection.getsockopt(socket.SOL_SOCKET, socket.SO_TYPE) != socket.SOCK_STREAM:
            raise EvidenceError("host store FD is not SOCK_STREAM")
        try:
            connection.getpeername()
        except OSError as error:
            raise EvidenceError("host store FD is not connected") from error

        if sys.platform == "darwin":
            option = getattr(socket, "LOCAL_PEERCRED", None)
            if type(option) is not int:
                raise EvidenceError("macOS LOCAL_PEERCRED is unavailable")
            raw = connection.getsockopt(0, option, 76)
            peer_uid, _ = parse_macos_local_peercred(raw)
        elif sys.platform.startswith("linux"):
            option = getattr(socket, "SO_PEERCRED", None)
            if type(option) is not int:
                raise EvidenceError("Linux SO_PEERCRED is unavailable")
            raw = connection.getsockopt(socket.SOL_SOCKET, option, 12)
            _, peer_uid, _ = parse_linux_so_peercred(raw)
        else:
            raise EvidenceError("host peer credentials are unsupported on this platform")

        validate_host_peer_identity(peer_uid, metadata.st_uid, os.geteuid())
        return connection
    except EvidenceError:
        if connection is not None:
            connection.close()
        elif duplicate >= 0:
            os.close(duplicate)
        raise
    except OSError as error:
        if connection is not None:
            connection.close()
        elif duplicate >= 0:
            os.close(duplicate)
        raise EvidenceError(f"cannot authenticate host store FD: {error}") from error


def _remaining_host_deadline(deadline: Any) -> float:
    if type(deadline) not in (int, float) or not math.isfinite(deadline):
        raise EvidenceError("host transaction deadline must be finite monotonic time")
    remaining = float(deadline) - time.monotonic()
    if remaining <= 0:
        raise EvidenceError("host transaction deadline expired")
    return remaining


def _require_host_socket(sock: Any) -> socket.socket:
    if not isinstance(sock, socket.socket):
        raise EvidenceError("host transport must be a socket")
    return sock


def _recv_host_exact(sock: socket.socket, size: int, deadline: float) -> bytes:
    received = bytearray()
    while len(received) < size:
        try:
            sock.settimeout(_remaining_host_deadline(deadline))
            chunk = sock.recv(size - len(received))
        except (OSError, TimeoutError) as error:
            raise EvidenceError(f"host frame receive failed: {error}") from error
        if not chunk:
            raise EvidenceError("host frame ended before its declared length")
        received.extend(chunk)
    return bytes(received)


def send_host_frame(sock: Any, value: Any, deadline: Any) -> None:
    """Send one bounded canonical length-prefixed host JSON object."""

    connection = _require_host_socket(sock)
    if type(value) is not dict:
        raise EvidenceError("host frame must be a JSON object")
    body = canonical_json_bytes(value)
    if not 1 <= len(body) <= HOST_FRAME_MAX_BYTES:
        raise EvidenceError("host frame body is outside 1..65536 bytes")
    frame = struct.pack(">I", len(body)) + body
    try:
        connection.settimeout(_remaining_host_deadline(deadline))
        connection.sendall(frame)
    except (OSError, TimeoutError) as error:
        raise EvidenceError(f"host frame send failed: {error}") from error


def recv_host_frame(sock: Any, deadline: Any) -> dict[str, Any]:
    """Receive one bounded byte-for-byte canonical host JSON object."""

    connection = _require_host_socket(sock)
    prefix = _recv_host_exact(connection, 4, deadline)
    size = struct.unpack(">I", prefix)[0]
    if not 1 <= size <= HOST_FRAME_MAX_BYTES:
        raise EvidenceError("host frame body is outside 1..65536 bytes")
    value = load_canonical_json_bytes(_recv_host_exact(connection, size, deadline))
    if type(value) is not dict:
        raise EvidenceError("host frame must be a JSON object")
    return value


def require_host_eof(sock: Any, deadline: Any) -> None:
    """Require peer write-half-close with no trailing byte."""

    connection = _require_host_socket(sock)
    try:
        connection.settimeout(_remaining_host_deadline(deadline))
        trailing = connection.recv(1)
    except (OSError, TimeoutError) as error:
        raise EvidenceError(f"host EOF observation failed: {error}") from error
    if trailing != b"":
        raise EvidenceError("host transaction contains trailing bytes")


def _require_closed_object(
    value: Any,
    keys: set[str],
    field: str,
) -> dict[str, Any]:
    if type(value) is not dict or set(value) != keys:
        actual = sorted(value) if type(value) is dict else type(value).__name__
        raise EvidenceError(
            f"{field} is not a closed object: expected={sorted(keys)!r} actual={actual!r}"
        )
    return value


def _require_literal(value: Any, expected: str, field: str) -> str:
    if type(value) is not str or value != expected:
        raise EvidenceError(f"{field} must be exactly {expected!r}")
    return value


def _require_enum(value: Any, allowed: set[str], field: str) -> str:
    if type(value) is not str or value not in allowed:
        raise EvidenceError(f"invalid {field}: {value!r}")
    return value


def _require_nullable_candidate(value: Any, field: str) -> str | None:
    if value is None:
        return None
    return _require_match(value, field, CANDIDATE_RE)


def _parse_rfc3339_nano(value: Any, field: str) -> tuple[datetime, int]:
    if type(value) is not str:
        raise EvidenceError(f"{field} must be an RFC 3339 UTC string")
    match = RFC3339_NANO_RE.fullmatch(value)
    if match is None:
        raise EvidenceError(f"invalid {field}: {value!r}")
    try:
        whole = datetime.strptime(
            f"{match.group('date')}T{match.group('time')}",
            "%Y-%m-%dT%H:%M:%S",
        ).replace(tzinfo=timezone.utc)
    except ValueError as error:
        raise EvidenceError(f"invalid {field}: {value!r}") from error
    return whole, int(match.group("fraction").ljust(9, "0"))


def _validate_context_body(value: Any) -> dict[str, Any]:
    keys = {
        "schema", "context_nonce", "harness_bundle_revision",
        "harness_bundle_sha256", "evidence_id", "evidence_attempt", "role",
        "wave", "review_attempt", "run_binding", "baseline_commit",
        "review_target", "canonical_task_path", "worktree_handle",
        "requested_model", "requested_reasoning_effort", "created_at",
    }
    body = _require_closed_object(value, keys, "context body")
    _require_literal(body["schema"], "tersh-host-dispatch-context-v1", "context schema")
    validate_sha256(body["context_nonce"], "context nonce")
    validate_candidate(body["harness_bundle_revision"])
    validate_sha256(body["harness_bundle_sha256"], "harness bundle sha256")
    validate_evidence_id(body["evidence_id"])
    validate_attempt(body["evidence_attempt"])
    _require_enum(
        body["role"],
        {"product", "architecture", "implementation", "safety", "verification"},
        "context role",
    )
    _require_enum(
        body["wave"],
        {"wave-a", "wave-b", "wave-c", "closure-a", "closure-b"},
        "context wave",
    )
    validate_attempt(body["review_attempt"])
    validate_run_binding(body["run_binding"])
    validate_candidate(body["baseline_commit"])
    _require_nullable_candidate(body["review_target"], "context review target")
    _require_match(body["canonical_task_path"], "context task path", TASK_PATH_RE)
    _require_match(body["worktree_handle"], "context worktree handle", HOST_ID_RE)
    _require_literal(body["requested_model"], "gpt-5.6-sol", "requested model")
    _require_literal(body["requested_reasoning_effort"], "xhigh", "requested effort")
    _parse_rfc3339_nano(body["created_at"], "context created_at")
    return body


def _validate_invocation_body(value: Any) -> dict[str, Any]:
    keys = {
        "schema", "context_nonce", "harness_bundle_revision",
        "harness_bundle_sha256", "dispatch_id", "requested_model",
        "requested_reasoning_effort", "selected_model",
        "selected_reasoning_effort", "dispatched_at",
    }
    body = _require_closed_object(value, keys, "invocation body")
    _require_literal(body["schema"], "tersh-host-spawn-invocation-v1", "invocation schema")
    validate_sha256(body["context_nonce"], "invocation context nonce")
    validate_candidate(body["harness_bundle_revision"])
    validate_sha256(body["harness_bundle_sha256"], "harness bundle sha256")
    validate_sha256(body["dispatch_id"], "invocation dispatch ID")
    _require_literal(body["requested_model"], "gpt-5.6-sol", "requested model")
    _require_literal(body["requested_reasoning_effort"], "xhigh", "requested effort")
    _require_literal(body["selected_model"], "gpt-5.6-sol", "selected model")
    _require_literal(body["selected_reasoning_effort"], "xhigh", "selected effort")
    _parse_rfc3339_nano(body["dispatched_at"], "invocation dispatched_at")
    return body


def _validate_response_body(value: Any) -> dict[str, Any]:
    keys = {
        "schema", "context_nonce", "harness_bundle_revision",
        "harness_bundle_sha256", "dispatch_id", "agent_id",
        "canonical_task_path", "agent_run_id", "started_at", "ended_at",
        "terminal_status", "reported_result_commit",
    }
    body = _require_closed_object(value, keys, "response body")
    _require_literal(body["schema"], "tersh-host-spawn-response-v1", "response schema")
    validate_sha256(body["context_nonce"], "response context nonce")
    validate_candidate(body["harness_bundle_revision"])
    validate_sha256(body["harness_bundle_sha256"], "harness bundle sha256")
    validate_sha256(body["dispatch_id"], "response dispatch ID")
    _require_match(body["agent_id"], "response agent ID", HOST_ID_RE)
    _require_match(body["canonical_task_path"], "response task path", TASK_PATH_RE)
    _require_match(body["agent_run_id"], "response agent run ID", HOST_ID_RE)
    started = _parse_rfc3339_nano(body["started_at"], "response started_at")
    ended = _parse_rfc3339_nano(body["ended_at"], "response ended_at")
    if started > ended:
        raise EvidenceError("response started_at is after ended_at")
    _require_enum(
        body["terminal_status"],
        {"completed", "failed", "cancelled", "interrupted"},
        "response terminal status",
    )
    _require_nullable_candidate(body["reported_result_commit"], "reported result commit")
    return body


def _validate_capture_relationships(
    operation: str,
    bodies: dict[str, dict[str, Any]],
) -> None:
    context = bodies["context"]
    created = _parse_rfc3339_nano(context["created_at"], "context created_at")
    if operation == "capture-invocation":
        invocation = bodies["invocation"]
        if invocation["context_nonce"] != context["context_nonce"]:
            raise EvidenceError("invocation context nonce does not match context")
        for field in (
            "harness_bundle_revision",
            "harness_bundle_sha256",
            "requested_model",
            "requested_reasoning_effort",
        ):
            if invocation[field] != context[field]:
                raise EvidenceError(f"invocation {field} does not match context")
        if created > _parse_rfc3339_nano(invocation["dispatched_at"], "invocation dispatched_at"):
            raise EvidenceError("context created_at is after invocation dispatched_at")
    elif operation == "capture-response":
        response = bodies["response"]
        if response["context_nonce"] != context["context_nonce"]:
            raise EvidenceError("response context nonce does not match context")
        for field in ("harness_bundle_revision", "harness_bundle_sha256"):
            if response[field] != context[field]:
                raise EvidenceError(f"response {field} does not match context")
        if response["canonical_task_path"] != context["canonical_task_path"]:
            raise EvidenceError("response task path does not match context")
        if created > _parse_rfc3339_nano(response["started_at"], "response started_at"):
            raise EvidenceError("context created_at is after response started_at")


def _validate_capture_result(
    operation: str,
    value: Any,
    predecessor: str | None,
) -> dict[str, Any]:
    member_by_operation = {
        "capture-context": None,
        "capture-invocation": "invocation_handle",
        "capture-response": "response_handle",
    }
    schema_by_operation = {
        "capture-context": "tersh-host-capture-context-result-v1",
        "capture-invocation": "tersh-host-capture-invocation-result-v1",
        "capture-response": "tersh-host-capture-response-result-v1",
    }
    member = member_by_operation[operation]
    keys = {"schema", "context_handle"}
    if member is not None:
        keys.add(member)
    result = _require_closed_object(value, keys, "capture result")
    _require_literal(result["schema"], schema_by_operation[operation], "capture result schema")
    successor = validate_sha256(result["context_handle"], "successor context handle")
    if predecessor is not None and successor == predecessor:
        raise EvidenceError("successor context handle aliases its consumed predecessor")
    if member is not None:
        member_handle = validate_sha256(result[member], member.replace("_", " "))
        if member_handle == successor or member_handle == predecessor:
            raise EvidenceError("member handle aliases its context generation")
    return result


def capture_host_envelope_on_authenticated_socket(
    sock: Any,
    operation: Any,
    context_handle: Any = None,
    *,
    deadline: Any = None,
) -> dict[str, Any]:
    """Execute one exact capture transaction over an authenticated socket."""

    connection = _require_host_socket(sock)
    body_order_by_operation = {
        "capture-context": ("context",),
        "capture-invocation": ("context", "invocation"),
        "capture-response": ("context", "response"),
    }
    if type(operation) is not str or operation not in body_order_by_operation:
        raise EvidenceError("invalid host capture operation")
    if operation == "capture-context":
        if context_handle is not None:
            raise EvidenceError("capture-context does not accept a context handle")
    else:
        validate_sha256(context_handle, "context handle")
    if deadline is None:
        deadline = time.monotonic() + HOST_TRANSACTION_TIMEOUT_SECONDS
    _remaining_host_deadline(deadline)

    transaction_nonce = secrets.token_hex(32)
    begin = {
        "schema": "tersh-host-transaction-begin-v1",
        "transaction_nonce": transaction_nonce,
        "operation": operation,
    }
    if context_handle is not None:
        begin["context_handle"] = context_handle
    send_host_frame(connection, begin, deadline)

    body_order = body_order_by_operation[operation]
    body_hashes: list[str] = []
    bodies: dict[str, dict[str, Any]] = {}
    validators = {
        "context": _validate_context_body,
        "invocation": _validate_invocation_body,
        "response": _validate_response_body,
    }
    wrapper_keys = {
        "schema", "transaction_nonce", "operation", "body_kind", "ordinal",
        "total", "body", "body_sha256",
    }
    for ordinal, body_kind in enumerate(body_order, start=1):
        wrapper = _require_closed_object(
            recv_host_frame(connection, deadline),
            wrapper_keys,
            "host BODY wrapper",
        )
        _require_literal(wrapper["schema"], "tersh-host-transaction-body-v1", "BODY schema")
        _require_literal(wrapper["transaction_nonce"], transaction_nonce, "BODY transaction nonce")
        _require_literal(wrapper["operation"], operation, "BODY operation")
        _require_literal(wrapper["body_kind"], body_kind, "BODY kind")
        if require_exact_int(wrapper["ordinal"], "BODY ordinal", minimum=1) != ordinal:
            raise EvidenceError("BODY ordinal is not the expected one-based value")
        if require_exact_int(wrapper["total"], "BODY total", minimum=1) != len(body_order):
            raise EvidenceError("BODY total does not match operation body count")
        body = validators[body_kind](wrapper["body"])
        digest = sha256_bytes(canonical_json_bytes(body))
        validate_sha256(wrapper["body_sha256"], "BODY sha256")
        if wrapper["body_sha256"] != digest:
            raise EvidenceError("BODY sha256 does not match its canonical body")
        bodies[body_kind] = body
        body_hashes.append(digest)

    body_end = _require_closed_object(
        recv_host_frame(connection, deadline),
        {"schema", "transaction_nonce", "operation", "total", "body_sha256s"},
        "host BODY-END",
    )
    _require_literal(body_end["schema"], "tersh-host-transaction-body-end-v1", "BODY-END schema")
    _require_literal(body_end["transaction_nonce"], transaction_nonce, "BODY-END transaction nonce")
    _require_literal(body_end["operation"], operation, "BODY-END operation")
    if require_exact_int(body_end["total"], "BODY-END total", minimum=1) != len(body_order):
        raise EvidenceError("BODY-END total does not match operation body count")
    if type(body_end["body_sha256s"]) is not list:
        raise EvidenceError("BODY-END body_sha256s must be an array")
    for digest in body_end["body_sha256s"]:
        validate_sha256(digest, "BODY-END digest")
    if body_end["body_sha256s"] != body_hashes:
        raise EvidenceError("BODY-END digests do not match BODY sequence")
    _validate_capture_relationships(operation, bodies)

    commit = {
        "schema": "tersh-host-transaction-commit-v1",
        "transaction_nonce": transaction_nonce,
        "operation": operation,
        "body_sha256s": body_hashes,
    }
    send_host_frame(connection, commit, deadline)
    request_end = {
        "schema": "tersh-host-transaction-request-end-v1",
        "transaction_nonce": transaction_nonce,
        "operation": operation,
        "commit_sha256": sha256_bytes(canonical_json_bytes(commit)),
    }
    send_host_frame(connection, request_end, deadline)
    try:
        connection.shutdown(socket.SHUT_WR)
    except OSError as error:
        raise EvidenceError(f"host request half-close failed: {error}") from error

    reply = _require_closed_object(
        recv_host_frame(connection, deadline),
        {"schema", "transaction_nonce", "operation", "body_sha256s", "result"},
        "host REPLY",
    )
    _require_literal(reply["schema"], "tersh-host-transaction-reply-v1", "REPLY schema")
    _require_literal(reply["transaction_nonce"], transaction_nonce, "REPLY transaction nonce")
    _require_literal(reply["operation"], operation, "REPLY operation")
    if type(reply["body_sha256s"]) is not list:
        raise EvidenceError("REPLY body_sha256s must be an array")
    for digest in reply["body_sha256s"]:
        validate_sha256(digest, "REPLY body digest")
    if reply["body_sha256s"] != body_hashes:
        raise EvidenceError("REPLY body digests do not match committed bodies")
    result = _validate_capture_result(operation, reply["result"], context_handle)

    reply_end = _require_closed_object(
        recv_host_frame(connection, deadline),
        {"schema", "transaction_nonce", "operation", "reply_sha256"},
        "host REPLY-END",
    )
    _require_literal(reply_end["schema"], "tersh-host-transaction-reply-end-v1", "REPLY-END schema")
    _require_literal(reply_end["transaction_nonce"], transaction_nonce, "REPLY-END transaction nonce")
    _require_literal(reply_end["operation"], operation, "REPLY-END operation")
    validate_sha256(reply_end["reply_sha256"], "REPLY-END digest")
    if reply_end["reply_sha256"] != sha256_bytes(canonical_json_bytes(reply)):
        raise EvidenceError("REPLY-END digest does not match REPLY")
    require_host_eof(connection, deadline)
    return result


def parse_output_root(raw: Any) -> tuple[bool, tuple[str, ...]]:
    """Validate a raw path without allowing pathlib/os.path normalization."""

    if type(raw) is not str or not raw or "\x00" in raw:
        raise EvidenceError("output root must be a nonempty NUL-free path")
    if raw in ("/", ".") or raw.endswith("/"):
        raise EvidenceError("output root must contain canonical path components")
    absolute = raw.startswith("/")
    pieces = raw.split("/")
    if absolute:
        pieces = pieces[1:]
    if not pieces or any(piece in ("", ".", "..") for piece in pieces):
        raise EvidenceError("output root contains an empty, dot, or parent component")
    return absolute, tuple(pieces)


def _nofollow_flag() -> int:
    flag = getattr(os, "O_NOFOLLOW", 0)
    if type(flag) is not int or flag == 0:
        raise EvidenceError("this platform does not provide O_NOFOLLOW")
    return flag


def _directory_flags() -> int:
    return os.O_RDONLY | os.O_DIRECTORY | _nofollow_flag()


def _open_start(absolute: bool) -> int:
    return os.open("/" if absolute else ".", _directory_flags())


def _open_or_create_dir(parent_fd: int, component: str) -> int:
    if component in ("", ".", "..") or "/" in component or "\x00" in component:
        raise EvidenceError(f"unsafe internal directory component: {component!r}")
    try:
        os.mkdir(component, 0o700, dir_fd=parent_fd)
    except FileExistsError:
        pass
    except OSError as error:
        raise EvidenceError(f"cannot create evidence directory {component!r}: {error}") from error
    try:
        before = os.stat(component, dir_fd=parent_fd, follow_symlinks=False)
    except OSError as error:
        raise EvidenceError(f"cannot inspect evidence directory {component!r}: {error}") from error
    if not stat.S_ISDIR(before.st_mode):
        raise EvidenceError(f"evidence component is not a real directory: {component!r}")
    try:
        child_fd = os.open(component, _directory_flags(), dir_fd=parent_fd)
    except OSError as error:
        raise EvidenceError(f"evidence directory is not a real no-follow directory: {component!r}") from error
    metadata = os.fstat(child_fd)
    if (
        not stat.S_ISDIR(metadata.st_mode)
        or (before.st_dev, before.st_ino) != (metadata.st_dev, metadata.st_ino)
    ):
        os.close(child_fd)
        raise EvidenceError(f"evidence directory identity changed while opening: {component!r}")
    return child_fd


def open_output_root(raw: str) -> int:
    absolute, pieces = parse_output_root(raw)
    current = _open_start(absolute)
    try:
        for component in pieces:
            child = _open_or_create_dir(current, component)
            os.close(current)
            current = child
        return current
    except BaseException:
        os.close(current)
        raise


def open_internal_tree(root_fd: int, components: Iterable[str]) -> int:
    current = os.dup(root_fd)
    try:
        for component in components:
            child = _open_or_create_dir(current, component)
            os.close(current)
            current = child
        return current
    except BaseException:
        os.close(current)
        raise


def _gate_namespace_entries(gates_fd: int, basename: str) -> set[str]:
    try:
        entries = os.listdir(gates_fd)
    except OSError as error:
        raise EvidenceError(f"cannot inventory gate evidence directory: {error}") from error
    visible_prefix = f"{basename}."
    internal_prefix = f".{basename}."
    return {
        entry
        for entry in entries
        if (
            entry == basename
            or entry.startswith(visible_prefix)
            or entry.startswith(internal_prefix)
        )
    }


def validate_gate_namespace(gates_fd: int, basename: str, state: str) -> None:
    """Require one exact gate namespace state using only its directory FD."""

    validate_gate_name(basename)
    finals = {f"{basename}.{suffix}" for suffix in ("json", "stdout", "stderr")}
    reservation = f".{basename}.reservation"
    expected_by_state = {
        "empty": set(),
        "reserved": {reservation},
        "published": finals | {reservation},
        "complete": finals,
    }
    if state not in expected_by_state:
        raise EvidenceError(f"unknown gate namespace state: {state!r}")
    expected = expected_by_state[state]
    entries = _gate_namespace_entries(gates_fd, basename)
    if entries != expected:
        raise EvidenceError(
            f"gate namespace is not {state}: "
            f"expected={sorted(expected)!r} actual={sorted(entries)!r}"
        )
    opened: list[int] = []
    try:
        for entry in sorted(expected):
            try:
                fd = os.open(entry, os.O_RDONLY | _nofollow_flag(), dir_fd=gates_fd)
            except OSError as error:
                raise EvidenceError(f"gate namespace entry is not no-follow readable: {entry!r}") from error
            opened.append(fd)
            metadata = os.fstat(fd)
            if not stat.S_ISREG(metadata.st_mode):
                raise EvidenceError(f"gate namespace entry is not regular: {entry!r}")
        if _gate_namespace_entries(gates_fd, basename) != expected:
            raise EvidenceError(f"gate namespace changed while validating state {state}")
    finally:
        close_fds(*opened)


@dataclass
class GateReservation:
    directory_fd: int
    basename: str
    reservation_name: str
    active: bool = True

    def release(self) -> None:
        if self.active:
            try:
                os.unlink(self.reservation_name, dir_fd=self.directory_fd)
                os.fsync(self.directory_fd)
            finally:
                self.active = False

    def close(self) -> None:
        try:
            self.release()
        finally:
            os.close(self.directory_fd)


def reserve_gate_triplet(gates_fd: int, basename: str) -> GateReservation:
    validate_gate_name(basename)
    validate_gate_namespace(gates_fd, basename, "empty")
    reservation_name = f".{basename}.reservation"
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | _nofollow_flag()
    try:
        reservation_fd = os.open(reservation_name, flags, 0o600, dir_fd=gates_fd)
    except FileExistsError as error:
        raise EvidenceError(f"gate evidence is concurrently reserved: {basename}") from error
    except OSError as error:
        raise EvidenceError(f"cannot reserve gate evidence {basename!r}: {error}") from error
    try:
        os.fchmod(reservation_fd, 0o600)
        os.fsync(reservation_fd)
    finally:
        os.close(reservation_fd)
    os.fsync(gates_fd)
    try:
        validate_gate_namespace(gates_fd, basename, "reserved")
    except BaseException:
        try:
            os.unlink(reservation_name, dir_fd=gates_fd)
        finally:
            os.fsync(gates_fd)
        raise
    return GateReservation(os.dup(gates_fd), basename, reservation_name)


def _write_all(fd: int, body: bytes) -> None:
    view = memoryview(body)
    while view:
        written = os.write(fd, view)
        if written <= 0:
            raise EvidenceError("short write while publishing evidence")
        view = view[written:]


def publish_new_at(parent_fd: int, final_name: str, body: bytes) -> None:
    """Publish one mode-0600 file using temp+fsync+hardlink create-new."""

    if (
        type(final_name) is not str
        or not final_name
        or final_name in (".", "..")
        or "/" in final_name
        or "\x00" in final_name
    ):
        raise EvidenceError(f"unsafe evidence filename: {final_name!r}")
    if type(body) is not bytes:
        raise EvidenceError("published evidence body must be bytes")
    temporary = f".{final_name}.tmp-{os.getpid()}-{secrets.token_hex(12)}"
    flags = os.O_RDWR | os.O_CREAT | os.O_EXCL | _nofollow_flag()
    temporary_created = False
    temporary_fd: int | None = None
    final_fd: int | None = None
    try:
        temporary_fd = os.open(temporary, flags, 0o600, dir_fd=parent_fd)
        temporary_created = True
        os.fchmod(temporary_fd, 0o600)
        _write_all(temporary_fd, body)
        os.fsync(temporary_fd)
        temporary_metadata = os.fstat(temporary_fd)
        if (
            not stat.S_ISREG(temporary_metadata.st_mode)
            or stat.S_IMODE(temporary_metadata.st_mode) != 0o600
            or temporary_metadata.st_size != len(body)
        ):
            raise EvidenceError("temporary evidence file metadata is invalid")
        try:
            os.link(
                temporary,
                final_name,
                src_dir_fd=parent_fd,
                dst_dir_fd=parent_fd,
                follow_symlinks=False,
            )
        except FileExistsError as error:
            raise EvidenceError(f"evidence file already exists: {final_name}") from error
        final_fd = os.open(
            final_name,
            os.O_RDONLY | _nofollow_flag(),
            dir_fd=parent_fd,
        )
        final_metadata = os.fstat(final_fd)
        named_metadata = os.stat(final_name, dir_fd=parent_fd, follow_symlinks=False)
        expected_identity = (temporary_metadata.st_dev, temporary_metadata.st_ino)
        if (
            (final_metadata.st_dev, final_metadata.st_ino) != expected_identity
            or (named_metadata.st_dev, named_metadata.st_ino) != expected_identity
            or not stat.S_ISREG(final_metadata.st_mode)
            or stat.S_IMODE(final_metadata.st_mode) != 0o600
            or final_metadata.st_size != len(body)
        ):
            raise EvidenceError(f"published evidence identity or metadata mismatch: {final_name}")
        expected_digest = hashlib.sha256(body).digest()
        actual_digest = hashlib.sha256()
        offset = 0
        while True:
            chunk = os.read(final_fd, 65536)
            if not chunk:
                break
            if body[offset : offset + len(chunk)] != chunk:
                raise EvidenceError(f"published evidence bytes mismatch: {final_name}")
            actual_digest.update(chunk)
            offset += len(chunk)
        if offset != len(body) or actual_digest.digest() != expected_digest:
            raise EvidenceError(f"published evidence hash or length mismatch: {final_name}")
        os.fsync(final_fd)
        os.fsync(parent_fd)
    except EvidenceError:
        raise
    except OSError as error:
        raise EvidenceError(f"cannot publish evidence file {final_name!r}: {error}") from error
    finally:
        close_fds(final_fd, temporary_fd)
        if temporary_created:
            try:
                os.unlink(temporary, dir_fd=parent_fd)
                os.fsync(parent_fd)
            except FileNotFoundError:
                pass


def sha256_bytes(body: bytes) -> str:
    return hashlib.sha256(body).hexdigest()


def candidate_relative_log_path(run_binding: str, gate_name: str, stream: str) -> str:
    validate_run_binding(run_binding)
    validate_gate_name(gate_name)
    if stream not in ("stdout", "stderr"):
        raise EvidenceError(f"invalid gate stream: {stream!r}")
    return f"{run_binding}/gates/{gate_name}.{stream}"


def gate_stream_record(
    drained: DrainedStream,
    run_binding: str,
    gate_name: str,
    stream_name: str,
) -> dict[str, Any]:
    if not isinstance(drained, DrainedStream):
        raise EvidenceError("gate stream must be a DrainedStream")
    return {
        "total_bytes": drained.total_bytes,
        "sha256": drained.sha256,
        "retained_bytes": len(drained.retained),
        "retained_sha256": drained.retained_sha256,
        "retained_log": candidate_relative_log_path(
            run_binding,
            gate_name,
            stream_name,
        ),
    }


def close_fds(*fds: int | None) -> None:
    for fd in fds:
        if fd is not None:
            try:
                os.close(fd)
            except OSError:
                pass


def resolved_cwd() -> str:
    return str(Path.cwd().resolve(strict=True))
