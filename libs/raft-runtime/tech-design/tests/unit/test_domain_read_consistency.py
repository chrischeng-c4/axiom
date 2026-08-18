from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.domain.read_consistency import (
    ANY,
    LEADER,
    READ_CONSISTENCY_HEADER,
    Any_,
    Bounded,
    Leader,
    is_strongest,
    tolerated_staleness_ms,
)


class TestDomainReadConsistency(unittest.TestCase):
    def test_header_constant_equals_x_read_consistency(self) -> None:
        self.assertEqual(READ_CONSISTENCY_HEADER, "x-read-consistency")

    def test_is_strongest_leader_is_true(self) -> None:
        self.assertTrue(is_strongest(LEADER))

    def test_is_strongest_bounded_zero_is_false(self) -> None:
        self.assertFalse(is_strongest(Bounded(0)))

    def test_is_strongest_any_is_false(self) -> None:
        self.assertFalse(is_strongest(ANY))

    def test_tolerated_staleness_ms_leader_returns_none(self) -> None:
        self.assertIsNone(tolerated_staleness_ms(LEADER))

    def test_tolerated_staleness_ms_any_returns_none(self) -> None:
        self.assertIsNone(tolerated_staleness_ms(ANY))

    def test_tolerated_staleness_ms_bounded_returns_max_staleness(
        self,
    ) -> None:
        self.assertEqual(tolerated_staleness_ms(Bounded(250)), 250)

    def test_bounded_equality_and_inequality(self) -> None:
        self.assertEqual(Bounded(250), Bounded(250))
        self.assertNotEqual(Bounded(250), Bounded(251))

    def test_singleton_instances_types(self) -> None:
        self.assertIsInstance(LEADER, Leader)
        self.assertIsInstance(ANY, Any_)


if __name__ == "__main__":
    unittest.main()
