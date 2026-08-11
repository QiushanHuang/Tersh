import hashlib
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


if __name__ == "__main__":
    unittest.main()
