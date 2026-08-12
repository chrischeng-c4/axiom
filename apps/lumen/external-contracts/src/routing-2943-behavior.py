"""EC behavior case for #2943 -- generation-aware request routing.

Every expected value below is an EC-owned literal transcribed from #2943:
R1 resolves the supplied current catalog generation for every request kind; R2
uses an owning shard for keyed requests and a generation-bound durable intent
for global mutations; R3 exposes global mutation visibility only after all
required acknowledgements match; R4 forwards writes and leader reads to the
leader; R5 permits an eligible non-voter for ``any`` reads; R6 admits every
data-pod coordinator; R7 performs one refresh-and-retry; R8 preserves caller
context; and AC5 explicitly routes the non-index/query API paths.
"""

from __future__ import annotations

from lumen.routing.admission import (
    decide_coordinator,
    decide_global_visibility,
    decide_read_target,
    decide_request_route,
    decide_stale_map_retry,
    forward_context,
)
from lumen.routing.spec import (
    Acknowledgement,
    CallerContext,
    Catalog,
    CoordinatorTopology,
    MutationIntent,
    PeerContext,
    Replica,
    Request,
    StaleMapResponse,
)
from lumen.routing.verdict import Rejection

MINIMUM_CHECKS = 17

ROUTING_2943_BEHAVIOR_MATRIX = (
    ("index_route_uses_current_catalog_generation", 41),
    ("query_route_uses_current_catalog_generation", 41),
    ("delete_route_uses_current_catalog_generation", 41),
    ("get_route_uses_current_catalog_generation", 41),
    ("backup_route_uses_current_catalog_generation", 41),
    ("collection_route_uses_current_catalog_generation", 41),
    ("admin_route_uses_current_catalog_generation", 41),
    ("keyed_index_routes_to_owning_shard", "shard-b"),
    ("global_schema_mutation_creates_generation_bound_durable_intent", "reconcile"),
    ("all_matching_acknowledgements_make_global_mutation_visible", "visible"),
    ("leader_consistent_read_forwards_to_leader", "pod-leader"),
    ("write_forwards_to_leader", "pod-leader"),
    ("any_read_may_select_eligible_non_voter", "pod-read-replica"),
    ("each_declared_data_pod_is_an_admitted_coordinator", "admitted"),
    ("first_stale_map_response_refreshes_and_retries", "refresh_and_retry"),
    ("forwarding_retains_verified_caller_identity", "ksa:tenant-a:writer"),
    ("forwarding_retains_request_and_trace_context", ("req-2943", "trace-2943")),
)


def _outcome(verdict):
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_routing_2943_behavior() -> dict:
    checks = []
    catalog = Catalog(
        generation=41,
        keyed_owners={"orders/42": "shard-b"},
        leaders={"shard-b": "pod-leader"},
    )

    # 1. R1 -- index records the supplied live catalog generation.
    index = decide_request_route(Request(kind="index", key="orders/42", mutation_id="m-index"), catalog)
    obs1 = index.catalog_generation
    exp1 = ROUTING_2943_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- query records the supplied live catalog generation.
    query = decide_request_route(Request(kind="query", key="orders/42"), catalog)
    obs2 = query.catalog_generation
    exp2 = ROUTING_2943_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1/AC5 -- delete cannot bypass generation-aware routing.
    delete = decide_request_route(Request(kind="delete", key="orders/42", mutation_id="m-delete"), catalog)
    obs3 = delete.catalog_generation
    exp3 = ROUTING_2943_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R1 -- get records the supplied live catalog generation.
    get = decide_request_route(Request(kind="get", key="orders/42"), catalog)
    obs4 = get.catalog_generation
    exp4 = ROUTING_2943_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R1/AC5 -- backup coordination cannot bypass the catalog.
    backup = decide_request_route(Request(kind="backup_coordination", collection="orders", mutation_id="m-backup"), catalog)
    obs5 = backup.catalog_generation
    exp5 = ROUTING_2943_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R1/AC5 -- collection mutation records the current generation.
    collection = decide_request_route(Request(kind="collection", collection="orders", mutation_id="m-collection"), catalog)
    obs6 = collection.catalog_generation
    exp6 = ROUTING_2943_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R1/AC5 -- admin mutation also resolves the live catalog.
    admin = decide_request_route(Request(kind="admin", collection="orders", mutation_id="m-admin"), catalog)
    obs7 = admin.catalog_generation
    exp7 = ROUTING_2943_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R2 -- a keyed operation names its owner, not a generic fan-out plan.
    obs8 = index.shard
    exp8 = ROUTING_2943_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R2 -- a global schema change has durable, generation-bound reconcile
    # work; it is not represented as best-effort per-shard fan-out.
    schema = decide_request_route(Request(kind="schema", collection="orders", mutation_id="m-schema"), catalog)
    obs9 = schema.reconcile_action
    exp9 = ROUTING_2943_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R3 -- visibility follows matching acknowledgement of the intent's
    # catalog generation by every required shard.
    intent = MutationIntent(mutation_id="m-schema", generation=41, required_shards=("shard-a", "shard-b"))
    visible = decide_global_visibility(intent, (Acknowledgement("shard-a", 41), Acknowledgement("shard-b", 41)))
    obs10 = visible.state
    exp10 = ROUTING_2943_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R4 -- leader-consistent reads name the leader forward target.
    leader_read = decide_request_route(Request(kind="get", key="orders/42", consistency="leader"), catalog)
    obs11 = leader_read.forward_target
    exp11 = ROUTING_2943_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R4 -- writes also name the leader forward target.
    obs12 = index.forward_target
    exp12 = ROUTING_2943_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R5 -- any-consistency can select an eligible non-voting replica.
    any_target = decide_read_target("any", (Replica("pod-leader", voting=True, eligible=True), Replica("pod-read-replica", voting=False, eligible=True)))
    obs13 = any_target.pod
    exp13 = ROUTING_2943_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R6 -- the coordinator decision admits every named data pod.
    coordinator = decide_coordinator(CoordinatorTopology(data_pods=("pod-a", "pod-b", "pod-c")), Request(kind="query", key="orders/42"))
    obs14 = _outcome(coordinator)
    exp14 = ROUTING_2943_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R7 -- retry count zero is exactly the one allowed stale-map retry.
    retry = decide_stale_map_retry(Request(kind="index", key="orders/42", mutation_id="m-index"), StaleMapResponse(generation=40), 0)
    obs15 = retry.action
    exp15 = ROUTING_2943_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16. R8 -- forwarding preserves the verified caller identity.
    forwarded = forward_context(CallerContext("ksa:tenant-a:writer", "write:orders", "req-2943", "trace-2943"), PeerContext("spiffe://lumen/pod-b"))
    obs16 = forwarded.caller_identity
    exp16 = ROUTING_2943_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    # 17. R8 -- forwarding separately preserves request and trace correlation.
    obs17 = (forwarded.request_id, forwarded.trace_context)
    exp17 = ROUTING_2943_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": ROUTING_2943_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    return {"case_id": "routing-2943-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
