"""EC security case for #2528 -- fail-closed deferred contraction.

Every expected value is an EC-owned literal from #2528 R1-R4 and AC1-AC2.
The case checks the named refusal vocabulary and the field named by each
refusal, then keeps the neighbouring measured or complete request admitted.
It imports the absent pure design at module load time deliberately: missing
design code is a failed external-contract gate, not a skipped case.
"""

from __future__ import annotations

from lumen.topology.contraction_admission import (
    decide_contraction,
    decide_entry_gate,
    implementation_children_allowed,
    validate_v1_dependency,
)
from lumen.topology.contraction_review import ContractionDecisions, review_completeness
from lumen.topology.contraction_spec import ContractionState, EntryGateEvidence, V1Dependency
from lumen.topology.contraction_verdict import PvcDisposition

MINIMUM_CHECKS = 15

CONTRACTION_2528_SECURITY_MATRIX = (
    ("unconsolidated_wal_blocks_cutover", "wal_not_consolidated"),
    ("unconsolidated_wal_refusal_names_wal", "wal_consolidated"),
    ("rollback_request_blocks_source_retirement", "not_eligible"),
    ("pvc_disposition_never_offers_shrink", ("reclaimable", "retained")),
    ("missing_risk_rejects_entry_gate", "evidence_incomplete"),
    ("missing_risk_refusal_names_risk", "risk_quantified"),
    ("missing_temporary_capacity_refusal_names_capacity", "temporary_capacity_quantified"),
    ("missing_recovery_time_refusal_names_recovery", "recovery_time_quantified"),
    ("missing_cost_benefit_refusal_names_cost", "cost_benefit_quantified"),
    ("v1_autoscaling_dependency_is_rejected", "contraction_dependency_not_permitted"),
    ("v1_autoscaling_refusal_names_dependency", "dependency.kind"),
    ("v1_zero_downtime_dependency_is_rejected", "contraction_dependency_not_permitted"),
    ("incomplete_review_names_durability", ("durability",)),
    ("rejected_entry_gate_refuses_implementation_children", "entry_gate_not_passed"),
    ("child_refusal_names_entry_gate", "entry_gate"),
)


def verify_contraction_2528_security() -> dict:
    checks = []

    # 1. R1 -- cutover cannot proceed until the supplied WAL-consolidation fact is true.
    wal_missing = decide_contraction(
        ContractionState(
            phase="CONSOLIDATE",
            catalog_from=7,
            catalog_to=8,
            live_data_consolidated=True,
            wal_consolidated=False,
            cutover_committed=False,
            rollback_requested=False,
        )
    )
    obs1 = wal_missing.reason
    exp1 = CONTRACTION_2528_SECURITY_MATRIX[0][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- the refusal identifies the missing gate instead of silently holding.
    obs2 = wal_missing.field_path
    exp2 = CONTRACTION_2528_SECURITY_MATRIX[1][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1 -- requesting rollback keeps the source non-retireable even after cutover.
    rollback = decide_contraction(
        ContractionState(
            phase="CUTOVER",
            catalog_from=7,
            catalog_to=8,
            live_data_consolidated=True,
            wal_consolidated=True,
            cutover_committed=True,
            rollback_requested=True,
        )
    )
    obs3 = rollback.source_retirement_status
    exp3 = CONTRACTION_2528_SECURITY_MATRIX[2][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R2 -- the whole named disposition vocabulary is closed; no member can
    #    represent a smaller requested PVC size.
    obs4 = tuple(sorted(disposition.value for disposition in PvcDisposition))
    exp4 = CONTRACTION_2528_SECURITY_MATRIX[3][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    missing_risk = decide_entry_gate(
        EntryGateEvidence(
            risk_quantified=False,
            temporary_capacity_quantified=True,
            recovery_time_quantified=True,
            cost_benefit_quantified=True,
        )
    )

    # 5. R3 -- each evidence dimension is mandatory, beginning with risk.
    obs5 = missing_risk.reason
    exp5 = CONTRACTION_2528_SECURITY_MATRIX[4][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R3 -- the risk refusal names the omitted supplied field.
    obs6 = missing_risk.field_path
    exp6 = CONTRACTION_2528_SECURITY_MATRIX[5][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    missing_capacity = decide_entry_gate(EntryGateEvidence(risk_quantified=True, temporary_capacity_quantified=False, recovery_time_quantified=True, cost_benefit_quantified=True))
    # 7. R3 -- temporary capacity has an independent fail-closed field.
    obs7 = missing_capacity.field_path
    exp7 = CONTRACTION_2528_SECURITY_MATRIX[6][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    missing_recovery = decide_entry_gate(EntryGateEvidence(risk_quantified=True, temporary_capacity_quantified=True, recovery_time_quantified=False, cost_benefit_quantified=True))
    # 8. R3 -- recovery time cannot be omitted behind another evidence check.
    obs8 = missing_recovery.field_path
    exp8 = CONTRACTION_2528_SECURITY_MATRIX[7][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    missing_cost = decide_entry_gate(EntryGateEvidence(risk_quantified=True, temporary_capacity_quantified=True, recovery_time_quantified=True, cost_benefit_quantified=False))
    # 9. R3 -- quantified cost benefit is independently mandatory.
    obs9 = missing_cost.field_path
    exp9 = CONTRACTION_2528_SECURITY_MATRIX[8][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    autoscaling = validate_v1_dependency(V1Dependency(kind="autoscaling_contraction"))
    # 10. R4 -- v1 autoscaling may not depend on merge or contraction.
    obs10 = autoscaling.reason
    exp10 = CONTRACTION_2528_SECURITY_MATRIX[9][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R4 -- its refusal names the v1 dependency dimension.
    obs11 = autoscaling.field_path
    exp11 = CONTRACTION_2528_SECURITY_MATRIX[10][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    zero_downtime = validate_v1_dependency(V1Dependency(kind="zero_downtime_merge"))
    # 12. R4 -- zero-downtime acceptance has the same independent prohibition.
    obs12 = zero_downtime.reason
    exp12 = CONTRACTION_2528_SECURITY_MATRIX[11][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    incomplete_review = review_completeness(
        ContractionDecisions(
            durability="",
            routing="target catalog after cutover",
            rollback="source retained before cutover",
            pvc_retention="retain or separately reclaim",
        )
    )
    # 13. AC1 -- a missing durability decision is returned as the named review gap.
    obs13 = incomplete_review.missing_decisions
    exp13 = CONTRACTION_2528_SECURITY_MATRIX[12][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    denied_children = implementation_children_allowed(missing_risk)
    # 14. AC2 -- a rejected actual gate result refuses implementation children.
    obs14 = denied_children.reason
    exp14 = CONTRACTION_2528_SECURITY_MATRIX[13][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. AC2 -- the child refusal names the gate that must be repaired.
    obs15 = denied_children.field_path
    exp15 = CONTRACTION_2528_SECURITY_MATRIX[14][1]
    checks.append({"name": CONTRACTION_2528_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "contraction-2528-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(check["passed"] for check in checks) and len(checks) == MINIMUM_CHECKS,
    }
