#!/usr/bin/env python3
"""perf_baseline.py - bench perf table (CPU time + peak RSS x cpython|mamba).

WHY THIS EXISTS (and why it is small): correctness is *stateless* - a fixture
self-asserts, so `mamba run <fixture>` exit-0 IS the answer (run it and you
know). The ONLY thing worth persisting is PERFORMANCE, because perf is a
*relative* metric: the ratio needs the other runtime's number, and CPython's
side is stable across mamba edits. So this records a tiny SQLite table - one
row per BENCH fixture, two runtime column-groups - and the edit-mamba loop
pulls CPython's stored baseline instead of re-measuring it.

    record --runtime cpython   # CPython is stable -> record once (hash-skip)
    record --runtime mamba     # mamba is what you tune -> re-record each edit
    view  [--lib X] [--only mamba|cpython]   # join -> ratios (no runtime run)

Metrics (from /usr/bin/time):
    cpu  = user+sys CPU seconds -> ns   (more stable than wall)
    rss  = peak resident set size (bytes)
Ratios (>=1.0 means mamba is not worse; mirrors cross_runtime.rs FLOOR=1.0):
    speed_ratio = cpy_cpu_ns / mam_cpu_ns
    mem_ratio   = cpy_rss    / mam_rss

Only BENCH fixtures (<bucket>/<lib>/bench/*.py) have perf; everything else is
correctness-only and never touches this table. stdlib only.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import shutil
import sqlite3
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent
CPYTHON_DIR = HERE.parent / "tests" / "cpython"
FIXTURES_ROOT = CPYTHON_DIR / "fixtures"
DB_PATH = CPYTHON_DIR / ".cache" / "perf.db"

SPEED_FLOOR = 1.0
MEM_FLOOR = 1.0

_USR_BIN_TIME = "/usr/bin/time"

# /usr/bin/time output parsers (BSD `-l` on macOS, GNU `-v` on Linux).
_BSD_USER = re.compile(r"([\d.]+)\s+user\b")
_BSD_SYS = re.compile(r"([\d.]+)\s+sys\b")
_BSD_RSS = re.compile(r"^\s*(\d+)\s+maximum resident set size\s*$", re.MULTILINE)
_GNU_USER = re.compile(r"^\s*User time \(seconds\):\s*([\d.]+)\s*$", re.MULTILINE)
_GNU_SYS = re.compile(r"^\s*System time \(seconds\):\s*([\d.]+)\s*$", re.MULTILINE)
_GNU_RSS = re.compile(r"^\s*Maximum resident set size \(kbytes\):\s*(\d+)\s*$",
                      re.MULTILINE)


def _time_available() -> bool:
    return os.path.isfile(_USR_BIN_TIME) and os.access(_USR_BIN_TIME, os.X_OK)


def _parse_time(stderr: str) -> tuple[int | None, int | None]:
    """Return (cpu_ns, rss_bytes) parsed from a /usr/bin/time stderr blob, or
    (None, None) parts when a shape isn't present. Tries BSD then GNU."""
    cpu_ns = rss = None
    mu, ms = _BSD_USER.search(stderr), _BSD_SYS.search(stderr)
    if mu and ms:
        cpu_ns = int((float(mu.group(1)) + float(ms.group(1))) * 1e9)
    else:
        gu, gs = _GNU_USER.search(stderr), _GNU_SYS.search(stderr)
        if gu and gs:
            cpu_ns = int((float(gu.group(1)) + float(gs.group(1))) * 1e9)
    mr = _BSD_RSS.search(stderr)
    if mr:
        rss = int(mr.group(1))            # macOS reports bytes
    else:
        gr = _GNU_RSS.search(stderr)
        if gr:
            rss = int(gr.group(1)) * 1024  # Linux reports kbytes
    return cpu_ns, rss


def _timed_run(cmd: list[str], *, timeout: int) -> tuple[int, int | None, int | None]:
    """Run `cmd` under /usr/bin/time. Return (rc, cpu_ns, rss_bytes). rc is the
    child's status (best-effort; the wrapper exits 128+N on a signal death)."""
    argv = [_USR_BIN_TIME, "-l" if sys.platform == "darwin" else "-v", *cmd]
    env = {**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}
    try:
        p = subprocess.run(argv, capture_output=True, text=True,
                           timeout=timeout, env=env)
    except subprocess.TimeoutExpired:
        return 124, None, None
    except FileNotFoundError:
        return -2, None, None
    cpu_ns, rss = _parse_time(p.stderr)
    return p.returncode, cpu_ns, rss


