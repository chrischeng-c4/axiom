from __future__ import annotations

from service_k8s.domain.condition import (
    Condition,
    ConditionFact,
    ConditionStatus,
    project,
)

MINIMUM_CHECKS = 12

CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX = (
    (
        "the_same_inputs_project_to_the_same_result_every_time",
        (True, "2026-01-01T00:00:00Z"),
    ),
    (
        "every_instant_comes_from_the_prior_or_the_caller_and_nowhere_else",
        ("2026-01-01T00:00:00Z", "2026-06-01T00:00:00Z", "2026-06-01T00:00:00Z"),
    ),
    (
        "the_injected_instant_is_carried_through_byte_for_byte",
        ("2026-06-01T00:00:00Z", "2026-06-01T00:00:00.123456Z"),
    ),
    ("a_projected_condition_cannot_be_mutated_after_the_fact", "FrozenInstanceError"),
    ("a_condition_fact_cannot_be_mutated_after_the_fact", "FrozenInstanceError"),
    ("the_projection_returns_an_immutable_sequence", ("tuple", "AttributeError")),
    ("a_stale_prior_cannot_smuggle_its_reason_or_message_forward", ("Issued", "")),
    ("a_prior_the_service_no_longer_reports_cannot_survive_on_its_own", ("Rotating",)),
    (
        "an_unknown_status_never_inherits_a_true_priors_instant",
        ("Unknown", "2026-06-01T00:00:00Z"),
    ),
    (
        "the_observed_generation_always_comes_from_the_caller",
        (2, "2026-01-01T00:00:00Z"),
    ),
    (
        "a_forged_prior_instant_is_honored_only_for_an_identical_status",
        ("2026-06-01T00:00:00Z", "1999-01-01T00:00:00Z"),
    ),
    (
        "the_serialized_condition_exposes_no_field_the_service_did_not_set",
        (
            "lastTransitionTime",
            "message",
            "observedGeneration",
            "reason",
            "status",
            "type",
        ),
    ),
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
FORGED_PRIOR = (
    Condition("Ready", "False", "Forged", "Forged", "1999-01-01T00:00:00Z", 1),
)


def mutation_error(obj: object, field: str, value: object) -> str:
    try:
        setattr(obj, field, value)
    except Exception as exc:
        return type(exc).__name__
    return "mutated"


def verify_clock_free_condition_projection_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the_same_inputs_project_to_the_same_result_every_time
    exp1 = CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[0][1]
    once1 = project((PRIOR_READY,), (FACT_READY,), 7, NOW)
    twice1 = project((PRIOR_READY,), (FACT_READY,), 7, NOW)
    obs1 = (once1 == twice1, once1[0].last_transition_time)
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. every_instant_comes_from_the_prior_or_the_caller_and_nowhere_else
    exp2 = CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[1][1]
    mixed2 = project(
        (PRIOR_READY,),
        (
            ConditionFact("Ready", ConditionStatus.TRUE, "R1"),
            ConditionFact("Rotating", ConditionStatus.FALSE, "R2"),
            ConditionFact("Available", ConditionStatus.TRUE, "R3"),
        ),
        4,
        NOW,
    )
    obs2 = tuple(c.last_transition_time for c in mixed2)
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. the_injected_instant_is_carried_through_byte_for_byte
    exp3 = CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[2][1]
    obs3 = (
        project((), (FACT_READY,), 1, "2026-06-01T00:00:00Z")[0].last_transition_time,
        project((), (FACT_READY,), 1, "2026-06-01T00:00:00.123456Z")[
            0
        ].last_transition_time,
    )
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. a_projected_condition_cannot_be_mutated_after_the_fact
    exp4 = CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[3][1]
    obs4 = mutation_error(
        project((), (FACT_READY,), 1, NOW)[0], "last_transition_time", "hacked"
    )
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. a_condition_fact_cannot_be_mutated_after_the_fact
    exp5 = CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[4][1]
    obs5 = mutation_error(FACT_READY, "reason", "hacked")
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. the_projection_returns_an_immutable_sequence
    exp6 = CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[5][1]
    obs6 = (
        type(project((), (FACT_READY,), 1, NOW)).__name__,
        mutation_error(project((), (FACT_READY,), 1, NOW), "0", "hacked"),
    )
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. a_stale_prior_cannot_smuggle_its_reason_or_message_forward
    exp7 = CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[6][1]
    res7 = project(
        (Condition("Ready", "True", "Attacker", "pwned", T_PRIOR, 1),),
        (ConditionFact("Ready", ConditionStatus.TRUE, "Issued", ""),),
        2,
        NOW,
    )
    obs7 = (res7[0].reason, res7[0].message)
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. a_prior_the_service_no_longer_reports_cannot_survive_on_its_own
    exp8 = CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[7][1]
    res8 = project(
        (
            Condition("Ready", "True", "R", "M", T_PRIOR, 1),
            Condition("Rotating", "True", "R", "M", T_PRIOR, 1),
        ),
        (ConditionFact("Rotating", ConditionStatus.TRUE, "R", "M"),),
        2,
        NOW,
    )
    obs8 = tuple(c.type_ for c in res8)
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. an_unknown_status_never_inherits_a_true_priors_instant
    exp9 = CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[8][1]
    res9 = project(
        (PRIOR_READY,),
        (ConditionFact("Ready", ConditionStatus.UNKNOWN, "Lost", "no signal"),),
        2,
        NOW,
    )
    obs9 = (res9[0].status, res9[0].last_transition_time)
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. the_observed_generation_always_comes_from_the_caller
    exp10 = CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[9][1]
    res10 = project(
        (Condition("Ready", "True", "R", "M", T_PRIOR, 99),),
        (ConditionFact("Ready", ConditionStatus.TRUE, "R", "M"),),
        2,
        NOW,
    )
    obs10 = (res10[0].observed_generation, res10[0].last_transition_time)
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. a_forged_prior_instant_is_honored_only_for_an_identical_status
    exp11 = CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[10][1]
    obs11 = (
        project(
            FORGED_PRIOR,
            (ConditionFact("Ready", ConditionStatus.TRUE, "R"),),
            2,
            NOW,
        )[0].last_transition_time,
        project(
            FORGED_PRIOR,
            (ConditionFact("Ready", ConditionStatus.FALSE, "R"),),
            2,
            NOW,
        )[0].last_transition_time,
    )
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. the_serialized_condition_exposes_no_field_the_service_did_not_set
    exp12 = CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[11][1]
    obs12 = tuple(
        sorted(Condition("Ready", "True", "R", "M", T_PRIOR, 5).to_json())
    )
    checks.append(
        {
            "name": CLOCK_FREE_CONDITION_PROJECTION_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    return {
        "case_id": "clock-free-condition-projection-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
