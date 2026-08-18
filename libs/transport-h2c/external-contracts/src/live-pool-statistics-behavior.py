from __future__ import annotations

from transport_h2c.infrastructure.connection import ConnectionState, reserve
from transport_h2c.infrastructure.stats import ManagerStats, snapshot

MINIMUM_CHECKS = 14

LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX = (
    ("an_empty_pool_reports_all_zeros", (0, 0, 0, 0, 0)),
    ("a_single_idle_connection_is_counted_once", (1, 1, 0, 0, 0)),
    ("three_connections_are_counted_and_their_streams_summed", (3, 3, 9, 0, 0)),
    ("the_health_split_shows_a_mostly_dead_pool", (4, 1, 0, 0, 0)),
    ("request_totals_accumulate_across_the_pool", (2, 2, 0, 30, 0)),
    ("error_totals_accumulate_across_the_pool", (2, 2, 0, 0, 7)),
    ("retired_requests_are_carried_into_the_total", (1, 1, 0, 105, 0)),
    ("retired_errors_are_carried_into_the_total", (1, 1, 0, 0, 42)),
    ("a_pool_with_only_retired_counters_still_reports_them", (0, 0, 0, 77, 11)),
    ("a_full_snapshot_reports_every_field_at_once", (3, 2, 12, 250, 19)),
    ("the_snapshot_is_taken_at_call_time_and_not_cached", (0, 1)),
    ("a_default_stats_value_is_all_zeros", (0, 0, 0, 0, 0)),
    ("an_unhealthy_connection_still_counts_toward_the_connection_total", (2, 0, 8, 0, 0)),
    ("a_dead_connections_counters_still_count_until_it_is_retired", (1, 0, 0, 9, 2)),
)


def _conn(
    ident: int,
    healthy: bool = True,
    in_flight: int = 0,
    total: int = 0,
    errors: int = 0,
    last_used_ms: int = 0,
) -> ConnectionState:
    return ConnectionState(
        id=ident,
        healthy=healthy,
        in_flight=in_flight,
        total=total,
        errors=errors,
        last_used_ms=last_used_ms,
    )


def _st(stats: ManagerStats) -> tuple:
    return (
        stats.connections,
        stats.healthy,
        stats.in_flight,
        stats.total_requests,
        stats.total_errors,
    )


def verify_live_pool_statistics_behavior() -> dict:
    checks = []

    # 1. an_empty_pool_reports_all_zeros
    exp1 = LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[0][1]
    obs1 = _st(snapshot((), 0, 0))
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_single_idle_connection_is_counted_once
    exp2 = LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[1][1]
    obs2 = _st(snapshot((_conn(1),), 0, 0))
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. three_connections_are_counted_and_their_streams_summed
    exp3 = LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[2][1]
    conns3 = (_conn(1, in_flight=2), _conn(2, in_flight=3), _conn(3, in_flight=4))
    obs3 = _st(snapshot(conns3, 0, 0))
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. the_health_split_shows_a_mostly_dead_pool
    exp4 = LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[3][1]
    conns4 = (
        _conn(1, healthy=False),
        _conn(2, healthy=False),
        _conn(3, healthy=True),
        _conn(4, healthy=False),
    )
    obs4 = _st(snapshot(conns4, 0, 0))
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. request_totals_accumulate_across_the_pool
    exp5 = LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[4][1]
    conns5 = (_conn(1, total=10), _conn(2, total=20))
    obs5 = _st(snapshot(conns5, 0, 0))
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. error_totals_accumulate_across_the_pool
    exp6 = LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[5][1]
    conns6 = (_conn(1, errors=3), _conn(2, errors=4))
    obs6 = _st(snapshot(conns6, 0, 0))
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. retired_requests_are_carried_into_the_total
    exp7 = LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[6][1]
    obs7 = _st(snapshot((_conn(1, total=5),), 100, 0))
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. retired_errors_are_carried_into_the_total
    exp8 = LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[7][1]
    obs8 = _st(snapshot((_conn(1, errors=2),), 0, 40))
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. a_pool_with_only_retired_counters_still_reports_them
    exp9 = LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[8][1]
    obs9 = _st(snapshot((), 77, 11))
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. a_full_snapshot_reports_every_field_at_once
    exp10 = LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[9][1]
    conns10 = (
        _conn(1, healthy=True, in_flight=5, total=100, errors=4),
        _conn(2, healthy=False, in_flight=3, total=50, errors=9),
        _conn(3, healthy=True, in_flight=4, total=60, errors=1),
    )
    obs10 = _st(snapshot(conns10, 40, 5))
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. the_snapshot_is_taken_at_call_time_and_not_cached
    exp11 = LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[10][1]
    live11 = _conn(1, in_flight=0)
    before11 = snapshot((live11,), 0, 0)
    reserve(live11)
    after11 = snapshot((live11,), 0, 0)
    obs11 = (before11.in_flight, after11.in_flight)
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. a_default_stats_value_is_all_zeros
    exp12 = LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[11][1]
    obs12 = _st(ManagerStats())
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. an_unhealthy_connection_still_counts_toward_the_connection_total
    exp13 = LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[12][1]
    conns13 = (
        _conn(1, healthy=False, in_flight=3),
        _conn(2, healthy=False, in_flight=5),
    )
    obs13 = _st(snapshot(conns13, 0, 0))
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. a_dead_connections_counters_still_count_until_it_is_retired
    exp14 = LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[13][1]
    obs14 = _st(snapshot((_conn(1, healthy=False, total=9, errors=2),), 0, 0))
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    return {
        "case_id": "live-pool-statistics-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
