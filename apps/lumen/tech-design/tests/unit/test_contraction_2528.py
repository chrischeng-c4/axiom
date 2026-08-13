"""Unit test suite for deferred dynamic shard contraction design model (#2528)."""
from __future__ import annotations

import unittest

from lumen.topology.contraction_admission import (
    decide_contraction,
    decide_entry_gate,
    implementation_children_allowed,
    validate_v1_dependency,
)
from lumen.topology.contraction_review import ContractionDecisions, review_completeness
from lumen.topology.contraction_spec import ContractionState, EntryGateEvidence, V1Dependency
from lumen.topology.contraction_verdict import ContractionReason, PvcDisposition


class TestContraction2528DesignModel(unittest.TestCase):
    """Test suite exercising contract rules and edge cases outside the EC matrix."""

    def test_contraction_valid_advancement(self) -> None:
        state = ContractionState(
            phase="CONSOLIDATE",
            catalog_from=1,
            catalog_to=2,
            live_data_consolidated=True,
            wal_consolidated=True,
            cutover_committed=False,
            rollback_requested=False,
        )
        verdict = decide_contraction(state)
        self.assertEqual(verdict.outcome, "admitted")
        self.assertEqual(verdict.next_phase, "CUTOVER")
        self.assertEqual(verdict.catalog_version_transition, (1, 2))
        self.assertEqual(verdict.rollback_status, "eligible")
        self.assertEqual(verdict.source_retirement_status, "not_eligible")

    def test_contraction_invalid_catalog_jump(self) -> None:
        # Non-adjacent jump (1 -> 3)
        state_jump = ContractionState(
            phase="CONSOLIDATE",
            catalog_from=1,
            catalog_to=3,
            live_data_consolidated=True,
            wal_consolidated=True,
            cutover_committed=False,
            rollback_requested=False,
        )
        verdict_jump = decide_contraction(state_jump)
        self.assertEqual(verdict_jump.outcome, "rejected")
        self.assertEqual(verdict_jump.reason, ContractionReason.INVALID_CATALOG_TRANSITION.value)
        self.assertEqual(verdict_jump.field_path, "catalog_to")

        # Backward transition (3 -> 2)
        state_back = ContractionState(
            phase="CONSOLIDATE",
            catalog_from=3,
            catalog_to=2,
            live_data_consolidated=True,
            wal_consolidated=True,
            cutover_committed=False,
            rollback_requested=False,
        )
        verdict_back = decide_contraction(state_back)
        self.assertEqual(verdict_back.outcome, "rejected")

    def test_contraction_unconsolidated_live_data(self) -> None:
        state = ContractionState(
            phase="CONSOLIDATE",
            catalog_from=2,
            catalog_to=3,
            live_data_consolidated=False,
            wal_consolidated=True,
            cutover_committed=False,
            rollback_requested=False,
        )
        verdict = decide_contraction(state)
        self.assertEqual(verdict.outcome, "rejected")
        self.assertEqual(verdict.reason, ContractionReason.LIVE_DATA_NOT_CONSOLIDATED.value)
        self.assertEqual(verdict.field_path, "live_data_consolidated")

    def test_entry_gate_evidence_all_missing(self) -> None:
        evidence = EntryGateEvidence(
            risk_quantified=False,
            temporary_capacity_quantified=False,
            recovery_time_quantified=False,
            cost_benefit_quantified=False,
        )
        verdict = decide_entry_gate(evidence)
        self.assertEqual(verdict.outcome, "rejected")
        self.assertEqual(verdict.reason, ContractionReason.EVIDENCE_INCOMPLETE.value)
        self.assertEqual(verdict.field_path, "risk_quantified")

    def test_v1_dependency_variations(self) -> None:
        # Allowed kinds
        for allowed_kind in ("read_replicas", "voter_placement", "ha_failover"):
            verdict = validate_v1_dependency(V1Dependency(kind=allowed_kind))
            self.assertEqual(verdict.outcome, "admitted")

        # Forbidden kinds with merge or contraction
        for forbidden_kind in ("shard_merge", "dynamic_contraction", "merge_topology"):
            verdict = validate_v1_dependency(V1Dependency(kind=forbidden_kind))
            self.assertEqual(verdict.outcome, "rejected")
            self.assertEqual(verdict.reason, ContractionReason.CONTRACTION_DEPENDENCY_NOT_PERMITTED.value)
            self.assertEqual(verdict.field_path, "dependency.kind")

    def test_review_completeness_multiple_missing(self) -> None:
        decisions = ContractionDecisions(
            durability="",
            routing="target catalog after cutover",
            rollback="",
            pvc_retention="retain or separately reclaim",
        )
        result = review_completeness(decisions)
        self.assertEqual(result.missing_decisions, ("durability", "rollback"))

    def test_pvc_disposition_enum_closed(self) -> None:
        dispositions = tuple(sorted(d.value for d in PvcDisposition))
        self.assertEqual(dispositions, ("reclaimable", "retained"))

    def test_implementation_children_allowed_gate(self) -> None:
        passing_gate = decide_entry_gate(
            EntryGateEvidence(
                risk_quantified=True,
                temporary_capacity_quantified=True,
                recovery_time_quantified=True,
                cost_benefit_quantified=True,
            )
        )
        self.assertEqual(implementation_children_allowed(passing_gate).outcome, "allowed")

        failing_gate = decide_entry_gate(
            EntryGateEvidence(
                risk_quantified=True,
                temporary_capacity_quantified=True,
                recovery_time_quantified=True,
                cost_benefit_quantified=False,
            )
        )
        denied = implementation_children_allowed(failing_gate)
        self.assertEqual(denied.outcome, "rejected")
        self.assertEqual(denied.reason, ContractionReason.ENTRY_GATE_NOT_PASSED.value)
        self.assertEqual(denied.field_path, "entry_gate")


if __name__ == "__main__":
    unittest.main()
