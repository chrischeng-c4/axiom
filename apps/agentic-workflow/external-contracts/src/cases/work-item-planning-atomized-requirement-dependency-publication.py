"""Black-box contract for atomized requirement dependency publication (#3304)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "work-item-planning-atomized-requirement-dependency-publication"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "atomized-requirement-dependency-publication"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-atomized-requirement-dependency-publication"
)
ASSERTIONS = (
    "an epic Verification Inventory Depends On column naming an unresolvable requirement id fails "
    "aw wi validate closed citing exactly the unknown requirement, and a separate epic whose two "
    "requirements depend on each other fails aw wi validate closed citing an acyclic-graph violation "
    "-- proving invalid Depends On declarations are rejected before an epic can ever reach open state",
    "atomizing a validated epic whose second requirement's Verification Inventory Depends On column "
    "names the first produces exactly two proposed changes, and the pending independent-review "
    "manifest's dependent mutation carries an add_labels entry referencing the first requirement's "
    "still-symbolic proposal id at manifest order 2 immediately after the prerequisite's own create "
    "at manifest order 1 -- proving the dependency survives as a genuine symbolic proposal edge ahead "
    "of any tracker write, ordered so the prerequisite always publishes first",
    "accepting that review, confirming it as human, and running plan-apply publishes both changes with "
    "the dependent carrying a depends-on label that aw wi show independently resolves to the exact "
    "prerequisite change and title, while the prerequisite itself carries no leftover symbolic or "
    "self-referential label -- proving the published dependency is a real, addressable tracker edge "
    "rather than a stranded proposal reference",
)

_REFERENCE_CONTEXT = (
    "\n\n## Scope\n\n### In Scope\n- a\n\n### Out of Scope\n- b\n\n"
    "## Acceptance Criteria\n\n- AC1: c\n\n"
    "## Reference Context\n\n### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| x.md | high |\n\n"
    "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n"
    "| x | modify | x.md |\n"
)

_UNKNOWN_TARGET_BODY = (
    "## Problem\n\ndemo\n\n## Capability Alignment\n\nCapability: x\nCapability Gap: y\nProgress Evidence: z\n\n"
    "## Requirements\n\n- R1: cover requirement one.\n- R2: cover requirement two.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `aw wi plan` | R1 verified. | - |\n"
    "| R2 | `aw wi graph` | R2 verified. | R9 |\n"
    + _REFERENCE_CONTEXT
)

_CYCLE_BODY = (
    "## Problem\n\ndemo\n\n## Capability Alignment\n\nCapability: x\nCapability Gap: y\nProgress Evidence: z\n\n"
    "## Requirements\n\n- R1: cover requirement one.\n- R2: cover requirement two.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `aw wi plan` | R1 verified. | R2 |\n"
    "| R2 | `aw wi graph` | R2 verified. | R1 |\n"
    + _REFERENCE_CONTEXT
)

_LINKED_EPIC_BODY = (
    "## Problem\n\ndemo\n\n## Capability Alignment\n\nCapability: x\nCapability Gap: y\nProgress Evidence: z\n\n"
    "## Requirements\n\n- R1: Publish the native Python EC replacement.\n"
    "- R2: Delete the delegated Rust EC wrapper.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `python3 native_ec.py` | Native EC passes. | - |\n"
    "| R2 | `test ! -e delegated.rs` | Delegated wrapper is absent. | R1 |\n"
    + _REFERENCE_CONTEXT
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


def verify() -> list[str]:
    # Assertion 1: an epic's own promotion fails closed on an invalid Depends
    # On graph -- an unresolvable target and a dependency cycle each name
    # their own violation.
    with project_fixture() as root:
        unknown_slug = create(
            root, "Unknown dependency epic", "epic", "--priority", "p1", "--body", _UNKNOWN_TARGET_BODY
        )["slug"]
        unknown_result = json.loads(
            run_aw(root, "wi", "validate", unknown_slug, "--json", expect_success=False).stdout
        )
        assert unknown_result["passed"] is False, unknown_result
        assert any("depends on unknown R9" in e for e in unknown_result["errors"]), unknown_result

        cycle_slug = create(root, "Cycle dependency epic", "epic", "--priority", "p1", "--body", _CYCLE_BODY)[
            "slug"
        ]
        cycle_result = json.loads(
            run_aw(root, "wi", "validate", cycle_slug, "--json", expect_success=False).stdout
        )
        assert cycle_result["passed"] is False, cycle_result
        assert any("must be acyclic" in e for e in cycle_result["errors"]), cycle_result

    # Assertions 2 and 3: a validated epic with two dependency-linked,
    # uncovered requirements atomizes into two proposed changes whose pending
    # review manifest carries a genuine symbolic dependency edge in
    # publish order, and a full accept/confirm/apply publishes a real,
    # independently addressable depends-on edge.
    with project_fixture() as root:
        epic_slug = create(
            root, "Dependent requirements epic", "epic", "--priority", "p1", "--body", _LINKED_EPIC_BODY
        )["slug"]
        validated = final_json(run_aw(root, "wi", "validate", epic_slug))
        assert validated["passed"] is True, validated

        normalize = final_json(run_aw(root, "wi", "plan", "--project", "demo", "--stage", "normalize", "--json"))
        root_id = normalize["root"]["id"]
        reconcile = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "reconcile", "--root", root_id, "--json")
        )
        if reconcile.get("next", {}).get("kind") == "hitl":
            question = reconcile["hitl_question"]
            reconcile = final_json(
                run_aw(
                    root, "wi", "plan-answer", "--payload", reconcile["next"]["payload_path"],
                    "--question", question["id"], "--choice", "approve", "--json",
                )
            )
        assert reconcile["status"] == "continue", reconcile

        atomize = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "atomize", "--root", root_id, "--json")
        )
        assert atomize["plan"]["proposed_change_count"] == 2, atomize

        plan_path = Path(atomize["plan"]["path"])
        plan = json.loads(plan_path.read_text(encoding="utf-8"))
        proposed_by_id = {p["id"]: p for p in plan["proposed_changes"]}
        prerequisite_id = f"proposal:change:{epic_slug}:requirement-1"
        dependent_id = f"proposal:change:{epic_slug}:requirement-2"
        assert proposed_by_id[prerequisite_id]["dependencies"] == [], proposed_by_id[prerequisite_id]
        assert proposed_by_id[dependent_id]["dependencies"] == [prerequisite_id], proposed_by_id[dependent_id]

        payload_path = Path(atomize["next"]["payload_path"])
        record = json.loads(payload_path.read_text(encoding="utf-8"))
        manifest = json.loads(Path(record["manifest_path"]).read_text(encoding="utf-8"))
        mutations_by_target = {m["target"]: m for m in manifest["mutations"]}
        prerequisite_mutation = mutations_by_target[prerequisite_id]
        dependent_mutation = mutations_by_target[dependent_id]
        assert prerequisite_mutation["order"] == 1, prerequisite_mutation
        assert dependent_mutation["order"] == 2, dependent_mutation
        assert f"depends-on:{prerequisite_id}" in dependent_mutation["add_labels"], dependent_mutation
        assert not any(
            label.startswith("depends-on:") for label in prerequisite_mutation["add_labels"]
        ), prerequisite_mutation

        accepted = dict(record)
        accepted.update(
            reviewer_kind="agent",
            reviewed_by="agent:reviewer-y",
            decision="accepted",
            summary="Independent review confirmed the dependency ordering and scope.",
            checklist=_FULL_CHECKLIST,
            findings=[],
            next_command=record["next_command"],
        )
        payload_path.write_text(json.dumps(accepted), encoding="utf-8")
        reviewed = final_json(run_aw(root, "wi", "plan-review", "--evidence-file", str(payload_path), "--json"))
        assert reviewed["next"]["kind"] == "hitl", reviewed

        human_answered = final_json(
            run_aw(
                root, "wi", "plan-answer", "--payload", reviewed["next"]["payload_path"],
                "--question", reviewed["hitl_question"]["id"], "--choice", "approve", "--json",
            )
        )
        applied = final_json(
            run_aw(root, "wi", "plan-apply", "--evidence-file", human_answered["next"]["payload_path"], "--json")
        )
        assert applied["action"] == "applied", applied

        listed = json.loads(run_aw(root, "wi", "list", "--project", "demo", "--json").stdout)
        prerequisite_issue = next(
            i for i in listed if i["title"] == "Publish the native Python EC replacement."
        )
        dependent_issue = next(i for i in listed if i["title"] == "Delete the delegated Rust EC wrapper.")
        assert not any(label.startswith("depends-on:") for label in prerequisite_issue["labels"]), (
            prerequisite_issue
        )
        (dependency_label,) = [label for label in dependent_issue["labels"] if label.startswith("depends-on:")]
        resolved_slug = dependency_label.split("depends-on:", 1)[1]
        assert not resolved_slug.startswith("proposal:"), dependency_label

        resolved = final_json(run_aw(root, "wi", "show", resolved_slug, "--json"))
        resolved_issue = resolved.get("issue", resolved)
        assert resolved_issue.get("title") == "Publish the native Python EC replacement.", resolved_issue

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
