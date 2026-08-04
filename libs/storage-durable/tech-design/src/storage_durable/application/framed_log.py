from __future__ import annotations

from dataclasses import dataclass

from storage_durable.domain.failure import (
    StorageFailure,
    is_tolerated_directory_sync_failure,
)
from storage_durable.domain.frame import (
    MAX_PAYLOAD_LENGTH,
    FrameRejection,
    LogFrame,
    encode_frame,
)
from storage_durable.domain.fsync_policy import FsyncPolicy
from storage_durable.domain.log_scan import frames_after, highest_seq, scan
from storage_durable.domain.pathing import (
    compaction_path,
    parent_directory,
)
from storage_durable.infrastructure.ports import FileSystemPort

@dataclass(frozen=True)
class RecoveredLog:
    frames: tuple[LogFrame, ...]
    good_end: int
    original_length: int
    rejection: FrameRejection | None

    @property
    def truncated(self) -> bool:
        return self.good_end < self.original_length

class FramedLogService:
    def __init__(self, filesystem: FileSystemPort) -> None:
        self._filesystem = filesystem

    def open_for_append(self, path: str, policy: FsyncPolicy) -> RecoveredLog:
        parent = parent_directory(path)
        if parent != "":
            self._filesystem.make_directories(parent)
        buffer = self._filesystem.read(path)
        if buffer is None:
            buffer = b""
        result = scan(buffer)
        original_len = len(buffer)
        if result.good_end < original_len:
            self._filesystem.truncate(path, result.good_end)
            if policy.forces_stable_storage():
                self._filesystem.sync_file(path)
        return RecoveredLog(result.frames, result.good_end, original_len, result.rejection)

    def append(self, path: str, seq: int, payload: bytes, policy: FsyncPolicy) -> None:
        if len(payload) > MAX_PAYLOAD_LENGTH:
            raise ValueError(f"Payload length {len(payload)} exceeds maximum {MAX_PAYLOAD_LENGTH}")
        self._filesystem.append(path, encode_frame(seq, payload))
        if policy.should_sync_immediately():
            self._filesystem.sync_file(path)

    def replay(self, path: str, from_seq: int) -> tuple[tuple[LogFrame, ...], int]:
        buffer = self._filesystem.read(path) or b""
        result = scan(buffer)
        selected = frames_after(result.frames, from_seq)
        return (selected, highest_seq(selected))

    def compact_through(self, path: str, through_seq: int, policy: FsyncPolicy) -> int:
        buffer = self._filesystem.read(path) or b""
        result = scan(buffer)
        retained = frames_after(result.frames, through_seq)
        staging = compaction_path(path)
        self._filesystem.remove(staging)
        rebuilt = b"".join(encode_frame(f.seq, f.payload) for f in retained)
        self._filesystem.write(staging, rebuilt)
        if policy.forces_stable_storage():
            self._filesystem.sync_file(staging)
        self._filesystem.rename(staging, path)
        if policy.forces_stable_storage():
            parent = parent_directory(path)
            if parent != "":
                try:
                    self._filesystem.sync_directory(parent)
                except StorageFailure as failure:
                    if not is_tolerated_directory_sync_failure(failure.kind):
                        raise
        return len(retained)
