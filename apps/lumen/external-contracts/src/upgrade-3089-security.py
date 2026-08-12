"""EC security case for #3089 -- fail-closed one-voter learner handoff.

Expected values are EC-owned literals from #3089: R1 refuses unrecognized or
bypassing phases; R2 blocks collocated or unapproved targets and names missing
approved capacity; R3 refuses cutover without every evidence predicate; R4
refuses learner quorum and uncommitted configuration changes; R5 refuses a
route switch before membership commit; R7 limits reclamation to non-authoritative
resources from a proven earlier generation; and R8 does not overclaim convergence.
"""

from __future__ import annotations

from lumen.topology.upgrade_handoff import (
    decide_membership_transition,
    decide_upgrade_start,
    decide_upgrade_transition,
    reclaimable_resources,
)
from lumen.topology.upgrade_spec import (
    MembershipTransition,
    NodeCapacityCandidate,
    TemporaryResource,
    UpgradeEvidence,
    UpgradePhase,
    UpgradeStartRequest,
    UpgradeTransitionRequest,
)
from lumen.topology.upgrade_status import UpgradeStatus, is_converged
from lumen.topology.upgrade_verdict import ReclamationPlan, RefusedUpgrade, UpgradeBlocked

MINIMUM_CHECKS = 18

UPGRADE_3089_SECURITY_MATRIX = (
    ("unknown_handoff_phase_is_refused", "unrecognized_phase"),
    ("unknown_handoff_refusal_names_phase", "phase"),
    ("collocated_target_is_blocked", "missing_approved_capacity"),
    ("collocated_target_block_names_capacity_identifier", "pool-b"),
    ("distinct_unapproved_target_is_blocked", "missing_approved_capacity"),
    ("catch_up_without_compatible_snapshot_log_does_not_cut_over", "CatchUp"),
    ("catch_up_without_required_watermark_does_not_cut_over", "CatchUp"),
    ("catch_up_without_readiness_does_not_cut_over", "CatchUp"),
    ("learner_in_quorum_is_refused", "learner_in_quorum"),
    ("learner_quorum_refusal_names_quorum_members", "quorum_members"),
    ("out_of_transition_configuration_change_is_refused", "committed_configuration_changed"),
    ("configuration_change_refusal_names_committed_configuration", "committed_configuration"),
    ("precommit_route_switch_is_refused", "membership_not_committed"),
    ("authoritative_or_current_generation_resource_is_not_reclaimable", ()),
    ("extra_voter_is_not_converged", False),
    ("multiple_stable_generations_are_not_converged", False),
    ("learner_is_not_converged", False),
    ("unowned_temporary_resource_is_not_converged", False),
)


def _reason(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, (RefusedUpgrade, UpgradeBlocked)) else "admitted"


