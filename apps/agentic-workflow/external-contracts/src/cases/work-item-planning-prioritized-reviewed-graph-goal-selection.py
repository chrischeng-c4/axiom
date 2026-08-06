"""Black-box contract for cross-root reviewed-graph goal selection parity (#3304)."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "work-item-planning-prioritized-reviewed-graph-goal-selection"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "prioritized-reviewed-graph-goal-selection"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-prioritized-reviewed-graph-goal-selection"
)
ASSERTIONS = (
    "given two epics with inverted epic/child priority (a p2 epic owning a p0-ready change, and a p0 epic "
    "owning only a p3-ready change), aw goal backlog dispatches the p0 epic's p3 change ahead of the p2 "
    "epic's p0 change, and calling aw goal wi directly on that same p0 epic independently selects the "
    "identical change id -- proving both roots agree on the same next change for the same reviewed graph "
    "rather than the epic-scoped root using different selection logic than the backlog root",
    "given one epic whose higher-priority child is blocked by a depends-on label on its lower-priority "
    "sibling, both aw goal backlog and aw goal wi on that epic dispatch the identical ready sibling change "
    "-- and aw goal wi's own completion.missing names the blocked change and its unmet dependency -- proving "
    "a blocked leaf is parked identically by both entry points without either one hiding the ready leaf",
    "once an epic's sole reviewed change is closed, aw goal backlog does not attempt its own terminal-epic "
    "bookkeeping: it reports the epic as current and hands off via next.command to aw goal wi on that exact "
    "epic, and calling that handed-off command directly is what actually resolves the terminal rollup, "
    "naming aw wi close <epic> --push as the next step -- proving the backlog root's rollup is genuinely "
    "delegated to, and consistent with, the epic root rather than a separately maintained duplicate path",
    "corrupting the on-disk project-plan manifest digest after a graph has been published makes both aw goal "
    "backlog and aw goal wi on the affected epic fail closed with the identical blocked diagnosis (plan and "
    "manifest do not describe the same root) and the identical bare-replan remedy, instead of one root "
    "silently dispatching against stale graph metadata while the other blocks",
)

_EPIC_BODY = (
    "## Problem\n\ndemo\n\n## Capability Alignment\n\nCapability: x\nCapability Gap: y\nProgress Evidence: z\n\n"
    "## Requirements\n\n- R1: Demonstrate ready-graph selection.\n\n## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `aw goal backlog --project demo` | Selection reflects the reviewed graph. | - |\n"
)
_CHANGE_BODY = (
    "## Problem\n\ndemo\n\n## Capability Alignment\n\nCapability: x\nCapability Gap: y\n"
    "Progress Evidence: z\n\n## Requirements\n\n- R1: trace ready-graph selection ordering.\n\n"
    "## Scope\n\n### In Scope\n- trace ready-graph selection ordering.\n\n"
    "### Out of Scope\n- Rework unrelated lifecycle stages.\n\n"
    "## Acceptance Criteria\n\n- AC1: both roots select the same next change.\n\n"
    "## Reference Context\n\n### Related Specs\n| Spec | Relevance |\n|------|-----------|\n"
    "| x.md | high |\n\n### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
    "|---------|--------|---------------|\n| x | modify | x.md |\n"
)


def _workspace_slug(root: Path) -> str:
    resolved = str(root.resolve())
    collapsed = re.sub(r"[^a-zA-Z0-9]+", "-", resolved)
    return collapsed.strip("-").lower()


def _manifest_path(root: Path, project: str = "demo") -> Path:
    return (
        Path("/tmp/aw/workspaces") / _workspace_slug(root) / "workitems" / project
        / "project-plan" / "project-plan.manifest.json"
    )


def _create_epic(root: Path, title: str, priority: str) -> str:
    created = create(root, title, "epic", "--priority", priority, "--body", _EPIC_BODY)
    slug = created["slug"]
    validated = final_json(run_aw(root, "wi", "validate", slug))
    assert validated["passed"] is True, validated
    return slug


def _create_change(root: Path, title: str, priority: str, epic: str) -> str:
    created = create(root, title, "change", "--priority", priority, "--epic", epic, "--body", _CHANGE_BODY)
    slug = created["slug"]
    validated = final_json(run_aw(root, "wi", "validate", slug))
    assert validated["passed"] is True, validated
    return slug


def _publish_graph(root: Path, project: str = "demo") -> None:
    normalize = final_json(run_aw(root, "wi", "plan", "--project", project, "--stage", "normalize", "--json"))
    root_id = normalize["root"]["id"]
    for stage in ("reconcile", "atomize"):
        response = final_json(
            run_aw(root, "wi", "plan", "--project", project, "--stage", stage, "--root", root_id, "--json")
        )
        assert response["status"] == "continue", (stage, response)
    verified = final_json(run_aw(root, "wi", "plan", "--project", project, "--stage", "verify", "--root", root_id, "--json"))
    assert verified["status"] == "done", verified


def _backlog(root: Path):
    return final_json(run_aw(root, "goal", "backlog", "--project", "demo"))


def _goal_wi(root: Path, wi_id: str, expect_success=True):
    return final_json(run_aw(root, "goal", "wi", wi_id, expect_success=expect_success))


def verify() -> list[str]:
    # Assertion 1: epic priority precedes child priority, and aw goal wi
    # <epic> independently arrives at the exact same dispatched change as
    # aw goal backlog picked for that epic across the whole project.
    with project_fixture() as root:
        low = _create_epic(root, "Low priority epic", "p2")
        high = _create_epic(root, "High priority epic", "p0")
        _create_change(root, "Low epic best change", "p0", low)
        winner = _create_change(root, "High epic worst change", "p3", high)
        _publish_graph(root)

        backlog_envelope = _backlog(root)
        assert backlog_envelope["current"] == {"kind": "change", "id": winner}, backlog_envelope

        epic_root_envelope = _goal_wi(root, high)
        assert epic_root_envelope["current"] == {"kind": "change", "id": winner}, epic_root_envelope

    # Assertion 2: a dependency-blocked higher-priority leaf is parked
    # identically by both roots, and the ready sibling is dispatched by
    # both, with the epic root's own completion.missing naming the block.
    with project_fixture() as root:
        epic = _create_epic(root, "Blocked leaf epic", "p1")
        ready_change = _create_change(root, "Ready change", "p3", epic)
        blocked_change = _create_change(root, "Blocked change", "p0", epic)
        final_json(run_aw(root, "wi", "update", blocked_change, "--add-label", f"depends-on:{ready_change}", "--json"))
        _publish_graph(root)

        backlog_envelope = _backlog(root)
        assert backlog_envelope["current"] == {"kind": "change", "id": ready_change}, backlog_envelope

        epic_root_envelope = _goal_wi(root, epic)
        assert epic_root_envelope["current"] == {"kind": "change", "id": ready_change}, epic_root_envelope
        missing = " ".join(epic_root_envelope.get("completion", {}).get("missing", []))
        assert blocked_change in missing and "blocked by open dependencies" in missing, epic_root_envelope

    # Assertion 3: once an epic's sole change is closed, the backlog root
    # delegates rollup to aw goal wi on that exact epic rather than doing
    # its own terminal bookkeeping; calling the delegated command is what
    # actually produces the terminal-close remediation.
    with project_fixture() as root:
        epic = _create_epic(root, "Rollup epic", "p1")
        only_change = _create_change(root, "Rollup only change", "p0", epic)
        _publish_graph(root)
        final_json(run_aw(root, "wi", "close", only_change, "--json"))

        backlog_envelope = _backlog(root)
        assert backlog_envelope["current"] == {"kind": "epic", "id": epic}, backlog_envelope
        assert backlog_envelope["next"]["command"] == f"aw goal wi {epic}", backlog_envelope

        epic_root_envelope = _goal_wi(root, epic)
        assert epic_root_envelope["current"] == {"kind": "epic", "id": epic}, epic_root_envelope
        assert epic_root_envelope["next"]["command"] == f"aw wi close {epic} --push", epic_root_envelope

    # Assertion 4: corrupted graph metadata fails closed identically for
    # both roots -- neither dispatches against a manifest that no longer
    # describes the reviewed plan it was checkpointed against.
    with project_fixture() as root:
        epic = _create_epic(root, "Stale metadata epic", "p1")
        _create_change(root, "Stale metadata change", "p0", epic)
        _publish_graph(root)

        manifest_path = _manifest_path(root)
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        original_digest = manifest["plan_digest"]
        manifest["plan_digest"] = "0" * len(original_digest)
        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

        backlog_blocked = _backlog(root)
        assert backlog_blocked["action"] == "blocked", backlog_blocked
        assert backlog_blocked["next"]["command"] == "aw wi plan --project demo --json", backlog_blocked
        assert "do not describe the same root" in " ".join(backlog_blocked["completion"]["missing"]), backlog_blocked

        epic_root_blocked = _goal_wi(root, epic, expect_success=None)
        assert epic_root_blocked["action"] == "blocked", epic_root_blocked
        assert "--stage" not in epic_root_blocked["next"]["command"], epic_root_blocked
        assert "atomize" not in epic_root_blocked["next"]["command"], epic_root_blocked
        assert "do not describe the same root" in " ".join(epic_root_blocked["completion"]["missing"]), epic_root_blocked

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
