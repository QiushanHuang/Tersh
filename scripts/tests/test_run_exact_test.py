import json
import os
import pathlib
import stat
import subprocess
import sys
import tempfile
import textwrap
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
RUNNER = ROOT / "scripts" / "run_exact_test.py"


FAKE_CARGO = r'''#!/usr/bin/env python3
import json
import os
import pathlib
import signal
import sys

args = sys.argv[1:]
with pathlib.Path(os.environ["FAKE_CARGO_LOG"]).open("a", encoding="utf-8") as stream:
    stream.write(json.dumps(args, separators=(",", ":")) + "\n")

mode = os.environ.get("FAKE_CARGO_MODE", "ok")
name = os.environ.get("FAKE_CARGO_NAME", "suite::works")
is_list = args[-2:] == ["--", "--list"]

if is_list:
    if mode == "nonzero-list":
        print("synthetic list failure", file=sys.stderr)
        raise SystemExit(9)
    if mode == "signal-list":
        os.kill(os.getpid(), signal.SIGTERM)
    if mode == "missing":
        names = []
    elif mode == "duplicate":
        names = [name, name]
    elif mode == "near-name":
        names = [name + "-near"]
    else:
        names = [name]
    for payload in json.loads(os.environ.get("FAKE_CARGO_LIST_CASE_PAYLOADS", "[]")):
        print("tersh-case-count-v1 " + payload)
    for listed_name in names:
        print(f"{listed_name}: test")
    print(f"{len(names)} tests, 0 benchmarks")
    raise SystemExit(0)

if mode == "nonzero-execute":
    print("synthetic execution failure", file=sys.stderr)
    raise SystemExit(8)
if mode == "signal-execute":
    os.kill(os.getpid(), signal.SIGTERM)

for payload in json.loads(os.environ.get("FAKE_CARGO_CASE_PAYLOADS", "[]")):
    print("tersh-case-count-v1 " + payload)

if mode == "zero-executed":
    print("test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s")
elif mode == "ignored-result":
    print(f"test {name} ... ignored")
    print("test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s")
elif mode == "malformed-summary":
    print(f"test {name} ... ok")
    print("finished without a libtest summary")
elif mode == "wrong-executed-name":
    print(f"test {name}-near ... ok")
    print("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s")
elif mode == "duplicate-executed-name":
    print(f"test {name} ... ok")
    print(f"test {name} ... ok")
    print("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s")
elif mode == "duplicate-summary":
    print(f"test {name} ... ok")
    summary = "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s"
    print(summary)
    print(summary)
else:
    print(f"test {name} ... ok")
    print("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s")
'''


