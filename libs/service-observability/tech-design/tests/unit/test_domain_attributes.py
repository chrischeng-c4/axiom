from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_observability.domain.attributes import (
    RESERVED_KEYS,
    SENSITIVE_KEYS,
    bounded_attributes,
    bounded_value,
    is_reserved_key,
    is_sensitive_key,
)


class TestDomainAttributes(unittest.TestCase):
    def test_reserved_keys_count_and_exact_matches(self) -> None:
        self.assertEqual(len(RESERVED_KEYS), 14)
        for key in RESERVED_KEYS:
            self.assertTrue(is_reserved_key(key))
        self.assertFalse(is_reserved_key("Severity"))
        self.assertFalse(is_reserved_key("custom_key"))

    def test_sensitive_keys_count_and_matching(self) -> None:
        self.assertEqual(len(SENSITIVE_KEYS), 6)
        self.assertTrue(is_sensitive_key("authorization"))
        self.assertTrue(is_sensitive_key("Authorization"))
        self.assertTrue(is_sensitive_key("X-Authorization"))
        self.assertTrue(is_sensitive_key("http.request.header.authorization"))
        self.assertTrue(is_sensitive_key("headers/cookie"))
        self.assertTrue(is_sensitive_key("req_set_cookie"))
        self.assertTrue(is_sensitive_key("tracestate"))
        self.assertTrue(is_sensitive_key("baggage"))

        # Negative controls
        self.assertFalse(is_sensitive_key("xauthorization"))
        self.assertFalse(is_sensitive_key("authorization_header"))

    def test_bounded_value(self) -> None:
        self.assertEqual(bounded_value("hello"), "hello")
        self.assertEqual(bounded_value("y" * 5000), "y" * 4096)
        self.assertEqual(bounded_value(123), 123)
        self.assertTrue(bounded_value(True) is True)
        self.assertIsNone(bounded_value(None))
        self.assertEqual(bounded_value([1, 2]), "[1, 2]")

    def test_bounded_attributes_sorted_and_screening(self) -> None:
        # Sorted order output
        self.assertEqual(bounded_attributes({"b": 2, "a": 1}), {"a": 1, "b": 2})

        # Reserved & sensitive filtering
        self.assertEqual(bounded_attributes({"severity": "X", "a": 1}), {"a": 1})
        self.assertEqual(bounded_attributes({"Cookie": "c", "a": 1}), {"a": 1})

        # Empty key truncation skipped
        self.assertEqual(bounded_attributes({"": 1, "a": 2}), {"a": 2})

    def test_bounded_attributes_max_64_limit(self) -> None:
        # Create 70 keys k000..k069 plus authorization
        input_map = {f"k{i:03d}": i for i in range(70)}
        input_map["authorization"] = "s"
        result = bounded_attributes(input_map)
        self.assertEqual(len(result), 64)
        self.assertEqual(list(result.keys()), [f"k{i:03d}" for i in range(64)])
        self.assertNotIn("authorization", result)

    def test_bounded_attributes_key_value_byte_truncation(self) -> None:
        long_key = "k" * 200
        long_val = "y" * 5000
        result = bounded_attributes({long_key: long_val})
        self.assertEqual(len(list(result.keys())[0].encode("utf-8")), 128)
        self.assertEqual(len(list(result.values())[0].encode("utf-8")), 4096)


if __name__ == "__main__":
    unittest.main()
