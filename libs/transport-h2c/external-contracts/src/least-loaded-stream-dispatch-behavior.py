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

MINIMUM_CHECKS = 15

LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX = (
    ("an_empty_pool_selects_nothing", "none"),
    ("a_wholly_dead_pool_selects_nothing", "none"),
    ("the_strictly_least_loaded_of_three_is_chosen", 2),
    ("only_healthy_connections_are_candidates", 3),
    ("the_least_loaded_is_not_the_first_listed", 2),
    ("a_tie_resolves_to_the_earliest_listed", 1),
    ("growth_begins_exactly_at_the_threshold", True),
    ("no_growth_one_stream_below_the_threshold", False),
    ("no_growth_at_the_ceiling", False),
    ("growth_one_connection_below_the_ceiling", True),
    ("an_empty_pool_always_grows", True),
    ("slot_reservation_grants_up_to_the_ceiling_then_refuses", ((4, True), (4, False))),
    ("slot_release_saturates_at_zero", (1, 0)),
    ("reserve_and_release_return_in_flight_to_zero", 0),
    ("a_release_below_zero_is_refused", 0),
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


def verify_least_loaded_stream_dispatch_behavior() -> dict:
    checks = []

    # 1. an_empty_pool_selects_nothing
    exp1 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[0][1]
    sel1 = select_least_loaded(())
    obs1 = sel1.id if sel1 is not None else "none"
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_wholly_dead_pool_selects_nothing
    exp2 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[1][1]
    conns2 = (
        _conn(1, healthy=False, in_flight=0),
        _conn(2, healthy=False, in_flight=1),
        _conn(3, healthy=False, in_flight=2),
    )
    sel2 = select_least_loaded(conns2)
    obs2 = sel2.id if sel2 is not None else "none"
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. the_strictly_least_loaded_of_three_is_chosen
    exp3 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[2][1]
    conns3 = (_conn(1, in_flight=7), _conn(2, in_flight=2), _conn(3, in_flight=5))
    sel3 = select_least_loaded(conns3)
    obs3 = sel3.id if sel3 is not None else "none"
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. only_healthy_connections_are_candidates
    exp4 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[3][1]
    conns4 = (
        _conn(1, healthy=False, in_flight=0),
        _conn(2, in_flight=9),
        _conn(3, in_flight=4),
    )
    sel4 = select_least_loaded(conns4)
    obs4 = sel4.id if sel4 is not None else "none"
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. the_least_loaded_is_not_the_first_listed
    exp5 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[4][1]
    conns5 = (_conn(1, in_flight=9), _conn(2, in_flight=1), _conn(3, in_flight=5))
    sel5 = select_least_loaded(conns5)
    obs5 = sel5.id if sel5 is not None else "none"
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. a_tie_resolves_to_the_earliest_listed
    exp6 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[5][1]
    conns6 = (_conn(1, in_flight=3), _conn(2, in_flight=3), _conn(3, in_flight=3))
    sel6 = select_least_loaded(conns6)
    obs6 = sel6.id if sel6 is not None else "none"
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. growth_begins_exactly_at_the_threshold
    exp7 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[6][1]
    obs7 = should_grow(_conn(1, in_flight=32), 2, _cfg())
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. no_growth_one_stream_below_the_threshold
    exp8 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[7][1]
    obs8 = should_grow(_conn(1, in_flight=31), 2, _cfg())
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. no_growth_at_the_ceiling
    exp9 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[8][1]
    obs9 = should_grow(_conn(1, in_flight=40), 4, _cfg())
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. growth_one_connection_below_the_ceiling
    exp10 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[9][1]
    obs10 = should_grow(_conn(1, in_flight=40), 3, _cfg())
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. an_empty_pool_always_grows
    exp11 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[10][1]
    obs11 = should_grow(None, 0, _cfg())
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. slot_reservation_grants_up_to_the_ceiling_then_refuses
    exp12 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[11][1]
    obs12 = (reserve_slot(3, _cfg()), reserve_slot(4, _cfg()))
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. slot_release_saturates_at_zero
    exp13 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[12][1]
    obs13 = (release_slot(2), release_slot(0))
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. reserve_and_release_return_in_flight_to_zero
    exp14 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[13][1]
    c14 = _conn(1, in_flight=0)
    reserve(c14)
    reserve(c14)
    release(c14)
    release(c14)
    obs14 = c14.in_flight
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. a_release_below_zero_is_refused
    exp15 = LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[14][1]
    c15 = _conn(1, in_flight=0)
    release(c15)
    release(c15)
    obs15 = c15.in_flight
    checks.append(
        {
            "name": LEAST_LOADED_STREAM_DISPATCH_BEHAVIOR_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    return {
        "case_id": "least-loaded-stream-dispatch-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
