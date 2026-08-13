"""Installed capacity catalog target selection and reachability validation."""
from __future__ import annotations

from typing import Final

from lumen.capacity.spec import ProfileAvailability, ProfileCatalog, TransitionGraph
from lumen.capacity.verdict import ProfileSelection

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity/catalog"


def select_profile(
    catalog: ProfileCatalog,
    transition_graph: TransitionGraph,
    current_profile: str,
    requested_profile: str,
) -> ProfileSelection:
    """Select target profile if installed, reachable, and available."""
    if requested_profile not in catalog.installed:
        return ProfileSelection(
            profile=None,
            reason="CapacityBlocked",
            field_path="requested_profile",
        )

    allowed = transition_graph.edges.get(current_profile, ())
    if requested_profile not in allowed:
        return ProfileSelection(
            profile=None,
            reason="CapacityBlocked",
            field_path="transition_graph",
        )

    avail = catalog.availability.get(requested_profile)
    if avail != ProfileAvailability.AVAILABLE:
        return ProfileSelection(
            profile=None,
            reason="CapacityBlocked",
            field_path="availability",
        )

    return ProfileSelection(
        profile=requested_profile,
        reason="ok",
        field_path="",
    )
