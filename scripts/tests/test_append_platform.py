import ast
import copy
import contextlib
import hashlib
import importlib
import inspect
import json
import pathlib
import re
import socket
import struct
import threading
import time
import unittest
from unittest import mock


ROOT = pathlib.Path(__file__).resolve().parents[2]
APPROVED_DESIGN_RELATIVE_PATH = (
    "docs/superpowers/specs/2026-08-11-tersh-append-platform-design.md"
)
APPROVED_DESIGN = ROOT / APPROVED_DESIGN_RELATIVE_PATH
APPROVED_DESIGN_SHA256 = (
    "fc761d1ee4550e14aac10e70211f2b8cd87eab1d2ac3b9ace32aefdb224dade9"
)
OLDER_PLANS = (
    ROOT
    / "docs/superpowers/plans/2026-08-10-tersh-implementation-iteration-evidence.md",
    ROOT
    / "docs/superpowers/plans/2026-08-10-tersh-seven-cycle-hardening-implementation.md",
)
APPROVED_BODY_KINDS = (
    "context",
    "invocation",
    "response",
    "recorder-session",
    "orchestration-record",
)
APPROVED_CONTRACT = (
    "append-platform imports the approved 2026-08-11 design verbatim: exact five "
    "BODYs, Host-built frozen BODY 5, client validation, no client RECORD stream, "
    "Host-ledger linearization, and Host-exclusive formal projection. Operator "
    "attestation is deferred and no formal attest CLI exists in this checkpoint."
)
FORBIDDEN_NORMATIVE_APPEND_PATTERNS = (
    r"append-platform`\s*=\s*`context, invocation, response`",
    r"record_orchestration\.py attest\b",
    r"recorder .*create-new publish.*formal projection",
)
OPERATIVE_BEGIN = "<!-- append-platform-operative:begin -->"
OPERATIVE_END = "<!-- append-platform-operative:end -->"
OPERATIVE_INVARIANT_HEADING = (
    "### Append-Platform Operative Invariants (Normative)"
)
APPROVED_OPERATIVE_INVARIANTS = (
    (
        "body-order",
        "context,invocation,response,recorder-session,orchestration-record",
    ),
    ("body-5-builder", "Host"),
    ("client-record-or-body-upload", "none"),
    ("formal-projection-writer-and-repairer", "Host-only"),
    ("attestation-operation", "none"),
    ("provenance-mode", "platform-envelope"),
)
APPROVED_CAPSULE_LINES = tuple(
    f"- `{key}` = `{value}`" for key, value in APPROVED_OPERATIVE_INVARIANTS
)
APPROVED_CAPSULE_TEXT = "\n".join(APPROVED_CAPSULE_LINES)
SUBORDINATE_GUIDANCE = (
    "The detailed append-platform sequence below is subordinate, non-authoritative "
    "implementation guidance. The pinned design and operative invariant capsule "
    "control if this guidance conflicts."
)
LEGACY_TWO_ARM_PROVENANCE_CLAIM = "both host-envelope provenance modes"
PLAN_REQUIRED_CLAIMS = {
    OLDER_PLANS[0]: (
        "`append-platform` uses exactly `context`, `invocation`, `response`, "
        "`recorder-session`, and `orchestration-record`; and "
        "`enumerate-evidence`"
    ),
    OLDER_PLANS[1]: (
        "single platform-envelope provenance mode, rejection of the deferred "
        "attest operation or missing invocation metadata"
    ),
}
CAPSULE_MUTATIONS = (
    (
        "changed-body-order",
        APPROVED_CAPSULE_TEXT.replace(
            APPROVED_CAPSULE_LINES[0],
            "- `body-order` = `context,invocation,response,recorder-session,"
            "orchestration-record,audit`",
        ),
    ),
    (
        "changed-body-builder",
        APPROVED_CAPSULE_TEXT.replace(
            APPROVED_CAPSULE_LINES[1],
            "- `body-5-builder` = `client`",
        ),
    ),
    (
        "changed-client-upload",
        APPROVED_CAPSULE_TEXT.replace(
            APPROVED_CAPSULE_LINES[2],
            "- `client-record-or-body-upload` = `allowed`",
        ),
    ),
    (
        "changed-projection-owner",
        APPROVED_CAPSULE_TEXT.replace(
            APPROVED_CAPSULE_LINES[3],
            "- `formal-projection-writer-and-repairer` = `Host-and-recorder`",
        ),
    ),
    (
        "changed-attestation",
        APPROVED_CAPSULE_TEXT.replace(
            APPROVED_CAPSULE_LINES[4],
            "- `attestation-operation` = `enabled`",
        ),
    ),
    (
        "changed-provenance-mode",
        APPROVED_CAPSULE_TEXT.replace(
            APPROVED_CAPSULE_LINES[5],
            "- `provenance-mode` = `platform-envelope-or-attest`",
        ),
    ),
    ("missing-line", "\n".join(APPROVED_CAPSULE_LINES[:-1])),
    (
        "reordered-lines",
        "\n".join(
            (APPROVED_CAPSULE_LINES[1], APPROVED_CAPSULE_LINES[0])
            + APPROVED_CAPSULE_LINES[2:]
        ),
    ),
    ("duplicate-line", APPROVED_CAPSULE_TEXT + "\n" + APPROVED_CAPSULE_LINES[0]),
    (
        "additional-conflict",
        APPROVED_CAPSULE_TEXT + "\n- `attestation-operation` = `fallback`",
    ),
)
FORBIDDEN_NORMATIVE_EXAMPLES = (
    ("three-body", "`append-platform` = `context, invocation, response`."),
    ("attest-cli", "Run `record_orchestration.py attest` for formal evidence."),
    (
        "recorder-projection",
        "The recorder may create-new publish the formal projection.",
    ),
)
SESSION_KEYS = {
    "schema", "producer_session_id", "attempt_binding_id",
    "predecessor_attempt_binding_id", "entrypoint", "producer_mode",
    "operation", "context_nonce", "dispatch_id", "evidence_id",
    "evidence_attempt", "run_binding", "candidate", "candidate_tree",
    "worktree_handle", "worktree_observed_at", "baseline_commit",
    "candidate_relation", "bundle_id", "runtime_profile_id",
    "policy_sha256", "policy_entry_id", "policy_entry_sha256",
    "projection_root_class", "record_class", "record_schema",
    "destination", "parent_finding_ids_sha256",
    "next_receipt_sequence", "previous_receipt_id",
}
ORCHESTRATION_RECORD_KEYS = {
    "schema", "evidence_id", "evidence_attempt", "run_binding", "role",
    "wave", "review_attempt", "baseline_commit", "reviewed_commit",
    "parent_finding_ids", "dispatch_id", "agent_id",
    "canonical_task_path", "agent_run_id", "model", "reasoning_effort",
    "dispatched_at", "started_at", "ended_at", "terminal_status",
    "provenance",
}
PRODUCER_RECEIPT_KEYS = {
    "schema", "receipt_id", "attempt_binding_id", "producer_session_id",
    "sequence", "previous_receipt_id", "producer_mode", "entrypoint",
    "bundle_id", "runtime_profile_id", "policy_entry_id",
    "policy_entry_sha256", "environment_capability",
    "projection_root_class", "record_class", "record_schema", "destination",
    "body_sha256", "byte_count", "dispatch_id", "reported_record_sha256",
    "created_at",
}
PRODUCER_RECORD_CLASSES = {
    "attempt-marker", "candidate-marker", "gate", "cumulative-gates",
    "runner-inventory", "before-repo-runs", "bootstrap", "selected-run",
    "jobs", "artifact-index", "external-candidate", "orchestration",
    "review", "implementation-entry", "requirements", "completion-audit",
    "audit-reservation-failure", "orchestration-failure",
    "agent-report-failure",
}


def normalized_paragraph(paragraph):
    return " ".join(line.strip() for line in paragraph.splitlines())


def normative_markdown(text):
    paragraphs = re.split(r"\n\s*\n", text)
    return "\n".join(
        normalized
        for paragraph in paragraphs
        if (
            normalized := normalized_paragraph(paragraph)
        )
        and not normalized.casefold().startswith(
            "> **superseded non-normative context:**"
        )
    )


class AppendPlatformContractTests(unittest.TestCase):
    def assert_authority_pin(self, path, text):
        relative = path.relative_to(ROOT)
        headings = re.findall(
            r"(?m)^## Approved Append-Platform Contract \(Normative\)[ \t]*$",
            text,
        )
        self.assertEqual(
            len(headings),
            1,
            f"{relative} must contain exactly one normative authority section",
        )
        section = re.search(
            r"(?ms)^## Approved Append-Platform Contract \(Normative\)\s*"
            r"(?P<body>.*?)(?=^## |\Z)",
            text,
        )
        self.assertIsNotNone(section)
        body = section.group("body")
        self.assertEqual(body.count(APPROVED_DESIGN_RELATIVE_PATH), 1)
        self.assertEqual(body.count(APPROVED_DESIGN_SHA256), 1)
        authority_text = normalized_paragraph(body)
        self.assertIn(APPROVED_DESIGN_RELATIVE_PATH, authority_text)
        self.assertIn(APPROVED_DESIGN_SHA256, authority_text)
        self.assertIn(APPROVED_CONTRACT, authority_text)

    def assert_exact_operative_capsule(self, path, text):
        relative = path.relative_to(ROOT)
        self.assertEqual(text.count(OPERATIVE_BEGIN), 1)
        self.assertEqual(text.count(OPERATIVE_END), 1)
        marked = re.search(
            rf"(?s){re.escape(OPERATIVE_BEGIN)}\s*(?P<body>.*?)\s*"
            rf"{re.escape(OPERATIVE_END)}",
            text,
        )
        self.assertIsNotNone(marked)
        capsule_lines = tuple(
            line.strip()
            for line in marked.group("body").splitlines()
            if line.strip()
        )
        self.assertEqual(
            capsule_lines,
            APPROVED_CAPSULE_LINES,
            f"{relative} structured operative capsule drifted",
        )
        self.assertEqual(text.count(OPERATIVE_INVARIANT_HEADING), 1)
        self.assertEqual(
            normalized_paragraph(text).count(SUBORDINATE_GUIDANCE),
            1,
            f"{relative} must label detailed prose as subordinate guidance",
        )

    def assert_plan_contract(self, path, text):
        self.assert_authority_pin(path, text)
        self.assert_exact_operative_capsule(path, text)
        relative = path.relative_to(ROOT)
        normative_text = normative_markdown(text)
        self.assertIn(PLAN_REQUIRED_CLAIMS[path], normative_text)
        self.assertNotIn(
            LEGACY_TWO_ARM_PROVENANCE_CLAIM,
            normative_text.casefold(),
            f"{relative} retains positive legacy two-arm provenance acceptance",
        )
        for pattern in FORBIDDEN_NORMATIVE_APPEND_PATTERNS:
            match = re.search(pattern, normative_text, flags=re.IGNORECASE)
            self.assertIsNone(
                match,
                f"{relative} contains obsolete normative append contract matching "
                f"{pattern!r}: {match.group(0)!r}" if match else "",
            )

    def replace_capsule(self, text, replacement):
        return text.replace(APPROVED_CAPSULE_TEXT, replacement, 1)

    def test_approved_append_contract_has_five_bodies_no_record_upload_and_no_attest_arm(
        self,
    ):
        design_bytes = APPROVED_DESIGN.read_bytes()
        self.assertEqual(
            hashlib.sha256(design_bytes).hexdigest(),
            APPROVED_DESIGN_SHA256,
            "approved append-platform design bytes drifted",
        )
        design_text = design_bytes.decode("utf-8")
        body_block = re.search(
            r"The Host sends exactly five BODY wrappers in this order:\s*"
            r"```text\s*(?P<body_kinds>.*?)```",
            design_text,
            flags=re.DOTALL,
        )
        self.assertIsNotNone(body_block)
        self.assertEqual(
            tuple(
                re.findall(
                    r"(?m)^\s*\d+\.\s+([a-z-]+)\s*$",
                    body_block.group("body_kinds"),
                )
            ),
            APPROVED_BODY_KINDS,
        )
        wire_section = re.search(
            r"(?ms)^## Append-Platform Wire Protocol\s*"
            r"(?P<body>.*?)(?=^## |\Z)",
            design_text,
        )
        self.assertIsNotNone(wire_section)
        self.assertRegex(
            normalized_paragraph(wire_section.group("body")),
            r"The client sends no RECORD-BEGIN, RECORD-CHUNK, RECORD-END, body JSON, "
            r"or upload frame\. Any such frame is a protocol error\.",
        )
        for path in OLDER_PLANS:
            self.assert_plan_contract(path, path.read_text(encoding="utf-8"))

    def test_operative_capsule_rejects_changed_missing_reordered_or_extra_lines(self):
        for path in OLDER_PLANS:
            text = path.read_text(encoding="utf-8")
            for label, mutated_capsule in CAPSULE_MUTATIONS:
                with self.subTest(plan=path.name, mutation=label):
                    mutated = self.replace_capsule(text, mutated_capsule)
                    with self.assertRaisesRegex(
                        AssertionError,
                        "structured operative capsule",
                    ):
                        self.assert_plan_contract(path, mutated)

    def test_frozen_forbidden_patterns_reject_normative_examples(self):
        for path in OLDER_PLANS:
            text = path.read_text(encoding="utf-8")
            for label, forbidden in FORBIDDEN_NORMATIVE_EXAMPLES:
                with self.subTest(plan=path.name, mutation=label):
                    with self.assertRaisesRegex(AssertionError, "obsolete normative"):
                        self.assert_plan_contract(path, text + "\n\n" + forbidden)

    def test_authority_pin_allows_later_citations_and_superseded_history(self):
        superseded = (
            "> **Superseded non-normative context:** Historical text used "
            "`append-platform` = `context, invocation, response`, enabled "
            "`record_orchestration.py attest`, and said the recorder could "
            "create-new publish a formal projection."
        )
        for path in OLDER_PLANS:
            with self.subTest(plan=path.name):
                text = path.read_text(encoding="utf-8")
                augmented = (
                    text
                    + "\n\nLater consistent citation: `"
                    + APPROVED_DESIGN_RELATIVE_PATH
                    + "` with SHA-256 `"
                    + APPROVED_DESIGN_SHA256
                    + "`.\n\n"
                    + superseded
                    + "\n"
                )
                self.assert_plan_contract(path, augmented)
                duplicate_authority = (
                    augmented
                    + "\n\n## Approved Append-Platform Contract (Normative)\n\n"
                    + "Conflicting authority: `docs/conflicting.md` with SHA-256 `"
                    + ("0" * 64)
                    + "`.\n"
                )
                with self.assertRaisesRegex(AssertionError, "exactly one"):
                    self.assert_plan_contract(path, duplicate_authority)


