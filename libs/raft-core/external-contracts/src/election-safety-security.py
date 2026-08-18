from __future__ import annotations

from raft_core.application.node import (
    RaftNode,
    new_node,
)
from raft_core.application.timing import election_timeout_for
from raft_core.domain.election_rules import (
    is_up_to_date,
    vote_granted,
)
from raft_core.domain.entry import LogEntry
from raft_core.domain.ids import Role
from raft_core.domain.membership import (
    auto_membership,
    majority,
)
from raft_core.infrastructure.messages import (
    AppendReq,
    AppendResp,
    VoteReq,
    VoteResp,
)

MINIMUM_CHECKS = 14

ELECTION_SAFETY_SECURITY_MATRIX = (
    ("the_second_candidate_in_a_term_is_refused", (True, False, 0)),
    ("the_up_to_date_comparison_ranks_term_first_and_length_second", (True, False, True, True, False)),
    ("a_candidate_whose_log_is_behind_is_refused_by_an_unspent_voter", (False, False, None, None)),
    ("the_grant_rule_refuses_a_request_outside_the_voters_current_term", (False, False, True, False, False)),
    ("a_request_carrying_a_lower_term_is_refused_without_moving_the_voter", (False, 4, 4, 2)),
    ("a_higher_term_in_a_vote_request_releases_the_vote_before_the_grant", (True, 9, 0, 'follower')),
    ("a_higher_term_in_a_vote_response_ends_the_campaign_without_counting_it", ('follower', 7, None, (0,), 'follower', 3, None, (0,))),
    ("a_higher_term_in_a_replication_message_drops_leadership", (('follower', 9, None, 1), ('follower', 9, None))),
    ("stepping_down_at_an_equal_term_keeps_the_vote_already_spent", ('follower', 4, 2, 2)),
    ("only_members_of_the_voter_set_are_counted_towards_a_majority", (5, 3, 'candidate', 'leader')),
    ("a_refused_vote_is_still_answered_so_no_candidate_is_left_waiting", (1, 0, False, 1)),
    ("a_candidate_ignores_a_grant_carried_by_an_older_term", ('candidate', (0,), 5)),
    ("a_learner_is_outside_the_voter_set_at_every_group_size", ((1, (1,), False), (1, (3,), False), (1, (5,), False), (1, (7,), False))),
    ("a_leader_that_has_stepped_down_stops_committing", ('follower', 9, 0)),
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


def verify_election_safety_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the second candidate in a term is refused because the vote is already spent
    exp1 = ELECTION_SAFETY_SECURITY_MATRIX[0][1]
    voter = follower(3, 1)
    voter.handle(0, VoteReq(term=1, candidate=0, last_log_index=0, last_log_term=0))
    first = sent(voter)[0].msg.granted
    voter.handle(2, VoteReq(term=1, candidate=2, last_log_index=0, last_log_term=0))
    second = sent(voter)[0].msg.granted
    obs1 = (first, second, voter.voted_for)
    checks.append({"name": ELECTION_SAFETY_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the up-to-date comparison ranks term first and length second
    exp2 = ELECTION_SAFETY_SECURITY_MATRIX[1][1]
    obs2 = (is_up_to_date(3, 1, 2, 99), is_up_to_date(1, 99, 2, 1), is_up_to_date(2, 5, 2, 5), is_up_to_date(2, 6, 2, 5), is_up_to_date(2, 4, 2, 5))
    checks.append({"name": ELECTION_SAFETY_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a candidate whose log is behind is refused even by a voter that has not voted
    exp3 = ELECTION_SAFETY_SECURITY_MATRIX[2][1]
    stale_term = follower(3, 1)
    stale_term.current_term = 5
    stale_term.log = [LogEntry(3, 1, b"a"), LogEntry(3, 2, b"b")]
    stale_term.handle(0, VoteReq(term=5, candidate=0, last_log_index=9, last_log_term=2))
    by_term = sent(stale_term)[0].msg.granted

    short_log = follower(3, 1)
    short_log.current_term = 5
    short_log.log = [LogEntry(3, 1, b"a"), LogEntry(3, 2, b"b")]
    short_log.handle(0, VoteReq(term=5, candidate=0, last_log_index=1, last_log_term=3))
    by_length = sent(short_log)[0].msg.granted
    obs3 = (by_term, by_length, stale_term.voted_for, short_log.voted_for)
    checks.append({"name": ELECTION_SAFETY_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the grant rule refuses a request that is not for the voter's current term
    exp4 = ELECTION_SAFETY_SECURITY_MATRIX[3][1]
    obs4 = (vote_granted(1, 2, None, 0, True), vote_granted(3, 2, None, 0, True), vote_granted(2, 2, None, 0, True), vote_granted(2, 2, 1, 0, True), vote_granted(2, 2, None, 0, False))
    checks.append({"name": ELECTION_SAFETY_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a request carrying a lower term is refused and does not move the voter
    exp5 = ELECTION_SAFETY_SECURITY_MATRIX[4][1]
    settled = follower(3, 1)
    settled.current_term = 4
    settled.voted_for = 2
    settled.handle(0, VoteReq(term=2, candidate=0, last_log_index=0, last_log_term=0))
    resp = sent(settled)[0].msg
    obs5 = (resp.granted, resp.term, settled.current_term, settled.voted_for)
    checks.append({"name": ELECTION_SAFETY_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a higher term in a vote request releases the vote before the grant is decided
    exp6 = ELECTION_SAFETY_SECURITY_MATRIX[5][1]
    spent = follower(3, 1)
    spent.current_term = 4
    spent.voted_for = 2
    spent.role = Role.CANDIDATE
    spent.handle(0, VoteReq(term=9, candidate=0, last_log_index=0, last_log_term=0))
    granted = sent(spent)[0].msg.granted
    obs6 = (granted, spent.current_term, spent.voted_for, spent.role.value)
    checks.append({"name": ELECTION_SAFETY_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a higher term in a vote response ends the campaign without counting it
    exp7 = ELECTION_SAFETY_SECURITY_MATRIX[6][1]
    campaigner = follower(3, 0)
    campaigner.role = Role.CANDIDATE
    campaigner.current_term = 2
    campaigner.voted_for = 0
    campaigner.votes = {0}
    campaigner.handle(1, VoteResp(term=7, granted=True))
    nudged = follower(3, 0)
    nudged.role = Role.CANDIDATE
    nudged.current_term = 2
    nudged.voted_for = 0
    nudged.votes = {0}
    nudged.handle(1, VoteResp(term=3, granted=True))
    obs7 = (campaigner.role.value, campaigner.current_term, campaigner.voted_for, tuple(sorted(campaigner.votes)), nudged.role.value, nudged.current_term, nudged.voted_for, tuple(sorted(nudged.votes)))
    checks.append({"name": ELECTION_SAFETY_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a higher term in a replication message drops leadership before it is acted on
    exp8 = ELECTION_SAFETY_SECURITY_MATRIX[7][1]
    nodes = leader_of(3, 0)
    boss = nodes[0]
    sent(boss)
    boss.handle(
        1,
        AppendReq(
            term=9,
            leader=1,
            prev_log_index=0,
            prev_log_term=0,
            entries=(),
            leader_commit=0,
        ),
    )
    by_request = (boss.role.value, boss.current_term, boss.voted_for, boss.leader_id)

    other = leader_of(3, 0)[0]
    sent(other)
    other.handle(1, AppendResp(term=9, success=False, match_index=0))
    by_response = (other.role.value, other.current_term, other.voted_for)
    obs8 = (by_request, by_response)
    checks.append({"name": ELECTION_SAFETY_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. stepping down at an equal term keeps the vote already spent
    exp9 = ELECTION_SAFETY_SECURITY_MATRIX[8][1]
    equal = follower(3, 1)
    equal.current_term = 4
    equal.voted_for = 2
    equal.role = Role.CANDIDATE
    equal.handle(
        2,
        AppendReq(
            term=4,
            leader=2,
            prev_log_index=0,
            prev_log_term=0,
            entries=(),
            leader_commit=0,
        ),
    )
    obs9 = (equal.role.value, equal.current_term, equal.voted_for, equal.leader_id)
    checks.append({"name": ELECTION_SAFETY_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. only members of the voter set are counted towards a majority
    exp10 = ELECTION_SAFETY_SECURITY_MATRIX[9][1]
    counted = follower(5, 0)
    counted.role = Role.CANDIDATE
    counted.current_term = 1
    counted.votes = {0, 7, 8, 9}
    counted.handle(7, VoteResp(term=1, granted=True))
    with_outsiders = counted.role.value
    counted.handle(1, VoteResp(term=1, granted=True))
    counted.handle(2, VoteResp(term=1, granted=True))
    obs10 = (len(counted.voters), majority(len(counted.voters)), with_outsiders, counted.role.value)
    checks.append({"name": ELECTION_SAFETY_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a refused vote is still answered, so a candidate is never left waiting
    exp11 = ELECTION_SAFETY_SECURITY_MATRIX[10][1]
    refuser = follower(3, 1)
    refuser.current_term = 1
    refuser.voted_for = 2
    refuser.handle(0, VoteReq(term=1, candidate=0, last_log_index=0, last_log_term=0))
    out = sent(refuser)
    obs11 = (len(out), out[0].to, out[0].msg.granted, out[0].msg.term)
    checks.append({"name": ELECTION_SAFETY_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a candidate ignores a grant carried by an older term
    exp12 = ELECTION_SAFETY_SECURITY_MATRIX[11][1]
    old = follower(3, 0)
    old.role = Role.CANDIDATE
    old.current_term = 5
    old.votes = {0}
    old.handle(1, VoteResp(term=3, granted=True))
    obs12 = (old.role.value, tuple(sorted(old.votes)), old.current_term)
    checks.append({"name": ELECTION_SAFETY_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. a learner is outside the voter set at every group size that has one
    exp13 = ELECTION_SAFETY_SECURITY_MATRIX[12][1]
    obs13 = tuple(((len(auto_membership(n).voters) % 2, auto_membership(n).learners, n in auto_membership(n).voters) for n in (2, 4, 6, 8)))
    checks.append({"name": ELECTION_SAFETY_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    # 14. a leader that has stepped down stops committing
    exp14 = ELECTION_SAFETY_SECURITY_MATRIX[13][1]
    demoted = leader_of(3, 0)[0]
    demoted.log = [LogEntry(1, 1, b"a")]
    demoted.match_index = {1: 1, 2: 1}
    demoted.handle(1, AppendResp(term=9, success=True, match_index=1))
    obs14 = (demoted.role.value, demoted.current_term, demoted.commit_index)
    checks.append({"name": ELECTION_SAFETY_SECURITY_MATRIX[13][0], "expected": exp14,
                   "observed": obs14, "passed": obs14 == exp14})

    return {
        "case_id": "election-safety-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
