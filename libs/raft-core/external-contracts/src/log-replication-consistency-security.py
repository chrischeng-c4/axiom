from __future__ import annotations

from raft_core.application.node import (
    RaftNode,
    new_node,
)
from raft_core.application.timing import election_timeout_for
from raft_core.domain.commit_rule import highest_committed
from raft_core.domain.entry import LogEntry
from raft_core.domain.ids import Role
from raft_core.domain.log_view import (
    LogView,
    prev_entry_matches,
)
from raft_core.domain.membership import auto_membership
from raft_core.infrastructure.messages import (
    AppendReq,
    AppendResp,
)

MINIMUM_CHECKS = 15

LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX = (
    ("an_append_whose_preceding_entry_disagrees_is_refused_not_applied", (False, 1, 2, ((1, 1, 'a'), (1, 2, 'b')))),
    ("the_match_check_refuses_a_prefix_beyond_the_end_of_the_log", (False, True, False, True)),
    ("a_divergent_suffix_is_discarded_rather_than_interleaved", (((1, 1, 'a'), (5, 2, 'new')), 2, True)),
    ("an_entry_below_the_compaction_point_is_never_grafted_back_on", (((4, 4, 'd'),), 3, 2, 4)),
    ("an_entry_from_an_earlier_term_is_not_committed_by_count_alone", (0, 2, 1)),
    ("the_commit_point_never_walks_backwards", (2, 1, 0)),
    ("a_delayed_success_response_never_walks_progress_backwards", ((3, 6), 3, 6)),
    ("a_stale_refusal_for_an_already_replicated_prefix_is_discarded", ((3, 4), 3, 4, 2)),
    ("retreating_steps_back_but_never_below_the_first_real_index", (3, 1)),
    ("an_append_carrying_an_older_term_is_refused_without_touching_the_log", (False, 7, 0, ((7, 1, 'a'),), 0, None)),
    ("a_node_that_is_not_leader_ignores_a_replication_response", ('follower', {}, {}, 0)),
    ("a_response_from_an_older_term_is_ignored_by_the_leader", (0, 0, 'leader', 1)),
    ("committing_is_a_leader_only_act", (0, 'follower', 1)),
    ("a_followers_commit_point_never_runs_past_the_entries_it_holds", (1, 1, ((1, 1, 'a'),), 1)),
    ("only_the_conflicting_suffix_is_dropped_never_the_agreed_prefix", (((1, 1, 'a'), (1, 2, 'b'), (6, 3, 'C')), 3, 3)),
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


def verify_log_replication_consistency_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an append whose preceding entry disagrees in term is refused, not applied
    exp1 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[0][1]
    conflict = follower(3, 1)
    conflict.current_term = 3
    conflict.log = [LogEntry(1, 1, b"a"), LogEntry(1, 2, b"b")]
    conflict.handle(
        0,
        AppendReq(
            term=3,
            leader=0,
            prev_log_index=2,
            prev_log_term=2,
            entries=(LogEntry(3, 3, b"z"),),
            leader_commit=0,
        ),
    )
    resp = sent(conflict)[0].msg
    obs1 = (resp.success, resp.match_index, len(conflict.log), tuple((entry_tuple(e) for e in conflict.log)))
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the match check refuses a prefix beyond the end of the follower's log
    exp2 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[1][1]
    view = LogView(0, 0, (LogEntry(1, 1, b"a"), LogEntry(2, 2, b"b")))
    obs2 = (prev_entry_matches(view, 3, 2), prev_entry_matches(view, 2, 2), prev_entry_matches(view, 2, 1), prev_entry_matches(view, 1, 1))
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a divergent suffix is discarded rather than interleaved
    exp3 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[2][1]
    forked = follower(3, 1)
    forked.current_term = 5
    forked.log = [
        LogEntry(1, 1, b"a"),
        LogEntry(2, 2, b"old"),
        LogEntry(2, 3, b"older"),
    ]
    forked.handle(
        0,
        AppendReq(
            term=5,
            leader=0,
            prev_log_index=1,
            prev_log_term=1,
            entries=(LogEntry(5, 2, b"new"),),
            leader_commit=0,
        ),
    )
    obs3 = (tuple((entry_tuple(e) for e in forked.log)), forked.last_index(), sent(forked)[0].msg.success)
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an entry below the compaction point is never grafted back on
    exp4 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[3][1]
    compacted = follower(3, 1)
    compacted.current_term = 4
    compacted.snapshot_index = 3
    compacted.snapshot_term = 2
    compacted.log = [LogEntry(4, 4, b"d")]
    compacted.handle(
        0,
        AppendReq(
            term=4,
            leader=0,
            prev_log_index=3,
            prev_log_term=2,
            entries=(LogEntry(4, 4, b"d"), LogEntry(9, 3, b"forged")),
            leader_commit=0,
        ),
    )
    obs4 = (tuple((entry_tuple(e) for e in compacted.log)), compacted.snapshot_index, compacted.term_at(3), compacted.last_index())
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. an entry from an earlier term is not committed by majority count alone
    exp5 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[4][1]
    view5 = LogView(0, 0, (LogEntry(1, 1, b"a"), LogEntry(2, 2, b"b")))
    obs5 = (highest_committed(view5, (0, 1, 2), {1: 1, 2: 1}, 0, 2, 0), highest_committed(view5, (0, 1, 2), {1: 2, 2: 1}, 0, 2, 0), highest_committed(view5, (0, 1, 2), {1: 1, 2: 1}, 0, 1, 0))
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the commit point never walks backwards
    exp6 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[5][1]
    view6 = LogView(0, 0, (LogEntry(1, 1, b"a"), LogEntry(1, 2, b"b")))
    obs6 = (highest_committed(view6, (0, 1, 2), {}, 0, 1, 2), highest_committed(view6, (0, 1, 2), {1: 0, 2: 0}, 0, 1, 1), highest_committed(view6, (0, 1, 2), {1: 2, 2: 2}, 0, 9, 0))
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a delayed success response never walks a peer's progress backwards
    exp7 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[6][1]
    boss = leader_of(3, 0)[0]
    boss.propose(b"a")
    boss.propose(b"b")
    boss.propose(b"c")
    sent(boss)
    boss.handle(1, AppendResp(term=1, success=True, match_index=3))
    boss.next_index[1] = 6
    high = (boss.match_index[1], boss.next_index[1])
    boss.handle(1, AppendResp(term=1, success=True, match_index=1))
    obs7 = (high, boss.match_index[1], boss.next_index[1])
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a stale refusal for a prefix already proved replicated is discarded
    exp8 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[7][1]
    proved = leader_of(3, 0)[0]
    proved.propose(b"a")
    proved.propose(b"b")
    proved.propose(b"c")
    sent(proved)
    proved.handle(1, AppendResp(term=1, success=True, match_index=3))
    before = (proved.match_index[1], proved.next_index[1])
    proved.handle(1, AppendResp(term=1, success=False, match_index=1))
    obs8 = (before, proved.match_index[1], proved.next_index[1], len(sent(proved)))
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. retreating never takes a peer's next index below the first real index
    exp9 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[8][1]
    floor = leader_of(3, 0)[0]
    for command in (b"a", b"b", b"c", b"d"):
        floor.propose(command)
    floor.next_index[1] = 5
    sent(floor)
    floor.handle(1, AppendResp(term=1, success=False, match_index=2))
    mid = floor.next_index[1]
    floor.next_index[1] = 1
    floor.match_index[1] = 0
    sent(floor)
    floor.handle(1, AppendResp(term=1, success=False, match_index=0))
    obs9 = (mid, floor.next_index[1])
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. an append carrying an older term is refused without touching the log
    exp10 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[9][1]
    settled = follower(3, 1)
    settled.current_term = 7
    settled.log = [LogEntry(7, 1, b"a")]
    settled.handle(
        0,
        AppendReq(
            term=3,
            leader=0,
            prev_log_index=0,
            prev_log_term=0,
            entries=(LogEntry(3, 1, b"stale"),),
            leader_commit=5,
        ),
    )
    resp10 = sent(settled)[0].msg
    obs10 = (resp10.success, resp10.term, resp10.match_index, tuple((entry_tuple(e) for e in settled.log)), settled.commit_index, settled.leader_id)
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a node that is not leader ignores a replication response entirely
    exp11 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[10][1]
    bystander = follower(3, 0)
    bystander.current_term = 1
    bystander.handle(1, AppendResp(term=1, success=True, match_index=5))
    obs11 = (bystander.role.value, dict(bystander.match_index), dict(bystander.next_index), bystander.commit_index)
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a response from an older term is ignored by the leader
    exp12 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[11][1]
    boss12 = leader_of(3, 0)[0]
    boss12.propose(b"a")
    sent(boss12)
    boss12.handle(1, AppendResp(term=0, success=True, match_index=1))
    obs12 = (boss12.match_index.get(1, 0), boss12.commit_index, boss12.role.value, boss12.current_term)
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. committing is a leader-only act
    exp13 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[12][1]
    demoted = leader_of(3, 0)[0]
    demoted.propose(b"a")
    demoted.match_index = {1: 1, 2: 1}
    demoted.role = Role.FOLLOWER
    demoted._maybe_commit()
    obs13 = (demoted.commit_index, demoted.role.value, demoted.last_index())
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    # 14. a follower's commit point never runs past the entries it actually holds
    exp14 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[13][1]
    modest = follower(3, 1)
    modest.handle(
        0,
        AppendReq(
            term=1,
            leader=0,
            prev_log_index=0,
            prev_log_term=0,
            entries=(LogEntry(1, 1, b"a"),),
            leader_commit=1000,
        ),
    )
    delivered = tuple(entry_tuple(e) for e in modest.take_committed())
    obs14 = (modest.commit_index, modest.last_index(), delivered, modest.last_applied)
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[13][0], "expected": exp14,
                   "observed": obs14, "passed": obs14 == exp14})

    # 15. only the conflicting suffix is dropped, never the agreed prefix
    exp15 = LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[14][1]
    trimmed = follower(3, 1)
    trimmed.current_term = 6
    trimmed.log = [
        LogEntry(1, 1, b"a"),
        LogEntry(1, 2, b"b"),
        LogEntry(4, 3, b"c"),
        LogEntry(4, 4, b"d"),
    ]
    trimmed.handle(
        0,
        AppendReq(
            term=6,
            leader=0,
            prev_log_index=2,
            prev_log_term=1,
            entries=(LogEntry(6, 3, b"C"),),
            leader_commit=0,
        ),
    )
    obs15 = (tuple((entry_tuple(e) for e in trimmed.log)), trimmed.last_index(), sent(trimmed)[0].msg.match_index)
    checks.append({"name": LOG_REPLICATION_CONSISTENCY_SECURITY_MATRIX[14][0], "expected": exp15,
                   "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "log-replication-consistency-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
