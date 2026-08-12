"""EC behavior case for #2946 -- direct machine-type capacity.

The literals in this matrix are transcribed from #2946 R1/R2/R3/R5/R7/R8/R10.
They are owned by this EC, never obtained from the capacity design.  The case
only drives the pure design model; Kubernetes, Terraform, IAM, and persistence
claims are deliberately runtime-stage concerns.
"""

from __future__ import annotations

from lumen.capacity.admission import CapacityRequest, decide_capacity_spec, decide_storage, preflight_capacity
from lumen.capacity.catalog import CapacityCatalog, CatalogEntry, resolve_machine_type
from lumen.capacity.policy import CapacityPolicy, decide_transition
from lumen.capacity.placement import Placement, resolve_shared_placement
from lumen.capacity.resources import CapacityVector, derive_requests
from lumen.capacity.spec import CapacitySpec, CapacityStorage
from lumen.capacity.status import CapacityState, apply_capacity_reapplication, project_capacity_status
from lumen.capacity.verdict import Rejection

MINIMUM_CHECKS = 24

CAPACITY_2946_BEHAVIOR_MATRIX = (
    ("default_initial_machine_type_is_e2_standard_2", "e2-standard-2"),
    ("explicit_default_machine_type_is_admitted", "admitted"),
    ("unchanged_reapplication_keeps_current_machine_type", "e2-standard-2"),
    ("unchanged_reapplication_keeps_target_machine_type", "e2-standard-2"),
    ("unchanged_reapplication_keeps_transition_generation", 17),
    ("unchanged_reapplication_keeps_phase", "Stable"),
    ("allowed_machine_type_resolves_to_a_nonempty_selector", True),
    ("same_machine_type_resolves_to_the_same_selector", True),
    ("public_spec_has_no_pool_identity_field", False),
    ("derived_cpu_request_subtracts_reserves_and_headroom", 2000),
    ("derived_memory_request_subtracts_reserves_and_headroom", 12288),
    ("default_data_pvc_size_is_10_gib", "10Gi"),
    ("default_data_pvc_storage_class_is_standard_rwo", "standard-rwo"),
    ("default_data_pvc_uses_pd_balanced", "pd-balanced"),
    ("larger_data_pvc_is_admitted_for_growth", "admitted"),
    ("configured_cooldown_is_exposed_by_transition_decision", 300),
    ("blocked_status_keeps_old_healthy_member_authoritative", True),
    ("blocked_status_retains_current_machine_type", "e2-standard-2"),
    ("blocked_status_retains_target_machine_type", "e2-standard-2"),
    ("public_spec_has_exactly_one_machine_type_field", 1),
    ("derived_cpu_request_is_below_advertised_allocatable", True),
    ("derived_memory_request_is_below_advertised_allocatable", True),
    ("equal_machine_type_placements_share_one_selector", True),
    ("transition_decision_exposes_configured_node_cap", 3),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_capacity_2946_behavior() -> dict:
    checks = []
    default_spec = CapacitySpec.default()

    # 1. R1 -- the sole create-time public value starts as a direct GCE type.
    obs1 = default_spec.initial_machine_type
    exp1 = CAPACITY_2946_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- explicitly spelling the default still enters the admission path.
    admitted_default = decide_capacity_spec(CapacitySpec(initial_machine_type="e2-standard-2"))
    obs2 = _outcome(admitted_default)
    exp2 = CAPACITY_2946_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    previous = CapacityState(current_machine_type="e2-standard-2", target_machine_type="e2-standard-2", transition_generation=17, phase="Stable")
    reapplied = apply_capacity_reapplication(previous, CapacitySpec(initial_machine_type="e2-standard-2"))

    # 3. R2 -- no-op reapplication cannot replace operator-owned current state.
    obs3 = reapplied.current_machine_type
    exp3 = CAPACITY_2946_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R2 -- it retains the target intent as well.
    obs4 = reapplied.target_machine_type
    exp4 = CAPACITY_2946_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R2 -- a reapply is not a hidden transition.
    obs5 = reapplied.transition_generation
    exp5 = CAPACITY_2946_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R2 -- and it retains phase rather than restarting reconciliation.
    obs6 = reapplied.phase
    exp6 = CAPACITY_2946_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    catalog = CapacityCatalog(entries=(CatalogEntry(machine_type="e2-standard-2", selector="capacity.gke.io/class=e2-standard-2", lifecycle="ready", schema_valid=True, compatible=True, schedulable=True, full=False),))
    resolved_once = resolve_machine_type("e2-standard-2", catalog)
    resolved_twice = resolve_machine_type("e2-standard-2", catalog)

    # 7. R3 -- the catalog yields an opaque, usable selector for an allowed type.
    obs7 = bool(resolved_once.selector) if not isinstance(resolved_once, Rejection) else False
    exp7 = CAPACITY_2946_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R3 -- repeated resolution is stable, not a per-request pool choice.
    obs8 = resolved_once.selector == resolved_twice.selector if not isinstance(resolved_once, Rejection) and not isinstance(resolved_twice, Rejection) else False
    exp8 = CAPACITY_2946_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R3 -- the public request cannot carry a pool name back into Lumen.
    obs9 = any("pool" in name.lower() for name in CapacitySpec.__dataclass_fields__)
    exp9 = CAPACITY_2946_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R5 -- CPU is allocatable less explicitly supplied reserve and headroom.
    requests = derive_requests(CapacityVector(cpu_millicores=4000, memory_mib=16384), CapacityVector(cpu_millicores=1000, memory_mib=2048), CapacityVector(cpu_millicores=1000, memory_mib=2048))
    obs10 = requests.cpu_millicores if not isinstance(requests, Rejection) else -1
    exp10 = CAPACITY_2946_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R5 -- memory follows the independent allocatable calculation.
    obs11 = requests.memory_mib if not isinstance(requests, Rejection) else -1
    exp11 = CAPACITY_2946_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    storage = CapacityStorage.default()

    # 12. R7 -- a new data PVC starts at ten GiB.
    obs12 = storage.size
    exp12 = CAPACITY_2946_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R7 -- its portable public class is standard-rwo.
    obs13 = storage.storage_class
    exp13 = CAPACITY_2946_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R7 -- its zonal backing semantics select pd-balanced, not pd-ssd.
    obs14 = storage.disk_type
    exp14 = CAPACITY_2946_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R7 -- an explicit larger size is the admitted neighbouring request.
    grown = decide_storage(CapacityStorage(size="20Gi", storage_class="standard-rwo", disk_type="pd-balanced"))
    obs15 = _outcome(grown)
    exp15 = CAPACITY_2946_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    policy = CapacityPolicy(allowed_transitions=("scale_out",), node_cap=3, read_replica_cap=2, shard_cap=4, cooldown_seconds=300)
    transition = decide_transition("e2-standard-2", "e2-standard-2", policy, catalog_maximum=3)

    # 16. R8 -- policy-derived cooldown is surfaced by the decision, not invented.
    obs16 = transition.cooldown_seconds if not isinstance(transition, Rejection) else -1
    exp16 = CAPACITY_2946_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    blocked_verdict = preflight_capacity(
        CapacityRequest(spec=CapacitySpec(initial_machine_type="e2-standard-2"), old_member_disrupted=False),
        CapacityCatalog(entries=(CatalogEntry("e2-standard-2", "a", "ready", True, True, True, True),)),
    )
    blocked = project_capacity_status(previous, blocked_verdict, old_member_healthy=True)

    # 17. R10 -- blocked capacity leaves the old healthy member authoritative.
    obs17 = blocked.old_member_authoritative
    exp17 = CAPACITY_2946_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    # 18. R10 -- failure leaves the existing current intent visible to operators.
    obs18 = blocked.current_machine_type
    exp18 = CAPACITY_2946_BEHAVIOR_MATRIX[17][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    # 19. R10 -- failure also retains target intent rather than erasing the request.
    obs19 = blocked.target_machine_type
    exp19 = CAPACITY_2946_BEHAVIOR_MATRIX[18][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    # 20. R1 -- the public shape has one create-time machine-type choice, not aliases.
    obs20 = sum("machine_type" in name for name in CapacitySpec.__dataclass_fields__)
    exp20 = CAPACITY_2946_BEHAVIOR_MATRIX[19][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    # 21. R5 -- the CPU request never reaches advertised allocatable capacity.
    obs21 = requests.cpu_millicores < 4000 if not isinstance(requests, Rejection) else False
    exp21 = CAPACITY_2946_BEHAVIOR_MATRIX[20][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})

    # 22. R5 -- memory independently leaves capacity after reserve and headroom.
    obs22 = requests.memory_mib < 16384 if not isinstance(requests, Rejection) else False
    exp22 = CAPACITY_2946_BEHAVIOR_MATRIX[21][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})

    # 23. R6 -- two instances selecting the same type receive the shared selector.
    shared = resolve_shared_placement("e2-standard-2", catalog, (Placement(instance="lumen-a", namespace="alpha", node_name="node-a"), Placement(instance="lumen-b", namespace="beta", node_name="node-b")))
    obs23 = shared.selectors["lumen-a"] == shared.selectors["lumen-b"] if not isinstance(shared, Rejection) else False
    exp23 = CAPACITY_2946_BEHAVIOR_MATRIX[22][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})

    # 24. R8 -- the decision carries the configured cap, bounded by catalog maximum.
    obs24 = transition.node_cap if not isinstance(transition, Rejection) else -1
    exp24 = CAPACITY_2946_BEHAVIOR_MATRIX[23][1]
    checks.append({"name": CAPACITY_2946_BEHAVIOR_MATRIX[23][0], "expected": exp24, "observed": obs24, "passed": obs24 == exp24})

    return {"case_id": "capacity-2946-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
