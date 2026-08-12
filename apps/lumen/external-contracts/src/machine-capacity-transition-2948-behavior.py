"""EC behavior case for #2948 -- catalog-backed machine transitions.

Every expected value is an EC-owned literal transcribed from #2948: R1 selects
one direct transition and preserves initial/generation state; R2 uses only the
precreated catalog and its stable selector; R3 selects direct compute or
highmem pressure steps; R4/AC4 retain an in-progress handoff; R5 projects the
operator state without a price; R6 gates old-member retirement; R7/AC3 report
capacity blocks; and AC2 permits at most one active movement per shard.
"""

from __future__ import annotations

from lumen.topology.capacity_blocked_state import (
    TargetCapacityObservation,
    decide_capacity_hold,
)
from lumen.topology.machine_capacity_transition import (
    DirectMachineTransition,
    MachineCapacityCatalog,
    MachineCapacityEntry,
    MachineTransitionRequest,
    MachineTransitionState,
    RetirementEvidence,
    admit_shard_transition,
    choose_pressure_step,
    decide_old_member_retirement,
    decide_transition,
    project_status,
    resolve_catalog_target,
    resume_transition,
)

MINIMUM_CHECKS = 14

MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX = (
    ("direct_transition_keeps_initial_machine_and_advances_generations", ("n2-standard-8", "n2-standard-4", 8, 9)),
    ("catalog_resolution_returns_the_declared_machine", "n2-standard-8"),
    ("catalog_resolution_returns_the_stable_selector", ("cloud.google.com/gke-machine-type", "n2-standard-8")),
    ("cpu_pressure_selects_the_compute_richer_direct_step", "n2-standard-8"),
    ("write_pressure_selects_the_compute_richer_direct_step", "n2-standard-8"),
    ("memory_pressure_selects_the_corresponding_highmem_direct_step", "n2-highmem-8"),
    ("resume_retains_target_identity_for_each_handoff_phase", ("n2-standard-8", "n2-standard-8", "n2-standard-8", "n2-standard-8")),
    ("resume_retains_phase_for_reapplied_original_input", ("planned", "target_scheduling", "catching_up", "retiring_old")),
    ("status_projects_all_required_transition_fields", ("initial_machine", "current_machine", "target_machine", "selector_identity", "current_generation", "target_generation", "phase", "old_member_count", "new_member_count", "blocked_reason")),
    ("status_has_no_monetary_or_surge_price_field", ()),
    ("fully_proven_target_admits_old_member_retirement", "retire_old_member"),
    ("one_active_shard_movement_is_admitted", "admitted"),
    ("missing_target_capacity_is_classified_capacity_blocked", "CapacityBlocked"),
    ("capacity_hold_retains_the_healthy_old_member", "retain_healthy_old_member"),
)


def _catalog() -> MachineCapacityCatalog:
    """Spell out catalog ownership so defaults cannot hide a transition edge."""
    return MachineCapacityCatalog(
        entries=(
            MachineCapacityEntry("n2-standard-4", ("cloud.google.com/gke-machine-type", "n2-standard-4")),
            MachineCapacityEntry("n2-standard-8", ("cloud.google.com/gke-machine-type", "n2-standard-8")),
            MachineCapacityEntry("n2-highmem-8", ("cloud.google.com/gke-machine-type", "n2-highmem-8")),
        ),
        direct_transitions=(
            DirectMachineTransition("n2-standard-4", "n2-standard-8", "compute"),
            DirectMachineTransition("n2-standard-4", "n2-highmem-8", "highmem"),
        ),
    )


def _state(**overrides: object) -> MachineTransitionState:
    values: dict[str, object] = {
        "shard_id": "orders-0",
        "initial_machine": "n2-standard-4",
        "current_machine": "n2-standard-4",
        "target_machine": None,
        "selector_identity": ("cloud.google.com/gke-machine-type", "n2-standard-4"),
        "current_generation": 8,
        "target_generation": 8,
        "phase": "planned",
        "old_member_count": 1,
        "new_member_count": 0,
        "blocked_reason": None,
        "active_member_movement": False,
    }
    values.update(overrides)
    return MachineTransitionState(**values)


def verify_machine_capacity_transition_2948_behavior() -> dict:
    checks = []
    catalog = _catalog()

    # 1. R1 -- one explicit compute request selects the sole direct edge and
    # preserves the immutable initial machine while advancing target state.
    decision = decide_transition(
        MachineTransitionRequest("orders-0", "compute", "n2-standard-4"), catalog, _state()
    )
    obs1 = (decision.target_machine, decision.initial_machine, decision.current_generation, decision.target_generation)
    exp1 = MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2-3. R2 -- resolving a direct edge exposes both the catalog machine and
    # its stable label identity, rather than an operator-invented selector.
    resolved = resolve_catalog_target("n2-standard-4", "compute", catalog)
    obs2 = resolved.machine
    exp2 = MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = resolved.selector_identity
    exp3 = MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4-6. R3 -- CPU and write share the compute-richer direct step; memory
    # uses its explicitly catalogued highmem counterpart.
    cpu = choose_pressure_step("cpu", "n2-standard-4", catalog)
    obs4 = cpu.machine
    exp4 = MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    write = choose_pressure_step("write", "n2-standard-4", catalog)
    obs5 = write.machine
    exp5 = MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    memory = choose_pressure_step("memory", "n2-standard-4", catalog)
    obs6 = memory.machine
    exp6 = MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7-8. R4/AC4 -- every modelled handoff phase survives re-applied original
    # GitOps input with the old target and phase, never a competing transition.
    phases = ("planned", "target_scheduling", "catching_up", "retiring_old")
    resumed = tuple(
        resume_transition(_state(target_machine="n2-standard-8", target_generation=9, phase=phase), "n2-standard-4", 8)
        for phase in phases
    )
    obs7 = tuple(item.target_machine for item in resumed)
    exp7 = MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = tuple(item.phase for item in resumed)
    exp8 = MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9-10. R5 -- status makes each required operator-owned field observable
    # and deliberately contains no monetary/surge-price dimension.
    status = project_status(_state(target_machine="n2-standard-8", target_generation=9))
    obs9 = tuple(status.__dataclass_fields__)
    exp9 = MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    price_free_status = project_status(_state(target_machine="n2-standard-8", target_generation=9))
    obs10 = tuple(field for field in price_free_status.__dataclass_fields__ if "price" in field or "cost" in field)
    exp10 = MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R6 -- retirement is admitted only after all named pure evidence is
    # supplied; live scheduling and catch-up proof remain runtime-owned.
    retirement = decide_old_member_retirement(
        _state(target_machine="n2-standard-8", phase="retiring_old"),
        RetirementEvidence(True, True, True, True, True),
    )
    obs11 = retirement.action
    exp11 = MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. AC2 -- a shard with no movement explicitly admits exactly one start.
    admitted = admit_shard_transition(_state(active_member_movement=False))
    obs12 = admitted.kind
    exp12 = MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13-14. R7/AC3 -- missing catalog capacity is a typed hold and a healthy
    # old member stays in place while a bounded retry is handled by the design.
    hold = decide_capacity_hold(
        TargetCapacityObservation("n2-standard-8", exists=False, draining=False, available_slots=0, at_maximum=False, quota_available=True, schedulable=False),
        old_member_health="healthy",
    )
    obs13 = hold.classification
    exp13 = MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = hold.old_member_action
    exp14 = MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    return {"case_id": "machine-capacity-transition-2948-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
