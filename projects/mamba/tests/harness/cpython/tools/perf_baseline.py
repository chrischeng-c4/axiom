#!/usr/bin/env python3
"""Create/query the CPython perf baseline SQLite database.

The perf baseline is machine-local derived data. Generate it before mamba
runtime work, then compare mamba runs against the recorded CPython CPU/time
and peak-RSS values:

    python3 tests/harness/cpython/tools/perf_baseline.py record
    python3 tests/harness/cpython/tools/perf_baseline.py record --missing-only --ready-only --limit 10
    python3 tests/harness/cpython/tools/perf_baseline.py get --pin tests/harness/cpython/config/perf/pins/abc_1447.toml

No third-party dependencies; stdlib only.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import resource
import shlex
import sqlite3
import subprocess
import sys
import time
import tomllib
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parent
CPYTHON_DIR = TOOLS_DIR.parents[2] / "cpython"  # tests/cpython (fixtures + .cache)
MAMBA_DIR = CPYTHON_DIR.parent.parent
PINS_DIR = TOOLS_DIR.parent / "config" / "perf" / "pins"  # config under harness/
DEFAULT_DB = CPYTHON_DIR / ".cache" / "perf" / "cpython_baseline.sqlite"

INTERNAL_RE = re.compile(r"^INTERNAL_TIME_NS=(\d+)\s*$", re.MULTILINE)
RSS_MACOS_RE = re.compile(r"^\s*(\d+)\s+maximum resident set size\s*$", re.MULTILINE)
RSS_LINUX_RE = re.compile(
    r"^\s*Maximum resident set size \(kbytes\):\s*(\d+)\s*$",
    re.MULTILINE,
)
CPU_MACOS_RE = re.compile(
    r"^\s*[\d.]+\s+real\s+([\d.]+)\s+user\s+([\d.]+)\s+sys\s*$",
    re.MULTILINE,
)
USER_LINUX_RE = re.compile(r"^\s*User time \(seconds\):\s*([\d.]+)\s*$", re.MULTILINE)
SYS_LINUX_RE = re.compile(r"^\s*System time \(seconds\):\s*([\d.]+)\s*$", re.MULTILINE)


def repo_rel(path: Path) -> str:
    return path.resolve().relative_to(MAMBA_DIR.resolve()).as_posix()


def default_time_prefix() -> list[str]:
    time_bin = Path("/usr/bin/time")
    if not time_bin.exists():
        return []
    return [str(time_bin), "-l" if sys.platform == "darwin" else "-v"]


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def parse_internal_time(stdout: str, stderr: str) -> int:
    for text in (stderr, stdout):
        m = INTERNAL_RE.search(text)
        if m:
            return int(m.group(1))
    raise RuntimeError("fixture output missing INTERNAL_TIME_NS")


def parse_peak_rss(stderr: str) -> int | None:
    m = RSS_MACOS_RE.search(stderr)
    if m:
        return int(m.group(1))
    m = RSS_LINUX_RE.search(stderr)
    if m:
        return int(m.group(1)) * 1024
    return None


def parse_cpu_time_ns(stderr: str) -> int | None:
    m = CPU_MACOS_RE.search(stderr)
    if m:
        return int((float(m.group(1)) + float(m.group(2))) * 1_000_000_000)
    user = USER_LINUX_RE.search(stderr)
    sys_time = SYS_LINUX_RE.search(stderr)
    if user and sys_time:
        return int((float(user.group(1)) + float(sys_time.group(1))) * 1_000_000_000)
    return None


def load_pin(path: Path) -> dict:
    with path.open("rb") as fh:
        data = tomllib.load(fh)
    for key in ("issue", "lib", "fixture", "floor", "samples"):
        if key not in data:
            raise ValueError(f"{path}: missing {key}")
    return data


def python_can_import(module: str, python: str) -> bool:
    return subprocess.run(
        [python, "-c", f"import {module}"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        text=True,
    ).returncode == 0


def missing_prereq_imports(pin: dict, python: str) -> list[str]:
    return [
        str(module)
        for module in pin.get("prereq_imports", [])
        if not python_can_import(str(module), python)
    ]


def measure_once(python: str, fixture: Path) -> dict:
    argv = [*default_time_prefix(), python, str(fixture)]
    if not argv:
        argv = [python, str(fixture)]
    started = time.time()
    before_usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    result = subprocess.run(argv, text=True, capture_output=True, timeout=120)
    after_usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    try:
        internal_time_ns = parse_internal_time(result.stdout, result.stderr)
    except RuntimeError:
        internal_time_ns = None

    if internal_time_ns is None:
        raise RuntimeError(
            f"{fixture}: CPython output missing INTERNAL_TIME_NS rc={result.returncode}\n"
            f"stdout={result.stdout}\nstderr={result.stderr}"
        )
    if result.returncode != 0:
        print(
            f"WARN {fixture}: timing wrapper returned rc={result.returncode} "
            "after fixture emitted INTERNAL_TIME_NS",
            file=sys.stderr,
        )
    child_cpu_time_ns = int(
        (
            (after_usage.ru_utime - before_usage.ru_utime)
            + (after_usage.ru_stime - before_usage.ru_stime)
        )
        * 1_000_000_000
    )
    return {
        "internal_time_ns": internal_time_ns,
        "cpu_time_ns": child_cpu_time_ns or parse_cpu_time_ns(result.stderr),
        "peak_rss_bytes": parse_peak_rss(result.stderr),
        "captured_at_unix": int(started),
    }


def median(values: list[int]) -> int:
    values = sorted(values)
    return values[len(values) // 2]


def measure_pin(pin_path: Path, python: str) -> dict | None:
    pin = load_pin(pin_path)
    if missing_prereq_imports(pin, python):
        return None

    fixture = MAMBA_DIR / pin["fixture"]
    samples = max(int(pin.get("samples", 1)), 1)
    rows = [measure_once(python, fixture) for _ in range(samples)]
    internal_values = [row["internal_time_ns"] for row in rows]
    cpu_values = [row["cpu_time_ns"] for row in rows if row["cpu_time_ns"] is not None]
    rss_values = [row["peak_rss_bytes"] for row in rows if row["peak_rss_bytes"] is not None]
    return {
        "pin_path": repo_rel(pin_path),
        "issue": int(pin["issue"]),
        "lib": str(pin["lib"]),
        "fixture": str(pin["fixture"]),
        "fixture_sha256": sha256_file(fixture),
        "samples": samples,
        "internal_time_ns": median(internal_values),
        "cpu_time_ns": median(cpu_values) if cpu_values else None,
        "peak_rss_bytes": min(rss_values) if rss_values else None,
        "python": python,
        "platform": platform.platform(),
        "argv": shlex.join([python, str(fixture)]),
        "captured_at_unix": int(time.time()),
    }


def connect(db_path: Path) -> sqlite3.Connection:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS cpython_perf_baseline (
            pin_path TEXT PRIMARY KEY,
            issue INTEGER NOT NULL,
            lib TEXT NOT NULL,
            fixture TEXT NOT NULL,
            fixture_sha256 TEXT NOT NULL,
            samples INTEGER NOT NULL,
            internal_time_ns INTEGER NOT NULL,
            cpu_time_ns INTEGER,
            peak_rss_bytes INTEGER,
            python TEXT NOT NULL,
            platform TEXT NOT NULL,
            argv TEXT NOT NULL,
            captured_at_unix INTEGER NOT NULL
        )
        """
    )
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_cpython_perf_baseline_fixture "
        "ON cpython_perf_baseline(fixture)"
    )
    return conn


