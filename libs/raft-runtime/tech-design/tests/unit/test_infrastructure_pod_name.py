from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.infrastructure.pod_name import (
    ASCII_DIGITS,
    BadOrdinal,
    NamelessPod,
    PodIdentity,
    split_pod_name,
)


class TestInfrastructurePodName(unittest.TestCase):
    def test_ascii_digits_constant(self) -> None:
        self.assertEqual(ASCII_DIGITS, "0123456789")

    def test_split_pod_name_splits_at_last_hyphen(self) -> None:
        self.assertEqual(
            split_pod_name("lumen-raft-host-2"),
            PodIdentity("lumen-raft-host", 2),
        )

    def test_split_pod_name_parses_leading_zeros(self) -> None:
        self.assertEqual(
            split_pod_name("pod-007"),
            PodIdentity("pod", 7),
        )

    def test_split_pod_name_no_hyphen_returns_nameless(self) -> None:
        self.assertEqual(split_pod_name("pod"), NamelessPod("pod"))

    def test_split_pod_name_empty_prefix_returns_nameless(self) -> None:
        self.assertEqual(split_pod_name("-3"), NamelessPod("-3"))

    def test_split_pod_name_empty_suffix_returns_bad_ordinal(self) -> None:
        self.assertEqual(split_pod_name("pod-"), BadOrdinal("pod-", ""))

    def test_split_pod_name_non_numeric_suffix_returns_bad_ordinal(
        self,
    ) -> None:
        self.assertEqual(split_pod_name("pod-x"), BadOrdinal("pod-x", "x"))

    def test_split_pod_name_double_hyphen_uses_last_hyphen(self) -> None:
        self.assertEqual(
            split_pod_name("pod--1"),
            PodIdentity("pod-", 1),
        )

    def test_split_pod_name_arabic_indic_digit_rejected(self) -> None:
        self.assertEqual(
            split_pod_name("pod-٢"),
            BadOrdinal("pod-٢", "٢"),
        )

    def test_split_pod_name_space_in_ordinal_rejected(self) -> None:
        self.assertEqual(
            split_pod_name("pod- 3"),
            BadOrdinal("pod- 3", " 3"),
        )


if __name__ == "__main__":
    unittest.main()
