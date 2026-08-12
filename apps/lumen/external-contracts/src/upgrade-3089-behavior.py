"""EC behavior case for #3089 -- one-voter learner handoff.

Every expected value below is an EC-owned literal transcribed from #3089:
R1 uses the Provision, Learner, CatchUp, Cutover, Drain, and Cleanup handoff
phases; R2 admits a target on a distinct approved node; R3 gates cutover on
snapshot/log compatibility, applied watermark, and readiness; R4 preserves the
committed configuration while promoting the target; R5 switches stable routing
only after membership commit and drains before old-voter removal; R6 recovers
backward before cutover and forward after it; R7 retains the old authoritative
PVC through verification; and R8 recognizes only the exact steady inventory.
"""

from __future__ import annotations

from lumen.topology.upgrade_handoff import (
    decide_membership_transition,
    decide_recovery,
    decide_upgrade_start,
    decide_upgrade_transition,
    reclaimable_resources,
)
from lumen.topology.upgrade_spec import (
    MembershipTransition,
    NodeCapacityCandidate,
    RecoveryRequest,
    TemporaryResource,
    UpgradeEvidence,
    UpgradePhase,
    UpgradeStartRequest,
    UpgradeTransitionRequest,
)
from lumen.topology.upgrade_status import UpgradeStatus, is_converged
from lumen.topology.upgrade_verdict import AdmittedUpgrade, ReclamationPlan

MINIMUM_CHECKS = 15

UPGRADE_3089_BEHAVIOR_MATRIX = (
    ("provision_advances_to_learner", "Learner"),
    ("learner_advances_to_catch_up", "CatchUp"),
    ("ready_target_advances_to_cutover", "Cutover"),
    ("distinct_approved_target_node_is_admitted", "node-b"),
    ("membership_promotion_preserves_committed_configuration", "config-7"),
    ("membership_promotion_makes_target_the_next_authority", "target_voter"),
    ("stable_route_switch_follows_membership_commit", True),
    ("stable_route_switch_drains_before_old_removal", "Drain"),
    ("drain_advances_to_cleanup", "Cleanup"),
    ("cleanup_is_terminal_and_idempotent", "Cleanup"),
    ("pre_cutover_failure_rolls_back_to_old_authority", "RollbackToOldAuthority"),
    ("post_cutover_failure_completes_forward", "CompleteForward"),
    ("old_authoritative_pvc_is_retained_through_verification", ("pvc-old",)),
    ("only_prior_non_authoritative_temporary_resource_is_reclaimable", ("learner-temp",)),
    ("only_exact_steady_inventory_is_converged", True),
)


def _phase(verdict) -> str:
    return verdict.next_phase.value if isinstance(verdict, AdmittedUpgrade) else "refused"


