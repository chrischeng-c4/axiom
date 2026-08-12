"""EC security case for #2538 — capacity guidance refuses unsupported claims.

Every expected value is an EC-owned literal transcribed from issue #2538:
R4 (price and regional-billing data are excluded), R5 (no live automatic
storage-class migration), R7 (legacy JSON and absent completed-format/bounded
write-amplification evidence cannot earn a threshold or machine), AC3 (N2
requires qualifying evidence), and AC4 (an explicit E2/pd-balanced rejection
is the sole state that permits a different default).
"""

from __future__ import annotations

from lumen.capacity.admission import decide_capacity_guidance
from lumen.capacity.spec import CapacitySpec, MachineFamily, StorageClass, StorageFormat
from lumen.capacity.verdict import AdmittedGuidance, Rejection

MINIMUM_CHECKS = 13

CAPACITY_2538_SECURITY_MATRIX = (
    ("price_data_is_refused_by_reason", "price_data_not_allowed"),
    ("price_data_refusal_names_the_record_schema", "declared_record_schema_fields"),
    ("price_free_neighbor_is_admitted", "admitted"),
    ("automatic_storage_migration_is_refused_by_reason", "automatic_storage_class_migration_unsupported"),
    ("automatic_storage_migration_refusal_names_the_request", "requested_automatic_storage_class_migration"),
    ("legacy_whole_state_json_is_refused_by_reason", "legacy_whole_state_json"),
    ("legacy_whole_state_json_refusal_names_storage_format", "storage_format"),
    ("legacy_whole_state_json_cannot_contain_recommendations", (None, None)),
    ("missing_completed_format_evidence_is_refused_by_reason", "missing_storage_evidence_prerequisite"),
    ("missing_completed_format_evidence_names_its_attestation", "bounded_steady_state_write_amplification_attested"),
    ("unearned_n2_is_refused_by_reason", "n2_evidence_not_qualified"),
    ("unearned_n2_refusal_names_machine_request", "requested_machine_family"),
    ("qualifying_n2_neighbor_is_admitted", "N2"),
)


def _complete_spec(**overrides: object) -> CapacitySpec:
    """Use literal, explicit inputs so defaults cannot bypass a refusal path."""
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


def verify_capacity_2538_security() -> dict:
    checks = []

    # 1-3. R4 — price fields are forbidden by a named schema refusal, while a
    #        neighbouring otherwise-identical price-free request is admitted.
    price_data = decide_capacity_guidance(
        _complete_spec(declared_record_schema_fields=frozenset({"throughput", "hourly_price"}))
    )
    obs1 = price_data.reason.value if isinstance(price_data, Rejection) else "admitted"
    exp1 = CAPACITY_2538_SECURITY_MATRIX[0][1]
    checks.append({"name": CAPACITY_2538_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    obs2 = price_data.field_path if isinstance(price_data, Rejection) else ""
    exp2 = CAPACITY_2538_SECURITY_MATRIX[1][1]
    checks.append({"name": CAPACITY_2538_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    price_free = decide_capacity_guidance(_complete_spec())
    obs3 = price_free.kind.value if isinstance(price_free, AdmittedGuidance) else price_free.reason.value
    exp3 = CAPACITY_2538_SECURITY_MATRIX[2][1]
    checks.append({"name": CAPACITY_2538_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4-5. R5 — an explicit migration request is never silently accepted.
    migration = decide_capacity_guidance(_complete_spec(requested_automatic_storage_class_migration=True))
    obs4 = migration.reason.value if isinstance(migration, Rejection) else "admitted"
    exp4 = CAPACITY_2538_SECURITY_MATRIX[3][1]
    checks.append({"name": CAPACITY_2538_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    obs5 = migration.field_path if isinstance(migration, Rejection) else ""
    exp5 = CAPACITY_2538_SECURITY_MATRIX[4][1]
    checks.append({"name": CAPACITY_2538_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6-8. R7 — the old whole-state JSON store is a named refusal and can
    #        never overclaim a threshold or machine recommendation.
    legacy = decide_capacity_guidance(_complete_spec(storage_format=StorageFormat.LEGACY_WHOLE_STATE_JSON))
    obs6 = legacy.reason.value if isinstance(legacy, Rejection) else "admitted"
    exp6 = CAPACITY_2538_SECURITY_MATRIX[5][1]
    checks.append({"name": CAPACITY_2538_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    obs7 = legacy.field_path if isinstance(legacy, Rejection) else ""
    exp7 = CAPACITY_2538_SECURITY_MATRIX[6][1]
    checks.append({"name": CAPACITY_2538_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    obs8 = (legacy.threshold, legacy.machine_recommendation) if isinstance(legacy, Rejection) else ("admitted", "admitted")
    exp8 = CAPACITY_2538_SECURITY_MATRIX[7][1]
    checks.append({"name": CAPACITY_2538_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9-10. R7/AC5 — missing bounded-write-amplification evidence is distinct
    #         from a legacy format and identifies the missing prerequisite.
    missing_evidence = decide_capacity_guidance(
        _complete_spec(bounded_steady_state_write_amplification_attested=False)
    )
    obs9 = missing_evidence.reason.value if isinstance(missing_evidence, Rejection) else "admitted"
    exp9 = CAPACITY_2538_SECURITY_MATRIX[8][1]
    checks.append({"name": CAPACITY_2538_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    obs10 = missing_evidence.field_path if isinstance(missing_evidence, Rejection) else ""
    exp10 = CAPACITY_2538_SECURITY_MATRIX[9][1]
    checks.append({"name": CAPACITY_2538_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11-13. AC3 — requesting N2 without earned evidence is rejected by name
    #          and field, while the qualifying neighbouring request is allowed.
    unearned_n2 = decide_capacity_guidance(_complete_spec(requested_machine_family=MachineFamily.N2))
    obs11 = unearned_n2.reason.value if isinstance(unearned_n2, Rejection) else "admitted"
    exp11 = CAPACITY_2538_SECURITY_MATRIX[10][1]
    checks.append({"name": CAPACITY_2538_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    obs12 = unearned_n2.field_path if isinstance(unearned_n2, Rejection) else ""
    exp12 = CAPACITY_2538_SECURITY_MATRIX[11][1]
    checks.append({"name": CAPACITY_2538_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    earned_n2 = decide_capacity_guidance(
        _complete_spec(
            n2_evidence_eligible=True,
            explicit_e2_pd_balanced_rejection=True,
            requested_machine_family=MachineFamily.N2,
        )
    )
    obs13 = earned_n2.machine_family.value if isinstance(earned_n2, AdmittedGuidance) else "rejected"
    exp13 = CAPACITY_2538_SECURITY_MATRIX[12][1]
    checks.append({"name": CAPACITY_2538_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    return {
        "case_id": "capacity-2538-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
