#!/usr/bin/env python3
"""Inventory CPython ``Lib/test`` as the outer conformance denominator.

The mamba fixture count is intentionally not the same metric as CPython's own
test count: one fixture is one runnable contract, while CPython's suite is a
mix of regrtest modules, unittest methods, doctests, generated cases, resource
tests, and platform gates. This tool gives agents a stable reference before
claiming coverage.
"""

from __future__ import annotations

import argparse
import ast
import json
import re
import subprocess
import sys
import sysconfig
import tomllib
import warnings
from collections import Counter
from pathlib import Path


MAMBA_DIR = Path(__file__).resolve().parent.parent
FIXTURES_ROOT = MAMBA_DIR / "tests" / "cpython" / "fixtures"
MAMBA_BLOCK_RE = re.compile(
    r"^# /// script[ \t]*\n(.*?)^# ///[ \t]*$",
    re.DOTALL | re.MULTILINE,
)


def test_root(stdlib: Path | None) -> Path:
    root = stdlib or Path(sysconfig.get_paths()["stdlib"])
    test = root / "test"
    if not test.exists():
        raise SystemExit(f"CPython Lib/test not found: {test}")
    return test


def regrtest_modules() -> list[str]:
    result = subprocess.run(
        [sys.executable, "-m", "test", "--list-tests"],
        capture_output=True,
        check=True,
        text=True,
    )
    return [line.strip() for line in result.stdout.splitlines() if line.strip()]


def module_path(root: Path, module: str) -> Path | None:
    name = module[5:] if module.startswith("test.") else module
    parts = name.split(".")
    as_file = root.joinpath(*parts).with_suffix(".py")
    if as_file.exists():
        return as_file
    as_pkg = root.joinpath(*parts) / "__init__.py"
    if as_pkg.exists():
        return as_pkg
    return None


def static_test_defs(path: Path) -> int:
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", SyntaxWarning)
        tree = ast.parse(path.read_text(encoding="utf-8", errors="replace"))
    return sum(
        isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        and node.name.startswith("test")
        for node in ast.walk(tree)
    )


def cpython_key(module: str) -> str:
    name = module[5:] if module.startswith("test.") else module
    top = name.split(".")[0]
    if top.startswith("test_"):
        top = top[5:]
    aliases = {
        "__all__": "all",
        "_locale": "locale",
        "_opcode": "opcode",
        "_osx_support": "osx_support",
        "httplib": "http",
        "minidom": "xml_dom_minidom",
        "robotparser": "urllib_robotparser",
        "sax": "xml_sax",
        "urlparse": "urllib_parse",
    }
    return aliases.get(top, top).replace(".", "_").replace("-", "_").lower()


def fixture_libs() -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    if not FIXTURES_ROOT.exists():
        return out
    for bucket_dir in FIXTURES_ROOT.iterdir():
        if not bucket_dir.is_dir():
            continue
        for lib_dir in bucket_dir.iterdir():
            if lib_dir.is_dir():
                key = lib_dir.name.replace("-", "_").lower()
                out.setdefault(key, []).append(bucket_dir.name)
    return {key: sorted(value) for key, value in out.items()}


def normalize_fixture_source(source: str) -> str:
    if source.startswith("Lib/test/"):
        return source.removeprefix("Lib/test/")
    if source.startswith("test/"):
        return source.removeprefix("test/")
    return source


def fixture_case_inventory() -> dict:
    cases = []
    migrated = []
    legacy = []
    by_dimension = Counter()
    by_bucket = Counter()
    by_source_path = Counter()
    invalid_metadata = []

    if not FIXTURES_ROOT.exists():
        return {
            "total_case_files": 0,
            "migrated_case_files": 0,
            "legacy_case_files": 0,
            "invalid_metadata_files": [],
            "by_dimension": {},
            "by_bucket": {},
        }

    for path in sorted(FIXTURES_ROOT.rglob("*.py")):
        if path.name.endswith("_stub.py") or "_invalid" in path.parts:
            continue
        rel = str(path.relative_to(FIXTURES_ROOT))
        cases.append(rel)
        text = path.read_text(encoding="utf-8", errors="replace")
        match = MAMBA_BLOCK_RE.search(text)
        if not match:
            legacy.append(rel)
            continue
        block_lines = []
        for line in match.group(1).splitlines():
            if line.startswith("# "):
                block_lines.append(line[2:])
            elif line == "#":
                block_lines.append("")
        try:
            meta = tomllib.loads("\n".join(block_lines)).get("tool", {}).get("mamba")
        except tomllib.TOMLDecodeError as exc:
            invalid_metadata.append(f"{rel}: {exc}")
            continue
        if not isinstance(meta, dict):
            legacy.append(rel)
            continue
        migrated.append(rel)
        by_dimension[str(meta.get("dimension", "<missing>"))] += 1
        by_bucket[str(meta.get("bucket", "<missing>"))] += 1
        source = meta.get("source")
        if isinstance(source, str) and source:
            by_source_path[normalize_fixture_source(source)] += 1

    return {
        "total_case_files": len(cases),
        "migrated_case_files": len(migrated),
        "legacy_case_files": len(legacy),
        "invalid_metadata_files": invalid_metadata,
        "by_dimension": dict(sorted(by_dimension.items())),
        "by_bucket": dict(sorted(by_bucket.items())),
        "by_source_path": dict(sorted(by_source_path.items())),
    }


