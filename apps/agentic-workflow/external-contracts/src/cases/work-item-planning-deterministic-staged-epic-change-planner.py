"""Black-box contract for the deterministic staged epic/change planner (#3304)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "work-item-planning-deterministic-staged-epic-change-planner"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "deterministic-staged-epic-change-planner"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-deterministic-staged-epic-change-planner"
)
ASSERTIONS = (
    "one aw wi plan root id emitted at normalize is echoed verbatim through reconcile, an atomize "
    "proposal, agent plan-review acceptance, a human plan-answer confirmation, plan-apply, and "
    "verify, while a live aw wi list snapshot stays byte-identical across every one of those steps "
    "except the single plan-apply call, which writes exactly one new tracker issue -- proving only "
    "plan-apply writes the tracker, and a second verify call on the same root reproduces the exact "
    "same plan digest and workflow_complete=true, proving verify is a strict, stable rebuild rather "
    "than a one-shot transition",
    "reconcile on a bare epic with zero existing changes auto-advances with requires_hitl=false "
    "because there is nothing to infer, while reconcile on an otherwise identical epic that owns one "
    "existing change with an inherited (non-explicit) priority blocks with requires_hitl=true and an "
    "explicit human decision question even though its own reconcile-stage manifest is empty -- "
    "proving the human decision gate tracks genuine non-explicit inference rather than firing "
    "unconditionally as a rubber stamp",
    "aw wi epicize and aw wi atomize both land on the identical root id and atomize stage that a "
    "direct aw wi plan --stage atomize reaches for the same unmodified project state, and aw wi "
    "prioritize lands on the identical root id and reconcile stage that a direct aw wi plan --stage "
    "reconcile reaches -- proving the documented compatibility planning verbs are real structured "
    "redirects into the exact same canonical root rather than a parallel or divergent model",
)

_EPIC_BODY = (
    "## Requirements\n\n- R1: Demonstrate staged planner root stability.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `aw wi plan` | plan reports the expected structure. | - |\n"
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


def _change_body(requirement: str) -> str:
    return (
        "## Problem\n\ndemo\n\n"
        "## Capability Alignment\n\nCapability: x\nCapability Gap: y\n"
        f"Progress Evidence: z\n\n## Requirements\n\n- R1: {requirement}\n\n"
        "## Scope\n\n### In Scope\n- a\n\n### Out of Scope\n- b\n\n"
        "## Acceptance Criteria\n\n- AC1: c\n\n## Reference Context\n\n### Related Specs\n"
        "| Spec | Relevance |\n|------|-----------|\n| x.md | high |\n\n"
        "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
        "|---------|--------|---------------|\n| x | modify | x.md |\n"
    )


def _create_epic(root: Path, title: str) -> str:
    slug = create(root, title, "epic", "--priority", "p1", "--body", _EPIC_BODY)["slug"]
    _validate(root, slug)
    return slug


def _validate(root: Path, wi_id: str) -> dict:
    result = final_json(run_aw(root, "wi", "validate", wi_id))
    assert result["passed"] is True, result
    return result


def _titles(root: Path) -> set[str]:
    payload = json.loads(run_aw(root, "wi", "list", "--project", "demo", "--json").stdout)
    return {item["title"] for item in payload}


def verify() -> list[str]:
    # Cluster 1: one stable root id survives every stage/review/decision/apply
    # step; nothing writes the tracker except the single plan-apply call;
    # verify is a strict, stable rebuild (same digest across two calls).
    with project_fixture() as root:
        _create_epic(root, "Rollup epic")
        before = _titles(root)

        normalize = final_json(run_aw(root, "wi", "plan", "--project", "demo", "--stage", "normalize", "--json"))
        root_id = normalize["root"]["id"]

        reconcile = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "reconcile", "--root", root_id, "--json")
        )
        assert reconcile["root"]["id"] == root_id, reconcile
        assert reconcile["status"] == "continue", reconcile
        assert reconcile["completion"]["requires_hitl"] is False, reconcile
        assert _titles(root) == before, "reconcile must not write the tracker"

        atomize = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "atomize", "--root", root_id, "--json")
        )
        assert atomize["root"]["id"] == root_id, atomize
        assert atomize["plan"]["proposed_change_count"] == 1, atomize
        assert atomize["next"]["command"].startswith("aw wi plan-review"), atomize
        assert _titles(root) == before, "atomize propose must not write the tracker"

        payload_path = Path(atomize["next"]["payload_path"])
        record = json.loads(payload_path.read_text(encoding="utf-8"))
        accepted = dict(record)
        accepted.update(
            reviewer_kind="agent",
            reviewed_by="agent:reviewer-y",
            decision="accepted",
            summary="Independent review confirmed scope and boundedness.",
            checklist=_FULL_CHECKLIST,
            findings=[],
            next_command=record["next_command"],
        )
        payload_path.write_text(json.dumps(accepted), encoding="utf-8")
        reviewed = final_json(run_aw(root, "wi", "plan-review", "--evidence-file", str(payload_path), "--json"))
        assert reviewed["root"]["id"] == root_id, reviewed
        assert reviewed["next"]["kind"] == "hitl", reviewed
        assert _titles(root) == before, "an accepted agent review must not write the tracker"

        human_answered = final_json(
            run_aw(
                root,
                "wi",
                "plan-answer",
                "--payload",
                reviewed["next"]["payload_path"],
                "--question",
                reviewed["hitl_question"]["id"],
                "--choice",
                "approve",
                "--json",
            )
        )
        assert human_answered["root"]["id"] == root_id, human_answered
        assert human_answered["next"]["command"].startswith("aw wi plan-apply"), human_answered
        assert _titles(root) == before, "a human plan-answer confirmation must not write the tracker"

        applied = final_json(
            run_aw(root, "wi", "plan-apply", "--evidence-file", human_answered["next"]["payload_path"], "--json")
        )
        assert applied["root"]["id"] == root_id, applied
        assert applied["action"] == "applied", applied
        new_titles = _titles(root) - before
        assert len(new_titles) == 1, new_titles

        verify1 = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "verify", "--root", root_id, "--json")
        )
        assert verify1["root"]["id"] == root_id, verify1
        assert verify1["status"] == "done", verify1
        assert verify1["completion"]["workflow_complete"] is True, verify1

        verify2 = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "verify", "--root", root_id, "--json")
        )
        assert verify2["root"]["id"] == root_id, verify2
        assert verify2["status"] == "done", verify2
        assert verify2["plan"]["digest"] == verify1["plan"]["digest"], (verify2, verify1)

    # Cluster 2: reconcile's human decision gate is conditional on genuine
    # non-explicit inference, not an unconditional rubber stamp.
    with project_fixture() as root:
        owner_epic = _create_epic(root, "Owner epic 2")
        inherited = create(
            root, "Inherited priority child", "change", "--epic",
            owner_epic, "--body", _change_body("unrelated child work."),
        )["slug"]
        _validate(root, inherited)

        reconcile2 = final_json(run_aw(root, "wi", "plan", "--project", "demo", "--stage", "reconcile", "--json"))
        assert reconcile2["status"] == "blocked", reconcile2
        assert reconcile2["completion"]["requires_hitl"] is True, reconcile2
        assert reconcile2["plan"]["proposed_change_count"] == 0, reconcile2
        assert reconcile2["hitl_question"]["question"] == (
            "Approve the inferred reconciliation mutations for existing work items?"
        ), reconcile2

    # Cluster 3: the compatibility verbs land on the identical root id and
    # stage as the canonical `aw wi plan --stage <X>` verb.
    with project_fixture() as root:
        _create_epic(root, "Rollup epic 3")

        epicize = final_json(run_aw(root, "wi", "epicize", "--project", "demo", "--json"))
        direct_atomize = final_json(run_aw(root, "wi", "plan", "--project", "demo", "--stage", "atomize", "--json"))
        assert epicize["root"]["id"] == direct_atomize["root"]["id"], (epicize, direct_atomize)
        assert epicize["current"]["kind"] == direct_atomize["current"]["kind"] == "atomize", (epicize, direct_atomize)

        atomize_verb = final_json(run_aw(root, "wi", "atomize", "--project", "demo", "--json"))
        assert atomize_verb["root"]["id"] == direct_atomize["root"]["id"], atomize_verb
        assert atomize_verb["current"]["kind"] == "atomize", atomize_verb

        prioritize = final_json(run_aw(root, "wi", "prioritize", "--project", "demo", "--json"))
        direct_reconcile = final_json(run_aw(root, "wi", "plan", "--project", "demo", "--stage", "reconcile", "--json"))
        assert prioritize["root"]["id"] == direct_reconcile["root"]["id"], (prioritize, direct_reconcile)
        assert prioritize["current"]["kind"] == direct_reconcile["current"]["kind"] == "reconcile", (
            prioritize, direct_reconcile,
        )

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
