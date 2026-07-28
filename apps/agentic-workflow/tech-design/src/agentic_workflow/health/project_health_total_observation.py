"""Canonical Python TD for the two-cell semantic-health contract.

@spec #2785
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


__aw_artifact_id__ = "artifact:agentic-workflow/project-health-total-observation"


class CellEvaluation(str, Enum):
    PASSED = "passed"
    FAILED = "failed"
    UNAVAILABLE = "unavailable"
    NOT_EVALUATED = "not_evaluated"


class HealthAssessment(str, Enum):
    HEALTHY = "healthy"
    BLOCKED = "blocked"
    INDETERMINATE = "indeterminate"


@dataclass(frozen=True)
class EcAcceptsTd:
    evaluation: CellEvaluation
    findings: tuple[str, ...] = ()


@dataclass(frozen=True)
class EcTdAlignment:
    missing_in_td: tuple[str, ...] = ()
    missing_in_ec: tuple[str, ...] = ()

    @property
    def evaluation(self) -> CellEvaluation:
        if self.missing_in_td or self.missing_in_ec:
            return CellEvaluation.FAILED
        return CellEvaluation.PASSED


@dataclass(frozen=True)
class SemanticHealth:
    ec_accepts_td: EcAcceptsTd
    ec_td_alignment: EcTdAlignment


def reduce_health(health: SemanticHealth) -> HealthAssessment:
    cells = (
        health.ec_accepts_td.evaluation,
        health.ec_td_alignment.evaluation,
    )
    if CellEvaluation.FAILED in cells:
        return HealthAssessment.BLOCKED
    if any(
        cell in {CellEvaluation.UNAVAILABLE, CellEvaluation.NOT_EVALUATED}
        for cell in cells
    ):
        return HealthAssessment.INDETERMINATE
    return HealthAssessment.HEALTHY


def aggregate_exit_code(assessment: HealthAssessment) -> int:
    if assessment in {HealthAssessment.BLOCKED, HealthAssessment.INDETERMINATE}:
        return 1
    return 0
