from __future__ import annotations

import inspect
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.application.fenced_assignment import FencedAssignment
from raft_runtime.domain.errors import (
    AlreadyAssigned,
    Expired,
    ExpiryNotLater,
    ExpiryNotInFuture,
    OwnerMismatch,
    StaleEpoch,
    Unassigned,
)
from raft_runtime.domain.fencing import ActiveAssignment, FenceToken


class TestApplicationFencedAssignment(unittest.TestCase):
    def test_worked_sequence_full_replay(self) -> None:
        f = FencedAssignment()
        self.assertEqual(f.epoch(), 0)
        self.assertEqual(f.assign("a", 100, 200), ExpiryNotInFuture(100, 200))
        self.assertEqual(f.epoch(), 0)
        self.assertEqual(f.assign("a", 100, 0), FenceToken("a", 1))
        self.assertEqual(f.assign("b", 100, 0), AlreadyAssigned("a", 1))
        self.assertIsNone(f.validate("a", 1, 50))
        self.assertEqual(f.validate("a", 1, 100), Expired(100, 100))
        self.assertEqual(f.renew("a", 1, 100, 50), ExpiryNotLater(100, 100))
        self.assertEqual(
            f.renew("a", 1, 300, 50), ActiveAssignment(FenceToken("a", 1), 300)
        )
        self.assertEqual(f.assign("b", 400, 150), AlreadyAssigned("a", 1))
        self.assertFalse(f.expire(150))
        self.assertTrue(f.expire(300))
        self.assertEqual(f.epoch(), 1)
        self.assertEqual(f.assign("b", 500, 300), FenceToken("b", 2))
        self.assertEqual(f.validate("a", 1, 350), StaleEpoch(2, 1))
        self.assertEqual(f.release("b", 1), StaleEpoch(2, 1))
        self.assertEqual(f.release("c", 2), OwnerMismatch("b", "c"))
        self.assertIsNone(f.release("b", 2))
        self.assertTrue(f.idle())
        self.assertEqual(f.epoch(), 2)
        self.assertEqual(f.release("b", 2), Unassigned())

    def test_fresh_instance_initial_state(self) -> None:
        f = FencedAssignment()
        self.assertEqual(f.epoch(), 0)
        self.assertTrue(f.idle())
        self.assertIsNone(f.active())
        self.assertIsNone(f.token())

    def test_assign_argument_check_precedes_state_check(self) -> None:
        f = FencedAssignment()
        self.assertEqual(f.assign("a", 1000, 0), FenceToken("a", 1))
        self.assertEqual(f.assign("b", 5, 500), ExpiryNotInFuture(5, 500))

    def test_assign_expired_assignment_does_not_block_new(self) -> None:
        f = FencedAssignment()
        self.assertEqual(f.assign("a", 100, 0), FenceToken("a", 1))
        self.assertEqual(f.assign("b", 500, 100), FenceToken("b", 2))

    def test_assign_reassignment_same_owner_bumps_epoch(self) -> None:
        f = FencedAssignment()
        self.assertEqual(f.assign("a", 100, 0), FenceToken("a", 1))
        self.assertTrue(f.expire(100))
        self.assertEqual(f.assign("a", 500, 100), FenceToken("a", 2))

    def test_release_ignores_expiry_no_now_ms_param(self) -> None:
        sig = inspect.signature(FencedAssignment.release)
        self.assertNotIn("now_ms", sig.parameters)
        f = FencedAssignment()
        f.assign("a", 100, 0)
        self.assertIsNone(f.release("a", 1))

    def test_release_retains_epoch(self) -> None:
        f = FencedAssignment()
        f.assign("a", 100, 0)
        f.release("a", 1)
        self.assertEqual(f.epoch(), 1)
        self.assertEqual(f.release("a", 1), Unassigned())

    def test_renew_keeps_same_token_and_epoch(self) -> None:
        f = FencedAssignment()
        f.assign("a", 100, 0)
        renewed = f.renew("a", 1, 300, 50)
        self.assertEqual(renewed, ActiveAssignment(FenceToken("a", 1), 300))
        self.assertEqual(f.epoch(), 1)
        self.assertEqual(f.token(), FenceToken("a", 1))

    def test_rejection_leaves_state_byte_identical(self) -> None:
        f = FencedAssignment()
        f.assign("a", 100, 0)
        ep_before = f.epoch()
        act_before = f.active()

        f.assign("b", 50, 10)  # rejected ExpiryNotInFuture
        self.assertEqual((f.epoch(), f.active()), (ep_before, act_before))

        f.assign("b", 500, 10)  # rejected AlreadyAssigned
        self.assertEqual((f.epoch(), f.active()), (ep_before, act_before))

    def test_renew_expiry_not_later(self) -> None:
        f = FencedAssignment()
        f.assign("a", 200, 0)
        self.assertEqual(f.renew("a", 1, 200, 50), ExpiryNotLater(200, 200))
        self.assertEqual(f.renew("a", 1, 150, 50), ExpiryNotLater(200, 150))

    def test_release_unassigned_and_owner_mismatch(self) -> None:
        f = FencedAssignment()
        self.assertEqual(f.release("a", 1), Unassigned())
        f.assign("a", 100, 0)
        self.assertEqual(f.release("b", 1), OwnerMismatch("a", "b"))

    def test_expire_sweeps_expired_assignment(self) -> None:
        f = FencedAssignment()
        self.assertFalse(f.expire(100))
        f.assign("a", 100, 0)
        self.assertFalse(f.expire(50))
        self.assertTrue(f.expire(100))
        self.assertTrue(f.idle())


if __name__ == "__main__":
    unittest.main()
