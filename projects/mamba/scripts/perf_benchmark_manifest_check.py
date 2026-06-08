#!/usr/bin/env python3
"""MVP performance benchmark manifest checker (closes #2567).

Parent: #2530 (performance gate suite).

Reads `validation/perf_benchmark_manifest.toml` and confirms every
listed benchmark fixture exists on disk. The manifest catalogs the
benchmark corpus used by the cross-runtime perf harness; without this
checker the harness has no way to fail fast when a required fixture
file goes missing (deleted, renamed, or never committed).

Acceptance (issue #2567):

    1. Gate fails if a required benchmark is missing. A `tier = "required"`
       entry whose `fixture_root / fixture` path does not exist exits 1.
    2. Exploratory benchmarks are reported but not counted in MVP geomean.
       Missing `tier = "exploratory"` fixtures surface in a separate
       warnings section and never trigger a non-zero exit.
    3. Manifest location and update command are documented. This script
       prints the `[update].location` and `[update].command` strings on
       every run so a worker knows where to edit.

Operating modes
---------------

`--format text` (default)
    Human-scannable one-line-per-entry to stderr; exit reflects pass/fail.

`--format json`
    {
      "schema_version": 1,
      "manifest_path": "...",
      "fixture_root": "...",
      "checked_count": N,
      "required_missing": [{"id": ..., "fixture": ..., "category": ..., "tier": "required"}, ...],
      "exploratory_missing": [...],
      "exit_code": 0|1
    }

`--manifest PATH`
    Override the default
    `projects/mamba/validation/perf_benchmark_manifest.toml`.

Exit codes
----------

    0   every required benchmark fixture exists.
    1   one or more required benchmark fixtures missing.
    100 usage / argument error.
    101 manifest file missing or unparseable.

Pure stdlib (Python 3.11+ for tomllib).
"""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


SCHEMA_VERSION = 1
EXIT_USAGE = 100
EXIT_IO = 101
EXIT_FAIL = 1

LEGAL_TIERS = ("required", "exploratory")
REQUIRED_TIER = "required"
EXPLORATORY_TIER = "exploratory"


@dataclass
class Entry:
    id: str
    fixture: str
    category: str
    tier: str
    weight: float | None
    resolved_path: Path


def _die(code: int, msg: str) -> None:
    sys.stderr.write(f"perf_benchmark_manifest_check: {msg}\n")
    sys.exit(code)


def _load(path: Path) -> dict[str, Any]:
    if not path.is_file():
        _die(EXIT_IO, f"manifest missing: {path}")
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as exc:
        _die(EXIT_IO, f"manifest invalid TOML ({exc}): {path}")
        return {}


def _collect(
    data: dict[str, Any], manifest_dir: Path
) -> tuple[list[Entry], list[str], Path]:
    """Return (entries, errors_about_manifest_shape, fixture_root)."""
    errors: list[str] = []
    fixture_root_raw = data.get("fixture_root")
    if not isinstance(fixture_root_raw, str) or not fixture_root_raw:
        _die(EXIT_IO, "manifest missing required string field 'fixture_root'")
        return [], [], manifest_dir
    fixture_root = (manifest_dir / fixture_root_raw).resolve()
    categories = data.get("categories") or []
    tiers = data.get("tiers") or list(LEGAL_TIERS)
    if not isinstance(categories, list) or not categories:
        errors.append("manifest 'categories' must be a non-empty list")
    if not isinstance(tiers, list) or not tiers:
        errors.append("manifest 'tiers' must be a non-empty list")
    entries_raw = data.get("benchmarks") or []
    if not isinstance(entries_raw, list):
        _die(EXIT_IO, "manifest 'benchmarks' must be an array of tables")
    seen_ids: set[str] = set()
    out: list[Entry] = []
    for idx, e in enumerate(entries_raw):
        if not isinstance(e, dict):
            errors.append(f"benchmarks[{idx}]: entry is not a TOML table")
            continue
        id_ = e.get("id")
        fixture = e.get("fixture")
        category = e.get("category")
        tier = e.get("tier")
        weight = e.get("weight")
        if not isinstance(id_, str) or not id_:
            errors.append(f"benchmarks[{idx}]: missing string 'id'")
            continue
        if id_ in seen_ids:
            errors.append(f"benchmarks[{idx}]: duplicate id {id_!r}")
        seen_ids.add(id_)
        if not isinstance(fixture, str) or not fixture:
            errors.append(f"benchmarks[{idx}] ({id_}): missing string 'fixture'")
            continue
        if not isinstance(category, str) or category not in categories:
            errors.append(
                f"benchmarks[{idx}] ({id_}): category {category!r} not in {categories}"
            )
        if not isinstance(tier, str) or tier not in tiers:
            errors.append(
                f"benchmarks[{idx}] ({id_}): tier {tier!r} not in {tiers}"
            )
            continue
        weight_val: float | None
        if weight is None:
            weight_val = None
        elif isinstance(weight, (int, float)):
            weight_val = float(weight)
        else:
            errors.append(
                f"benchmarks[{idx}] ({id_}): weight must be numeric, got {weight!r}"
            )
            weight_val = None
        resolved = (fixture_root / fixture).resolve()
        out.append(Entry(id_, fixture, category or "", tier, weight_val, resolved))
    return out, errors, fixture_root


