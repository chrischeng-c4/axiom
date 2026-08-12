"""EC behavior case for #2349 -- pure GKE topology-matrix policy.

Every expected value below is an EC-owned literal transcribed from #2349:
R2 requires the 1x1, Nx1, 1xR, and NxR matrix to retain separate voter and
non-voting read-replica counts; R4 admits placements with distinct data-member
nodes; R6 permits read-replica and machine-type scale-downs; and AC3 names the
different one- and three-voter unexpected-node-loss promises.  The concrete
``N=2`` and ``R=(3 voters, 1 read replica)`` inputs are the smallest nontrivial
members of the issue's N and R classes, not defaults read from the design.
"""

from __future__ import annotations

import lumen.topology.matrix as topology_matrix
import lumen.topology.mutation as topology_mutation
from lumen.topology.admission import (
    decide_placement,
    decide_topology_mutation,
    decide_topology_spec,
)
from lumen.topology.availability import availability_promise
from lumen.topology.spec import TopologySpec
from lumen.topology.verdict import Rejection

MINIMUM_CHECKS = 9

MATRIX_2349_BEHAVIOR_MATRIX = (
    ("one_by_one_records_one_shard_one_voter_and_no_read_replicas", (1, 1, 0)),
    ("many_by_one_records_many_shards_one_voter_and_no_read_replicas", (2, 1, 0)),
    ("one_by_replicated_records_voters_separately_from_read_replicas", (1, 3, 1)),
    ("many_by_replicated_records_every_topology_dimension", (2, 3, 1)),
    ("unique_data_member_nodes_are_admitted", "admitted"),
    ("read_replica_scale_down_is_admitted", "admitted"),
    ("machine_type_scale_down_is_admitted", "admitted"),
    ("one_voter_has_no_unexpected_node_loss_promise", "no_promise_on_unexpected_node_loss"),
    ("three_voters_survive_one_unexpected_node_loss", "survives_one_unexpected_node_loss"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _topology(verdict) -> tuple[int, int, int]:
    if isinstance(verdict, Rejection):
        return (-1, -1, -1)
    return (verdict.shard_count, verdict.voters, verdict.read_replicas)


def verify_matrix_2349_behavior() -> dict:
    checks = []
    matrix_by_name = {descriptor.name: descriptor for descriptor in topology_matrix.TOPOLOGY_MATRIX}

    one_by_one_descriptor = matrix_by_name["1x1"]
    one_by_one = decide_topology_spec(
        TopologySpec(
            shard_minimum=one_by_one_descriptor.shard_count,
            voters=one_by_one_descriptor.voters,
            read_replicas=one_by_one_descriptor.read_replicas,
            shard_pvc_capacity_gib=100,
            machine_type="n2-standard-4",
        )
    )

    # 1. R2 -- the smallest matrix member remains an explicit 1x1 value.
    obs1 = _topology(one_by_one)
    exp1 = MATRIX_2349_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": MATRIX_2349_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    many_by_one_descriptor = matrix_by_name["Nx1"]
    many_by_one = decide_topology_spec(
        TopologySpec(
            shard_minimum=many_by_one_descriptor.shard_count,
            voters=many_by_one_descriptor.voters,
            read_replicas=many_by_one_descriptor.read_replicas,
            shard_pvc_capacity_gib=100,
            machine_type="n2-standard-4",
        )
    )

    # 2. R2 -- Nx1 changes shards without silently adding voters or readers.
    obs2 = _topology(many_by_one)
    exp2 = MATRIX_2349_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": MATRIX_2349_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    one_by_replicated_descriptor = matrix_by_name["1xR"]
    one_by_replicated = decide_topology_spec(
        TopologySpec(
            shard_minimum=one_by_replicated_descriptor.shard_count,
            voters=one_by_replicated_descriptor.voters,
            read_replicas=one_by_replicated_descriptor.read_replicas,
            shard_pvc_capacity_gib=100,
            machine_type="n2-standard-4",
        )
    )

    # 3. R2 -- 1xR preserves the quorum and non-voting role as separate values.
    obs3 = _topology(one_by_replicated)
    exp3 = MATRIX_2349_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": MATRIX_2349_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    many_by_replicated_descriptor = matrix_by_name["NxR"]
    many_by_replicated = decide_topology_spec(
        TopologySpec(
            shard_minimum=many_by_replicated_descriptor.shard_count,
            voters=many_by_replicated_descriptor.voters,
            read_replicas=many_by_replicated_descriptor.read_replicas,
            shard_pvc_capacity_gib=100,
            machine_type="n2-standard-4",
        )
    )

    # 4. R2 -- NxR must carry all three independent topology dimensions.
    obs4 = _topology(many_by_replicated)
    exp4 = MATRIX_2349_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": MATRIX_2349_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    unique_placement = decide_placement(
        (("lumen-a/member-0", "node-a"), ("lumen-b/member-0", "node-b"))
    )

    # 5. R4 -- the neighbouring valid placement remains admitted.
    obs5 = _outcome(unique_placement)
    exp5 = MATRIX_2349_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": MATRIX_2349_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    read_replica_scale_down = decide_topology_mutation(
        TopologySpec(shard_minimum=2, voters=3, read_replicas=2, shard_pvc_capacity_gib=100, machine_type="n2-standard-4"),
        TopologySpec(shard_minimum=2, voters=3, read_replicas=1, shard_pvc_capacity_gib=100, machine_type="n2-standard-4"),
    )

    # 6. R6 -- v1 permits a pure read-replica reduction.
    obs6 = _outcome(read_replica_scale_down)
    exp6 = MATRIX_2349_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": MATRIX_2349_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    machine_type_scale_down = decide_topology_mutation(
        TopologySpec(shard_minimum=2, voters=3, read_replicas=1, shard_pvc_capacity_gib=100, machine_type="n2-standard-8"),
        TopologySpec(shard_minimum=2, voters=3, read_replicas=1, shard_pvc_capacity_gib=100, machine_type="n2-standard-4"),
    )

    # 7. R6 -- v1 also permits a machine-type reduction without contraction.
    obs7 = _outcome(machine_type_scale_down)
    exp7 = MATRIX_2349_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": MATRIX_2349_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. AC3 -- one voter never gains an HA promise from the matrix spelling.
    obs8 = availability_promise(1).value
    exp8 = MATRIX_2349_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": MATRIX_2349_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. AC3 -- the three-voter availability promise is equally explicit.
    obs9 = availability_promise(3).value
    exp9 = MATRIX_2349_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": MATRIX_2349_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    return {
        "case_id": "matrix-2349-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(check["passed"] for check in checks) and len(checks) == MINIMUM_CHECKS,
    }
