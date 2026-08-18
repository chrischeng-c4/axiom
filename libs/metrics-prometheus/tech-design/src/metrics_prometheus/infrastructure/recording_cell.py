from __future__ import annotations

from dataclasses import dataclass, field
from metrics_prometheus.infrastructure.ports import IntegerCell


@dataclass
class RecordingCell(IntegerCell):
    name: str
    log: list[tuple[str, str]]
    _value: int = 0

    def load(self) -> int:
        self.log.append((self.name, "load"))
        return self._value

    def store(self, value: int) -> None:
        self.log.append((self.name, "store"))
        self._value = value

    def add(self, delta: int) -> int:
        self.log.append((self.name, "add"))
        previous = self._value
        self._value += delta
        return previous


@dataclass
class RecordingCellFactory:
    log: list[tuple[str, str]] = field(default_factory=list)

    def __call__(self, name: str) -> IntegerCell:
        return RecordingCell(name=name, log=self.log)
