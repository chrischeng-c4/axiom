"""Executable design for guard.report/1 state and summary semantics."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

__aw_artifact_id__ = "artifact:guard/design-report-model"


class ReportState(Enum):
    CLEAN = "clean"
    FINDINGS = "findings"
    TOOL_ERROR = "tool_error"


@dataclass(frozen=True)
class ReportDecision:
    state: ReportState
    exit_code: int
    completion_clean: bool


def reduce_report_state(actionable_findings: int, tool_error_code: int | None) -> ReportDecision:
    if tool_error_code is not None:
        return ReportDecision(ReportState.TOOL_ERROR, tool_error_code, False)
    if actionable_findings > 0:
        return ReportDecision(ReportState.FINDINGS, 1, False)
    return ReportDecision(ReportState.CLEAN, 0, True)


def required_report_fields() -> tuple[str, ...]:
    return (
        "schema_version",
        "tool_version",
        "verb",
        "target",
        "policy_profile",
        "status",
        "exit_code",
        "summary",
        "findings",
        "evidence",
        "completion",
        "integrations",
        "agent_prompt",
    )
