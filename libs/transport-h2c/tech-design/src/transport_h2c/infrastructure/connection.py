from __future__ import annotations

from dataclasses import dataclass

from transport_h2c.domain.errors import H2cError, is_connection_lost


@dataclass
class ConnectionState:
    id: int
    healthy: bool = True
    in_flight: int = 0
    total: int = 0
    errors: int = 0
    last_used_ms: int = 0


def reserve(state: ConnectionState) -> None:
    state.in_flight += 1


def release(state: ConnectionState) -> None:
    state.in_flight -= 1


def mark_dead(state: ConnectionState) -> None:
    state.healthy = False


def touch(state: ConnectionState, now_ms: int) -> None:
    state.last_used_ms = now_ms


def idle_ms(state: ConnectionState, now_ms: int) -> int:
    return max(now_ms - state.last_used_ms, 0)


def record_send(state: ConnectionState, outcome: H2cError | None) -> None:
    state.total += 1
    if outcome is not None:
        state.errors += 1
        if is_connection_lost(outcome):
            mark_dead(state)
