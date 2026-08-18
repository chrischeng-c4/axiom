from __future__ import annotations

from dataclasses import dataclass

from raft_core.domain.entry import LogEntry
from raft_core.domain.ids import Index, NodeId, Term


@dataclass(frozen=True)
class PersistedState:
    term: Term = 0
    voted_for: NodeId | None = None
    log: tuple[LogEntry, ...] = ()
    commit_index: Index = 0
    snapshot_index: Index = 0
    snapshot_term: Term = 0
    snapshot: bytes = b""
