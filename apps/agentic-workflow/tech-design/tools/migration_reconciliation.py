"""Reconcile the legacy Markdown TD corpus before Python-only migration.

The checked-in manifest is a frozen, digest-bound inventory.  ``refresh`` is
an explicit authoring operation; normal ``verify`` commands never discover a
new artifact and silently bless it.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import runpy
import shutil
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
BASELINE_PYTHON_COUNT = 12
MAX_BATCH_ARTIFACTS = 50
FOUNDATION_DEPENDENCIES = ("2711", "2712", "2713")
PUBLICATION_OWNER_WI = "2714"
PUBLICATION_EPIC = "2707"
PUBLICATION_PROJECT = "agentic-workflow"
TERMINAL_GUIDANCE_ID = "guidance-retirement"
TERMINAL_PROOF_ID = "lock-proof-dogfood"
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

    render = subcommands.add_parser(
        "render-project-plan",
        help="project the frozen batch manifest into epic requirements",
    )
    render.add_argument("--body", type=Path, required=True)
    render.add_argument("--output", type=Path, required=True)

    materialize = subcommands.add_parser(
        "materialize",
        help="materialize one reviewed generated-projection batch as Python source",
    )
    materialize.add_argument("--batch", required=True)

    verify = subcommands.add_parser("verify")
    verify.add_argument("--baseline", action="store_true")
    verify.add_argument("--batch-plan", action="store_true")
    verify.add_argument("--published-batches", action="store_true")
    verify.add_argument("--guidance-retired", action="store_true")
    verify.add_argument("--migration-complete", action="store_true")
    verify.add_argument("--batch")
    args = parser.parse_args()
    if (
        args.command == "verify"
        and not args.baseline
        and not args.batch_plan
        and not args.published_batches
        and not args.guidance_retired
        and not args.migration_complete
        and args.batch is None
    ):
        parser.error(
            "verify requires --baseline, --batch-plan, --published-batches, "
            "--guidance-retired, --migration-complete, or --batch <id>"
        )
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


def _projection_artifact_id(entry: dict[str, Any]) -> str:
    relative = Path(entry["path"]).relative_to(
        "apps/agentic-workflow/tech-design"
    )
    family = _python_identifier(entry["family"]).replace("_", "-")
    name = "-".join(
        _python_identifier(part).replace("_", "-")
        for part in relative.with_suffix("").parts
    )
    # `_target_paths` appends a digest suffix only when two distinct legacy
    # paths normalize to the same Python module name. Preserve the semantic
    # legacy TD identity and give colliding generated mirrors a stable,
    # role-explicit identity so a replay cannot recreate a project-wide
    # artifact-id collision.
    target_stem = Path(entry.get("target_path") or "").stem
    collision = re.search(r"_([0-9a-f]{8})$", target_stem)
    if entry.get("role") == "generated_mirror" and collision is not None:
        name = f"{name}-generated-projection-{collision.group(1)}"
    return f"artifact:{family}/{name}"


def _render_python_projection(entry: dict[str, Any], markdown: str) -> str:
    """Render a self-contained Python producer for one legacy projection.

    The Markdown output is preserved byte-for-byte as executable Python data.
    Its digest is also compiled into the canonical Python TD IR through the
    function annotation, so content drift cannot be hidden in a function body.
    """

    artifact_id = _projection_artifact_id(entry)
    encoded_markdown = json.dumps(markdown, ensure_ascii=False)
    return (
        f'"""Canonical Python producer for `{entry["path"]}`.\n\n'
        f'Migrated by batch `{entry["batch_id"]}`.\n"""\n\n'
        "from __future__ import annotations\n\n"
        "from typing import Annotated\n\n\n"
        f'__aw_artifact_id__ = "{artifact_id}"\n'
        f'__legacy_projection_path__ = "{entry["path"]}"\n'
        f'__legacy_projection_digest__ = "{entry["sha256"]}"\n\n\n'
        "def render_markdown() -> "
        f'Annotated[str, "{entry["sha256"]}"]:\n'
        '    """Render the preserved generated projection byte-for-byte."""\n\n'
        f"    return {encoded_markdown}\n"
    )


