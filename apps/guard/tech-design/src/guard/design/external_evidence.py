"""Executable design for external evidence normalization."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

__aw_artifact_id__ = "artifact:guard/design-external-evidence"


class EvidenceStatus(Enum):
    CLEAN = "clean"
    FINDINGS = "findings"
    TOOL_ERROR = "tool_error"


@dataclass(frozen=True)
class EvidenceDecision:
    status: EvidenceStatus
    clean: bool
    finding_count: int


def normalize_external_result(
    process_success: bool,
    report_clean: bool | None,
    finding_count: int,
) -> EvidenceDecision:
    clean = process_success and (report_clean if report_clean is not None else True)
    if clean:
        return EvidenceDecision(EvidenceStatus.CLEAN, True, finding_count)
    return EvidenceDecision(EvidenceStatus.FINDINGS, False, finding_count)


def required_integration_tools() -> tuple[str, ...]:
    return ("vat", "rig", "meter")
