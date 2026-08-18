from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_core.application.node import from_persisted, new_node
from raft_core.domain.entry import LogEntry
from raft_core.domain.ids import Role
from raft_core.domain.membership import auto_membership
from raft_core.infrastructure.messages import (
    InstallSnapshotReq,
    InstallSnapshotResp,
    VoteResp,
)
from raft_core.infrastructure.persistence import PersistedState


class TestApplicationSnapshot(unittest.TestCase):
    def test_compact_bounds_no_op(self) -> None:
        node = new_node(0, auto_membership(3))
        node.log = [
            LogEntry(term=1, index=i, command=f"cmd{i}".encode()) for i in range(1, 6)
        ]
        node.last_applied = 3
        node.snapshot_index = 0

        node.compact(5, b"snap")
        self.assertEqual(node.snapshot_index, 0)
        self.assertEqual(len(node.log), 5)

        node.compact(0, b"snap")
        self.assertEqual(node.snapshot_index, 0)
        self.assertEqual(len(node.log), 5)

    def test_legal_compact(self) -> None:
        node = new_node(0, auto_membership(3))
        node.log = [
            LogEntry(term=1, index=i, command=f"cmd{i}".encode()) for i in range(1, 6)
        ]
        node.last_applied = 3
        node.snapshot_index = 0

        node.compact(3, b"snap3")
        self.assertEqual(node.snapshot_index, 3)
        self.assertEqual(node.snapshot_term, 1)
        self.assertEqual(len(node.log), 2)
        self.assertEqual(node.log[0].index, 4)
        self.assertEqual(node.last_index(), 5)

    def test_leader_sends_install_snapshot_when_next_index_behind_snapshot(
        self,
    ) -> None:
        leader = new_node(0, auto_membership(3))
        for _ in range(50):
            leader.tick()
        leader.handle(1, VoteResp(term=1, granted=True))
        leader.take_outgoing()

        leader.log = [
            LogEntry(term=1, index=i, command=f"cmd{i}".encode()) for i in range(1, 6)
        ]
        leader.last_applied = 4
        leader.compact(4, b"snap4")
        leader.next_index[1] = 3
        leader.take_outgoing()

        leader._send_append_to(1)
        out = leader.take_outgoing()
        self.assertEqual(len(out), 1)
        req = out[0].msg
        self.assertIsInstance(req, InstallSnapshotReq)
        self.assertEqual(req.snapshot_index, 4)
        self.assertEqual(req.data, b"snap4")

    def test_install_snapshot_older_term_ignored(self) -> None:
        follower = new_node(1, auto_membership(3))
        follower.current_term = 5
        follower.handle(
            0,
            InstallSnapshotReq(
                term=2, leader=0, snapshot_index=10, snapshot_term=2, data=b"old"
            ),
        )
        self.assertEqual(follower.snapshot_index, 0)
        out = follower.take_outgoing()
        self.assertEqual(len(out), 1)
        resp = out[0].msg
        self.assertIsInstance(resp, InstallSnapshotResp)
        self.assertEqual(resp.term, 5)

    def test_newer_install_snapshot_applied(self) -> None:
        follower = new_node(1, auto_membership(3))
        follower.current_term = 1
        follower.log = [
            LogEntry(term=1, index=1, command=b"1"),
            LogEntry(term=1, index=2, command=b"2"),
        ]
        follower.handle(
            0,
            InstallSnapshotReq(
                term=1, leader=0, snapshot_index=5, snapshot_term=1, data=b"snap5"
            ),
        )
        self.assertEqual(len(follower.log), 0)
        self.assertEqual(follower.snapshot_index, 5)
        self.assertEqual(follower.snapshot_term, 1)
        self.assertEqual(follower.commit_index, 5)
        self.assertEqual(follower.last_applied, 5)
        self.assertEqual(follower.last_index(), 5)

        self.assertEqual(follower.take_installed_snapshot(), b"snap5")
        self.assertIsNone(follower.take_installed_snapshot())

        out = follower.take_outgoing()
        self.assertEqual(len(out), 1)
        resp = out[0].msg
        self.assertIsInstance(resp, InstallSnapshotResp)
        self.assertEqual(resp.snapshot_index, 5)

    def test_from_persisted_clamping_and_state(self) -> None:
        mem = auto_membership(3)
        e5 = LogEntry(term=3, index=5, command=b"5")
        e6 = LogEntry(term=3, index=6, command=b"6")
        state = PersistedState(
            term=7,
            voted_for=2,
            log=(e5, e6),
            commit_index=99,
            snapshot_index=4,
            snapshot_term=3,
            snapshot=b"S",
        )
        node = from_persisted(0, mem, state)
        self.assertEqual(node.role, Role.FOLLOWER)
        self.assertEqual(node.current_term, 7)
        self.assertEqual(node.voted_for, 2)
        self.assertEqual(node.last_index(), 6)
        self.assertEqual(node.commit_index, 6)
        self.assertEqual(node.last_applied, 4)
        self.assertEqual(node.take_installed_snapshot(), b"S")


if __name__ == "__main__":
    unittest.main()
