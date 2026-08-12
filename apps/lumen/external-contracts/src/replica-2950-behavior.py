"""EC behavior case for #2950 -- non-voting read-replica decisions.

Every expected value is an EC-owned literal transcribed from #2950 R1--R7.
R1/R7 preserve voter and shard policy while changing the requested non-voting
replica count; R2/R3 qualify and route reads; R4 orders the scale-in lifecycle;
R5 selects exactly a safely removed reconstructible replica; and R6 preserves
rollback before commitment while closing routing after it.  The imports are
deliberately present before the pure design exists, so this case fails closed.
"""

from __future__ import annotations

from lumen.topology.read_routing import (
    ReadTargetRequest,
    ReplicaEligibilityRequest,
    ReplicaReadCandidate,
    decide_read_target,
    replica_eligibility,
)
from lumen.topology.replica import ReplicaChangeRequest, TopologyPolicy, decide_replica_change
from lumen.topology.replica_reclamation import ReclamationRequest, select
from lumen.topology.replica_scale_in import ScaleInAdvanceRequest, ScaleInResumeRequest, advance, resume

MINIMUM_CHECKS = 20

REPLICA_2950_BEHAVIOR_MATRIX = (
    ("scale_out_preserves_one_voter_policy", 1),
    ("scale_out_preserves_shard_policy", 2),
    ("scale_out_changes_only_read_replica_target", 3),
    ("zero_read_replicas_is_an_admitted_target", 0),
    ("zero_replica_target_preserves_voter_policy", 1),
    ("zero_replica_target_preserves_shard_policy", 2),
    ("caught_up_replica_at_lag_bound_is_eligible", "eligible"),
    ("incomplete_catch_up_is_ineligible", "catch_up_incomplete"),
    ("lag_above_bound_is_ineligible", "apply_lag_exceeds_bound"),
    ("leader_read_targets_leader", "leader-a"),
    ("strong_read_targets_leader", "leader-a"),
    ("any_read_can_target_eligible_replica", "replica-a"),
    ("bounded_read_can_target_eligible_replica", "replica-a"),
    ("drain_advances_only_to_non_voter_removal", "remove_non_voter"),
    ("removal_advances_only_to_tombstone", "record_tombstone"),
    ("tombstone_advances_only_to_grace_check", "rollback_grace_check"),
    ("safe_reclamation_selects_reclaim_action", "reclaim"),
    ("safe_reclamation_selects_exact_identity", "replica-a"),
    ("safe_reclamation_selects_exact_generation", 7),
    ("precommit_interruption_retains_rollback", "retain_for_rollback"),
)


def verify_replica_2950_behavior() -> dict:
    checks = []
    policy = TopologyPolicy(voters=1, shards=2, read_replicas=1)

    scaled = decide_replica_change(ReplicaChangeRequest(policy=policy, target_read_replicas=3))
    # 1-3. R1 -- a replica-only change carries voter/shard policy unchanged.
    obs1 = scaled.target.voters; exp1 = REPLICA_2950_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = scaled.target.shards; exp2 = REPLICA_2950_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = scaled.target.read_replicas; exp3 = REPLICA_2950_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    zero = decide_replica_change(ReplicaChangeRequest(policy=policy, target_read_replicas=0))
    # 4-6. R1/R7 -- zero is explicit and preserves voter/shard policy too.
    obs4 = zero.target.read_replicas; exp4 = REPLICA_2950_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = zero.target.voters; exp5 = REPLICA_2950_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = zero.target.shards; exp6 = REPLICA_2950_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    eligible = replica_eligibility(ReplicaEligibilityRequest(catch_up_complete=True, advertised_apply_lag=25, lag_bound=25))
    incomplete = replica_eligibility(ReplicaEligibilityRequest(catch_up_complete=False, advertised_apply_lag=0, lag_bound=25))
    lagging = replica_eligibility(ReplicaEligibilityRequest(catch_up_complete=True, advertised_apply_lag=26, lag_bound=25))
    # 7-9. R2 -- equality is eligible; each independent failed predicate is not.
    obs7 = eligible.status; exp7 = REPLICA_2950_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = incomplete.status; exp8 = REPLICA_2950_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = lagging.status; exp9 = REPLICA_2950_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    candidates = (ReplicaReadCandidate(identity="replica-a", catch_up_complete=True, advertised_apply_lag=25, lag_bound=25),)
    leader = decide_read_target(ReadTargetRequest(consistency="leader", leader_identity="leader-a", replicas=candidates))
    strong = decide_read_target(ReadTargetRequest(consistency="strong", leader_identity="leader-a", replicas=candidates))
    any_read = decide_read_target(ReadTargetRequest(consistency="any", leader_identity="leader-a", replicas=candidates))
    bounded = decide_read_target(ReadTargetRequest(consistency="bounded", leader_identity="leader-a", replicas=candidates))
    # 10-13. R3 -- leader/strong forward to the leader; eligible replicas serve any/bounded.
    obs10 = leader.target_identity; exp10 = REPLICA_2950_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    obs11 = strong.target_identity; exp11 = REPLICA_2950_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = any_read.target_identity; exp12 = REPLICA_2950_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = bounded.target_identity; exp13 = REPLICA_2950_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    drained = advance(ScaleInAdvanceRequest(current_phase="routing_drain", requested_phase="remove_non_voter"))
    removed = advance(ScaleInAdvanceRequest(current_phase="remove_non_voter", requested_phase="record_tombstone"))
    tombstoned = advance(ScaleInAdvanceRequest(current_phase="record_tombstone", requested_phase="rollback_grace_check"))
    # 14-16. R4 -- each durable transition names its sole permitted successor.
    obs14 = drained.next_phase; exp14 = REPLICA_2950_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    obs15 = removed.next_phase; exp15 = REPLICA_2950_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    obs16 = tombstoned.next_phase; exp16 = REPLICA_2950_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    reclaim = select(ReclamationRequest(identity="replica-a", generation=7, role="non_voter", safely_removed=True, reconstructible=True, authoritative_handoff=False))
    # 17-19. R5 -- safe reclamation preserves action, identity, and generation.
    obs17 = reclaim.action; exp17 = REPLICA_2950_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    obs18 = reclaim.identity; exp18 = REPLICA_2950_BEHAVIOR_MATRIX[17][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})
    obs19 = reclaim.generation; exp19 = REPLICA_2950_BEHAVIOR_MATRIX[18][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    interrupted = resume(ScaleInResumeRequest(identity="replica-a", generation=7, committed_removal=False, requested_routing_reentry=True, reclaim_identity="replica-a", reclaim_generation=7))
    # 20. R6 -- an explicit pre-commit interruption leaves a bounded rollback path.
    obs20 = interrupted.outcome; exp20 = REPLICA_2950_BEHAVIOR_MATRIX[19][1]
    checks.append({"name": REPLICA_2950_BEHAVIOR_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    return {"case_id": "replica-2950-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(check["passed"] for check in checks) and len(checks) == MINIMUM_CHECKS}