def _render_python_migration(entry: dict[str, Any], markdown: str) -> str:
    """Render one legacy Markdown TD as its canonical Python owner.

    The legacy document remains executable data instead of an active Markdown
    authoring surface.  Its stable artifact identity and digest are compiled
    into the Python TD IR, while ``render_markdown`` preserves every reviewed
    byte for audit and later semantic extraction.
    """

    artifact_id = _projection_artifact_id(entry)
    encoded_markdown = json.dumps(markdown, ensure_ascii=False)
    return (
        f'"""Canonical Python tech design migrated from `{entry["path"]}`.\n\n'
        f'Migrated by batch `{entry["batch_id"]}`.\n"""\n\n'
        "from __future__ import annotations\n\n"
        "from typing import Annotated\n\n\n"
        f'__aw_artifact_id__ = "{artifact_id}"\n'
        f'__legacy_td_path__ = "{entry["path"]}"\n'
        f'__legacy_td_digest__ = "{entry["sha256"]}"\n\n\n'
        "def render_markdown() -> "
        f'Annotated[str, "{entry["sha256"]}"]:\n'
        '    """Render the preserved legacy design byte-for-byte."""\n\n'
        f"    return {encoded_markdown}\n"
    )


def _materialize_batch(manifest: dict[str, Any], batch_id: str) -> dict[str, Any]:
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
    planned: list[tuple[dict[str, Any], Path, Path, str]] = []
    materialized: list[str] = []
    for path in batch["artifact_paths"]:
        entry = entries.get(path)
        if entry is None:
            failures.append(f"{path}: missing manifest entry")
            continue
        if entry["disposition"] not in {"generated_projection", "migrate"}:
            failures.append(
                f"{path}: materializer requires migrate or generated_projection"
            )
            continue
        source = REPOSITORY_ROOT / path
        target_value = entry.get("target_path")
        if not source.is_file():
            failures.append(f"{path}: missing source artifact")
            continue
        if target_value is None:
            failures.append(f"{path}: missing Python migration target")
            continue
        if _sha256(source) != entry["sha256"]:
            failures.append(f"{path}: digest drift")
            continue
        target = REPOSITORY_ROOT / target_value
        markdown = source.read_text(encoding="utf-8")
        rendered = (
            _render_python_projection(entry, markdown)
            if entry["disposition"] == "generated_projection"
            else _render_python_migration(entry, markdown)
        )
        if target.exists() and (
            not target.is_file()
            or target.read_text(encoding="utf-8") != rendered
        ):
            failures.append(f"{target_value}: target collision")
            continue
        planned.append((entry, source, target, rendered))
    if failures:
        raise RuntimeError(
            json.dumps(
                {
                    "schema": manifest["schema"],
                    "batch": batch_id,
                    "status": "blocked",
                    "failures": failures,
                },
                sort_keys=True,
            )
        )
    for entry, _source, target, rendered in planned:
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(rendered, encoding="utf-8")
    for entry, source, _target, _rendered in planned:
        if entry["disposition"] == "migrate":
            source.unlink()
        entry["status"] = "completed"
        materialized.append(entry["target_path"])
    manifest["digest"] = _manifest_digest(manifest)
    MANIFEST_PATH.write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return {
        "schema": manifest["schema"],
        "batch": batch_id,
        "artifact_count": len(materialized),
        "status": "materialized",
        "manifest_digest": manifest["digest"],
        "targets": materialized,
    }


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
        if disposition == "migrate":
            if target is None:
                if not source.is_file():
                    failures.append(f"{entry['path']}: incomplete in-place reconciliation")
            elif source.exists() or not target.is_file():
                failures.append(f"{entry['path']}: incomplete migration")
        if disposition == "delete" and source.exists():
            failures.append(f"{entry['path']}: incomplete deletion")
        if disposition == "generated_projection" and (
            not source.is_file() or target is None or not target.is_file()
        ):
            failures.append(f"{entry['path']}: projection producer is incomplete")
        if disposition in {"generated_projection", "migrate"} and (
            target is not None and target.is_file()
        ):
            try:
                namespace = runpy.run_path(str(target))
            except Exception as error:
                failures.append(
                    f"{entry['path']}: Python producer cannot execute: {error}"
                )
            else:
                renderer = namespace.get("render_markdown")
                if not callable(renderer):
                    failures.append(
                        f"{entry['path']}: Python producer has no render_markdown"
                    )
                else:
                    rendered = renderer()
                    rendered_digest = (
                        "sha256:"
                        + hashlib.sha256(rendered.encode("utf-8")).hexdigest()
                    )
                    if rendered_digest != entry["sha256"]:
                        failures.append(
                            f"{entry['path']}: Python producer output drift"
                        )
                    if (
                        disposition == "generated_projection"
                        and rendered != source.read_text(encoding="utf-8")
                    ):
                        failures.append(
                            f"{entry['path']}: projection producer output drift"
                        )
                if namespace.get("__aw_artifact_id__") != _projection_artifact_id(
                    entry
                ):
                    failures.append(
                        f"{entry['path']}: Python artifact identity drift"
                    )
                path_key = (
                    "__legacy_projection_path__"
                    if disposition == "generated_projection"
                    else "__legacy_td_path__"
                )
                digest_key = (
                    "__legacy_projection_digest__"
                    if disposition == "generated_projection"
                    else "__legacy_td_digest__"
                )
                if namespace.get(path_key) != entry["path"]:
                    failures.append(
                        f"{entry['path']}: Python source identity drift"
                    )
                if namespace.get(digest_key) != entry["sha256"]:
                    failures.append(
                        f"{entry['path']}: Python digest identity drift"
                    )
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


