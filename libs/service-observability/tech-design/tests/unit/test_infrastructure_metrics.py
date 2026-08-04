from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_observability.infrastructure.metrics import LifecycleMetrics


class TestInfrastructureMetrics(unittest.TestCase):
    def test_fresh_instance_renders_zeros(self) -> None:
        metrics = LifecycleMetrics()
        self.assertEqual(metrics.accepted(), 0)
        self.assertEqual(metrics.rejected(), 0)
        self.assertEqual(metrics.closed(), 0)

        expected = (
            "# HELP service_connections_accepted_total Total accepted service connections.\n"
            "# TYPE service_connections_accepted_total counter\n"
            "service_connections_accepted_total 0\n"
            "# HELP service_connections_rejected_total Total service connections rejected by admission.\n"
            "# TYPE service_connections_rejected_total counter\n"
            "service_connections_rejected_total 0\n"
            "# HELP service_connections_closed_total Total completed or failed service connections.\n"
            "# TYPE service_connections_closed_total counter\n"
            "service_connections_closed_total 0\n"
        )
        self.assertEqual(metrics.render_metrics(), expected)

    def test_counter_increments_and_rendered_output(self) -> None:
        metrics = LifecycleMetrics()
        metrics.connection_accepted()
        metrics.connection_accepted()
        metrics.connection_rejected()
        metrics.connection_closed()

        self.assertEqual(metrics.accepted(), 2)
        self.assertEqual(metrics.rejected(), 1)
        self.assertEqual(metrics.closed(), 1)

        expected = (
            "# HELP service_connections_accepted_total Total accepted service connections.\n"
            "# TYPE service_connections_accepted_total counter\n"
            "service_connections_accepted_total 2\n"
            "# HELP service_connections_rejected_total Total service connections rejected by admission.\n"
            "# TYPE service_connections_rejected_total counter\n"
            "service_connections_rejected_total 1\n"
            "# HELP service_connections_closed_total Total completed or failed service connections.\n"
            "# TYPE service_connections_closed_total counter\n"
            "service_connections_closed_total 1\n"
        )
        self.assertEqual(metrics.render_metrics(), expected)


if __name__ == "__main__":
    unittest.main()
