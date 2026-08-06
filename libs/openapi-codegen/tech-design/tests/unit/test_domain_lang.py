from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.domain.lang import Lang, lang_from_id


class TestDomainLang(unittest.TestCase):
    def test_enum_members_exist(self) -> None:
        self.assertEqual(len(Lang), 3)
        self.assertIn(Lang.TS, Lang)
        self.assertIn(Lang.PY, Lang)
        self.assertIn(Lang.RUST, Lang)

    def test_enum_id_property(self) -> None:
        self.assertEqual(Lang.TS.id, "typescript")
        self.assertEqual(Lang.PY.id, "python")
        self.assertEqual(Lang.RUST.id, "rust")

    def test_lang_from_id_valid(self) -> None:
        self.assertIs(lang_from_id("typescript"), Lang.TS)
        self.assertIs(lang_from_id("python"), Lang.PY)
        self.assertIs(lang_from_id("rust"), Lang.RUST)

    def test_lang_from_id_case_sensitivity(self) -> None:
        # Paired test: exact match vs case-folding
        self.assertIsNone(lang_from_id("Python"))
        self.assertIsNone(lang_from_id("TypeScript"))
        self.assertIsNone(lang_from_id("RUST"))

    def test_lang_from_id_invalid(self) -> None:
        self.assertIsNone(lang_from_id("go"))
        self.assertIsNone(lang_from_id(""))
        self.assertIsNone(lang_from_id("python3"))

    def test_lang_from_id_whitespace(self) -> None:
        self.assertIsNone(lang_from_id(" python "))
        self.assertIsNone(lang_from_id("typescript\n"))

    def test_lang_enum_values(self) -> None:
        self.assertEqual(Lang.TS.value, "typescript")
        self.assertEqual(Lang.PY.value, "python")
        self.assertEqual(Lang.RUST.value, "rust")

    def test_lang_identity_comparison(self) -> None:
        self.assertEqual(Lang.PY, Lang.PY)
        self.assertNotEqual(Lang.PY, Lang.TS)
        self.assertNotEqual(Lang.PY, Lang.RUST)

    def test_lang_hashable(self) -> None:
        lang_set = {Lang.TS, Lang.PY, Lang.RUST}
        self.assertEqual(len(lang_set), 3)
        self.assertIn(Lang.PY, lang_set)

    def test_lang_from_id_no_substring(self) -> None:
        self.assertIsNone(lang_from_id("py"))
        self.assertIsNone(lang_from_id("rustc"))
        self.assertIsNone(lang_from_id("ts"))

    def test_lang_id_returns_str(self) -> None:
        self.assertIsInstance(Lang.TS.id, str)
        self.assertIsInstance(Lang.PY.id, str)
        self.assertIsInstance(Lang.RUST.id, str)

    def test_lang_enum_iteration_order(self) -> None:
        members = list(Lang)
        self.assertEqual(members, [Lang.TS, Lang.PY, Lang.RUST])


if __name__ == "__main__":
    unittest.main()
