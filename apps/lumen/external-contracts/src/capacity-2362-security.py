"""EC security case for #2362 -- capacity arbitration fails closed.

EC-owned literals below pin R1/R6/R7/R8/R9 and AC3/AC4/AC5/AC6.  Refusals
name both the contract vocabulary and the offending field, while neighbouring
safe inputs remain admitted.  The absent pure design imports are deliberate:
the case must remain red until a separate implementation worker lands it.
"""

from __future__ import annotations

from lumen.capacity.arbitration import decide_capacity
from lumen.capacity.catalog import select_profile
from lumen.capacity.projection import evaluate_downgrade
from lumen.capacity.spec import CapacityInput, CapacityPolicy, CapacitySignals, CapacityState, ProfileAvailability, ProfileCatalog, SyntheticClock, TransitionGraph
from lumen.capacity.status import CapacityStatus
from lumen.capacity.verdict import ActionKind

MINIMUM_CHECKS = 36

CAPACITY_2362_SECURITY_MATRIX = (
    ("incomplete_telemetry_holds", "HOLD"),
    ("incomplete_telemetry_hold_names_completeness", "telemetry_complete"),
    ("fresh_complete_telemetry_neighbour_is_admitted", "READ_REPLICA"),
    ("stale_telemetry_holds", "HOLD"),
    ("stale_telemetry_hold_names_freshness", "telemetry_fresh"),
    ("generation_mismatch_holds", "HOLD"),
    ("generation_mismatch_hold_names_generation", "signal_generation"),
    ("active_mutation_holds", "HOLD"),
    ("active_mutation_hold_names_mutation_fence", "mutation_active"),
    ("cooldown_holds", "cooldown"),
    ("deadband_holds", "deadband"),
    ("pre_convergence_window_holds", "post_convergence_window"),
    ("automatic_change_limit_holds", "automatic_change_limit"),
    ("cpu_unsafe_downgrade_names_cpu", "cpu"),
    ("memory_unsafe_downgrade_names_memory_or_working_set", "memory_or_working_set"),
    ("recovery_unsafe_downgrade_names_recovery", "recovery"),
    ("reserve_unsafe_downgrade_names_system_reserve", "system_reserve"),
    ("compaction_unsafe_downgrade_names_compaction", "compaction"),
    ("absent_installed_target_is_capacity_blocked", "CapacityBlocked"),
    ("absent_target_returns_no_undeclared_fallback", None),
    ("draining_installed_target_is_capacity_blocked", "CapacityBlocked"),
    ("full_installed_target_is_capacity_blocked", "CapacityBlocked"),
    ("quota_blocked_installed_target_is_capacity_blocked", "CapacityBlocked"),
    ("unschedulable_installed_target_is_capacity_blocked", "CapacityBlocked"),
    ("unavailable_target_refusal_names_availability", "availability"),
    ("policy_has_no_monetary_or_currency_input", ()),
    ("action_vocabulary_excludes_voters_merges_shrinks_hpa_and_vpa", ("HOLD", "PVC_GROW", "SPLIT", "READ_REPLICA", "MACHINE_UPGRADE", "HIGHMEM_UPGRADE", "READ_REPLICA_REMOVE", "MACHINE_DOWNGRADE")),
    ("mismatched_capacity_generations_are_not_accepted_as_bound", (41, 42, 41)),
    ("unsafe_cpu_projection_holds_downgrade", "HOLD"),
    ("declared_but_unreachable_target_is_capacity_blocked", "CapacityBlocked"),
    ("unreachable_target_returns_no_profile", None),
    ("scale_in_before_slower_post_convergence_window_holds", "HOLD"),
    ("scale_in_at_slower_post_convergence_window_downgrades", "MACHINE_DOWNGRADE"),
    ("scale_in_sustained_window_exceeds_scale_out_window", True),
    ("equal_capacity_generations_are_generation_bound", True),
    ("mismatched_capacity_generations_are_not_generation_bound", False),
)


