from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_backup.domain.policy import Retention, ScheduledBackupPolicy
from service_backup.infrastructure.wire import (
    is_structural,
    retention_to_json,
    scheduled_policy_schema,
    scheduled_policy_to_json,
)


class TestInfrastructureWire(unittest.TestCase):
    def test_retention_to_json(self) -> None:
        self.assertEqual(retention_to_json(Retention()), {})
        self.assertEqual(retention_to_json(Retention(None)), {})
        self.assertEqual(retention_to_json(Retention(0)), {"maxAgeSeconds": 0})
        self.assertEqual(retention_to_json(Retention(3600)), {"maxAgeSeconds": 3600})

    def test_scheduled_policy_to_json_with_retention(self) -> None:
        p = ScheduledBackupPolicy("0 * * * *", "s3://bucket/prefix", 3600)
        expected = {
            "schedule": "0 * * * *",
            "destination": "s3://bucket/prefix",
            "retentionSecs": 3600,
        }
        self.assertEqual(scheduled_policy_to_json(p), expected)

    def test_scheduled_policy_to_json_without_retention(self) -> None:
        p = ScheduledBackupPolicy("0 * * * *", "s3://bucket/prefix")
        res = scheduled_policy_to_json(p)
        self.assertNotIn("retentionSecs", res)
        self.assertEqual(
            res,
            {"schedule": "0 * * * *", "destination": "s3://bucket/prefix"},
        )

    def test_scheduled_policy_to_json_zero_retention(self) -> None:
        p = ScheduledBackupPolicy("s", "d", 0)
        self.assertEqual(
            scheduled_policy_to_json(p),
            {"schedule": "s", "destination": "d", "retentionSecs": 0},
        )

    def test_scheduled_policy_schema_flatness(self) -> None:
        schema = scheduled_policy_schema()
        props = schema.get("properties", {})
        assert isinstance(props, dict)
        dest_prop = props.get("destination", {})
        assert isinstance(dest_prop, dict)
        self.assertEqual(dest_prop.get("type"), "string")
        self.assertIn("schedule", props)
        self.assertIn("retentionSecs", props)

    def test_is_structural(self) -> None:
        self.assertTrue(is_structural(scheduled_policy_schema()))
        self.assertFalse(is_structural({"oneOf": []}))
        self.assertFalse(is_structural({"properties": {"x": {"oneOf": []}}}))
        self.assertFalse(is_structural({"a": [{"b": {"anyOf": []}}]}))
        self.assertFalse(is_structural({"a": [{"b": {"allOf": []}}]}))
        self.assertTrue(is_structural({"type": "object"}))
        self.assertTrue(is_structural("string"))
        self.assertTrue(is_structural(None))


if __name__ == "__main__":
    unittest.main()
