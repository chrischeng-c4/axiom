"""EC security case for #2949 -- capacity growth fails closed and never overclaims.

The literals pin the TD-observable refusal rules from #2949: R3 rejects
index-byte-only input, R4 withholds commit until every required expansion proof
exists, R6 refuses shrink and selects split-needed at capacity or an I/O
ceiling, and AC4 projects unsupported and failed expansion as non-converged
operator states. Kubernetes execution and restart survival remain runtime-only.
"""

from __future__ import annotations

from lumen.topology.capacity import (
    CapacityObservation,
    CeilingState,
    GrowthPolicy,
    MemberExpansionProof,
    ResizeIntent,
    StorageGeneration,
)
from lumen.topology.capacity_admission import (
    admit_capacity_observation,
    can_commit_storage_generation,
    decide_capacity_next_step,
)
from lumen.topology.capacity_status import project_resize_status
from lumen.topology.capacity_verdict import RejectedCapacity

MINIMUM_CHECKS = 12

CAPACITY_2949_SECURITY_MATRIX = (
    ("index_bytes_only_observation_is_refused", "index_bytes_only"),
    ("index_bytes_refusal_names_physical_capacity", "physical_capacity"),
    ("physical_capacity_and_io_neighbor_is_admitted", "eligible"),
    ("missing_controller_proof_withholds_generation_commit", "await_expansion_proofs"),
    ("missing_controller_proof_names_the_member", "member_proofs.raft-lumen-1.controller_expanded"),
    ("shrink_request_is_refused", "shrink_not_permitted"),
    ("configured_maximum_returns_split_needed", "split_needed"),
    ("measured_io_ceiling_returns_split_needed", "split_needed"),
    ("unsupported_expansion_projects_operator_action", "operator_action_required"),
    ("failed_expansion_projects_split_needed", "split_needed"),
    ("missing_node_proof_withholds_generation_commit", "await_expansion_proofs"),
    ("insufficient_filesystem_capacity_withholds_generation_commit", "await_expansion_proofs"),
)


def _kind(verdict) -> str:
    return verdict.kind.value


def _reason(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, RejectedCapacity) else "admitted"


