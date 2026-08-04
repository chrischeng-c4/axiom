from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_observability.domain.correlation import (
    field_string,
    preferred_hex,
    preferred_request_id,
    valid_lower_hex,
    valid_request_id,
)


class TestDomainCorrelation(unittest.TestCase):
    def test_valid_lower_hex(self) -> None:
        self.assertTrue(valid_lower_hex("0" * 31 + "1", 32, True))
        self.assertFalse(valid_lower_hex("0" * 32, 32, True))
        self.assertTrue(valid_lower_hex("0" * 32, 32, False))
        self.assertFalse(valid_lower_hex("A" * 32, 32, True))
        self.assertFalse(valid_lower_hex("g" + "0" * 30 + "1", 32, True))
        self.assertFalse(valid_lower_hex("1" * 31, 32, True))
        self.assertFalse(valid_lower_hex("1" * 33, 32, True))
        self.assertTrue(valid_lower_hex("00", 2, False))
        self.assertTrue(valid_lower_hex("01", 2, True))
        self.assertFalse(valid_lower_hex("0" * 16, 16, True))

    def test_valid_request_id(self) -> None:
        self.assertTrue(valid_request_id("request-42"))
        self.assertFalse(valid_request_id(""))
        self.assertTrue(valid_request_id("a" * 128))
        self.assertFalse(valid_request_id("a" * 129))
        self.assertFalse(valid_request_id("a\nb"))
        self.assertFalse(valid_request_id("a\tb"))
        self.assertFalse(valid_request_id("a\x00b"))
        self.assertTrue(valid_request_id("日" * 42))  # 126 bytes
        self.assertFalse(valid_request_id("日" * 43))  # 129 bytes

    def test_field_string(self) -> None:
        self.assertEqual(field_string({"k": "v"}, "k"), "v")
        self.assertIsNone(field_string({"k": 123}, "k"))
        self.assertIsNone(field_string({"k": True}, "k"))
        self.assertIsNone(field_string({}, "k"))

    def test_preferred_hex(self) -> None:
        # Event value takes precedence
        self.assertEqual(
            preferred_hex({"trace_id": "1" * 32}, {"trace_id": "2" * 32}, "trace_id", 32, True),
            "1" * 32,
        )
        # Invalid event value falls through to valid span value
        self.assertEqual(
            preferred_hex({"trace_id": "zz"}, {"trace_id": "2" * 32}, "trace_id", 32, True),
            "2" * 32,
        )
        # Non-string event value falls through
        self.assertEqual(
            preferred_hex({"trace_id": 7}, {"trace_id": "2" * 32}, "trace_id", 32, True),
            "2" * 32,
        )
        # Empty maps yield None
        self.assertIsNone(
            preferred_hex({}, {}, "trace_id", 32, True)
        )

    def test_preferred_request_id(self) -> None:
        # Source outranks spelling: event http.request.id outranks span request_id
        self.assertEqual(
            preferred_request_id({"http.request.id": "E"}, {"request_id": "S"}),
            "E",
        )
        # Span http.request.id used when event is empty
        self.assertEqual(
            preferred_request_id({}, {"http.request.id": "S"}),
            "S",
        )
        # Invalid first spelling falls through to next valid spelling in same source
        self.assertEqual(
            preferred_request_id({"request_id": "", "request.id": "R"}, {}),
            "R",
        )


if __name__ == "__main__":
    unittest.main()
