"""Complete-backup status observability models and deciders."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-backup-status"


@dataclass(frozen=True)
class BackupState:
    pinned_generation: int
    shard_artifact_progress: dict[str, str]
    last_successful_manifest: str
    failure_reason: str


@dataclass(frozen=True)
class BackupObservation:
    pinned_generation: int
    shard_artifact_progress: dict[str, str]
    last_successful_manifest: str
    failure_reason: str


def decide_backup_observation(state: BackupState) -> BackupObservation:
    """Project durable backup state into an immutable backup observation."""
    return BackupObservation(
        pinned_generation=state.pinned_generation,
        shard_artifact_progress=state.shard_artifact_progress,
        last_successful_manifest=state.last_successful_manifest,
        failure_reason=state.failure_reason,
    )