def _format_text(
    manifest_path: Path,
    update: dict[str, Any],
    fixture_root: Path,
    entries: list[Entry],
    schema_errors: list[str],
    required_missing: list[Entry],
    exploratory_missing: list[Entry],
) -> str:
    lines = [
        f"perf_benchmark_manifest_check: manifest={manifest_path}",
        f"  fixture_root={fixture_root}",
        f"  checked={len(entries)} required_missing={len(required_missing)} "
        f"exploratory_missing={len(exploratory_missing)} schema_errors={len(schema_errors)}",
    ]
    location = update.get("location")
    command = update.get("command")
    if isinstance(location, str):
        lines.append(f"  update.location={location}")
    if isinstance(command, str):
        lines.append(f"  update.command={command}")
    if schema_errors:
        lines.append("schema errors:")
        for e in schema_errors:
            lines.append(f"  {e}")
    if required_missing:
        lines.append("required benchmarks missing fixture (gate fails):")
        for e in required_missing:
            lines.append(
                f"  id={e.id:<24} fixture={e.fixture:<24} category={e.category}"
            )
    if exploratory_missing:
        lines.append("exploratory benchmarks missing fixture (reported, not gating):")
        for e in exploratory_missing:
            lines.append(
                f"  id={e.id:<24} fixture={e.fixture:<24} category={e.category}"
            )
    if not (required_missing or schema_errors):
        lines.append("perf_benchmark_manifest_check: clean")
    return "\n".join(lines) + "\n"


def _format_json(
    manifest_path: Path,
    update: dict[str, Any],
    fixture_root: Path,
    entries: list[Entry],
    schema_errors: list[str],
    required_missing: list[Entry],
    exploratory_missing: list[Entry],
    exit_code: int,
) -> str:
    def row(e: Entry) -> dict[str, Any]:
        return {
            "id": e.id,
            "fixture": e.fixture,
            "category": e.category,
            "tier": e.tier,
            "weight": e.weight,
            "resolved_path": str(e.resolved_path),
        }
    payload = {
        "schema_version": SCHEMA_VERSION,
        "manifest_path": str(manifest_path),
        "fixture_root": str(fixture_root),
        "update_location": update.get("location"),
        "update_command": update.get("command"),
        "checked_count": len(entries),
        "schema_errors": schema_errors,
        "required_missing": [row(e) for e in required_missing],
        "exploratory_missing": [row(e) for e in exploratory_missing],
        "exit_code": exit_code,
    }
    return json.dumps(payload, indent=2, sort_keys=True) + "\n"


def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="perf_benchmark_manifest_check",
        description="Validate the MVP perf benchmark manifest (#2567).",
    )
    p.add_argument(
        "--manifest",
        type=Path,
        default=None,
        help="path to perf_benchmark_manifest.toml "
             "(default: projects/mamba/validation/perf_benchmark_manifest.toml)",
    )
    p.add_argument("--format", choices=("text", "json"), default="text")
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    ns = _parse_args(list(sys.argv[1:] if argv is None else argv))

    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    default_path = project_root / "validation" / "perf_benchmark_manifest.toml"
    manifest_path = (ns.manifest or default_path).resolve()
    manifest_dir = manifest_path.parent

    data = _load(manifest_path)
    entries, schema_errors, fixture_root = _collect(data, manifest_dir)

    required_missing: list[Entry] = []
    exploratory_missing: list[Entry] = []
    for e in entries:
        if e.resolved_path.is_file():
            continue
        if e.tier == REQUIRED_TIER:
            required_missing.append(e)
        else:
            exploratory_missing.append(e)
    required_missing.sort(key=lambda e: e.id)
    exploratory_missing.sort(key=lambda e: e.id)

    exit_code = EXIT_FAIL if (required_missing or schema_errors) else 0

    update = data.get("update") or {}
    if not isinstance(update, dict):
        update = {}

    if ns.format == "json":
        sys.stdout.write(
            _format_json(
                manifest_path, update, fixture_root, entries, schema_errors,
                required_missing, exploratory_missing, exit_code,
            )
        )
    else:
        sys.stderr.write(
            _format_text(
                manifest_path, update, fixture_root, entries, schema_errors,
                required_missing, exploratory_missing,
            )
        )
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
