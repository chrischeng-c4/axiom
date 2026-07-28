"""Focused contracts for TD migration batch planning and publication."""

from __future__ import annotations

import importlib.util
import hashlib
import json
import re
import runpy
import tempfile
import unittest
from pathlib import Path
from unittest import mock


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

    def test_materialize_migrate_moves_markdown_into_canonical_python(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source_rel = (
                "apps/agentic-workflow/tech-design/config/example.md"
            )
            target_rel = (
                "apps/agentic-workflow/tech-design/src/"
                "agentic_workflow/migrated/config/example.py"
            )
            source = root / source_rel
            source.parent.mkdir(parents=True)
            markdown = "# Example\n\nReviewed behavior.\n"
            source.write_text(markdown, encoding="utf-8")
            digest = "sha256:" + hashlib.sha256(markdown.encode()).hexdigest()
            entry = {
                "batch_id": "semantic-config-test",
                "disposition": "migrate",
                "family": "config",
                "path": source_rel,
                "role": "legacy_markdown_td",
                "sha256": digest,
                "status": "pending",
                "target_path": target_rel,
            }
            manifest = {
                "schema": "aw.python-td-migration-reconciliation.v1",
                "batches": [
                    {
                        "id": "semantic-config-test",
                        "artifact_paths": [source_rel],
                        "family": "semantic:config",
                    }
                ],
                "markdown_td": [entry],
            }
            manifest["digest"] = migration._manifest_digest(manifest)
            manifest_path = root / "manifest.json"

            with (
                mock.patch.object(migration, "REPOSITORY_ROOT", root),
                mock.patch.object(migration, "MANIFEST_PATH", manifest_path),
            ):
                result = migration._materialize_batch(
                    manifest,
                    "semantic-config-test",
                )
                loaded = json.loads(manifest_path.read_text(encoding="utf-8"))
                self.assertEqual(
                    migration._batch(loaded, "semantic-config-test")["status"],
                    "ready",
                )

            target = root / target_rel
            namespace = runpy.run_path(str(target))
            self.assertEqual(result["artifact_count"], 1)
            self.assertFalse(source.exists())
            self.assertEqual(namespace["render_markdown"](), markdown)
            self.assertEqual(namespace["__legacy_td_path__"], source_rel)
            self.assertEqual(namespace["__legacy_td_digest__"], digest)

    def test_materialize_preflights_complete_batch_before_writes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            entries = []
            paths = []
            for name in ("first", "second"):
                source_rel = (
                    f"apps/agentic-workflow/tech-design/config/{name}.md"
                )
                target_rel = (
                    "apps/agentic-workflow/tech-design/src/"
                    f"agentic_workflow/migrated/config/{name}.py"
                )
                source = root / source_rel
                source.parent.mkdir(parents=True, exist_ok=True)
                markdown = f"# {name}\n"
                source.write_text(markdown, encoding="utf-8")
                entries.append(
                    {
                        "batch_id": "semantic-config-test",
                        "disposition": "migrate",
                        "family": "config",
                        "path": source_rel,
                        "role": "legacy_markdown_td",
                        "sha256": (
                            "sha256:"
                            + hashlib.sha256(markdown.encode()).hexdigest()
                        ),
                        "status": "pending",
                        "target_path": target_rel,
                    }
                )
                paths.append(source_rel)
            collision = root / entries[1]["target_path"]
            collision.parent.mkdir(parents=True)
            collision.write_text("unowned collision\n", encoding="utf-8")
            manifest = {
                "schema": "aw.python-td-migration-reconciliation.v1",
                "batches": [
                    {
                        "id": "semantic-config-test",
                        "artifact_paths": paths,
                        "family": "semantic:config",
                    }
                ],
                "markdown_td": entries,
            }
            manifest["digest"] = migration._manifest_digest(manifest)

            with (
                mock.patch.object(migration, "REPOSITORY_ROOT", root),
                mock.patch.object(
                    migration,
                    "MANIFEST_PATH",
                    root / "manifest.json",
                ),
                self.assertRaisesRegex(RuntimeError, "target collision"),
            ):
                migration._materialize_batch(
                    manifest,
                    "semantic-config-test",
                )

            self.assertTrue(all((root / path).is_file() for path in paths))
            self.assertFalse(
                (root / entries[0]["target_path"]).exists()
            )
            self.assertEqual(
                collision.read_text(encoding="utf-8"),
                "unowned collision\n",
            )


if __name__ == "__main__":
    unittest.main()
