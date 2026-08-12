"""EC behavior case for #2961 -- embedded-to-Raft migration.

Every expected value below is an EC-owned literal transcribed from #2961:
R1 keeps non-empty legacy data authoritative while empty Raft requires
migration; R3 permits the final write fence only after a durable checkpoint and
a caught-up watermark; R4 commits the named target catalog generation only at
the verified watermark; R5 derives authority from persisted cutover state; and
R7 sends a new installation directly to the canonical one-voter Raft path.
"""

from __future__ import annotations

from lumen.topology.migration import (
    InstallationKind,
    MigrationPhase,
    MigrationProgress,
    MigrationSpec,
    StoreState,
)
from lumen.topology.migration_admission import (
    decide_catalog_cutover,
    decide_installation_path,
    decide_migration,
    decide_next_phase,
)
from lumen.topology.migration import advance_migration, recover_authority
from lumen.topology.verdict import Rejection

MINIMUM_CHECKS = 10

MIGRATION_2961_BEHAVIOR_MATRIX = (
    ("non_empty_legacy_empty_raft_requires_migration", "migration_required"),
    ("migration_required_keeps_legacy_authoritative", "legacy"),
    ("durable_caught_up_tail_admits_final_write_fence", "final_write_fence"),
    ("final_write_fence_waits_for_a_safe_boundary", "wait"),
    ("verified_target_admits_the_requested_catalog_generation", 17),
    ("persisted_pre_cutover_state_derives_legacy_authority", "legacy"),
    ("persisted_post_cutover_state_derives_raft_authority", "raft"),
    ("pre_cutover_recovery_is_retryable_legacy", ("legacy", "retry")),
    ("post_cutover_recovery_is_raft_with_source_retention", ("raft", "retain")),
    ("new_installation_selects_canonical_raft", "canonical_raft"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_migration_2961_behavior() -> dict:
    checks = []

    legacy_to_empty_raft = MigrationSpec(
        installation=InstallationKind.LEGACY,
        legacy_state=StoreState.NON_EMPTY,
        raft_state=StoreState.EMPTY,
    )
    migration = decide_migration(legacy_to_empty_raft)

    # 1. R1 -- an empty Raft store cannot replace non-empty embedded state.
    obs1 = migration.phase.value if not isinstance(migration, Rejection) else _outcome(migration)
    exp1 = MIGRATION_2961_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": MIGRATION_2961_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- the migration-required verdict names legacy as authority.
    obs2 = migration.authoritative_store.value if not isinstance(migration, Rejection) else _outcome(migration)
    exp2 = MIGRATION_2961_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": MIGRATION_2961_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    caught_up_tail = MigrationProgress(
        phase=MigrationPhase.TAILING,
        checkpoint_durable=True,
        acknowledged_watermark=41,
        target_watermark=41,
        verified_watermark=41,
        catalog_cutover_committed=False,
        catalog_generation=16,
        post_cutover_restart_verified=False,
        oracle_verified=False,
        source_is_authoritative=False,
        source_is_related=True,
    )
    final_fence = decide_next_phase(caught_up_tail, MigrationPhase.FINAL_WRITE_FENCE)

    # 3. R3 -- checkpoint plus acknowledged tail admits the final fence.
    obs3 = final_fence.phase.value if not isinstance(final_fence, Rejection) else _outcome(final_fence)
    exp3 = MIGRATION_2961_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": MIGRATION_2961_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R3 -- the admitted fence waits/retries for the safe boundary; it does
    #    not claim a one-sided acknowledgement outcome.
    obs4 = final_fence.required_action if not isinstance(final_fence, Rejection) else _outcome(final_fence)
    exp4 = MIGRATION_2961_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": MIGRATION_2961_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    cutover = decide_catalog_cutover(caught_up_tail, target_catalog_generation=17)

    # 5. R4 -- the sole admitted routing generation is the requested target.
    obs5 = cutover.catalog_generation if not isinstance(cutover, Rejection) else -1
    exp5 = MIGRATION_2961_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": MIGRATION_2961_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    pre_cutover = MigrationProgress(
        phase=MigrationPhase.TAILING,
        checkpoint_durable=True,
        acknowledged_watermark=41,
        target_watermark=41,
        verified_watermark=41,
        catalog_cutover_committed=False,
        catalog_generation=16,
        post_cutover_restart_verified=False,
        oracle_verified=False,
        source_is_authoritative=False,
        source_is_related=True,
    )
    advanced_pre_cutover = advance_migration(pre_cutover)

    # 6. R5 -- persisted state before catalog cutover still serves legacy.
    obs6 = advanced_pre_cutover.authoritative_store.value
    exp6 = MIGRATION_2961_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": MIGRATION_2961_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    post_cutover = MigrationProgress(
        phase=MigrationPhase.CATALOG_CUTOVER,
        checkpoint_durable=True,
        acknowledged_watermark=41,
        target_watermark=41,
        verified_watermark=41,
        catalog_cutover_committed=True,
        catalog_generation=17,
        post_cutover_restart_verified=True,
        oracle_verified=True,
        source_is_authoritative=False,
        source_is_related=True,
    )
    advanced_post_cutover = advance_migration(post_cutover)

    # 7. R5 -- after commit, no derived migration state reactivates embedded.
    obs7 = advanced_post_cutover.authoritative_store.value
    exp7 = MIGRATION_2961_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": MIGRATION_2961_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. AC2/AC3 -- recovery before cutover is one retryable legacy choice.
    recovered_pre = recover_authority(pre_cutover, legacy_to_empty_raft)
    obs8 = (recovered_pre.authoritative_store.value, recovered_pre.required_action)
    exp8 = MIGRATION_2961_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": MIGRATION_2961_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. AC2/AC3 -- recovery after cutover is one Raft choice and retains source.
    recovered_post = recover_authority(post_cutover, legacy_to_empty_raft)
    obs9 = (recovered_post.authoritative_store.value, recovered_post.source_retention.value)
    exp9 = MIGRATION_2961_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": MIGRATION_2961_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    new_installation = MigrationSpec(
        installation=InstallationKind.NEW,
        legacy_state=StoreState.EMPTY,
        raft_state=StoreState.EMPTY,
    )
    installation_path = decide_installation_path(new_installation)

    # 10. R7 -- a new installation never starts the compatibility workflow.
    obs10 = installation_path.path.value if not isinstance(installation_path, Rejection) else _outcome(installation_path)
    exp10 = MIGRATION_2961_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": MIGRATION_2961_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "migration-2961-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
