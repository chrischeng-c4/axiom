from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.domain.errors import DigestMismatch
from cli_std.infrastructure.digest import sha256_hex, verify_sha256


class TestInfrastructureDigest(unittest.TestCase):
    def test_sha256_hex_empty_string(self) -> None:
        expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        self.assertEqual(sha256_hex(b""), expected)

    def test_verify_sha256_uppercase_and_whitespace(self) -> None:
        upper_expected = (
            "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855\n"
        )
        self.assertIsNone(verify_sha256(b"", upper_expected))

    def test_verify_sha256_mismatch(self) -> None:
        res = verify_sha256(b"", "  DEADBEEF  \n")
        self.assertIsInstance(res, DigestMismatch)
        if isinstance(res, DigestMismatch):
            self.assertEqual(res.expected, "DEADBEEF")
            self.assertEqual(
                res.actual,
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            )

    def test_sha256_hex_differs_on_mutation(self) -> None:
        d1 = sha256_hex(b"hello")
        d2 = sha256_hex(b"hellp")
        self.assertNotEqual(d1, d2)

    def test_sha256_hex_non_empty_bytes(self) -> None:
        d = sha256_hex(b"test data")
        self.assertEqual(len(d), 64)
        self.assertTrue(all(c in "0123456789abcdef" for c in d))

    def test_verify_sha256_lowercase_match(self) -> None:
        hex_val = sha256_hex(b"payload")
        self.assertIsNone(verify_sha256(b"payload", hex_val))

    def test_verify_sha256_mismatch_strips_expected(self) -> None:
        res = verify_sha256(b"abc", "\t1234\n")
        self.assertIsNotNone(res)
        if isinstance(res, DigestMismatch):
            self.assertEqual(res.expected, "1234")

    def test_sha256_hex_type(self) -> None:
        res = sha256_hex(b"x")
        self.assertIsInstance(res, str)


if __name__ == "__main__":
    unittest.main()
