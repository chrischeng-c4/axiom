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

MINIMUM_CHECKS = 11

LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX = (
    ("a_leader_applies_the_proposal_on_this_node",
     ('Local', (), True)),
    ("a_follower_forwards_to_the_replica_the_view_names_as_leader",
     ('Remote', ((2, 'http://b:9000'),))),
    ("no_known_leader_is_a_retryable_unknown_not_a_failure",
     ('Unknown', ())),
    ("a_leader_id_the_topology_does_not_carry_is_also_unknown",
     ('Unknown', (), None)),
    ("forwarded_proposals_travel_the_published_peer_route",
     ('/raft/publish', True)),
    ("the_propose_budget_is_spent_only_once_the_timeout_arrives",
     (False, True, True, False)),
    ("the_retry_count_is_the_budget_divided_by_the_fixed_interval",
     (500, 20, 5, 0)),
    ("the_documented_host_defaults_are_the_ones_a_bare_config_reports",
     (20, 5, 400, 10000, 20, 5, 400, 10000)),
    ("the_shutdown_drain_window_is_twice_the_rpc_timeout",
     (800, 100, 0)),
    ("only_the_leader_role_applies_locally",
     (True, False, False, False)),
    ("the_leader_peer_is_the_one_whose_node_id_matches_the_leader_id",
     ((2, 'http://b:9000'), (3, 'http://c:9000'), None)),
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


def verify_leader_routed_proposal_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a leader applies the proposal on this node
    exp1 = LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[0][1]
    here = route_proposal(view(RaftRole.LEADER, 1))
    obs1 = plain((route_name(here), here, is_leader(view(RaftRole.LEADER,
        1))))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a follower forwards to the replica the view names as leader
    exp2 = LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[1][1]
    away = route_proposal(view(RaftRole.FOLLOWER, 2))
    obs2 = plain((route_name(away), away))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. no known leader is a retryable unknown not a failure
    exp3 = LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[2][1]
    blind = route_proposal(view(RaftRole.FOLLOWER, None))
    obs3 = plain((route_name(blind), blind))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a leader id the topology does not carry is also unknown
    exp4 = LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[3][1]
    missing = route_proposal(view(RaftRole.FOLLOWER, 9))
    obs4 = plain((route_name(missing), missing,
        leader_peer(view(RaftRole.FOLLOWER, 9))))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. forwarded proposals travel the published peer route
    exp5 = LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[4][1]
    obs5 = plain((forward_path(), forward_path() == "/raft/publish"))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the propose budget is spent only once the timeout arrives
    exp6 = LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[5][1]
    cfg = HostConfig()
    obs6 = plain((retry_deadline_reached(9999, cfg),
        retry_deadline_reached(10000, cfg), retry_deadline_reached(10001,
        cfg), retry_deadline_reached(0, cfg)))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the retry count is the budget divided by the fixed interval
    exp7 = LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[6][1]
    obs7 = plain((propose_attempts(HostConfig()), PROPOSE_RETRY_MS,
        propose_attempts(HostConfig(propose_timeout_ms=100)),
        propose_attempts(HostConfig(propose_timeout_ms=10))))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the documented host defaults are the ones a bare config reports
    exp8 = LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[7][1]
    bare = HostConfig()
    obs8 = plain((DEFAULT_TICK_MS, DEFAULT_PUMP_MS,
        DEFAULT_RPC_TIMEOUT_MS, DEFAULT_PROPOSE_TIMEOUT_MS, bare.tick_ms,
        bare.pump_ms, bare.rpc_timeout_ms, bare.propose_timeout_ms))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the shutdown drain window is twice the rpc timeout
    exp9 = LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[8][1]
    obs9 = plain((drain_budget_ms(HostConfig()),
        drain_budget_ms(HostConfig(rpc_timeout_ms=50)),
        drain_budget_ms(HostConfig(rpc_timeout_ms=0))))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. only the leader role applies locally
    exp10 = LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[9][1]
    obs10 = plain((is_leader(view(RaftRole.LEADER, 1)),
        is_leader(view(RaftRole.FOLLOWER, 1)),
        is_leader(view(RaftRole.CANDIDATE, 1)),
        is_leader(view(RaftRole.LEARNER, 1))))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the leader peer is the one whose node id matches the leader id
    exp11 = LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[10][1]
    obs11 = plain((leader_peer(view(RaftRole.FOLLOWER, 2)),
        leader_peer(view(RaftRole.FOLLOWER, 3)),
        leader_peer(view(RaftRole.FOLLOWER, None))))
    checks.append({"name": LEADER_ROUTED_PROPOSAL_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "leader-routed-proposal-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