def _publication_requirements(
    manifest: dict[str, Any],
) -> list[dict[str, Any]]:
    """Project one bounded requirement per batch plus two terminal changes."""

    requirements: list[dict[str, Any]] = []
    requirement_by_batch: dict[str, str] = {}
    for offset, batch in enumerate(manifest["batches"], start=6):
        requirement_id = f"R{offset}"
        requirement_by_batch[batch["id"]] = requirement_id
        dependencies = ["R5"]
        dependencies.extend(
            requirement_by_batch[dependency.removeprefix("batch:")]
            for dependency in batch["depends_on"]
            if dependency.startswith("batch:")
        )
        requirements.append(
            {
                "id": requirement_id,
                "kind": "batch",
                "batch_id": batch["id"],
                "text": (
                    f"Migration batch `{batch['id']}`: materialize the terminal "
                    f"disposition of exactly {batch['artifact_count']} manifest-owned "
                    f"artifact(s) in family `{batch['family']}` without modifying "
                    "artifacts owned by another batch. Preserve every manifest identity "
                    "and update an entry to terminal status only after its Python source "
                    "or generated projection is complete."
                ),
                "gate": batch["checker"],
                "oracle": (
                    f"Batch `{batch['id']}` owns exactly "
                    f"{batch['artifact_count']} artifact(s), reports terminal status, "
                    "and rejects missing sources, producers, or manifest identity drift"
                ),
                "depends_on": dependencies,
            }
        )

    batch_requirement_ids = [item["id"] for item in requirements]
    guidance_requirement_id = f"R{len(requirements) + 6}"
    requirements.append(
        {
            "id": guidance_requirement_id,
            "kind": "terminal",
            "terminal_id": TERMINAL_GUIDANCE_ID,
            "text": (
                f"Migration terminal `{TERMINAL_GUIDANCE_ID}`: after all "
                f"{len(batch_requirement_ids)} corpus batches are green, update "
                "CLI/help/guidance projections and remove retired Markdown TD/EC "
                "implementation without changing migrated product behavior."
            ),
            "gate": (
                "python3 apps/agentic-workflow/tech-design/tools/"
                "migration_reconciliation.py verify --guidance-retired"
            ),
            "oracle": (
                "Every manifest batch is terminal, coupled guidance is reconciled, "
                "and no active or canonical Markdown TD/EC authoring path remains"
            ),
            "depends_on": batch_requirement_ids,
        }
    )
    requirements.append(
        {
            "id": f"R{len(requirements) + 6}",
            "kind": "terminal",
            "terminal_id": TERMINAL_PROOF_ID,
            "text": (
                f"Migration terminal `{TERMINAL_PROOF_ID}`: regenerate Python EC/TD "
                "locks, obtain digest-bound independent EC acceptance, pass the final "
                "Python/cold/Rust invariant gates, and run existing #2688 to terminal "
                "without creating duplicate dogfood work."
            ),
            "gate": (
                "python3 apps/agentic-workflow/tech-design/tools/"
                "migration_reconciliation.py verify --migration-complete"
            ),
            "oracle": (
                "Reports all published migration children terminal, exact disposition "
                "totals, zero active canonical Markdown, accepted locks/evidence, and "
                "#2688 terminal"
            ),
            "depends_on": [guidance_requirement_id],
        }
    )
    return requirements


