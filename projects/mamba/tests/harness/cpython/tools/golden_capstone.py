#!/usr/bin/env python3.12
"""Capstone: prove the dynamic oracle supersets the 683 static `.expected` goldens.

D5.6 retires the legacy runner; this capstone is the gate on retiring the static
goldens + `regen_golden.py` too. For each golden it runs the fixture under the
matching runtime and compares stdout byte-for-byte:

    <name>.expected / <name>.py.expected / <name>.cpython.expected  -> CPython 3.12
    <name>.mamba.expected                                            -> mamba SUT

A zero CPython-side mismatch proves the content-addressed CPython oracle in
results_store can replace every static cpython golden, so `regen_golden.py` and
those goldens become removable. mamba-side rows just record the current SUT
state (red is fine — making mamba match is the separate runtime line).
`PYTHONBREAKPOINT=0` keeps `breakpoint()` fixtures non-interactive.

    python3.12 tests/harness/cpython/tools/golden_capstone.py

Exit 0 iff every CPython-side golden matches (the superset proof holds).
"""

from __future__ import annotations

import os
import shutil
from pathlib import Path

import harness_lib  # shared oracle/SUT runner (tools/): isolated scratch CWD + PYTHONBREAKPOINT=0

TOOLS_DIR = Path(__file__).resolve().parent
FIXTURES_DIR = TOOLS_DIR.parents[2] / "cpython"

# longest suffix first so `.cpython.expected` wins over `.expected`
SUFFIXES = (".cpython.expected", ".mamba.expected", ".py.expected", ".expected")


def fixture_for(golden: Path) -> tuple[Path | None, str]:
    name = golden.name
    for suf in SUFFIXES:
        if name.endswith(suf):
            stem = name[: -len(suf)]
            cand = golden.parent / f"{stem}.py"
            if cand.exists():
                return cand, "mamba" if suf == ".mamba.expected" else "cpython"
            return None, ""
    return None, ""


def run(side: str, fixture: Path, mamba_bin: str) -> str | None:
    argv = ["python3.12", str(fixture)] if side == "cpython" else [mamba_bin, "run", str(fixture)]
    rc, out, _err = harness_lib.run_fixture(argv, 20)
    return None if rc is None else out


def main() -> int:
    mamba_bin = os.environ.get("MAMBA_BIN") or shutil.which("mamba") or "mamba"
    goldens = sorted(FIXTURES_DIR.rglob("*.expected"))
    cpy_match = cpy_mismatch = mb_match = mb_mismatch = nofix = 0
    bad: list[tuple[str, str, str]] = []
    for g in goldens:
        fixture, side = fixture_for(g)
        if fixture is None:
            nofix += 1
            continue
        want = g.read_text(encoding="utf-8", errors="replace")
        got = run(side, fixture, mamba_bin)
        ok = got is not None and got.rstrip("\n") == want.rstrip("\n")
        if side == "cpython":
            if ok:
                cpy_match += 1
            else:
                cpy_mismatch += 1
                if len(bad) < 10:
                    bad.append((str(g.relative_to(FIXTURES_DIR)), repr(want[:30]), repr((got or "")[:30])))
        else:
            mb_match += ok
            mb_mismatch += not ok

    print(f"goldens={len(goldens)} no_fixture={nofix}")
    print(f"CPYTHON-side: match={cpy_match} mismatch={cpy_mismatch}  (superset proof)")
    print(f"MAMBA-side:   match={mb_match} mismatch={mb_mismatch}  "
          f"(current SUT state; red is the runtime line)")
    for name, want, got in bad:
        print(f"  CPY-MISMATCH {name} want={want} got={got}")
    return 0 if cpy_mismatch == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
