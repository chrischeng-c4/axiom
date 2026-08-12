"""EC security case for #2941 -- fail-closed placement and disruption safety.

Expected values are EC-owned literals transcribed from issue #2941: R3 names
and locates cross-instance node co-location; R5 selects exactly committed
voters or the joint old/new union; R6 excludes readers; and R7/AC3 refuse an
eviction that would lose either joint quorum and sequence replacement coverage
before old-voter unprotection.
"""

from __future__ import annotations

from lumen.topology.admission import decide_placement
from lumen.topology.disruption import (
    can_voluntarily_evict,
    plan_membership_update,
    protected_voters,
)
from lumen.topology.verdict import Rejection

MINIMUM_CHECKS = 11

WORKLOAD_2941_SECURITY_MATRIX = (
    ("cross_instance_node_conflict_uses_the_exact_reason", "data_member_node_conflict"),
    ("cross_instance_node_conflict_names_the_node_field", "placement.node_name"),
    ("distinct_nodes_remain_admitted", "admitted"),
    ("steady_state_protects_exactly_committed_voters", ("lumen-orders-0-0", "lumen-orders-0-1", "lumen-orders-0-2")),
    ("joint_membership_protects_the_old_new_voter_union", ("lumen-orders-0-0", "lumen-orders-0-1", "lumen-orders-0-2", "lumen-orders-0-3")),
    ("voter_protection_excludes_read_replicas", ("lumen-orders-0-0", "lumen-orders-0-1", "lumen-orders-0-2")),
    ("old_joint_quorum_loss_refuses_voluntary_eviction", False),
    ("new_joint_quorum_loss_refuses_voluntary_eviction", False),
    ("reader_eviction_remains_admitted", True),
    ("replacement_protection_precedes_old_voter_unprotection", ("protect:lumen-orders-0-3", "unprotect:lumen-orders-0-0")),
    ("update_plan_covers_every_committed_voter", ("lumen-orders-0-1", "lumen-orders-0-2", "lumen-orders-0-3")),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_workload_2941_security() -> dict:
    checks = []

    conflicting = decide_placement((
        ("lumen-orders", "lumen-orders-0-0", "node-a"),
        ("lumen-catalog", "lumen-catalog-0-0", "node-a"),
    ))

    # 1. R3 -- a cross-instance collision uses the issue's precise vocabulary.
    obs1 = _outcome(conflicting)
    exp1 = WORKLOAD_2941_SECURITY_MATRIX[0][1]
    checks.append({"name": WORKLOAD_2941_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R3 -- the same refusal points at the unsafe placement field.
    obs2 = conflicting.field_path if isinstance(conflicting, Rejection) else ""
    exp2 = WORKLOAD_2941_SECURITY_MATRIX[1][1]
    checks.append({"name": WORKLOAD_2941_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R3 -- the nearest valid placement is not over-refused.
    distinct = decide_placement((
        ("lumen-orders", "lumen-orders-0-0", "node-a"),
        ("lumen-catalog", "lumen-catalog-0-0", "node-b"),
    ))
    obs3 = _outcome(distinct)
    exp3 = WORKLOAD_2941_SECURITY_MATRIX[2][1]
    checks.append({"name": WORKLOAD_2941_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    committed = {
        "voters": ("lumen-orders-0-0", "lumen-orders-0-1", "lumen-orders-0-2"),
        "read_replicas": ("lumen-orders-0-r0",),
    }
    voter_only_committed = {"voters": ("lumen-orders-0-0", "lumen-orders-0-1", "lumen-orders-0-2")}

    # 4. R5 -- steady state protects exactly the committed voter set.
    obs4 = tuple(protected_voters(voter_only_committed, None))
    exp4 = WORKLOAD_2941_SECURITY_MATRIX[3][1]
    checks.append({"name": WORKLOAD_2941_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    joint = {
        "old_voters": ("lumen-orders-0-0", "lumen-orders-0-1", "lumen-orders-0-2"),
        "new_voters": ("lumen-orders-0-1", "lumen-orders-0-2", "lumen-orders-0-3"),
        "read_replicas": ("lumen-orders-0-r0",),
    }

    # 5. R5 -- while joint, neither committed set may lose its protection.
    obs5 = tuple(protected_voters(committed, joint))
    exp5 = WORKLOAD_2941_SECURITY_MATRIX[4][1]
    checks.append({"name": WORKLOAD_2941_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R6 -- readers are excluded even when supplied beside committed voters.
    obs6 = tuple(protected_voters(committed, None))
    exp6 = WORKLOAD_2941_SECURITY_MATRIX[5][1]
    checks.append({"name": WORKLOAD_2941_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. AC3 -- removing an old-only voter takes the old set below quorum.
    obs7 = can_voluntarily_evict({"joint": joint, "healthy_members": ("lumen-orders-0-0", "lumen-orders-0-1", "lumen-orders-0-3")}, "lumen-orders-0-0")
    exp7 = WORKLOAD_2941_SECURITY_MATRIX[6][1]
    checks.append({"name": WORKLOAD_2941_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. AC3 -- removing a new-only voter takes the new set below quorum.
    obs8 = can_voluntarily_evict({"joint": joint, "healthy_members": ("lumen-orders-0-0", "lumen-orders-0-1", "lumen-orders-0-3")}, "lumen-orders-0-3")
    exp8 = WORKLOAD_2941_SECURITY_MATRIX[7][1]
    checks.append({"name": WORKLOAD_2941_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R6 -- a reader does not consume voter availability or block scale-in.
    obs9 = can_voluntarily_evict({"committed": committed, "healthy_members": ("lumen-orders-0-0", "lumen-orders-0-1", "lumen-orders-0-2", "lumen-orders-0-r0")}, "lumen-orders-0-r0")
    exp9 = WORKLOAD_2941_SECURITY_MATRIX[8][1]
    checks.append({"name": WORKLOAD_2941_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    update = plan_membership_update(
        {"voters": ("lumen-orders-0-0", "lumen-orders-0-1", "lumen-orders-0-2")},
        {"voters": ("lumen-orders-0-1", "lumen-orders-0-2", "lumen-orders-0-3")},
    )

    # 10. R7 -- the replacement becomes protected before the old voter is released.
    obs10 = (update["actions"][0], update["actions"][-1])
    exp10 = WORKLOAD_2941_SECURITY_MATRIX[9][1]
    checks.append({"name": WORKLOAD_2941_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R7 -- independently, every newly committed voter remains covered.
    obs11 = tuple(update["covered_voters"])
    exp11 = WORKLOAD_2941_SECURITY_MATRIX[10][1]
    checks.append({"name": WORKLOAD_2941_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "workload-2941-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
