"""In-memory controllable Clock adapter for unit testing."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class ManualClock:
    now: int = 0

    def now_seconds(self) -> int:
        return self.now

    def advance(self, seconds: int) -> None:
        self.now += seconds