def all_test_py_files(root: Path) -> list[Path]:
    return [
        path
        for path in root.rglob("*.py")
        if "__pycache__" not in path.parts
        and (path.name.startswith("test") or path.parent.name.startswith("test"))
    ]


def build_inventory(root: Path, *, top: int) -> dict:
    modules = regrtest_modules()
    libs = fixture_libs()
    mamba_cases = fixture_case_inventory()
    source_paths = mamba_cases["by_source_path"]
    rows = []
    for module in modules:
        path = module_path(root, module)
        tests = static_test_defs(path) if path else 0
        key = cpython_key(module)
        rel_path = str(path.relative_to(root)) if path else None
        source_count = source_paths.get(rel_path, 0) if rel_path else 0
        exact_lib_buckets = libs.get(key, [])
        rows.append(
            {
                "module": module,
                "key": key,
                "path": rel_path,
                "static_test_defs": tests,
                "fixture_buckets_exact_lib_match": exact_lib_buckets,
                "fixture_source_match_count": source_count,
                "fixture_match": bool(exact_lib_buckets or source_count),
            }
        )

    all_files = all_test_py_files(root)
    all_defs = 0
    parse_errors: list[str] = []
    for path in all_files:
        try:
            all_defs += static_test_defs(path)
        except SyntaxError as exc:
            parse_errors.append(f"{path.relative_to(root)}: {exc}")

    exact_missing = [row for row in rows if not row["fixture_buckets_exact_lib_match"]]
    exact_mapped = [row for row in rows if row["fixture_buckets_exact_lib_match"]]
    source_mapped = [row for row in rows if row["fixture_source_match_count"]]
    fixture_missing = [row for row in rows if not row["fixture_match"]]
    return {
        "cpython_test_root": str(root),
        "python": sys.version.split()[0],
        "mamba_fixture_cases": mamba_cases,
        "cpython_test_case_candidates": {
            "test_py_files": len(all_files),
            "regrtest_modules": len(rows),
            "static_test_defs_in_regrtest_modules": sum(
                row["static_test_defs"] for row in rows
            ),
            "static_test_defs_in_all_test_files": all_defs,
        },
        "cpython_regrtest_modules": len(rows),
        "static_test_defs_in_regrtest_modules": sum(
            row["static_test_defs"] for row in rows
        ),
        "static_test_defs_in_all_test_files": all_defs,
        "exact_fixture_lib_matches": len(exact_mapped),
        "no_exact_fixture_lib_match": len(exact_missing),
        "source_fixture_matches": len(source_mapped),
        "no_fixture_lib_or_source_match": len(fixture_missing),
        "top_no_exact_fixture_lib_match": sorted(
            exact_missing, key=lambda row: row["static_test_defs"], reverse=True
        )[:top],
        "top_no_fixture_lib_or_source_match": sorted(
            fixture_missing, key=lambda row: row["static_test_defs"], reverse=True
        )[:top],
        "top_exact_fixture_lib_match": sorted(
            exact_mapped, key=lambda row: row["static_test_defs"], reverse=True
        )[:top],
        "parse_errors": parse_errors,
    }


def print_text(data: dict) -> None:
    mamba = data["mamba_fixture_cases"]
    cpython = data["cpython_test_case_candidates"]
    print(f"CPython test root: {data['cpython_test_root']}")
    print(f"Python: {data['python']}")
    print(f"mamba fixture cases: {mamba['total_case_files']}")
    print(f"  migrated metadata cases: {mamba['migrated_case_files']}")
    print(f"  legacy cases: {mamba['legacy_case_files']}")
    print(f"  migrated by dimension: {mamba['by_dimension']}")
    print(f"  migrated by bucket: {mamba['by_bucket']}")
    print(f"CPython test .py files: {cpython['test_py_files']}")
    print(f"CPython regrtest modules: {cpython['regrtest_modules']}")
    print(
        "static test defs in regrtest modules: "
        f"{cpython['static_test_defs_in_regrtest_modules']}"
    )
    print(
        "static test defs in all test files: "
        f"{cpython['static_test_defs_in_all_test_files']}"
    )
    print(f"exact fixture-lib matches: {data['exact_fixture_lib_matches']}")
    print(f"no exact fixture-lib match: {data['no_exact_fixture_lib_match']}")
    print(f"source metadata matches: {data['source_fixture_matches']}")
    print(
        "no fixture-lib or source match: "
        f"{data['no_fixture_lib_or_source_match']}"
    )
    print("\nTop CPython modules without fixture-lib or source metadata match:")
    for row in data["top_no_fixture_lib_or_source_match"]:
        print(
            f"  {row['module']:<44} "
            f"defs={row['static_test_defs']:<4} key={row['key']}"
        )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--stdlib",
        type=Path,
        help="Path to CPython Lib directory; defaults to this interpreter.",
    )
    parser.add_argument("--top", type=int, default=30)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)

    data = build_inventory(test_root(args.stdlib), top=args.top)
    if args.json:
        print(json.dumps(data, indent=2))
    else:
        print_text(data)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
