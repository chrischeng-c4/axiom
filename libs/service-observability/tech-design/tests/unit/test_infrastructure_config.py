from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_observability.infrastructure.config import (
    LogFormat,
    ObservabilityConfig,
    collector_compatible,
)


class TestInfrastructureConfig(unittest.TestCase):
    def test_config_defaults(self) -> None:
        config = ObservabilityConfig()
        self.assertEqual(config.log_level, "info")
        self.assertEqual(config.log_format, LogFormat.JSON)
        self.assertIsNone(config.otlp_endpoint)

    def test_collector_compatible(self) -> None:
        self.assertTrue(collector_compatible(LogFormat.JSON))
        self.assertFalse(collector_compatible(LogFormat.PRETTY))


if __name__ == "__main__":
    unittest.main()
