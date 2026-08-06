from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.infrastructure.headers import (
    CONTENT_LENGTH_HEADER,
    CONTENT_TYPE_HEADER,
    DEFAULT_RETRY_AFTER_NS,
    RETRY_AFTER_HEADER,
    SERVER_TIMING_HEADER,
    TRACEPARENT_HEADER,
    content_length_exceeds,
    retry_after_seconds,
    retry_after_value,
)


class TestInfrastructureHeaders(unittest.TestCase):
    def test_constants(self) -> None:
        self.assertEqual(TRACEPARENT_HEADER, "traceparent")
        self.assertEqual(CONTENT_LENGTH_HEADER, "content-length")
        self.assertEqual(RETRY_AFTER_HEADER, "retry-after")
        self.assertEqual(SERVER_TIMING_HEADER, "server-timing")
        self.assertEqual(CONTENT_TYPE_HEADER, "content-type")
        self.assertEqual(DEFAULT_RETRY_AFTER_NS, 1_000_000_000)

    def test_content_length_exceeds_below_at_above(self) -> None:
        self.assertFalse(content_length_exceeds("7", 8))
        self.assertFalse(content_length_exceeds("8", 8))
        self.assertTrue(content_length_exceeds("9", 8))

    def test_content_length_exceeds_none_and_unparseable(self) -> None:
        self.assertFalse(content_length_exceeds(None, 8))
        self.assertFalse(content_length_exceeds("many", 8))
        self.assertFalse(content_length_exceeds("-1", 8))

    def test_retry_after_seconds(self) -> None:
        self.assertEqual(retry_after_seconds(None), 1)
        self.assertEqual(retry_after_seconds(0), 1)
        self.assertEqual(retry_after_seconds(1), 1)
        self.assertEqual(retry_after_seconds(5_000_000_000), 5)
        self.assertEqual(retry_after_seconds(5_000_000_001), 6)
        self.assertEqual(retry_after_seconds(10_000_000_000), 10)

    def test_retry_after_value(self) -> None:
        self.assertEqual(retry_after_value(5_000_000_001), "6")
