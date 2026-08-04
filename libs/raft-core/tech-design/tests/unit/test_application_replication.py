from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_core.application.node import new_node
from raft_core.domain.entry import LogEntry
from raft_core.domain.membership import auto_membership
from raft_core.infrastructure.messages import AppendReq, AppendResp, VoteResp


class TestApplicationReplication(unittest.TestCase):
    def test_propose_on_follower_returns_none(self) -> None:
        node = new_node(0, auto_membership(3))
        res = node.propose(b"cmd")
        self.assertIsNone(res)
        self.assertEqual(len(node.log), 0)

    def test_propose_on_leader_broadcasts_append(self) -> None:
        node = new_node(0, auto_membership(3))
        for _ in range(50):
            node.tick()
        node.handle(1, VoteResp(term=1, granted=True))
        node.take_outgoing()  # clear leader transition messages

        idx = node.propose(b"data")
        self.assertEqual(idx, 1)
        outgoing = node.take_outgoing()
        self.assertEqual(len(outgoing), 2)
        for out in outgoing:
            self.assertIsInstance(out.msg, AppendReq)
            self.assertEqual(len(out.msg.entries), 1)

    def test_follower_rejects_append_beyond_log_with_backoff_hint(self) -> None:
        node = new_node(1, auto_membership(3))
        req = AppendReq(
            term=1, leader=0, prev_log_index=5, prev_log_term=1, entries=(), leader_commit=0
        )
        node.handle(0, req)
        outgoing = node.take_outgoing()
        self.assertEqual(len(outgoing), 1)
        resp = outgoing[0].msg
        self.assertIsInstance(resp, AppendResp)
        self.assertFalse(resp.success)
        self.assertEqual(resp.match_index, 0)

    def test_follower_truncates_conflicting_suffix(self) -> None:
        node = new_node(1, auto_membership(3))
        node.current_term = 1
        node.log = [
            LogEntry(term=1, index=1, command=b"1"),
            LogEntry(term=1, index=2, command=b"2"),
            LogEntry(term=1, index=3, command=b"3"),
        ]
        req = AppendReq(
            term=2,
            leader=0,
            prev_log_index=1,
            prev_log_term=1,
            entries=(LogEntry(term=2, index=2, command=b"new2"),),
            leader_commit=0,
        )
        node.handle(0, req)
        self.assertEqual(len(node.log), 2)
        self.assertEqual(node.log[1].term, 2)
        self.assertEqual(node.log[1].command, b"new2")
        out = node.take_outgoing()
        self.assertEqual(len(out), 1)
        self.assertTrue(out[0].msg.success)
        self.assertEqual(out[0].msg.match_index, 2)

    def test_follower_ignores_entry_at_or_below_snapshot(self) -> None:
        node = new_node(1, auto_membership(3))
        node.current_term = 1
        node.snapshot_index = 3
        node.snapshot_term = 1
        req = AppendReq(
            term=1,
            leader=0,
            prev_log_index=2,
            prev_log_term=1,
            entries=(LogEntry(term=1, index=3, command=b"old"),),
            leader_commit=0,
        )
        node.handle(0, req)
        self.assertEqual(len(node.log), 0)
        out = node.take_outgoing()
        self.assertTrue(out[0].msg.success)

    def test_commit_needs_majority(self) -> None:
        leader = new_node(0, auto_membership(3))
        for _ in range(50):
            leader.tick()
        leader.handle(1, VoteResp(term=1, granted=True))
        leader.take_outgoing()

        leader.propose(b"x")
        self.assertEqual(leader.commit_index, 0)

        leader.handle(1, AppendResp(term=1, success=True, match_index=1))
        self.assertEqual(leader.commit_index, 1)

    def test_stale_successful_append_resp_monotonic(self) -> None:
        leader = new_node(0, auto_membership(3))
        for _ in range(50):
            leader.tick()
        leader.handle(1, VoteResp(term=1, granted=True))
        leader.match_index[1] = 5
        leader.next_index[1] = 6

        leader.handle(1, AppendResp(term=1, success=True, match_index=3))
        self.assertEqual(leader.match_index[1], 5)
        self.assertEqual(leader.next_index[1], 6)

    def test_failing_append_resp_below_known_match_ignored(self) -> None:
        leader = new_node(0, auto_membership(3))
        for _ in range(50):
            leader.tick()
        leader.handle(1, VoteResp(term=1, granted=True))
        leader.match_index[1] = 5
        leader.next_index[1] = 6
        leader.take_outgoing()

        leader.handle(1, AppendResp(term=1, success=False, match_index=4))
        self.assertEqual(leader.match_index[1], 5)
        self.assertEqual(leader.next_index[1], 6)
        self.assertEqual(leader.take_outgoing(), ())

    def test_take_committed_advances_last_applied(self) -> None:
        leader = new_node(0, auto_membership(3))
        for _ in range(50):
            leader.tick()
        leader.handle(1, VoteResp(term=1, granted=True))

        leader.propose(b"x")
        leader.handle(1, AppendResp(term=1, success=True, match_index=1))
        self.assertEqual(leader.commit_index, 1)

        committed = leader.take_committed()
        self.assertEqual(len(committed), 1)
        self.assertEqual(committed[0].index, 1)
        self.assertEqual(committed[0].command, b"x")

        self.assertEqual(leader.take_committed(), ())


if __name__ == "__main__":
    unittest.main()
