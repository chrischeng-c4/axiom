"""Canonical work-item authoring profiles.

@spec #2594
"""

from __future__ import annotations

from enum import StrEnum


class WorkItemTemplate(StrEnum):
    CHANGE = "problem,capability_alignment,requirements,scope,acceptance_criteria,reference_context"
    SPIKE = "question,evidence_plan,exit_criteria,timebox"
    REPORT = "repro,diagnostics,expected_vs_actual"


def requires_capability_alignment(work_item_type: str) -> bool:
    """Only executable change leaves carry delivery alignment at intake."""

    return work_item_type == "change"


def required_sections(work_item_type: str) -> tuple[str, ...]:
    """Return the exact H2 profile for a canonical work-item type."""

    if work_item_type == "spike":
        return ("Question", "Evidence Plan", "Exit Criteria", "Timebox")
    if work_item_type == "report":
        return ("Repro", "Diagnostics", "Expected vs Actual")
    return (
        "Problem",
        "Capability Alignment",
        "Requirements",
        "Scope",
        "Acceptance Criteria",
        "Reference Context",
    )
