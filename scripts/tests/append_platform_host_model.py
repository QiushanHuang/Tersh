"""Test-only reference Host seam for append-platform behavior.

This module models source resolution but is not a UID-0 Host ledger and grants
no production authority.
"""

from __future__ import annotations

import copy
import hashlib
from typing import Any

from scripts import evidence_core as core


_RECEIPT_KEYS = {
    "finding_id",
    "evidence_id",
    "evidence_attempt",
    "source_kind",
    "receipt_id",
    "body_sha256",
}
_SOURCE_KEYS = _RECEIPT_KEYS | {"body"}
_FINDING_KEYS = {
    "finding_id",
    "severity",
    "requirement",
    "file",
    "line",
    "counterexample",
    "required_correction",
}
_REVIEW_BODY_KEYS = {
    "schema",
    "evidence_id",
    "evidence_attempt",
    "verdict",
    "findings",
}
_AUDIT_FAILURE_BODY_KEYS = {
    "schema",
    "evidence_id",
    "evidence_attempt",
    "candidate",
    "audit_revision",
    "failure_class",
    "findings",
}
_SOURCE_KINDS = {"review", "audit-failure"}
_AUDIT_FAILURE_CLASSES = {
    "candidate-repair-required",
    "external-evidence-invalid",
    "trusted-audit-input-missing",
}


def _require_closed_object(
    value: Any,
    expected_keys: set[str],
    field: str,
) -> dict[str, Any]:
    if type(value) is not dict:
        raise core.EvidenceError(f"{field} must be an exact closed object")
    if any(type(key) is not str for key in value):
        raise core.EvidenceError(f"{field} contains a non-exact-string key")
    if set(value) != expected_keys:
        raise core.EvidenceError(f"{field} has the wrong closed key set")
    return value


def _require_exact_string(value: Any, field: str) -> str:
    if type(value) is not str or not value:
        raise core.EvidenceError(f"{field} must be a nonempty exact string")
    return value


def _require_literal(value: Any, literal: str, field: str) -> str:
    if type(value) is not str or value != literal:
        raise core.EvidenceError(f"{field} must be exactly {literal!r}")
    return value


def _require_enum(value: Any, allowed: set[str], field: str) -> str:
    if type(value) is not str or value not in allowed:
        raise core.EvidenceError(f"invalid {field}")
    return value


def _validate_finding(value: Any, evidence_id: str) -> dict[str, Any]:
    finding = _require_closed_object(value, _FINDING_KEYS, "finding")
    finding_id = _require_exact_string(finding["finding_id"], "finding ID")
    match = core.FINDING_ID_RE.fullmatch(finding_id)
    if match is None or match.group("evidence") != evidence_id:
        raise core.EvidenceError("finding ID does not match source evidence ID")
    _require_enum(finding["severity"], {"P0", "P1", "P2"}, "severity")
    for field in (
        "requirement",
        "file",
        "counterexample",
        "required_correction",
    ):
        _require_exact_string(finding[field], f"finding {field}")
    core.require_exact_int(finding["line"], "finding line", minimum=1)
    return dict(finding)


def _validate_findings(value: Any, evidence_id: str) -> list[dict[str, Any]]:
    if type(value) is not list or not value:
        raise core.EvidenceError("source findings must be a nonempty exact list")
    return [_validate_finding(finding, evidence_id) for finding in value]


def _validate_review_body(value: Any) -> dict[str, Any]:
    body = _require_closed_object(value, _REVIEW_BODY_KEYS, "review source body")
    _require_literal(
        body["schema"],
        "tersh-test-review-finding-source-v1",
        "review fixture schema",
    )
    evidence_id = core.validate_evidence_id(body["evidence_id"])
    evidence_attempt = core.validate_attempt(body["evidence_attempt"])
    _require_literal(body["verdict"], "FAIL", "review verdict")
    return {
        "schema": body["schema"],
        "evidence_id": evidence_id,
        "evidence_attempt": evidence_attempt,
        "verdict": body["verdict"],
        "findings": _validate_findings(body["findings"], evidence_id),
    }


def _validate_audit_failure_body(value: Any) -> dict[str, Any]:
    body = _require_closed_object(
        value,
        _AUDIT_FAILURE_BODY_KEYS,
        "audit-failure source body",
    )
    _require_literal(
        body["schema"],
        "tersh-test-audit-failure-finding-source-v1",
        "audit-failure fixture schema",
    )
    evidence_id = core.validate_evidence_id(body["evidence_id"])
    evidence_attempt = core.validate_attempt(body["evidence_attempt"])
    candidate = core.validate_candidate(body["candidate"])
    audit_revision = core.validate_attempt(body["audit_revision"])
    failure_class = _require_enum(
        body["failure_class"],
        _AUDIT_FAILURE_CLASSES,
        "audit failure class",
    )
    return {
        "schema": body["schema"],
        "evidence_id": evidence_id,
        "evidence_attempt": evidence_attempt,
        "candidate": candidate,
        "audit_revision": audit_revision,
        "failure_class": failure_class,
        "findings": _validate_findings(body["findings"], evidence_id),
    }


