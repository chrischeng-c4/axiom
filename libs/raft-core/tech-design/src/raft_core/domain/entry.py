from __future__ import annotations

from dataclasses import dataclass

from raft_core.domain.ids import Index, Term


@dataclass(frozen=True)
class LogEntry:
    term: Term
    index: Index
    command: bytes
