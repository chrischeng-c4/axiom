"""EC security case for #3095 -- fail-closed Retain-only deletion.

Expected values are EC-owned literals from #3095: R1 has no Destroy policy;
R2 refuses every named concurrent lifecycle mutation while a deletion is
claimed; R5 keeps the finalizer for ambiguous closed-UID inventory; R7 exposes
that block to an operator; and R8 turns unsafe waiting into DeletionBlocked,
never a destructive timeout.  Kubernetes I/O, restart survival, and a
break-glass runbook are runtime-only and intentionally absent.
"""

from __future__ import annotations

from lumen.deletion.admission import decide_mutation
from lumen.deletion.finalizer import decide_finalizer
from lumen.deletion.inventory import assess_closed_uid_inventory
from lumen.deletion.phase import decide_wait_outcome
from lumen.deletion.spec import (
    DeletionRequest,
    MutationRequest,
    RetainedResourceFacts,
    WaitFacts,
)
from lumen.deletion.status import project
from lumen.deletion.verdict import Rejection

MINIMUM_CHECKS = 16

DELETION_3095_SECURITY_MATRIX = (
    ("destroy_policy_is_refused_until_an_explicit_destroy_change", "destroy_not_supported"),
    ("destroy_policy_refusal_names_requested_policy", "requested_policy"),
    ("retain_policy_neighbour_is_admitted", "admitted"),
    ("upgrade_is_refused_while_deletion_is_claimed", "deletion_in_progress"),
    ("capacity_is_refused_while_deletion_is_claimed", "deletion_in_progress"),
    ("split_is_refused_while_deletion_is_claimed", "deletion_in_progress"),
    ("handoff_is_refused_while_deletion_is_claimed", "deletion_in_progress"),
    ("backup_is_refused_while_deletion_is_claimed", "deletion_in_progress"),
    ("restore_is_refused_while_deletion_is_claimed", "deletion_in_progress"),
    ("second_delete_instance_is_refused_while_deletion_is_claimed", "duplicate_deletion"),
    ("ambiguous_closed_uid_inventory_is_refused", "ambiguous_closed_uid_inventory"),
    ("ambiguous_inventory_forbids_finalizer_removal", "retain_finalizer"),
    ("ambiguous_inventory_projects_deletion_blocked", "DeletionBlocked"),
    ("unsafe_wait_projects_deletion_blocked", "DeletionBlocked"),
    ("unsafe_wait_names_the_inventory_resolution_next_action", "resolve_closed_uid_inventory"),
    ("timed_out_unsafe_wait_never_selects_data_deletion", "DeletionBlocked"),
)


def _reason(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_deletion_3095_security() -> dict:
    checks = []

    destroy = decide_finalizer(DeletionRequest(instance_uid="uid-3095", requested_policy="Destroy"))

    # 1. R1 -- Destroy is explicitly refused until its own change exists.
    obs1 = _reason(destroy)
    exp1 = DELETION_3095_SECURITY_MATRIX[0][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- the refusal identifies the requested policy field.
    obs2 = destroy.field_path if isinstance(destroy, Rejection) else "admitted"
    exp2 = DELETION_3095_SECURITY_MATRIX[1][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1 -- the neighboring Retain policy remains admitted.
    retain = decide_finalizer(DeletionRequest(instance_uid="uid-3095", requested_policy="Retain"))
    obs3 = _reason(retain)
    exp3 = DELETION_3095_SECURITY_MATRIX[2][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R2 -- upgrade is explicitly supplied to deletion admission.
    upgrade = decide_mutation(MutationRequest(operation="upgrade", deletion_timestamp=True, already_claimed=True))
    obs4 = _reason(upgrade)
    exp4 = DELETION_3095_SECURITY_MATRIX[3][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R2 -- capacity is explicitly supplied to deletion admission.
    capacity = decide_mutation(MutationRequest(operation="capacity", deletion_timestamp=True, already_claimed=True))
    obs5 = _reason(capacity)
    exp5 = DELETION_3095_SECURITY_MATRIX[4][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R2 -- split is explicitly supplied to deletion admission.
    split = decide_mutation(MutationRequest(operation="split", deletion_timestamp=True, already_claimed=True))
    obs6 = _reason(split)
    exp6 = DELETION_3095_SECURITY_MATRIX[5][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R2 -- handoff is explicitly supplied to deletion admission.
    handoff = decide_mutation(MutationRequest(operation="handoff", deletion_timestamp=True, already_claimed=True))
    obs7 = _reason(handoff)
    exp7 = DELETION_3095_SECURITY_MATRIX[6][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R2 -- backup is explicitly supplied to deletion admission.
    backup = decide_mutation(MutationRequest(operation="backup", deletion_timestamp=True, already_claimed=True))
    obs8 = _reason(backup)
    exp8 = DELETION_3095_SECURITY_MATRIX[7][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R2 -- restore is explicitly supplied to deletion admission.
    restore = decide_mutation(MutationRequest(operation="restore", deletion_timestamp=True, already_claimed=True))
    obs9 = _reason(restore)
    exp9 = DELETION_3095_SECURITY_MATRIX[8][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R2 -- a second DeleteInstance is explicitly supplied to admission.
    second_delete = decide_mutation(MutationRequest(operation="DeleteInstance", deletion_timestamp=True, already_claimed=True))
    obs10 = _reason(second_delete)
    exp10 = DELETION_3095_SECURITY_MATRIX[9][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    ambiguous = assess_closed_uid_inventory(
        RetainedResourceFacts(
            closed_uid="uid-3095",
            authoritative_pvcs=("data-0",),
            complete_backup_sets=(),
            ambiguous=True,
        )
    )

    # 11. R5 -- ambiguous inventory is a typed refusal.
    obs11 = _reason(ambiguous)
    exp11 = DELETION_3095_SECURITY_MATRIX[10][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R5 -- the same ambiguity forbids finalizer removal.
    obs12 = ambiguous.finalizer_disposition if isinstance(ambiguous, Rejection) else "admitted"
    exp12 = DELETION_3095_SECURITY_MATRIX[11][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R7 -- the block is projected as an actionable status condition.
    blocked_status = project(retain, None, ambiguous)
    obs13 = blocked_status.condition
    exp13 = DELETION_3095_SECURITY_MATRIX[12][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    unsafe_wait = decide_wait_outcome(
        WaitFacts(waiting_for="closed_uid_inventory", unsafe=True, timeout_elapsed=False)
    )

    # 14. R8 -- unsafe waiting reaches a typed block.
    obs14 = unsafe_wait.condition if not isinstance(unsafe_wait, Rejection) else unsafe_wait.reason.value
    exp14 = DELETION_3095_SECURITY_MATRIX[13][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R8 -- that block states the action which resolves it.
    obs15 = unsafe_wait.next_action if not isinstance(unsafe_wait, Rejection) else "rejected"
    exp15 = DELETION_3095_SECURITY_MATRIX[14][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    timed_out_unsafe_wait = decide_wait_outcome(
        WaitFacts(waiting_for="closed_uid_inventory", unsafe=True, timeout_elapsed=True)
    )

    # 16. R8 -- a timeout changes no data-retention policy: it remains blocked.
    obs16 = timed_out_unsafe_wait.condition if not isinstance(timed_out_unsafe_wait, Rejection) else timed_out_unsafe_wait.reason.value
    exp16 = DELETION_3095_SECURITY_MATRIX[15][1]
    checks.append({"name": DELETION_3095_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    return {
        "case_id": "deletion-3095-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
