"""EC behavior case for #2360 — retained capacity transition decisions.

Every expected value is an EC-owned literal transcribed from #2360: R2
(exact-resume and one active mutation), R3 (reapply is a no-op preserving the
automatic target), R4 (admitted downgrade after its gates), R5 (committed
member size and reconstructible read-replica reclamation), R6 (the existing
one- and three-voter availability vocabulary), R7 (machine-type shared pools
with distinct nodes), and R8 (CapacityBlocked retains member and generation
then resumes that generation after correction).
"""

from __future__ import annotations

from lumen.capacity.blocking import decide_capacity_block
from lumen.capacity.ownership import decide_reapply
from lumen.capacity.placement import decide_pool_assignments
from lumen.capacity.resume import decide_resume
from lumen.capacity.storage import decide_member_storage
from lumen.capacity.transition import decide_downgrade
from lumen.topology.availability import availability_promise

MINIMUM_CHECKS = 14

CAPACITY_2360_BEHAVIOR_MATRIX = (
    ("interrupted_action_is_the_sole_resumed_mutation", "resize-42"),
    ("reapply_preserves_the_automatic_target", "n2-standard-8"),
    ("reapply_preserves_initial_ownership", "user"),
    ("reapply_preserves_current_ownership", "automatic"),
    ("unchanged_reapply_is_a_no_op", "no_op"),
    ("eligible_downgrade_is_admitted", "admitted"),
    ("new_member_receives_the_committed_desired_size", "200Gi"),
    ("read_replica_drain_reclaims_its_reconstructible_pvc", "reclaim"),
    ("one_voter_has_no_unexpected_loss_promise", "no_promise_on_unexpected_node_loss"),
    ("three_voters_survive_one_unexpected_node_loss", "survives_one_unexpected_node_loss"),
    ("same_machine_type_uses_one_shared_data_pool", "data-n2-standard-8"),
    ("same_pool_members_occupy_distinct_nodes", 2),
    ("other_machine_type_uses_its_corresponding_shared_pool", "data-n2-standard-16"),
    ("corrected_capacity_resumes_the_blocked_generation", 17),
)


def verify_capacity_2360_behavior() -> dict:
    checks = []

    # 1. R2 — the persisted action, not a newly requested action, is the only
    #    mutation allowed to resume after an interruption.
    resume = decide_resume(
        interrupted_state={"active_mutation": None, "phase": "interrupted"},
        persisted_action={"identifier": "resize-42", "kind": "machine_upgrade"},
        requested_actions=({"identifier": "resize-43", "kind": "pvc_growth"},),
    )
    obs1 = resume.next_mutation.identifier
    exp1 = CAPACITY_2360_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CAPACITY_2360_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R3 — rendered input cannot overwrite the automatic target already in
    #    progress merely because the unchanged CR is applied again.
    reapply = decide_reapply(
        initial={"machine_type": "n2-standard-4", "owner": "user"},
        current={"machine_type": "n2-standard-4", "owner": "automatic"},
        target={"machine_type": "n2-standard-8", "owner": "automatic"},
        rendered_input={"machine_type": "n2-standard-4"},
    )
    obs2 = reapply.target.machine_type
    exp2 = CAPACITY_2360_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CAPACITY_2360_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R3 — reapplication keeps the initial user's ownership compartment.
    obs3 = reapply.initial.owner
    exp3 = CAPACITY_2360_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CAPACITY_2360_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R3 — current ownership remains distinct from initial and target.
    obs4 = reapply.current.owner
    exp4 = CAPACITY_2360_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CAPACITY_2360_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R3 — unchanged input is explicitly a no-op, never a competing action.
    obs5 = reapply.action.value
    exp5 = CAPACITY_2360_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CAPACITY_2360_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R4 — once stable, headroom-fit, bounded, and outside cooldown, the
    #    proposed downgrade is admitted by the pure policy model.
    downgrade = decide_downgrade(
        policy={"stable_window_seconds": 300, "cooldown_seconds": 600, "pool_maximum": 3},
        current={"machine_type": "n2-standard-8", "node_count": 3, "stable_since": 0, "last_transition_at": -1000},
        proposed={"machine_type": "n2-standard-4", "node_count": 2, "projected_allocatable_headroom": 1},
        observed_at=1000,
    )
    obs6 = downgrade.kind.value
    exp6 = CAPACITY_2360_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CAPACITY_2360_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R5 — creation uses the committed desired size, rather than a catalog
    #    default or the capacity of an old member.
    new_member = decide_member_storage(
        catalog={"committed_desired_size": "200Gi", "existing_member_size": "100Gi"},
        member_role="voter",
        lifecycle_event="created",
    )
    obs7 = new_member.desired_size
    exp7 = CAPACITY_2360_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CAPACITY_2360_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R5 — the drain decision identifies the reclaim operation for the
    #    reconstructible read-replica volume.
    replica_drain = decide_member_storage(
        catalog={"committed_desired_size": "200Gi", "storage_class": "standard"},
        member_role="read_replica",
        lifecycle_event="drained",
    )
    obs8 = replica_drain.reclaim.value
    exp8 = CAPACITY_2360_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CAPACITY_2360_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R6 — one voter is classified honestly, not as unexpected-loss HA.
    obs9 = availability_promise(1)
    exp9 = CAPACITY_2360_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CAPACITY_2360_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R6 — three voters retain the established unexpected-loss promise.
    obs10 = availability_promise(3)
    exp10 = CAPACITY_2360_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CAPACITY_2360_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R7 — two namespaces using the same machine type select one shared
    #    data pool; namespace is not part of the pool key.
    placement = decide_pool_assignments(
        instances=(
            {"namespace": "alpha", "instance": "a", "machine_type": "n2-standard-8"},
            {"namespace": "bravo", "instance": "b", "machine_type": "n2-standard-8"},
            {"namespace": "charlie", "instance": "c", "machine_type": "n2-standard-16"},
        ),
        placements=(
            {"instance": "a", "node": "node-a"},
            {"instance": "b", "node": "node-b"},
            {"instance": "c", "node": "node-c"},
        ),
    )
    obs11 = placement.assignments["a"].pool_key
    exp11 = CAPACITY_2360_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": CAPACITY_2360_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R7 — the two members in that shared pool remain on different nodes.
    obs12 = len({placement.assignments["a"].node, placement.assignments["b"].node})
    exp12 = CAPACITY_2360_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": CAPACITY_2360_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R7 — a different machine type maps to its matching shared pool.
    obs13 = placement.assignments["c"].pool_key
    exp13 = CAPACITY_2360_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": CAPACITY_2360_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R8 — correction resumes the retained generation, not a replacement
    #     generation that would abandon the interrupted capacity request.
    recovered = decide_capacity_block(
        condition={"kind": "corrected"}, old_member={"identifier": "member-0", "healthy": True}, generation=17
    )
    obs14 = recovered.resume_generation
    exp14 = CAPACITY_2360_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": CAPACITY_2360_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    return {
        "case_id": "capacity-2360-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