def verify_upgrade_3089_behavior() -> dict:
    checks = []
    ready = UpgradeEvidence(
        compatible_snapshot_log=True,
        required_applied_watermark=True,
        target_ready=True,
    )

    # 1. R1 -- the persisted handoff starts by provisioning the learner path.
    provision = decide_upgrade_transition(
        UpgradeTransitionRequest(phase=UpgradePhase.PROVISION, evidence=ready)
    )
    obs1 = _phase(provision)
    exp1 = UPGRADE_3089_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- learner setup cannot skip the explicit catch-up phase.
    learner = decide_upgrade_transition(
        UpgradeTransitionRequest(phase=UpgradePhase.LEARNER, evidence=ready)
    )
    obs2 = _phase(learner)
    exp2 = UPGRADE_3089_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R3 -- only all three evidence predicates permit the cutover phase.
    caught_up = decide_upgrade_transition(
        UpgradeTransitionRequest(phase=UpgradePhase.CATCH_UP, evidence=ready)
    )
    obs3 = _phase(caught_up)
    exp3 = UPGRADE_3089_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R2 -- a named approved target on a different node is admitted.
    started = decide_upgrade_start(
        UpgradeStartRequest(
            old_voter_node="node-a",
            candidates=(NodeCapacityCandidate(node="node-b", approved_capacity_id="pool-b"),),
        )
    )
    obs4 = started.target_node if isinstance(started, AdmittedUpgrade) else "blocked"
    exp4 = UPGRADE_3089_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R4 -- promotion carries the supplied committed configuration verbatim.
    promoted = decide_membership_transition(
        MembershipTransition(
            committed_configuration="config-7",
            current_authority="old_voter",
            requested_authority="target_voter",
            voters=("old", "target"),
            learners=(),
            quorum_members=("old", "target"),
        )
    )
    obs5 = promoted.committed_configuration if isinstance(promoted, AdmittedUpgrade) else "refused"
    exp5 = UPGRADE_3089_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R4 -- authority is transferred to the promoted target, not a learner.
    obs6 = promoted.next_authority_role if isinstance(promoted, AdmittedUpgrade) else "refused"
    exp6 = UPGRADE_3089_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R5 -- the transition records the stable-route switch only after commit.
    committed = decide_upgrade_transition(
        UpgradeTransitionRequest(
            phase=UpgradePhase.CUTOVER,
            evidence=ready,
            membership_committed=True,
            stable_route_switched=False,
        )
    )
    obs7 = committed.stable_route_switch if isinstance(committed, AdmittedUpgrade) else False
    exp7 = UPGRADE_3089_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R5 -- after that switch, old-voter removal first enters Drain.
    routed = decide_upgrade_transition(
        UpgradeTransitionRequest(
            phase=UpgradePhase.CUTOVER,
            evidence=ready,
            membership_committed=True,
            stable_route_switched=True,
        )
    )
    obs8 = _phase(routed)
    exp8 = UPGRADE_3089_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R1/R5 -- the drained old process proceeds only to Cleanup.
    drained = decide_upgrade_transition(
        UpgradeTransitionRequest(phase=UpgradePhase.DRAIN, evidence=ready)
    )
    obs9 = _phase(drained)
    exp9 = UPGRADE_3089_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R1/R8 -- retrying terminal Cleanup cannot re-enter the handoff.
    cleaned_up = decide_upgrade_transition(
        UpgradeTransitionRequest(phase=UpgradePhase.CLEANUP, evidence=ready)
    )
    obs10 = _phase(cleaned_up)
    exp10 = UPGRADE_3089_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R6 -- a failed target before cutover restores old authority.
    before_cutover = decide_recovery(
        RecoveryRequest(persisted_phase=UpgradePhase.CATCH_UP, cutover_complete=False, target_failed=True)
    )
    obs11 = before_cutover.action.value
    exp11 = UPGRADE_3089_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R6 -- a failed target after cutover resumes completion rather than rollback.
    after_cutover = decide_recovery(
        RecoveryRequest(persisted_phase=UpgradePhase.DRAIN, cutover_complete=True, target_failed=True)
    )
    obs12 = after_cutover.action.value
    exp12 = UPGRADE_3089_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R7 -- verification precedes disposal of the old authoritative PVC.
    reclamation = reclaimable_resources(
        resources=(
            TemporaryResource(name="pvc-old", role="authoritative", generation=7),
            TemporaryResource(name="learner-temp", role="learner", generation=6),
        ),
        post_cutover_verified=False,
        authoritative_generation=7,
    )
    obs13 = reclamation.retained_resources if isinstance(reclamation, ReclamationPlan) else ()
    exp13 = UPGRADE_3089_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R7 -- only the explicitly non-authoritative earlier-generation learner is reclaimable.
    verified_reclamation = reclaimable_resources(
        resources=(
            TemporaryResource(name="pvc-old", role="authoritative", generation=7),
            TemporaryResource(name="learner-temp", role="learner", generation=6),
        ),
        post_cutover_verified=True,
        authoritative_generation=7,
    )
    obs14 = verified_reclamation.reclaimable_resources if isinstance(verified_reclamation, ReclamationPlan) else ()
    exp14 = UPGRADE_3089_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R8 -- one voter, one stable generation, no learner or orphan is exact convergence.
    obs15 = is_converged(
        UpgradeStatus(voters=1, stable_workload_generations=1, learners=0, unowned_temporary_resources=0)
    )
    exp15 = UPGRADE_3089_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": UPGRADE_3089_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "upgrade-3089-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
