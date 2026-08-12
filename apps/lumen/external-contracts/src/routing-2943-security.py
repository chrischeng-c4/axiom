"""EC security case for #2943 -- fail-closed routing decisions.

Every expected value is an EC-owned literal for #2943: R3 keeps incomplete or
generation-mismatched global mutations pending with reconcile work; R5 exposes
bounded reads only with a real apply-lag bound; R6 refuses pod-pinned and
restart-map coordinators; R7 refuses a second stale-map retry; and R8 keeps
peer mTLS membership separate from the original caller's authority.
"""

from __future__ import annotations

from lumen.routing.admission import (
    decide_coordinator,
    decide_global_visibility,
    decide_read_target,
    decide_stale_map_retry,
    forward_context,
)
from lumen.routing.spec import (
    Acknowledgement,
    CallerContext,
    CoordinatorTopology,
    MutationIntent,
    PeerContext,
    Replica,
    Request,
    StaleMapResponse,
)
from lumen.routing.verdict import Rejection

MINIMUM_CHECKS = 16

ROUTING_2943_SECURITY_MATRIX = (
    ("missing_global_acknowledgement_is_pending", "pending"),
    ("missing_acknowledgement_names_required_shards", "required_shards"),
    ("pending_global_mutation_has_reconcile_action", "reconcile"),
    ("generation_mismatched_acknowledgement_is_pending", "pending"),
    ("bounded_read_without_enforced_lag_is_rejected", "missing_apply_lag_bound"),
    ("bounded_read_refusal_names_replicas", "replicas"),
    ("bounded_read_with_enforced_lag_is_admitted", "admitted"),
    ("pinned_coordinator_is_rejected", "pod_pinning_not_allowed"),
    ("pinned_coordinator_refusal_names_pinned_pod", "pinned_pod"),
    ("restart_only_coordinator_map_is_rejected", "restart_only_map_not_allowed"),
    ("restart_map_refusal_names_restart_map", "restart_only_map"),
    ("ordinary_live_catalog_coordinator_is_admitted", "admitted"),
    ("second_stale_map_retry_is_rejected", "stale_map_retry_exhausted"),
    ("second_retry_refusal_names_retry_count", "retry_count"),
    ("peer_identity_cannot_replace_caller_authorization_principal", "ksa:tenant-a:denied"),
    ("forwarded_authorization_context_is_not_elevated_by_peer", "read:orders"),
)


def _outcome(verdict):
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_routing_2943_security() -> dict:
    checks = []
    intent = MutationIntent(mutation_id="m-schema", generation=41, required_shards=("shard-a", "shard-b"))

    # 1. R3 -- incomplete acknowledgement cannot be visible.
    missing = decide_global_visibility(intent, (Acknowledgement("shard-a", 41),))
    obs1 = missing.state
    exp1 = ROUTING_2943_SECURITY_MATRIX[0][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R3 -- the pending result identifies the missing durability boundary.
    obs2 = missing.field_path
    exp2 = ROUTING_2943_SECURITY_MATRIX[1][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R3 -- pending work carries an automatic convergence action.
    obs3 = missing.reconcile_action
    exp3 = ROUTING_2943_SECURITY_MATRIX[2][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R3 -- an acknowledgement from the wrong generation is equally not a
    # successful global mutation, even when every shard has replied.
    mismatched = decide_global_visibility(intent, (Acknowledgement("shard-a", 41), Acknowledgement("shard-b", 40)))
    obs4 = mismatched.state
    exp4 = ROUTING_2943_SECURITY_MATRIX[3][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R5 -- bounded reads fail closed without an enforced lag bound.
    unbounded = decide_read_target("bounded", (Replica("pod-read", voting=False, eligible=True, apply_lag_bound=None),))
    obs5 = _outcome(unbounded)
    exp5 = ROUTING_2943_SECURITY_MATRIX[4][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R5 -- the refusal identifies the unsafe candidate set.
    obs6 = unbounded.field_path if isinstance(unbounded, Rejection) else ""
    exp6 = ROUTING_2943_SECURITY_MATRIX[5][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R5 -- the immediate enforced-bound neighbour remains admitted.
    bounded = decide_read_target("bounded", (Replica("pod-read", voting=False, eligible=True, apply_lag_bound=25),))
    obs7 = _outcome(bounded)
    exp7 = ROUTING_2943_SECURITY_MATRIX[6][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R6 -- explicit pod pinning is forbidden.
    pinned = decide_coordinator(CoordinatorTopology(data_pods=("pod-a", "pod-b"), pinned_pod="pod-a"), Request(kind="query", key="orders/42"))
    obs8 = _outcome(pinned)
    exp8 = ROUTING_2943_SECURITY_MATRIX[7][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R6 -- pinning refusal names the explicit pinned-pod input.
    obs9 = pinned.field_path if isinstance(pinned, Rejection) else ""
    exp9 = ROUTING_2943_SECURITY_MATRIX[8][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R6 -- a restart-only coordinator map is separately forbidden.
    restart_map = decide_coordinator(CoordinatorTopology(data_pods=("pod-a", "pod-b"), restart_only_map={"orders/42": "pod-a"}), Request(kind="query", key="orders/42"))
    obs10 = _outcome(restart_map)
    exp10 = ROUTING_2943_SECURITY_MATRIX[9][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R6 -- restart-map refusal names the unsafe map input.
    obs11 = restart_map.field_path if isinstance(restart_map, Rejection) else ""
    exp11 = ROUTING_2943_SECURITY_MATRIX[10][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R6 -- a normal live-catalog topology remains admitted.
    live = decide_coordinator(CoordinatorTopology(data_pods=("pod-a", "pod-b")), Request(kind="query", key="orders/42"))
    obs12 = _outcome(live)
    exp12 = ROUTING_2943_SECURITY_MATRIX[11][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R7 -- a second stale-map signal is refused.
    exhausted = decide_stale_map_retry(Request(kind="index", key="orders/42", mutation_id="m-index"), StaleMapResponse(generation=40), 1)
    obs13 = _outcome(exhausted)
    exp13 = ROUTING_2943_SECURITY_MATRIX[12][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R7 -- the refusal identifies the exhausted retry budget.
    obs14 = exhausted.field_path if isinstance(exhausted, Rejection) else ""
    exp14 = ROUTING_2943_SECURITY_MATRIX[13][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R8 -- the peer cannot replace the denied caller as principal.
    forwarded = forward_context(CallerContext("ksa:tenant-a:denied", "read:orders", "req-2943", "trace-2943"), PeerContext("spiffe://lumen/cluster-admin"))
    obs15 = forwarded.authorization_principal
    exp15 = ROUTING_2943_SECURITY_MATRIX[14][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16. R8 -- peer membership cannot elevate the caller authorization scope.
    obs16 = forwarded.authorization_context
    exp16 = ROUTING_2943_SECURITY_MATRIX[15][1]
    checks.append({"name": ROUTING_2943_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    return {"case_id": "routing-2943-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
