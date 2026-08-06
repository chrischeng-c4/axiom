from __future__ import annotations

from raft_core.application.node import (
    RaftNode,
    from_persisted,
    new_node,
)
from raft_core.application.timing import election_timeout_for
from raft_core.domain.entry import LogEntry
from raft_core.domain.membership import auto_membership
from raft_core.infrastructure.messages import (
    AppendReq,
    InstallSnapshotReq,
    InstallSnapshotResp,
)
from raft_core.infrastructure.persistence import PersistedState

MINIMUM_CHECKS = 14

SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX = (
    ("compaction_drops_the_applied_prefix_without_renumbering_anything", ((4, 4, 4), 2, 4, 2)),
    ("compaction_records_the_index_and_the_term_it_stopped_at", (3, 4, 'S3')),
    ("an_index_below_the_compaction_point_still_reads_back_its_term", (0, 4, 4, 5, 5)),
    ("the_surviving_entries_keep_their_original_indices", (((4, 3, 'c'), (5, 4, 'd')), (4, 3, 'c'), None, 0)),
    ("a_peer_whose_next_entry_was_compacted_away_is_sent_the_snapshot", ('InstallSnapshotReq', 5, 3, 4, 'SNAP', 0)),
    ("a_peer_still_inside_the_resident_log_is_sent_an_ordinary_append", ('AppendReq', 3, 4, ((5, 4, 'd'),))),
    ("a_strictly_newer_snapshot_replaces_the_followers_resident_log", (0, 5, 3, 5, 3)),
    ("installing_a_snapshot_carries_the_commit_and_applied_cursors_up", (5, 5, 4, 0, 'follower')),
    ("the_installed_bytes_are_surfaced_to_the_driver_exactly_once", ('NEW', None, 'NEW')),
    ("the_follower_answers_with_the_snapshot_point_it_now_holds", (1, 0, 'InstallSnapshotResp', 4, 5)),
    ("the_leader_resumes_ordinary_replication_from_the_snapshot_point", (3, 4, 1, 'AppendReq', 3, 4, ((5, 4, 'd'),))),
    ("a_follower_caught_up_by_snapshot_accepts_ordinary_appends_again", (True, 6, ((4, 6, 'f'),), 6)),
    ("a_restart_re_offers_the_snapshot_the_node_was_holding", (5, 3, 'KEPT', None, 6)),
    ("compaction_leaves_the_log_observable_through_the_same_counters", ((4, 4, 0), 0, 4, 4)),
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


def text(blob: bytes | None) -> str | None:
    """Every observation must survive json.dumps; raw bytes do not."""
    return None if blob is None else blob.decode()


def applied_log() -> list[LogEntry]:
    """Four applied entries spanning four different terms."""
    return [
        LogEntry(1, 1, b"a"),
        LogEntry(2, 2, b"b"),
        LogEntry(4, 3, b"c"),
        LogEntry(5, 4, b"d"),
    ]


def compactable() -> RaftNode:
    """A node holding four applied entries whose terms all differ."""
    node = follower(3, 0)
    node.current_term = 5
    node.log = applied_log()
    node.commit_index = 4
    node.last_applied = 4
    return node


def compactable_leader() -> RaftNode:
    """The same log, on a node that has actually won an election."""
    node = leader_of(3, 0)[0]
    node.current_term = 5
    node.log = applied_log()
    node.commit_index = 4
    node.last_applied = 4
    node.take_committed()
    node.take_outgoing()
    return node


def verify_snapshot_compaction_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. compaction drops the applied prefix without renumbering anything
    exp1 = SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[0][1]
    node = compactable()
    before = (len(node.log), node.last_index(), node.last_applied)
    node.compact(2, b"S2")
    obs1 = (before, len(node.log), node.last_index(), node.snapshot_index)
    checks.append({"name": SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. compaction records the index and the term it stopped at
    exp2 = SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[1][1]
    node2 = compactable()
    node2.compact(3, b"S3")
    obs2 = (node2.snapshot_index, node2.snapshot_term, text(node2.snapshot))
    checks.append({"name": SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. an index below the compaction point still reads back its own term
    exp3 = SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[2][1]
    node3 = compactable()
    node3.compact(3, b"S3")
    obs3 = (node3.term_at(0), node3.term_at(1), node3.term_at(3), node3.term_at(4), node3.last_term())
    checks.append({"name": SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the surviving entries keep their original indices
    exp4 = SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[3][1]
    node4 = compactable()
    node4.compact(2, b"S2")
    obs4 = (tuple((entry_tuple(e) for e in node4.log)), entry_tuple(node4.view().entry_at(3)), node4.view().entry_at(2), node4.view().position_of(3))
    checks.append({"name": SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a peer whose next entry has been compacted away is sent the snapshot
    exp5 = SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[4][1]
    ship = compactable_leader()
    ship.compact(3, b"SNAP")
    sent(ship)
    ship.next_index[1] = 3
    ship._send_append_to(1)
    msg = sent(ship)[0].msg
    obs5 = (type(msg).__name__, msg.term, msg.snapshot_index, msg.snapshot_term, text(msg.data), msg.leader)
    checks.append({"name": SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a peer still inside the resident log is sent an ordinary append
    exp6 = SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[5][1]
    ship6 = compactable_leader()
    ship6.compact(2, b"SNAP")
    sent(ship6)
    ship6.next_index[1] = 4
    ship6._send_append_to(1)
    msg6 = sent(ship6)[0].msg
    obs6 = (type(msg6).__name__, msg6.prev_log_index, msg6.prev_log_term, tuple((entry_tuple(e) for e in msg6.entries)))
    checks.append({"name": SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a strictly newer snapshot replaces the follower's resident log
    exp7 = SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[6][1]
    taker = follower(3, 1)
    taker.log = [LogEntry(1, 1, b"a"), LogEntry(1, 2, b"b")]
    taker.handle(
        0,
        InstallSnapshotReq(
            term=4, leader=0, snapshot_index=5, snapshot_term=3, data=b"NEW"
        ),
    )
    obs7 = (len(taker.log), taker.snapshot_index, taker.snapshot_term, taker.last_index(), taker.last_term())
    checks.append({"name": SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. installing a snapshot carries the commit and applied cursors up with it
    exp8 = SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[7][1]
    cursors = follower(3, 1)
    cursors.handle(
        0,
        InstallSnapshotReq(
            term=4, leader=0, snapshot_index=5, snapshot_term=3, data=b"NEW"
        ),
    )
    obs8 = (cursors.commit_index, cursors.last_applied, cursors.current_term, cursors.leader_id, cursors.role.value)
    checks.append({"name": SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the installed bytes are surfaced to the driver exactly once
    exp9 = SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[8][1]
    once = follower(3, 1)
    once.handle(
        0,
        InstallSnapshotReq(
            term=4, leader=0, snapshot_index=5, snapshot_term=3, data=b"NEW"
        ),
    )
    first = once.take_installed_snapshot()
    obs9 = (text(first), text(once.take_installed_snapshot()), text(once.snapshot))
    checks.append({"name": SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the follower answers with the snapshot point it now holds
    exp10 = SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[9][1]
    answer = follower(3, 1)
    answer.handle(
        0,
        InstallSnapshotReq(
            term=4, leader=0, snapshot_index=5, snapshot_term=3, data=b"NEW"
        ),
    )
    out = sent(answer)
    obs10 = (len(out), out[0].to, type(out[0].msg).__name__, out[0].msg.term, out[0].msg.snapshot_index)
    checks.append({"name": SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the leader resumes ordinary replication from the snapshot point
    exp11 = SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[10][1]
    resume = compactable_leader()
    resume.compact(3, b"SNAP")
    sent(resume)
    resume.handle(1, InstallSnapshotResp(term=5, snapshot_index=3))
    follow_up = sent(resume)
    obs11 = (resume.match_index[1], resume.next_index[1], len(follow_up), type(follow_up[0].msg).__name__, follow_up[0].msg.prev_log_index, follow_up[0].msg.prev_log_term, tuple((entry_tuple(e) for e in follow_up[0].msg.entries)))
    checks.append({"name": SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a follower caught up by snapshot accepts ordinary appends again
    exp12 = SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[11][1]
    caught = follower(3, 1)
    caught.handle(
        0,
        InstallSnapshotReq(
            term=4, leader=0, snapshot_index=5, snapshot_term=3, data=b"NEW"
        ),
    )
    sent(caught)
    caught.handle(
        0,
        AppendReq(
            term=4,
            leader=0,
            prev_log_index=5,
            prev_log_term=3,
            entries=(LogEntry(4, 6, b"f"),),
            leader_commit=6,
        ),
    )
    resp12 = sent(caught)[0].msg
    obs12 = (resp12.success, resp12.match_index, tuple((entry_tuple(e) for e in caught.log)), caught.commit_index)
    checks.append({"name": SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. a restart re-offers the snapshot the node was holding
    exp13 = SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[12][1]
    state = PersistedState(
        term=4,
        voted_for=2,
        log=(LogEntry(4, 6, b"f"),),
        commit_index=6,
        snapshot_index=5,
        snapshot_term=3,
        snapshot=b"KEPT",
    )
    revived = from_persisted(1, auto_membership(3), state)
    obs13 = (revived.snapshot_index, revived.snapshot_term, text(revived.take_installed_snapshot()), text(revived.take_installed_snapshot()), revived.last_index())
    checks.append({"name": SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    # 14. compaction leaves the log observable through the same three counters
    exp14 = SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[13][1]
    counters = compactable()
    wide = (len(counters.log), counters.last_index(), counters.snapshot_index)
    counters.compact(4, b"ALL")
    obs14 = (wide, len(counters.log), counters.last_index(), counters.snapshot_index)
    checks.append({"name": SNAPSHOT_COMPACTION_BEHAVIOR_MATRIX[13][0], "expected": exp14,
                   "observed": obs14, "passed": obs14 == exp14})

    return {
        "case_id": "snapshot-compaction-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
