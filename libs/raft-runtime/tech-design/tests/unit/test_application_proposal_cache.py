from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.application.proposal_cache import (
    DEFAULT_PROPOSAL_CACHE_CAPACITY,
    ProposalCache,
)


class TestApplicationProposalCache(unittest.TestCase):
    def test_default_capacity_and_custom_capacity(self) -> None:
        self.assertEqual(DEFAULT_PROPOSAL_CACHE_CAPACITY, 4096)
        c_def = ProposalCache()
        self.assertEqual(c_def.capacity(), 4096)
        c_custom = ProposalCache(100)
        self.assertEqual(c_custom.capacity(), 100)

    def test_invalid_capacity_raises_value_error(self) -> None:
        with self.assertRaises(ValueError):
            ProposalCache(0)
        with self.assertRaises(ValueError):
            ProposalCache(-1)

    def test_insert_first_outcome_wins(self) -> None:
        c = ProposalCache(4)
        self.assertEqual(c.insert("k", b"first"), b"first")
        self.assertEqual(c.insert("k", b"second"), b"first")
        self.assertEqual(c.get("k"), b"first")
        self.assertEqual(len(c), 1)

    def test_repeat_insert_does_not_refresh_position(self) -> None:
        c = ProposalCache(3)
        for k in "abc":
            c.insert(k, k.encode("ascii"))
        self.assertEqual(c.insert("a", b"again"), b"a")
        c.insert("d", b"d")
        self.assertIsNone(c.get("a"))
        self.assertEqual(c.get("b"), b"b")
        self.assertEqual(
            c.snapshot(), (("b", b"b"), ("c", b"c"), ("d", b"d"))
        )

    def test_restore_drops_oldest_when_over_capacity(self) -> None:
        c = ProposalCache(2)
        c.restore([("x", b"1"), ("y", b"2"), ("z", b"3")])
        self.assertEqual(c.snapshot(), (("y", b"2"), ("z", b"3")))

    def test_snapshot_returns_tuple(self) -> None:
        c = ProposalCache(2)
        c.insert("a", b"1")
        snap = c.snapshot()
        self.assertIsInstance(snap, tuple)
        self.assertEqual(snap, (("a", b"1"),))

    def test_instances_share_nothing(self) -> None:
        c1 = ProposalCache(2)
        c2 = ProposalCache(2)
        c1.insert("k", b"v")
        self.assertEqual(len(c1), 1)
        self.assertEqual(len(c2), 0)

    def test_basic_get_and_len(self) -> None:
        c = ProposalCache(2)
        self.assertEqual(len(c), 0)
        self.assertIsNone(c.get("absent"))
        c.insert("k1", b"v1")
        self.assertEqual(len(c), 1)
        self.assertEqual(c.get("k1"), b"v1")


if __name__ == "__main__":
    unittest.main()
