from __future__ import annotations

from dataclasses import dataclass

from raft_core.domain.ids import NodeId


@dataclass(frozen=True)
class Membership:
    voters: tuple[NodeId, ...]
    learners: tuple[NodeId, ...]


def auto_membership(n: int) -> Membership:
    size = max(n, 1)
    voters_count = size if size % 2 != 0 else size - 1
    voters = tuple(range(voters_count))
    learners = tuple(range(voters_count, size))
    return Membership(voters=voters, learners=learners)


def majority(voter_count: int) -> int:
    return voter_count // 2 + 1
