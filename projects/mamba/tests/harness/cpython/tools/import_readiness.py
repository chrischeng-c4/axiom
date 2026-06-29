#!/usr/bin/env python3.12
"""Import/package/module-system readiness accounting for #708."""

from __future__ import annotations

import argparse
import json
import re
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

from platform_readiness import (
    CPYTHON_DIR,
    EXIT_NOT_READY,
    Fixture,
    RESULTS_DB,
    classify_metadata,
    detect_dimension,
    detect_skip_reason,
    fixture_case,
    load_mamba_rows,
    repo_rel,
    runtime_state,
    tool_mamba,
)


IMPORT_SCOPES: dict[str, tuple[str, ...]] = {
    "core_import": (
        "core/import",
        "core/import_cache",
        "core/imports",
        "core/circular_import",
        "core/relative_import",
        "core/star_import",
        "core/grammar/test_import",
        "core/language/imports",
    ),
    "stdlib_import_api": (
        "_frozen_importlib",
        "_frozen_importlib_external",
        "importlib",
        "importlib__abc",
        "importlib_abc",
        "importlib_readers",
        "importlib_util",
        "modulefinder",
        "pkgutil",
        "threaded_import",
        "zipimport",
        "zipimport_support",
    ),
    "package_semantics": (
        "namespace_pkgs",
        "pkg_import",
        "runpy",
        "site",
    ),
    "module_objects": (
        "code_module",
        "module",
    ),
    "type_import_surface": (
        "_frozen_importlib",
        "_frozen_importlib_external",
        "importlib",
        "importlib__abc",
        "importlib_abc",
        "importlib_readers",
        "importlib_util",
        "modulefinder",
        "pkgutil",
        "zipimport",
    ),
    "resources_metadata": (
        "importlib_metadata",
        "importlib_metadata__meta",
        "importlib_metadata_diagnose",
        "importlib_resources",
        "importlib_resources__common",
        "importlib_resources__functional",
        "importlib_resources_abc",
        "importlib_resources_readers",
        "importlib_resources_simple",
    ),
}

STD_LIB_TO_SCOPE: dict[str, str] = {}
for scope, keys in IMPORT_SCOPES.items():
    if scope in {"core_import", "type_import_surface"}:
        continue
    for key in keys:
        STD_LIB_TO_SCOPE[key] = scope

SEMANTIC_CLASS_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("module_cache_reload", re.compile(r"(sys\.modules|reload|cache|invalidate_caches|import_cache)", re.I)),
    ("namespace_packages", re.compile(r"(namespace|portion|namespace_pkg)", re.I)),
    ("relative_imports", re.compile(r"(relative_import|from \.|level|parent package)", re.I)),
    ("star_imports", re.compile(r"(star_import|import \*)", re.I)),
    ("circular_imports", re.compile(r"(circular_import|partially initialized)", re.I)),
    ("import_hooks", re.compile(r"(meta_path|path_hooks|finder|loader|module_spec|__spec__|__loader__|importlib)", re.I)),
    ("resources_metadata", re.compile(r"(importlib_resources|importlib_metadata|resources|metadata|entry_points|files)", re.I)),
    ("zipimport", re.compile(r"(zipimport|zipdata|zipimporter)", re.I)),
    ("site_venv_paths", re.compile(r"(site-packages|venv|virtualenv|sitecustomize|user site|pth)", re.I)),
    ("package_execution", re.compile(r"(pkgutil|runpy|__main__|__package__|pkg_import|namespace_pkgs)", re.I)),
    ("module_objects", re.compile(r"(code_module|modulefinder|\bmodule\b|module object|module attribute|module repr)", re.I)),
)

REQUIRED_SEMANTIC_CLASSES = (
    "module_cache_reload",
    "namespace_packages",
    "relative_imports",
    "star_imports",
    "circular_imports",
    "import_hooks",
    "resources_metadata",
    "zipimport",
    "site_venv_paths",
    "package_execution",
    "module_objects",
)


