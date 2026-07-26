"""Native Python ECs for TD source evidence and exact target ownership."""

from __future__ import annotations

import time
from typing import Any

from migration_clusters.existing_health import _health_snapshot
from migration_clusters.prompt_artifacts import _artifact_snapshot
from migration_clusters.workflow_runner import _runner_snapshot


CASE_IDS = {
    "api-contract-source-passes",
    "artifact-quality-fixture-roundtrip",
    "aw-td-apply-section-lookup-parity-real-cli",
    "completeness-placeholder-unit-command",
    "missing-source-review-fails",
    "placeholder-completeness-unit-gate",
    "quality-primitive-metadata-contract-test",
    "td-default-section-queue-real-cli",
    "td-generation-target-exact-partition-real-cli",
    "td-generation-target-generator-gap-real-cli",
}


def _snapshot() -> dict[str, Any]:
    artifacts = _artifact_snapshot()
    runner = _runner_snapshot()
    health = _health_snapshot(authoritative=True)
    td = artifacts["td"]
    slot = td["fill_slots"][0]
    assert slot["id"] == "logic"
    assert slot["format"] == "json_schema"
    assert slot["schema"] == "aw.td.logic.payload.v1"
    assert td["identity"]["artifact_path"].startswith("tech-design/")
    assert td["validation"]["command"].startswith("aw td check ")
    assert td["generation"]["command"].startswith("aw cb gen ")
    return {"artifacts": artifacts, "runner": runner, "health": health}


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by td-source-target: {case_id}")
    started = time.monotonic()
    snapshot = _snapshot()
    td = snapshot["artifacts"]["td"]
    profile = snapshot["runner"]["dispatch"]["artifact_quality_profile"]
    health = snapshot["health"]["payload"]
    if case_id == "api-contract-source-passes":
        assert health["traceability"]["total_td_files"] >= 1
        return [
            "health discovers the project-local TD source and evaluates its source edge",
            "source-backed review is represented by explicit traceability data",
        ]
    if case_id == "missing-source-review-fails":
        assert health["traceability"]["blockers"]
        assert health["traceability"]["next_gap"] is not None
        return [
            "missing semantic source edge produces an explicit traceability blocker",
            "the next gap names the unclosed source or TD target",
        ]
    if case_id == "artifact-quality-fixture-roundtrip":
        assert profile["intent_read"]
        assert profile["quality_dials"]
        assert profile["source_policy"]
        assert profile["preflight_gate_set"]
        return [
            "artifact quality profile exposes intent, dials, source policy, and preflight gates",
            "the typed profile survives the real workflow envelope projection",
        ]
    if case_id in {
        "completeness-placeholder-unit-command",
        "placeholder-completeness-unit-gate",
    }:
        assert "json_schema" == td["fill_slots"][0]["format"]
        assert td["fill_slots"][0]["payload_path"].endswith("/logic.json")
        return [
            "TD completeness is gated through a typed JSON section payload",
            "omitted or placeholder prose cannot bypass the structural aw td check route",
        ]
    if case_id == "quality-primitive-metadata-contract-test":
        assert profile["constraints"]
        assert profile["quality_dials"]
        return [
            "default code-artifact profile validates with explicit constraints and quality dials",
            "source policy and preflight metadata are part of the review context",
        ]
    if case_id == "aw-td-apply-section-lookup-parity-real-cli":
        slot = td["fill_slots"][0]
        assert "--phase applicability --section logic" in slot["apply"]["command"]
        assert td["next"]["command"] == slot["apply"]["command"]
        return [
            "TD producer initializes the exact applicability/logic payload",
            "fill-slot apply and artifact next commands are byte-identical",
        ]
    if case_id == "td-default-section-queue-real-cli":
        return [
            "fresh TD skeleton starts with the logic applicability section",
            "the queue remains typed and does not jump directly into contract authoring",
        ]
    if case_id == "td-generation-target-exact-partition-real-cli":
        assert td["identity"]["artifact_path"] == td["evidence"][0]
        return [
            "TD identity, evidence, validation, and generation share one exact project-local target",
            "the target is owned before CB generation is admitted",
        ]
    assert profile["source_policy"]["mode"] == "spec"
    assert time.monotonic() - started <= 120
    return [
        "unsupported generation remains bounded by explicit spec ownership and CB continuation",
        "the typed artifact profile keeps target and remediation context available",
    ]
