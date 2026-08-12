"""EC security case for #2360 — capacity decisions fail closed.

Expected values are EC-owned literals from #2360: R2 refuses a second active
mutation, R4 names each downgrade gate, R5 permits reclamation only for
reconstructible read-replica PVCs, R7 never derives a pool from namespace and
rejects same-pool node sharing, and R8 reports CapacityBlocked while retaining
the healthy old member and exact generation for absent, maximum, quota, and
unschedulable capacity.
"""

from __future__ import annotations

from lumen.capacity.blocking import decide_capacity_block
from lumen.capacity.placement import decide_pool_assignments
from lumen.capacity.resume import decide_resume
from lumen.capacity.storage import decide_member_storage
from lumen.capacity.transition import decide_downgrade

MINIMUM_CHECKS = 15

CAPACITY_2360_SECURITY_MATRIX = (
    ("second_mutation_is_refused_while_one_is_active", "another_mutation_active"),
    ("slow_downgrade_names_the_stable_window_refusal", "stable-window-not-elapsed"),
    ("downgrade_names_the_headroom_refusal", "insufficient-headroom"),
    ("downgrade_names_the_pool_maximum_refusal", "pool-maximum-exceeded"),
    ("downgrade_names_the_cooldown_refusal", "cooldown-active"),
    ("voter_pvc_is_not_reclaimable_on_drain", "retain"),
    ("namespace_does_not_change_the_shared_pool_key", "data-n2-standard-8"),
    ("same_pool_members_sharing_a_node_are_refused", "data_member_node_conflict"),
    ("absent_capacity_reports_capacity_blocked", "CapacityBlocked"),
    ("at_maximum_capacity_reports_capacity_blocked", "CapacityBlocked"),
    ("quota_blocked_capacity_reports_capacity_blocked", "CapacityBlocked"),
    ("unschedulable_capacity_reports_capacity_blocked", "CapacityBlocked"),
    ("blocked_capacity_retains_a_healthy_old_member", True),
    ("blocked_capacity_retains_the_old_member_identity", "member-0"),
    ("blocked_capacity_retains_the_requested_generation", 17),
)


def _reason(verdict) -> str:
    return verdict.reason.value if hasattr(verdict, "reason") else verdict.kind.value


