from __future__ import annotations


class ManualClock:
    def __init__(self, epoch_seconds: int | None) -> None:
        self._epoch_seconds = epoch_seconds

    def epoch_seconds(self) -> int | None:
        return self._epoch_seconds
