from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.domain.errors import OutputPathEscape
from openapi_codegen.domain.output_path import (
    check_output_path,
    joined_output_path,
)


class TestDomainOutputPath(unittest.TestCase):
    def test_check_output_path_simple(self) -> None:
        self.assertEqual(check_output_path("types.ts"), ("types.ts",))

    def test_check_output_path_nested(self) -> None:
        self.assertEqual(check_output_path("a/b/c.ts"), ("a", "b", "c.ts"))

    def test_check_output_path_current_dir(self) -> None:
        self.assertEqual(check_output_path("./types.ts"), ("types.ts",))

    def test_check_output_path_empty_segments(self) -> None:
        self.assertEqual(check_output_path("a//b.ts"), ("a", "b.ts"))

    def test_check_output_path_empty_string(self) -> None:
        # Tell 16: empty string is accepted and returns ()
        self.assertEqual(check_output_path(""), ())

    def test_check_output_path_absolute(self) -> None:
        res = check_output_path("/etc/passwd")
        self.assertIsInstance(res, OutputPathEscape)
        assert isinstance(res, OutputPathEscape)
        self.assertEqual(res.reason, "absolute")

    def test_check_output_path_parent_component(self) -> None:
        res = check_output_path("../out.ts")
        self.assertIsInstance(res, OutputPathEscape)
        assert isinstance(res, OutputPathEscape)
        self.assertEqual(res.reason, "parent-component")

    def test_check_output_path_parent_component_traversal(self) -> None:
        # Tell 14: check_output_path("a/../b.ts") is rejected per-segment
        res = check_output_path("a/../b.ts")
        self.assertIsInstance(res, OutputPathEscape)
        assert isinstance(res, OutputPathEscape)
        self.assertEqual(res.rel_path, "a/../b.ts")
        self.assertEqual(res.reason, "parent-component")

    def test_check_output_path_hidden_file(self) -> None:
        # Tell 15: check_output_path("..hidden.ts") == ("..hidden.ts",)
        self.assertEqual(check_output_path("..hidden.ts"), ("..hidden.ts",))

    def test_joined_output_path_valid(self) -> None:
        res = joined_output_path("/out/", "a//b.ts")
        self.assertEqual(res, "/out/a/b.ts")

    def test_joined_output_path_empty(self) -> None:
        res = joined_output_path("/out/", "")
        self.assertEqual(res, "/out")

    def test_joined_output_path_escape(self) -> None:
        res = joined_output_path("/out", "../out.ts")
        self.assertIsInstance(res, OutputPathEscape)
        assert isinstance(res, OutputPathEscape)
        self.assertEqual(res.rel_path, "../out.ts")


if __name__ == "__main__":
    unittest.main()
