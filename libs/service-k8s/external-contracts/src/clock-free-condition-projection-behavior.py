from __future__ import annotations

from service_k8s.domain.condition import (
    Condition,
    ConditionFact,
    ConditionStatus,
    project,
)

MINIMUM_CHECKS = 14

CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX = (
    ("an_unchanged_status_keeps_its_transition_instant", "2026-01-01T00:00:00Z"),
    ("everything_but_the_transition_instant_refreshes", ("NewReason", "NewMessage", 7)),
    ("a_flip_takes_the_injected_instant", ("False", "2026-06-01T00:00:00Z")),
    ("a_first_sighting_takes_the_injected_instant", "2026-06-01T00:00:00Z"),
    ("a_condition_the_service_stops_reporting_leaves_the_projection", (1, "Ready")),
    (
        "a_resurrected_condition_is_a_new_condition",
        ("2026-01-01T00:00:00Z", 0, "2026-03-01T00:00:00Z"),
    ),
    (
        "the_projection_preserves_the_order_the_service_emitted",
        (
            ("Ready", "Rotating", "Available"),
            ("Available", "Rotating", "Ready"),
        ),
    ),
    (
        "a_prior_of_the_same_type_but_a_different_status_donates_nothing",
        "2026-06-01T00:00:00Z",
    ),
    (
        "the_serialized_shape_is_exactly_metav1s",
        (
            (
                "type",
                "status",
                "reason",
                "message",
                "lastTransitionTime",
                "observedGeneration",
            ),
            "Ready",
            "True",
            "2026-01-01T00:00:00Z",
            5,
        ),
    ),
    (
        "an_absent_observed_generation_is_omitted_rather_than_serialized_as_null",
        (False, 5),
    ),
    ("an_empty_message_still_appears_in_the_serialized_shape", (True, "")),
    (
        "the_status_tokens_are_the_three_kubernetes_spellings",
        ("True", "False", "Unknown", 3),
    ),
    ("a_boolean_readiness_never_becomes_unknown", ("True", "False")),
    ("an_empty_fact_set_projects_an_empty_sequence", (0, "tuple")),
)

T_PRIOR = "2026-01-01T00:00:00Z"
NOW = "2026-06-01T00:00:00Z"

PRIOR_READY = Condition(
    type_="Ready",
    status="True",
    reason="OldReason",
    message="OldMessage",
    last_transition_time=T_PRIOR,
    observed_generation=1,
)
FACT_READY = ConditionFact(
    type_="Ready",
    status=ConditionStatus.TRUE,
    reason="NewReason",
    message="NewMessage",
)


def verify_clock_free_condition_projection_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an_unchanged_status_keeps_its_transition_instant
    exp1 = CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[0][1]
    res1 = project((PRIOR_READY,), (FACT_READY,), 7, NOW)
    obs1 = res1[0].last_transition_time
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. everything_but_the_transition_instant_refreshes
    exp2 = CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[1][1]
    res2 = project((PRIOR_READY,), (FACT_READY,), 7, NOW)
    obs2 = (res2[0].reason, res2[0].message, res2[0].observed_generation)
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_flip_takes_the_injected_instant
    exp3 = CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[2][1]
    res3 = project(
        (PRIOR_READY,),
        (ConditionFact("Ready", ConditionStatus.FALSE, "NotReadyReason", "Degraded"),),
        2,
        NOW,
    )
    obs3 = (res3[0].status, res3[0].last_transition_time)
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. a_first_sighting_takes_the_injected_instant
    exp4 = CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[3][1]
    res4 = project((), (FACT_READY,), 1, NOW)
    obs4 = res4[0].last_transition_time
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. a_condition_the_service_stops_reporting_leaves_the_projection
    exp5 = CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[4][1]
    res5 = project(
        (PRIOR_READY, Condition("Rotating", "True", "R", "M", T_PRIOR, 1)),
        (FACT_READY,),
        2,
        NOW,
    )
    obs5 = (len(res5), res5[0].type_)
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. a_resurrected_condition_is_a_new_condition
    exp6 = CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[5][1]
    step1 = project((), (FACT_READY,), 1, "2026-01-01T00:00:00Z")
    step2 = project(step1, (), 2, "2026-02-01T00:00:00Z")
    step3 = project(step2, (FACT_READY,), 3, "2026-03-01T00:00:00Z")
    obs6 = (
        step1[0].last_transition_time,
        len(step2),
        step3[0].last_transition_time,
    )
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. the_projection_preserves_the_order_the_service_emitted
    exp7 = CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[6][1]
    three7 = (
        ConditionFact("Ready", ConditionStatus.TRUE, "R1"),
        ConditionFact("Rotating", ConditionStatus.FALSE, "R2"),
        ConditionFact("Available", ConditionStatus.TRUE, "R3"),
    )
    obs7 = (
        tuple(c.type_ for c in project((), three7, 1, NOW)),
        tuple(c.type_ for c in project((), tuple(reversed(three7)), 1, NOW)),
    )
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. a_prior_of_the_same_type_but_a_different_status_donates_nothing
    exp8 = CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[7][1]
    res8 = project(
        (Condition("Ready", "False", "OldReason", "OldMsg", T_PRIOR, 1),),
        (ConditionFact("Ready", ConditionStatus.TRUE, "NewReason", "NewMsg"),),
        2,
        NOW,
    )
    obs8 = res8[0].last_transition_time
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. the_serialized_shape_is_exactly_metav1s
    exp9 = CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[8][1]
    full9 = Condition("Ready", "True", "Reason", "Msg", T_PRIOR, 5).to_json()
    obs9 = (
        tuple(full9.keys()),
        full9.get("type"),
        full9.get("status"),
        full9.get("lastTransitionTime"),
        full9.get("observedGeneration"),
    )
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. an_absent_observed_generation_is_omitted_rather_than_serialized_as_null
    exp10 = CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[9][1]
    partial10 = Condition("Ready", "True", "Reason", "Msg", T_PRIOR, None).to_json()
    obs10 = ("observedGeneration" in partial10, len(partial10))
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. an_empty_message_still_appears_in_the_serialized_shape
    exp11 = CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[10][1]
    empty11 = Condition("Ready", "True", "Reason", "", T_PRIOR, 1).to_json()
    obs11 = ("message" in empty11, empty11.get("message"))
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. the_status_tokens_are_the_three_kubernetes_spellings
    exp12 = CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[11][1]
    obs12 = (
        ConditionStatus.TRUE.token,
        ConditionStatus.FALSE.token,
        ConditionStatus.UNKNOWN.token,
        len(tuple(ConditionStatus)),
    )
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. a_boolean_readiness_never_becomes_unknown
    exp13 = CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[12][1]
    obs13 = (
        ConditionStatus.from_bool(True).token,
        ConditionStatus.from_bool(False).token,
    )
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. an_empty_fact_set_projects_an_empty_sequence
    exp14 = CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[13][1]
    res14 = project((PRIOR_READY,), (), 3, NOW)
    obs14 = (len(res14), type(res14).__name__)
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    return {
        "case_id": "clock-free-condition-projection-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
