from __future__ import annotations

from dataclasses import dataclass

from storage_durable.domain.failure import (
    StorageFailure,
    is_tolerated_directory_sync_failure,
)
from storage_durable.domain.fsync_policy import FsyncPolicy
from storage_durable.domain.pathing import parent_directory, staging_path
from storage_durable.infrastructure.ports import FileSystemPort

@dataclass(frozen=True)
class ReplaceRequest:
    path: str
    payload: bytes
    policy: FsyncPolicy

class DurableReplaceService:
    def __init__(self, filesystem: FileSystemPort) -> None:
        self._filesystem = filesystem

    def replace(self, request: ReplaceRequest) -> None:
        parent = parent_directory(request.path)
        if parent != "":
            self._filesystem.make_directories(parent)
        staging = staging_path(request.path)
        self._filesystem.remove(staging)
        self._filesystem.write(staging, request.payload)
        if request.policy.forces_stable_storage():
            self._filesystem.sync_file(staging)
        self._filesystem.rename(staging, request.path)
        if request.policy.forces_stable_storage():
            self._sync_parent(parent)

    def _sync_parent(self, parent: str) -> None:
        if parent == "":
            return
        try:
            self._filesystem.sync_directory(parent)
        except StorageFailure as failure:
            if is_tolerated_directory_sync_failure(failure.kind):
                return
            raise
