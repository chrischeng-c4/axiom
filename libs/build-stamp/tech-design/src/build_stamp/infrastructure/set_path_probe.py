from __future__ import annotations

from typing import Iterable


class SetPathProbe:
    def __init__(self, existing: Iterable[str] = ()) -> None:
        self._existing = frozenset(existing)

    def exists(self, path: str) -> bool:
        return path in self._existing
