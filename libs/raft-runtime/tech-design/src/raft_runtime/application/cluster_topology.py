from __future__ import annotations

from dataclasses import dataclass

from raft_runtime.domain.consensus import PeerAddr
from raft_runtime.domain.errors import (
    MembershipChanged,
    NonPositiveDimension,
    TopologyError,
)
from raft_runtime.domain.topology import (
    ClusterDims,
    dims_problem,
    peer_ordinal,
)
from raft_runtime.infrastructure.env import (
    NODE_ID_KEY,
    POD_NAME_KEY,
    REPLICAS_PER_SHARD_KEY,
    SHARD_COUNT_KEY,
    VOTER_COUNT_KEY,
    Lookup,
    read_int,
)
from raft_runtime.infrastructure.peer_url import (
    peer_url,
    scheme_problem,
)
from raft_runtime.infrastructure.pod_name import PodIdentity, split_pod_name


@dataclass(frozen=True, slots=True)
class ClusterTopology:
    dims: ClusterDims
    node_id: int
    prefix: str
    peers: tuple[PeerAddr, ...]


def topology_from_env(
    lookup: Lookup,
    *,
    fallback_prefix: str,
    scheme: str,
    service: str,
    port: int,
) -> ClusterTopology | TopologyError:
    sp = scheme_problem(scheme)
    if sp is not None:
        return sp

    shard_count = read_int(lookup, SHARD_COUNT_KEY, 1)
    if shard_count is None:
        return NonPositiveDimension(name=SHARD_COUNT_KEY, value=0)

    replicas_per_shard = read_int(lookup, REPLICAS_PER_SHARD_KEY, 1)
    if replicas_per_shard is None:
        return NonPositiveDimension(name=REPLICAS_PER_SHARD_KEY, value=0)

    voter_count = read_int(lookup, VOTER_COUNT_KEY, 1)
    if voter_count is None:
        return NonPositiveDimension(name=VOTER_COUNT_KEY, value=0)

    node_id = read_int(lookup, NODE_ID_KEY, 0)
    if node_id is None:
        return NonPositiveDimension(name=NODE_ID_KEY, value=0)

    dp = dims_problem(
        shard_count=shard_count,
        replicas_per_shard=replicas_per_shard,
        voter_count=voter_count,
        node_id=node_id,
    )
    if dp is not None:
        return dp

    raw_pod = lookup(POD_NAME_KEY)
    prefix: str
    ordinal: int
    if raw_pod is None or raw_pod.strip() == "":
        prefix = fallback_prefix
        ordinal = node_id
    else:
        parsed = split_pod_name(raw_pod)
        if isinstance(parsed, PodIdentity):
            prefix = parsed.prefix
            ordinal = parsed.ordinal
        else:
            prefix = fallback_prefix
            ordinal = node_id

    dims = ClusterDims(
        shard_count=shard_count,
        replicas_per_shard=replicas_per_shard,
        voter_count=voter_count,
        ordinal=ordinal,
    )

    peers_list: list[PeerAddr] = []
    for replica in range(replicas_per_shard):
        ordinal_r = peer_ordinal(shard_count, dims.shard_index, replica)
        url_or_err = peer_url(scheme, prefix, ordinal_r, service, port)
        if isinstance(url_or_err, TopologyError):
            return url_or_err
        peers_list.append(PeerAddr(node_id=replica, url=url_or_err))

    return ClusterTopology(
        dims=dims,
        node_id=node_id,
        prefix=prefix,
        peers=tuple(peers_list),
    )


def ensure_static_membership_unchanged(
    current: int, desired: int
) -> TopologyError | None:
    if current != desired:
        return MembershipChanged(current=current, desired=desired)
    return None
