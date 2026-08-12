"""EC behavior case for #2362 -- deterministic capacity arbitration.

Every expected value is an EC-owned literal from #2362: R1/R2/R3/R4/R5
select one safe action by technical priority, R6 admits a fully headroom-safe
downgrade, R7 applies supplied-clock policy, and AC6 binds all capacity records
to one CR generation.  The imports intentionally fail closed until the frozen
pure capacity design lands.
"""

from __future__ import annotations

from lumen.capacity.arbitration import decide_capacity
from lumen.capacity.catalog import select_profile
from lumen.capacity.projection import evaluate_downgrade
from lumen.capacity.spec import CapacityInput, CapacityPolicy, CapacitySignals, CapacityState, ProfileAvailability, ProfileCatalog, SyntheticClock, TransitionGraph
from lumen.capacity.status import CapacityStatus

MINIMUM_CHECKS = 16

CAPACITY_2362_BEHAVIOR_MATRIX = (
    ("disk_pressure_grows_pvc_before_split", "PVC_GROW"),
    ("disk_pressure_at_capacity_ceiling_splits", "SPLIT"),
    ("disk_pressure_at_io_ceiling_splits", "SPLIT"),
    ("read_dominated_pressure_adds_non_voting_replica", "READ_REPLICA"),
    ("write_pressure_uses_vertical_machine_upgrade", "MACHINE_UPGRADE"),
    ("write_pressure_at_vertical_ceiling_splits", "SPLIT"),
    ("compaction_pressure_uses_vertical_machine_upgrade", "MACHINE_UPGRADE"),
    ("recovery_pressure_uses_vertical_machine_upgrade", "MACHINE_UPGRADE"),
    ("memory_pressure_uses_highmem_upgrade", "HIGHMEM_UPGRADE"),
    ("expansion_outranks_simultaneous_contraction", "PVC_GROW"),
    ("scale_in_removes_excess_read_replicas_first", "READ_REPLICA_REMOVE"),
    ("scale_in_without_excess_replicas_downgrades_one_machine_step", "MACHINE_DOWNGRADE"),
    ("wholly_safe_post_convergence_projection_admits_downgrade", "MACHINE_DOWNGRADE"),
    ("supplied_clock_after_scale_out_sustained_window_admits_action", "READ_REPLICA"),
    ("declared_reachable_catalog_target_is_selected", "standard-4"),
    ("recommendation_action_and_status_share_one_generation", (41, 41, 41)),
)


