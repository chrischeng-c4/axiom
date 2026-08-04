from __future__ import annotations

from enum import Enum
from typing import Final

class FailureKind(str, Enum):
    NOT_FOUND = "not-found"
    PERMISSION_DENIED = "permission-denied"
    UNSUPPORTED = "unsupported"
    OTHER = "other"
    IO = "io"

class StorageFailure(Exception):
    kind: FailureKind
    path: str

    def __init__(self, kind: FailureKind, path: str) -> None:
        self.kind = kind
        self.path = path
        super().__init__(f"{kind.value}: {path}")

TOLERATED_DIRECTORY_SYNC_FAILURES: Final[frozenset[FailureKind]] = frozenset(
    {FailureKind.PERMISSION_DENIED, FailureKind.UNSUPPORTED, FailureKind.OTHER}
)

def is_tolerated_directory_sync_failure(kind: FailureKind) -> bool:
    return kind in TOLERATED_DIRECTORY_SYNC_FAILURES
