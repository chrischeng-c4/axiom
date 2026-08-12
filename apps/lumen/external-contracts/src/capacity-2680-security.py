"""EC security case for #2680 -- serving capacity fails closed.

Expected values are EC-owned literals from #2680's decidable core: R1 and AC2
reject every retired autoscaling name as an unknown field and identify the
field, R4 rejects CPU, memory, and disk replacement thresholds by name, AC1
keeps a neighbouring supported input admitted, and R2 retains any HPA not
proven to be the exact unwanted operator-rendered object.
"""

from __future__ import annotations

from lumen.capacity import admission, hpa_handoff, spec, verdict

MINIMUM_CHECKS = 16

CAPACITY_2680_SECURITY_MATRIX = (
    ("autoscaling_is_rejected_as_an_unknown_retired_field", "unknown_retired_field"),
    ("autoscaling_refusal_names_the_autoscaling_path", "autoscaling"),
    ("min_replicas_is_rejected_as_an_unknown_retired_field", "unknown_retired_field"),
    ("min_replicas_refusal_names_its_field_path", "minReplicas"),
    ("max_replicas_is_rejected_as_an_unknown_retired_field", "unknown_retired_field"),
    ("max_replicas_refusal_names_its_field_path", "maxReplicas"),
    ("target_cpu_utilization_is_rejected_as_an_unknown_retired_field", "unknown_retired_field"),
    ("target_cpu_utilization_refusal_names_its_field_path", "targetCpuUtilization"),
    ("cpu_threshold_is_rejected_as_a_prohibited_replacement", "prohibited_replacement_threshold"),
    ("cpu_threshold_refusal_names_its_field_path", "cpuThreshold"),
    ("memory_threshold_is_rejected_as_a_prohibited_replacement", "prohibited_replacement_threshold"),
    ("memory_threshold_refusal_names_its_field_path", "memoryThreshold"),
    ("disk_threshold_is_rejected_as_a_prohibited_replacement", "prohibited_replacement_threshold"),
    ("disk_threshold_refusal_names_its_field_path", "diskThreshold"),
    ("neighbouring_supported_surface_remains_admitted", "admitted"),
    ("wrong_named_hpa_is_never_deleted", "KEEP"),
)


def verify_capacity_2680_security() -> dict:
    checks = []

    # 1-2. R1/AC2 -- a literal retired field must receive the defined strict
    #        unknown-field vocabulary and name the exact offending path.
    autoscaling = admission.decide_serving_surface(
        frozenset({"replicas", "resources", "autoscaling"})
    )
    obs1 = autoscaling.reason.value
    exp1 = CAPACITY_2680_SECURITY_MATRIX[0][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = autoscaling.field_path
    exp2 = CAPACITY_2680_SECURITY_MATRIX[1][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3-4. R1 -- each retired bound is explicit input, not a default hidden
    #        behind an otherwise valid serving request.
    minimum = admission.decide_serving_surface(
        frozenset({"replicas", "resources", "minReplicas"})
    )
    obs3 = minimum.reason.value
    exp3 = CAPACITY_2680_SECURITY_MATRIX[2][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = minimum.field_path
    exp4 = CAPACITY_2680_SECURITY_MATRIX[3][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5-6. R1 -- the maximum bound is independently prohibited and named.
    maximum = admission.decide_serving_surface(
        frozenset({"replicas", "resources", "maxReplicas"})
    )
    obs5 = maximum.reason.value
    exp5 = CAPACITY_2680_SECURITY_MATRIX[4][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = maximum.field_path
    exp6 = CAPACITY_2680_SECURITY_MATRIX[5][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7-8. R1 -- the retired CPU target is also an explicit strict-schema
    #        input and its field path must survive the refusal.
    target_cpu = admission.decide_serving_surface(
        frozenset({"replicas", "resources", "targetCpuUtilization"})
    )
    obs7 = target_cpu.reason.value
    exp7 = CAPACITY_2680_SECURITY_MATRIX[6][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = target_cpu.field_path
    exp8 = CAPACITY_2680_SECURITY_MATRIX[7][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9-10. R4 -- a replacement CPU policy is refused and the response points
    #        to the actual new public field, rather than a generic error.
    cpu = admission.decide_serving_surface(
        frozenset({"replicas", "resources", "cpuThreshold"})
    )
    obs9 = cpu.reason.value
    exp9 = CAPACITY_2680_SECURITY_MATRIX[8][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = cpu.field_path
    exp10 = CAPACITY_2680_SECURITY_MATRIX[9][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11-12. R4 -- memory thresholds are not a substitute capacity API and
    #        their refusal must identify the field, not a generic category.
    memory = admission.decide_serving_surface(
        frozenset({"replicas", "resources", "memoryThreshold"})
    )
    obs11 = memory.reason.value
    exp11 = CAPACITY_2680_SECURITY_MATRIX[10][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = memory.field_path
    exp12 = CAPACITY_2680_SECURITY_MATRIX[11][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13-14. R4 -- neither are disk thresholds, and the field identity is
    #        preserved for this independently forbidden replacement surface.
    disk = admission.decide_serving_surface(
        frozenset({"replicas", "resources", "diskThreshold"})
    )
    obs13 = disk.reason.value
    exp13 = CAPACITY_2680_SECURITY_MATRIX[12][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = disk.field_path
    exp14 = CAPACITY_2680_SECURITY_MATRIX[13][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. AC1 -- rejection is narrow: removing the forbidden member yields an
    #     explicit admitted result instead of a blanket serving-surface ban.
    neighbour = admission.decide_serving_surface(frozenset({"replicas", "resources"}))
    obs15 = neighbour.outcome.value
    exp15 = CAPACITY_2680_SECURITY_MATRIX[14][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16. R2 -- expected labels alone do not authorize a delete against an HPA
    #     whose name belongs to another object.
    labels = {"app.kubernetes.io/managed-by": "lumen", "lumen.axiom.dev/name": "search"}
    wrong_name = hpa_handoff.decide_stale_hpa_cleanup(
        "search-serving", labels, "other-serving", labels, False
    )
    obs16 = wrong_name.action.value
    exp16 = CAPACITY_2680_SECURITY_MATRIX[15][1]
    checks.append({"name": CAPACITY_2680_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    return {
        "case_id": "capacity-2680-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
