#!/usr/bin/env python3
"""Bulk-port CPython regrtest modules into one-case fixture files.

This is intentionally conservative: a case is written only when the rendered
fixture exits 0 under CPython. Unsupported or CPython-failing cases are counted
but not emitted, so the repository never gains a broken oracle fixture.
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path

import cpython_port
import cpython_regrtest_inventory as inventory


FIXTURES_ROOT = cpython_port.FIXTURES_ROOT
CORE_KEYS = {
    "ann_module",
    "annotations",
    "asyncgen",
    "compile",
    "coroutines",
    "decorators",
    "descr",
    "dictcomps",
    "exception_group",
    "exceptions",
    "fstring",
    "funcattrs",
    "future",
    "generators",
    "grammar",
    "import",
    "listcomps",
    "opcode",
    "patma",
    "peepholer",
    "scopes",
    "syntax",
    "sys_settrace",
    "with",
    "yield_from",
}
BUILTIN_KEYS = {
    "builtin",
    "bool",
    "bytes",
    "bytearray",
    "complex",
    "dict",
    "enumerate",
    "float",
    "frozenset",
    "int",
    "iter",
    "list",
    "memoryview",
    "range",
    "set",
    "slice",
    "str",
    "tuple",
    "type",
    "zip",
}
KEY_ALIASES = {
    "builtin": "builtins",
    "set": "set_methods",
    "list": "list_methods",
    "dict": "dict_methods",
    "tuple": "tuple_methods",
    "str": "string_methods",
    "bytes": "bytes",
    "bytearray": "bytes",
    "float": "float_methods",
    "int": "int_methods",
    "patma": "pattern_matching",
    "sys_settrace": "sys_settrace",
}


@dataclass
class ModuleResult:
    module: str
    source: str
    bucket: str
    lib: str
    extracted: int = 0
    portable: int = 0
    skipped: int = 0
    written: int = 0
    invalid: int = 0
    collisions: int = 0
    error: str = ""


def classify(module: str) -> tuple[str, str]:
    key = inventory.cpython_key(module)
    lib = KEY_ALIASES.get(key, key)
    if key in BUILTIN_KEYS:
        return "builtin-libs", lib
    if key in CORE_KEYS:
        return "core", lib
    return "std-libs", lib


def source_label_for(root: Path, path: Path) -> str:
    return str(path.relative_to(root))


def already_covered_sources() -> set[str]:
    cases = inventory.fixture_case_inventory()
    return set(cases.get("by_source_path", {}))


def port_module(
    *,
    module: str,
    path: Path,
    root: Path,
    timeout: float,
    overwrite: bool,
    xfail_reason: str,
) -> ModuleResult:
    bucket, lib = classify(module)
    result = ModuleResult(
        module=module,
        source=source_label_for(root, path),
        bucket=bucket,
        lib=lib,
    )
    try:
        src = cpython_port.load_source(path)
        methods = cpython_port.extract(src)
    except Exception as exc:
        result.error = f"extract: {type(exc).__name__}: {exc}"
        return result

    result.extracted = len(methods)
    portable = [method for method in methods if not method.skip]
    result.portable = len(portable)
    result.skipped = len(methods) - len(portable)
    if not portable:
        return result

    dest = FIXTURES_ROOT / bucket / lib / "behavior"
    dest.mkdir(parents=True, exist_ok=True)
    for method in portable:
        filename = cpython_port.safe_filename(method)
        fixture_path = dest / filename
        if fixture_path.exists() and not overwrite:
            result.collisions += 1
            continue
        try:
            content = cpython_port.render_fixture(
                src,
                method,
                bucket=bucket,
                lib=lib,
                dimension="behavior",
                xfail=xfail_reason,
            )
            fixture_path.write_text(content)
            ok, _err = cpython_port.cpython_passes_with_timeout(fixture_path, timeout)
        except Exception as exc:
            result.invalid += 1
            result.error = result.error or f"render/run: {type(exc).__name__}: {exc}"
            if fixture_path.exists():
                fixture_path.unlink()
            continue
        if ok:
            result.written += 1
        else:
            result.invalid += 1
            fixture_path.unlink()
    return result


def selected_modules(root: Path, include_covered: bool, names: set[str]) -> list[tuple[str, Path]]:
    covered = already_covered_sources()
    out: list[tuple[str, Path]] = []
    for module in inventory.regrtest_modules():
        if names and module not in names and module.removeprefix("test.") not in names:
            continue
        path = inventory.module_path(root, module)
        if path is None:
            continue
        rel = source_label_for(root, path)
        if not include_covered and rel in covered:
            continue
        out.append((module, path))
    return out


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--stdlib",
        type=Path,
        help="Path to CPython Lib directory; defaults to this interpreter.",
    )
    parser.add_argument(
        "--module",
        action="append",
        default=[],
        help="Only port this regrtest module; may be repeated.",
    )
    parser.add_argument("--limit", type=int, help="Stop after N candidate modules.")
    parser.add_argument("--include-covered", action="store_true")
    parser.add_argument("--overwrite", action="store_true")
    parser.add_argument("--timeout", type=float, default=10.0)
    parser.add_argument(
        "--report",
        type=Path,
        default=Path("/private/tmp/mamba-bulk-cpython-port-report.json"),
    )
    parser.add_argument(
        "--xfail-reason",
        default="auto-ported CPython test; mamba promotion pending",
    )
    args = parser.parse_args(argv)

    root = inventory.test_root(args.stdlib)
    names = set(args.module)
    modules = selected_modules(root, args.include_covered, names)
    if args.limit is not None:
        modules = modules[: args.limit]

    print(f"CPython test root: {root}", flush=True)
    print(f"candidate modules: {len(modules)}", flush=True)

    results: list[ModuleResult] = []
    started = time.monotonic()
    for index, (module, path) in enumerate(modules, start=1):
        before = time.monotonic()
        row = port_module(
            module=module,
            path=path,
            root=root,
            timeout=args.timeout,
            overwrite=args.overwrite,
            xfail_reason=args.xfail_reason,
        )
        results.append(row)
        elapsed = time.monotonic() - before
        print(
            f"[{index:03}/{len(modules):03}] {module:<38} "
            f"wrote={row.written:<4} invalid={row.invalid:<4} "
            f"skip={row.skipped:<4} coll={row.collisions:<3} "
            f"{elapsed:5.1f}s"
            + (f" ERROR {row.error}" if row.error else ""),
            flush=True,
        )

    summary = {
        "root": str(root),
        "elapsed_seconds": round(time.monotonic() - started, 3),
        "modules": len(results),
        "written": sum(row.written for row in results),
        "invalid": sum(row.invalid for row in results),
        "skipped": sum(row.skipped for row in results),
        "collisions": sum(row.collisions for row in results),
        "errors": [asdict(row) for row in results if row.error],
        "results": [asdict(row) for row in results],
    }
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(summary, indent=2, sort_keys=True))
    print(
        "SUMMARY "
        f"modules={summary['modules']} written={summary['written']} "
        f"invalid={summary['invalid']} skipped={summary['skipped']} "
        f"collisions={summary['collisions']} report={args.report}",
        flush=True,
    )
    return 1 if summary["errors"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
