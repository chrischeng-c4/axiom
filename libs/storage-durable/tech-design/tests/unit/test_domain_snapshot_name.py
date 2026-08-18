from __future__ import annotations

from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from storage_durable.domain.snapshot_name import (
    SnapshotName,
    order_by_sequence,
    parse_name,
    render_name,
)

class TestDomainSnapshotName(unittest.TestCase):
    def test_parse_name_foreign_rejections(self) -> None:
        foreign_matrix = [
            "snap-x.bin",
            "snap--1.bin",
            "snap-+7.bin",
            "snap- 7.bin",
            "snap-007.bin",
            "snap-7 .bin",
            "other-7.bin",
            "snap-7.dat",
            "snap7.bin",
            "snap-.bin",
            "snap-².bin",
        ]
        for name in foreign_matrix:
            with self.subTest(name=name):
                self.assertIsNone(
                    parse_name(name, prefix="snap", extension="bin"),
                    f"Expected None for {name}",
                )

    def test_parse_name_valid_sequences(self) -> None:
        valid_matrix = [
            ("snap-0.bin", 0),
            ("snap-7.bin", 7),
            ("snap-10.bin", 10),
        ]
        for name, expected_seq in valid_matrix:
            with self.subTest(name=name):
                self.assertEqual(
                    parse_name(name, prefix="snap", extension="bin"),
                    expected_seq,
                )

    def test_order_by_sequence_numeric_sort(self) -> None:
        entries = [(10, "snap-10.bin"), (9, "snap-9.bin")]
        ordered = order_by_sequence(entries)
        self.assertEqual(ordered, ((9, "snap-9.bin"), (10, "snap-10.bin")))

    def test_render_name(self) -> None:
        rendered = render_name("snap", 7, "bin")
        self.assertEqual(rendered, "snap-7.bin")

    def test_parse_name_leading_zero_rejection(self) -> None:
        self.assertIsNone(parse_name("snap-007.bin", prefix="snap", extension="bin"))
        self.assertIsNone(parse_name("snap-00.bin", prefix="snap", extension="bin"))

    def test_parse_name_unicode_digit_rejection(self) -> None:
        self.assertIsNone(parse_name("snap-².bin", prefix="snap", extension="bin"))

    def test_snapshot_name_dataclass(self) -> None:
        snap = SnapshotName("snap", 42, "bin")
        self.assertEqual(snap.prefix, "snap")
        self.assertEqual(snap.seq, 42)
        self.assertEqual(snap.extension, "bin")

if __name__ == "__main__":
    unittest.main()
