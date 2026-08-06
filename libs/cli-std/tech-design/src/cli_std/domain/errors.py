from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class ChainableViolation:
    reason: str


@dataclass(frozen=True)
class UnknownTopic:
    topic: str
    known: tuple[str, ...]


@dataclass(frozen=True)
class DigestMismatch:
    expected: str
    actual: str


@dataclass(frozen=True)
class MissingInnerBinary:
    inner_path: str


@dataclass(frozen=True)
class MalformedRepo:
    repo: str