def canonical_json(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


class ExactRunnerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.cargo = self.root / "fake cargo;not-a-shell"
        self.cargo.write_text(textwrap.dedent(FAKE_CARGO), encoding="utf-8")
        self.cargo.chmod(self.cargo.stat().st_mode | stat.S_IXUSR)
        self.log = self.root / "argv.jsonl"

    def run_runner(
        self, *extra, mode="ok", name="suite::works", case_payloads=None,
        list_case_payloads=None,
    ):
        env = os.environ.copy()
        env.update(
            FAKE_CARGO_LOG=str(self.log),
            FAKE_CARGO_MODE=mode,
            FAKE_CARGO_NAME=name,
            FAKE_CARGO_CASE_PAYLOADS=json.dumps(case_payloads or []),
            FAKE_CARGO_LIST_CASE_PAYLOADS=json.dumps(list_case_payloads or []),
            PYTHONDONTWRITEBYTECODE="1",
        )
        return subprocess.run(
            [sys.executable, str(RUNNER), *extra, "--cargo-bin", str(self.cargo)],
            cwd=ROOT,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

    def argv(self):
        if not self.log.exists():
            return []
        return [json.loads(line) for line in self.log.read_text(encoding="utf-8").splitlines()]

    def assert_rejected(self, result, code, expected_call_count):
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertIn(f"run_exact_test: {code}:", result.stderr)
        self.assertNotIn("can't open file", result.stderr)
        self.assertNotIn("Traceback", result.stderr)
        self.assertEqual(len(self.argv()), expected_call_count)

    def success_record(self, result):
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(result.stdout.endswith("\n"))
        self.assertEqual(len(result.stdout.splitlines()), 1)
        parsed = json.loads(result.stdout)
        self.assertEqual(result.stdout, canonical_json(parsed) + "\n")
        self.assertEqual(result.stderr, "")
        return parsed

    def case_record(self, expected, executed=None, **overrides):
        executed = expected if executed is None else executed
        record = {
            "matrix": "frozen-v1",
            "expected_ids": expected,
            "executed_ids": executed,
            "expected_count": len(expected),
            "executed_count": len(executed),
        }
        record.update(overrides)
        return record

    def test_exact_runner_requires_one_discovered_and_one_executed(self):
        result = self.run_runner("--test", "fixture", "--name", "suite::works")
        record = self.success_record(result)
        self.assertEqual(
            record,
            {
                "case_matrix": None,
                "discovered_count": 1,
                "executed_count": 1,
                "name": "suite::works",
                "schema": "tersh-exact-test-v1",
                "selector": {"kind": "integration", "target": "fixture"},
            },
        )

        for mode in ("near-name", "wrong-executed-name", "duplicate-executed-name", "duplicate-summary"):
            with self.subTest(mode=mode):
                self.log.unlink(missing_ok=True)
                rejected = self.run_runner("--test", "fixture", "--name", "suite::works", mode=mode)
                code = "discovery-count" if mode == "near-name" else "execution-proof"
                calls = 1 if mode == "near-name" else 2
                self.assert_rejected(rejected, code, calls)

    def test_exact_runner_rejects_missing_duplicate_ignored_or_zero_without_explicit_flags(self):
        cases = (
            ("missing", "discovery-count", 1),
            ("duplicate", "discovery-count", 1),
            ("ignored-result", "execution-summary", 2),
            ("zero-executed", "execution-summary", 2),
            ("malformed-summary", "execution-summary", 2),
            ("nonzero-list", "list-exit", 1),
            ("signal-list", "list-signal", 1),
            ("nonzero-execute", "execute-exit", 2),
            ("signal-execute", "execute-signal", 2),
        )
        for mode, code, calls in cases:
            with self.subTest(mode=mode):
                self.log.unlink(missing_ok=True)
                result = self.run_runner("--test", "fixture", "--name", "suite::works", mode=mode)
                self.assert_rejected(result, code, calls)

    def test_exact_runner_passes_argv_without_a_shell_and_serializes_when_requested(self):
        result = self.run_runner(
            "--test", "fixture", "--name", "suite::works", "--ignored", "--serial"
        )
        self.success_record(result)
        self.assertEqual(
            self.argv(),
            [
                ["test", "--locked", "--test", "fixture", "--", "--list"],
                [
                    "test", "--locked", "--test", "fixture", "suite::works", "--",
                    "--exact", "--nocapture", "--ignored", "--test-threads=1",
                ],
            ],
        )

    def test_exact_runner_supports_private_lib_tests_and_rejects_mixed_selectors(self):
        name = "module::tests::works"
        result = self.run_runner("--lib", "--name", name, name=name)
        record = self.success_record(result)
        self.assertEqual(record["selector"], {"kind": "lib", "target": None})

        self.log.unlink(missing_ok=True)
        mixed = self.run_runner("--lib", "--test", "fixture", "--name", name, name=name)
        self.assert_rejected(mixed, "selector", 0)

    def test_exact_runner_lib_lists_then_exactly_executes_one_crate_private_test(self):
        name = "module::tests::one_private_test"
        result = self.run_runner("--lib", "--name", name, name=name)
        self.success_record(result)
        self.assertEqual(
            self.argv(),
            [
                ["test", "--locked", "--lib", "--", "--list"],
                ["test", "--locked", "--lib", name, "--", "--exact", "--nocapture"],
            ],
        )

    def test_exact_runner_rejects_both_or_neither_test_and_lib_selector(self):
        invalid_selectors = (
            ("--name", "suite::works"),
            ("--test", "fixture", "--lib", "--name", "suite::works"),
            ("--test", "fixture-a", "--test", "fixture-b", "--name", "suite::works"),
            ("--lib", "--lib", "--name", "suite::works"),
        )
        for argv in invalid_selectors:
            with self.subTest(argv=argv):
                self.log.unlink(missing_ok=True)
                result = self.run_runner(*argv)
                self.assert_rejected(result, "selector", 0)

    def test_exact_runner_lib_rejects_zero_discovered_or_zero_executed(self):
        cases = (
            ("missing", "discovery-count", 1),
            ("zero-executed", "execution-summary", 2),
            ("malformed-summary", "execution-summary", 2),
            ("nonzero-list", "list-exit", 1),
            ("signal-list", "list-signal", 1),
            ("nonzero-execute", "execute-exit", 2),
            ("signal-execute", "execute-signal", 2),
        )
        for mode, code, calls in cases:
            with self.subTest(mode=mode):
                self.log.unlink(missing_ok=True)
                result = self.run_runner("--lib", "--name", "suite::works", mode=mode)
                self.assert_rejected(result, code, calls)

    def test_exact_runner_always_uses_nocapture(self):
        result = self.run_runner("--test", "fixture", "--name", "suite::works")
        self.success_record(result)
        execute = self.argv()[1]
        self.assertEqual(execute[execute.index("--") + 1 :], ["--exact", "--nocapture"])

    def test_exact_runner_validates_frozen_parameter_case_ids_and_count(self):
        cases = ["alpha", "beta"]
        valid = self.case_record(cases)
        result = self.run_runner(
            "--test", "fixture", "--name", "suite::works", "--case-matrix", "frozen-v1",
            "--expect-case", "alpha", "--expect-case", "beta",
            case_payloads=[canonical_json(valid)],
        )
        record = self.success_record(result)
        self.assertEqual(record["case_matrix"], valid)

        invalid_records = (
            self.case_record(cases, matrix="wrong-v1"),
            self.case_record(["beta", "alpha"]),
            self.case_record(cases, expected_count=1),
            self.case_record(cases, executed_count=1),
            {**self.case_record(cases), "extra": True},
        )
        for invalid in invalid_records:
            with self.subTest(invalid=invalid):
                self.log.unlink(missing_ok=True)
                rejected = self.run_runner(
                    "--test", "fixture", "--name", "suite::works", "--case-matrix", "frozen-v1",
                    "--expect-case", "alpha", "--expect-case", "beta",
                    case_payloads=[canonical_json(invalid)],
                )
                self.assert_rejected(rejected, "case-record-schema", 2)

        self.log.unlink(missing_ok=True)
        noncanonical = json.dumps(valid, ensure_ascii=False, sort_keys=False)
        rejected = self.run_runner(
            "--test", "fixture", "--name", "suite::works", "--case-matrix", "frozen-v1",
            "--expect-case", "alpha", "--expect-case", "beta", case_payloads=[noncanonical],
        )
        self.assert_rejected(rejected, "case-record-canonical", 2)

    def test_exact_runner_requires_exactly_one_case_record_and_rejects_it_without_matrix(self):
        valid = canonical_json(self.case_record(["alpha"]))
        cases = (
            ([], True, "case-record-count"),
            ([valid, valid], True, "case-record-count"),
            ([valid], False, "unexpected-case-record"),
            (["{not-json"], True, "case-record-json"),
        )
        for payloads, declare_matrix, code in cases:
            with self.subTest(payloads=payloads, declare_matrix=declare_matrix):
                self.log.unlink(missing_ok=True)
                args = ["--test", "fixture", "--name", "suite::works"]
                if declare_matrix:
                    args += ["--case-matrix", "frozen-v1", "--expect-case", "alpha"]
                result = self.run_runner(*args, case_payloads=payloads)
                self.assert_rejected(result, code, 2)

        for declare_matrix in (False, True):
            with self.subTest(list_marker=True, declare_matrix=declare_matrix):
                self.log.unlink(missing_ok=True)
                args = ["--test", "fixture", "--name", "suite::works"]
                if declare_matrix:
                    args += ["--case-matrix", "frozen-v1", "--expect-case", "alpha"]
                result = self.run_runner(*args, list_case_payloads=[valid])
                self.assert_rejected(result, "unexpected-list-case-record", 1)

    def test_exact_runner_rejects_missing_duplicate_extra_or_reordered_case_ids(self):
        expected = ["alpha", "beta"]
        variants = (
            ["alpha"],
            ["alpha", "alpha"],
            ["alpha", "beta", "gamma"],
            ["beta", "alpha"],
        )
        for executed in variants:
            with self.subTest(executed=executed):
                self.log.unlink(missing_ok=True)
                payload = canonical_json(self.case_record(expected, executed))
                result = self.run_runner(
                    "--test", "fixture", "--name", "suite::works", "--case-matrix", "frozen-v1",
                    "--expect-case", "alpha", "--expect-case", "beta", case_payloads=[payload],
                )
                self.assert_rejected(result, "case-record-schema", 2)

        for cli_cases in ([], ["alpha", "alpha"]):
            with self.subTest(cli_cases=cli_cases):
                self.log.unlink(missing_ok=True)
                args = ["--test", "fixture", "--name", "suite::works", "--case-matrix", "frozen-v1"]
                for case_id in cli_cases:
                    args += ["--expect-case", case_id]
                result = self.run_runner(*args)
                self.assert_rejected(result, "case-config", 0)


if __name__ == "__main__":
    unittest.main()
