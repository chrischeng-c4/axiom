from __future__ import annotations

from raft_core.application.node import (
    RaftNode,
    from_persisted,
    new_node,
)
from raft_core.application.timing import (
    ELECTION_MIN,
    ElectionClock,
    HEARTBEAT_TIMEOUT,
    election_timeout_for,
)
from raft_core.domain.entry import LogEntry
from raft_core.domain.membership import (
    auto_membership,
    majority,
)
from raft_core.infrastructure.messages import (
    VoteReq,
    VoteResp,
)
from raft_core.infrastructure.persistence import PersistedState

MINIMUM_CHECKS = 16

ELECTION_SAFETY_BEHAVIOR_MATRIX = (
    ("the_voter_set_is_odd_at_every_group_size", (((0,), ()), ((0,), ()), ((0,), (1,)), ((0, 1, 2), ()), ((0, 1, 2), (3,)), ((0, 1, 2, 3, 4), ()))),
    ("a_majority_is_more_than_half_the_voters", (1, 2, 3, 4, 5)),
    ("a_fresh_node_is_a_follower_at_term_zero_with_no_vote_spent", ('follower', 0, None, None, True, 0)),
    ("each_node_has_its_own_election_timeout_so_one_voter_fires_first", (50, 3, (50, 51, 52, 53, 54))),
    ("the_election_timer_fires_on_the_tick_that_reaches_the_timeout", (False, False, True, (False, 0, 3))),
    ("a_lone_voter_wins_its_own_election_immediately", ('leader', True, 1, 0, 0)),
    ("campaigning_bumps_the_term_and_spends_the_nodes_own_vote", ('candidate', 1, 0, None, (0,), 2, (1, 2))),
    ("the_vote_request_carries_the_candidates_own_last_index_and_term", (1, 0, 2, 4)),
    ("a_majority_of_grants_promotes_the_candidate", ('leader', 1, 0, 'follower', 0, 0)),
    ("taking_office_initialises_every_peers_cursors_from_the_leaders_log", (((1, 3), (2, 3)), ((1, 0), (2, 0)), 2, 'leader')),
    ("a_learner_never_campaigns_no_matter_how_long_it_waits", ((3,), False, 'follower', 0, 0)),
    ("a_repeated_request_from_the_same_candidate_costs_nothing", (True, True, 0, 1)),
    ("granting_a_vote_restarts_the_voters_own_election_timer", (40, 0, 0)),
    ("the_term_and_the_vote_survive_a_restart", (1, 0, 1, 0, 'follower', 0, None, 0, 0)),
    ("a_leader_heartbeats_on_its_own_cadence", ((0, 0, 2, 0, 0, 2), 'leader')),
    ("the_outcome_of_an_election_is_observable_on_every_node", ((True, False, False), (1, 1, 1), (0, 0, 0))),
)


def cluster(n: int) -> dict[int, RaftNode]:
    m = auto_membership(n)
    return {i: new_node(i, m) for i in range(n)}


def pump(nodes: dict[int, RaftNode], rounds: int = 40) -> None:
    for _ in range(rounds):
        traffic = [(s, o) for s in sorted(nodes) for o in nodes[s].take_outgoing()]
        if not traffic:
            return
        for sender, out in traffic:
            if out.to in nodes:
                nodes[out.to].handle(sender, out.msg)


def elect(nodes: dict[int, RaftNode], leader: int = 0) -> None:
    for _ in range(election_timeout_for(leader)):
        nodes[leader].tick()
    pump(nodes)


def leader_of(n: int = 3, at: int = 0) -> dict[int, RaftNode]:
    nodes = cluster(n)
    elect(nodes, at)
    return nodes


def follower(n: int = 3, node_id: int = 0) -> RaftNode:
    return new_node(node_id, auto_membership(n))


def sent(node: RaftNode) -> tuple:
    return tuple(node.take_outgoing())


