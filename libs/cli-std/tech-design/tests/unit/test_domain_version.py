from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.domain.version import (
    Action,
    Version,
    compare_versions,
    decide_action,
    next_command_after_check,
    parse_tag,
    parse_version,
    select_version,
)


class TestDomainVersion(unittest.TestCase):
    def test_parse_version_leading_v(self) -> None:
        self.assertIsNone(parse_version("v1.2.3"))

    def test_parse_version_padded_zero(self) -> None:
        self.assertIsNone(parse_version("01.2.3"))
        valid = parse_version("0.2.3")
        self.assertIsNotNone(valid)
        self.assertEqual(valid, Version(0, 2, 3, ""))

    def test_parse_version_invalid_shapes(self) -> None:
        self.assertIsNone(parse_version("1.2.3-"))
        self.assertIsNone(parse_version("1.2"))

    def test_compare_versions_prerelease_outranked(self) -> None:
        stable = Version(1, 0, 0, "")
        pre = Version(1, 0, 0, "rc1")
        self.assertEqual(compare_versions(stable, pre), 1)

    def test_compare_versions_numeric(self) -> None:
        v10 = Version(0, 10, 0, "")
        v9 = Version(0, 9, 0, "")
        self.assertEqual(compare_versions(v10, v9), 1)

    def test_version_no_lt_attribute(self) -> None:
        self.assertFalse(
            hasattr(Version, "__lt__")
            and Version.__lt__ is not object.__lt__
        )

    def test_select_version_unpinned_replay(self) -> None:
        tags = [
            "lumen@0.9.0",
            "lumen@1.0.0-rc1",
            "tape@2.0.0",
            "lumen@1.0.0",
            "lumen@0.10.0",
            "lumen@1.0.0",
            "not-a-tag",
            "lumen@v1.1.0",
        ]
        res_unpinned = select_version(tags, "lumen@", None)
        self.assertIsNotNone(res_unpinned)
        tag_str, v_obj = res_unpinned
        self.assertIs(tag_str, tags[5])
        self.assertEqual(v_obj, Version(1, 0, 0, ""))

        res_pin_exact = select_version(tags, "lumen@", "1.0.0")
        self.assertIsNotNone(res_pin_exact)
        tag_pin_str, _ = res_pin_exact
        self.assertIs(tag_pin_str, tags[3])

        res_pin_prefix = select_version(tags, "lumen@", "lumen@1.0.0")
        self.assertIsNotNone(res_pin_prefix)
        tag_prefix_str, _ = res_pin_prefix
        self.assertIs(tag_prefix_str, tags[3])

        res_pin_rc = select_version(tags, "lumen@", "1.0.0-rc1")
        self.assertEqual(res_pin_rc, ("lumen@1.0.0-rc1", Version(1, 0, 0, "rc1")))

        self.assertIsNone(parse_tag("tape@2.0.0", "lumen@"))
        self.assertIsNone(parse_tag("lumen@v1.1.0", "lumen@"))
        self.assertEqual(parse_tag("lumen@0.10.0", "lumen@"), Version(0, 10, 0, ""))

    def test_select_version_unmatched_pin(self) -> None:
        tags = ["lumen@0.9.0", "lumen@1.0.0"]
        self.assertIsNone(select_version(tags, "lumen@", "9.9.9"))

    def test_decide_action_force_and_equality(self) -> None:
        v = Version(1, 0, 0, "")
        self.assertEqual(decide_action(v, v, force=False), Action.UP_TO_DATE)
        self.assertEqual(decide_action(v, v, force=True), Action.INSTALL)

    def test_next_command_after_check_behavior(self) -> None:
        v1 = Version(1, 0, 0, "")
        v2 = Version(1, 1, 0, "")
        v0 = Version(0, 9, 0, "")
        self.assertEqual(next_command_after_check("lumen", v1, v1), "done")
        self.assertEqual(next_command_after_check("lumen", v1, v2), "lumen upgrade")
        self.assertEqual(next_command_after_check("lumen", v1, v0), "done")

    def test_parse_version_non_ascii_digits(self) -> None:
        self.assertIsNone(parse_version("١.٢.٣"))
        self.assertIsNone(parse_version("1.2.²"))


if __name__ == "__main__":
    unittest.main()
