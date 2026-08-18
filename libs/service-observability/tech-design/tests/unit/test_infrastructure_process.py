from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_observability.infrastructure.process import (
    ProcessSampleError,
    ProcessUsage,
    parse_cpu_time,
    parse_ps_usage,
)


class TestInfrastructureProcess(unittest.TestCase):
    def test_parse_cpu_time_valid(self) -> None:
        self.assertEqual(parse_cpu_time("0:00.00"), 0.0)
        self.assertEqual(parse_cpu_time("1:02.50"), 62.5)
        self.assertEqual(parse_cpu_time("01:00:00"), 3600.0)
        self.assertEqual(parse_cpu_time("2-03:04:05"), 183845.0)

    def test_parse_cpu_time_invalid(self) -> None:
        with self.assertRaises(ProcessSampleError):
            parse_cpu_time("90")
        with self.assertRaises(ProcessSampleError):
            parse_cpu_time("1:2:3:4")
        with self.assertRaises(ProcessSampleError):
            parse_cpu_time("x-01:00")

    def test_parse_ps_usage_witnesses(self) -> None:
        # Witness 1
        usage1 = parse_ps_usage("  12345 1:02.50\n")
        self.assertEqual(usage1, ProcessUsage(cpu_seconds=62.5, rss_bytes=12_641_280))

        # Witness 2
        usage2 = parse_ps_usage("42 2-03:04:05\n")
        self.assertEqual(
            usage2, ProcessUsage(cpu_seconds=183_845.0, rss_bytes=43_008)
        )

        # Witness 3
        usage3 = parse_ps_usage("100 01:00:00\n")
        self.assertEqual(
            usage3, ProcessUsage(cpu_seconds=3_600.0, rss_bytes=102_400)
        )

    def test_parse_ps_usage_invalid_and_errors(self) -> None:
        with self.assertRaises(ProcessSampleError):
            parse_ps_usage("")
        with self.assertRaises(ProcessSampleError):
            parse_ps_usage("abc 1:00")
        with self.assertRaises(ProcessSampleError):
            parse_ps_usage("123")
        with self.assertRaises(ProcessSampleError):
            parse_ps_usage("123 abc")

    def test_parse_ps_usage_saturating(self) -> None:
        u64_max = 2**64 - 1
        usage = parse_ps_usage(f"{u64_max} 0:01")
        self.assertEqual(usage.rss_bytes, u64_max)


if __name__ == "__main__":
    unittest.main()
