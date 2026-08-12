"""EC behavior case for #2962 -- topology-consistent BackupSet decisions.

Every expected value below is an EC-owned literal transcribed from #2962:
R1 pins one catalog generation and inventories the catalog plus every shard;
R2 permits only a stable, generation-consistent barrier; R4 resolves a public
request to the topology coordinator; R5 orders catalog and all shards before
membership rebind; R6 makes valid replay idempotent; R7 supplies the daily,
two-successful-set default and preserves an explicit policy; and R8 exposes
only verified-manifest/actual-Job success.
"""

from __future__ import annotations

from lumen.topology.backup.admission import (
    decide_backup_barrier,
    decide_backup_set,
    decide_coordinator_route,
)
from lumen.topology.backup.policy import derive_schedule_policy
from lumen.topology.backup.restore import advance_restore, plan_restore
from lumen.topology.backup.spec import (
    BackupManifest,
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

MINIMUM_CHECKS = 16

SPEC_2962_BEHAVIOR_MATRIX = (
    ("complete_metadata_is_admitted", "admitted"),
    ("admitted_metadata_pins_catalog_generation", "catalog-gen-7"),
    ("admitted_metadata_carries_catalog_artifact", "catalog-checkpoint-7"),
    ("admitted_metadata_carries_every_logical_shard", ("orders", "users")),
    ("stable_consistent_barrier_is_permitted", "permit"),
    ("coordinator_route_names_designated_target", "lumen-topology-coordinator"),
    ("empty_unready_compatible_target_gets_catalog_first", "catalog"),
    ("restore_plan_applies_every_shard_before_rebind", ("orders", "users")),
    ("restore_plan_rebinds_target_generation_last", "target-gen-9"),
    ("duplicate_valid_apply_is_idempotent", "catalog_applied"),
    ("destination_default_schedule_is_daily", "daily"),
    ("destination_default_retains_two_successful_complete_sets", 2),
    ("explicit_operator_policy_is_preserved", "weekly"),
    ("verified_success_exposes_manifest_identifier", "manifest-7"),
    ("verified_success_exposes_manifest_time", "2026-08-12T00:00:00Z"),
    ("verified_success_exposes_manifest_generation", "catalog-gen-7"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_spec_2962_behavior() -> dict:
    checks = []
    metadata = BackupSetMetadata(
        catalog_generation="catalog-gen-7",
        catalog=CatalogArtifact(
            checkpoint="catalog-checkpoint-7",
            applied_watermark="catalog-watermark-7",
            schema_generation="schema-gen-7",
            restore_compatibility=RestoreCompatibility("lumen-restore-v1"),
        ),
        shards=(
            ShardArtifact("orders", "orders-checkpoint-7", "orders-watermark-7", "schema-gen-7", RestoreCompatibility("lumen-restore-v1")),
            ShardArtifact("users", "users-checkpoint-7", "users-watermark-7", "schema-gen-7", RestoreCompatibility("lumen-restore-v1")),
        ),
    )
    admitted_metadata = decide_backup_set(metadata)

    # 1. R1 -- a catalog plus all logical shards is a candidate the model admits.
    obs1 = _outcome(admitted_metadata)
    exp1 = SPEC_2962_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- admission retains the single catalog generation the set pins.
    obs2 = admitted_metadata.catalog_generation if not isinstance(admitted_metadata, Rejection) else _outcome(admitted_metadata)
    exp2 = SPEC_2962_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1 -- catalog checkpoint identity is not dropped from the admitted set.
    obs3 = admitted_metadata.catalog.checkpoint if not isinstance(admitted_metadata, Rejection) else "rejected"
    exp3 = SPEC_2962_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R1 -- both logical shards remain in the inventory rather than one local snapshot.
    obs4 = tuple(shard.logical_shard for shard in admitted_metadata.shards) if not isinstance(admitted_metadata, Rejection) else ()
    exp4 = SPEC_2962_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    stable = TopologyState(
        catalog_generation="catalog-gen-7",
        artifact_generations=("catalog-gen-7", "catalog-gen-7", "catalog-gen-7"),
        mutation_state="stable",
        coordinator_target="lumen-topology-coordinator",
    )
    barrier = decide_backup_barrier(stable, metadata)

    # 5. R2 -- only the explicitly stable, generation-consistent barrier permits backup.
    obs5 = barrier.outcome
    exp5 = SPEC_2962_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R4 -- a public request resolves to the designated coordinator, never a local snapshot.
    route = decide_coordinator_route(stable, request_target="lumen-public-1")
    obs6 = route.coordinator_target if not isinstance(route, Rejection) else _outcome(route)
    exp6 = SPEC_2962_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    target = RestoreTarget(empty=True, ready=False, generation="target-gen-9")
    restore_plan = plan_restore(target, metadata)

    # 7. R5 -- catalog application is the first restore operation.
    obs7 = restore_plan.apply_order[0] if not isinstance(restore_plan, Rejection) else "rejected"
    exp7 = SPEC_2962_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R5 -- the plan names every shard before membership rebind is possible.
    obs8 = restore_plan.shard_apply_order if not isinstance(restore_plan, Rejection) else ()
    exp8 = SPEC_2962_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R5 -- rebind is bound to the target generation, after artifact application.
    obs9 = restore_plan.rebind_generation if not isinstance(restore_plan, Rejection) else "rejected"
    exp9 = SPEC_2962_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R6/AC3 -- replaying the same valid apply event reaches the same state once.
    initial = RestoreState.unready(previous_successful_set="set-6", generation="target-gen-9")
    apply_catalog = RestoreEvent(kind="apply_catalog", artifact="catalog-checkpoint-7")
    once = advance_restore(initial, apply_catalog)
    twice = advance_restore(once, apply_catalog)
    obs10 = twice.phase if once == twice else "not_idempotent"
    exp10 = SPEC_2962_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    default_policy = derive_schedule_policy(destination="s3://lumen-backups", operator_policy=None)

    # 11. R7/AC5 -- destination with no policy means daily, not merely configured.
    obs11 = default_policy.schedule
    exp11 = SPEC_2962_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R7/AC5 -- retention counts successful complete sets, and defaults to two.
    obs12 = default_policy.successful_complete_set_retention
    exp12 = SPEC_2962_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    explicit_policy = derive_schedule_policy(destination="s3://lumen-backups", operator_policy={"schedule": "weekly", "successful_complete_set_retention": 5})

    # 13. R7/AC5 -- an operator selection replaces the defaults without rewriting it.
    obs13 = explicit_policy.schedule
    exp13 = SPEC_2962_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    truthful_status = project_backup_status(
        verified_manifest=BackupManifest(
            identifier="manifest-7",
            verified_at="2026-08-12T00:00:00Z",
            catalog_generation="catalog-gen-7",
            verified=True,
        ),
        job_outcome=JobOutcome(configured_schedule="daily", succeeded=True),
    )

    # 14. R8 -- success exposes the identifier of the verified manifest, not a schedule.
    obs14 = truthful_status.manifest_identifier
    exp14 = SPEC_2962_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R8 -- it separately exposes when that exact manifest was verified.
    obs15 = truthful_status.manifest_time
    exp15 = SPEC_2962_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16. R8 -- it also exposes the generation that makes the status meaningful.
    obs16 = truthful_status.catalog_generation
    exp16 = SPEC_2962_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": SPEC_2962_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    return {
        "case_id": "spec-2962-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
