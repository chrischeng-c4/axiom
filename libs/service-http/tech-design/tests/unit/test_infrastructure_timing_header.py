from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.domain.timing import Disclosure, Phase
from service_http.infrastructure.timing_header import (
    format_ms,
    render_header,
    render_metric,
    sanitize_token,
)


class TestInfrastructureTimingHeader(unittest.TestCase):
    def test_sanitize_token_conforming(self) -> None:
        self.assertEqual(sanitize_token("db.query-1_a"), "db.query-1_a")

    def test_sanitize_token_semicolons(self) -> None:
        self.assertEqual(sanitize_token(";;;"), "___")

    def test_sanitize_token_empty(self) -> None:
        self.assertEqual(sanitize_token(""), "phase")

    def test_sanitize_token_non_ascii(self) -> None:
        self.assertEqual(sanitize_token("café"), "caf_")
        self.assertEqual(sanitize_token("a b,c"), "a_b_c")

    def test_format_ms(self) -> None:
        self.assertEqual(format_ms(0), "0.000")
        self.assertEqual(format_ms(999), "0.001")
        self.assertEqual(format_ms(1_000_000), "1.000")
        self.assertEqual(format_ms(1_500_000), "1.500")
        self.assertEqual(format_ms(12_345_678), "12.346")

    def test_render_metric(self) -> None:
        self.assertEqual(render_metric("db q", 1_500_000), "db_q;dur=1.500")

    def test_render_header_total_only_ignores_phases(self) -> None:
        phases = (Phase("db", 1_000_000),)
        res_with_phases = render_header(2_000_000, Disclosure.TOTAL_ONLY, phases)
        res_empty = render_header(2_000_000, Disclosure.TOTAL_ONLY, ())
        self.assertEqual(res_with_phases, "app;dur=2.000")
        self.assertEqual(res_with_phases, res_empty)

    def test_render_header_full_with_phases(self) -> None:
        phases = (Phase("db", 1_000_000), Phase("cache", 500_000))
        res = render_header(2_000_000, Disclosure.FULL, phases)
        self.assertEqual(
            res, "app;dur=2.000, db;dur=1.000, cache;dur=0.500"
        )
