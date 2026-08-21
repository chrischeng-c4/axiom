#!/usr/bin/env python3
"""Single results store for the mamba production test gate (D5.3/D5.4).

Machine-local, gitignored SQLite holding every `fixture x runtime x dimension`
result. CPython rows are *content-addressed*: the oracle runs once per fixture
content and is re-run only when the fixture bytes change. This is the
generalization of `perf_baseline.py`'s sqlite-cache pattern from the perf/pin
model to all dimensions, with measurement taken EXTERNALLY (getrusage +
`/usr/bin/time`) instead of trusting a fixture-emitted `INTERNAL_TIME_NS`
marker (PRODUCTION-GATE.md D5.2/D5.3). Mamba SUT rows are recorded by the
D5.4 pooled/sandboxed collector.

    python3 tests/harness/cpython/tools/results_store.py record-oracle --limit 20
    python3 tests/harness/cpython/tools/results_store.py record-oracle            # all fixtures, incremental
    python3 tests/harness/cpython/tools/results_store.py stats

stdlib only; no third-party dependencies.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import resource
import shlex
import shutil
import subprocess
import sys
import time
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parent
CPYTHON_DIR = TOOLS_DIR.parents[2] / "cpython"  # tests/cpython (fixtures + .cache)
MAMBA_DIR = CPYTHON_DIR.parent.parent
FIXTURES_DIR = CPYTHON_DIR
DEFAULT_DB = CPYTHON_DIR / ".cache" / "conformance" / "results.sqlite"

# dimension subdir names per FIXTURE-LAYOUT.md
DIMENSIONS = ("surface", "behavior", "errors", "bench", "real_world", "security", "concurrency")

RSS_MACOS_RE = re.compile(r"^\s*(\d+)\s+maximum resident set size\s*$", re.MULTILINE)
RSS_LINUX_RE = re.compile(
    r"^\s*Maximum resident set size \(kbytes\):\s*(\d+)\s*$", re.MULTILINE
)
# CPython traceback final line: "SomeError: message"
RAISED_RE = re.compile(r"^([A-Za-z_][A-Za-z0-9_.]*(?:Error|Exception|Warning|Interrupt|Iteration|Exit)):", re.MULTILINE)


def repo_rel(path: Path) -> str:
    return path.resolve().relative_to(MAMBA_DIR.resolve()).as_posix()


def default_time_prefix() -> list[str]:
    time_bin = Path("/usr/bin/time")
    if not time_bin.exists():
        return []
    return [str(time_bin), "-l" if sys.platform == "darwin" else "-v"]


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def parse_peak_rss(stderr: str) -> int | None:
    m = RSS_MACOS_RE.search(stderr)
    if m:
        return int(m.group(1))
    m = RSS_LINUX_RE.search(stderr)
    if m:
        return int(m.group(1)) * 1024
    return None


def parse_raised_type(stderr: str) -> str | None:
    matches = RAISED_RE.findall(stderr)
    return matches[-1] if matches else None


def dimension_of(fixture: Path) -> str:
    """Infer the dimension from the path; bucket-shaped trees use a dimension subdir."""
    for part in fixture.parts:
        if part in DIMENSIONS:
            return part
    if "type-strict" in fixture.parts:
        return "type-strict"
    return "behavior"


def cpython_version(python: str) -> str:
    out = subprocess.run(
        [python, "--version"], text=True, capture_output=True
    )
    return (out.stdout or out.stderr).strip() or "python3"


def run_fixture(python: str, fixture: Path, timeout: int) -> dict:
    """Run a fixture under CPython, measuring CPU + peak-RSS EXTERNALLY."""
    prefix = default_time_prefix()
    argv = [*prefix, python, str(fixture)] if prefix else [python, str(fixture)]
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    timed_out = 0
    try:
        result = subprocess.run(argv, text=True, capture_output=True, timeout=timeout)
        returncode = result.returncode
        stdout, stderr = result.stdout, result.stderr
    except subprocess.TimeoutExpired as exc:
        timed_out = 1
        returncode = None
        stdout = exc.stdout.decode() if isinstance(exc.stdout, bytes) else (exc.stdout or "")
        stderr = exc.stderr.decode() if isinstance(exc.stderr, bytes) else (exc.stderr or "")
    after = resource.getrusage(resource.RUSAGE_CHILDREN)

    cpu_time_ns = int(
        ((after.ru_utime - before.ru_utime) + (after.ru_stime - before.ru_stime))
        * 1_000_000_000
    )
    signal = None
    exit_code = None
    if returncode is not None:
        if returncode < 0:
            signal = -returncode
        else:
            exit_code = returncode
    return {
        "exit_code": exit_code,
        "stdout_hash": sha256_bytes(stdout.encode()),
        "raised_type": parse_raised_type(stderr),
        "cpu_time_ns": cpu_time_ns,
        "peak_rss_bytes": parse_peak_rss(stderr),
        "signal": signal,
        "timed_out": timed_out,
    }


def connect(db_path: Path):
    import sqlite3

    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS results (
            fixture_id       TEXT NOT NULL,
            content_hash     TEXT NOT NULL,
            runtime          TEXT NOT NULL,
            runtime_version  TEXT NOT NULL,
            dimension        TEXT NOT NULL,
            verdict          TEXT,
            exit_code        INTEGER,
            stdout_hash      TEXT,
            raised_type      TEXT,
            cpu_time_ns      INTEGER,
            peak_rss_bytes   INTEGER,
            signal           INTEGER,
            timed_out        INTEGER,
            recorded_at      INTEGER NOT NULL,
            PRIMARY KEY (fixture_id, content_hash, runtime, runtime_version)
        )
        """
    )
    conn.execute("CREATE INDEX IF NOT EXISTS idx_results_dim ON results(dimension)")
    conn.execute("CREATE INDEX IF NOT EXISTS idx_results_verdict ON results(verdict)")
    return conn


