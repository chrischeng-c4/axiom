from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.application.body_limit import (
    DEFAULT_BODY_LIMIT_BYTES,
    BodyOutcome,
    classify,
    rewrite_status,
)


class TestApplicationBodyLimit(unittest.TestCase):
    def test_default_body_limit_constant(self) -> None:
        self.assertEqual(DEFAULT_BODY_LIMIT_BYTES, 8388608)

    def test_classify_declared_rejection_zero_streamed(self) -> None:
        self.assertEqual(
            classify("999999", 0, 8), BodyOutcome.REJECTED_DECLARED
        )

    def test_classify_streamed_rejection_no_header(self) -> None:
        self.assertEqual(classify(None, 9, 8), BodyOutcome.REJECTED_STREAMED)

    def test_classify_pass_at_limit(self) -> None:
        self.assertEqual(classify("8", 8, 8), BodyOutcome.PASS)
        self.assertEqual(classify(None, 8, 8), BodyOutcome.PASS)

    def test_classify_pass_unparseable_header(self) -> None:
        self.assertEqual(classify("abc", 0, 8), BodyOutcome.PASS)

    def test_rewrite_status(self) -> None:
        err = rewrite_status(413)
        self.assertIsNotNone(err)
        self.assertEqual(err.kind, "payload_too_large")
        self.assertEqual(err.status, 413)

        self.assertIsNone(rewrite_status(400))
        self.assertIsNone(rewrite_status(200))
