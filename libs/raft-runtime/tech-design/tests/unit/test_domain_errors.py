from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.domain.errors import (
    AlreadyAssigned,
    AppliedIndexError,
    BadOrdinal,
    Expired,
    ExpiryNotInFuture,
    NamelessPod,
    NodeIdOutOfRange,
    NonPositiveDimension,
    OwnerMismatch,
    StaleEpoch,
    Unassigned,
    UnsupportedScheme,
    VoterCountOutOfRange,
)


class TestDomainErrors(unittest.TestCase):
    def test_error_dataclasses_are_frozen_and_slotted(self) -> None:
        err = StaleEpoch(expected=2, supplied=1)
        with self.assertRaises(AttributeError):
            err.expected = 3  # type: ignore[misc]
        self.assertFalse(hasattr(err, "__dict__"))

    def test_structurally_equal_errors_compare_equal(self) -> None:
        self.assertEqual(StaleEpoch(2, 1), StaleEpoch(2, 1))
        self.assertEqual(
            OwnerMismatch("owner1", "owner2"), OwnerMismatch("owner1", "owner2")
        )
        self.assertEqual(Expired(100, 100), Expired(100, 100))

    def test_differently_typed_errors_with_same_fields_do_not_compare_equal(
        self,
    ) -> None:
        self.assertNotEqual(
            NonPositiveDimension("a", 0), NonPositiveDimension("b", 0)
        )
        self.assertNotEqual(Unassigned(), NamelessPod(""))

    def test_unassigned_equality_and_inequality(self) -> None:
        self.assertEqual(Unassigned(), Unassigned())
        self.assertNotEqual(Unassigned(), AlreadyAssigned("owner", 1))

    def test_applied_index_error_is_subclass_of_value_error(self) -> None:
        self.assertTrue(issubclass(AppliedIndexError, ValueError))

    def test_no_other_domain_error_is_subclass_of_exception(self) -> None:
        error_classes = [
            Unassigned,
            StaleEpoch,
            OwnerMismatch,
            Expired,
            AlreadyAssigned,
            ExpiryNotInFuture,
            NamelessPod,
            BadOrdinal,
            NonPositiveDimension,
            VoterCountOutOfRange,
            NodeIdOutOfRange,
            UnsupportedScheme,
        ]
        for cls in error_classes:
            self.assertFalse(
                issubclass(cls, Exception),
                f"{cls.__name__} should not inherit Exception",
            )

    def test_error_representations_and_fields(self) -> None:
        err = ExpiryNotInFuture(expires_at_ms=50, now_ms=100)
        self.assertEqual(err.expires_at_ms, 50)
        self.assertEqual(err.now_ms, 100)


if __name__ == "__main__":
    unittest.main()