def upsert(conn: sqlite3.Connection, row: dict) -> None:
    conn.execute(
        """
        INSERT INTO cpython_perf_baseline (
            pin_path, issue, lib, fixture, fixture_sha256, samples,
            internal_time_ns, cpu_time_ns, peak_rss_bytes, python, platform,
            argv, captured_at_unix
        ) VALUES (
            :pin_path, :issue, :lib, :fixture, :fixture_sha256, :samples,
            :internal_time_ns, :cpu_time_ns, :peak_rss_bytes, :python,
            :platform, :argv, :captured_at_unix
        )
        ON CONFLICT(pin_path) DO UPDATE SET
            issue = excluded.issue,
            lib = excluded.lib,
            fixture = excluded.fixture,
            fixture_sha256 = excluded.fixture_sha256,
            samples = excluded.samples,
            internal_time_ns = excluded.internal_time_ns,
            cpu_time_ns = excluded.cpu_time_ns,
            peak_rss_bytes = excluded.peak_rss_bytes,
            python = excluded.python,
            platform = excluded.platform,
            argv = excluded.argv,
            captured_at_unix = excluded.captured_at_unix
        """,
        row,
    )


def existing_pin_paths(conn: sqlite3.Connection) -> set[str]:
    rows = conn.execute("SELECT pin_path FROM cpython_perf_baseline").fetchall()
    return {str(row[0]) for row in rows}