def content_hash(test: Path) -> str:
    try:
        return hashlib.sha256(test.read_bytes()).hexdigest()
    except OSError:
        return ""


def mamba_buildid(mamba: str) -> str:
    """Identity of the mamba binary: version + realpath mtime, so a rebuild
    invalidates every stored mamba column."""
    try:
        ver = subprocess.run([mamba, "--version"], capture_output=True,
                             text=True, timeout=10).stdout.strip()
    except (OSError, subprocess.SubprocessError):
        ver = "?"
    real = shutil.which(mamba) or mamba
    try:
        mt = os.path.getmtime(os.path.realpath(real))
    except OSError:
        mt = 0.0
    return f"{ver}@{mt:.0f}"


def db(path: Path = DB_PATH) -> sqlite3.Connection:
    path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(path))
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("""CREATE TABLE IF NOT EXISTS perf (
        rel TEXT PRIMARY KEY, lib TEXT,
        cpy_hash TEXT, cpy_cpu_ns INTEGER, cpy_rss INTEGER, cpy_ts REAL,
        mam_hash TEXT, mam_buildid TEXT, mam_cpu_ns INTEGER, mam_rss INTEGER,
        mam_ts REAL)""")
    return conn


def bench_fixtures(lib: str | None):
    """Yield (rel, lib, path) for every bench fixture, optional --lib filter.
    Bench fixtures live at <bucket>/<lib>/bench/<case>.py."""
    for path in sorted(FIXTURES_ROOT.rglob("bench/*.py")):
        if path.name.endswith("_stub.py"):
            continue
        # lib = the dir that contains bench/
        lib_name = path.parent.parent.name
        if lib and lib_name != lib:
            continue
        yield str(path.relative_to(HERE.parent / "tests" / "cpython")), lib_name, path


def cmd_record(args) -> int:
    if not _time_available():
        print("/usr/bin/time unavailable; cannot record perf", file=sys.stderr)
        return 2
    runtime = args.runtime
    conn = db(Path(args.db))
    existing = {r["rel"]: r for r in conn.execute("SELECT * FROM perf")}
    buildid = mamba_buildid(args.mamba) if runtime == "mamba" else None

    todo, skipped = [], 0
    for rel, lib, path in bench_fixtures(args.lib):
        h = content_hash(path)
        row = existing.get(rel)
        if not args.force and row is not None:
            if runtime == "cpython" and row["cpy_hash"] == h and row["cpy_cpu_ns"] is not None:
                skipped += 1
                continue
            if (runtime == "mamba" and row["mam_hash"] == h
                    and row["mam_buildid"] == buildid and row["mam_cpu_ns"] is not None):
                skipped += 1
                continue
        todo.append((rel, lib, path, h))

    def work(item):
        rel, lib, path, h = item
        if runtime == "cpython":
            cmd = [args.python3, str(path)]
        else:
            cmd = [args.mamba, "run", str(path)]
        rc, cpu_ns, rss = _timed_run(cmd, timeout=args.timeout)
        return (rel, lib, h, rc, cpu_ns, rss)

    rows = []
    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        for r in ex.map(work, todo):
            rows.append(r)

    now = time.time()
    if runtime == "cpython":
        sql = ("INSERT INTO perf (rel,lib,cpy_hash,cpy_cpu_ns,cpy_rss,cpy_ts) "
               "VALUES (?,?,?,?,?,?) ON CONFLICT(rel) DO UPDATE SET "
               "lib=excluded.lib,cpy_hash=excluded.cpy_hash,"
               "cpy_cpu_ns=excluded.cpy_cpu_ns,cpy_rss=excluded.cpy_rss,"
               "cpy_ts=excluded.cpy_ts")
        params = [(rel, lib, h, cpu, rss, now)
                  for rel, lib, h, rc, cpu, rss in rows]
    else:
        sql = ("INSERT INTO perf (rel,lib,mam_hash,mam_buildid,mam_cpu_ns,mam_rss,mam_ts) "
               "VALUES (?,?,?,?,?,?,?) ON CONFLICT(rel) DO UPDATE SET "
               "lib=excluded.lib,mam_hash=excluded.mam_hash,"
               "mam_buildid=excluded.mam_buildid,mam_cpu_ns=excluded.mam_cpu_ns,"
               "mam_rss=excluded.mam_rss,mam_ts=excluded.mam_ts")
        params = [(rel, lib, h, buildid, cpu, rss, now)
                  for rel, lib, h, rc, cpu, rss in rows]
    with conn:
        conn.executemany(sql, params)
    conn.close()
    bad = sum(1 for *_, cpu, rss in rows if cpu is None or rss is None)
    print(f"recorded {len(rows)} {runtime} bench fixtures "
          f"(skipped {skipped} unchanged"
          + (f", {bad} with unparseable time/rss" if bad else "") + ")",
          file=sys.stderr)
    return 0


