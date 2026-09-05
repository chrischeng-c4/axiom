#!/usr/bin/env python3
"""Tests for the fleet renderer: the tree matches its templates, and every
divergence the renderer is meant to catch is actually caught."""

from __future__ import annotations

import importlib.util
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("render_fleet.py")
SPEC = importlib.util.spec_from_file_location("render_fleet", SCRIPT)
assert SPEC and SPEC.loader
render_fleet = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = render_fleet
SPEC.loader.exec_module(render_fleet)

REPO = Path(__file__).resolve().parents[2]
OWNED_DIRS = (".claude/agents", ".codex/agents", "scripts/agents/templates")


def copy_tree() -> Path:
    tmp = Path(tempfile.mkdtemp(prefix="render-fleet-"))
    for rel in OWNED_DIRS:
        shutil.copytree(REPO / rel, tmp / rel)
    return tmp


def run_cli(root: Path, *flags: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(SCRIPT), "--root", str(root), *flags],
        text=True,
        capture_output=True,
        check=False,
    )


class RenderFleetTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = copy_tree()
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)

    # -- the checked-in tree -------------------------------------------------

    def test_repo_tree_matches_templates(self) -> None:
        self.assertEqual(render_fleet.check(REPO), [])

    def test_cli_check_passes_on_repo(self) -> None:
        result = run_cli(REPO, "--check")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("fleet matches", result.stdout)

    def test_rendered_population(self) -> None:
        rendered = render_fleet.rendered_agents(REPO)
        self.assertEqual(len(rendered), 133)
        self.assertNotIn("aw-dev", rendered)
        self.assertNotIn("aw-e2e-dev", rendered)
        self.assertIn("aw-pm", rendered)
        self.assertIn("lumen-e2e-dev", rendered)
        self.assertIn("lumen-pm", rendered)
        self.assertIn("build-stamp-dev", rendered)
        self.assertIn("build-stamp-pm", rendered)

    def test_singletons_are_projected_not_rewritten(self) -> None:
        expected = render_fleet.expected_files(REPO)
        agents = render_fleet.claude_agents_dir(REPO)
        codex = render_fleet.codex_agents_dir(REPO)
        for singleton in ("aw-dev", "gke-operator", "agy-operator", "cto",
                          "project-manager", "tech-design"):
            self.assertNotIn(agents / f"{singleton}.md", expected)
            self.assertIn(codex / f"{singleton}.toml", expected)

    def test_write_on_a_copy_changes_nothing(self) -> None:
        self.assertEqual(render_fleet.write(self.tmp), [])
        for rel in OWNED_DIRS:
            for path in sorted((REPO / rel).rglob("*")):
                if path.is_file():
                    twin = self.tmp / path.relative_to(REPO)
                    self.assertEqual(twin.read_bytes(), path.read_bytes(), twin)

    # -- projection shape ----------------------------------------------------

    def test_projection_escapes_description_and_keeps_body_raw(self) -> None:
        markdown = (
            "---\nname: demo-dev\ndescription: Demo — with \"quotes\"\n"
            "model: sonnet\neffort: medium\n---\n\nYou are **demo-dev** — body.\n"
        )
        toml = render_fleet.toml_projection(markdown, "demo-dev.md")
        self.assertIn('description = "Demo \\u2014 with \\"quotes\\""', toml)
        self.assertIn('model_reasoning_effort = "medium"', toml)
        self.assertIn('nickname_candidates = ["demo-dev", "demo_dev"]', toml)
        self.assertTrue(toml.endswith("'''\nYou are **demo-dev** — body.\n'''\n"))

    def test_projection_refuses_missing_effort(self) -> None:
        markdown = "---\nname: demo-dev\ndescription: Demo\n---\n\nbody\n"
        with self.assertRaisesRegex(render_fleet.RenderError, "no effort"):
            render_fleet.toml_projection(markdown, "demo-dev.md")

    # -- negative controls: each divergence class is caught ------------------

    def test_template_mutation_is_caught(self) -> None:
        template = self.tmp / "scripts/agents/templates/app/dev.md"
        template.write_text(
            template.read_text(encoding="utf-8").replace("## Goal", "## Goal!", 1),
            encoding="utf-8",
        )
        findings = render_fleet.check(self.tmp)
        self.assertIn("differs: .claude/agents/lumen-dev.md", findings)
        self.assertIn("differs: .codex/agents/lumen-dev.toml", findings)
        self.assertNotIn("differs: .claude/agents/build-stamp-dev.md", findings)
        result = run_cli(self.tmp, "--check")
        self.assertEqual(result.returncode, 1, result.stdout)

    def test_hand_edited_rendered_file_is_caught(self) -> None:
        path = self.tmp / ".claude/agents/tape-dev.md"
        path.write_text(path.read_text(encoding="utf-8") + "\n- extra\n",
                        encoding="utf-8")
        self.assertEqual(render_fleet.check(self.tmp),
                         ["differs: .claude/agents/tape-dev.md"])

    def test_stale_projection_is_caught_and_removed(self) -> None:
        (self.tmp / ".codex/agents/ghost-dev.toml").write_text(
            'name = "ghost-dev"\nmodel_reasoning_effort = "max"\n', encoding="utf-8"
        )
        self.assertEqual(render_fleet.check(self.tmp),
                         ["stray projection: .codex/agents/ghost-dev.toml"])
        self.assertEqual(render_fleet.write(self.tmp),
                         ["removed: .codex/agents/ghost-dev.toml"])
        self.assertEqual(render_fleet.check(self.tmp), [])

    def test_missing_projection_is_caught_and_written(self) -> None:
        (self.tmp / ".codex/agents/cap-e2e-dev.toml").unlink()
        self.assertEqual(render_fleet.check(self.tmp),
                         ["missing: .codex/agents/cap-e2e-dev.toml"])
        self.assertEqual(render_fleet.write(self.tmp),
                         ["wrote: .codex/agents/cap-e2e-dev.toml"])
        self.assertEqual(render_fleet.check(self.tmp), [])

    def test_singleton_edit_reprojects_only_its_toml(self) -> None:
        path = self.tmp / ".claude/agents/gke-operator.md"
        path.write_text(path.read_text(encoding="utf-8") + "\n- extra\n",
                        encoding="utf-8")
        self.assertEqual(render_fleet.check(self.tmp),
                         ["differs: .codex/agents/gke-operator.toml"])
        self.assertEqual(render_fleet.write(self.tmp),
                         ["wrote: .codex/agents/gke-operator.toml"])
        self.assertTrue(path.read_text(encoding="utf-8").endswith("- extra\n"))


if __name__ == "__main__":
    unittest.main()
