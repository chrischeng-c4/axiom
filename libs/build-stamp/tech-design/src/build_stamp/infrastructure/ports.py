from __future__ import annotations

from typing import Protocol, runtime_checkable


@runtime_checkable
class ShaSource(Protocol):
    def read_short_sha(self) -> tuple[bool, bytes]: ...


@runtime_checkable
class ClockSource(Protocol):
    def epoch_seconds(self) -> int | None: ...


@runtime_checkable
class TargetSource(Protocol):
    def target_triple(self) -> str | None: ...


@runtime_checkable
class PathProbe(Protocol):
    def exists(self, path: str) -> bool: ...
