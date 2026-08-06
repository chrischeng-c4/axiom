from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class NoMatchingRelease:
    prefix: str
    pin: str | None


@dataclass(frozen=True)
class UnreadableCurrentVersion:
    text: str


@dataclass(frozen=True)
class ProtocolInvalid:
    reason: str


@dataclass(frozen=True)
class StepInvalid:
    step_id: str
    reason: str
