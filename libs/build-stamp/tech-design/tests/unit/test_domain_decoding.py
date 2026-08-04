from __future__ import annotations

from pathlib import Path
import sys
import unittest

SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
if str(SRC_ROOT) not in sys.path:
    sys.path.insert(0, str(SRC_ROOT))

from build_stamp.domain.build_time import format_built_at
from build_stamp.domain.fallback import UNKNOWN
from build_stamp.domain.sha import decode_short_sha
from build_stamp.domain.target import decode_target


class TestDomainDecoding(unittest.TestCase):
    def test_decode_short_sha_failure(self) -> None:
        self.assertIsNone(decode_short_sha(False, b"c3ff13cd"))

    def test_decode_short_sha_empty(self) -> None:
        self.assertIsNone(decode_short_sha(True, b""))

    def test_decode_short_sha_whitespace_only(self) -> None:
        self.assertIsNone(decode_short_sha(True, b"   \n\t "))

    def test_decode_short_sha_trailing_newline(self) -> None:
        self.assertEqual(decode_short_sha(True, b"c3ff13cd\n"), "c3ff13cd")

    def test_decode_short_sha_surrounding_spaces(self) -> None:
        self.assertEqual(decode_short_sha(True, b"  c3ff13cd  "), "c3ff13cd")

    def test_decode_short_sha_invalid_utf8(self) -> None:
        self.assertEqual(decode_short_sha(True, b"\xff\xfe ab \n"), "\ufffd\ufffd ab")

    def test_decode_target_none(self) -> None:
        self.assertEqual(decode_target(None), UNKNOWN)

    def test_decode_target_empty_str(self) -> None:
        self.assertEqual(decode_target(""), "")

    def test_decode_target_valid(self) -> None:
        self.assertEqual(decode_target("aarch64-apple-darwin"), "aarch64-apple-darwin")

    def test_format_built_at_positive(self) -> None:
        self.assertEqual(format_built_at(1700000000), "1700000000")

    def test_format_built_at_zero(self) -> None:
        self.assertEqual(format_built_at(0), "0")

    def test_format_built_at_negative(self) -> None:
        self.assertEqual(format_built_at(-1), UNKNOWN)


if __name__ == "__main__":
    unittest.main()