def _replace_markdown_section(body: str, heading: str, content: str) -> str:
    pattern = re.compile(
        rf"(?ms)^## {re.escape(heading)}\n.*?(?=^## |\Z)"
    )
    replacement = f"## {heading}\n\n{content.rstrip()}\n\n"
    if not pattern.search(body):
        raise RuntimeError(f"epic body is missing ## {heading}")
    return pattern.sub(replacement, body, count=1).rstrip() + "\n"


def _render_project_plan_body(
    manifest: dict[str, Any],
    source_body: str,
) -> str:
    requirements = _publication_requirements(manifest)
    source_requirements = re.search(
        r"(?ms)^## Requirements\n\n(.*?)(?=^## |\Z)",
        source_body,
    )
    if source_requirements is None:
        raise RuntimeError("epic body is missing ## Requirements")
    foundation = [
        requirement
        for requirement in re.split(
            r"\n(?=- R\d+:)",
            source_requirements.group(1).strip(),
        )
        if re.match(r"- R[1-5]:", requirement)
    ]
    if len(foundation) != 5:
        raise RuntimeError(
            "epic body must retain exactly R1-R5 before batch projection"
        )
    requirement_lines = [
        *foundation,
        *(
            f"- {requirement['id']}: {requirement['text']}"
            for requirement in requirements
        ),
    ]
    body = _replace_markdown_section(
        source_body,
        "Requirements",
        "\n".join(requirement_lines),
    )

    source_inventory = re.search(
        r"(?ms)^## Verification Inventory\n\n(.*?)(?=^## |\Z)",
        source_body,
    )
    if source_inventory is None:
        raise RuntimeError("epic body is missing ## Verification Inventory")
    inventory_rows = [
        line
        for line in source_inventory.group(1).splitlines()
        if line.startswith("|")
    ]
    if len(inventory_rows) < 7:
        raise RuntimeError("epic verification inventory is incomplete")
    foundation_rows = [
        row for row in inventory_rows[2:] if re.match(r"\| R[1-5] \|", row)
    ]
    if len(foundation_rows) != 5:
        raise RuntimeError(
            "epic verification inventory must retain exactly R1-R5"
        )
    projected_rows = []
    for requirement in requirements:
        dependencies = ", ".join(requirement["depends_on"]) or "-"
        projected_rows.append(
            f"| {requirement['id']} | `{requirement['gate']}` | "
            f"{requirement['oracle']} | {dependencies} |"
        )
    inventory = "\n".join(
        [
            "| Requirement | Gate | Oracle | Depends On |",
            "|---|---|---|---|",
            *foundation_rows,
            *projected_rows,
        ]
    )
    return _replace_markdown_section(body, "Verification Inventory", inventory)


def _aw_binary() -> str:
    configured = os.environ.get("AW_BIN")
    if configured:
        return configured
    local = REPOSITORY_ROOT / "target" / "debug" / "aw"
    if local.is_file():
        return str(local)
    installed = shutil.which("aw")
    if installed:
        return installed
    raise RuntimeError("cannot locate aw binary; set AW_BIN")


