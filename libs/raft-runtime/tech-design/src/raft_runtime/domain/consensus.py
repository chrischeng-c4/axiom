from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

Command = bytes  # opaque to this crate


class RaftRole(Enum):
    LEADER = "leader"
    FOLLOWER = "follower"
    CANDIDATE = "candidate"
    LEARNER = "learner"


@dataclass(frozen=True, slots=True)
class PeerAddr:
    node_id: int
    url: str


@dataclass(frozen=True, slots=True)
class ClusterStateView:
    node_id: int
    role: RaftRole
    term: int
    leader_id: int | None
    applied_index: int
    peers: tuple[PeerAddr, ...]


def is_leader(view: ClusterStateView) -> bool:
    return view.role is RaftRole.LEADER


def leader_peer(view: ClusterStateView) -> PeerAddr | None:
    if view.leader_id is None:
        return None
    for peer in view.peers:
        if peer.node_id == view.leader_id:
            return peer
    return None
