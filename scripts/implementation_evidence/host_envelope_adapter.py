#!/usr/bin/env python3
"""Thin fail-closed CLI for one authenticated Host Envelope capture."""

from __future__ import annotations

import argparse
import importlib.util
import re
import sys
import time
from pathlib import Path
from typing import Any, Sequence


HARNESS_ROOT = Path(__file__).resolve().parents[2]
CORE_PATH = HARNESS_ROOT / "scripts" / "evidence_core.py"
CORE_MODULE_NAME = "scripts.evidence_core"


def _load_trusted_core() -> Any:
    spec = importlib.util.spec_from_file_location(CORE_MODULE_NAME, CORE_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot construct trusted evidence core module spec")
    module = importlib.util.module_from_spec(spec)
    previous = sys.modules.get(CORE_MODULE_NAME)
    sys.modules[CORE_MODULE_NAME] = module
    try:
        spec.loader.exec_module(module)
    finally:
        if previous is None:
            sys.modules.pop(CORE_MODULE_NAME, None)
        else:
            sys.modules[CORE_MODULE_NAME] = previous
    return module


core = _load_trusted_core()


DIAGNOSTIC_LIMIT = 4096
FD_RE = re.compile(r"^[0-9]+$")
MAX_POSIX_FD = (1 << 31) - 1


class AdapterError(core.EvidenceError):
    pass


class ClosedParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:
        raise AdapterError(f"arguments: {message}")


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
            raise AdapterError(f"arguments: {option_string} may be supplied only once")
        seen.add(self.dest)
        namespace._single_use = seen
        setattr(namespace, self.dest, values)


def _parse_fd(raw: str) -> int:
    if FD_RE.fullmatch(raw) is None:
        raise argparse.ArgumentTypeError("must be a decimal file descriptor")
    if len(raw) > 10 or (len(raw) == 10 and raw > str(MAX_POSIX_FD)):
        raise argparse.ArgumentTypeError("file descriptor exceeds the POSIX int range")
    return int(raw)


def _parser() -> ClosedParser:
    parser = ClosedParser(
        prog="host_envelope_adapter.py",
        allow_abbrev=False,
        description="Capture one platform Host Envelope transaction.",
    )
    subparsers = parser.add_subparsers(dest="operation", required=True)
    for operation in ("capture-context", "capture-invocation", "capture-response"):
        command = subparsers.add_parser(operation, allow_abbrev=False)
        command.add_argument(
            "--host-store-fd",
            action=SingleUseAction,
            type=_parse_fd,
            required=True,
        )
        if operation != "capture-context":
            command.add_argument(
                "--context-handle",
                action=SingleUseAction,
                required=True,
            )
    return parser


def _bounded_diagnostic(error: object) -> str:
    message = str(error).replace("\r", "\\r").replace("\n", "\\n")
    encoded = message.encode("utf-8", errors="backslashreplace")
    suffix = b"...[truncated]"
    if len(encoded) > DIAGNOSTIC_LIMIT:
        prefix = encoded[: DIAGNOSTIC_LIMIT - len(suffix)]
        encoded = prefix.decode("utf-8", errors="ignore").encode("utf-8") + suffix
    return encoded.decode("utf-8")


def _parse_arguments(argv: Sequence[str] | None) -> argparse.Namespace:
    arguments = _parser().parse_args(list(sys.argv[1:] if argv is None else argv))
    core.require_exact_int(arguments.host_store_fd, "host store FD", minimum=3)
    context_handle = getattr(arguments, "context_handle", None)
    if arguments.operation == "capture-context":
        if context_handle is not None:
            raise AdapterError("capture-context does not accept a context handle")
    else:
        core.validate_sha256(context_handle, "context handle")
    return arguments


def main(argv: Sequence[str] | None = None) -> int:
    try:
        arguments = _parse_arguments(argv)
        authenticated = core.open_authenticated_host_store_socket(
            arguments.host_store_fd
        )
        try:
            result = core.capture_host_envelope_on_authenticated_socket(
                authenticated,
                arguments.operation,
                getattr(arguments, "context_handle", None),
                deadline=time.monotonic() + core.HOST_TRANSACTION_TIMEOUT_SECONDS,
            )
        finally:
            authenticated.close()
        sys.stdout.write(core.canonical_json_bytes(result).decode("utf-8"))
        return 0
    except SystemExit as error:
        return error.code if type(error.code) is int else 2
    except (AdapterError, core.EvidenceError) as error:
        sys.stderr.write(f"host-envelope-adapter: {_bounded_diagnostic(error)}\n")
        return 2


def _isolated_cli_main() -> int:
    """Reject executable use unless the trusted supervisor isolated Python."""

    if not (
        sys.flags.isolated == 1
        and sys.flags.no_site == 1
        and sys.flags.dont_write_bytecode == 1
    ):
        sys.stderr.write(
            "host-envelope-adapter: executable use requires Python -I -S -B\n"
        )
        return 2
    return main()


if __name__ == "__main__":
    raise SystemExit(_isolated_cli_main())
