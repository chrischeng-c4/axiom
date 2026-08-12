"""EC security case for #2947 -- capacity telemetry fails closed.

Every expected value is an EC-owned literal from #2947: R1 refuses absent
member signals; R2 refuses a logical/in-memory storage estimate and incomplete
physical storage evidence; R3/R4/AC3 hold scale-in for missing, stale,
mixed-generation, or partial populations; and R5 redacts member payload and
user content.  Each refusal is driven with explicit forbidden input, checks its
reason vocabulary and named field, and leaves a neighbouring admissible input
observable.  The absent design imports are intentional.
"""

from __future__ import annotations

from lumen.capacity.admission import decide_capacity_window, decide_scale_in_eligibility
from lumen.capacity.projection import redact_status
from lumen.capacity.spec import CapacityWindow, MemberSample, StorageObservation, Topology, WorkloadObservation
from lumen.capacity.verdict import AcceptedWindow, Hold, Rejection

MINIMUM_CHECKS = 33

CAPACITY_2947_SECURITY_MATRIX = (
    ("missing_write_workload_is_rejected", "missing_signal"),
    ("missing_write_workload_rejection_names_the_workload", "workloads.write"),
    ("complete_workload_neighbour_is_accepted", "accepted"),
    ("in_memory_storage_estimate_is_rejected", "in_memory_storage_source"),
    ("in_memory_storage_rejection_names_the_source", "storage.source"),
    ("mounted_filesystem_without_capacity_is_rejected", "missing_storage_field"),
    ("physical_storage_rejection_names_capacity", "storage.capacity_bytes"),
    ("missing_read_workload_is_rejected", "missing_signal"),
    ("missing_read_workload_rejection_names_the_workload", "workloads.read"),
    ("missing_compaction_workload_is_rejected", "missing_signal"),
    ("missing_compaction_workload_rejection_names_the_workload", "workloads.compaction"),
    ("missing_snapshot_workload_is_rejected", "missing_signal"),
    ("missing_snapshot_workload_rejection_names_the_workload", "workloads.snapshot"),
    ("missing_raft_catch_up_workload_is_rejected", "missing_signal"),
    ("missing_raft_catch_up_workload_rejection_names_the_workload", "workloads.raft_catch_up"),
    ("missing_cpu_is_rejected", "missing_signal"),
    ("missing_cpu_rejection_names_the_signal", "cpu"),
    ("missing_memory_is_rejected", "missing_signal"),
    ("missing_memory_rejection_names_the_signal", "memory"),
    ("missing_expected_member_is_rejected", "missing_signal"),
    ("missing_expected_member_rejection_names_the_member", "members.member-1"),
    ("mounted_filesystem_without_used_bytes_names_the_field", "storage.used_bytes"),
    ("mounted_filesystem_without_growth_headroom_names_the_field", "storage.growth_headroom_bytes"),
    ("mounted_filesystem_without_latency_names_the_field", "storage.latency_ms"),
    ("mounted_filesystem_without_saturation_names_the_field", "storage.saturation"),
    ("missing_population_holds_scale_in", "missing"),
    ("missing_population_hold_names_the_member", "members.member-1"),
    ("fresh_complete_current_population_is_eligible", "eligible"),
    ("stale_population_holds_scale_in", "stale"),
    ("mixed_generation_population_holds_scale_in", "mixed_generation"),
    ("partial_population_holds_scale_in", "partial"),
    ("redacted_status_excludes_member_payload", ()),
    ("redacted_status_excludes_user_content", ()),
)


def _workloads(*, omit: str | None = None) -> tuple[WorkloadObservation, ...]:
    values = (
        ("read", 101, 3), ("write", 103, 7), ("compaction", 107, 11),
        ("snapshot", 109, 13), ("raft_catch_up", 113, 17),
    )
    return tuple(
        WorkloadObservation(kind=kind, requests=requests, latency_ms=latency, cpu=5)
        for kind, requests, latency in values if kind != omit
    )


def _storage(*, source: str = "mounted_filesystem", used_bytes: int | None = 1000,
             capacity_bytes: int | None = 4000, growth_headroom_bytes: int | None = 3000,
             latency_ms: int | None = 23, saturation: int | None = 31) -> StorageObservation:
    return StorageObservation(
        source=source, used_bytes=used_bytes, capacity_bytes=capacity_bytes,
        growth_headroom_bytes=growth_headroom_bytes, latency_ms=latency_ms, saturation=saturation,
    )


def _window(*, members: tuple[MemberSample, ...], expected_members: tuple[str, ...] = ("member-0",), generation: int = 73, complete: bool = True, fresh: bool = True) -> CapacityWindow:
    return CapacityWindow(members=members, expected_members=expected_members, generation=generation, complete=complete, fresh=fresh)