def scope_for_stdlib(lib: str) -> str | None:
    if lib in STD_LIB_TO_SCOPE:
        return STD_LIB_TO_SCOPE[lib]
    if lib.startswith("importlib_metadata"):
        return "resources_metadata"
    if lib.startswith("importlib_resources"):
        return "resources_metadata"
    if lib.startswith("importlib") or lib.startswith("_frozen_importlib"):
        return "stdlib_import_api"
    return None


def type_import_target(parts: tuple[str, ...]) -> tuple[str, str] | None:
    if len(parts) >= 3 and parts[0] == "type" and parts[1] == "std-libs":
        lib = parts[2]
        if lib.startswith("importlib_metadata") or lib.startswith("importlib_resources"):
            return "resources_metadata", lib
        if lib in IMPORT_SCOPES["type_import_surface"]:
            return "type_import_surface", lib
        if lib.startswith("importlib") or lib.startswith("_frozen_importlib"):
            return "type_import_surface", lib
    return None


def core_import_target(parts: tuple[str, ...]) -> tuple[str, str] | None:
    if "core" not in parts:
        return None
    core_idx = parts.index("core")
    tail = "/".join(parts[core_idx:])
    for key in IMPORT_SCOPES["core_import"]:
        if tail.startswith(key):
            return "core_import", key.replace("/", "_")
    return None


def detect_import_target(path: Path) -> tuple[str, str] | None:
    rel_parts = path.relative_to(CPYTHON_DIR).parts
    if target := type_import_target(rel_parts):
        return target
    if target := core_import_target(rel_parts):
        return target
    for index, part in enumerate(rel_parts[:-1]):
        if part == "std-libs":
            lib = rel_parts[index + 1]
            scope = scope_for_stdlib(lib)
            if scope:
                return scope, lib
    return None


def discover_fixtures() -> tuple[list[Fixture], list[dict[str, str]]]:
    fixtures: list[Fixture] = []
    parse_errors: list[dict[str, str]] = []
    for path in sorted(CPYTHON_DIR.rglob("*.py")):
        rel_parts = path.relative_to(CPYTHON_DIR).parts
        if path.name.endswith("_stub.py"):
            continue
        if "_invalid" in rel_parts:
            continue
        if "bench" in rel_parts:
            continue
        target = detect_import_target(path)
        if target is None:
            continue
        scope, lib = target
        text = path.read_text(encoding="utf-8", errors="replace")
        meta, parse_error = tool_mamba(text)
        rel = repo_rel(path)
        if parse_error and "_regression" not in rel_parts:
            parse_errors.append({"path": rel, "error": parse_error})
        if parse_error and "_regression" in rel_parts:
            meta = {
                "dimension": "_regression",
                "case": path.stem,
                "xfail": "#708 legacy _regression import fixture lacks PEP 723 metadata",
            }
            parse_error = "legacy_regression_unaccounted"
        xfail = meta.get("xfail", "")
        fixtures.append(
            Fixture(
                path=path,
                rel=rel,
                scope=scope,
                lib=lib,
                dimension=detect_dimension(path, meta),
                case=fixture_case(path, meta),
                xfail=xfail if isinstance(xfail, str) else "",
                skip_reason=detect_skip_reason(text),
                parse_error=parse_error,
            )
        )
    return fixtures, parse_errors


def semantic_classes(fixture: Fixture) -> list[str]:
    haystack = "\n".join([fixture.rel, fixture.case, fixture.lib, fixture.scope])
    classes = [
        name for name, pattern in SEMANTIC_CLASS_PATTERNS if pattern.search(haystack)
    ]
    if classes:
        return classes
    if fixture.scope == "core_import":
        return ["import_statement_basics"]
    return ["import_surface_misc"]


