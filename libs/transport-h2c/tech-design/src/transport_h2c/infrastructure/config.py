from __future__ import annotations

from dataclasses import dataclass, replace

from transport_h2c.domain.sizing import recommended_connections


@dataclass(frozen=True)
class ManagerConfig:
    min_connections: int
    max_connections: int
    max_keepalive_connections: int
    max_in_flight_per_origin: int
    grow_threshold: int
    pool_timeout_seconds: float
    connect_timeout_seconds: float
    request_timeout_seconds: float | None
    ping_interval_seconds: float
    idle_timeout_seconds: float
    stream_window_bytes: int
    conn_window_bytes: int
    max_frame_bytes: int


def default_config(parallelism: int) -> ManagerConfig:
    return ManagerConfig(
        min_connections=1,
        max_connections=max(recommended_connections(128, parallelism), 1),
        max_keepalive_connections=16,
        max_in_flight_per_origin=128,
        grow_threshold=32,
        pool_timeout_seconds=5.0,
        connect_timeout_seconds=5.0,
        request_timeout_seconds=30.0,
        ping_interval_seconds=15.0,
        idle_timeout_seconds=5.0,
        stream_window_bytes=1024 * 1024,
        conn_window_bytes=4 * 1024 * 1024,
        max_frame_bytes=16 * 1024,
    )


def for_concurrency(concurrency: int, parallelism: int) -> ManagerConfig:
    base = default_config(parallelism)
    return replace(
        base,
        max_connections=max(
            recommended_connections(concurrency, parallelism),
            base.min_connections,
        ),
        max_in_flight_per_origin=max(concurrency, 1),
    )


def admission_permits(config: ManagerConfig) -> int:
    return max(config.max_in_flight_per_origin, 1)
