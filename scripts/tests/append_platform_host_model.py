"""Test-only reference Host seam for append-platform behavior.

This module models source resolution but is not a UID-0 Host ledger and grants
no production authority.
"""

from __future__ import annotations

import copy
import hashlib
import socket
import threading
import time
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


class FixtureClock:
    """Deterministic monotonic clock for Host lease tests."""

    def __init__(self, now: float = 0.0) -> None:
        self._now = float(now)

    def __call__(self) -> float:
        return self._now

    def advance(self, seconds: float) -> None:
        if type(seconds) not in (int, float) or seconds < 0:
            raise ValueError("clock advance must be nonnegative")
        self._now += float(seconds)


class AppendPlatformHostModel:
    """Deterministic test-only Host oracle; it grants no production authority."""

    ROOT_SUPERVISOR_PRINCIPAL = "uid-0-root-supervisor"
    _ATTEMPT_FIELDS = (
        "attempt_binding_id",
        "predecessor_attempt_binding_id",
        "evidence_id",
        "evidence_attempt",
        "run_binding",
        "candidate",
        "candidate_tree",
        "worktree_handle",
        "baseline_commit",
        "bundle_id",
        "runtime_profile_id",
        "policy_sha256",
        "policy_entry_id",
        "policy_entry_sha256",
        "projection_root_class",
        "record_class",
        "record_schema",
    )

    def __init__(self, *, clock: Any | None = None) -> None:
        self._clock = FixtureClock() if clock is None else clock
        self._ledger_lock = threading.RLock()
        self._receipted_finding_sources: list[dict[str, Any]] = []
        self._state: dict[str, Any] = {
            "attempts": {},
            "lineages": {},
            "handles": {},
            "sessions": {},
            "leases": {},
            "connections": {},
            "replays": {},
            "raw_commits": {},
            "worktrees": {},
            "predecessors": {},
            "records": [],
            "blobs": {},
            "receipts": [],
            "authorities": {},
            "next_id": 1,
            "next_connection_generation": 1,
        }

    @staticmethod
    def _new_id(state: dict[str, Any], label: str) -> str:
        sequence = state["next_id"]
        state["next_id"] += 1
        return hashlib.sha256(f"{label}:{sequence}".encode("ascii")).hexdigest()

    @staticmethod
    def _body_sha256(body: Any) -> str:
        return hashlib.sha256(core.canonical_json_bytes(body)).hexdigest()

    def snapshot(self) -> dict[str, Any]:
        snapshot = copy.deepcopy(self._state)
        snapshot["receipted_finding_sources"] = copy.deepcopy(
            self._receipted_finding_sources
        )
        return snapshot

    def register_raw_commit(
        self,
        commit: str,
        *,
        tree: str,
        parents: tuple[str, ...] | list[str],
        view: str = "raw",
    ) -> None:
        core.validate_candidate(commit)
        core.validate_candidate(tree)
        if type(parents) not in (tuple, list):
            raise core.EvidenceError("raw commit parents must be an exact sequence")
        row = {
            "tree": tree,
            "parents": [core.validate_candidate(parent) for parent in parents],
            "view": view,
        }
        current = self._state["raw_commits"].get(commit)
        if current is not None and current != row:
            raise core.EvidenceError("raw commit fact is immutable")
        self._state["raw_commits"][commit] = copy.deepcopy(row)

    def register_worktree_observation(
        self,
        worktree_handle: str,
        *,
        head: str | None,
        tree: str,
        clean: bool,
        view: str,
    ) -> None:
        if head is not None:
            core.validate_candidate(head)
        core.validate_candidate(tree)
        row = {"head": head, "tree": tree, "clean": clean, "view": view}
        current = self._state["worktrees"].get(worktree_handle)
        if current is not None and current != row:
            raise core.EvidenceError("worktree observation is immutable")
        self._state["worktrees"][worktree_handle] = copy.deepcopy(row)

    def register_attempt_predecessor(
        self,
        attempt_binding_id: str,
        *,
        candidate: str,
    ) -> None:
        core.validate_sha256(attempt_binding_id, "attempt binding ID")
        core.validate_candidate(candidate)
        current = self._state["predecessors"].get(attempt_binding_id)
        if current is not None and current != candidate:
            raise core.EvidenceError("attempt predecessor fact is immutable")
        self._state["predecessors"][attempt_binding_id] = candidate

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

    def _is_raw_ancestor(self, ancestor: str, candidate: str) -> bool:
        if ancestor == candidate:
            return True
        pending = [candidate]
        visited: set[str] = set()
        while pending:
            current = pending.pop()
            if current in visited:
                continue
            visited.add(current)
            row = self._state["raw_commits"].get(current)
            if row is None or row["view"] != "raw":
                return False
            for parent in row["parents"]:
                if parent == ancestor:
                    ancestor_row = self._state["raw_commits"].get(parent)
                    return ancestor_row is not None and ancestor_row["view"] == "raw"
                pending.append(parent)
        return False

    def _validate_candidate_facts(
        self,
        context_value: Any,
        session_value: Any,
    ) -> tuple[dict[str, Any], dict[str, Any]]:
        context = core.validate_dispatch_context_v2(context_value)
        if type(session_value) is not dict:
            raise core.EvidenceError("recorder session must be an exact object")
        session = copy.deepcopy(session_value)
        for field in self._ATTEMPT_FIELDS:
            if field not in session:
                raise core.EvidenceError(f"recorder session is missing {field}")
        fixed = {
            "entrypoint": "record-orchestration",
            "producer_mode": "harness",
            "operation": "append-platform",
            "projection_root_class": "local",
            "record_class": "orchestration",
            "record_schema": "tersh-evidence-orchestration-v1",
        }
        if any(session.get(field) != value for field, value in fixed.items()):
            raise core.EvidenceError("recorder session mode is not append-platform")
        joins = {
            "context_nonce": "context_nonce",
            "evidence_id": "evidence_id",
            "evidence_attempt": "evidence_attempt",
            "run_binding": "run_binding",
            "worktree_handle": "worktree_handle",
            "baseline_commit": "baseline_commit",
            "bundle_id": "harness_bundle_sha256",
        }
        for session_field, context_field in joins.items():
            if session.get(session_field) != context[context_field]:
                raise core.EvidenceError(
                    f"recorder session {session_field} does not match context"
                )

        candidate = core.validate_candidate(session["candidate"])
        baseline = core.validate_candidate(session["baseline_commit"])
        candidate_tree = core.validate_candidate(session["candidate_tree"])
        candidate_row = self._state["raw_commits"].get(candidate)
        if candidate_row is None:
            raise core.EvidenceError("candidate raw object is missing")
        if candidate_row["view"] != "raw":
            raise core.EvidenceError("candidate ancestry requires the raw object view")
        observation = self._state["worktrees"].get(session["worktree_handle"])
        if observation is None:
            raise core.EvidenceError("worktree observation is missing")
        if observation["view"] != "raw":
            raise core.EvidenceError("worktree observation uses an alternate view")
        if observation["head"] is None:
            raise core.EvidenceError("worktree HEAD is unborn")
        if not observation["clean"]:
            raise core.EvidenceError("worktree observation is dirty")
        if observation["head"] != candidate:
            raise core.EvidenceError("BODY candidate drifted from observed HEAD")
        if observation["tree"] != candidate_tree or candidate_row["tree"] != candidate_tree:
            raise core.EvidenceError("candidate tree does not match Host observation")

        relation = "equal" if candidate == baseline else "descendant"
        if session.get("candidate_relation") != relation:
            raise core.EvidenceError("candidate relation tag does not match raw ancestry")
        if relation == "descendant" and not self._is_raw_ancestor(baseline, candidate):
            raise core.EvidenceError("candidate is unrelated in raw ancestry")

        wave = context["wave"]
        if wave == "wave-a":
            if not (
                baseline == context["review_target"] == candidate
            ):
                raise core.EvidenceError("Wave A baseline drift is forbidden")
        elif wave == "wave-b":
            if context["role"] != "implementation":
                raise core.EvidenceError("Wave B requires the implementation role")
            if context["review_target"] != baseline:
                raise core.EvidenceError("Wave B review target must equal baseline")
            predecessor_id = session.get("predecessor_attempt_binding_id")
            predecessor_candidate = self._state["predecessors"].get(predecessor_id)
            if predecessor_candidate != baseline:
                raise core.EvidenceError(
                    "Wave B baseline must equal the immediate predecessor candidate"
                )
        elif context["review_target"] != candidate:
            raise core.EvidenceError("review target must equal the bound candidate")
        return context, session

    def open_attempt(self, request: Any, *, fault_at: Any = None) -> dict[str, Any]:
        if type(request) is not dict or set(request) != {"context", "session"}:
            raise core.EvidenceError("open-attempt request must be exact")
        context, session = self._validate_candidate_facts(
            request["context"], request["session"]
        )
        binding = {field: copy.deepcopy(session[field]) for field in self._ATTEMPT_FIELDS}
        binding_id = binding["attempt_binding_id"]
        current = self._state["attempts"].get(binding_id)
        if current is not None and current["binding"] != binding:
            raise core.EvidenceError("attempt binding is immutable")
        session_id = session.get("producer_session_id")
        core.validate_sha256(session_id, "producer session ID")
        existing_session = self._state["sessions"].get(session_id)
        if existing_session is not None:
            if existing_session["body"] != session:
                raise core.EvidenceError("recorder session is immutable")
            return copy.deepcopy(current)

        shadow = copy.deepcopy(self._state)
        if current is None:
            shadow["attempts"][binding_id] = {
                **copy.deepcopy(binding),
                "binding": copy.deepcopy(binding),
                "state": "ACTIVE",
                "registered_session_ids": [],
                "marker_receipt_chain": {
                    "next_receipt_sequence": session["next_receipt_sequence"],
                    "previous_receipt_id": session["previous_receipt_id"],
                },
            }
        shadow["attempts"][binding_id]["registered_session_ids"].append(session_id)
        shadow["sessions"][session_id] = {
            "body": copy.deepcopy(session),
            "live": True,
            "lineage_id": None,
            "active_lease_id": None,
        }
        matching_lineages = [
            row
            for row in shadow["lineages"].values()
            if row["attempt_binding_id"] == binding_id
            and row["context_nonce"] == context["context_nonce"]
        ]
        if len(matching_lineages) == 1:
            lineage = matching_lineages[0]
            shadow["sessions"][session_id]["lineage_id"] = lineage["lineage_id"]
            lineage["registered_session_ids"].append(session_id)
        self._state = shadow
        return copy.deepcopy(self._state["attempts"][binding_id])

    def force_receipt_chain_head(
        self,
        attempt_binding_id: str,
        *,
        next_receipt_sequence: int,
        previous_receipt_id: str | None,
    ) -> None:
        """Test-only injection of a concurrently advanced Host receipt chain."""

        core.validate_sha256(attempt_binding_id, "attempt binding ID")
        core.require_exact_int(
            next_receipt_sequence,
            "next receipt sequence",
            minimum=1,
        )
        if next_receipt_sequence == 1:
            if previous_receipt_id is not None:
                raise core.EvidenceError("sequence one requires null predecessor")
        else:
            core.validate_sha256(previous_receipt_id, "previous receipt ID")
        with self._ledger_lock:
            attempt = self._state["attempts"].get(attempt_binding_id)
            if attempt is None:
                raise core.EvidenceError("attempt binding is unknown")
            shadow = copy.deepcopy(self._state)
            shadow["attempts"][attempt_binding_id]["marker_receipt_chain"] = {
                "next_receipt_sequence": next_receipt_sequence,
                "previous_receipt_id": previous_receipt_id,
            }
            self._state = shadow

    @staticmethod
    def _lineage_result(lineage: dict[str, Any]) -> dict[str, Any]:
        return {
            "lineage_id": lineage["lineage_id"],
            "state": lineage["state"],
            "transition_index": lineage["transition_index"],
            "recovery_generation": lineage["recovery_generation"],
            "context_handle": lineage["context_handle"],
            "invocation_handle": lineage["invocation_handle"],
            "response_handle": lineage["response_handle"],
        }

    def seed_lineage(self, *, context: Any) -> dict[str, Any]:
        validated = core.validate_dispatch_context_v2(context)
        matches = [
            (session_id, row)
            for session_id, row in self._state["sessions"].items()
            if row["body"]["context_nonce"] == validated["context_nonce"]
            and row["body"]["evidence_id"] == validated["evidence_id"]
            and row["body"]["evidence_attempt"] == validated["evidence_attempt"]
        ]
        if len(matches) != 1 or matches[0][1]["lineage_id"] is not None:
            raise core.EvidenceError("context does not select one unseeded attempt")
        session_id, session_row = matches[0]
        shadow = copy.deepcopy(self._state)
        lineage_id = self._new_id(shadow, "lineage")
        context_handle = self._new_id(shadow, "context-h0")
        handle_row = {
            "lineage_id": lineage_id,
            "kind": "context",
            "transition_index": 0,
            "body": copy.deepcopy(validated),
            "body_sha256": self._body_sha256(validated),
            "live": True,
        }
        lineage = {
            "lineage_id": lineage_id,
            "context_nonce": validated["context_nonce"],
            "state": "CREATED",
            "transition_index": 0,
            "recovery_generation": 0,
            "attempt_binding_id": session_row["body"]["attempt_binding_id"],
            "registered_session_ids": [session_id],
            "terminal_state": None,
            "mode": "platform-envelope",
            "context_handle": context_handle,
            "invocation_handle": None,
            "response_handle": None,
        }
        shadow["handles"][context_handle] = handle_row
        shadow["lineages"][lineage_id] = lineage
        shadow["sessions"][session_id]["lineage_id"] = lineage_id
        self._state = shadow
        return self._lineage_result(lineage)

    def _current_handle(
        self,
        handle_id: Any,
        *,
        kind: str,
    ) -> tuple[dict[str, Any], dict[str, Any]]:
        if type(handle_id) is not str:
            raise core.EvidenceError("handle must be an exact string")
        handle = self._state["handles"].get(handle_id)
        if handle is None or not handle["live"] or handle["kind"] != kind:
            raise core.EvidenceError("handle generation is stale or invalid")
        lineage = self._state["lineages"][handle["lineage_id"]]
        return handle, lineage

    def capture_invocation(
        self,
        context_handle: Any,
        invocation: Any,
        *,
        lose_reply: bool = False,
    ) -> dict[str, Any]:
        _, lineage = self._current_handle(context_handle, kind="context")
        if lineage["state"] != "CREATED" or lineage["context_handle"] != context_handle:
            raise core.EvidenceError("lineage is not in CREATED")
        body = copy.deepcopy(core._validate_invocation_body(invocation))
        context_body = self._state["handles"][context_handle]["body"]
        for field in ("context_nonce", "harness_bundle_revision", "harness_bundle_sha256"):
            if body[field] != context_body[field]:
                raise core.EvidenceError("invocation does not join context lineage")
        shadow = copy.deepcopy(self._state)
        shadow["handles"][context_handle]["live"] = False
        next_context = self._new_id(shadow, "context-h1")
        invocation_handle = self._new_id(shadow, "invocation-hi")
        shadow["handles"][next_context] = {
            "lineage_id": lineage["lineage_id"],
            "kind": "context",
            "transition_index": 1,
            "body": copy.deepcopy(context_body),
            "body_sha256": self._body_sha256(context_body),
            "live": True,
        }
        shadow["handles"][invocation_handle] = {
            "lineage_id": lineage["lineage_id"],
            "kind": "invocation",
            "transition_index": 1,
            "body": body,
            "body_sha256": self._body_sha256(body),
            "live": True,
        }
        updated = shadow["lineages"][lineage["lineage_id"]]
        updated.update(
            {
                "state": "INVOKED",
                "transition_index": 1,
                "context_handle": next_context,
                "invocation_handle": invocation_handle,
            }
        )
        self._state = shadow
        return self._lineage_result(updated)

    def capture_response(
        self,
        context_handle: Any,
        response: Any,
        *,
        lose_reply: bool = False,
    ) -> dict[str, Any]:
        _, lineage = self._current_handle(context_handle, kind="context")
        if lineage["state"] != "INVOKED" or lineage["context_handle"] != context_handle:
            raise core.EvidenceError("lineage is not in INVOKED")
        invocation_handle = lineage["invocation_handle"]
        invocation = self._state["handles"][invocation_handle]["body"]
        body = copy.deepcopy(core._validate_response_body(response))
        if body["context_nonce"] != lineage["context_nonce"]:
            raise core.EvidenceError("response does not join context lineage")
        if body["dispatch_id"] != invocation["dispatch_id"]:
            raise core.EvidenceError("response does not join invocation lineage")
        shadow = copy.deepcopy(self._state)
        shadow["handles"][context_handle]["live"] = False
        next_context = self._new_id(shadow, "context-h2")
        response_handle = self._new_id(shadow, "response-hr")
        context_body = self._state["handles"][context_handle]["body"]
        shadow["handles"][next_context] = {
            "lineage_id": lineage["lineage_id"],
            "kind": "context",
            "transition_index": 2,
            "body": copy.deepcopy(context_body),
            "body_sha256": self._body_sha256(context_body),
            "live": True,
        }
        shadow["handles"][response_handle] = {
            "lineage_id": lineage["lineage_id"],
            "kind": "response",
            "transition_index": 2,
            "body": body,
            "body_sha256": self._body_sha256(body),
            "live": True,
        }
        updated = shadow["lineages"][lineage["lineage_id"]]
        updated.update(
            {
                "state": "RESPONDED_PLATFORM",
                "transition_index": 2,
                "context_handle": next_context,
                "response_handle": response_handle,
            }
        )
        self._state = shadow
        return self._lineage_result(updated)

    @staticmethod
    def _connection_identity(connection: Any) -> tuple[int, int, Any]:
        try:
            numeric_fd = connection.fileno()
        except (AttributeError, OSError, ValueError) as error:
            raise core.EvidenceError("connection has no live numeric FD") from error
        if type(numeric_fd) is not int or numeric_fd < 0:
            raise core.EvidenceError("connection has no live numeric FD")
        kernel_identity = getattr(connection, "kernel_identity", None)
        if callable(kernel_identity):
            kernel_identity = kernel_identity()
        if kernel_identity is None:
            try:
                kernel_identity = (
                    connection.getsockname(),
                    connection.getpeername(),
                )
            except (AttributeError, OSError):
                kernel_identity = ("opaque-socket", numeric_fd)
        return id(connection), numeric_fd, copy.deepcopy(kernel_identity)

    def bind_append_connection(
        self,
        handles: Any,
        *,
        connection: Any,
    ) -> dict[str, Any]:
        if getattr(connection, "peer_principal", None) != self.ROOT_SUPERVISOR_PRINCIPAL:
            raise core.EvidenceError("root supervisor peer authentication failed")
        connection_session_id = getattr(connection, "producer_session_id", None)
        supplied_session_id = (
            handles.get("producer_session_id") if type(handles) is dict else None
        )
        session_row = self._state["sessions"].get(connection_session_id)
        if (
            type(connection_session_id) is not str
            or connection_session_id != supplied_session_id
            or session_row is None
            or not session_row["live"]
        ):
            raise core.EvidenceError("recorder session authentication failed")
        expected_keys = {
            "producer_session_id",
            "context_handle",
            "invocation_handle",
            "response_handle",
            "mode",
        }
        if set(handles) != expected_keys:
            raise core.EvidenceError("append handle tuple must be exact")
        if handles["mode"] != "platform-envelope":
            raise core.EvidenceError("append handle tuple mixes producer modes")
        handle_ids = (
            handles["context_handle"],
            handles["invocation_handle"],
            handles["response_handle"],
        )
        if len(set(handle_ids)) != 3:
            raise core.EvidenceError("append handles must not alias")
        context_row, context_lineage = self._current_handle(
            handle_ids[0], kind="context"
        )
        invocation_row, invocation_lineage = self._current_handle(
            handle_ids[1], kind="invocation"
        )
        response_row, response_lineage = self._current_handle(
            handle_ids[2], kind="response"
        )
        lineage_ids = {
            context_row["lineage_id"],
            invocation_row["lineage_id"],
            response_row["lineage_id"],
        }
        if len(lineage_ids) != 1:
            raise core.EvidenceError("append handles cross lineages")
        lineage = context_lineage
        if invocation_lineage is not lineage or response_lineage is not lineage:
            lineage = self._state["lineages"][context_row["lineage_id"]]
        if (
            lineage["state"] != "RESPONDED_PLATFORM"
            or lineage["context_handle"] != handle_ids[0]
            or lineage["invocation_handle"] != handle_ids[1]
            or lineage["response_handle"] != handle_ids[2]
            or session_row["lineage_id"] != lineage["lineage_id"]
            or session_row["body"]["attempt_binding_id"]
            != lineage["attempt_binding_id"]
        ):
            raise core.EvidenceError("append handles are not the current responded tuple")
        active_lease_id = session_row["active_lease_id"]
        if active_lease_id is not None:
            active = self._state["leases"].get(active_lease_id)
            if active is not None and active["valid"]:
                raise core.EvidenceError("recorder session already has a launch lease")

        object_id, numeric_fd, kernel_identity = self._connection_identity(connection)
        shadow = copy.deepcopy(self._state)
        generation = shadow["next_connection_generation"]
        shadow["next_connection_generation"] += 1
        lease_id = self._new_id(shadow, "append-lease")
        shadow["connections"][str(object_id)] = {
            "numeric_fd": numeric_fd,
            "kernel_identity": kernel_identity,
            "generation": generation,
        }
        lease = {
            "lease_id": lease_id,
            "producer_session_id": connection_session_id,
            "lineage_id": lineage["lineage_id"],
            "connection_object_id": object_id,
            "numeric_fd": numeric_fd,
            "kernel_identity": kernel_identity,
            "connection_generation": generation,
            "launch_deadline": self._clock() + 5.0,
            "transaction_nonce": None,
            "transaction_deadline": None,
            "handles": copy.deepcopy(handles),
            "valid": True,
            "invalid_reason": None,
        }
        shadow["leases"][lease_id] = lease
        shadow["sessions"][connection_session_id]["active_lease_id"] = lease_id
        self._state = shadow
        return copy.deepcopy(lease)

    def _invalidate_lease(self, lease_id: str, reason: str) -> None:
        shadow = copy.deepcopy(self._state)
        lease = shadow["leases"][lease_id]
        lease["valid"] = False
        lease["invalid_reason"] = reason
        session = shadow["sessions"].get(lease["producer_session_id"])
        if session is not None and session["active_lease_id"] == lease_id:
            session["active_lease_id"] = None
        self._state = shadow

    def build_host_orchestration_record(
        self,
        context: Any,
        invocation: Any,
        response: Any,
        recorder_session: Any,
    ) -> dict[str, Any]:
        provenance_input = {
            "mode": "platform-envelope",
            **{
                kind: {
                    "body": copy.deepcopy(body),
                    "sha256": self._body_sha256(body),
                }
                for kind, body in (
                    ("context", context),
                    ("invocation", invocation),
                    ("response", response),
                )
            },
        }
        provenance = core.validate_platform_envelope_provenance(provenance_input)
        context_body = provenance["context"]["body"]
        invocation_body = provenance["invocation"]["body"]
        response_body = provenance["response"]["body"]
        session = core.validate_orchestration_recorder_session(
            recorder_session,
            context=context_body,
            invocation=invocation_body,
            response=response_body,
        )
        return {
            "schema": "tersh-evidence-orchestration-v1",
            "evidence_id": context_body["evidence_id"],
            "evidence_attempt": context_body["evidence_attempt"],
            "run_binding": context_body["run_binding"],
            "role": context_body["role"],
            "wave": context_body["wave"],
            "review_attempt": context_body["review_attempt"],
            "baseline_commit": context_body["baseline_commit"],
            "reviewed_commit": session["candidate"],
            "parent_finding_ids": copy.deepcopy(
                context_body["parent_finding_ids"]
            ),
            "dispatch_id": invocation_body["dispatch_id"],
            "agent_id": response_body["agent_id"],
            "canonical_task_path": context_body["canonical_task_path"],
            "agent_run_id": response_body["agent_run_id"],
            "model": invocation_body["selected_model"],
            "reasoning_effort": invocation_body["selected_reasoning_effort"],
            "dispatched_at": invocation_body["dispatched_at"],
            "started_at": response_body["started_at"],
            "ended_at": response_body["ended_at"],
            "terminal_status": response_body["terminal_status"],
            "provenance": provenance,
        }

    def _bound_append_lease_before_handles(
        self,
        lease_value: Any,
        connection: Any,
        *,
        check_launch_deadline: bool = True,
    ) -> tuple[dict[str, Any], dict[str, Any]]:
        if getattr(connection, "peer_principal", None) != self.ROOT_SUPERVISOR_PRINCIPAL:
            raise core.EvidenceError("root supervisor peer authentication failed")
        lease_id = lease_value.get("lease_id") if type(lease_value) is dict else None
        lease = self._state["leases"].get(lease_id)
        if lease is None:
            raise core.EvidenceError("recorder session lease is unknown")
        if not lease["valid"]:
            if lease["invalid_reason"] == "consumed":
                raise core.EvidenceError("recorder session replay rejected")
            raise core.EvidenceError("recorder session lease is permanently invalid")
        session = self._state["sessions"].get(lease["producer_session_id"])
        if (
            session is None
            or not session["live"]
            or session["active_lease_id"] != lease_id
            or getattr(connection, "producer_session_id", None)
            != lease["producer_session_id"]
        ):
            self._invalidate_lease(lease_id, "session")
            raise core.EvidenceError("recorder session authentication failed")
        object_id, numeric_fd, kernel_identity = self._connection_identity(connection)
        connection_row = self._state["connections"].get(str(object_id))
        if (
            object_id != lease["connection_object_id"]
            or numeric_fd != lease["numeric_fd"]
            or kernel_identity != lease["kernel_identity"]
            or connection_row is None
            or connection_row["generation"] != lease["connection_generation"]
        ):
            self._invalidate_lease(lease_id, "connection-generation")
            raise core.EvidenceError("connection generation changed before handle lookup")
        if check_launch_deadline and self._clock() >= lease["launch_deadline"]:
            self._invalidate_lease(lease_id, "launch-expired")
            raise core.EvidenceError("launch lease expired before handle lookup")
        return lease, session

    def _append_body_tuple(
        self,
        lease: dict[str, Any],
        session: dict[str, Any],
    ) -> tuple[
        dict[str, Any],
        dict[str, Any],
        dict[str, Any],
        dict[str, Any],
        dict[str, Any],
    ]:
        handles = lease["handles"]
        context_row, lineage = self._current_handle(
            handles["context_handle"], kind="context"
        )
        invocation_row, invocation_lineage = self._current_handle(
            handles["invocation_handle"], kind="invocation"
        )
        response_row, response_lineage = self._current_handle(
            handles["response_handle"], kind="response"
        )
        if (
            lineage["state"] != "RESPONDED_PLATFORM"
            or lineage["context_handle"] != handles["context_handle"]
            or lineage["invocation_handle"] != handles["invocation_handle"]
            or lineage["response_handle"] != handles["response_handle"]
            or {
                context_row["lineage_id"],
                invocation_row["lineage_id"],
                response_row["lineage_id"],
            }
            != {lineage["lineage_id"]}
            or invocation_lineage["lineage_id"] != lineage["lineage_id"]
            or response_lineage["lineage_id"] != lineage["lineage_id"]
        ):
            raise core.EvidenceError("append handle tuple is no longer current")
        context = copy.deepcopy(context_row["body"])
        invocation = copy.deepcopy(invocation_row["body"])
        response = copy.deepcopy(response_row["body"])
        session_body = copy.deepcopy(session["body"])
        self._validate_candidate_facts(context, session_body)
        record = self.build_host_orchestration_record(
            context,
            invocation,
            response,
            session_body,
        )
        return context, invocation, response, session_body, record

    def _linearize_append_happy_path(
        self,
        lease: dict[str, Any],
        transaction_nonce: str,
        bodies: tuple[
            dict[str, Any],
            dict[str, Any],
            dict[str, Any],
            dict[str, Any],
            dict[str, Any],
        ],
        frozen_record: bytes,
    ) -> dict[str, Any]:
        context, invocation, response, session_body, record = bodies
        lease_id = lease["lease_id"]
        session_id = lease["producer_session_id"]
        handles = lease["handles"]
        attempt_binding_id = session_body["attempt_binding_id"]
        attempt = self._state["attempts"].get(attempt_binding_id)
        if attempt is None or attempt["state"] != "ACTIVE":
            raise core.EvidenceError("append attempt is not active")
        expected_chain = {
            "next_receipt_sequence": session_body["next_receipt_sequence"],
            "previous_receipt_id": session_body["previous_receipt_id"],
        }
        if attempt["marker_receipt_chain"] != expected_chain:
            raise core.EvidenceError("append receipt chain head drifted")
        if self._clock() >= lease["transaction_deadline"]:
            raise core.EvidenceError("append transaction deadline expired")

        record_sha256 = hashlib.sha256(frozen_record).hexdigest()
        if record_sha256 in self._state["blobs"]:
            raise core.EvidenceError("append record blob already exists")
        shadow = copy.deepcopy(self._state)
        receipt_id = self._new_id(shadow, "producer-receipt")
        receipt = {
            "schema": "tersh-host-producer-receipt-v1",
            "receipt_id": receipt_id,
            "attempt_binding_id": attempt_binding_id,
            "producer_session_id": session_body["producer_session_id"],
            "sequence": session_body["next_receipt_sequence"],
            "previous_receipt_id": session_body["previous_receipt_id"],
            "producer_mode": session_body["producer_mode"],
            "entrypoint": session_body["entrypoint"],
            "bundle_id": session_body["bundle_id"],
            "runtime_profile_id": session_body["runtime_profile_id"],
            "policy_entry_id": session_body["policy_entry_id"],
            "policy_entry_sha256": session_body["policy_entry_sha256"],
            "environment_capability": None,
            "projection_root_class": session_body["projection_root_class"],
            "record_class": session_body["record_class"],
            "record_schema": session_body["record_schema"],
            "destination": session_body["destination"],
            "body_sha256": record_sha256,
            "byte_count": len(frozen_record),
            "dispatch_id": None,
            "reported_record_sha256": None,
            "created_at": response["ended_at"],
        }
        core._validate_append_platform_producer_receipt(
            receipt,
            recorder_session=session_body,
            record=record,
        )

        shadow_lease = shadow["leases"][lease_id]
        shadow_lease["valid"] = False
        shadow_lease["invalid_reason"] = "consumed"
        for handle_id in (
            handles["context_handle"],
            handles["invocation_handle"],
            handles["response_handle"],
        ):
            shadow["handles"][handle_id]["live"] = False
        lineage = shadow["lineages"][lease["lineage_id"]]
        lineage["state"] = "APPENDED_PLATFORM"
        lineage["terminal_state"] = (
            "appended-no-report"
            if response["reported_record_sha256"] is None
            else None
        )
        shadow_session = shadow["sessions"][session_id]
        shadow_session["live"] = False
        shadow_session["active_lease_id"] = None
        shadow["blobs"][record_sha256] = bytes(frozen_record)
        shadow["receipts"].append(copy.deepcopy(receipt))
        shadow["records"].append(
            {
                "attempt_binding_id": attempt_binding_id,
                "lineage_id": lease["lineage_id"],
                "body": copy.deepcopy(record),
                "body_bytes": bytes(frozen_record),
                "body_sha256": record_sha256,
                "receipt_id": receipt_id,
            }
        )
        shadow_attempt = shadow["attempts"][attempt_binding_id]
        shadow_attempt["marker_receipt_chain"] = {
            "next_receipt_sequence": receipt["sequence"] + 1,
            "previous_receipt_id": receipt_id,
        }
        if response["reported_record_sha256"] is not None:
            authority = {
                "schema": "tersh-host-agent-report-authority-v1",
                "dispatch_id": invocation["dispatch_id"],
                "attempt_binding_id": attempt_binding_id,
                "agent_id": response["agent_id"],
                "canonical_task_path": context["canonical_task_path"],
                "agent_run_id": response["agent_run_id"],
                "context_body_sha256": self._body_sha256(context),
                "invocation_body_sha256": self._body_sha256(invocation),
                "response_body_sha256": self._body_sha256(response),
                "reported_result_commit": response["reported_result_commit"],
                "reported_record_sha256": response["reported_record_sha256"],
                "orchestration_receipt_id": receipt_id,
                "draft_path": (
                    f"private-agent-reports/{invocation['dispatch_id']}.json"
                ),
                "review_destination": (
                    f"attempt-{context['evidence_attempt']}/"
                    f"candidate-{session_body['candidate']}/review/"
                    f"{context['role']}.{context['wave']}."
                    f"{context['review_attempt']}.json"
                ),
            }
            shadow["authorities"][invocation["dispatch_id"]] = authority
        replay_key = (
            f"{session_id}:{lease['connection_generation']}:"
            f"{transaction_nonce}"
        )
        shadow["replays"][replay_key] = {
            "lease_id": lease_id,
            "lineage_id": lease["lineage_id"],
            "receipt_id": receipt_id,
        }
        self._state = shadow
        return copy.deepcopy(receipt)

    def serve_append_platform(
        self,
        sock: Any,
        lease_value: Any,
        *,
        fault_at: Any = None,
    ) -> None:
        """Serve the Task-7 non-faulted append-platform wire transcript."""

        if fault_at is not None:
            raise core.EvidenceError("Task 7 Host wire does not implement fault hooks")
        lease, session = self._bound_append_lease_before_handles(
            lease_value,
            sock,
        )
        lease_id = lease["lease_id"]
        launch_wire_deadline = time.monotonic() + max(
            0.0,
            lease["launch_deadline"] - self._clock(),
        )
        committed = False
        try:
            begin = _require_closed_object(
                core.recv_host_frame(sock, launch_wire_deadline),
                {
                    "schema",
                    "transaction_nonce",
                    "operation",
                    "context_handle",
                    "invocation_handle",
                    "response_handle",
                },
                "append-platform BEGIN",
            )
            _require_literal(
                begin["schema"],
                "tersh-host-transaction-begin-v1",
                "append-platform BEGIN schema",
            )
            _require_literal(
                begin["operation"],
                "append-platform",
                "append-platform BEGIN operation",
            )
            transaction_nonce = core.validate_sha256(
                begin["transaction_nonce"],
                "append-platform transaction nonce",
            )
            wire_deadline = time.monotonic() + core.HOST_TRANSACTION_TIMEOUT_SECONDS

            shadow = copy.deepcopy(self._state)
            shadow["leases"][lease_id]["transaction_nonce"] = transaction_nonce
            shadow["leases"][lease_id]["transaction_deadline"] = self._clock() + 5.0
            self._state = shadow
            lease = self._state["leases"][lease_id]
            for field in (
                "context_handle",
                "invocation_handle",
                "response_handle",
            ):
                if begin[field] != lease["handles"][field]:
                    raise core.EvidenceError(
                        "append-platform BEGIN handle does not match bound lease"
                    )

            bodies = self._append_body_tuple(lease, session)
            body_kinds = (
                "context",
                "invocation",
                "response",
                "recorder-session",
                "orchestration-record",
            )
            body_hashes = [self._body_sha256(body) for body in bodies]
            frozen_record = core.canonical_json_bytes(bodies[4])
            if not 1 <= len(frozen_record) <= core.ORCHESTRATION_RECORD_MAX_BYTES:
                raise core.EvidenceError("Host record is outside 1..61440 bytes")
            for ordinal, (body_kind, body, digest) in enumerate(
                zip(body_kinds, bodies, body_hashes),
                start=1,
            ):
                core.send_host_frame(
                    sock,
                    {
                        "schema": "tersh-host-transaction-body-v1",
                        "transaction_nonce": transaction_nonce,
                        "operation": "append-platform",
                        "body_kind": body_kind,
                        "ordinal": ordinal,
                        "total": 5,
                        "body": body,
                        "body_sha256": digest,
                    },
                    wire_deadline,
                )
            core.send_host_frame(
                sock,
                {
                    "schema": "tersh-host-transaction-body-end-v1",
                    "transaction_nonce": transaction_nonce,
                    "operation": "append-platform",
                    "total": 5,
                    "body_sha256s": body_hashes,
                },
                wire_deadline,
            )

            expected_commit = {
                "schema": "tersh-host-transaction-commit-v1",
                "transaction_nonce": transaction_nonce,
                "operation": "append-platform",
                "body_sha256s": body_hashes,
                "record_facts": {
                    "evidence_id": bodies[0]["evidence_id"],
                    "evidence_attempt": bodies[0]["evidence_attempt"],
                    "run_binding": bodies[0]["run_binding"],
                    "candidate": bodies[3]["candidate"],
                    "destination": bodies[3]["destination"],
                    "record_sha256": body_hashes[4],
                },
            }
            commit = core.recv_host_frame(sock, wire_deadline)
            if commit != expected_commit:
                raise core.EvidenceError("append-platform COMMIT is not exact")
            expected_request_end = {
                "schema": "tersh-host-transaction-request-end-v1",
                "transaction_nonce": transaction_nonce,
                "operation": "append-platform",
                "commit_sha256": hashlib.sha256(
                    core.canonical_json_bytes(commit)
                ).hexdigest(),
            }
            request_end = core.recv_host_frame(sock, wire_deadline)
            if request_end != expected_request_end:
                raise core.EvidenceError("append-platform REQUEST-END is not exact")
            core.require_host_eof(sock, wire_deadline)

            lease, current_session = self._bound_append_lease_before_handles(
                lease_value,
                sock,
                check_launch_deadline=False,
            )
            current_bodies = self._append_body_tuple(lease, current_session)
            if current_bodies != bodies:
                raise core.EvidenceError("append-platform BODY facts drifted before commit")
            if core.canonical_json_bytes(current_bodies[4]) != frozen_record:
                raise core.EvidenceError("append-platform frozen record bytes drifted")
            receipt = self._linearize_append_happy_path(
                lease,
                transaction_nonce,
                bodies,
                frozen_record,
            )
            committed = True
            reply = {
                "schema": "tersh-host-transaction-reply-v1",
                "transaction_nonce": transaction_nonce,
                "operation": "append-platform",
                "body_sha256s": body_hashes,
                "result": {
                    "schema": "tersh-host-record-result-v1",
                    "receipt": receipt,
                },
            }
            core.send_host_frame(sock, reply, wire_deadline)
            core.send_host_frame(
                sock,
                {
                    "schema": "tersh-host-transaction-reply-end-v1",
                    "transaction_nonce": transaction_nonce,
                    "operation": "append-platform",
                    "reply_sha256": hashlib.sha256(
                        core.canonical_json_bytes(reply)
                    ).hexdigest(),
                },
                wire_deadline,
            )
            try:
                sock.shutdown(socket.SHUT_WR)
            except OSError as error:
                raise core.EvidenceError(
                    f"Host append reply half-close failed: {error}"
                ) from error
        except BaseException:
            if (
                not committed
                and lease_id in self._state["leases"]
                and self._state["leases"][lease_id]["valid"]
            ):
                self._invalidate_lease(lease_id, "pre-linearization-failure")
            raise

    def append_platform(
        self,
        lease_value: Any,
        *,
        connection: Any,
        transaction_nonce: str,
        fault_at: str | None = None,
        start_barrier: Any = None,
    ) -> dict[str, Any]:
        pre_linearization_faults = {
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
        }
        post_linearization_faults = {
            "linearize.after-commit",
            "reply.after-reply",
            "reply.after-reply-end",
            "reply.before-eof",
        }
        if fault_at is not None and fault_at not in (
            pre_linearization_faults | post_linearization_faults
        ):
            raise core.EvidenceError("unknown append fault point")
        if start_barrier is not None:
            try:
                start_barrier.wait(timeout=2)
            except threading.BrokenBarrierError as error:
                raise core.EvidenceError("append concurrency barrier broke") from error

        core.validate_sha256(transaction_nonce, "transaction nonce")
        lease_id = lease_value.get("lease_id") if type(lease_value) is dict else None
        committed = False
        with self._ledger_lock:
            try:
                lease, session = self._bound_append_lease_before_handles(
                    lease_value,
                    connection,
                )
                shadow = copy.deepcopy(self._state)
                shadow["leases"][lease_id]["transaction_nonce"] = transaction_nonce
                shadow["leases"][lease_id]["transaction_deadline"] = self._clock() + 5.0
                self._state = shadow
                lease = self._state["leases"][lease_id]
                session = self._state["sessions"][lease["producer_session_id"]]
                bodies = self._append_body_tuple(lease, session)
                frozen_record = core.canonical_json_bytes(bodies[4])
                if not 1 <= len(frozen_record) <= core.ORCHESTRATION_RECORD_MAX_BYTES:
                    raise core.EvidenceError("Host record is outside 1..61440 bytes")

                if fault_at in pre_linearization_faults:
                    raise core.EvidenceError(
                        f"injected pre-linearization fault at {fault_at}"
                    )
                receipt = self._linearize_append_happy_path(
                    lease,
                    transaction_nonce,
                    bodies,
                    frozen_record,
                )
                committed = True
                if fault_at in post_linearization_faults:
                    raise core.EvidenceError(
                        f"injected post-commit fault at {fault_at}"
                    )
                return receipt
            except BaseException:
                if (
                    not committed
                    and lease_id in self._state["leases"]
                    and self._state["leases"][lease_id]["valid"]
                ):
                    self._invalidate_lease(
                        lease_id,
                        "pre-linearization-failure",
                    )
                raise

    def enumerate_attempt(self, attempt_binding_id: str) -> dict[str, Any]:
        attempt = self._state["attempts"].get(attempt_binding_id)
        if attempt is None:
            raise core.EvidenceError("attempt binding is unknown")
        return copy.deepcopy(attempt)

    def invoke_root_internal(
        self,
        operation: Any,
        request: Any,
        *,
        principal: Any,
        fault_at: Any = None,
    ) -> dict[str, Any]:
        if principal != self.ROOT_SUPERVISOR_PRINCIPAL:
            raise core.EvidenceError("root supervisor authentication failed")
        if operation != "enumerate-attempt":
            raise core.EvidenceError("unsupported root-internal operation")
        if type(request) is not dict or set(request) != {"attempt_binding_id"}:
            raise core.EvidenceError("enumerate-attempt request must be exact")
        return self.enumerate_attempt(request["attempt_binding_id"])
