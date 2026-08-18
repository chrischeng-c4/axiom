from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from service_k8s.domain.digest import hex_sha256


class TestDomainDigest(unittest.TestCase):
    def test_empty_input_known_hash(self) -> None:
        expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        self.assertEqual(hex_sha256(b""), expected)

    def test_length_is_always_64(self) -> None:
        inputs = [b"", b"hello", b"lumen-test-data-payload-12345"]
        for data in inputs:
            self.assertEqual(len(hex_sha256(data)), 64)

    def test_lowercase_hex_digits(self) -> None:
        digest = hex_sha256(b"sample data for hex check")
        valid_chars = set("0123456789abcdef")
        self.assertTrue(all(c in valid_chars for c in digest))
        self.assertFalse(any(c.isupper() for c in digest))

    def test_deterministic_and_distinct(self) -> None:
        d1_a = hex_sha256(b"input_one")
        d1_b = hex_sha256(b"input_one")
        d2 = hex_sha256(b"input_two")
        self.assertEqual(d1_a, d1_b)
        self.assertNotEqual(d1_a, d2)


if __name__ == "__main__":
    unittest.main()