def _member(member_id: str = "member-0", *, generation: int = 73, omit: str | None = None,
            cpu: int | None = 17, memory: int | None = 29,
            storage: StorageObservation | None = None) -> MemberSample:
    return MemberSample(
        member_id=member_id, shard="shard-a", role="voter", generation=generation,
        cpu=cpu, memory=memory, workloads=_workloads(omit=omit), storage=storage or _storage(),
    )


def _outcome(verdict) -> str:
    return "accepted" if isinstance(verdict, AcceptedWindow) else verdict.reason.value


def verify_capacity_2947_security() -> dict:
    checks = []
    topology = Topology(generation=73, members=("member-0", "member-1"))

    # 1-2. R1 -- explicit omission of write exercises both the signal guard and
    #        the field path a user needs to repair it.
    missing_write = decide_capacity_window(_window(members=(_member(omit="write"),)))
    obs1 = _outcome(missing_write)
    exp1 = CAPACITY_2947_SECURITY_MATRIX[0][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = missing_write.field_path if isinstance(missing_write, Rejection) else ""
    exp2 = CAPACITY_2947_SECURITY_MATRIX[1][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1 -- its nearest complete neighbour remains admissible.
    complete = decide_capacity_window(_window(members=(_member(),)))
    obs3 = _outcome(complete)
    exp3 = CAPACITY_2947_SECURITY_MATRIX[2][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4-5. R2/AC2 -- logical engine bytes are never physical-disk evidence.
    in_memory = decide_capacity_window(_window(members=(_member(storage=_storage(source="in_memory_estimate")),)))
    obs4 = _outcome(in_memory)
    exp4 = CAPACITY_2947_SECURITY_MATRIX[3][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = in_memory.field_path if isinstance(in_memory, Rejection) else ""
    exp5 = CAPACITY_2947_SECURITY_MATRIX[4][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6-7. R2 -- even a declared physical source must name every storage field.
    missing_capacity = decide_capacity_window(_window(members=(_member(storage=_storage(capacity_bytes=None)),)))
    obs6 = _outcome(missing_capacity)
    exp6 = CAPACITY_2947_SECURITY_MATRIX[5][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = missing_capacity.field_path if isinstance(missing_capacity, Rejection) else ""
    exp7 = CAPACITY_2947_SECURITY_MATRIX[6][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8-21. R1 -- every workload, member resource signal, and expected member
    #   is independently required by capacity-window admission, and each refusal
    #   names the one signal it refused for. Written as one row per signal
    #   rather than a loop: a row appended inside a loop is a single static
    #   site, so self-report and distinct-checks would inspect it once and these
    #   fourteen observations would pass those gates unexamined.
    def _refusal(window) -> tuple[str, str]:
        verdict = decide_capacity_window(window)
        return (_outcome(verdict), verdict.field_path if isinstance(verdict, Rejection) else "")

    obs_read, path_read = _refusal(_window(members=(_member(omit="read"),)))
    exp_read = CAPACITY_2947_SECURITY_MATRIX[7][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[7][0], "expected": exp_read, "observed": obs_read, "passed": obs_read == exp_read})
    exp_read_path = CAPACITY_2947_SECURITY_MATRIX[8][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[8][0], "expected": exp_read_path, "observed": path_read, "passed": path_read == exp_read_path})

    obs_compaction, path_compaction = _refusal(_window(members=(_member(omit="compaction"),)))
    exp_compaction = CAPACITY_2947_SECURITY_MATRIX[9][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[9][0], "expected": exp_compaction, "observed": obs_compaction, "passed": obs_compaction == exp_compaction})
    exp_compaction_path = CAPACITY_2947_SECURITY_MATRIX[10][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[10][0], "expected": exp_compaction_path, "observed": path_compaction, "passed": path_compaction == exp_compaction_path})

    obs_snapshot, path_snapshot = _refusal(_window(members=(_member(omit="snapshot"),)))
    exp_snapshot = CAPACITY_2947_SECURITY_MATRIX[11][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[11][0], "expected": exp_snapshot, "observed": obs_snapshot, "passed": obs_snapshot == exp_snapshot})
    exp_snapshot_path = CAPACITY_2947_SECURITY_MATRIX[12][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[12][0], "expected": exp_snapshot_path, "observed": path_snapshot, "passed": path_snapshot == exp_snapshot_path})

    obs_catch_up, path_catch_up = _refusal(_window(members=(_member(omit="raft_catch_up"),)))
    exp_catch_up = CAPACITY_2947_SECURITY_MATRIX[13][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[13][0], "expected": exp_catch_up, "observed": obs_catch_up, "passed": obs_catch_up == exp_catch_up})
    exp_catch_up_path = CAPACITY_2947_SECURITY_MATRIX[14][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[14][0], "expected": exp_catch_up_path, "observed": path_catch_up, "passed": path_catch_up == exp_catch_up_path})

    obs_cpu, path_cpu = _refusal(_window(members=(_member(cpu=None),)))
    exp_cpu = CAPACITY_2947_SECURITY_MATRIX[15][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[15][0], "expected": exp_cpu, "observed": obs_cpu, "passed": obs_cpu == exp_cpu})
    exp_cpu_path = CAPACITY_2947_SECURITY_MATRIX[16][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[16][0], "expected": exp_cpu_path, "observed": path_cpu, "passed": path_cpu == exp_cpu_path})

    obs_memory, path_memory = _refusal(_window(members=(_member(memory=None),)))
    exp_memory = CAPACITY_2947_SECURITY_MATRIX[17][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[17][0], "expected": exp_memory, "observed": obs_memory, "passed": obs_memory == exp_memory})
    exp_memory_path = CAPACITY_2947_SECURITY_MATRIX[18][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[18][0], "expected": exp_memory_path, "observed": path_memory, "passed": path_memory == exp_memory_path})

    obs_member, path_member = _refusal(_window(members=(_member(),), expected_members=("member-0", "member-1")))
    exp_member = CAPACITY_2947_SECURITY_MATRIX[19][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[19][0], "expected": exp_member, "observed": obs_member, "passed": obs_member == exp_member})
    exp_member_path = CAPACITY_2947_SECURITY_MATRIX[20][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[20][0], "expected": exp_member_path, "observed": path_member, "passed": path_member == exp_member_path})

    # 22-25. R2 -- every physical storage field is independently required.
    _, path_used = _refusal(_window(members=(_member(storage=_storage(used_bytes=None)),)))
    exp_used = CAPACITY_2947_SECURITY_MATRIX[21][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[21][0], "expected": exp_used, "observed": path_used, "passed": path_used == exp_used})
    _, path_headroom = _refusal(_window(members=(_member(storage=_storage(growth_headroom_bytes=None)),)))
    exp_headroom = CAPACITY_2947_SECURITY_MATRIX[22][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[22][0], "expected": exp_headroom, "observed": path_headroom, "passed": path_headroom == exp_headroom})
    _, path_latency = _refusal(_window(members=(_member(storage=_storage(latency_ms=None)),)))
    exp_latency = CAPACITY_2947_SECURITY_MATRIX[23][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[23][0], "expected": exp_latency, "observed": path_latency, "passed": path_latency == exp_latency})
    _, path_saturation = _refusal(_window(members=(_member(storage=_storage(saturation=None)),)))
    exp_saturation = CAPACITY_2947_SECURITY_MATRIX[24][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[24][0], "expected": exp_saturation, "observed": path_saturation, "passed": path_saturation == exp_saturation})

    # 26-27. R3/R4/AC3 -- a member absent from the stated topology holds scale-in.
    missing = decide_scale_in_eligibility(_window(members=(_member(),), expected_members=("member-0", "member-1")), 73)
    obs8 = missing.reason.value if isinstance(missing, Hold) else "eligible"
    exp8 = CAPACITY_2947_SECURITY_MATRIX[25][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[25][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = missing.field_path if isinstance(missing, Hold) else ""
    exp9 = CAPACITY_2947_SECURITY_MATRIX[26][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[26][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R4 -- complete, fresh, same-generation evidence is the only eligible neighbour.
    eligible = decide_scale_in_eligibility(_window(members=(_member("member-0"), _member("member-1"))), 73)
    obs10 = "eligible" if not isinstance(eligible, Hold) else eligible.reason.value
    exp10 = CAPACITY_2947_SECURITY_MATRIX[27][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[27][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11-13. R4 -- each independent unsafe population form has its literal hold vocabulary.
    stale = decide_scale_in_eligibility(_window(members=(_member("member-0"), _member("member-1")), fresh=False), 73)
    obs11 = stale.reason.value if isinstance(stale, Hold) else "eligible"
    exp11 = CAPACITY_2947_SECURITY_MATRIX[28][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[28][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    mixed = decide_scale_in_eligibility(_window(members=(_member("member-0"), _member("member-1", generation=72))), 73)
    obs12 = mixed.reason.value if isinstance(mixed, Hold) else "eligible"
    exp12 = CAPACITY_2947_SECURITY_MATRIX[29][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[29][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    partial = decide_scale_in_eligibility(_window(members=(_member("member-0"), _member("member-1")), complete=False), 73)
    obs13 = partial.reason.value if isinstance(partial, Hold) else "eligible"
    exp13 = CAPACITY_2947_SECURITY_MATRIX[30][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[30][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14-15. R5 -- projection exposes neither member payload nor user content.
    status = redact_status(complete)
    obs14 = tuple(sorted(key for key in status if key in {"member_payload", "members"}))
    exp14 = CAPACITY_2947_SECURITY_MATRIX[31][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[31][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    obs15 = tuple(sorted(key for key in status if key in {"user_content", "content"}))
    exp15 = CAPACITY_2947_SECURITY_MATRIX[32][1]
    checks.append({"name": CAPACITY_2947_SECURITY_MATRIX[32][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "capacity-2947-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
