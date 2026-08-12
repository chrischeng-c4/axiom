"""EC behavior case for #2955 -- converged-plane outage recovery.

Every expected value below is an EC-owned literal transcribed from #2955:
R2 preserves the supplied serving generations and starts no mutation during an
outage; R3 resumes exactly one persisted intent at its stored phase; R4 retains
controller-owned capacity through a fixed installation reapply; R5 admits a
capacity target that the supplied capacity can satisfy; and R6 keeps the
control-plane cost separate from each named data-plane instance cost.
"""

from __future__ import annotations

from lumen.control_plane.capacity import decide_target_capacity
from lumen.control_plane.cost import separate_costs
from lumen.control_plane.ownership import decide_reapply
from lumen.control_plane.recovery import decide_outage_state, decide_recovery
from lumen.control_plane.spec import PersistedIntent, RecoveryState, TransitionState
from lumen.control_plane.verdict import Rejection

MINIMUM_CHECKS = 15

CONTROL_PLANE_2955_BEHAVIOR_MATRIX = (
    ("outage_is_frozen_serving", "frozen-serving"),
    ("outage_retains_catalog_generation", 17),
    ("outage_retains_routing_generation", 23),
    ("outage_starts_no_requested_mutation", "not-started"),
    ("recovery_resumes_one_persisted_intent", "resume"),
    ("recovery_retains_persisted_intent_identifier", "reshard-orders-7"),
    ("recovery_retains_persisted_intent_phase", "catching-up"),
    ("recovery_contains_exactly_one_resume", 1),
    ("fixed_reapply_is_admitted", "admitted"),
    ("fixed_reapply_preserves_current_capacity", "n2-standard-4"),
    ("fixed_reapply_preserves_target_capacity", "c3-standard-4"),
    ("available_target_capacity_is_admitted", "admitted"),
    ("cost_keeps_control_plane_value_separate", 17.25),
    ("cost_keeps_orders_instance_value_separate", 4.5),
    ("cost_keeps_catalog_instance_value_separate", 8.75),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_control_plane_2955_behavior() -> dict:
    checks = []

    outage = decide_outage_state(False, 17, 23, True)

    # 1-4. R2 -- no available control-plane replica freezes new work while the
    # last supplied serving generations remain the actual serving values.
    obs1 = outage.kind if not isinstance(outage, Rejection) else outage.reason.value
    exp1 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = outage.catalog_generation if not isinstance(outage, Rejection) else -1
    exp2 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = outage.routing_generation if not isinstance(outage, Rejection) else -1
    exp3 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = outage.mutation_action if not isinstance(outage, Rejection) else outage.reason.value
    exp4 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    recovery = decide_recovery(RecoveryState(
        control_plane_available=True,
        persisted_intents=(PersistedIntent(intent_id="reshard-orders-7", phase="catching-up"),),
    ))

    # 5-8. R3 -- recovery carries one durable intent's identity and exact phase,
    # rather than reconstructing a new transition or restarting it at admission.
    obs5 = recovery.action if not isinstance(recovery, Rejection) else recovery.reason.value
    exp5 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = recovery.intent_id if not isinstance(recovery, Rejection) else "rejected"
    exp6 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = recovery.phase if not isinstance(recovery, Rejection) else "rejected"
    exp7 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = recovery.resumed_intent_count if not isinstance(recovery, Rejection) else -1
    exp8 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    reapplied = decide_reapply(
        {"installation_owner": "platform-team", "image": "lumen-operator:v2"},
        {"current_capacity": "n2-standard-4", "target_capacity": "c3-standard-4"},
        TransitionState(phase="preflight-pending", transition_id="capacity-11"),
    )

    # 9-11. R4 -- installation-owned fields apply, but controller capacity is
    # carried through the pure merge unchanged.
    obs9 = _outcome(reapplied)
    exp9 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = reapplied.current_capacity if not isinstance(reapplied, Rejection) else "rejected"
    exp10 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    obs11 = reapplied.target_capacity if not isinstance(reapplied, Rejection) else "rejected"
    exp11 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    capacity = decide_target_capacity(3, 5, 5)

    # 12. R5 -- a target at the supplied available capacity clears pure preflight.
    obs12 = _outcome(capacity)
    exp12 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    costs = separate_costs(17.25, {"orders": 4.5, "catalog": 8.75})

    # 13-15. R6 -- cost values preserve the control-plane and both supplied
    # instance dimensions, instead of collapsing them into a single total.
    obs13 = costs.control_plane_cost
    exp13 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = costs.instance_costs["orders"]
    exp14 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    obs15 = costs.instance_costs["catalog"]
    exp15 = CONTROL_PLANE_2955_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": CONTROL_PLANE_2955_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {"case_id": "control-plane-2955-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
