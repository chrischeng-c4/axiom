from __future__ import annotations

from dataclasses import dataclass
from typing import Final


@dataclass(frozen=True)
class Bucket:
    label: str
    upper_bound: int


OVERFLOW: Final[None] = None


def assign(bounds: tuple[Bucket, ...], value: int) -> int | None:
    for index, bucket in enumerate(bounds):
        if value <= bucket.upper_bound:
            return index
    return None


def cumulative(counts: tuple[int, ...]) -> tuple[int, ...]:
    running = 0
    out: list[int] = []
    for c in counts:
        running += c
        out.append(running)
    return tuple(out)
