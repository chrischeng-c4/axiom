"""Black-box contract for parent-ownership reference extraction (#3304)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "work-item-planning-parent-ownership-reference-extraction"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "parent-ownership-reference-extraction"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-parent-ownership-reference-extraction"
)
ASSERTIONS = (
    "a bare epic slug declared through a body 'Parent epic:' prose line, a canonical `epic:` label, a "
    "`parent-epic:` label, and a `parent:` label all resolve, through the live aw wi graph CLI, to the "
    "exact same owning epic with zero diagnostics -- proving body and label parent prefixes share one "
    "extractor and that bare id/slug compatibility holds on every surface",
    "a body 'Parent epic: #404 ... Depends on #405' line and a label `epic:#406 (owner) #407` each "
    "independently surface a missing_epic_parent diagnostic naming only the first hash reference "
    "(404 and 406 respectively) and never the trailing prose or dependency-shaped mention that follows "
    "it, and a bare owner/repo/id body path resolves the same diagnostic to just its trailing digit "
    "segment (408) -- proving first-hash-wins and owner/repo/path compatibility are real extraction "
    "outcomes rather than coincidental string matches",
    "a body paragraph that only documents the `Parent epic:`, `Parent WI:`, and `Parent:` prefixes in "
    "backticks, plus an empty-valued `parent:` label on the same change, produce no missing_epic_parent "
    "diagnostic at all while the change is still correctly reported unowned through the unrelated "
    "unowned_change diagnostic -- proving syntax documentation is read and rejected rather than silently "
    "skipped, so it creates no phantom parent declaration, and the four resolving declarations above are "
    "not vacuous",
)

_EPIC_BODY = (
    "## Requirements\n\n- R1: Demonstrate parent-reference extraction.\n\n"
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
        "Progress Evidence: z\n\n## Requirements\n\n- R1: trace graph projection.\n\n"
        "## Scope\n\n### In Scope\n- a\n\n### Out of Scope\n- b\n\n"
        "## Acceptance Criteria\n\n- AC1: c\n\n## Reference Context\n\n### Related Specs\n"
        "| Spec | Relevance |\n|------|-----------|\n| x.md | high |\n\n"
        "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
        "|---------|--------|---------------|\n| x | modify | x.md |\n"
    )


def _update(root: Path, wi_id: str, *args: str) -> dict:
    return final_json(run_aw(root, "wi", "update", wi_id, *args, "--json"))


def _validate(root: Path, wi_id: str) -> dict:
    result = final_json(run_aw(root, "wi", "validate", wi_id))
    assert result["passed"] is True, result
    return result


def _diagnostics(payload: dict, issue_id: str, code: str) -> list[dict]:
    return [d for d in payload["diagnostics"] if d["issue"] == issue_id and d["code"] == code]


def verify() -> list[str]:
    with project_fixture() as root:
        epic = create(root, "Parent target epic", "epic", "--priority", "p1", "--body", _EPIC_BODY)["slug"]
        _validate(root, epic)

        # Cluster 1: body and label parent prefixes share one extractor, and
        # bare id/slug compatibility resolves correctly on every surface.
        a = create(
            root, "Body parent epic prefix", "change", "--body", _change_body(f"Parent epic: {epic}")
        )["slug"]
        _validate(root, a)

        b = create(root, "Canonical epic label", "change", "--epic", epic, "--body", _change_body())["slug"]
        _validate(root, b)

        c = create(root, "parent-epic label", "change", "--body", _change_body())["slug"]
        _update(root, c, "--add-label", f"parent-epic:{epic}")
        _validate(root, c)

        d = create(root, "parent label", "change", "--body", _change_body())["slug"]
        _update(root, d, "--add-label", f"parent:{epic}")
        _validate(root, d)

        graphed = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))
        assert graphed["valid"] is True, graphed
        assert graphed["diagnostics"] == [], graphed

        by_id = {ch["id"]: ch for ch in graphed["changes"]}
        for wi_id in (a, b, c, d):
            assert by_id[wi_id]["parent"] == epic, (wi_id, by_id[wi_id])

        # Cluster 2: first #<digits> wins over trailing prose/dependency
        # mentions; owner/repo/id path compatibility; documentation-style
        # prose declares nothing (no phantom declaration), still correctly
        # bucketed as unowned through the unrelated diagnostic.
        f = create(
            root,
            "Body first-hash-wins",
            "change",
            "--body",
            _change_body(
                "Parent epic: #404 (Operations baseline row 5). "
                "Depends on #405, which most teams need too."
            ),
        )["slug"]
        _validate(root, f)

        g = create(root, "Label first-hash-wins", "change", "--body", _change_body())["slug"]
        _update(root, g, "--add-label", "epic:#406 (owner) #407")
        _validate(root, g)

        h = create(
            root,
            "Owner repo path compatibility",
            "change",
            "--body",
            _change_body("Parent: some/owner/epic-repo/408"),
        )["slug"]
        _validate(root, h)

        i = create(
            root,
            "Doc prose declares nothing",
            "change",
            "--body",
            _change_body(
                "`Parent epic:`, `Parent WI:`, and `Parent:` are legacy migration inputs "
                "documented here for reference."
            ),
        )["slug"]
        _update(root, i, "--add-label", "parent:")
        _validate(root, i)

        invalid = run_aw(root, "wi", "graph", "--project", "demo", "--json", expect_success=False)
        payload = json.loads(invalid.stdout)
        assert payload["valid"] is False, payload
        assert payload["action"] == "blocked", payload

        (f_diag,) = _diagnostics(payload, f, "missing_epic_parent")
        assert f_diag["related"] == "404", f_diag

        (g_diag,) = _diagnostics(payload, g, "missing_epic_parent")
        assert g_diag["related"] == "406", g_diag

        (h_diag,) = _diagnostics(payload, h, "missing_epic_parent")
        assert h_diag["related"] == "408", h_diag

        assert _diagnostics(payload, i, "missing_epic_parent") == [], payload["diagnostics"]
        assert len(_diagnostics(payload, i, "unowned_change")) == 1, payload["diagnostics"]

        by_id2 = {ch["id"]: ch for ch in payload["changes"]}
        for wi_id in (a, b, c, d):
            assert by_id2[wi_id]["parent"] == epic, (wi_id, by_id2[wi_id])

        assert "invalid" in invalid.stderr, invalid.stderr

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
