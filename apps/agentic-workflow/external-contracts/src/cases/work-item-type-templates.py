"""Python EC implementation for type-specific WI authoring profiles."""

from __future__ import annotations

import sys
from pathlib import Path


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, project_fixture, run_aw, show, verify_case


CASE_ID = "work-item-type-templates"


def verify() -> list[str]:
    with project_fixture() as root:
        spike = create(root, "Which retry policy should the runner use?", "spike")
        spike_body = show(root, spike["slug"])["body"]
        for heading in ("Question", "Evidence Plan", "Exit Criteria", "Timebox"):
            assert f"## {heading}" in spike_body
        assert "## Capability Alignment" not in spike_body

        report = create(root, "CLI exits without a remediation command", "report")
        report_body = show(root, report["slug"])["body"]
        for heading in ("Repro", "Diagnostics", "Expected vs Actual"):
            assert f"## {heading}" in report_body
        assert "## Capability Alignment" not in report_body

        invalid = run_aw(
            root,
            "wi",
            "create",
            "--title",
            "Cross-profile report",
            "--type",
            "report",
            "--project",
            "demo",
            "--body",
            "## Repro\n\n- reproduce\n\n"
            "## Diagnostics\n\n- logs\n\n"
            "## Expected vs Actual\n\nExpected: success\nActual: failure\n\n"
            "## Scope\n\n- forbidden\n",
            expect_success=False,
        )
        assert "report work-item accepts only these H2 sections" in invalid.stderr
    return [
        "Spike and Report expose exact type-specific profiles",
        "Report intake is exempt from Capability Alignment",
        "cross-profile sections are rejected",
    ]


if __name__ == "__main__":
    verify_case(CASE_ID, verify)
