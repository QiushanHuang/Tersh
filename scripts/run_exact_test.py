#!/usr/bin/env python3
"""Run one named Rust test only after proving its exact libtest inventory."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from typing import Any, Sequence


CASE_PREFIX = "tersh-case-count-v1 "
CASE_KEYS = {
    "matrix",
    "expected_ids",
    "executed_ids",
    "expected_count",
    "executed_count",
}
DIAGNOSTIC_LIMIT = 4096
SUMMARY_RE = re.compile(
    r"^test result: ok\. "
    r"(?P<passed>[0-9]+) passed; "
    r"(?P<failed>[0-9]+) failed; "
    r"(?P<ignored>[0-9]+) ignored; "
    r"(?P<measured>[0-9]+) measured; "
    r"(?P<filtered>[0-9]+) filtered out; "
    r"finished in .+$"
)


@dataclass(frozen=True)
class Rejection(Exception):
    code: str
    detail: str


class ContractArgumentParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:
        raise Rejection("arguments", message)


def canonical_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def diagnostic_text(value: str) -> str:
    single_line = value.replace("\r", "\\r").replace("\n", "\\n")
    if len(single_line) > DIAGNOSTIC_LIMIT:
        return single_line[:DIAGNOSTIC_LIMIT] + "...[truncated]"
    return single_line


def child_excerpt(output: bytes) -> str:
    return diagnostic_text(output[:DIAGNOSTIC_LIMIT].decode("utf-8", errors="replace"))


def run_child(argv: Sequence[str], phase: str) -> str:
    try:
        completed = subprocess.run(
            list(argv),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
    except OSError as error:
        raise Rejection(f"{phase}-spawn", str(error)) from error

    if completed.returncode < 0:
        raise Rejection(
            f"{phase}-signal",
            f"child terminated by signal {-completed.returncode}; stderr={child_excerpt(completed.stderr)}",
        )
    if completed.returncode != 0:
        raise Rejection(
            f"{phase}-exit",
            f"child exited {completed.returncode}; stderr={child_excerpt(completed.stderr)}",
        )
    try:
        return completed.stdout.decode("utf-8", errors="strict")
    except UnicodeDecodeError as error:
        raise Rejection(f"{phase}-output", "child stdout is not UTF-8") from error


def parse_arguments(argv: Sequence[str] | None) -> argparse.Namespace:
    parser = ContractArgumentParser(prog="run_exact_test.py")
    parser.add_argument("--test", dest="integration_targets", action="append", default=[])
    parser.add_argument("--lib", dest="lib_count", action="count", default=0)
    parser.add_argument("--name", required=True)
    parser.add_argument("--ignored", action="store_true")
    parser.add_argument("--serial", action="store_true")
    parser.add_argument("--case-matrix")
    parser.add_argument("--expect-case", action="append", default=[])
    parser.add_argument("--cargo-bin", default="cargo")
    arguments = parser.parse_args(argv)

    selector_count = len(arguments.integration_targets) + arguments.lib_count
    if selector_count != 1:
        raise Rejection("selector", "supply exactly one of --test TARGET or --lib")
    arguments.integration_target = (
        arguments.integration_targets[0] if arguments.integration_targets else None
    )
    arguments.lib = arguments.lib_count == 1
    if arguments.integration_target == "":
        raise Rejection("selector", "--test TARGET must not be empty")
    if arguments.name == "":
        raise Rejection("arguments", "--name must not be empty")
    if arguments.cargo_bin == "":
        raise Rejection("arguments", "--cargo-bin must not be empty")

    if arguments.case_matrix is None:
        if arguments.expect_case:
            raise Rejection("case-config", "--expect-case requires --case-matrix")
    else:
        expected = arguments.expect_case
        if arguments.case_matrix == "" or not expected:
            raise Rejection("case-config", "a matrix requires at least one expected case")
        if any(case_id == "" for case_id in expected) or len(set(expected)) != len(expected):
            raise Rejection("case-config", "expected case IDs must be nonempty and unique")
    return arguments


def selector_and_commands(arguments: argparse.Namespace) -> tuple[dict[str, Any], list[str], list[str]]:
    if arguments.lib:
        selector = {"kind": "lib", "target": None}
        cargo_selector = ["--lib"]
    else:
        selector = {"kind": "integration", "target": arguments.integration_target}
        cargo_selector = ["--test", arguments.integration_target]

    list_argv = [arguments.cargo_bin, "test", "--locked", *cargo_selector, "--", "--list"]
    execute_argv = [
        arguments.cargo_bin,
        "test",
        "--locked",
        *cargo_selector,
        arguments.name,
        "--",
        "--exact",
        "--nocapture",
    ]
    if arguments.ignored:
        execute_argv.append("--ignored")
    if arguments.serial:
        execute_argv.append("--test-threads=1")
    return selector, list_argv, execute_argv


def require_exact_discovery(output: str, name: str) -> None:
    discovered_names = [line[:-6] for line in output.splitlines() if line.endswith(": test")]
    if discovered_names.count(name) != 1:
        raise Rejection(
            "discovery-count",
            f"expected one complete-name discovery for {name!r}, found {discovered_names.count(name)}",
        )


def require_exact_execution(output: str, name: str) -> None:
    summaries = [match for line in output.splitlines() if (match := SUMMARY_RE.fullmatch(line))]
    if not summaries:
        raise Rejection("execution-summary", "missing a complete successful libtest summary")
    if len(summaries) != 1:
        raise Rejection("execution-proof", f"expected one libtest summary, found {len(summaries)}")

    counts = {key: int(value) for key, value in summaries[0].groupdict().items()}
    if (
        counts["passed"] != 1
        or counts["failed"] != 0
        or counts["ignored"] != 0
        or counts["measured"] != 0
    ):
        raise Rejection("execution-summary", "libtest did not report exactly one executed passing test")

    exact_line = re.compile(rf"^test {re.escape(name)} \.\.\. ok$")
    executed_count = sum(exact_line.fullmatch(line) is not None for line in output.splitlines())
    if executed_count != 1:
        raise Rejection(
            "execution-proof",
            f"expected one exact execution proof for {name!r}, found {executed_count}",
        )


def validate_case_record(output: str, arguments: argparse.Namespace) -> dict[str, Any] | None:
    payloads = [line[len(CASE_PREFIX) :] for line in output.splitlines() if line.startswith(CASE_PREFIX)]
    if arguments.case_matrix is None:
        if payloads:
            raise Rejection("unexpected-case-record", "test emitted a case record without --case-matrix")
        return None
    if len(payloads) != 1:
        raise Rejection("case-record-count", f"expected one case record, found {len(payloads)}")

    payload = payloads[0]
    try:
        record = json.loads(payload)
    except (json.JSONDecodeError, UnicodeDecodeError) as error:
        raise Rejection("case-record-json", "case record is not valid JSON") from error
    if canonical_json(record) != payload:
        raise Rejection("case-record-canonical", "case record is not sorted compact canonical JSON")

    expected = arguments.expect_case
    valid_shape = (
        isinstance(record, dict)
        and set(record) == CASE_KEYS
        and type(record.get("matrix")) is str
        and type(record.get("expected_ids")) is list
        and type(record.get("executed_ids")) is list
        and all(type(case_id) is str for case_id in record.get("expected_ids", []))
        and all(type(case_id) is str for case_id in record.get("executed_ids", []))
        and type(record.get("expected_count")) is int
        and type(record.get("executed_count")) is int
        and record.get("matrix") == arguments.case_matrix
        and record.get("expected_ids") == expected
        and record.get("executed_ids") == expected
        and record.get("expected_count") == len(expected)
        and record.get("executed_count") == len(expected)
        and record.get("expected_count", 0) > 0
        and record.get("executed_count", 0) > 0
    )
    if not valid_shape:
        raise Rejection("case-record-schema", "case record does not match the frozen external matrix")
    return record


def main(argv: Sequence[str] | None = None) -> int:
    try:
        arguments = parse_arguments(argv)
        selector, list_argv, execute_argv = selector_and_commands(arguments)
        list_output = run_child(list_argv, "list")
        if any(line.startswith(CASE_PREFIX) for line in list_output.splitlines()):
            raise Rejection(
                "unexpected-list-case-record",
                "case records are forbidden during test discovery",
            )
        require_exact_discovery(list_output, arguments.name)
        execute_output = run_child(execute_argv, "execute")
        require_exact_execution(execute_output, arguments.name)
        case_matrix = validate_case_record(execute_output, arguments)
        result = {
            "schema": "tersh-exact-test-v1",
            "selector": selector,
            "name": arguments.name,
            "discovered_count": 1,
            "executed_count": 1,
            "case_matrix": case_matrix,
        }
        sys.stdout.write(canonical_json(result) + "\n")
        return 0
    except Rejection as error:
        sys.stderr.write(
            f"run_exact_test: {error.code}: {diagnostic_text(error.detail)}\n"
        )
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
