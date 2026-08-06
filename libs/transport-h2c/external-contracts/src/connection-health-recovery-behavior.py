from __future__ import annotations

from transport_h2c.application.request import (
    Delivered,
    Failed,
    resolve_request,
    should_retry,
)
from transport_h2c.application.supervision import plan_sweep
from transport_h2c.domain.errors import (
    Connect,
    H2Protocol,
    InvalidRequest,
    NoConnection,
    Shutdown,
    Timeout,
    is_connection_lost,
)
from transport_h2c.infrastructure.config import ManagerConfig
from transport_h2c.infrastructure.connection import (
    ConnectionState,
    idle_ms,
    record_send,
)

MINIMUM_CHECKS = 16

CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX = (
    (
        "the_protocol_flag_matrix_decides_connection_loss",
        (False, True, True, True, True, True, True, True),
    ),
    ("a_failed_connect_and_an_absent_connection_are_both_loss", (True, True)),
    ("backpressure_shutdown_and_a_bad_request_are_not_loss", (False, False, False)),
    ("a_first_attempt_connection_loss_is_retried", True),
    ("a_second_attempt_connection_loss_is_not_retried", False),
    ("a_first_attempt_application_error_is_not_retried", False),
    ("a_retried_request_that_then_succeeds_reports_two_attempts", ("delivered", 2)),
    ("a_request_that_succeeds_first_time_reports_one_attempt", ("delivered", 1)),
    ("a_non_retryable_first_failure_stops_at_one_attempt", ("failed", "InvalidRequest", 1)),
    ("two_losses_exhaust_the_budget_and_the_last_is_reported", ("failed", "Connect", 2)),
    ("a_send_that_loses_the_connection_counts_and_retires_it", (2, 1, False)),
    ("a_send_that_fails_without_loss_counts_but_keeps_the_connection", (1, 1, True)),
    ("idle_time_never_runs_backwards", (0, 600)),
    ("a_sweep_evicts_the_dead_and_carries_their_counters_forward", ((2, 3), "none", 12, 4, 0)),
    ("a_sweep_that_leaves_the_pool_short_asks_for_replacements", ((2, 3), "none", 4, 1, 2)),
    ("an_idle_surplus_connection_is_shed_and_its_counters_retired", ((), 1, 3, 1, 0)),
)


