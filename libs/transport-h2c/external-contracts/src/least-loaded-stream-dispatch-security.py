from __future__ import annotations

from transport_h2c.application.dispatch import (
    release_slot,
    reserve_slot,
    select_least_loaded,
    should_grow,
)
from transport_h2c.infrastructure.config import ManagerConfig
from transport_h2c.infrastructure.connection import (
    ConnectionState,
    release,
    reserve,
)

MINIMUM_CHECKS = 12

LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX = (
    (
        "no_dead_connection_is_selected_anywhere_in_the_health_matrix",
        ("none", 3, 2, 3, 1, 3, 2, 3),
    ),
    ("an_idle_dead_connection_loses_to_a_saturated_live_one", 2),
    ("growth_is_refused_at_the_ceiling_at_every_load", (False, False, False, False)),
    ("growth_is_refused_below_the_threshold_at_every_pool_size", (False, False, False, False)),
    ("reservation_never_exceeds_the_ceiling_from_any_starting_count", (1, 2, 3, 4, 4, 5)),
    ("repeated_release_cannot_drive_the_slot_count_negative", 0),
    ("repeated_release_cannot_drive_a_connections_load_negative", 0),
    ("an_over_released_connection_does_not_become_permanently_preferred", (1, 2)),
    ("growth_is_permitted_with_nothing_to_dispatch_onto_at_any_pool_size", (True, True)),
    ("the_threshold_comes_from_the_configuration_not_a_constant", (True, False)),
    ("the_ceiling_comes_from_the_configuration_not_a_constant", (False, True)),
    ("a_pool_of_one_dead_connection_yields_no_dispatch_target", "none"),
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


def verify_least_loaded_stream_dispatch_security() -> dict:
    checks = []

    # 1. no_dead_connection_is_selected_anywhere_in_the_health_matrix
    exp1 = LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[0][1]
    health_tuples = (
        (False, False, False),
        (False, False, True),
        (False, True, False),
        (False, True, True),
        (True, False, False),
        (True, False, True),
        (True, True, False),
        (True, True, True),
    )
    selected_1 = []
    for h1, h2, h3 in health_tuples:
        conns = (
            _conn(1, healthy=h1, in_flight=5),
            _conn(2, healthy=h2, in_flight=3),
            _conn(3, healthy=h3, in_flight=1),
        )
        sel = select_least_loaded(conns)
        selected_1.append(sel.id if sel is not None else "none")
    obs1 = tuple(selected_1)
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. an_idle_dead_connection_loses_to_a_saturated_live_one
    exp2 = LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[1][1]
    conns2 = (_conn(1, healthy=False, in_flight=0), _conn(2, healthy=True, in_flight=100))
    sel2 = select_least_loaded(conns2)
    obs2 = sel2.id if sel2 is not None else "none"
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. growth_is_refused_at_the_ceiling_at_every_load
    exp3 = LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[2][1]
    res3 = []
    for inf in (32, 33, 100, 1000000):
        res3.append(should_grow(_conn(1, in_flight=inf), 4, _cfg()))
    obs3 = tuple(res3)
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. growth_is_refused_below_the_threshold_at_every_pool_size
    exp4 = LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[3][1]
    res4 = []
    for ps in (0, 1, 2, 3):
        res4.append(should_grow(_conn(1, in_flight=31), ps, _cfg()))
    obs4 = tuple(res4)
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. reservation_never_exceeds_the_ceiling_from_any_starting_count
    exp5 = LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[4][1]
    res5 = []
    for s in (0, 1, 2, 3, 4, 5):
        cnt, _ = reserve_slot(s, _cfg())
        res5.append(cnt)
    obs5 = tuple(res5)
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. repeated_release_cannot_drive_the_slot_count_negative
    exp6 = LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[5][1]
    cnt6 = 2
    for _ in range(5):
        cnt6 = release_slot(cnt6)
    obs6 = cnt6
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. repeated_release_cannot_drive_a_connections_load_negative
    exp7 = LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[6][1]
    c7 = _conn(1, in_flight=0)
    release(c7)
    release(c7)
    release(c7)
    obs7 = c7.in_flight
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. an_over_released_connection_does_not_become_permanently_preferred
    exp8 = LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[7][1]
    c8_1 = _conn(1, in_flight=0)
    c8_2 = _conn(2, in_flight=0)
    release(c8_1)
    release(c8_1)
    sel8_a = select_least_loaded((c8_1, c8_2))
    id_a = sel8_a.id if sel8_a is not None else "none"
    reserve(c8_1)
    sel8_b = select_least_loaded((c8_1, c8_2))
    id_b = sel8_b.id if sel8_b is not None else "none"
    obs8 = (id_a, id_b)
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. growth_is_permitted_with_nothing_to_dispatch_onto_at_any_pool_size
    exp9 = LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[8][1]
    obs9 = (should_grow(None, 0, _cfg()), should_grow(None, 4, _cfg()))
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. the_threshold_comes_from_the_configuration_not_a_constant
    exp10 = LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[9][1]
    g10_a = should_grow(_conn(1, in_flight=1), 1, _cfg(grow_threshold=1))
    g10_b = should_grow(_conn(2, in_flight=999), 1, _cfg(grow_threshold=1000))
    obs10 = (g10_a, g10_b)
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. the_ceiling_comes_from_the_configuration_not_a_constant
    exp11 = LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[10][1]
    g11_a = should_grow(_conn(1, in_flight=50), 2, _cfg(max_connections=2))
    g11_b = should_grow(_conn(2, in_flight=50), 2, _cfg(max_connections=8))
    obs11 = (g11_a, g11_b)
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. a_pool_of_one_dead_connection_yields_no_dispatch_target
    exp12 = LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[11][1]
    c12 = _conn(1, healthy=False, in_flight=0)
    sel12 = select_least_loaded((c12,))
    obs12 = sel12.id if sel12 is not None else "none"
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    return {
        "case_id": "least-loaded-stream-dispatch-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
