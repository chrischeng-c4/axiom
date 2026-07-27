"""Focused contracts for TD migration batch planning and publication."""

from __future__ import annotations

import importlib.util
import re
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "migration_reconciliation.py"
SPEC = importlib.util.spec_from_file_location("migration_reconciliation", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
migration = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(migration)


def _published_issues(manifest: dict) -> list[dict]:
    issue_id_by_batch = {
        batch["id"]: str(3000 + index)
        for index, batch in enumerate(manifest["batches"])
    }
    previous_by_family: dict[str, str] = {}
    issues = []
    for batch in manifest["batches"]:
        dependencies = [migration.PUBLICATION_OWNER_WI]
        previous = previous_by_family.get(batch["family"])
        if previous:
            dependencies.append(issue_id_by_batch[previous])
        issue_id = issue_id_by_batch[batch["id"]]
        issues.append(
            {
                "github_id": int(issue_id),
                "state": "open",
                "dependencies": dependencies,
                "body": (
                    f"Migration batch `{batch['id']}`\n"
                    f"Gate: `{batch['checker']}`\n"
                    "<!-- aw:planning-transaction:sha256:test -->"
                ),
            }
        )
        previous_by_family[batch["family"]] = batch["id"]
    batch_ids = set(issue_id_by_batch.values())
    issues.append(
        {
            "github_id": 4000,
            "state": "open",
            "dependencies": sorted(batch_ids),
            "body": (
                f"Migration terminal `{migration.TERMINAL_GUIDANCE_ID}`\n"
                "<!-- aw:planning-transaction:sha256:test -->"
            ),
        }
    )
    issues.append(
        {
            "github_id": 4001,
            "state": "open",
            "dependencies": ["4000"],
            "body": (
                f"Migration terminal `{migration.TERMINAL_PROOF_ID}`\n"
                "Reuse existing #2688.\n"
                "<!-- aw:planning-transaction:sha256:test -->"
            ),
        }
    )
    return issues


class MigrationReconciliationTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = migration._load_manifest()

    def test_publication_requirements_are_bounded_and_dependency_ordered(self) -> None:
        requirements = migration._publication_requirements(self.manifest)

        batches = [item for item in requirements if item["kind"] == "batch"]
        terminals = [item for item in requirements if item["kind"] == "terminal"]
        self.assertEqual(len(batches), 42)
        self.assertEqual(len(terminals), 2)
        self.assertEqual([item["id"] for item in batches], [f"R{i}" for i in range(6, 48)])
        self.assertTrue(
            all(
                1
                <= next(
                    batch["artifact_count"]
                    for batch in self.manifest["batches"]
                    if batch["id"] == item["batch_id"]
                )
                <= migration.MAX_BATCH_ARTIFACTS
                for item in batches
            )
        )
        self.assertEqual(
            terminals[0]["depends_on"],
            [item["id"] for item in batches],
        )
        self.assertEqual(terminals[1]["depends_on"], [terminals[0]["id"]])

    def test_render_replaces_unbounded_r6_with_batches_and_terminals(self) -> None:
        foundation_requirements = "\n".join(
            f"- R{index}: foundation {index}" for index in range(1, 6)
        )
        foundation_inventory = "\n".join(
            f"| R{index} | `gate-{index}` | oracle {index} | - |"
            for index in range(1, 6)
        )
        body = (
            "## Requirements\n\n"
            f"{foundation_requirements}\n"
            "- R6: unbounded migration coordinator\n\n"
            "## Acceptance Criteria\n\n- all\n\n"
            "## Verification Inventory\n\n"
            "| Requirement | Gate | Oracle | Depends On |\n"
            "|---|---|---|---|\n"
            f"{foundation_inventory}\n"
            "| R6 | `old-gate` | old oracle | R5 |\n\n"
            "## Reference Context\n\n- retained\n"
        )

        rendered = migration._render_project_plan_body(self.manifest, body)

        self.assertNotIn("unbounded migration coordinator", rendered)
        self.assertEqual(rendered.count("- R"), 49)
        self.assertEqual(
            sum(bool(re.match(r"^\| R\d+ \|", line)) for line in rendered.splitlines()),
            49,
        )
        self.assertIn("Migration batch `evidence-core-readme-md-01`", rendered)
        self.assertIn(
            f"Migration terminal `{migration.TERMINAL_PROOF_ID}`",
            rendered,
        )

    def test_published_batches_requires_exact_reviewed_projection(self) -> None:
        result = migration._published_batches(
            self.manifest,
            _published_issues(self.manifest),
        )

        self.assertEqual(result["batch_count"], 42)
        self.assertEqual(result["terminal_change_count"], 2)
        self.assertEqual(result["published_change_count"], 44)
        self.assertEqual(result["unmatched"], 0)

    def test_batch_gate_rejects_pending_artifacts(self) -> None:
        pending_batch = next(
            batch
            for batch in self.manifest["batches"]
            if any(
                entry["status"] == "pending"
                for entry in self.manifest["markdown_td"]
                if entry["path"] in batch["artifact_paths"]
            )
        )

        with self.assertRaisesRegex(RuntimeError, "disposition is not terminal"):
            migration._batch(self.manifest, pending_batch["id"])


if __name__ == "__main__":
    unittest.main()
