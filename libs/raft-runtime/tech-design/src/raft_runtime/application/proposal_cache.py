from __future__ import annotations

from collections.abc import Sequence

DEFAULT_PROPOSAL_CACHE_CAPACITY: int = 4096


class ProposalCache:
    def __init__(
        self, capacity: int = DEFAULT_PROPOSAL_CACHE_CAPACITY
    ) -> None:
        if capacity <= 0:
            raise ValueError("Capacity must be positive")
        self._capacity: int = capacity
        self._store: dict[str, bytes] = {}

    def __len__(self) -> int:
        return len(self._store)

    def capacity(self) -> int:
        return self._capacity

    def insert(self, key: str, outcome: bytes) -> bytes:
        if key in self._store:
            return self._store[key]

        self._store[key] = outcome
        if len(self._store) > self._capacity:
            oldest_key = next(iter(self._store))
            del self._store[oldest_key]

        return outcome

    def get(self, key: str) -> bytes | None:
        return self._store.get(key)

    def snapshot(self) -> tuple[tuple[str, bytes], ...]:
        return tuple(self._store.items())

    def restore(self, entries: Sequence[tuple[str, bytes]]) -> None:
        self._store.clear()
        for k, v in entries:
            if k not in self._store:
                self._store[k] = v
        while len(self._store) > self._capacity:
            oldest_key = next(iter(self._store))
            del self._store[oldest_key]
