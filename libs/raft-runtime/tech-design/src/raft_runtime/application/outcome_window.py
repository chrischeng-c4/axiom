from __future__ import annotations

OUTCOME_WINDOW_DEFAULT_CAPACITY: int = 8192


class OutcomeWindow:
    def __init__(
        self, capacity: int = OUTCOME_WINDOW_DEFAULT_CAPACITY
    ) -> None:
        if capacity <= 0:
            raise ValueError("Capacity must be positive")
        self._capacity: int = capacity
        self._floor: int = 0
        self._store: dict[int, bytes] = {}

    def __len__(self) -> int:
        return len(self._store)

    def capacity(self) -> int:
        return self._capacity

    def floor(self) -> int:
        return self._floor

    def insert(self, index: int, outcome: bytes) -> None:
        if index < self._floor:
            return
        self._store[index] = outcome

    def claim(self, index: int) -> bytes | None:
        return self._store.get(index)

    def advance(self, applied_index: int) -> int:
        cutoff = max(0, applied_index - self._capacity)
        if cutoff <= self._floor:
            return 0

        evicted_count = 0
        to_remove = [idx for idx in self._store if idx < cutoff]
        for idx in to_remove:
            del self._store[idx]
            evicted_count += 1

        self._floor = cutoff
        return evicted_count
