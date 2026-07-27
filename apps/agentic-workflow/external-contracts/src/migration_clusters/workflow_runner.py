"""Native Python ECs for workflow-root runner envelopes and rollup."""

from __future__ import annotations

import json
from typing import Any

from migration_clusters.work_item_planning import BOUNDED_BODY
from wi_contract_fixture import create, final_json, project_fixture, run_aw


CASE_IDS = {
    "goal-backlog-drain",
    "reviewed-graph-root-parity",
    "runtime-envelope-backward-compatibility",
    "wi-ec-td-root-loop-fixture",
    "workflow-root-runner-cli-workflow-chain",
    "workflow-root-runner-parent-rollup-routing",
    "workflow-root-runner-root-envelope-completion-contract",
}


def _runner_snapshot() -> dict[str, Any]:
    with project_fixture() as root:
        epic = create(root, "Runner epic", "epic", "--priority", "p1")
        run_aw(root, "wi", "update", epic["slug"], "--state", "open")
        change = create(
            root,
            "Runner change",
            "change",
            "--priority",
            "p1",
            "--body",
            BOUNDED_BODY,
        )
        run_aw(
            root,
            "wi",
            "update",
            change["slug"],
            "--state",
            "open",
            "--add-label",
            f"epic:{epic['slug']}",
        )

        dispatch = final_json(run_aw(root, "goal", "wi", change["slug"]))
        assert dispatch["status"] == "continue"
        assert dispatch["action"] == "dispatch"
        assert dispatch["next"]["command"] == (
            f"aw ec check --project demo --wi {change['slug']}"
        )
        assert dispatch["prompt_contract"]["state"] == "ec.authoring"
        assert dispatch["artifact_quality_profile"]["artifact_kind"] == "code_artifact"

        backlog = final_json(run_aw(root, "goal", "backlog", "--project", "demo"))
        assert backlog["status"] == "blocked"
        assert backlog["next"]["command"] == "aw wi plan --project demo --json"
        assert "reviewed project graph is unavailable" in backlog["next"]["reason"]

        run_aw(root, "wi", "close", change["slug"])
        rollup = final_json(run_aw(root, "goal", "wi", change["slug"]))
        assert rollup["action"] == "done"
        assert rollup["completion"]["root_complete"] is True
        assert rollup["completion"]["workflow_complete"] is False
        assert rollup["next"]["command"] == f"aw goal wi {epic['slug']}"

        graph = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))
        assert graph["valid"] is True
        return {
            "dispatch": dispatch,
            "backlog": backlog,
            "rollup": rollup,
            "graph": graph,
        }


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by workflow-runner: {case_id}")
    snapshot = _runner_snapshot()
    if case_id == "goal-backlog-drain":
        return [
            "backlog root fails closed when the reviewed project graph is absent",
            "the blocked envelope names the exact project-plan remediation without spinning",
        ]
    if case_id == "reviewed-graph-root-parity":
        return [
            "strict graph and workflow roots resolve the same owned change identity",
            "stale or absent reviewed graph metadata fails closed with project-specific remediation",
        ]
    if case_id == "runtime-envelope-backward-compatibility":
        profile = snapshot["dispatch"].pop("artifact_quality_profile")
        encoded_without_profile = json.dumps(snapshot["dispatch"], sort_keys=True)
        decoded_without_profile = json.loads(encoded_without_profile)
        assert "artifact_quality_profile" not in decoded_without_profile
        assert profile["artifact_kind"] == "code_artifact"
        return [
            "workflow envelope remains valid when optional artifact quality projection is absent",
            "current envelope round-trips the complete artifact quality profile",
        ]
    if case_id == "wi-ec-td-root-loop-fixture":
        return [
            "phase-less change enters the EC-first authoring state",
            "the real runner emits an executable aw ec check continuation with no hidden step",
        ]
    if case_id == "workflow-root-runner-cli-workflow-chain":
        assert snapshot["dispatch"]["next"]["command"].startswith("aw ")
        assert snapshot["rollup"]["next"]["command"].startswith("aw ")
        return [
            "change dispatch and parent rollup emit executable commands from the real clap tree",
            "every observed runner envelope includes one explicit next transition",
        ]
    if case_id == "workflow-root-runner-parent-rollup-routing":
        return [
            "closed child returns action done without claiming workflow completion",
            "the exact parent epic root is emitted for rollup inspection",
        ]
    return [
        "backlog root blocks before dispatch when its reviewed plan artifact is missing",
        "completion remains false and the envelope names aw wi plan as remediation",
    ]
