"""Python EC implementation for the canonical four-type WI taxonomy."""

from __future__ import annotations

import sys
from pathlib import Path


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw, show, verify_case


CASE_ID = "work-item-four-type-taxonomy"


def verify() -> list[str]:
    with project_fixture() as root:
        help_output = run_aw(root, "wi", "create", "--help").stdout
        assert "[possible values: epic, change, spike, report]" in help_output
        assert "Closed enum: epic | change | spike | report" in help_output

        for work_item_type in ("epic", "change", "spike", "report"):
            extra = ("--priority", "p1") if work_item_type == "epic" else ()
            created = create(root, work_item_type, work_item_type, *extra)
            issue = show(root, created["slug"])
            assert issue["type"] == work_item_type
            assert f"type:{work_item_type}" in issue["labels"]

        graph = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))
        assert graph["valid"] is True
        assert len(graph["changes"]) == 1
        assert graph["diagnostics"] == []
    return [
        "epic, change, spike, and report round-trip through the real CLI",
        "only change enters the executable project graph",
    ]


if __name__ == "__main__":
    verify_case(CASE_ID, verify)
