"""EC security case for #2962 -- fail-closed topology backup and restore.

Every expected value is an EC-owned literal transcribed from #2962: R1/R2
reject incomplete or generation-inconsistent backup candidates and wait during
splits or handoffs; R4 refuses unsupported topology rather than classifying a
local snapshot as cluster backup; R5 rejects non-empty or ready restore targets;
R6 returns an unready, actionable failure while retaining the prior successful
set; and R8 never projects schedule configuration as successful backup status.
"""

from __future__ import annotations

from lumen.topology.backup.admission import (
    decide_backup_barrier,
    decide_backup_set,
    decide_coordinator_route,
)
from lumen.topology.backup.restore import advance_restore, plan_restore
from lumen.topology.backup.spec import (
    BackupSetMetadata,
    CatalogArtifact,
    JobOutcome,
    RestoreCompatibility,
    RestoreEvent,
    RestoreState,
    RestoreTarget,
    ShardArtifact,
    TopologyState,
)
from lumen.topology.backup.status import project_backup_status
from lumen.topology.backup.verdict import Rejection

MINIMUM_CHECKS = 19

SPEC_2962_SECURITY_MATRIX = (
    ("missing_shard_watermark_is_rejected", "missing_applied_watermark"),
    ("missing_shard_watermark_refusal_names_shard_field", "shards.orders.applied_watermark"),
    ("complete_neighbouring_metadata_is_admitted", "admitted"),
    ("generation_mismatch_is_rejected", "artifact_generation_mismatch"),
    ("generation_mismatch_names_artifact_generation", "artifact_generations"),
    ("split_topology_waits_instead_of_permitting", "wait"),
    ("handoff_topology_waits_instead_of_permitting", "wait"),
    ("unsupported_topology_is_explicitly_rejected", "unsupported_topology"),
    ("unsupported_route_names_request_target", "request_target"),
    ("ready_target_is_rejected", "target_ready"),
    ("ready_target_refusal_names_ready_field", "target.ready"),
    ("nonempty_target_is_rejected", "target_not_empty"),
    ("missing_restore_input_is_unready", "unready"),
    ("duplicate_restore_input_is_unready", "unready"),
    ("incompatible_restore_input_is_unready", "unready"),
    ("corrupt_restore_input_is_unready", "unready"),
    ("corrupt_apply_names_catalog_shard_and_generation", ("catalog", "orders", "target-gen-9")),
    ("failure_preserves_prior_successful_set", "set-6"),
    ("configured_schedule_is_not_backup_success", "not_success"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_spec_2962_security() -> dict:
    checks = []
    complete_metadata = BackupSetMetadata(
        catalog_generation="catalog-gen-7",
        catalog=CatalogArtifact("catalog-checkpoint-7", "catalog-watermark-7", "schema-gen-7", RestoreCompatibility("lumen-restore-v1")),
        shards=(
            ShardArtifact("orders", "orders-checkpoint-7", "orders-watermark-7", "schema-gen-7", RestoreCompatibility("lumen-restore-v1")),
            ShardArtifact("users", "users-checkpoint-7", "users-watermark-7", "schema-gen-7", RestoreCompatibility("lumen-restore-v1")),
        ),
    )
    missing_watermark = BackupSetMetadata(
        catalog_generation="catalog-gen-7",
        catalog=complete_metadata.catalog,
        shards=(
            ShardArtifact("orders", "orders-checkpoint-7", None, "schema-gen-7", RestoreCompatibility("lumen-restore-v1")),
            complete_metadata.shards[1],
        ),
    )
    missing_verdict = decide_backup_set(missing_watermark)

    # 1. R1 -- an explicit missing watermark is rejected, rather than accepted by a default.
    obs1 = _outcome(missing_verdict)
    exp1 = SPEC_2962_SECURITY_MATRIX[0][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- refusal identifies the exact metadata field the operator must repair.
    obs2 = missing_verdict.field_path if isinstance(missing_verdict, Rejection) else "admitted"
    exp2 = SPEC_2962_SECURITY_MATRIX[1][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1 -- the nearest complete metadata candidate remains admissible.
    complete_verdict = decide_backup_set(complete_metadata)
    obs3 = _outcome(complete_verdict)
    exp3 = SPEC_2962_SECURITY_MATRIX[2][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    mismatch = TopologyState("catalog-gen-7", ("catalog-gen-7", "catalog-gen-8", "catalog-gen-7"), "stable", "lumen-topology-coordinator")
    mismatch_verdict = decide_backup_barrier(mismatch, complete_metadata)

    # 4. R2 -- a single artifact from another generation cannot cross the barrier.
    obs4 = mismatch_verdict.outcome
    exp4 = SPEC_2962_SECURITY_MATRIX[3][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R2 -- the barrier diagnosis names the mismatched generation dimension.
    obs5 = mismatch_verdict.field_path
    exp5 = SPEC_2962_SECURITY_MATRIX[4][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R2 -- an active split waits; it never becomes an optimistic permit.
    split = decide_backup_barrier(TopologyState("catalog-gen-7", ("catalog-gen-7",) * 3, "split", "lumen-topology-coordinator"), complete_metadata)
    obs6 = split.outcome
    exp6 = SPEC_2962_SECURITY_MATRIX[5][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R2 -- handoff receives the same explicit wait treatment at this entry point.
    handoff = decide_backup_barrier(TopologyState("catalog-gen-7", ("catalog-gen-7",) * 3, "handoff", "lumen-topology-coordinator"), complete_metadata)
    obs7 = handoff.outcome
    exp7 = SPEC_2962_SECURITY_MATRIX[6][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    unsupported = decide_coordinator_route(TopologyState("catalog-gen-7", (), "unsupported", None), request_target="lumen-public-1")

    # 8. R4 -- unsupported topology is a typed refusal, never local success.
    obs8 = _outcome(unsupported)
    exp8 = SPEC_2962_SECURITY_MATRIX[7][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R4 -- the routing refusal identifies the public request target.
    obs9 = unsupported.field_path if isinstance(unsupported, Rejection) else "admitted"
    exp9 = SPEC_2962_SECURITY_MATRIX[8][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    ready_target = plan_restore(RestoreTarget(empty=True, ready=True, generation="target-gen-9"), complete_metadata)

    # 10. R5 -- naming ready=True explicitly exercises rejection of a ready target.
    obs10 = _outcome(ready_target)
    exp10 = SPEC_2962_SECURITY_MATRIX[9][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R5 -- it also says which target state made restore unsafe.
    obs11 = ready_target.field_path if isinstance(ready_target, Rejection) else "admitted"
    exp11 = SPEC_2962_SECURITY_MATRIX[10][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    nonempty_target = plan_restore(RestoreTarget(empty=False, ready=False, generation="target-gen-9"), complete_metadata)

    # 12. R5 -- non-empty but unready is independently unsafe.
    obs12 = _outcome(nonempty_target)
    exp12 = SPEC_2962_SECURITY_MATRIX[11][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    missing = advance_restore(
        RestoreState.unready(previous_successful_set="set-6", generation="target-gen-9"),
        RestoreEvent(kind="missing_shard", artifact="orders-checkpoint-7", shard="orders", detail="object absent"),
    )

    # 13. R6 -- missing input cannot make a partial topology ready.
    obs13 = missing.readiness
    exp13 = SPEC_2962_SECURITY_MATRIX[12][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    duplicate = advance_restore(
        RestoreState.unready(previous_successful_set="set-6", generation="target-gen-9"),
        RestoreEvent(kind="duplicate_shard", artifact="orders-checkpoint-7", shard="orders", detail="two objects"),
    )

    # 14. R6 -- duplicate input is likewise an explicit unready failure.
    obs14 = duplicate.readiness
    exp14 = SPEC_2962_SECURITY_MATRIX[13][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    incompatible = advance_restore(
        RestoreState.unready(previous_successful_set="set-6", generation="target-gen-9"),
        RestoreEvent(kind="incompatible_shard", artifact="orders-checkpoint-7", shard="orders", detail="restore-v2"),
    )

    # 15. R6 -- restore compatibility cannot be ignored during apply.
    obs15 = incompatible.readiness
    exp15 = SPEC_2962_SECURITY_MATRIX[14][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    failed = advance_restore(
        RestoreState.unready(previous_successful_set="set-6", generation="target-gen-9"),
        RestoreEvent(kind="corrupt_catalog", artifact="catalog-checkpoint-7", shard="orders", detail="crc mismatch"),
    )

    # 16. R6/AC2 -- corrupt input remains unready, never an incomplete ready topology.
    obs16 = failed.readiness
    exp16 = SPEC_2962_SECURITY_MATRIX[15][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    # 17. R6/AC3 -- failure detail identifies the catalog, shard, and target generation.
    obs17 = (failed.catalog_detail, failed.shard_detail, failed.generation_detail)
    exp17 = SPEC_2962_SECURITY_MATRIX[16][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    # 18. R6/AC2 -- failure retains the previously successful complete set.
    obs18 = failed.previous_successful_set
    exp18 = SPEC_2962_SECURITY_MATRIX[17][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    # 19. R8 -- schedule configuration without a verified manifest and successful Job is not success.
    scheduled = project_backup_status(verified_manifest=None, job_outcome=JobOutcome(configured_schedule="daily", succeeded=False))
    obs19 = scheduled.classification
    exp19 = SPEC_2962_SECURITY_MATRIX[18][1]
    checks.append({"name": SPEC_2962_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    return {
        "case_id": "spec-2962-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
