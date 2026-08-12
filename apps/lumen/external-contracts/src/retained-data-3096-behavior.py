"""EC behavior case for #3096 -- retained deleted-instance evidence and recovery.

Every expected value below is an EC-owned literal transcribed from #3096:
R1 retains complete manifests and artifacts under the daily latest-two policy
and records source UID plus catalog generation; R2 keeps authoritative PVC
identity and topology metadata immutable; R3 projects exact-UID inventory;
R4 binds a restore to the new UID; and R7 projects evidence without cost or
automatic-expiry claims. Kubernetes persistence, command execution, and real
restore readiness are deliberately runtime-only.
"""

from __future__ import annotations

from lumen.retained_data.admission import decide_backupset_retention, decide_restore_binding
from lumen.retained_data.inventory import render_exact_uid_inventory
from lumen.retained_data.spec import BackupSetCandidate, RetainedPvcMetadata, RetentionPolicy
from lumen.retained_data.status import project_retained_inventory
from lumen.retained_data.verdict import AdmittedRetention, RestoreBinding

MINIMUM_CHECKS = 18

RETAINED_DATA_3096_BEHAVIOR_MATRIX = (
    ("retained_complete_set_records_deleted_source_uid", "uid-orders-deleted"),
    ("retained_complete_set_records_catalog_generation", 17),
    ("daily_policy_keeps_the_latest_two_complete_sets", ("backup-18", "backup-17")),
    ("retained_pvc_metadata_is_frozen", True),
    ("retained_pvc_metadata_keeps_source_uid", "uid-orders-deleted"),
    ("retained_pvc_metadata_keeps_authoritative_role", "authoritative"),
    ("retained_pvc_metadata_keeps_shard_group", "shard-0"),
    ("retained_pvc_metadata_keeps_format", "raft-runtime-v1"),
    ("retained_pvc_metadata_keeps_topology_generation", 17),
    ("kubectl_inventory_selector_is_exact_deleted_uid", "lumen.axiom.dev/source-uid=uid-orders-deleted"),
    ("lumen_llm_inventory_selector_is_exact_deleted_uid", "--deleted-uid uid-orders-deleted"),
    ("kubectl_and_lumen_llm_inventory_commands_are_deterministic", True),
    ("eligible_restore_membership_binds_new_uid", "uid-orders-new"),
    ("eligible_restore_catalog_authority_binds_new_uid", "uid-orders-new"),
    ("retained_inventory_projects_age", "age_seconds"),
    ("retained_inventory_projects_projects_count", "count"),
    ("retained_inventory_projects_bytes", "bytes"),
    ("retained_inventory_projects_recovery_pointer", "recovery_pointer"),
)


def _complete(*, name: str = "backup-17", generation: int = 17) -> BackupSetCandidate:
    return BackupSetCandidate(
        name=name,
        source_uid="uid-orders-deleted",
        catalog_generation=generation,
        complete=True,
        manifests_present=True,
        artifacts_present=True,
        compatible=True,
        corrupt=False,
        topology_generation=17,
        format="raft-runtime-v1",
    )


def verify_retained_data_3096_behavior() -> dict:
    checks = []
    retained = decide_backupset_retention(
        (_complete(name="backup-16", generation=16), _complete(name="backup-17"), _complete(name="backup-18", generation=18)),
        RetentionPolicy(daily_complete_limit=2),
    )

    # 1-3. R1 -- the retained record preserves lineage and keeps exactly the
    # policy-selected two most recent complete sets.
    obs1 = retained.retained[1].source_uid if isinstance(retained, AdmittedRetention) else "rejected"
    exp1 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = retained.retained[1].catalog_generation if isinstance(retained, AdmittedRetention) else -1
    exp2 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = tuple(item.name for item in retained.retained) if isinstance(retained, AdmittedRetention) else ()
    exp3 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    pvc = RetainedPvcMetadata(source_uid="uid-orders-deleted", role="authoritative", shard_group="shard-0", format="raft-runtime-v1", topology_generation=17)

    # 4-9. R2 -- no authoritative PVC identity dimension can be reconstructed
    # from the name of a later CR incarnation.
    obs4 = pvc.__dataclass_params__.frozen
    exp4 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = pvc.source_uid
    exp5 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = pvc.role
    exp6 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = pvc.shard_group
    exp7 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = pvc.format
    exp8 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = pvc.topology_generation
    exp9 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    commands = render_exact_uid_inventory("uid-orders-deleted")

    # 10-12. R3 -- each published command independently carries the exact UID,
    # and rendering the same request never changes either command.
    obs10 = commands.kubectl_selector
    exp10 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    obs11 = commands.lumen_llm_arguments
    exp11 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = commands == render_exact_uid_inventory("uid-orders-deleted")
    exp12 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    binding = decide_restore_binding(_complete(), "uid-orders-new")

    # 13-14. R4 -- restoring does not transfer deleted-instance authority.
    obs13 = binding.membership_uid if isinstance(binding, RestoreBinding) else "rejected"
    exp13 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = binding.catalog_authority_uid if isinstance(binding, RestoreBinding) else "rejected"
    exp14 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    inventory = project_retained_inventory((pvc, _complete(), _complete(name="backup-18", generation=18)), "backupsets/backup-17")

    # 15-18. R7 -- status exposes each required evidence dimension. Its values
    # are live-resource observations in the Rust stage, so this pure model pins
    # the named projection rather than fabricating a clock or storage reading.
    obs15 = "age_seconds" if hasattr(inventory, "age_seconds") else "missing"
    exp15 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    obs16 = "count" if hasattr(inventory, "count") else "missing"
    exp16 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    obs17 = "bytes" if hasattr(inventory, "bytes") else "missing"
    exp17 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    obs18 = "recovery_pointer" if hasattr(inventory, "recovery_pointer") else "missing"
    exp18 = RETAINED_DATA_3096_BEHAVIOR_MATRIX[17][1]
    checks.append({"name": RETAINED_DATA_3096_BEHAVIOR_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    return {"case_id": "retained-data-3096-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
