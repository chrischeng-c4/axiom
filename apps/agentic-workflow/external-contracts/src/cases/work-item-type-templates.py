"""Python EC implementation for type-specific WI authoring profiles."""

from __future__ import annotations

import sys
from pathlib import Path


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, project_fixture, run_aw, show, verify_case


CASE_ID = "work-item-type-templates"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "typed-work-item-authoring-profiles"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "python3 apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-type-templates"
)
ASSERTIONS = (
    "Spike and Report expose exact type-specific profiles",
    "Report intake is exempt from Capability Alignment",
    "cross-profile sections are rejected",
    "boundedness reads title and In Scope while ignoring descriptive and anti-scope prose",
)


def change_body(in_scope: str) -> str:
    return (
        "## Problem\n\nThe entire project currently exposes this message.\n\n"
        "## Capability Alignment\n\n"
        "Capability: Work item planning\n"
        "Capability Gap: boundedness diagnostics are imprecise\n"
        "Progress Evidence: the public create result is the evidence\n\n"
        "## Requirements\n\n- R1: Fix one parser diagnostic.\n\n"
        f"## Scope\n\n### In Scope\n- {in_scope}\n\n"
        "### Out of Scope\n- Rework the whole suite.\n\n"
        "## Acceptance Criteria\n\n- AC1: the bounded item is accepted.\n\n"
        "## Reference Context\n\n### Related Specs\n"
        "| Spec | Relevance |\n|------|-----------|\n"
        "| complete-platform.md | describes the environment |\n\n"
        "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
        "|---------|--------|---------------|\n"
        "| boundedness | update | complete-platform.md |\n"
    )


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

        create(
            root,
            "Fix one parser diagnostic",
            "change",
            "--body",
            change_body("Fix one parser diagnostic."),
        )
        oversized = run_aw(
            root,
            "wi",
            "create",
            "--title",
            "Rewrite parser diagnostics",
            "--type",
            "change",
            "--project",
            "demo",
            "--body",
            change_body("Rewrite the entire project."),
            expect_success=False,
        )
        assert "too-large" in oversized.stderr
    return list(ASSERTIONS)


if __name__ == "__main__":
    verify_case(CASE_ID, verify)
