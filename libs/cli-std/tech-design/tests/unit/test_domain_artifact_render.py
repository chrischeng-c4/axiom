from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.domain.artifact_render import (
    ensure_trailing_newline,
    release_tag,
    replace_kubernetes_namespace,
    strip_source_ownership_markers,
)


class TestDomainArtifactRender(unittest.TestCase):
    def test_release_tag_formatting(self) -> None:
        self.assertEqual(release_tag("tape", "tape@1.2.3", "0.0.0"), "tape@1.2.3")
        self.assertEqual(release_tag("tape", "  ", "0.4.7"), "tape@0.4.7")
        self.assertEqual(release_tag("tape", None, "0.4.7"), "tape@0.4.7")

    def test_strip_source_ownership_markers(self) -> None:
        input_text = "  # SPEC-MANAGED: x\nb\n# CODEGEN-BEGIN extra\n"
        stripped = strip_source_ownership_markers(input_text)
        self.assertNotIn("SPEC-MANAGED", stripped)
        self.assertIn("# CODEGEN-BEGIN extra", stripped)
        self.assertIn("b\n", stripped)

    def test_strip_source_ownership_markers_empty_and_single_line(self) -> None:
        self.assertEqual(strip_source_ownership_markers("a"), "a\n")
        self.assertEqual(strip_source_ownership_markers(""), "")

    def test_replace_kubernetes_namespace(self) -> None:
        input_yaml = (
            "name: default\n"
            "namespace: default\n"
            "image: default/app:latest\n"
        )
        res = replace_kubernetes_namespace(input_yaml, "default", "prod")
        self.assertIn("name: prod", res)
        self.assertIn("namespace: prod", res)
        self.assertIn("image: default/app:latest", res)

    def test_ensure_trailing_newline(self) -> None:
        self.assertEqual(ensure_trailing_newline(""), "\n")
        self.assertEqual(ensure_trailing_newline("a\n"), "a\n")

    def test_release_tag_existing_prefix(self) -> None:
        self.assertEqual(release_tag("lumen", "1.0.0", "0.0.0"), "lumen@1.0.0")
        self.assertEqual(release_tag("lumen", "lumen@1.0.0", "0.0.0"), "lumen@1.0.0")

    def test_replace_kubernetes_namespace_multiple_occurrences(self) -> None:
        text = "name: old\nnamespace: old\nother: old\n"
        out = replace_kubernetes_namespace(text, "old", "new")
        self.assertEqual(out, "name: new\nnamespace: new\nother: old\n")

    def test_ensure_trailing_newline_already_present(self) -> None:
        self.assertEqual(ensure_trailing_newline("hello\n"), "hello\n")
        self.assertEqual(ensure_trailing_newline("hello"), "hello\n")


if __name__ == "__main__":
    unittest.main()
