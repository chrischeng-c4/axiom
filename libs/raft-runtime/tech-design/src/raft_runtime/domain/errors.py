from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class Unassigned:
    pass


@dataclass(frozen=True, slots=True)
class StaleEpoch:
    expected: int
    supplied: int


@dataclass(frozen=True, slots=True)
class OwnerMismatch:
    expected: str
    supplied: str


@dataclass(frozen=True, slots=True)
class Expired:
    expires_at_ms: int
    now_ms: int


@dataclass(frozen=True, slots=True)
class AlreadyAssigned:
    owner: str
    epoch: int


@dataclass(frozen=True, slots=True)
class ExpiryNotInFuture:
    expires_at_ms: int
    now_ms: int


@dataclass(frozen=True, slots=True)
class ExpiryNotLater:
    current_ms: int
    supplied: int


AssignmentError = (
    Unassigned
    | StaleEpoch
    | OwnerMismatch
    | Expired
    | AlreadyAssigned
    | ExpiryNotInFuture
    | ExpiryNotLater
)


@dataclass(frozen=True, slots=True)
class NamelessPod:
    pod_name: str


@dataclass(frozen=True, slots=True)
class BadOrdinal:
    pod_name: str
    suffix: str


@dataclass(frozen=True, slots=True)
class NonPositiveDimension:
    name: str
    value: int


@dataclass(frozen=True, slots=True)
class VoterCountOutOfRange:
    voter_count: int
    replicas_per_shard: int


@dataclass(frozen=True, slots=True)
class NodeIdOutOfRange:
    node_id: int
    replicas_per_shard: int


@dataclass(frozen=True, slots=True)
class UnsupportedScheme:
    scheme: str
    supported: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class MembershipChanged:
    current: int
    desired: int


TopologyError = (
    NamelessPod
    | BadOrdinal
    | NonPositiveDimension
    | VoterCountOutOfRange
    | NodeIdOutOfRange
    | UnsupportedScheme
    | MembershipChanged
)


class AppliedIndexError(ValueError):
    """Raised when a persisted applied-index payload cannot be interpreted."""
