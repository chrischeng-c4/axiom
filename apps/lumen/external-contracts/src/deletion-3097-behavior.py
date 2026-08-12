"""EC behavior case for #3097 -- explicit, closed-UID data deletion.

Every expected value is an EC-owned literal transcribed from #3097: R1/AC1
make Retain the default and reserve Delete for an explicit finalization;
R3/AC4 close an immutable exact-UID inventory; R4/AC2 fix destruction order;
R5 makes exact manifest-listed not-found idempotent; R6/AC3 resumes only the
closed inventory; R7 gates finalizer release; and R8 projects safe deletion
facts. Runtime Kubernetes and object-store actions are deliberately absent.
"""

from __future__ import annotations

from lumen.deletion.inventory import ArtifactRecord, ClosedInventory, close_inventory
from lumen.deletion.plan import plan_exact_destruction
from lumen.deletion.policy import (
    DataDeletionPolicy,
    DeletionLifecycle,
    decide_deletion_start,
    effective_data_deletion_policy,
)
from lumen.deletion.progress import (
    ArtifactDeleteResult,
    ArtifactProgress,
    apply_artifact_result,
    may_remove_finalizer,
    resume_closed_inventory,
)
from lumen.deletion.status import project_deletion_status

MINIMUM_CHECKS = 12

DELETION_3097_BEHAVIOR_MATRIX = (
    ("omitted_policy_resolves_to_retain", "Retain"),
    ("explicit_retain_policy_resolves_to_retain", "Retain"),
    ("explicit_delete_policy_resolves_to_delete", "Delete"),
    ("delete_enters_destruction_at_finalizer_start", "destructive"),
    ("retain_never_enters_destruction_at_finalizer_start", "non_destructive"),
    ("agreeing_inventories_produce_an_immutable_closed_inventory", True),
    ("closed_inventory_retains_the_exact_agreed_uids", ("uid-pvc-7", "uid-stage-7", "uid-backup-7", "uid-meta-7")),
    ("quiesced_destruction_uses_the_fixed_role_order", ("staging", "pvc", "backup", "metadata")),
    ("exact_manifest_listed_not_found_marks_the_target_deleted", "deleted"),
    ("resume_returns_only_the_persisted_remaining_closed_uid", ("uid-backup-7",)),
    ("all_confirmed_artifacts_and_no_children_allow_finalizer_release", "remove_finalizer"),
    ("status_projects_uid_role_generation_and_counts", ("uid-pvc-7", "pvc", 17, 4, 1)),
)


def _artifacts() -> tuple[ArtifactRecord, ...]:
    return (
        ArtifactRecord(uid="uid-pvc-7", role="pvc", generation=17, manifest_key=None),
        ArtifactRecord(uid="uid-stage-7", role="staging", generation=17, manifest_key=None),
        ArtifactRecord(uid="uid-backup-7", role="backup", generation=17, manifest_key="backups/orders/7"),
        ArtifactRecord(uid="uid-meta-7", role="metadata", generation=17, manifest_key=None),
    )


def verify_deletion_3097_behavior() -> dict:
    checks = []

    # 1. R1/AC1 -- missing means Retain.
    obs1 = effective_data_deletion_policy(None).value
    exp1 = DELETION_3097_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": DELETION_3097_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- an explicit Retain remains Retain.
    obs2 = effective_data_deletion_policy(DataDeletionPolicy.RETAIN).value
    exp2 = DELETION_3097_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": DELETION_3097_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. AC1 -- Delete is never inferred; it must be explicit.
    obs3 = effective_data_deletion_policy(DataDeletionPolicy.DELETE).value
    exp3 = DELETION_3097_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": DELETION_3097_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R2 -- Delete becomes destructive at finalizer start.
    obs4 = decide_deletion_start(DataDeletionPolicy.DELETE, DeletionLifecycle.FINALIZER_STARTED).phase.value
    exp4 = DELETION_3097_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": DELETION_3097_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R2 -- Retain never becomes destructive at finalizer start.
    obs5 = decide_deletion_start(DataDeletionPolicy.RETAIN, DeletionLifecycle.FINALIZER_STARTED).phase.value
    exp5 = DELETION_3097_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": DELETION_3097_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R3 -- agreeing inventories produce one frozen inventory.
    closed = close_inventory(_artifacts(), _artifacts(), _artifacts())
    obs6 = isinstance(closed, ClosedInventory) and getattr(closed, "__dataclass_params__").frozen
    exp6 = DELETION_3097_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": DELETION_3097_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R3 -- the frozen inventory retains its exact agreed UIDs.
    obs7 = tuple(artifact.uid for artifact in closed.artifacts) if isinstance(closed, ClosedInventory) else ()
    exp7 = DELETION_3097_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": DELETION_3097_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R4/AC2 -- order is staging, retained/authoritative PVCs, BackupSet,
    # then metadata; targets are records rather than a namespace or prefix.
    plan = plan_exact_destruction(closed, quiesced=True)
    obs8 = tuple(target.role for target in plan.targets)
    exp8 = DELETION_3097_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": DELETION_3097_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R5 -- only a supplied exact manifest target gets idempotent success.
    initial = ArtifactProgress.for_inventory(closed)
    after_not_found = apply_artifact_result(initial, _artifacts()[2], ArtifactDeleteResult.NOT_FOUND)
    obs9 = after_not_found.artifact_states["uid-backup-7"].value
    exp9 = DELETION_3097_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": DELETION_3097_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R6/AC3 -- a persisted per-UID state leaves only that closed UID.
    persisted = ArtifactProgress.from_states(closed, {"uid-pvc-7": "deleted", "uid-stage-7": "deleted", "uid-backup-7": "pending", "uid-meta-7": "deleted"})
    resumed = resume_closed_inventory(closed, persisted)
    obs10 = tuple(target.uid for target in resumed.remaining)
    exp10 = DELETION_3097_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": DELETION_3097_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R7 -- removal is possible only after every closed artifact and child.
    complete = ArtifactProgress.from_states(closed, {artifact.uid: "deleted" for artifact in _artifacts()})
    obs11 = "remove_finalizer" if may_remove_finalizer(complete, non_data_children_gone=True) else "retain_finalizer"
    exp11 = DELETION_3097_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": DELETION_3097_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R8 -- the pure projection carries the named safe facts and counts.
    status = project_deletion_status(closed, persisted, generation=17, refusal=None)
    obs12 = (status.artifacts[0].uid, status.artifacts[0].role, status.generation, status.total_count, status.deleted_count)
    exp12 = DELETION_3097_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": DELETION_3097_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    return {"case_id": "deletion-3097-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
