from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass

from transport_h2c.infrastructure.config import ManagerConfig
from transport_h2c.infrastructure.connection import ConnectionState, idle_ms


@dataclass(frozen=True)
class SweepPlan:
    evicted: tuple[int, ...]
    shrunk: int | None
    retired_requests: int
    retired_errors: int
    replenish: int


def plan_sweep(
    connections: Sequence[ConnectionState],
    config: ManagerConfig,
    now_ms: int,
) -> SweepPlan:
    evicted_entries = [c for c in connections if not c.healthy]
    survivors = [c for c in connections if c.healthy]

    keepalive_ceiling = max(
        config.max_keepalive_connections, config.min_connections
    )
    idle_limit_ms = config.idle_timeout_seconds * 1000

    shed: ConnectionState | None = None
    if len(survivors) > config.min_connections:
        for c in survivors:
            if c.in_flight == 0 and (
                len(survivors) > keepalive_ceiling
                or idle_ms(c, now_ms) >= idle_limit_ms
            ):
                shed = c
                break

    retired = evicted_entries + ([shed] if shed is not None else [])
    retired_requests = sum(c.total for c in retired)
    retired_errors = sum(c.errors for c in retired)

    remaining = len(survivors) - (1 if shed is not None else 0)
    replenish = max(config.min_connections - remaining, 0)

    evicted_ids = tuple(c.id for c in evicted_entries)
    return SweepPlan(
        evicted=evicted_ids,
        shrunk=shed.id if shed is not None else None,
        retired_requests=retired_requests,
        retired_errors=retired_errors,
        replenish=replenish,
    )


def plan_shutdown(
    connections: Sequence[ConnectionState],
) -> tuple[int, int]:
    return (
        sum(c.total for c in connections),
        sum(c.errors for c in connections),
    )
