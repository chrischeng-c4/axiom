from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_observability.application.formatter import (
    EventMetadata,
    format_event,
    merge_attributes,
    preferred_string,
    resolve_event_name,
    resolve_message,
)
from service_observability.domain.identity import ServiceIdentity


class TestApplicationFormatter(unittest.TestCase):
    def setUp(self) -> None:
        self.metadata = EventMetadata(
            name="svc::mod::event",
            target="svc::mod",
            severity="INFO",
        )
        self.identity = ServiceIdentity(name="obs", version="0.1.0")
        self.timestamp = "2026-08-04T00:00:00Z"

    def test_event_with_no_fields(self) -> None:
        event = format_event(
            event_fields={},
            span_fields={},
            metadata=self.metadata,
            identity=self.identity,
            timestamp=self.timestamp,
        )
        self.assertEqual(event.schema, "axiom.service.log.v1")
        self.assertEqual(event.severity, "INFO")
        self.assertEqual(event.event, "svc::mod::event")
        self.assertEqual(event.message, "svc::mod::event")
        self.assertIsNone(event.trace_id)
        self.assertIsNone(event.span_id)
        self.assertIsNone(event.parent_span_id)
        self.assertIsNone(event.trace_flags)
        self.assertIsNone(event.request_id)
        self.assertEqual(event.attributes, {"target": "svc::mod"})

    def test_event_name_resolution_and_empty_fallback(self) -> None:
        # Explicit event field wins
        self.assertEqual(
            resolve_event_name({"event": "accepted"}, {}, self.metadata),
            "accepted",
        )
        # Empty string on event field falls back to callsite name, not span field
        self.assertEqual(
            resolve_event_name({"event": ""}, {"event": "span_ev"}, self.metadata),
            "svc::mod::event",
        )

    def test_event_name_span_field(self) -> None:
        # Span-scoped event field wins when event has none
        self.assertEqual(
            resolve_event_name({}, {"event": "span_ev"}, self.metadata),
            "span_ev",
        )

    def test_message_resolution(self) -> None:
        # Explicit message wins
        self.assertEqual(resolve_message({"message": "hello"}, "ev"), "hello")
        # Explicit empty message DOES win
        self.assertEqual(resolve_message({"message": ""}, "ev"), "")
        # Fallback to event_name when None
        self.assertEqual(resolve_message({}, "ev"), "ev")
        # Span message is ignored
        event = format_event(
            event_fields={},
            span_fields={"message": "S"},
            metadata=self.metadata,
            identity=self.identity,
            timestamp=self.timestamp,
        )
        self.assertEqual(event.message, "svc::mod::event")

    def test_merge_attributes_precedence_and_target(self) -> None:
        # Event wins key collision
        merged = merge_attributes({"k": "E"}, {"k": "S"}, self.metadata)
        self.assertEqual(merged["k"], "E")
        self.assertEqual(merged["target"], "svc::mod")

        # Explicit target preserved
        merged_explicit = merge_attributes({"target": "explicit"}, {}, self.metadata)
        self.assertEqual(merged_explicit["target"], "explicit")

        merged_span = merge_attributes({}, {"target": "span_t"}, self.metadata)
        self.assertEqual(merged_span["target"], "span_t")

    def test_correlation_extraction(self) -> None:
        event = format_event(
            event_fields={
                "trace_id": "1" * 32,
                "span_id": "2" * 16,
                "parent_span_id": "3" * 16,
                "trace_flags": "00",
                "request_id": "r-1",
            },
            span_fields={},
            metadata=self.metadata,
            identity=self.identity,
            timestamp=self.timestamp,
        )
        self.assertEqual(event.trace_id, "1" * 32)
        self.assertEqual(event.span_id, "2" * 16)
        self.assertEqual(event.parent_span_id, "3" * 16)
        self.assertEqual(event.trace_flags, "00")
        self.assertEqual(event.request_id, "r-1")

        # Invalid event value falls through to valid span value
        event_fallthrough = format_event(
            event_fields={"trace_id": "0" * 32},
            span_fields={"trace_id": "2" * 32},
            metadata=self.metadata,
            identity=self.identity,
            timestamp=self.timestamp,
        )
        self.assertEqual(event_fallthrough.trace_id, "2" * 32)

    def test_event_name_and_message_bounds(self) -> None:
        self.assertEqual(
            resolve_event_name({"event": "e" * 200}, {}, self.metadata),
            "e" * 128,
        )
        self.assertEqual(
            resolve_message({"message": "m" * 5000}, "ev"),
            "m" * 4096,
        )

    def test_identity_bounds(self) -> None:
        long_identity = ServiceIdentity(name="n" * 200, version="v" * 200)
        event = format_event(
            event_fields={},
            span_fields={},
            metadata=self.metadata,
            identity=long_identity,
            timestamp=self.timestamp,
        )
        self.assertEqual(event.service.name, "n" * 128)
        self.assertEqual(event.service.version, "v" * 128)


if __name__ == "__main__":
    unittest.main()
