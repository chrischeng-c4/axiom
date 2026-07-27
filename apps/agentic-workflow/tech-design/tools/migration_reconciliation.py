"""Reconcile the legacy Markdown TD corpus before Python-only migration.

The checked-in manifest is a frozen, digest-bound inventory.  ``refresh`` is
an explicit authoring operation; normal ``verify`` commands never discover a
new artifact and silently bless it.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from collections import defaultdict
from functools import lru_cache
from pathlib import Path
from typing import Any, Iterable


TECH_DESIGN_ROOT = Path(__file__).resolve().parents[1]
PROJECT_ROOT = TECH_DESIGN_ROOT.parent
REPOSITORY_ROOT = PROJECT_ROOT.parents[1]
MANIFEST_PATH = Path(__file__).with_name("migration_reconciliation_manifest.json")
SCRIPT_PATH = Path(__file__).resolve()
BASELINE_MARKDOWN_COUNT = 979
BASELINE_PYTHON_COUNT = 11
MAX_BATCH_ARTIFACTS = 50
FOUNDATION_DEPENDENCIES = ("2711", "2712", "2713")
DISPOSITIONS = {"migrate", "generated_projection", "historical_evidence", "delete"}
REFERENCE_TOKENS = (
    "aw td",
    "aw ec",
    "tech-design/",
    "external-contracts/",
    "python_td",
    "Python TD",
    "Python EC",
    "Markdown TD",
    "Markdown EC",
    "td.lock",
    "ec.lock",
)
LOCK_INPUTS = {
    "apps/agentic-workflow/aw.toml",
    "apps/agentic-workflow/templates/cli/aw.toml",
    "apps/agentic-workflow/tech-design/pyproject.toml",
    "apps/agentic-workflow/tech-design/td.lock",
    "apps/agentic-workflow/external-contracts/pyproject.toml",
    "apps/agentic-workflow/external-contracts/ec.lock",
}
CONTROL_PATHS = {
    "apps/agentic-workflow/tech-design/tools/migration_reconciliation.py",
    "apps/agentic-workflow/tech-design/tools/migration_reconciliation_manifest.json",
}
TEXT_SUFFIXES = {".lock", ".md", ".rs", ".py", ".toml", ".json"}
SOURCE_REFERENCE_TOKENS = (
    "aw td",
    "aw ec",
    "TdCommand",
    "EcCommand",
    "TdArgs",
    "EcArgs",
    "PythonTd",
    "python_td",
    "python-td",
    "td_lock",
    "ec_lock",
    "td.lock",
    "ec.lock",
    "tech_design_root",
    "external_contract",
)
GUIDANCE_PREFIXES = (
    "apps/agentic-workflow/templates/",
    ".agents/agents/aw-",
    ".agents/agents/agentic-workflow",
    ".agents/skills/aw-",
    ".codex/agents/aw-",
    ".codex/agents/agentic-workflow",
    ".claude/agents/aw-",
    ".claude/agents/agentic-workflow",
)
GUIDANCE_PATHS = {
    "AGENTS.md",
    "CONTRIBUTING.md",
    "apps/agentic-workflow/README.md",
    "apps/agentic-workflow/CAPABILITIES.md",
}


def _arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subcommands = parser.add_subparsers(dest="command", required=True)
    refresh = subcommands.add_parser(
        "refresh",
        help="explicitly rewrite the frozen inventory for review",
    )
    refresh.add_argument("--output", type=Path, default=MANIFEST_PATH)

    verify = subcommands.add_parser("verify")
    verify.add_argument("--baseline", action="store_true")
    verify.add_argument("--batch-plan", action="store_true")
    verify.add_argument("--batch")
    args = parser.parse_args()
    if (
        args.command == "verify"
        and not args.baseline
        and not args.batch_plan
        and args.batch is None
    ):
        parser.error("verify requires --baseline, --batch-plan, or --batch <id>")
    return args


def _relative(path: Path) -> str:
    return path.resolve().relative_to(REPOSITORY_ROOT).as_posix()


def _sha256(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def _manifest_digest(manifest: dict[str, Any]) -> str:
    document = {key: value for key, value in manifest.items() if key != "digest"}
    canonical = json.dumps(document, sort_keys=True, separators=(",", ":"))
    return "sha256:" + hashlib.sha256(canonical.encode()).hexdigest()


def _tracked_or_untracked_files() -> Iterable[Path]:
    completed = subprocess.run(
        ["git", "ls-files", "-co", "--exclude-standard", "-z"],
        cwd=REPOSITORY_ROOT,
        capture_output=True,
        check=True,
    )
    for raw in completed.stdout.split(b"\0"):
        if raw:
            yield REPOSITORY_ROOT / raw.decode()


def _markdown_paths() -> list[Path]:
    return sorted(TECH_DESIGN_ROOT.rglob("*.md"))


def _python_td_paths() -> list[Path]:
    source_root = TECH_DESIGN_ROOT / "src"
    return sorted(
        path
        for path in source_root.rglob("*.py")
        if path.resolve() != SCRIPT_PATH
    )


def _semantic_family(path: Path) -> str:
    relative = path.relative_to(TECH_DESIGN_ROOT)
    parts = relative.parts
    if len(parts) == 1:
        return "root"
    if parts[0] in {"logic", "semantic", "specs", "validate", "config"}:
        return parts[0]
    return "/".join(parts[:2])


@lru_cache(maxsize=1)
def _generated_projection_paths() -> frozenset[str]:
    projections: set[str] = set()
    pattern = re.compile(r"SPEC-MANAGED:\s+([^\s#]+\.md)#source")
    for source in (PROJECT_ROOT / "src").rglob("*.rs"):
        text = source.read_text(encoding="utf-8")
        for value in pattern.findall(text):
            candidate = (
                REPOSITORY_ROOT / value
                if value.startswith("apps/")
                else TECH_DESIGN_ROOT / value
            )
            if candidate.is_file() and candidate.is_relative_to(TECH_DESIGN_ROOT):
                projections.add(_relative(candidate))
    return frozenset(projections)


def _is_generated_projection(path: Path) -> bool:
    return _relative(path) in _generated_projection_paths()


def _markdown_disposition(path: Path) -> str:
    relative = path.relative_to(TECH_DESIGN_ROOT).as_posix()
    if relative == "core/README.md":
        return "historical_evidence"
    if _is_generated_projection(path):
        return "generated_projection"
    return "migrate"


def _python_identifier(value: str) -> str:
    identifier = re.sub(r"[^a-zA-Z0-9_]", "_", value).strip("_").lower()
    if not identifier:
        identifier = "artifact"
    if identifier[0].isdigit():
        identifier = f"td_{identifier}"
    return identifier


def _candidate_python_target(path: Path) -> str:
    relative = path.relative_to(TECH_DESIGN_ROOT).with_suffix("")
    directories = [_python_identifier(part) for part in relative.parts[:-1]]
    filename = _python_identifier(relative.name) + ".py"
    target = (
        TECH_DESIGN_ROOT
        / "src"
        / "agentic_workflow"
        / "migrated"
        / Path(*directories)
        / filename
    )
    return _relative(target)


def _target_paths(markdown_paths: list[Path]) -> dict[str, str]:
    candidates: dict[str, list[Path]] = defaultdict(list)
    for path in markdown_paths:
        if _markdown_disposition(path) in {"migrate", "generated_projection"}:
            candidates[_candidate_python_target(path)].append(path)
    targets: dict[str, str] = {}
    for candidate, paths in candidates.items():
        for path in paths:
            target = candidate
            if len(paths) > 1:
                suffix = hashlib.sha256(_relative(path).encode()).hexdigest()[:8]
                target_path = Path(candidate)
                target = str(
                    target_path.with_name(f"{target_path.stem}_{suffix}.py")
                )
            targets[_relative(path)] = target
    return targets


def _entry_role(path: Path) -> str:
    if _is_generated_projection(path):
        return "generated_mirror"
    if path.relative_to(TECH_DESIGN_ROOT).as_posix() == "core/README.md":
        return "historical_index"
    return "legacy_markdown_td"


def _batch_family(entry: dict[str, Any]) -> str:
    prefixes = {
        "migrate": "semantic",
        "generated_projection": "projection",
        "historical_evidence": "evidence",
        "delete": "retirement",
    }
    return f"{prefixes[entry['disposition']]}:{entry['family']}"


def _batch_id(family: str, chunk_index: int) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", family.lower()).strip("-")
    return f"{slug}-{chunk_index:02d}"


def _build_batches(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for entry in entries:
        grouped[_batch_family(entry)].append(entry)
    batches: list[dict[str, Any]] = []
    for family in sorted(grouped):
        artifacts = sorted(grouped[family], key=lambda item: item["path"])
        previous: str | None = None
        for offset in range(0, len(artifacts), MAX_BATCH_ARTIFACTS):
            chunk = artifacts[offset : offset + MAX_BATCH_ARTIFACTS]
            identifier = _batch_id(family, offset // MAX_BATCH_ARTIFACTS + 1)
            dependencies = list(FOUNDATION_DEPENDENCIES)
            if previous is not None:
                dependencies.append(f"batch:{previous}")
            for artifact in chunk:
                artifact["batch_id"] = identifier
            batches.append(
                {
                    "id": identifier,
                    "family": family,
                    "artifact_count": len(chunk),
                    "artifact_paths": [item["path"] for item in chunk],
                    "depends_on": dependencies,
                    "checker": (
                        "python3 apps/agentic-workflow/tech-design/tools/"
                        f"migration_reconciliation.py verify --batch {identifier}"
                    ),
                }
            )
            previous = identifier
    return batches


def _coupled_role(path: Path) -> str:
    relative = _relative(path)
    if relative in LOCK_INPUTS:
        return "lock_input"
    if relative.startswith("apps/agentic-workflow/src/cli/"):
        return "legacy_cli_entry_point"
    if relative.endswith(".md"):
        return "documentation_reference"
    if relative.startswith("apps/agentic-workflow/src/"):
        return "runtime_consumer"
    if "/templates/" in relative or relative.startswith(".agents/"):
        return "agent_guidance"
    return "configuration_or_guidance"


def _coupled_paths() -> list[Path]:
    result: list[Path] = []
    for path in _tracked_or_untracked_files():
        if not path.is_file() or path.suffix not in TEXT_SUFFIXES:
            continue
        relative = _relative(path)
        if relative in CONTROL_PATHS or relative.startswith(".aw/"):
            continue
        if path.is_relative_to(TECH_DESIGN_ROOT):
            if path.suffix == ".md" or path.is_relative_to(
                TECH_DESIGN_ROOT / "src"
            ):
                continue
        if relative in LOCK_INPUTS:
            result.append(path)
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        is_source = relative.startswith("apps/agentic-workflow/src/")
        is_external_contract_control = relative.startswith(
            "apps/agentic-workflow/external-contracts/src/"
        ) and "/cases/" not in relative
        is_guidance = (
            relative in GUIDANCE_PATHS
            or any(relative.startswith(prefix) for prefix in GUIDANCE_PREFIXES)
        )
        if (
            is_source
            and any(token in text for token in SOURCE_REFERENCE_TOKENS)
        ) or (
            (is_external_contract_control or is_guidance)
            and any(token in text for token in REFERENCE_TOKENS)
        ):
            result.append(path)
    return sorted(set(result))


def _new_manifest() -> dict[str, Any]:
    markdown_paths = _markdown_paths()
    python_paths = _python_td_paths()
    targets = _target_paths(markdown_paths)
    markdown: list[dict[str, Any]] = []
    for path in markdown_paths:
        relative = _relative(path)
        disposition = _markdown_disposition(path)
        markdown.append(
            {
                "path": relative,
                "sha256": _sha256(path),
                "role": _entry_role(path),
                "family": _semantic_family(path),
                "disposition": disposition,
                "status": (
                    "classified"
                    if disposition == "historical_evidence"
                    else "pending"
                ),
                "target_path": targets.get(relative),
            }
        )
    batches = _build_batches(markdown)
    manifest: dict[str, Any] = {
        "schema": "aw.python-td-migration-reconciliation.v1",
        "owner_epic": 2707,
        "owner_wi": 2710,
        "baseline": {
            "markdown_td": BASELINE_MARKDOWN_COUNT,
            "python_td": BASELINE_PYTHON_COUNT,
            "max_batch_artifacts": MAX_BATCH_ARTIFACTS,
        },
        "markdown_td": markdown,
        "python_td": [
            {
                "path": _relative(path),
                "sha256": _sha256(path),
                "role": "canonical_python_td",
                "disposition": "historical_evidence",
                "status": "canonical",
            }
            for path in python_paths
        ],
        "coupled_artifacts": [
            {
                "path": _relative(path),
                "sha256": _sha256(path),
                "role": _coupled_role(path),
                "disposition": "migrate",
                "status": "pending",
            }
            for path in _coupled_paths()
        ],
        "batches": batches,
    }
    manifest["digest"] = _manifest_digest(manifest)
    return manifest


def _load_manifest() -> dict[str, Any]:
    return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))


def _duplicates(values: list[str]) -> list[str]:
    counts: dict[str, int] = defaultdict(int)
    for value in values:
        counts[value] += 1
    return sorted(value for value, count in counts.items() if count > 1)


def _allowed_missing(entry: dict[str, Any]) -> bool:
    return entry["status"] == "completed" and entry["disposition"] in {
        "migrate",
        "delete",
    }


def _verify_entry(entry: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    source = REPOSITORY_ROOT / entry["path"]
    if not source.is_file():
        if not _allowed_missing(entry):
            failures.append(f"{entry['path']}: missing source artifact")
    elif entry["status"] != "completed" and _sha256(source) != entry["sha256"]:
        failures.append(f"{entry['path']}: digest drift")
    if entry["status"] == "completed":
        disposition = entry["disposition"]
        target_value = entry.get("target_path")
        target = REPOSITORY_ROOT / target_value if target_value else None
        if disposition == "migrate" and (source.exists() or target is None or not target.is_file()):
            failures.append(f"{entry['path']}: incomplete migration")
        if disposition == "delete" and source.exists():
            failures.append(f"{entry['path']}: incomplete deletion")
        if disposition == "generated_projection" and (
            not source.is_file() or target is None or not target.is_file()
        ):
            failures.append(f"{entry['path']}: projection producer is incomplete")
    return failures


def _verify_structure(manifest: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    if manifest.get("schema") != "aw.python-td-migration-reconciliation.v1":
        failures.append("unsupported manifest schema")
    if manifest.get("digest") != _manifest_digest(manifest):
        failures.append("manifest digest mismatch")
    markdown = manifest.get("markdown_td", [])
    python_td = manifest.get("python_td", [])
    coupled = manifest.get("coupled_artifacts", [])
    all_paths = [
        *(entry.get("path", "") for entry in markdown),
        *(entry.get("path", "") for entry in python_td),
        *(entry.get("path", "") for entry in coupled),
    ]
    duplicate_paths = _duplicates(all_paths)
    if duplicate_paths:
        failures.append(f"duplicate artifact paths: {duplicate_paths}")
    duplicate_targets = _duplicates(
        [
            entry["target_path"]
            for entry in markdown
            if entry.get("target_path") is not None
        ]
    )
    if duplicate_targets:
        failures.append(f"duplicate target paths: {duplicate_targets}")
    for entry in [*markdown, *python_td, *coupled]:
        if entry.get("disposition") not in DISPOSITIONS:
            failures.append(f"{entry.get('path')}: unsupported disposition")
    return failures


def _baseline(manifest: dict[str, Any]) -> dict[str, Any]:
    failures = _verify_structure(manifest)
    markdown = manifest["markdown_td"]
    python_td = manifest["python_td"]
    coupled = manifest["coupled_artifacts"]
    expected_markdown = {entry["path"] for entry in markdown}
    discovered_markdown = {_relative(path) for path in _markdown_paths()}
    completed_removed = {
        entry["path"]
        for entry in markdown
        if _allowed_missing(entry)
    }
    unclassified_markdown = sorted(discovered_markdown - expected_markdown)
    missing_markdown = sorted(
        expected_markdown - completed_removed - discovered_markdown
    )
    expected_python = {entry["path"] for entry in python_td}
    declared_targets = {
        entry["target_path"]
        for entry in markdown
        if entry.get("target_path") is not None
    }
    discovered_python = {_relative(path) for path in _python_td_paths()}
    unexpected_python = sorted(
        discovered_python - expected_python - declared_targets
    )
    expected_coupled = {entry["path"] for entry in coupled}
    discovered_coupled = {_relative(path) for path in _coupled_paths()}
    unclassified_coupled = sorted(discovered_coupled - expected_coupled)
    missing_coupled = sorted(expected_coupled - discovered_coupled)
    for entry in [*markdown, *python_td, *coupled]:
        failures.extend(_verify_entry(entry))
    unmatched = (
        len(unclassified_markdown)
        + len(missing_markdown)
        + len(unexpected_python)
        + len(unclassified_coupled)
        + len(missing_coupled)
    )
    duplicate = len(_duplicates(
        [
            *(entry["path"] for entry in markdown),
            *(entry["path"] for entry in python_td),
            *(entry["path"] for entry in coupled),
        ]
    ))
    result = {
        "schema": manifest["schema"],
        "markdown_td": len(markdown),
        "python_td": len(python_td),
        "coupled_artifacts": len(coupled),
        "unmatched": unmatched,
        "duplicate": duplicate,
        "unclassified_markdown": unclassified_markdown,
        "missing_markdown": missing_markdown,
        "unexpected_python_td": unexpected_python,
        "unclassified_coupled": unclassified_coupled,
        "missing_coupled": missing_coupled,
        "manifest_digest": manifest["digest"],
    }
    if (
        len(markdown) != BASELINE_MARKDOWN_COUNT
        or len(python_td) != BASELINE_PYTHON_COUNT
        or unmatched
        or duplicate
        or failures
    ):
        result["failures"] = failures
        raise RuntimeError(json.dumps(result, sort_keys=True))
    return result


def _batch_plan(manifest: dict[str, Any]) -> dict[str, Any]:
    failures = _verify_structure(manifest)
    markdown = manifest["markdown_td"]
    batches = manifest["batches"]
    batch_ids = [batch["id"] for batch in batches]
    duplicate_batch_ids = _duplicates(batch_ids)
    if duplicate_batch_ids:
        failures.append(f"duplicate batch ids: {duplicate_batch_ids}")
    assigned: list[str] = []
    previous_by_family: dict[str, str] = {}
    for batch in batches:
        paths = batch["artifact_paths"]
        assigned.extend(paths)
        if batch["artifact_count"] != len(paths):
            failures.append(f"{batch['id']}: artifact_count mismatch")
        if not 1 <= len(paths) <= MAX_BATCH_ARTIFACTS:
            failures.append(f"{batch['id']}: batch size is out of bounds")
        expected_checker = (
            "python3 apps/agentic-workflow/tech-design/tools/"
            f"migration_reconciliation.py verify --batch {batch['id']}"
        )
        if batch.get("checker") != expected_checker:
            failures.append(f"{batch['id']}: checker is not executable contract")
        dependencies = batch.get("depends_on", [])
        if not all(item in dependencies for item in FOUNDATION_DEPENDENCIES):
            failures.append(f"{batch['id']}: foundation dependencies are incomplete")
        previous = previous_by_family.get(batch["family"])
        if previous and f"batch:{previous}" not in dependencies:
            failures.append(f"{batch['id']}: prior family batch dependency is missing")
        previous_by_family[batch["family"]] = batch["id"]
    markdown_paths = [entry["path"] for entry in markdown]
    if sorted(assigned) != sorted(markdown_paths):
        failures.append("batch artifact coverage does not equal Markdown inventory")
    entry_batches = {entry["path"]: entry.get("batch_id") for entry in markdown}
    projected_batches = {
        path: batch["id"]
        for batch in batches
        for path in batch["artifact_paths"]
    }
    if entry_batches != projected_batches:
        failures.append("entry batch_id projection disagrees with batch manifest")
    result = {
        "schema": manifest["schema"],
        "batch_count": len(batches),
        "artifact_count": len(assigned),
        "max_batch_artifacts": max(
            (batch["artifact_count"] for batch in batches),
            default=0,
        ),
        "unmatched": 0 if not failures else len(failures),
        "duplicate": len(duplicate_batch_ids),
        "manifest_digest": manifest["digest"],
    }
    if failures:
        result["failures"] = failures
        raise RuntimeError(json.dumps(result, sort_keys=True))
    return result


def _batch(manifest: dict[str, Any], batch_id: str) -> dict[str, Any]:
    batch = next(
        (item for item in manifest["batches"] if item["id"] == batch_id),
        None,
    )
    if batch is None:
        raise RuntimeError(f"unknown migration batch: {batch_id}")
    entries = {
        entry["path"]: entry
        for entry in manifest["markdown_td"]
    }
    failures: list[str] = []
    for path in batch["artifact_paths"]:
        entry = entries.get(path)
        if entry is None:
            failures.append(f"{path}: missing manifest entry")
            continue
        failures.extend(_verify_entry(entry))
    result = {
        "schema": manifest["schema"],
        "batch": batch_id,
        "family": batch["family"],
        "artifact_count": len(batch["artifact_paths"]),
        "status": "ready" if not failures else "blocked",
        "manifest_digest": manifest["digest"],
    }
    if failures:
        result["failures"] = failures
        raise RuntimeError(json.dumps(result, sort_keys=True))
    return result


def main() -> None:
    args = _arguments()
    try:
        if args.command == "refresh":
            manifest = _new_manifest()
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(
                json.dumps(manifest, indent=2, sort_keys=True) + "\n",
                encoding="utf-8",
            )
            result = {
                "status": "refreshed",
                "path": _relative(args.output),
                "markdown_td": len(manifest["markdown_td"]),
                "python_td": len(manifest["python_td"]),
                "coupled_artifacts": len(manifest["coupled_artifacts"]),
                "batch_count": len(manifest["batches"]),
                "manifest_digest": manifest["digest"],
            }
        else:
            manifest = _load_manifest()
            result: dict[str, Any] = {}
            if args.baseline:
                result["baseline"] = _baseline(manifest)
            if args.batch_plan:
                result["batch_plan"] = _batch_plan(manifest)
            if args.batch is not None:
                result["batch"] = _batch(manifest, args.batch)
            if len(result) == 1:
                result = next(iter(result.values()))
        print(json.dumps(result, indent=2, sort_keys=True))
    except (OSError, KeyError, ValueError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"TD migration reconciliation failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error


if __name__ == "__main__":
    main()
