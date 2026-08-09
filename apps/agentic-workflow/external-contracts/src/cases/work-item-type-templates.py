"""Python EC implementation for type-specific WI authoring profiles."""

from __future__ import annotations

import sys
from pathlib import Path


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, project_fixture, run_aw, show, verify_case


CASE_ID = "work-item-type-templates"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "terminology-first-four-type-wi-taxonomy"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
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
        "## Goal\n\n"
        f"When aw wi create runs, it executes for {in_scope}\n\n"
        "## How\n\n"
        "### Verified premises\n\n"
        "- apps/agentic-workflow/external-contracts/src/cases/work-item-type-templates.py:30 defines the type template case.\n\n"
        "### Change points\n\n"
        "- apps/agentic-workflow/external-contracts/src/cases/work-item-type-templates.py — update change body to GHAN.\n\n"
        "### Frozen decisions\n\n"
        "Spike and report bodies remain unchanged.\n\n"
        "## Acceptance\n\n"
        "| # | command | current | target | why it cannot hold by accident |\n"
        "|---|---------|---------|--------|--------------------------------|\n"
        "| 1 | `aw wi create` | legacy refusal | admitted | validates work item template |\n\n"
        "### Negative control\n\n"
        "Under line 32 mutation the gate must go red restoring to sha256 d98340ceb2562c4435d3dbce0eb13980632f276a000fc24037770d12897646b6\n\n"
        "## Never\n\n"
        "This addresses the worker implementing this work item, not the controller reviewing it.\n\n"
        "### Must not touch\n\n"
        "- apps/agentic-workflow/src/issues/ghan.rs — validator is fixed.\n\n"
        "### Must not do\n\n"
        "- Do not alter spike or report profiles.\n"
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
