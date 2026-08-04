from __future__ import annotations

from collections.abc import Sequence

from transport_h2c.infrastructure.config import ManagerConfig
from transport_h2c.infrastructure.connection import ConnectionState


def select_least_loaded(
    connections: Sequence[ConnectionState],
) -> ConnectionState | None:
    candidates = [c for c in connections if c.healthy]
    if not candidates:
        return None
    return min(candidates, key=lambda c: c.in_flight)


def should_grow(
    best: ConnectionState | None,
    pool_size: int,
    config: ManagerConfig,
) -> bool:
    if best is None:
        return True
    return (
        best.in_flight >= config.grow_threshold
        and pool_size < config.max_connections
    )


def reserve_slot(slots: int, config: ManagerConfig) -> tuple[int, bool]:
    if slots >= config.max_connections:
        return (slots, False)
    return (slots + 1, True)


def release_slot(slots: int) -> int:
    return max(slots - 1, 0)
