"""Unit test suite for topology phase observability and backup status (#2533)."""
from __future__ import annotations

from dataclasses import FrozenInstanceError
import unittest

from lumen.topology.backup_status import BackupObservation, BackupState, decide_backup_observation
from lumen.topology.observability import (
    MutationKind,
    MutationObservation,
    MutationState,
    Phase,
    ProgressCounters,
    StallPolicy,
    StallSignal,
    decide_mutation_observation,
    decide_stall_signal,
    phase_age_seconds,
)


class TestObservability2533DesignModel(unittest.TestCase):
    """Test suite exercising contract rules and edge cases outside the EC matrix."""

    def test_custom_phase_thresholds_dynamic_evaluation(self) -> None:
        policy_60 = StallPolicy(phase_threshold_seconds={Phase.SPLITTING: 60})

        state_70 = MutationState(
            mutation_kind=MutationKind.SHARD_SPLIT,
            phase=Phase.SPLITTING,
            generation=10,
            phase_entered_at=1_000,
            progress_counters=ProgressCounters({"shards_split": 1}),
            last_progress_at=1_000,
            instance="lumen-node-1",
            shard_or_group="shard-a",
        )
        obs_70 = decide_mutation_observation(state_70, now_epoch_seconds=1_070)
        signal_70 = decide_stall_signal(obs_70, policy_60)
        self.assertEqual(signal_70.status, "stalled")
        self.assertEqual(signal_70.phase_age_seconds, 70)
        self.assertEqual(signal_70.instance, "lumen-node-1")
        self.assertEqual(signal_70.shard_or_group, "shard-a")
        self.assertEqual(signal_70.generation, 10)

        policy_120 = StallPolicy(phase_threshold_seconds={Phase.SPLITTING: 120})
        signal_120 = decide_stall_signal(obs_70, policy_120)
        self.assertEqual(signal_120.status, "not_stalled")

    def test_phase_age_clamping_on_future_persisted_timestamp(self) -> None:
        self.assertEqual(phase_age_seconds(2_000, 1_500), 0)

    def test_missing_phase_threshold_dynamic_field_path(self) -> None:
        policy = StallPolicy(phase_threshold_seconds={Phase.HANDOFF: 300})
        state = MutationState(
            mutation_kind=MutationKind.EMBEDDED_TO_RAFT_MIGRATION,
            phase=Phase.EMBEDDED_TO_RAFT_MIGRATION,
            generation=5,
            phase_entered_at=100,
            progress_counters=ProgressCounters({"migrated": 0}),
            last_progress_at=100,
            instance="lumen-node-2",
            shard_or_group="raft-group-1",
        )
        obs = decide_mutation_observation(state, now_epoch_seconds=500)
        signal = decide_stall_signal(obs, policy)
        self.assertEqual(signal.status, "policy_missing_phase_threshold")
        self.assertEqual(signal.field_path, "phase_thresholds.embedded_to_raft_migration")

    def test_mutation_observation_immutability(self) -> None:
        state = MutationState(
            mutation_kind=MutationKind.MEMBER_HANDOFF,
            phase=Phase.HANDOFF,
            generation=1,
            phase_entered_at=0,
            progress_counters=ProgressCounters({}),
            last_progress_at=0,
        )
        obs = decide_mutation_observation(state, now_epoch_seconds=10)
        with self.assertRaises(FrozenInstanceError):
            obs.generation = 2  # type: ignore[misc]

    def test_backup_observation_separation(self) -> None:
        backup_state = BackupState(
            pinned_generation=15,
            shard_artifact_progress={"shard-1": "complete", "shard-2": "in_progress"},
            last_successful_manifest="s3://backups/manifest-14.json",
            failure_reason="none",
        )
        obs = decide_backup_observation(backup_state)
        self.assertIsInstance(obs, BackupObservation)
        self.assertEqual(obs.pinned_generation, 15)
        self.assertEqual(obs.shard_artifact_progress["shard-1"], "complete")
        self.assertEqual(obs.last_successful_manifest, "s3://backups/manifest-14.json")
        self.assertEqual(obs.failure_reason, "none")
        self.assertFalse(hasattr(obs, "mutation_kind"))
        self.assertFalse(hasattr(obs, "phase"))

        with self.assertRaises(FrozenInstanceError):
            obs.failure_reason = "timeout"  # type: ignore[misc]


if __name__ == "__main__":
    unittest.main()
