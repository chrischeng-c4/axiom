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
    InstallSnapshotReq,
    InstallSnapshotResp,
)
from raft_core.infrastructure.persistence import PersistedState

MINIMUM_CHECKS = 13

SNAPSHOT_COMPACTION_SECURITY_MATRIX = (
    ("compaction_refuses_to_run_past_what_has_actually_been_applied", (0, '', 4, 4)),
    ("compaction_stops_exactly_at_the_applied_cursor", (2, 'OK', 2, 4)),
    ("compaction_refuses_to_move_backwards_or_to_stand_still", ((3, 'S3'), 3, 'S3', 1)),
    ("no_index_is_renumbered_by_compaction_at_any_compaction_point", ((4, 4, 4), 4, 0)),
    ("an_older_snapshot_does_not_truncate_a_follower_that_is_ahead", (5, 'MINE', 2, None, 7)),
    ("a_snapshot_at_the_point_already_held_changes_nothing", (5, 'MINE', None, 0, 0, 0)),
    ("a_snapshot_from_a_stale_term_is_answered_without_being_installed", (0, '', None, None, 9, 0)),
    ("the_driver_is_never_handed_the_same_snapshot_twice", (None, None, 'NEW')),
    ("a_higher_term_in_a_snapshot_response_drops_leadership", ('follower', 9, None, 2)),
    ("a_node_that_is_not_leader_ignores_a_snapshot_response", ('follower', {}, {})),
    ("a_snapshot_response_from_an_older_term_is_ignored", ((2, 3), 2, 3, 'leader')),
    ("a_restart_never_resumes_below_the_compaction_point", (5, 5, 5, 6)),
    ("a_restart_never_claims_to_have_committed_more_than_it_holds", (6, 6, 0, 0, None, None, 3)),
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


def verify_snapshot_compaction_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. compaction refuses to run past what has actually been applied
    exp1 = SNAPSHOT_COMPACTION_SECURITY_MATRIX[0][1]
    node = compactable()
    node.last_applied = 2
    node.compact(3, b"TOO FAR")
    obs1 = (node.snapshot_index, text(node.snapshot), len(node.log), node.last_index())
    checks.append({"name": SNAPSHOT_COMPACTION_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. compaction stops exactly at the applied cursor
    exp2 = SNAPSHOT_COMPACTION_SECURITY_MATRIX[1][1]
    edge = compactable()
    edge.last_applied = 2
    edge.compact(2, b"OK")
    obs2 = (edge.snapshot_index, text(edge.snapshot), len(edge.log), edge.last_index())
    checks.append({"name": SNAPSHOT_COMPACTION_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. compaction refuses to move backwards or to stand still
    exp3 = SNAPSHOT_COMPACTION_SECURITY_MATRIX[2][1]
    back = compactable()
    back.compact(3, b"S3")
    back.compact(2, b"BACK")
    lower = (back.snapshot_index, text(back.snapshot))
    back.compact(3, b"SAME")
    obs3 = (lower, back.snapshot_index, text(back.snapshot), len(back.log))
    checks.append({"name": SNAPSHOT_COMPACTION_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. no index is renumbered by compaction, at any compaction point
    exp4 = SNAPSHOT_COMPACTION_SECURITY_MATRIX[3][1]
    stable = compactable()
    marks = []
    stable.compact(1, b"a")
    marks.append(stable.last_index())
    stable.compact(2, b"b")
    marks.append(stable.last_index())
    stable.compact(4, b"d")
    marks.append(stable.last_index())
    obs4 = (tuple(marks), stable.snapshot_index, len(stable.log))
    checks.append({"name": SNAPSHOT_COMPACTION_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. an older snapshot does not truncate a follower that is ahead
    exp5 = SNAPSHOT_COMPACTION_SECURITY_MATRIX[4][1]
    ahead = follower(3, 1)
    ahead.snapshot_index = 5
    ahead.snapshot_term = 3
    ahead.snapshot = b"MINE"
    ahead.log = [LogEntry(4, 6, b"f"), LogEntry(4, 7, b"g")]
    ahead.current_term = 4
    ahead.handle(
        0,
        InstallSnapshotReq(
            term=4, leader=0, snapshot_index=2, snapshot_term=1, data=b"OLD"
        ),
    )
    obs5 = (ahead.snapshot_index, text(ahead.snapshot), len(ahead.log), text(ahead.installed_snapshot), ahead.last_index())
    checks.append({"name": SNAPSHOT_COMPACTION_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a snapshot at the point already held changes nothing
    exp6 = SNAPSHOT_COMPACTION_SECURITY_MATRIX[5][1]
    equal = follower(3, 1)
    equal.snapshot_index = 5
    equal.snapshot_term = 3
    equal.snapshot = b"MINE"
    equal.current_term = 4
    equal.handle(
        0,
        InstallSnapshotReq(
            term=4, leader=0, snapshot_index=5, snapshot_term=3, data=b"AGAIN"
        ),
    )
    obs6 = (equal.snapshot_index, text(equal.snapshot), text(equal.installed_snapshot), equal.leader_id, equal.commit_index, equal.last_applied)
    checks.append({"name": SNAPSHOT_COMPACTION_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a snapshot from a stale term is answered without being installed
    exp7 = SNAPSHOT_COMPACTION_SECURITY_MATRIX[6][1]
    stale = follower(3, 1)
    stale.current_term = 9
    stale.handle(
        0,
        InstallSnapshotReq(
            term=4, leader=0, snapshot_index=5, snapshot_term=3, data=b"STALE"
        ),
    )
    out = sent(stale)
    obs7 = (stale.snapshot_index, text(stale.snapshot), text(stale.installed_snapshot), stale.leader_id, out[0].msg.term, out[0].msg.snapshot_index)
    checks.append({"name": SNAPSHOT_COMPACTION_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the driver is never handed the same snapshot twice
    exp8 = SNAPSHOT_COMPACTION_SECURITY_MATRIX[7][1]
    single = follower(3, 1)
    single.handle(
        0,
        InstallSnapshotReq(
            term=4, leader=0, snapshot_index=5, snapshot_term=3, data=b"NEW"
        ),
    )
    single.take_installed_snapshot()
    obs8 = (text(single.take_installed_snapshot()), text(single.take_installed_snapshot()), text(single.snapshot))
    checks.append({"name": SNAPSHOT_COMPACTION_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a higher term in a snapshot response drops leadership
    exp9 = SNAPSHOT_COMPACTION_SECURITY_MATRIX[8][1]
    boss = compactable_leader()
    boss.match_index[1] = 2
    boss.handle(1, InstallSnapshotResp(term=9, snapshot_index=4))
    obs9 = (boss.role.value, boss.current_term, boss.voted_for, boss.match_index[1])
    checks.append({"name": SNAPSHOT_COMPACTION_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a node that is not leader ignores a snapshot response
    exp10 = SNAPSHOT_COMPACTION_SECURITY_MATRIX[9][1]
    bystander = follower(3, 0)
    bystander.current_term = 1
    bystander.handle(1, InstallSnapshotResp(term=1, snapshot_index=5))
    obs10 = (bystander.role.value, dict(bystander.match_index), dict(bystander.next_index))
    checks.append({"name": SNAPSHOT_COMPACTION_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a snapshot response from an older term is ignored
    exp11 = SNAPSHOT_COMPACTION_SECURITY_MATRIX[10][1]
    old = compactable_leader()
    old.match_index[1] = 2
    old.next_index[1] = 3
    before = (old.match_index[1], old.next_index[1])
    old.handle(1, InstallSnapshotResp(term=4, snapshot_index=99))
    obs11 = (before, old.match_index[1], old.next_index[1], old.role.value)
    checks.append({"name": SNAPSHOT_COMPACTION_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a restart never resumes below the compaction point
    exp12 = SNAPSHOT_COMPACTION_SECURITY_MATRIX[11][1]
    low = from_persisted(
        1,
        auto_membership(3),
        PersistedState(
            term=4,
            log=(LogEntry(4, 6, b"f"),),
            commit_index=1,
            snapshot_index=5,
            snapshot_term=3,
            snapshot=b"KEPT",
        ),
    )
    obs12 = (low.commit_index, low.last_applied, low.snapshot_index, low.last_index())
    checks.append({"name": SNAPSHOT_COMPACTION_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. a restart never claims to have committed more than it holds
    exp13 = SNAPSHOT_COMPACTION_SECURITY_MATRIX[12][1]
    high = from_persisted(
        1,
        auto_membership(3),
        PersistedState(
            term=4,
            log=(LogEntry(4, 6, b"f"),),
            commit_index=99,
            snapshot_index=5,
            snapshot_term=3,
            snapshot=b"KEPT",
        ),
    )
    empty = from_persisted(
        1,
        auto_membership(3),
        PersistedState(term=4, commit_index=99),
    )
    bodiless = from_persisted(
        1,
        auto_membership(3),
        PersistedState(term=4, snapshot_index=3, snapshot_term=2),
    )
    obs13 = (high.commit_index, high.last_index(), empty.commit_index, empty.last_index(), text(empty.installed_snapshot), text(bodiless.installed_snapshot), bodiless.snapshot_index)
    checks.append({"name": SNAPSHOT_COMPACTION_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    return {
        "case_id": "snapshot-compaction-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
