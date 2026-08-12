"""EC behavior case for #2941 -- stable per-shard workload planning.

Every expected value below is an EC-owned literal transcribed from issue #2941:
R1/AC1 preserve an existing member's shard, StatefulSet, and PVC identities
when another shard is added; R2 supplies deterministic peer DNS, shard-scoped
claims, and retention; R4 requires hostname and zone spreading for voters; and
R8/AC4 provide both public eligible-data selection and headless peer identity.
"""

from __future__ import annotations

from lumen.topology.workload import (
    derive_service_plan,
    derive_workload_plan,
    member_identity,
    voter_placement_requirements,
)

MINIMUM_CHECKS = 12

WORKLOAD_2941_BEHAVIOR_MATRIX = (
    ("existing_shard_keeps_its_statefulset_name_after_shard_growth", "lumen-orders-shard-0"),
    ("existing_member_keeps_its_shard_assignment_after_shard_growth", "orders-0"),
    ("existing_member_keeps_its_pvc_identity_after_shard_growth", "data-lumen-orders-0-0"),
    ("member_dns_is_deterministic_and_shard_scoped", "lumen-orders-0-0.lumen-orders-peers.default.svc"),
    ("member_claim_is_shard_scoped", "data-lumen-orders-0-0"),
    ("existing_voter_claim_is_retained", "retain"),
    ("voter_placement_requires_hostname_spreading", "kubernetes.io/hostname"),
    ("voter_placement_requires_zone_spreading", "topology.kubernetes.io/zone"),
    ("public_service_is_a_cluster_ip_service", "ClusterIP"),
    ("public_service_selects_every_eligible_data_member", ("lumen-orders-0-0", "lumen-orders-0-1", "lumen-orders-1-0")),
    ("peer_service_is_headless", "None"),
    ("peer_service_preserves_group_member_dns_identity", "lumen-orders-0-0.lumen-orders-peers.default.svc"),
)


def verify_workload_2941_behavior() -> dict:
    checks = []

    current = {
        "instance_id": "lumen-orders",
        "namespace": "default",
        "shards": {
            "orders-0": {
                "members": (
                    {"member_id": "lumen-orders-0-0", "ordinal": 0, "role": "voter", "pvc_id": "data-lumen-orders-0-0"},
                    {"member_id": "lumen-orders-0-1", "ordinal": 1, "role": "voter", "pvc_id": "data-lumen-orders-0-1"},
                )
            }
        },
    }
    target = {
        "instance_id": "lumen-orders",
        "namespace": "default",
        "shards": {
            **current["shards"],
            "orders-1": {
                "members": (
                    {"member_id": "lumen-orders-1-0", "ordinal": 0, "role": "voter", "pvc_id": "data-lumen-orders-1-0"},
                )
            },
        },
    }
    workloads = derive_workload_plan(current, target)

    # 1. R1 -- shard growth adds a workload; it does not rename the old one.
    obs1 = workloads["orders-0"]["statefulset_name"]
    exp1 = WORKLOAD_2941_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": WORKLOAD_2941_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1/AC1 -- the member remains assigned to its original shard.
    obs2 = workloads["orders-0"]["members"]["lumen-orders-0-0"]["shard_id"]
    exp2 = WORKLOAD_2941_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": WORKLOAD_2941_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. AC1 -- its durable claim identity is equally stable across expansion.
    obs3 = workloads["orders-0"]["members"]["lumen-orders-0-0"]["pvc_id"]
    exp3 = WORKLOAD_2941_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": WORKLOAD_2941_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    identity = member_identity({
        "instance_id": "lumen-orders", "namespace": "default", "shard_id": "orders-0",
        "member_id": "lumen-orders-0-0", "ordinal": 0, "role": "voter",
        "pvc_id": "data-lumen-orders-0-0", "existing": True,
    })

    # 4. R2 -- peer DNS is stable at the member and shard boundary.
    obs4 = identity["peer_dns"]
    exp4 = WORKLOAD_2941_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": WORKLOAD_2941_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R2 -- the claim name carries the shard/member identity, not a flat ordinal.
    obs5 = identity["pvc_id"]
    exp5 = WORKLOAD_2941_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": WORKLOAD_2941_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R2 -- an existing voter claim is retained unless policy explicitly selects reclamation.
    obs6 = identity["claim_retention"]
    exp6 = WORKLOAD_2941_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": WORKLOAD_2941_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    placement = voter_placement_requirements({"shard_id": "orders-0", "voters": ("lumen-orders-0-0", "lumen-orders-0-1")})

    # 7. R4 -- voters are deliberately spread by hostname.
    obs7 = placement["hostname_topology_key"]
    exp7 = WORKLOAD_2941_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": WORKLOAD_2941_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R4 -- and independently by zone when capacity can satisfy it.
    obs8 = placement["zone_topology_key"]
    exp8 = WORKLOAD_2941_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": WORKLOAD_2941_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    services = derive_service_plan(workloads)

    # 9. R8/AC4 -- clients see one ordinary ClusterIP service.
    obs9 = services["public"]["service_type"]
    exp9 = WORKLOAD_2941_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": WORKLOAD_2941_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R8/AC4 -- its selector includes all eligible data members, across shards.
    obs10 = tuple(services["public"]["eligible_member_ids"])
    exp10 = WORKLOAD_2941_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": WORKLOAD_2941_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R8 -- peers use a distinct headless service.
    obs11 = services["peers"]["cluster_ip"]
    exp11 = WORKLOAD_2941_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": WORKLOAD_2941_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R8/AC4 -- the peer plan retains the stable group/member DNS identity.
    obs12 = services["peers"]["member_dns"]["lumen-orders-0-0"]
    exp12 = WORKLOAD_2941_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": WORKLOAD_2941_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "workload-2941-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
