from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.domain.read_consistency import (
    ANY,
    LEADER,
    Bounded,
)
from raft_runtime.infrastructure.headers import (
    BOUNDED_PREFIX,
    BOUNDED_SUFFIX,
    parse_read_consistency,
)


class TestInfrastructureHeaders(unittest.TestCase):
    def test_constants_definitions(self) -> None:
        self.assertEqual(BOUNDED_PREFIX, "bounded(")
        self.assertEqual(BOUNDED_SUFFIX, ")")

    def test_parse_read_consistency_none_and_empty_fallback_to_leader(
        self,
    ) -> None:
        self.assertEqual(parse_read_consistency(None), LEADER)
        self.assertEqual(parse_read_consistency(""), LEADER)
        self.assertEqual(parse_read_consistency("   "), LEADER)

    def test_parse_read_consistency_leader_case_insensitive(self) -> None:
        self.assertEqual(parse_read_consistency("  LEADER "), LEADER)

    def test_parse_read_consistency_any_case_insensitive(self) -> None:
        self.assertEqual(parse_read_consistency("any"), ANY)
        self.assertEqual(parse_read_consistency("ANY"), ANY)

    def test_parse_read_consistency_bounded_valid_values(self) -> None:
        self.assertEqual(parse_read_consistency("bounded(250)"), Bounded(250))
        self.assertEqual(parse_read_consistency("BOUNDED(0)"), Bounded(0))

    def test_parse_read_consistency_bounded_invalid_fallbacks_to_leader(
        self,
    ) -> None:
        self.assertEqual(parse_read_consistency("bounded()"), LEADER)
        self.assertEqual(parse_read_consistency("bounded(-1)"), LEADER)
        self.assertEqual(parse_read_consistency("bounded(abc)"), LEADER)
        self.assertEqual(parse_read_consistency("bounded(250"), LEADER)
        self.assertEqual(parse_read_consistency("bounded 250"), LEADER)

    def test_parse_read_consistency_unrecognized_mode_fallbacks_to_leader(
        self,
    ) -> None:
        self.assertEqual(parse_read_consistency("strong"), LEADER)

    def test_parse_read_consistency_non_ascii_digits_fallbacks_to_leader(
        self,
    ) -> None:
        self.assertEqual(parse_read_consistency("bounded(٢)"), LEADER)

    def test_parse_read_consistency_positive_direction_bounded(
        self,
    ) -> None:
        result = parse_read_consistency("bounded(250)")
        self.assertIsNot(result, LEADER)
        self.assertEqual(result, Bounded(250))


if __name__ == "__main__":
    unittest.main()
