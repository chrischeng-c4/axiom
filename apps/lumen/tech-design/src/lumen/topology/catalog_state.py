"""Immutable topology catalog state dataclass."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-catalog-state"


@dataclass(frozen=True)
class CatalogState:
    shard_ranges: tuple[tuple[int, int, str], ...] = ()
    shard_group_ids: tuple[str, ...] = ()
    member_roles: tuple[tuple[str, str], ...] = ()
    collection_schema_generations: tuple[tuple[str, int], ...] = ()
    mutation_intent: str = ""
    current_generation: int = 0
    converged_generation: int = 0
