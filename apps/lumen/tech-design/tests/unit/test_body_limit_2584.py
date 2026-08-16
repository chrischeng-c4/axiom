"""Unit tests for Lumen request body limit decisions (#2584) outside EC matrix."""
import unittest

import _design_path  # noqa: F401

from lumen.body_limit.admission import (
    DEFAULT_BODY_LIMIT_BYTES,
    MAX_BODY_LIMIT_BYTES,
    MIN_BODY_LIMIT_BYTES,
    decide_body_limit_spec,
)
from lumen.body_limit.spec import BodyLimitSpec
from lumen.body_limit.verdict import (
    AdmittedBodyLimit,
    Rejection,
    RejectionReason,
)


class TestBodyLimitAdmission2584(unittest.TestCase):
    def test_additional_valid_overrides_admitted(self) -> None:
        valid_candidates = (4096, 1048576, 52428800)
        for val in valid_candidates:
            with self.subTest(val=val):
                res = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=val))
                self.assertIsInstance(res, AdmittedBodyLimit)
                assert isinstance(res, AdmittedBodyLimit)
                self.assertEqual(res.configured_limit_bytes, val)
                self.assertEqual(res.effective_limit_bytes, val)

    def test_boolean_false_rejected_as_non_integer(self) -> None:
        res = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=False))
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, RejectionReason.NOT_INTEGER)
        self.assertEqual(res.reason.value, "body_limit_not_integer")
        self.assertEqual(res.field_path, "body_limit_bytes")

    def test_string_inputs_rejected_as_non_integer(self) -> None:
        string_candidates = ("8388608", "16777216", "invalid", "")
        for candidate in string_candidates:
            with self.subTest(candidate=candidate):
                res = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=candidate))
                self.assertIsInstance(res, Rejection)
                assert isinstance(res, Rejection)
                self.assertEqual(res.reason, RejectionReason.NOT_INTEGER)
                self.assertEqual(res.field_path, "body_limit_bytes")

    def test_float_inputs_rejected_as_non_integer(self) -> None:
        float_candidates = (8388608.0, 1.5, -1.5, 0.0)
        for candidate in float_candidates:
            with self.subTest(candidate=candidate):
                res = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=candidate))
                self.assertIsInstance(res, Rejection)
                assert isinstance(res, Rejection)
                self.assertEqual(res.reason, RejectionReason.NOT_INTEGER)
                self.assertEqual(res.field_path, "body_limit_bytes")

    def test_object_and_collection_inputs_rejected_as_non_integer(self) -> None:
        complex_candidates = ([], {}, object(), (1, 2))
        for candidate in complex_candidates:
            with self.subTest(candidate=type(candidate)):
                res = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=candidate))
                self.assertIsInstance(res, Rejection)
                assert isinstance(res, Rejection)
                self.assertEqual(res.reason, RejectionReason.NOT_INTEGER)
                self.assertEqual(res.field_path, "body_limit_bytes")

    def test_additional_out_of_range_negative_integers_rejected(self) -> None:
        negative_candidates = (-500, -99999, -2**31)
        for candidate in negative_candidates:
            with self.subTest(candidate=candidate):
                res = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=candidate))
                self.assertIsInstance(res, Rejection)
                assert isinstance(res, Rejection)
                self.assertEqual(res.reason, RejectionReason.OUT_OF_RANGE)
                self.assertEqual(res.reason.value, "body_limit_out_of_range")
                self.assertEqual(res.field_path, "body_limit_bytes")

    def test_additional_overflow_integers_rejected(self) -> None:
        overflow_candidates = (18446744073709551616 + 1000, 2**65, 2**128)
        for candidate in overflow_candidates:
            with self.subTest(candidate=candidate):
                res = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=candidate))
                self.assertIsInstance(res, Rejection)
                assert isinstance(res, Rejection)
                self.assertEqual(res.reason, RejectionReason.OUT_OF_RANGE)
                self.assertEqual(res.field_path, "body_limit_bytes")

    def test_constants_and_bounds(self) -> None:
        self.assertEqual(DEFAULT_BODY_LIMIT_BYTES, 8388608)
        self.assertEqual(MIN_BODY_LIMIT_BYTES, 1)
        self.assertEqual(MAX_BODY_LIMIT_BYTES, 18446744073709551615)


if __name__ == "__main__":
    unittest.main()
