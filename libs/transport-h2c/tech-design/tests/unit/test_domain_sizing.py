from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from transport_h2c.domain.sizing import recommended_connections


class TestDomainSizing(unittest.TestCase):
    def test_small_concurrency_zero(self) -> None:
        self.assertEqual(recommended_connections(0, 8), 1)

    def test_small_concurrency_one(self) -> None:
        self.assertEqual(recommended_connections(1, 8), 1)

    def test_small_concurrency_two(self) -> None:
        self.assertEqual(recommended_connections(2, 8), 1)

    def test_log_shaped_scaling(self) -> None:
        self.assertEqual(recommended_connections(16, 64), 3)
        self.assertEqual(recommended_connections(64, 64), 5)
        self.assertEqual(recommended_connections(256, 64), 6)
        self.assertEqual(recommended_connections(1024, 64), 7)
        self.assertEqual(recommended_connections(4096, 64), 9)

    def test_parallelism_cap(self) -> None:
        self.assertEqual(recommended_connections(1_000_000, 4), 4)

    def test_zero_parallelism_cap(self) -> None:
        self.assertEqual(recommended_connections(1024, 0), 1)

    def test_knee_transitions(self) -> None:
        self.assertEqual(recommended_connections(3, 64), 2)
        self.assertEqual(recommended_connections(20, 64), 3)
        self.assertEqual(recommended_connections(21, 64), 4)

    def test_default_concurrency_128(self) -> None:
        self.assertEqual(recommended_connections(128, 64), 5)

    def test_large_concurrency_22026(self) -> None:
        self.assertEqual(recommended_connections(22026, 64), 10)


if __name__ == "__main__":
    unittest.main()
