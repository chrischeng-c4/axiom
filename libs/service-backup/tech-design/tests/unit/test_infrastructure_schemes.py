from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_backup.infrastructure.schemes import (
    BuildFeatures,
    find_scheme,
    scheme_names,
    supported_schemes,
    topic_destination_section,
    unavailable_schemes,
)


class TestInfrastructureSchemes(unittest.TestCase):
    def test_scheme_names_table_order(self) -> None:
        f0 = BuildFeatures(s3=False)
        f1 = BuildFeatures(s3=True)
        expected = ("file://", "s3://", "gs://")
        self.assertEqual(scheme_names(f0), expected)
        self.assertEqual(scheme_names(f1), expected)

    def test_supported_schemes_sink_availability(self) -> None:
        f0 = BuildFeatures(s3=False)
        f1 = BuildFeatures(s3=True)

        s_f0 = supported_schemes(f0)
        self.assertTrue(s_f0[0].sink_available)
        self.assertFalse(s_f0[1].sink_available)
        self.assertTrue(s_f0[2].sink_available)

        s_f1 = supported_schemes(f1)
        self.assertTrue(s_f1[0].sink_available)
        self.assertTrue(s_f1[1].sink_available)
        self.assertTrue(s_f1[2].sink_available)

    def test_find_scheme(self) -> None:
        f0 = BuildFeatures(s3=False)
        entry = find_scheme("s3://", f0)
        self.assertIsNotNone(entry)
        assert entry is not None
        self.assertFalse(entry.sink_available)

        self.assertIsNone(find_scheme("s3", f0))
        self.assertIsNone(find_scheme("s3://x", f0))
        self.assertIsNone(find_scheme("FILE://", f0))

    def test_unavailable_schemes(self) -> None:
        f0 = BuildFeatures(s3=False)
        f1 = BuildFeatures(s3=True)
        self.assertEqual(unavailable_schemes(f0), ("s3://",))
        self.assertEqual(unavailable_schemes(f1), ())

    def test_topic_destination_section_contains_all_schemes(self) -> None:
        f0 = BuildFeatures(s3=False)
        f1 = BuildFeatures(s3=True)

        section_f0 = topic_destination_section(f0)
        section_f1 = topic_destination_section(f1)

        for s_name in scheme_names(f0):
            self.assertIn(s_name, section_f0)
            self.assertIn(s_name, section_f1)

        self.assertIn("(not linked in this build)", section_f0)
        self.assertNotIn("(not linked in this build)", section_f1)


if __name__ == "__main__":
    unittest.main()
