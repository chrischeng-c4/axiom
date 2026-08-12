"""EC behavior case for #2538 — capacity guidance admissions and defaults.

Every expected value is an EC-owned literal transcribed from issue #2538:
R5 (pd-ssd is initial-placement/future, never an automatic migration), R6
(E2/pd-balanced is the default absent an explicit qualifying rejection), R7
(only a completed, attested storage format earns guidance), AC3 (N2 requires
qualifying evidence and pd-ssd remains initial-only/future), and AC4 (the
E2/pd-balanced default remains selected until explicitly rejected).
"""

from __future__ import annotations

from lumen.capacity.admission import decide_capacity_guidance
from lumen.capacity.spec import CapacitySpec, MachineFamily, StorageClass, StorageFormat
from lumen.capacity.verdict import AdmittedGuidance

MINIMUM_CHECKS = 6

CAPACITY_2538_BEHAVIOR_MATRIX = (
    ("eligible_guidance_is_admitted", "admitted"),
    ("eligible_guidance_selects_e2", "E2"),
    ("eligible_guidance_selects_pd_balanced", "pd-balanced"),
    ("eligible_guidance_labels_pd_ssd_initial_only_future", "INITIAL_ONLY_FUTURE"),
    ("qualifying_e2_rejection_permits_n2", "N2"),
    ("qualifying_e2_rejection_keeps_pd_balanced", "pd-balanced"),
)


def _eligible_spec(**overrides: object) -> CapacitySpec:
    """Name every admission input so a default cannot hide a missing guard."""
    values: dict[str, object] = {
        "declared_record_schema_fields": frozenset(
            {
                "throughput",
                "p50_latency",
                "p95_latency",
                "p99_latency",
                "cpu",
                "memory",
                "disk_latency",
                "disk_throughput",
                "recovery_time",
                "persistence_bytes",
                "fsyncs",
                "peak_memory",
            }
        ),
        "storage_format": StorageFormat.SEGMENT_CHECKPOINT,
        "storage_format_attested": True,
        "bounded_steady_state_write_amplification_attested": True,
        "n2_evidence_eligible": False,
        "explicit_e2_pd_balanced_rejection": False,
        "requested_automatic_storage_class_migration": False,
        "requested_machine_family": MachineFamily.E2,
        "requested_storage_class": StorageClass.PD_BALANCED,
    }
    values.update(overrides)
    return CapacitySpec(**values)


def verify_capacity_2538_behavior() -> dict:
    checks = []

    # 1-4. R5/R6/AC3/AC4 — an explicitly complete eligible request produces
    #        the ordinary v1 guidance: E2, pd-balanced, and pd-ssd only as a
    #        future initial-placement disposition.
    eligible = decide_capacity_guidance(_eligible_spec())
    obs1 = eligible.kind.value if isinstance(eligible, AdmittedGuidance) else eligible.reason.value
    exp1 = CAPACITY_2538_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CAPACITY_2538_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    obs2 = eligible.machine_family.value if isinstance(eligible, AdmittedGuidance) else "rejected"
    exp2 = CAPACITY_2538_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CAPACITY_2538_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    obs3 = eligible.storage_class.value if isinstance(eligible, AdmittedGuidance) else "rejected"
    exp3 = CAPACITY_2538_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CAPACITY_2538_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    obs4 = eligible.pd_ssd_disposition.value if isinstance(eligible, AdmittedGuidance) else "rejected"
    exp4 = CAPACITY_2538_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CAPACITY_2538_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5-6. R6/AC3/AC4 — a different default is permitted only with the named
    #        E2/pd-balanced rejection and qualifying N2 evidence; it does not
    #        turn the storage default into pd-ssd.
    rejected_e2 = decide_capacity_guidance(
        _eligible_spec(
            n2_evidence_eligible=True,
            explicit_e2_pd_balanced_rejection=True,
            requested_machine_family=MachineFamily.N2,
        )
    )
    obs5 = rejected_e2.machine_family.value if isinstance(rejected_e2, AdmittedGuidance) else "rejected"
    exp5 = CAPACITY_2538_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CAPACITY_2538_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    obs6 = rejected_e2.storage_class.value if isinstance(rejected_e2, AdmittedGuidance) else "rejected"
    exp6 = CAPACITY_2538_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CAPACITY_2538_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    return {
        "case_id": "capacity-2538-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