def verify_election_safety_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. membership derivation keeps the voter count odd at every group size
    exp1 = ELECTION_SAFETY_BEHAVIOR_MATRIX[0][1]
    shapes = tuple(
        (auto_membership(n).voters, auto_membership(n).learners) for n in range(0, 6)
    )
    obs1 = shapes
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a majority of an odd voter set is a strict one
    exp2 = ELECTION_SAFETY_BEHAVIOR_MATRIX[1][1]
    obs2 = tuple((majority(k) for k in (1, 3, 5, 7, 9)))
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a fresh node is a follower at term zero with no vote spent
    exp3 = ELECTION_SAFETY_BEHAVIOR_MATRIX[2][1]
    fresh = follower(3, 0)
    obs3 = (fresh.role.value, fresh.current_term, fresh.voted_for, fresh.leader_id, fresh.is_voter, fresh.last_index())
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. every node has its own election timeout so one voter always fires first
    exp4 = ELECTION_SAFETY_BEHAVIOR_MATRIX[3][1]
    obs4 = (ELECTION_MIN, HEARTBEAT_TIMEOUT, tuple((election_timeout_for(i) for i in range(5))))
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the election timer fires on the tick that reaches the timeout, not before
    exp5 = ELECTION_SAFETY_BEHAVIOR_MATRIX[4][1]
    clock = ElectionClock(election_timeout=3)
    before = clock.election_due()
    clock.tick()
    clock.tick()
    mid = clock.election_due()
    clock.tick()
    at = clock.election_due()
    clock.reset_election()
    after_reset = (clock.election_due(), clock.election_elapsed, clock.heartbeat_elapsed)
    obs5 = (before, mid, at, after_reset)
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a lone voter wins its own election immediately
    exp6 = ELECTION_SAFETY_BEHAVIOR_MATRIX[5][1]
    solo = cluster(1)
    elect(solo, 0)
    obs6 = (solo[0].role.value, solo[0].is_leader(), solo[0].current_term, solo[0].leader_id, solo[0].voted_for)
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. campaigning bumps the term, takes the node's own vote, and drops the leader
    exp7 = ELECTION_SAFETY_BEHAVIOR_MATRIX[6][1]
    cand = follower(3, 0)
    for _ in range(election_timeout_for(0)):
        cand.tick()
    out = sent(cand)
    obs7 = (cand.role.value, cand.current_term, cand.voted_for, cand.leader_id, tuple(sorted(cand.votes)), len(out), tuple((o.to for o in out)))
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the vote request carries the candidate's own last index and term
    exp8 = ELECTION_SAFETY_BEHAVIOR_MATRIX[7][1]
    ahead = follower(3, 0)
    ahead.log = [LogEntry(2, 1, b"x"), LogEntry(4, 2, b"y")]
    for _ in range(election_timeout_for(0)):
        ahead.tick()
    req = sent(ahead)[0].msg
    obs8 = (req.term, req.candidate, req.last_log_index, req.last_log_term)
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a majority of grants promotes the candidate
    exp9 = ELECTION_SAFETY_BEHAVIOR_MATRIX[8][1]
    nodes = leader_of(3, 0)
    obs9 = (nodes[0].role.value, nodes[0].current_term, nodes[0].leader_id, nodes[1].role.value, nodes[1].leader_id, nodes[2].leader_id)
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. taking office initialises every peer's cursors from the leader's own log
    exp10 = ELECTION_SAFETY_BEHAVIOR_MATRIX[9][1]
    promoted = follower(3, 0)
    promoted.log = [LogEntry(1, 1, b"a"), LogEntry(1, 2, b"b")]
    for _ in range(election_timeout_for(0)):
        promoted.tick()
    promoted.handle(1, VoteResp(term=1, granted=True))
    obs10 = (tuple(sorted(promoted.next_index.items())), tuple(sorted(promoted.match_index.items())), promoted.last_index(), promoted.role.value)
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a learner never campaigns no matter how long it waits
    exp11 = ELECTION_SAFETY_BEHAVIOR_MATRIX[10][1]
    learner_group = auto_membership(4)
    learner = new_node(3, learner_group)
    for _ in range(200):
        learner.tick()
    obs11 = (learner_group.learners, learner.is_voter, learner.role.value, learner.current_term, len(sent(learner)))
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a repeated request from the same candidate in the same term is idempotent
    exp12 = ELECTION_SAFETY_BEHAVIOR_MATRIX[11][1]
    voter = follower(3, 1)
    voter.handle(0, VoteReq(term=1, candidate=0, last_log_index=0, last_log_term=0))
    first = sent(voter)[0].msg.granted
    voter.handle(0, VoteReq(term=1, candidate=0, last_log_index=0, last_log_term=0))
    second = sent(voter)[0].msg.granted
    obs12 = (first, second, voter.voted_for, voter.current_term)
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. granting a vote restarts the voter's own election timer
    exp13 = ELECTION_SAFETY_BEHAVIOR_MATRIX[12][1]
    patient = follower(3, 1)
    patient.current_term = 1
    for _ in range(40):
        patient.tick()
    elapsed_before = patient.clock.election_elapsed
    patient.handle(0, VoteReq(term=1, candidate=0, last_log_index=0, last_log_term=0))
    obs13 = (elapsed_before, patient.clock.election_elapsed, patient.voted_for)
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    # 14. the term and the vote survive a restart
    exp14 = ELECTION_SAFETY_BEHAVIOR_MATRIX[13][1]
    persisted = leader_of(3, 0)[1].persisted()
    revived = from_persisted(1, auto_membership(3), persisted)
    blank = PersistedState()
    obs14 = (persisted.term, persisted.voted_for, revived.current_term, revived.voted_for, revived.role.value, blank.term, blank.voted_for, blank.commit_index, blank.snapshot_index)
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[13][0], "expected": exp14,
                   "observed": obs14, "passed": obs14 == exp14})

    # 15. a leader heartbeats on its own cadence rather than on the election timer
    exp15 = ELECTION_SAFETY_BEHAVIOR_MATRIX[14][1]
    beat = leader_of(3, 0)
    sent(beat[0])
    quiet = []
    for _ in range(HEARTBEAT_TIMEOUT * 2):
        beat[0].tick()
        quiet.append(len(sent(beat[0])))
    obs15 = (tuple(quiet), beat[0].role.value)
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[14][0], "expected": exp15,
                   "observed": obs15, "passed": obs15 == exp15})

    # 16. the outcome of an election is observable on every node in the group
    exp16 = ELECTION_SAFETY_BEHAVIOR_MATRIX[15][1]
    seen = leader_of(3, 0)
    obs16 = (tuple((seen[i].is_leader() for i in range(3))), tuple((seen[i].current_term for i in range(3))), tuple((seen[i].leader_id for i in range(3))))
    checks.append({"name": ELECTION_SAFETY_BEHAVIOR_MATRIX[15][0], "expected": exp16,
                   "observed": obs16, "passed": obs16 == exp16})

    return {
        "case_id": "election-safety-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
