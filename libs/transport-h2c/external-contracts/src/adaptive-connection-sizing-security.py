from __future__ import annotations

from transport_h2c.domain.sizing import recommended_connections
from transport_h2c.infrastructure.config import (
    admission_permits,
    default_config,
    for_concurrency,
)

MINIMUM_CHECKS = 12

ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX = (
    ("the_count_never_exceeds_the_core_cap_at_any_concurrency", (2, 3, 3, 3, 3, 3)),
    ("the_count_is_never_below_one_at_any_core_count", (1, 1, 2, 7)),
    ("a_negative_core_count_still_yields_one_connection", 1),
    ("an_enormous_concurrency_cannot_exhaust_the_core_cap", 8),
    ("a_negative_target_concurrency_takes_the_one_connection_shortcut", 1),
    ("a_zero_target_yields_a_usable_manager_on_every_bound", (1, 1, 1)),
    ("the_connection_ceiling_never_falls_below_the_floor", (1, 1, 1, 2)),
    ("a_single_core_host_still_gets_a_bracket_of_one", (1, 1)),
    ("admission_permits_track_the_target_not_the_default", (4, 512)),
    ("sizing_leaves_the_growth_threshold_alone", (32, 32)),
    ("sizing_leaves_the_keepalive_target_alone", 16),
    ("sizing_leaves_the_admission_deadline_alone", 5.0),
)


def verify_adaptive_connection_sizing_security() -> dict:
    checks = []

    # 1. the_count_never_exceeds_the_core_cap_at_any_concurrency
    exp1 = ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[0][1]
    concurrencies1 = (3, 10, 100, 1000, 10000, 1000000)
    res1 = []
    for concurrency in concurrencies1:
        res1.append(recommended_connections(concurrency, 3))
    obs1 = tuple(res1)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. the_count_is_never_below_one_at_any_core_count
    exp2 = ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[1][1]
    cores2 = (0, 1, 2, 64)
    res2 = []
    for parallelism in cores2:
        res2.append(recommended_connections(1000, parallelism))
    obs2 = tuple(res2)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_negative_core_count_still_yields_one_connection
    exp3 = ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[2][1]
    obs3 = recommended_connections(1000, -4)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. an_enormous_concurrency_cannot_exhaust_the_core_cap
    exp4 = ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[3][1]
    obs4 = recommended_connections(1000000000, 8)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. a_negative_target_concurrency_takes_the_one_connection_shortcut
    exp5 = ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[4][1]
    obs5 = recommended_connections(-5, 64)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. a_zero_target_yields_a_usable_manager_on_every_bound
    exp6 = ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[5][1]
    cfg6 = for_concurrency(0, 64)
    obs6 = (cfg6.min_connections, cfg6.max_connections, cfg6.max_in_flight_per_origin)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. the_connection_ceiling_never_falls_below_the_floor
    exp7 = ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[6][1]
    targets7 = (0, 1, 2, 3)
    res7 = []
    for concurrency in targets7:
        res7.append(for_concurrency(concurrency, 64).max_connections)
    obs7 = tuple(res7)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. a_single_core_host_still_gets_a_bracket_of_one
    exp8 = ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[7][1]
    obs8 = (default_config(1).min_connections, default_config(1).max_connections)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. admission_permits_track_the_target_not_the_default
    exp9 = ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[8][1]
    obs9 = (
        admission_permits(for_concurrency(4, 64)),
        admission_permits(for_concurrency(512, 64)),
    )
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. sizing_leaves_the_growth_threshold_alone
    exp10 = ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[9][1]
    obs10 = (for_concurrency(4, 64).grow_threshold, default_config(64).grow_threshold)
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. sizing_leaves_the_keepalive_target_alone
    exp11 = ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[10][1]
    obs11 = for_concurrency(4, 64).max_keepalive_connections
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. sizing_leaves_the_admission_deadline_alone
    exp12 = ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[11][1]
    obs12 = for_concurrency(4, 64).pool_timeout_seconds
    checks.append(
        {
            "name": ADAPTIVE_CONNECTION_SIZING_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    return {
        "case_id": "adaptive-connection-sizing-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
