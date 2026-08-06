"""Black-box contract for body dependency declaration extraction (#3304)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "work-item-planning-body-dependency-declaration-extraction"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "body-dependency-declaration-extraction"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-body-dependency-declaration-extraction"
)
ASSERTIONS = (
    "a canonical `depends-on:` label resolves to its target through the live aw wi graph CLI, and the "
    "identical target id and a `blocked_by_dependency` lane appear verbatim in the aw wi plan artifact "
    "after a full normalize/reconcile/atomize/verify publish, while a body 'Depends on: <bare-slug>' line "
    "naming that same target resolves to zero dependencies on both surfaces -- proving canonical labels "
    "remain authoritative over body text and graph/planning-lane decoding share one parser",
    "a body 'This change depends on #503 shipping first, per the roadmap' sentence and a "
    "'Depends on: #<id> is the legacy placeholder format' syntax example both leave a fully valid graph "
    "with zero diagnostics and zero dependencies for each change, and the aw wi plan artifact reports "
    "both changes in the ready_now lane with an empty dependency list -- proving explanatory prose and "
    "syntax-example placeholders are read and rejected rather than accidentally matched",
    "a declaration-shaped '- Depends on #505 finishing first' body line drives the live aw wi graph CLI "
    "to valid=false with a missing_relation_target diagnostic naming exactly 505, and a single aw wi plan "
    "--stage normalize call on the identical fixture reports the byte-for-byte same diagnostic object in "
    "its on-disk plan artifact -- proving a declaration-shaped legacy body line creates a real dependency "
    "edge attempt through the exact same extractor the graph uses, not a second independent parser",
)

_EPIC_BODY = (
    "## Requirements\n\n- R1: Demonstrate dependency extraction.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `aw wi graph` | graph reports the expected structure. | - |\n"
)


def _change_body(extra: str = "") -> str:
    extra_block = f"{extra}\n\n" if extra else ""
    return (
        "## Problem\n\ndemo\n\n" + extra_block +
        "## Capability Alignment\n\nCapability: x\nCapability Gap: y\n"
        "Progress Evidence: z\n\n## Requirements\n\n- R1: trace dependency extraction.\n\n"
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


def _update(root: Path, wi_id: str, *args: str) -> dict:
    return final_json(run_aw(root, "wi", "update", wi_id, *args, "--json"))


def _validate(root: Path, wi_id: str) -> dict:
    result = final_json(run_aw(root, "wi", "validate", wi_id))
    assert result["passed"] is True, result
    return result


def _publish(root: Path, project: str = "demo") -> dict:
    """Drive normalize -> reconcile -> atomize -> verify, approving a HITL if one fires."""
    normalize = final_json(run_aw(root, "wi", "plan", "--project", project, "--stage", "normalize"))
    assert normalize["status"] == "continue", normalize
    root_id = normalize["root"]["id"]

    reconcile = final_json(
        run_aw(root, "wi", "plan", "--project", project, "--stage", "reconcile", "--root", root_id)
    )
    if reconcile.get("next", {}).get("kind") == "hitl":
        question = reconcile["hitl_question"]
        reconcile = final_json(
            run_aw(
                root,
                "wi",
                "plan-answer",
                "--payload",
                reconcile["next"]["payload_path"],
                "--question",
                question["id"],
                "--choice",
                "approve",
                "--json",
            )
        )
    assert reconcile["status"] == "continue", reconcile

    atomize = final_json(
        run_aw(root, "wi", "plan", "--project", project, "--stage", "atomize", "--root", root_id)
    )
    assert atomize["status"] == "continue", atomize
    verified = final_json(
        run_aw(root, "wi", "plan", "--project", project, "--stage", "verify", "--root", root_id)
    )
    assert verified["status"] == "done", verified
    return verified


def verify() -> list[str]:
    # Cluster 1: the canonical depends-on: label is authoritative and resolves
    # identically on both the graph and the plan/lane surface; a body
    # "Depends on: <bare-slug>" line naming the same target extracts nothing
    # (no bare-slug fallback for body-text dependency declarations).
    with project_fixture() as root:
        epic = _create_epic(root, "Dependency epic")
        target = create(root, "Dependency target", "change", "--epic", epic, "--body", _change_body())["slug"]
        _validate(root, target)

        labeled = create(root, "Label dependency", "change", "--epic", epic, "--body", _change_body())["slug"]
        _update(root, labeled, "--add-label", f"depends-on:{target}")
        _validate(root, labeled)

        bare_slug = create(
            root, "Body bare-slug non-extraction", "change", "--epic", epic, "--body",
            _change_body(f"Depends on: {target}"),
        )["slug"]
        _validate(root, bare_slug)

        graphed = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))
        assert graphed["valid"] is True, graphed
        assert graphed["diagnostics"] == [], graphed
        by_id = {c["id"]: c for c in graphed["changes"]}
        assert by_id[labeled]["dependencies"] == [target], by_id[labeled]
        assert by_id[bare_slug]["dependencies"] == [], by_id[bare_slug]

        verified = _publish(root)
        plan_path = Path(verified["plan"]["path"])
        plan = json.loads(plan_path.read_text(encoding="utf-8"))
        planned = {c["id"]: c for c in plan["changes"]}
        assert planned[labeled]["lane"] == "blocked_by_dependency", planned[labeled]
        assert planned[labeled]["dependencies"] == [target], planned[labeled]
        assert planned[bare_slug]["lane"] == "ready_now", planned[bare_slug]
        assert planned[bare_slug]["dependencies"] == [], planned[bare_slug]

    # Cluster 2: explanatory prose and a hash-shaped syntax-example placeholder
    # both extract nothing, on a graph that stays fully valid so the plan/lane
    # surface can be read directly (not just via a diagnostic proxy).
    with project_fixture() as root:
        epic2 = _create_epic(root, "Dependency epic 2")
        prose = create(
            root, "Explanatory prose mention", "change", "--epic", epic2, "--body",
            _change_body("This change depends on #503 shipping first, per the roadmap."),
        )["slug"]
        _validate(root, prose)

        placeholder = create(
            root, "Syntax example placeholder", "change", "--epic", epic2, "--body",
            _change_body("Depends on: #<id> is the legacy placeholder format."),
        )["slug"]
        _validate(root, placeholder)

        graphed2 = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))
        assert graphed2["valid"] is True, graphed2
        assert graphed2["diagnostics"] == [], graphed2
        by_id2 = {c["id"]: c for c in graphed2["changes"]}
        assert by_id2[prose]["dependencies"] == [], by_id2[prose]
        assert by_id2[placeholder]["dependencies"] == [], by_id2[placeholder]

        verified2 = _publish(root)
        plan_path2 = Path(verified2["plan"]["path"])
        plan2 = json.loads(plan_path2.read_text(encoding="utf-8"))
        planned2 = {c["id"]: c for c in plan2["changes"]}
        assert planned2[prose]["lane"] == "ready_now", planned2[prose]
        assert planned2[prose]["dependencies"] == [], planned2[prose]
        assert planned2[placeholder]["lane"] == "ready_now", planned2[placeholder]
        assert planned2[placeholder]["dependencies"] == [], planned2[placeholder]

    # Cluster 3: a declaration-shaped, hash-valued body line creates a real
    # dependency-edge attempt -- proven through a missing_relation_target
    # diagnostic that is byte-for-byte identical between wi graph and a single
    # wi plan --stage normalize call on the same fixture, showing both surfaces
    # decode through the exact same extractor.
    with project_fixture() as root:
        epic3 = _create_epic(root, "Dependency epic 3")
        declared = create(
            root, "Declaration shaped hash dependency", "change", "--epic", epic3, "--body",
            _change_body("- Depends on #505 finishing first"),
        )["slug"]
        _validate(root, declared)

        invalid = run_aw(root, "wi", "graph", "--project", "demo", "--json", expect_success=False)
        graph_payload = json.loads(invalid.stdout)
        assert graph_payload["valid"] is False, graph_payload
        assert graph_payload["action"] == "blocked", graph_payload
        (graph_diag,) = [
            d for d in graph_payload["diagnostics"]
            if d["issue"] == declared and d["code"] == "missing_relation_target"
        ]
        assert graph_diag["related"] == "505", graph_diag
        assert "invalid" in invalid.stderr, invalid.stderr

        plan_result = final_json(
            run_aw(root, "wi", "plan", "--project", "demo", "--stage", "normalize", "--json")
        )
        assert plan_result["status"] == "blocked", plan_result
        plan_path3 = Path(plan_result["plan"]["path"])
        plan3 = json.loads(plan_path3.read_text(encoding="utf-8"))
        assert plan3["diagnostics"] == graph_payload["diagnostics"], (plan3["diagnostics"], graph_payload["diagnostics"])

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
