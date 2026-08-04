from __future__ import annotations

from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from metrics_prometheus.application.accumulators import Counter, Gauge, Histogram, Latency
from metrics_prometheus.domain.bucket import Bucket
from metrics_prometheus.infrastructure.recording_cell import RecordingCellFactory


class TestApplicationAccumulators(unittest.TestCase):
    def setUp(self) -> None:
        self.factory = RecordingCellFactory()

    def test_counter_incr_and_add(self) -> None:
        counter = Counter(self.factory, "requests_total")
        self.assertEqual(counter.get(), 0)
        counter.incr()
        self.assertEqual(counter.get(), 1)
        counter.add(5)
        self.assertEqual(counter.get(), 6)

    def test_counter_negative_delta_raises(self) -> None:
        counter = Counter(self.factory, "requests_total")
        with self.assertRaises(ValueError):
            counter.add(-1)

    def test_gauge_set_and_overwrite(self) -> None:
        gauge = Gauge(self.factory, "temperature")
        self.assertEqual(gauge.get(), 0)
        gauge.set(42)
        self.assertEqual(gauge.get(), 42)
        gauge.set(10)
        self.assertEqual(gauge.get(), 10)

    def test_latency_observe(self) -> None:
        latency = Latency(self.factory, "http_duration_ms")
        self.assertEqual(latency.sum(), 0)
        self.assertEqual(latency.count(), 0)
        latency.observe(150)
        latency.observe(250)
        self.assertEqual(latency.sum(), 400)
        self.assertEqual(latency.count(), 2)

    def test_latency_negative_duration_raises(self) -> None:
        latency = Latency(self.factory, "http_duration_ms")
        with self.assertRaises(ValueError):
            latency.observe(-5)

    def test_histogram_invalid_bounds(self) -> None:
        with self.assertRaises(ValueError):
            Histogram(self.factory, "test", ())

        unordered = (Bucket("10", 10), Bucket("5", 5))
        with self.assertRaises(ValueError):
            Histogram(self.factory, "test", unordered)

    def test_histogram_observe_within_bounds(self) -> None:
        bounds = (Bucket("10", 10), Bucket("100", 100))
        hist = Histogram(self.factory, "req_bytes", bounds)
        hist.observe(5)
        observed = list(self.factory.log)

        self.assertEqual(hist.bucket_counts(), (1, 0))
        self.assertEqual(hist.sum(), 5)
        self.assertEqual(hist.count(), 1)

        expected_log = [
            ("req_bytes_bucket_0", "add"),
            ("req_bytes_sum", "add"),
            ("req_bytes_count", "add"),
        ]
        self.assertEqual(observed, expected_log)

    def test_histogram_observe_overflow(self) -> None:
        bounds = (Bucket("10", 10), Bucket("100", 100))
        factory = RecordingCellFactory()
        hist = Histogram(factory, "req_bytes", bounds)
        hist.observe(500)
        observed = list(factory.log)

        self.assertEqual(hist.bucket_counts(), (0, 0))
        self.assertEqual(hist.sum(), 500)
        self.assertEqual(hist.count(), 1)

        expected_log = [
            ("req_bytes_sum", "add"),
            ("req_bytes_count", "add"),
        ]
        self.assertEqual(observed, expected_log)


if __name__ == "__main__":
    unittest.main()
