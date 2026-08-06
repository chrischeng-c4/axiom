from __future__ import annotations

from raft_runtime.domain.read_consistency import (
    ANY,
    LEADER,
    Bounded,
    ReadConsistency,
)
from raft_runtime.infrastructure.pod_name import ASCII_DIGITS

BOUNDED_PREFIX: str = "bounded("
BOUNDED_SUFFIX: str = ")"


def parse_read_consistency(raw: str | None) -> ReadConsistency:
    if raw is None:
        return LEADER
    cleaned = raw.strip().lower()
    if cleaned == "leader":
        return LEADER
    if cleaned == "any":
        return ANY
    if cleaned.startswith(BOUNDED_PREFIX) and cleaned.endswith(BOUNDED_SUFFIX):
        inner = cleaned[len(BOUNDED_PREFIX) : -len(BOUNDED_SUFFIX)]
        if inner and all(c in ASCII_DIGITS for c in inner):
            return Bounded(max_staleness_ms=int(inner))
    return LEADER
