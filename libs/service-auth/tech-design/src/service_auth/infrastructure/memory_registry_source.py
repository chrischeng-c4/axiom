"""In-memory RegistrySource adapter for unit testing."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class MemoryRegistrySource:
    name: str
    payload: str | None

    def read(self) -> str:
        if self.payload is None:
            raise RuntimeError(f"Read failure for source {self.name}")
        return self.payload
