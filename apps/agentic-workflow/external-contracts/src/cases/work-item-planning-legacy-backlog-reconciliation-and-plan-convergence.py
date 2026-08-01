"""Black-box contract for legacy backlog reconciliation and plan convergence (#3304)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "work-item-planning-legacy-backlog-reconciliation-and-plan-convergence"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "legacy-backlog-reconciliation-and-plan-convergence"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-legacy-backlog-reconciliation-and-plan-convergence"
)
ASSERTIONS = (
    "declaring ownership solely through a legacy `Parent: <epic>.` body line (no --epic flag, no "
    "epic: label) resolves the change's graph parent to that epic and is graph-valid; normalize "
    "proposes exactly one deterministic, non-HITL canonicalize mutation that adds the epic: label "
    "from that declaration; after applying it, a second normalize reports no eligible mutations and "
    "reconcile no longer requires a human decision -- proving the legacy Parent: import converges to "
    "the canonical labeled form without ever needing a human decision",
    "a genuinely unowned change (no parent declared at all) fails aw wi graph closed with an "
    "unowned_change diagnostic; atomize blocks and reroutes to reconcile instead of ever publishing "
    "an auto-invented bootstrap epic; reconcile's HITL choices are exactly the real existing epics "
    "plus revise, with no proposal:-prefixed id ever offered; and choosing the real epic assigns that "
    "exact epic label without creating any new epic -- proving unowned legacy changes require an "
    "explicit existing owner or a tracker revision, never an invented bootstrap epic",
    "an epic's ## Requirements are the only atomization input -- Scope, Out of Scope, and Acceptance "
    "Criteria prose never appear as separate requirement entries -- and a closed change whose title "
    "textually matches one of two requirements marks only that matching requirement covered, leaving "
    "the textually-unrelated requirement a genuine gap; atomize then proposes exactly one new change "
    "covering only the uncovered requirement -- proving closed changes remain valid, "
    "requirement-specific coverage rather than either blanket-covering every requirement or being "
    "ignored because they are closed",
    "once every requirement of an epic is covered by closed changes, a brand new planning root (a "
    "fresh normalize with no prior --root) proposes zero new changes and zero new epics, aw wi graph "
    "reports the graph strictly valid with zero diagnostics, and two successive verify calls on that "
    "fresh root reproduce byte-identical plan digests -- proving strict-graph reconciliation over an "
    "already-fulfilled legacy graph is genuinely digest-stable and duplicate-free rather than "
    "re-proposing redundant work on every replan",
)

_REFERENCE_CONTEXT = (
    "\n\n## Scope\n\n### In Scope\n- a\n\n### Out of Scope\n- b\n\n"
    "## Acceptance Criteria\n\n- AC1: c\n\n"
    "## Reference Context\n\n### Related Specs\n| Spec | Relevance |\n|------|-----------|\n"
    "| x.md | high |\n\n### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
    "|---------|--------|---------------|\n| x | modify | x.md |\n"
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


def _epic_body(requirements_md: str, verification_rows: str) -> str:
    return (
        "## Problem\n\ndemo\n\n## Capability Alignment\n\nCapability: x\nCapability Gap: y\n"
        f"Progress Evidence: z\n\n## Requirements\n\n{requirements_md}\n\n## Verification Inventory\n\n"
        "| Requirement | Gate | Oracle | Depends On |\n|-------------|------|--------|------------|\n"
        f"{verification_rows}" + _REFERENCE_CONTEXT
    )


def _change_body(requirement_text: str, extra_top: str = "") -> str:
    return (
        "## Problem\n\n" + extra_top + "demo\n\n"
        "## Capability Alignment\n\nCapability: x\nCapability Gap: y\nProgress Evidence: z\n\n"
        f"## Requirements\n\n- R1: {requirement_text}\n" + _REFERENCE_CONTEXT
    )


def _drive_to_apply(root: Path, root_id: str | None = None) -> tuple[str, str]:
    """Normalize (or resume at root_id) -> reconcile (approve a HITL if one
    fires) -> atomize -> independent agent review acceptance -> human
    plan-answer confirmation. Returns (root_id, evidence_path_for_apply)."""
    if root_id is None:
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
        summary="Independent review confirmed scope and coverage.",
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
    return root_id, human_answered["next"]["payload_path"]


def verify() -> list[str]:
    # Assertion 1: a legacy `Parent: <epic>.` body-only declaration resolves
    # ownership, converges via exactly one deterministic non-HITL mutation,
    # and never requires a human decision to reach a clean, zero-mutation
    # steady state.
    with project_fixture() as root:
        epic_slug = create(
            root, "Legacy import epic", "epic", "--priority", "p1",
            "--body", _epic_body("- R1: Ship the legacy-imported widget.", "| R1 | `true` | ok. | - |\n"),
        )["slug"]
        final_json(run_aw(root, "wi", "validate", epic_slug))

        body = _change_body("trace legacy parent import.", extra_top=f"Parent: {epic_slug}.\n\n")
        change_slug = create(root, "Legacy imported change", "change", "--priority", "p2", "--body", body)["slug"]
        final_json(run_aw(root, "wi", "validate", change_slug))

        graphed = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))
        graph_change = next(c for c in graphed["changes"] if c["id"] == change_slug)
        assert graph_change["parent"] == epic_slug, graph_change
        assert graphed["valid"] is True, graphed

        normalize1 = final_json(run_aw(root, "wi", "plan", "--project", "demo", "--stage", "normalize", "--json"))
        manifest1 = json.loads(Path(normalize1["next"]["payload_path"]).read_text(encoding="utf-8"))
        mine1 = [m for m in manifest1["mutations"] if m["target"] == change_slug]
        assert len(mine1) == 1, mine1
        assert mine1[0]["certainty"] == "deterministic", mine1
        assert mine1[0]["requires_hitl"] is False, mine1
        assert mine1[0]["decision_source"] == "explicit_metadata", mine1
        assert mine1[0]["add_labels"] == [f"epic:{epic_slug}"], mine1

        applied1 = final_json(
            run_aw(root, "wi", "plan-apply", "--evidence-file", normalize1["next"]["payload_path"], "--json")
        )
        assert applied1["action"] == "applied", applied1

        normalize2 = final_json(run_aw(root, "wi", "plan", "--project", "demo", "--stage", "normalize", "--json"))
        assert normalize2["next"]["reason"] == "current planning stage has no eligible mutations", normalize2

        reconcile2 = final_json(run_aw(root, "wi", "plan", "--project", "demo", "--stage", "reconcile", "--json"))
        assert reconcile2["completion"]["requires_hitl"] is False, reconcile2

    # Assertion 2: a genuinely unowned change is never grouped under an
    # invented bootstrap epic; atomize blocks and reroutes to reconcile with
    # the owner-decision question; the offered choices are exactly the real
    # epics plus revise, and applying an owner choice never creates a new
    # epic.
    with project_fixture() as root:
        epic_slug = create(
            root, "Real owner epic", "epic", "--priority", "p1",
            "--body", _epic_body("- R1: Ship the rescued widget.", "| R1 | `true` | ok. | - |\n"),
        )["slug"]
        final_json(run_aw(root, "wi", "validate", epic_slug))

        orphan_body = _change_body("an unowned legacy change with no parent declared at all.")
        orphan_slug = create(root, "Orphan legacy change", "change", "--priority", "p2", "--body", orphan_body)[
            "slug"
        ]
        final_json(run_aw(root, "wi", "validate", orphan_slug))

        before_graph = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json", expect_success=False))
        assert before_graph["valid"] is False, before_graph
        assert any(d["code"] == "unowned_change" for d in before_graph["diagnostics"]), before_graph
        before_epics = {e["id"] for e in before_graph["epics"]}

        atomize = final_json(run_aw(root, "wi", "plan", "--project", "demo", "--stage", "atomize", "--json"))
        assert atomize["action"] == "blocked", atomize
        assert atomize["next"]["command"] == (
            f"aw wi plan --project demo --stage reconcile --root {atomize['root']['id']} --json"
        ), atomize

        reconcile = final_json(
            run_aw(
                root, "wi", "plan", "--project", "demo", "--stage", "reconcile",
                "--root", atomize["root"]["id"], "--json",
            )
        )
        assert reconcile["next"]["reason"] == "select an explicit epic owner for the next unowned change", reconcile
        assert reconcile["hitl_question"]["question"] == (
            "Approve the inferred reconciliation mutations for existing work items?"
        ), reconcile
        choice_ids = {c["id"] for c in reconcile["hitl_question"]["choices"]}
        assert choice_ids == {epic_slug, "revise"}, choice_ids
        assert not any(cid.startswith("proposal:") for cid in choice_ids), choice_ids

        assigned = final_json(
            run_aw(
                root, "wi", "plan-answer", "--payload", reconcile["next"]["payload_path"],
                "--question", reconcile["hitl_question"]["id"], "--choice", epic_slug, "--json",
            )
        )
        assert assigned["next"]["command"].startswith("aw wi plan-apply"), assigned

        applied = final_json(run_aw(root, "wi", "plan-apply", "--evidence-file", assigned["next"]["payload_path"], "--json"))
        assert applied["action"] == "applied", applied

        after_epics = {e["id"] for e in final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))["epics"]}
        assert after_epics == before_epics, (before_epics, after_epics)

        shown = final_json(run_aw(root, "wi", "show", orphan_slug, "--json"))
        issue = shown.get("issue", shown)
        assert f"epic:{epic_slug}" in issue["labels"], issue["labels"]

    # Assertion 3: only ## Requirements drives atomization -- Scope/AC prose
    # never becomes its own requirement -- and a closed change covers only
    # the requirement its title actually matches, leaving an unrelated
    # sibling requirement a genuine gap that atomize alone proposes for.
    with project_fixture() as root:
        r1_text = "Migrate the archived telemetry pipeline to durable storage."
        r2_text = "Rotate the customer signing certificate before expiry."
        requirements_md = f"- {r1_text}\n- {r2_text}"
        verification_rows = "| R1 | `true` | ok. | - |\n| R2 | `true` | ok. | - |\n"
        epic_slug = create(
            root, "Coverage epic", "epic", "--priority", "p1",
            "--body", _epic_body(requirements_md, verification_rows),
        )["slug"]
        final_json(run_aw(root, "wi", "validate", epic_slug))

        delivered_slug = create(
            root, r1_text, "change", "--priority", "p2", "--epic", epic_slug, "--body", _change_body(r1_text),
        )["slug"]
        final_json(run_aw(root, "wi", "validate", delivered_slug))
        final_json(run_aw(root, "wi", "close", delivered_slug, "--json"))

        atomize = final_json(run_aw(root, "wi", "plan", "--project", "demo", "--stage", "atomize", "--json"))
        plan = json.loads(Path(atomize["plan"]["path"]).read_text(encoding="utf-8"))
        requirements = plan["epics"][0]["requirements"]
        assert len(requirements) == 2, requirements
        req_by_text = {r["text"]: r for r in requirements}
        assert req_by_text[r1_text]["status"] == "covered", req_by_text[r1_text]
        assert req_by_text[r1_text]["covered_by"] == [delivered_slug], req_by_text[r1_text]
        assert req_by_text[r2_text]["status"] == "gap", req_by_text[r2_text]
        assert req_by_text[r2_text]["covered_by"] == [], req_by_text[r2_text]

        assert atomize["plan"]["proposed_change_count"] == 1, atomize
        proposed = plan["proposed_changes"]
        assert len(proposed) == 1, proposed
        assert proposed[0]["covers"] == [f"{epic_slug}:requirement-2"], proposed
        assert proposed[0]["title"] == r2_text, proposed

    # Assertion 4: once every requirement is covered by closed changes, a
    # brand new planning root proposes nothing new, aw wi graph stays
    # strictly valid, and two verify calls on the fresh root are
    # digest-identical -- proving convergence is duplicate-free, not merely
    # stable within a single already-open root.
    with project_fixture() as root:
        r1_text = "Decommission the archived ingest worker fleet."
        r2_text = "Renew the expiring domain registration."
        requirements_md = f"- {r1_text}\n- {r2_text}"
        verification_rows = "| R1 | `true` | ok. | - |\n| R2 | `true` | ok. | - |\n"
        epic_slug = create(
            root, "Converged epic", "epic", "--priority", "p1",
            "--body", _epic_body(requirements_md, verification_rows),
        )["slug"]
        final_json(run_aw(root, "wi", "validate", epic_slug))

        delivered_slug = create(
            root, r1_text, "change", "--priority", "p2", "--epic", epic_slug, "--body", _change_body(r1_text),
        )["slug"]
        final_json(run_aw(root, "wi", "validate", delivered_slug))
        final_json(run_aw(root, "wi", "close", delivered_slug, "--json"))

        first_root_id, evidence = _drive_to_apply(root)
        applied = final_json(run_aw(root, "wi", "plan-apply", "--evidence-file", evidence, "--json"))
        assert applied["action"] == "applied", applied

        graphed = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))
        new_change = next(c for c in graphed["changes"] if c["id"] != delivered_slug)
        final_json(run_aw(root, "wi", "close", new_change["id"], "--json"))

        normalize2 = final_json(run_aw(root, "wi", "plan", "--project", "demo", "--stage", "normalize", "--json"))
        second_root_id = normalize2["root"]["id"]
        assert second_root_id != first_root_id, (first_root_id, second_root_id)

        atomize2 = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "atomize", "--root", second_root_id, "--json")
        )
        assert atomize2["plan"]["proposed_change_count"] == 0, atomize2
        assert atomize2["plan"]["proposed_epic_count"] == 0, atomize2

        strict_graph = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))
        assert strict_graph["valid"] is True, strict_graph
        assert strict_graph["diagnostics"] == [], strict_graph

        verify_a = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "verify", "--root", second_root_id, "--json")
        )
        assert verify_a["status"] == "done", verify_a
        assert verify_a["completion"]["workflow_complete"] is True, verify_a

        verify_b = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "verify", "--root", second_root_id, "--json")
        )
        assert verify_b["plan"]["digest"] == verify_a["plan"]["digest"], (verify_a, verify_b)

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
