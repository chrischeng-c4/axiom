#!/usr/bin/env python3.12
"""The honest "how far is mamba" number — mamba-vs-CPython pass rate.

`gate_check.py` (GATE: N/10) measures whether the INSTRUMENT is built. This
measures the SUBJECT. For each fixture it runs CPython 3.12 (the oracle) and
mamba, then classifies:

    PASS         oracle exit 0, mamba exit 0, stdout identical
    MAMBA_RED    mamba non-zero / crash / timeout — not yet implemented
    DIVERGE      both exit 0 but stdout differs — wrong behavior
    ORACLE_SKIP  oracle itself can't run here (missing 3rd-party module, etc.)

pass_rate = PASS / (graded), graded = total - ORACLE_SKIP. That single number is
how far mamba is from Py3.12 parity — it should rise as the runtime line lands
fixes, and it is the real "is the goal true or false" signal.

    python3.12 tests/harness/cpython/tools/gate_status.py --per-bucket 120   # stratified sample
    python3.12 tests/harness/cpython/tools/gate_status.py                    # every fixture (slow)

mamba is ulimit-sandboxed; PYTHONBREAKPOINT=0 keeps breakpoint fixtures quiet.
"""

from __future__ import annotations

import argparse
import os
import shutil
from collections import Counter
from pathlib import Path

import harness_lib  # shared oracle/SUT runner (tools/): isolated scratch CWD + PYTHONBREAKPOINT=0

TOOLS_DIR = Path(__file__).resolve().parent  # tests/harness/cpython/tools
FIXTURES_DIR = TOOLS_DIR.parents[2] / "cpython"  # tests/cpython
# Dimension-first layout (STRUCTURE.md): the top-level dirs are FACETS, not
# buckets. Stratify the pass-rate sample by facet so iteration walks real
# fixture dirs (the old {core,std-libs,...} bucket dirs are no longer top-level).
BUCKETS = (
    "type", "surface", "behavior", "errors", "real_world", "security",
    "perf", "security-matrix",
)


def run(argv, timeout):
    rc, out, _err = harness_lib.run_fixture(argv, timeout)
    return rc, out


def classify(fixture: Path, mamba_bin: str, timeout: int) -> str:
    orc, oout = run(["python3.12", str(fixture)], timeout)
    if orc != 0:
        return harness_lib.ORACLE_SKIP
    inner = f"ulimit -t {timeout} 2>/dev/null; ulimit -c 0 2>/dev/null; exec {mamba_bin} run {fixture}"
    mrc, mout = run(["/bin/sh", "-c", inner], timeout + 5)
    if mrc != 0 or mrc is None:
        return harness_lib.MAMBA_RED
    return harness_lib.PASS if mout == oout else harness_lib.DIVERGE


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--per-bucket", type=int, default=None,
                    help="stratified sample: at most N spread across each bucket")
    ap.add_argument("--timeout", type=int, default=10)
    args = ap.parse_args(argv)

    mamba_bin = os.environ.get("MAMBA_BIN") or shutil.which("mamba") or "mamba"
    overall = Counter()
    print(f"{'bucket':14} {'n':>6} {'PASS':>6} {'MAMBA_RED':>10} {'DIVERGE':>8} {'SKIP':>6}  pass_rate")
    for bucket in BUCKETS:
        root = FIXTURES_DIR / bucket
        if not root.exists():
            continue
        fixtures = sorted(root.rglob("*.py"))
        if args.per_bucket and len(fixtures) > args.per_bucket:
            step = len(fixtures) // args.per_bucket
            fixtures = fixtures[::step][: args.per_bucket]
        c = Counter()
        for fx in fixtures:
            c[classify(fx, mamba_bin, args.timeout)] += 1
        overall += c
        _, graded, pr = harness_lib.compute_pass_rate(c)
        print(f"{bucket:14} {sum(c.values()):6} {c[harness_lib.PASS]:6} {c[harness_lib.MAMBA_RED]:10} "
              f"{c[harness_lib.DIVERGE]:8} {c[harness_lib.ORACLE_SKIP]:6}  {pr:5.1f}%")

    _, graded, pr = harness_lib.compute_pass_rate(overall)
    print(f"{'TOTAL':14} {sum(overall.values()):6} {overall[harness_lib.PASS]:6} {overall[harness_lib.MAMBA_RED]:10} "
          f"{overall[harness_lib.DIVERGE]:8} {overall[harness_lib.ORACLE_SKIP]:6}  {pr:5.1f}%")
    print(f"\nmamba Py3.12 PASS_RATE = {pr:.1f}%  "
          f"({overall[harness_lib.PASS]}/{graded} graded fixtures; "
          f"{overall[harness_lib.MAMBA_RED]} not-yet-implemented, {overall[harness_lib.DIVERGE]} wrong)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
