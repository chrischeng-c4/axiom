"""Executable acceptance tests for tape WI #3052's WAL/group-commit design."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path


SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
sys.path.insert(0, str(SRC_ROOT))

from tape.work_items.replace_the_single_node_whole_file_journal_rewrite_with_an_appen import (  # noqa: E402
    AcceptanceCriterion,
    CommitBatch,
    CommitOutcome,
    CommitRequest,
    DesignInvariant,
    JournalStoreKind,
    RecoveryAction,
    TapeCommandKind,
    TrapKind,
    acceptance_criteria,
    apply_command_extraction_contract,
    apply_order_for_batch,
    benchmark_is_durable_and_not_vacuous,
    commit_batch_outcome,
    design_contract,
    design_invariants,
    drain_pending_commit_batch,
    implementation_traps,
    legacy_store_corruption_is_fatal,
    out_of_scope_boundaries,
    performance_gate_contract,
    recovery_action_for_wal_scan,
    resolve_journal_store_kind,
    should_enter_storage_degraded_mode,
    should_snapshot_and_truncate,
    wal_frame_contract,
    wal_tail_corruption_is_fatal,
)


class ApplyCommandExtractionDesignTest(unittest.TestCase):
    def test_extraction_covers_every_reusable_command_kind(self) -> None:
        contract = apply_command_extraction_contract()
        self.assertEqual(contract.source_module, "apps/tape/src/raft.rs")
        self.assertEqual(contract.source_type, "TapeStateMachine")
        self.assertEqual(contract.source_method, "apply")
        self.assertEqual(contract.extracted_function_name, "apply_command")
        self.assertEqual(set(contract.commands_covered), set(TapeCommandKind))
        self.assertEqual(len(contract.commands_covered), 6)


class WalFrameDesignTest(unittest.TestCase):
    def test_wal_frames_encode_commands_not_journal_state(self) -> None:
        contract = wal_frame_contract()
        self.assertEqual(contract.encodes, "TapeCommand")
        self.assertIn("enforce_retention", contract.reason)


class JournalStoreResolutionDesignTest(unittest.TestCase):
    def test_explicit_store_always_wins(self) -> None:
        for data_dir in (True, False):
            for replica_mode in (True, False):
                with self.subTest(data_dir=data_dir, replica_mode=replica_mode):
                    self.assertEqual(
                        resolve_journal_store_kind(
                            explicit_store=True,
                            data_dir=data_dir,
                            replica_mode=replica_mode,
                        ),
                        JournalStoreKind.LEGACY_FILE,
                    )

    def test_data_dir_without_explicit_store_now_resolves_to_wal(self) -> None:
        self.assertEqual(
            resolve_journal_store_kind(
                explicit_store=False, data_dir=True, replica_mode=False
            ),
            JournalStoreKind.WAL,
        )

    def test_replica_mode_without_explicit_store_resolves_to_none(self) -> None:
        self.assertEqual(
            resolve_journal_store_kind(
                explicit_store=False, data_dir=True, replica_mode=True
            ),
            JournalStoreKind.NONE,
        )

    def test_neither_data_dir_nor_explicit_store_resolves_to_none(self) -> None:
        self.assertEqual(
            resolve_journal_store_kind(
                explicit_store=False, data_dir=False, replica_mode=False
            ),
            JournalStoreKind.NONE,
        )


class GroupCommitOrderingDesignTest(unittest.TestCase):
    def test_batch_drain_preserves_fifo_order_and_respects_max_size(self) -> None:
        pending = tuple(
            CommitRequest(request_id=i, command=TapeCommandKind.APPEND)
            for i in range(5)
        )
        batch = drain_pending_commit_batch(pending, max_batch_size=3)
        self.assertEqual(apply_order_for_batch(batch), (0, 1, 2))

    def test_apply_order_matches_submission_order_for_a_full_batch(self) -> None:
        pending = tuple(
            CommitRequest(request_id=i, command=TapeCommandKind.RETENTION_PUT)
            for i in (7, 3, 9)
        )
        batch = CommitBatch(requests=pending)
        self.assertEqual(apply_order_for_batch(batch), (7, 3, 9))


class CommitOutcomeDesignTest(unittest.TestCase):
    def test_success_requires_write_fsync_and_apply(self) -> None:
        self.assertEqual(
            commit_batch_outcome(write_ok=True, fsync_ok=True, apply_ok=True),
            CommitOutcome.ACKED,
        )

    def test_write_or_fsync_failure_never_acks(self) -> None:
        self.assertEqual(
            commit_batch_outcome(write_ok=False, fsync_ok=True, apply_ok=True),
            CommitOutcome.FAILED_FSYNC,
        )
        self.assertEqual(
            commit_batch_outcome(write_ok=True, fsync_ok=False, apply_ok=True),
            CommitOutcome.FAILED_FSYNC,
        )

    def test_apply_failure_after_a_durable_write_is_distinguished(self) -> None:
        self.assertEqual(
            commit_batch_outcome(write_ok=True, fsync_ok=True, apply_ok=False),
            CommitOutcome.FAILED_APPLY,
        )

    def test_enospc_or_eio_failures_latch_sticky_degraded_mode(self) -> None:
        self.assertTrue(
            should_enter_storage_degraded_mode(
                CommitOutcome.FAILED_FSYNC, is_enospc=True, is_eio=False
            )
        )
        self.assertTrue(
            should_enter_storage_degraded_mode(
                CommitOutcome.FAILED_FSYNC, is_enospc=False, is_eio=True
            )
        )
        self.assertTrue(
            should_enter_storage_degraded_mode(
                CommitOutcome.FAILED_APPLY, is_enospc=True, is_eio=True
            )
        )

    def test_a_failure_that_is_neither_enospc_nor_eio_does_not_latch_degraded_mode(
        self,
    ) -> None:
        self.assertFalse(
            should_enter_storage_degraded_mode(
                CommitOutcome.FAILED_FSYNC, is_enospc=False, is_eio=False
            )
        )

    def test_an_acked_batch_never_latches_degraded_mode(self) -> None:
        self.assertFalse(
            should_enter_storage_degraded_mode(
                CommitOutcome.ACKED, is_enospc=True, is_eio=True
            )
        )


class RecoveryDesignTest(unittest.TestCase):
    def test_empty_wal_starts_empty(self) -> None:
        self.assertEqual(
            recovery_action_for_wal_scan(has_existing_frames=False, tail_is_torn=True),
            RecoveryAction.START_EMPTY,
        )

    def test_torn_tail_is_truncated_not_refused(self) -> None:
        self.assertEqual(
            recovery_action_for_wal_scan(has_existing_frames=True, tail_is_torn=True),
            RecoveryAction.TRUNCATE_TORN_TAIL,
        )

    def test_clean_tail_replays(self) -> None:
        self.assertEqual(
            recovery_action_for_wal_scan(has_existing_frames=True, tail_is_torn=False),
            RecoveryAction.REPLAY,
        )

    def test_only_the_wal_path_changes_corruption_tolerance(self) -> None:
        self.assertTrue(legacy_store_corruption_is_fatal())
        self.assertFalse(wal_tail_corruption_is_fatal())


class SnapshotDesignTest(unittest.TestCase):
    def test_snapshot_threshold_is_a_simple_counter_comparison(self) -> None:
        self.assertFalse(should_snapshot_and_truncate(9, threshold=10))
        self.assertTrue(should_snapshot_and_truncate(10, threshold=10))
        self.assertTrue(should_snapshot_and_truncate(11, threshold=10))


class BoundaryAndTrapDesignTest(unittest.TestCase):
    def test_out_of_scope_boundary_covers_storage_durable_and_raft_and_fsync_policy(
        self,
    ) -> None:
        areas = {boundary.area for boundary in out_of_scope_boundaries()}
        self.assertIn("libs/storage-durable", areas)
        self.assertIn("snapshot/backup wire format", areas)
        self.assertIn("FsyncPolicy::Always", areas)
        self.assertTrue(any("Raft" in area for area in areas))

    def test_named_traps_are_all_present(self) -> None:
        names = {trap.name for trap in implementation_traps()}
        self.assertEqual(
            names,
            {
                "data_dir_has_existing_state",
                "storage_full_probe filename collision",
                "in-crate persist tests must be rewritten, not deleted",
            },
        )

    def test_data_dir_has_existing_state_is_confirm_do_not_change(self) -> None:
        traps = {trap.name: trap for trap in implementation_traps()}
        trap = traps["data_dir_has_existing_state"]
        self.assertEqual(trap.kind, TrapKind.CONFIRM_DO_NOT_CHANGE)
        self.assertIn("Do not edit this function", trap.hazard)

    def test_the_other_two_named_traps_are_must_handle(self) -> None:
        traps = {trap.name: trap for trap in implementation_traps()}
        self.assertEqual(
            traps["storage_full_probe filename collision"].kind,
            TrapKind.MUST_HANDLE,
        )
        self.assertEqual(
            traps["in-crate persist tests must be rewritten, not deleted"].kind,
            TrapKind.MUST_HANDLE,
        )


class PerformanceGateDesignTest(unittest.TestCase):
    def test_baseline_matches_the_measured_flat_throughput_ceiling(self) -> None:
        contract = performance_gate_contract()
        self.assertEqual(contract.baseline_ops_per_sec_low, 85.0)
        self.assertEqual(contract.baseline_ops_per_sec_high, 89.0)

    def test_required_ratio_is_a_scaled_versus_baseline_connection_comparison(
        self,
    ) -> None:
        contract = performance_gate_contract()
        self.assertEqual(contract.baseline_connections, 1)
        self.assertEqual(contract.scaled_connections, 16)
        self.assertEqual(
            contract.required_scaled_over_baseline_connection_throughput_ratio,
            4.0,
        )

    def test_benchmark_must_use_real_disk_io_and_not_bypass_persist(self) -> None:
        self.assertTrue(
            benchmark_is_durable_and_not_vacuous(
                uses_real_disk_io=True, bypasses_persist=False
            )
        )
        self.assertFalse(
            benchmark_is_durable_and_not_vacuous(
                uses_real_disk_io=False, bypasses_persist=False
            )
        )
        self.assertFalse(
            benchmark_is_durable_and_not_vacuous(
                uses_real_disk_io=True, bypasses_persist=True
            )
        )


class DesignInvariantDesignTest(unittest.TestCase):
    def test_there_are_exactly_eight_design_invariants_with_stable_ids(self) -> None:
        invariants = design_invariants()
        self.assertEqual(len(invariants), 8)
        self.assertTrue(
            all(isinstance(item, DesignInvariant) for item in invariants)
        )
        self.assertEqual(
            [item.id for item in invariants],
            [f"DI{n}" for n in range(1, 9)],
        )
        self.assertTrue(all(item.statement for item in invariants))


class AcceptanceCriteriaDesignTest(unittest.TestCase):
    def test_there_are_exactly_eight_acceptance_criteria_with_stable_ids(self) -> None:
        criteria = acceptance_criteria()
        self.assertEqual(len(criteria), 8)
        self.assertTrue(all(isinstance(item, AcceptanceCriterion) for item in criteria))
        self.assertEqual(
            [item.id for item in criteria],
            [f"AC{n}" for n in range(1, 9)],
        )
        self.assertTrue(all(item.statement for item in criteria))
        self.assertTrue(all(item.verified_by for item in criteria))

    def test_ac6_backup_byte_identity_is_carried_as_required_closure_r7(self) -> None:
        criteria = {item.id: item for item in acceptance_criteria()}
        ac6 = criteria["AC6"]
        self.assertIn("byte-identical", ac6.statement)
        self.assertIn("admin/backup", ac6.statement)
        self.assertIn("Required Closure", ac6.verified_by)

    def test_ac7_enospc_is_explicitly_not_covered_by_either_ec_case(self) -> None:
        criteria = {item.id: item for item in acceptance_criteria()}
        ac7 = criteria["AC7"]
        self.assertIn("507", ac7.statement)
        self.assertIn("excludes AC7", ac7.verified_by)

    def test_ac4_durability_matches_the_achievable_ec_wording(self) -> None:
        criteria = {item.id: item for item in acceptance_criteria()}
        ac4 = criteria["AC4"]
        self.assertIn("ec-3052-durability", ac4.verified_by)
        self.assertIn("achievable half", ac4.verified_by)

    def test_ac1_scaling_is_covered_by_the_scaling_ec(self) -> None:
        criteria = {item.id: item for item in acceptance_criteria()}
        ac1 = criteria["AC1"]
        self.assertIn("ec-3052-scaling", ac1.verified_by)

    def test_ac8_is_the_cargo_test_gate(self) -> None:
        criteria = {item.id: item for item in acceptance_criteria()}
        ac8 = criteria["AC8"]
        self.assertEqual(ac8.statement, "cargo test -p tape passes")


class DesignContractSummaryTest(unittest.TestCase):
    def test_design_contract_names_the_bounded_change_and_its_boundary(self) -> None:
        summary = design_contract()
        self.assertIn("AppState::persist", summary)
        self.assertIn("write-ahead log", summary)
        self.assertIn("group commit", summary)
        self.assertIn("--store", summary)


if __name__ == "__main__":
    unittest.main()
