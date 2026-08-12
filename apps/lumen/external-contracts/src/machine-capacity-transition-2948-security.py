"""EC security case for #2948 -- fail-closed machine-capacity transitions.

Expected values are EC-owned literals from #2948: R1 refuses a request without
one configured direct step; R2 refuses undeclared catalog targets; R4/AC4 do
not reset an in-progress handoff; R6 holds retirement for every missing proof;
R7/AC3 use CapacityBlocked, retain a healthy old member, and never claim a
pool mutation; and AC2 refuses a second member movement for the same shard.
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
    resolve_catalog_target,
    resume_transition,
)

MINIMUM_CHECKS = 13

MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX = (
    ("transition_without_a_direct_edge_is_refused", "no_direct_transition"),
    ("transition_refusal_names_the_requested_direction", "requested_direction"),
    ("undeclared_catalog_target_is_refused", "catalog_target_undeclared"),
    ("catalog_refusal_names_the_current_machine", "current_machine"),
    ("pressure_without_a_matching_direct_step_is_refused", "no_direct_pressure_step"),
    ("reapplied_original_input_does_not_reset_the_existing_target", "n2-standard-8"),
    ("each_missing_retirement_proof_has_a_specific_hold_reason", ("target_not_scheduled", "target_machine_unverified", "target_not_caught_up", "voter_pdb_unprotected", "target_headroom_unproven")),
    ("active_shard_movement_is_refused", "shard_movement_active"),
    ("active_movement_refusal_names_the_shard", "shard_id"),
    ("all_unavailable_target_forms_are_capacity_blocked", ("CapacityBlocked", "CapacityBlocked", "CapacityBlocked", "CapacityBlocked", "CapacityBlocked")),
    ("capacity_block_never_claims_pool_create_resize_or_delete", "none"),
    ("capacity_block_uses_a_bounded_retry_hold", "bounded_retry_hold"),
    ("unavailable_target_keeps_a_healthy_old_member", "retain_healthy_old_member"),
)


def _catalog() -> MachineCapacityCatalog:
    return MachineCapacityCatalog(
        entries=(
            MachineCapacityEntry("n2-standard-4", ("cloud.google.com/gke-machine-type", "n2-standard-4")),
            MachineCapacityEntry("n2-standard-8", ("cloud.google.com/gke-machine-type", "n2-standard-8")),
        ),
        direct_transitions=(DirectMachineTransition("n2-standard-4", "n2-standard-8", "compute"),),
    )


def _state(**overrides: object) -> MachineTransitionState:
    values: dict[str, object] = {
        "shard_id": "orders-0", "initial_machine": "n2-standard-4", "current_machine": "n2-standard-4",
        "target_machine": None, "selector_identity": ("cloud.google.com/gke-machine-type", "n2-standard-4"),
        "current_generation": 8, "target_generation": 8, "phase": "planned", "old_member_count": 1,
        "new_member_count": 0, "blocked_reason": None, "active_member_movement": False,
    }
    values.update(overrides)
    return MachineTransitionState(**values)


def _reason(value) -> str:
    reason = getattr(value, "reason", None)
    return reason.value if hasattr(reason, "value") else reason if reason is not None else "admitted"


def verify_machine_capacity_transition_2948_security() -> dict:
    checks = []
    catalog = _catalog()

    # 1-2. R1 -- a request spells out an invalid direction rather than relying
    # on a default, and is refused with an actionable field name.
    no_edge = decide_transition(MachineTransitionRequest("orders-0", "highmem", "n2-standard-4"), catalog, _state())
    obs1 = _reason(no_edge)
    exp1 = MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[0][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    no_edge_field = decide_transition(MachineTransitionRequest("orders-0", "highmem", "n2-standard-4"), catalog, _state())
    obs2 = no_edge_field.field_path
    exp2 = MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[1][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3-4. R2 -- an explicit machine absent from the supplied catalog cannot
    # become a target, and the refusal identifies the source input.
    undeclared = resolve_catalog_target("n2-standard-16", "compute", catalog)
    obs3 = _reason(undeclared)
    exp3 = MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[2][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    undeclared_field = resolve_catalog_target("n2-standard-16", "compute", catalog)
    obs4 = undeclared_field.field_path
    exp4 = MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[3][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R3/R2 -- pressure selection also fails closed when its named direct
    # highmem edge is missing, rather than inventing a machine family.
    no_pressure_step = choose_pressure_step("memory", "n2-standard-4", catalog)
    obs5 = _reason(no_pressure_step)
    exp5 = MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[4][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R4/AC4 -- reapplying the original seed during an actual handoff keeps
    # the recorded target rather than replacing it from fresh input.
    resumed = resume_transition(_state(target_machine="n2-standard-8", target_generation=9, phase="catching_up"), "n2-standard-4", 8)
    obs6 = resumed.target_machine
    exp6 = MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[5][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R6 -- each required proof independently holds retirement; a generic
    # boolean cannot conceal which prerequisite a controller skipped.
    evidence_rows = (
        RetirementEvidence(False, True, True, True, True), RetirementEvidence(True, False, True, True, True),
        RetirementEvidence(True, True, False, True, True), RetirementEvidence(True, True, True, False, True),
        RetirementEvidence(True, True, True, True, False),
    )
    obs7 = tuple(_reason(decide_old_member_retirement(_state(target_machine="n2-standard-8", phase="retiring_old"), item)) for item in evidence_rows)
    exp7 = MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[6][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8-9. AC2 -- the second movement is refused and points at the affected
    # shard, while the behavior case exercises the neighbouring admissible one.
    occupied = admit_shard_transition(_state(active_member_movement=True))
    obs8 = _reason(occupied)
    exp8 = MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[7][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    occupied_field = admit_shard_transition(_state(active_member_movement=True))
    obs9 = occupied_field.field_path
    exp9 = MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[8][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10-13. R7/AC3 -- all catalog-capacity failures become a typed bounded
    # hold: no pool operation is claimed and a healthy old member is retained.
    unavailable = (
        TargetCapacityObservation("n2-standard-8", False, False, 0, False, True, False),
        TargetCapacityObservation("n2-standard-8", True, True, 1, False, True, False),
        TargetCapacityObservation("n2-standard-8", True, False, 0, True, True, False),
        TargetCapacityObservation("n2-standard-8", True, False, 1, False, False, False),
        TargetCapacityObservation("n2-standard-8", True, False, 1, False, True, False),
    )
    holds = tuple(decide_capacity_hold(item, "healthy") for item in unavailable)
    obs10 = tuple(item.classification for item in holds)
    exp10 = MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[9][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    obs11 = holds[0].pool_action
    exp11 = MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[10][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = holds[1].retry_disposition
    exp12 = MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[11][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = holds[2].old_member_action
    exp13 = MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[12][1]
    checks.append({"name": MACHINE_CAPACITY_TRANSITION_2948_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    return {"case_id": "machine-capacity-transition-2948-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
