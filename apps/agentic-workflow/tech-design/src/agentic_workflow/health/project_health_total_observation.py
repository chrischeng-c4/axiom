"""Canonical Python TD for the Agentic Workflow health observation contract.

@spec #2785
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


__aw_artifact_id__ = "artifact:health/project-health-total-observation"


class AxisRequirement(str, Enum):
    REQUIRED = "required"
    ADVISORY = "advisory"
    NOT_APPLICABLE = "not_applicable"


class AxisEvaluation(str, Enum):
    PASSED = "passed"
    FAILED = "failed"
    UNAVAILABLE = "unavailable"
    NOT_EVALUATED = "not_evaluated"
    NOT_CONFIGURED = "not_configured"
    NOT_APPLICABLE = "not_applicable"


class HealthAssessment(str, Enum):
    HEALTHY = "healthy"
    DEGRADED = "degraded"
    BLOCKED = "blocked"
    INDETERMINATE = "indeterminate"


@dataclass(frozen=True)
class AxisAssessment:
    requirement: AxisRequirement
    evaluation: AxisEvaluation
    findings: tuple[str, ...] = ()


def reduce_health(axes: tuple[AxisAssessment, ...]) -> HealthAssessment:
    if any(
        axis.requirement is AxisRequirement.REQUIRED
        and axis.evaluation is AxisEvaluation.FAILED
        for axis in axes
    ):
        return HealthAssessment.BLOCKED
    if any(
        axis.requirement is AxisRequirement.REQUIRED
        and axis.evaluation
        in {
            AxisEvaluation.UNAVAILABLE,
            AxisEvaluation.NOT_EVALUATED,
            AxisEvaluation.NOT_CONFIGURED,
        }
        for axis in axes
    ):
        return HealthAssessment.INDETERMINATE
    if any(
        axis.requirement is AxisRequirement.ADVISORY
        and axis.evaluation in {AxisEvaluation.FAILED, AxisEvaluation.UNAVAILABLE}
        for axis in axes
    ):
        return HealthAssessment.DEGRADED
    return HealthAssessment.HEALTHY


def aggregate_exit_code(assessment: HealthAssessment) -> int:
    if assessment in {HealthAssessment.BLOCKED, HealthAssessment.INDETERMINATE}:
        return 1
    return 0


def focused_exit_code(evaluation: AxisEvaluation) -> int:
    if evaluation in {AxisEvaluation.FAILED, AxisEvaluation.UNAVAILABLE}:
        return 1
    return 0
