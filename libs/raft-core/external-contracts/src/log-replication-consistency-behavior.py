from __future__ import annotations

from raft_core.application.node import (
    RaftNode,
    new_node,
)
from raft_core.application.timing import election_timeout_for
from raft_core.domain.commit_rule import highest_committed
from raft_core.domain.entry import LogEntry
from raft_core.domain.log_view import (
    LogView,
    backoff_hint,
    prev_entry_matches,
)
from raft_core.domain.membership import auto_membership
from raft_core.infrastructure.messages import (
    AppendReq,
    AppendResp,
)

MINIMUM_CHECKS = 17

LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX = (
    ("only_a_leader_accepts_a_proposal", (None, 0, 1, 1, 1)),
    ("successive_proposals_take_successive_indices_in_the_leaders_term", ((1, 2, 3), ((1, 1, 'a'), (1, 2, 'b'), (1, 3, 'c')))),
    ("a_proposal_is_broadcast_to_every_peer_at_once", (2, (1, 2), (1, 1))),
    ("draining_the_outbox_hands_each_message_over_exactly_once", (2, 0)),
    ("an_append_names_the_entry_the_follower_must_already_hold", (1, 0, 1, 1, ((1, 2, 'b'),), 0)),
    ("an_accepted_append_is_answered_with_the_index_now_matched", (True, 1, 2, ((1, 1, 'a'), (1, 2, 'b')))),
    ("a_follower_adopts_the_leaders_commit_point_clamped_to_what_it_holds", (1, 2, 2)),
    ("a_proposal_a_majority_holds_is_committed_on_every_node", ((1, 1, 1), (1, 1, 1), (1, 1, 1))),
    ("committed_entries_are_handed_over_in_index_order_and_exactly_once", (((1, 1, 'a'), (1, 2, 'b')), (), 2)),
    ("the_leader_counts_its_own_log_towards_the_majority", (1, 0, 1, 0)),
    ("a_success_response_advances_the_peers_cursors_together", (2, 3, 2)),
    ("a_refused_append_answers_with_the_index_to_retreat_to", (False, 2, 2)),
    ("the_retreat_hint_never_runs_past_either_end_of_the_log", (0, 0, 2, 2)),
    ("a_follower_that_has_fallen_behind_is_caught_up_by_retreat_and_retry", ((3, 3, 3), (4, 4, 4), ((1, 1, 'a'), (1, 2, 'b'), (1, 3, 'c'), (1, 4, 'd')), 4)),
    ("an_empty_prefix_always_matches_so_a_fresh_follower_can_be_filled", (True, True, False, 0, 0)),
    ("replaying_entries_a_follower_already_holds_changes_nothing", (((1, 1, 'a'), (1, 2, 'b')), ((1, 1, 'a'), (1, 2, 'b')), 2)),
    ("the_leaders_per_peer_cursors_show_how_far_each_follower_has_got", (((1, 2), (2, 2)), ((1, 3), (2, 3)), 2, (2, 2))),
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


def entry_tuple(e: LogEntry) -> tuple:
    return (e.term, e.index, e.command.decode())


def verify_log_replication_consistency_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. only a leader accepts a proposal
    exp1 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[0][1]
    quiet = follower(3, 1)
    refused = quiet.propose(b"a")
    boss = leader_of(3, 0)[0]
    accepted = boss.propose(b"a")
    obs1 = (refused, len(quiet.log), accepted, len(boss.log), boss.last_index())
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. successive proposals take successive indices in the leader's own term
    exp2 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[1][1]
    many = leader_of(3, 0)[0]
    idx = (many.propose(b"a"), many.propose(b"b"), many.propose(b"c"))
    obs2 = (idx, tuple((entry_tuple(e) for e in many.log)))
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a proposal is broadcast to every peer at once
    exp3 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[2][1]
    caster = leader_of(3, 0)[0]
    sent(caster)
    caster.propose(b"a")
    out = sent(caster)
    obs3 = (len(out), tuple((o.to for o in out)), tuple((len(o.msg.entries) for o in out)))
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. draining the outbox hands each message over exactly once
    exp4 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[3][1]
    drainer = leader_of(3, 0)[0]
    sent(drainer)
    drainer.propose(b"a")
    obs4 = (len(sent(drainer)), len(sent(drainer)))
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. an append request names the entry it expects the follower to already hold
    exp5 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[4][1]
    shaper = leader_of(3, 0)[0]
    shaper.propose(b"a")
    shaper.propose(b"b")
    sent(shaper)
    shaper.next_index[1] = 2
    shaper._send_append_to(1)
    req = sent(shaper)[0].msg
    obs5 = (req.term, req.leader, req.prev_log_index, req.prev_log_term, tuple((entry_tuple(e) for e in req.entries)), req.leader_commit)
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. an accepted append is answered with the index the follower now matches
    exp6 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[5][1]
    taker = follower(3, 1)
    taker.handle(
        0,
        AppendReq(
            term=1,
            leader=0,
            prev_log_index=0,
            prev_log_term=0,
            entries=(LogEntry(1, 1, b"a"), LogEntry(1, 2, b"b")),
            leader_commit=0,
        ),
    )
    resp = sent(taker)[0].msg
    obs6 = (resp.success, resp.term, resp.match_index, tuple((entry_tuple(e) for e in taker.log)))
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a follower adopts the leader's commit point, clamped to what it holds
    exp7 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[6][1]
    ahead = follower(3, 1)
    ahead.handle(
        0,
        AppendReq(
            term=1,
            leader=0,
            prev_log_index=0,
            prev_log_term=0,
            entries=(LogEntry(1, 1, b"a"),),
            leader_commit=9,
        ),
    )
    clamped = ahead.commit_index
    ahead.handle(
        0,
        AppendReq(
            term=1,
            leader=0,
            prev_log_index=1,
            prev_log_term=1,
            entries=(LogEntry(1, 2, b"b"),),
            leader_commit=2,
        ),
    )
    obs7 = (clamped, ahead.last_index(), ahead.commit_index)
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a proposal that a majority holds is committed on every node
    exp8 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[7][1]
    nodes = leader_of(3, 0)
    nodes[0].propose(b"a")
    pump(nodes)
    obs8 = (tuple((nodes[i].commit_index for i in range(3))), tuple((nodes[i].last_index() for i in range(3))), tuple((len(nodes[i].log) for i in range(3))))
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. committed entries are handed over in index order and exactly once
    exp9 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[8][1]
    ordered = leader_of(3, 0)
    ordered[0].propose(b"a")
    ordered[0].propose(b"b")
    pump(ordered)
    first = tuple(entry_tuple(e) for e in ordered[0].take_committed())
    again = tuple(entry_tuple(e) for e in ordered[0].take_committed())
    obs9 = (first, again, ordered[0].last_applied)
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the leader counts its own log towards the majority
    exp10 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[9][1]
    view = LogView(0, 0, (LogEntry(1, 1, b"a"),))
    obs10 = (highest_committed(view, (0, 1, 2), {1: 1, 2: 0}, 0, 1, 0), highest_committed(view, (0, 1, 2), {1: 0, 2: 0}, 0, 1, 0), highest_committed(view, (0, 1, 2), {1: 1, 2: 1}, 0, 1, 0), highest_committed(view, (0, 1, 2, 3, 4), {1: 1}, 0, 1, 0))
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a success response advances the peer's cursors together
    exp11 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[10][1]
    boss11 = leader_of(3, 0)[0]
    boss11.propose(b"a")
    boss11.propose(b"b")
    sent(boss11)
    boss11.handle(1, AppendResp(term=1, success=True, match_index=2))
    obs11 = (boss11.match_index[1], boss11.next_index[1], boss11.commit_index)
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a refused append answers with the index the leader should retreat to
    exp12 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[11][1]
    diverged = follower(3, 1)
    diverged.log = [LogEntry(1, 1, b"a"), LogEntry(1, 2, b"b")]
    diverged.current_term = 2
    diverged.handle(
        0,
        AppendReq(
            term=2,
            leader=0,
            prev_log_index=5,
            prev_log_term=2,
            entries=(),
            leader_commit=0,
        ),
    )
    resp12 = sent(diverged)[0].msg
    obs12 = (resp12.success, resp12.match_index, resp12.term)
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. the retreat hint never runs past either end of the follower's log
    exp13 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[12][1]
    view13 = LogView(0, 0, (LogEntry(1, 1, b"a"), LogEntry(1, 2, b"b")))
    obs13 = (backoff_hint(view13, 0), backoff_hint(view13, 1), backoff_hint(view13, 3), backoff_hint(view13, 99))
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    # 14. a follower that has fallen behind is caught up by retreat and retry
    exp14 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[13][1]
    behind = cluster(3)
    behind[0].log = [
        LogEntry(1, 1, b"a"),
        LogEntry(1, 2, b"b"),
        LogEntry(1, 3, b"c"),
    ]
    elect(behind, 0)
    filled_by_retry = tuple(len(behind[i].log) for i in range(3))
    behind[0].propose(b"d")
    pump(behind)
    obs14 = (filled_by_retry, tuple((len(behind[i].log) for i in range(3))), tuple((entry_tuple(e) for e in behind[1].log)), behind[0].commit_index)
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[13][0], "expected": exp14,
                   "observed": obs14, "passed": obs14 == exp14})

    # 15. an empty prefix always matches, so a fresh follower can be filled
    exp15 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[14][1]
    empty = LogView(0, 0, ())
    obs15 = (prev_entry_matches(empty, 0, 0), prev_entry_matches(empty, 0, 7), prev_entry_matches(empty, 1, 0), empty.last_index(), empty.last_term())
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[14][0], "expected": exp15,
                   "observed": obs15, "passed": obs15 == exp15})

    # 16. replaying entries a follower already holds changes nothing
    exp16 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[15][1]
    replayed = follower(3, 1)
    req16 = AppendReq(
        term=1,
        leader=0,
        prev_log_index=0,
        prev_log_term=0,
        entries=(LogEntry(1, 1, b"a"), LogEntry(1, 2, b"b")),
        leader_commit=0,
    )
    replayed.handle(0, req16)
    once = tuple(entry_tuple(e) for e in replayed.log)
    replayed.handle(0, req16)
    twice = tuple(entry_tuple(e) for e in replayed.log)
    obs16 = (once, twice, len(replayed.log))
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[15][0], "expected": exp16,
                   "observed": obs16, "passed": obs16 == exp16})

    # 17. the leader's per-peer cursors show how far each follower has got
    exp17 = LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[16][1]
    watched = leader_of(3, 0)
    watched[0].propose(b"a")
    watched[0].propose(b"b")
    pump(watched)
    obs17 = (tuple(sorted(watched[0].match_index.items())), tuple(sorted(watched[0].next_index.items())), watched[0].commit_index, tuple((watched[i].commit_index for i in (1, 2))))
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_BEHAVIOR_MATRIX[16][0], "expected": exp17,
                   "observed": obs17, "passed": obs17 == exp17})

    return {
        "case_id": "log-replication-consistency-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
