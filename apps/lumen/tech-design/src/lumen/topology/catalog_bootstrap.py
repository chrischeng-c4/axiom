"""Lumen catalog bootstrap decider."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final

from lumen.topology.catalog_spec import BootstrapSeed
from lumen.topology.catalog_verdict import CatalogRejectionReason, Rejection

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-catalog-bootstrap"


@dataclass(frozen=True)
class AdmittedBootstrapSeed:
    seed: BootstrapSeed


def decide_bootstrap(
    seed: BootstrapSeed,
    expected_instance_id: str,
    retained_generation: int,
) -> AdmittedBootstrapSeed | Rejection:
    """Decide seed identity and bootstrap discovery admission."""
    if seed.instance_id != expected_instance_id:
        return Rejection(
            reason=CatalogRejectionReason.INSTANCE_ID_MISMATCH,
            field_path="seed.instance_id",
            message=f"seed instance_id '{seed.instance_id}' does not match expected '{expected_instance_id}'",
        )

    if seed.generation < retained_generation:
        return Rejection(
            reason=CatalogRejectionReason.STALE_SEED_GENERATION,
            field_path="generation",
            message=f"seed generation {seed.generation} is staler than retained generation {retained_generation}",
        )

    return AdmittedBootstrapSeed(seed=seed)
