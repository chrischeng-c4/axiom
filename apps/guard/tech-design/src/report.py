"""Guard report contract and fail-closed state reduction."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

__aw_artifact_id__ = "artifact:guard/security-ec-profile"
__aw_public_contract__ = True


class ReportState(Enum):
    CLEAN = "clean"
    FINDINGS = "findings"
    TOOL_ERROR = "tool_error"


@dataclass(frozen=True)
class ReportDecision:
    state: ReportState
    exit_code: int
    completion_clean: bool


class ReportDesign:
    REQUIRED_FIELDS = (
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

    @staticmethod
    def reduce_state(
        actionable_findings: int,
        tool_error_code: int | None,
    ) -> ReportDecision:
        if tool_error_code is not None:
            return ReportDecision(ReportState.TOOL_ERROR, tool_error_code, False)
        if actionable_findings > 0:
            return ReportDecision(ReportState.FINDINGS, 1, False)
        return ReportDecision(ReportState.CLEAN, 0, True)


def aw_health_security_metric() -> str:
    return "status, exit code, findings, completion, and prompt change together"


def ec_security_evidence_command() -> str:
    return "adapter exit, report, and findings fold independently and fail closed"


def security_report_consumer_contract() -> str:
    return "public report fields derive one unambiguous lifecycle decision"


def stable_security_metric_projection() -> str:
    return "equivalent scans preserve the path-independent lifecycle metric"
