"""EC behavior case for #3090 -- serial replicated-shard release rollout.

Every expected value below is an EC-owned literal transcribed from #3090:
R1 advances one eligible member, R2 uses a non-voting canary outside quorum,
R3 replaces voting followers before the leader, R4 transfers leadership to an
upgraded caught-up voter, R6 routes mixed-version traffic from supplied facts,
R7 records an exact paused failure, and R8 defers replica work and cleanup
until voters converge.  The rows deliberately inspect values, never a design
provided ``is_valid`` or ``passed`` flag.
"""

from __future__ import annotations

from lumen.topology.release_rollout import (
    ReleaseMember,
    ReleaseRequest,
    ReleaseRolloutState,
    decide_disruption_protection,
    decide_release_rollout,
    decide_request_route,
    record_replacement_failure,
)
from lumen.topology.release_rollout_verdict import Refusal

MINIMUM_CHECKS = 14

RELEASE_3090_BEHAVIOR_MATRIX = (
    ("advance_names_exactly_one_member", ("canary-a",)),
    ("non_voting_canary_precedes_voter_actions", "canary-a"),
    ("canary_is_excluded_from_derived_quorum_voters", ("follower-a", "leader-a")),
    ("voting_follower_is_selected_before_current_leader", "follower-a"),
    ("leadership_transfer_targets_upgraded_caught_up_voter", ("transfer_leadership", "follower-a")),
    ("confirmed_new_leader_can_be_drained", ("drain_member", "leader-a")),
    ("joint_configuration_preserves_old_and_new_voter_sets", (("follower-a", "leader-a"), ("follower-a", "leader-b"))),
    ("writes_route_to_supplied_current_leader", "leader-a"),
    ("bounded_consistent_reads_route_to_caught_up_follower", "follower-a"),
    ("failure_pause_records_member_exactly", "follower-a"),
    ("failure_pause_records_target_release_exactly", "2.1.0"),
    ("failure_pause_records_blocker_exactly", "image_pull_failed"),
    ("converged_voters_admit_read_replica_work", ("upgrade_read_replica", "replica-a")),
    ("converged_voters_admit_cleanup_without_temporary_generation", ("cleanup_temporary_generation", ())),
)


def _members(*, leader: str = "leader-a", follower_release: str = "2.0.0", canary_release: str = "2.0.0", replica_release: str = "2.0.0"):
    return (
        ReleaseMember("canary-a", "non_voting", canary_release, applied_lag=0, available=True),
        ReleaseMember("follower-a", "voter", follower_release, applied_lag=0, available=True),
        ReleaseMember(leader, "leader", "2.0.0", applied_lag=0, available=True),
        ReleaseMember("replica-a", "read_replica", replica_release, applied_lag=0, available=True),
    )


def _state(**overrides):
    values = {
        "members": _members(),
        "target_release": "2.1.0",
        "committed_membership": True,
        "healthy_quorum": True,
        "max_applied_lag": 0,
        "current_leader": "leader-a",
        "current_term": 7,
        "joint_old_voters": ("follower-a", "leader-a"),
        "joint_new_voters": ("follower-a", "leader-b"),
    }
    values.update(overrides)
    return ReleaseRolloutState(**values)


def verify_release_3090_behavior() -> dict:
    checks = []

    # 1. R1 -- one decision names one replacement, never a batch across shards.
    serial = decide_release_rollout(_state())
    obs1 = serial.member_ids if not isinstance(serial, Refusal) else ()
    exp1 = RELEASE_3090_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": RELEASE_3090_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2-3. R2 -- an available non-voter is the first compatibility canary and
    #      does not change the voter quorum calculation.
    obs2 = serial.member_id if not isinstance(serial, Refusal) else "refused"
    exp2 = RELEASE_3090_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": RELEASE_3090_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = serial.quorum_voters if not isinstance(serial, Refusal) else ()
    exp3 = RELEASE_3090_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": RELEASE_3090_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R3 -- once the canary is target-release, the voting follower comes
    #    before the still-old current leader.
    follower = decide_release_rollout(_state(members=_members(canary_release="2.1.0")))
    obs4 = follower.member_id if not isinstance(follower, Refusal) else "refused"
    exp4 = RELEASE_3090_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": RELEASE_3090_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R4 -- an upgraded, caught-up voter is the only transfer target.
    transfer = decide_release_rollout(_state(members=_members(canary_release="2.1.0", follower_release="2.1.0")))
    obs5 = (transfer.kind, transfer.member_id) if not isinstance(transfer, Refusal) else ("refused", "")
    exp5 = RELEASE_3090_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": RELEASE_3090_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R4/AC2 -- the old leader may drain only after the supplied identity
    #    and newer term confirm the upgraded voter became leader.
    drained = decide_release_rollout(_state(members=_members(leader="follower-a", canary_release="2.1.0", follower_release="2.1.0"), current_leader="follower-a", current_term=8, confirmed_leader_term=8))
    obs6 = (drained.kind, drained.member_id) if not isinstance(drained, Refusal) else ("refused", "")
    exp6 = RELEASE_3090_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": RELEASE_3090_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R5 -- joint configuration derives protection from both voter sets.
    protection = decide_disruption_protection(_state())
    obs7 = (protection.old_voters, protection.new_voters) if not isinstance(protection, Refusal) else ((), ())
    exp7 = RELEASE_3090_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": RELEASE_3090_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R6 -- all mixed-version writes go to the supplied current leader.
    write = decide_request_route(_state(), ReleaseRequest("write", "linearizable", max_lag=0))
    obs8 = write.member_id if not isinstance(write, Refusal) else "refused"
    exp8 = RELEASE_3090_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": RELEASE_3090_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R6 -- a read with declared bounded lag may use the supplied caught-up follower.
    read = decide_request_route(_state(), ReleaseRequest("read", "bounded_lag", max_lag=0))
    obs9 = read.member_id if not isinstance(read, Refusal) else "refused"
    exp9 = RELEASE_3090_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": RELEASE_3090_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10-12. R7 -- pause state preserves the supplied causal tuple exactly.
    paused = record_replacement_failure(_state(), "follower-a", "2.1.0", "image_pull_failed")
    obs10 = paused.blocked_member
    exp10 = RELEASE_3090_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": RELEASE_3090_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    obs11 = paused.target_release
    exp11 = RELEASE_3090_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": RELEASE_3090_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = paused.blocker
    exp12 = RELEASE_3090_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": RELEASE_3090_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13-14. R8 -- only after all voters converge are replica work and cleanup
    #       admitted; cleanup leaves no desired temporary generation.
    converged = _state(members=_members(canary_release="2.1.0", follower_release="2.1.0", replica_release="2.0.0", leader="leader-a"), current_leader="leader-a", confirmed_leader_term=7)
    replica_work = decide_release_rollout(converged)
    obs13 = (replica_work.kind, replica_work.member_id) if not isinstance(replica_work, Refusal) else ("refused", "")
    exp13 = RELEASE_3090_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": RELEASE_3090_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    cleanup = decide_release_rollout(_state(members=_members(canary_release="2.1.0", follower_release="2.1.0", replica_release="2.1.0"), current_leader="leader-a", confirmed_leader_term=7, temporary_generations=("surge-a",)))
    obs14 = (cleanup.kind, cleanup.desired_temporary_generations) if not isinstance(cleanup, Refusal) else ("refused", ())
    exp14 = RELEASE_3090_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": RELEASE_3090_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    return {"case_id": "release-3090-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
