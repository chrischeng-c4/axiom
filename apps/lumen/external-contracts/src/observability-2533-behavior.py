"""EC behavior case for #2533 -- topology-mutation observations.

Every expected value is an EC-owned literal from #2533 R1--R5 and AC2--AC4.
The pure design is intentionally imported before it exists: this verifier must
fail closed until an independent implementation publishes the observation and
backup-status models.
"""

from __future__ import annotations

from lumen.topology.backup_status import BackupState, decide_backup_observation
from lumen.topology.observability import (
    MutationKind,
    MutationState,
    Phase,
    ProgressCounters,
    StallPolicy,
    decide_mutation_observation,
    decide_stall_signal,
    phase_age_seconds,
)

MINIMUM_CHECKS = 20

OBSERVABILITY_2533_BEHAVIOR_MATRIX = (
    ("mutation_observation_has_the_required_value_fields", ("mutation_kind", "phase", "generation", "phase_entered_at", "phase_age_seconds", "progress_counters", "last_progress_at")),
    ("mutation_kind_vocabulary_covers_all_required_workflows", ("member_handoff", "embedded_to_raft_migration", "shard_split")),
    ("phase_vocabulary_covers_handoff_migration_and_every_shard_split_phase", ("handoff", "embedded_to_raft_migration", "prepare_split", "splitting", "catching_up")),
    ("observation_reports_member_handoff_kind", "member_handoff"),
    ("observation_reports_the_durable_generation", 41),
    ("observation_reports_phase_age_from_the_supplied_epoch", 600),
    ("phase_age_function_uses_the_supplied_persisted_epoch", 600),
    ("observation_reports_progress_counters", {"members_transferred": 2, "members_total": 3}),
    ("observation_reports_last_durable_progress_time", 1_440),
    ("old_phase_signal_is_stalled", "stalled"),
    ("stall_signal_reports_phase_age", 600),
    ("stall_signal_identifies_the_instance", "lumen-search"),
    ("stall_signal_identifies_the_shard_or_group", "orders-3"),
    ("stall_signal_identifies_the_generation", 41),
    ("stall_signal_names_a_safe_operator_action", "inspect durable topology state before clearing or retrying the mutation"),
    ("recent_progress_is_not_stalled", "not_stalled"),
    ("catching_up_is_independently_stall_eligible", "stalled"),
    ("prepare_split_is_independently_stall_eligible", "stalled"),
    ("backup_observation_has_only_backup_status_fields", ("pinned_generation", "shard_artifact_progress", "last_successful_manifest", "failure_reason")),
    ("incomplete_backup_preserves_prior_manifest_and_failure", (41, {"orders-3": "partial"}, "manifest://backup/40", "upload_failed")),
)


def verify_observability_2533_behavior() -> dict:
    checks = []
    policy = StallPolicy(
        phase_threshold_seconds={
            Phase.HANDOFF: 300,
            Phase.CATCHING_UP: 300,
            Phase.PREPARE_SPLIT: 300,
        }
    )
    old_state = MutationState(
        mutation_kind=MutationKind.MEMBER_HANDOFF,
        phase=Phase.HANDOFF,
        generation=41,
        phase_entered_at=900,
        progress_counters=ProgressCounters({"members_transferred": 2, "members_total": 3}),
        last_progress_at=1_440,
        instance="lumen-search",
        shard_or_group="orders-3",
    )
    observation = decide_mutation_observation(old_state, now_epoch_seconds=1_500)

    # 1. R1 -- the public observation has every durable-state-derived value.
    obs1 = tuple(observation.__dataclass_fields__)
    exp1 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R2 -- every mutation family has a distinct vocabulary value.
    obs2 = tuple(kind.value for kind in MutationKind)
    exp2 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R2/AC3 -- handoff, migration, and every split phase are named rather
    #    than hidden behind the final write fence.
    obs3 = tuple(phase.value for phase in Phase)
    exp3 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R1 -- observation exposes the actual mutation family.
    obs4 = observation.mutation_kind.value
    exp4 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R1/R3 -- generation is carried into the operator-visible value.
    obs5 = observation.generation
    exp5 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R1/R4 -- phase age is calculated from the supplied durable timestamp.
    obs6 = observation.phase_age_seconds
    exp6 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R4 -- direct calculation uses only the supplied persisted timestamp
    #    and supplied now value, so a process-local start time cannot reset it.
    obs7 = phase_age_seconds(900, 1_500)
    exp7 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R1 -- progress is a value, not an implementation-owned valid flag.
    obs8 = observation.progress_counters.values
    exp8 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R1 -- the last durable progress watermark is independently visible.
    obs9 = observation.last_progress_at
    exp9 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    old_signal = decide_stall_signal(observation, policy)

    # 10. AC2 -- an old phase produces the observable stalled status.
    obs10 = old_signal.status
    exp10 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R3 -- a signal independently carries the phase age it reports.
    obs11 = old_signal.phase_age_seconds
    exp11 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R3 -- a signal names the affected Lumen instance.
    obs12 = old_signal.instance
    exp12 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R3 -- a signal names its shard or Raft group.
    obs13 = old_signal.shard_or_group
    exp13 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R3 -- a signal carries the affected generation.
    obs14 = old_signal.generation
    exp14 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R3 -- a signal gives the safe operator action rather than a bare alert.
    obs15 = old_signal.operator_action
    exp15 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    recent = decide_mutation_observation(
        MutationState(
            mutation_kind=MutationKind.MEMBER_HANDOFF, phase=Phase.HANDOFF, generation=41,
            phase_entered_at=1_450, progress_counters=ProgressCounters({"members_transferred": 3, "members_total": 3}),
            last_progress_at=1_499, instance="lumen-search", shard_or_group="orders-3",
        ),
        now_epoch_seconds=1_500,
    )
    # 16. AC2 -- recent durable progress remains explicitly non-stalled.
    obs16 = decide_stall_signal(recent, policy).status
    exp16 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    catching_up = decide_mutation_observation(
        MutationState(MutationKind.SHARD_SPLIT, Phase.CATCHING_UP, 42, 900, ProgressCounters({"catch_up": 1}), 1_000, "lumen-search", "orders-4"),
        now_epoch_seconds=1_500,
    )
    # 17. AC3 -- the final catching-up phase has its own signal decision.
    obs17 = decide_stall_signal(catching_up, policy).status
    exp17 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    prepare_split = decide_mutation_observation(
        MutationState(MutationKind.SHARD_SPLIT, Phase.PREPARE_SPLIT, 42, 900, ProgressCounters({"prepared": 0}), 1_000, "lumen-search", "orders-4"),
        now_epoch_seconds=1_500,
    )
    # 18. AC3 -- a non-final split phase is independently signal-eligible too.
    obs18 = decide_stall_signal(prepare_split, policy).status
    exp18 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[17][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    backup = decide_backup_observation(
        BackupState(41, {"orders-3": "partial"}, "manifest://backup/40", "upload_failed")
    )
    # 19. R5 -- complete-backup status is a separate four-value projection.
    obs19 = tuple(backup.__dataclass_fields__)
    exp19 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[18][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    # 20. AC4 -- failed/incomplete work keeps both its prior manifest and cause.
    obs20 = (backup.pinned_generation, backup.shard_artifact_progress, backup.last_successful_manifest, backup.failure_reason)
    exp20 = OBSERVABILITY_2533_BEHAVIOR_MATRIX[19][1]
    checks.append({"name": OBSERVABILITY_2533_BEHAVIOR_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    return {"case_id": "observability-2533-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(check["passed"] for check in checks) and len(checks) == MINIMUM_CHECKS}
