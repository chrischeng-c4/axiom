from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.application.server_timing import PhaseCollector
from service_http.domain.timing import Disclosure


class TestApplicationServerTiming(unittest.TestCase):
    def test_total_only_render_leaves_pending(self) -> None:
        collector = PhaseCollector()
        collector.push("db", 1_000_000)
        collector.push("cache", 500_000)

        header = collector.render(2_000_000, Disclosure.TOTAL_ONLY)
        self.assertEqual(header, "app;dur=2.000")
        self.assertEqual(len(collector.pending()), 2)

    def test_full_render_empties_pending(self) -> None:
        collector = PhaseCollector()
        collector.push("db", 1_000_000)
        collector.push("cache", 500_000)

        header = collector.render(2_000_000, Disclosure.FULL)
        self.assertEqual(
            header, "app;dur=2.000, db;dur=1.000, cache;dur=0.500"
        )
        self.assertEqual(len(collector.pending()), 0)

    def test_collectors_independent(self) -> None:
        c1 = PhaseCollector()
        c2 = PhaseCollector()

        c1.push("db", 1_000_000)
        self.assertEqual(len(c1.pending()), 1)
        self.assertEqual(len(c2.pending()), 0)

    def test_pending_is_tuple(self) -> None:
        collector = PhaseCollector()
        collector.push("db", 1_000_000)
        self.assertIsInstance(collector.pending(), tuple)

    def test_pending_names_in_push_order(self) -> None:
        collector = PhaseCollector()
        collector.push("db", 1)
        collector.push("render", 2)
        names = tuple(p.name for p in collector.pending())
        self.assertEqual(names, ("db", "render"))
