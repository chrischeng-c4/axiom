#!/usr/bin/env python3.12
"""Parallel failure triage for the mamba-vs-CPython gate.

`gate_status.py` answers "how far is mamba" with one number, but it is serial
and only aggregates — it never says WHICH fixtures fail or WHY. This tool is
the actionable half: it classifies fixtures in parallel (the work is
subprocess-bound, so threads scale to all cores), then groups every MAMBA_RED
by a normalized error signature and every DIVERGE by its first differing line,
so the output ranks runtime bugs by how many fixtures each one blocks.

    python3.12 tests/harness/cpython/tools/gate_triage.py --per-bucket 200
    python3.12 tests/harness/cpython/tools/gate_triage.py --buckets behavior
    MAMBA_BIN=target/release/mamba python3.12 .../gate_triage.py

Full per-fixture verdicts land in tests/cpython/.cache/triage/triage.json so a
fix-verify loop can re-run exactly the fixtures a signature group contains.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import sys
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import harness_lib  # noqa: E402

TOOLS_DIR = Path(__file__).resolve().parent
FIXTURES_DIR = TOOLS_DIR.parents[2] / "cpython"
CACHE_DIR = FIXTURES_DIR / ".cache" / "triage"
BUCKETS = (
    "type", "surface", "behavior", "errors", "real_world", "security",
    "perf", "security-matrix",
)

_HEX = re.compile(r"0x[0-9a-fA-F]+")
_NUM = re.compile(r"\b\d+\b")
_PATH = re.compile(r"/[^\s:,)]+")


def signature(stderr: str, rc) -> str:
    """Normalize a mamba failure into a groupable signature."""
    if rc is None:
        return "<timeout>"
    if isinstance(rc, int) and rc < 0:
        return f"<signal {-rc}>"
    lines = [ln.strip() for ln in stderr.splitlines() if ln.strip()]
    if not lines:
        return f"<exit {rc}, empty stderr>"
    ln = lines[-1]
    ln = _PATH.sub("<path>", ln)
    ln = _HEX.sub("<hex>", ln)
    ln = _NUM.sub("<n>", ln)
    return ln[:160]


def first_diff(a: str, b: str) -> str:
    al, bl = a.splitlines(), b.splitlines()
    for i, (x, y) in enumerate(zip(al, bl)):
        if x != y:
            return f"line {i+1}: oracle={x[:60]!r} mamba={y[:60]!r}"
    if len(al) != len(bl):
        return f"line-count: oracle={len(al)} mamba={len(bl)}"
    return "<identical?>"


def classify(fixture: Path, mamba_bin: str, timeout: int):
    orc, oout, _ = harness_lib.run_fixture(["python3.12", str(fixture)], timeout)
    oout = str(oout)
    if orc != 0:
        return fixture, harness_lib.ORACLE_SKIP, ""
    inner = (f"ulimit -t {timeout} 2>/dev/null; ulimit -c 0 2>/dev/null; "
             f"exec {mamba_bin} run {fixture}")
    mrc, mout, merr = harness_lib.run_fixture(["/bin/sh", "-c", inner], timeout + 5)
    if mrc != 0 or mrc is None:
        return fixture, harness_lib.MAMBA_RED, signature(str(merr), mrc)
    if str(mout) == oout:
        return fixture, harness_lib.PASS, ""
    return fixture, harness_lib.DIVERGE, first_diff(oout, str(mout))


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--per-bucket", type=int, default=None)
    ap.add_argument("--timeout", type=int, default=10)
    ap.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--buckets", default=None,
                    help="comma-separated facet filter (default: all)")
    ap.add_argument("--top", type=int, default=25,
                    help="how many signature groups to print")
    args = ap.parse_args(argv)

    mamba_bin = os.environ.get("MAMBA_BIN") or shutil.which("mamba") or "mamba"
    buckets = args.buckets.split(",") if args.buckets else BUCKETS

    work: list[Path] = []
    for bucket in buckets:
        root = FIXTURES_DIR / bucket
        if not root.exists():
            continue
        fixtures = sorted(root.rglob("*.py"))
        if args.per_bucket and len(fixtures) > args.per_bucket:
            step = len(fixtures) // args.per_bucket
            fixtures = fixtures[::step][: args.per_bucket]
        work.extend(fixtures)

    print(f"triage: {len(work)} fixtures, {args.jobs} jobs, bin={mamba_bin}",
          file=sys.stderr)
    overall = Counter()
    per_bucket: dict[str, Counter] = defaultdict(Counter)
    groups: dict[tuple[str, str], list[str]] = defaultdict(list)
    records = []
    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        for fx, verdict, sig in ex.map(
                lambda f: classify(f, mamba_bin, args.timeout), work):
            rel = str(fx.relative_to(FIXTURES_DIR))
            bucket = rel.split("/", 1)[0]
            overall[verdict] += 1
            per_bucket[bucket][verdict] += 1
            records.append({"fixture": rel, "verdict": verdict, "detail": sig})
            if verdict in (harness_lib.MAMBA_RED, harness_lib.DIVERGE):
                groups[(verdict, sig)].append(rel)

    print(f"{'bucket':14} {'n':>6} {'PASS':>6} {'RED':>6} {'DIVERGE':>8} {'SKIP':>6}  pass_rate")
    for bucket in buckets:
        c = per_bucket.get(bucket)
        if not c:
            continue
        _, _, pr = harness_lib.compute_pass_rate(c)
        print(f"{bucket:14} {sum(c.values()):6} {c[harness_lib.PASS]:6} "
              f"{c[harness_lib.MAMBA_RED]:6} {c[harness_lib.DIVERGE]:8} "
              f"{c[harness_lib.ORACLE_SKIP]:6}  {pr:5.1f}%")
    passed, graded, pr = harness_lib.compute_pass_rate(overall)
    print(f"{'TOTAL':14} {sum(overall.values()):6} {passed:6} "
          f"{overall[harness_lib.MAMBA_RED]:6} {overall[harness_lib.DIVERGE]:8} "
          f"{overall[harness_lib.ORACLE_SKIP]:6}  {pr:5.1f}%")

    print(f"\ntop {args.top} failure groups (count  verdict  signature):")
    ranked = sorted(groups.items(), key=lambda kv: -len(kv[1]))
    for (verdict, sig), members in ranked[: args.top]:
        print(f"{len(members):6}  {verdict:9}  {sig}")
        for m in members[:2]:
            print(f"        e.g. {m}")

    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    out = CACHE_DIR / "triage.json"
    out.write_text(json.dumps({
        "mamba_bin": mamba_bin,
        "pass_rate": pr,
        "graded": graded,
        "records": records,
    }, indent=1))
    print(f"\nfull verdicts -> {out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
