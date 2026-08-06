from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.application.host_config import HostConfig
from raft_runtime.application.proposal_routing import (
    LOCAL,
    UNKNOWN,
    Local,
    Remote,
    Unknown,
    forward_path,
    retry_deadline_reached,
    route_proposal,
)
from raft_runtime.domain.consensus import (
    ClusterStateView,
    PeerAddr,
    RaftRole,
)
from raft_runtime.infrastructure.routes import PUBLISH_PATH


class TestApplicationProposalRouting(unittest.TestCase):
    def test_route_proposal_leader_returns_local_identity(self) -> None:
        view = ClusterStateView(
            node_id=0,
            role=RaftRole.LEADER,
            term=1,
            leader_id=0,
            applied_index=0,
            peers=(),
        )
        res = route_proposal(view)
        self.assertIs(res, LOCAL)

    def test_follower_with_matching_leader_id_routes_to_remote(self) -> None:
        p0 = PeerAddr(0, "http://p0:80")
        view = ClusterStateView(
            node_id=0,
            role=RaftRole.FOLLOWER,
            term=1,
            leader_id=0,
            applied_index=0,
            peers=(p0,),
        )
        res = route_proposal(view)
        self.assertEqual(res, Remote(p0))
        self.assertIsNot(res, LOCAL)

    def test_route_proposal_unknown_leader(self) -> None:
        # None leader_id
        v1 = ClusterStateView(
            node_id=0,
            role=RaftRole.FOLLOWER,
            term=1,
            leader_id=None,
            applied_index=0,
            peers=(),
        )
        self.assertIs(route_proposal(v1), UNKNOWN)

        # leader_id absent from peers
        v2 = ClusterStateView(
            node_id=0,
            role=RaftRole.FOLLOWER,
            term=1,
            leader_id=9,
            applied_index=0,
            peers=(PeerAddr(0, "u0"), PeerAddr(1, "u1")),
        )
        self.assertIs(route_proposal(v2), UNKNOWN)

    def test_forward_path_returns_publish_path_identity(self) -> None:
        self.assertIs(forward_path(), PUBLISH_PATH)

    def test_retry_deadline_reached_boundary(self) -> None:
        cfg = HostConfig()
        self.assertFalse(retry_deadline_reached(9_999, cfg))
        self.assertTrue(retry_deadline_reached(10_000, cfg))
        self.assertFalse(retry_deadline_reached(0, cfg))

    def test_route_proposal_remote_follower(self) -> None:
        p1 = PeerAddr(1, "http://p1:80")
        view = ClusterStateView(
            node_id=0,
            role=RaftRole.FOLLOWER,
            term=1,
            leader_id=1,
            applied_index=0,
            peers=(p1,),
        )
        self.assertEqual(route_proposal(view), Remote(p1))

    def test_singletons_identity(self) -> None:
        self.assertIsInstance(LOCAL, Local)
        self.assertIsInstance(UNKNOWN, Unknown)


if __name__ == "__main__":
    unittest.main()
