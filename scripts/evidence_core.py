"""Shared fail-closed primitives for implementation and hardening evidence.

This module intentionally has no command-line entry point.  Path creation and
publication are dirfd-relative so validation is not followed by a pathname
re-open that could traverse a substituted symlink.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import secrets
import stat
import subprocess
import threading
from dataclasses import dataclass
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
DEFAULT_RETAIN_LIMIT = 1024 * 1024


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
) -> None:
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
        try:
            pipe.close()
        except BaseException as error:
            errors.append(error)


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
    try:
        child = subprocess.Popen(
            command,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            shell=False,
        )
    except OSError as error:
        raise EvidenceError(f"child could not start: {error}") from error
    if child.stdout is None or child.stderr is None:
        child.kill()
        child.wait()
        raise EvidenceError("child pipes were not created")

    drained: dict[str, DrainedStream] = {}
    errors: list[BaseException] = []
    readers = [
        threading.Thread(
            target=_drain_stream,
            args=(child.stdout, drained, "stdout", errors, retain_limit),
        ),
        threading.Thread(
            target=_drain_stream,
            args=(child.stderr, drained, "stderr", errors, retain_limit),
        ),
    ]
    for reader in readers:
        reader.start()
    returncode = child.wait()
    for reader in readers:
        reader.join()
    if errors:
        raise EvidenceError(f"child output drain failed: {errors[0]}")
    if set(drained) != {"stdout", "stderr"}:
        raise EvidenceError("child output drain did not produce both streams")
    return returncode, drained["stdout"], drained["stderr"]


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


def _entry_exists(parent_fd: int, name: str) -> bool:
    try:
        os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
    except FileNotFoundError:
        return False
    except OSError as error:
        raise EvidenceError(f"cannot inspect evidence entry {name!r}: {error}") from error
    return True


def _gate_namespace_extras(
    gates_fd: int,
    basename: str,
    *,
    allow_reservation: bool,
) -> list[str]:
    finals = {f"{basename}.{suffix}" for suffix in ("json", "stdout", "stderr")}
    allowed = set(finals)
    if allow_reservation:
        allowed.add(f".{basename}.reservation")
    try:
        entries = os.listdir(gates_fd)
    except OSError as error:
        raise EvidenceError(f"cannot inventory gate evidence directory: {error}") from error
    visible_prefix = f"{basename}."
    internal_prefix = f".{basename}."
    return sorted(
        entry
        for entry in entries
        if (
            entry == basename
            or entry.startswith(visible_prefix)
            or entry.startswith(internal_prefix)
        )
        and entry not in allowed
    )


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
    finals = tuple(f"{basename}.{suffix}" for suffix in ("json", "stdout", "stderr"))
    extras = _gate_namespace_extras(gates_fd, basename, allow_reservation=False)
    if extras:
        raise EvidenceError(f"unexpected gate evidence entries: {', '.join(extras)}")
    colliding = [name for name in finals if _entry_exists(gates_fd, name)]
    if colliding:
        raise EvidenceError(f"gate evidence already exists: {', '.join(colliding)}")
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
    extras = _gate_namespace_extras(gates_fd, basename, allow_reservation=True)
    colliding = [name for name in finals if _entry_exists(gates_fd, name)]
    if extras or colliding:
        try:
            os.unlink(reservation_name, dir_fd=gates_fd)
        finally:
            os.fsync(gates_fd)
        if extras:
            raise EvidenceError(f"unexpected gate evidence entries appeared: {', '.join(extras)}")
        raise EvidenceError(f"gate evidence appeared during reservation: {', '.join(colliding)}")
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
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | _nofollow_flag()
    temporary_created = False
    try:
        temporary_fd = os.open(temporary, flags, 0o600, dir_fd=parent_fd)
        temporary_created = True
        try:
            os.fchmod(temporary_fd, 0o600)
            _write_all(temporary_fd, body)
            os.fsync(temporary_fd)
        finally:
            os.close(temporary_fd)
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
        os.fsync(parent_fd)
    except EvidenceError:
        raise
    except OSError as error:
        raise EvidenceError(f"cannot publish evidence file {final_name!r}: {error}") from error
    finally:
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
