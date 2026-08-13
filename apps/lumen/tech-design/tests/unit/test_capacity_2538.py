"""Unit tests for capacity admission decisions (#2538) outside EC matrix."""
from __future__ import annotations

import unittest

from lumen.capacity.admission import decide_capacity_guidance
from lumen.capacity.spec import (
    CapacitySpec,
    MachineFamily,
    PdSsdDisposition,
    StorageClass,
    StorageFormat,
)
from lumen.capacity.verdict import (
    AdmittedGuidance,
    GuidanceKind,
    Rejection,
    RejectionReason,
)


class TestCapacityAdmission2538(unittest.TestCase):
    def test_custom_schema_fields_admitted(self) -> None:
        spec = CapacitySpec(
            declared_record_schema_fields=frozenset(
                {"throughput", "custom_metric_alpha", "read_latency_p99"}
            ),
            storage_format=StorageFormat.SEGMENT_CHECKPOINT,
            storage_format_attested=True,
            bounded_steady_state_write_amplification_attested=True,
            requested_machine_family=MachineFamily.E2,
        )
        res = decide_capacity_guidance(spec)
        self.assertIsInstance(res, AdmittedGuidance)
        assert isinstance(res, AdmittedGuidance)
        self.assertEqual(res.kind, GuidanceKind.ADMITTED)
        self.assertEqual(res.machine_family, MachineFamily.E2)
        self.assertEqual(res.storage_class, StorageClass.PD_BALANCED)
        self.assertEqual(
            res.pd_ssd_disposition, PdSsdDisposition.INITIAL_ONLY_FUTURE
        )

    def test_custom_price_field_rejected(self) -> None:
        spec = CapacitySpec(
            declared_record_schema_fields=frozenset(
                {"throughput", "spot_price_cents"}
            ),
            storage_format=StorageFormat.SEGMENT_CHECKPOINT,
            storage_format_attested=True,
            bounded_steady_state_write_amplification_attested=True,
        )
        res = decide_capacity_guidance(spec)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, RejectionReason.PRICE_DATA_NOT_ALLOWED)
        self.assertEqual(res.field_path, "declared_record_schema_fields")
        self.assertIsNone(res.threshold)
        self.assertIsNone(res.machine_recommendation)

    def test_custom_billing_field_rejected(self) -> None:
        spec = CapacitySpec(
            declared_record_schema_fields=frozenset(
                {"billing_tier", "latency"}
            ),
            storage_format=StorageFormat.SEGMENT_CHECKPOINT,
            storage_format_attested=True,
            bounded_steady_state_write_amplification_attested=True,
        )
        res = decide_capacity_guidance(spec)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, RejectionReason.PRICE_DATA_NOT_ALLOWED)
        self.assertEqual(res.field_path, "declared_record_schema_fields")

    def test_e2_requested_with_explicit_rejection_retains_e2(self) -> None:
        # A caller requesting E2 when explicit E2 rejection is True must NOT be rewritten to N2
        spec = CapacitySpec(
            storage_format=StorageFormat.SEGMENT_CHECKPOINT,
            storage_format_attested=True,
            bounded_steady_state_write_amplification_attested=True,
            n2_evidence_eligible=True,
            explicit_e2_pd_balanced_rejection=True,
            requested_machine_family=MachineFamily.E2,
        )
        res = decide_capacity_guidance(spec)
        self.assertIsInstance(res, AdmittedGuidance)
        assert isinstance(res, AdmittedGuidance)
        self.assertEqual(res.machine_family, MachineFamily.E2)

    def test_multiple_rejections_priority_order(self) -> None:
        # Price check comes before migration check
        spec = CapacitySpec(
            declared_record_schema_fields=frozenset({"unit_price"}),
            requested_automatic_storage_class_migration=True,
            storage_format=StorageFormat.LEGACY_WHOLE_STATE_JSON,
        )
        res = decide_capacity_guidance(spec)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, RejectionReason.PRICE_DATA_NOT_ALLOWED)
        self.assertEqual(res.field_path, "declared_record_schema_fields")

    def test_rejection_field_paths_are_distinct(self) -> None:
        # Price rejection
        r1 = decide_capacity_guidance(
            CapacitySpec(
                declared_record_schema_fields=frozenset({"hourly_price"}),
                storage_format=StorageFormat.SEGMENT_CHECKPOINT,
                storage_format_attested=True,
                bounded_steady_state_write_amplification_attested=True,
            )
        )
        # Migration rejection
        r2 = decide_capacity_guidance(
            CapacitySpec(
                requested_automatic_storage_class_migration=True,
                storage_format=StorageFormat.SEGMENT_CHECKPOINT,
                storage_format_attested=True,
                bounded_steady_state_write_amplification_attested=True,
            )
        )
        # Legacy format rejection
        r3 = decide_capacity_guidance(
            CapacitySpec(
                storage_format=StorageFormat.LEGACY_WHOLE_STATE_JSON,
                storage_format_attested=True,
                bounded_steady_state_write_amplification_attested=True,
            )
        )
        # Write amp rejection
        r4 = decide_capacity_guidance(
            CapacitySpec(
                storage_format=StorageFormat.SEGMENT_CHECKPOINT,
                storage_format_attested=True,
                bounded_steady_state_write_amplification_attested=False,
            )
        )
        # Format attestation rejection
        r5 = decide_capacity_guidance(
            CapacitySpec(
                storage_format=StorageFormat.SEGMENT_CHECKPOINT,
                storage_format_attested=False,
                bounded_steady_state_write_amplification_attested=True,
            )
        )
        # N2 evidence rejection
        r6 = decide_capacity_guidance(
            CapacitySpec(
                requested_machine_family=MachineFamily.N2,
                n2_evidence_eligible=False,
                storage_format=StorageFormat.SEGMENT_CHECKPOINT,
                storage_format_attested=True,
                bounded_steady_state_write_amplification_attested=True,
            )
        )

        assert isinstance(r1, Rejection)
        assert isinstance(r2, Rejection)
        assert isinstance(r3, Rejection)
        assert isinstance(r4, Rejection)
        assert isinstance(r5, Rejection)
        assert isinstance(r6, Rejection)

        field_paths = {
            r1.field_path,
            r2.field_path,
            r3.field_path,
            r4.field_path,
            r5.field_path,
            r6.field_path,
        }
        self.assertEqual(len(field_paths), 6)


if __name__ == "__main__":
    unittest.main()
