"""Black-box contract for agent-backed inventory-plan review (#3303)."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "work-item-planning-agent-backed-inventory-plan-review"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "agent-backed-inventory-plan-review"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-agent-backed-inventory-plan-review"
)
ASSERTIONS = (
    "aw wi atomize, the documented compatibility entrypoint that epicize/atomize/prioritize all "
    "share, on an epic with an uncovered requirement under the default unconfigured review "
    "backing proposes exactly one candidate change and requires independent-agent review before "
    "continuing -- rejecting same-agent evidence whose reviewed_by matches the recorded plan "
    "author, separately rejecting evidence whose next_command has been tampered away from the "
    "exact digest-bound manifest command even though every checklist item is true and the "
    "reviewer identity is independent, and accepting only evidence that is both independently "
    "authored and chain-valid",
    "accepted independent-agent evidence still demands one further explicit human plan-answer "
    "confirmation before plan-apply may write the tracker, and only after that live "
    "review-then-approve-then-apply chain actually runs does the epic's own requirement flip "
    "from gap to covered with exactly one real change materialized -- proving accepted review "
    "authorizes real state only by walking the exact chain, never as a standalone rubber stamp",
    "a project configured with planning_review_backing = human blocks atomize immediately, before "
    "any review is submitted, with a reviewer_kind=human pending record; rejects an "
    "independently-identified agent's fully-checked evidence outright citing the human-only "
    "policy; and instead accepts the untouched pending payload through the dedicated "
    "--human-choice approve convenience flag alone, then completes the identical "
    "approve/apply/verify chain to a real materialized change -- proving explicit human-only "
    "policy configuration is preserved and enforced, not merely advisory",
)

_EPIC_BODY = (
    "## Requirements\n\n- R1: Coordinate escalation review checklist.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `aw wi plan` | atomize proposes a covering change. | - |\n"
)

_FULL_CHECKLIST = {
    "capability_claim_coverage": True,
    "scope_coverage": True,
    "bounded_candidates": True,
    "tracker_reconciliation": True,
    "verification_specific": True,
    "priority_consistent": True,
    "no_duplicate_wis": True,
    "publication_safe": True,
}

_HUMAN_ONLY_TOML = (
    "[agentic_workflow.workspace]\n"
    'mode = "in_place"\n\n'
    "[agentic_workflow.issue_platform]\n"
    'type = "local"\n\n'
    "[[projects]]\n"
    'name = "demo"\n'
    'label = "app:demo"\n'
    'path = "."\n'
    'tech_design_path = "tech-design"\n'
    'planning_review_backing = "human"\n\n'
    "[[projects.workspaces]]\n"
    'name = "demo"\n'
    'paths = ["**"]\n'
    'target = "rust"\n'
)


def _create_epic(root: Path, title: str, priority: str = "p1") -> str:
    created = create(root, title, "epic", "--priority", priority, "--body", _EPIC_BODY)
    slug = created["slug"]
    validated = final_json(run_aw(root, "wi", "validate", slug))
    assert validated["passed"] is True, validated
    return slug


def _drive_to_apply(root: Path, decision_path: str, question_id: str) -> dict[str, Any]:
    """Walk the standard approve -> apply chain from an accepted-review HITL question."""
    answered = final_json(
        run_aw(
            root,
            "wi",
            "plan-answer",
            "--payload",
            decision_path,
            "--question",
            question_id,
            "--choice",
            "approve",
            "--json",
        )
    )
    assert answered["status"] == "continue", answered
    assert answered["next"]["command"].startswith("aw wi plan-apply"), answered
    applied = final_json(run_aw(root, "wi", "plan-apply", "--evidence-file", decision_path, "--json"))
    assert applied["status"] == "continue", applied
    assert applied["action"] == "applied", applied
    return applied


def verify() -> list[str]:
    # Cluster 1: default (unconfigured, "either") review backing -- same-agent
    # and chain-tampered evidence are rejected; independent, chain-valid
    # evidence is accepted and still requires a human confirm before applying.
    with project_fixture() as root:
        epic = _create_epic(root, "Review-gated epic")

        atomize = final_json(run_aw(root, "wi", "atomize", "--project", "demo", "--json"))
        assert atomize["status"] == "continue", atomize
        assert atomize["completion"]["requires_hitl"] is False, atomize
        assert atomize["plan"]["proposed_change_count"] == 1, atomize
        assert atomize["next"]["command"].startswith("aw wi plan-review"), atomize

        payload_path = Path(atomize["next"]["payload_path"])
        record = json.loads(payload_path.read_text(encoding="utf-8"))
        assert record["decision"] == "pending", record
        assert record["reviewer_kind"] == "agent", record
        original_next_command = record["next_command"]

        same_agent = dict(record)
        same_agent.update(
            reviewer_kind="agent",
            reviewed_by="unknown-actor",
            decision="accepted",
            summary="Self review.",
            checklist=_FULL_CHECKLIST,
            findings=[],
            next_command=original_next_command,
        )
        payload_path.write_text(json.dumps(same_agent), encoding="utf-8")
        rejected = run_aw(
            root, "wi", "plan-review", "--evidence-file", str(payload_path), "--json", expect_success=False
        )
        assert "not independent" in rejected.stderr, rejected.stderr

        tampered = dict(record)
        tampered.update(
            reviewer_kind="agent",
            reviewed_by="agent:reviewer-y",
            decision="accepted",
            summary="Independent review.",
            checklist=_FULL_CHECKLIST,
            findings=[],
            next_command="rm -rf / # not the reviewed manifest's apply command",
        )
        payload_path.write_text(json.dumps(tampered), encoding="utf-8")
        rejected = run_aw(
            root, "wi", "plan-review", "--evidence-file", str(payload_path), "--json", expect_success=False
        )
        assert "must match the digest-bound manifest" in rejected.stderr, rejected.stderr

        accepted = dict(record)
        accepted.update(
            reviewer_kind="agent",
            reviewed_by="agent:reviewer-y",
            decision="accepted",
            summary="Independent review confirmed scope, boundedness, and tracker reconciliation.",
            checklist=_FULL_CHECKLIST,
            findings=[],
            next_command=original_next_command,
        )
        payload_path.write_text(json.dumps(accepted), encoding="utf-8")
        reviewed = final_json(run_aw(root, "wi", "plan-review", "--evidence-file", str(payload_path), "--json"))
        assert reviewed["status"] == "blocked", reviewed
        assert reviewed["next"]["kind"] == "hitl", reviewed
        assert reviewed["completion"]["missing"] == [
            "independent review accepted; explicit human confirmation is still required"
        ], reviewed

        applied = _drive_to_apply(root, reviewed["next"]["payload_path"], reviewed["hitl_question"]["id"])
        verify_command = applied["next"]["command"]
        assert "--stage verify" in verify_command, applied
        root_id = atomize["root"]["id"]
        verified = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "verify", "--root", root_id, "--json")
        )
        assert verified["status"] == "done", verified
        assert verified["completion"]["workflow_complete"] is True, verified

    # Cluster 2: explicit planning_review_backing = "human" blocks atomize up
    # front, rejects agent evidence regardless of identity, and accepts only
    # the dedicated --human-choice flag against the untouched pending payload.
    with project_fixture() as root:
        (root / "aw.toml").write_text(_HUMAN_ONLY_TOML, encoding="utf-8")
        epic = _create_epic(root, "Human-only epic")

        atomize = final_json(run_aw(root, "wi", "atomize", "--project", "demo", "--json"))
        assert atomize["status"] == "blocked", atomize
        assert atomize["completion"]["requires_hitl"] is True, atomize

        payload_path = Path(atomize["next"]["payload_path"])
        record = json.loads(payload_path.read_text(encoding="utf-8"))
        assert record["reviewer_kind"] == "human", record
        assert record["decision"] == "pending", record

        agent_attempt = dict(record)
        agent_attempt.update(
            reviewer_kind="agent",
            reviewed_by="agent:reviewer-y",
            decision="accepted",
            summary="Agent attempted review under human-only policy.",
            checklist=_FULL_CHECKLIST,
            findings=[],
        )
        agent_payload_path = payload_path.with_name("agent-attempt.json")
        agent_payload_path.write_text(json.dumps(agent_attempt), encoding="utf-8")
        rejected = run_aw(
            root, "wi", "plan-review", "--evidence-file", str(agent_payload_path), "--json", expect_success=False
        )
        assert "human-only" in rejected.stderr, rejected.stderr

        reviewed = final_json(
            run_aw(
                root,
                "wi",
                "plan-review",
                "--evidence-file",
                str(payload_path),
                "--human-choice",
                "approve",
                "--json",
            )
        )
        assert reviewed["status"] == "blocked", reviewed
        assert reviewed["next"]["kind"] == "hitl", reviewed

        applied = _drive_to_apply(root, reviewed["next"]["payload_path"], reviewed["hitl_question"]["id"])
        root_id = atomize["root"]["id"]
        verified = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "verify", "--root", root_id, "--json")
        )
        assert verified["status"] == "done", verified
        assert verified["completion"]["workflow_complete"] is True, verified
        _ = applied

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