def _cfg(
    min_connections: int = 1,
    max_connections: int = 4,
    max_keepalive_connections: int = 16,
    max_in_flight_per_origin: int = 128,
    grow_threshold: int = 32,
    idle_timeout_seconds: float = 5.0,
) -> ManagerConfig:
    return ManagerConfig(
        min_connections=min_connections,
        max_connections=max_connections,
        max_keepalive_connections=max_keepalive_connections,
        max_in_flight_per_origin=max_in_flight_per_origin,
        grow_threshold=grow_threshold,
        pool_timeout_seconds=5.0,
        connect_timeout_seconds=5.0,
        request_timeout_seconds=30.0,
        ping_interval_seconds=15.0,
        idle_timeout_seconds=idle_timeout_seconds,
        stream_window_bytes=1048576,
        conn_window_bytes=4194304,
        max_frame_bytes=16384,
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


def verify_connection_health_recovery_behavior() -> dict:
    checks = []

    # 1. the_protocol_flag_matrix_decides_connection_loss
    exp1 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[0][1]
    flags1 = (
        (False, False, False),
        (False, False, True),
        (False, True, False),
        (False, True, True),
        (True, False, False),
        (True, False, True),
        (True, True, False),
        (True, True, True),
    )
    res1 = []
    for g, i, r in flags1:
        res1.append(is_connection_lost(H2Protocol(go_away=g, io=i, reset=r)))
    obs1 = tuple(res1)
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_failed_connect_and_an_absent_connection_are_both_loss
    exp2 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[1][1]
    obs2 = (
        is_connection_lost(Connect("peer:1", "refused")),
        is_connection_lost(NoConnection("peer:1")),
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. backpressure_shutdown_and_a_bad_request_are_not_loss
    exp3 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[2][1]
    obs3 = (
        is_connection_lost(Timeout(5.0)),
        is_connection_lost(Shutdown()),
        is_connection_lost(InvalidRequest("bad")),
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. a_first_attempt_connection_loss_is_retried
    exp4 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[3][1]
    obs4 = should_retry(0, NoConnection("peer:1"))
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. a_second_attempt_connection_loss_is_not_retried
    exp5 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[4][1]
    obs5 = should_retry(1, NoConnection("peer:1"))
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. a_first_attempt_application_error_is_not_retried
    exp6 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[5][1]
    obs6 = should_retry(0, InvalidRequest("bad"))
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. a_retried_request_that_then_succeeds_reports_two_attempts
    exp7 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[6][1]
    req7 = resolve_request("peer:1", (NoConnection("peer:1"), None))
    obs7 = (
        ("delivered", req7.attempts)
        if isinstance(req7, Delivered)
        else ("failed", type(req7.error).__name__, req7.attempts)
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. a_request_that_succeeds_first_time_reports_one_attempt
    exp8 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[7][1]
    req8 = resolve_request("peer:1", (None,))
    obs8 = (
        ("delivered", req8.attempts)
        if isinstance(req8, Delivered)
        else ("failed", type(req8.error).__name__, req8.attempts)
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. a_non_retryable_first_failure_stops_at_one_attempt
    exp9 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[8][1]
    req9 = resolve_request("peer:1", (InvalidRequest("bad"), None))
    obs9 = (
        ("delivered", req9.attempts)
        if isinstance(req9, Delivered)
        else ("failed", type(req9.error).__name__, req9.attempts)
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. two_losses_exhaust_the_budget_and_the_last_is_reported
    exp10 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[9][1]
    req10 = resolve_request(
        "peer:1", (NoConnection("peer:1"), Connect("peer:1", "refused"))
    )
    obs10 = (
        ("delivered", req10.attempts)
        if isinstance(req10, Delivered)
        else ("failed", type(req10.error).__name__, req10.attempts)
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. a_send_that_loses_the_connection_counts_and_retires_it
    exp11 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[10][1]
    c11 = _conn(1)
    record_send(c11, None)
    record_send(c11, Connect("peer:1", "refused"))
    obs11 = (c11.total, c11.errors, c11.healthy)
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. a_send_that_fails_without_loss_counts_but_keeps_the_connection
    exp12 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[11][1]
    c12 = _conn(1)
    record_send(c12, InvalidRequest("bad"))
    obs12 = (c12.total, c12.errors, c12.healthy)
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. idle_time_never_runs_backwards
    exp13 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[12][1]
    c13 = _conn(1, last_used_ms=1000)
    obs13 = (idle_ms(c13, 400), idle_ms(c13, 1600))
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. a_sweep_evicts_the_dead_and_carries_their_counters_forward
    exp14 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[13][1]
    cfg14 = _cfg(min_connections=1)
    conns14 = (
        _conn(1, healthy=True, total=10, errors=2),
        _conn(2, healthy=False, total=5, errors=3),
        _conn(3, healthy=False, total=7, errors=1),
    )
    p14 = plan_sweep(conns14, cfg14, now_ms=0)
    obs14 = (
        p14.evicted,
        p14.shrunk if p14.shrunk is not None else "none",
        p14.retired_requests,
        p14.retired_errors,
        p14.replenish,
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. a_sweep_that_leaves_the_pool_short_asks_for_replacements
    exp15 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[14][1]
    cfg15 = _cfg(min_connections=3)
    conns15 = (
        _conn(1, healthy=True),
        _conn(2, healthy=False, total=4, errors=1),
        _conn(3, healthy=False),
    )
    p15 = plan_sweep(conns15, cfg15, now_ms=0)
    obs15 = (
        p15.evicted,
        p15.shrunk if p15.shrunk is not None else "none",
        p15.retired_requests,
        p15.retired_errors,
        p15.replenish,
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    # 16. an_idle_surplus_connection_is_shed_and_its_counters_retired
    exp16 = CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[15][1]
    cfg16 = _cfg(
        min_connections=1, max_keepalive_connections=16, idle_timeout_seconds=5.0
    )
    conns16 = (
        _conn(1, healthy=True, in_flight=0, total=3, errors=1, last_used_ms=0),
        _conn(2, healthy=True, in_flight=0, total=2, errors=0, last_used_ms=9000),
    )
    p16 = plan_sweep(conns16, cfg16, now_ms=9000)
    obs16 = (
        p16.evicted,
        p16.shrunk if p16.shrunk is not None else "none",
        p16.retired_requests,
        p16.retired_errors,
        p16.replenish,
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_BEHAVIOR_MATRIX[15][0],
            "expected": exp16,
            "observed": obs16,
            "passed": obs16 == exp16,
        }
    )

    return {
        "case_id": "connection-health-recovery-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