def verify_capacity_2949_security() -> dict:
    checks = []

    # 1-2. R3 -- index bytes alone are explicitly insufficient, and the
    # refusal points to the missing physical-capacity provenance.
    index_only = admit_capacity_observation(CapacityObservation(index_bytes=9_000_000_000))
    obs1 = _reason(index_only)
    exp1 = CAPACITY_2949_SECURITY_MATRIX[0][1]
    checks.append({"name": CAPACITY_2949_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    obs2 = index_only.field_path if isinstance(index_only, RejectedCapacity) else ""
    exp2 = CAPACITY_2949_SECURITY_MATRIX[1][1]
    checks.append({"name": CAPACITY_2949_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R3 -- the neighbouring explicitly complete observation is admitted;
    # a design that rejects all evidence cannot satisfy fail-closed admission.
    eligible = admit_capacity_observation(CapacityObservation(index_bytes=9_000_000_000, physical_used_gib=9, physical_capacity_gib=10, disk_latency_ms=25, disk_throughput_mib_s=120))
    obs3 = _kind(eligible)
    exp3 = CAPACITY_2949_SECURITY_MATRIX[2][1]
    checks.append({"name": CAPACITY_2949_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4-5. R4 -- a named member without controller expansion proof blocks
    # commit and identifies the exact missing proof rather than overclaiming.
    incomplete = can_commit_storage_generation(
        ResizeIntent(desired_gib=20, generation=8),
        (
            MemberExpansionProof(member="raft-lumen-0", controller_expanded=True, node_expanded=True, filesystem_capacity_gib=20),
            MemberExpansionProof(member="raft-lumen-1", controller_expanded=False, node_expanded=True, filesystem_capacity_gib=20),
        ),
    )
    obs4 = _kind(incomplete)
    exp4 = CAPACITY_2949_SECURITY_MATRIX[3][1]
    checks.append({"name": CAPACITY_2949_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    obs5 = incomplete.field_path if isinstance(incomplete, RejectedCapacity) else ""
    exp5 = CAPACITY_2949_SECURITY_MATRIX[4][1]
    checks.append({"name": CAPACITY_2949_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    policy = GrowthPolicy(configured_gib=(10, 20, 40), maximum_gib=40)
    current = StorageGeneration(generation=8, desired_gib=20)

    # 6. R6 -- the explicit lower requested size exercises the prohibition;
    # a default current size would never tell shrink refusal from ignoring it.
    shrink = decide_capacity_next_step(policy, current, CeilingState(requested_gib=10))
    obs6 = _reason(shrink)
    exp6 = CAPACITY_2949_SECURITY_MATRIX[5][1]
    checks.append({"name": CAPACITY_2949_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R6 -- at the configured maximum, automatic resize stops and asks for
    # a split instead of attempting a zero-growth or shrink operation.
    at_maximum = decide_capacity_next_step(policy, StorageGeneration(generation=9, desired_gib=40), CeilingState(requested_gib=40))
    obs7 = _kind(at_maximum)
    exp7 = CAPACITY_2949_SECURITY_MATRIX[6][1]
    checks.append({"name": CAPACITY_2949_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R6 -- the explicitly supplied I/O ceiling takes the same split path.
    io_ceiling = decide_capacity_next_step(policy, current, CeilingState(requested_gib=20, io_ceiling_measured=True))
    obs8 = _kind(io_ceiling)
    exp8 = CAPACITY_2949_SECURITY_MATRIX[7][1]
    checks.append({"name": CAPACITY_2949_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9-10. AC4 -- neither unsupported nor failed expansion is projected as
    # converged. Their distinct statuses tell the operator what must happen.
    unsupported = project_resize_status(ResizeIntent(desired_gib=20, generation=8), "unsupported_expansion")
    obs9 = unsupported.status.value
    exp9 = CAPACITY_2949_SECURITY_MATRIX[8][1]
    checks.append({"name": CAPACITY_2949_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    failed = project_resize_status(ResizeIntent(desired_gib=20, generation=8), "expansion_failed")
    obs10 = failed.status.value
    exp10 = CAPACITY_2949_SECURITY_MATRIX[9][1]
    checks.append({"name": CAPACITY_2949_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R4 -- controller confirmation alone cannot commit the generation;
    # every required member must also have completed node-side expansion.
    node_incomplete = can_commit_storage_generation(
        ResizeIntent(desired_gib=20, generation=8),
        (
            MemberExpansionProof(member="raft-lumen-0", controller_expanded=True, node_expanded=True, filesystem_capacity_gib=20),
            MemberExpansionProof(member="raft-lumen-1", controller_expanded=True, node_expanded=False, filesystem_capacity_gib=20),
        ),
    )
    obs11 = _kind(node_incomplete)
    exp11 = CAPACITY_2949_SECURITY_MATRIX[10][1]
    checks.append({"name": CAPACITY_2949_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R4 -- a PVC's control-plane resize is insufficient until the
    # filesystem exposes at least the persisted desired capacity.
    filesystem_incomplete = can_commit_storage_generation(
        ResizeIntent(desired_gib=20, generation=8),
        (
            MemberExpansionProof(member="raft-lumen-0", controller_expanded=True, node_expanded=True, filesystem_capacity_gib=20),
            MemberExpansionProof(member="raft-lumen-1", controller_expanded=True, node_expanded=True, filesystem_capacity_gib=19),
        ),
    )
    obs12 = _kind(filesystem_incomplete)
    exp12 = CAPACITY_2949_SECURITY_MATRIX[11][1]
    checks.append({"name": CAPACITY_2949_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    return {"case_id": "capacity-2949-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