def verify_capacity_2362_security() -> dict:
    checks = []
    policy = CapacityPolicy.default()
    clock = SyntheticClock(now=1_000)

    # 1. R1 -- missing telemetry never produces a capacity action.
    incomplete = decide_capacity(CapacityInput(signals=CapacitySignals(read_dominated=True, telemetry_complete=False), state=CapacityState(), policy=policy), clock)
    obs1 = incomplete.action.kind
    exp1 = CAPACITY_2362_SECURITY_MATRIX[0][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- the hold tells an operator exactly what must be repaired.
    obs2 = incomplete.field_path
    exp2 = CAPACITY_2362_SECURITY_MATRIX[1][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1 -- the nearest complete/fresh/generation-bound input remains selectable.
    fresh = decide_capacity(CapacityInput(signals=CapacitySignals(read_dominated=True, telemetry_complete=True, telemetry_fresh=True, signal_generation=9), state=CapacityState(current_generation=9), policy=policy), clock)
    obs3 = fresh.action.kind
    exp3 = CAPACITY_2362_SECURITY_MATRIX[2][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R1 -- stale evidence is held even when its pressure is otherwise actionable.
    stale = decide_capacity(CapacityInput(signals=CapacitySignals(read_dominated=True, telemetry_fresh=False), state=CapacityState(), policy=policy), clock)
    obs4 = stale.action.kind
    exp4 = CAPACITY_2362_SECURITY_MATRIX[3][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R1 -- stale evidence names freshness rather than hiding it behind a generic hold.
    obs5 = stale.field_path
    exp5 = CAPACITY_2362_SECURITY_MATRIX[4][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R1 -- a signal from another CR generation is never acted on.
    mismatched = decide_capacity(CapacityInput(signals=CapacitySignals(read_dominated=True, signal_generation=8), state=CapacityState(current_generation=9), policy=policy), clock)
    obs6 = mismatched.action.kind
    exp6 = CAPACITY_2362_SECURITY_MATRIX[5][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R1 -- the stale-generation hold identifies the signal generation field.
    obs7 = mismatched.field_path
    exp7 = CAPACITY_2362_SECURITY_MATRIX[6][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R1 -- no second capacity/topology mutation begins while one is active.
    mutating = decide_capacity(CapacityInput(signals=CapacitySignals(read_dominated=True), state=CapacityState(mutation_active=True), policy=policy), clock)
    obs8 = mutating.action.kind
    exp8 = CAPACITY_2362_SECURITY_MATRIX[7][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R1 -- the fence is an observable reason, not a silent no-op.
    obs9 = mutating.field_path
    exp9 = CAPACITY_2362_SECURITY_MATRIX[8][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R7 -- scale-out inside its explicitly supplied cooldown is held.
    cooldown = decide_capacity(CapacityInput(signals=CapacitySignals(read_dominated=True), state=CapacityState(last_change_at=950), policy=policy), clock)
    obs10 = cooldown.reason
    exp10 = CAPACITY_2362_SECURITY_MATRIX[9][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R7 -- pressure inside the deadband does not flap the topology.
    deadband = decide_capacity(CapacityInput(signals=CapacitySignals(read_dominated=True, within_deadband=True), state=CapacityState(), policy=policy), clock)
    obs11 = deadband.reason
    exp11 = CAPACITY_2362_SECURITY_MATRIX[10][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R6/R7/AC2 -- a low window that overlaps convergence cannot downgrade.
    pre_convergence = decide_capacity(CapacityInput(signals=CapacitySignals(low_utilization=True, window_started_at=800), state=CapacityState(converged_at=900), policy=policy), clock)
    obs12 = pre_convergence.reason
    exp12 = CAPACITY_2362_SECURITY_MATRIX[11][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R7 -- an exhausted automatic-change budget is a named hold.
    change_limit = decide_capacity(CapacityInput(signals=CapacitySignals(read_dominated=True), state=CapacityState(automatic_change_limit_reached=True), policy=policy), clock)
    obs13 = change_limit.reason
    exp13 = CAPACITY_2362_SECURITY_MATRIX[12][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R6/AC3 -- projected CPU over the target is the exact downgrade constraint.
    cpu = evaluate_downgrade(CapacitySignals(cpu_p95=99, memory_p95=20, compaction_p95=20, recovery_p95=20, system_reserve_p95=20), "standard-4", headroom=20)
    obs14 = cpu.failing_constraint
    exp14 = CAPACITY_2362_SECURITY_MATRIX[13][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R6/AC3 -- working-set or memory headroom is independently mandatory.
    memory = evaluate_downgrade(CapacitySignals(cpu_p95=20, memory_p95=99, compaction_p95=20, recovery_p95=20, system_reserve_p95=20), "standard-4", headroom=20)
    obs15 = memory.failing_constraint
    exp15 = CAPACITY_2362_SECURITY_MATRIX[14][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16. R6/AC3 -- recovery headroom cannot be erased from the downgrade proof.
    recovery = evaluate_downgrade(CapacitySignals(cpu_p95=20, memory_p95=20, compaction_p95=20, recovery_p95=99, system_reserve_p95=20), "standard-4", headroom=20)
    obs16 = recovery.failing_constraint
    exp16 = CAPACITY_2362_SECURITY_MATRIX[15][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    # 17. R6/AC3 -- reserve headroom is a capacity constraint of its own.
    reserve = evaluate_downgrade(CapacitySignals(cpu_p95=20, memory_p95=20, compaction_p95=20, recovery_p95=20, system_reserve_p95=99), "standard-4", headroom=20)
    obs17 = reserve.failing_constraint
    exp17 = CAPACITY_2362_SECURITY_MATRIX[16][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    catalog = ProfileCatalog(installed=("standard-4",), availability={"standard-4": ProfileAvailability.DRAINING})
    blocked = select_profile(catalog, TransitionGraph({"standard-2": ("standard-4",)}), "standard-2", "standard-4")

    # 18. R6/AC3 -- compaction headroom is independently mandatory too.
    compaction = evaluate_downgrade(CapacitySignals(cpu_p95=20, memory_p95=20, compaction_p95=99, recovery_p95=20, system_reserve_p95=20), "standard-4", headroom=20)
    obs18 = compaction.failing_constraint
    exp18 = CAPACITY_2362_SECURITY_MATRIX[17][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    # 19. R8/R9/AC5 -- an absent requested target never becomes an undeclared fallback.
    absent = select_profile(ProfileCatalog(installed=("standard-4",), availability={"standard-4": ProfileAvailability.AVAILABLE}), TransitionGraph({"standard-2": ("standard-4",)}), "standard-2", "missing-8")
    obs19 = absent.reason
    exp19 = CAPACITY_2362_SECURITY_MATRIX[18][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    # 20. R9/AC5 -- an absent request produces no invented target profile.
    obs20 = absent.profile
    exp20 = CAPACITY_2362_SECURITY_MATRIX[19][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    # 21. R8/R9/AC5 -- an installed but draining target is CapacityBlocked, never fallback.
    obs21 = blocked.reason
    exp21 = CAPACITY_2362_SECURITY_MATRIX[20][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})

    # 22. R9/AC5 -- full capacity has the same closed CapacityBlocked outcome.
    full = select_profile(ProfileCatalog(installed=("standard-4",), availability={"standard-4": ProfileAvailability.FULL}), TransitionGraph({"standard-2": ("standard-4",)}), "standard-2", "standard-4")
    obs22 = full.reason
    exp22 = CAPACITY_2362_SECURITY_MATRIX[21][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})

    # 23. R9/AC5 -- quota blocks hold rather than silently choosing a different profile.
    quota = select_profile(ProfileCatalog(installed=("standard-4",), availability={"standard-4": ProfileAvailability.QUOTA_BLOCKED}), TransitionGraph({"standard-2": ("standard-4",)}), "standard-2", "standard-4")
    obs23 = quota.reason
    exp23 = CAPACITY_2362_SECURITY_MATRIX[22][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})

    # 24. R9/AC5 -- unschedulable capacity is also a closed CapacityBlocked result.
    unschedulable = select_profile(ProfileCatalog(installed=("standard-4",), availability={"standard-4": ProfileAvailability.UNSCHEDULABLE}), TransitionGraph({"standard-2": ("standard-4",)}), "standard-2", "standard-4")
    obs24 = unschedulable.reason
    exp24 = CAPACITY_2362_SECURITY_MATRIX[23][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[23][0], "expected": exp24, "observed": obs24, "passed": obs24 == exp24})

    # 25. R9 -- every unavailable state names availability as the responsible field.
    obs25 = blocked.field_path
    exp25 = CAPACITY_2362_SECURITY_MATRIX[24][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[24][0], "expected": exp25, "observed": obs25, "passed": obs25 == exp25})

    # 26. R7 -- policy exposes technical controls only, never monetary, price, cost, or budget input.
    obs26 = tuple(sorted(name for name in CapacityPolicy.__dataclass_fields__ if any(term in name.lower() for term in ("currency", "price", "monetary", "cost", "budget"))))
    exp26 = CAPACITY_2362_SECURITY_MATRIX[25][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[25][0], "expected": exp26, "observed": obs26, "passed": obs26 == exp26})

    # 27. R4/R5/AC4 -- the complete action vocabulary contains no voter, merge, shrink, HPA, or VPA action.
    obs27 = tuple(member.value for member in ActionKind)
    exp27 = CAPACITY_2362_SECURITY_MATRIX[26][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[26][0], "expected": exp27, "observed": obs27, "passed": obs27 == exp27})

    # 28. AC6 -- expose the unequal record values; do not accept an is-bound boolean.
    unbound = CapacityStatus(recommendation_generation=41, action_generation=42, status_generation=41)
    obs28 = (unbound.recommendation_generation, unbound.action_generation, unbound.status_generation)
    exp28 = CAPACITY_2362_SECURITY_MATRIX[27][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[27][0], "expected": exp28, "observed": obs28, "passed": obs28 == exp28})

    # 29. R6/AC3 -- an unsafe projected constraint must produce a hold, never a downgrade action.
    obs29 = cpu.action.kind
    exp29 = CAPACITY_2362_SECURITY_MATRIX[28][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[28][0], "expected": exp29, "observed": obs29, "passed": obs29 == exp29})

    unreachable = select_profile(ProfileCatalog(installed=("standard-4", "highmem-4"), availability={"standard-4": ProfileAvailability.AVAILABLE, "highmem-4": ProfileAvailability.AVAILABLE}), TransitionGraph({"standard-2": ("standard-4",)}), "standard-2", "highmem-4")

    # 30. R8/R9 -- an installed and available target without a declared edge is CapacityBlocked.
    obs30 = unreachable.reason
    exp30 = CAPACITY_2362_SECURITY_MATRIX[29][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[29][0], "expected": exp30, "observed": obs30, "passed": obs30 == exp30})

    # 31. R8 -- an unreachable request selects no profile, even when the requested profile is installed.
    obs31 = unreachable.profile
    exp31 = CAPACITY_2362_SECURITY_MATRIX[30][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[30][0], "expected": exp31, "observed": obs31, "passed": obs31 == exp31})

    # 32. R7/AC2 -- 1,799 seconds of a wholly post-convergence low window remains too early.
    slow_scale_in_early = decide_capacity(CapacityInput(signals=CapacitySignals(low_utilization=True, window_started_at=901), state=CapacityState(converged_at=900), policy=policy), SyntheticClock(now=2_700))
    obs32 = slow_scale_in_early.action.kind
    exp32 = CAPACITY_2362_SECURITY_MATRIX[31][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[31][0], "expected": exp32, "observed": obs32, "passed": obs32 == exp32})

    # 33. R7/AC2 -- the same wholly post-convergence window admits exactly at the 1,800-second boundary.
    slow_scale_in_ready = decide_capacity(CapacityInput(signals=CapacitySignals(low_utilization=True, window_started_at=901), state=CapacityState(converged_at=900), policy=policy), SyntheticClock(now=2_701))
    obs33 = slow_scale_in_ready.action.kind
    exp33 = CAPACITY_2362_SECURITY_MATRIX[32][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[32][0], "expected": exp33, "observed": obs33, "passed": obs33 == exp33})

    # 34. R7 -- the configured slow scale-in window is materially longer than scale-out's 300 seconds.
    obs34 = policy.scale_in_sustained_seconds > policy.scale_out_sustained_seconds
    exp34 = CAPACITY_2362_SECURITY_MATRIX[33][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[33][0], "expected": exp34, "observed": obs34, "passed": obs34 == exp34})

    # 35. AC6 -- the status predicate accepts exactly matching recommendation, action, and status generations.
    obs35 = CapacityStatus(recommendation_generation=41, action_generation=41, status_generation=41).is_generation_bound()
    exp35 = CAPACITY_2362_SECURITY_MATRIX[34][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[34][0], "expected": exp35, "observed": obs35, "passed": obs35 == exp35})

    # 36. AC6 -- the same predicate rejects a mismatched action generation.
    obs36 = unbound.is_generation_bound()
    exp36 = CAPACITY_2362_SECURITY_MATRIX[35][1]
    checks.append({"name": CAPACITY_2362_SECURITY_MATRIX[35][0], "expected": exp36, "observed": obs36, "passed": obs36 == exp36})

    return {"case_id": "capacity-2362-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