def build_report(show: int, db: Path) -> dict[str, Any]:
    fixtures, parse_errors = discover_fixtures()
    rows = load_mamba_rows(fixtures, db)
    by_scope: dict[str, Counter[str]] = defaultdict(Counter)
    by_lib: dict[str, Counter[str]] = defaultdict(Counter)
    by_dimension: dict[str, Counter[str]] = defaultdict(Counter)
    by_semantic_class: dict[str, Counter[str]] = defaultdict(Counter)
    metadata_counts: Counter[str] = Counter()
    runtime_counts: Counter[str] = Counter()
    gaps: list[dict[str, Any]] = []

    for fixture in fixtures:
        metadata_state, reason, owners = classify_metadata(fixture)
        runtime = runtime_state(rows.get(fixture.rel))
        classes = semantic_classes(fixture)
        metadata_counts[metadata_state] += 1
        if fixture.parse_error == "legacy_regression_unaccounted":
            metadata_counts[metadata_state] -= 1
            metadata_state = "legacy_regression_unaccounted"
            reason = "#708 legacy _regression import fixture lacks PEP 723 metadata"
            owners = ["#708"]
            metadata_counts[metadata_state] += 1
        runtime_counts[runtime] += 1
        for counter in (by_scope[fixture.scope], by_lib[fixture.lib], by_dimension[fixture.dimension]):
            counter["fixtures"] += 1
            counter[metadata_state] += 1
            counter[runtime] += 1
        for semantic_class in classes:
            by_semantic_class[semantic_class]["fixtures"] += 1
            by_semantic_class[semantic_class][metadata_state] += 1
            by_semantic_class[semantic_class][runtime] += 1

        if metadata_state != "pass_candidate":
            gaps.append(
                {
                    "kind": metadata_state,
                    "path": fixture.rel,
                    "scope": fixture.scope,
                    "lib": fixture.lib,
                    "dimension": fixture.dimension,
                    "case": fixture.case,
                    "semantic_classes": classes,
                    "reason": reason,
                    "owner_refs": owners,
                    "owned": bool(owners),
                }
            )
        if runtime != "runtime_ok":
            gaps.append(
                {
                    "kind": runtime,
                    "path": fixture.rel,
                    "scope": fixture.scope,
                    "lib": fixture.lib,
                    "dimension": fixture.dimension,
                    "case": fixture.case,
                    "semantic_classes": classes,
                    "reason": "no current mamba results-store row for this fixture"
                    if runtime == "unmeasured"
                    else runtime,
                    "owner_refs": [],
                    "owned": False,
                }
            )

    missing_scopes = [
        {"kind": "missing_import_scope", "scope": scope}
        for scope in IMPORT_SCOPES
        if by_scope[scope]["fixtures"] == 0
    ]
    missing_semantic_classes = [
        {"kind": "missing_semantic_class", "semantic_class": name}
        for name in REQUIRED_SEMANTIC_CLASSES
        if by_semantic_class[name]["fixtures"] == 0
    ]
    unowned_gap_count = sum(1 for gap in gaps if not gap.get("owned"))
    runtime_problem_count = sum(
        runtime_counts[state]
        for state in ("unmeasured", "runtime_fail", "runtime_timeout", "runtime_crash")
    )
    metadata_problem_count = sum(
        metadata_counts[state]
        for state in (
            "metadata_error",
            "promotion_pending",
            "runtime_failure_debt",
            "sandbox_denied",
            "unsupported_platform",
            "legacy_regression_unaccounted",
        )
    )
    ready = (
        not missing_scopes
        and not missing_semantic_classes
        and not parse_errors
        and metadata_problem_count == 0
        and runtime_problem_count == 0
        and unowned_gap_count == 0
    )
    blockers = [
        *missing_scopes,
        *missing_semantic_classes,
        *({"kind": "metadata_parse_error", **item} for item in parse_errors[:show]),
        *[gap for gap in gaps if not gap.get("owned")][:show],
    ]
    return {
        "schema_version": 1,
        "owner_issue": "#708",
        "ready": ready,
        "status": "green" if ready else "red",
        "counts": {
            "fixtures": len(fixtures),
            "scopes": len(IMPORT_SCOPES),
            "missing_scopes": len(missing_scopes),
            "semantic_classes": len(by_semantic_class),
            "missing_semantic_classes": len(missing_semantic_classes),
            "parse_errors": len(parse_errors),
            "metadata_problem_count": metadata_problem_count,
            "runtime_problem_count": runtime_problem_count,
            "unowned_gap_count": unowned_gap_count,
            "pass_candidate": metadata_counts["pass_candidate"],
            "promotion_pending": metadata_counts["promotion_pending"],
            "runtime_failure_debt": metadata_counts["runtime_failure_debt"],
            "legacy_regression_unaccounted": metadata_counts[
                "legacy_regression_unaccounted"
            ],
            "sandbox_denied": metadata_counts["sandbox_denied"],
            "unsupported_platform": metadata_counts["unsupported_platform"],
            "metadata_error": metadata_counts["metadata_error"],
            "runtime_ok": runtime_counts["runtime_ok"],
            "runtime_fail": runtime_counts["runtime_fail"],
            "runtime_timeout": runtime_counts["runtime_timeout"],
            "runtime_crash": runtime_counts["runtime_crash"],
            "unmeasured": runtime_counts["unmeasured"],
        },
        "target_scopes": IMPORT_SCOPES,
        "required_semantic_classes": REQUIRED_SEMANTIC_CLASSES,
        "by_scope": {
            key: dict(sorted(value.items())) for key, value in sorted(by_scope.items())
        },
        "by_lib": {
            key: dict(sorted(value.items())) for key, value in sorted(by_lib.items())
        },
        "by_dimension": {
            key: dict(sorted(value.items()))
            for key, value in sorted(by_dimension.items())
        },
        "by_semantic_class": {
            key: dict(sorted(value.items()))
            for key, value in sorted(by_semantic_class.items())
        },
        "blocker_count": len(missing_scopes)
        + len(missing_semantic_classes)
        + len(parse_errors)
        + unowned_gap_count,
        "blockers": blockers,
        "evidence_commands": [
            "python3.12 projects/mamba/tests/harness/cpython/tools/import_readiness.py --json",
            "python3.12 projects/mamba/tests/harness/cpython/tools/results_store.py collect --bucket behavior --limit 20",
            "python3.12 projects/mamba/tests/harness/cpython/tools/verify_cpython_oracle.py --ready-only --bucket behavior",
            "target/debug/mamba run projects/mamba/tests/cpython/behavior/core/import/import_tests__test_import_name_binding.py",
        ],
    }


def print_human(report: dict[str, Any]) -> None:
    print(f"import readiness: {report['status']}")
    counts = report["counts"]
    print(
        "  fixtures={fixtures} pass_candidate={pass_candidate} "
        "unmeasured={unmeasured} promotion_pending={promotion_pending} "
            "runtime_failure_debt={runtime_failure_debt} legacy_regression_unaccounted={legacy_regression_unaccounted} "
            "missing_semantic_classes={missing_semantic_classes}".format(
            **counts
        )
    )
    for scope, scope_counts in report["by_scope"].items():
        print(f"- {scope}: {scope_counts}")
    for blocker in report["blockers"][:5]:
        label = blocker.get("path") or blocker.get("semantic_class") or blocker.get("scope")
        print(f"  blocker: {blocker['kind']} {label}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--show", type=int, default=10)
    parser.add_argument("--db", type=Path, default=RESULTS_DB)
    args = parser.parse_args(argv)

    report = build_report(args.show, args.db)
    if args.json:
        print(json.dumps(report, sort_keys=True))
    else:
        print_human(report)
    return 0 if report["ready"] else EXIT_NOT_READY


if __name__ == "__main__":
    raise SystemExit(main())
