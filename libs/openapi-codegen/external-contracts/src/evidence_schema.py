"""Strict evidence model shared by every external-contract case."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Iterable


PROTOCOL = "openapi-codegen.ec-evidence.v1"


def passed(check_id: str, observed_count: int, details: dict[str, Any]) -> dict[str, Any]:
    return {
        "id": check_id,
        "status": "passed",
        "observed_count": observed_count,
        "details": details,
    }


def failed(check_id: str, message: str, details: dict[str, Any] | None = None) -> dict[str, Any]:
    return {
        "id": check_id,
        "status": "failed",
        "observed_count": 0,
        "details": {"error": message, **(details or {})},
    }


def build_evidence(case_id: str, checks: Iterable[dict[str, Any]]) -> dict[str, Any]:
    materialized = list(checks)
    return {
        "protocol": PROTOCOL,
        "case_id": case_id,
        "status": "passed"
        if materialized and all(check.get("status") == "passed" for check in materialized)
        else "failed",
        "total_checks": len(materialized),
        "checks": materialized,
    }


def validate_evidence(evidence: dict[str, Any], expected_ids: tuple[str, ...]) -> list[str]:
    findings: list[str] = []
    if evidence.get("protocol") != PROTOCOL:
        findings.append(f"protocol must be {PROTOCOL}")
    checks = evidence.get("checks")
    if not isinstance(checks, list):
        return [*findings, "checks must be a list"]
    actual_ids = [check.get("id") for check in checks if isinstance(check, dict)]
    if tuple(actual_ids) != expected_ids:
        findings.append(f"check ids must be exactly {expected_ids!r}, got {tuple(actual_ids)!r}")
    if len(set(actual_ids)) != len(actual_ids):
        findings.append("check ids must be unique")
    if evidence.get("total_checks") != len(expected_ids) or len(checks) != len(expected_ids):
        findings.append("total_checks must equal the exact required check count")
    for check in checks:
        if not isinstance(check, dict):
            findings.append("every check must be an object")
            continue
        if check.get("status") != "passed":
            findings.append(f"{check.get('id', '<missing>')} did not pass")
        count = check.get("observed_count")
        if not isinstance(count, int) or isinstance(count, bool) or count <= 0:
            findings.append(f"{check.get('id', '<missing>')} must have observed_count > 0")
        if not isinstance(check.get("details"), dict) or not check["details"]:
            findings.append(f"{check.get('id', '<missing>')} must carry non-empty details")
    if evidence.get("status") != "passed" or findings:
        findings.append("aggregate status must be passed with no schema findings")
    return findings


def write_evidence(path: Path, evidence: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n")
    temporary.replace(path)
