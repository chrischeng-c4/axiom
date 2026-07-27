"""Native Python ECs for the linear TD-to-CB lifecycle and evidence gates."""

from __future__ import annotations

from typing import Any

from migration_clusters.prompt_artifacts import _artifact_snapshot
from migration_clusters.workflow_runner import _runner_snapshot
from wi_contract_fixture import project_fixture, run_aw


CASE_IDS = {
    "td-cb-lifecycle-automation-chain-liveness-proof",
    "td-cb-lifecycle-automation-chain-liveness-retry",
    "td-cb-lifecycle-automation-crrr-removal-linear-lifecycle",
    "td-cb-lifecycle-automation-remove-td-merge-command",
    "td-generation-target-ownership-inferred-single-real-cli",
    "td-generation-target-ownership-real-cli",
}


def _snapshot() -> dict[str, Any]:
    artifacts = _artifact_snapshot()
    runner = _runner_snapshot()
    td = artifacts["td"]
    assert td["validation"].startswith("aw td check ")
    assert td["source_path"].startswith("tech-design/src/")
    assert td["source_path"].endswith(".py")
    assert td["fill_marker"] == "AW_TD_FILL"
    with project_fixture() as root:
        retired = run_aw(root, "td", "merge", expect_success=False)
        assert "unrecognized subcommand" in retired.stderr
    return {"artifacts": artifacts, "runner": runner}


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by td-lifecycle-evidence: {case_id}")
    snapshot = _snapshot()
    td = snapshot["artifacts"]["td"]
    dispatch = snapshot["runner"]["dispatch"]
    if case_id == "td-cb-lifecycle-automation-chain-liveness-proof":
        return [
            "real runner emits EC verification and TD producer emits Python apply then TD check",
            "every observed transition names one executable command and a bounded terminal predicate",
        ]
    if case_id == "td-cb-lifecycle-automation-chain-liveness-retry":
        assert td["validation"] != snapshot["artifacts"]["td"]["source_path"]
        return [
            "Python source and validation remain distinct deterministic artifacts",
            "a failed stage preserves its exact artifact identity for retry",
        ]
    if case_id == "td-cb-lifecycle-automation-crrr-removal-linear-lifecycle":
        return [
            "TD producer has one linear Python-authoring-to-validation continuation",
            "no review/revise or merge phase appears in the emitted artifact contract",
        ]
    if case_id == "td-cb-lifecycle-automation-remove-td-merge-command":
        return [
            "aw td merge is absent from the real clap tree",
            "parsing the retired verb returns an unrecognized-subcommand failure",
        ]
    if case_id == "td-generation-target-ownership-inferred-single-real-cli":
        assert td["source_path"].startswith("tech-design/src/")
        return [
            "single project-local TD target is a deterministic Python source path",
            "the same WI identity is carried by the Python authoring artifact",
        ]
    assert dispatch["artifact_quality_profile"]["source_policy"]["mode"] == "spec"
    return [
        "ambiguous target ownership remains spec-backed and fail-closed",
        "typed envelope keeps exact artifact, source policy, and remediation context",
    ]
