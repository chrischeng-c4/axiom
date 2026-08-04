from __future__ import annotations

from collections.abc import Mapping

from raft_core.domain.ids import Index, NodeId, Term
from raft_core.domain.log_view import LogView
from raft_core.domain.membership import majority


def highest_committed(
    view: LogView,
    voters: tuple[NodeId, ...],
    matches: Mapping[NodeId, Index],
    leader_id: NodeId,
    current_term: Term,
    commit_index: Index,
) -> Index:
    last = view.last_index()
    new_commit = commit_index
    for n in range(commit_index + 1, last + 1):
        if view.term_at(n) != current_term:
            continue
        count = 0
        for v in voters:
            m = last if v == leader_id else matches.get(v, 0)
            if m >= n:
                count += 1
        if count >= majority(len(voters)):
            new_commit = n
    return new_commit
