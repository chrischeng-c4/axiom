from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class HardState:
    term: int
    voted_for: int | None


INITIAL_HARD_STATE: HardState = HardState(term=0, voted_for=None)


@dataclass(frozen=True, slots=True)
class SnapshotExists:
    path: str


class RaftStore:
    def __init__(self) -> None:
        self._last_saved: HardState | None = None

    def last_saved(self) -> HardState | None:
        return self._last_saved

    def load(self) -> HardState:
        if self._last_saved is None:
            return INITIAL_HARD_STATE
        return self._last_saved

    def save(self, state: HardState) -> bool:
        if self._last_saved is not None and state == self._last_saved:
            return False
        self._last_saved = state
        return True

    def seed_snapshot(
        self, path: str, exists: Callable[[str], bool]
    ) -> str | SnapshotExists:
        if exists(path):
            return SnapshotExists(path=path)
        return path
