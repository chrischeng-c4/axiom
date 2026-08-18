from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.domain.errors import AppliedIndexError
from raft_runtime.infrastructure.applied_index_file import (
    decode_applied_index,
    encode_applied_index,
)


class TestInfrastructureAppliedIndexFile(unittest.TestCase):
    def test_encode_applied_index_ascii_bytes(self) -> None:
        self.assertEqual(encode_applied_index(0), b"0")
        self.assertEqual(encode_applied_index(42), b"42")

    def test_decode_applied_index_none_returns_zero(self) -> None:
        self.assertEqual(decode_applied_index(None), 0)

    def test_decode_applied_index_empty_and_whitespace_returns_zero(
        self,
    ) -> None:
        self.assertEqual(decode_applied_index(b""), 0)
        self.assertEqual(decode_applied_index(b"   "), 0)

    def test_decode_applied_index_valid_payload_with_whitespace(self) -> None:
        self.assertEqual(decode_applied_index(b" 42 \n"), 42)

    def test_decode_applied_index_garbage_raises_applied_index_error(
        self,
    ) -> None:
        with self.assertRaises(AppliedIndexError):
            decode_applied_index(b"garbage")

    def test_decode_applied_index_negative_raises_applied_index_error(
        self,
    ) -> None:
        with self.assertRaises(AppliedIndexError):
            decode_applied_index(b"-1")

    def test_decode_applied_index_float_raises_applied_index_error(
        self,
    ) -> None:
        with self.assertRaises(AppliedIndexError):
            decode_applied_index(b"4.2")

    def test_decode_applied_index_invalid_utf8_raises_applied_index_error(
        self,
    ) -> None:
        with self.assertRaises(AppliedIndexError):
            decode_applied_index(b"\xff\xfe")

    def test_encode_decode_applied_index_round_trip_identity(self) -> None:
        for n in (0, 1, 7, 4096, 2**40):
            self.assertEqual(
                decode_applied_index(encode_applied_index(n)),
                n,
                f"Round trip failed for index {n}",
            )


if __name__ == "__main__":
    unittest.main()
