"""Python EC implementation for Spike terminal convergence."""

from __future__ import annotations

import sys
from pathlib import Path


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw, show, verify_case


CASE_ID = "work-item-spike-terminal"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "terminology-first-four-type-wi-taxonomy"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-spike-terminal"
)
ASSERTIONS = (
    "Spike roots cannot enter product EC/TD/CB work",
    "a decision requires spawned WIs or explicit no-action",
    "expired Spikes terminate as gave_up",
)


def verify() -> list[str]:
    with project_fixture() as root:
        decided = create(root, "Choose retry policy", "spike")
        slug = decided["slug"]
        goal = final_json(run_aw(root, "goal", "wi", slug))
        assert goal["action"] == "blocked"
        assert goal["next"]["command"].startswith("aw wi spike resolve")
        assert "artifact_quality_profile" not in goal

        missing_exit = run_aw(
            root,
            "wi",
            "spike",
            "resolve",
            slug,
            "--decision",
            "Use bounded exponential backoff.",
            expect_success=False,
        )
        assert "--spawned-wi" in missing_exit.stderr
        terminal = final_json(
            run_aw(
                root,
                "wi",
                "spike",
                "resolve",
                slug,
                "--decision",
                "Use bounded exponential backoff.",
                "--no-action",
            )
        )
        assert terminal["terminal_state"] == "decided"
        issue = show(root, slug)
        assert issue["state"] == "closed"
        assert "Status: decided" in issue["body"]
        assert "Follow-up: no-action" in issue["body"]

        expired_body = (
            "## Question\n\nWhat should expire?\n\n"
            "## Evidence Plan\n\n- Gather evidence.\n\n"
            "## Exit Criteria\n\n- Record a decision.\n\n"
            "## Timebox\n\nExpires At: 2000-01-01T00:00:00Z\n"
        )
        expired = create(
            root,
            "Expired investigation",
            "spike",
            "--body",
            expired_body,
        )
        terminal = final_json(run_aw(root, "wi", "spike", "expire", expired["slug"]))
        assert terminal["terminal_state"] == "gave_up"
        assert "Status: gave_up" in show(root, expired["slug"])["body"]
    return list(ASSERTIONS)


if __name__ == "__main__":
    verify_case(CASE_ID, verify)
