from __future__ import annotations

from collections.abc import Callable

POD_NAME_KEY: str = "POD_NAME"
SHARD_COUNT_KEY: str = "SHARD_COUNT"
REPLICAS_PER_SHARD_KEY: str = "REPLICAS_PER_SHARD"
VOTER_COUNT_KEY: str = "VOTER_COUNT"
NODE_ID_KEY: str = "NODE_ID"
PEER_OVERRIDES_KEY: str = "PEER_OVERRIDES"

ALL_KEYS: tuple[str, ...] = (
    POD_NAME_KEY,
    SHARD_COUNT_KEY,
    REPLICAS_PER_SHARD_KEY,
    VOTER_COUNT_KEY,
    NODE_ID_KEY,
    PEER_OVERRIDES_KEY,
)

Lookup = Callable[[str], str | None]


def parse_peer_overrides(raw: str | None) -> tuple[str, ...]:
    if not raw:
        return ()
    items = [item.strip() for item in raw.split(",")]
    non_empty = [item for item in items if item]
    return tuple(non_empty)


def read_int(lookup: Lookup, key: str, default: int) -> int | None:
    val = lookup(key)
    if val is None:
        return default
    stripped = val.strip()
    if not stripped:
        return default
    try:
        return int(stripped)
    except ValueError:
        return None


def replica_mode(lookup: Lookup) -> bool:
    val = read_int(lookup, REPLICAS_PER_SHARD_KEY, 1)
    if val is None:
        return False
    return val > 1
