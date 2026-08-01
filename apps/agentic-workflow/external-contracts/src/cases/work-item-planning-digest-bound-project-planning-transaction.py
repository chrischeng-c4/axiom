"""Black-box contract for the digest-bound project planning transaction (#3304)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "work-item-planning-digest-bound-project-planning-transaction"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "digest-bound-project-planning-transaction"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-digest-bound-project-planning-transaction"
)
ASSERTIONS = (
    "externally retitling a reviewed epic after independent review accepts its plan, but before plan-apply "
    "runs, makes plan-apply fail closed naming the exact drifted issue and writes nothing at all -- proving "
    "an accepted transaction is bound to the precise tracker snapshot it was reviewed against rather than "
    "whatever the tracker happens to contain at apply time",
    "corrupting only the source_digest field of an otherwise-untouched accepted decision file makes "
    "plan-apply reject it as stale before any tracker read, while the exact same decision file with its "
    "genuine digest restored applies successfully and creates precisely the reviewed change set -- proving "
    "the transaction binds the ordered mutation manifest by digest rather than trusting decision-file shape "
    "alone, with a positive control ruling out a vacuously-always-failing check",
    "a change explicitly labeled duplicate-of another change is surfaced by the reconciled plan as a "
    "duplicate recommendation naming its canonical original, and a change explicitly labeled as "
    "superseded-by a sibling replacement is surfaced by aw wi graph as a symmetric supersedes/superseded-by "
    "relation -- while both the duplicate and the superseded change remain open and present in the tracker "
    "throughout, proving the transaction records these as recommendations without ever deleting or closing "
    "the issues it recommends against",
    "declaring one change supersedes another change that lives under a different epic fails aw wi graph "
    "closed with a supersession_not_sibling diagnostic naming both changes -- a positive control proving the "
    "supersession relation is genuinely validated rather than accepted unconditionally",
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
    "## Problem\n\ndemo\n\n## Capability Alignment\n\nCapability: x\nCapability Gap: y\nProgress Evidence: z\n\n"
    "## Requirements\n\n- R1: Ship the canonical widget.\n\n## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n|-------------|------|--------|------------|\n"
    "| R1 | `true` | ok. | - |\n" + _REFERENCE_CONTEXT
)
_OTHER_EPIC_BODY = (
    "## Problem\n\ndemo\n\n## Capability Alignment\n\nCapability: x\nCapability Gap: y\nProgress Evidence: z\n\n"
    "## Requirements\n\n- R1: Ship a different widget.\n\n## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n|-------------|------|--------|------------|\n"
    "| R1 | `true` | ok. | - |\n" + _REFERENCE_CONTEXT
)
_CHANGE_BODY = (
    "## Problem\n\ndemo\n\n## Capability Alignment\n\nCapability: x\nCapability Gap: y\n"
    "Progress Evidence: z\n\n## Requirements\n\n- R1: x.\n" + _REFERENCE_CONTEXT
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
    # Assertion 1: an accepted transaction is bound to the exact reviewed
    # tracker snapshot. Retitling the epic after acceptance but before apply
    # must reject the entire apply before any mutation.
    with project_fixture() as root:
        epic_slug = create(root, "Drift epic", "epic", "--priority", "p1", "--body", _LINKED_EPIC_BODY)["slug"]
        final_json(run_aw(root, "wi", "validate", epic_slug))
        evidence_file = _drive_to_apply(root, epic_slug)

        final_json(run_aw(root, "wi", "update", epic_slug, "--title", "Externally retitled after review", "--json"))

        count_before = len(final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))["changes"])
        drifted = run_aw(root, "wi", "plan-apply", "--evidence-file", evidence_file, "--json", expect_success=False)
        assert "tracker drift" in drifted.stderr and epic_slug in drifted.stderr, drifted.stderr
        count_after = len(final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))["changes"])
        assert count_after == count_before, "tracker drift must block every mutation"

    # Assertion 2: the review digest binds the ordered mutation manifest. A
    # decision file with a corrupted source_digest is rejected before any
    # tracker read; the same evidence with its genuine digest restored still
    # applies -- proving the check is real, not vacuously always-failing.
    with project_fixture() as root:
        epic_slug2 = create(root, "Digest epic", "epic", "--priority", "p1", "--body", _LINKED_EPIC_BODY)["slug"]
        final_json(run_aw(root, "wi", "validate", epic_slug2))
        evidence_file2 = _drive_to_apply(root, epic_slug2)

        evidence_path = Path(evidence_file2)
        decision = json.loads(evidence_path.read_text(encoding="utf-8"))
        tampered = dict(decision)
        tampered["source_digest"] = "sha256:" + ("0" * 64)
        tampered_path = evidence_path.with_name(evidence_path.stem + ".tampered.json")
        tampered_path.write_text(json.dumps(tampered), encoding="utf-8")

        count_before = len(final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))["changes"])
        forged = run_aw(root, "wi", "plan-apply", "--evidence-file", str(tampered_path), "--json", expect_success=False)
        assert "stale" in (forged.stdout + forged.stderr).lower(), forged.stderr
        count_after_forged = len(final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))["changes"])
        assert count_after_forged == count_before, "forged digest must not mutate anything"

        genuine = final_json(run_aw(root, "wi", "plan-apply", "--evidence-file", evidence_file2, "--json"))
        assert genuine["action"] == "applied", genuine
        count_after_genuine = len(final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))["changes"])
        assert count_after_genuine == count_before + 2, "genuine digest must apply the reviewed changes"

    # Assertions 3 and 4: duplicate-of and supersedes/superseded-by relations
    # are recorded as recommendations and never trigger deletion or closure.
    with project_fixture() as root:
        epic_slug3 = create(root, "Retention epic", "epic", "--priority", "p1", "--body", _SIMPLE_EPIC_BODY)["slug"]
        final_json(run_aw(root, "wi", "validate", epic_slug3))

        canonical = create(root, "Ship the canonical widget.", "change", "--epic", epic_slug3, "--body", _CHANGE_BODY)[
            "slug"
        ]
        dup = create(
            root, "Ship the canonical widget again by mistake.", "change", "--epic", epic_slug3, "--body", _CHANGE_BODY
        )["slug"]
        original = create(root, "Old approach to the widget.", "change", "--epic", epic_slug3, "--body", _CHANGE_BODY)[
            "slug"
        ]
        replacement = create(
            root, "New approach to the widget.", "change", "--epic", epic_slug3, "--body", _CHANGE_BODY
        )["slug"]
        for slug in (canonical, dup, original, replacement):
            final_json(run_aw(root, "wi", "validate", slug))

        final_json(run_aw(root, "wi", "update", dup, "--add-label", f"duplicate-of:{canonical}", "--json"))
        final_json(run_aw(root, "wi", "update", replacement, "--add-label", f"supersedes:{original}", "--json"))
        final_json(run_aw(root, "wi", "update", original, "--add-label", f"superseded-by:{replacement}", "--json"))

        normalize = final_json(run_aw(root, "wi", "plan", "--project", "demo", "--stage", "normalize", "--json"))
        root_id = normalize["root"]["id"]
        reconcile = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "reconcile", "--root", root_id, "--json")
        )
        plan = json.loads(Path(reconcile["plan"]["path"]).read_text(encoding="utf-8"))
        dup_entry = next(c for c in plan["changes"] if c["id"] == dup)
        assert dup_entry["lane"] == "duplicate", dup_entry
        assert dup_entry["duplicate_of"] == canonical, dup_entry

        graphed = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))
        graph_by_id = {c["id"]: c for c in graphed["changes"]}
        assert graph_by_id[original]["superseded_by"] == [replacement], graph_by_id[original]
        assert graph_by_id[replacement]["supersedes"] == [original], graph_by_id[replacement]

        for slug in (canonical, dup, original, replacement):
            assert slug in graph_by_id, f"{slug} must remain present in the graph"
            shown = final_json(run_aw(root, "wi", "show", slug, "--json"))
            issue = shown.get("issue", shown)
            assert issue.get("state") == "open", (slug, issue.get("state"))

    # Negative control: a supersedes relation crossing an epic boundary fails
    # the graph closed, proving the relation is genuinely validated.
    with project_fixture() as root:
        epic_a = create(root, "Cross epic A", "epic", "--priority", "p1", "--body", _SIMPLE_EPIC_BODY)["slug"]
        epic_b = create(root, "Cross epic B", "epic", "--priority", "p1", "--body", _OTHER_EPIC_BODY)["slug"]
        final_json(run_aw(root, "wi", "validate", epic_a))
        final_json(run_aw(root, "wi", "validate", epic_b))
        change_a = create(root, "Widget in epic A.", "change", "--epic", epic_a, "--body", _CHANGE_BODY)["slug"]
        change_b = create(root, "Widget in epic B.", "change", "--epic", epic_b, "--body", _CHANGE_BODY)["slug"]
        final_json(run_aw(root, "wi", "validate", change_a))
        final_json(run_aw(root, "wi", "validate", change_b))
        final_json(run_aw(root, "wi", "update", change_b, "--add-label", f"supersedes:{change_a}", "--json"))

        invalid = run_aw(root, "wi", "graph", "--project", "demo", "--json", expect_success=False)
        graph_payload = json.loads(invalid.stdout)
        assert graph_payload["valid"] is False, graph_payload
        assert any(
            d["code"] == "supersession_not_sibling" and d["issue"] == change_b and d["related"] == change_a
            for d in graph_payload["diagnostics"]
        ), graph_payload["diagnostics"]

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
