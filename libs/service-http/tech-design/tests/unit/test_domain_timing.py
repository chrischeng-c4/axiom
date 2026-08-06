from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.domain.timing import (
    BASELINE_METRIC,
    DEFAULT_DISCLOSURE,
    FALLBACK_TOKEN,
    Disclosure,
    Phase,
    drains_phases,
    reveals_phases,
)


class TestDomainTiming(unittest.TestCase):
    def test_default_disclosure(self) -> None:
        self.assertEqual(DEFAULT_DISCLOSURE, Disclosure.TOTAL_ONLY)
        self.assertEqual(BASELINE_METRIC, "app")
        self.assertEqual(FALLBACK_TOKEN, "phase")

    def test_reveals_phases_full(self) -> None:
        self.assertTrue(reveals_phases(Disclosure.FULL))

    def test_reveals_phases_total_only(self) -> None:
        self.assertFalse(reveals_phases(Disclosure.TOTAL_ONLY))

    def test_drains_phases_full(self) -> None:
        self.assertTrue(drains_phases(Disclosure.FULL))

    def test_drains_phases_total_only(self) -> None:
        self.assertFalse(drains_phases(Disclosure.TOTAL_ONLY))

    def test_phase_fields(self) -> None:
        phase = Phase("db", 1_000_000)
        self.assertEqual(phase.name, "db")
        self.assertEqual(phase.duration_ns, 1_000_000)
