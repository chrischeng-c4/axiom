"""EC security case for #2950 -- fail-closed non-voting replica boundaries.

Expected literals are owned by this EC and transcribed from #2950 R1--R6.
The rows exercise every design entry point that can admit unsafe replica
membership, routing, lifecycle, reclamation, or resume state.  Missing design
imports are intentional: the contract remains red until the separate design
worker supplies these pure entry points.
"""

from __future__ import annotations

from lumen.topology.read_routing import ReadTargetRequest, ReplicaEligibilityRequest, ReplicaReadCandidate, decide_read_target, replica_eligibility
from lumen.topology.replica import ReplicaChangeRequest, TopologyPolicy, decide_replica_change
from lumen.topology.replica_reclamation import ReclamationRequest, select
from lumen.topology.replica_scale_in import ScaleInAdvanceRequest, ScaleInResumeRequest, advance, resume

MINIMUM_CHECKS = 23

REPLICA_2950_SECURITY_MATRIX = (
    ("negative_replica_target_is_refused", "negative_read_replicas"),
    ("negative_replica_target_names_target_field", "target_read_replicas"),
    ("zero_replica_target_remains_admitted", 0),
    ("incomplete_replica_is_refused_by_read_routing", "replica_not_eligible"),
    ("incomplete_routing_refusal_names_candidate", "replicas[0].catch_up_complete"),
    ("incomplete_eligibility_names_catch_up_field", "catch_up_complete"),
    ("lagging_eligibility_names_lag_field", "advertised_apply_lag"),
    ("lagging_replica_is_refused_by_bounded_routing", "replica_not_eligible"),
    ("at_bound_replica_remains_routable", "replica-a"),
    ("skipped_scale_in_phase_is_refused", "required_predecessor_skipped"),
    ("skipped_phase_refusal_names_requested_phase", "requested_phase"),
    ("ordered_scale_in_neighbour_is_admitted", "remove_non_voter"),
    ("voter_pvc_is_refused", "protected_role"),
    ("voter_pvc_refusal_names_role", "role"),
    ("shard_pvc_is_refused", "protected_role"),
    ("authoritative_handoff_pvc_is_refused", "authoritative_handoff"),
    ("safe_non_voter_neighbour_is_reclaimable", "reclaim"),
    ("committed_removal_refuses_routing_reentry", "routing_reentry_after_commit"),
    ("committed_reentry_refusal_names_commit_state", "committed_removal"),
    ("reused_identity_is_not_reclaimable", "identity_mismatch"),
    ("reused_identity_refusal_names_reclaim_identity", "reclaim_identity"),
    ("reused_generation_is_not_reclaimable", "generation_mismatch"),
    ("reused_generation_refusal_names_reclaim_generation", "reclaim_generation"),
)


