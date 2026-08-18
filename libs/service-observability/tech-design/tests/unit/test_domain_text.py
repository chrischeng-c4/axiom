from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_observability.domain.text import byte_len, truncate_utf8


class TestDomainText(unittest.TestCase):
    def test_byte_len(self) -> None:
        self.assertEqual(byte_len("abc"), 3)
        self.assertEqual(byte_len("日本語"), 9)
        self.assertEqual(byte_len(""), 0)

    def test_truncate_utf8_ascii_under_bound(self) -> None:
        self.assertEqual(truncate_utf8("abc", 10), "abc")

    def test_truncate_utf8_ascii_over_bound(self) -> None:
        self.assertEqual(truncate_utf8("abcdef", 3), "abc")

    def test_truncate_utf8_bound_zero(self) -> None:
        self.assertEqual(truncate_utf8("abc", 0), "")

    def test_truncate_utf8_multibyte_continuation_step_back(self) -> None:
        # "héllo": 'h' (1 byte), 'é' (2 bytes), 'l' (1 byte), 'l' (1 byte), 'o' (1 byte)
        self.assertEqual(truncate_utf8("héllo", 2), "h")
        self.assertEqual(truncate_utf8("héllo", 3), "hé")

    def test_truncate_utf8_japanese_boundary(self) -> None:
        # "日本語": each character is 3 bytes (9 bytes total)
        self.assertEqual(truncate_utf8("日本語", 4), "日")
        self.assertEqual(truncate_utf8("日本語", 9), "日本語")


if __name__ == "__main__":
    unittest.main()
