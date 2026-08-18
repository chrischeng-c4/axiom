from __future__ import annotations

from dataclasses import dataclass

from raft_core.domain.entry import LogEntry
from raft_core.domain.ids import Index, NodeId, Term


@dataclass(frozen=True)
class VoteReq:
    term: Term
    candidate: NodeId
    last_log_index: Index
    last_log_term: Term


@dataclass(frozen=True)
class VoteResp:
    term: Term
    granted: bool


@dataclass(frozen=True)
class AppendReq:
    term: Term
    leader: NodeId
    prev_log_index: Index
    prev_log_term: Term
    entries: tuple[LogEntry, ...]
    leader_commit: Index


@dataclass(frozen=True)
class AppendResp:
    term: Term
    success: bool
    match_index: Index


@dataclass(frozen=True)
class InstallSnapshotReq:
    term: Term
    leader: NodeId
    snapshot_index: Index
    snapshot_term: Term
    data: bytes


@dataclass(frozen=True)
class InstallSnapshotResp:
    term: Term
    snapshot_index: Index


RaftMsg = (
    VoteReq
    | VoteResp
    | AppendReq
    | AppendResp
    | InstallSnapshotReq
    | InstallSnapshotResp
)


@dataclass(frozen=True)
class Outgoing:
    to: NodeId
    msg: RaftMsg
