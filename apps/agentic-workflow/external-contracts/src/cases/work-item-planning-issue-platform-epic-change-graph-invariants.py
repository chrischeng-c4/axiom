"""Black-box contract for issue-platform epic/change graph invariants (#3304)."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "work-item-planning-issue-platform-epic-change-graph-invariants"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "issue-platform-epic-change-graph-invariants"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-issue-platform-epic-change-graph-invariants"
)
ASSERTIONS = (
    "a fixture with one open epic and seven owned changes -- one inheriting the epic's own priority "
    "label, one declaring its own explicit priority, one declaring a depends-on relation, a closed "
    "duplicate target with its open duplicate-of change, and a closed original with its open sibling "
    "supersedes change -- reports through the live aw wi graph CLI as fully valid with zero "
    "diagnostics, correctly attributing every ownership, inherited-vs-explicit priority source, "
    "dependency, duplicate-of, and supersedes/superseded_by pairing by exact id, and leaves every "
    "on-disk tracker record byte-for-byte unmodified across the read-only projection",
    "an unowned open change and a separate cross-epic supersedes declaration each independently "
    "invalidate that same live graph projection, driving it to valid=false, action=blocked, and a "
    "non-zero process exit even though the full diagnostic JSON is still emitted on stdout -- proving "
    "deterministic ownership and sibling-supersession are enforced invariants that fail closed rather "
    "than incidental labels, so the all-valid projection above is not vacuous",
)

_EPIC_BODY = (
    "## Requirements\n\n- R1: Demonstrate graph projection.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `aw wi graph` | graph reports the expected structure. | - |\n"
)


def _change_body() -> str:
    return (
        "## Problem\n\ndemo\n\n## Capability Alignment\n\nCapability: x\nCapability Gap: y\n"
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


def _create_epic(root: Path, title: str) -> str:
    slug = create(root, title, "epic", "--priority", "p1", "--body", _EPIC_BODY)["slug"]
    _validate(root, slug)
    return slug


def _create_change(root: Path, title: str, epic: str, priority: str | None = None) -> str:
    args = ["--epic", epic]
    if priority:
        args += ["--priority", priority]
    slug = create(root, title, "change", *args, "--body", _change_body())["slug"]
    _validate(root, slug)
    return slug


def _workspace_slug(root: Path) -> str:
    resolved = str(root.resolve())
    collapsed = re.sub(r"[^a-zA-Z0-9]+", "-", resolved)
    return collapsed.strip("-").lower()


def _issue_path(root: Path, slug: str, state: str = "open") -> Path:
    return Path("/tmp/aw/workspaces") / _workspace_slug(root) / "issues" / state / f"{slug}.md"


def verify() -> list[str]:
    with project_fixture() as root:
        epic = _create_epic(root, "Rollup epic")

        c1 = _create_change(root, "Inherits priority", epic)
        c2 = _create_change(root, "Explicit priority", epic, priority="p2")
        c3 = _create_change(root, "Depends on c1", epic)
        _update(root, c3, "--add-label", f"depends-on:{c1}")

        c4 = _create_change(root, "Duplicate target", epic)
        run_aw(root, "wi", "close", c4, "--json")
        c5 = _create_change(root, "Duplicate of c4", epic)
        _update(root, c5, "--add-label", f"duplicate-of:{c4}")

        c7 = _create_change(root, "Superseded original", epic)
        run_aw(root, "wi", "close", c7, "--json")
        c6 = _create_change(root, "Supersedes c7", epic)
        _update(root, c6, "--add-label", f"supersedes:{c7}")

        # Cluster 1: fully valid graph -- ownership, priority inheritance vs.
        # explicit priority, dependency, duplicate, and sibling supersession,
        # plus a no-mutation check on the on-disk tracker record.
        c1_path = _issue_path(root, c1)
        before = c1_path.read_text(encoding="utf-8")

        graphed = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))
        assert graphed["valid"] is True, graphed
        assert graphed["diagnostics"] == [], graphed

        after = c1_path.read_text(encoding="utf-8")
        assert after == before, (before, after)

        (epic_entry,) = [e for e in graphed["epics"] if e["id"] == epic]
        assert epic_entry["state"] == "open", epic_entry
        assert epic_entry["priority"] == "p1", epic_entry
        assert set(epic_entry["children"]) == {c1, c2, c3, c4, c5, c6, c7}, epic_entry

        by_id = {c["id"]: c for c in graphed["changes"]}
        assert by_id[c1]["state"] == "open", by_id[c1]
        assert by_id[c1]["parent"] == epic, by_id[c1]
        assert by_id[c1]["priority"] == {
            "value": "p1",
            "source": "inherited",
            "inherited_from": epic,
        }, by_id[c1]

        assert by_id[c2]["priority"]["value"] == "p2", by_id[c2]
        assert by_id[c2]["priority"]["source"] == "explicit", by_id[c2]
        assert by_id[c2]["priority"].get("inherited_from") is None, by_id[c2]

        assert by_id[c3]["dependencies"] == [c1], by_id[c3]

        assert by_id[c4]["state"] == "closed", by_id[c4]
        assert by_id[c5]["duplicate_of"] == c4, by_id[c5]

        assert by_id[c6]["supersedes"] == [c7], by_id[c6]
        assert by_id[c7]["superseded_by"] == [c6], by_id[c7]
        assert by_id[c7]["state"] == "closed", by_id[c7]

        # Cluster 2: two independent ways to fail closed, driven in one call.
        c8 = create(root, "Unowned change", "change", "--body", _change_body())["slug"]
        _validate(root, c8)

        epic2 = _create_epic(root, "Second epic")
        c9 = _create_change(root, "Other-epic original", epic2)
        run_aw(root, "wi", "close", c9, "--json")
        c10 = _create_change(root, "Cross-epic supersedes", epic)
        _update(root, c10, "--add-label", f"supersedes:{c9}")

        invalid = run_aw(root, "wi", "graph", "--project", "demo", "--json", expect_success=False)
        payload = json.loads(invalid.stdout)
        assert payload["valid"] is False, payload
        assert payload["action"] == "blocked", payload
        codes = {d["code"] for d in payload["diagnostics"]}
        assert "unowned_change" in codes, payload
        assert "supersession_not_sibling" in codes, payload

        (unowned,) = [d for d in payload["diagnostics"] if d["issue"] == c8]
        assert unowned["next_command"] == f"aw wi show {c8}", unowned

        (cross_epic,) = [d for d in payload["diagnostics"] if d["code"] == "supersession_not_sibling"]
        assert cross_epic["issue"] == c10 and cross_epic["related"] == c9, cross_epic
        assert "invalid" in invalid.stderr, invalid.stderr

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
