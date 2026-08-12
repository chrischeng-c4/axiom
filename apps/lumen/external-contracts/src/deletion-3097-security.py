"""EC security case for #3097 -- deletion refuses ambiguity and overclaim.

Expected values are EC-owned literals from #3097: R1 closes the policy
vocabulary; R2 blocks ordinary reconcile; R3/AC4 make every missing, extra,
unlabeled, or contradictory inventory a DeletionBlocked refusal with no plan;
R4 denies unquiesced destruction; R5 blocks non-idempotent failure classes and
unknown keys; R6 never expands work; R7 preserves the finalizer; and R8 keeps
credentials out of the pure status projection.
"""

from __future__ import annotations

from dataclasses import asdict
import json

from lumen.deletion.inventory import ArtifactRecord, ClosedInventory, DeletionBlocked, close_inventory
from lumen.deletion.plan import plan_exact_destruction
from lumen.deletion.policy import DataDeletionPolicy, DeletionLifecycle, decide_deletion_start, effective_data_deletion_policy
from lumen.deletion.progress import ArtifactDeleteResult, ArtifactProgress, apply_artifact_result, may_remove_finalizer, resume_closed_inventory
from lumen.deletion.status import project_deletion_status

MINIMUM_CHECKS = 19

DELETION_3097_SECURITY_MATRIX = (
    ("unknown_policy_is_refused", "DeletionBlocked"),
    ("unknown_policy_refusal_names_the_policy_field", "dataDeletionPolicy"),
    ("explicit_delete_is_the_neighbouring_admitted_policy", "Delete"),
    ("ordinary_reconcile_with_delete_is_not_destructive", "non_destructive"),
    ("missing_catalog_artifact_blocks_closure", "missing"),
    ("extra_selector_artifact_blocks_without_a_plan", None),
    ("unlabeled_artifact_blocks_closure", "unlabeled"),
    ("contradictory_manifest_artifact_blocks_closure", "contradictory"),
    ("unquiesced_inventory_is_refused", "DeletionBlocked"),
    ("destruction_targets_have_only_exact_inventory_uids", ("uid-stage-7", "uid-pvc-7", "uid-backup-7", "uid-meta-7")),
    ("destruction_target_vocabulary_has_no_prefix_or_namespace_sweep", ()),
    ("unknown_key_not_found_is_blocked", "DeletionBlocked"),
    ("permission_failure_leaves_artifact_nonterminal", "pending"),
    ("auth_failure_is_blocked", "DeletionBlocked"),
    ("checksum_failure_is_blocked", "DeletionBlocked"),
    ("resume_cannot_expand_to_newly_discovered_uid", ("uid-pvc-7", "uid-stage-7", "uid-backup-7", "uid-meta-7")),
    ("remaining_non_data_child_retains_finalizer", "retain_finalizer"),
    ("one_pending_closed_artifact_retains_finalizer", "retain_finalizer"),
    ("status_refusal_is_credential_free_and_uses_deletionblocked", ("DeletionBlocked", False)),
)


def _artifacts() -> tuple[ArtifactRecord, ...]:
    return (
        ArtifactRecord(uid="uid-pvc-7", role="pvc", generation=17, manifest_key=None),
        ArtifactRecord(uid="uid-stage-7", role="staging", generation=17, manifest_key=None),
        ArtifactRecord(uid="uid-backup-7", role="backup", generation=17, manifest_key="backups/orders/7"),
        ArtifactRecord(uid="uid-meta-7", role="metadata", generation=17, manifest_key=None),
    )


def _closed() -> ClosedInventory:
    inventory = close_inventory(_artifacts(), _artifacts(), _artifacts())
    assert isinstance(inventory, ClosedInventory)
    return inventory


