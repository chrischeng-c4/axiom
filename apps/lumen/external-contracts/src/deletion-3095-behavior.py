"""EC behavior case for #3095 -- Retain-only deletion finalization.

Every expected value below is an EC-owned literal transcribed from #3095:
R1 installs one ``lumen.dev/data-lifecycle`` finalizer with Retain-only
behavior; R2 claims DeleteInstance; R3 decides the active-operation boundary;
R4 fixes the quiesce order; R5 keeps the closed-UID retained inventory; R6
chooses one idempotent resume action; and R7 projects actionable deletion
status.  Live Kubernetes cleanup, restart survival, and emitted Events/logs
are deliberately excluded: those are runtime-only proofs, not pure-design
observations.
"""

from __future__ import annotations

from lumen.deletion.admission import decide_mutation
from lumen.deletion.finalizer import decide_finalizer
from lumen.deletion.inventory import assess_closed_uid_inventory
from lumen.deletion.phase import (
    decide_active_operation_boundary,
    decide_resume,
    plan_quiesce,
)
from lumen.deletion.spec import (
    ActiveOperation,
    DeletionRequest,
    MutationRequest,
    RetainedResourceFacts,
    ResumeFacts,
)
from lumen.deletion.status import project
from lumen.deletion.verdict import Rejection

MINIMUM_CHECKS = 13

DELETION_3095_BEHAVIOR_MATRIX = (
    ("data_lifecycle_finalizer_has_the_required_identifier", "lumen.dev/data-lifecycle"),
    ("data_lifecycle_finalizer_is_retain_only", "Retain"),
    ("deleting_instance_claims_delete_instance", "admitted"),
    ("reversible_active_operation_rolls_back_before_cleanup", "rollback_before_cleanup"),
    ("irreversible_active_operation_completes_forward_before_cleanup", "complete_forward_before_cleanup"),
    ("quiesce_plan_orders_all_non_data_cleanup_boundaries", ("remove_public_routing_and_readiness", "suspend_new_backup_and_maintenance_jobs", "drain_serving_members", "remove_non_data_children")),
    ("closed_uid_inventory_retains_authoritative_pvcs", ("data-0", "data-1")),
    ("closed_uid_inventory_retains_complete_backup_sets", ("backup-7",)),
    ("resume_selects_one_idempotent_next_action", "remove_non_data_children"),
    ("status_projects_deletion_pending_condition", "DeletionPending"),
    ("status_projects_active_operation_disposition", "rollback_before_cleanup"),
    ("status_projects_retained_inventory_summary", "2 PVCs, 1 BackupSet"),
    ("status_projects_the_next_action", "rollback_before_cleanup"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_deletion_3095_behavior() -> dict:
    checks = []

    retain = decide_finalizer(DeletionRequest(instance_uid="uid-3095", requested_policy="Retain"))

    # 1. R1 -- every reconciliation decision names one data-lifecycle finalizer.
    obs1 = retain.finalizer_identifier if not isinstance(retain, Rejection) else "rejected"
    exp1 = DELETION_3095_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": DELETION_3095_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- until a separate destroy change lands, that finalizer has only Retain policy.
    obs2 = retain.data_policy if not isinstance(retain, Rejection) else "rejected"
    exp2 = DELETION_3095_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": DELETION_3095_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    deleting = decide_mutation(
        MutationRequest(operation="DeleteInstance", deletion_timestamp=True, already_claimed=False)
    )

    # 3. R2 -- deletion is an explicit operation claim, not an accidental
    # consequence of a timestamp that lets another lifecycle writer proceed.
    obs3 = _outcome(deleting)
    exp3 = DELETION_3095_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": DELETION_3095_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    reversible = decide_active_operation_boundary(
        ActiveOperation(kind="upgrade", phase="pre_cutover", reversible=True)
    )

    # 4. R3 -- a reversible operation must give cleanup a rollback boundary.
    obs4 = reversible.cleanup_boundary if not isinstance(reversible, Rejection) else "rejected"
    exp4 = DELETION_3095_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": DELETION_3095_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    irreversible = decide_active_operation_boundary(
        ActiveOperation(kind="split", phase="post_cutover", reversible=False)
    )

    # 5. R3 -- post-cutover work is authoritative only after it completes forward.
    obs5 = irreversible.cleanup_boundary if not isinstance(irreversible, Rejection) else "rejected"
    exp5 = DELETION_3095_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": DELETION_3095_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    quiesce = plan_quiesce()

    # 6. R4 -- removing non-data children comes after public withdrawal, job
    # suspension, and member drain; this observes the order, not a pass flag.
    obs6 = quiesce.actions
    exp6 = DELETION_3095_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": DELETION_3095_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    inventory = assess_closed_uid_inventory(
        RetainedResourceFacts(
            closed_uid="uid-3095",
            authoritative_pvcs=("data-0", "data-1"),
            complete_backup_sets=("backup-7",),
            ambiguous=False,
        )
    )

    # 7. R5 -- exact closed-UID facts retain authoritative PVCs.
    obs7 = inventory.retained_pvcs if not isinstance(inventory, Rejection) else ()
    exp7 = DELETION_3095_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": DELETION_3095_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R5 -- complete BackupSets are independently retained in the inventory.
    obs8 = inventory.retained_backup_sets if not isinstance(inventory, Rejection) else ()
    exp8 = DELETION_3095_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": DELETION_3095_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    resume = decide_resume(
        ResumeFacts(
            persisted_phase="members_drained",
            public_routing_present=False,
            backup_jobs_suspended=True,
            maintenance_jobs_suspended=True,
            serving_members_drained=True,
            non_data_children_present=True,
        )
    )

    # 9. R6 -- persisted phase and child facts select one safe next action.
    obs9 = resume.next_action if not isinstance(resume, Rejection) else "rejected"
    exp9 = DELETION_3095_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": DELETION_3095_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    status = project(retain, reversible, inventory)

    # 10. R7 -- status makes the deletion condition actionable.
    obs10 = status.condition
    exp10 = DELETION_3095_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": DELETION_3095_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R7 -- status makes the active-operation disposition actionable.
    obs11 = status.active_operation_disposition
    exp11 = DELETION_3095_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": DELETION_3095_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R7 -- status makes the retained inventory summary actionable.
    obs12 = status.retained_inventory_summary
    exp12 = DELETION_3095_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": DELETION_3095_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R7 -- status makes the next action actionable.
    obs13 = status.next_action
    exp13 = DELETION_3095_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": DELETION_3095_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    return {
        "case_id": "deletion-3095-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
