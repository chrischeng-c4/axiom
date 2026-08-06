from __future__ import annotations

from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from metrics_prometheus.domain.bucket import Bucket, assign, cumulative


class TestDomainBucket(unittest.TestCase):
    def setUp(self) -> None:
        self.bounds = (
            Bucket(label="10", upper_bound=10),
            Bucket(label="100", upper_bound=100),
            Bucket(label="1000", upper_bound=1000),
        )

    def test_assign_value_below_first_bound(self) -> None:
        self.assertEqual(assign(self.bounds, 5), 0)

    def test_assign_value_equal_to_bound(self) -> None:
        self.assertEqual(assign(self.bounds, 10), 0)

    def test_assign_value_between_bounds(self) -> None:
        self.assertEqual(assign(self.bounds, 11), 1)

    def test_assign_value_equal_to_last_bound(self) -> None:
        self.assertEqual(assign(self.bounds, 1000), 2)

    def test_assign_value_above_every_bound(self) -> None:
        self.assertIsNone(assign(self.bounds, 1001))

    def test_assign_empty_bounds(self) -> None:
        self.assertIsNone(assign((), 50))

    def test_cumulative_sum(self) -> None:
        self.assertEqual(cumulative((1, 0, 2)), (1, 1, 3))
        self.assertEqual(cumulative((0, 0, 0)), (0, 0, 0))
        self.assertEqual(cumulative((5, 10, 15)), (5, 15, 30))


if __name__ == "__main__":
    unittest.main()
