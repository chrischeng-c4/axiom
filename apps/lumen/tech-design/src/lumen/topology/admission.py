"""Topology admission, placement, and mutation deciders."""
from __future__ import annotations

from typing import Final, Iterable

from lumen.topology.spec import TopologySpec
from lumen.topology.verdict import (
    AdmittedTopology,
    Rejection,
    RejectionReason,
    TopologyVerdict,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-admission"


def decide_topology_spec(spec: TopologySpec) -> TopologyVerdict:
    """Fail-closed admission for a candidate TopologySpec."""
    if spec.legacy_replicas is not None:
        return Rejection(
            reason=RejectionReason.LEGACY_REPLICA_VOCABULARY,
            field_path="legacy_replicas",
            message="legacy replica field is deprecated and ambiguous",
        )

    if bool(spec.hpa_knobs):
        return Rejection(
            reason=RejectionReason.HPA_KNOB_NOT_OWNED,
            field_path="hpa_knobs",
            message="HPA/VPA thresholds are not owned by Lumen topology",
        )

    if spec.shard_minimum <= 0:
        return Rejection(
            reason=RejectionReason.ZERO_SHARD_MINIMUM,
            field_path="shard_minimum",
            message="shard_minimum must be greater than zero",
        )

    if spec.voters <= 0:
        return Rejection(
            reason=RejectionReason.ZERO_VOTERS,
            field_path="voters",
            message="voter count cannot be zero or negative",
        )

    if spec.voters % 2 == 0:
        return Rejection(
            reason=RejectionReason.EVEN_VOTER_COUNT,
            field_path="voters",
            message="even voter counts are unsafe due to split-brain deadlock risk",
        )

    if spec.voters not in (1, 3):
        return Rejection(
            reason=RejectionReason.UNSUPPORTED_VOTER_COUNT,
            field_path="voters",
            message=f"voter count {spec.voters} is unsupported; only 1 or 3 voters are allowed",
        )

    if spec.read_replicas < 0:
        return Rejection(
            reason=RejectionReason.NEGATIVE_READ_REPLICAS,
            field_path="read_replicas",
            message="read_replicas cannot be negative",
        )

    return AdmittedTopology(
        shard_count=spec.shard_minimum,
        voters=spec.voters,
        read_replicas=spec.read_replicas,
    )


def decide_topology_mutation(
    current_spec: TopologySpec, target_spec: TopologySpec
) -> TopologyVerdict:
    """Decide whether a topology spec mutation is safe and admitted."""
    target_verdict = decide_topology_spec(target_spec)
    if isinstance(target_verdict, Rejection):
        return target_verdict

    current_verdict = decide_topology_spec(current_spec)
    if isinstance(current_verdict, Rejection):
        return current_verdict

    current_pvc = current_spec.shard_pvc_capacity_gib
    target_pvc = target_spec.shard_pvc_capacity_gib

    if target_spec.shard_minimum < current_spec.shard_minimum:
        return Rejection(
            reason=RejectionReason.SHARD_CONTRACTION_NOT_SUPPORTED,
            field_path="shard_minimum",
            message="automatic shard count contraction is not supported in v1",
        )

    if target_spec.voters < current_spec.voters:
        return Rejection(
            reason=RejectionReason.VOTER_CONTRACTION_NOT_SUPPORTED,
            field_path="voters",
            message="automatic voter count contraction is not supported in v1",
        )

    if target_pvc < current_pvc:
        return Rejection(
            reason=RejectionReason.SHARD_PVC_CAPACITY_CONTRACTION_NOT_SUPPORTED,
            field_path="shard_pvc_capacity_gib",
            message="automatic voter/shard PVC capacity contraction is not supported in v1",
        )

    if target_spec.shard_minimum != current_spec.shard_minimum or target_spec.voters != current_spec.voters:
        return Rejection(
            reason=RejectionReason.NO_SAFE_TOPOLOGY_MUTATION,
            field_path="shard_minimum" if target_spec.shard_minimum != current_spec.shard_minimum else "voters",
            message="no safe controller exists for topology mutation",
        )

    return target_verdict


def decide_placement(
    placements: Iterable[tuple[str, str]]
) -> TopologyVerdict:
    """Decide whether a set of (instance_id, node_name) member placements is valid.
    
    R2/AC3: Enforces one data member per Kubernetes node across all Lumen instances.
    """
    seen_nodes: dict[str, str] = {}
    total_members = 0

    for instance_id, node_name in placements:
        total_members += 1
        if node_name in seen_nodes:
            return Rejection(
                reason=RejectionReason.DATA_MEMBER_NODE_CONFLICT,
                field_path="placement.node_name",
                message=f"node {node_name!r} has co-located data members ({seen_nodes[node_name]} and {instance_id})",
            )
        seen_nodes[node_name] = instance_id

    return AdmittedTopology(
        shard_count=len(seen_nodes),
        voters=total_members,
        read_replicas=0,
    )
