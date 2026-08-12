"""EC behavior case for #3098 -- UID-bound Retain/Delete admission.

Every expected value below is an EC-owned literal transcribed from #3098:
R2/AC1 retain the exact authoritative PVC and complete BackupSet inventory
when policy is omitted; R4/AC3 select only the closed UID inventory for an
explicit ``Delete``; and R3/AC2 admit a retained complete BackupSet restore
only for a same-name *new* UID.  GKE disappearance, mounted volumes, query
parity, and evidence cleanup remain runtime-only and are deliberately absent.
"""

from __future__ import annotations

from lumen.deletion.admission import decide_deletion_inventory, decide_restore_target
from lumen.deletion.spec import (
    BackupSet,
    DeletionInventoryRequest,
    ResourceIdentity,
    RestoreTargetRequest,
)
from lumen.deletion.verdict import DeletionBlocked

MINIMUM_CHECKS = 8

DELETION_3098_BEHAVIOR_MATRIX = (
    ("omitted_retain_preserves_exact_authoritative_pvcs", (("uid-3098-old", "pvc", "data-0"), ("uid-3098-old", "pvc", "data-1"))),
    ("omitted_retain_preserves_exact_complete_backup_sets", (("uid-3098-old", "BackupSet", "backup-3098"),)),
    ("omitted_retain_selects_only_public_and_compute_resources", (("uid-3098-old", "service", "search"), ("uid-3098-old", "statefulset", "lumen"))),
    ("explicit_delete_selects_exact_closed_uid_pvcs", (("uid-3098-old", "pvc", "data-0"), ("uid-3098-old", "pvc", "data-1"))),
    ("explicit_delete_selects_exact_closed_uid_complete_backup_sets", (("uid-3098-old", "BackupSet", "backup-3098"),)),
    ("explicit_delete_retains_same_name_other_uid_and_unrelated_identities", (("uid-3098-new", "pvc", "data-0"), ("uid-other", "pvc", "data-0"))),
    ("complete_backup_set_restore_is_admitted_for_new_uid", "admitted"),
    ("admitted_restore_target_is_bound_to_the_new_uid", "uid-3098-new"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, DeletionBlocked) else "admitted"


def _identities(resources) -> tuple[tuple[str, str, str], ...]:
    return tuple((resource.uid, resource.kind, resource.name) for resource in resources)


def verify_deletion_3098_behavior() -> dict:
    checks = []

    old_pvc_0 = ResourceIdentity(uid="uid-3098-old", kind="pvc", name="data-0")
    old_pvc_1 = ResourceIdentity(uid="uid-3098-old", kind="pvc", name="data-1")
    old_backup = BackupSet(uid="uid-3098-old", name="backup-3098", complete=True)
    omitted_retain = decide_deletion_inventory(
        DeletionInventoryRequest(
            instance_uid="uid-3098-old",
            policy=None,
            public_resources=(ResourceIdentity(uid="uid-3098-old", kind="service", name="search"),),
            compute_resources=(ResourceIdentity(uid="uid-3098-old", kind="statefulset", name="lumen"),),
            pvcs=(old_pvc_0, old_pvc_1),
            backup_sets=(old_backup,),
            inventory_complete=True,
            inventory_labeled=True,
            inventory_ambiguous=False,
        )
    )

    # 1. R2/AC1 -- omission selects Retain and preserves the authoritative
    #    closed-UID PVC identity set rather than a controller-derived subset.
    obs1 = _identities(omitted_retain.retained_pvcs) if not isinstance(omitted_retain, DeletionBlocked) else ()
    exp1 = DELETION_3098_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": DELETION_3098_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R2/AC1 -- complete BackupSets are a separately retained durable class.
    obs2 = _identities(omitted_retain.retained_backup_sets) if not isinstance(omitted_retain, DeletionBlocked) else ()
    exp2 = DELETION_3098_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": DELETION_3098_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R2/AC1 -- Retain still selects public and compute resources, but not
    #    the durable identities observed in the preceding two rows.
    obs3 = _identities(omitted_retain.selected_destructive) if not isinstance(omitted_retain, DeletionBlocked) else ()
    exp3 = DELETION_3098_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": DELETION_3098_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    delete = decide_deletion_inventory(
        DeletionInventoryRequest(
            instance_uid="uid-3098-old",
            policy="Delete",
            public_resources=(ResourceIdentity(uid="uid-3098-old", kind="service", name="search"),),
            compute_resources=(ResourceIdentity(uid="uid-3098-old", kind="statefulset", name="lumen"),),
            pvcs=(old_pvc_0, old_pvc_1),
            backup_sets=(old_backup,),
            inventory_complete=True,
            inventory_labeled=True,
            inventory_ambiguous=False,
            counterexample_resources=(
                ResourceIdentity(uid="uid-3098-new", kind="pvc", name="data-0"),
                ResourceIdentity(uid="uid-other", kind="pvc", name="data-0"),
            ),
        )
    )

    # 4. R4/AC3 -- explicit Delete closes over both old-UID PVC identities.
    obs4 = _identities(delete.selected_pvcs) if not isinstance(delete, DeletionBlocked) else ()
    exp4 = DELETION_3098_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": DELETION_3098_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R4/AC3 -- it separately closes over the complete old-UID BackupSet.
    obs5 = _identities(delete.selected_backup_sets) if not isinstance(delete, DeletionBlocked) else ()
    exp5 = DELETION_3098_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": DELETION_3098_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R4/AC3 -- supplied same-name other-UID and unrelated counterexamples
    #    remain outside the selected destructive closure.
    obs6 = _identities(delete.retained_counterexamples) if not isinstance(delete, DeletionBlocked) else ()
    exp6 = DELETION_3098_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": DELETION_3098_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    restore = decide_restore_target(
        RestoreTargetRequest(
            old_uid="uid-3098-old",
            new_uid="uid-3098-new",
            backup_set=old_backup,
            mount_pvc=None,
        )
    )

    # 7. R3/AC2 -- a retained complete BackupSet admits recovery for a new UID.
    obs7 = _outcome(restore)
    exp7 = DELETION_3098_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": DELETION_3098_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R3/AC2 -- the admitted target explicitly carries that new UID.
    obs8 = restore.target_uid if not isinstance(restore, DeletionBlocked) else "blocked"
    exp8 = DELETION_3098_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": DELETION_3098_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    return {
        "case_id": "deletion-3098-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
