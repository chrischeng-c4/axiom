from __future__ import annotations

from dataclasses import dataclass
from typing import Mapping

from storage_durable.domain.failure import FailureKind, StorageFailure
from storage_durable.domain.pathing import parent_directory
from storage_durable.infrastructure.ports import FileSystemPort

@dataclass(frozen=True)
class Operation:
    name: str
    path: str

class MemoryFileSystem:
    def __init__(self, files: Mapping[str, bytes] | None = None) -> None:
        self._files: dict[str, bytes] = dict(files) if files is not None else {}
        self._ops: list[Operation] = []
        self._fail_once: dict[tuple[str, str], FailureKind] = {}
        self._fail_always: dict[tuple[str, str], FailureKind] = {}

    def fail_once(self, operation: str, path: str, kind: FailureKind) -> None:
        self._fail_once[(operation, path)] = kind

    def fail_always(self, operation: str, path: str, kind: FailureKind) -> None:
        self._fail_always[(operation, path)] = kind

    def snapshot(self) -> dict[str, bytes]:
        return dict(self._files)

    def operations(self) -> tuple[Operation, ...]:
        return tuple(self._ops)

    def _record_and_check_failure(self, operation: str, path: str) -> None:
        self._ops.append(Operation(operation, path))
        key = (operation, path)
        if key in self._fail_once:
            kind = self._fail_once.pop(key)
            raise StorageFailure(kind, path)
        if key in self._fail_always:
            kind = self._fail_always[key]
            raise StorageFailure(kind, path)

    def read(self, path: str) -> bytes | None:
        self._record_and_check_failure("read", path)
        return self._files.get(path)

    def write(self, path: str, data: bytes) -> None:
        self._record_and_check_failure("write", path)
        self._files[path] = bytes(data)

    def append(self, path: str, data: bytes) -> None:
        self._record_and_check_failure("append", path)
        existing = self._files.get(path, b"")
        self._files[path] = existing + bytes(data)

    def remove(self, path: str) -> bool:
        self._record_and_check_failure("remove", path)
        if path in self._files:
            del self._files[path]
            return True
        return False

    def rename(self, source: str, target: str) -> None:
        self._record_and_check_failure("rename", source)
        if source not in self._files:
            raise StorageFailure(FailureKind.NOT_FOUND, source)
        data = self._files.pop(source)
        self._files[target] = data

    def exists(self, path: str) -> bool:
        self._record_and_check_failure("exists", path)
        return path in self._files

    def size(self, path: str) -> int | None:
        self._record_and_check_failure("size", path)
        data = self._files.get(path)
        if data is None:
            return None
        return len(data)

    def truncate(self, path: str, length: int) -> None:
        self._record_and_check_failure("truncate", path)
        if path not in self._files:
            raise StorageFailure(FailureKind.NOT_FOUND, path)
        data = self._files[path]
        if length < len(data):
            self._files[path] = data[:length]

    def sync_file(self, path: str) -> None:
        self._record_and_check_failure("sync_file", path)

    def sync_directory(self, path: str) -> None:
        self._record_and_check_failure("sync_directory", path)

    def list_directory(self, path: str) -> tuple[str, ...]:
        self._record_and_check_failure("list_directory", path)
        root = path if path == "/" else path.rstrip("/")
        entries: set[str] = set()
        for p in self._files:
            if parent_directory(p) == root:
                entry = p[p.rfind("/") + 1 :] if "/" in p else p
                entries.add(entry)
        return tuple(entries)

    def make_directories(self, path: str) -> None:
        self._record_and_check_failure("make_directories", path)
