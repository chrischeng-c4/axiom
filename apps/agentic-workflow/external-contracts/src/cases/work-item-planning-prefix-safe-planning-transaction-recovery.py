"""Black-box contract for prefix-safe planning transaction recovery (#3304)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "work-item-planning-prefix-safe-planning-transaction-recovery"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "prefix-safe-planning-transaction-recovery"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-prefix-safe-planning-transaction-recovery"
)
ASSERTIONS = (
    "publishing ten uncovered requirements where requirement-9 depends on requirement-10 resolves the "
    "published depends-on label to requirement-10's real change even though requirement-1's symbolic "
    "proposal id is a byte-for-byte prefix of requirement-10's -- proving symbolic proposal ids resolve "
    "by longest overlap rather than first or naive substring match",
    "reapplying the identical accepted evidence file a second time immediately after a successful "
    "plan-apply is a clean no-op that creates nothing new and leaves every published change identity "
    "untouched -- proving the accepted plan's bytes remain valid and idempotently replayable across "
    "canonical stage advancement rather than being a one-shot, single-use artifact",
    "externally corrupting a published dependent change's depends-on label and then reapplying that "
    "same accepted evidence file repairs the label back to the exact originally reviewed prerequisite "
    "without creating a single duplicate change -- proving marker-owned retries recover the precise "
    "reviewed managed graph rather than blindly recreating it",
    "an open change carrying a depends-on label naming an id that resolves to no real change invalidates "
    "aw wi graph with a missing_relation_target diagnostic naming exactly that id -- proving unresolved "
    "dependencies on open changes fail the graph closed on the label surface exactly as they do on the "
    "body-declaration surface",
)

_REFERENCE_CONTEXT = (
    "\n\n## Scope\n\n### In Scope\n- a\n\n### Out of Scope\n- b\n\n"
    "## Acceptance Criteria\n\n- AC1: c\n\n"
    "## Reference Context\n\n### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| x.md | high |\n\n"
    "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n"
    "| x | modify | x.md |\n"
)
_LINKED_EPIC_BODY = (
    "## Problem\n\ndemo\n\n## Capability Alignment\n\nCapability: x\nCapability Gap: y\nProgress Evidence: z\n\n"
    "## Requirements\n\n- R1: Publish the native Python EC replacement.\n"
    "- R2: Delete the delegated Rust EC wrapper.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `python3 native_ec.py` | Native EC passes. | - |\n"
    "| R2 | `test ! -e delegated.rs` | Delegated wrapper is absent. | R1 |\n" + _REFERENCE_CONTEXT
)
_SIMPLE_EPIC_BODY = (
    "## Requirements\n\n- R1: x.\n\n## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n|-------------|------|--------|------------|\n"
    "| R1 | `true` | ok. | - |\n"
)
_CHANGE_BODY = (
    "## Problem\n\ndemo\n\n## Capability Alignment\n\nCapability: x\nCapability Gap: y\n"
    "Progress Evidence: z\n\n## Requirements\n\n- R1: x.\n\n## Scope\n\n### In Scope\n- a\n\n"
    "### Out of Scope\n- b\n\n## Acceptance Criteria\n\n- AC1: c\n\n## Reference Context\n\n"
    "### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| x.md | high |\n\n"
    "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n"
    "| x | modify | x.md |\n"
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


def _requirements_prefix_epic_body() -> str:
    req_lines = []
    inv_rows = []
    for i in range(1, 11):
        if i == 9:
            label = "Filler requirement nine, depends on requirement ten."
            depends = "R10"
        elif i == 10:
            label = "Requirement ten (resolution target)."
            depends = "-"
        else:
            label = f"Filler requirement {i}."
            depends = "-"
        req_lines.append(f"- R{i}: {label}")
        inv_rows.append(f"| R{i} | `true` | R{i} verified. | {depends} |")
    return (
        "## Problem\n\ndemo\n\n## Capability Alignment\n\nCapability: x\nCapability Gap: y\nProgress Evidence: z\n\n"
        "## Requirements\n\n" + "\n".join(req_lines) + "\n\n"
        "## Verification Inventory\n\n"
        "| Requirement | Gate | Oracle | Depends On |\n"
        "|-------------|------|--------|------------|\n" + "\n".join(inv_rows) + "\n" + _REFERENCE_CONTEXT
    )


def _drive_to_apply(root: Path, epic_slug: str) -> str:
    """Normalize -> reconcile (approving a HITL if one fires) -> atomize -> agent
    review acceptance -> human plan-answer confirmation, returning the
    evidence-file path handed to plan-apply."""
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
    payload_path = Path(atomize["next"]["payload_path"])
    record = json.loads(payload_path.read_text(encoding="utf-8"))
    accepted = dict(record)
    accepted.update(
        reviewer_kind="agent",
        reviewed_by="agent:reviewer-y",
        decision="accepted",
        summary="Independent review confirmed scope and dependency ordering.",
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
    return human_answered["next"]["payload_path"]


def verify() -> list[str]:
    # Assertion 1: longest-prefix-first symbolic resolution. Ten uncovered
    # requirements push the proposal counter into double digits so
    # "requirement-1" is a literal byte-prefix of "requirement-10".
    with project_fixture() as root:
        epic_slug = create(
            root, "Prefix overlap epic", "epic", "--priority", "p1", "--body", _requirements_prefix_epic_body()
        )["slug"]
        validated = final_json(run_aw(root, "wi", "validate", epic_slug))
        assert validated["passed"] is True, validated

        evidence_file = _drive_to_apply(root, epic_slug)
        applied = final_json(run_aw(root, "wi", "plan-apply", "--evidence-file", evidence_file, "--json"))
        assert applied["action"] == "applied", applied

        listed = json.loads(run_aw(root, "wi", "list", "--project", "demo", "--json").stdout)
        r9_issue = next(i for i in listed if "Filler requirement nine" in i["title"])
        dependency_label = next(l for l in r9_issue["labels"] if l.startswith("depends-on:"))
        resolved_slug = dependency_label.split("depends-on:", 1)[1]
        assert not resolved_slug.startswith("proposal:"), dependency_label

        resolved = final_json(run_aw(root, "wi", "show", resolved_slug, "--json"))
        resolved_issue = resolved.get("issue", resolved)
        assert "Requirement ten" in resolved_issue.get("title", ""), resolved_issue
        assert "Filler requirement 1." not in resolved_issue.get("title", ""), resolved_issue

    # Assertions 2 and 3: accepted plan bytes survive a clean idempotent
    # reapply, and a marker-owned retry repairs an externally corrupted
    # dependency label without duplicating any change.
    with project_fixture() as root:
        epic_slug2 = create(root, "Retry recovery epic", "epic", "--priority", "p1", "--body", _LINKED_EPIC_BODY)[
            "slug"
        ]
        final_json(run_aw(root, "wi", "validate", epic_slug2))
        evidence_file2 = _drive_to_apply(root, epic_slug2)

        applied1 = final_json(run_aw(root, "wi", "plan-apply", "--evidence-file", evidence_file2, "--json"))
        assert applied1["action"] == "applied", applied1
        graphed1 = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))
        assert len(graphed1["changes"]) == 2, graphed1

        applied2 = final_json(run_aw(root, "wi", "plan-apply", "--evidence-file", evidence_file2, "--json"))
        assert applied2["action"] == "applied", applied2
        graphed2 = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))
        assert len(graphed2["changes"]) == 2, "clean reapply must not create anything new"
        assert {c["id"] for c in graphed1["changes"]} == {c["id"] for c in graphed2["changes"]}, (
            "clean reapply must not alter identities"
        )

        dependent_slug = next(
            c["id"] for c in graphed2["changes"] if c["title"] == "Delete the delegated Rust EC wrapper."
        )
        dependent_shown = final_json(run_aw(root, "wi", "show", dependent_slug, "--json"))
        dependent_before = dependent_shown.get("issue", dependent_shown)
        real_dependency_label = next(l for l in dependent_before["labels"] if l.startswith("depends-on:"))
        final_json(
            run_aw(
                root, "wi", "update", dependent_slug, "--remove-label", real_dependency_label,
                "--add-label", "depends-on:99999999", "--json",
            )
        )
        corrupted_shown = final_json(run_aw(root, "wi", "show", dependent_slug, "--json"))
        corrupted = corrupted_shown.get("issue", corrupted_shown)
        assert "depends-on:99999999" in corrupted["labels"], corrupted

        applied3 = final_json(run_aw(root, "wi", "plan-apply", "--evidence-file", evidence_file2, "--json"))
        assert applied3["action"] == "applied", applied3
        graphed3 = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))
        assert len(graphed3["changes"]) == 2, "repair must not create duplicates"

        repaired_shown = final_json(run_aw(root, "wi", "show", dependent_slug, "--json"))
        repaired = repaired_shown.get("issue", repaired_shown)
        assert "depends-on:99999999" not in repaired["labels"], repaired
        assert real_dependency_label in repaired["labels"], repaired

    # Assertion 4: an unresolved depends-on label on an open change fails the
    # graph closed exactly like the equivalent body-declaration case does.
    with project_fixture() as root:
        epic_slug3 = create(root, "Graph invalidation epic", "epic", "--priority", "p1", "--body", _SIMPLE_EPIC_BODY)[
            "slug"
        ]
        final_json(run_aw(root, "wi", "validate", epic_slug3))
        labeled_slug = create(root, "Labeled dependency change", "change", "--epic", epic_slug3, "--body", _CHANGE_BODY)[
            "slug"
        ]
        final_json(run_aw(root, "wi", "update", labeled_slug, "--add-label", "depends-on:8675309", "--json"))
        final_json(run_aw(root, "wi", "validate", labeled_slug))

        invalid = run_aw(root, "wi", "graph", "--project", "demo", "--json", expect_success=False)
        graph_payload = json.loads(invalid.stdout)
        assert graph_payload["valid"] is False, graph_payload
        assert any(
            d["issue"] == labeled_slug and d["code"] == "missing_relation_target" and d["related"] == "8675309"
            for d in graph_payload["diagnostics"]
        ), graph_payload["diagnostics"]

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