def verify_upgrade_3089_security() -> dict:
    checks = []

    # 1. R1 -- a fabricated phase cannot create an upgrade-only bypass.
    unknown = decide_upgrade_transition(
        UpgradeTransitionRequest(phase="RestartOld", evidence=UpgradeEvidence(True, True, True))
    )
    obs1 = _reason(unknown)
    exp1 = UPGRADE_3089_SECURITY_MATRIX[0][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- the refusal tells the caller the bypassing handoff field.
    obs2 = unknown.field_path if isinstance(unknown, RefusedUpgrade) else ""
    exp2 = UPGRADE_3089_SECURITY_MATRIX[1][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R2 -- approved capacity on the old node is not an eligible target.
    collocated = decide_upgrade_start(
        UpgradeStartRequest(
            old_voter_node="node-a",
            candidates=(NodeCapacityCandidate(node="node-a", approved_capacity_id="pool-b"),),
        )
    )
    obs3 = _reason(collocated)
    exp3 = UPGRADE_3089_SECURITY_MATRIX[2][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R2 -- blockage identifies the missing approved-capacity allocation.
    obs4 = collocated.missing_approved_capacity_id if isinstance(collocated, UpgradeBlocked) else ""
    exp4 = UPGRADE_3089_SECURITY_MATRIX[3][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R2 -- a different node without an explicit approval is blocked too.
    unapproved = decide_upgrade_start(
        UpgradeStartRequest(
            old_voter_node="node-a",
            candidates=(NodeCapacityCandidate(node="node-b", approved_capacity_id=None),),
        )
    )
    obs5 = _reason(unapproved)
    exp5 = UPGRADE_3089_SECURITY_MATRIX[4][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R3 -- missing compatible snapshot/log evidence keeps CatchUp active.
    no_snapshot = decide_upgrade_transition(
        UpgradeTransitionRequest(phase=UpgradePhase.CATCH_UP, evidence=UpgradeEvidence(False, True, True))
    )
    obs6 = no_snapshot.next_phase.value
    exp6 = UPGRADE_3089_SECURITY_MATRIX[5][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R3 -- snapshot compatibility alone cannot substitute for the watermark.
    no_watermark = decide_upgrade_transition(
        UpgradeTransitionRequest(phase=UpgradePhase.CATCH_UP, evidence=UpgradeEvidence(True, False, True))
    )
    obs7 = no_watermark.next_phase.value
    exp7 = UPGRADE_3089_SECURITY_MATRIX[6][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R3 -- neither can substitute for a ready target.
    not_ready = decide_upgrade_transition(
        UpgradeTransitionRequest(phase=UpgradePhase.CATCH_UP, evidence=UpgradeEvidence(True, True, False))
    )
    obs8 = not_ready.next_phase.value
    exp8 = UPGRADE_3089_SECURITY_MATRIX[7][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R4 -- learners replicate but never count toward a voter quorum.
    learner_quorum = decide_membership_transition(
        MembershipTransition(
            committed_configuration="config-7",
            current_authority="old_voter",
            requested_authority="target_voter",
            voters=("old",),
            learners=("target",),
            quorum_members=("old", "target"),
        )
    )
    obs9 = _reason(learner_quorum)
    exp9 = UPGRADE_3089_SECURITY_MATRIX[8][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R4 -- the refusal identifies the quorum input rather than a generic error.
    obs10 = learner_quorum.field_path if isinstance(learner_quorum, RefusedUpgrade) else ""
    exp10 = UPGRADE_3089_SECURITY_MATRIX[9][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R4 -- a membership step cannot silently replace the persisted config.
    changed_configuration = decide_membership_transition(
        MembershipTransition(
            committed_configuration="config-7",
            supplied_configuration="config-8",
            current_authority="old_voter",
            requested_authority="target_voter",
            voters=("old", "target"),
            learners=(),
            quorum_members=("old", "target"),
        )
    )
    obs11 = _reason(changed_configuration)
    exp11 = UPGRADE_3089_SECURITY_MATRIX[10][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R4 -- that refusal identifies the persisted configuration boundary.
    obs12 = changed_configuration.field_path if isinstance(changed_configuration, RefusedUpgrade) else ""
    exp12 = UPGRADE_3089_SECURITY_MATRIX[11][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R5 -- a route cannot become stable before membership is committed.
    precommit = decide_upgrade_transition(
        UpgradeTransitionRequest(
            phase=UpgradePhase.CUTOVER,
            evidence=UpgradeEvidence(True, True, True),
            membership_committed=False,
            stable_route_switched=True,
        )
    )
    obs13 = _reason(precommit)
    exp13 = UPGRADE_3089_SECURITY_MATRIX[12][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R7 -- neither the authoritative role nor its current generation is reclaimable.
    current_authoritative = reclaimable_resources(
        resources=(
            TemporaryResource(name="pvc-old", role="authoritative", generation=7),
            TemporaryResource(name="target-current", role="temporary", generation=7),
        ),
        post_cutover_verified=True,
        authoritative_generation=7,
    )
    obs14 = current_authoritative.reclaimable_resources if isinstance(current_authoritative, ReclamationPlan) else ()
    exp14 = UPGRADE_3089_SECURITY_MATRIX[13][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R8 -- a second voter is not the specified one-voter steady state.
    obs15 = is_converged(
        UpgradeStatus(voters=2, stable_workload_generations=1, learners=0, unowned_temporary_resources=0)
    )
    exp15 = UPGRADE_3089_SECURITY_MATRIX[14][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16. R8 -- two stable workload generations leave an unfinished handoff.
    obs16 = is_converged(
        UpgradeStatus(voters=1, stable_workload_generations=2, learners=0, unowned_temporary_resources=0)
    )
    exp16 = UPGRADE_3089_SECURITY_MATRIX[15][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    # 17. R8 -- an attached learner prevents the convergence claim.
    obs17 = is_converged(
        UpgradeStatus(voters=1, stable_workload_generations=1, learners=1, unowned_temporary_resources=0)
    )
    exp17 = UPGRADE_3089_SECURITY_MATRIX[16][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    # 18. R8 -- an orphaned temporary resource prevents the convergence claim.
    obs18 = is_converged(
        UpgradeStatus(voters=1, stable_workload_generations=1, learners=0, unowned_temporary_resources=1)
    )
    exp18 = UPGRADE_3089_SECURITY_MATRIX[17][1]
    checks.append({"name": UPGRADE_3089_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    return {
        "case_id": "upgrade-3089-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
