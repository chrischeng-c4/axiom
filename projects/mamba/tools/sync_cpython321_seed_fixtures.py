#!/usr/bin/env python3
"""Promote CPython 3.12.1-style seed contracts into fixture manifests.

The seed corpus under tests/cpython/config/seeds is an external denominator:
small, assertion-heavy CPython contracts collected from core language,
builtins, and stdlib behavior. This tool turns the pass/spec seed files into
regular PEP-723 fixtures under tests/cpython/fixtures so the CPython oracle can
prove the fixtures themselves before mamba runtime work starts.

Generated fixtures are mamba-xfail by default. They define the target contract
now without making today's runtime gate red; runtime work promotes individual
cases by removing the xfail once mamba catches up.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parent
CPYTHON_DIR = MAMBA_DIR / "tests" / "cpython"
SEEDS_DIR = CPYTHON_DIR / "config" / "seeds"
MANIFESTS_DIR = CPYTHON_DIR / "config" / "manifests"
FIXTURE_GEN = TOOLS_DIR / "fixture_gen.py"

SKIP_STEMS = {
    "test_canary_3p",
    "test_click",
    "test_httpx",
    "test_pluggy",
    "test_pytest",
    "test_pyyaml",
    "test_requests",
    "test_urllib3",
}

BUILTIN_STEM_HINTS = (
    "any_all",
    "builtin",
    "builtins",
    "bytearray",
    "bytes",
    "complex",
    "dict",
    "enumerate",
    "filter",
    "frozenset",
    "hash_id",
    "int_float",
    "iter",
    "list",
    "map",
    "memoryview",
    "min_max",
    "next",
    "range",
    "reversed",
    "round",
    "set",
    "slice",
    "sorted",
    "str_",
    "string_",
    "sum",
    "tuple",
    "zip",
)


def toml_str(value: str) -> str:
    return '"' + value.replace("\\", "\\\\").replace('"', '\\"') + '"'


def toml_multiline(value: str) -> str:
    if "'''" in value:
        value = value.replace("'''", "\"\"\"")
    return "'''\n" + value.rstrip() + "\n'''"


def snake_subject(stem: str) -> str:
    return re.sub(r"[^a-zA-Z0-9_]+", "_", stem).strip("_").lower()


def classify(path: Path) -> tuple[str, str]:
    stem = path.stem
    if stem.startswith("lang_") or stem.startswith("test_lang_"):
        return "core", "cpython321_core_lang"
    if stem.startswith("test_generator") or stem in {
        "test_class_ops",
        "test_custom_exception_attribute_ops",
        "test_dunder_walrus_match_decorator_value_ops",
        "test_exception_ops",
        "test_exceptions_bytes_iter_file_traceback_value_ops",
        "test_iterators_builtins_generators_value_ops",
        "test_unpacking_ops",
    }:
        return "core", "cpython321_core_lang"
    if any(hint in stem for hint in BUILTIN_STEM_HINTS):
        return "builtin-libs", "cpython321_builtins"
    return "std-libs", "cpython321_stdlib"


def source_paths() -> list[tuple[str, Path]]:
    paths: list[tuple[str, Path]] = []
    for subdir in ("pass", "spec"):
        root = SEEDS_DIR / subdir
        for path in sorted(root.glob("*.py")):
            if path.stem in SKIP_STEMS:
                continue
            paths.append((subdir, path))
    return paths


def render_manifest(bucket: str, lib: str, rows: list[tuple[str, Path]]) -> str:
    lines = [
        f"bucket = {toml_str(bucket)}",
        f"lib = {toml_str(lib)}",
        "",
    ]
    for subdir, path in rows:
        stem = path.stem
        raw = path.read_text(encoding="utf-8")
        reason = f"CPython 3.12 seed {subdir}; mamba promotion pending"
        body = f"# mamba-xfail: {reason}\n" + raw.rstrip() + "\n"
        lines.extend(
            [
                "[[case]]",
                'dimension = "real_world"',
                f"case = {toml_str(stem)}",
                f"subject = {toml_str('cpython321.' + snake_subject(stem))}",
                'kind = "semantic"',
                f"xfail = {toml_str(reason)}",
                f"source = {toml_str(str(path.relative_to(MAMBA_DIR)))}",
                f"intent = {toml_str('execute CPython 3.12 seed ' + stem)}",
                "body = " + toml_multiline(body),
                "",
            ]
        )
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--no-generate", action="store_true")
    args = parser.parse_args(argv)

    grouped: dict[tuple[str, str], list[tuple[str, Path]]] = {}
    for subdir, path in source_paths():
        bucket, lib = classify(path)
        grouped.setdefault((bucket, lib), []).append((subdir, path))

    manifest_paths: list[Path] = []
    for (bucket, lib), rows in sorted(grouped.items()):
        out = MANIFESTS_DIR / bucket / f"{lib}.toml"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(render_manifest(bucket, lib, rows), encoding="utf-8")
        manifest_paths.append(out)
        print(f"wrote {out.relative_to(MAMBA_DIR)} ({len(rows)} cases)")

    if args.no_generate:
        return 0

    for manifest in manifest_paths:
        subprocess.run([sys.executable, str(FIXTURE_GEN), str(manifest)], check=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
