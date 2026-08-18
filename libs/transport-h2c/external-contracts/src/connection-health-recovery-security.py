from __future__ import annotations

from transport_h2c.application.request import (
    Delivered,
    Failed,
    resolve_request,
    should_retry,
)
from transport_h2c.application.supervision import plan_shutdown, plan_sweep
from transport_h2c.domain.errors import (
    Connect,
    H2Protocol,
    InvalidRequest,
    NoConnection,
    Shutdown,
    Timeout,
)
from transport_h2c.infrastructure.config import ManagerConfig
from transport_h2c.infrastructure.connection import (
    ConnectionState,
    mark_dead,
    record_send,
    release,
    reserve,
    touch,
)

MINIMUM_CHECKS = 14

CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX = (
    ("a_pool_at_its_floor_sheds_nothing_however_idle", ((), "none", 0, 0, 0)),
    ("a_connection_carrying_streams_is_never_shed_however_idle", ((), "none", 0, 0, 0)),
    ("a_pool_grown_past_its_keepalive_target_shrinks_back_toward_it", ((), 1, 0, 0, 0)),
    ("shutdown_reports_the_full_lifetime_totals", (23, 5)),
    ("an_over_full_pool_never_asks_for_a_negative_number_of_replacements", ((), "none", 0, 0, 0)),
    ("self_healing_cannot_reduce_the_lifetime_request_total", (100, 50)),
    ("every_error_class_that_ends_a_connection_retires_it", (False, False, False, False, False)),
    ("an_error_class_that_does_not_end_a_connection_leaves_it_alive", (True, True, True)),
    ("a_retired_connection_never_returns_to_health_by_itself", (False, 4, 1)),
    ("no_application_error_is_ever_replayed_to_the_peer", (False, False, False)),
    ("the_attempt_budget_is_two_and_a_third_outcome_is_never_consumed", ("failed", "NoConnection", 2)),
    ("an_empty_outcome_sequence_fails_closed", ("failed", "NoConnection", 0)),
    ("a_protocol_failure_on_the_final_attempt_is_reported_not_swallowed", ("failed", "H2Protocol", 2)),
    ("no_ordinary_operation_resurrects_a_dead_connection", (False, False, False, False)),
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


def verify_connection_health_recovery_security() -> dict:
    checks = []

    # 1. a_pool_at_its_floor_sheds_nothing_however_idle
    exp1 = CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[0][1]
    cfg1 = _cfg(min_connections=2, idle_timeout_seconds=5.0)
    conns1 = (
        _conn(1, healthy=True, in_flight=0, total=4, errors=1, last_used_ms=0),
        _conn(2, healthy=True, in_flight=0, total=6, errors=2, last_used_ms=0),
    )
    p1 = plan_sweep(conns1, cfg1, now_ms=1000000)
    obs1 = (
        p1.evicted,
        p1.shrunk if p1.shrunk is not None else "none",
        p1.retired_requests,
        p1.retired_errors,
        p1.replenish,
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_connection_carrying_streams_is_never_shed_however_idle
    exp2 = CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[1][1]
    cfg2 = _cfg(
        min_connections=1, max_keepalive_connections=16, idle_timeout_seconds=5.0
    )
    conns2 = (
        _conn(1, healthy=True, in_flight=3, total=5, errors=2, last_used_ms=0),
        _conn(2, healthy=True, in_flight=4, total=9, errors=0, last_used_ms=0),
    )
    p2 = plan_sweep(conns2, cfg2, now_ms=1000000)
    obs2 = (
        p2.evicted,
        p2.shrunk if p2.shrunk is not None else "none",
        p2.retired_requests,
        p2.retired_errors,
        p2.replenish,
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_pool_grown_past_its_keepalive_target_shrinks_back_toward_it
    exp3 = CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[2][1]
    cfg3 = _cfg(
        min_connections=1,
        max_keepalive_connections=2,
        idle_timeout_seconds=1000.0,
    )
    conns3 = (
        _conn(1, healthy=True, in_flight=0, last_used_ms=1000),
        _conn(2, healthy=True, in_flight=0, last_used_ms=1000),
        _conn(3, healthy=True, in_flight=0, last_used_ms=1000),
        _conn(4, healthy=True, in_flight=0, last_used_ms=1000),
    )
    p3 = plan_sweep(conns3, cfg3, now_ms=1000)
    obs3 = (
        p3.evicted,
        p3.shrunk if p3.shrunk is not None else "none",
        p3.retired_requests,
        p3.retired_errors,
        p3.replenish,
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. shutdown_reports_the_full_lifetime_totals
    exp4 = CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[3][1]
    conns4 = (
        _conn(1, total=5, errors=1),
        _conn(2, total=7, errors=0),
        _conn(3, total=11, errors=4),
    )
    obs4 = plan_shutdown(conns4)
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. an_over_full_pool_never_asks_for_a_negative_number_of_replacements
    exp5 = CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[4][1]
    cfg5 = _cfg(
        min_connections=2,
        max_keepalive_connections=16,
        idle_timeout_seconds=1000.0,
    )
    conns5 = tuple(
        _conn(i, healthy=True, in_flight=1, last_used_ms=1000) for i in range(1, 6)
    )
    p5 = plan_sweep(conns5, cfg5, now_ms=1000)
    obs5 = (
        p5.evicted,
        p5.shrunk if p5.shrunk is not None else "none",
        p5.retired_requests,
        p5.retired_errors,
        p5.replenish,
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. self_healing_cannot_reduce_the_lifetime_request_total
    exp6 = CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[5][1]
    cfg6 = _cfg(min_connections=1)
    conns6 = (
        _conn(1, healthy=True, total=1, errors=0),
        _conn(2, healthy=False, total=100, errors=50),
    )
    p6 = plan_sweep(conns6, cfg6, now_ms=0)
    obs6 = (p6.retired_requests, p6.retired_errors)
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. every_error_class_that_ends_a_connection_retires_it
    exp7 = CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[6][1]
    errs7 = (
        Connect("p", "x"),
        NoConnection("p"),
        H2Protocol(go_away=True),
        H2Protocol(io=True),
        H2Protocol(reset=True),
    )
    res7 = []
    for e in errs7:
        c = _conn(1)
        record_send(c, e)
        res7.append(c.healthy)
    obs7 = tuple(res7)
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. an_error_class_that_does_not_end_a_connection_leaves_it_alive
    exp8 = CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[7][1]
    errs8 = (Timeout(5.0), Shutdown(), InvalidRequest("bad"))
    res8 = []
    for e in errs8:
        c = _conn(1)
        record_send(c, e)
        res8.append(c.healthy)
    obs8 = tuple(res8)
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. a_retired_connection_never_returns_to_health_by_itself
    exp9 = CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[8][1]
    c9 = _conn(1)
    record_send(c9, Connect("p", "x"))
    record_send(c9, None)
    record_send(c9, None)
    record_send(c9, None)
    obs9 = (c9.healthy, c9.total, c9.errors)
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. no_application_error_is_ever_replayed_to_the_peer
    exp10 = CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[9][1]
    errs10 = (Timeout(5.0), Shutdown(), InvalidRequest("bad"))
    res10 = []
    for e in errs10:
        res10.append(should_retry(0, e))
    obs10 = tuple(res10)
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. the_attempt_budget_is_two_and_a_third_outcome_is_never_consumed
    exp11 = CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[10][1]
    req11 = resolve_request(
        "peer:1", (NoConnection("peer:1"), NoConnection("peer:1"), None)
    )
    obs11 = (
        ("delivered", req11.attempts)
        if isinstance(req11, Delivered)
        else ("failed", type(req11.error).__name__, req11.attempts)
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. an_empty_outcome_sequence_fails_closed
    exp12 = CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[11][1]
    req12 = resolve_request("peer:1", ())
    obs12 = (
        ("delivered", req12.attempts)
        if isinstance(req12, Delivered)
        else ("failed", type(req12.error).__name__, req12.attempts)
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. a_protocol_failure_on_the_final_attempt_is_reported_not_swallowed
    exp13 = CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[12][1]
    req13 = resolve_request(
        "peer:1", (H2Protocol(go_away=True), H2Protocol(io=True))
    )
    obs13 = (
        ("delivered", req13.attempts)
        if isinstance(req13, Delivered)
        else ("failed", type(req13.error).__name__, req13.attempts)
    )
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. no_ordinary_operation_resurrects_a_dead_connection
    exp14 = CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[13][1]
    c14 = _conn(1)
    mark_dead(c14)
    h_after = []
    reserve(c14)
    h_after.append(c14.healthy)
    release(c14)
    h_after.append(c14.healthy)
    touch(c14, 500)
    h_after.append(c14.healthy)
    record_send(c14, None)
    h_after.append(c14.healthy)
    obs14 = tuple(h_after)
    checks.append(
        {
            "name": CONNECTION_HEALTH_RECOVERY_SECURITY_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    return {
        "case_id": "connection-health-recovery-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
