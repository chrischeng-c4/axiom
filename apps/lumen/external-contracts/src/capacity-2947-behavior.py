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

MINIMUM_CHECKS = 15

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
    ("incomplete_window_refuses_aggregation", True),
    ("mixed_generation_window_refuses_aggregation", True),
    ("read_dominated_window_retains_read_cpu_attribution", 31),
    ("non_read_dominated_window_retains_write_cpu_attribution", 37),
    ("paired_windows_keep_workload_cpu_latency_attribution_and_dominance", (31, 3, 7, 7, 11, 11, 13, 17, 17, 19, 5, 3, 37, 41, 11, 11, 13, 17, 17, 19, True, True)),
)


def _workloads(*, read_cpu: int = 5, write_cpu: int = 7, write_latency: int = 7) -> tuple[WorkloadObservation, ...]:
    return (
        WorkloadObservation(kind="read", requests=101, latency_ms=3, cpu=read_cpu),
        WorkloadObservation(kind="write", requests=103, latency_ms=write_latency, cpu=write_cpu),
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


def _synthetic_window(*, read_cpu: int, write_cpu: int, write_latency: int = 7) -> CapacityWindow:
    window = _complete_window()
    member = window.members[0]
    return CapacityWindow(
        members=(
            MemberSample(
                member_id=member.member_id, shard=member.shard, role=member.role, generation=member.generation,
                cpu=member.cpu, memory=member.memory,
                workloads=_workloads(read_cpu=read_cpu, write_cpu=write_cpu, write_latency=write_latency),
                storage=member.storage,
            ),
        ),
        expected_members=window.expected_members, generation=window.generation,
        complete=window.complete, fresh=window.fresh,
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

    # 11-12. R3 -- aggregation refuses incomplete and mixed-generation windows.
    incomplete = CapacityWindow(
        members=window.members, expected_members=window.expected_members, generation=73,
        complete=False, fresh=True,
    )
    obs11 = aggregate_window(incomplete, topology) is None
    exp11 = CAPACITY_2947_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    mixed_generation = CapacityWindow(
        members=(window.members[0], MemberSample(
            member_id="member-1", shard="shard-a", role="voter", generation=72,
            cpu=17, memory=29, workloads=_workloads(), storage=window.members[0].storage,
        )),
        expected_members=("member-0", "member-1"), generation=73, complete=True, fresh=True,
    )
    mixed_topology = Topology(generation=73, members=("member-0", "member-1"))
    obs12 = aggregate_window(mixed_generation, mixed_topology) is None
    exp12 = CAPACITY_2947_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13-15. AC1 -- paired synthetic windows retain each workload's CPU and
    # latency contribution while making read and write pressure distinguishable.
    read_dominated = aggregate_window(_synthetic_window(read_cpu=31, write_cpu=7), topology)
    write_dominated = aggregate_window(_synthetic_window(read_cpu=5, write_cpu=37, write_latency=41), topology)
    obs13 = read_dominated.by_instance.workloads["read"].cpu
    exp13 = CAPACITY_2947_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = write_dominated.by_instance.workloads["write"].cpu
    exp14 = CAPACITY_2947_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    read_workloads = read_dominated.by_instance.workloads
    write_workloads = write_dominated.by_instance.workloads
    obs15 = (
        read_workloads["read"].cpu, read_workloads["read"].latency_ms,
        read_workloads["write"].cpu, read_workloads["write"].latency_ms,
        read_workloads["compaction"].cpu, read_workloads["compaction"].latency_ms,
        read_workloads["snapshot"].cpu, read_workloads["snapshot"].latency_ms,
        read_workloads["raft_catch_up"].cpu, read_workloads["raft_catch_up"].latency_ms,
        write_workloads["read"].cpu, write_workloads["read"].latency_ms,
        write_workloads["write"].cpu, write_workloads["write"].latency_ms,
        write_workloads["compaction"].cpu, write_workloads["compaction"].latency_ms,
        write_workloads["snapshot"].cpu, write_workloads["snapshot"].latency_ms,
        write_workloads["raft_catch_up"].cpu, write_workloads["raft_catch_up"].latency_ms,
        read_workloads["read"].cpu > read_workloads["write"].cpu,
        write_workloads["write"].cpu > write_workloads["read"].cpu,
    )
    exp15 = CAPACITY_2947_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": CAPACITY_2947_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "capacity-2947-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
