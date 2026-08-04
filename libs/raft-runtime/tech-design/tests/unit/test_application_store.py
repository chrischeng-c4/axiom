from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.application.store import (
    INITIAL_HARD_STATE,
    HardState,
    RaftStore,
    SnapshotExists,
)


class TestApplicationStore(unittest.TestCase):
    def test_initial_hard_state_and_fresh_store(self) -> None:
        self.assertEqual(INITIAL_HARD_STATE, HardState(term=0, voted_for=None))
        s = RaftStore()
        self.assertIsNone(s.last_saved())
        self.assertEqual(s.load(), INITIAL_HARD_STATE)

    def test_save_initial_value_first_time_is_real_write(self) -> None:
        s = RaftStore()
        self.assertTrue(s.save(HardState(0, None)))
        self.assertFalse(s.save(HardState(0, None)))

    def test_save_distinguishes_term_and_voted_for(self) -> None:
        s = RaftStore()
        self.assertTrue(s.save(HardState(1, None)))
        self.assertTrue(s.save(HardState(1, 0)))
        self.assertFalse(s.save(HardState(1, 0)))
        self.assertEqual(s.last_saved(), HardState(1, 0))

    def test_seed_snapshot_refuses_when_path_exists(self) -> None:
        s = RaftStore()
        res_exists = s.seed_snapshot("/p", lambda path: True)
        self.assertEqual(res_exists, SnapshotExists("/p"))
        self.assertNotEqual(res_exists, "/p")

        res_absent = s.seed_snapshot("/q", lambda path: False)
        self.assertEqual(res_absent, "/q")

    def test_seed_snapshot_calls_exists_once(self) -> None:
        s = RaftStore()
        count = 0

        def probe(path: str) -> bool:
            nonlocal count
            count += 1
            return False

        s.seed_snapshot("/test", probe)
        self.assertEqual(count, 1)

    def test_load_returns_initial_or_last_saved(self) -> None:
        s = RaftStore()
        self.assertEqual(s.load(), HardState(0, None))
        s.save(HardState(5, 2))
        self.assertEqual(s.load(), HardState(5, 2))

    def test_save_same_state_returns_false(self) -> None:
        s = RaftStore()
        st = HardState(2, 1)
        self.assertTrue(s.save(st))
        self.assertFalse(s.save(st))

    def test_stores_share_nothing(self) -> None:
        s1 = RaftStore()
        s2 = RaftStore()
        s1.save(HardState(1, 1))
        self.assertEqual(s1.last_saved(), HardState(1, 1))
        self.assertIsNone(s2.last_saved())


if __name__ == "__main__":
    unittest.main()
