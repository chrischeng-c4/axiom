from __future__ import annotations

from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from metrics_prometheus.application.accumulators import Histogram
from metrics_prometheus.application.exposition import (
    render,
    render_histogram,
    render_label_set,
    render_labeled,
)
from metrics_prometheus.domain.bucket import Bucket
from metrics_prometheus.domain.sample import Label, LabeledSample, MetricKind, Sample, SampleGroup
from metrics_prometheus.infrastructure.recording_cell import RecordingCellFactory


class TestApplicationExposition(unittest.TestCase):
    def test_render_unlabeled_samples(self) -> None:
        samples = (
            Sample(name="http_requests_total", kind=MetricKind.COUNTER, help="Total HTTP requests", value=3),
            Sample(name="active_connections", kind=MetricKind.GAUGE, help="Active connections", value=42),
        )
        expected = (
            "# HELP http_requests_total Total HTTP requests\n"
            "# TYPE http_requests_total counter\n"
            "http_requests_total 3\n"
            "# HELP active_connections Active connections\n"
            "# TYPE active_connections gauge\n"
            "active_connections 42\n"
        )
        self.assertEqual(render(samples), expected)

    def test_render_label_set_canonical_ordering(self) -> None:
        labels1 = (Label(name="method", value="GET"), Label(name="status", value="200"))
        labels2 = (Label(name="status", value="200"), Label(name="method", value="GET"))

        rendered1 = render_label_set(labels1)
        rendered2 = render_label_set(labels2)

        self.assertEqual(rendered1, '{method="GET",status="200"}')
        self.assertEqual(rendered1, rendered2)

    def test_render_label_set_empty(self) -> None:
        self.assertEqual(render_label_set(()), "")

    def test_render_labeled_sample_group(self) -> None:
        group = SampleGroup(
            name="http_requests",
            kind=MetricKind.COUNTER,
            help="HTTP requests count",
            samples=(
                LabeledSample(
                    labels=(Label(name="path", value="/api\n"), Label(name="env", value='prod"')),
                    value=10,
                ),
            ),
        )
        expected = (
            "# HELP http_requests HTTP requests count\n"
            "# TYPE http_requests counter\n"
            'http_requests{env="prod\\"",path="/api\\n"} 10\n'
        )
        self.assertEqual(render_labeled((group,)), expected)

    def test_render_histogram(self) -> None:
        factory = RecordingCellFactory()
        bounds = (Bucket("10", 10), Bucket("100", 100))
        hist = Histogram(factory, "req_latency", bounds)

        hist.observe(5)
        hist.observe(500)

        output = render_histogram(hist, name="req_latency", help="Request latency in seconds", divisor=1000)
        expected = (
            "# HELP req_latency Request latency in seconds\n"
            "# TYPE req_latency histogram\n"
            'req_latency_bucket{le="10"} 1\n'
            'req_latency_bucket{le="100"} 1\n'
            'req_latency_bucket{le="+Inf"} 2\n'
            "req_latency_sum 0.505\n"
            "req_latency_count 2\n"
        )
        self.assertEqual(output, expected)


if __name__ == "__main__":
    unittest.main()
