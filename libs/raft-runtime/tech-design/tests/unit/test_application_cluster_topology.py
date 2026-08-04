from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.application.cluster_topology import (
    ClusterTopology,
    ensure_static_membership_unchanged,
    topology_from_env,
)
from raft_runtime.domain.consensus import PeerAddr
from raft_runtime.domain.errors import (
    MembershipChanged,
    NonPositiveDimension,
    UnsupportedScheme,
)
from raft_runtime.infrastructure.env import (
    NODE_ID_KEY,
    POD_NAME_KEY,
    REPLICAS_PER_SHARD_KEY,
    SHARD_COUNT_KEY,
    VOTER_COUNT_KEY,
)


class TestApplicationClusterTopology(unittest.TestCase):
    def test_worked_example_full_replay(self) -> None:
        lookup_dict = {
            POD_NAME_KEY: "lumen-raft-host-5",
            SHARD_COUNT_KEY: "2",
            REPLICAS_PER_SHARD_KEY: "3",
            VOTER_COUNT_KEY: "2",
            NODE_ID_KEY: "2",
        }
        res = topology_from_env(
            lookup_dict.get,
            fallback_prefix="raft-runtime",
            scheme="https",
            service="peers",
            port=8443,
        )
        self.assertIsInstance(res, ClusterTopology)
        assert isinstance(res, ClusterTopology)
        self.assertEqual(res.prefix, "lumen-raft-host")
        self.assertEqual(res.dims.ordinal, 5)
        self.assertEqual(res.dims.shard_index, 1)
        self.assertEqual(res.dims.replica_index, 2)
        self.assertFalse(res.dims.is_voter)
        expected_peers = (
            PeerAddr(0, "https://lumen-raft-host-1.peers:8443"),
            PeerAddr(1, "https://lumen-raft-host-3.peers:8443"),
            PeerAddr(2, "https://lumen-raft-host-5.peers:8443"),
        )
        self.assertEqual(res.peers, expected_peers)

    def test_valid_pod_name_beats_fallback_prefix(self) -> None:
        lookup_dict = {POD_NAME_KEY: "lumen-raft-host-5"}
        res = topology_from_env(
            lookup_dict.get,
            fallback_prefix="raft-runtime",
            scheme="https",
            service="peers",
            port=8443,
        )
        self.assertIsInstance(res, ClusterTopology)
        assert isinstance(res, ClusterTopology)
        self.assertEqual(res.prefix, "lumen-raft-host")
        self.assertFalse(res.prefix.startswith("raft-runtime"))

    def test_fallback_applies_when_pod_name_missing_or_unparseable(
        self,
    ) -> None:
        # Missing POD_NAME
        dict1 = {
            SHARD_COUNT_KEY: "1",
            REPLICAS_PER_SHARD_KEY: "3",
            VOTER_COUNT_KEY: "2",
            NODE_ID_KEY: "1",
        }
        res1 = topology_from_env(
            dict1.get,
            fallback_prefix="raft-runtime",
            scheme="http",
            service="s",
            port=80,
        )
        self.assertIsInstance(res1, ClusterTopology)
        assert isinstance(res1, ClusterTopology)
        self.assertEqual(res1.prefix, "raft-runtime")
        self.assertEqual(res1.dims.ordinal, 1)

        # Unparseable POD_NAME
        dict2 = {
            POD_NAME_KEY: "nameless",
            SHARD_COUNT_KEY: "1",
            REPLICAS_PER_SHARD_KEY: "3",
            VOTER_COUNT_KEY: "2",
            NODE_ID_KEY: "1",
        }
        res2 = topology_from_env(
            dict2.get,
            fallback_prefix="raft-runtime",
            scheme="http",
            service="s",
            port=80,
        )
        self.assertIsInstance(res2, ClusterTopology)
        assert isinstance(res2, ClusterTopology)
        self.assertEqual(res2.prefix, "raft-runtime")
        self.assertEqual(res2.dims.ordinal, 1)

    def test_scheme_checked_before_environment(self) -> None:
        lookup_dict = {SHARD_COUNT_KEY: "0"}
        res = topology_from_env(
            lookup_dict.get,
            fallback_prefix="raft",
            scheme="h2c",
            service="s",
            port=80,
        )
        self.assertEqual(res, UnsupportedScheme("h2c", ("http", "https")))

    def test_unparseable_numeric_env_returns_non_positive_dimension_in_key_order(
        self,
    ) -> None:
        lookup1 = {SHARD_COUNT_KEY: "x", VOTER_COUNT_KEY: "y"}
        res1 = topology_from_env(
            lookup1.get,
            fallback_prefix="f",
            scheme="http",
            service="s",
            port=80,
        )
        self.assertEqual(res1, NonPositiveDimension(SHARD_COUNT_KEY, 0))

        lookup2 = {VOTER_COUNT_KEY: "y"}
        res2 = topology_from_env(
            lookup2.get,
            fallback_prefix="f",
            scheme="http",
            service="s",
            port=80,
        )
        self.assertEqual(res2, NonPositiveDimension(VOTER_COUNT_KEY, 0))

    def test_peers_tuple_includes_self(self) -> None:
        lookup_dict = {REPLICAS_PER_SHARD_KEY: "3"}
        res = topology_from_env(
            lookup_dict.get,
            fallback_prefix="f",
            scheme="http",
            service="s",
            port=80,
        )
        self.assertIsInstance(res, ClusterTopology)
        assert isinstance(res, ClusterTopology)
        self.assertEqual(len(res.peers), 3)

    def test_ensure_static_membership_unchanged(self) -> None:
        self.assertIsNone(ensure_static_membership_unchanged(3, 3))
        self.assertEqual(
            ensure_static_membership_unchanged(3, 5), MembershipChanged(3, 5)
        )
        self.assertEqual(
            ensure_static_membership_unchanged(5, 3), MembershipChanged(5, 3)
        )

    def test_topology_from_env_default_single_node(self) -> None:
        res = topology_from_env(
            lambda k: None,
            fallback_prefix="default-node",
            scheme="http",
            service="svc",
            port=8080,
        )
        self.assertIsInstance(res, ClusterTopology)
        assert isinstance(res, ClusterTopology)
        self.assertEqual(res.node_id, 0)
        self.assertEqual(res.dims.shard_count, 1)
        self.assertEqual(res.dims.replicas_per_shard, 1)
        self.assertEqual(res.dims.voter_count, 1)

    def test_topology_from_env_dims_problem_propagation(self) -> None:
        lookup_dict = {REPLICAS_PER_SHARD_KEY: "2", NODE_ID_KEY: "5"}
        res = topology_from_env(
            lookup_dict.get,
            fallback_prefix="f",
            scheme="http",
            service="s",
            port=80,
        )
        from raft_runtime.domain.errors import NodeIdOutOfRange

        self.assertEqual(res, NodeIdOutOfRange(5, 2))


if __name__ == "__main__":
    unittest.main()
