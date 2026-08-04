from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.application.replication import (
    ApplyReport,
    RaftStateMachine,
    apply_committed,
    replay_plan,
)
from raft_runtime.domain.consensus import Command


class FakeRaftStateMachine:
    def __init__(self, floor: int, failing_index: int | None = None) -> None:
        self._floor = floor
        self._failing_index = failing_index
        self.calls: list[tuple[int, Command]] = []
        self.applied_index_call_count = 0

    def apply(self, index: int, command: Command) -> None:
        self.calls.append((index, command))
        if self._failing_index is not None and index == self._failing_index:
            raise RuntimeError(f"Apply failed at index {index}")

    def snapshot(self) -> bytes:
        return b"snapshot"

    def restore(self, blob: bytes) -> None:
        pass

    def applied_index(self) -> int:
        self.applied_index_call_count += 1
        return self._floor


class TestApplicationReplication(unittest.TestCase):
    def test_raft_state_machine_is_protocol(self) -> None:
        self.assertTrue(getattr(RaftStateMachine, "_is_protocol", False))

    def test_replay_plan_behavior(self) -> None:
        self.assertEqual(replay_plan(0, ()), ())
        self.assertEqual(replay_plan(3, (5, 1, 4, 3, 5)), (4, 5))
        self.assertEqual(replay_plan(0, (2, 1)), (1, 2))

    def test_worked_example_full_replay(self) -> None:
        fake = FakeRaftStateMachine(floor=3, failing_index=6)
        entries = (
            (7, b"g"),
            (2, b"a"),
            (5, b"e"),
            (3, b"c"),
            (6, b"f"),
            (5, b"e2"),
        )
        report = apply_committed(fake, entries)
        self.assertEqual(report.applied, (5, 6, 7))
        self.assertEqual(report.skipped, (2, 3, 5))
        self.assertEqual(report.failed, (6,))
        self.assertEqual(fake.calls, [(5, b"e"), (6, b"f"), (7, b"g")])

    def test_ascending_application_regardless_of_input_order(self) -> None:
        fake = FakeRaftStateMachine(floor=0)
        entries = [(10, b"cmd10"), (2, b"cmd2"), (5, b"cmd5")]
        apply_committed(fake, entries)
        self.assertEqual(
            fake.calls, [(2, b"cmd2"), (5, b"cmd5"), (10, b"cmd10")]
        )

    def test_raising_apply_lands_in_applied_and_failed_and_does_not_raise(
        self,
    ) -> None:
        fake = FakeRaftStateMachine(floor=0, failing_index=5)
        entries = [(5, b"fail")]
        report = apply_committed(fake, entries)
        self.assertEqual(report.applied, (5,))
        self.assertEqual(report.failed, (5,))

    def test_applied_index_read_exactly_once(self) -> None:
        fake = FakeRaftStateMachine(floor=0)
        entries = [(1, b"a"), (2, b"b"), (3, b"c"), (4, b"d"), (5, b"e")]
        apply_committed(fake, entries)
        self.assertEqual(fake.applied_index_call_count, 1)

    def test_in_batch_duplicate_is_skipped(self) -> None:
        fake = FakeRaftStateMachine(floor=0)
        entries = [(5, b"e1"), (5, b"e2")]
        report = apply_committed(fake, entries)
        self.assertEqual(report.applied, (5,))
        self.assertEqual(report.skipped, (5,))
        self.assertEqual(fake.calls, [(5, b"e1")])

    def test_apply_committed_all_below_floor_skipped(self) -> None:
        fake = FakeRaftStateMachine(floor=10)
        entries = [(1, b"a"), (5, b"b"), (10, b"c")]
        report = apply_committed(fake, entries)
        self.assertEqual(report.applied, ())
        self.assertEqual(report.skipped, (1, 5, 10))
        self.assertEqual(len(fake.calls), 0)


if __name__ == "__main__":
    unittest.main()
