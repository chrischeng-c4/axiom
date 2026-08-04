from __future__ import annotations

from dataclasses import dataclass

READ_CONSISTENCY_HEADER: str = "x-read-consistency"


@dataclass(frozen=True, slots=True)
class Leader:
    pass


@dataclass(frozen=True, slots=True)
class Bounded:
    max_staleness_ms: int


@dataclass(frozen=True, slots=True)
class Any_:
    pass


ReadConsistency = Leader | Bounded | Any_

LEADER: Leader = Leader()
ANY: Any_ = Any_()


def is_strongest(mode: ReadConsistency) -> bool:
    return isinstance(mode, Leader)


def tolerated_staleness_ms(mode: ReadConsistency) -> int | None:
    if isinstance(mode, Bounded):
        return mode.max_staleness_ms
    return None
