"""Unit tests for #2349 topology matrix, admission, mutation, placement, and availability."""
from __future__ import annotations

import pathlib
import sys
import unittest

SRC_DIR = pathlib.Path(__file__).parents[2] / "src"
if str(SRC_DIR) not in sys.path:
    sys.path.insert(0, str(SRC_DIR))

import lumen.topology.matrix as topology_matrix
from lumen.topology.admission import (
    decide_placement,
    decide_topology_mutation,
    decide_topology_spec,
)
from lumen.topology.availability import AvailabilityPromise, availability_promise
from lumen.topology.spec import TopologySpec
from lumen.topology.verdict import AdmittedTopology, Rejection, RejectionReason


class TestMatrix2349(unittest.TestCase):
    def test_topology_matrix_descriptors_exist_and_have_distinct_values(self) -> None:
        descriptors = {d.name: d for d in topology_matrix.TOPOLOGY_MATRIX}
        self.assertIn("1x1", descriptors)
        self.assertIn("Nx1", descriptors)
        self.assertIn("1xR", descriptors)
        self.assertIn("NxR", descriptors)

        d_1x1 = descriptors["1x1"]
        self.assertEqual((d_1x1.shard_count, d_1x1.voters, d_1x1.read_replicas), (1, 1, 0))

        d_Nx1 = descriptors["Nx1"]
        self.assertEqual((d_Nx1.shard_count, d_Nx1.voters, d_Nx1.read_replicas), (2, 1, 0))

        d_1xR = descriptors["1xR"]
        self.assertEqual((d_1xR.shard_count, d_1xR.voters, d_1xR.read_replicas), (1, 3, 1))

        d_NxR = descriptors["NxR"]
        self.assertEqual((d_NxR.shard_count, d_NxR.voters, d_NxR.read_replicas), (2, 3, 1))

    def test_novel_spec_admission_and_refusal(self) -> None:
        valid_spec = TopologySpec(
            shard_minimum=5,
            voters=3,
            read_replicas=4,
            shard_pvc_capacity_gib=250,
            machine_type="n2-standard-16",
        )
        verdict = decide_topology_spec(valid_spec)
        self.assertIsInstance(verdict, AdmittedTopology)
        if isinstance(verdict, AdmittedTopology):
            self.assertEqual(verdict.shard_count, 5)
            self.assertEqual(verdict.voters, 3)
            self.assertEqual(verdict.read_replicas, 4)

        invalid_voters = TopologySpec(shard_minimum=2, voters=5, read_replicas=1)
        rej = decide_topology_spec(invalid_voters)
        self.assertIsInstance(rej, Rejection)
        if isinstance(rej, Rejection):
            self.assertEqual(rej.reason, RejectionReason.UNSUPPORTED_VOTER_COUNT)
            self.assertEqual(rej.field_path, "voters")

    def test_novel_mutation_expansion_and_contraction(self) -> None:
        c = TopologySpec(shard_minimum=2, voters=3, read_replicas=1, shard_pvc_capacity_gib=50, machine_type="n2-standard-4")
        t_expansion = TopologySpec(shard_minimum=2, voters=3, read_replicas=3, shard_pvc_capacity_gib=200, machine_type="n2-standard-16")
        res_exp = decide_topology_mutation(c, t_expansion)
        self.assertIsInstance(res_exp, AdmittedTopology)

        c_large = TopologySpec(shard_minimum=4, voters=3, read_replicas=2, shard_pvc_capacity_gib=200, machine_type="n2-standard-16")

        t_shard_contract = TopologySpec(shard_minimum=2, voters=3, read_replicas=2, shard_pvc_capacity_gib=200, machine_type="n2-standard-16")
        res_sc = decide_topology_mutation(c_large, t_shard_contract)
        self.assertIsInstance(res_sc, Rejection)
        if isinstance(res_sc, Rejection):
            self.assertEqual(res_sc.reason, RejectionReason.SHARD_CONTRACTION_NOT_SUPPORTED)
            self.assertEqual(res_sc.field_path, "shard_minimum")

        t_voter_contract = TopologySpec(shard_minimum=4, voters=1, read_replicas=2, shard_pvc_capacity_gib=200, machine_type="n2-standard-16")
        res_vc = decide_topology_mutation(c_large, t_voter_contract)
        self.assertIsInstance(res_vc, Rejection)
        if isinstance(res_vc, Rejection):
            self.assertEqual(res_vc.reason, RejectionReason.VOTER_CONTRACTION_NOT_SUPPORTED)
            self.assertEqual(res_vc.field_path, "voters")

        t_pvc_contract = TopologySpec(shard_minimum=4, voters=3, read_replicas=2, shard_pvc_capacity_gib=100, machine_type="n2-standard-16")
        res_pc = decide_topology_mutation(c_large, t_pvc_contract)
        self.assertIsInstance(res_pc, Rejection)
        if isinstance(res_pc, Rejection):
            self.assertEqual(res_pc.reason, RejectionReason.SHARD_PVC_CAPACITY_CONTRACTION_NOT_SUPPORTED)
            self.assertEqual(res_pc.field_path, "shard_pvc_capacity_gib")

    def test_novel_placement(self) -> None:
        placements = (
            ("lumen-1/m-0", "node-10"),
            ("lumen-1/m-1", "node-11"),
            ("lumen-2/m-0", "node-12"),
            ("lumen-2/m-1", "node-13"),
        )
        res = decide_placement(placements)
        self.assertIsInstance(res, AdmittedTopology)
        if isinstance(res, AdmittedTopology):
            self.assertEqual(res.shard_count, 4)
            self.assertEqual(res.voters, 4)

        conflict_placements = (
            ("lumen-1/m-0", "node-10"),
            ("lumen-2/m-0", "node-10"),
        )
        res_conflict = decide_placement(conflict_placements)
        self.assertIsInstance(res_conflict, Rejection)
        if isinstance(res_conflict, Rejection):
            self.assertEqual(res_conflict.reason, RejectionReason.DATA_MEMBER_NODE_CONFLICT)
            self.assertEqual(res_conflict.field_path, "placement.node_name")

    def test_availability_promises(self) -> None:
        self.assertEqual(availability_promise(1), AvailabilityPromise.NO_PROMISE_ON_UNEXPECTED_NODE_LOSS)
        self.assertEqual(availability_promise(3), AvailabilityPromise.SURVIVES_ONE_UNEXPECTED_NODE_LOSS)
        with self.assertRaises(ValueError):
            availability_promise(4)


if __name__ == "__main__":
    unittest.main()
