#!/usr/bin/env python3.12
"""Batch driver for the (2) Behavior wall: run behavior_extract --emit over every
Lib/test module with uncovered cases, most-uncovered first, N modules in parallel.
Each worker writes denominator-gap keys to its own shard file (no concurrent-append
race); shards are merged into the canonical behavior_gaps.txt at the end. Logs one
line per finished module so a watcher can batch-commit and report (2) %.

Temporary scaffold (not a fixture); delete after the wall reaches 100%.

    python3.12 _behavior_batch.py                 # all remaining modules, 8-wide
    python3.12 _behavior_batch.py --jobs 6
"""
from __future__ import annotations

import argparse
import os
import subprocess
import sys
import tempfile
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

TOOLS = Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS))
import behavior_wall_gen as b  # noqa: E402
from wall_gen_core import parse_header  # noqa: E402

CPY = TOOLS.parents[2] / "cpython"
GAPS = TOOLS.parent / "config" / "behavior_gaps.txt" if (TOOLS.parent / "config").exists() \
    else TOOLS.parents[1] / "cpython" / "config" / "behavior_gaps.txt"
EXTRACT = TOOLS / "behavior_extract.py"


def covered_keys() -> set[str]:
    out: set[str] = set()
    for py in (CPY / "behavior").rglob("*.py"):
        subj = parse_header(py).get("subject", "")
        segs = subj.split(".")
        if len(segs) >= 2:
            out.add(f"{segs[-2]}.{segs[-1]}")
    return out


def gap_keys() -> set[str]:
    out: set[str] = set()
    if GAPS.exists():
        for ln in GAPS.read_text().splitlines():
            ln = ln.strip()
            if ln and not ln.startswith("#"):
                segs = ln.split(".")
                if len(segs) >= 2:
                    out.add(f"{segs[-2]}.{segs[-1]}")
    return out


def ordered_modules() -> list[tuple[int, str]]:
    td = b.lib_test_dir()
    cov = covered_keys()
    gaps = gap_keys()
    mod_keys: dict[str, set[str]] = defaultdict(set)
    for mod, cls, method in b.candidates(td):
        mod_keys[mod].add(f"{cls.split('.')[-1]}.{method}")
    rows = []
    for mod, keys in mod_keys.items():
        uncov = keys - cov - gaps
        if uncov:
            rows.append((len(uncov), mod))
    rows.sort(reverse=True)
    return rows


def run_module(mod: str, uncov: int, shard: Path) -> str:
    env = dict(os.environ, MAMBA_BEHAVIOR_GAPS_FILE=str(shard))
    try:
        r = subprocess.run(
            ["python3.12", str(EXTRACT), "--module", mod, "--emit"],
            capture_output=True, text=True, timeout=5400, env=env)
        tail = (r.stdout.strip().splitlines() or ["(no output)"])[-1]
        status = "OK" if r.returncode == 0 else f"RC{r.returncode}"
        err = r.stderr.strip().splitlines()[-1] if r.returncode and r.stderr.strip() else ""
        return f"{status} {mod} (uncov={uncov}) :: {tail} {err}".rstrip()
    except subprocess.TimeoutExpired:
        return f"TIMEOUT {mod} (uncov={uncov})"
    except Exception as e:  # noqa: BLE001
        return f"CRASH {mod} (uncov={uncov}) :: {e}"


def merge_gaps(shards: list[Path]) -> int:
    header = ""
    existing = set()
    if GAPS.exists():
        lines = GAPS.read_text().splitlines()
        header = "\n".join(l for l in lines if l.startswith("#"))
        existing = {l.strip() for l in lines if l.strip() and not l.startswith("#")}
    before = len(existing)
    for sh in shards:
        if sh.exists():
            for l in sh.read_text().splitlines():
                l = l.strip()
                if l and not l.startswith("#"):
                    existing.add(l)
    body = "\n".join(sorted(existing))
    GAPS.write_text((header + "\n" if header else "") + body + "\n", encoding="utf-8")
    return len(existing) - before


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--limit", type=int, default=10_000)
    args = ap.parse_args()

    rows = ordered_modules()[: args.limit]
    print(f"START modules_with_gaps={len(rows)} jobs={args.jobs}", flush=True)

    tmpdir = Path(tempfile.mkdtemp(prefix="behavior_gaps_"))
    shards = {mod: tmpdir / f"{i}.txt" for i, (_, mod) in enumerate(rows)}

    done = 0
    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        futs = {ex.submit(run_module, mod, uncov, shards[mod]): mod
                for uncov, mod in rows}
        for fut in as_completed(futs):
            done += 1
            print(f"[{done:3d}/{len(rows)}] {fut.result()}", flush=True)

    added = merge_gaps(list(shards.values()))
    print(f"MERGED_GAPS added={added}", flush=True)
    print("DONE", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
