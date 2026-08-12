"""Lumen catalog spec admission decider."""
from __future__ import annotations

from typing import Final, Sequence

from lumen.topology.catalog_spec import CatalogSpec, EligibleMember
from lumen.topology.catalog_verdict import (
    AdmittedCatalogPlan,
    CatalogRejectionReason,
    Rejection,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-catalog-admission"


def _select_voters(sorted_members: list[EligibleMember], count: int) -> list[EligibleMember]:
    by_host: dict[str, list[EligibleMember]] = {}
    for m in sorted_members:
        by_host.setdefault(m.hostname, []).append(m)

    hosts = sorted(by_host.keys())
    selected: list[EligibleMember] = []
    idx = 0
    while len(selected) < count and any(by_host.values()):
        h = hosts[idx % len(hosts)]
        if by_host[h]:
            selected.append(by_host[h].pop(0))
        idx += 1
    return selected


def decide_catalog_spec(
    spec: CatalogSpec,
    eligible_members: Sequence[EligibleMember],
) -> AdmittedCatalogPlan | Rejection:
    """Decide catalog specification admission and deterministic member placement."""
    if spec.mode == "non-ha":
        if len(eligible_members) < 1:
            return Rejection(
                reason=CatalogRejectionReason.INSUFFICIENT_ELIGIBLE_MEMBERS,
                field_path="eligible_members",
                message="insufficient eligible members for non-ha mode",
            )
        sorted_members = sorted(
            eligible_members,
            key=lambda m: (m.hostname, m.zone, m.member_id),
        )
        selected = sorted_members[0]
        return AdmittedCatalogPlan(
            voter_count=1,
            member_ids=(selected.member_id,),
            hostnames=(selected.hostname,),
            zones=(selected.zone,),
            limitation="non-HA single-voter mode has no fault tolerance",
        )

    if spec.mode == "three-voter-ha":
        if len(eligible_members) < 3:
            return Rejection(
                reason=CatalogRejectionReason.INSUFFICIENT_ELIGIBLE_MEMBERS,
                field_path="eligible_members",
                message="insufficient eligible members for three-voter-ha mode",
            )
        sorted_members = sorted(
            eligible_members,
            key=lambda m: (m.hostname, m.zone, m.member_id),
        )
        voters = _select_voters(sorted_members, 3)
        return AdmittedCatalogPlan(
            voter_count=3,
            member_ids=tuple(v.member_id for v in voters),
            hostnames=tuple(v.hostname for v in voters),
            zones=tuple(v.zone for v in voters),
            limitation=None,
        )

    return Rejection(
        reason=CatalogRejectionReason.UNSUPPORTED_CATALOG_MODE,
        field_path="mode",
        message=f"unsupported catalog mode: {spec.mode}",
    )
