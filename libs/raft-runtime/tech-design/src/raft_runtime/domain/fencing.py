from __future__ import annotations

from dataclasses import dataclass

from raft_runtime.domain.errors import (
    AssignmentError,
    Expired,
    OwnerMismatch,
    StaleEpoch,
    Unassigned,
)

AssignmentEpoch = int  # type alias, monotonic, first assignment is 1

FIRST_EPOCH: AssignmentEpoch = 1


@dataclass(frozen=True, slots=True)
class FenceToken:
    owner: str
    epoch: AssignmentEpoch


@dataclass(frozen=True, slots=True)
class ActiveAssignment:
    token: FenceToken
    expires_at_ms: int


def next_epoch(previous: AssignmentEpoch) -> AssignmentEpoch:
    return previous + 1


def is_expired(expires_at_ms: int, now_ms: int) -> bool:
    return now_ms >= expires_at_ms


def fence_problem(
    active: ActiveAssignment | None,
    owner: str,
    epoch: AssignmentEpoch,
    now_ms: int,
) -> AssignmentError | None:
    if active is None:
        return Unassigned()
    if epoch != active.token.epoch:
        return StaleEpoch(expected=active.token.epoch, supplied=epoch)
    if owner != active.token.owner:
        return OwnerMismatch(expected=active.token.owner, supplied=owner)
    if is_expired(active.expires_at_ms, now_ms):
        return Expired(expires_at_ms=active.expires_at_ms, now_ms=now_ms)
    return None
