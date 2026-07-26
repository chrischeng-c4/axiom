"""Independent repository-config oracle for Keep's project-local TD routing."""

from __future__ import annotations

import json
import sys
import tomllib
from pathlib import Path


def repository_root() -> Path:
    current = Path(__file__).resolve()
    for parent in current.parents:
        if (parent / "aw.toml").is_file() and (
            parent / "apps" / "agentic-workflow"
        ).is_dir():
            return parent
    raise RuntimeError("repository root with aw.toml was not found")


def main() -> int:
    root = repository_root()
    with (root / "aw.toml").open("rb") as config_file:
        config = tomllib.load(config_file)

    keep_rows = [
        row for row in config.get("projects", []) if row.get("name") == "keep"
    ]
    if len(keep_rows) != 1:
        raise AssertionError(
            f"expected exactly one root [[projects]] row for keep, got {len(keep_rows)}"
        )

    keep = keep_rows[0]
    if keep.get("path") != "apps/keep":
        raise AssertionError(f"keep path must be apps/keep, got {keep.get('path')!r}")
    if "td_path" in keep:
        raise AssertionError(
            f"keep must use the project-local TD default, got td_path={keep['td_path']!r}"
        )

    workspaces = [
        workspace
        for workspace in keep.get("workspaces", [])
        if workspace.get("name") == "keep"
    ]
    if len(workspaces) != 1:
        raise AssertionError(
            f"expected exactly one Keep workspace, got {len(workspaces)}"
        )

    paths = workspaces[0].get("paths", [])
    if paths != ["apps/keep/**"]:
        raise AssertionError(
            f"Keep workspace paths must be exactly ['apps/keep/**'], got {paths!r}"
        )
    retired = [
        path
        for path in paths
        if any(part == ".aw" for part in Path(path).parts)
    ]
    if retired:
        raise AssertionError(f"Keep workspace contains retired .aw paths: {retired!r}")

    print(
        json.dumps(
            {
                "schema_version": "aw.keep-td-routing-config.v1",
                "status": "passed",
                "project_path": keep["path"],
                "effective_td_path": "apps/keep/tech-design",
                "workspace_paths": paths,
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"keep TD routing config oracle failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
