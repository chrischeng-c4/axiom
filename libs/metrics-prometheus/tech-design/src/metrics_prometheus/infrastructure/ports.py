from __future__ import annotations

from typing import Protocol


class IntegerCell(Protocol):
    def load(self) -> int:
        ...

    def store(self, value: int) -> None:
        ...

    def add(self, delta: int) -> int:
        ...


class CellFactory(Protocol):
    def __call__(self, name: str) -> IntegerCell:
        ...
