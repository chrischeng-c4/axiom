"""EC security case for #2946 -- fail-closed direct capacity admission.

Expected literals are EC-owned transcriptions of #2946 R1/R3/R4/R5/R6/R8/R10.
This is intentionally a pure model contract: live RBAC, Nodes, quota, and
operator restart survival are runtime-only and do not appear here.
"""

from __future__ import annotations

from lumen.capacity.admission import CapacityRequest, decide_capacity_spec, preflight_capacity
from lumen.capacity.catalog import CapacityCatalog, CatalogEntry, resolve_machine_type
from lumen.capacity.placement import Placement, resolve_shared_placement
from lumen.capacity.policy import CapacityPolicy, decide_transition
from lumen.capacity.resources import CapacityVector, derive_requests
from lumen.capacity.spec import CapacitySpec
from lumen.capacity.status import CapacityState, project_capacity_status
from lumen.capacity.verdict import Rejection

MINIMUM_CHECKS = 23

CAPACITY_2946_SECURITY_MATRIX = (
    ("service_tier_value_is_rejected_at_spec_admission", "unsupported_machine_type"),
    ("service_tier_refusal_names_initial_machine_type", "initial_machine_type"),
    ("direct_machine_type_neighbour_is_admitted", "admitted"),
    ("unsupported_direct_type_is_rejected_by_catalog", "unsupported_machine_type"),
    ("catalog_refusal_names_machine_type", "machine_type"),
    ("missing_catalog_is_blocked_before_disruption", "catalog_missing"),
    ("ambiguous_catalog_is_blocked_before_disruption", "catalog_ambiguous"),
    ("draining_catalog_is_blocked_before_disruption", "catalog_draining"),
    ("full_catalog_is_blocked_before_disruption", "capacity_full"),
    ("preflight_block_names_catalog", "catalog"),
    ("insufficient_allocatable_capacity_is_rejected", "insufficient_allocatable"),
    ("insufficient_allocatable_refusal_names_allocatable", "allocatable"),
    ("cross_namespace_duplicate_node_is_rejected", "data_member_node_conflict"),
    ("duplicate_node_refusal_names_placements", "placements"),
    ("monetary_policy_field_is_rejected", "monetary_policy_not_allowed"),
    ("blocked_status_is_capacity_blocked", "CapacityBlocked"),
    ("incompatible_catalog_is_blocked_before_disruption", "catalog_incompatible"),
    ("forbidden_transition_is_rejected", "transition_not_allowed"),
    ("transition_decision_exposes_configured_read_replica_cap", 6),
    ("transition_decision_exposes_configured_shard_cap", 9),
    ("currency_policy_field_is_rejected", "monetary_policy_not_allowed"),
    ("price_policy_field_is_rejected", "monetary_policy_not_allowed"),
    ("cost_ceiling_policy_field_is_rejected", "monetary_policy_not_allowed"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_capacity_2946_security() -> dict:
    checks = []

    # 1-3. R1 -- a service tier is not a public machine type; the explicit
    # direct-GCE neighbouring input remains admitted.
    tier = decide_capacity_spec(CapacitySpec(initial_machine_type="lumen-premium"))
    obs1 = _outcome(tier)
    exp1 = CAPACITY_2946_SECURITY_MATRIX[0][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    obs2 = tier.field_path if isinstance(tier, Rejection) else ""
    exp2 = CAPACITY_2946_SECURITY_MATRIX[1][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    direct = decide_capacity_spec(CapacitySpec(initial_machine_type="e2-standard-2"))
    obs3 = _outcome(direct)
    exp3 = CAPACITY_2946_SECURITY_MATRIX[2][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4-5. R3 -- catalog resolution independently refuses unknown direct types.
    unknown = resolve_machine_type("e2-ultramem-16", CapacityCatalog(entries=()))
    obs4 = _outcome(unknown)
    exp4 = CAPACITY_2946_SECURITY_MATRIX[3][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    obs5 = unknown.field_path if isinstance(unknown, Rejection) else ""
    exp5 = CAPACITY_2946_SECURITY_MATRIX[4][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    request = CapacityRequest(spec=CapacitySpec(initial_machine_type="e2-standard-2"), old_member_disrupted=False)

    # 6. R4 -- absent catalog data cannot be treated as an empty node inventory.
    missing = preflight_capacity(request, None)
    obs6 = _outcome(missing)
    exp6 = CAPACITY_2946_SECURITY_MATRIX[5][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R4 -- multiple matching catalog records are no safer than none.
    ambiguous = preflight_capacity(request, CapacityCatalog(entries=(CatalogEntry("e2-standard-2", "a", "ready", True, True, True, False), CatalogEntry("e2-standard-2", "b", "ready", True, True, True, False))))
    obs7 = _outcome(ambiguous)
    exp7 = CAPACITY_2946_SECURITY_MATRIX[6][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R4 -- a declared draining selector is never a replacement target.
    draining = preflight_capacity(request, CapacityCatalog(entries=(CatalogEntry("e2-standard-2", "a", "draining", True, True, True, False),)))
    obs8 = _outcome(draining)
    exp8 = CAPACITY_2946_SECURITY_MATRIX[7][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R4 -- explicit fullness also blocks before old-member disruption.
    full = preflight_capacity(request, CapacityCatalog(entries=(CatalogEntry("e2-standard-2", "a", "ready", True, True, True, True),)))
    obs9 = _outcome(full)
    exp9 = CAPACITY_2946_SECURITY_MATRIX[8][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R4 -- a blocked verdict identifies catalog input rather than an opaque error.
    obs10 = full.field_path if isinstance(full, Rejection) else ""
    exp10 = CAPACITY_2946_SECURITY_MATRIX[9][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11-12. R5 -- reserves plus headroom may exhaust allocatable capacity.
    insufficient = derive_requests(CapacityVector(cpu_millicores=1000, memory_mib=1024), CapacityVector(cpu_millicores=800, memory_mib=800), CapacityVector(cpu_millicores=300, memory_mib=300))
    obs11 = _outcome(insufficient)
    exp11 = CAPACITY_2946_SECURITY_MATRIX[10][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    obs12 = insufficient.field_path if isinstance(insufficient, Rejection) else ""
    exp12 = CAPACITY_2946_SECURITY_MATRIX[11][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13-14. R6 -- namespace and instance differences do not permit node sharing.
    conflict = resolve_shared_placement("e2-standard-2", CapacityCatalog(entries=(CatalogEntry("e2-standard-2", "a", "ready", True, True, True, False),)), (Placement(instance="lumen-a", namespace="alpha", node_name="node-1"), Placement(instance="lumen-b", namespace="beta", node_name="node-1")))
    obs13 = _outcome(conflict)
    exp13 = CAPACITY_2946_SECURITY_MATRIX[12][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    obs14 = conflict.field_path if isinstance(conflict, Rejection) else ""
    exp14 = CAPACITY_2946_SECURITY_MATRIX[13][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R8 -- cost-shaped configuration is rejected rather than becoming policy.
    monetary = decide_transition("e2-standard-2", "e2-standard-2", CapacityPolicy(allowed_transitions=("scale_out",), node_cap=3, read_replica_cap=2, shard_cap=4, cooldown_seconds=300, monthly_budget="100USD"), catalog_maximum=3)
    obs15 = _outcome(monetary)
    exp15 = CAPACITY_2946_SECURITY_MATRIX[14][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16. R10 -- a typed preflight block projects the public blocked status.
    status = project_capacity_status(CapacityState("e2-standard-2", "e2-standard-2", 17, "Stable"), full, old_member_healthy=True)
    obs16 = status.phase
    exp16 = CAPACITY_2946_SECURITY_MATRIX[15][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    # 17. R4 -- an incompatible catalog entry is a typed preflight block.
    incompatible = preflight_capacity(request, CapacityCatalog(entries=(CatalogEntry("e2-standard-2", "a", "ready", True, False, True, False),)))
    obs17 = _outcome(incompatible)
    exp17 = CAPACITY_2946_SECURITY_MATRIX[16][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    # 18. R8 -- a transition outside configured policy is refused.
    forbidden = decide_transition("e2-standard-2", "n2-standard-4", CapacityPolicy(allowed_transitions=(), node_cap=3, read_replica_cap=2, shard_cap=4, cooldown_seconds=300), catalog_maximum=3)
    obs18 = _outcome(forbidden)
    exp18 = CAPACITY_2946_SECURITY_MATRIX[17][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    admitted_policy = CapacityPolicy(allowed_transitions=("scale_out",), node_cap=3, read_replica_cap=6, shard_cap=9, cooldown_seconds=300)
    admitted_transition = decide_transition("e2-standard-2", "e2-standard-2", admitted_policy, catalog_maximum=3)

    # 19. R8 -- read-replica cap comes from operator configuration.
    obs19 = admitted_transition.read_replica_cap if not isinstance(admitted_transition, Rejection) else -1
    exp19 = CAPACITY_2946_SECURITY_MATRIX[18][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    # 20. R8 -- shard cap comes from operator configuration.
    obs20 = admitted_transition.shard_cap if not isinstance(admitted_transition, Rejection) else -1
    exp20 = CAPACITY_2946_SECURITY_MATRIX[19][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    # 21-23. R8 -- each monetary policy field is independently forbidden.
    currency = decide_transition("e2-standard-2", "e2-standard-2", CapacityPolicy(allowed_transitions=("scale_out",), node_cap=3, read_replica_cap=2, shard_cap=4, cooldown_seconds=300, currency="USD"), catalog_maximum=3)
    obs21 = _outcome(currency)
    exp21 = CAPACITY_2946_SECURITY_MATRIX[20][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})

    price = decide_transition("e2-standard-2", "e2-standard-2", CapacityPolicy(allowed_transitions=("scale_out",), node_cap=3, read_replica_cap=2, shard_cap=4, cooldown_seconds=300, price="0.10"), catalog_maximum=3)
    obs22 = _outcome(price)
    exp22 = CAPACITY_2946_SECURITY_MATRIX[21][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})

    cost_ceiling = decide_transition("e2-standard-2", "e2-standard-2", CapacityPolicy(allowed_transitions=("scale_out",), node_cap=3, read_replica_cap=2, shard_cap=4, cooldown_seconds=300, cost_ceiling="100USD"), catalog_maximum=3)
    obs23 = _outcome(cost_ceiling)
    exp23 = CAPACITY_2946_SECURITY_MATRIX[22][1]
    checks.append({"name": CAPACITY_2946_SECURITY_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})

    return {"case_id": "capacity-2946-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
