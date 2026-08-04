from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_backup.infrastructure.keys import (
    build_key,
    list_prefix,
    local_object_name,
    normalize_prefix,
    parse_backup_key,
)


class TestInfrastructureKeys(unittest.TestCase):
    def test_normalize_prefix(self) -> None:
        self.assertEqual(normalize_prefix(""), "")
        self.assertEqual(normalize_prefix("/"), "")
        self.assertEqual(normalize_prefix("///"), "")
        self.assertEqual(normalize_prefix("p"), "p")
        self.assertEqual(normalize_prefix("/nested/prefix/"), "nested/prefix")
        self.assertEqual(normalize_prefix("//a//b//"), "a//b")
        self.assertEqual(normalize_prefix("a/b"), "a/b")

    def test_build_key(self) -> None:
        self.assertEqual(build_key("", 42), "backup-42.json")
        self.assertEqual(build_key("", 0), "backup-0.json")
        self.assertEqual(build_key("nested/prefix", 42), "nested/prefix/backup-42.json")
        self.assertEqual(build_key("p", 1_700_000_000), "p/backup-1700000000.json")

    def test_parse_backup_key_valid(self) -> None:
        self.assertEqual(parse_backup_key("", "backup-42.json"), 42)
        self.assertEqual(parse_backup_key("", "backup-0.json"), 0)
        self.assertEqual(parse_backup_key("nested/prefix", "nested/prefix/backup-42.json"), 42)
        self.assertEqual(parse_backup_key("a", "a/backup-1.json"), 1)
        self.assertEqual(
            parse_backup_key("p", "p/backup-9007199254740993.json"),
            9007199254740993,
        )

    def test_parse_backup_key_round_trip(self) -> None:
        prefixes = ("", "p", "nested/prefix")
        timestamps = (0, 1, 42, 1_700_000_000)
        for p in prefixes:
            for n in timestamps:
                key = build_key(p, n)
                self.assertEqual(parse_backup_key(p, key), n)

    def test_parse_backup_key_rejections(self) -> None:
        self.assertIsNone(parse_backup_key("nested/prefix", "nested/prefix/not-a-backup.json"))
        self.assertIsNone(parse_backup_key("nested/prefix", "backup-42.json"))
        self.assertIsNone(parse_backup_key("a", "ab/backup-1.json"))
        self.assertIsNone(parse_backup_key("", "backup-42.txt"))
        self.assertIsNone(parse_backup_key("", "snapshot-42.json"))

    def test_parse_backup_key_digit_strictness(self) -> None:
        self.assertIsNone(parse_backup_key("", "backup-.json"))
        self.assertIsNone(parse_backup_key("", "backup-+5.json"))
        self.assertIsNone(parse_backup_key("", "backup- 5.json"))
        self.assertIsNone(parse_backup_key("", "backup-5_0.json"))
        self.assertIsNone(parse_backup_key("", "backup--1.json"))
        self.assertIsNone(parse_backup_key("", "backup-٥.json"))
        self.assertIsNone(parse_backup_key("", "backup-4.5.json"))
        self.assertIsNone(parse_backup_key("", ""))

    def test_local_object_name(self) -> None:
        self.assertEqual(local_object_name("backup", 42), "backup-42.json")
        self.assertEqual(local_object_name("snap", 0), "snap-0.json")

    def test_list_prefix(self) -> None:
        self.assertIsNone(list_prefix(""))
        self.assertEqual(list_prefix("p"), "p/")
        self.assertEqual(list_prefix("nested/prefix"), "nested/prefix/")


if __name__ == "__main__":
    unittest.main()
