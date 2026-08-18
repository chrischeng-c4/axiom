from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.domain.errors import (
    NodeIdOutOfRange,
    NonPositiveDimension,
    VoterCountOutOfRange,
)
from raft_runtime.domain.topology import (
    ClusterDims,
    dims_problem,
    peer_ordinal,
)


class TestDomainTopology(unittest.TestCase):
    def test_cluster_dims_shard_index(self) -> None:
        dims = ClusterDims(shard_count=2, replicas_per_shard=3, voter_count=2, ordinal=5)
        self.assertEqual(dims.shard_index, 1)

    def test_cluster_dims_replica_index(self) -> None:
        dims = ClusterDims(shard_count=2, replicas_per_shard=3, voter_count=2, ordinal=5)
        self.assertEqual(dims.replica_index, 2)

    def test_cluster_dims_is_voter_false_when_replica_equals_or_exceeds_voters(
        self,
    ) -> None:
        dims = ClusterDims(shard_count=2, replicas_per_shard=3, voter_count=2, ordinal=5)
        self.assertFalse(dims.is_voter)

    def test_cluster_dims_is_voter_true_when_replica_less_than_voters(
        self,
    ) -> None:
        dims = ClusterDims(shard_count=2, replicas_per_shard=3, voter_count=3, ordinal=5)
        self.assertTrue(dims.is_voter)

    def test_peer_ordinal_calculation(self) -> None:
        self.assertEqual(peer_ordinal(2, 1, 2), 5)

    def test_peer_ordinal_round_trip_grid_identity(self) -> None:
        for sc in (1, 2, 3, 4):
            for ordinal in range(12):
                dims = ClusterDims(
                    shard_count=sc,
                    replicas_per_shard=10,
                    voter_count=1,
                    ordinal=ordinal,
                )
                computed = peer_ordinal(
                    sc, dims.shard_index, dims.replica_index
                )
                self.assertEqual(
                    computed,
                    ordinal,
                    f"Round trip failed for shard_count={sc}, ordinal={ordinal}",
                )

    def test_dims_problem_non_positive_shard_count(self) -> None:
        self.assertEqual(
            dims_problem(0, 3, 2, 0),
            NonPositiveDimension("SHARD_COUNT", 0),
        )

    def test_dims_problem_non_positive_replicas(self) -> None:
        self.assertEqual(
            dims_problem(2, 0, 2, 0),
            NonPositiveDimension("REPLICAS_PER_SHARD", 0),
        )

    def test_dims_problem_voter_count_zero_out_of_range(self) -> None:
        self.assertEqual(
            dims_problem(2, 3, 0, 0),
            VoterCountOutOfRange(0, 3),
        )

    def test_dims_problem_voter_count_exceeding_replicas(self) -> None:
        self.assertEqual(
            dims_problem(2, 3, 4, 0),
            VoterCountOutOfRange(4, 3),
        )

    def test_dims_problem_node_id_out_of_range(self) -> None:
        self.assertEqual(
            dims_problem(2, 3, 3, 3),
            NodeIdOutOfRange(3, 3),
        )

    def test_dims_problem_valid_dimensions_returns_none(self) -> None:
        self.assertIsNone(dims_problem(2, 3, 3, 2))

    def test_dims_problem_all_zero_returns_shard_count_first(self) -> None:
        self.assertEqual(
            dims_problem(0, 0, 0, 0),
            NonPositiveDimension("SHARD_COUNT", 0),
        )


if __name__ == "__main__":
    unittest.main()
