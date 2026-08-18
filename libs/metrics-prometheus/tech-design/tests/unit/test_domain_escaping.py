from __future__ import annotations

from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from metrics_prometheus.domain.escaping import escape_label_value


class TestDomainEscaping(unittest.TestCase):
    def test_escape_quote_alone(self) -> None:
        self.assertEqual(escape_label_value('a"b'), 'a\\"b')

    def test_escape_backslash_alone(self) -> None:
        self.assertEqual(escape_label_value("a\\b"), "a\\\\b")

    def test_escape_newline_alone(self) -> None:
        self.assertEqual(escape_label_value("a\nb"), "a\\nb")

    def test_escape_backslash_followed_by_quote(self) -> None:
        self.assertEqual(escape_label_value('\\"'), '\\\\\\"')

    def test_escape_empty_quotes(self) -> None:
        self.assertEqual(escape_label_value('""'), '\\"\\"')

    def test_escape_multiple_occurrences(self) -> None:
        self.assertEqual(escape_label_value('a"b"c\n\n\\d'), 'a\\"b\\"c\\n\\n\\\\d')

    def test_ordinary_string_unchanged(self) -> None:
        self.assertEqual(escape_label_value("plain_text"), "plain_text")


if __name__ == "__main__":
    unittest.main()
