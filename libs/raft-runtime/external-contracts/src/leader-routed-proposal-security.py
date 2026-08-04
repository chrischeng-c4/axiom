from __future__ import annotations

from raft_runtime.application.host_config import (
    DEFAULT_PROPOSE_TIMEOUT_MS,
    DEFAULT_PUMP_MS,
    DEFAULT_RPC_TIMEOUT_MS,
    DEFAULT_TICK_MS,
    PROPOSE_RETRY_MS,
    HostConfig,
    drain_budget_ms,
    propose_attempts,
)
from raft_runtime.application.proposal_routing import (
    forward_path,
    retry_deadline_reached,
    route_proposal,
)
from raft_runtime.domain.consensus import (
    ClusterStateView,
    PeerAddr,
    RaftRole,
    is_leader,
    leader_peer,
)

MINIMUM_CHECKS = 10

LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX = (
    ("the_forwarding_target_is_an_entry_of_the_derived_topology",
     ('Remote', True, True)),
    ("a_leader_id_naming_no_known_peer_yields_no_target_at_all",
     ('Unknown', None, None)),
    ("an_empty_topology_can_never_produce_a_forwarding_target",
     ('Unknown', 'Unknown')),
    ("the_route_never_names_an_address_outside_the_view",
     ('http://c:9000', 3, True)),
    ("every_non_leader_role_forwards_rather_than_applying_here",
     ('Remote', 'Remote', 'Remote', 'Local')),
    ("an_unknown_route_is_a_different_shape_from_a_forwarding_route",
     (False, 'Unknown', 'Remote', ())),
    ("a_proposal_cannot_outlive_the_budget_it_was_given",
     (False, True, True)),
    ("a_zero_budget_is_already_spent_before_the_first_retry",
     (True, 0, False)),
    ("the_forward_path_is_a_fixed_route_not_derived_from_any_view",
     (True, '/raft/publish', True)),
    ("the_leader_peer_matches_on_node_id_and_not_on_list_position",
     ((3, 'http://c:9000'), (2, 'http://b:9000'), None)),
)


def plain(value: object) -> object:
    """A literal-shaped view: records by their fields, enum members by value.

    An expected value has to be a plain literal, and `repr` of a dataclass or
    an enum member is not one. Reading a record as the tuple of its fields
    keeps every field observable while staying transcribable.
    """
    fields = getattr(type(value), "__dataclass_fields__", None)
    if fields is not None:
        return tuple(plain(getattr(value, n)) for n in fields)
    if getattr(type(value), "__members__", None) is not None:
        return plain(value.value)
    if isinstance(value, tuple):
        return tuple(plain(v) for v in value)
    if isinstance(value, list):
        return [plain(v) for v in value]
    if isinstance(value, dict):
        return {k: plain(v) for k, v in value.items()}
    return value


PEERS: tuple[PeerAddr, ...] = (
    PeerAddr(node_id=2, url="http://b:9000"),
    PeerAddr(node_id=3, url="http://c:9000"),
)


def view(
    role: RaftRole,
    leader_id: int | None,
    peers: tuple[PeerAddr, ...] = PEERS,
) -> ClusterStateView:
    """A cluster view carrying only the fields proposal routing consults."""
    return ClusterStateView(
        node_id=1,
        role=role,
        term=7,
        leader_id=leader_id,
        applied_index=0,
        peers=peers,
    )


def route_name(route: object) -> str:
    """The route variant's name.

    `Local` and `Unknown` both carry no fields, so the field view alone
    cannot tell "apply here" apart from "nobody is leader yet".
    """
    return type(route).__name__


def verify_leader_routed_proposal_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the forwarding target is an entry of the derived topology
    exp1 = LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[0][1]
    picked = route_proposal(view(RaftRole.FOLLOWER, 2))
    obs1 = plain((route_name(picked), picked.peer is PEERS[0], picked.peer
        in PEERS))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a leader id naming no known peer yields no target at all
    exp2 = LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[1][1]
    unknown = route_proposal(view(RaftRole.FOLLOWER, 9))
    obs2 = plain((route_name(unknown), leader_peer(view(RaftRole.FOLLOWER,
        9)), leader_peer(view(RaftRole.FOLLOWER, 2, ()))))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. an empty topology can never produce a forwarding target
    exp3 = LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[2][1]
    obs3 = plain((route_name(route_proposal(view(RaftRole.FOLLOWER, 2,
        ()))), route_name(route_proposal(view(RaftRole.FOLLOWER, None,
        ())))))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the route never names an address outside the view
    exp4 = LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[3][1]
    chosen = route_proposal(view(RaftRole.FOLLOWER, 3))
    obs4 = plain((chosen.peer.url, chosen.peer.node_id, chosen.peer.url in
        (PEERS[0].url, PEERS[1].url)))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. every non leader role forwards rather than applying here
    exp5 = LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[4][1]
    obs5 = plain((route_name(route_proposal(view(RaftRole.FOLLOWER, 2))),
        route_name(route_proposal(view(RaftRole.CANDIDATE, 2))),
        route_name(route_proposal(view(RaftRole.LEARNER, 2))),
        route_name(route_proposal(view(RaftRole.LEADER, 2)))))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. an unknown route is a different shape from a forwarding route
    exp6 = LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[5][1]
    nowhere = route_proposal(view(RaftRole.FOLLOWER, None))
    somewhere = route_proposal(view(RaftRole.FOLLOWER, 2))
    obs6 = plain((route_name(nowhere) == route_name(somewhere),
        route_name(nowhere), route_name(somewhere), nowhere))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a proposal cannot outlive the budget it was given
    exp7 = LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[6][1]
    tight = HostConfig(propose_timeout_ms=100)
    obs7 = plain((retry_deadline_reached(99, tight),
        retry_deadline_reached(100, tight),
        retry_deadline_reached(1000000, tight)))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a zero budget is already spent before the first retry
    exp8 = LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[7][1]
    obs8 = plain((retry_deadline_reached(0,
        HostConfig(propose_timeout_ms=0)),
        propose_attempts(HostConfig(propose_timeout_ms=0)),
        retry_deadline_reached(0, HostConfig(propose_timeout_ms=1))))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the forward path is a fixed route not derived from any view
    exp9 = LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[8][1]
    obs9 = plain((forward_path() == forward_path(), forward_path(),
        forward_path().startswith("/raft/")))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the leader peer matches on node id and not on list position
    exp10 = LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[9][1]
    obs10 = plain((leader_peer(view(RaftRole.FOLLOWER, 3)),
        leader_peer(view(RaftRole.FOLLOWER, 2, (PEERS[1], PEERS[0]))),
        leader_peer(view(RaftRole.FOLLOWER, 1))))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "leader-routed-proposal-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