def _run_aw_json(*args: str) -> Any:
    completed = subprocess.run(
        [_aw_binary(), *args],
        cwd=REPOSITORY_ROOT,
        capture_output=True,
        check=True,
        text=True,
    )
    return json.loads(completed.stdout)


def _published_issue_inventory() -> list[dict[str, Any]]:
    graph = _run_aw_json(
        "wi",
        "graph",
        "--project",
        PUBLICATION_PROJECT,
        "--json",
    )
    if not graph.get("valid"):
        raise RuntimeError("published work-item graph is invalid")
    children = [
        change
        for change in graph.get("changes", [])
        if change.get("parent") == PUBLICATION_EPIC
    ]
    inventory = []
    for child in children:
        issue = _run_aw_json("wi", "show", str(child["id"]))
        issue["dependencies"] = child.get("dependencies", [])
        inventory.append(issue)
    return inventory


def _body_marker(body: str, marker: str) -> str | None:
    match = re.search(
        rf"{re.escape(marker)}\s+`([^`]+)`",
        body,
        re.IGNORECASE,
    )
    return match.group(1) if match else None


def _publication_projection(
    manifest: dict[str, Any],
    issues: list[dict[str, Any]],
) -> tuple[dict[str, dict[str, Any]], dict[str, dict[str, Any]], list[str]]:
    failures: list[str] = []
    batches: dict[str, dict[str, Any]] = {}
    terminals: dict[str, dict[str, Any]] = {}
    for issue in issues:
        body = issue.get("body", "")
        batch_id = _body_marker(body, "Migration batch")
        terminal_id = _body_marker(body, "Migration terminal")
        if batch_id:
            if batch_id in batches:
                failures.append(f"{batch_id}: duplicate published batch")
            batches[batch_id] = issue
        if terminal_id:
            if terminal_id in terminals:
                failures.append(f"{terminal_id}: duplicate terminal change")
            terminals[terminal_id] = issue

    expected_batch_ids = {batch["id"] for batch in manifest["batches"]}
    unknown_batches = sorted(set(batches) - expected_batch_ids)
    missing_batches = sorted(expected_batch_ids - set(batches))
    if unknown_batches:
        failures.append(f"unknown published batches: {unknown_batches}")
    if missing_batches:
        failures.append(f"missing published batches: {missing_batches}")
    expected_terminals = {TERMINAL_GUIDANCE_ID, TERMINAL_PROOF_ID}
    if set(terminals) != expected_terminals:
        failures.append(
            "terminal changes mismatch: "
            f"expected={sorted(expected_terminals)} actual={sorted(terminals)}"
        )
    return batches, terminals, failures


