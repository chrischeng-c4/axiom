"""Admission and policy deciders for deferred dynamic shard contraction (#2528)."""
from __future__ import annotations

from typing import Final

from lumen.topology.contraction_spec import ContractionState, EntryGateEvidence, V1Dependency
from lumen.topology.contraction_verdict import (
    ContractionReason,
    ContractionVerdict,
    EntryGateVerdict,
    ImplementationChildrenVerdict,
    V1DependencyVerdict,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/contraction-admission"


def decide_contraction(state: ContractionState) -> ContractionVerdict:
    """Decide phase progression, catalog transition, and eligibility for contraction state."""
    if state.catalog_from < 1 or state.catalog_to != state.catalog_from + 1:
        return ContractionVerdict(
            outcome="rejected",
            reason=ContractionReason.INVALID_CATALOG_TRANSITION.value,
            field_path="catalog_to",
            message="catalog version transition must move forward by exactly one version",
        )

    if not state.live_data_consolidated:
        return ContractionVerdict(
            outcome="rejected",
            reason=ContractionReason.LIVE_DATA_NOT_CONSOLIDATED.value,
            field_path="live_data_consolidated",
            message="live data consolidation is required before contraction cutover",
        )

    if not state.wal_consolidated:
        return ContractionVerdict(
            outcome="rejected",
            reason=ContractionReason.WAL_NOT_CONSOLIDATED.value,
            field_path="wal_consolidated",
            message="WAL consolidation is required before contraction cutover",
        )

    transition = (state.catalog_from, state.catalog_to)

    if state.phase == "CONSOLIDATE":
        next_phase = "CUTOVER"
        rollback_status = (
            "eligible" if (not state.cutover_committed and not state.rollback_requested) else "not_eligible"
        )
        source_retirement_status = "not_eligible"
    elif state.phase == "CUTOVER":
        next_phase = (
            "RETIRE" if (state.cutover_committed and not state.rollback_requested) else ("ROLLBACK" if state.rollback_requested else "CUTOVER")
        )
        rollback_status = (
            "eligible" if (not state.cutover_committed and not state.rollback_requested) else "not_eligible"
        )
        source_retirement_status = (
            "eligible" if (state.cutover_committed and not state.rollback_requested) else "not_eligible"
        )
    elif state.phase == "RETIRE":
        next_phase = "COMPLETE"
        rollback_status = "not_eligible"
        source_retirement_status = "eligible" if not state.rollback_requested else "not_eligible"
    else:
        return ContractionVerdict(
            outcome="rejected",
            reason=ContractionReason.INVALID_CATALOG_TRANSITION.value,
            field_path="phase",
            message=f"unsupported contraction phase: {state.phase}",
        )

    return ContractionVerdict(
        outcome="admitted",
        next_phase=next_phase,
        catalog_version_transition=transition,
        rollback_status=rollback_status,
        source_retirement_status=source_retirement_status,
    )


def decide_entry_gate(evidence: EntryGateEvidence) -> EntryGateVerdict:
    """Evaluate mandatory measured evidence before opening implementation work."""
    if not evidence.risk_quantified:
        return EntryGateVerdict(
            outcome="rejected",
            reason=ContractionReason.EVIDENCE_INCOMPLETE.value,
            field_path="risk_quantified",
            message="risk must be quantified before opening implementation work",
        )

    if not evidence.temporary_capacity_quantified:
        return EntryGateVerdict(
            outcome="rejected",
            reason=ContractionReason.EVIDENCE_INCOMPLETE.value,
            field_path="temporary_capacity_quantified",
            message="temporary capacity must be quantified before opening implementation work",
        )

    if not evidence.recovery_time_quantified:
        return EntryGateVerdict(
            outcome="rejected",
            reason=ContractionReason.EVIDENCE_INCOMPLETE.value,
            field_path="recovery_time_quantified",
            message="recovery time must be quantified before opening implementation work",
        )

    if not evidence.cost_benefit_quantified:
        return EntryGateVerdict(
            outcome="rejected",
            reason=ContractionReason.EVIDENCE_INCOMPLETE.value,
            field_path="cost_benefit_quantified",
            message="cost benefit must be quantified before opening implementation work",
        )

    return EntryGateVerdict(outcome="passed")


def validate_v1_dependency(dependency: V1Dependency) -> V1DependencyVerdict:
    """Enforce preservation of the split-only v1 contract."""
    kind_lower = dependency.kind.lower()
    if "contraction" in kind_lower or "merge" in kind_lower:
        return V1DependencyVerdict(
            outcome="rejected",
            reason=ContractionReason.CONTRACTION_DEPENDENCY_NOT_PERMITTED.value,
            field_path="dependency.kind",
            message="v1 features must not depend on merge or dynamic contraction",
        )

    return V1DependencyVerdict(outcome="admitted")


def implementation_children_allowed(
    entry_gate_verdict: EntryGateVerdict,
) -> ImplementationChildrenVerdict:
    """Decide whether implementation child issues are allowed."""
    if entry_gate_verdict.outcome == "passed":
        return ImplementationChildrenVerdict(outcome="allowed")

    return ImplementationChildrenVerdict(
        outcome="rejected",
        reason=ContractionReason.ENTRY_GATE_NOT_PASSED.value,
        field_path="entry_gate",
        message="implementation child issues require a passing entry gate verdict",
    )