def _normalize_receipted_finding_source(
    receipt: Any,
    body: Any,
) -> dict[str, Any]:
    receipt_object = _require_closed_object(
        receipt,
        _RECEIPT_KEYS,
        "finding source receipt",
    )
    finding_id = _require_exact_string(
        receipt_object["finding_id"],
        "receipt finding ID",
    )
    finding_match = core.FINDING_ID_RE.fullmatch(finding_id)
    if finding_match is None:
        raise core.EvidenceError("invalid receipt finding ID")
    evidence_id = core.validate_evidence_id(receipt_object["evidence_id"])
    evidence_attempt = core.validate_attempt(receipt_object["evidence_attempt"])
    source_kind = _require_enum(
        receipt_object["source_kind"],
        _SOURCE_KINDS,
        "source kind",
    )
    receipt_id = core.validate_sha256(
        receipt_object["receipt_id"],
        "finding source receipt ID",
    )
    body_sha256 = core.validate_sha256(
        receipt_object["body_sha256"],
        "finding source body sha256",
    )

    if source_kind == "review":
        validated_body = _validate_review_body(body)
    else:
        validated_body = _validate_audit_failure_body(body)

    if finding_match.group("evidence") != evidence_id:
        raise core.EvidenceError("receipt finding ID does not match evidence ID")
    if validated_body["evidence_id"] != evidence_id:
        raise core.EvidenceError("source body evidence ID does not match receipt")
    if validated_body["evidence_attempt"] != evidence_attempt:
        raise core.EvidenceError("source body attempt does not match receipt")
    occurrence_count = sum(
        finding["finding_id"] == finding_id
        for finding in validated_body["findings"]
    )
    if occurrence_count != 1:
        raise core.EvidenceError(
            "source body must contain its receipt finding ID exactly once"
        )
    canonical_body = core.canonical_json_bytes(validated_body)
    if hashlib.sha256(canonical_body).hexdigest() != body_sha256:
        raise core.EvidenceError("source body canonical digest does not match receipt")

    return {
        "finding_id": finding_id,
        "evidence_id": evidence_id,
        "evidence_attempt": evidence_attempt,
        "source_kind": source_kind,
        "receipt_id": receipt_id,
        "body": copy.deepcopy(validated_body),
        "body_sha256": body_sha256,
    }


def _validate_normalized_source(value: Any) -> dict[str, Any]:
    source = _require_closed_object(value, _SOURCE_KEYS, "normalized finding source")
    receipt = {
        "finding_id": source["finding_id"],
        "evidence_id": source["evidence_id"],
        "evidence_attempt": source["evidence_attempt"],
        "source_kind": source["source_kind"],
        "receipt_id": source["receipt_id"],
        "body_sha256": source["body_sha256"],
    }
    return _normalize_receipted_finding_source(receipt, source["body"])


class AppendPlatformHostModel:
    """Test-only parent-source registry; not a production Host implementation."""

    def __init__(self) -> None:
        self._receipted_finding_sources: list[dict[str, Any]] = []

    def register_receipted_finding_source(self, receipt: Any, body: Any) -> None:
        source = _normalize_receipted_finding_source(receipt, body)
        self._receipted_finding_sources.append(source)

    def validate_context_parent_sources(self, context: Any) -> list[str]:
        validated_context = core.validate_dispatch_context_v2(context)
        source_index: dict[str, dict[str, Any]] = {}
        for registered_source in self._receipted_finding_sources:
            source = _validate_normalized_source(registered_source)
            finding_id = source["finding_id"]
            if finding_id in source_index:
                raise core.EvidenceError(
                    "finding ID is ambiguous across receipted source bodies"
                )
            source_index[finding_id] = source

        context_attempt = int(validated_context["evidence_attempt"])
        for parent_finding_id in validated_context["parent_finding_ids"]:
            source = source_index.get(parent_finding_id)
            if source is None:
                raise core.EvidenceError(
                    "parent finding ID has no unique receipted Host source"
                )
            if source["evidence_id"] != validated_context["evidence_id"]:
                raise core.EvidenceError(
                    "parent finding source does not match context evidence ID"
                )
            if int(source["evidence_attempt"]) >= context_attempt:
                raise core.EvidenceError(
                    "parent finding source must precede the context attempt"
                )

        return copy.deepcopy(validated_context["parent_finding_ids"])