def verify_capacity_2360_security() -> dict:
    checks = []

    # 1. R2 — an explicit active mutation exercises the concurrency guard; a
    #    default empty state would not prove that the guard exists.
    active = decide_resume(
        interrupted_state={"active_mutation": "resize-41", "phase": "running"},
        persisted_action={"identifier": "resize-42", "kind": "machine_upgrade"},
        requested_actions=({"identifier": "resize-43", "kind": "pvc_growth"},),
    )
    obs1 = _reason(active)
    exp1 = CAPACITY_2360_SECURITY_MATRIX[0][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    policy = {"stable_window_seconds": 300, "cooldown_seconds": 600, "pool_maximum": 3}

    # 2. R4 — insufficient stable time blocks a slow downgrade by name.
    stable = decide_downgrade(
        policy=policy,
        current={"machine_type": "n2-standard-8", "node_count": 3, "stable_since": 900, "last_transition_at": 0},
        proposed={"machine_type": "n2-standard-4", "node_count": 2, "projected_allocatable_headroom": 1},
        observed_at=1000,
    )
    obs2 = _reason(stable)
    exp2 = CAPACITY_2360_SECURITY_MATRIX[1][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R4 — a target that cannot fit projected allocatable headroom is not
    #    admitted just because its stable window elapsed.
    headroom = decide_downgrade(
        policy=policy,
        current={"machine_type": "n2-standard-8", "node_count": 3, "stable_since": 0, "last_transition_at": 0},
        proposed={"machine_type": "n2-standard-4", "node_count": 2, "projected_allocatable_headroom": 0},
        observed_at=1000,
    )
    obs3 = _reason(headroom)
    exp3 = CAPACITY_2360_SECURITY_MATRIX[2][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R4 — a proposed node count above the declared maximum is refused.
    maximum = decide_downgrade(
        policy=policy,
        current={"machine_type": "n2-standard-8", "node_count": 3, "stable_since": 0, "last_transition_at": 0},
        proposed={"machine_type": "n2-standard-4", "node_count": 4, "projected_allocatable_headroom": 1},
        observed_at=1000,
    )
    obs4 = _reason(maximum)
    exp4 = CAPACITY_2360_SECURITY_MATRIX[3][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R4 — an otherwise feasible reversal during cooldown remains refused.
    cooldown = decide_downgrade(
        policy=policy,
        current={"machine_type": "n2-standard-8", "node_count": 3, "stable_since": 0, "last_transition_at": 900},
        proposed={"machine_type": "n2-standard-4", "node_count": 2, "projected_allocatable_headroom": 1},
        observed_at=1000,
    )
    obs5 = _reason(cooldown)
    exp5 = CAPACITY_2360_SECURITY_MATRIX[4][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R5 — draining a voter is explicit forbidden input for PVC reclamation.
    voter_drain = decide_member_storage(
        catalog={"committed_desired_size": "200Gi", "storage_class": "standard"},
        member_role="voter",
        lifecycle_event="drained",
    )
    obs6 = voter_drain.reclaim.value
    exp6 = CAPACITY_2360_SECURITY_MATRIX[5][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R7 — the same machine type gets the same key even in different
    #    namespaces, preventing namespace-derived pool creation.
    separated = decide_pool_assignments(
        instances=(
            {"namespace": "alpha", "instance": "a", "machine_type": "n2-standard-8"},
            {"namespace": "bravo", "instance": "b", "machine_type": "n2-standard-8"},
        ),
        placements=({"instance": "a", "node": "node-a"}, {"instance": "b", "node": "node-b"}),
    )
    obs7 = separated.assignments["b"].pool_key
    exp7 = CAPACITY_2360_SECURITY_MATRIX[6][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R7 — shared-pool placement fails closed when the two data members are
    #    deliberately supplied on one physical node.
    collision = decide_pool_assignments(
        instances=(
            {"namespace": "alpha", "instance": "a", "machine_type": "n2-standard-8"},
            {"namespace": "bravo", "instance": "b", "machine_type": "n2-standard-8"},
        ),
        placements=({"instance": "a", "node": "node-a"}, {"instance": "b", "node": "node-a"}),
    )
    obs8 = _reason(collision)
    exp8 = CAPACITY_2360_SECURITY_MATRIX[7][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9-12. R8 — each named capacity failure becomes CapacityBlocked; a
    #        generic error or admission would hide the operator action needed.
    absent = decide_capacity_block(
        condition={"kind": "absent"}, old_member={"identifier": "member-0", "healthy": True}, generation=17
    )
    obs9 = absent.condition.type
    exp9 = CAPACITY_2360_SECURITY_MATRIX[8][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    at_maximum = decide_capacity_block(
        condition={"kind": "at_maximum"}, old_member={"identifier": "member-0", "healthy": True}, generation=17
    )
    obs10 = at_maximum.condition.type
    exp10 = CAPACITY_2360_SECURITY_MATRIX[9][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    quota = decide_capacity_block(
        condition={"kind": "quota_blocked"}, old_member={"identifier": "member-0", "healthy": True}, generation=17
    )
    obs11 = quota.condition.type
    exp11 = CAPACITY_2360_SECURITY_MATRIX[10][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    unschedulable = decide_capacity_block(
        condition={"kind": "unschedulable"}, old_member={"identifier": "member-0", "healthy": True}, generation=17
    )
    obs12 = unschedulable.condition.type
    exp12 = CAPACITY_2360_SECURITY_MATRIX[11][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13-15. R8 — blockage preserves the healthy incumbent, its identity, and
    #         generation rather than deleting it or starting a competing one.
    obs13 = quota.old_member.healthy
    exp13 = CAPACITY_2360_SECURITY_MATRIX[12][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    obs14 = quota.old_member.identifier
    exp14 = CAPACITY_2360_SECURITY_MATRIX[13][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    obs15 = quota.generation
    exp15 = CAPACITY_2360_SECURITY_MATRIX[14][1]
    checks.append({"name": CAPACITY_2360_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "capacity-2360-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
