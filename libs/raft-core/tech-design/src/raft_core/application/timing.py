from __future__ import annotations

from dataclasses import dataclass
from typing import Final

from raft_core.domain.ids import NodeId

ELECTION_MIN: Final[int] = 50
HEARTBEAT_TIMEOUT: Final[int] = 3


def election_timeout_for(node_id: NodeId) -> int:
    # distinct per node so one voter always times out first
    return ELECTION_MIN + node_id


@dataclass
class ElectionClock:
    election_timeout: int
    election_elapsed: int = 0
    heartbeat_elapsed: int = 0

    def tick(self) -> None:
        self.election_elapsed += 1
        self.heartbeat_elapsed += 1

    def election_due(self) -> bool:
        return self.election_elapsed >= self.election_timeout

    def heartbeat_due(self) -> bool:
        return self.heartbeat_elapsed >= HEARTBEAT_TIMEOUT

    def reset_election(self) -> None:
        self.election_elapsed = 0

    def reset_heartbeat(self) -> None:
        self.heartbeat_elapsed = 0