def _ratio(num, den):
    if not num or not den:
        return None
    return num / den


def cmd_view(args) -> int:
    conn = db(Path(args.db))
    live = {rel for rel, _, _ in bench_fixtures(args.lib)}
    rows = [r for r in conn.execute("SELECT * FROM perf ORDER BY rel")
            if r["rel"] in live]
    conn.close()
    regressions = 0
    shown = 0
    for r in rows:
        rel = r["rel"]
        speed = _ratio(r["cpy_cpu_ns"], r["mam_cpu_ns"])
        mem = _ratio(r["cpy_rss"], r["mam_rss"])
        if args.only == "mamba":
            cpu = f"{r['mam_cpu_ns']/1e6:.1f}ms" if r["mam_cpu_ns"] else "?"
            rss = f"{r['mam_rss']/1e6:.1f}MB" if r["mam_rss"] else "?"
            line = f"  {rel:60} mamba cpu={cpu} rss={rss}"
        elif args.only == "cpython":
            cpu = f"{r['cpy_cpu_ns']/1e6:.1f}ms" if r["cpy_cpu_ns"] else "?"
            rss = f"{r['cpy_rss']/1e6:.1f}MB" if r["cpy_rss"] else "?"
            line = f"  {rel:60} cpython cpu={cpu} rss={rss}"
        else:
            sp = f"{speed:.2f}x" if speed else "n/a"
            mm = f"{mem:.2f}x" if mem else "n/a"
            flags = []
            if speed is not None and speed < SPEED_FLOOR:
                flags.append("SLOW")
            if mem is not None and mem < MEM_FLOOR:
                flags.append("MEM")
            if (speed is not None and speed < SPEED_FLOOR) or \
               (mem is not None and mem < MEM_FLOOR):
                regressions += 1
            tag = (" [" + ",".join(flags) + "]") if flags else ""
            miss = "" if (speed and mem) else "  (need both records)"
            line = f"  {rel:60} speed={sp} mem={mm}{tag}{miss}"
        is_reg = bool(args.only is None and
                      ((speed is not None and speed < SPEED_FLOOR) or
                       (mem is not None and mem < MEM_FLOOR)))
        if not args.fails or is_reg or args.only:
            print(line)
            shown += 1
    if args.only is None:
        print(f"\n{len(rows)} bench fixtures, {regressions} regression(s) "
              f"(speed<{SPEED_FLOOR} or mem<{MEM_FLOOR})")
        return 1 if regressions else 0
    print(f"\n{shown} bench fixtures ({args.only} only)")
    return 0


def main(argv=None) -> int:
    doc = (__doc__ or "").splitlines()
    ap = argparse.ArgumentParser(description=doc[0] if doc else "perf baseline")
    ap.add_argument("cmd", nargs="?", choices=("record", "view"), default="view")
    ap.add_argument("--runtime", choices=("cpython", "mamba"),
                    help="which runtime to record (record only)")
    ap.add_argument("--lib", default=None, help="restrict to one lib")
    ap.add_argument("--only", choices=("cpython", "mamba"), default=None,
                    help="view one runtime's raw numbers (no ratio)")
    ap.add_argument("--fails", action="store_true",
                    help="view: only rows below a floor")
    ap.add_argument("--force", action="store_true",
                    help="record even unchanged fixtures")
    ap.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--timeout", type=int, default=120)
    ap.add_argument("--python3",
                    default=os.environ.get("PYTHON3", "python3"))
    ap.add_argument("--mamba", default=os.environ.get("MAMBA", "mamba"))
    ap.add_argument("--db", default=str(DB_PATH))
    args = ap.parse_args(argv)

    if args.cmd == "record":
        if not args.runtime:
            ap.error("record requires --runtime cpython|mamba")
        return cmd_record(args)
    return cmd_view(args)


if __name__ == "__main__":
    raise SystemExit(main())
