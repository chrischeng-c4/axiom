#!/usr/bin/env python3
"""Deterministic tests for the Lumen issue-intake classifier and workflow."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/meta/lumen_issue_intake_labels.py"
WORKFLOW_PATH = ROOT / ".github/workflows/lumen-issue-intake-labels.yml"

SPEC = importlib.util.spec_from_file_location("lumen_issue_intake_labels", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
CLASSIFIER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CLASSIFIER)


def diagnostics(project: str = "lumen") -> str:
    return "\n".join(
        [
            "## Diagnostics",
            f"- {project} version: 0.4.31",
            "- target: aarch64-apple-darwin",
            "- git sha: abc1234",
            "- built at: 1788171566",
            "- os/arch: macos/aarch64",
        ]
    )


class ClassifierTests(unittest.TestCase):
    def test_complete_lumen_diagnostics_gets_canonical_labels(self) -> None:
        body = f"failure details\n\n---\n{diagnostics()}\n"
        self.assertEqual(
            CLASSIFIER.labels_for_body(body), ["app:lumen", "type:report"]
        )

    def test_diagnostics_only_body_is_supported(self) -> None:
        self.assertEqual(
            CLASSIFIER.labels_for_body(diagnostics()),
            ["app:lumen", "type:report"],
        )

    def test_optional_canonical_node_line_is_supported(self) -> None:
        body = f"{diagnostics()}\n- node: http://127.0.0.1:7373 → version=0.4.31 healthz=200"
        self.assertEqual(
            CLASSIFIER.labels_for_body(body),
            ["app:lumen", "type:report"],
        )

    def test_plain_quoted_and_fenced_mentions_do_not_label(self) -> None:
        samples = [
            "the text says lumen version: 0.4.31",
            "\n".join(f"> {line}" for line in diagnostics().splitlines()),
            f"```text\n{diagnostics()}\n```",
        ]
        for body in samples:
            with self.subTest(body=body):
                self.assertEqual(CLASSIFIER.labels_for_body(body), [])

    def test_other_project_or_incomplete_block_does_not_label(self) -> None:
        self.assertEqual(CLASSIFIER.labels_for_body(diagnostics("jet")), [])
        incomplete = diagnostics().replace("- built at: 1788171566\n", "")
        self.assertEqual(CLASSIFIER.labels_for_body(incomplete), [])

    def test_trailing_prose_or_interrupted_fields_do_not_label(self) -> None:
        samples = [
            f"{diagnostics()}\nthis prose is not a canonical diagnostics field",
            diagnostics().replace(
                "- git sha: abc1234",
                "```text\nignored\n```\n- git sha: abc1234",
            ),
            diagnostics().replace("- target: aarch64-apple-darwin", "- target:   "),
            f"{diagnostics()}\n- node:   ",
        ]
        for body in samples:
            with self.subTest(body=body):
                self.assertEqual(CLASSIFIER.labels_for_body(body), [])

    def test_later_complete_block_survives_an_earlier_decoy(self) -> None:
        body = f"## Diagnostics\n- lumen version: quoted prose\n\n{diagnostics()}"
        self.assertEqual(
            CLASSIFIER.labels_for_body(body), ["app:lumen", "type:report"]
        )


class WorkflowContractTests(unittest.TestCase):
    def test_workflow_is_opened_only_and_least_privilege(self) -> None:
        source = WORKFLOW_PATH.read_text(encoding="utf-8")
        self.assertIn("issues:\n    types: [opened]", source)
        self.assertNotIn("types: [edited]", source)
        self.assertNotIn("pull_request_target", source)
        self.assertIn("permissions: {}", source)
        self.assertIn("contents: read\n      issues: write", source)

    def test_workflow_uses_classifier_and_adds_without_replacing(self) -> None:
        source = WORKFLOW_PATH.read_text(encoding="utf-8")
        self.assertIn("python3 scripts/meta/lumen_issue_intake_labels.py", source)
        self.assertIn("/issues/${ISSUE_NUMBER}/labels", source)
        self.assertIn("--method POST", source)
        self.assertNotIn("--method PUT", source)
        uses = re.findall(r"uses:\s+[^@\s]+@([^\s#]+)", source)
        self.assertTrue(uses)
        self.assertTrue(all(re.fullmatch(r"[0-9a-f]{40}", pin) for pin in uses))


if __name__ == "__main__":
    unittest.main()
