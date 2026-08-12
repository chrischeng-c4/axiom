"""EC behavior case for #2953 — R3, R4, R5, R6, R7, AC2, and AC4.

Every expected value is an EC-owned literal from the issue's pure-design
contract.  Benchmark collection, durable leader election, restart survival,
and manifest rendering are deliberately runtime-only and are not modeled here.
"""

from __future__ import annotations

from lumen.topology.control_plane_admission import (
    decide_control_plane_policy,
    decide_generation_commit,
    decide_initial_machine_and_transitions,
    decide_target_transition,
)
from lumen.topology.control_plane_spec import (
    CalibrationEvidence,
    ControlPlanePolicySpec,
    GenerationCommitIntent,
    GenerationCommitRequest,
    MachineBenchmark,
    TransitionHistory,
    TransitionPolicy,
    TransitionSnapshot,
)
from lumen.topology.control_plane_verdict import TargetPolicyVerdict

MINIMUM_CHECKS = 12

CONTROL_2953_BEHAVIOR_MATRIX = (
    ("complete_calibration_selects_the_initial_direct_machine", "n2-standard-4"),
    ("complete_calibration_retains_only_the_two_one_step_edges", (("n2-standard-4", "n2-standard-8"), ("n2-standard-4", "n2-highmem-4"))),
    ("cpu_backlog_edge_has_its_own_reason_class", "CPU_BACKLOG"),
    ("memory_highmem_edge_has_its_own_reason_class", "MEMORY_HIGHMEM"),
    ("elected_leader_with_expected_generation_commits", "COMMIT"),
    ("two_replica_non_hpa_policy_is_admitted", "ADMITTED"),
    ("admitted_policy_fixes_replica_count_at_two", 2),
    ("admitted_policy_disables_horizontal_autoscaling", False),
    ("cpu_backlog_pressure_selects_the_standard_step", "n2-standard-8"),
    ("memory_pressure_selects_the_highmem_step", "n2-highmem-4"),
    ("fully_post_convergence_headroom_allows_the_slow_downgrade", "DOWNGRADE"),
    ("eligible_downgrade_reports_the_standard_target", "e2-standard-2"),
)


def _calibration() -> CalibrationEvidence:
    """Name every benchmark value; no default can silently supply evidence."""
    return CalibrationEvidence(
        benchmarks=(
            MachineBenchmark("e2-standard-2", throughput=100, recovery_seconds=30, slo_headroom=20, hourly_cost=1),
            MachineBenchmark("n2-standard-4", throughput=160, recovery_seconds=25, slo_headroom=35, hourly_cost=2),
            MachineBenchmark("n2-standard-8", throughput=220, recovery_seconds=25, slo_headroom=45, hourly_cost=3),
            MachineBenchmark("n2-highmem-4", throughput=130, recovery_seconds=25, slo_headroom=40, hourly_cost=2),
        ),
        complete=True,
    )


def _policy() -> TransitionPolicy:
    return TransitionPolicy(
        initial_machine="n2-standard-4",
        allowed_edges=(("n2-standard-4", "n2-standard-8", "CPU_BACKLOG"), ("n2-standard-4", "n2-highmem-4", "MEMORY_HIGHMEM")),
        sustained_seconds=300,
        cooldown_seconds=600,
        daily_change_budget=2,
        spend_ceiling=10,
        slow_downgrade_seconds=1800,
        post_convergence_seconds=900,
    )


def _history(**overrides: object) -> TransitionHistory:
    values: dict[str, object] = {
        "last_change_at": 0,
        "changes_today": 0,
        "spend_today": 0,
        "converged_at": 0,
    }
    values.update(overrides)
    return TransitionHistory(**values)


def verify_control_2953_behavior() -> dict:
    checks = []

    # 1-4. R3 — complete CP calibration selects one direct E2 machine and the
    # two bounded one-step routes, retaining distinct reason classes.
    target_policy = decide_initial_machine_and_transitions(_calibration())
    obs1 = target_policy.initial_machine if isinstance(target_policy, TargetPolicyVerdict) else "REFUSED"
    exp1 = CONTROL_2953_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CONTROL_2953_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = target_policy.allowed_edges if isinstance(target_policy, TargetPolicyVerdict) else ()
    exp2 = CONTROL_2953_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CONTROL_2953_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = target_policy.allowed_edges[0].reason if isinstance(target_policy, TargetPolicyVerdict) else "REFUSED"
    exp3 = CONTROL_2953_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CONTROL_2953_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = target_policy.allowed_edges[1].reason if isinstance(target_policy, TargetPolicyVerdict) else "REFUSED"
    exp4 = CONTROL_2953_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CONTROL_2953_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R4 — leader and expected generation are explicit input, not defaults.
    commit = decide_generation_commit(
        GenerationCommitIntent(target_machine="n2-standard-8", generation=9),
        GenerationCommitRequest(elected_leader="operator-a", requester="operator-a", expected_generation=9),
    )
    obs5 = commit.action
    exp5 = CONTROL_2953_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CONTROL_2953_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6-8. R7 — admission produces the fixed two-replica, non-HPA policy.
    admitted = decide_control_plane_policy(ControlPlanePolicySpec(replica_count=2, horizontal_autoscaling=False))
    obs6 = admitted.outcome
    exp6 = CONTROL_2953_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CONTROL_2953_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = admitted.replica_count
    exp7 = CONTROL_2953_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CONTROL_2953_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = admitted.horizontal_autoscaling
    exp8 = CONTROL_2953_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CONTROL_2953_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    policy = _policy()
    # 9-10. R5/R6/AC2 — each pressure class chooses only its named direct edge.
    cpu = decide_target_transition(policy, TransitionSnapshot(cpu_pressure=True, backlog_pressure=True, memory_pressure=False, telemetry_complete=True, leader_headroom=50, follower_headroom=50, reserve_headroom=50), _history(), 1000)
    obs9 = cpu.target_machine
    exp9 = CONTROL_2953_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CONTROL_2953_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    memory = decide_target_transition(policy, TransitionSnapshot(cpu_pressure=False, backlog_pressure=False, memory_pressure=True, telemetry_complete=True, leader_headroom=50, follower_headroom=50, reserve_headroom=50), _history(), 1000)
    obs10 = memory.target_machine
    exp10 = CONTROL_2953_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CONTROL_2953_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11-12. R5/R6/AC4 — all downgrade predicates pass only after the full
    # post-convergence window and both replicas plus reserve fit the target.
    downgrade = decide_target_transition(policy, TransitionSnapshot(cpu_pressure=False, backlog_pressure=False, memory_pressure=False, telemetry_complete=True, leader_headroom=50, follower_headroom=50, reserve_headroom=50, downgrade_requested=True), _history(last_change_at=0, converged_at=0), 2000)
    obs11 = downgrade.action
    exp11 = CONTROL_2953_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": CONTROL_2953_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = downgrade.target_machine
    exp12 = CONTROL_2953_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": CONTROL_2953_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    return {"case_id": "control-2953-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