def verify_deletion_3097_security() -> dict:
    checks = []

    # 1. R1 -- the policy vocabulary is closed.
    invalid_policy = effective_data_deletion_policy("Purge")
    obs1 = invalid_policy.reason.value if isinstance(invalid_policy, DeletionBlocked) else "admitted"
    exp1 = DELETION_3097_SECURITY_MATRIX[0][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- the policy refusal names the user's field.
    obs2 = invalid_policy.field_path if isinstance(invalid_policy, DeletionBlocked) else ""
    exp2 = DELETION_3097_SECURITY_MATRIX[1][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1 -- explicit Delete remains the admitted neighbouring value.
    admitted_delete = effective_data_deletion_policy(DataDeletionPolicy.DELETE)
    obs3 = admitted_delete.value if isinstance(admitted_delete, DataDeletionPolicy) else "refused"
    exp3 = DELETION_3097_SECURITY_MATRIX[2][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R2 -- an explicitly destructive policy is still inert in reconcile.
    obs4 = decide_deletion_start(DataDeletionPolicy.DELETE, DeletionLifecycle.RECONCILING).phase.value
    exp4 = DELETION_3097_SECURITY_MATRIX[3][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R3/AC4 -- a missing catalog artifact blocks the whole closure.
    missing = close_inventory(_artifacts(), _artifacts()[:-1], _artifacts())
    obs5 = missing.category.value if isinstance(missing, DeletionBlocked) else "closed"
    exp5 = DELETION_3097_SECURITY_MATRIX[4][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R3/AC4 -- an extra selector artifact produces no partial plan.
    extra = close_inventory(_artifacts(), _artifacts() + (ArtifactRecord(uid="uid-foreign-8", role="pvc", generation=17, manifest_key=None),), _artifacts())
    obs6 = extra.plan if isinstance(extra, DeletionBlocked) else "closed"
    exp6 = DELETION_3097_SECURITY_MATRIX[5][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R3 -- an unlabeled artifact blocks closure.
    unlabeled = close_inventory(_artifacts(), _artifacts() + (ArtifactRecord(uid="", role="pvc", generation=17, manifest_key=None),), _artifacts())
    obs7 = unlabeled.category.value if isinstance(unlabeled, DeletionBlocked) else "closed"
    exp7 = DELETION_3097_SECURITY_MATRIX[6][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R3 -- contradictory BackupSet facts block closure.
    contradictory_manifest = _artifacts()[:-1] + (ArtifactRecord(uid="uid-meta-7", role="backup", generation=17, manifest_key="backups/orders/7"),)
    contradictory = close_inventory(_artifacts(), _artifacts(), contradictory_manifest)
    obs8 = contradictory.category.value if isinstance(contradictory, DeletionBlocked) else "closed"
    exp8 = DELETION_3097_SECURITY_MATRIX[7][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R4 -- plan entry refuses an explicitly unquiesced inventory.
    unquiesced = plan_exact_destruction(_closed(), quiesced=False)
    obs9 = unquiesced.reason.value if isinstance(unquiesced, DeletionBlocked) else "planned"
    exp9 = DELETION_3097_SECURITY_MATRIX[8][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. AC2 -- a plan can never widen from exact inventory UIDs.
    exact_plan = plan_exact_destruction(_closed(), quiesced=True)
    obs10 = tuple(target.uid for target in exact_plan.targets)
    exp10 = DELETION_3097_SECURITY_MATRIX[9][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R4/AC2 -- exact-target records have no prefix or namespace sweep.
    obs11 = tuple(sorted({field for target in exact_plan.targets for field in getattr(type(target), "__dataclass_fields__", ()) if field in {"prefix", "namespace"}}))
    exp11 = DELETION_3097_SECURITY_MATRIX[10][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R5 -- not-found for an unknown key is never idempotent success.
    progress = ArtifactProgress.for_inventory(_closed())
    unknown_key = apply_artifact_result(progress, ArtifactRecord(uid="uid-unknown", role="backup", generation=17, manifest_key="backups/other"), ArtifactDeleteResult.NOT_FOUND)
    obs12 = unknown_key.reason.value if isinstance(unknown_key, DeletionBlocked) else "deleted"
    exp12 = DELETION_3097_SECURITY_MATRIX[11][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R5 -- a permission failure leaves the exact artifact nonterminal.
    permission = apply_artifact_result(progress, _artifacts()[2], ArtifactDeleteResult.PERMISSION)
    obs13 = permission.artifact_states["uid-backup-7"].value if not isinstance(permission, DeletionBlocked) else "blocked"
    exp13 = DELETION_3097_SECURITY_MATRIX[12][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R5 -- authentication failure also preserves the closed work.
    auth = apply_artifact_result(progress, _artifacts()[2], ArtifactDeleteResult.AUTH)
    obs14 = auth.reason.value if isinstance(auth, DeletionBlocked) else "deleted"
    exp14 = DELETION_3097_SECURITY_MATRIX[13][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R5 -- a checksum failure is blocking, not an idempotent deletion.
    checksum = apply_artifact_result(progress, _artifacts()[2], ArtifactDeleteResult.CHECKSUM)
    obs15 = checksum.reason.value if isinstance(checksum, DeletionBlocked) else "deleted"
    exp15 = DELETION_3097_SECURITY_MATRIX[14][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16. R6/AC3 -- resume has no discovery input and returns no novel UID.
    resumed = resume_closed_inventory(_closed(), progress)
    obs16 = tuple(target.uid for target in resumed.remaining)
    exp16 = DELETION_3097_SECURITY_MATRIX[15][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    # 17. R7 -- a remaining non-data child retains the finalizer.
    completed = ArtifactProgress.from_states(_closed(), {artifact.uid: "deleted" for artifact in _artifacts()})
    obs17 = "remove_finalizer" if may_remove_finalizer(completed, non_data_children_gone=False) else "retain_finalizer"
    exp17 = DELETION_3097_SECURITY_MATRIX[16][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    # 18. R7 -- one pending exact artifact also retains the finalizer.
    obs18 = "remove_finalizer" if may_remove_finalizer(progress, non_data_children_gone=True) else "retain_finalizer"
    exp18 = DELETION_3097_SECURITY_MATRIX[17][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    # 19. R8 -- a blocked projection says why without copying an opaque
    # credential-shaped value that appeared in an exact-but-unknown key.
    credential_sentinel = "credential-sentinel-3097-never-project"
    unknown_sentinel_key = apply_artifact_result(
        progress,
        ArtifactRecord(
            uid="uid-unknown-sentinel",
            role="backup",
            generation=17,
            manifest_key=f"backups/{credential_sentinel}",
        ),
        ArtifactDeleteResult.NOT_FOUND,
    )
    status = project_deletion_status(_closed(), progress, generation=17, refusal=unknown_sentinel_key)
    serialized_status = json.dumps(asdict(status), default=str, sort_keys=True)
    obs19 = (status.refusal.reason.value, credential_sentinel in serialized_status)
    exp19 = DELETION_3097_SECURITY_MATRIX[18][1]
    checks.append({"name": DELETION_3097_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    return {"case_id": "deletion-3097-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
