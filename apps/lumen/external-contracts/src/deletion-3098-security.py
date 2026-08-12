"""EC security case for #3098 -- fail-closed deletion and restore admission.

Every expected value below is an EC-owned literal from #3098: R6/AC4 require
incomplete, unlabeled, and ambiguous inventories to reach ``DeletionBlocked``
with no destructive subset; R3/AC2 require a new UID and prohibit an old-UID
PVC mount during restore.  Restart, handover, timeout, and object-store fault
execution are runtime-only and intentionally absent from this pure design case.
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

MINIMUM_CHECKS = 15

DELETION_3098_SECURITY_MATRIX = (
    ("incomplete_inventory_is_deletion_blocked", "incomplete_inventory"),
    ("incomplete_inventory_refusal_names_inventory_completeness", "inventory_complete"),
    ("incomplete_inventory_selects_no_destructive_subset", ()),
    ("unlabeled_inventory_is_deletion_blocked", "unlabeled_inventory"),
    ("unlabeled_inventory_refusal_names_inventory_labels", "inventory_labeled"),
    ("unlabeled_inventory_selects_no_destructive_subset", ()),
    ("ambiguous_inventory_is_deletion_blocked", "ambiguous_inventory"),
    ("ambiguous_inventory_refusal_names_inventory_ambiguity", "inventory_ambiguous"),
    ("ambiguous_inventory_selects_no_destructive_subset", ()),
    ("old_uid_pvc_restore_mount_is_deletion_blocked", "old_uid_pvc_mount"),
    ("old_uid_pvc_restore_refusal_names_mount_pvc", "mount_pvc"),
    ("same_uid_restore_target_is_deletion_blocked", "new_uid_required"),
    ("same_uid_restore_refusal_names_new_uid", "new_uid"),
    ("complete_labeled_unambiguous_delete_neighbour_is_admitted", "admitted"),
    ("new_uid_restore_without_old_pvc_neighbour_is_admitted", "admitted"),
)


def _reason(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, DeletionBlocked) else "admitted"


def verify_deletion_3098_security() -> dict:
    checks = []

    incomplete = decide_deletion_inventory(
        DeletionInventoryRequest(
            instance_uid="uid-3098-old",
            policy="Delete",
            public_resources=(ResourceIdentity(uid="uid-3098-old", kind="service", name="search"),),
            compute_resources=(ResourceIdentity(uid="uid-3098-old", kind="statefulset", name="lumen"),),
            pvcs=(ResourceIdentity(uid="uid-3098-old", kind="pvc", name="data-0"),),
            backup_sets=(BackupSet(uid="uid-3098-old", name="backup-3098", complete=True),),
            inventory_complete=False,
            inventory_labeled=True,
            inventory_ambiguous=False,
        )
    )

    # 1-3. R6/AC4 -- an explicitly incomplete Delete inventory is blocked,
    # its reason identifies completeness, and it cannot select a subset.
    obs1 = _reason(incomplete)
    exp1 = DELETION_3098_SECURITY_MATRIX[0][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = incomplete.field_path if isinstance(incomplete, DeletionBlocked) else "admitted"
    exp2 = DELETION_3098_SECURITY_MATRIX[1][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = incomplete.selected_destructive if isinstance(incomplete, DeletionBlocked) else ("unexpected_admission",)
    exp3 = DELETION_3098_SECURITY_MATRIX[2][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    unlabeled = decide_deletion_inventory(
        DeletionInventoryRequest(
            instance_uid="uid-3098-old",
            policy="Delete",
            public_resources=(ResourceIdentity(uid="uid-3098-old", kind="service", name="search"),),
            compute_resources=(ResourceIdentity(uid="uid-3098-old", kind="statefulset", name="lumen"),),
            pvcs=(ResourceIdentity(uid="uid-3098-old", kind="pvc", name="data-0"),),
            backup_sets=(BackupSet(uid="uid-3098-old", name="backup-3098", complete=True),),
            inventory_complete=True,
            inventory_labeled=False,
            inventory_ambiguous=False,
        )
    )

    # 4-6. R6/AC4 -- an explicitly unlabeled inventory gets its own refusal;
    # it cannot silently use the incomplete-inventory path or delete anything.
    obs4 = _reason(unlabeled)
    exp4 = DELETION_3098_SECURITY_MATRIX[3][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = unlabeled.field_path if isinstance(unlabeled, DeletionBlocked) else "admitted"
    exp5 = DELETION_3098_SECURITY_MATRIX[4][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = unlabeled.selected_destructive if isinstance(unlabeled, DeletionBlocked) else ("unexpected_admission",)
    exp6 = DELETION_3098_SECURITY_MATRIX[5][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    ambiguous = decide_deletion_inventory(
        DeletionInventoryRequest(
            instance_uid="uid-3098-old",
            policy="Delete",
            public_resources=(ResourceIdentity(uid="uid-3098-old", kind="service", name="search"),),
            compute_resources=(ResourceIdentity(uid="uid-3098-old", kind="statefulset", name="lumen"),),
            pvcs=(ResourceIdentity(uid="uid-3098-old", kind="pvc", name="data-0"),),
            backup_sets=(BackupSet(uid="uid-3098-old", name="backup-3098", complete=True),),
            inventory_complete=True,
            inventory_labeled=True,
            inventory_ambiguous=True,
        )
    )

    # 7-9. R6/AC4 -- ambiguity is independently visible and selects no
    # destructive subset, even though every other input is explicitly valid.
    obs7 = _reason(ambiguous)
    exp7 = DELETION_3098_SECURITY_MATRIX[6][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = ambiguous.field_path if isinstance(ambiguous, DeletionBlocked) else "admitted"
    exp8 = DELETION_3098_SECURITY_MATRIX[7][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = ambiguous.selected_destructive if isinstance(ambiguous, DeletionBlocked) else ("unexpected_admission",)
    exp9 = DELETION_3098_SECURITY_MATRIX[8][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    old_uid_mount = decide_restore_target(
        RestoreTargetRequest(
            old_uid="uid-3098-old",
            new_uid="uid-3098-new",
            backup_set=BackupSet(uid="uid-3098-old", name="backup-3098", complete=True),
            mount_pvc=ResourceIdentity(uid="uid-3098-old", kind="pvc", name="data-0"),
        )
    )

    # 10-11. R3/AC2 -- an old-UID PVC is explicitly supplied as the mount
    # target and must be refused with the field that names the unsafe target.
    obs10 = _reason(old_uid_mount)
    exp10 = DELETION_3098_SECURITY_MATRIX[9][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    obs11 = old_uid_mount.field_path if isinstance(old_uid_mount, DeletionBlocked) else "admitted"
    exp11 = DELETION_3098_SECURITY_MATRIX[10][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    same_uid = decide_restore_target(
        RestoreTargetRequest(
            old_uid="uid-3098-old",
            new_uid="uid-3098-old",
            backup_set=BackupSet(uid="uid-3098-old", name="backup-3098", complete=True),
            mount_pvc=None,
        )
    )

    # 12-13. R3/AC2 -- the restore entry point separately requires a new UID;
    # it may not admit same-name recovery by reusing the old identity.
    obs12 = _reason(same_uid)
    exp12 = DELETION_3098_SECURITY_MATRIX[11][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = same_uid.field_path if isinstance(same_uid, DeletionBlocked) else "admitted"
    exp13 = DELETION_3098_SECURITY_MATRIX[12][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    safe_delete = decide_deletion_inventory(
        DeletionInventoryRequest(
            instance_uid="uid-3098-old",
            policy="Delete",
            public_resources=(ResourceIdentity(uid="uid-3098-old", kind="service", name="search"),),
            compute_resources=(ResourceIdentity(uid="uid-3098-old", kind="statefulset", name="lumen"),),
            pvcs=(ResourceIdentity(uid="uid-3098-old", kind="pvc", name="data-0"),),
            backup_sets=(BackupSet(uid="uid-3098-old", name="backup-3098", complete=True),),
            inventory_complete=True,
            inventory_labeled=True,
            inventory_ambiguous=False,
        )
    )

    # 14. R6/AC4 -- the three refusal inputs are meaningful constraints rather
    # than a blanket deletion ban: their complete/labeled/unambiguous neighbour
    # is admitted at the very same inventory entry point.
    obs14 = _reason(safe_delete)
    exp14 = DELETION_3098_SECURITY_MATRIX[13][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    safe_restore = decide_restore_target(
        RestoreTargetRequest(
            old_uid="uid-3098-old",
            new_uid="uid-3098-new",
            backup_set=BackupSet(uid="uid-3098-old", name="backup-3098", complete=True),
            mount_pvc=None,
        )
    )

    # 15. R3/AC2 -- the restore refusals do not prevent a new-UID target that
    # avoids an old-UID PVC mount from being admitted at that entry point.
    obs15 = _reason(safe_restore)
    exp15 = DELETION_3098_SECURITY_MATRIX[14][1]
    checks.append({"name": DELETION_3098_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "deletion-3098-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
