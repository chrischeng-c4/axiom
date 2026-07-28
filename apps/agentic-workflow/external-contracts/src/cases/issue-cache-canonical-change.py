"""Black-box cache round-trip contract for canonical Change work items."""

from __future__ import annotations

import json
from pathlib import Path

from migration_clusters.work_item_planning import BOUNDED_BODY
from wi_contract_fixture import create, final_json, project_fixture, run_aw, show


CASE_ID = "issue-cache-canonical-change"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "canonical-change-cache-round-trip"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "python3 apps/agentic-workflow/external-contracts/src/runner.py "
    "--case issue-cache-canonical-change"
)
ASSERTIONS = (
    "canonical change serializes as type change in the issue cache",
    "cached change round-trips through list and show as type change",
    "one cached change cannot poison EC draft for another work item",
)


def _runtime_root(root: Path) -> Path:
    raw = str(root.resolve())
    slug: list[str] = []
    last_dash = True
    for character in raw:
        if character.isascii() and character.isalnum():
            slug.append(character.lower())
            last_dash = False
        elif not last_dash:
            slug.append("-")
            last_dash = True
    return Path("/tmp/aw/workspaces") / "".join(slug).strip("-")


def verify() -> list[str]:
    with project_fixture() as root:
        cached = create(
            root,
            "Cached canonical change",
            "change",
            "--body",
            BOUNDED_BODY,
        )
        target = create(
            root,
            "Independent EC target",
            "change",
            "--body",
            BOUNDED_BODY,
        )

        cached_path = (
            _runtime_root(root) / "issues" / "open" / f"{cached['slug']}.md"
        )
        raw_cache = cached_path.read_text(encoding="utf-8")
        assert "\ntype: change\n" in raw_cache

        listed = json.loads(
            run_aw(
                root,
                "wi",
                "list",
                "--project",
                "demo",
                "--type",
                "change",
                "--json",
            ).stdout
        )
        by_title = {issue["title"]: issue for issue in listed}
        assert by_title["Cached canonical change"]["type"] == "change"
        assert by_title["Independent EC target"]["type"] == "change"
        assert show(root, cached["slug"])["type"] == "change"

        drafted = final_json(
            run_aw(
                root,
                "ec",
                "draft",
                "cache-round-trip-fixture",
                "--project",
                "demo",
                "--wi",
                target["slug"],
                "--capability-id",
                "work-item-planning",
                "--title",
                "Cached change does not poison EC lifecycle",
                "--json",
            )
        )
        assert drafted["action"] == "python_ec_scaffold_created"
        assert drafted["next"]["command"].startswith("aw ec check ")
        assert all(
            path.startswith("external-contracts/") for path in drafted["artifacts"]
        )

    return list(ASSERTIONS)
