from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.domain.consensus import (
    ClusterStateView,
    PeerAddr,
    RaftRole,
    is_leader,
    leader_peer,
)


class TestDomainConsensus(unittest.TestCase):
    def test_raft_role_lowercase_values(self) -> None:
        self.assertEqual(RaftRole.LEADER.value, "leader")
        self.assertEqual(RaftRole.FOLLOWER.value, "follower")
        self.assertEqual(RaftRole.CANDIDATE.value, "candidate")
        self.assertEqual(RaftRole.LEARNER.value, "learner")

    def test_is_leader_true_when_role_is_leader(self) -> None:
        view = ClusterStateView(
            node_id=1,
            role=RaftRole.LEADER,
            term=2,
            leader_id=1,
            applied_index=10,
            peers=(),
        )
        self.assertTrue(is_leader(view))

    def test_follower_whose_leader_id_matches_node_id_is_not_leader(
        self,
    ) -> None:
        view = ClusterStateView(
            node_id=1,
            role=RaftRole.FOLLOWER,
            term=2,
            leader_id=1,
            applied_index=10,
            peers=(),
        )
        self.assertFalse(is_leader(view))

    def test_leader_peer_returns_none_when_leader_id_is_none(self) -> None:
        view = ClusterStateView(
            node_id=1,
            role=RaftRole.FOLLOWER,
            term=2,
            leader_id=None,
            applied_index=10,
            peers=(PeerAddr(2, "http://p2:80"),),
        )
        self.assertIsNone(leader_peer(view))

    def test_leader_peer_returns_none_when_no_peer_matches(self) -> None:
        view = ClusterStateView(
            node_id=1,
            role=RaftRole.FOLLOWER,
            term=2,
            leader_id=9,
            applied_index=10,
            peers=(PeerAddr(0, "http://p0:80"), PeerAddr(1, "http://p1:80")),
        )
        self.assertIsNone(leader_peer(view))

    def test_leader_peer_finds_matching_peer(self) -> None:
        p0 = PeerAddr(0, "http://p0:80")
        p1 = PeerAddr(1, "http://p1:80")
        view = ClusterStateView(
            node_id=0,
            role=RaftRole.FOLLOWER,
            term=2,
            leader_id=1,
            applied_index=10,
            peers=(p0, p1),
        )
        self.assertEqual(leader_peer(view), p1)

    def test_cluster_state_view_peers_type_is_tuple(self) -> None:
        view = ClusterStateView(
            node_id=0,
            role=RaftRole.LEADER,
            term=1,
            leader_id=0,
            applied_index=5,
            peers=(),
        )
        self.assertEqual(type(view.peers), tuple)


if __name__ == "__main__":
    unittest.main()
