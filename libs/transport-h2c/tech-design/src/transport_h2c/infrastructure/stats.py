from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass

from transport_h2c.infrastructure.connection import ConnectionState


@dataclass(frozen=True)
class ManagerStats:
    connections: int = 0
    healthy: int = 0
    in_flight: int = 0
    total_requests: int = 0
    total_errors: int = 0


def snapshot(
    connections: Sequence[ConnectionState],
    retired_requests: int,
    retired_errors: int,
) -> ManagerStats:
    return ManagerStats(
        connections=len(connections),
        healthy=sum(1 for c in connections if c.healthy),
        in_flight=sum(c.in_flight for c in connections),
        total_requests=sum(c.total for c in connections) + retired_requests,
        total_errors=sum(c.errors for c in connections) + retired_errors,
    )