def verify_replica_2950_security() -> dict:
    checks = []
    policy = TopologyPolicy(voters=3, shards=2, read_replicas=1)

    negative = decide_replica_change(ReplicaChangeRequest(policy=policy, target_read_replicas=-1))
    # 1-3. R1/R7 -- explicit invalid and zero targets must not share a default path.
    obs1 = negative.reason; exp1 = REPLICA_2950_SECURITY_MATRIX[0][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = negative.field_path; exp2 = REPLICA_2950_SECURITY_MATRIX[1][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    zero = decide_replica_change(ReplicaChangeRequest(policy=policy, target_read_replicas=0))
    obs3 = zero.target.read_replicas; exp3 = REPLICA_2950_SECURITY_MATRIX[2][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    incomplete_candidate = (ReplicaReadCandidate(identity="replica-a", catch_up_complete=False, advertised_apply_lag=0, lag_bound=25),)
    incomplete_route = decide_read_target(ReadTargetRequest(consistency="any", leader_identity="leader-a", replicas=incomplete_candidate))
    incomplete = replica_eligibility(ReplicaEligibilityRequest(catch_up_complete=False, advertised_apply_lag=0, lag_bound=25))
    lagging = replica_eligibility(ReplicaEligibilityRequest(catch_up_complete=True, advertised_apply_lag=26, lag_bound=25))
    lagging_route = decide_read_target(ReadTargetRequest(consistency="bounded", leader_identity="leader-a", replicas=(ReplicaReadCandidate(identity="replica-a", catch_up_complete=True, advertised_apply_lag=26, lag_bound=25),)))
    at_bound = decide_read_target(ReadTargetRequest(consistency="bounded", leader_identity="leader-a", replicas=(ReplicaReadCandidate(identity="replica-a", catch_up_complete=True, advertised_apply_lag=25, lag_bound=25),)))
    # 4-9. R2/R3 -- routing and eligibility each reject failed predicates, while equality remains usable.
    obs4 = incomplete_route.reason; exp4 = REPLICA_2950_SECURITY_MATRIX[3][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = incomplete_route.field_path; exp5 = REPLICA_2950_SECURITY_MATRIX[4][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = incomplete.field_path; exp6 = REPLICA_2950_SECURITY_MATRIX[5][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = lagging.field_path; exp7 = REPLICA_2950_SECURITY_MATRIX[6][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = lagging_route.reason; exp8 = REPLICA_2950_SECURITY_MATRIX[7][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = at_bound.target_identity; exp9 = REPLICA_2950_SECURITY_MATRIX[8][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    skipped = advance(ScaleInAdvanceRequest(current_phase="routing_drain", requested_phase="record_tombstone"))
    ordered = advance(ScaleInAdvanceRequest(current_phase="routing_drain", requested_phase="remove_non_voter"))
    # 10-12. R4 -- tombstone cannot bypass committed non-voter removal.
    obs10 = skipped.reason; exp10 = REPLICA_2950_SECURITY_MATRIX[9][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    obs11 = skipped.field_path; exp11 = REPLICA_2950_SECURITY_MATRIX[10][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = ordered.next_phase; exp12 = REPLICA_2950_SECURITY_MATRIX[11][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    voter = select(ReclamationRequest(identity="voter-a", generation=7, role="voter", safely_removed=True, reconstructible=True, authoritative_handoff=False))
    shard = select(ReclamationRequest(identity="shard-a", generation=7, role="shard", safely_removed=True, reconstructible=True, authoritative_handoff=False))
    handoff = select(ReclamationRequest(identity="replica-a", generation=7, role="non_voter", safely_removed=True, reconstructible=True, authoritative_handoff=True))
    safe = select(ReclamationRequest(identity="replica-a", generation=7, role="non_voter", safely_removed=True, reconstructible=True, authoritative_handoff=False))
    # 13-17. R5 -- role and live authority independently protect PVCs.
    obs13 = voter.reason; exp13 = REPLICA_2950_SECURITY_MATRIX[12][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = voter.field_path; exp14 = REPLICA_2950_SECURITY_MATRIX[13][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    obs15 = shard.reason; exp15 = REPLICA_2950_SECURITY_MATRIX[14][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    obs16 = handoff.reason; exp16 = REPLICA_2950_SECURITY_MATRIX[15][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    obs17 = safe.action; exp17 = REPLICA_2950_SECURITY_MATRIX[16][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    committed = resume(ScaleInResumeRequest(identity="replica-a", generation=7, committed_removal=True, requested_routing_reentry=True, reclaim_identity="replica-a", reclaim_generation=7))
    reused_identity = resume(ScaleInResumeRequest(identity="replica-a", generation=7, committed_removal=True, requested_routing_reentry=False, reclaim_identity="replica-b", reclaim_generation=7))
    reused_generation = resume(ScaleInResumeRequest(identity="replica-a", generation=7, committed_removal=True, requested_routing_reentry=False, reclaim_identity="replica-a", reclaim_generation=8))
    # 18-23. R6 -- commitment closes routing and binds cleanup to identity plus generation.
    obs18 = committed.reason; exp18 = REPLICA_2950_SECURITY_MATRIX[17][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})
    obs19 = committed.field_path; exp19 = REPLICA_2950_SECURITY_MATRIX[18][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})
    obs20 = reused_identity.reason; exp20 = REPLICA_2950_SECURITY_MATRIX[19][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})
    obs21 = reused_identity.field_path; exp21 = REPLICA_2950_SECURITY_MATRIX[20][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})
    obs22 = reused_generation.reason; exp22 = REPLICA_2950_SECURITY_MATRIX[21][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})
    obs23 = reused_generation.field_path; exp23 = REPLICA_2950_SECURITY_MATRIX[22][1]
    checks.append({"name": REPLICA_2950_SECURITY_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})

    return {"case_id": "replica-2950-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(check["passed"] for check in checks) and len(checks) == MINIMUM_CHECKS}
