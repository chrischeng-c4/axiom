#!/usr/bin/env python3
"""Regenerate conformance golden files from CPython 3.12.

Usage:
    python3 tests/harness/cpython/tools/regen_golden.py [tests/cpython/fixtures]

Walks all .py files under the conformance directory, runs each with
CPython, and writes the stdout to a .expected file alongside.

Type-strict fixtures (under `tests/cpython/fixtures/type-strict/`) use the
two-golden form: each fixture produces `<name>.cpython.expected` AND
`<name>.mamba.expected` because the runtimes are EXPECTED to diverge
by design (mamba raises TypeError where CPython accepts wrong-typed
code). When `mamba` is on PATH the script also produces the mamba
golden; otherwise it warns and skips the mamba side so the cpython
golden can still be regenerated locally.
"""

import os
import shutil
import subprocess
import sys
from pathlib import Path


def _is_type_strict(p: Path) -> bool:
    """Fixture belongs to the type-strict bucket?"""
    parts = p.parts
    if "type-strict" in parts:
        return True
    # Cheap directive sniff so a stray fixture in another bucket still
    # gets two-golden form when it opts in via the directive.
    try:
        head = p.read_text(encoding="utf-8", errors="replace")[:2048]
    except OSError:
        return False
    return "# mamba-strict-type:" in head


def _run(interp: list[str], py_file: Path) -> str:
    try:
        result = subprocess.run(
            [*interp, str(py_file)],
            capture_output=True,
            text=True,
            timeout=10,
        )
        output = result.stdout
        if result.returncode != 0 and result.stderr:
            lines = result.stderr.strip().split("\n")
            exc_line = lines[-1] if lines else ""
            output += f"EXCEPTION: {exc_line}\n"
        return output
    except subprocess.TimeoutExpired:
        return "TIMEOUT\n"


def _write_if_changed(path: Path, content: str) -> bool:
    old = path.read_text() if path.exists() else None
    if old == content:
        print(f"  OK      {path}")
        return False
    path.write_text(content)
    print(f"  UPDATED {path}")
    return True


def main():
    base = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("tests/cpython/fixtures")
    if not base.exists():
        print(f"Error: {base} does not exist", file=sys.stderr)
        sys.exit(1)

    mamba_bin = os.environ.get("MAMBA") or shutil.which("mamba")
    py_files = sorted(base.rglob("*.py"))
    py_files = [f for f in py_files if f.name != "regen_golden.py"]

    updated = 0
    for py_file in py_files:
        if _is_type_strict(py_file):
            stem = py_file.with_suffix("")
            cpython_gold = stem.with_suffix(".cpython.expected")
            mamba_gold = stem.with_suffix(".mamba.expected")

            cpy_out = _run([sys.executable], py_file)
            if _write_if_changed(cpython_gold, cpy_out):
                updated += 1

            if mamba_bin:
                mb_out = _run([mamba_bin, "run"], py_file)
                if _write_if_changed(mamba_gold, mb_out):
                    updated += 1
            else:
                print(f"  SKIP_MAMBA {mamba_gold} (mamba not on PATH; "
                      f"set $MAMBA or install mamba to regenerate)")
            continue

        expected_file = py_file.with_suffix(".expected")
        output = _run([sys.executable], py_file)
        if _write_if_changed(expected_file, output):
            updated += 1

    print(f"\nDone: {len(py_files)} files, {updated} goldens updated.")


if __name__ == "__main__":
    main()
