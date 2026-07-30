"""Python EC implementation for health's typed intake queue."""

from __future__ import annotations

import sys
from pathlib import Path


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw, verify_case


CASE_ID = "work-item-intake-health"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "terminology-first-four-type-wi-taxonomy"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "python3 apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-intake-health"
)
ASSERTIONS = (
    "health counts open Reports and expired Spikes in a typed intake axis",
    "health prioritizes Report triage before expired Spike remediation",
)


def health(root: Path) -> dict:
    completed = run_aw(root, "health", "--project", "demo", expect_success=False)
    return final_json(completed)


def verify() -> list[str]:
    with project_fixture() as root:
        report = create(root, "Untriaged CLI report", "report")
        expired_body = (
            "## Question\n\nWhat should expire?\n\n"
            "## Evidence Plan\n\n- Gather evidence.\n\n"
            "## Exit Criteria\n\n- Record a decision.\n\n"
            "## Timebox\n\nExpires At: 2000-01-01T00:00:00Z\n"
        )
        spike = create(root, "Expired investigation", "spike", "--body", expired_body)
        final_json(run_aw(root, "wi", "validate", report["slug"]))
        final_json(run_aw(root, "wi", "validate", spike["slug"]))

        intake = health(root)["axes"]["intake_queue"]
        assert intake["status"] == "pending", intake
        assert intake["open_report_count"] == 1
        assert intake["expired_spike_count"] == 1
        assert intake["next_command"] == (
            f"aw wi triage {report['slug']} --verdict accepted"
        )

        final_json(
            run_aw(
                root,
                "wi",
                "triage",
                report["slug"],
                "--verdict",
                "duplicate",
            )
        )
        intake = health(root)["axes"]["intake_queue"]
        assert intake["open_report_count"] == 0
        assert intake["expired_spike_count"] == 1
        assert intake["next_command"] == f"aw wi spike expire {spike['slug']}"
    return list(ASSERTIONS)


if __name__ == "__main__":
    verify_case(CASE_ID, verify)