def has_row(conn, fixture_id: str, content_hash: str, runtime: str, version: str) -> bool:
    row = conn.execute(
        "SELECT 1 FROM results WHERE fixture_id=? AND content_hash=? "
        "AND runtime=? AND runtime_version=? LIMIT 1",
        (fixture_id, content_hash, runtime, version),
    ).fetchone()
    return row is not None


def upsert(conn, row: dict) -> None:
    conn.execute(
        """
        INSERT INTO results (
            fixture_id, content_hash, runtime, runtime_version, dimension,
            verdict, exit_code, stdout_hash, raised_type, cpu_time_ns,
            peak_rss_bytes, signal, timed_out, recorded_at
        ) VALUES (
            :fixture_id, :content_hash, :runtime, :runtime_version, :dimension,
            :verdict, :exit_code, :stdout_hash, :raised_type, :cpu_time_ns,
            :peak_rss_bytes, :signal, :timed_out, :recorded_at
        )
        ON CONFLICT(fixture_id, content_hash, runtime, runtime_version) DO UPDATE SET
            dimension=excluded.dimension, verdict=excluded.verdict,
            exit_code=excluded.exit_code, stdout_hash=excluded.stdout_hash,
            raised_type=excluded.raised_type, cpu_time_ns=excluded.cpu_time_ns,
            peak_rss_bytes=excluded.peak_rss_bytes, signal=excluded.signal,
            timed_out=excluded.timed_out, recorded_at=excluded.recorded_at
        """,
        row,
    )


def oracle_verdict(measured: dict) -> str:
    if measured["timed_out"]:
        return "ORACLE_TIMEOUT"
    if measured["signal"] is not None:
        return "ORACLE_CRASH"
    if measured["exit_code"] == 0:
        return "ORACLE_OK"
    return "ORACLE_NONZERO"


def iter_fixtures(bucket: str | None, dimension: str | None = None) -> list[Path]:
    root = FIXTURES_DIR / bucket if bucket else FIXTURES_DIR
    fixtures = sorted(p for p in root.rglob("*.py"))
    if dimension:
        fixtures = [f for f in fixtures if dimension_of(f) == dimension]
    return fixtures


def record_oracle(args: argparse.Namespace) -> int:
    conn = connect(Path(args.db))
    version = cpython_version(args.python)
    fixtures = iter_fixtures(args.bucket, getattr(args, "dimension", None))
    hit = miss = failed = 0
    for fixture in fixtures:
        if args.limit is not None and (miss) >= args.limit:
            break
        fixture_id = repo_rel(fixture)
        content_hash = sha256_bytes(fixture.read_bytes())
        if has_row(conn, fixture_id, content_hash, "cpython", version):
            hit += 1
            continue
        try:
            measured = run_fixture(args.python, fixture, args.timeout)
        except Exception as exc:  # noqa: BLE001 - record-and-continue
            print(f"FAIL {fixture_id}: {exc}", file=sys.stderr)
            failed += 1
            continue
        row = {
            "fixture_id": fixture_id,
            "content_hash": content_hash,
            "runtime": "cpython",
            "runtime_version": version,
            "dimension": dimension_of(fixture),
            "verdict": oracle_verdict(measured),
            "recorded_at": int(time.time()),
            **measured,
        }
        upsert(conn, row)
        conn.commit()
        miss += 1
    print(f"results db: {Path(args.db)}")
    print(f"oracle runtime={version} hit={hit} miss={miss} failed={failed}")
    return 0


