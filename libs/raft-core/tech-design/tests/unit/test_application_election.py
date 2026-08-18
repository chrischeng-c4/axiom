from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_core.application.node import new_node
from raft_core.domain.ids import Role
from raft_core.domain.membership import auto_membership
from raft_core.infrastructure.messages import VoteReq, VoteResp


class TestApplicationElection(unittest.TestCase):
    def test_sole_voter_becomes_leader_on_timeout(self) -> None:
        mem = auto_membership(1)
        node = new_node(0, mem)
        for _ in range(50):
            node.tick()
        self.assertEqual(node.role, Role.LEADER)
        self.assertEqual(node.current_term, 1)
        self.assertEqual(node.take_outgoing(), ())

    def test_candidate_sends_vote_reqs(self) -> None:
        mem = auto_membership(3)
        node = new_node(0, mem)
        for _ in range(50):
            node.tick()
        self.assertEqual(node.role, Role.CANDIDATE)
        self.assertEqual(node.current_term, 1)
        outgoing = node.take_outgoing()
        self.assertEqual(len(outgoing), 2)
        destinations = {out.to for out in outgoing}
        self.assertEqual(destinations, {1, 2})
        for out in outgoing:
            req = out.msg
            self.assertIsInstance(req, VoteReq)
            self.assertEqual(req.term, 1)
            self.assertEqual(req.candidate, 0)
            self.assertEqual(req.last_log_index, 0)
            self.assertEqual(req.last_log_term, 0)

    def test_granting_vote_resp_makes_leader(self) -> None:
        mem = auto_membership(3)
        node = new_node(0, mem)
        for _ in range(50):
            node.tick()
        node.take_outgoing()  # clear election outbox
        node.handle(1, VoteResp(term=1, granted=True))
        self.assertEqual(node.role, Role.LEADER)
        self.assertEqual(node.next_index, {1: 1, 2: 1})
        self.assertEqual(node.match_index, {1: 0, 2: 0})

    def test_second_vote_req_refused_and_answered(self) -> None:
        mem = auto_membership(3)
        node = new_node(0, mem)
        # candidate 1 asks for vote first
        node.handle(1, VoteReq(term=1, candidate=1, last_log_index=0, last_log_term=0))
        out1 = node.take_outgoing()
        self.assertEqual(len(out1), 1)
        self.assertIsInstance(out1[0].msg, VoteResp)
        self.assertTrue(out1[0].msg.granted)

        # candidate 2 asks for vote in same term
        node.handle(2, VoteReq(term=1, candidate=2, last_log_index=0, last_log_term=0))
        out2 = node.take_outgoing()
        self.assertEqual(len(out2), 1)
        self.assertIsInstance(out2[0].msg, VoteResp)
        self.assertFalse(out2[0].msg.granted)

    def test_candidate_behind_refused(self) -> None:
        mem = auto_membership(3)
        node = new_node(0, mem)
        node.current_term = 2
        node.snapshot_term = 2
        node.snapshot_index = 1
        node.handle(1, VoteReq(term=2, candidate=1, last_log_index=1, last_log_term=1))
        out = node.take_outgoing()
        self.assertEqual(len(out), 1)
        self.assertIsInstance(out[0].msg, VoteResp)
        self.assertFalse(out[0].msg.granted)

    def test_vote_resp_higher_term_steps_down(self) -> None:
        mem = auto_membership(3)
        node = new_node(0, mem)
        for _ in range(50):
            node.tick()
        self.assertEqual(node.role, Role.CANDIDATE)
        node.handle(1, VoteResp(term=5, granted=False))
        self.assertEqual(node.role, Role.FOLLOWER)
        self.assertEqual(node.current_term, 5)


if __name__ == "__main__":
    unittest.main()
