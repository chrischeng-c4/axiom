from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.application.artifact_output import (
    FileOutput,
    StdoutOutput,
    has_extension,
    join_path,
    plan_output,
)


class TestApplicationArtifactOutput(unittest.TestCase):
    def test_plan_output_stdout(self) -> None:
        self.assertEqual(
            plan_output(None, "artifact.yaml", "body"), StdoutOutput("body")
        )

    def test_plan_output_directory_path(self) -> None:
        self.assertEqual(
            plan_output("out/render", "artifact.yaml", "body"),
            FileOutput("out/render/artifact.yaml", "body"),
        )

    def test_plan_output_explicit_file_path(self) -> None:
        self.assertEqual(
            plan_output("out/render.yaml", "artifact.yaml", "body"),
            FileOutput("out/render.yaml", "body"),
        )

    def test_has_extension_gitignore_leading_dot(self) -> None:
        self.assertFalse(has_extension(".gitignore"))
        self.assertTrue(has_extension(".gitignore.bak"))

    def test_has_extension_various_paths(self) -> None:
        self.assertTrue(has_extension("archive.tar.gz"))
        self.assertFalse(has_extension("out/"))
        self.assertFalse(has_extension(".."))
        self.assertFalse(has_extension("dir.v2/file"))

    def test_join_path_slash_normalization(self) -> None:
        self.assertEqual(join_path("out/", "a.yaml"), "out/a.yaml")
        self.assertEqual(join_path("out", "a.yaml"), "out/a.yaml")
        self.assertEqual(join_path("", "a.yaml"), "a.yaml")

    def test_has_extension_dot_segments(self) -> None:
        self.assertFalse(has_extension("."))
        self.assertFalse(has_extension(""))
        self.assertFalse(has_extension("a/b/c"))

    def test_join_path_empty_directory(self) -> None:
        self.assertEqual(join_path("", "file.txt"), "file.txt")

    def test_plan_output_file_output_dataclass(self) -> None:
        fo = FileOutput("p.txt", "b")
        self.assertEqual(fo.path, "p.txt")
        self.assertEqual(fo.body, "b")

    def test_plan_output_stdout_dataclass(self) -> None:
        so = StdoutOutput("b")
        self.assertEqual(so.body, "b")


if __name__ == "__main__":
    unittest.main()
