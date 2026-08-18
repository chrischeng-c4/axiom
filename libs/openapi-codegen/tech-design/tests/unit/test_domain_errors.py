from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.domain.errors import (
    MissingPolicyKey,
    OutputPathEscape,
    PolicyLanguageMismatch,
    TargetLanguageMismatch,
    UnknownTargetProfile,
)
from openapi_codegen.domain.lang import Lang


class TestDomainErrors(unittest.TestCase):
    def test_unknown_target_profile_message(self) -> None:
        err = UnknownTargetProfile("custom-1.0")
        expected_msg = (
            "unknown target profile 'custom-1.0'; expected one of: "
            "python-3.11, python-3.12, python-3.13, python-3.14, "
            "typescript-5.0, rust-2021, rust-2024"
        )
        self.assertEqual(err.message(), expected_msg)

    def test_unknown_target_profile_fields(self) -> None:
        err = UnknownTargetProfile("bad-profile")
        self.assertEqual(err.value, "bad-profile")

    def test_policy_language_mismatch_message(self) -> None:
        err = PolicyLanguageMismatch("targets.python", Lang.PY, "rust-2021")
        self.assertEqual(
            err.message(), "targets.python must select python, got rust-2021"
        )

    def test_policy_language_mismatch_fields(self) -> None:
        err = PolicyLanguageMismatch("targets.python", Lang.PY, "rust-2021")
        self.assertEqual(err.key, "targets.python")
        self.assertEqual(err.expected, Lang.PY)
        self.assertEqual(err.got, "rust-2021")

    def test_target_language_mismatch_message(self) -> None:
        err = TargetLanguageMismatch("rust-2021", Lang.RUST, Lang.PY)
        self.assertEqual(
            err.message(),
            "target profile rust-2021 is for rust, not requested language python",
        )

    def test_target_language_mismatch_fields(self) -> None:
        err = TargetLanguageMismatch("rust-2021", Lang.RUST, Lang.PY)
        self.assertEqual(err.profile_id, "rust-2021")
        self.assertEqual(err.profile_lang, Lang.RUST)
        self.assertEqual(err.requested, Lang.PY)

    def test_missing_policy_key_message(self) -> None:
        err = MissingPolicyKey("python")
        self.assertEqual(err.message(), "target policy is missing required key python")

    def test_missing_policy_key_fields(self) -> None:
        err = MissingPolicyKey("typescript")
        self.assertEqual(err.key, "typescript")

    def test_output_path_escape_message_absolute(self) -> None:
        err = OutputPathEscape("/etc/passwd", "absolute")
        self.assertEqual(
            err.message(),
            "generated file path must stay under output directory: '/etc/passwd' (absolute)",
        )

    def test_output_path_escape_message_parent(self) -> None:
        err = OutputPathEscape("../out.ts", "parent-component")
        self.assertEqual(
            err.message(),
            "generated file path must stay under output directory: '../out.ts' (parent-component)",
        )

    def test_dataclasses_equality(self) -> None:
        self.assertEqual(
            UnknownTargetProfile("foo"), UnknownTargetProfile("foo")
        )
        self.assertNotEqual(
            UnknownTargetProfile("foo"), UnknownTargetProfile("bar")
        )
        self.assertEqual(MissingPolicyKey("key"), MissingPolicyKey("key"))

    def test_dataclasses_hashable(self) -> None:
        errors_set = {
            MissingPolicyKey("key1"),
            MissingPolicyKey("key2"),
            UnknownTargetProfile("prof"),
        }
        self.assertEqual(len(errors_set), 3)
        self.assertIn(MissingPolicyKey("key1"), errors_set)


if __name__ == "__main__":
    unittest.main()