def stats(args: argparse.Namespace) -> int:
    conn = connect(Path(args.db))
    total = conn.execute("SELECT COUNT(*) FROM results").fetchone()[0]
    print(f"results db: {Path(args.db)}  rows={total}")
    print("by runtime x dimension x verdict:")
    for runtime, dim, verdict, n in conn.execute(
        "SELECT runtime, dimension, verdict, COUNT(*) FROM results "
        "GROUP BY runtime, dimension, verdict ORDER BY runtime, dimension, verdict"
    ):
        print(f"  {runtime:8} {dim:12} {verdict:16} {n}")
    return 0


# fixture dimension subdir -> production-gate tier (PRODUCTION-GATE.md)
DIM_TO_TIER = {
    "type-strict": "D1",
    "behavior": "D2", "surface": "D2", "errors": "D2", "real_world": "D2",
    "bench": "D3",
    "security": "D4",
}
TIER_NAME = {"D1": "type-strict", "D2": "behavior", "D3": "perf",
             "D4": "safety", "D5": "harness"}


def summary(args: argparse.Namespace) -> int:
    """Machine-readable D1-D5 per-dimension roll-up (+ optional --since delta)."""
    conn = connect(Path(args.db))
    tiers = {
        t: {"name": TIER_NAME[t], "rows": 0, "by_verdict": {}}
        for t in ("D1", "D2", "D3", "D4", "D5")
    }
    tiers["D5"]["note"] = "harness meta — built/checked by gate_check.py; no fixture rows"
    for dim, runtime, verdict, n in conn.execute(
        "SELECT dimension, runtime, verdict, COUNT(*) FROM results "
        "GROUP BY dimension, runtime, verdict"
    ):
        bucket = tiers[DIM_TO_TIER.get(dim, "D2")]
        bucket["rows"] += n
        key = f"{runtime}:{verdict}"
        bucket["by_verdict"][key] = bucket["by_verdict"].get(key, 0) + n

    delta = None
    if args.since:
        # SUT rows exist, but they are keyed by runtime_version rather than a
        # normalized git sha. Keep the delta honest until the release metadata
        # path supplies a stable commit key.
        delta = {"since": args.since, "newly_red": 0, "newly_green": 0,
                 "regressions": 0, "note": "mamba SUT git-sha delta pending release metadata"}

    out = {"dimensions": tiers, "delta": delta}
    if args.json:
        print(json.dumps(out, indent=2, sort_keys=True))
    else:
        for t in ("D1", "D2", "D3", "D4", "D5"):
            b = tiers[t]
            print(f"{t} {b['name']:12} rows={b['rows']:6} {b['by_verdict']}")
        if delta:
            print(f"delta since {args.since}: {delta}")
    return 0


# ── mamba SUT collector (D5.4): pooled, per-fixture sandboxed child ───────────

SIGNAL_NAMES = {4: "ILL", 6: "ABRT", 8: "FPE", 9: "KILL", 10: "BUS", 11: "SEGV"}


def mamba_bin() -> str:
    return os.environ.get("MAMBA_BIN") or shutil.which("mamba") or "mamba"


def mamba_version() -> str:
    try:
        p = subprocess.run([mamba_bin(), "--version"], text=True,
                           capture_output=True, timeout=15)
        return (p.stdout or p.stderr).strip() or "mamba-unknown"
    except Exception:
        return "mamba-unknown"


def run_mamba_sandboxed(fixture: Path, cpu_secs: int, timeout: int) -> dict:
    """Run `mamba run <fixture>` as an isolated child under a ulimit sandbox.

    ulimit caps CPU seconds and disables core dumps so a runaway/segfaulting
    fixture is contained by the kernel, never the harness. Signal-kills are
    surfaced as CRASH_SIG_* so a crashing fixture cannot take the pool down.
    """
    inner = (f"ulimit -t {cpu_secs} 2>/dev/null; ulimit -c 0 2>/dev/null; "
             f"exec {shlex.quote(mamba_bin())} run {shlex.quote(str(fixture))}")
    argv = ["/bin/sh", "-c", inner]
    timed_out = 0
    try:
        p = subprocess.run(argv, text=True, capture_output=True, timeout=timeout)
        rc, stdout, stderr = p.returncode, p.stdout, p.stderr
    except subprocess.TimeoutExpired as exc:
        timed_out, rc = 1, None
        stdout = exc.stdout.decode(errors="replace") if isinstance(exc.stdout, bytes) else (exc.stdout or "")
        stderr = exc.stderr.decode(errors="replace") if isinstance(exc.stderr, bytes) else (exc.stderr or "")
    signal = exit_code = None
    if rc is not None:
        if rc < 0:
            signal = -rc
        elif rc > 128:
            signal = rc - 128  # /bin/sh convention for signal-terminated child
        else:
            exit_code = rc
    return {
        "exit_code": exit_code,
        "stdout_hash": sha256_bytes(stdout.encode()),
        "raised_type": parse_raised_type(stderr),
        "cpu_time_ns": None,  # accurate per-child CPU is D3's serial/`/usr/bin/time` path
        "peak_rss_bytes": parse_peak_rss(stderr),
        "signal": signal,
        "timed_out": timed_out,
    }


