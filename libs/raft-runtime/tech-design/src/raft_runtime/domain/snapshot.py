from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class Disabled:
    pass


@dataclass(frozen=True, slots=True)
class EveryEntries:
    interval: int


@dataclass(frozen=True, slots=True)
class External:
    pass


SnapshotPolicy = Disabled | EveryEntries | External

DEFAULT_SNAPSHOT_POLICY: SnapshotPolicy = Disabled()


def should_snapshot(
    policy: SnapshotPolicy, applied_index: int, last_snapshot_index: int
) -> bool:
    if isinstance(policy, EveryEntries):
        return (
            policy.interval > 0
            and (applied_index - last_snapshot_index) >= policy.interval
        )
    return False


def compactable_upto(applied_index: int) -> int:
    return applied_index
