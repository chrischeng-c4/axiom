from __future__ import annotations

from transport_h2c.infrastructure.connection import ConnectionState
from transport_h2c.infrastructure.stats import snapshot

MINIMUM_CHECKS = 12

LIVE_POOL_STATISTICS_SECURITY_MATRIX = (
    ("an_all_success_run_has_a_proven_zero_error_point", 0),
    ("the_error_counter_does_not_drift_across_repeated_snapshots", (0, 0, 0)),
    ("a_degraded_pool_is_distinguishable_from_a_healthy_one", ((3, 3), (3, 1))),
    ("evicting_a_connection_does_not_reset_the_operators_error_rate", (15, 15)),
    ("evicting_a_connection_does_not_reset_the_request_total", (300, 300)),
    ("retired_counters_cannot_be_dropped_by_an_empty_pool", (1000, 250)),
    ("the_health_count_tracks_the_live_subset_at_every_mix", (0, 1, 2, 3)),
    ("in_flight_is_a_stream_count_not_a_connection_count", (0, 6, 300)),
    ("a_saturated_connection_reports_its_streams_not_itself", 128),
    ("an_untouched_pool_reports_no_phantom_work", (0, 0)),
    ("every_field_is_independent_and_none_aliases_another", (5, 3, 17, 91, 23)),
    ("a_fully_dead_pool_reports_zero_healthy_but_its_connections_remain", (4, 0)),
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


def verify_live_pool_statistics_security() -> dict:
    checks = []

    # 1. an_all_success_run_has_a_proven_zero_error_point
    exp1 = LIVE_POOL_STATISTICS_SECURITY_MATRIX[0][1]
    conns1 = (_conn(1, total=20), _conn(2, total=22), _conn(3, total=22))
    obs1 = snapshot(conns1, 0, 0).total_errors
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. the_error_counter_does_not_drift_across_repeated_snapshots
    exp2 = LIVE_POOL_STATISTICS_SECURITY_MATRIX[1][1]
    conns2 = (_conn(1, total=64), _conn(2, total=64))
    res2 = []
    for _ in range(3):
        res2.append(snapshot(conns2, 0, 0).total_errors)
    obs2 = tuple(res2)
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_degraded_pool_is_distinguishable_from_a_healthy_one
    exp3 = LIVE_POOL_STATISTICS_SECURITY_MATRIX[2][1]
    healthy3 = (_conn(1), _conn(2), _conn(3))
    degraded3 = (_conn(1), _conn(2, healthy=False), _conn(3, healthy=False))
    a3 = snapshot(healthy3, 0, 0)
    b3 = snapshot(degraded3, 0, 0)
    obs3 = ((a3.connections, a3.healthy), (b3.connections, b3.healthy))
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. evicting_a_connection_does_not_reset_the_operators_error_rate
    exp4 = LIVE_POOL_STATISTICS_SECURITY_MATRIX[3][1]
    before4 = (_conn(1, errors=5), _conn(2, errors=5), _conn(3, errors=5))
    after4 = (_conn(1, errors=5), _conn(2, errors=5))
    obs4 = (
        snapshot(before4, 0, 0).total_errors,
        snapshot(after4, 0, 5).total_errors,
    )
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. evicting_a_connection_does_not_reset_the_request_total
    exp5 = LIVE_POOL_STATISTICS_SECURITY_MATRIX[4][1]
    before5 = (_conn(1, total=100), _conn(2, total=100), _conn(3, total=100))
    after5 = (_conn(1, total=100), _conn(2, total=100))
    obs5 = (
        snapshot(before5, 0, 0).total_requests,
        snapshot(after5, 100, 0).total_requests,
    )
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. retired_counters_cannot_be_dropped_by_an_empty_pool
    exp6 = LIVE_POOL_STATISTICS_SECURITY_MATRIX[5][1]
    empty6 = snapshot((), 1000, 250)
    obs6 = (empty6.total_requests, empty6.total_errors)
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. the_health_count_tracks_the_live_subset_at_every_mix
    exp7 = LIVE_POOL_STATISTICS_SECURITY_MATRIX[6][1]
    mixes7 = (
        (_conn(1, healthy=False), _conn(2, healthy=False), _conn(3, healthy=False)),
        (_conn(1, healthy=True), _conn(2, healthy=False), _conn(3, healthy=False)),
        (_conn(1, healthy=True), _conn(2, healthy=True), _conn(3, healthy=False)),
        (_conn(1, healthy=True), _conn(2, healthy=True), _conn(3, healthy=True)),
    )
    res7 = []
    for mix in mixes7:
        res7.append(snapshot(mix, 0, 0).healthy)
    obs7 = tuple(res7)
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. in_flight_is_a_stream_count_not_a_connection_count
    exp8 = LIVE_POOL_STATISTICS_SECURITY_MATRIX[7][1]
    pools8 = (
        (_conn(1), _conn(2), _conn(3)),
        (_conn(1, in_flight=1), _conn(2, in_flight=2), _conn(3, in_flight=3)),
        (_conn(1, in_flight=100), _conn(2, in_flight=100), _conn(3, in_flight=100)),
    )
    res8 = []
    for pool in pools8:
        res8.append(snapshot(pool, 0, 0).in_flight)
    obs8 = tuple(res8)
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. a_saturated_connection_reports_its_streams_not_itself
    exp9 = LIVE_POOL_STATISTICS_SECURITY_MATRIX[8][1]
    obs9 = snapshot((_conn(1, in_flight=128),), 0, 0).in_flight
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. an_untouched_pool_reports_no_phantom_work
    exp10 = LIVE_POOL_STATISTICS_SECURITY_MATRIX[9][1]
    quiet10 = snapshot((_conn(1), _conn(2)), 0, 0)
    obs10 = (quiet10.total_requests, quiet10.total_errors)
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. every_field_is_independent_and_none_aliases_another
    exp11 = LIVE_POOL_STATISTICS_SECURITY_MATRIX[10][1]
    conns11 = (
        _conn(1, healthy=True, in_flight=2, total=10, errors=1),
        _conn(2, healthy=True, in_flight=4, total=20, errors=2),
        _conn(3, healthy=True, in_flight=5, total=30, errors=3),
        _conn(4, healthy=False, in_flight=3, total=15, errors=4),
        _conn(5, healthy=False, in_flight=3, total=16, errors=5),
    )
    s11 = snapshot(conns11, 0, 8)
    obs11 = (
        s11.connections,
        s11.healthy,
        s11.in_flight,
        s11.total_requests,
        s11.total_errors,
    )
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. a_fully_dead_pool_reports_zero_healthy_but_its_connections_remain
    exp12 = LIVE_POOL_STATISTICS_SECURITY_MATRIX[11][1]
    dead12 = (
        _conn(1, healthy=False),
        _conn(2, healthy=False),
        _conn(3, healthy=False),
        _conn(4, healthy=False),
    )
    s12 = snapshot(dead12, 0, 0)
    obs12 = (s12.connections, s12.healthy)
    checks.append(
        {
            "name": LIVE_POOL_STATISTICS_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    return {
        "case_id": "live-pool-statistics-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
