from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.application.outcome_window import (
    OUTCOME_WINDOW_DEFAULT_CAPACITY,
    OutcomeWindow,
)


class TestApplicationOutcomeWindow(unittest.TestCase):
    def test_worked_example_full_replay(self) -> None:
        w = OutcomeWindow(capacity=4)
        for i in range(10):
            w.insert(i, bytes([i]))
        self.assertEqual(len(w), 10)
        self.assertEqual(w.advance(10), 6)
        self.assertEqual(w.floor(), 6)
        self.assertEqual(w.claim(6), b"\x06")
        self.assertIsNone(w.claim(5))
        self.assertEqual(w.claim(6), b"\x06")
        self.assertEqual(w.advance(2), 0)
        self.assertEqual(w.floor(), 6)
        w.insert(3, b"x")
        self.assertEqual(len(w), 4)

    def test_default_capacity(self) -> None:
        self.assertEqual(OUTCOME_WINDOW_DEFAULT_CAPACITY, 8192)
        w = OutcomeWindow()
        self.assertEqual(w.capacity(), 8192)

    def test_invalid_capacity_raises_value_error(self) -> None:
        with self.assertRaises(ValueError):
            OutcomeWindow(0)
        with self.assertRaises(ValueError):
            OutcomeWindow(-5)

    def test_boundary_entry_at_cutoff_survives(self) -> None:
        w = OutcomeWindow(capacity=4)
        for i in range(10):
            w.insert(i, bytes([i]))
        self.assertEqual(w.advance(10), 6)
        self.assertEqual(w.claim(6), b"\x06")

    def test_claim_does_not_remove_entry(self) -> None:
        w = OutcomeWindow(capacity=4)
        w.insert(1, b"val")
        len_before = len(w)
        res1 = w.claim(1)
        res2 = w.claim(1)
        self.assertEqual(res1, b"val")
        self.assertEqual(res2, b"val")
        self.assertEqual(len(w), len_before)

    def test_advance_is_monotone(self) -> None:
        w = OutcomeWindow(capacity=4)
        for i in range(10):
            w.insert(i, bytes([i]))
        self.assertEqual(w.advance(10), 6)
        self.assertEqual(w.floor(), 6)
        self.assertEqual(w.advance(5), 0)
        self.assertEqual(w.floor(), 6)

    def test_insert_below_floor_is_no_op(self) -> None:
        w = OutcomeWindow(capacity=4)
        w.advance(10)  # floor is now 6
        self.assertEqual(w.floor(), 6)
        w.insert(3, b"x")
        self.assertIsNone(w.claim(3))
        w.insert(6, b"at_floor")
        self.assertEqual(w.claim(6), b"at_floor")

    def test_instances_share_nothing(self) -> None:
        w1 = OutcomeWindow(4)
        w2 = OutcomeWindow(4)
        w1.insert(1, b"a")
        self.assertEqual(len(w1), 1)
        self.assertEqual(len(w2), 0)


if __name__ == "__main__":
    unittest.main()
