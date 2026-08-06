from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.domain.snapshot import (
    DEFAULT_SNAPSHOT_POLICY,
    Disabled,
    EveryEntries,
    External,
    compactable_upto,
    should_snapshot,
)


class TestDomainSnapshot(unittest.TestCase):
    def test_default_snapshot_policy_is_disabled(self) -> None:
        self.assertEqual(DEFAULT_SNAPSHOT_POLICY, Disabled())

    def test_should_snapshot_disabled_always_false(self) -> None:
        self.assertFalse(should_snapshot(Disabled(), 10**9, 0))

    def test_should_snapshot_external_always_false(self) -> None:
        self.assertFalse(should_snapshot(External(), 10**9, 0))

    def test_should_snapshot_every_entries_zero_interval_never_fires(
        self,
    ) -> None:
        self.assertFalse(should_snapshot(EveryEntries(0), 100, 0))

    def test_should_snapshot_every_entries_below_interval(self) -> None:
        self.assertFalse(should_snapshot(EveryEntries(100), 99, 0))

    def test_should_snapshot_every_entries_exact_interval(self) -> None:
        self.assertTrue(should_snapshot(EveryEntries(100), 100, 0))

    def test_should_snapshot_every_entries_relative_to_last_snapshot(
        self,
    ) -> None:
        self.assertFalse(should_snapshot(EveryEntries(100), 150, 100))

    def test_compactable_upto_zero(self) -> None:
        self.assertEqual(compactable_upto(0), 0)

    def test_compactable_upto_applied_index(self) -> None:
        self.assertEqual(compactable_upto(42), 42)


if __name__ == "__main__":
    unittest.main()
