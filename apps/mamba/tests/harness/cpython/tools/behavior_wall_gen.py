#!/usr/bin/env python3.12
"""Generate ② Behavior wall: one fixture per CPython 3.12 Lib/test method.

The Behavior wall = every Lib/test test method has a mamba fixture that runs it
via unittest against the live CPython oracle. mamba red (no unittest loader / no
test package yet) is a correct wall marker — the keep must make mamba run it.
The denominator is the installed CPython 3.12 Lib/test suite (the only authority
on what 3.12 does), so the wall is generated straight from it.

    python3.12 behavior_wall_gen.py --dry-run
    python3.12 behavior_wall_gen.py --write

Each fixture loads exactly one TestCase.method via unittest and prints PASS/FAIL
— a single Lib/test method, the finest behavior unit.
"""

from __future__ import annotations

import argparse
import ast
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from wall_gen_core import PEP723Header  # noqa: E402

MAMBA_DIR = Path(__file__).resolve().parents[4]
OUT_DIR = MAMBA_DIR / "tests" / "cpython" / "libtest"


def lib_test_dir() -> Path | None:
    try:
        out = subprocess.run(
            ["python3.12", "-c", "import test,os;print(os.path.dirname(test.__file__))"],
            capture_output=True, text=True, timeout=30).stdout.strip()
    except (OSError, subprocess.SubprocessError):
        return None
    d = Path(out)
    return d if d.is_dir() else None


def looks_like_testcase(cls: ast.ClassDef) -> bool:
    """True if the class has any test* method (matches the denominator's count)."""
    return any(
        isinstance(m, (ast.FunctionDef, ast.AsyncFunctionDef)) and m.name.startswith("test")
        for m in cls.body
    )


def _walk(body, mod, cls):
    """Yield (mod, dotted_class, method) for class-bound test* methods, recursing
    into nested classes and If-version blocks. Module-level and function-local
    test* functions are skipped — they are not unittest test cases (the former are
    not auto-collected; the latter are local helpers that merely share the name)."""
    for n in body:
        if isinstance(n, ast.ClassDef):
            sub = n.name if cls is None else f"{cls}.{n.name}"
            yield from _walk(n.body, mod, sub)
        elif isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)):
            if cls is not None and n.name.startswith("test"):
                yield mod, cls, n.name
        elif isinstance(n, ast.If):
            yield from _walk(n.body, mod, cls)
            yield from _walk(n.orelse, mod, cls)


def candidates(testdir: Path):
    """Yield (test_module, TestClass, test_method) for each class-bound Lib/test method."""
    for f in sorted(testdir.rglob("test_*.py")):
        mod = f.stem  # e.g. test_calendar (also reaches subpackage tests)
        try:
            tree = ast.parse(f.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        yield from _walk(tree.body, mod, None)


def render(mod: str, cls: str, method: str) -> tuple[str, str]:
    lib = mod[5:] if mod.startswith("test_") else mod  # calendar
    case = f"{cls.replace('.', '_')}__{method}"
    header = PEP723Header(
        bucket="libtest", lib=lib, dimension="behavior", case=case,
        subject=f"test.{mod}.{cls}.{method}", kind="behavior",
        xfail="mamba must run CPython Lib/test methods via unittest; support pending",
        mem_carveout="", source=f"Lib/test/{mod}.py", status="filled",
    ).render()
    text = header + f'''"""Behavior wall: CPython 3.12 Lib/test {mod}.{cls}.{method}.

Runs exactly one Lib/test method via unittest and prints PASS/FAIL. The live
CPython oracle is the authority; mamba must reproduce it (red until mamba runs
unittest + the test package)."""

import unittest
from test import {mod}

suite = unittest.defaultTestLoader.loadTestsFromName("{cls}.{method}", {mod})
result = unittest.TextTestRunner(verbosity=0).run(suite)
print("PASS" if result.wasSuccessful() else "FAIL")
'''
    return f"{lib}/{case}.py", text


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--module", help="only this test module (e.g. test_calendar)")
    ap.add_argument("--write", action="store_true")
    args = ap.parse_args()

    testdir = lib_test_dir()
    if testdir is None:
        print("python3.12 test package unavailable")
        return 1

    rows = list(candidates(testdir))
    if args.module:
        rows = [r for r in rows if r[0] == args.module]

    if args.dry_run or not (args.write or args.module):
        by_mod: dict[str, int] = {}
        for mod, *_ in rows:
            by_mod[mod] = by_mod.get(mod, 0) + 1
        print(f"generable behavior-wall cases: {len(rows)} across {len(by_mod)} test modules")
        for mod, n in sorted(by_mod.items(), key=lambda kv: -kv[1])[:12]:
            print(f"  {n:4d}  {mod}")
        return 0

    written = 0
    seen: set[str] = set()
    for mod, cls, method in rows:
        rel, text = render(mod, cls, method)
        if rel in seen:
            continue
        seen.add(rel)
        path = OUT_DIR / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        written += 1
    print(f"wrote {written} behavior-wall cases under {OUT_DIR}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
