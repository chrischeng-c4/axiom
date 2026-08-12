"""EC behavior case for #2528 -- deferred dynamic shard contraction.

Every expected value is an EC-owned literal from #2528's td-observable rules:
R1 constrains the proposed contraction transition, R2 the closed PVC
disposition vocabulary, R3 the four-part measured entry gate, R4 the preserved
split-only v1 boundary, AC1 the four reviewed decisions, and AC2 the only
condition under which implementation children may be allowed.  The imports
intentionally fail closed until the pure design model lands.
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

MINIMUM_CHECKS = 10

CONTRACTION_2528_BEHAVIOR_MATRIX = (
    ("ready_contraction_advances_to_cutover", "CUTOVER"),
    ("ready_contraction_versions_the_catalog_forward", (7, 8)),
    ("ready_pre_cutover_contraction_keeps_rollback_eligible", "eligible"),
    ("ready_pre_cutover_contraction_does_not_retire_the_source", "not_eligible"),
    ("pvc_disposition_vocabulary_is_retained_or_reclaimable", ("reclaimable", "retained")),
    ("fully_measured_entry_gate_passes", "passed"),
    ("v1_unrelated_dependency_remains_admitted", "admitted"),
    ("complete_review_has_no_missing_decisions", ()),
    ("passing_entry_gate_allows_implementation_children", "allowed"),
    ("cutover_eligible_contraction_retires_the_source", "eligible"),
)


def verify_contraction_2528_behavior() -> dict:
    checks = []

    ready = decide_contraction(
        ContractionState(
            phase="CONSOLIDATE",
            catalog_from=7,
            catalog_to=8,
            live_data_consolidated=True,
            wal_consolidated=True,
            cutover_committed=False,
            rollback_requested=False,
        )
    )

    # 1. R1 -- consolidated live data and WAL proceed through the named cutover phase.
    obs1 = ready.next_phase
    exp1 = CONTRACTION_2528_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CONTRACTION_2528_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- contraction advances an explicit catalog-version transition.
    obs2 = ready.catalog_version_transition
    exp2 = CONTRACTION_2528_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CONTRACTION_2528_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1 -- before cutover, this state remains rollback eligible.
    obs3 = ready.rollback_status
    exp3 = CONTRACTION_2528_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CONTRACTION_2528_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R1 -- source retirement is not pulled forward into the rollback window.
    obs4 = ready.source_retirement_status
    exp4 = CONTRACTION_2528_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CONTRACTION_2528_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R2 -- the enum is closed: retained PVCs or separately reclaimable PVCs,
    #    never a third disposition that could encode PVC shrink.
    obs5 = tuple(sorted(disposition.value for disposition in PvcDisposition))
    exp5 = CONTRACTION_2528_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CONTRACTION_2528_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    measured = decide_entry_gate(
        EntryGateEvidence(
            risk_quantified=True,
            temporary_capacity_quantified=True,
            recovery_time_quantified=True,
            cost_benefit_quantified=True,
        )
    )

    # 6. R3 -- all four supplied measurements admit the deferred implementation gate.
    obs6 = measured.outcome
    exp6 = CONTRACTION_2528_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CONTRACTION_2528_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R4 -- an unrelated v1 dependency is still admissible; preservation is
    #    not a blanket rejection of v1 acceptance criteria.
    independent_v1 = validate_v1_dependency(V1Dependency(kind="data_durability"))
    obs7 = independent_v1.outcome
    exp7 = CONTRACTION_2528_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CONTRACTION_2528_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    complete = review_completeness(
        ContractionDecisions(
            durability="catalog version before cutover",
            routing="target catalog after cutover",
            rollback="source retained before cutover",
            pvc_retention="retain or separately reclaim",
        )
    )

    # 8. AC1 -- all four review decisions leave no named gap.
    obs8 = complete.missing_decisions
    exp8 = CONTRACTION_2528_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CONTRACTION_2528_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    children = implementation_children_allowed(measured)

    # 9. AC2 -- only the actual passing gate result permits child work.
    obs9 = children.outcome
    exp9 = CONTRACTION_2528_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CONTRACTION_2528_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    retired = decide_contraction(
        ContractionState(
            phase="CUTOVER",
            catalog_from=7,
            catalog_to=8,
            live_data_consolidated=True,
            wal_consolidated=True,
            cutover_committed=True,
            rollback_requested=False,
        )
    )

    # 10. R1 -- source retirement is eligible only after the cutover is committed.
    obs10 = retired.source_retirement_status
    exp10 = CONTRACTION_2528_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CONTRACTION_2528_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "contraction-2528-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(check["passed"] for check in checks) and len(checks) == MINIMUM_CHECKS,
    }
