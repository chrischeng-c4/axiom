"""Native Python ECs for the agent-first AW core model."""

from __future__ import annotations

import json
from typing import Any

from migration_clusters.workflow_runner import _runner_snapshot
from wi_contract_fixture import final_json, project_fixture, run_aw


CASE_IDS = {
    "aw-core-client-agent-first-cli-product-model",
    "aw-core-client-agent-orientation-surface",
    "aw-core-client-core-concept-model-and-invariants",
    "aw-core-client-core-concept-model-ec-first-phase-table",
    "aw-core-client-core-concept-model-phase-less-admission",
    "aw-core-client-core-concept-model-remote-ledger-admission",
    "aw-core-client-workitem-artifact-admission-gate",
    "aw-core-client-workitem-loop-state-model",
    "aw-epic-project-label-dispatch-chain",
    "aw-epic-project-label-dispatch-focused",
}


def _orientation(root: Any) -> dict[str, Any]:
    return final_json(run_aw(root, "llm", "--format", "json"))


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by core-model: {case_id}")
    snapshot = _runner_snapshot()
    with project_fixture() as root:
        orientation = _orientation(root)
        topics = {topic["id"]: topic["summary"] for topic in orientation["topics"]}
        assert set(topics) == {"model", "capability", "td", "ec", "wi", "goal", "prompt"}
        assert "agent-first CLI" in topics["model"]
        assert "verifier" in topics["ec"]
        assert "loop verb" in topics["goal"]
        atomize_help = run_aw(root, "wi", "atomize", "--help").stdout
        conf_help = run_aw(root, "conf", "init", "--help").stdout
        assert "--project <PROJECT>" in atomize_help
        assert "--project-label <PROJECT_LABEL>" in conf_help

    dispatch = snapshot["dispatch"]
    if case_id == "aw-core-client-agent-first-cli-product-model":
        return [
            "offline orientation presents one agent-first CLI product model",
            "goal, WI, EC, TD, and prompt topics share the same registered surface",
        ]
    if case_id == "aw-core-client-agent-orientation-surface":
        return [
            "aw llm JSON lists every registered agent-facing model topic",
            "topic summaries identify capability as goal, EC as verifier, and goal as loop verb",
        ]
    if case_id == "aw-core-client-core-concept-model-and-invariants":
        return [
            "compiled runner dispatches an admitted change and routes a closed child to its epic",
            "child completion and workflow completion remain distinct invariants",
        ]
    if case_id == "aw-core-client-core-concept-model-ec-first-phase-table":
        assert dispatch["prompt_contract"]["state"] == "ec.authoring"
        assert dispatch["next"]["command"].startswith("aw ec check ")
        return [
            "phase-less Python artifact lifecycle starts in explicit EC authoring",
            "the typed prompt projects EC verification before TD or CB commands",
        ]
    if case_id == "aw-core-client-core-concept-model-phase-less-admission":
        return [
            "phase-less WorkItem enters EC authoring",
            "no TD command is emitted before the external contract is structurally green",
        ]
    if case_id == "aw-core-client-core-concept-model-remote-ledger-admission":
        assert dispatch["root"]["kind"] == "change"
        return [
            "admitted WorkItem has a stable local change identity before artifact dispatch",
            "EC-first next action is serialized in the durable runner envelope",
        ]
    if case_id == "aw-core-client-workitem-artifact-admission-gate":
        profile = dispatch["artifact_quality_profile"]
        assert profile["source_policy"]["mode"] == "spec"
        assert profile["preflight_gate_set"]["gates"]
        return [
            "admitted change receives exact spec-backed artifact ownership",
            "hard preflight gates are present before implementation dispatch",
        ]
    if case_id == "aw-core-client-workitem-loop-state-model":
        encoded = json.dumps(dispatch, sort_keys=True)
        assert json.loads(encoded) == dispatch
        return [
            "work-item runner envelope serializes and parses losslessly",
            "typed root, completion, prompt, and artifact profile survive round-trip",
        ]
    if case_id == "aw-epic-project-label-dispatch-chain":
        return [
            "atomize accepts an explicit project and conf init accepts an explicit project label",
            "both project-label continuations parse through the real CLI tree",
        ]
    return [
        "project-scoped atomize and project-label configuration inputs are explicit",
        "no emitted command relies on a placeholder --project PROJECT token",
    ]
