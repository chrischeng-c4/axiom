"""EC security case for #3090 -- fail-closed replicated-shard rollout.

Expected literals are EC-owned values from #3090: R1 admits advance only with
committed membership, quorum health, and caught-up facts; R3/R4 refuse unsafe
leader work; R5 rejects two unavailable voters even in joint configuration;
R6 refuses inconsistent reads; R7 freezes later shards after failure; and R8
refuses replica work or cleanup before voter convergence.
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

MINIMUM_CHECKS = 16

RELEASE_3090_SECURITY_MATRIX = (
    ("uncommitted_membership_refuses_advance", "membership_not_committed"),
    ("uncommitted_membership_names_membership", "committed_membership"),
    ("unhealthy_quorum_refuses_advance", "quorum_unhealthy"),
    ("unhealthy_quorum_names_quorum", "healthy_quorum"),
    ("applied_lag_refuses_advance", "applied_lag_not_caught_up"),
    ("applied_lag_names_member", "members.follower-a.applied_lag"),
    ("leader_replacement_is_refused_while_voting_follower_remains", "voting_follower_pending"),
    ("unconfirmed_leader_drain_is_refused", "leader_transfer_unconfirmed"),
    ("unconfirmed_leader_drain_names_leader_term", "confirmed_leader_term"),
    ("joint_configuration_rejects_two_unavailable_voters", "too_many_unavailable_voters"),
    ("joint_configuration_refusal_names_new_voters", "joint_new_voters"),
    ("inconsistent_read_is_refused", "read_consistency_unsatisfied"),
    ("inconsistent_read_refusal_names_declared_consistency", "request.consistency"),
    ("replacement_failure_refuses_later_shard_actions", "rollout_paused"),
    ("replica_work_before_voter_convergence_is_refused", "voters_not_converged"),
    ("cleanup_before_voter_convergence_is_refused", "voters_not_converged"),
)


def _reason(verdict):
    return verdict.reason.value if isinstance(verdict, Refusal) else "admitted"


def _members(*, follower_lag: int = 0, follower_release: str = "2.0.0", unavailable: tuple[str, ...] = ()):
    return (
        ReleaseMember("canary-a", "non_voting", "2.1.0", applied_lag=0, available="canary-a" not in unavailable),
        ReleaseMember("follower-a", "voter", follower_release, applied_lag=follower_lag, available="follower-a" not in unavailable),
        ReleaseMember("leader-a", "leader", "2.0.0", applied_lag=0, available="leader-a" not in unavailable),
        ReleaseMember("replica-a", "read_replica", "2.0.0", applied_lag=0, available=True),
    )


def _state(**overrides):
    values = {
        "members": _members(), "target_release": "2.1.0", "committed_membership": True,
        "healthy_quorum": True, "max_applied_lag": 0, "current_leader": "leader-a",
        "current_term": 7, "joint_old_voters": ("follower-a", "leader-a"),
        "joint_new_voters": ("follower-a", "leader-b"),
    }
    values.update(overrides)
    return ReleaseRolloutState(**values)


def verify_release_3090_security() -> dict:
    checks = []

    # 1-2. R1 -- membership must be the supplied committed membership fact.
    membership = decide_release_rollout(_state(committed_membership=False))
    obs1 = _reason(membership); exp1 = RELEASE_3090_SECURITY_MATRIX[0][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = membership.field_path if isinstance(membership, Refusal) else ""; exp2 = RELEASE_3090_SECURITY_MATRIX[1][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3-4. R1 -- a decision may not advance through an unhealthy quorum.
    quorum = decide_release_rollout(_state(healthy_quorum=False))
    obs3 = _reason(quorum); exp3 = RELEASE_3090_SECURITY_MATRIX[2][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = quorum.field_path if isinstance(quorum, Refusal) else ""; exp4 = RELEASE_3090_SECURITY_MATRIX[3][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5-6. R1 -- a named member over the supplied applied-lag bound stops rollout.
    lag = decide_release_rollout(_state(members=_members(follower_lag=1)))
    obs5 = _reason(lag); exp5 = RELEASE_3090_SECURITY_MATRIX[4][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = lag.field_path if isinstance(lag, Refusal) else ""; exp6 = RELEASE_3090_SECURITY_MATRIX[5][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R3 -- a request explicitly asking for a leader replacement cannot skip
    #    the voting follower still on the old release.
    leader_first = decide_release_rollout(_state(requested_member="leader-a"))
    obs7 = _reason(leader_first); exp7 = RELEASE_3090_SECURITY_MATRIX[6][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8-9. R4/AC2 -- transfer intent is insufficient; identity plus new term
    #      must confirm the new leader before draining the old one.
    unconfirmed = decide_release_rollout(_state(members=_members(follower_release="2.1.0"), requested_action="drain_member"))
    obs8 = _reason(unconfirmed); exp8 = RELEASE_3090_SECURITY_MATRIX[7][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = unconfirmed.field_path if isinstance(unconfirmed, Refusal) else ""; exp9 = RELEASE_3090_SECURITY_MATRIX[8][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10-11. R5 -- joint consensus protects both voter sets and rejects two
    #        unavailable voters, naming the new set that would lose quorum.
    joint = decide_disruption_protection(_state(members=_members(unavailable=("follower-a", "leader-a"))))
    obs10 = _reason(joint); exp10 = RELEASE_3090_SECURITY_MATRIX[9][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    obs11 = joint.field_path if isinstance(joint, Refusal) else ""; exp11 = RELEASE_3090_SECURITY_MATRIX[10][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12-13. R6 -- reads whose declared consistency cannot be met are named
    #        refusals, never a best-effort route.
    stale_read = decide_request_route(_state(members=_members(follower_lag=1)), ReleaseRequest("read", "bounded_lag", max_lag=0))
    obs12 = _reason(stale_read); exp12 = RELEASE_3090_SECURITY_MATRIX[11][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = stale_read.field_path if isinstance(stale_read, Refusal) else ""; exp13 = RELEASE_3090_SECURITY_MATRIX[12][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R7 -- a failure freezes later shards rather than continuing serially.
    paused = record_replacement_failure(_state(), "follower-a", "2.1.0", "image_pull_failed")
    later = decide_release_rollout(paused, shard_id="shard-b")
    obs14 = _reason(later); exp14 = RELEASE_3090_SECURITY_MATRIX[13][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15-16. R8 -- neither a replica operation nor temporary cleanup can make
    #        progress while an old voter remains.
    replica_early = decide_release_rollout(_state(members=_members(follower_release="2.1.0"), requested_action="upgrade_read_replica"))
    obs15 = _reason(replica_early); exp15 = RELEASE_3090_SECURITY_MATRIX[14][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    cleanup_early = decide_release_rollout(_state(members=_members(follower_release="2.1.0"), temporary_generations=("surge-a",), requested_action="cleanup_temporary_generation"))
    obs16 = _reason(cleanup_early); exp16 = RELEASE_3090_SECURITY_MATRIX[15][1]
    checks.append({"name": RELEASE_3090_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    return {"case_id": "release-3090-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
