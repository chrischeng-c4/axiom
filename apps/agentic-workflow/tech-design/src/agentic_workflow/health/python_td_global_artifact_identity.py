"""Canonical Python TD for project-wide artifact identity validation.

@spec #2786
"""

from __future__ import annotations

from dataclasses import dataclass


__aw_artifact_id__ = "artifact:agentic-workflow/python-td-global-artifact-identity"


@dataclass(frozen=True)
class ArtifactOwner:
    artifact_id: str
    module_path: str
    migration_role: str | None = None


@dataclass(frozen=True)
class ArtifactIdentityCollision:
    artifact_id: str
    module_paths: tuple[str, ...]


def duplicate_artifact_identities(
    owners: tuple[ArtifactOwner, ...],
) -> tuple[ArtifactIdentityCollision, ...]:
    paths_by_id: dict[str, list[str]] = {}
    for owner in owners:
        paths_by_id.setdefault(owner.artifact_id, []).append(owner.module_path)
    return tuple(
        ArtifactIdentityCollision(artifact_id, tuple(sorted(paths)))
        for artifact_id, paths in sorted(paths_by_id.items())
        if len(paths) > 1
    )


def colliding_projection_identity(
    base_artifact_id: str,
    migration_role: str,
    target_suffix: str,
) -> str:
    if migration_role != "generated_mirror":
        return base_artifact_id
    return f"{base_artifact_id}-generated-projection-{target_suffix}"
