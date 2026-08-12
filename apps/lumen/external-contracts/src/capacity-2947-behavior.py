"""EC behavior case for #2947 -- topology-bound capacity observations.

Every expected value is an EC-owned literal transcribed from #2947: R1 records
CPU, memory, and separately attributable read, write, compaction, snapshot,
and Raft catch-up work; R2 accepts complete mounted-filesystem/PVC evidence;
R3 aggregates a complete current-generation window by shard, member role, and
instance; R5 publishes derived fresh signals without member payload; and AC1
keeps the five workload kinds distinct.  The imports deliberately fail closed
until the frozen pure-Python capacity design lands.
"""

from __future__ import annotations

from lumen.capacity.admission import decide_capacity_window
from lumen.capacity.aggregate import aggregate_window
from lumen.capacity.projection import redact_status
from lumen.capacity.spec import CapacityWindow, MemberSample, StorageObservation, Topology, WorkloadObservation
from lumen.capacity.verdict import AcceptedWindow

MINIMUM_CHECKS = 10

CAPACITY_2947_BEHAVIOR_MATRIX = (
    ("complete_current_window_is_accepted", "accepted"),
    ("aggregate_retains_shard_cpu_observation", 17),
    ("aggregate_retains_member_role_memory_observation", 29),
    ("aggregate_is_tagged_with_the_current_topology_generation", 73),
    ("aggregate_retains_read_request_attribution", 101),
    ("aggregate_retains_write_latency_attribution", 7),
    ("aggregate_retains_compaction_latency_attribution", 11),
    ("aggregate_retains_snapshot_request_attribution", 13),
    ("aggregate_retains_raft_catch_up_latency_attribution", 19),
    ("redacted_status_publishes_only_derived_signals_and_freshness", ("cpu", "freshness", "memory", "reason", "storage", "workloads")),
)


def _workloads() -> tuple[WorkloadObservation, ...]:
    return (
        WorkloadObservation(kind="read", requests=101, latency_ms=3, cpu=5),
        WorkloadObservation(kind="write", requests=103, latency_ms=7, cpu=7),
        WorkloadObservation(kind="compaction", requests=107, latency_ms=11, cpu=11),
        WorkloadObservation(kind="snapshot", requests=13, latency_ms=17, cpu=13),
        WorkloadObservation(kind="raft_catch_up", requests=127, latency_ms=19, cpu=17),
    )


def _complete_window() -> CapacityWindow:
    storage = StorageObservation(
        source="mounted_filesystem",
        used_bytes=1000,
        capacity_bytes=4000,
        growth_headroom_bytes=3000,
        latency_ms=23,
        saturation=31,
    )
    return CapacityWindow(
        members=(
            MemberSample(
                member_id="member-0", shard="shard-a", role="voter", generation=73,
                cpu=17, memory=29, workloads=_workloads(), storage=storage,
            ),
        ),
        expected_members=("member-0",),
        generation=73,
        complete=True,
        fresh=True,
    )


def verify_capacity_2947_behavior() -> dict:
    checks = []
    window = _complete_window()
    topology = Topology(generation=73, members=("member-0",))

    # 1. R1/R2/R3 -- a complete, fresh window names every required signal and
    #    has physical storage evidence, so it is the neighbouring admissible shape.
    admitted = decide_capacity_window(window)
    obs1 = "accepted" if isinstance(admitted, AcceptedWindow) else admitted.reason.value
    exp1 = CAPACITY_2947_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    aggregate = aggregate_window(window, topology)

    # 2. R3 -- shard aggregation keeps the CPU signal attributable to shard-a.
    obs2 = aggregate.by_shard["shard-a"].cpu
    exp2 = CAPACITY_2947_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1/R3 -- role aggregation retains memory separately from CPU.
    obs3 = aggregate.by_member_role["voter"].memory
    exp3 = CAPACITY_2947_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R3 -- a usable aggregate carries the one current topology generation.
    obs4 = aggregate.generation
    exp4 = CAPACITY_2947_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5-9. R1/AC1 -- the workload contribution records are distinct; no generic
    #        "workload pressure" value may erase their request/latency attribution.
    obs5 = aggregate.by_instance.workloads["read"].requests
    exp5 = CAPACITY_2947_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    obs6 = aggregate.by_instance.workloads["write"].latency_ms
    exp6 = CAPACITY_2947_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    obs7 = aggregate.by_instance.workloads["compaction"].latency_ms
    exp7 = CAPACITY_2947_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    obs8 = aggregate.by_instance.workloads["snapshot"].requests
    exp8 = CAPACITY_2947_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    obs9 = aggregate.by_instance.workloads["raft_catch_up"].latency_ms
    exp9 = CAPACITY_2947_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R5 -- the published shape has derived signals and freshness/reason,
    #     but its key set cannot expose the member sample or user content.
    status = redact_status(admitted)
    obs10 = tuple(sorted(status))
    exp10 = CAPACITY_2947_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "capacity-2947-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
