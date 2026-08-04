from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_observability.domain.bounds import SERVICE_LOG_SCHEMA_V1
from service_observability.infrastructure.envelope import (
    LogEventV1,
    LogIdentityV1,
    to_json_line,
    to_mapping,
)


class TestInfrastructureEnvelope(unittest.TestCase):
    def test_event_without_correlation_fields(self) -> None:
        event = LogEventV1(
            schema=SERVICE_LOG_SCHEMA_V1,
            timestamp="2026-08-04T12:00:00Z",
            severity="INFO",
            service=LogIdentityV1(name="test-svc", version="1.0.0"),
            event="user_login",
            message="User logged in",
        )
        mapping = to_mapping(event)
        self.assertNotIn("trace_id", mapping)
        self.assertNotIn("span_id", mapping)
        self.assertNotIn("parent_span_id", mapping)
        self.assertNotIn("trace_flags", mapping)
        self.assertNotIn("request_id", mapping)
        self.assertIn("attributes", mapping)
        self.assertEqual(mapping["attributes"], {})

    def test_event_with_all_correlation_fields(self) -> None:
        event = LogEventV1(
            schema=SERVICE_LOG_SCHEMA_V1,
            timestamp="2026-08-04T12:00:00Z",
            severity="INFO",
            service=LogIdentityV1(name="test-svc", version="1.0.0"),
            event="user_login",
            message="User logged in",
            trace_id="1" * 32,
            span_id="2" * 16,
            parent_span_id="3" * 16,
            trace_flags="01",
            request_id="req-123",
            attributes={"env": "prod"},
        )
        mapping = to_mapping(event)
        self.assertEqual(mapping["trace_id"], "1" * 32)
        self.assertEqual(mapping["span_id"], "2" * 16)
        self.assertEqual(mapping["parent_span_id"], "3" * 16)
        self.assertEqual(mapping["trace_flags"], "01")
        self.assertEqual(mapping["request_id"], "req-123")

    def test_to_json_line_single_line_and_roundtrip(self) -> None:
        event = LogEventV1(
            schema=SERVICE_LOG_SCHEMA_V1,
            timestamp="2026-08-04T12:00:00Z",
            severity="ERROR",
            service=LogIdentityV1(name="svc", version="1.0"),
            event="failure",
            message="line1\nline2",
        )
        line = to_json_line(event)
        self.assertNotIn("\n", line)
        parsed = json.loads(line)
        self.assertEqual(parsed["message"], "line1\nline2")


if __name__ == "__main__":
    unittest.main()