def record(args: argparse.Namespace) -> int:
    pin_paths = [Path(args.pin)] if args.pin else sorted(PINS_DIR.glob("*.toml"))
    conn = connect(Path(args.db))
    if args.missing_only:
        existing = existing_pin_paths(conn)
        pin_paths = [path for path in pin_paths if repo_rel(path) not in existing]

    if args.limit is not None and args.limit < 1:
        raise ValueError("--limit must be >= 1")

    recorded = 0
    skipped = 0
    failed = 0
    for pin_path in pin_paths:
        if args.limit is not None and recorded >= args.limit:
            break
        pin_path = pin_path.resolve()
        try:
            pin = load_pin(pin_path)
        except Exception as exc:
            if args.keep_going:
                print(f"FAIL {repo_rel(pin_path)}: {exc}", file=sys.stderr)
                skipped += 1
                failed += 1
                continue
            raise
        missing_prereqs = missing_prereq_imports(pin, args.python)
        if missing_prereqs:
            if not args.ready_only:
                print(
                    f"SKIP {repo_rel(pin_path)}: missing prereq import "
                    f"{', '.join(missing_prereqs)}"
                )
            skipped += 1
            continue
        try:
            row = measure_pin(pin_path, args.python)
        except Exception as exc:
            if args.keep_going:
                print(f"FAIL {repo_rel(pin_path)}: {exc}", file=sys.stderr)
                skipped += 1
                failed += 1
                continue
            raise
        if row is None:
            if not args.ready_only:
                print(f"SKIP {repo_rel(pin_path)}: missing prereq import")
            skipped += 1
            continue
        upsert(conn, row)
        conn.commit()
        recorded += 1
        print(
            f"REC {row['pin_path']}: internal={row['internal_time_ns']}ns "
            f"cpu={row['cpu_time_ns']} rss={row['peak_rss_bytes']}"
        )
    print(f"baseline db: {Path(args.db)}")
    print(f"selected={len(pin_paths)} recorded={recorded} skipped={skipped} failed={failed}")
    return 0


def get(args: argparse.Namespace) -> int:
    pin_path = repo_rel(Path(args.pin))
    conn = connect(Path(args.db))
    conn.row_factory = sqlite3.Row
    row = conn.execute(
        "SELECT * FROM cpython_perf_baseline WHERE pin_path = ?",
        (pin_path,),
    ).fetchone()
    if row is None:
        return 2
    print(json.dumps(dict(row), sort_keys=True))
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--db", default=os.environ.get("MAMBA_CPYTHON_PERF_BASELINE_DB", str(DEFAULT_DB)))
    sub = parser.add_subparsers(dest="cmd", required=True)

    rec = sub.add_parser("record")
    rec.add_argument("--pin", help="record one pin TOML instead of all pins")
    rec.add_argument("--python", default=os.environ.get("PYTHON", "python3"))
    rec.add_argument("--keep-going", action="store_true")
    rec.add_argument("--missing-only", action="store_true", help="only record pins absent from the baseline DB")
    rec.add_argument("--ready-only", action="store_true", help="silently skip pins whose prereq imports are unavailable")
    rec.add_argument("--limit", type=int, help="stop after recording this many pins")
    rec.set_defaults(func=record)

    getp = sub.add_parser("get")
    getp.add_argument("--pin", required=True)
    getp.set_defaults(func=get)

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
