from __future__ import annotations

import pathlib
import sys
import unittest

SRC_DIR = pathlib.Path(__file__).parents[2] / "src"
if str(SRC_DIR) not in sys.path:
    sys.path.insert(0, str(SRC_DIR))

from lumen.topology.admission import (
    decide_placement,
    decide_topology_mutation,
    decide_topology_spec,
)
from lumen.topology.availability import AvailabilityPromise, availability_promise
from lumen.topology.spec import TopologySpec
from lumen.topology.status import TopologyStatus
from lumen.topology.verdict import AdmittedTopology, Rejection, RejectionReason


class TestTopologyContract(unittest.TestCase):
    def test_refused_even_voter_count_other_than_two(self) -> None:
        spec = TopologySpec(shard_minimum=1, voters=4, read_replicas=0)
        verdict = decide_topology_spec(spec)
        self.assertIsInstance(verdict, Rejection)
        if isinstance(verdict, Rejection):
            self.assertEqual(verdict.reason, RejectionReason.EVEN_VOTER_COUNT)
            self.assertEqual(verdict.field_path, "voters")

    def test_refused_odd_voter_count_other_than_five(self) -> None:
        spec = TopologySpec(shard_minimum=1, voters=7, read_replicas=0)
        verdict = decide_topology_spec(spec)
        self.assertIsInstance(verdict, Rejection)
        if isinstance(verdict, Rejection):
            self.assertEqual(verdict.reason, RejectionReason.UNSUPPORTED_VOTER_COUNT)
            self.assertEqual(verdict.field_path, "voters")

    def test_admitted_shard_minimum_other_than_one_and_four(self) -> None:
        spec = TopologySpec(shard_minimum=3, voters=3, read_replicas=2)
        verdict = decide_topology_spec(spec)
        self.assertIsInstance(verdict, AdmittedTopology)
        if isinstance(verdict, AdmittedTopology):
            self.assertEqual(verdict.shard_count, 3)
            self.assertEqual(verdict.voters, 3)
            self.assertEqual(verdict.read_replicas, 2)

    def test_generation_mismatch_with_committed_render_is_not_converged(self) -> None:
        admitted = AdmittedTopology(shard_count=1, voters=1, read_replicas=0)
        status = TopologyStatus(
            policy=TopologySpec.default(),
            current=admitted,
            target=admitted,
            observed_generation=10,
            converged_generation=11,
            render_committed=True,
        )
        self.assertFalse(status.is_converged())

    def test_target_mismatch_with_committed_render_is_not_converged(self) -> None:
        current = AdmittedTopology(shard_count=1, voters=1, read_replicas=0)
        target = AdmittedTopology(shard_count=1, voters=3, read_replicas=0)
        status = TopologyStatus(
            policy=TopologySpec.default(),
            current=current,
            target=target,
            observed_generation=10,
            converged_generation=10,
            render_committed=True,
        )
        self.assertFalse(status.is_converged())

    def test_availability_promise_refuses_unadmitted_voter_counts(self) -> None:
        with self.assertRaises(ValueError):
            availability_promise(2)
        with self.assertRaises(ValueError):
            availability_promise(5)
        with self.assertRaises(ValueError):
            availability_promise(0)

    def test_placement_accepts_distinct_nodes(self) -> None:
        placements = (("instance-a", "node-1"), ("instance-b", "node-2"), ("instance-c", "node-3"))
        verdict = decide_placement(placements)
        self.assertIsInstance(verdict, AdmittedTopology)
        if isinstance(verdict, AdmittedTopology):
            self.assertEqual(verdict.shard_count, 3)
            self.assertEqual(verdict.voters, 3)

    def test_placement_rejects_co_located_nodes(self) -> None:
        placements = (("instance-a", "node-1"), ("instance-a", "node-2"), ("instance-b", "node-1"))
        verdict = decide_placement(placements)
        self.assertIsInstance(verdict, Rejection)
        if isinstance(verdict, Rejection):
            self.assertEqual(verdict.reason, RejectionReason.DATA_MEMBER_NODE_CONFLICT)
            self.assertEqual(verdict.field_path, "placement.node_name")

    def test_topology_mutation_accepts_identical_spec(self) -> None:
        spec = TopologySpec.default()
        verdict = decide_topology_mutation(spec, spec)
        self.assertIsInstance(verdict, AdmittedTopology)

    def test_topology_mutation_rejects_shard_minimum_change(self) -> None:
        spec1 = TopologySpec(shard_minimum=1, voters=3, read_replicas=0)
        spec2 = TopologySpec(shard_minimum=2, voters=3, read_replicas=0)
        verdict = decide_topology_mutation(spec1, spec2)
        self.assertIsInstance(verdict, Rejection)
        if isinstance(verdict, Rejection):
            self.assertEqual(verdict.reason, RejectionReason.NO_SAFE_TOPOLOGY_MUTATION)
            self.assertEqual(verdict.field_path, "shard_minimum")


if __name__ == "__main__":
    unittest.main()
