from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.infrastructure.numbers import (
    parse_ascii_unsigned,
    parse_positive,
)


class TestInfrastructureNumbers(unittest.TestCase):
    def test_parse_ascii_unsigned_none_and_empty(self) -> None:
        self.assertIsNone(parse_ascii_unsigned(None))
        self.assertIsNone(parse_ascii_unsigned(""))

    def test_parse_ascii_unsigned_whitespace(self) -> None:
        self.assertIsNone(parse_ascii_unsigned(" 5"))
        self.assertIsNone(parse_ascii_unsigned("5 "))

    def test_parse_ascii_unsigned_underscore(self) -> None:
        self.assertIsNone(parse_ascii_unsigned("5_0"))

    def test_parse_ascii_unsigned_negative(self) -> None:
        self.assertIsNone(parse_ascii_unsigned("-1"))

    def test_parse_ascii_unsigned_non_ascii_digits(self) -> None:
        self.assertIsNone(parse_ascii_unsigned("٥"))
        self.assertIsNone(parse_ascii_unsigned("²"))

    def test_parse_ascii_unsigned_leading_plus(self) -> None:
        self.assertEqual(parse_ascii_unsigned("+5"), 5)

    def test_parse_ascii_unsigned_double_plus(self) -> None:
        self.assertIsNone(parse_ascii_unsigned("++5"))

    def test_parse_ascii_unsigned_zero(self) -> None:
        self.assertEqual(parse_ascii_unsigned("0"), 0)

    def test_parse_ascii_unsigned_leading_zeros(self) -> None:
        self.assertEqual(parse_ascii_unsigned("007"), 7)

    def test_parse_positive_zero_vs_positive(self) -> None:
        self.assertIsNone(parse_positive("0"))
        self.assertEqual(parse_positive("+7"), 7)
        self.assertIsNone(parse_positive(None))
