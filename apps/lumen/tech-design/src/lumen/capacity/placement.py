"""Shared pool placement and node anti-affinity decider."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Final, Union

from lumen.capacity.verdict import CapacityReason, CapacityRejection

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity-placement"


@dataclass(frozen=True)
class Assignment:
    instance: str
    namespace: str
    machine_type: str
    pool_key: str
    node: str


@dataclass(frozen=True)
class PlacementVerdict:
    assignments: dict[str, Assignment]


def _get(obj: Any, key: str, default: Any = None) -> Any:
    if isinstance(obj, dict):
        return obj.get(key, default)
    return getattr(obj, key, default)


def decide_pool_assignments(
    instances: Any,
    placements: Any,
) -> Union[PlacementVerdict, CapacityRejection]:
    """Assign instances to machine pools and enforce node anti-affinity per pool."""
    instance_map: dict[str, Any] = {}
    for inst in instances:
        inst_id = _get(inst, "instance")
        if inst_id:
            instance_map[str(inst_id)] = inst

    placement_map: dict[str, str] = {}
    for p in placements:
        inst_id = _get(p, "instance")
        node_id = _get(p, "node")
        if inst_id and node_id:
            placement_map[str(inst_id)] = str(node_id)

    # Check node anti-affinity per pool key
    pool_nodes: dict[str, set[str]] = {}
    assignments: dict[str, Assignment] = {}

    for inst_id, inst_obj in instance_map.items():
        namespace = str(_get(inst_obj, "namespace", ""))
        machine_type = str(_get(inst_obj, "machine_type", ""))
        pool_key = f"data-{machine_type}"
        node = placement_map.get(inst_id, "")

        if pool_key not in pool_nodes:
            pool_nodes[pool_key] = set()

        if node in pool_nodes[pool_key]:
            return CapacityRejection(
                reason=CapacityReason.DATA_MEMBER_NODE_CONFLICT,
                field_path="placements",
                message=f"data members in pool {pool_key!r} conflict on node {node!r}",
            )
        pool_nodes[pool_key].add(node)

        assignments[inst_id] = Assignment(
            instance=inst_id,
            namespace=namespace,
            machine_type=machine_type,
            pool_key=pool_key,
            node=node,
        )

    return PlacementVerdict(assignments=assignments)
