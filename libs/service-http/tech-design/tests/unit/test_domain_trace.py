from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.domain.trace import (
    DEFAULT_TRACE_FLAGS,
    HYPHEN_POSITIONS,
    PARENT_SPAN_ID_SPAN,
    SUPPORTED_VERSION,
    TRACE_FLAGS_SPAN,
    TRACE_ID_SPAN,
    TRACEPARENT_LENGTH,
    VERSION_SPAN,
    TraceContext,
    is_all_zero,
    is_local_root,
    is_lower_hex,
)


class TestDomainTrace(unittest.TestCase):
    def test_is_lower_hex_lowercase(self) -> None:
        self.assertTrue(is_lower_hex("00ff"))

    def test_is_lower_hex_uppercase(self) -> None:
        self.assertFalse(is_lower_hex("00FF"))

    def test_is_lower_hex_non_hex(self) -> None:
        self.assertFalse(is_lower_hex("00g0"))

    def test_is_lower_hex_empty(self) -> None:
        self.assertTrue(is_lower_hex(""))

    def test_is_all_zero_zeros(self) -> None:
        self.assertTrue(is_all_zero("0000"))

    def test_is_all_zero_mixed(self) -> None:
        self.assertFalse(is_all_zero("0001"))

    def test_is_all_zero_empty(self) -> None:
        self.assertTrue(is_all_zero(""))

    def test_constants_and_spans(self) -> None:
        sample = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
        self.assertEqual(len(sample), TRACEPARENT_LENGTH)
        self.assertEqual(SUPPORTED_VERSION, "00")
        self.assertEqual(DEFAULT_TRACE_FLAGS, "00")
        self.assertEqual(sample[VERSION_SPAN[0] : VERSION_SPAN[1]], "00")
        self.assertEqual(
            sample[TRACE_ID_SPAN[0] : TRACE_ID_SPAN[1]],
            "4bf92f3577b34da6a3ce929d0e0e4736",
        )
        self.assertEqual(
            sample[PARENT_SPAN_ID_SPAN[0] : PARENT_SPAN_ID_SPAN[1]],
            "00f067aa0ba902b7",
        )
        self.assertEqual(
            sample[TRACE_FLAGS_SPAN[0] : TRACE_FLAGS_SPAN[1]], "01"
        )
        self.assertTrue(TRACE_FLAGS_SPAN[1] == TRACEPARENT_LENGTH)
        for pos in HYPHEN_POSITIONS:
            self.assertEqual(sample[pos], "-")

    def test_is_local_root_true(self) -> None:
        ctx = TraceContext("t", "s", None, "00")
        self.assertTrue(is_local_root(ctx))

    def test_is_local_root_false(self) -> None:
        ctx = TraceContext("t", "s", "p", "00")
        self.assertFalse(is_local_root(ctx))
