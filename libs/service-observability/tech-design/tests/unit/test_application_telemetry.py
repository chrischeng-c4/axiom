from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_observability.application.telemetry import (
    LoggingOnly,
    Otel,
    OtelFallback,
    OtelUnavailable,
    tracing_mode,
    valid_otlp_endpoint,
)
from service_observability.domain.identity import ServiceIdentity
from service_observability.infrastructure.config import ObservabilityConfig


class TestApplicationTelemetry(unittest.TestCase):
    def setUp(self) -> None:
        self.identity = ServiceIdentity(name="obs", version="0.1.0")

    def test_no_endpoint_yields_logging_only(self) -> None:
        config = ObservabilityConfig(otlp_endpoint=None)
        self.assertEqual(tracing_mode(config, self.identity, True), LoggingOnly())
        self.assertEqual(tracing_mode(config, self.identity, False), LoggingOnly())

    def test_valid_endpoint_feature_on(self) -> None:
        config = ObservabilityConfig(otlp_endpoint="http://c:4317")
        mode = tracing_mode(config, self.identity, True)
        self.assertEqual(mode, Otel(endpoint="http://c:4317", identity=self.identity))

    def test_valid_endpoint_feature_off(self) -> None:
        config = ObservabilityConfig(otlp_endpoint="http://c:4317")
        mode = tracing_mode(config, self.identity, False)
        self.assertEqual(
            mode,
            OtelUnavailable(
                endpoint="http://c:4317",
                reason=OtelFallback.FEATURE_DISABLED,
            ),
        )

    def test_invalid_endpoint_validity_precedes_feature_check(self) -> None:
        config = ObservabilityConfig(otlp_endpoint="not-an-endpoint")
        # Feature ON
        self.assertEqual(
            tracing_mode(config, self.identity, True),
            OtelUnavailable(
                endpoint="not-an-endpoint",
                reason=OtelFallback.INVALID_ENDPOINT,
            ),
        )
        # Feature OFF (validity check comes before feature check)
        self.assertEqual(
            tracing_mode(config, self.identity, False),
            OtelUnavailable(
                endpoint="not-an-endpoint",
                reason=OtelFallback.INVALID_ENDPOINT,
            ),
        )

    def test_valid_otlp_endpoint_truth_table(self) -> None:
        self.assertTrue(valid_otlp_endpoint("http://otel-collector:4317"))
        self.assertTrue(valid_otlp_endpoint("https://collector.example:443"))
        self.assertFalse(valid_otlp_endpoint("not-an-endpoint"))
        self.assertFalse(valid_otlp_endpoint(""))
        self.assertFalse(valid_otlp_endpoint("ftp://host:21"))
        self.assertFalse(valid_otlp_endpoint("localhost:4317"))
        self.assertFalse(valid_otlp_endpoint("http:///nohost"))


if __name__ == "__main__":
    unittest.main()
