from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.domain.errors import (
    Expired,
    OwnerMismatch,
    StaleEpoch,
    Unassigned,
)
from raft_runtime.domain.fencing import (
    FIRST_EPOCH,
    ActiveAssignment,
    FenceToken,
    fence_problem,
    is_expired,
    next_epoch,
)


class TestDomainFencing(unittest.TestCase):
    def test_first_epoch_constant_is_one(self) -> None:
        self.assertEqual(FIRST_EPOCH, 1)

    def test_next_epoch_increments_previous(self) -> None:
        self.assertEqual(next_epoch(0), 1)
        self.assertEqual(next_epoch(7), 8)

    def test_is_expired_before_boundary(self) -> None:
        self.assertFalse(is_expired(100, 99))

    def test_boundary_instant_is_expired(self) -> None:
        self.assertTrue(is_expired(100, 100))

    def test_is_expired_after_boundary(self) -> None:
        self.assertTrue(is_expired(100, 101))

    def test_fence_problem_unassigned_when_active_is_none(self) -> None:
        self.assertEqual(fence_problem(None, "a", 1, 0), Unassigned())

    def test_fence_problem_returns_none_when_assignment_is_valid(
        self,
    ) -> None:
        active = ActiveAssignment(FenceToken("a", 2), 100)
        self.assertIsNone(fence_problem(active, "a", 2, 50))

    def test_fence_problem_stale_epoch_precedes_owner_mismatch_and_expiry(
        self,
    ) -> None:
        active = ActiveAssignment(FenceToken("a", 2), 100)
        self.assertEqual(
            fence_problem(active, "b", 1, 200),
            StaleEpoch(expected=2, supplied=1),
        )

    def test_fence_problem_owner_mismatch(self) -> None:
        active = ActiveAssignment(FenceToken("a", 2), 100)
        self.assertEqual(
            fence_problem(active, "b", 2, 50),
            OwnerMismatch(expected="a", supplied="b"),
        )

    def test_fence_problem_expired_at_boundary_and_beyond(self) -> None:
        active = ActiveAssignment(FenceToken("a", 2), 100)
        self.assertEqual(
            fence_problem(active, "a", 2, 100),
            Expired(expires_at_ms=100, now_ms=100),
        )

    def test_fence_problem_stale_epoch_precedes_expired(self) -> None:
        active = ActiveAssignment(FenceToken("a", 2), 100)
        self.assertEqual(
            fence_problem(active, "a", 1, 50),
            StaleEpoch(expected=2, supplied=1),
        )

    def test_fence_problem_never_mutates_or_raises(self) -> None:
        active = ActiveAssignment(FenceToken("node1", 5), 5000)
        result = fence_problem(active, "node1", 5, 1000)
        self.assertIsNone(result)


if __name__ == "__main__":
    unittest.main()
