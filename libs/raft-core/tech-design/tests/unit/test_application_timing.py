from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_core.application.timing import ElectionClock, election_timeout_for


class TestApplicationTiming(unittest.TestCase):
    def test_election_timeout_for_distinct_per_node(self) -> None:
        t0 = election_timeout_for(0)
        t1 = election_timeout_for(1)
        self.assertEqual(t0, 50)
        self.assertEqual(t1, 51)
        self.assertNotEqual(t0, t1)

    def test_clock_election_due_exact_tick(self) -> None:
        clock = ElectionClock(election_timeout=5)
        for _ in range(4):
            clock.tick()
            self.assertFalse(clock.election_due())
        clock.tick()
        self.assertTrue(clock.election_due())

    def test_heartbeat_due_and_reset(self) -> None:
        clock = ElectionClock(election_timeout=50)
        clock.tick()
        clock.tick()
        self.assertFalse(clock.heartbeat_due())
        clock.tick()
        self.assertTrue(clock.heartbeat_due())
        clock.reset_heartbeat()
        self.assertFalse(clock.heartbeat_due())


if __name__ == "__main__":
    unittest.main()
