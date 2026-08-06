"""Black-box contract for prioritized reviewed-graph goal selection (#3298)."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw


CASE_ID = "workflow-root-runner-prioritized-reviewed-graph-goal-selection"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "prioritized-reviewed-graph-goal-selection"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case workflow-root-runner-prioritized-reviewed-graph-goal-selection"
)
ASSERTIONS = (
    "two open epics of different priority, each owning one ready change whose own change-priority is inverted relative to its epic (the higher-priority epic holding the lower-priority change and vice versa), make the live aw goal backlog selection pick the change owned by the higher-priority epic -- proving epic priority chooses project direction strictly before any ready child's own priority is ever consulted",
    "within a single epic, a change that depends on a still-open sibling change is skipped in favor of a lower-priority sibling that has no open dependencies, and the live envelope records a dependency-blocked reason naming the blocking change -- proving a blocked leaf is parked rather than hiding the next ready leaf",
    "an epic whose only change is closed after publish, with no re-planning in between, makes the live aw goal backlog dispatch roll up to the epic itself rather than reporting completion or an error -- proving a fully-closed epic's terminal state is a live rollup the same reviewed graph produces, not a special-cased dead end",
    "directly corrupting the published project-plan transaction manifest on disk so it no longer matches its own plan digest makes the live aw goal backlog fail closed with a blocked envelope whose remediation command is a bare `aw wi plan --project <p> --json` -- a fresh full replan starting at normalize, never a --stage atomize resume of the stale transaction -- proving invalid graph metadata is refused without re-atomizing",
)


_EPIC_BODY = (
    "## Requirements\n\n- R1: Demonstrate ready-graph selection.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `aw goal backlog --project demo` | Selection reflects the reviewed graph. | - |\n"
)


def _change_body() -> str:
    return (
        "## Problem\n\nDemonstrate ready-graph goal selection ordering.\n\n"
        "## Capability Alignment\n\n"
        "Capability: Workflow root runner\n"
        "Capability Gap: none, this fixture only drives the existing ready-graph selector\n"
        "Progress Evidence: the public goal backlog envelope is the evidence\n\n"
        "## Requirements\n\n- R1: trace ready-graph selection ordering.\n\n"
        "## Scope\n\n### In Scope\n- trace ready-graph selection ordering.\n\n"
        "### Out of Scope\n- Rework unrelated lifecycle stages.\n\n"
        "## Acceptance Criteria\n\n- AC1: selection reflects epic priority before change priority.\n\n"
        "## Reference Context\n\n### Related Specs\n"
        "| Spec | Relevance |\n|------|-----------|\n"
        "| complete-platform.md | describes the environment |\n\n"
        "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
        "|---------|--------|---------------|\n"
        "| ready-graph-trace | update | complete-platform.md |\n"
    )


def _workspace_slug(root: Path) -> str:
    resolved = str(root.resolve())
    collapsed = re.sub(r"[^a-zA-Z0-9]+", "-", resolved)
    return collapsed.strip("-").lower()


def _manifest_path(root: Path, project: str = "demo") -> Path:
    return (
        Path("/tmp/aw/workspaces")
        / _workspace_slug(root)
        / "workitems"
        / project
        / "project-plan"
        / "project-plan.manifest.json"
    )


def _create_epic(root: Path, title: str, priority: str) -> str:
    created = create(root, title, "epic", "--priority", priority, "--body", _EPIC_BODY)
    slug = created["slug"]
    validated = final_json(run_aw(root, "wi", "validate", slug))
    assert validated["passed"] is True, validated
    assert validated["new_state"] == "open", validated
    return slug


def _create_change(root: Path, title: str, priority: str, epic: str) -> str:
    created = create(
        root, title, "change", "--priority", priority, "--epic", epic, "--body", _change_body()
    )
    slug = created["slug"]
    validated = final_json(run_aw(root, "wi", "validate", slug))
    assert validated["passed"] is True, validated
    assert validated["new_state"] == "open", validated
    return slug


def _publish_graph(root: Path, project: str = "demo") -> None:
    normalize = final_json(
        run_aw(root, "wi", "plan", "--project", project, "--stage", "normalize")
    )
    assert normalize["status"] == "continue", normalize
    root_id = normalize["root"]["id"]

    for stage in ("reconcile", "atomize"):
        response = final_json(
            run_aw(root, "wi", "plan", "--project", project, "--stage", stage, "--root", root_id)
        )
        assert response["status"] == "continue", (stage, response)

    verified = final_json(
        run_aw(root, "wi", "plan", "--project", project, "--stage", "verify", "--root", root_id)
    )
    assert verified["status"] == "done", verified
    assert verified["completion"]["workflow_complete"] is True, verified


def _backlog(root: Path) -> dict[str, Any]:
    return final_json(run_aw(root, "goal", "backlog", "--project", "demo"))


def verify() -> list[str]:
    # Cluster 1: epic priority chooses project direction strictly before any
    # ready child's own priority is ever consulted.
    with project_fixture() as root:
        low_priority_epic = _create_epic(root, "Low priority epic", "p2")
        high_priority_epic = _create_epic(root, "High priority epic", "p0")
        _create_change(root, "Low epic best change", "p0", low_priority_epic)
        worst_change_in_high_epic = _create_change(
            root, "High epic worst change", "p3", high_priority_epic
        )
        _publish_graph(root)

        envelope = _backlog(root)
        assert envelope["action"] == "dispatch", envelope
        assert envelope["current"] == {
            "kind": "change",
            "id": worst_change_in_high_epic,
        }, envelope

    # Cluster 2: a dependency-blocked leaf is parked and never hides the next
    # ready leaf, even when the blocked leaf outranks it on priority alone.
    with project_fixture() as root:
        epic = _create_epic(root, "Blocked leaf epic", "p1")
        ready_change = _create_change(root, "Ready change", "p3", epic)
        blocked_change = _create_change(root, "Blocked change", "p0", epic)
        updated = final_json(
            run_aw(
                root,
                "wi",
                "update",
                blocked_change,
                "--add-label",
                f"depends-on:{ready_change}",
                "--json",
            )
        )
        assert f"depends-on:{ready_change}" in updated["labels"], updated
        _publish_graph(root)

        envelope = _backlog(root)
        assert envelope["action"] == "dispatch", envelope
        assert envelope["current"] == {"kind": "change", "id": ready_change}, envelope
        missing = envelope["completion"]["missing"]
        assert any(
            blocked_change in item and "blocked by open dependencies" in item
            for item in missing
        ), envelope

    # Cluster 3: an epic whose only change closes after publish, with no
    # re-planning in between, rolls the live dispatch up to the epic itself.
    with project_fixture() as root:
        epic = _create_epic(root, "Rollup epic", "p1")
        only_change = _create_change(root, "Rollup only change", "p0", epic)
        _publish_graph(root)

        pre_close = _backlog(root)
        assert pre_close["current"] == {"kind": "change", "id": only_change}, pre_close

        closed = final_json(run_aw(root, "wi", "close", only_change, "--json"))
        assert closed["state"] == "closed", closed

        post_close = _backlog(root)
        assert post_close["action"] == "dispatch", post_close
        assert post_close["current"] == {"kind": "epic", "id": epic}, post_close
        assert post_close["next"]["command"] == f"aw goal wi {epic}", post_close

    # Cluster 4: corrupting the published transaction manifest so it no
    # longer matches its own plan digest fails closed with a bare replan
    # command, never a --stage atomize resume of the stale transaction.
    with project_fixture() as root:
        epic = _create_epic(root, "Stale metadata epic", "p1")
        _create_change(root, "Stale metadata change", "p0", epic)
        _publish_graph(root)

        manifest_path = _manifest_path(root)
        assert manifest_path.is_file(), manifest_path
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        original_digest = manifest["plan_digest"]
        manifest["plan_digest"] = "0" * len(original_digest)
        assert manifest["plan_digest"] != original_digest, manifest
        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

        blocked = _backlog(root)
        assert blocked["action"] == "blocked", blocked
        assert blocked["next"]["command"] == "aw wi plan --project demo --json", blocked
        assert "--stage" not in blocked["next"]["command"], blocked
        assert "atomize" not in blocked["next"]["command"], blocked
        assert "do not describe the same root" in blocked["completion"]["missing"][0], blocked

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
