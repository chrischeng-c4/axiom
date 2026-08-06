from __future__ import annotations

from transport_h2c.domain.sizing import recommended_connections
from transport_h2c.infrastructure.config import default_config, for_concurrency

MINIMUM_CHECKS = 14

ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX = (
    ("a_concurrency_of_zero_needs_one_connection", 1),
    ("a_concurrency_of_one_needs_one_connection", 1),
    ("a_concurrency_of_two_needs_one_connection", 1),
    ("a_concurrency_of_three_crosses_into_the_logarithm", 2),
    ("the_curve_is_the_natural_logarithm_not_log_two", 6),
    ("the_curve_rounds_up_not_down", 3),
    ("a_thousandfold_burst_stays_in_single_digits", 7),
    ("the_core_count_is_a_hard_ceiling", 4),
    ("a_reported_parallelism_of_zero_still_yields_one_connection", 1),
    ("the_curve_is_monotone_across_the_decades", (2, 3, 5, 7, 10)),
    ("the_default_manager_is_sized_for_a_hundred_and_twenty_eight_requests", 5),
    ("the_default_admission_cap_is_a_hundred_and_twenty_eight", 128),
    ("sizing_for_a_target_moves_both_bounds_together", (7, 1000)),
    ("a_small_core_count_caps_the_configured_ceiling", 2),
)


def verify_adaptive_connection_sizing_behavior() -> dict:
    checks = []

    # 1. a_concurrency_of_zero_needs_one_connection
    exp1 = ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[0][1]
    obs1 = recommended_connections(0, 64)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_concurrency_of_one_needs_one_connection
    exp2 = ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[1][1]
    obs2 = recommended_connections(1, 64)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_concurrency_of_two_needs_one_connection
    exp3 = ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[2][1]
    obs3 = recommended_connections(2, 64)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. a_concurrency_of_three_crosses_into_the_logarithm
    exp4 = ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[3][1]
    obs4 = recommended_connections(3, 64)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. the_curve_is_the_natural_logarithm_not_log_two
    exp5 = ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[4][1]
    obs5 = recommended_connections(256, 64)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. the_curve_rounds_up_not_down
    exp6 = ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[5][1]
    obs6 = recommended_connections(10, 64)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. a_thousandfold_burst_stays_in_single_digits
    exp7 = ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[6][1]
    obs7 = recommended_connections(1000, 64)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. the_core_count_is_a_hard_ceiling
    exp8 = ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[7][1]
    obs8 = recommended_connections(10000, 4)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. a_reported_parallelism_of_zero_still_yields_one_connection
    exp9 = ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[8][1]
    obs9 = recommended_connections(100, 0)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. the_curve_is_monotone_across_the_decades
    exp10 = ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[9][1]
    decades10 = (3, 10, 100, 1000, 10000)
    res10 = []
    for concurrency in decades10:
        res10.append(recommended_connections(concurrency, 64))
    obs10 = tuple(res10)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. the_default_manager_is_sized_for_a_hundred_and_twenty_eight_requests
    exp11 = ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[10][1]
    obs11 = default_config(64).max_connections
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. the_default_admission_cap_is_a_hundred_and_twenty_eight
    exp12 = ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[11][1]
    obs12 = default_config(64).max_in_flight_per_origin
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. sizing_for_a_target_moves_both_bounds_together
    exp13 = ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[12][1]
    obs13 = (
        for_concurrency(1000, 64).max_connections,
        for_concurrency(1000, 64).max_in_flight_per_origin,
    )
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. a_small_core_count_caps_the_configured_ceiling
    exp14 = ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[13][1]
    obs14 = for_concurrency(1000, 2).max_connections
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    return {
        "case_id": "adaptive-connection-sizing-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