class AppendPlatformSchemaTests(unittest.TestCase):
    @contextlib.contextmanager
    def assert_exact_evidence_error(self, error_class):
        with self.assertRaises(error_class) as raised:
            yield
        self.assertIs(type(raised.exception), error_class)

    @staticmethod
    def context_v2(parent_finding_ids, *, evidence_id="impl-01"):
        return {
            "schema": "tersh-host-dispatch-context-v2",
            "context_nonce": "c" * 64,
            "harness_bundle_revision": "7" * 40,
            "harness_bundle_sha256": "8" * 64,
            "evidence_id": evidence_id,
            "evidence_attempt": "002",
            "role": "safety",
            "wave": "wave-c",
            "review_attempt": "001",
            "run_binding": "run-local",
            "baseline_commit": "a" * 40,
            "review_target": "b" * 40,
            "parent_finding_ids": list(parent_finding_ids),
            "canonical_task_path": "/root/safety/reviewer",
            "worktree_handle": "fixture-worktree",
            "requested_model": "gpt-5.6-sol",
            "requested_reasoning_effort": "xhigh",
            "created_at": "2026-08-10T00:00:00.000000001Z",
        }

    def context_v2_validator(self):
        core = importlib.import_module("scripts.evidence_core")
        validate = getattr(core, "validate_dispatch_context_v2", None)
        self.assertTrue(
            callable(validate),
            "validate_dispatch_context_v2 is required for context v2",
        )
        return core, validate

    def recorder_session_validator(self):
        core = importlib.import_module("scripts.evidence_core")
        validate = getattr(
            core,
            "validate_orchestration_recorder_session",
            None,
        )
        self.assertTrue(
            callable(validate),
            "validate_orchestration_recorder_session is required",
        )
        parameters = tuple(inspect.signature(validate).parameters.values())
        self.assertEqual(
            tuple(parameter.name for parameter in parameters),
            ("value", "context", "invocation", "response"),
        )
        self.assertIs(
            parameters[0].kind,
            inspect.Parameter.POSITIONAL_OR_KEYWORD,
        )
        for parameter in parameters[1:]:
            self.assertIs(parameter.kind, inspect.Parameter.KEYWORD_ONLY)
            self.assertIs(parameter.default, inspect.Parameter.empty)
        return core, validate

    def orchestration_record_deriver(self):
        core = importlib.import_module("scripts.evidence_core")
        derive = getattr(core, "derive_platform_orchestration_record", None)
        self.assertTrue(
            callable(derive),
            "derive_platform_orchestration_record is required",
        )
        self.assertEqual(
            tuple(inspect.signature(derive).parameters),
            ("context", "invocation", "response", "recorder_session"),
        )
        return core, derive

    def producer_receipt_validators(self):
        core = importlib.import_module("scripts.evidence_core")
        validate = getattr(core, "validate_producer_receipt", None)
        self.assertTrue(callable(validate), "validate_producer_receipt is required")
        self.assertEqual(tuple(inspect.signature(validate).parameters), ("value",))
        validate_append = getattr(
            core,
            "_validate_append_platform_producer_receipt",
            None,
        )
        self.assertTrue(
            callable(validate_append),
            "append-platform producer receipt join is required",
        )
        self.assertEqual(
            tuple(inspect.signature(validate_append).parameters),
            ("value", "recorder_session", "record"),
        )
        return core, validate, validate_append

    @classmethod
    def append_platform_fixture(
        cls,
        *,
        evidence_attempt="002",
        parent_finding_ids=("impl-01-F001",),
        candidate=None,
        candidate_relation=None,
        next_receipt_sequence=2,
    ):
        context = cls.context_v2(list(parent_finding_ids))
        context["evidence_attempt"] = evidence_attempt
        context["worktree_handle"] = "e" * 64
        dispatch_id = "d" * 64
        invocation = {
            "schema": "tersh-host-spawn-invocation-v1",
            "context_nonce": context["context_nonce"],
            "harness_bundle_revision": context["harness_bundle_revision"],
            "harness_bundle_sha256": context["harness_bundle_sha256"],
            "dispatch_id": dispatch_id,
            "requested_model": context["requested_model"],
            "requested_reasoning_effort": context[
                "requested_reasoning_effort"
            ],
            "selected_model": "gpt-5.6-sol",
            "selected_reasoning_effort": "xhigh",
            "dispatched_at": "2026-08-10T00:00:01.000000001Z",
        }
        response = {
            "schema": "tersh-host-spawn-response-v2",
            "context_nonce": context["context_nonce"],
            "harness_bundle_revision": context["harness_bundle_revision"],
            "harness_bundle_sha256": context["harness_bundle_sha256"],
            "dispatch_id": dispatch_id,
            "agent_id": "fixture-agent",
            "canonical_task_path": context["canonical_task_path"],
            "agent_run_id": "fixture-run",
            "started_at": "2026-08-10T00:00:02.000000001Z",
            "ended_at": "2026-08-10T00:00:03.000000001Z",
            "terminal_status": "completed",
            "reported_result_commit": context["review_target"],
            "reported_record_sha256": None,
        }
        selected_candidate = candidate or context["review_target"]
        relation = candidate_relation or (
            "equal"
            if selected_candidate == context["baseline_commit"]
            else "descendant"
        )
        previous_receipt_id = (
            None if next_receipt_sequence == 1 else "9" * 64
        )
        session = {
            "schema": "tersh-host-orchestration-recorder-session-v1",
            "producer_session_id": "1" * 64,
            "attempt_binding_id": "2" * 64,
            "predecessor_attempt_binding_id": (
                None if evidence_attempt == "001" else "3" * 64
            ),
            "entrypoint": "record-orchestration",
            "producer_mode": "harness",
            "operation": "append-platform",
            "context_nonce": context["context_nonce"],
            "dispatch_id": dispatch_id,
            "evidence_id": context["evidence_id"],
            "evidence_attempt": evidence_attempt,
            "run_binding": context["run_binding"],
            "candidate": selected_candidate,
            "candidate_tree": "4" * 40,
            "worktree_handle": context["worktree_handle"],
            "worktree_observed_at": "2026-08-10T00:00:00.500000001Z",
            "baseline_commit": context["baseline_commit"],
            "candidate_relation": relation,
            "bundle_id": context["harness_bundle_sha256"],
            "runtime_profile_id": "5" * 64,
            "policy_sha256": "6" * 64,
            "policy_entry_id": "record-orchestration",
            "policy_entry_sha256": "7" * 64,
            "projection_root_class": "local",
            "record_class": "orchestration",
            "record_schema": "tersh-evidence-orchestration-v1",
            "destination": (
                f"attempt-{evidence_attempt}/candidate-{selected_candidate}/"
                f"orchestration/{context['role']}.{context['wave']}."
                f"{context['review_attempt']}.json"
            ),
            "parent_finding_ids_sha256": hashlib.sha256(
                cls.canonical_fixture_body_bytes(
                    context["parent_finding_ids"]
                )
            ).hexdigest(),
            "next_receipt_sequence": next_receipt_sequence,
            "previous_receipt_id": previous_receipt_id,
        }
        provenance = {
            "mode": "platform-envelope",
            **{
                kind: {
                    "body": body,
                    "sha256": hashlib.sha256(
                        cls.canonical_fixture_body_bytes(body)
                    ).hexdigest(),
                }
                for kind, body in (
                    ("context", context),
                    ("invocation", invocation),
                    ("response", response),
                )
            },
        }
        return provenance, session

    @classmethod
    def rehash_platform_arm(cls, provenance, body_kind):
        provenance[body_kind]["sha256"] = hashlib.sha256(
            cls.canonical_fixture_body_bytes(
                provenance[body_kind]["body"]
            )
        ).hexdigest()

    @classmethod
    def expected_orchestration_record(cls, core, provenance, session):
        detached_provenance = {
            "mode": "platform-envelope",
            **{
                body_kind: {
                    "body": copy.deepcopy(provenance[body_kind]["body"]),
                    "sha256": provenance[body_kind]["sha256"],
                }
                for body_kind in ("context", "invocation", "response")
            },
        }
        validated_platform_provenance = (
            core.validate_platform_envelope_provenance(detached_provenance)
        )
        context = validated_platform_provenance["context"]["body"]
        invocation = validated_platform_provenance["invocation"]["body"]
        response = validated_platform_provenance["response"]["body"]
        return {
            "schema": "tersh-evidence-orchestration-v1",
            "evidence_id": context["evidence_id"],
            "evidence_attempt": context["evidence_attempt"],
            "run_binding": context["run_binding"],
            "role": context["role"],
            "wave": context["wave"],
            "review_attempt": context["review_attempt"],
            "baseline_commit": context["baseline_commit"],
            "reviewed_commit": session["candidate"],
            "parent_finding_ids": copy.deepcopy(context["parent_finding_ids"]),
            "dispatch_id": invocation["dispatch_id"],
            "agent_id": response["agent_id"],
            "canonical_task_path": context["canonical_task_path"],
            "agent_run_id": response["agent_run_id"],
            "model": invocation["selected_model"],
            "reasoning_effort": invocation["selected_reasoning_effort"],
            "dispatched_at": invocation["dispatched_at"],
            "started_at": response["started_at"],
            "ended_at": response["ended_at"],
            "terminal_status": response["terminal_status"],
            "provenance": validated_platform_provenance,
        }

    @classmethod
    def candidate_rule_fixture(
        cls,
        *,
        wave,
        candidate,
        reported_result_commit,
        role="safety",
        review_target=None,
    ):
        relation = "equal" if candidate == "a" * 40 else "descendant"
        provenance, session = cls.append_platform_fixture(
            candidate=candidate,
            candidate_relation=relation,
        )
        context = provenance["context"]["body"]
        response = provenance["response"]["body"]
        context["wave"] = wave
        context["role"] = role
        context["review_target"] = (
            candidate if review_target is None else review_target
        )
        response["reported_result_commit"] = reported_result_commit
        session["destination"] = (
            f"attempt-{context['evidence_attempt']}/candidate-{candidate}/"
            f"orchestration/{role}.{wave}.{context['review_attempt']}.json"
        )
        cls.rehash_platform_arm(provenance, "context")
        cls.rehash_platform_arm(provenance, "response")
        return provenance, session

    @classmethod
    def producer_receipt_fixture(
        cls,
        session,
        record,
        *,
        producer_mode="harness",
        environment_capability=None,
    ):
        agent_report = producer_mode == "agent-report"
        body = cls.canonical_fixture_body_bytes(record)
        return {
            "schema": "tersh-host-producer-receipt-v1",
            "receipt_id": "0" * 64,
            "attempt_binding_id": session["attempt_binding_id"],
            "producer_session_id": session["producer_session_id"],
            "sequence": session["next_receipt_sequence"],
            "previous_receipt_id": session["previous_receipt_id"],
            "producer_mode": producer_mode,
            "entrypoint": (
                "seal-agent-record" if agent_report else session["entrypoint"]
            ),
            "bundle_id": session["bundle_id"],
            "runtime_profile_id": session["runtime_profile_id"],
            "policy_entry_id": session["policy_entry_id"],
            "policy_entry_sha256": session["policy_entry_sha256"],
            "environment_capability": copy.deepcopy(environment_capability),
            "projection_root_class": session["projection_root_class"],
            "record_class": session["record_class"],
            "record_schema": session["record_schema"],
            "destination": session["destination"],
            "body_sha256": hashlib.sha256(body).hexdigest(),
            "byte_count": len(body),
            "dispatch_id": "d" * 64 if agent_report else None,
            "reported_record_sha256": "e" * 64 if agent_report else None,
            "created_at": "2026-08-10T00:00:04.000000001Z",
        }

    @staticmethod
    def environment_capability(attempt_binding_id):
        return {
            "schema": "tersh-host-environment-capability-v1",
            "capability_id": "f" * 64,
            "kind": "distinct-writable-filesystems-v1",
            "attempt_binding_id": attempt_binding_id,
            "root_a": {
                "schema": "tersh-host-opened-directory-v1",
                "path": "/var/tmp/tersh-root-a",
                "device": 11,
                "inode": 101,
                "owner_uid": 501,
                "mode": 448,
            },
            "root_b": {
                "schema": "tersh-host-opened-directory-v1",
                "path": "/private/tmp/tersh-root-b",
                "device": 12,
                "inode": 102,
                "owner_uid": 501,
                "mode": 448,
            },
            "created_at": "2026-08-10T00:00:00.000000001Z",
            "expires_at": "2026-08-10T00:05:00.000000001Z",
        }

    @staticmethod
    def validate_recorder_session(validate, session, provenance):
        return validate(
            session,
            context=provenance["context"]["body"],
            invocation=provenance["invocation"]["body"],
            response=provenance["response"]["body"],
        )

    def host_parent_source_model(self):
        core = importlib.import_module("scripts.evidence_core")
        host_model = importlib.import_module(
            "scripts.tests.append_platform_host_model"
        )
        model_class = getattr(host_model, "AppendPlatformHostModel", None)
        self.assertTrue(
            callable(model_class),
            "AppendPlatformHostModel is required for Host parent-source checks",
        )
        model = model_class()
        self.assertTrue(callable(model.register_receipted_finding_source))
        self.assertTrue(callable(model.validate_context_parent_sources))
        return core, model

    @staticmethod
    def finding(finding_id):
        return {
            "finding_id": finding_id,
            "severity": "P1",
            "requirement": "AP-PARENT-001",
            "file": "scripts/evidence_core.py",
            "line": 1,
            "counterexample": "the parent finding remains unresolved",
            "required_correction": "resolve the receipted parent finding",
        }

    @classmethod
    def finding_source_body(
        cls,
        finding_id,
        *,
        evidence_id="impl-01",
        evidence_attempt="001",
        source_kind="review",
        findings=None,
    ):
        source_findings = (
            [cls.finding(finding_id)] if findings is None else findings
        )
        if source_kind == "review":
            return {
                "schema": "tersh-test-review-finding-source-v1",
                "evidence_id": evidence_id,
                "evidence_attempt": evidence_attempt,
                "verdict": "FAIL",
                "findings": source_findings,
            }
        if source_kind == "audit-failure":
            return {
                "schema": "tersh-test-audit-failure-finding-source-v1",
                "evidence_id": evidence_id,
                "evidence_attempt": evidence_attempt,
                "candidate": "d" * 40,
                "audit_revision": "001",
                "failure_class": "candidate-repair-required",
                "findings": source_findings,
            }
        raise AssertionError(f"unsupported fixture source kind: {source_kind}")

    @staticmethod
    def canonical_fixture_body_bytes(body):
        return (
            json.dumps(
                body,
                ensure_ascii=False,
                sort_keys=True,
                separators=(",", ":"),
                allow_nan=False,
            ).encode("utf-8", errors="strict")
            + b"\n"
        )

    @classmethod
    def receipted_finding_source(
        cls,
        finding_id,
        *,
        evidence_id="impl-01",
        evidence_attempt="001",
        source_kind="review",
        receipt_digit="1",
        findings=None,
    ):
        body = cls.finding_source_body(
            finding_id,
            evidence_id=evidence_id,
            evidence_attempt=evidence_attempt,
            source_kind=source_kind,
            findings=findings,
        )
        body_sha256 = hashlib.sha256(
            cls.canonical_fixture_body_bytes(body)
        ).hexdigest()
        receipt = {
            "finding_id": finding_id,
            "evidence_id": evidence_id,
            "evidence_attempt": evidence_attempt,
            "source_kind": source_kind,
            "receipt_id": receipt_digit * 64,
            "body_sha256": body_sha256,
        }
        return receipt, body

    def test_context_v2_binds_bounded_existing_parent_finding_ids(self):
        _, validate = self.context_v2_validator()
        positive_matrix = (
            ("zero-parents", []),
            ("one-parent", ["impl-01-F999"]),
            (
                "hardening-family-parent",
                ["hardening-07-F999"],
                "hardening-07",
            ),
            (
                "128-parents",
                [f"impl-01-F{sequence:03d}" for sequence in range(1, 129)],
            ),
            (
                "sparse-parents",
                ["impl-01-F001", "impl-01-F003", "impl-01-F999"],
            ),
        )

        for positive_case in positive_matrix:
            case_id, parent_finding_ids, *evidence_ids = positive_case
            evidence_id = evidence_ids[0] if evidence_ids else "impl-01"
            with self.subTest(case_id=case_id):
                body = self.context_v2(
                    parent_finding_ids,
                    evidence_id=evidence_id,
                )
                before = copy.deepcopy(body)
                nested_before = copy.deepcopy(body["parent_finding_ids"])
                nested_identity = body["parent_finding_ids"]

                validated = validate(body)

                self.assertEqual(body, before, "validation mutated its input object")
                self.assertIs(body["parent_finding_ids"], nested_identity)
                self.assertEqual(body["parent_finding_ids"], nested_before)
                self.assertIs(type(validated), dict)
                self.assertEqual(validated, before)
                self.assertIsNot(validated, body)
                self.assertEqual(
                    validated["schema"],
                    "tersh-host-dispatch-context-v2",
                )
                self.assertIs(type(validated["parent_finding_ids"]), list)
                self.assertIsNot(
                    validated["parent_finding_ids"],
                    body["parent_finding_ids"],
                )

    def test_context_v2_structurally_rejects_legacy_alias_duplicate_reordered_or_cross_evidence_parents(
        self,
    ):
        core, validate = self.context_v2_validator()

        class HostileParentList(list):
            def __deepcopy__(self, memo):
                raise AssertionError("invalid list subclass was deep-copied")

        class HostileFindingId(str):
            def __deepcopy__(self, memo):
                raise AssertionError("invalid string subclass was deep-copied")

        class HostileContextKey(str):
            def __deepcopy__(self, memo):
                raise TypeError("invalid string-subclass key was deep-copied")

        class HostileContextValue(str):
            def __repr__(self):
                raise TypeError("invalid string-subclass value was represented")

            def __deepcopy__(self, memo):
                raise TypeError("invalid string-subclass value was deep-copied")

        def context_v1(body):
            body["schema"] = "tersh-host-dispatch-context-v1"
            del body["parent_finding_ids"]

        def v1_plus_optional_parents(body):
            body["schema"] = "tersh-host-dispatch-context-v1"

        def missing_parents(body):
            del body["parent_finding_ids"]

        def set_parents(value):
            def mutate(body):
                body["parent_finding_ids"] = value

            return mutate

        def replace_context_key(original, replacement):
            def mutate(body):
                value = body.pop(original)
                body[replacement] = value

            return mutate

        def set_context_field(field, value):
            def mutate(body):
                body[field] = value

            return mutate

        negative_matrix = (
            ("context-v1", context_v1),
            ("v1-plus-optional-parents", v1_plus_optional_parents),
            ("missing-parents", missing_parents),
            (
                "hostile-string-subclass-key",
                replace_context_key("schema", HostileContextKey("schema")),
            ),
            ("mixed-integer-key", replace_context_key("schema", 0)),
            (
                "hostile-evidence-id-subclass",
                set_context_field(
                    "evidence_id",
                    HostileContextValue("impl-01"),
                ),
            ),
            (
                "hostile-role-subclass",
                set_context_field("role", HostileContextValue("safety")),
            ),
            ("null-parents", set_parents(None)),
            ("bool-parents", set_parents(True)),
            (
                "list-subclass",
                set_parents(HostileParentList(["impl-01-F001"])),
            ),
            (
                "129-parents",
                set_parents(
                    [f"impl-01-F{sequence:03d}" for sequence in range(1, 130)]
                ),
            ),
            ("non-string-member", set_parents([1])),
            ("bool-member", set_parents([True])),
            (
                "string-subclass-member",
                set_parents([HostileFindingId("impl-01-F001")]),
            ),
            ("finding-000", set_parents(["impl-01-F000"])),
            ("finding-outside-001..999", set_parents(["impl-01-F1000"])),
            (
                "unicode-fullwidth-digits",
                set_parents(["impl-01-F００１"]),
            ),
            ("wrong-evidence-prefix", set_parents(["hardening-01-F001"])),
            ("wrong-evidence-number", set_parents(["impl-02-F001"])),
            (
                "duplicate",
                set_parents(["impl-01-F001", "impl-01-F001"]),
            ),
            (
                "descending",
                set_parents(["impl-01-F002", "impl-01-F001"]),
            ),
            (
                "same-sequence-alias",
                set_parents(["impl-01-F01"]),
            ),
            (
                "same-sequence-alias-F0001",
                set_parents(["impl-01-F0001"]),
            ),
        )

        for case_id, mutate in negative_matrix:
            with self.subTest(case_id=case_id):
                body = self.context_v2([])
                mutate(body)
                before = dict(body)
                keys_before = tuple(body.keys())
                values_before = tuple(body.values())
                nested_identity = body.get("parent_finding_ids")
                nested_before = (
                    list(nested_identity)
                    if isinstance(nested_identity, list)
                    else nested_identity
                )

                with self.assertRaises(core.EvidenceError):
                    validate(body)

                self.assertEqual(body, before, "rejection mutated its input object")
                self.assertEqual(tuple(body.keys()), keys_before)
                for actual_key, original_key in zip(body, keys_before):
                    self.assertIs(actual_key, original_key)
                for actual_value, original_value in zip(
                    body.values(),
                    values_before,
                ):
                    self.assertIs(actual_value, original_value)
                if isinstance(nested_identity, list):
                    self.assertIs(body["parent_finding_ids"], nested_identity)
                    self.assertEqual(list(body["parent_finding_ids"]), nested_before)

    def test_context_v2_rejects_v1_optional_alias_duplicate_reordered_cross_evidence_or_future_parents(
        self,
    ):
        core, positive_model = self.host_parent_source_model()
        resolver_parameters = tuple(
            inspect.signature(
                positive_model.validate_context_parent_sources
            ).parameters.values()
        )
        self.assertEqual(
            tuple(parameter.name for parameter in resolver_parameters),
            ("context",),
            "parent resolution must not accept caller source selectors",
        )
        self.assertIs(
            resolver_parameters[0].default,
            inspect.Parameter.empty,
        )
        self.assertIs(
            resolver_parameters[0].kind,
            inspect.Parameter.POSITIONAL_OR_KEYWORD,
        )
        receipt, body = self.receipted_finding_source("impl-01-F001")
        try:
            positive_model.register_receipted_finding_source(receipt, body)
        except core.EvidenceError as error:
            self.fail(f"valid fixture-only review source was rejected: {error}")
        context = self.context_v2(["impl-01-F001"])
        context_before = copy.deepcopy(context)

        validated = positive_model.validate_context_parent_sources(context)

        self.assertEqual(validated, ["impl-01-F001"])
        self.assertIs(type(validated), list)
        self.assertIsNot(validated, context["parent_finding_ids"])
        self.assertEqual(context, context_before)

        def context_v1(value):
            value["schema"] = "tersh-host-dispatch-context-v1"
            del value["parent_finding_ids"]

        def v1_plus_optional_parents(value):
            value["schema"] = "tersh-host-dispatch-context-v1"

        def missing_parents(value):
            del value["parent_finding_ids"]

        def set_parents(parents):
            def mutate(value):
                value["parent_finding_ids"] = parents

            return mutate

        structural_matrix = (
            ("context-v1", context_v1),
            ("v1-plus-optional-parents", v1_plus_optional_parents),
            ("missing-parents", missing_parents),
            ("sequence-alias", set_parents(["impl-01-F01"])),
            (
                "duplicate",
                set_parents(["impl-01-F001", "impl-01-F001"]),
            ),
            (
                "reordered",
                set_parents(["impl-01-F002", "impl-01-F001"]),
            ),
            ("cross-evidence", set_parents(["hardening-01-F001"])),
        )
        for case_id, mutate in structural_matrix:
            with self.subTest(case_id=case_id):
                invalid_context = self.context_v2(["impl-01-F001"])
                mutate(invalid_context)
                before = copy.deepcopy(invalid_context)

                with self.assert_exact_evidence_error(core.EvidenceError):
                    positive_model.validate_context_parent_sources(invalid_context)

                self.assertEqual(invalid_context, before)

        for case_id, source_attempt in (
            ("same-attempt-parent-source", "002"),
            ("future-attempt-parent-source", "003"),
        ):
            with self.subTest(case_id=case_id):
                _, model = self.host_parent_source_model()
                receipt, body = self.receipted_finding_source(
                    "impl-01-F001",
                    evidence_attempt=source_attempt,
                )
                model.register_receipted_finding_source(receipt, body)
                invalid_context = self.context_v2(["impl-01-F001"])
                before = copy.deepcopy(invalid_context)

                with self.assert_exact_evidence_error(core.EvidenceError):
                    model.validate_context_parent_sources(invalid_context)

                self.assertEqual(invalid_context, before)

    def test_context_v2_rejects_projection_only_mismatched_ambiguous_or_same_attempt_parent_sources(
        self,
    ):
        core, model = self.host_parent_source_model()
        review_receipt, review_body = self.receipted_finding_source(
            "impl-01-F001",
            source_kind="review",
            receipt_digit="1",
        )
        audit_receipt, audit_body = self.receipted_finding_source(
            "impl-01-F002",
            source_kind="audit-failure",
            receipt_digit="2",
        )
        review_receipt_before = copy.deepcopy(review_receipt)
        review_body_before = copy.deepcopy(review_body)
        audit_receipt_before = copy.deepcopy(audit_receipt)
        audit_body_before = copy.deepcopy(audit_body)

        try:
            review_registration = model.register_receipted_finding_source(
                review_receipt,
                review_body,
            )
            audit_registration = model.register_receipted_finding_source(
                audit_receipt,
                audit_body,
            )
        except core.EvidenceError as error:
            self.fail(f"valid fixture-only finding source was rejected: {error}")
        self.assertIsNone(review_registration)
        self.assertIsNone(audit_registration)
        self.assertEqual(review_receipt, review_receipt_before)
        self.assertEqual(review_body, review_body_before)
        self.assertEqual(audit_receipt, audit_receipt_before)
        self.assertEqual(audit_body, audit_body_before)

        review_receipt["finding_id"] = "impl-01-F999"
        review_body["findings"].clear()
        audit_receipt["source_kind"] = "review"
        audit_body["candidate"] = "e" * 40
        context = self.context_v2(["impl-01-F001", "impl-01-F002"])
        context_before = copy.deepcopy(context)

        validated = model.validate_context_parent_sources(context)

        self.assertEqual(validated, ["impl-01-F001", "impl-01-F002"])
        self.assertIs(type(validated), list)
        self.assertIsNot(validated, context["parent_finding_ids"])
        self.assertEqual(context, context_before)
        validated.clear()
        self.assertEqual(
            model.validate_context_parent_sources(context),
            ["impl-01-F001", "impl-01-F002"],
            "validated parents or registered sources aliased caller-owned values",
        )

        def seed_distinct_source(target_model):
            seed_receipt, seed_body = self.receipted_finding_source(
                "impl-01-F999",
                receipt_digit="9",
            )
            self.assertIsNone(
                target_model.register_receipted_finding_source(
                    seed_receipt,
                    seed_body,
                )
            )
            self.assertEqual(
                target_model.validate_context_parent_sources(
                    self.context_v2(["impl-01-F999"])
                ),
                ["impl-01-F999"],
            )

        def assert_seed_preserved_and_target_absent(target_model):
            try:
                preserved = target_model.validate_context_parent_sources(
                    self.context_v2(["impl-01-F999"])
                )
            except core.EvidenceError as error:
                self.fail(
                    "registration rejection erased existing Host source state: "
                    f"{error}"
                )
            self.assertEqual(
                preserved,
                ["impl-01-F999"],
                "registration rejection erased existing Host source state",
            )
            with self.assert_exact_evidence_error(core.EvidenceError):
                target_model.validate_context_parent_sources(
                    self.context_v2(["impl-01-F001"])
                )

        with self.subTest(case_id="projection-only-omission"):
            _, empty_model = self.host_parent_source_model()
            with self.assert_exact_evidence_error(core.EvidenceError):
                empty_model.validate_context_parent_sources(
                    self.context_v2(["impl-01-F001"])
                )

        with self.subTest(case_id="projection-only-body"):
            _, projection_model = self.host_parent_source_model()
            seed_distinct_source(projection_model)
            projection_receipt, projection_body = self.receipted_finding_source(
                "impl-01-F001"
            )
            projection_body = {
                "schema": "tersh-formal-evidence-projection-v1",
                "evidence_id": "impl-01",
                "evidence_attempt": "001",
                "findings": [self.finding("impl-01-F001")],
            }
            projection_receipt["body_sha256"] = hashlib.sha256(
                self.canonical_fixture_body_bytes(projection_body)
            ).hexdigest()
            with self.assert_exact_evidence_error(core.EvidenceError):
                projection_model.register_receipted_finding_source(
                    projection_receipt,
                    projection_body,
                )
            assert_seed_preserved_and_target_absent(projection_model)
            valid_receipt, valid_body = self.receipted_finding_source(
                "impl-01-F001"
            )
            self.assertIsNone(
                projection_model.register_receipted_finding_source(
                    valid_receipt,
                    valid_body,
                )
            )
            self.assertEqual(
                projection_model.validate_context_parent_sources(
                    self.context_v2(["impl-01-F001", "impl-01-F999"])
                ),
                ["impl-01-F001", "impl-01-F999"],
                "projection-only rejection partially admitted a source",
            )

        registration_mutations = []

        def add_registration_case(
            case_id,
            mutate,
            *,
            source_kind="review",
            rehash_body=True,
        ):
            receipt, body = self.receipted_finding_source(
                "impl-01-F001",
                source_kind=source_kind,
            )
            mutate(receipt, body)
            if rehash_body:
                receipt["body_sha256"] = hashlib.sha256(
                    self.canonical_fixture_body_bytes(body)
                ).hexdigest()
            registration_mutations.append((case_id, receipt, body))

        for receipt_field in (
            "finding_id",
            "evidence_id",
            "evidence_attempt",
            "source_kind",
            "receipt_id",
            "body_sha256",
        ):
            add_registration_case(
                f"missing-receipt-{receipt_field}",
                lambda receipt, _body, field=receipt_field: receipt.pop(field),
                rehash_body=receipt_field != "body_sha256",
            )
        add_registration_case(
            "extra-receipt-key",
            lambda receipt, _body: receipt.__setitem__("projection_path", "/tmp/x"),
        )
        add_registration_case(
            "source-kind-alias",
            lambda receipt, _body: receipt.__setitem__(
                "source_kind",
                "audit_failure",
            ),
        )
        add_registration_case(
            "uppercase-receipt-id",
            lambda receipt, _body: receipt.__setitem__("receipt_id", "A" * 64),
        )
        add_registration_case(
            "canonical-digest-mismatch",
            lambda receipt, _body: receipt.__setitem__("body_sha256", "0" * 64),
            rehash_body=False,
        )
        add_registration_case(
            "uppercase-body-sha256",
            lambda receipt, _body: receipt.__setitem__("body_sha256", "A" * 64),
            rehash_body=False,
        )

        def use_no_lf_digest(receipt, body):
            canonical = self.canonical_fixture_body_bytes(body)
            receipt["body_sha256"] = hashlib.sha256(canonical[:-1]).hexdigest()

        add_registration_case(
            "digest-omits-canonical-lf",
            use_no_lf_digest,
            rehash_body=False,
        )
        add_registration_case(
            "wrong-evidence-metadata",
            lambda receipt, _body: receipt.__setitem__(
                "evidence_id",
                "hardening-01",
            ),
        )
        add_registration_case(
            "review-body-extra-key",
            lambda _receipt, body: body.__setitem__("projection", True),
        )
        add_registration_case(
            "reduced-body-under-production-review-schema",
            lambda _receipt, body: body.__setitem__(
                "schema",
                "tersh-evidence-review-v1",
            ),
        )
        for review_field in (
            "schema",
            "evidence_id",
            "evidence_attempt",
            "verdict",
            "findings",
        ):
            add_registration_case(
                f"review-body-missing-{review_field}",
                lambda _receipt, body, field=review_field: body.pop(field),
            )
        add_registration_case(
            "review-body-attempt-mismatch",
            lambda _receipt, body: body.__setitem__("evidence_attempt", "002"),
        )
        add_registration_case(
            "findings-not-exact-list",
            lambda _receipt, body: body.__setitem__(
                "findings",
                tuple(body["findings"]),
            ),
        )
        add_registration_case(
            "finding-extra-key",
            lambda _receipt, body: body["findings"][0].__setitem__(
                "projection_path",
                "/tmp/x",
            ),
        )
        for finding_field in (
            "finding_id",
            "severity",
            "requirement",
            "file",
            "line",
            "counterexample",
            "required_correction",
        ):
            add_registration_case(
                f"finding-missing-{finding_field}",
                lambda _receipt, body, field=finding_field: body[
                    "findings"
                ][0].pop(field),
            )
        add_registration_case(
            "review-verdict-with-finding-is-pass",
            lambda _receipt, body: body.__setitem__("verdict", "PASS"),
        )
        add_registration_case(
            "finding-line-bool",
            lambda _receipt, body: body["findings"][0].__setitem__("line", True),
        )
        add_registration_case(
            "finding-requirement-non-string",
            lambda _receipt, body: body["findings"][0].__setitem__(
                "requirement",
                True,
            ),
        )
        add_registration_case(
            "finding-wrong-evidence",
            lambda _receipt, body: body["findings"][0].__setitem__(
                "finding_id",
                "hardening-01-F001",
            ),
        )
        add_registration_case(
            "target-finding-absent",
            lambda _receipt, body: body.__setitem__(
                "findings",
                [self.finding("impl-01-F002")],
            ),
        )
        add_registration_case(
            "target-finding-duplicated-in-body",
            lambda _receipt, body: body.__setitem__(
                "findings",
                [
                    self.finding("impl-01-F001"),
                    self.finding("impl-01-F001"),
                ],
            ),
        )
        add_registration_case(
            "audit-body-extra-key",
            lambda _receipt, body: body.__setitem__("projection", True),
            source_kind="audit-failure",
        )
        add_registration_case(
            "reduced-body-under-production-audit-failure-schema",
            lambda _receipt, body: body.__setitem__(
                "schema",
                "tersh-audit-requirements-failure-v1",
            ),
            source_kind="audit-failure",
        )
        for audit_field in (
            "schema",
            "evidence_id",
            "evidence_attempt",
            "candidate",
            "audit_revision",
            "failure_class",
            "findings",
        ):
            add_registration_case(
                f"audit-body-missing-{audit_field}",
                lambda _receipt, body, field=audit_field: body.pop(field),
                source_kind="audit-failure",
            )
        add_registration_case(
            "audit-revision-bool",
            lambda _receipt, body: body.__setitem__("audit_revision", True),
            source_kind="audit-failure",
        )
        add_registration_case(
            "audit-revision-integer-alias",
            lambda _receipt, body: body.__setitem__("audit_revision", 1),
            source_kind="audit-failure",
        )
        add_registration_case(
            "audit-failure-class-alias",
            lambda _receipt, body: body.__setitem__(
                "failure_class",
                "candidate_repair_required",
            ),
            source_kind="audit-failure",
        )

        review_receipt, _ = self.receipted_finding_source(
            "impl-01-F001",
            source_kind="review",
        )
        _, cross_audit_body = self.receipted_finding_source(
            "impl-01-F001",
            source_kind="audit-failure",
        )
        review_receipt["body_sha256"] = hashlib.sha256(
            self.canonical_fixture_body_bytes(cross_audit_body)
        ).hexdigest()
        audit_receipt, _ = self.receipted_finding_source(
            "impl-01-F001",
            source_kind="audit-failure",
        )
        _, cross_review_body = self.receipted_finding_source(
            "impl-01-F001",
            source_kind="review",
        )
        audit_receipt["body_sha256"] = hashlib.sha256(
            self.canonical_fixture_body_bytes(cross_review_body)
        ).hexdigest()
        registration_mutations.extend(
            (
                (
                    "review-kind-with-audit-failure-body",
                    review_receipt,
                    cross_audit_body,
                ),
                (
                    "audit-failure-kind-with-review-body",
                    audit_receipt,
                    cross_review_body,
                ),
            )
        )

        class ReceiptDict(dict):
            pass

        class BodyDict(dict):
            pass

        class FindingsList(list):
            pass

        class FindingDict(dict):
            pass

        class SourceKind(str):
            pass

        class ReceiptKey(str):
            pass

        class BodyKey(str):
            pass

        class FindingKey(str):
            pass

        class ReceiptId(str):
            pass

        class BodySchema(str):
            pass

        class ExactStringValue(str):
            pass

        receipt, body = self.receipted_finding_source("impl-01-F001")
        receipt_key = dict(receipt)
        receipt_id = receipt_key.pop("receipt_id")
        receipt_key[ReceiptKey("receipt_id")] = receipt_id
        body_key = copy.deepcopy(body)
        body_schema = body_key.pop("schema")
        body_key[BodyKey("schema")] = body_schema
        finding_key = copy.deepcopy(body)
        finding_id = finding_key["findings"][0].pop("finding_id")
        finding_key["findings"][0][FindingKey("finding_id")] = finding_id
        registration_mutations.extend(
            (
                (
                    "receipt-dict-subclass",
                    ReceiptDict(receipt),
                    copy.deepcopy(body),
                ),
                (
                    "body-dict-subclass",
                    dict(receipt),
                    BodyDict(body),
                ),
                (
                    "findings-list-subclass",
                    dict(receipt),
                    {
                        **body,
                        "findings": FindingsList(body["findings"]),
                    },
                ),
                (
                    "finding-dict-subclass",
                    dict(receipt),
                    {
                        **body,
                        "findings": [FindingDict(body["findings"][0])],
                    },
                ),
                (
                    "source-kind-string-subclass",
                    {
                        **receipt,
                        "source_kind": SourceKind("review"),
                    },
                    copy.deepcopy(body),
                ),
                ("receipt-key-subclass", receipt_key, copy.deepcopy(body)),
                ("body-key-subclass", dict(receipt), body_key),
                ("finding-key-subclass", dict(receipt), finding_key),
                (
                    "receipt-id-string-subclass",
                    {
                        **receipt,
                        "receipt_id": ReceiptId(receipt["receipt_id"]),
                    },
                    copy.deepcopy(body),
                ),
                (
                    "body-schema-string-subclass",
                    dict(receipt),
                    {
                        **body,
                        "schema": BodySchema(body["schema"]),
                    },
                ),
                *tuple(
                    (
                        f"receipt-{field}-string-subclass",
                        {
                            **receipt,
                            field: ExactStringValue(receipt[field]),
                        },
                        copy.deepcopy(body),
                    )
                    for field in (
                        "finding_id",
                        "evidence_id",
                        "evidence_attempt",
                        "body_sha256",
                    )
                ),
                *tuple(
                    (
                        f"receipt-{field}-non-string",
                        {**receipt, field: True},
                        copy.deepcopy(body),
                    )
                    for field in (
                        "finding_id",
                        "evidence_id",
                        "evidence_attempt",
                        "source_kind",
                        "receipt_id",
                        "body_sha256",
                    )
                ),
                (
                    "review-evidence-id-string-subclass",
                    dict(receipt),
                    {
                        **body,
                        "evidence_id": ExactStringValue(body["evidence_id"]),
                    },
                ),
                (
                    "review-attempt-string-subclass",
                    dict(receipt),
                    {
                        **body,
                        "evidence_attempt": ExactStringValue(
                            body["evidence_attempt"]
                        ),
                    },
                ),
                (
                    "review-verdict-string-subclass",
                    dict(receipt),
                    {
                        **body,
                        "verdict": ExactStringValue(body["verdict"]),
                    },
                ),
                (
                    "finding-id-string-subclass",
                    dict(receipt),
                    {
                        **body,
                        "findings": [
                            {
                                **body["findings"][0],
                                "finding_id": ExactStringValue(
                                    body["findings"][0]["finding_id"]
                                ),
                            }
                        ],
                    },
                ),
                (
                    "finding-severity-string-subclass",
                    dict(receipt),
                    {
                        **body,
                        "findings": [
                            {
                                **body["findings"][0],
                                "severity": ExactStringValue("P1"),
                            }
                        ],
                    },
                ),
            )
        )

        for case_id, invalid_receipt, invalid_body in registration_mutations:
            with self.subTest(case_id=case_id):
                _, invalid_model = self.host_parent_source_model()
                seed_distinct_source(invalid_model)
                receipt_before = copy.deepcopy(invalid_receipt)
                body_before = copy.deepcopy(invalid_body)

                with self.assert_exact_evidence_error(core.EvidenceError):
                    invalid_model.register_receipted_finding_source(
                        invalid_receipt,
                        invalid_body,
                    )

                self.assertEqual(invalid_receipt, receipt_before)
                self.assertEqual(invalid_body, body_before)
                assert_seed_preserved_and_target_absent(invalid_model)
                valid_receipt, valid_body = self.receipted_finding_source(
                    "impl-01-F001"
                )
                self.assertIsNone(
                    invalid_model.register_receipted_finding_source(
                        valid_receipt,
                        valid_body,
                    )
                )
                self.assertEqual(
                    invalid_model.validate_context_parent_sources(
                        self.context_v2(["impl-01-F001", "impl-01-F999"])
                    ),
                    ["impl-01-F001", "impl-01-F999"],
                    "rejected registration partially admitted a source",
                )

        class ArmedHostile:
            armed = False

            def arm(self):
                self.hook_calls = []
                self.armed = True

            def trip_if_armed(self, hook):
                if self.armed:
                    self.hook_calls.append(hook)
                    raise AssertionError(
                        f"{type(self).__name__}.{hook} was invoked"
                    )

        class ArmedDict(dict, ArmedHostile):
            def __bool__(self):
                self.trip_if_armed("__bool__")
                return dict.__len__(self) != 0

            def __contains__(self, key):
                self.trip_if_armed("__contains__")
                return dict.__contains__(self, key)

            def __deepcopy__(self, memo):
                self.trip_if_armed("__deepcopy__")
                return self

            def __eq__(self, other):
                self.trip_if_armed("__eq__")
                return dict.__eq__(self, other)

            def __getitem__(self, key):
                self.trip_if_armed("__getitem__")
                return dict.__getitem__(self, key)

            def __iter__(self):
                self.trip_if_armed("__iter__")
                return dict.__iter__(self)

            def __len__(self):
                self.trip_if_armed("__len__")
                return dict.__len__(self)

            def __repr__(self):
                self.trip_if_armed("__repr__")
                return dict.__repr__(self)

            def items(self):
                self.trip_if_armed("items")
                return dict.items(self)

            def keys(self):
                self.trip_if_armed("keys")
                return dict.keys(self)

            def values(self):
                self.trip_if_armed("values")
                return dict.values(self)

        class ArmedList(list, ArmedHostile):
            def __bool__(self):
                self.trip_if_armed("__bool__")
                return list.__len__(self) != 0

            def __contains__(self, value):
                self.trip_if_armed("__contains__")
                return list.__contains__(self, value)

            def __deepcopy__(self, memo):
                self.trip_if_armed("__deepcopy__")
                return self

            def __eq__(self, other):
                self.trip_if_armed("__eq__")
                return list.__eq__(self, other)

            def __getitem__(self, key):
                self.trip_if_armed("__getitem__")
                return list.__getitem__(self, key)

            def __iter__(self):
                self.trip_if_armed("__iter__")
                return list.__iter__(self)

            def __len__(self):
                self.trip_if_armed("__len__")
                return list.__len__(self)

            def __repr__(self):
                self.trip_if_armed("__repr__")
                return list.__repr__(self)

        class ArmedString(str, ArmedHostile):
            def __bool__(self):
                self.trip_if_armed("__bool__")
                return str.__len__(self) != 0

            def __contains__(self, value):
                self.trip_if_armed("__contains__")
                return str.__contains__(self, value)

            def __deepcopy__(self, memo):
                self.trip_if_armed("__deepcopy__")
                return self

            def __eq__(self, other):
                self.trip_if_armed("__eq__")
                return str.__eq__(self, other)

            def __format__(self, format_spec):
                self.trip_if_armed("__format__")
                return str.__format__(self, format_spec)

            def __hash__(self):
                self.trip_if_armed("__hash__")
                return str.__hash__(self)

            def __iter__(self):
                self.trip_if_armed("__iter__")
                return str.__iter__(self)

            def __repr__(self):
                self.trip_if_armed("__repr__")
                return str.__repr__(self)

        def mapping_identity_snapshot(value):
            return tuple(dict.items(value))

        def sequence_identity_snapshot(value):
            return tuple(list.__iter__(value))

        def assert_mapping_identity_unchanged(value, snapshot):
            actual = tuple(dict.items(value))
            self.assertEqual(len(actual), len(snapshot))
            for (actual_key, actual_value), (before_key, before_value) in zip(
                actual,
                snapshot,
            ):
                self.assertIs(actual_key, before_key)
                self.assertIs(actual_value, before_value)

        def assert_sequence_identity_unchanged(value, snapshot):
            actual = tuple(list.__iter__(value))
            self.assertEqual(len(actual), len(snapshot))
            for actual_value, before_value in zip(actual, snapshot):
                self.assertIs(actual_value, before_value)

        def snapshot_source_inputs(receipt_value, body_value, hostiles):
            findings = dict.__getitem__(body_value, "findings")
            finding = list.__getitem__(findings, 0)
            snapshots = (
                ("mapping", receipt_value, mapping_identity_snapshot(receipt_value)),
                ("mapping", body_value, mapping_identity_snapshot(body_value)),
                ("sequence", findings, sequence_identity_snapshot(findings)),
                ("mapping", finding, mapping_identity_snapshot(finding)),
            )
            for hostile in hostiles:
                hostile.arm()
            return snapshots

        def assert_snapshots_unchanged(snapshots):
            for snapshot_kind, value, snapshot in snapshots:
                if snapshot_kind == "mapping":
                    assert_mapping_identity_unchanged(value, snapshot)
                else:
                    assert_sequence_identity_unchanged(value, snapshot)

        hostile_registration_cases = []

        receipt, body = self.receipted_finding_source("impl-01-F001")
        hostile_receipt = ArmedDict(receipt)
        hostile_registration_cases.append(
            (
                "armed-receipt-dict-subclass",
                hostile_receipt,
                body,
                (hostile_receipt,),
            )
        )

        receipt, body = self.receipted_finding_source("impl-01-F001")
        hostile_body = ArmedDict(body)
        hostile_registration_cases.append(
            ("armed-body-dict-subclass", receipt, hostile_body, (hostile_body,))
        )

        receipt, body = self.receipted_finding_source("impl-01-F001")
        hostile_findings = ArmedList(body["findings"])
        body["findings"] = hostile_findings
        hostile_registration_cases.append(
            (
                "armed-findings-list-subclass",
                receipt,
                body,
                (hostile_findings,),
            )
        )

        receipt, body = self.receipted_finding_source("impl-01-F001")
        hostile_finding = ArmedDict(body["findings"][0])
        body["findings"][0] = hostile_finding
        hostile_registration_cases.append(
            (
                "armed-finding-dict-subclass",
                receipt,
                body,
                (hostile_finding,),
            )
        )

        receipt, body = self.receipted_finding_source("impl-01-F001")
        hostile_source_kind = ArmedString(receipt["source_kind"])
        receipt["source_kind"] = hostile_source_kind
        hostile_registration_cases.append(
            (
                "armed-receipt-value-string-subclass",
                receipt,
                body,
                (hostile_source_kind,),
            )
        )

        receipt, body = self.receipted_finding_source("impl-01-F001")
        hostile_receipt_key = ArmedString("receipt_id")
        receipt_id = receipt.pop("receipt_id")
        receipt[hostile_receipt_key] = receipt_id
        hostile_registration_cases.append(
            (
                "armed-receipt-key-string-subclass",
                receipt,
                body,
                (hostile_receipt_key,),
            )
        )

        receipt, body = self.receipted_finding_source("impl-01-F001")
        hostile_body_schema = ArmedString(body["schema"])
        body["schema"] = hostile_body_schema
        hostile_registration_cases.append(
            (
                "armed-body-value-string-subclass",
                receipt,
                body,
                (hostile_body_schema,),
            )
        )

        receipt, body = self.receipted_finding_source("impl-01-F001")
        hostile_finding_id = ArmedString(body["findings"][0]["finding_id"])
        body["findings"][0]["finding_id"] = hostile_finding_id
        hostile_registration_cases.append(
            (
                "armed-finding-value-string-subclass",
                receipt,
                body,
                (hostile_finding_id,),
            )
        )

        for case_id, invalid_receipt, invalid_body, hostiles in (
            hostile_registration_cases
        ):
            with self.subTest(case_id=case_id):
                snapshots = snapshot_source_inputs(
                    invalid_receipt,
                    invalid_body,
                    hostiles,
                )
                _, hostile_model = self.host_parent_source_model()
                seed_distinct_source(hostile_model)

                with self.assert_exact_evidence_error(core.EvidenceError):
                    hostile_model.register_receipted_finding_source(
                        invalid_receipt,
                        invalid_body,
                    )

                assert_snapshots_unchanged(snapshots)
                for hostile in hostiles:
                    self.assertEqual(hostile.hook_calls, [])
                assert_seed_preserved_and_target_absent(hostile_model)

        hostile_resolver_cases = []

        context = self.context_v2(["impl-01-F999"])
        hostile_context = ArmedDict(context)
        hostile_resolver_cases.append(
            ("armed-context-dict-subclass", hostile_context, (hostile_context,))
        )

        context = self.context_v2(["impl-01-F999"])
        hostile_parents = ArmedList(context["parent_finding_ids"])
        context["parent_finding_ids"] = hostile_parents
        hostile_resolver_cases.append(
            ("armed-parent-list-subclass", context, (hostile_parents,))
        )

        context = self.context_v2([])
        hostile_parent = ArmedString("impl-01-F999")
        context["parent_finding_ids"] = [hostile_parent]
        hostile_resolver_cases.append(
            ("armed-parent-string-subclass", context, (hostile_parent,))
        )

        context = self.context_v2(["impl-01-F999"])
        hostile_context_key = ArmedString("schema")
        schema = context.pop("schema")
        context[hostile_context_key] = schema
        hostile_resolver_cases.append(
            ("armed-context-key-string-subclass", context, (hostile_context_key,))
        )

        context = self.context_v2(["impl-01-F999"])
        hostile_evidence_id = ArmedString(context["evidence_id"])
        context["evidence_id"] = hostile_evidence_id
        hostile_resolver_cases.append(
            (
                "armed-context-value-string-subclass",
                context,
                (hostile_evidence_id,),
            )
        )

        for case_id, invalid_context, hostiles in hostile_resolver_cases:
            with self.subTest(case_id=case_id):
                context_snapshot = mapping_identity_snapshot(invalid_context)
                parents = dict.__getitem__(
                    invalid_context,
                    "parent_finding_ids",
                )
                parents_snapshot = sequence_identity_snapshot(parents)
                for hostile in hostiles:
                    hostile.arm()
                _, hostile_model = self.host_parent_source_model()
                seed_distinct_source(hostile_model)

                with self.assert_exact_evidence_error(core.EvidenceError):
                    hostile_model.validate_context_parent_sources(invalid_context)

                assert_mapping_identity_unchanged(
                    invalid_context,
                    context_snapshot,
                )
                assert_sequence_identity_unchanged(parents, parents_snapshot)
                for hostile in hostiles:
                    self.assertEqual(hostile.hook_calls, [])
                self.assertEqual(
                    hostile_model.validate_context_parent_sources(
                        self.context_v2(["impl-01-F999"])
                    ),
                    ["impl-01-F999"],
                )

        for case_id, source_attempt in (
            ("same-attempt", "002"),
            ("future-attempt", "003"),
        ):
            with self.subTest(case_id=case_id):
                _, attempt_model = self.host_parent_source_model()
                receipt, body = self.receipted_finding_source(
                    "impl-01-F001",
                    evidence_attempt=source_attempt,
                )
                attempt_model.register_receipted_finding_source(receipt, body)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    attempt_model.validate_context_parent_sources(
                        self.context_v2(["impl-01-F001"])
                    )

        with self.subTest(case_id="numeric-prior-attempt"):
            _, prior_model = self.host_parent_source_model()
            receipt, body = self.receipted_finding_source(
                "impl-01-F003",
                evidence_attempt="009",
            )
            prior_model.register_receipted_finding_source(receipt, body)
            later_context = self.context_v2(["impl-01-F003"])
            later_context["evidence_attempt"] = "010"
            self.assertEqual(
                prior_model.validate_context_parent_sources(later_context),
                ["impl-01-F003"],
            )

        with self.subTest(case_id="same-id-in-two-receipted-bodies"):
            _, ambiguous_model = self.host_parent_source_model()
            first_receipt, first_body = self.receipted_finding_source(
                "impl-01-F001",
                source_kind="review",
                receipt_digit="3",
            )
            second_receipt, second_body = self.receipted_finding_source(
                "impl-01-F001",
                source_kind="audit-failure",
                receipt_digit="4",
            )
            self.assertNotEqual(
                first_receipt["body_sha256"],
                second_receipt["body_sha256"],
            )
            ambiguous_model.register_receipted_finding_source(
                first_receipt,
                first_body,
            )
            ambiguous_model.register_receipted_finding_source(
                second_receipt,
                second_body,
            )
            with self.assert_exact_evidence_error(core.EvidenceError):
                ambiguous_model.validate_context_parent_sources(
                    self.context_v2(["impl-01-F001"])
                )

    def test_recorder_session_schema_rejects_wrong_schema_extra_missing_alias_null_bool_and_wrong_type_fields(
        self,
    ):
        core, validate = self.recorder_session_validator()
        positive_fixtures = (
            ("attempt-002-sequence-2", *self.append_platform_fixture()),
            (
                "attempt-002-sequence-1",
                *self.append_platform_fixture(next_receipt_sequence=1),
            ),
            (
                "attempt-001-null-predecessor",
                *self.append_platform_fixture(
                    evidence_attempt="001",
                    next_receipt_sequence=1,
                ),
            ),
            (
                "equal-candidate",
                *self.append_platform_fixture(
                    candidate="a" * 40,
                    candidate_relation="equal",
                ),
            ),
            (
                "opaque-host-assertions",
                *self.append_platform_fixture(candidate="c" * 40),
            ),
        )
        positive_fixtures[-1][2].update(
            {
                "producer_session_id": "a" * 64,
                "attempt_binding_id": "b" * 64,
                "predecessor_attempt_binding_id": "c" * 64,
                "candidate_tree": "d" * 40,
                "worktree_observed_at": "2026-08-09T23:59:59.1Z",
                "runtime_profile_id": "e" * 64,
                "policy_sha256": "f" * 64,
                "policy_entry_id": "alternate-policy-row",
                "policy_entry_sha256": "0" * 64,
            }
        )
        for case_id, provenance, session in positive_fixtures:
            with self.subTest(case_id=case_id):
                self.assertEqual(set(session), SESSION_KEYS)
                provenance_before = copy.deepcopy(provenance)
                session_before = copy.deepcopy(session)

                validated = self.validate_recorder_session(
                    validate,
                    session,
                    provenance,
                )

                self.assertIs(type(validated), dict)
                self.assertEqual(validated, session_before)
                self.assertIsNot(validated, session)
                self.assertEqual(session, session_before)
                self.assertEqual(provenance, provenance_before)

        provenance, session = self.append_platform_fixture()

        class SessionDict(dict):
            pass

        class SessionKey(str):
            pass

        class SessionString(str):
            pass

        class SessionInteger(int):
            pass

        structural_cases = []
        extra = copy.deepcopy(session)
        extra["projection_path"] = "/tmp/untrusted"
        structural_cases.append(("extra-key", extra))
        for field in sorted(SESSION_KEYS):
            missing = copy.deepcopy(session)
            del missing[field]
            structural_cases.append((f"missing-{field}", missing))
        alias = copy.deepcopy(session)
        alias["session_id"] = alias.pop("producer_session_id")
        structural_cases.extend(
            (
                ("legacy-session-id-alias", alias),
                ("dict-subclass", SessionDict(session)),
            )
        )
        key_subclass = copy.deepcopy(session)
        schema = key_subclass.pop("schema")
        key_subclass[SessionKey("schema")] = schema
        structural_cases.append(("string-subclass-key", key_subclass))

        for case_id, invalid in structural_cases:
            with self.subTest(case_id=case_id):
                before = copy.deepcopy(invalid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(validate, invalid, provenance)
                self.assertEqual(invalid, before)

        for field in sorted(SESSION_KEYS):
            with self.subTest(case_id=f"null-{field}"):
                invalid = copy.deepcopy(session)
                invalid[field] = None
                before = copy.deepcopy(invalid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(validate, invalid, provenance)
                self.assertEqual(invalid, before)

            with self.subTest(case_id=f"bool-{field}"):
                invalid = copy.deepcopy(session)
                invalid[field] = True
                before = copy.deepcopy(invalid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(validate, invalid, provenance)
                self.assertEqual(invalid, before)

        for field, value in session.items():
            if type(value) is not str:
                continue
            with self.subTest(case_id=f"wrong-type-{field}"):
                invalid = copy.deepcopy(session)
                invalid[field] = []
                before = copy.deepcopy(invalid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(validate, invalid, provenance)
                self.assertEqual(invalid, before)
            with self.subTest(case_id=f"string-subclass-{field}"):
                invalid = copy.deepcopy(session)
                invalid[field] = SessionString(value)
                before = copy.deepcopy(invalid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(validate, invalid, provenance)
                self.assertEqual(invalid, before)

        for case_id, sequence in (
            ("sequence-string", "2"),
            ("sequence-float", 2.0),
            ("sequence-int-subclass", SessionInteger(2)),
        ):
            with self.subTest(case_id=case_id):
                invalid = copy.deepcopy(session)
                invalid["next_receipt_sequence"] = sequence
                before = copy.deepcopy(invalid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(validate, invalid, provenance)
                self.assertEqual(invalid, before)

        malformed_fields = {
            "schema": "tersh-host-orchestration-recorder-session-v2",
            "producer_session_id": "A" * 64,
            "attempt_binding_id": "A" * 64,
            "predecessor_attempt_binding_id": "A" * 64,
            "entrypoint": "record_orchestration",
            "producer_mode": "agent-report",
            "operation": "append_platform",
            "context_nonce": "A" * 64,
            "dispatch_id": "A" * 64,
            "evidence_id": "impl-08",
            "evidence_attempt": "2",
            "run_binding": "run_local",
            "candidate": "A" * 40,
            "candidate_tree": "A" * 40,
            "worktree_handle": "fixture-worktree",
            "worktree_observed_at": "2026-08-10T00:00:00Z",
            "baseline_commit": "A" * 40,
            "candidate_relation": "ancestor",
            "bundle_id": "A" * 64,
            "runtime_profile_id": "A" * 64,
            "policy_sha256": "A" * 64,
            "policy_entry_id": "record_orchestration",
            "policy_entry_sha256": "A" * 64,
            "projection_root_class": "repository",
            "record_class": "review",
            "record_schema": "tersh-evidence-orchestration-v2",
            "destination": "/absolute/orchestration.json",
            "parent_finding_ids_sha256": "0" * 64,
            "next_receipt_sequence": 0,
            "previous_receipt_id": "A" * 64,
        }
        self.assertEqual(set(malformed_fields), SESSION_KEYS)
        for field, value in malformed_fields.items():
            with self.subTest(case_id=f"malformed-{field}"):
                invalid = copy.deepcopy(session)
                invalid[field] = value
                before = copy.deepcopy(invalid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(validate, invalid, provenance)
                self.assertEqual(invalid, before)

        hex64_fields = (
            "producer_session_id",
            "attempt_binding_id",
            "predecessor_attempt_binding_id",
            "context_nonce",
            "dispatch_id",
            "worktree_handle",
            "bundle_id",
            "runtime_profile_id",
            "policy_sha256",
            "policy_entry_sha256",
            "parent_finding_ids_sha256",
            "previous_receipt_id",
        )
        oid40_fields = ("candidate", "candidate_tree", "baseline_commit")
        for field_group, malformed_values in (
            (hex64_fields, ("a" * 63, "a" * 65, "g" * 64)),
            (oid40_fields, ("a" * 39, "a" * 41, "g" * 40)),
        ):
            for field in field_group:
                for value in malformed_values:
                    with self.subTest(
                        case_id=f"boundary-or-nonhex-{field}-{len(value)}"
                    ):
                        invalid = copy.deepcopy(session)
                        invalid[field] = value
                        before = copy.deepcopy(invalid)
                        with self.assert_exact_evidence_error(
                            core.EvidenceError
                        ):
                            self.validate_recorder_session(
                                validate,
                                invalid,
                                provenance,
                            )
                        self.assertEqual(invalid, before)

        attempt_one_provenance, attempt_one_session = (
            self.append_platform_fixture(evidence_attempt="001")
        )
        attempt_one_session["predecessor_attempt_binding_id"] = "3" * 64
        receipt_one_provenance, receipt_one_session = (
            self.append_platform_fixture(next_receipt_sequence=1)
        )
        receipt_one_session["previous_receipt_id"] = "9" * 64
        crossed_pairs = (
            (
                "attempt-001-with-predecessor",
                attempt_one_provenance,
                attempt_one_session,
            ),
            (
                "attempt-002-without-predecessor",
                provenance,
                {**session, "predecessor_attempt_binding_id": None},
            ),
            (
                "sequence-1-with-previous",
                receipt_one_provenance,
                receipt_one_session,
            ),
            (
                "sequence-2-without-previous",
                provenance,
                {**session, "previous_receipt_id": None},
            ),
        )
        for case_id, case_provenance, invalid in crossed_pairs:
            with self.subTest(case_id=case_id):
                before = copy.deepcopy(invalid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(
                        validate,
                        invalid,
                        case_provenance,
                    )
                self.assertEqual(invalid, before)

        class ArmedHostile:
            armed = False

            def arm(self):
                self.hook_calls = []
                self.armed = True

            def trip_if_armed(self, hook):
                if self.armed:
                    self.hook_calls.append(hook)
                    raise AssertionError(
                        f"{type(self).__name__}.{hook} was invoked"
                    )

        class ArmedSessionDict(dict, ArmedHostile):
            def __deepcopy__(self, memo):
                self.trip_if_armed("__deepcopy__")
                return self

            def __iter__(self):
                self.trip_if_armed("__iter__")
                return dict.__iter__(self)

            def __repr__(self):
                self.trip_if_armed("__repr__")
                return dict.__repr__(self)

        class ArmedSessionString(str, ArmedHostile):
            def __deepcopy__(self, memo):
                self.trip_if_armed("__deepcopy__")
                return self

            def __eq__(self, other):
                self.trip_if_armed("__eq__")
                return str.__eq__(self, other)

            def __format__(self, format_spec):
                self.trip_if_armed("__format__")
                return str.__format__(self, format_spec)

            def __hash__(self):
                self.trip_if_armed("__hash__")
                return str.__hash__(self)

            def __repr__(self):
                self.trip_if_armed("__repr__")
                return str.__repr__(self)

        hostile_session = ArmedSessionDict(session)
        hostile_session.arm()
        hostile_key = ArmedSessionString("schema")
        key_session = copy.deepcopy(session)
        schema = key_session.pop("schema")
        key_session[hostile_key] = schema
        hostile_key.arm()
        hostile_value = ArmedSessionString(session["producer_session_id"])
        value_session = copy.deepcopy(session)
        value_session["producer_session_id"] = hostile_value
        hostile_value.arm()
        for case_id, invalid, hostile in (
            ("armed-dict-subclass", hostile_session, hostile_session),
            ("armed-key-subclass", key_session, hostile_key),
            ("armed-value-subclass", value_session, hostile_value),
        ):
            with self.subTest(case_id=case_id):
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(validate, invalid, provenance)
                self.assertEqual(hostile.hook_calls, [])

    def test_append_platform_rejects_each_context_session_and_dispatch_identity_drift(
        self,
    ):
        core, validate = self.recorder_session_validator()
        provenance, session = self.append_platform_fixture(parent_finding_ids=())
        validated_provenance = core.validate_platform_envelope_provenance(
            provenance
        )
        self.assertEqual(
            self.validate_recorder_session(
                validate,
                session,
                validated_provenance,
            ),
            session,
        )

        alternate = copy.deepcopy(provenance)
        alternate_context = alternate["context"]["body"]
        alternate_context.update(
            {
                "context_nonce": "0" * 64,
                "harness_bundle_revision": "6" * 40,
                "harness_bundle_sha256": "b" * 64,
                "evidence_id": "hardening-07",
                "evidence_attempt": "010",
                "role": "product",
                "wave": "closure-b",
                "review_attempt": "999",
                "run_binding": "run-cumulative",
                "baseline_commit": "c" * 40,
                "review_target": "d" * 40,
                "canonical_task_path": "/root/product/reviewer",
                "worktree_handle": "f" * 64,
            }
        )
        alternate["invocation"]["body"].update(
            {
                "context_nonce": alternate_context["context_nonce"],
                "harness_bundle_revision": alternate_context[
                    "harness_bundle_revision"
                ],
                "harness_bundle_sha256": alternate_context[
                    "harness_bundle_sha256"
                ],
                "dispatch_id": "a" * 64,
            }
        )
        alternate["response"]["body"].update(
            {
                "context_nonce": alternate_context["context_nonce"],
                "harness_bundle_revision": alternate_context[
                    "harness_bundle_revision"
                ],
                "harness_bundle_sha256": alternate_context[
                    "harness_bundle_sha256"
                ],
                "dispatch_id": "a" * 64,
                "canonical_task_path": alternate_context[
                    "canonical_task_path"
                ],
                "reported_result_commit": alternate_context[
                    "review_target"
                ],
            }
        )
        for body_kind in ("context", "invocation", "response"):
            self.rehash_platform_arm(alternate, body_kind)
        alternate = core.validate_platform_envelope_provenance(alternate)
        alternate_session = copy.deepcopy(session)
        alternate_session.update(
            {
                "context_nonce": alternate_context["context_nonce"],
                "dispatch_id": "a" * 64,
                "evidence_id": alternate_context["evidence_id"],
                "evidence_attempt": alternate_context["evidence_attempt"],
                "run_binding": alternate_context["run_binding"],
                "candidate": alternate_context["review_target"],
                "worktree_handle": alternate_context["worktree_handle"],
                "baseline_commit": alternate_context["baseline_commit"],
                "bundle_id": alternate_context["harness_bundle_sha256"],
                "destination": (
                    "attempt-010/candidate-"
                    f"{alternate_context['review_target']}/orchestration/"
                    "product.closure-b.999.json"
                ),
            }
        )
        alternate_before = copy.deepcopy(alternate)
        alternate_session_before = copy.deepcopy(alternate_session)
        self.assertEqual(
            self.validate_recorder_session(
                validate,
                alternate_session,
                alternate,
            ),
            alternate_session,
            "valid nondefault route and identity values must not be hardcoded",
        )
        self.assertEqual(alternate, alternate_before)
        self.assertEqual(alternate_session, alternate_session_before)

        for case_id, field, value in (
            ("context-nonce", "context_nonce", "0" * 64),
            ("evidence-id", "evidence_id", "impl-02"),
            ("evidence-attempt", "evidence_attempt", "003"),
            ("run-binding", "run_binding", "run-cumulative"),
            ("worktree-handle", "worktree_handle", "f" * 64),
            ("baseline-commit", "baseline_commit", "c" * 40),
            ("bundle-id", "bundle_id", "0" * 64),
            ("dispatch-id", "dispatch_id", "0" * 64),
        ):
            with self.subTest(case_id=case_id):
                invalid = copy.deepcopy(session)
                invalid[field] = value
                before = copy.deepcopy(invalid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(
                        validate,
                        invalid,
                        validated_provenance,
                    )
                self.assertEqual(invalid, before)

        coherent_body_drifts = (
            (
                "coherent-context-nonce",
                {
                    "context": {"context_nonce": "0" * 64},
                    "invocation": {"context_nonce": "0" * 64},
                    "response": {"context_nonce": "0" * 64},
                },
            ),
            (
                "coherent-bundle-id",
                {
                    "context": {"harness_bundle_sha256": "0" * 64},
                    "invocation": {"harness_bundle_sha256": "0" * 64},
                    "response": {"harness_bundle_sha256": "0" * 64},
                },
            ),
            (
                "coherent-dispatch-id",
                {
                    "invocation": {"dispatch_id": "0" * 64},
                    "response": {"dispatch_id": "0" * 64},
                },
            ),
            ("context-evidence-id", {"context": {"evidence_id": "impl-02"}}),
            (
                "context-evidence-attempt",
                {"context": {"evidence_attempt": "003"}},
            ),
            (
                "context-run-binding",
                {"context": {"run_binding": "run-cumulative"}},
            ),
            (
                "context-worktree-handle",
                {"context": {"worktree_handle": "f" * 64}},
            ),
            (
                "context-baseline-commit",
                {"context": {"baseline_commit": "c" * 40}},
            ),
        )
        for case_id, changes in coherent_body_drifts:
            with self.subTest(case_id=case_id):
                drifted = copy.deepcopy(provenance)
                for body_kind, fields in changes.items():
                    drifted[body_kind]["body"].update(fields)
                    self.rehash_platform_arm(drifted, body_kind)
                    self.assertEqual(
                        drifted[body_kind]["sha256"],
                        hashlib.sha256(
                            self.canonical_fixture_body_bytes(
                                drifted[body_kind]["body"]
                            )
                        ).hexdigest(),
                    )
                locally_valid = core.validate_platform_envelope_provenance(
                    drifted
                )
                locally_valid_before = copy.deepcopy(locally_valid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(
                        validate,
                        session,
                        locally_valid,
                    )
                self.assertEqual(locally_valid, locally_valid_before)

        internally_incoherent_drifts = (
            ("context-only-nonce", "context", "context_nonce", "0" * 64),
            ("invocation-only-nonce", "invocation", "context_nonce", "0" * 64),
            ("response-only-nonce", "response", "context_nonce", "0" * 64),
            (
                "context-only-bundle-revision",
                "context",
                "harness_bundle_revision",
                "6" * 40,
            ),
            (
                "invocation-only-bundle-revision",
                "invocation",
                "harness_bundle_revision",
                "6" * 40,
            ),
            (
                "response-only-bundle-revision",
                "response",
                "harness_bundle_revision",
                "6" * 40,
            ),
            (
                "context-only-bundle-id",
                "context",
                "harness_bundle_sha256",
                "0" * 64,
            ),
            (
                "invocation-only-bundle-id",
                "invocation",
                "harness_bundle_sha256",
                "0" * 64,
            ),
            (
                "response-only-bundle-id",
                "response",
                "harness_bundle_sha256",
                "0" * 64,
            ),
            ("invocation-only-dispatch", "invocation", "dispatch_id", "0" * 64),
            ("response-only-dispatch", "response", "dispatch_id", "0" * 64),
            (
                "response-only-task-path",
                "response",
                "canonical_task_path",
                "/root/alternate/reviewer",
            ),
            (
                "invocation-only-requested-model",
                "invocation",
                "requested_model",
                "gpt-5.6-terra",
            ),
            (
                "invocation-only-requested-reasoning-effort",
                "invocation",
                "requested_reasoning_effort",
                "high",
            ),
        )
        for case_id, body_kind, field, value in internally_incoherent_drifts:
            with self.subTest(case_id=case_id):
                drifted = copy.deepcopy(provenance)
                drifted[body_kind]["body"][field] = value
                self.rehash_platform_arm(drifted, body_kind)
                self.assertEqual(
                    drifted[body_kind]["sha256"],
                    hashlib.sha256(
                        self.canonical_fixture_body_bytes(
                            drifted[body_kind]["body"]
                        )
                    ).hexdigest(),
                )
                drifted_before = copy.deepcopy(drifted)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(
                        validate,
                        session,
                        drifted,
                    )
                self.assertEqual(drifted, drifted_before)

        original_invocation_validator = core._validate_invocation_body
        for case_id, field, alternate in (
            (
                "isolated-invocation-requested-model-join",
                "requested_model",
                "gpt-5.6-terra",
            ),
            (
                "isolated-invocation-requested-reasoning-effort-join",
                "requested_reasoning_effort",
                "high",
            ),
        ):
            with self.subTest(case_id=case_id):
                drifted = copy.deepcopy(provenance)
                drifted["invocation"]["body"][field] = alternate
                self.rehash_platform_arm(drifted, "invocation")
                self.assertEqual(
                    drifted["invocation"]["sha256"],
                    hashlib.sha256(
                        self.canonical_fixture_body_bytes(
                            drifted["invocation"]["body"]
                        )
                    ).hexdigest(),
                )

                def validate_locally_alternate_invocation(value):
                    if (
                        type(value) is not dict
                        or value.get(field) != alternate
                    ):
                        return original_invocation_validator(value)
                    normalized = copy.deepcopy(value)
                    normalized[field] = provenance["context"]["body"][field]
                    original_invocation_validator(normalized)
                    normalized[field] = alternate
                    return normalized

                drifted_before = copy.deepcopy(drifted)
                session_before = copy.deepcopy(session)
                with mock.patch.object(
                    core,
                    "_validate_invocation_body",
                    side_effect=validate_locally_alternate_invocation,
                ):
                    self.assertEqual(
                        core._validate_invocation_body(
                            drifted["invocation"]["body"]
                        ),
                        drifted["invocation"]["body"],
                        "alternate invocation must reach the cross-body join",
                    )
                    with self.assert_exact_evidence_error(core.EvidenceError):
                        self.validate_recorder_session(
                            validate,
                            session,
                            drifted,
                        )
                self.assertEqual(drifted, drifted_before)
                self.assertEqual(session, session_before)

        revision_only = copy.deepcopy(provenance)
        for body_kind in ("context", "invocation", "response"):
            revision_only[body_kind]["body"][
                "harness_bundle_revision"
            ] = "6" * 40
            self.rehash_platform_arm(revision_only, body_kind)
        revision_only = core.validate_platform_envelope_provenance(
            revision_only
        )
        self.assertEqual(
            self.validate_recorder_session(
                validate,
                session,
                revision_only,
            ),
            session,
            "pure validation must not assert unavailable installed revision state",
        )

    def test_append_platform_rejects_candidate_tree_policy_destination_or_parent_join_drift(
        self,
    ):
        core, validate = self.recorder_session_validator()
        provenance, session = self.append_platform_fixture()
        equal_provenance, equal_session = self.append_platform_fixture(
            candidate="a" * 40,
            candidate_relation="equal",
        )
        for case_id, valid_provenance, valid_session in (
            ("unequal-descendant", provenance, session),
            ("equal-equal", equal_provenance, equal_session),
        ):
            with self.subTest(case_id=case_id):
                self.assertEqual(
                    self.validate_recorder_session(
                        validate,
                        valid_session,
                        valid_provenance,
                    ),
                    valid_session,
                )

        equal_session_wrong = copy.deepcopy(equal_session)
        equal_session_wrong["candidate_relation"] = "descendant"
        relation_crosses = (
            ("unequal-marked-equal", provenance, {**session, "candidate_relation": "equal"}),
            ("equal-marked-descendant", equal_provenance, equal_session_wrong),
        )
        for case_id, case_provenance, invalid in relation_crosses:
            with self.subTest(case_id=case_id):
                before = copy.deepcopy(invalid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(
                        validate,
                        invalid,
                        case_provenance,
                    )
                self.assertEqual(invalid, before)

        malformed_host_assertions = (
            ("candidate", "candidate", "A" * 40),
            ("candidate-tree", "candidate_tree", "A" * 40),
            ("policy-sha256", "policy_sha256", "A" * 64),
            ("runtime-profile-id", "runtime_profile_id", "A" * 64),
            ("policy-entry", "policy_entry_id", "record_orchestration"),
            ("policy-entry-sha256", "policy_entry_sha256", "A" * 64),
        )
        for case_id, field, value in malformed_host_assertions:
            with self.subTest(case_id=case_id):
                invalid = copy.deepcopy(session)
                invalid[field] = value
                before = copy.deepcopy(invalid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(validate, invalid, provenance)
                self.assertEqual(invalid, before)

        expected_destination = session["destination"]
        destination_drifts = (
            ("wrong-attempt", expected_destination.replace("attempt-002", "attempt-003")),
            ("wrong-candidate", expected_destination.replace("candidate-b", "candidate-c")),
            ("wrong-role", expected_destination.replace("safety.wave-c", "product.wave-c")),
            ("wrong-wave", expected_destination.replace("safety.wave-c", "safety.wave-a")),
            ("wrong-review-attempt", expected_destination.replace(".001.json", ".002.json")),
            ("wrong-suffix", expected_destination.removesuffix(".json") + ".txt"),
            ("absolute", "/" + expected_destination),
            ("traversal", "../" + expected_destination),
        )
        for case_id, destination in destination_drifts:
            with self.subTest(case_id=f"destination-{case_id}"):
                invalid = copy.deepcopy(session)
                invalid["destination"] = destination
                before = copy.deepcopy(invalid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(validate, invalid, provenance)
                self.assertEqual(invalid, before)

        for case_id, field, value in (
            ("context-role", "role", "product"),
            ("context-wave", "wave", "wave-a"),
            ("context-review-attempt", "review_attempt", "002"),
        ):
            with self.subTest(case_id=case_id):
                drifted = copy.deepcopy(provenance)
                drifted["context"]["body"][field] = value
                self.rehash_platform_arm(drifted, "context")
                locally_valid = core.validate_platform_envelope_provenance(
                    drifted
                )
                before = copy.deepcopy(locally_valid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(
                        validate,
                        session,
                        locally_valid,
                    )
                self.assertEqual(locally_valid, before)

        canonical_parents = self.canonical_fixture_body_bytes(
            provenance["context"]["body"]["parent_finding_ids"]
        )
        self.assertTrue(canonical_parents.endswith(b"\n"))
        without_lf_digest = hashlib.sha256(canonical_parents[:-1]).hexdigest()
        self.assertNotEqual(
            without_lf_digest,
            session["parent_finding_ids_sha256"],
        )
        for case_id, digest in (
            ("canonical-array-without-lf", without_lf_digest),
            (
                "different-canonical-array",
                hashlib.sha256(
                    self.canonical_fixture_body_bytes([])
                ).hexdigest(),
            ),
        ):
            with self.subTest(case_id=case_id):
                invalid = copy.deepcopy(session)
                invalid["parent_finding_ids_sha256"] = digest
                before = copy.deepcopy(invalid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    self.validate_recorder_session(validate, invalid, provenance)
                self.assertEqual(invalid, before)

        changed_parent_provenance = copy.deepcopy(provenance)
        changed_parent_provenance["context"]["body"][
            "parent_finding_ids"
        ] = ["impl-01-F001", "impl-01-F002"]
        self.rehash_platform_arm(changed_parent_provenance, "context")
        changed_parent_provenance = (
            core.validate_platform_envelope_provenance(
                changed_parent_provenance
            )
        )
        changed_parent_before = copy.deepcopy(changed_parent_provenance)
        with self.assert_exact_evidence_error(core.EvidenceError):
            self.validate_recorder_session(
                validate,
                session,
                changed_parent_provenance,
            )
        self.assertEqual(changed_parent_provenance, changed_parent_before)

        opaque_provenance, opaque_session = self.append_platform_fixture(
            candidate="c" * 40,
            candidate_relation="descendant",
        )
        opaque_session.update(
            {
                "producer_session_id": "a" * 64,
                "attempt_binding_id": "b" * 64,
                "predecessor_attempt_binding_id": "c" * 64,
                "candidate_tree": "d" * 40,
                "runtime_profile_id": "e" * 64,
                "policy_sha256": "f" * 64,
                "policy_entry_id": "syntactic-policy-row",
                "policy_entry_sha256": "0" * 64,
            }
        )
        tripwire = AssertionError(
            "pure recorder-session validation attempted subprocess or Git I/O"
        )
        with (
            mock.patch.object(core.subprocess, "Popen", side_effect=tripwire),
            mock.patch.object(core.subprocess, "run", side_effect=tripwire),
            mock.patch.object(
                core.subprocess,
                "check_output",
                side_effect=tripwire,
            ),
            mock.patch.object(core.subprocess, "call", side_effect=tripwire),
            mock.patch.object(core.os, "system", side_effect=tripwire),
            mock.patch.object(core.os, "popen", side_effect=tripwire),
            mock.patch.object(core.os, "open", side_effect=tripwire),
            mock.patch.object(core.os, "stat", side_effect=tripwire),
            mock.patch.object(core.os, "listdir", side_effect=tripwire),
            mock.patch.object(core.Path, "open", side_effect=tripwire),
            mock.patch.object(core.Path, "read_bytes", side_effect=tripwire),
            mock.patch.object(core.Path, "read_text", side_effect=tripwire),
            mock.patch.object(core, "run_and_drain", side_effect=tripwire),
            mock.patch.object(core.socket, "socket", side_effect=tripwire),
        ):
            self.assertEqual(
                self.validate_recorder_session(
                    validate,
                    opaque_session,
                    opaque_provenance,
                ),
                opaque_session,
                "syntactic Host assertions must not trigger origin or ancestry lookup",
            )

    def test_orchestration_record_schema_has_one_authoritative_source_per_field(self):
        core, derive = self.orchestration_record_deriver()
        provenance, session = self.append_platform_fixture()
        expected = self.expected_orchestration_record(
            core,
            provenance,
            session,
        )
        inputs_before = copy.deepcopy((provenance, session))

        actual = derive(
            provenance["context"]["body"],
            provenance["invocation"]["body"],
            provenance["response"]["body"],
            session,
        )

        self.assertEqual(set(actual), ORCHESTRATION_RECORD_KEYS)
        self.assertEqual(actual, expected)
        self.assertEqual((provenance, session), inputs_before)
        self.assertIsNot(actual["parent_finding_ids"], provenance["context"]["body"]["parent_finding_ids"])
        for body_kind in ("context", "invocation", "response"):
            self.assertIsNot(
                actual["provenance"][body_kind]["body"],
                provenance[body_kind]["body"],
            )
        actual["parent_finding_ids"].append("impl-01-F002")
        actual["provenance"]["context"]["body"]["parent_finding_ids"].clear()
        self.assertEqual((provenance, session), inputs_before)

    def test_orchestration_record_rejects_extra_missing_mixed_or_overbound_fields(self):
        core, derive = self.orchestration_record_deriver()
        provenance, session = self.append_platform_fixture()
        expected = self.expected_orchestration_record(core, provenance, session)
        self.assertEqual(
            derive(
                provenance["context"]["body"],
                provenance["invocation"]["body"],
                provenance["response"]["body"],
                session,
            ),
            expected,
        )

        invalid_inputs = []
        extra_context = copy.deepcopy(provenance)
        extra_context["context"]["body"]["candidate"] = session["candidate"]
        invalid_inputs.append(("extra-context-field", extra_context, session))
        missing_response = copy.deepcopy(provenance)
        del missing_response["response"]["body"]["agent_id"]
        invalid_inputs.append(("missing-response-field", missing_response, session))
        mixed_dispatch = copy.deepcopy(provenance)
        mixed_dispatch["response"]["body"]["dispatch_id"] = "0" * 64
        invalid_inputs.append(("mixed-dispatch-lineage", mixed_dispatch, session))
        overbound_parent_set = copy.deepcopy(provenance)
        overbound_parent_set["context"]["body"]["parent_finding_ids"] = [
            f"impl-01-F{sequence:03d}" for sequence in range(1, 130)
        ]
        invalid_inputs.append(("overbound-parent-set", overbound_parent_set, session))
        for case_id, invalid, invalid_session in invalid_inputs:
            with self.subTest(case_id=case_id):
                before = copy.deepcopy((invalid, invalid_session))
                with self.assert_exact_evidence_error(core.EvidenceError):
                    derive(
                        invalid["context"]["body"],
                        invalid["invocation"]["body"],
                        invalid["response"]["body"],
                        invalid_session,
                    )
                self.assertEqual((invalid, invalid_session), before)

        original_canonical = core.canonical_json_bytes

        def overbound_record_bytes(value):
            if (
                type(value) is dict
                and value.get("schema") == "tersh-evidence-orchestration-v1"
            ):
                return b"x" * 61441
            return original_canonical(value)

        with mock.patch.object(
            core,
            "canonical_json_bytes",
            side_effect=overbound_record_bytes,
        ):
            with self.assert_exact_evidence_error(core.EvidenceError):
                derive(
                    provenance["context"]["body"],
                    provenance["invocation"]["body"],
                    provenance["response"]["body"],
                    session,
                )

    def test_append_platform_candidate_rules_cover_wave_a_wave_b_changed_and_unchanged(self):
        core, derive = self.orchestration_record_deriver()
        baseline = "a" * 40
        candidate = "b" * 40
        valid_cases = (
            (
                "wave-a",
                self.candidate_rule_fixture(
                    wave="wave-a",
                    candidate=baseline,
                    reported_result_commit=None,
                ),
            ),
            (
                "wave-b-changed",
                self.candidate_rule_fixture(
                    wave="wave-b",
                    role="implementation",
                    candidate=candidate,
                    review_target=baseline,
                    reported_result_commit=candidate,
                ),
            ),
            (
                "wave-b-unchanged-null",
                self.candidate_rule_fixture(
                    wave="wave-b",
                    role="implementation",
                    candidate=baseline,
                    review_target=baseline,
                    reported_result_commit=None,
                ),
            ),
            (
                "wave-b-unchanged-reported",
                self.candidate_rule_fixture(
                    wave="wave-b",
                    role="implementation",
                    candidate=baseline,
                    review_target=baseline,
                    reported_result_commit=baseline,
                ),
            ),
            (
                "wave-c-descendant-tag",
                self.candidate_rule_fixture(
                    wave="wave-c",
                    candidate=candidate,
                    reported_result_commit=candidate,
                ),
            ),
        )
        tripwire = AssertionError("pure record derivation attempted Git or filesystem I/O")
        with (
            mock.patch.object(core.subprocess, "Popen", side_effect=tripwire),
            mock.patch.object(core.subprocess, "run", side_effect=tripwire),
            mock.patch.object(core.os, "system", side_effect=tripwire),
            mock.patch.object(core.os, "open", side_effect=tripwire),
            mock.patch.object(core.socket, "socket", side_effect=tripwire),
        ):
            for case_id, (case_provenance, case_session) in valid_cases:
                with self.subTest(case_id=case_id):
                    record = derive(
                        case_provenance["context"]["body"],
                        case_provenance["invocation"]["body"],
                        case_provenance["response"]["body"],
                        case_session,
                    )
                    self.assertEqual(record["reviewed_commit"], case_session["candidate"])

        invalid_cases = []
        wave_a_changed = self.candidate_rule_fixture(
            wave="wave-a",
            candidate=candidate,
            reported_result_commit=candidate,
        )
        invalid_cases.append(("wave-a-changed", wave_a_changed))
        wave_b_wrong_role = self.candidate_rule_fixture(
            wave="wave-b",
            role="safety",
            candidate=candidate,
            review_target=baseline,
            reported_result_commit=candidate,
        )
        invalid_cases.append(("wave-b-wrong-role", wave_b_wrong_role))
        wave_b_changed_null = self.candidate_rule_fixture(
            wave="wave-b",
            role="implementation",
            candidate=candidate,
            review_target=baseline,
            reported_result_commit=None,
        )
        invalid_cases.append(("wave-b-changed-null", wave_b_changed_null))
        wave_b_wrong_review_target = self.candidate_rule_fixture(
            wave="wave-b",
            role="implementation",
            candidate=candidate,
            review_target=candidate,
            reported_result_commit=candidate,
        )
        invalid_cases.append(("wave-b-review-target-not-baseline", wave_b_wrong_review_target))
        wave_c_null_target = self.candidate_rule_fixture(
            wave="wave-c",
            candidate=candidate,
            review_target=candidate,
            reported_result_commit=candidate,
        )
        wave_c_null_target[0]["context"]["body"]["review_target"] = None
        self.rehash_platform_arm(wave_c_null_target[0], "context")
        invalid_cases.append(("outside-wave-b-null-review-target", wave_c_null_target))
        for case_id, (case_provenance, case_session) in invalid_cases:
            with self.subTest(case_id=case_id):
                before = copy.deepcopy((case_provenance, case_session))
                with self.assert_exact_evidence_error(core.EvidenceError):
                    derive(
                        case_provenance["context"]["body"],
                        case_provenance["invocation"]["body"],
                        case_provenance["response"]["body"],
                        case_session,
                    )
                self.assertEqual((case_provenance, case_session), before)

    def test_producer_receipt_schema_is_closed_typed_and_detached(self):
        core, validate, validate_append = self.producer_receipt_validators()
        provenance, session = self.append_platform_fixture()
        record = self.expected_orchestration_record(core, provenance, session)
        receipt = self.producer_receipt_fixture(session, record)
        receipt_before = copy.deepcopy(receipt)

        validated = validate(receipt)

        self.assertEqual(set(validated), PRODUCER_RECEIPT_KEYS)
        self.assertEqual(validated, receipt_before)
        self.assertIsNot(validated, receipt)
        self.assertEqual(
            validate_append(
                receipt,
                recorder_session=session,
                record=record,
            ),
            receipt_before,
            "null harness dispatch must not be compared to captured dispatch",
        )
        self.assertEqual(receipt, receipt_before)

        for record_class in sorted(PRODUCER_RECORD_CLASSES):
            with self.subTest(record_class=record_class):
                alternate = copy.deepcopy(receipt)
                alternate["record_class"] = record_class
                self.assertEqual(validate(alternate), alternate)

        capability = self.environment_capability(session["attempt_binding_id"])
        environment_receipt = self.producer_receipt_fixture(
            session,
            record,
            environment_capability=capability,
        )
        detached_environment = validate(environment_receipt)
        self.assertEqual(detached_environment, environment_receipt)
        self.assertIsNot(
            detached_environment["environment_capability"],
            environment_receipt["environment_capability"],
        )
        self.assertIsNot(
            detached_environment["environment_capability"]["root_a"],
            environment_receipt["environment_capability"]["root_a"],
        )
        detached_environment["environment_capability"]["root_a"]["path"] = "/mutated"
        self.assertEqual(environment_receipt["environment_capability"], capability)

        agent_receipt = self.producer_receipt_fixture(
            session,
            record,
            producer_mode="agent-report",
        )
        self.assertEqual(validate(agent_receipt), agent_receipt)

        structural_cases = []
        extra = copy.deepcopy(receipt)
        extra["projection_path"] = "/tmp/untrusted"
        structural_cases.append(("extra-field", extra))
        for field in sorted(PRODUCER_RECEIPT_KEYS):
            missing = copy.deepcopy(receipt)
            del missing[field]
            structural_cases.append((f"missing-{field}", missing))
        for case_id, invalid in structural_cases:
            with self.subTest(case_id=case_id):
                before = copy.deepcopy(invalid)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    validate(invalid)
                self.assertEqual(invalid, before)

        typed_cases = (
            ("sequence-bool", "sequence", True),
            ("sequence-zero", "sequence", 0),
            ("byte-count-bool", "byte_count", True),
            ("byte-count-zero", "byte_count", 0),
            ("policy-entry", "policy_entry_id", "Record_Orchestration"),
            ("record-class", "record_class", "orchestrate"),
            ("record-schema", "record_schema", "orchestration"),
            ("destination-absolute", "destination", "/attempt-002/a.json"),
            ("destination-traversal", "destination", "attempt-002/../a.json"),
            ("destination-wrong-prefix", "destination", "candidate-b/a.json"),
            ("created-at", "created_at", "2026-08-10T00:00:04Z"),
        )
        for case_id, field, value in typed_cases:
            with self.subTest(case_id=case_id):
                invalid = copy.deepcopy(receipt)
                invalid[field] = value
                with self.assert_exact_evidence_error(core.EvidenceError):
                    validate(invalid)

        for case_id, field, value in (
            ("harness-dispatch", "dispatch_id", "d" * 64),
            ("harness-reported-record", "reported_record_sha256", "e" * 64),
            ("agent-missing-dispatch", "dispatch_id", None),
            ("agent-missing-reported-record", "reported_record_sha256", None),
            ("agent-wrong-entrypoint", "entrypoint", "record-orchestration"),
        ):
            with self.subTest(case_id=case_id):
                invalid = copy.deepcopy(
                    receipt if case_id.startswith("harness") else agent_receipt
                )
                invalid[field] = value
                with self.assert_exact_evidence_error(core.EvidenceError):
                    validate(invalid)

        invalid_capabilities = []
        capability_extra = copy.deepcopy(environment_receipt)
        capability_extra["environment_capability"]["root_path"] = "/tmp"
        invalid_capabilities.append(("capability-extra", capability_extra))
        root_extra = copy.deepcopy(environment_receipt)
        root_extra["environment_capability"]["root_a"]["writable"] = True
        invalid_capabilities.append(("root-extra", root_extra))
        same_device = copy.deepcopy(environment_receipt)
        same_device["environment_capability"]["root_b"]["device"] = 11
        invalid_capabilities.append(("same-device", same_device))
        wrong_owner = copy.deepcopy(environment_receipt)
        wrong_owner["environment_capability"]["root_b"]["owner_uid"] = 502
        invalid_capabilities.append(("owner-mismatch", wrong_owner))
        wrong_mode = copy.deepcopy(environment_receipt)
        wrong_mode["environment_capability"]["root_a"]["mode"] = 0o755
        invalid_capabilities.append(("wrong-mode", wrong_mode))
        relative_root = copy.deepcopy(environment_receipt)
        relative_root["environment_capability"]["root_a"]["path"] = "tmp/root-a"
        invalid_capabilities.append(("relative-root", relative_root))
        expired = copy.deepcopy(environment_receipt)
        expired["environment_capability"]["expires_at"] = expired[
            "environment_capability"
        ]["created_at"]
        invalid_capabilities.append(("nonfuture-expiry", expired))
        for case_id, invalid in invalid_capabilities:
            with self.subTest(case_id=case_id):
                with self.assert_exact_evidence_error(core.EvidenceError):
                    validate(invalid)

        join_drifts = (
            ("attempt-binding", "attempt_binding_id", "a" * 64),
            ("producer-session", "producer_session_id", "a" * 64),
            ("sequence", "sequence", receipt["sequence"] + 1),
            ("previous-receipt", "previous_receipt_id", "a" * 64),
            ("entrypoint", "entrypoint", "alternate-entrypoint"),
            ("bundle", "bundle_id", "a" * 64),
            ("runtime", "runtime_profile_id", "a" * 64),
            ("policy-entry", "policy_entry_id", "alternate-policy"),
            ("policy-digest", "policy_entry_sha256", "a" * 64),
            ("projection-class", "projection_root_class", "external"),
            ("record-class", "record_class", "review"),
            ("record-schema", "record_schema", "tersh-evidence-review-v1"),
            ("destination", "destination", "attempt-002/alternate.json"),
            ("body-digest", "body_sha256", "a" * 64),
            ("byte-count", "byte_count", receipt["byte_count"] + 1),
            ("environment", "environment_capability", capability),
            ("detached-dispatch", "dispatch_id", "d" * 64),
            ("detached-reported-record", "reported_record_sha256", "e" * 64),
        )
        for case_id, field, value in join_drifts:
            with self.subTest(case_id=f"append-join-{case_id}"):
                invalid = copy.deepcopy(receipt)
                invalid[field] = copy.deepcopy(value)
                with self.assert_exact_evidence_error(core.EvidenceError):
                    validate_append(
                        invalid,
                        recorder_session=session,
                        record=record,
                    )


class _AppendStateConnection:
    def __init__(
        self,
        fd,
        kernel_identity,
        producer_session_id,
        *,
        peer_principal="uid-0-root-supervisor",
    ):
        self._fd = fd
        self.kernel_identity = kernel_identity
        self.producer_session_id = producer_session_id
        self.peer_principal = peer_principal

    def fileno(self):
        return self._fd


class _AppendWireHostSocket(socket.socket):
    def __init__(self, *args, **kwargs):
        self.sent_frames = []
        self.frame_observer = None
        super().__init__(*args, **kwargs)

    def sendall(self, data, flags=0):
        if flags == 0 and len(data) >= 4:
            size = struct.unpack(">I", data[:4])[0]
            if size == len(data) - 4:
                frame = json.loads(data[4:])
                self.sent_frames.append(frame)
                if self.frame_observer is not None:
                    self.frame_observer(copy.deepcopy(frame))
        return super().sendall(data, flags)


class AppendPlatformStateTests(unittest.TestCase):
    @contextlib.contextmanager
    def assert_evidence_error(self, error_class, message=None):
        with self.assertRaises(error_class) as raised:
            yield
        self.assertIs(type(raised.exception), error_class)
        if message is not None:
            self.assertIn(message, str(raised.exception))

    def host_state_model(self):
        host_model = importlib.import_module(
            "scripts.tests.append_platform_host_model"
        )
        model_class = getattr(host_model, "AppendPlatformHostModel", None)
        self.assertTrue(callable(model_class), "AppendPlatformHostModel is required")
        fixture_clock = getattr(host_model, "FixtureClock", None)
        self.assertTrue(callable(fixture_clock), "FixtureClock is required")
        clock = fixture_clock()
        model = model_class(clock=clock)
        for method_name in (
            "open_attempt",
            "register_raw_commit",
            "register_worktree_observation",
            "register_attempt_predecessor",
            "seed_lineage",
            "capture_invocation",
            "capture_response",
            "bind_append_connection",
            "append_platform",
            "build_host_orchestration_record",
            "invoke_root_internal",
            "snapshot",
        ):
            self.assertTrue(
                callable(getattr(model, method_name, None)),
                f"AppendPlatformHostModel.{method_name} is required",
            )
        return host_model, importlib.import_module("scripts.evidence_core"), clock, model

    @classmethod
    def fixture(
        cls,
        *,
        wave="wave-c",
        role="safety",
        baseline="a" * 40,
        candidate="b" * 40,
        reported_result_commit="b" * 40,
        review_target=None,
        evidence_attempt="002",
        nonce_digit="c",
        dispatch_digit="d",
        binding_digit="2",
        session_digit="1",
        predecessor_binding="3" * 64,
    ):
        provenance, session = AppendPlatformSchemaTests.append_platform_fixture(
            evidence_attempt=evidence_attempt,
            candidate=candidate,
            candidate_relation=("equal" if candidate == baseline else "descendant"),
        )
        context = provenance["context"]["body"]
        invocation = provenance["invocation"]["body"]
        response = provenance["response"]["body"]
        context.update(
            {
                "context_nonce": nonce_digit * 64,
                "wave": wave,
                "role": role,
                "baseline_commit": baseline,
                "review_target": candidate if review_target is None else review_target,
            }
        )
        invocation.update(
            {
                "context_nonce": context["context_nonce"],
                "dispatch_id": dispatch_digit * 64,
            }
        )
        response.update(
            {
                "context_nonce": context["context_nonce"],
                "dispatch_id": invocation["dispatch_id"],
                "reported_result_commit": reported_result_commit,
            }
        )
        session.update(
            {
                "producer_session_id": session_digit * 64,
                "attempt_binding_id": binding_digit * 64,
                "predecessor_attempt_binding_id": (
                    None if evidence_attempt == "001" else predecessor_binding
                ),
                "context_nonce": context["context_nonce"],
                "dispatch_id": invocation["dispatch_id"],
                "evidence_attempt": evidence_attempt,
                "candidate": candidate,
                "baseline_commit": baseline,
                "candidate_relation": (
                    "equal" if candidate == baseline else "descendant"
                ),
                "destination": (
                    f"attempt-{evidence_attempt}/candidate-{candidate}/"
                    f"orchestration/{role}.{wave}.001.json"
                ),
            }
        )
        for body_kind in ("context", "invocation", "response"):
            AppendPlatformSchemaTests.rehash_platform_arm(provenance, body_kind)
        return {
            "context": context,
            "invocation": invocation,
            "response": response,
            "session": session,
        }

    def install_facts(
        self,
        model,
        fixture,
        *,
        candidate_parents=None,
        raw_view="raw",
        clean=True,
        observed_head=None,
        observed_tree=None,
        install_candidate=True,
        predecessor_candidate=None,
    ):
        context = fixture["context"]
        session = fixture["session"]
        baseline = session["baseline_commit"]
        candidate = session["candidate"]
        model.register_raw_commit(
            baseline,
            tree=(session["candidate_tree"] if candidate == baseline else "a" * 40),
            parents=(),
        )
        if install_candidate and candidate != baseline:
            model.register_raw_commit(
                candidate,
                tree=session["candidate_tree"],
                parents=(baseline,) if candidate_parents is None else candidate_parents,
                view=raw_view,
            )
        model.register_worktree_observation(
            context["worktree_handle"],
            head=candidate if observed_head is None else observed_head,
            tree=(
                session["candidate_tree"]
                if observed_tree is None
                else observed_tree
            ),
            clean=clean,
            view="raw",
        )
        predecessor_binding = session["predecessor_attempt_binding_id"]
        if predecessor_binding is not None:
            model.register_attempt_predecessor(
                predecessor_binding,
                candidate=(
                    baseline
                    if predecessor_candidate is None
                    else predecessor_candidate
                ),
            )

    def ready_lineage(self, model, fixture):
        model.open_attempt(
            {"context": fixture["context"], "session": fixture["session"]}
        )
        created = model.seed_lineage(context=fixture["context"])
        invoked = model.capture_invocation(
            created["context_handle"], fixture["invocation"]
        )
        responded = model.capture_response(
            invoked["context_handle"], fixture["response"]
        )
        handles = {
            "producer_session_id": fixture["session"]["producer_session_id"],
            "context_handle": responded["context_handle"],
            "invocation_handle": responded["invocation_handle"],
            "response_handle": responded["response_handle"],
            "mode": "platform-envelope",
        }
        return created, invoked, responded, handles

    @staticmethod
    def durable_state(snapshot):
        return {
            key: copy.deepcopy(snapshot[key])
            for key in ("attempts", "lineages", "handles", "records")
        }

    def test_append_platform_rejects_cross_lineage_generation_alias_duplicate_or_mode_mixed_handles(self):
        _, core, _, model = self.host_state_model()
        fixture = self.fixture()
        self.install_facts(model, fixture)
        created, _, responded, handles = self.ready_lineage(model, fixture)
        connection = _AppendStateConnection(
            40, "socket-positive", fixture["session"]["producer_session_id"]
        )
        lease = model.bind_append_connection(handles, connection=connection)
        model.append_platform(lease, connection=connection, transaction_nonce="0" * 64)
        self.assertEqual(
            model.snapshot()["lineages"][responded["lineage_id"]]["state"],
            "APPENDED_PLATFORM",
        )

        for case_id in ("old-generation", "alias", "mode-mixed"):
            with self.subTest(case_id=case_id):
                _, core, _, case_model = self.host_state_model()
                case_fixture = self.fixture()
                self.install_facts(case_model, case_fixture)
                case_created, _, _, case_handles = self.ready_lineage(
                    case_model, case_fixture
                )
                invalid = copy.deepcopy(case_handles)
                if case_id == "old-generation":
                    invalid["context_handle"] = case_created["context_handle"]
                elif case_id == "alias":
                    invalid["response_handle"] = invalid["context_handle"]
                else:
                    invalid["mode"] = "agent-report"
                before = case_model.snapshot()
                case_connection = _AppendStateConnection(
                    41, case_id, case_fixture["session"]["producer_session_id"]
                )
                with self.assert_evidence_error(core.EvidenceError):
                    case_model.bind_append_connection(
                        invalid, connection=case_connection
                    )
                self.assertEqual(case_model.snapshot(), before)

        _, core, _, cross_model = self.host_state_model()
        first = self.fixture()
        second = self.fixture(
            evidence_attempt="003",
            nonce_digit="a",
            dispatch_digit="b",
            binding_digit="4",
            session_digit="5",
            predecessor_binding="6" * 64,
        )
        for item in (first, second):
            self.install_facts(cross_model, item)
        _, _, _, first_handles = self.ready_lineage(cross_model, first)
        _, _, _, second_handles = self.ready_lineage(cross_model, second)
        cross = copy.deepcopy(first_handles)
        cross["response_handle"] = second_handles["response_handle"]
        before = cross_model.snapshot()
        with self.assert_evidence_error(core.EvidenceError):
            cross_model.bind_append_connection(
                cross,
                connection=_AppendStateConnection(
                    42, "cross", first["session"]["producer_session_id"]
                ),
            )
        self.assertEqual(cross_model.snapshot(), before)

        before = cross_model.snapshot()
        with self.assert_evidence_error(core.EvidenceError):
            cross_model.capture_response(
                first_handles["context_handle"], first["response"]
            )
        self.assertEqual(cross_model.snapshot(), before)

        changed_binding = {
            "context": first["context"],
            "session": copy.deepcopy(first["session"]),
        }
        changed_binding["session"]["runtime_profile_id"] = "f" * 64
        before = cross_model.snapshot()
        with self.assert_evidence_error(core.EvidenceError, "immutable"):
            cross_model.open_attempt(changed_binding)
        self.assertEqual(cross_model.snapshot(), before)

    def test_root_peer_with_wrong_recorder_session_fails_before_handle_lookup(self):
        _, core, _, model = self.host_state_model()
        fixture = self.fixture()
        self.install_facts(model, fixture)
        _, _, _, handles = self.ready_lineage(model, fixture)
        positive = _AppendStateConnection(
            50, "root-session-positive", fixture["session"]["producer_session_id"]
        )
        self.assertIn(
            "lease_id", model.bind_append_connection(handles, connection=positive)
        )

        _, core, _, rejected_model = self.host_state_model()
        rejected_fixture = self.fixture()
        self.install_facts(rejected_model, rejected_fixture)
        self.ready_lineage(rejected_model, rejected_fixture)
        before = rejected_model.snapshot()
        wrong_session = _AppendStateConnection(50, "wrong-session", "f" * 64)
        unknown_handles = {
            "producer_session_id": "f" * 64,
            "context_handle": "not-looked-up",
            "invocation_handle": "not-looked-up",
            "response_handle": "not-looked-up",
            "mode": "platform-envelope",
        }
        with self.assert_evidence_error(
            core.EvidenceError, "recorder session authentication"
        ):
            rejected_model.bind_append_connection(
                unknown_handles, connection=wrong_session
            )
        self.assertEqual(rejected_model.snapshot(), before)

    def test_recorder_session_launch_lease_replay_and_fd_generation_reuse_fail_before_handle_lookup(self):
        _, core, _, positive_model = self.host_state_model()
        positive_fixture = self.fixture()
        self.install_facts(positive_model, positive_fixture)
        _, _, _, positive_handles = self.ready_lineage(
            positive_model, positive_fixture
        )
        positive_connection = _AppendStateConnection(
            60, "lease-positive", positive_fixture["session"]["producer_session_id"]
        )
        positive_lease = positive_model.bind_append_connection(
            positive_handles, connection=positive_connection
        )
        positive_model.append_platform(
            positive_lease,
            connection=positive_connection,
            transaction_nonce="1" * 64,
        )

        _, core, clock, expired_model = self.host_state_model()
        expired_fixture = self.fixture()
        self.install_facts(expired_model, expired_fixture)
        _, _, _, expired_handles = self.ready_lineage(expired_model, expired_fixture)
        expired_connection = _AppendStateConnection(
            61, "expired", expired_fixture["session"]["producer_session_id"]
        )
        expired_lease = expired_model.bind_append_connection(
            expired_handles, connection=expired_connection
        )
        durable_before = self.durable_state(expired_model.snapshot())
        clock.advance(5.001)
        with self.assert_evidence_error(core.EvidenceError, "launch lease expired"):
            expired_model.append_platform(
                expired_lease,
                connection=expired_connection,
                transaction_nonce="2" * 64,
            )
        self.assertEqual(
            self.durable_state(expired_model.snapshot()), durable_before
        )

        _, core, _, reuse_model = self.host_state_model()
        reuse_fixture = self.fixture()
        self.install_facts(reuse_model, reuse_fixture)
        _, _, _, reuse_handles = self.ready_lineage(reuse_model, reuse_fixture)
        old_connection = _AppendStateConnection(
            62, "kernel-generation-1", reuse_fixture["session"]["producer_session_id"]
        )
        old_lease = reuse_model.bind_append_connection(
            reuse_handles, connection=old_connection
        )
        durable_before = self.durable_state(reuse_model.snapshot())
        reused_fd_connection = _AppendStateConnection(
            62, "kernel-generation-2", reuse_fixture["session"]["producer_session_id"]
        )
        with self.assert_evidence_error(core.EvidenceError, "connection generation"):
            reuse_model.append_platform(
                old_lease,
                connection=reused_fd_connection,
                transaction_nonce="3" * 64,
            )
        self.assertEqual(self.durable_state(reuse_model.snapshot()), durable_before)
        fresh_lease = reuse_model.bind_append_connection(
            reuse_handles, connection=reused_fd_connection
        )
        reuse_model.append_platform(
            fresh_lease,
            connection=reused_fd_connection,
            transaction_nonce="4" * 64,
        )
        committed = reuse_model.snapshot()
        with self.assert_evidence_error(core.EvidenceError, "replay"):
            reuse_model.append_platform(
                fresh_lease,
                connection=reused_fd_connection,
                transaction_nonce="4" * 64,
            )
        self.assertEqual(reuse_model.snapshot(), committed)

    def test_root_internal_authentication_precedes_nonce_session_or_handle_lookup(self):
        _, core, _, model = self.host_state_model()
        fixture = self.fixture()
        self.install_facts(model, fixture)
        model.open_attempt({"context": fixture["context"], "session": fixture["session"]})
        before = model.snapshot()
        with self.assert_evidence_error(core.EvidenceError, "root supervisor"):
            model.invoke_root_internal(
                "not-an-operation",
                {"nonce": "invalid", "handle": "invalid"},
                principal="nonroot-recorder",
            )
        self.assertEqual(model.snapshot(), before)
        result = model.invoke_root_internal(
            "enumerate-attempt",
            {"attempt_binding_id": fixture["session"]["attempt_binding_id"]},
            principal=model.ROOT_SUPERVISOR_PRINCIPAL,
        )
        self.assertEqual(result["candidate"], fixture["session"]["candidate"])
        with self.assert_evidence_error(core.EvidenceError, "root-internal operation"):
            model.invoke_root_internal(
                "not-an-operation",
                {},
                principal=model.ROOT_SUPERVISOR_PRINCIPAL,
            )

    def test_append_platform_rejects_wave_a_baseline_drift_and_unrelated_wave_b_candidate(self):
        for positive_fixture in (
            self.fixture(
                wave="wave-a",
                baseline="a" * 40,
                candidate="a" * 40,
                reported_result_commit=None,
                review_target="a" * 40,
            ),
            self.fixture(
                wave="wave-b",
                role="implementation",
                baseline="a" * 40,
                candidate="b" * 40,
                reported_result_commit="b" * 40,
                review_target="a" * 40,
            ),
        ):
            _, _, _, model = self.host_state_model()
            self.install_facts(model, positive_fixture)
            self.assertEqual(
                model.open_attempt(
                    {
                        "context": positive_fixture["context"],
                        "session": positive_fixture["session"],
                    }
                )["candidate"],
                positive_fixture["session"]["candidate"],
            )

        invalid_fixtures = (
            (
                "wave-a-drift",
                self.fixture(
                    wave="wave-a",
                    baseline="a" * 40,
                    candidate="b" * 40,
                    review_target="a" * 40,
                ),
                None,
            ),
            (
                "wave-b-unrelated",
                self.fixture(
                    wave="wave-b",
                    role="implementation",
                    baseline="a" * 40,
                    candidate="c" * 40,
                    reported_result_commit="c" * 40,
                    review_target="a" * 40,
                ),
                ("d" * 40,),
            ),
        )
        for case_id, invalid, parents in invalid_fixtures:
            with self.subTest(case_id=case_id):
                _, core, _, model = self.host_state_model()
                if parents == ("d" * 40,):
                    model.register_raw_commit("d" * 40, tree="d" * 40, parents=())
                self.install_facts(model, invalid, candidate_parents=parents)
                before = model.snapshot()
                with self.assert_evidence_error(core.EvidenceError):
                    model.open_attempt(
                        {"context": invalid["context"], "session": invalid["session"]}
                    )
                self.assertEqual(model.snapshot(), before)

    def test_candidate_relation_matches_raw_ancestry_in_wave_c_and_closure(self):
        for wave in ("wave-c", "closure-a"):
            host_model, core, _, model = self.host_state_model()
            fixture = self.fixture(wave=wave)
            self.install_facts(model, fixture)
            model.open_attempt({"context": fixture["context"], "session": fixture["session"]})
            record_tripwire = AssertionError("Host record builder used client derivation")
            io_tripwire = AssertionError("Host record builder launched external I/O")
            with (
                mock.patch.object(
                    core,
                    "derive_platform_orchestration_record",
                    side_effect=record_tripwire,
                ),
                mock.patch.object(core.subprocess, "run", side_effect=io_tripwire),
                mock.patch.object(core.subprocess, "Popen", side_effect=io_tripwire),
                mock.patch.object(core.os, "system", side_effect=io_tripwire),
            ):
                record = model.build_host_orchestration_record(
                    fixture["context"],
                    fixture["invocation"],
                    fixture["response"],
                    fixture["session"],
                )
            self.assertEqual(record["reviewed_commit"], fixture["session"]["candidate"])
            self.assertNotIn(
                "derive_platform_orchestration_record",
                inspect.getsource(host_model),
            )

        negative_cases = (
            ("replacement", {"raw_view": "replacement"}),
            ("graft", {"raw_view": "graft"}),
            ("shallow", {"raw_view": "shallow"}),
            ("alternate", {"raw_view": "alternate"}),
            ("wrong-tree", {"observed_tree": "f" * 40}),
            ("dirty", {"clean": False}),
            ("missing-object", {"install_candidate": False}),
            ("unborn-head", {"observed_head": None, "unborn": True}),
            ("body-to-commit-drift", {"observed_head": "f" * 40}),
        )
        for case_id, options in negative_cases:
            with self.subTest(case_id=case_id):
                _, core, _, model = self.host_state_model()
                fixture = self.fixture()
                options = dict(options)
                if options.pop("unborn", False):
                    model.register_raw_commit("a" * 40, tree="a" * 40, parents=())
                    model.register_raw_commit(
                        "b" * 40,
                        tree=fixture["session"]["candidate_tree"],
                        parents=("a" * 40,),
                    )
                    model.register_worktree_observation(
                        fixture["context"]["worktree_handle"],
                        head=None,
                        tree=fixture["session"]["candidate_tree"],
                        clean=True,
                        view="raw",
                    )
                    model.register_attempt_predecessor("3" * 64, candidate="a" * 40)
                else:
                    self.install_facts(model, fixture, **options)
                before = model.snapshot()
                with self.assert_evidence_error(core.EvidenceError):
                    model.open_attempt(
                        {"context": fixture["context"], "session": fixture["session"]}
                    )
                self.assertEqual(model.snapshot(), before)

        _, core, _, model = self.host_state_model()
        wrong_relation = self.fixture()
        wrong_relation["session"]["candidate_relation"] = "equal"
        self.install_facts(model, wrong_relation)
        before = model.snapshot()
        with self.assert_evidence_error(core.EvidenceError):
            model.open_attempt(
                {"context": wrong_relation["context"], "session": wrong_relation["session"]}
            )
        self.assertEqual(model.snapshot(), before)

    def test_wave_b_baseline_is_immediate_predecessor_not_older_context_ancestor(self):
        older = "a" * 40
        immediate = "b" * 40
        candidate = "c" * 40
        _, core, _, model = self.host_state_model()
        model.register_raw_commit(older, tree="a" * 40, parents=())
        model.register_raw_commit(immediate, tree="b" * 40, parents=(older,))
        model.register_raw_commit(candidate, tree="4" * 40, parents=(immediate,))
        model.register_attempt_predecessor("3" * 64, candidate=immediate)

        invalid = self.fixture(
            wave="wave-b",
            role="implementation",
            baseline=older,
            candidate=candidate,
            reported_result_commit=candidate,
            review_target=older,
        )
        model.register_worktree_observation(
            invalid["context"]["worktree_handle"],
            head=candidate,
            tree="4" * 40,
            clean=True,
            view="raw",
        )
        before = model.snapshot()
        with self.assert_evidence_error(core.EvidenceError, "immediate predecessor"):
            model.open_attempt({"context": invalid["context"], "session": invalid["session"]})
        self.assertEqual(model.snapshot(), before)

        valid = self.fixture(
            wave="wave-b",
            role="implementation",
            baseline=immediate,
            candidate=candidate,
            reported_result_commit=candidate,
            review_target=immediate,
        )
        self.assertEqual(
            model.open_attempt({"context": valid["context"], "session": valid["session"]})[
                "baseline_commit"
            ],
            immediate,
        )


class AppendPlatformWireTests(unittest.TestCase):
    def append_platform_client(self):
        core = importlib.import_module("scripts.evidence_core")
        append = getattr(core, "append_platform_on_authenticated_socket", None)
        self.assertTrue(
            callable(append),
            "append_platform_on_authenticated_socket is required",
        )
        self.assertEqual(
            tuple(inspect.signature(append).parameters),
            (
                "sock",
                "context_handle",
                "invocation_handle",
                "response_handle",
            ),
        )
        return core, append

    def run_reference_wire(self, *, reported_record_sha256=None):
        core, append = self.append_platform_client()
        host_module = importlib.import_module(
            "scripts.tests.append_platform_host_model"
        )
        model = host_module.AppendPlatformHostModel()
        fixture = AppendPlatformStateTests.fixture()
        fixture["response"]["reported_record_sha256"] = reported_record_sha256
        state_helpers = AppendPlatformStateTests(methodName="runTest")
        state_helpers.install_facts(model, fixture)
        _, _, responded, handles = state_helpers.ready_lineage(model, fixture)

        client, raw_host = socket.socketpair(socket.AF_UNIX, socket.SOCK_STREAM)
        host = _AppendWireHostSocket(fileno=raw_host.detach())
        host.peer_principal = model.ROOT_SUPERVISOR_PRINCIPAL
        host.producer_session_id = fixture["session"]["producer_session_id"]
        precommit_snapshots = []

        def observe_frame(frame):
            if frame.get("body_kind") == "orchestration-record":
                precommit_snapshots.append(model.snapshot())

        host.frame_observer = observe_frame
        lease = model.bind_append_connection(handles, connection=host)
        host_errors = []

        def serve():
            try:
                model.serve_append_platform(host, lease)
            except BaseException as error:
                host_errors.append(error)

        thread = threading.Thread(target=serve)
        thread.start()
        try:
            result = append(
                client,
                handles["context_handle"],
                handles["invocation_handle"],
                handles["response_handle"],
            )
        finally:
            client.close()
            thread.join(timeout=3)
            sent_frames = copy.deepcopy(host.sent_frames)
            host.close()
        self.assertFalse(thread.is_alive(), "reference Host wire did not terminate")
        self.assertEqual(host_errors, [])
        return (
            core,
            model,
            fixture,
            responded,
            handles,
            result,
            sent_frames,
            precommit_snapshots,
        )

    def run_scripted_client(self, scenario):
        core, append = self.append_platform_client()
        fixture = AppendPlatformStateTests.fixture()
        host_module = importlib.import_module(
            "scripts.tests.append_platform_host_model"
        )
        host_builder = host_module.AppendPlatformHostModel()
        record = host_builder.build_host_orchestration_record(
            fixture["context"],
            fixture["invocation"],
            fixture["response"],
            fixture["session"],
        )
        if scenario == "mutated-record":
            record["agent_id"] = "mutated-host-agent"
        bodies = (
            fixture["context"],
            fixture["invocation"],
            fixture["response"],
            fixture["session"],
            record,
        )
        body_kinds = APPROVED_BODY_KINDS
        body_hashes = [
            hashlib.sha256(core.canonical_json_bytes(body)).hexdigest()
            for body in bodies
        ]
        handles = ("a" * 64, "b" * 64, "c" * 64)
        client, host = socket.socketpair(socket.AF_UNIX, socket.SOCK_STREAM)
        host_errors = []

        def scripted_host():
            deadline = time.monotonic() + 2.0
            try:
                begin = core.recv_host_frame(host, deadline)
                nonce = begin["transaction_nonce"]
                self.assertEqual(
                    begin,
                    {
                        "schema": "tersh-host-transaction-begin-v1",
                        "transaction_nonce": nonce,
                        "operation": "append-platform",
                        "context_handle": handles[0],
                        "invocation_handle": handles[1],
                        "response_handle": handles[2],
                    },
                )
                limit = 4 if scenario == "hash-only" else 5
                for ordinal, (body_kind, body, digest) in enumerate(
                    zip(body_kinds[:limit], bodies[:limit], body_hashes[:limit]),
                    start=1,
                ):
                    schema = "tersh-host-transaction-body-v1"
                    if scenario == "record-frame" and ordinal == 5:
                        schema = "tersh-host-transaction-record-begin-v1"
                    core.send_host_frame(
                        host,
                        {
                            "schema": schema,
                            "transaction_nonce": nonce,
                            "operation": "append-platform",
                            "body_kind": body_kind,
                            "ordinal": ordinal,
                            "total": 5,
                            "body": body,
                            "body_sha256": digest,
                        },
                        deadline,
                    )
                    if ordinal == 5 and scenario in {
                        "record-frame",
                        "mutated-record",
                    }:
                        try:
                            host.shutdown(socket.SHUT_WR)
                        except OSError:
                            pass
                        return
                core.send_host_frame(
                    host,
                    {
                        "schema": "tersh-host-transaction-body-end-v1",
                        "transaction_nonce": nonce,
                        "operation": "append-platform",
                        "total": limit,
                        "body_sha256s": body_hashes[:limit],
                    },
                    deadline,
                )
                if scenario == "hash-only":
                    try:
                        host.shutdown(socket.SHUT_WR)
                    except OSError:
                        # A rejecting client may already have closed its peer.
                        pass
                    return

                commit = core.recv_host_frame(host, deadline)
                self.assertEqual(
                    commit,
                    {
                        "schema": "tersh-host-transaction-commit-v1",
                        "transaction_nonce": nonce,
                        "operation": "append-platform",
                        "body_sha256s": body_hashes,
                        "record_facts": {
                            "evidence_id": fixture["context"]["evidence_id"],
                            "evidence_attempt": fixture["context"]["evidence_attempt"],
                            "run_binding": fixture["context"]["run_binding"],
                            "candidate": fixture["session"]["candidate"],
                            "destination": fixture["session"]["destination"],
                            "record_sha256": body_hashes[4],
                        },
                    },
                )
                request_end = core.recv_host_frame(host, deadline)
                self.assertEqual(
                    request_end,
                    {
                        "schema": "tersh-host-transaction-request-end-v1",
                        "transaction_nonce": nonce,
                        "operation": "append-platform",
                        "commit_sha256": hashlib.sha256(
                            core.canonical_json_bytes(commit)
                        ).hexdigest(),
                    },
                )
                core.require_host_eof(host, deadline)
                receipt = AppendPlatformSchemaTests.producer_receipt_fixture(
                    fixture["session"],
                    record,
                )
                if scenario == "receipt-drift":
                    receipt["destination"] = "attempt-002/alternate.json"
                result = {
                    "schema": "tersh-host-record-result-v1",
                    "receipt": receipt,
                }
                if scenario == "private-result":
                    result["pending_report_authority"] = "private-canary"
                reply = {
                    "schema": "tersh-host-transaction-reply-v1",
                    "transaction_nonce": nonce,
                    "operation": "append-platform",
                    "body_sha256s": body_hashes,
                    "result": result,
                }
                core.send_host_frame(host, reply, deadline)
                if scenario in {"receipt-drift", "private-result"}:
                    try:
                        host.shutdown(socket.SHUT_WR)
                    except OSError:
                        pass
                    return
                core.send_host_frame(
                    host,
                    {
                        "schema": "tersh-host-transaction-reply-end-v1",
                        "transaction_nonce": nonce,
                        "operation": "append-platform",
                        "reply_sha256": hashlib.sha256(
                            core.canonical_json_bytes(reply)
                        ).hexdigest(),
                    },
                    deadline,
                )
                host.shutdown(socket.SHUT_WR)
            except BaseException as error:
                host_errors.append(error)
            finally:
                host.close()

        thread = threading.Thread(target=scripted_host)
        thread.start()
        result = None
        error = None
        try:
            result = append(client, *handles)
        except BaseException as caught:
            error = caught
        finally:
            client.close()
            thread.join(timeout=3)
        self.assertFalse(thread.is_alive(), f"scripted Host hung for {scenario}")
        self.assertEqual(host_errors, [])
        return core, result, error

    def test_append_platform_exact_session_and_host_built_record_body_order(self):
        _, model, _, responded, handles, result, sent_frames, _ = (
            self.run_reference_wire()
        )
        self.assertEqual(result["schema"], "tersh-host-record-result-v1")
        snapshot = model.snapshot()
        self.assertEqual(
            snapshot["lineages"][responded["lineage_id"]]["state"],
            "APPENDED_PLATFORM",
        )
        self.assertEqual(len(snapshot["records"]), 1)
        self.assertEqual(
            [frame["body_kind"] for frame in sent_frames[:5]],
            list(APPROVED_BODY_KINDS),
        )
        self.assertEqual(
            [frame["ordinal"] for frame in sent_frames[:5]],
            [1, 2, 3, 4, 5],
        )
        self.assertTrue(all(frame["total"] == 5 for frame in sent_frames[:5]))
        self.assertEqual(
            sent_frames[5],
            {
                "schema": "tersh-host-transaction-body-end-v1",
                "transaction_nonce": sent_frames[0]["transaction_nonce"],
                "operation": "append-platform",
                "total": 5,
                "body_sha256s": [
                    frame["body_sha256"] for frame in sent_frames[:5]
                ],
            },
        )
        self.assertTrue(
            all(not snapshot["handles"][handle]["live"] for handle in (
                handles["context_handle"],
                handles["invocation_handle"],
                handles["response_handle"],
            ))
        )

    def test_host_record_constructor_is_independent_from_client_derivation(self):
        core, _ = self.append_platform_client()
        host_module = importlib.import_module(
            "scripts.tests.append_platform_host_model"
        )
        model = host_module.AppendPlatformHostModel()
        fixture = AppendPlatformStateTests.fixture()
        tripwire = AssertionError("Host constructor called client derivation")
        with mock.patch.object(
            core,
            "derive_platform_orchestration_record",
            side_effect=tripwire,
        ):
            record = model.build_host_orchestration_record(
                fixture["context"],
                fixture["invocation"],
                fixture["response"],
                fixture["session"],
            )
        host_tree = ast.parse(
            inspect.getsource(host_module),
            filename=str(host_module.__file__),
        )
        forbidden_references = [
            node
            for node in ast.walk(host_tree)
            if (
                isinstance(node, ast.Name)
                and node.id == "derive_platform_orchestration_record"
            )
            or (
                isinstance(node, ast.Attribute)
                and node.attr == "derive_platform_orchestration_record"
            )
            or (
                isinstance(node, ast.alias)
                and (
                    node.name == "derive_platform_orchestration_record"
                    or node.asname == "derive_platform_orchestration_record"
                )
            )
        ]
        self.assertEqual(forbidden_references, [])
        mutated = copy.deepcopy(record)
        mutated["agent_id"] = "mutated-host-agent"
        derived = core.derive_platform_orchestration_record(
            fixture["context"],
            fixture["invocation"],
            fixture["response"],
            fixture["session"],
        )
        self.assertNotEqual(mutated, derived)
        scripted_core, result, error = self.run_scripted_client("mutated-record")
        self.assertIsNone(result)
        self.assertIs(type(error), scripted_core.EvidenceError)

    def test_append_platform_rejects_record_frame_or_hash_only_commit(self):
        for scenario in ("record-frame", "hash-only"):
            with self.subTest(scenario=scenario):
                core, result, error = self.run_scripted_client(scenario)
                self.assertIsNone(result)
                self.assertIs(type(error), core.EvidenceError)

    def test_host_spools_exact_record_body_sent_before_commit(self):
        core, model, _, _, _, result, _, precommit_snapshots = (
            self.run_reference_wire()
        )
        self.assertEqual(len(precommit_snapshots), 1)
        self.assertEqual(precommit_snapshots[0]["blobs"], {})
        self.assertEqual(precommit_snapshots[0]["receipts"], [])
        snapshot = model.snapshot()
        record_row = snapshot["records"][0]
        frozen = core.canonical_json_bytes(record_row["body"])
        self.assertEqual(record_row["body_bytes"], frozen)
        self.assertEqual(
            snapshot["blobs"],
            {hashlib.sha256(frozen).hexdigest(): frozen},
        )
        self.assertEqual(
            result["receipt"]["body_sha256"],
            hashlib.sha256(frozen).hexdigest(),
        )

    def test_record_reply_receipt_joins_session_route_body_and_chain(self):
        core, model, fixture, _, _, result, _, _ = self.run_reference_wire()
        receipt = result["receipt"]
        session = fixture["session"]
        record = model.snapshot()["records"][0]["body"]
        validated = core._validate_append_platform_producer_receipt(
            receipt,
            recorder_session=session,
            record=record,
        )
        self.assertEqual(validated, receipt)
        self.assertEqual(receipt["sequence"], session["next_receipt_sequence"])
        self.assertEqual(receipt["previous_receipt_id"], session["previous_receipt_id"])
        self.assertNotIn("authority", result)
        for scenario in ("receipt-drift", "private-result"):
            with self.subTest(scenario=scenario):
                scripted_core, invalid_result, error = self.run_scripted_client(
                    scenario
                )
                self.assertIsNone(invalid_result)
                self.assertIs(type(error), scripted_core.EvidenceError)

    def test_append_platform_one_absolute_deadline_covers_begin_bodies_commit_request_end_eof_and_reply(self):
        core, append = self.append_platform_client()
        fixture = AppendPlatformStateTests.fixture()
        host_builder = importlib.import_module(
            "scripts.tests.append_platform_host_model"
        ).AppendPlatformHostModel()
        record = host_builder.build_host_orchestration_record(
            fixture["context"],
            fixture["invocation"],
            fixture["response"],
            fixture["session"],
        )
        bodies = {
            "context": fixture["context"],
            "invocation": fixture["invocation"],
            "response": fixture["response"],
            "recorder-session": fixture["session"],
            "orchestration-record": record,
        }
        body_hashes = [
            hashlib.sha256(core.canonical_json_bytes(bodies[kind])).hexdigest()
            for kind in APPROVED_BODY_KINDS
        ]
        nonce = "f" * 64
        receipt = AppendPlatformSchemaTests.producer_receipt_fixture(
            fixture["session"],
            record,
        )
        reply = {
            "schema": "tersh-host-transaction-reply-v1",
            "transaction_nonce": nonce,
            "operation": "append-platform",
            "body_sha256s": body_hashes,
            "result": {
                "schema": "tersh-host-record-result-v1",
                "receipt": receipt,
            },
        }
        reply_end = {
            "schema": "tersh-host-transaction-reply-end-v1",
            "transaction_nonce": nonce,
            "operation": "append-platform",
            "reply_sha256": hashlib.sha256(
                core.canonical_json_bytes(reply)
            ).hexdigest(),
        }
        observed_deadlines = []

        def receive_bodies(*args):
            observed_deadlines.append(args[-1])
            return copy.deepcopy(bodies), list(body_hashes)

        def receive_reply(_sock, deadline):
            observed_deadlines.append(deadline)
            return copy.deepcopy(
                receive_reply.frames.pop(0)
            )

        receive_reply.frames = [reply, reply_end]
        client, peer = socket.socketpair(socket.AF_UNIX, socket.SOCK_STREAM)
        try:
            with (
                mock.patch.object(core.time, "monotonic", return_value=100.0) as clock,
                mock.patch.object(core.secrets, "token_hex", return_value=nonce),
                mock.patch.object(
                    core,
                    "send_host_frame",
                    side_effect=lambda _sock, _value, deadline: observed_deadlines.append(deadline),
                ),
                mock.patch.object(
                    core,
                    "_receive_host_body_sequence",
                    side_effect=receive_bodies,
                ),
                mock.patch.object(
                    core,
                    "_send_host_request_end",
                    side_effect=lambda _sock, _nonce, _operation, _commit, deadline: observed_deadlines.append(deadline),
                ),
                mock.patch.object(core, "recv_host_frame", side_effect=receive_reply),
                mock.patch.object(
                    core,
                    "require_host_eof",
                    side_effect=lambda _sock, deadline: observed_deadlines.append(deadline),
                ),
            ):
                result = append(client, "a" * 64, "b" * 64, "c" * 64)
            self.assertEqual(result["receipt"], receipt)
            self.assertEqual(observed_deadlines, [105.0] * 6)
            clock.assert_called_once_with()
        finally:
            client.close()
            peer.close()

    def test_append_platform_deadline_is_not_reset_or_caller_configurable(self):
        core, append = self.append_platform_client()
        function_tree = ast.parse(inspect.getsource(append))
        monotonic_calls = [
            node
            for node in ast.walk(function_tree)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and isinstance(node.func.value, ast.Name)
            and node.func.value.id == "time"
            and node.func.attr == "monotonic"
        ]
        self.assertEqual(len(monotonic_calls), 1)
        client, peer = socket.socketpair(socket.AF_UNIX, socket.SOCK_STREAM)
        try:
            with self.assertRaises(TypeError):
                append(
                    client,
                    "a" * 64,
                    "b" * 64,
                    "c" * 64,
                    deadline=1.0,
                )
            with mock.patch.object(core.time, "monotonic", return_value=105.001):
                with self.assertRaises(core.EvidenceError):
                    core._remaining_host_deadline(105.0)
        finally:
            client.close()
            peer.close()


class AppendPlatformAtomicityTests(unittest.TestCase):
    PRE_LINEARIZATION_FAULTS = (
        "wire.after-begin",
        "wire.after-body-context",
        "wire.after-body-invocation",
        "wire.after-body-response",
        "wire.after-body-session",
        "wire.after-body-record",
        "wire.after-body-end",
        "wire.after-commit",
        "wire.after-request-end",
        "wire.after-client-eof",
        "linearize.after-session-consume",
        "linearize.after-handle-consume",
        "linearize.after-blob-insert",
        "linearize.after-receipt-append",
        "linearize.after-authority-insert",
        "linearize.before-commit",
    )
    POST_LINEARIZATION_FAULTS = (
        "linearize.after-commit",
        "reply.after-reply",
        "reply.after-reply-end",
        "reply.before-eof",
    )

    def ready_model(self, *, reported_record_sha256="e" * 64):
        host_module = importlib.import_module(
            "scripts.tests.append_platform_host_model"
        )
        core = importlib.import_module("scripts.evidence_core")
        clock = host_module.FixtureClock()
        model = host_module.AppendPlatformHostModel(clock=clock)
        fixture = AppendPlatformStateTests.fixture()
        fixture["response"]["reported_record_sha256"] = reported_record_sha256
        helpers = AppendPlatformStateTests(methodName="runTest")
        helpers.install_facts(model, fixture)
        _, _, responded, handles = helpers.ready_lineage(model, fixture)
        connection = _AppendStateConnection(
            90,
            "atomic-connection",
            fixture["session"]["producer_session_id"],
        )
        lease = model.bind_append_connection(handles, connection=connection)
        return core, clock, model, fixture, responded, handles, connection, lease

    @staticmethod
    def ledger_vector(snapshot):
        live_handles = {
            handle_id: row["live"] for handle_id, row in snapshot["handles"].items()
        }
        chains = {
            binding_id: copy.deepcopy(row["marker_receipt_chain"])
            for binding_id, row in snapshot["attempts"].items()
        }
        return {
            "live_handles": live_handles,
            "chains": chains,
            "blobs": copy.deepcopy(snapshot["blobs"]),
            "receipts": copy.deepcopy(snapshot["receipts"]),
            "authorities": copy.deepcopy(snapshot["authorities"]),
            "records": copy.deepcopy(snapshot["records"]),
        }

    def test_append_platform_atomic_handle_blob_receipt_authority_fault_matrix(self):
        for index, fault_id in enumerate(
            (
                "linearize.after-session-consume",
                "linearize.after-handle-consume",
                "linearize.after-blob-insert",
                "linearize.after-receipt-append",
                "linearize.after-authority-insert",
                "linearize.before-commit",
            )
        ):
            with self.subTest(fault_id=fault_id):
                core, _, model, fixture, _, handles, connection, lease = (
                    self.ready_model()
                )
                before = self.ledger_vector(model.snapshot())
                with self.assertRaises(core.EvidenceError):
                    model.append_platform(
                        lease,
                        connection=connection,
                        transaction_nonce=f"{index + 1:x}" * 64,
                        fault_at=fault_id,
                    )
                self.assertEqual(self.ledger_vector(model.snapshot()), before)
                retry_connection = _AppendStateConnection(
                    100 + index,
                    f"atomic-retry-{index}",
                    fixture["session"]["producer_session_id"],
                )
                retry_lease = model.bind_append_connection(
                    handles, connection=retry_connection
                )
                receipt = model.append_platform(
                    retry_lease,
                    connection=retry_connection,
                    transaction_nonce=f"{index + 7:x}" * 64,
                )
                self.assertEqual(
                    receipt["sequence"],
                    fixture["session"]["next_receipt_sequence"],
                )
                self.assertEqual(len(model.snapshot()["authorities"]), 1)

    def test_append_platform_deadline_after_commit_before_request_end_or_eof_is_prelinearization(self):
        for fault_id in (
            "wire.after-commit",
            "wire.after-request-end",
            "wire.after-client-eof",
        ):
            with self.subTest(fault_id=fault_id):
                core, clock, model, _, _, handles, connection, lease = (
                    self.ready_model()
                )
                before = self.ledger_vector(model.snapshot())
                clock.advance(4.999)
                with self.assertRaises(core.EvidenceError):
                    model.append_platform(
                        lease,
                        connection=connection,
                        transaction_nonce="a" * 64,
                        fault_at=fault_id,
                    )
                self.assertEqual(self.ledger_vector(model.snapshot()), before)
                self.assertTrue(
                    all(model.snapshot()["handles"][handle]["live"] for handle in (
                        handles["context_handle"],
                        handles["invocation_handle"],
                        handles["response_handle"],
                    ))
                )

    def test_append_platform_concurrent_calls_create_one_receipt_and_authority(self):
        core, _, model, _, _, _, connection, lease = self.ready_model()
        barrier = threading.Barrier(2)
        outcomes = []

        def append_once(nonce):
            try:
                outcomes.append(
                    model.append_platform(
                        lease,
                        connection=connection,
                        transaction_nonce=nonce,
                        start_barrier=barrier,
                    )
                )
            except core.EvidenceError as error:
                outcomes.append(error)

        threads = [
            threading.Thread(target=append_once, args=(digit * 64,))
            for digit in ("b", "c")
        ]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join(timeout=3)
            self.assertFalse(thread.is_alive())
        self.assertEqual(sum(type(value) is dict for value in outcomes), 1)
        self.assertEqual(sum(type(value) is core.EvidenceError for value in outcomes), 1)
        snapshot = model.snapshot()
        self.assertEqual(len(snapshot["blobs"]), 1)
        self.assertEqual(len(snapshot["receipts"]), 1)
        self.assertEqual(len(snapshot["authorities"]), 1)

    def test_append_platform_chain_head_drift_fails_before_linearization_and_retries(self):
        core, _, model, fixture, _, handles, connection, lease = self.ready_model()
        model.force_receipt_chain_head(
            fixture["session"]["attempt_binding_id"],
            next_receipt_sequence=2,
            previous_receipt_id="f" * 64,
        )
        with self.assertRaises(core.EvidenceError):
            model.append_platform(
                lease,
                connection=connection,
                transaction_nonce="d" * 64,
            )
        self.assertEqual(model.snapshot()["receipts"], [])

        refreshed = copy.deepcopy(fixture["session"])
        refreshed.update(
            {
                "producer_session_id": "8" * 64,
                "next_receipt_sequence": 2,
                "previous_receipt_id": "f" * 64,
            }
        )
        model.open_attempt({"context": fixture["context"], "session": refreshed})
        retry_handles = copy.deepcopy(handles)
        retry_handles["producer_session_id"] = refreshed["producer_session_id"]
        retry_connection = _AppendStateConnection(
            91, "chain-retry", refreshed["producer_session_id"]
        )
        retry_lease = model.bind_append_connection(
            retry_handles, connection=retry_connection
        )
        receipt = model.append_platform(
            retry_lease,
            connection=retry_connection,
            transaction_nonce="e" * 64,
        )
        self.assertEqual(receipt["sequence"], 2)
        self.assertEqual(receipt["previous_receipt_id"], "f" * 64)

    def test_every_prelinearization_failure_preserves_exact_state_vector(self):
        for index, fault_id in enumerate(self.PRE_LINEARIZATION_FAULTS):
            with self.subTest(fault_id=fault_id):
                core, _, model, _, _, _, connection, lease = self.ready_model()
                before = self.ledger_vector(model.snapshot())
                with self.assertRaises(core.EvidenceError):
                    model.append_platform(
                        lease,
                        connection=connection,
                        transaction_nonce=f"{index + 1:064x}",
                        fault_at=fault_id,
                    )
                after = model.snapshot()
                self.assertEqual(self.ledger_vector(after), before)
                self.assertFalse(after["leases"][lease["lease_id"]]["valid"])
                self.assertIsNone(
                    after["sessions"][lease["producer_session_id"]][
                        "active_lease_id"
                    ]
                )

    def test_every_postcommit_reply_fault_preserves_one_durable_state_vector(self):
        for index, fault_id in enumerate(self.POST_LINEARIZATION_FAULTS):
            with self.subTest(fault_id=fault_id):
                core, _, model, _, _, _, connection, lease = self.ready_model()
                with self.assertRaises(core.EvidenceError):
                    model.append_platform(
                        lease,
                        connection=connection,
                        transaction_nonce=f"{index + 33:064x}",
                        fault_at=fault_id,
                    )
                committed = model.snapshot()
                self.assertEqual(len(committed["blobs"]), 1)
                self.assertEqual(len(committed["receipts"]), 1)
                self.assertEqual(len(committed["authorities"]), 1)
                with self.assertRaises(core.EvidenceError):
                    model.append_platform(
                        lease,
                        connection=connection,
                        transaction_nonce=f"{index + 33:064x}",
                    )
                self.assertEqual(model.snapshot(), committed)


class AppendPlatformRecoveryTests(unittest.TestCase):
    RECOVER_FAULTS = (
        "recover.after-old-tuple-invalidate",
        "recover.after-new-tuple-insert",
        "recover.after-replay-row",
        "recover.before-commit",
    )
    ABANDON_FAULTS = (
        "abandon.after-handle-invalidate",
        "abandon.after-terminal-row",
        "abandon.after-attempt-state",
        "abandon.after-replay-row",
        "abandon.before-commit",
    )

    def lineage_at(self, state):
        host_module = importlib.import_module(
            "scripts.tests.append_platform_host_model"
        )
        core = importlib.import_module("scripts.evidence_core")
        model = host_module.AppendPlatformHostModel()
        fixture = AppendPlatformStateTests.fixture()
        helpers = AppendPlatformStateTests(methodName="runTest")
        helpers.install_facts(model, fixture)
        model.open_attempt({"context": fixture["context"], "session": fixture["session"]})
        result = model.seed_lineage(context=fixture["context"])
        if state in {"invoked", "responded-platform"}:
            result = model.capture_invocation(
                result["context_handle"], fixture["invocation"], lose_reply=True
            )
        if state == "responded-platform":
            result = model.capture_response(
                result["context_handle"], fixture["response"], lose_reply=True
            )
        return core, model, fixture, result

    @staticmethod
    def recover_request(fixture, state, transition, generation=0):
        return {
            "schema": "tersh-host-recover-dispatch-lineage-request-v1",
            "context_nonce": fixture["context"]["context_nonce"],
            "expected_state": state,
            "transition_index": transition,
            "recovery_generation": generation,
            "reason": "capture-reply-unrecoverable",
        }

    @staticmethod
    def abandon_request(fixture, state, transition, generation=0):
        return {
            "schema": "tersh-host-abandon-dispatch-lineage-request-v1",
            "context_nonce": fixture["context"]["context_nonce"],
            "expected_state": state,
            "transition_index": transition,
            "recovery_generation": generation,
            "reason": "capture-reply-unrecoverable",
        }

    def invoke(self, model, operation, request, **kwargs):
        return model.invoke_root_internal(
            operation,
            request,
            principal=model.ROOT_SUPERVISOR_PRINCIPAL,
            **kwargs,
        )

    def test_capture_reply_orphan_recovery_rotates_private_handles_and_preserves_response(self):
        _, model, fixture, old = self.lineage_at("responded-platform")
        old_bodies = {
            kind: copy.deepcopy(model.snapshot()["handles"][old[f"{kind}_handle"]]["body"])
            for kind in ("context", "invocation", "response")
        }
        result = self.invoke(
            model,
            "recover-dispatch-lineage",
            self.recover_request(fixture, "responded-platform", 2),
        )
        self.assertEqual(
            set(result),
            {
                "schema",
                "context_nonce",
                "state",
                "transition_index",
                "recovery_generation",
                "context_handle",
                "invocation_handle",
                "response_handle",
            },
        )
        self.assertEqual(result["recovery_generation"], 1)
        snapshot = model.snapshot()
        for kind in ("context", "invocation", "response"):
            self.assertNotEqual(result[f"{kind}_handle"], old[f"{kind}_handle"])
            self.assertFalse(snapshot["handles"][old[f"{kind}_handle"]]["live"])
            self.assertEqual(
                snapshot["handles"][result[f"{kind}_handle"]]["body"],
                old_bodies[kind],
            )

    def test_recovery_generation_allows_invoked_then_responded_lost_reply_recovery(self):
        _, model, fixture, _ = self.lineage_at("invoked")
        invoked = self.invoke(
            model,
            "recover-dispatch-lineage",
            self.recover_request(fixture, "invoked", 1),
        )
        responded = model.capture_response(
            invoked["context_handle"], fixture["response"], lose_reply=True
        )
        recovered = self.invoke(
            model,
            "recover-dispatch-lineage",
            self.recover_request(fixture, "responded-platform", 2, generation=1),
        )
        self.assertEqual(recovered["recovery_generation"], 2)
        self.assertNotEqual(recovered["response_handle"], responded["response_handle"])

    def test_recovery_result_state_generation_and_nullable_handles_are_exact(self):
        cases = (
            ("created", 0, (True, False, False)),
            ("invoked", 1, (True, True, False)),
            ("responded-platform", 2, (True, True, True)),
        )
        for state, transition, applicable in cases:
            with self.subTest(state=state):
                _, model, fixture, _ = self.lineage_at(state)
                result = self.invoke(
                    model,
                    "recover-dispatch-lineage",
                    self.recover_request(fixture, state, transition),
                )
                self.assertEqual(result["state"], state)
                self.assertEqual(result["transition_index"], transition)
                self.assertEqual(result["recovery_generation"], 1)
                for kind, required in zip(
                    ("context", "invocation", "response"), applicable
                ):
                    self.assertIs(type(result[f"{kind}_handle"]), str if required else type(None))
                nonnull = [
                    result[f"{kind}_handle"]
                    for kind in ("context", "invocation", "response")
                    if result[f"{kind}_handle"] is not None
                ]
                self.assertEqual(len(nonnull), len(set(nonnull)))

    def test_capture_orphan_abandon_rejects_responded_state_and_never_reuses_persisted_attempt(self):
        core, responded_model, responded_fixture, _ = self.lineage_at(
            "responded-platform"
        )
        before = responded_model.snapshot()
        with self.assertRaises(core.EvidenceError):
            self.invoke(
                responded_model,
                "abandon-dispatch-lineage",
                self.abandon_request(
                    responded_fixture, "responded-platform", 2
                ),
            )
        self.assertEqual(responded_model.snapshot(), before)

        _, model, fixture, _ = self.lineage_at("created")
        result = self.invoke(
            model,
            "abandon-dispatch-lineage",
            self.abandon_request(fixture, "created", 0),
        )
        self.assertEqual(result["state"], "abandoned")
        attempt = model.snapshot()["attempts"][fixture["session"]["attempt_binding_id"]]
        self.assertEqual(attempt["state"], "CLOSING_FAILED")

    def test_recover_and_abandon_closed_schemas_and_atomic_fault_matrix(self):
        for fault_id in self.RECOVER_FAULTS:
            with self.subTest(fault_id=fault_id):
                core, model, fixture, _ = self.lineage_at("created")
                before = model.snapshot()
                with self.assertRaises(core.EvidenceError):
                    self.invoke(
                        model,
                        "recover-dispatch-lineage",
                        self.recover_request(fixture, "created", 0),
                        fault_at=fault_id,
                    )
                self.assertEqual(model.snapshot(), before)
        for fault_id in self.ABANDON_FAULTS:
            with self.subTest(fault_id=fault_id):
                core, model, fixture, _ = self.lineage_at("invoked")
                before = model.snapshot()
                with self.assertRaises(core.EvidenceError):
                    self.invoke(
                        model,
                        "abandon-dispatch-lineage",
                        self.abandon_request(fixture, "invoked", 1),
                        fault_at=fault_id,
                    )
                self.assertEqual(model.snapshot(), before)

        core, model, fixture, _ = self.lineage_at("created")
        invalid = self.recover_request(fixture, "created", 0)
        invalid["lineage_id"] = "caller-selector"
        before = model.snapshot()
        with self.assertRaises(core.EvidenceError):
            self.invoke(model, "recover-dispatch-lineage", invalid)
        self.assertEqual(model.snapshot(), before)

        core, model, fixture, _ = self.lineage_at("created")
        request = self.recover_request(fixture, "created", 0)
        with self.assertRaises(core.EvidenceError):
            self.invoke(
                model,
                "recover-dispatch-lineage",
                request,
                fault_at="recover.after-commit-before-result",
            )
        self.assertEqual(self.invoke(model, "recover-dispatch-lineage", request)["recovery_generation"], 1)

    def test_recover_abandon_capture_and_append_race_has_exactly_one_durable_winner(self):
        core, model, fixture, _ = self.lineage_at("created")
        barrier = threading.Barrier(2)
        outcomes = []

        def run(operation, request):
            barrier.wait(timeout=2)
            try:
                outcomes.append(self.invoke(model, operation, request))
            except core.EvidenceError as error:
                outcomes.append(error)

        threads = (
            threading.Thread(
                target=run,
                args=(
                    "recover-dispatch-lineage",
                    self.recover_request(fixture, "created", 0),
                ),
            ),
            threading.Thread(
                target=run,
                args=(
                    "abandon-dispatch-lineage",
                    self.abandon_request(fixture, "created", 0),
                ),
            ),
        )
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join(timeout=3)
            self.assertFalse(thread.is_alive())
        self.assertEqual(sum(type(value) is dict for value in outcomes), 1)
        self.assertEqual(sum(type(value) is core.EvidenceError for value in outcomes), 1)

    def test_binding_retaining_abandon_enters_closing_failed_without_reusing_binding(self):
        _, model, fixture, old = self.lineage_at("invoked")
        result = self.invoke(
            model,
            "abandon-dispatch-lineage",
            self.abandon_request(fixture, "invoked", 1),
        )
        self.assertEqual(result["state"], "abandoned")
        snapshot = model.snapshot()
        lineage = snapshot["lineages"][old["lineage_id"]]
        self.assertEqual(lineage["state"], "ABANDONED")
        self.assertTrue(all(not snapshot["handles"][handle]["live"] for handle in (
            old["context_handle"], old["invocation_handle"]
        )))
        attempt = snapshot["attempts"][fixture["session"]["attempt_binding_id"]]
        self.assertEqual(attempt["state"], "CLOSING_FAILED")


class AppendPlatformFailureTests(unittest.TestCase):
    DISPATCH_REASONS = (
        "candidate-object-missing",
        "candidate-relation-invalid",
        "worktree-identity-drift",
        "attempt-policy-drift",
        "record-construction-invalid",
    )
    AUTHORITY_REASONS = (
        "draft-missing",
        "draft-digest-mismatch",
        "draft-schema-invalid",
        "draft-path-invalid",
        "sealer-policy-invalid",
    )

    def responded_model(self):
        helper = AppendPlatformRecoveryTests(methodName="runTest")
        return helper.lineage_at("responded-platform")

    def authority_model(self):
        helper = AppendPlatformAtomicityTests(methodName="runTest")
        core, _, model, fixture, responded, _, connection, lease = (
            helper.ready_model(reported_record_sha256="e" * 64)
        )
        model.append_platform(
            lease,
            connection=connection,
            transaction_nonce="9" * 64,
        )
        return core, model, fixture, responded

    @staticmethod
    def dispatch_request(fixture, reason):
        return {
            "schema": "tersh-host-fail-responded-dispatch-request-v1",
            "context_nonce": fixture["context"]["context_nonce"],
            "transition_index": 2,
            "recovery_generation": 0,
            "reason": reason,
        }

    @staticmethod
    def authority_request(fixture, reason):
        return {
            "schema": "tersh-host-fail-agent-report-authority-request-v1",
            "context_nonce": fixture["context"]["context_nonce"],
            "reason": reason,
        }

    def invoke(self, model, operation, request, **kwargs):
        return model.invoke_root_internal(
            operation,
            request,
            principal=model.ROOT_SUPERVISOR_PRINCIPAL,
            **kwargs,
        )

    def test_irrecoverable_responded_dispatch_atomically_records_failure(self):
        _, model, fixture, responded = self.responded_model()
        reason = "candidate-object-missing"
        model.set_failure_observation(fixture["context"]["context_nonce"], reason)
        result = self.invoke(
            model,
            "fail-responded-dispatch",
            self.dispatch_request(fixture, reason),
        )
        self.assertEqual(result["state"], "failed")
        snapshot = model.snapshot()
        self.assertEqual(len(snapshot["receipts"]), 1)
        self.assertEqual(len(snapshot["records"]), 1)
        self.assertEqual(snapshot["lineages"][responded["lineage_id"]]["state"], "DISPATCH_FAILED")
        self.assertEqual(
            snapshot["attempts"][fixture["session"]["attempt_binding_id"]]["state"],
            "CLOSING_FAILED",
        )
        self.assertEqual(snapshot["authorities"], {})

    def test_dispatch_failure_lost_reply_replays_one_receipt_and_finalizer_rejects_attempt(self):
        core, model, fixture, _ = self.responded_model()
        reason = "candidate-relation-invalid"
        model.set_failure_observation(fixture["context"]["context_nonce"], reason)
        request = self.dispatch_request(fixture, reason)
        with self.assertRaises(core.EvidenceError):
            self.invoke(
                model,
                "fail-responded-dispatch",
                request,
                fault_at="fail-dispatch.after-commit-before-result",
            )
        replay = self.invoke(model, "fail-responded-dispatch", request)
        snapshot = model.snapshot()
        self.assertEqual(replay["failure_receipt_id"], snapshot["receipts"][0]["receipt_id"])
        self.assertEqual(len(snapshot["receipts"]), 1)
        self.assertEqual(
            snapshot["attempts"][fixture["session"]["attempt_binding_id"]]["state"],
            "CLOSING_FAILED",
        )

    def test_dispatch_failure_body_receipt_and_route_source_map_is_closed(self):
        _, model, fixture, _ = self.responded_model()
        reason = "worktree-identity-drift"
        model.set_failure_observation(fixture["context"]["context_nonce"], reason)
        self.invoke(model, "fail-responded-dispatch", self.dispatch_request(fixture, reason))
        snapshot = model.snapshot()
        record = snapshot["records"][0]
        body = record["body"]
        self.assertEqual(
            set(body),
            {
                "schema",
                "evidence_id",
                "evidence_attempt",
                "candidate",
                "context_nonce",
                "dispatch_id",
                "reason",
                "context",
                "invocation",
                "response",
                "created_at",
            },
        )
        receipt = snapshot["receipts"][0]
        expected_destination = (
            f"attempt-{fixture['context']['evidence_attempt']}/"
            f"candidate-{fixture['session']['candidate']}/"
            f"orchestration-failures/{fixture['invocation']['dispatch_id']}.json"
        )
        self.assertEqual(receipt["destination"], expected_destination)
        self.assertEqual(receipt["body_sha256"], record["body_sha256"])
        self.assertIsNone(receipt["dispatch_id"])
        self.assertIsNone(receipt["reported_record_sha256"])
        self.assertIsNone(receipt["environment_capability"])

    def test_unsealable_report_authority_atomically_records_callback_failure(self):
        _, model, fixture, responded = self.authority_model()
        reason = "draft-digest-mismatch"
        model.set_failure_observation(fixture["context"]["context_nonce"], reason)
        result = self.invoke(
            model,
            "fail-agent-report-authority",
            self.authority_request(fixture, reason),
        )
        self.assertEqual(result["state"], "failed")
        snapshot = model.snapshot()
        self.assertEqual(snapshot["authorities"], {})
        self.assertEqual(snapshot["lineages"][responded["lineage_id"]]["state"], "REPORT_FAILED")
        self.assertEqual(len(snapshot["receipts"]), 2)
        self.assertEqual(snapshot["records"][-1]["body"]["reported_record_sha256"], "e" * 64)

    def test_report_authority_failure_lost_result_and_sealer_race_have_one_winner(self):
        core, model, fixture, _ = self.authority_model()
        reason = "draft-missing"
        nonce = fixture["context"]["context_nonce"]
        model.set_failure_observation(nonce, reason)
        request = self.authority_request(fixture, reason)
        with self.assertRaises(core.EvidenceError):
            self.invoke(
                model,
                "fail-agent-report-authority",
                request,
                fault_at="fail-authority.after-commit-before-result",
            )
        replay = self.invoke(model, "fail-agent-report-authority", request)
        self.assertEqual(replay["state"], "failed")
        with self.assertRaises(core.EvidenceError):
            model.seal_agent_report_authority(nonce)

        _, race_model, race_fixture, _ = self.authority_model()
        race_nonce = race_fixture["context"]["context_nonce"]
        race_model.set_failure_observation(race_nonce, reason)
        barrier = threading.Barrier(2)
        outcomes = []

        def fail():
            barrier.wait(timeout=2)
            try:
                outcomes.append(
                    self.invoke(
                        race_model,
                        "fail-agent-report-authority",
                        self.authority_request(race_fixture, reason),
                    )
                )
            except core.EvidenceError as error:
                outcomes.append(error)

        def seal():
            barrier.wait(timeout=2)
            try:
                outcomes.append(race_model.seal_agent_report_authority(race_nonce))
            except core.EvidenceError as error:
                outcomes.append(error)

        threads = (threading.Thread(target=fail), threading.Thread(target=seal))
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join(timeout=3)
            self.assertFalse(thread.is_alive())
        self.assertEqual(sum(type(value) is dict for value in outcomes), 1)
        self.assertEqual(sum(type(value) is core.EvidenceError for value in outcomes), 1)

    def test_dispatch_and_report_failure_reasons_require_current_host_observation(self):
        for reason in self.DISPATCH_REASONS:
            with self.subTest(kind="dispatch", reason=reason):
                core, model, fixture, _ = self.responded_model()
                before = model.snapshot()
                with self.assertRaises(core.EvidenceError):
                    self.invoke(
                        model,
                        "fail-responded-dispatch",
                        self.dispatch_request(fixture, reason),
                    )
                self.assertEqual(model.snapshot(), before)
        for reason in self.AUTHORITY_REASONS:
            with self.subTest(kind="authority", reason=reason):
                core, model, fixture, _ = self.authority_model()
                before = model.snapshot()
                model.set_failure_observation(
                    fixture["context"]["context_nonce"], "different-observation"
                )
                with self.assertRaises(core.EvidenceError):
                    self.invoke(
                        model,
                        "fail-agent-report-authority",
                        self.authority_request(fixture, reason),
                    )
                after = model.snapshot()
                after["failure_observations"] = before["failure_observations"]
                self.assertEqual(after, before)


if __name__ == "__main__":
    unittest.main()
