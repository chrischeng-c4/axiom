#!/usr/bin/env python3.12
"""3-runtime language comparison — SAME case, SAME method.

For each benches/lang/<case>/ holding <case>.py + <case>.go:
  - python3 and mamba run the IDENTICAL <case>.py (clean A/B, same source)
  - go runs the sibling <case>.go (same algorithm + same N)
  - the SAME measure() (best-of-N min wall + min peak-RSS via `/usr/bin/time -l`)
    is applied to all three, on this machine, so it's apples-to-apples
  - outputs are checked IDENTICAL before any ratio is reported (else the
    comparison is meaningless)

Headline: mamba/go (target -> 1.0x = Go-class) and cpython/go (the gap mamba is
closing). CPython is the floor anchor; Go is the ceiling.

    MAMBA_BIN=/tmp/mamba-release python3.12 benches/lang/lang_bench.py
    ... --iters 5            # best-of-N (default 3)
"""
from __future__ import annotations

import os
import re
import subprocess
import sys

LANG_DIR = os.path.dirname(os.path.abspath(__file__))
MAMBA = os.environ.get("MAMBA_BIN", "/tmp/mamba-release")
ITERS = 3
for i, a in enumerate(sys.argv):
    if a == "--iters" and i + 1 < len(sys.argv):
        ITERS = int(sys.argv[i + 1])

_REAL = re.compile(r"([\d.]+)\s+real")
_RSS = re.compile(r"(\d+)\s+maximum resident set size")


def measure(cmd):
    """Best-of-ITERS: (min wall_ms, min peak_rss_mb, stdout) | (None, None, err)."""
    best_ms = float("inf")
    best_rss = float("inf")
    out = None
    for _ in range(ITERS):
        r = subprocess.run(["/usr/bin/time", "-l", *cmd], capture_output=True, text=True)
        if r.returncode != 0:
            return None, None, f"FAIL rc={r.returncode}: {(r.stderr or '').strip()[-160:]}"
        out = r.stdout.strip()
        if m := _REAL.search(r.stderr):
            best_ms = min(best_ms, float(m.group(1)) * 1000.0)
        if m := _RSS.search(r.stderr):
            best_rss = min(best_rss, int(m.group(1)) / 1048576.0)
    return best_ms, best_rss, out


def run_case(case_dir):
    case = os.path.basename(case_dir)
    py = os.path.join(case_dir, f"{case}.py")
    go_src = os.path.join(case_dir, f"{case}.go")
    go_bin = os.path.join(case_dir, f"{case}_go")
    if not (os.path.exists(py) and os.path.exists(go_src)):
        return
    # compile go once (release-equivalent; go build is already optimized)
    cb = subprocess.run(["go", "build", "-o", go_bin, go_src], capture_output=True, text=True)
    if cb.returncode != 0:
        print(f"\n## {case}\n  go build FAILED: {cb.stderr.strip()[-200:]}")
        return

    runtimes = [
        ("cpython", ["python3.12", py]),
        ("mamba", [MAMBA, "run", py]),
        ("go", [go_bin]),
    ]
    results = {}
    print(f"\n## {case}  (best-of-{ITERS})")
    for name, cmd in runtimes:
        ms, rss, out = measure(cmd)
        results[name] = (ms, rss, out)
        if ms is None:
            print(f"  {name:8} {out}")
        else:
            print(f"  {name:8} wall={ms:9.1f} ms   rss={rss:7.1f} MB   out={out}")

    # correctness gate: all three must agree
    outs = {n: r[2] for n, r in results.items() if r[0] is not None}
    if len(set(outs.values())) > 1:
        print(f"  !! OUTPUT MISMATCH — comparison void: {outs}")
        return
    if {"mamba", "go", "cpython"} - set(outs):
        print("  !! a runtime failed — ratios skipped")
        return

    cms, _, _ = results["cpython"]
    mms, mrss, _ = results["mamba"]
    gms, grss, _ = results["go"]
    print(f"  -> wall: mamba/go = {mms / gms:5.2f}x   cpython/go = {cms / gms:6.2f}x   "
          f"mamba/cpython = {cms / mms:5.2f}x faster")
    print(f"  -> rss:  mamba/go = {mrss / grss:5.2f}x   cpython/go = {results['cpython'][1] / grss:5.2f}x")


def main():
    cases = sorted(
        d for d in (os.path.join(LANG_DIR, x) for x in os.listdir(LANG_DIR))
        if os.path.isdir(d)
    )
    print(f"lang_bench — mamba={MAMBA}  go={subprocess.run(['go', 'version'], capture_output=True, text=True).stdout.strip()}")
    for c in cases:
        run_case(c)


if __name__ == "__main__":
    main()