def mamba_verdict(m: dict) -> str:
    if m["timed_out"]:
        return "MAMBA_TIMEOUT"
    if m["signal"] is not None:
        return f"CRASH_SIG_{SIGNAL_NAMES.get(m['signal'], m['signal'])}"
    if m["exit_code"] == 0:
        return "MAMBA_OK"
    return "MAMBA_NONZERO"


def collect(args: argparse.Namespace) -> int:
    from concurrent.futures import ThreadPoolExecutor, as_completed

    conn = connect(Path(args.db))
    version = mamba_version()
    fixtures = iter_fixtures(args.bucket, getattr(args, "dimension", None))
    todo = []
    for fixture in fixtures:
        content_hash = sha256_bytes(fixture.read_bytes())
        if has_row(conn, repo_rel(fixture), content_hash, "mamba", version):
            continue
        todo.append((fixture, content_hash))
        if args.limit is not None and len(todo) >= args.limit:
            break

    done = locked = 0
    verdicts: dict[str, int] = {}
    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        fut_meta = {
            ex.submit(run_mamba_sandboxed, fx, args.cpu, args.timeout): (fx, ch)
            for fx, ch in todo
        }
        for fut in as_completed(fut_meta):  # main thread = single writer
            fixture, content_hash = fut_meta[fut]
            try:
                measured = fut.result()
            except Exception as exc:  # noqa: BLE001 - never let one fixture kill the pool
                measured = {"exit_code": None, "stdout_hash": "", "raised_type": f"collector-error:{exc}",
                            "cpu_time_ns": None, "peak_rss_bytes": None, "signal": None, "timed_out": 0}
            verdict = mamba_verdict(measured)
            verdicts[verdict] = verdicts.get(verdict, 0) + 1
            row = {
                "fixture_id": repo_rel(fixture),
                "content_hash": content_hash,
                "runtime": "mamba",
                "runtime_version": version,
                "dimension": dimension_of(fixture),
                "verdict": verdict,
                "recorded_at": int(time.time()),
                **measured,
            }
            try:
                upsert(conn, row)
                conn.commit()
            except Exception as exc:  # noqa: BLE001
                if "locked" in str(exc).lower():
                    locked += 1
            done += 1

    print(f"results db: {Path(args.db)}")
    print(f"collect mamba={version!r} jobs={args.jobs} done={done} "
          f"locked_errors={locked}")
    print(f"verdicts: {verdicts}")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--db",
        default=os.environ.get("MAMBA_RESULTS_DB", str(DEFAULT_DB)),
    )
    sub = parser.add_subparsers(dest="cmd", required=True)

    rec = sub.add_parser("record-oracle", help="content-addressed CPython oracle pass")
    rec.add_argument("--python", default=os.environ.get("PYTHON", "python3"))
    rec.add_argument("--bucket", help="limit to one bucket subdir under fixtures/")
    rec.add_argument("--dimension", help="limit to one dimension (behavior/bench/security/...)")
    rec.add_argument("--limit", type=int, help="stop after this many MISS (new) records")
    rec.add_argument("--timeout", type=int, default=120)
    rec.set_defaults(func=record_oracle)

    st = sub.add_parser("stats", help="print store row counts")
    st.set_defaults(func=stats)

    sm = sub.add_parser("summary", help="D1-D5 per-dimension roll-up")
    sm.add_argument("--since", help="git sha to diff mamba results against")
    sm.add_argument("--json", action="store_true")
    sm.set_defaults(func=summary)

    col = sub.add_parser("collect", help="pooled sandboxed mamba SUT pass")
    col.add_argument("--bucket", help="limit to one bucket subdir under fixtures/")
    col.add_argument("--dimension", help="limit to one dimension (behavior/bench/security/...)")
    col.add_argument("--limit", type=int, help="stop after this many new fixtures")
    col.add_argument("--jobs", type=int, default=max((os.cpu_count() or 4) - 2, 1))
    col.add_argument("--cpu", type=int, default=10, help="per-child CPU-seconds ulimit")
    col.add_argument("--timeout", type=int, default=20, help="per-child wall timeout")
    col.set_defaults(func=collect)

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
