import copy
import contextlib
import hashlib
import importlib
import inspect
import json
import pathlib
import re
import unittest


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


if __name__ == "__main__":
    unittest.main()
