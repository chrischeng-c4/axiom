"""EC security case for #2953 — R3, R4, R5, R6, R7, AC2, AC3, AC4, and AC5.

The expected refusal and hold vocabulary is owned by this EC.  It drives only
the pure model: live leader contention, restart durability, telemetry I/O, and
Kubernetes HPA rendering are intentionally runtime-stage concerns.
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

MINIMUM_CHECKS = 17

CONTROL_2953_SECURITY_MATRIX = (
    ("incomplete_calibration_is_refused", "INCOMPLETE_CALIBRATION"),
    ("incomplete_calibration_refusal_names_the_evidence", "calibration.complete"),
    ("complete_calibration_neighbour_is_admitted", "ADMITTED"),
    ("non_leader_cannot_commit_a_target_generation", "NOT_ELECTED"),
    ("generation_conflict_cannot_commit_a_target_generation", "GENERATION_CONFLICT"),
    ("non_two_replica_policy_is_refused", "TWO_REPLICAS_REQUIRED"),
    ("replica_refusal_names_the_replica_count", "replica_count"),
    ("horizontal_autoscaling_policy_is_refused", "HORIZONTAL_AUTOSCALING_FORBIDDEN"),
    ("hpa_refusal_names_the_hpa_field", "horizontal_autoscaling"),
    ("incomplete_telemetry_holds_transition", "INCOMPLETE_TELEMETRY"),
    ("insufficient_downgrade_headroom_holds_transition", "INSUFFICIENT_HEADROOM"),
    ("cooldown_holds_an_otherwise_eligible_transition", "COOLDOWN"),
    ("daily_change_budget_holds_an_otherwise_eligible_transition", "DAILY_CHANGE_BUDGET"),
    ("spend_ceiling_holds_an_otherwise_eligible_transition", "SPEND_CEILING"),
    ("sustained_upgrade_boundary_holds_transient_pressure", "SUSTAINED_UPGRADE"),
    ("slow_downgrade_boundary_holds_an_early_downgrade", "SLOW_DOWNGRADE"),
    ("post_convergence_boundary_holds_a_pre_window_downgrade", "POST_CONVERGENCE_WINDOW"),
)


def _complete_calibration(*, complete: bool) -> CalibrationEvidence:
    return CalibrationEvidence(
        benchmarks=(
            MachineBenchmark("e2-standard-2", throughput=100, recovery_seconds=30, slo_headroom=20, hourly_cost=1),
            MachineBenchmark("n2-standard-4", throughput=160, recovery_seconds=25, slo_headroom=35, hourly_cost=2),
            MachineBenchmark("n2-standard-8", throughput=220, recovery_seconds=25, slo_headroom=45, hourly_cost=3),
            MachineBenchmark("n2-highmem-4", throughput=130, recovery_seconds=25, slo_headroom=40, hourly_cost=2),
        ),
        complete=complete,
    )


def _policy() -> TransitionPolicy:
    return TransitionPolicy(initial_machine="n2-standard-4", allowed_edges=(("n2-standard-4", "n2-standard-8", "CPU_BACKLOG"), ("n2-standard-4", "n2-highmem-4", "MEMORY_HIGHMEM")), sustained_seconds=300, cooldown_seconds=600, daily_change_budget=2, spend_ceiling=10, slow_downgrade_seconds=1800, post_convergence_seconds=900)


def _snapshot(**overrides: object) -> TransitionSnapshot:
    values: dict[str, object] = {"cpu_pressure": True, "backlog_pressure": True, "memory_pressure": False, "telemetry_complete": True, "leader_headroom": 50, "follower_headroom": 50, "reserve_headroom": 50, "downgrade_requested": False}
    values.update(overrides)
    return TransitionSnapshot(**values)


def _history(**overrides: object) -> TransitionHistory:
    values: dict[str, object] = {"last_change_at": 0, "changes_today": 0, "spend_today": 0, "converged_at": 0}
    values.update(overrides)
    return TransitionHistory(**values)


def verify_control_2953_security() -> dict:
    checks = []

    # 1-3. R3 — calibration admission is fail-closed, names its missing
    # evidence, and still admits the complete neighbouring input.
    incomplete = decide_initial_machine_and_transitions(_complete_calibration(complete=False))
    obs1 = incomplete.reason
    exp1 = CONTROL_2953_SECURITY_MATRIX[0][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = incomplete.field_path
    exp2 = CONTROL_2953_SECURITY_MATRIX[1][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    complete = decide_initial_machine_and_transitions(_complete_calibration(complete=True))
    obs3 = complete.outcome
    exp3 = CONTROL_2953_SECURITY_MATRIX[2][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4-5. R4/AC3 — both leader identity and expected generation gate commits.
    not_elected = decide_generation_commit(GenerationCommitIntent(target_machine="n2-standard-8", generation=9), GenerationCommitRequest(elected_leader="operator-a", requester="operator-b", expected_generation=9))
    obs4 = not_elected.reason
    exp4 = CONTROL_2953_SECURITY_MATRIX[3][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    conflict = decide_generation_commit(GenerationCommitIntent(target_machine="n2-standard-8", generation=9), GenerationCommitRequest(elected_leader="operator-a", requester="operator-a", expected_generation=8))
    obs5 = conflict.reason
    exp5 = CONTROL_2953_SECURITY_MATRIX[4][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6-9. R7/AC5 — both forbidden policy inputs name their offending field.
    wrong_replicas = decide_control_plane_policy(ControlPlanePolicySpec(replica_count=3, horizontal_autoscaling=False))
    obs6 = wrong_replicas.reason
    exp6 = CONTROL_2953_SECURITY_MATRIX[5][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = wrong_replicas.field_path
    exp7 = CONTROL_2953_SECURITY_MATRIX[6][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    hpa = decide_control_plane_policy(ControlPlanePolicySpec(replica_count=2, horizontal_autoscaling=True))
    obs8 = hpa.reason
    exp8 = CONTROL_2953_SECURITY_MATRIX[7][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = hpa.field_path
    exp9 = CONTROL_2953_SECURITY_MATRIX[8][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    policy = _policy()
    # 10-17. R5/R6/AC2/AC4 — every independent unsafe input holds rather than
    # producing a target decision; timestamps are literal supplied boundaries.
    telemetry = decide_target_transition(policy, _snapshot(telemetry_complete=False), _history(), 1000)
    obs10 = telemetry.reason
    exp10 = CONTROL_2953_SECURITY_MATRIX[9][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    headroom = decide_target_transition(policy, _snapshot(cpu_pressure=False, backlog_pressure=False, downgrade_requested=True, leader_headroom=0), _history(), 2000)
    obs11 = headroom.reason
    exp11 = CONTROL_2953_SECURITY_MATRIX[10][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    cooldown = decide_target_transition(policy, _snapshot(), _history(last_change_at=900), 1000)
    obs12 = cooldown.reason
    exp12 = CONTROL_2953_SECURITY_MATRIX[11][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    daily = decide_target_transition(policy, _snapshot(), _history(changes_today=2), 1000)
    obs13 = daily.reason
    exp13 = CONTROL_2953_SECURITY_MATRIX[12][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    spend = decide_target_transition(policy, _snapshot(), _history(spend_today=10), 1000)
    obs14 = spend.reason
    exp14 = CONTROL_2953_SECURITY_MATRIX[13][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    sustained = decide_target_transition(policy, _snapshot(), _history(), 299)
    obs15 = sustained.reason
    exp15 = CONTROL_2953_SECURITY_MATRIX[14][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    slow = decide_target_transition(policy, _snapshot(cpu_pressure=False, backlog_pressure=False, downgrade_requested=True), _history(), 1799)
    obs16 = slow.reason
    exp16 = CONTROL_2953_SECURITY_MATRIX[15][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    pre_window = decide_target_transition(policy, _snapshot(cpu_pressure=False, backlog_pressure=False, downgrade_requested=True), _history(converged_at=1000), 1800)
    obs17 = pre_window.reason
    exp17 = CONTROL_2953_SECURITY_MATRIX[16][1]
    checks.append({"name": CONTROL_2953_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    return {"case_id": "control-2953-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
