from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.domain.errors import (
    ErrorEnvelope,
    InvalidPolicy,
    InvalidValue,
    OrphanedCommonSetting,
    describe,
    envelope_fields,
    envelope_of,
    payload_too_large,
    rate_limited,
)


class TestDomainErrors(unittest.TestCase):
    def test_envelope_fields_order(self) -> None:
        env = ErrorEnvelope("a", "b")
        self.assertEqual(envelope_fields(env), (("error", "a"), ("message", "b")))

    def test_envelope_of_rate_limited(self) -> None:
        err = rate_limited()
        env = envelope_of(err)
        self.assertEqual(
            env, ErrorEnvelope("rate_limited", "request admission limit exceeded")
        )

    def test_envelope_of_payload_too_large(self) -> None:
        err = payload_too_large()
        env = envelope_of(err)
        self.assertEqual(
            env,
            ErrorEnvelope(
                "payload_too_large",
                "request body exceeds the configured size limit",
            ),
        )

    def test_status_codes_and_kinds(self) -> None:
        rl = rate_limited()
        self.assertEqual(rl.status, 429)
        self.assertEqual(rl.kind, "rate_limited")

        ptl = payload_too_large()
        self.assertEqual(ptl.status, 413)
        self.assertEqual(ptl.kind, "payload_too_large")

    def test_rate_limited_fresh_instances(self) -> None:
        self.assertIsNot(rate_limited(), rate_limited())

    def test_describe_invalid_value(self) -> None:
        err = InvalidValue("X_ADMISSION_READ_CAPACITY", "abc")
        desc = describe(err)
        self.assertEqual(
            desc,
            "X_ADMISSION_READ_CAPACITY must be a positive integer, got `abc`",
        )
        self.assertIn("X_ADMISSION_READ_CAPACITY", desc)

    def test_describe_orphaned_common_setting(self) -> None:
        err = OrphanedCommonSetting("X_ADMISSION_REFILL_SECS")
        desc = describe(err)
        expected = (
            "X_ADMISSION_REFILL_SECS is set but no admission capacity is"
            " configured; set at least one capacity key or remove"
            " X_ADMISSION_REFILL_SECS"
        )
        self.assertEqual(desc, expected)
        self.assertIn("X_ADMISSION_REFILL_SECS", desc)
        self.assertEqual(desc.count("X_ADMISSION_REFILL_SECS"), 2)
        self.assertNotIn("\n", desc)

    def test_describe_invalid_policy(self) -> None:
        err = InvalidPolicy("read", "capacity must be positive")
        desc = describe(err)
        self.assertEqual(
            desc,
            "admission policy for class `read` is invalid: capacity must be"
            " positive",
        )
        self.assertIn("`read`", desc)
        self.assertIn("capacity must be positive", desc)
