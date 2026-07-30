"""Python EC implementation for typed Report triage."""

from __future__ import annotations

import sys
from pathlib import Path


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw, show, verify_case


CASE_ID = "work-item-report-triage"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "terminology-first-four-type-wi-taxonomy"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-report-triage"
)
ASSERTIONS = (
    "Report type is immutable",
    "accepted triage spawns a linked delivery Change and closes the Report",
    "duplicate triage closes without spawning delivery work",
)


def verify() -> list[str]:
    with project_fixture() as root:
        epic = create(root, "Demo delivery", "epic", "--priority", "p1")
        report = create(root, "Runner exits silently", "report")
        slug = report["slug"]

        immutable = run_aw(
            root,
            "wi",
            "update",
            slug,
            "--add-label",
            "type:change",
            expect_success=False,
        )
        assert "type is immutable" in immutable.stderr

        terminal = final_json(
            run_aw(
                root,
                "wi",
                "triage",
                slug,
                "--verdict",
                "accepted",
                "--epic",
                epic["slug"],
            )
        )
        assert terminal["terminal_state"] == "accepted"
        assert terminal["spawned"]["type"] == "change"
        spawned = show(root, terminal["spawned"]["id"])
        assert spawned["type"] == "change"
        assert f"epic:{epic['slug']}" in spawned["labels"]
        closed_report = show(root, slug)
        assert closed_report["type"] == "report"
        assert closed_report["state"] == "closed"
        assert "Verdict: accepted" in closed_report["body"]

        duplicate = create(root, "Duplicate report", "report")
        terminal = final_json(
            run_aw(
                root,
                "wi",
                "triage",
                duplicate["slug"],
                "--verdict",
                "duplicate",
                "--reason",
                "Already tracked.",
            )
        )
        assert terminal["terminal_state"] == "duplicate"
        assert terminal["spawned"] is None
    return list(ASSERTIONS)


if __name__ == "__main__":
    verify_case(CASE_ID, verify)
