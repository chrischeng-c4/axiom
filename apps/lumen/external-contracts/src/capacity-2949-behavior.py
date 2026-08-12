"""EC behavior case for #2949 -- persisted online PVC-growth decisions.

Every expected value in this case is an EC-owned literal that pins #2949's
TD-observable rules: R1 selects a strictly larger configured geometric storage
generation and uses direct PVC work; R2 gives later members at least the
committed desired capacity; R3 admits physical-capacity plus I/O evidence; R4
commits only after all required expansion proofs; and R5 keeps resize intent
monotonic and idempotent. Runtime Kubernetes, filesystem, traffic, and restart
proof is deliberately excluded from this pure design contract.
"""

from __future__ import annotations

from lumen.topology.capacity import (
    CapacityObservation,
    CapacityPressure,
    ExpansionPermission,
    GrowthPolicy,
    MemberExpansionProof,
    ReconciliationResult,
    ResizeIntent,
    StorageGeneration,
)
from lumen.topology.capacity_admission import (
    admit_capacity_observation,
    advance_resize_intent,
    can_commit_storage_generation,
    decide_storage_growth,
    desired_storage_for_member,
)
from lumen.topology.capacity_verdict import GrowthPlanned

MINIMUM_CHECKS = 8

CAPACITY_2949_BEHAVIOR_MATRIX = (
    ("pressure_selects_the_next_configured_geometric_generation", (20, 8)),
    ("growth_verdict_excludes_statefulset_template_mutation", "grow_pvcs_directly"),
    ("committed_generation_is_the_member_storage_floor", (20, 8)),
    ("later_generation_receives_its_committed_storage", (40, 9)),
    ("physical_capacity_and_io_observation_is_eligible", "eligible"),
    ("all_required_expansion_proofs_allow_commit", "commit_generation"),
    ("reapplying_a_completed_reconciliation_preserves_target", (20, 8)),
    ("reconciliation_transition_never_lowers_requested_capacity", (20, 8)),
)


def _kind(verdict) -> str:
    return verdict.kind.value


def verify_capacity_2949_behavior() -> dict:
    checks = []
    policy = GrowthPolicy(configured_gib=(10, 20, 40), maximum_gib=40)
    intent = ResizeIntent(desired_gib=10, generation=7)
    pressure = CapacityPressure(
        physical_used_gib=9,
        physical_capacity_gib=10,
        disk_latency_ms=25,
        disk_throughput_mib_s=120,
    )

    # 1. R1 -- pressure chooses the next configured size, never an arbitrary
    #    byte count, and advances the persisted desired generation once.
    growth = decide_storage_growth(intent, pressure, ExpansionPermission(enabled=True), policy)
    obs1 = (growth.target_gib, growth.next_generation) if isinstance(growth, GrowthPlanned) else (-1, -1)
    exp1 = CAPACITY_2949_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CAPACITY_2949_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- the pure plan names direct PVC growth; a template patch action
    #    would let the design overclaim an immutable live StatefulSet update.
    obs2 = growth.operation if isinstance(growth, GrowthPlanned) else _kind(growth)
    exp2 = CAPACITY_2949_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CAPACITY_2949_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R2/AC3 -- a new member rendered for the committed 20-Gi generation
    #    receives that capacity rather than the original 10-Gi request.
    committed_20 = StorageGeneration(generation=8, desired_gib=20)
    request_20 = desired_storage_for_member(committed_20)
    obs3 = (request_20.requested_gib, request_20.generation)
    exp3 = CAPACITY_2949_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CAPACITY_2949_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R2 -- the same floor is held at a later committed generation too.
    committed_40 = StorageGeneration(generation=9, desired_gib=40)
    request_40 = desired_storage_for_member(committed_40)
    obs4 = (request_40.requested_gib, request_40.generation)
    exp4 = CAPACITY_2949_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CAPACITY_2949_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R3 -- index data is not the oracle; physical volume use and measured
    #    latency/throughput together make an observation eligible.
    physical_io = admit_capacity_observation(
        CapacityObservation(
            index_bytes=1,
            physical_used_gib=9,
            physical_capacity_gib=10,
            disk_latency_ms=25,
            disk_throughput_mib_s=120,
        )
    )
    obs5 = _kind(physical_io)
    exp5 = CAPACITY_2949_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CAPACITY_2949_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R4 -- each named required member supplies controller, node, and
    #    filesystem-visible capacity before the generation may commit.
    proofs = (
        MemberExpansionProof(member="raft-lumen-0", controller_expanded=True, node_expanded=True, filesystem_capacity_gib=20),
        MemberExpansionProof(member="raft-lumen-1", controller_expanded=True, node_expanded=True, filesystem_capacity_gib=20),
    )
    commit = can_commit_storage_generation(ResizeIntent(desired_gib=20, generation=8), proofs)
    obs6 = _kind(commit)
    exp6 = CAPACITY_2949_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CAPACITY_2949_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R5/AC2 -- retrying exactly the completed result retains the persisted
    #    target rather than allocating a second generation.
    completed = ReconciliationResult(applied_generation=8, requested_gib=20, required_members=("raft-lumen-0", "raft-lumen-1"))
    once = advance_resize_intent(ResizeIntent(desired_gib=20, generation=8), completed)
    twice = advance_resize_intent(once, completed)
    obs7 = (twice.desired_gib, twice.generation)
    exp7 = CAPACITY_2949_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CAPACITY_2949_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R5/AC2 -- a stale result cannot pull the persisted desired generation
    #    back to the old 10-Gi request.
    stale = ReconciliationResult(applied_generation=7, requested_gib=10, required_members=("raft-lumen-0", "raft-lumen-1"))
    monotonic = advance_resize_intent(ResizeIntent(desired_gib=20, generation=8), stale)
    obs8 = (monotonic.desired_gib, monotonic.generation)
    exp8 = CAPACITY_2949_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CAPACITY_2949_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    return {"case_id": "capacity-2949-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
