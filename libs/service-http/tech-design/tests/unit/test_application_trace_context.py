from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.application.trace_context import (
    parse_traceparent,
    request_trace_context,
    span_fields,
)
from service_http.domain.trace import TraceParent


class TestApplicationTraceContext(unittest.TestCase):
    def test_valid_traceparent_parsing(self) -> None:
        raw = ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",)
        parsed = parse_traceparent(raw)
        self.assertEqual(
            parsed,
            TraceParent(
                "00",
                "4bf92f3577b34da6a3ce929d0e0e4736",
                "00f067aa0ba902b7",
                "01",
            ),
        )

    def test_request_trace_context_success_fresh_span_id(self) -> None:
        raw = ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",)
        ctx = request_trace_context(raw, "FRESH_TRACE", "FRESH_SPAN")
        self.assertEqual(ctx.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736")
        self.assertEqual(ctx.span_id, "FRESH_SPAN")
        self.assertNotEqual(ctx.span_id, "00f067aa0ba902b7")
        self.assertEqual(ctx.parent_span_id, "00f067aa0ba902b7")
        self.assertEqual(ctx.trace_flags, "01")

    def test_trace_flags_zero_is_valid(self) -> None:
        raw = ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00",)
        parsed = parse_traceparent(raw)
        self.assertIsNotNone(parsed)
        self.assertEqual(parsed.trace_flags, "00")

    def test_all_zero_trace_id_rejected(self) -> None:
        raw = ("00-00000000000000000000000000000000-00f067aa0ba902b7-01",)
        self.assertIsNone(parse_traceparent(raw))

    def test_all_zero_parent_span_id_rejected(self) -> None:
        raw = ("00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01",)
        self.assertIsNone(parse_traceparent(raw))

    def test_uppercase_trace_id_rejected(self) -> None:
        raw = ("00-4BF92F3577B34DA6A3CE929D0E0E4736-00f067aa0ba902b7-01",)
        self.assertIsNone(parse_traceparent(raw))

    def test_wrong_length_rejected(self) -> None:
        raw = ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0",)
        self.assertIsNone(parse_traceparent(raw))

    def test_wrong_version_rejected(self) -> None:
        raw = ("01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",)
        self.assertIsNone(parse_traceparent(raw))

    def test_missing_hyphen_rejected(self) -> None:
        raw = ("00_4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",)
        self.assertIsNone(parse_traceparent(raw))

    def test_multiple_headers_rejected(self) -> None:
        raw = (
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
        )
        self.assertIsNone(parse_traceparent(raw))

    def test_no_headers_empty_tuple(self) -> None:
        ctx = request_trace_context((), "T", "S")
        self.assertEqual(ctx.trace_id, "T")
        self.assertEqual(ctx.span_id, "S")
        self.assertIsNone(ctx.parent_span_id)
        self.assertEqual(ctx.trace_flags, "00")

    def test_non_ascii_rejected(self) -> None:
        raw = ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-éé",)
        self.assertIsNone(parse_traceparent(raw))

    def test_equality_of_no_header_and_malformed_header(self) -> None:
        ctx_none = request_trace_context((), "T", "S")
        ctx_garbage = request_trace_context(("garbage",), "T", "S")
        self.assertEqual(ctx_none, ctx_garbage)

    def test_span_fields_local_root_and_child(self) -> None:
        root_ctx = request_trace_context((), "T", "S")
        root_fields = span_fields(root_ctx)
        self.assertNotIn("parent_span_id", root_fields)
        self.assertEqual(
            root_fields, {"trace_id": "T", "span_id": "S", "trace_flags": "00"}
        )

        child_raw = ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",)
        child_ctx = request_trace_context(child_raw, "T", "S")
        child_fields = span_fields(child_ctx)
        self.assertIn("parent_span_id", child_fields)
        self.assertEqual(
            child_fields["parent_span_id"], "00f067aa0ba902b7"
        )
