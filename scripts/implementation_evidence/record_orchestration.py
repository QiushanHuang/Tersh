#!/usr/bin/env python3
"""Isolated, non-authoritative append-platform recorder client."""

from __future__ import annotations

import argparse
import importlib.util
import re
import sys
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


class RecorderError(core.EvidenceError):
    pass


class ClosedParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:
        raise RecorderError(f"arguments: {message}")


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
            raise RecorderError(
                f"arguments: {option_string} may be supplied only once"
            )
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
        prog="record_orchestration.py",
        allow_abbrev=False,
        description="Append one Host-built platform orchestration record.",
    )
    commands = parser.add_subparsers(dest="operation", required=True)
    append = commands.add_parser("append-platform", allow_abbrev=False)
    for option in (
        "context-handle",
        "invocation-handle",
        "response-handle",
    ):
        append.add_argument(
            f"--{option}",
            action=SingleUseAction,
            required=True,
        )
    append.add_argument(
        "--host-store-fd",
        action=SingleUseAction,
        type=_parse_fd,
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
    for field in (
        "context_handle",
        "invocation_handle",
        "response_handle",
    ):
        try:
            core.validate_sha256(getattr(arguments, field), field.replace("_", " "))
        except core.EvidenceError as error:
            raise RecorderError(f"invalid {field.replace('_', ' ')}") from error
    return arguments


def main(argv: Sequence[str] | None = None) -> int:
    try:
        arguments = _parse_arguments(argv)
        authenticated = core.open_authenticated_host_store_socket(
            arguments.host_store_fd
        )
        try:
            result = core.append_platform_on_authenticated_socket(
                authenticated,
                arguments.context_handle,
                arguments.invocation_handle,
                arguments.response_handle,
            )
        finally:
            authenticated.close()
        sys.stdout.write(core.canonical_json_bytes(result).decode("utf-8"))
        return 0
    except SystemExit as error:
        return error.code if type(error.code) is int else 2
    except (RecorderError, core.EvidenceError) as error:
        sys.stderr.write(f"record-orchestration: {_bounded_diagnostic(error)}\n")
        return 2


def _isolated_cli_main() -> int:
    if not (
        sys.flags.isolated == 1
        and sys.flags.no_site == 1
        and sys.flags.dont_write_bytecode == 1
    ):
        sys.stderr.write(
            "record-orchestration: executable use requires Python -I -S -B\n"
        )
        return 2
    return main()


if __name__ == "__main__":
    raise SystemExit(_isolated_cli_main())
