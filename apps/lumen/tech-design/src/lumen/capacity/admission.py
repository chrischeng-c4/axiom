"""Capacity admission decision logic for Raft data-plane capacity envelopes."""
from __future__ import annotations

from typing import Final

from lumen.capacity.spec import (
    CapacitySpec,
    MachineFamily,
    PdSsdDisposition,
    StorageFormat,
)
from lumen.capacity.verdict import (
    AdmittedGuidance,
    Rejection,
    RejectionReason,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity/admission"


def decide_capacity_guidance(spec: CapacitySpec) -> AdmittedGuidance | Rejection:
    """Decide capacity guidance admission and envelope selection for a spec."""
    # R4: Price and regional billing data are forbidden.
    for field in spec.declared_record_schema_fields:
        if "price" in field or "billing" in field:
            return Rejection(
                reason=RejectionReason.PRICE_DATA_NOT_ALLOWED,
                field_path="declared_record_schema_fields",
            )

    # R5: Automatic storage-class migration is unsupported in v1.
    if spec.requested_automatic_storage_class_migration:
        return Rejection(
            reason=RejectionReason.AUTOMATIC_STORAGE_CLASS_MIGRATION_UNSUPPORTED,
            field_path="requested_automatic_storage_class_migration",
        )

    # R7: Legacy whole-state JSON format is refused.
    if spec.storage_format == StorageFormat.LEGACY_WHOLE_STATE_JSON:
        return Rejection(
            reason=RejectionReason.LEGACY_WHOLE_STATE_JSON,
            field_path="storage_format",
        )

    # R7 / AC5: Missing storage evidence prerequisites (attestations).
    if not spec.bounded_steady_state_write_amplification_attested:
        return Rejection(
            reason=RejectionReason.MISSING_STORAGE_EVIDENCE_PREREQUISITE,
            field_path="bounded_steady_state_write_amplification_attested",
        )

    if not spec.storage_format_attested:
        return Rejection(
            reason=RejectionReason.MISSING_STORAGE_EVIDENCE_PREREQUISITE,
            field_path="storage_format_attested",
        )

    # AC3: N2 machine family requires qualifying evidence.
    if (
        spec.requested_machine_family == MachineFamily.N2
        and not spec.n2_evidence_eligible
    ):
        return Rejection(
            reason=RejectionReason.N2_EVIDENCE_NOT_QUALIFIED,
            field_path="requested_machine_family",
        )

    # R6 / AC4: Machine family selection — E2 is default unless requested N2 + eligible + explicit rejection of E2.
    if (
        spec.requested_machine_family == MachineFamily.N2
        and spec.n2_evidence_eligible
        and spec.explicit_e2_pd_balanced_rejection
    ):
        machine_family = MachineFamily.N2
    else:
        machine_family = MachineFamily.E2

    return AdmittedGuidance(
        machine_family=machine_family,
        storage_class=spec.requested_storage_class,
        pd_ssd_disposition=PdSsdDisposition.INITIAL_ONLY_FUTURE,
    )
