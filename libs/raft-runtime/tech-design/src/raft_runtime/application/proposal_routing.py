from __future__ import annotations

from dataclasses import dataclass

from raft_runtime.application.host_config import HostConfig
from raft_runtime.domain.consensus import (
    ClusterStateView,
    PeerAddr,
    is_leader,
    leader_peer,
)
from raft_runtime.infrastructure.routes import PUBLISH_PATH


@dataclass(frozen=True, slots=True)
class Local:
    pass


@dataclass(frozen=True, slots=True)
class Remote:
    peer: PeerAddr


@dataclass(frozen=True, slots=True)
class Unknown:
    pass


ProposalRoute = Local | Remote | Unknown

LOCAL: Local = Local()
UNKNOWN: Unknown = Unknown()


def route_proposal(view: ClusterStateView) -> ProposalRoute:
    if is_leader(view):
        return LOCAL
    peer = leader_peer(view)
    if peer is None:
        return UNKNOWN
    return Remote(peer=peer)


def forward_path() -> str:
    return PUBLISH_PATH


def retry_deadline_reached(elapsed_ms: int, config: HostConfig) -> bool:
    return elapsed_ms >= config.propose_timeout_ms
