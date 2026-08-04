from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from transport_h2c.application.dispatch import (
    release_slot,
    reserve_slot,
    select_least_loaded,
    should_grow,
)
from transport_h2c.infrastructure.config import default_config
from transport_h2c.infrastructure.connection import ConnectionState


class TestApplicationDispatch(unittest.TestCase):
    def setUp(self) -> None:
        self.cfg = default_config(8)

    def test_select_least_loaded_empty(self) -> None:
        self.assertIsNone(select_least_loaded([]))
        dead = ConnectionState(id=1, healthy=False, in_flight=0)
        self.assertIsNone(select_least_loaded([dead]))

    def test_select_least_loaded_unhealthy_skipped(self) -> None:
        c1 = ConnectionState(id=1, healthy=False, in_flight=0)
        c2 = ConnectionState(id=2, healthy=True, in_flight=9)
        sel = select_least_loaded([c1, c2])
        self.assertIsNotNone(sel)
        self.assertEqual(sel.id, 2)

    def test_select_least_loaded_ties_earliest(self) -> None:
        c1 = ConnectionState(id=1, healthy=True, in_flight=5)
        c2 = ConnectionState(id=2, healthy=True, in_flight=2)
        c3 = ConnectionState(id=3, healthy=True, in_flight=2)
        sel = select_least_loaded([c1, c2, c3])
        self.assertIsNotNone(sel)
        self.assertEqual(sel.id, 2)

        c1_all_tied = ConnectionState(id=1, healthy=True, in_flight=2)
        c2_all_tied = ConnectionState(id=2, healthy=True, in_flight=2)
        c3_all_tied = ConnectionState(id=3, healthy=True, in_flight=2)
        sel2 = select_least_loaded([c1_all_tied, c2_all_tied, c3_all_tied])
        self.assertIsNotNone(sel2)
        self.assertEqual(sel2.id, 1)

    def test_should_grow_best_none(self) -> None:
        self.assertTrue(should_grow(None, 0, self.cfg))
        self.assertTrue(should_grow(None, 5, self.cfg))
        self.assertTrue(should_grow(None, 99, self.cfg))

    def test_should_grow_below_threshold(self) -> None:
        best = ConnectionState(id=1, healthy=True, in_flight=31)
        self.assertFalse(should_grow(best, 1, self.cfg))

    def test_should_grow_at_or_above_threshold(self) -> None:
        best32 = ConnectionState(id=1, healthy=True, in_flight=32)
        self.assertTrue(should_grow(best32, 1, self.cfg))
        best33 = ConnectionState(id=1, healthy=True, in_flight=33)
        self.assertTrue(should_grow(best33, 4, self.cfg))

    def test_should_grow_at_max_connections(self) -> None:
        best32 = ConnectionState(id=1, healthy=True, in_flight=32)
        self.assertFalse(should_grow(best32, 5, self.cfg))
        best99 = ConnectionState(id=1, healthy=True, in_flight=99)
        self.assertFalse(should_grow(best99, 6, self.cfg))

    def test_reserve_slot_outcomes(self) -> None:
        self.assertEqual(reserve_slot(0, self.cfg), (1, True))
        self.assertEqual(reserve_slot(4, self.cfg), (5, True))
        self.assertEqual(reserve_slot(5, self.cfg), (5, False))
        self.assertEqual(reserve_slot(6, self.cfg), (6, False))

    def test_release_slot_floors_at_zero(self) -> None:
        self.assertEqual(release_slot(3), 2)
        self.assertEqual(release_slot(1), 0)
        self.assertEqual(release_slot(0), 0)
        self.assertEqual(release_slot(-1), 0)


if __name__ == "__main__":
    unittest.main()
