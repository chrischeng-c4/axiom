from __future__ import annotations

from dataclasses import dataclass, field
from typing import Protocol

from raft_core.domain.ids import NodeId
from raft_core.infrastructure.messages import Outgoing, RaftMsg


class RaftTransport(Protocol):
    def deliver(self, sender: NodeId, out: Outgoing) -> None: ...


@dataclass
class Outbox:
    pending: list[Outgoing] = field(default_factory=list)

    def send(self, to: NodeId, msg: RaftMsg) -> None:
        self.pending.append(Outgoing(to, msg))

    def drain(self) -> tuple[Outgoing, ...]:
        out = tuple(self.pending)
        self.pending.clear()
        return out
