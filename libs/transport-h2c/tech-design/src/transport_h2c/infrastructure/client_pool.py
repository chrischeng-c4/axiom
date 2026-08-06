"""Client-helper pool for unmanaged h2c traffic.

H2cManager provides managed connection pooling (health tracking, dynamic
growth, admission limits, and statistics). H2cPool provides a complementary,
unmanaged surface: a fixed set of prior-knowledge HTTP/2 cleartext clients
handed out round-robin.

The unmanaged pool guarantees two key invariants:
1. Successive handouts rotate round-robin across all clients rather than
   pinning requests to the first client.
2. The pool size floor is enforced at construction, so requesting zero or
   negative connections still yields one usable client.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Final

from transport_h2c.domain.sizing import recommended_connections


@dataclass(frozen=True)
class ClientSettings:
    timeout_seconds: float | None = None
    user_agent: str | None = None


@dataclass(frozen=True)
class ClientPool:
    size: int
    settings: ClientSettings = ClientSettings()


def pool_of(
    connections: int,
    settings: ClientSettings = ClientSettings(),
) -> ClientPool:
    return ClientPool(size=max(connections, 1), settings=settings)


def pool_for_concurrency(
    concurrency: int,
    parallelism: int,
    settings: ClientSettings = ClientSettings(),
) -> ClientPool:
    rec = recommended_connections(concurrency, parallelism)
    return pool_of(rec, settings=settings)


def client_index(pool: ClientPool, cursor: int) -> int:
    return cursor % pool.size


def next_cursor(cursor: int) -> int:
    return cursor + 1


def handout(pool: ClientPool, cursor: int, count: int) -> tuple[int, ...]:
    if count <= 0:
        return ()
    out: list[int] = []
    c = cursor
    for _ in range(count):
        out.append(client_index(pool, c))
        c = next_cursor(c)
    return tuple(out)


PRIOR_KNOWLEDGE: Final[bool] = True


def builder_settings(settings: ClientSettings) -> dict[str, object]:
    result: dict[str, object] = {"http2_prior_knowledge": PRIOR_KNOWLEDGE}
    if settings.timeout_seconds is not None:
        result["timeout_seconds"] = settings.timeout_seconds
    if settings.user_agent is not None:
        result["user_agent"] = settings.user_agent
    return result
