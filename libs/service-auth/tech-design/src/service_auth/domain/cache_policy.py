"""Domain cache policy and classification for delegated authorization."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class CacheOutcome(str, Enum):
    HIT = "hit"
    MISS = "miss"
    STALE = "stale"


@dataclass(frozen=True)
class CachePolicy:
    allow_ttl_seconds: int = 300
    deny_ttl_seconds: int = 30
    stale_window_seconds: int = 60
    max_entries: int = 8192

    def ttl_for(self, allowed: bool) -> int:
        return self.allow_ttl_seconds if allowed else self.deny_ttl_seconds

    def revocation_bound_seconds(self) -> int:
        return self.allow_ttl_seconds + self.stale_window_seconds


def classify(
    policy: CachePolicy, stored_at: int, now: int, allowed: bool
) -> CacheOutcome:
    """Classify cache entry age as HIT, STALE, or MISS."""
    age = now - stored_at
    ttl = policy.ttl_for(allowed)
    if age < ttl:
        return CacheOutcome.HIT
    if age < ttl + policy.stale_window_seconds:
        return CacheOutcome.STALE
    return CacheOutcome.MISS
