from __future__ import annotations

from raft_core.domain.ids import Index, NodeId, Term


def is_up_to_date(
    candidate_last_term: Term,
    candidate_last_index: Index,
    local_last_term: Term,
    local_last_index: Index,
) -> bool:
    return candidate_last_term > local_last_term or (
        candidate_last_term == local_last_term and candidate_last_index >= local_last_index
    )


def vote_granted(
    request_term: Term,
    current_term: Term,
    voted_for: NodeId | None,
    candidate: NodeId,
    up_to_date: bool,
) -> bool:
    return (
        request_term == current_term
        and (voted_for is None or voted_for == candidate)
        and up_to_date
    )
