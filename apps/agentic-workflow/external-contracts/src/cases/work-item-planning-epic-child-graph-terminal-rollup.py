"""Black-box contract for epic child-graph terminal rollup (#3303)."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "work-item-planning-epic-child-graph-terminal-rollup"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "epic-child-graph-terminal-rollup"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-epic-child-graph-terminal-rollup"
)
ASSERTIONS = (
    "an epic whose two children are associated purely through the machine epic:<id> label -- one left open, one closed before the graph is published -- makes the live aw goal wi dispatch pick the single open child by id and report exactly one open child remaining, proving unfinished children are dispatched and a terminal sibling never masks or duplicates that count",
    "an epic whose only child is associated purely through legacy 'Parent Epic: <id>' body prose, with no machine label at all, is discovered by the live project-plan reconcile stage as a real parent relation requiring one explicit accept decision, then dispatches to that child while it is open and rolls the live aw goal wi dispatch up to `aw wi close <epic> --push` the moment that same child closes -- proving parent-relation discovery honors the legacy body contract and terminal rollup fires only once every discovered child is terminal",
    "in one project, an epic with no attached change at all is proposed a brand-new atomic child by the live wi plan atomize stage with reason missing_requirement_coverage, while a sibling epic whose single open child's own requirements text already covers its epic requirement is proposed nothing in that same atomize pass -- proving the planner re-atomizes exactly when no covering child graph exists and never when one already does",
)

_EPIC_BODY = (
    "## Requirements\n\n- R1: Demonstrate rollup graph selection.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `aw goal wi <slug>` | goal wi reflects rollup graph selection. | - |\n"
)


def _change_body(parent_prose: str = "") -> str:
    parent_line = f"{parent_prose}\n\n" if parent_prose else ""
    return (
        f"## Problem\n\ndemo\n\n## Capability Alignment\n\n{parent_line}"
        "Capability: x\nCapability Gap: y\nProgress Evidence: z\n\n"
        "## Requirements\n\n- R1: trace rollup graph selection ordering.\n\n"
        "## Scope\n\n### In Scope\n- a\n\n### Out of Scope\n- b\n\n"
        "## Acceptance Criteria\n\n- AC1: c\n\n## Reference Context\n\n### Related Specs\n"
        "| Spec | Relevance |\n|------|-----------|\n| x.md | high |\n\n"
        "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
        "|---------|--------|---------------|\n| x | modify | x.md |\n"
    )


def _create_epic(root: Path, title: str, priority: str = "p1") -> str:
    created = create(root, title, "epic", "--priority", priority, "--body", _EPIC_BODY)
    slug = created["slug"]
    validated = final_json(run_aw(root, "wi", "validate", slug))
    assert validated["passed"] is True, validated
    return slug


def _create_labeled_change(root: Path, title: str, epic: str, priority: str = "p1") -> str:
    created = create(
        root, title, "change", "--priority", priority, "--epic", epic, "--body", _change_body()
    )
    slug = created["slug"]
    validated = final_json(run_aw(root, "wi", "validate", slug))
    assert validated["passed"] is True, validated
    return slug


def _create_legacy_change(root: Path, title: str, epic: str, priority: str = "p1") -> str:
    created = create(
        root,
        title,
        "change",
        "--priority",
        priority,
        "--body",
        _change_body(f"Parent Epic: {epic}"),
    )
    slug = created["slug"]
    validated = final_json(run_aw(root, "wi", "validate", slug))
    assert validated["passed"] is True, validated
    return slug


def _workspace_slug(root: Path) -> str:
    resolved = str(root.resolve())
    collapsed = re.sub(r"[^a-zA-Z0-9]+", "-", resolved)
    return collapsed.strip("-").lower()


def _plan_path(root: Path, project: str = "demo") -> Path:
    return (
        Path("/tmp/aw/workspaces")
        / _workspace_slug(root)
        / "workitems"
        / project
        / "project-plan"
        / "project-plan.json"
    )


def _publish_simple(root: Path, project: str = "demo") -> None:
    """Publish a graph whose reconcile stage needs no human decision."""
    normalize = final_json(run_aw(root, "wi", "plan", "--project", project, "--stage", "normalize"))
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


def _publish_with_reconcile_approval(root: Path, project: str = "demo") -> None:
    """Publish a graph whose reconcile stage proposes a legacy-body promotion
    mutation and therefore stops for one explicit human accept decision."""
    response = final_json(run_aw(root, "wi", "plan", "--project", project, "--stage", "normalize"))
    assert response["status"] == "continue", response
    root_id = response["root"]["id"]

    response = final_json(
        run_aw(root, "wi", "plan", "--project", project, "--stage", "reconcile", "--root", root_id)
    )
    assert response["next"]["kind"] == "hitl", response
    question = response["hitl_question"]
    response = final_json(
        run_aw(
            root,
            "wi",
            "plan-answer",
            "--payload",
            response["next"]["payload_path"],
            "--question",
            question["id"],
            "--choice",
            "approve",
            "--json",
        )
    )
    assert response["status"] == "continue", response

    atomize = final_json(
        run_aw(root, "wi", "plan", "--project", project, "--stage", "atomize", "--root", root_id)
    )
    assert atomize["status"] == "continue", atomize
    verified = final_json(
        run_aw(root, "wi", "plan", "--project", project, "--stage", "verify", "--root", root_id)
    )
    assert verified["status"] == "done", verified


def verify() -> list[str]:
    # Cluster 1: machine-label association; dispatch to the sole open child
    # and never lose count to a closed sibling.
    with project_fixture() as root:
        epic = _create_epic(root, "Dispatch epic")
        open_child = _create_labeled_change(root, "Open child", epic, priority="p2")
        closed_child = _create_labeled_change(root, "Closed child", epic, priority="p0")
        closed = final_json(run_aw(root, "wi", "close", closed_child, "--json"))
        assert closed["state"] == "closed", closed
        _publish_simple(root)

        goal = final_json(run_aw(root, "goal", "wi", epic))
        assert goal["action"] == "dispatch", goal
        assert goal["current"] == {"kind": "change", "id": open_child}, goal
        assert goal["next"]["command"] == f"aw goal wi {open_child}", goal
        assert closed_child not in goal["next"]["command"], goal
        assert "epic has 1 open reviewed change(s) remaining" in goal["completion"]["missing"], goal

    # Cluster 2: legacy body-prose association with no machine label at all;
    # discovery requires one explicit reconcile accept decision, dispatches
    # to the open child, then rolls up to close the instant it terminates.
    with project_fixture() as root:
        epic = _create_epic(root, "Legacy parent epic")
        child = _create_legacy_change(root, "Legacy child", epic, priority="p0")
        _publish_with_reconcile_approval(root)

        plan = json.loads(_plan_path(root).read_text(encoding="utf-8"))
        (planned_epic,) = [e for e in plan["epics"] if e["id"] == epic]
        assert planned_epic["change_ids"] == [child], plan

        pre_close = final_json(run_aw(root, "goal", "wi", epic))
        assert pre_close["current"] == {"kind": "change", "id": child}, pre_close

        closed = final_json(run_aw(root, "wi", "close", child, "--json"))
        assert closed["state"] == "closed", closed

        post_close = final_json(run_aw(root, "goal", "wi", epic))
        assert post_close["action"] == "dispatch", post_close
        assert post_close["next"]["command"] == f"aw wi close {epic} --push", post_close
        assert "all known epic children are terminal" in post_close["next"]["reason"], post_close

    # Cluster 3: re-atomize proposes a new atomic child exactly when no
    # covering child graph exists, and proposes nothing once one does.
    with project_fixture() as root:
        childless_epic = _create_epic(root, "Childless epic")
        covered_epic = _create_epic(root, "Covered epic")
        _create_labeled_change(root, "Covering child", covered_epic, priority="p1")

        normalize = final_json(run_aw(root, "wi", "plan", "--project", "demo", "--stage", "normalize"))
        assert normalize["status"] == "continue", normalize
        root_id = normalize["root"]["id"]
        reconcile = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "reconcile", "--root", root_id)
        )
        assert reconcile["status"] == "continue", reconcile
        atomize = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "atomize", "--root", root_id)
        )
        assert atomize["plan"]["proposed_change_count"] == 1, atomize

        plan = json.loads(_plan_path(root).read_text(encoding="utf-8"))
        (proposal,) = plan["proposed_changes"]
        assert proposal["owner_epic"] == childless_epic, plan
        assert proposal["reason"] == "missing_requirement_coverage", plan

        (childless_planned,) = [e for e in plan["epics"] if e["id"] == childless_epic]
        (covered_planned,) = [e for e in plan["epics"] if e["id"] == covered_epic]
        assert childless_planned["requirements"][0]["status"] == "gap", plan
        assert covered_planned["requirements"][0]["status"] == "covered", plan

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
