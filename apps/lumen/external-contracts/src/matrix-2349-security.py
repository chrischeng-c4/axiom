"""EC security case for #2349 -- fail-closed topology-matrix policy.

Every expected value below is an EC-owned literal transcribed from #2349: R4
rejects two Lumen data members on one node; R6 refuses automatic shard, voter,
and shard-PVC-capacity contraction while still admitting read-replica and
machine-type scale-down.  Refusal rows inspect the reason and named field, not
a design-computed validity flag or a generic rejection.
"""

from __future__ import annotations

import lumen.topology.matrix as topology_matrix
import lumen.topology.mutation as topology_mutation
from lumen.topology.admission import decide_placement, decide_topology_mutation
from lumen.topology.spec import TopologySpec
from lumen.topology.verdict import Rejection

MINIMUM_CHECKS = 11

MATRIX_2349_SECURITY_MATRIX = (
    ("co_located_data_members_are_rejected", "data_member_node_conflict"),
    ("co_location_refusal_names_the_node_field", "placement.node_name"),
    ("distinct_node_neighbour_is_admitted", "admitted"),
    ("automatic_shard_contraction_is_refused", "shard_contraction_not_supported"),
    ("shard_contraction_refusal_names_shard_count", "shard_count"),
    ("automatic_voter_contraction_is_refused", "voter_contraction_not_supported"),
    ("voter_contraction_refusal_names_voters", "voters"),
    ("automatic_shard_pvc_capacity_contraction_is_refused", "shard_pvc_capacity_contraction_not_supported"),
    ("pvc_capacity_contraction_refusal_names_capacity", "shard_pvc_capacity_gib"),
    ("read_replica_scale_down_neighbour_is_admitted", "admitted"),
    ("machine_type_scale_down_neighbour_is_admitted", "admitted"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _field(verdict) -> str:
    return verdict.field_path if isinstance(verdict, Rejection) else "unexpected_admission"


def verify_matrix_2349_security() -> dict:
    checks = []

    co_located = decide_placement(
        (("lumen-a/member-0", "node-a"), ("lumen-b/member-0", "node-a"))
    )

    # 1. R4 -- the admission surface rejects a duplicate node explicitly.
    obs1 = _outcome(co_located)
    exp1 = MATRIX_2349_SECURITY_MATRIX[0][1]
    checks.append({"name": MATRIX_2349_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R4 -- the rejection names the field that carries the failure domain.
    obs2 = _field(co_located)
    exp2 = MATRIX_2349_SECURITY_MATRIX[1][1]
    checks.append({"name": MATRIX_2349_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    distinct_nodes = decide_placement(
        (("lumen-a/member-0", "node-a"), ("lumen-b/member-0", "node-b"))
    )

    # 3. R4 -- the closest non-conflicting placement remains available.
    obs3 = _outcome(distinct_nodes)
    exp3 = MATRIX_2349_SECURITY_MATRIX[2][1]
    checks.append({"name": MATRIX_2349_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    shard_contraction = decide_topology_mutation(
        TopologySpec(shard_minimum=2, voters=3, read_replicas=1, shard_pvc_capacity_gib=100, machine_type="n2-standard-4"),
        TopologySpec(shard_minimum=1, voters=3, read_replicas=1, shard_pvc_capacity_gib=100, machine_type="n2-standard-4"),
    )

    # 4. R6 -- automatic shard-count contraction fails closed in v1.
    obs4 = _outcome(shard_contraction)
    exp4 = MATRIX_2349_SECURITY_MATRIX[3][1]
    checks.append({"name": MATRIX_2349_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R6 -- its refusal identifies the contracted shard dimension.
    obs5 = _field(shard_contraction)
    exp5 = MATRIX_2349_SECURITY_MATRIX[4][1]
    checks.append({"name": MATRIX_2349_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    voter_contraction = decide_topology_mutation(
        TopologySpec(shard_minimum=2, voters=3, read_replicas=1, shard_pvc_capacity_gib=100, machine_type="n2-standard-4"),
        TopologySpec(shard_minimum=2, voters=1, read_replicas=1, shard_pvc_capacity_gib=100, machine_type="n2-standard-4"),
    )

    # 6. R6 -- voter contraction is independently refused, not folded into shards.
    obs6 = _outcome(voter_contraction)
    exp6 = MATRIX_2349_SECURITY_MATRIX[5][1]
    checks.append({"name": MATRIX_2349_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R6 -- the voter refusal names its own input field.
    obs7 = _field(voter_contraction)
    exp7 = MATRIX_2349_SECURITY_MATRIX[6][1]
    checks.append({"name": MATRIX_2349_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    pvc_capacity_contraction = decide_topology_mutation(
        TopologySpec(shard_minimum=2, voters=3, read_replicas=1, shard_pvc_capacity_gib=100, machine_type="n2-standard-4"),
        TopologySpec(shard_minimum=2, voters=3, read_replicas=1, shard_pvc_capacity_gib=50, machine_type="n2-standard-4"),
    )

    # 8. R6 -- automatic voter/shard-PVC capacity contraction is also refused.
    obs8 = _outcome(pvc_capacity_contraction)
    exp8 = MATRIX_2349_SECURITY_MATRIX[7][1]
    checks.append({"name": MATRIX_2349_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R6 -- that refusal identifies the PVC-capacity input, not an opaque error.
    obs9 = _field(pvc_capacity_contraction)
    exp9 = MATRIX_2349_SECURITY_MATRIX[8][1]
    checks.append({"name": MATRIX_2349_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    replica_neighbour = decide_topology_mutation(
        TopologySpec(shard_minimum=2, voters=3, read_replicas=2, shard_pvc_capacity_gib=100, machine_type="n2-standard-4"),
        TopologySpec(shard_minimum=2, voters=3, read_replicas=1, shard_pvc_capacity_gib=100, machine_type="n2-standard-4"),
    )

    # 10. R6 -- the permitted read-replica neighbour guards against over-refusal.
    obs10 = _outcome(replica_neighbour)
    exp10 = MATRIX_2349_SECURITY_MATRIX[9][1]
    checks.append({"name": MATRIX_2349_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    machine_neighbour = decide_topology_mutation(
        TopologySpec(shard_minimum=2, voters=3, read_replicas=1, shard_pvc_capacity_gib=100, machine_type="n2-standard-8"),
        TopologySpec(shard_minimum=2, voters=3, read_replicas=1, shard_pvc_capacity_gib=100, machine_type="n2-standard-4"),
    )

    # 11. R6 -- the permitted machine-type neighbour is separately protected.
    obs11 = _outcome(machine_neighbour)
    exp11 = MATRIX_2349_SECURITY_MATRIX[10][1]
    checks.append({"name": MATRIX_2349_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "matrix-2349-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(check["passed"] for check in checks) and len(checks) == MINIMUM_CHECKS,
    }
