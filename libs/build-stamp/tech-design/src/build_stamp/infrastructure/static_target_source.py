from __future__ import annotations


class StaticTargetSource:
    def __init__(self, value: str | None) -> None:
        self._value = value

    def target_triple(self) -> str | None:
        return self._value
