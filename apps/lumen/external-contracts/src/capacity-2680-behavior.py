"""EC behavior case for #2680 -- retired serving-autoscaling admission and HPA handoff.

Every expected value is an EC-owned literal transcribed from #2680's decidable
requirements: R1 removes the retired autoscaling vocabulary, R2/AC3 retains
deletion only for the operator-rendered stale HPA, R4 adds no replacement
capacity thresholds, and AC1 admits a supported serving surface which omits
the retired field.  The imports intentionally fail closed until the frozen
pure capacity design lands.
"""

from __future__ import annotations

from lumen.capacity import admission, hpa_handoff, spec, verdict

MINIMUM_CHECKS = 8

CAPACITY_2680_BEHAVIOR_MATRIX = (
    ("supported_serving_surface_without_autoscaling_is_admitted", "admitted"),
    ("admitted_allowed_vocabulary_excludes_retired_autoscaling_names", ()),
    ("matching_unwanted_hpa_is_deleted", "DELETE"),
    ("matching_unwanted_hpa_with_multiple_labels_is_deleted", "DELETE"),
    ("missing_hpa_is_not_deleted", "KEEP"),
    ("foreign_labelled_hpa_is_not_deleted", "KEEP"),
    ("foreign_labelled_hpa_with_expected_labels_is_not_deleted", "KEEP"),
    ("wanted_matching_hpa_is_not_deleted", "KEEP"),
)


def verify_capacity_2680_behavior() -> dict:
    checks = []

    # 1. AC1 -- explicitly name a supported, non-retired surface.  An empty
    #    input would let a model admit nothing while appearing to satisfy it.
    admitted = admission.decide_serving_surface(
        frozenset({"replicas", "resources"})
    )
    obs1 = admitted.outcome.value
    exp1 = CAPACITY_2680_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CAPACITY_2680_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- admission exposes the vocabulary it admits; the retired names
    #    must be absent rather than merely ignored when supplied by a client.
    retired_names = frozenset(
        {"autoscaling", "minReplicas", "maxReplicas", "targetCpuUtilization"}
    )
    obs2 = tuple(sorted(frozenset(admitted.allowed_fields) & retired_names))
    exp2 = CAPACITY_2680_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CAPACITY_2680_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    labels = {"app.kubernetes.io/managed-by": "lumen", "lumen.axiom.dev/name": "search"}

    # 3. R2/AC3 -- a retired operator-rendered HPA with the exact expected
    #    identity and labels is the one stale object this handoff may delete.
    matching = hpa_handoff.decide_stale_hpa_cleanup(
        "search-serving", labels, "search-serving", labels, False
    )
    obs3 = matching.action.value
    exp3 = CAPACITY_2680_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CAPACITY_2680_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R2/AC3 -- exact label *sets*, not a single sentinel label, protect
    #    the object identity used for a second stale-HPA deletion path.
    multi_labels = {**labels, "lumen.axiom.dev/role": "serving"}
    matching_multi = hpa_handoff.decide_stale_hpa_cleanup(
        "search-serving", multi_labels, "search-serving", multi_labels, False
    )
    obs4 = matching_multi.action.value
    exp4 = CAPACITY_2680_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CAPACITY_2680_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R2 -- absence does not turn a cleanup decision into a claimed delete.
    missing = hpa_handoff.decide_stale_hpa_cleanup(
        "search-serving", labels, None, None, False
    )
    obs5 = missing.action.value
    exp5 = CAPACITY_2680_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CAPACITY_2680_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R2 -- matching a name is insufficient: a foreign label set is never
    #    an operator-rendered HPA that this reconciliation owns.
    foreign = hpa_handoff.decide_stale_hpa_cleanup(
        "search-serving", labels, "search-serving", {"app": "foreign"}, False
    )
    obs6 = foreign.action.value
    exp6 = CAPACITY_2680_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CAPACITY_2680_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R2 -- containment is insufficient: even an HPA with every expected
    #    label is foreign when its live label set contains an extra label.
    expected_plus_extra = {**labels, "app.kubernetes.io/part-of": "foreign"}
    expected_plus_extra_hpa = hpa_handoff.decide_stale_hpa_cleanup(
        "search-serving", labels, "search-serving", expected_plus_extra, False
    )
    obs7 = expected_plus_extra_hpa.action.value
    exp7 = CAPACITY_2680_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CAPACITY_2680_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R2 -- the deletion handoff exists only once no HPA is wanted; a live
    #    matching object remains when the desired state still wants one.
    wanted = hpa_handoff.decide_stale_hpa_cleanup(
        "search-serving", labels, "search-serving", labels, True
    )
    obs8 = wanted.action.value
    exp8 = CAPACITY_2680_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CAPACITY_2680_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    return {
        "case_id": "capacity-2680-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