def _published_batches(
    manifest: dict[str, Any],
    issues: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    failures = _verify_structure(manifest)
    failures.extend(_batch_plan(manifest).get("failures", []))
    inventory = _published_issue_inventory() if issues is None else issues
    batches, terminals, projection_failures = _publication_projection(
        manifest,
        inventory,
    )
    failures.extend(projection_failures)
    published_id_by_batch = {
        batch_id: str(issue.get("github_id"))
        for batch_id, issue in batches.items()
    }
    previous_by_family: dict[str, str] = {}
    for batch in manifest["batches"]:
        issue = batches.get(batch["id"])
        if issue is None:
            continue
        body = issue.get("body", "")
        dependencies = {str(value) for value in issue.get("dependencies", [])}
        if batch["checker"] not in body:
            failures.append(f"{batch['id']}: checker is missing from issue body")
        if "aw:planning-transaction:" not in body:
            failures.append(
                f"{batch['id']}: reviewed planning transaction marker is missing"
            )
        if PUBLICATION_OWNER_WI not in dependencies:
            failures.append(
                f"{batch['id']}: dependency on publication WI #{PUBLICATION_OWNER_WI} is missing"
            )
        previous = previous_by_family.get(batch["family"])
        if previous:
            expected = published_id_by_batch.get(previous)
            if expected is None or expected not in dependencies:
                failures.append(
                    f"{batch['id']}: published dependency on prior batch {previous} is missing"
                )
        previous_by_family[batch["family"]] = batch["id"]

    guidance = terminals.get(TERMINAL_GUIDANCE_ID)
    proof = terminals.get(TERMINAL_PROOF_ID)
    if guidance is not None:
        dependencies = {str(value) for value in guidance.get("dependencies", [])}
        expected = set(published_id_by_batch.values())
        if dependencies != expected:
            failures.append(
                f"{TERMINAL_GUIDANCE_ID}: dependency set does not equal all published batches"
            )
        if "aw:planning-transaction:" not in guidance.get("body", ""):
            failures.append(
                f"{TERMINAL_GUIDANCE_ID}: reviewed planning transaction marker is missing"
            )
    if proof is not None:
        dependencies = {str(value) for value in proof.get("dependencies", [])}
        guidance_id = str(guidance.get("github_id")) if guidance else None
        if guidance_id is None or dependencies != {guidance_id}:
            failures.append(
                f"{TERMINAL_PROOF_ID}: dependency must be exactly the guidance terminal"
            )
        body = proof.get("body", "")
        if "#2688" not in body:
            failures.append(f"{TERMINAL_PROOF_ID}: existing #2688 dogfood is not referenced")
        if "aw:planning-transaction:" not in body:
            failures.append(
                f"{TERMINAL_PROOF_ID}: reviewed planning transaction marker is missing"
            )

    result = {
        "schema": manifest["schema"],
        "batch_count": len(batches),
        "terminal_change_count": len(terminals),
        "published_change_count": len(batches) + len(terminals),
        "unmatched": len(failures),
        "duplicate": sum("duplicate" in failure for failure in failures),
        "manifest_digest": manifest["digest"],
    }
    if failures:
        result["failures"] = failures
        raise RuntimeError(json.dumps(result, sort_keys=True))
    return result


def _migration_complete(
    manifest: dict[str, Any],
    issues: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    guidance = _guidance_retired(manifest, issues)
    publication = guidance["publication"]
    failures: list[str] = []
    disposition_counts: dict[str, int] = defaultdict(int)
    terminal_counts: dict[str, int] = defaultdict(int)
    for entry in manifest["markdown_td"]:
        disposition_counts[entry["disposition"]] += 1
        terminal = entry["status"] == "completed" or (
            entry["disposition"] == "historical_evidence"
            and entry["status"] == "classified"
            and not (REPOSITORY_ROOT / entry["path"]).is_relative_to(
                TECH_DESIGN_ROOT / "src"
            )
        )
        if terminal:
            terminal_counts[entry["disposition"]] += 1
        else:
            failures.append(f"{entry['path']}: disposition is not terminal")
        failures.extend(_verify_entry(entry))
    live_issues = guidance["issues"]
    _, terminals, _ = _publication_projection(manifest, live_issues)
    migration_children = [
        issue
        for issue in live_issues
        if _body_marker(issue.get("body", ""), "Migration batch")
        or _body_marker(issue.get("body", ""), "Migration terminal")
    ]
    open_children = [
        (
            str(issue.get("github_id")),
            _body_marker(issue.get("body", ""), "Migration terminal"),
        )
        for issue in migration_children
        if issue.get("state") != "closed"
    ]
    blocking_open_children = [
        issue_id
        for issue_id, terminal_id in open_children
        if terminal_id != TERMINAL_PROOF_ID
    ]
    if blocking_open_children:
        failures.append(
            "published migration prerequisites are not terminal: "
            f"{blocking_open_children}"
        )
    if terminals and TERMINAL_PROOF_ID not in terminals:
        failures.append("terminal lock-proof change is missing")
    if issues is None:
        dogfood = _run_aw_json("wi", "show", "2688")
        if dogfood.get("state") != "closed":
            failures.append("#2688 is not terminal")
    result = {
        "schema": manifest["schema"],
        "children_terminal": (
            f"{len(migration_children) - len(open_children)}/"
            f"{len(migration_children)}"
        ),
        "terminal_proof_ready": not blocking_open_children,
        "migrated": (
            f"{terminal_counts['migrate']}/{disposition_counts['migrate']}"
        ),
        "projections_terminal": (
            f"{terminal_counts['generated_projection']}/"
            f"{disposition_counts['generated_projection']}"
        ),
        "historical_evidence": (
            f"{terminal_counts['historical_evidence']}/"
            f"{disposition_counts['historical_evidence']}"
        ),
        "reconciled": (
            f"{sum(terminal_counts.values())}/"
            f"{len(manifest['markdown_td'])}"
        ),
        "active_markdown_td": sum(
            entry["status"] != "completed"
            and entry["disposition"] != "historical_evidence"
            for entry in manifest["markdown_td"]
        ),
        "published_change_count": publication["published_change_count"],
        "manifest_digest": manifest["digest"],
    }
    if failures:
        result["failures"] = failures
        raise RuntimeError(json.dumps(result, sort_keys=True))
    return result


def _guidance_retired(
    manifest: dict[str, Any],
    issues: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    live_issues = _published_issue_inventory() if issues is None else issues
    publication = _published_batches(manifest, live_issues)
    failures: list[str] = []
    batch_issues = [
        issue
        for issue in live_issues
        if _body_marker(issue.get("body", ""), "Migration batch")
    ]
    open_batches = [
        str(issue.get("github_id"))
        for issue in batch_issues
        if issue.get("state") != "closed"
    ]
    if open_batches:
        failures.append(f"published migration batches are not terminal: {open_batches}")
    pending_coupled = []
    for entry in manifest["coupled_artifacts"]:
        if entry["status"] != "completed":
            pending_coupled.append(entry["path"])
        failures.extend(_verify_entry(entry))
    if pending_coupled:
        failures.append(
            f"coupled guidance is not reconciled: {pending_coupled}"
        )
    result = {
        "schema": manifest["schema"],
        "batches_terminal": f"{len(batch_issues) - len(open_batches)}/{len(batch_issues)}",
        "coupled_terminal": (
            f"{len(manifest['coupled_artifacts']) - len(pending_coupled)}/"
            f"{len(manifest['coupled_artifacts'])}"
        ),
        "publication": publication,
        "issues": live_issues,
        "manifest_digest": manifest["digest"],
    }
    if failures:
        public_result = {
            key: value
            for key, value in result.items()
            if key not in {"publication", "issues"}
        }
        public_result["failures"] = failures
        raise RuntimeError(json.dumps(public_result, sort_keys=True))
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
        if entry["status"] != "completed" and not (
            entry["disposition"] == "historical_evidence"
            and entry["status"] == "classified"
        ):
            failures.append(f"{path}: disposition is not terminal")
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
        elif args.command == "render-project-plan":
            manifest = _load_manifest()
            rendered = _render_project_plan_body(
                manifest,
                args.body.read_text(encoding="utf-8"),
            )
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(rendered, encoding="utf-8")
            result = {
                "status": "rendered",
                "path": args.output.resolve().as_posix(),
                "batch_requirement_count": len(manifest["batches"]),
                "terminal_requirement_count": 2,
                "manifest_digest": manifest["digest"],
            }
        elif args.command == "materialize":
            result = _materialize_batch(_load_manifest(), args.batch)
        else:
            manifest = _load_manifest()
            result: dict[str, Any] = {}
            if args.baseline:
                result["baseline"] = _baseline(manifest)
            if args.batch_plan:
                result["batch_plan"] = _batch_plan(manifest)
            if args.published_batches:
                result["published_batches"] = _published_batches(manifest)
            if args.guidance_retired:
                guidance = _guidance_retired(manifest)
                result["guidance_retired"] = {
                    key: value
                    for key, value in guidance.items()
                    if key not in {"publication", "issues"}
                }
            if args.migration_complete:
                result["migration_complete"] = _migration_complete(manifest)
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