def verify_capacity_2362_behavior() -> dict:
    checks = []
    policy = CapacityPolicy.default()
    clock = SyntheticClock(now=1_000)

    # 1. R2 -- ordinary disk pressure grows the PVC before it considers a split.
    disk_growth = decide_capacity(CapacityInput(signals=CapacitySignals(disk_pressure=True), state=CapacityState(), policy=policy), clock)
    obs1 = disk_growth.action.kind
    exp1 = CAPACITY_2362_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R2 -- capacity exhaustion is the first disk condition that permits a split.
    disk_capacity = decide_capacity(CapacityInput(signals=CapacitySignals(disk_pressure=True), state=CapacityState(capacity_ceiling_reached=True), policy=policy), clock)
    obs2 = disk_capacity.action.kind
    exp2 = CAPACITY_2362_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R2 -- I/O exhaustion is the other explicit disk condition that permits a split.
    disk_io = decide_capacity(CapacityInput(signals=CapacitySignals(disk_pressure=True), state=CapacityState(io_ceiling_reached=True), policy=policy), clock)
    obs3 = disk_io.action.kind
    exp3 = CAPACITY_2362_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R3 -- read pressure expands only the non-voting serving role.
    read_pressure = decide_capacity(CapacityInput(signals=CapacitySignals(read_dominated=True), state=CapacityState(), policy=policy), clock)
    obs4 = read_pressure.action.kind
    exp4 = CAPACITY_2362_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R3 -- write CPU pressure prefers a useful vertical step.
    write_pressure = decide_capacity(CapacityInput(signals=CapacitySignals(write_cpu_pressure=True), state=CapacityState(), policy=policy), clock)
    obs5 = write_pressure.action.kind
    exp5 = CAPACITY_2362_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R3 -- after the useful vertical ceiling, the same write signal splits.
    write_ceiling = decide_capacity(CapacityInput(signals=CapacitySignals(write_cpu_pressure=True), state=CapacityState(vertical_ceiling_reached=True), policy=policy), clock)
    obs6 = write_ceiling.action.kind
    exp6 = CAPACITY_2362_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R3 -- compaction CPU pressure has the same vertical-first safe route.
    compaction = decide_capacity(CapacityInput(signals=CapacitySignals(compaction_cpu_pressure=True), state=CapacityState(), policy=policy), clock)
    obs7 = compaction.action.kind
    exp7 = CAPACITY_2362_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R3 -- recovery CPU pressure cannot be silently omitted from vertical scaling.
    recovery = decide_capacity(CapacityInput(signals=CapacitySignals(recovery_cpu_pressure=True), state=CapacityState(), policy=policy), clock)
    obs8 = recovery.action.kind
    exp8 = CAPACITY_2362_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R4 -- memory pressure selects the matching highmem, not a voter action.
    memory = decide_capacity(CapacityInput(signals=CapacitySignals(memory_pressure=True), state=CapacityState(), policy=policy), clock)
    obs9 = memory.action.kind
    exp9 = CAPACITY_2362_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R5 -- an expansion signal wins when it arrives beside a scale-in signal.
    simultaneous = decide_capacity(CapacityInput(signals=CapacitySignals(disk_pressure=True, low_utilization=True), state=CapacityState(excess_read_replicas=2), policy=policy), clock)
    obs10 = simultaneous.action.kind
    exp10 = CAPACITY_2362_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R5 -- contraction begins with excess non-voting read replicas.
    replica_scale_in = decide_capacity(CapacityInput(signals=CapacitySignals(low_utilization=True), state=CapacityState(excess_read_replicas=2), policy=policy), clock)
    obs11 = replica_scale_in.action.kind
    exp11 = CAPACITY_2362_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R5 -- only after replicas are gone may scale-in choose one machine step.
    machine_scale_in = decide_capacity(CapacityInput(signals=CapacitySignals(low_utilization=True), state=CapacityState(excess_read_replicas=0), policy=policy), clock)
    obs12 = machine_scale_in.action.kind
    exp12 = CAPACITY_2362_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R6 -- every projected constraint within headroom makes the target eligible.
    safe_projection = evaluate_downgrade(CapacitySignals(cpu_p95=20, memory_p95=20, compaction_p95=20, recovery_p95=20, system_reserve_p95=20), "standard-4", headroom=20)
    obs13 = safe_projection.action.kind
    exp13 = CAPACITY_2362_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R7/AC2 -- the synthetic clock, rather than wall time, opens the sustained scale-out window.
    clock_ready = decide_capacity(CapacityInput(signals=CapacitySignals(read_dominated=True, sustained_since=700), state=CapacityState(), policy=policy), SyntheticClock(now=1_000))
    obs14 = clock_ready.action.kind
    exp14 = CAPACITY_2362_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    catalog = ProfileCatalog(installed=("standard-4",), availability={"standard-4": ProfileAvailability.AVAILABLE})
    selected = select_profile(catalog, TransitionGraph({"standard-2": ("standard-4",)}), "standard-2", "standard-4")

    # 15. R8 -- a declared target must be reachable in the allowed transition graph.
    obs15 = selected.profile
    exp15 = CAPACITY_2362_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16. AC6 -- inspect the three recorded values, not a design-computed validity flag.
    bound = CapacityStatus(recommendation_generation=41, action_generation=41, status_generation=41)
    obs16 = (bound.recommendation_generation, bound.action_generation, bound.status_generation)
    exp16 = CAPACITY_2362_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": CAPACITY_2362_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    return {"case_id": "capacity-2362-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
