from __future__ import annotations

from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from metrics_prometheus.domain.scaling import scale_decimal


class TestDomainScaling(unittest.TestCase):
    def test_power_of_ten_divisor_with_zero_padding(self) -> None:
        self.assertEqual(scale_decimal(1, 1000), "0.001")
        self.assertEqual(scale_decimal(2, 1000000), "0.000002")

    def test_power_of_ten_divisor_standard(self) -> None:
        self.assertEqual(scale_decimal(1500, 1000), "1.500")

    def test_divisor_of_one(self) -> None:
        self.assertEqual(scale_decimal(7, 1), "7")

    def test_divisor_of_zero(self) -> None:
        self.assertEqual(scale_decimal(7, 0), "7")

    def test_non_power_of_ten_divisor_degrades(self) -> None:
        self.assertEqual(scale_decimal(7, 3), "2")

    def test_large_value_exceeding_float_precision(self) -> None:
        self.assertEqual(scale_decimal(9007199254740993, 1000), "9007199254740.993")

    def test_negative_values(self) -> None:
        with self.assertRaises(ValueError):
            scale_decimal(-1500, 1000)
        with self.assertRaises(ValueError):
            scale_decimal(1500, -1000)

    def test_zero_value(self) -> None:
        self.assertEqual(scale_decimal(0, 1000), "0.000")


if __name__ == "__main__":
    unittest.main()
