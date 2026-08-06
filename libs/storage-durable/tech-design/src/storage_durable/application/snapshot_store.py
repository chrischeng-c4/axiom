from __future__ import annotations

from dataclasses import dataclass

from storage_durable.application.durable_replace import (
    DurableReplaceService,
    ReplaceRequest,
)
from storage_durable.domain.fsync_policy import FsyncPolicy
from storage_durable.domain.pathing import join_path
from storage_durable.domain.snapshot_name import (
    order_by_sequence,
    parse_name,
    render_name,
)
from storage_durable.infrastructure.ports import FileSystemPort

@dataclass(frozen=True)
class SnapshotStoreConfig:
    root: str
    prefix: str
    extension: str
    policy: FsyncPolicy

@dataclass(frozen=True)
class SnapshotEntry:
    seq: int
    name: str

class SnapshotStoreService:
    def __init__(self, filesystem: FileSystemPort, replace: DurableReplaceService) -> None:
        self._filesystem = filesystem
        self._replace = replace

    def save(self, config: SnapshotStoreConfig, seq: int, payload: bytes) -> str:
        name = render_name(config.prefix, seq, config.extension)
        path = join_path(config.root, name)
        self._replace.replace(ReplaceRequest(path, payload, config.policy))
        return path

    def entries(self, config: SnapshotStoreConfig) -> tuple[SnapshotEntry, ...]:
        found: list[tuple[int, str]] = []
        for name in self._filesystem.list_directory(config.root):
            parsed = parse_name(name, config.prefix, config.extension)
            if parsed is not None:
                found.append((parsed, name))
        return tuple(SnapshotEntry(seq, name) for seq, name in order_by_sequence(found))

    def load_latest(self, config: SnapshotStoreConfig) -> bytes | None:
        listed = self.entries(config)
        if not listed:
            return None
        return self._filesystem.read(join_path(config.root, listed[-1].name))

    def prune(self, config: SnapshotStoreConfig, keep: int) -> int:
        if keep < 0:
            raise ValueError(f"keep must be non-negative, got {keep}")
        listed = self.entries(config)
        if len(listed) <= keep:
            return 0
        removed = 0
        for entry in listed[: len(listed) - keep]:
            if self._filesystem.remove(join_path(config.root, entry.name)):
                removed += 1
        return removed
