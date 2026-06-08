#!/usr/bin/env python3
"""Executable definition of DONE for the mamba production test gate.

Runs the 10 success-metrics from projects/mamba/tests/PRODUCTION-GATE.md and
prints one line `GATE: N/10 criteria met`, naming each criterion PASS/FAIL with
a short detail. The gate is DONE — the measuring instrument is complete and
honest — when this prints `GATE: 10/10 criteria met`. It does NOT assert that
mamba passes any fixture (red is correct); it asserts the GATE itself is built.

    python3 tests/harness/cpython/tools/gate_check.py
    python3 tests/harness/cpython/tools/gate_check.py --json

stdlib only; read-only (never mutates fixtures, src, or the store).
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

# tools live under tests/harness/cpython/tools/ (harness owns the mechanism);
# fixtures + the .cache store live under tests/cpython/ (pure data).
TOOLS_DIR = Path(__file__).resolve().parent                 # tests/harness/cpython/tools
HARNESS_CPYTHON_DIR = TOOLS_DIR.parent                      # tests/harness/cpython
TESTS_DIR = TOOLS_DIR.parents[2]                            # tests
MAMBA_DIR = TOOLS_DIR.parents[3]                            # projects/mamba
REPO_ROOT = MAMBA_DIR.parent
CPYTHON_DIR = TESTS_DIR / "cpython"                         # fixtures + .cache root

FIXTURES_DIR = CPYTHON_DIR / "fixtures"
HARNESS_DIR = TESTS_DIR / "harness"
BENCHES_DIR = MAMBA_DIR / "benches"
RESULTS_DB = CPYTHON_DIR / ".cache" / "conformance" / "results.sqlite"
RESULTS_STORE = TOOLS_DIR / "results_store.py"

# The self-timing anti-pattern is a fixture measuring its OWN hot loop:
# `time.perf_counter()` around the loop + emitting an `INTERNAL_TIME_NS` marker.
# A bare `import time` is NOT the signal — fixtures testing the `time` module
# legitimately import it — so it is excluded to avoid false positives.
SELF_TIMING = ("perf_counter", "INTERNAL_TIME_NS")


# ── helpers ──────────────────────────────────────────────────────────────────

def py_files(*roots: Path):
    for root in roots:
        if root.exists():
            yield from root.rglob("*.py")


def files_containing(roots, patterns, suffix="*.py") -> list[Path]:
    hits = []
    for root in roots:
        if not root.exists():
            continue
        for path in root.rglob(suffix):
            try:
                text = path.read_text(errors="replace")
            except Exception:
                continue
            if any(p in text for p in patterns):
                hits.append(path)
    return hits


def run(argv, timeout=30):
    try:
        p = subprocess.run(argv, text=True, capture_output=True, timeout=timeout)
        return p.returncode, p.stdout, p.stderr
    except Exception as exc:  # noqa: BLE001
        return None, "", str(exc)


def sqlite_query(db: Path, sql: str):
    if not db.exists():
        return None
    import sqlite3

    try:
        conn = sqlite3.connect(db)
        rows = conn.execute(sql).fetchall()
        conn.close()
        return rows
    except Exception:
        return None


# ── the 10 criteria ──────────────────────────────────────────────────────────

def c_d5_1():
    # Only BENCH fixtures self-time. behavior/surface fixtures testing
    # time.perf_counter legitimately contain it, so scope to the bench dimension.
    # (benches/ outside tests/ is also out of scope.)
    hits = [p for p in FIXTURES_DIR.rglob("*.py")
            if "bench" in p.parts
            and any(sig in p.read_text(errors="replace") for sig in SELF_TIMING)]
    return (len(hits) == 0,
            f"{len(hits)} self-timing bench fixtures; target 0 "
            f"(non-bench fixtures testing perf_counter legitimately excluded)")


def c_d5_2():
    harness_reads = files_containing([HARNESS_DIR], ["INTERNAL_TIME_NS"], "*.rs")
    external = files_containing([HARNESS_DIR], ["getrusage", "maximum resident set size"], "*.rs")
    ok = len(harness_reads) == 0 and len(external) > 0
    return (ok, f"harness files reading INTERNAL_TIME_NS={len(harness_reads)} (want 0); "
                f"external-measurement files={len(external)} (want >0)")


def c_d5_3():
    if not RESULTS_DB.exists():
        return (False, "results.sqlite absent")
    tables = sqlite_query(RESULTS_DB, "SELECT name FROM sqlite_master WHERE type='table'")
    has_results = tables is not None and any(r[0] == "results" for r in tables)
    rows = sqlite_query(RESULTS_DB, "SELECT COUNT(*) FROM results WHERE runtime='cpython'")
    cpy = rows[0][0] if rows else 0
    rc, _, _ = run(["git", "check-ignore", str(RESULTS_DB)])
    ignored = rc == 0
    ok = has_results and cpy > 0 and ignored
    return (ok, f"results table={has_results}, cpython rows={cpy}, gitignored={ignored}")


def c_d5_4():
    # collector must exist AND have produced isolated mamba SUT rows
    rc, _, _ = run([sys.executable, str(RESULTS_STORE), "collect", "--help"])
    has_cmd = rc == 0
    rows = sqlite_query(RESULTS_DB, "SELECT COUNT(*) FROM results WHERE runtime='mamba'")
    mamba_rows = rows[0][0] if rows else 0
    ok = has_cmd and mamba_rows > 0
    return (ok, f"collect subcommand={'yes' if has_cmd else 'no'} (pooled, "
                f"ulimit-sandboxed, single-writer); mamba SUT rows={mamba_rows}")


def c_d5_5():
    rc, out, err = run([sys.executable, str(RESULTS_STORE), "summary", "--json"])
    body = out + err
    has_dims = all(d in body for d in ("D1", "D2", "D3", "D4", "D5"))
    ok = rc == 0 and has_dims
    return (ok, "summary command with D1-D5 dimensions "
                + ("present" if ok else "not implemented"))


def c_d5_6():
    # D5.6 + golden capstone (option B): the legacy runner (run.py) is retired AND
    # the 683 static .expected goldens are retired — runner.rs runs the live
    # CPython oracle, which golden_capstone.py proved reproduces every golden.
    # regen_golden.py remains ONLY because src/main.rs (the `mamba --regen` CLI
    # handler) still invokes it, and src/ is out of tests/** scope; retiring the
    # regenerator is the final cross-scope step. See PRODUCTION-GATE.md D5.6.
    runpy = [p for p in TESTS_DIR.rglob("run.py") if "fixtures" not in p.parts]
    goldens = list(FIXTURES_DIR.rglob("*.expected"))
    doc_refs = files_containing([HARNESS_CPYTHON_DIR / "conventions"], ["tests/cpython/run.py"], "*.md")
    if (REPO_ROOT / "CONTRIBUTING.md").exists():
        if "tests/cpython/run.py" in (REPO_ROOT / "CONTRIBUTING.md").read_text(errors="replace"):
            doc_refs.append(REPO_ROOT / "CONTRIBUTING.md")
    ok = len(runpy) == 0 and len(goldens) == 0 and len(doc_refs) == 0
    regen = (TOOLS_DIR / "regen_golden.py").exists()
    defer = " (regen_golden.py retained: invoked by src/main.rs --regen, cross-scope)" if regen else ""
    return (ok, f"run.py={len(runpy)}, static goldens={len(goldens)}, "
                f"run.py doc refs={len(doc_refs)}{defer}")


def c_d1():
    ts_dir = FIXTURES_DIR / "type-strict"
    fixtures = list(ts_dir.rglob("*.py")) if ts_dir.exists() else []
    markers = ("mamba-strict-type:", "no_typeerror:", "typeerror:")
    unmarked = [f for f in fixtures
                if not any(m in f.read_text(errors="replace") for m in markers)]
    verdict = files_containing([HARNESS_DIR], ["STRICT_TYPE_OK", "MAMBA_TYPE_LEAKED"], "*.rs")
    ok = bool(fixtures) and not unmarked and len(verdict) > 0
    return (ok, f"type-strict fixtures={len(fixtures)}, unmarked={len(unmarked)}, "
                f"harness type-strict verdict files={len(verdict)}")


def c_d2():
    lint = TOOLS_DIR / "fixture_lint.py"
    if not lint.exists():
        return (False, "fixture_lint.py not built")
    py = shutil.which("python3.12") or sys.executable  # tomllib needs 3.11+
    rc, out, _ = run([py, str(lint)])
    last = out.strip().splitlines()[-1] if out.strip() else f"exit={rc}"
    return (rc == 0, last)


def c_d3():
    bench_fixtures = list(FIXTURES_DIR.rglob("bench/*.py")) if FIXTURES_DIR.exists() else []
    n_bench = len(bench_fixtures)
    # distinct fixture_id, not row count — a fixture re-stripped/edited leaves an
    # older content-addressed row behind, which must not double-count.
    rows = sqlite_query(
        RESULTS_DB,
        "SELECT COUNT(DISTINCT fixture_id) FROM results WHERE runtime='cpython' "
        "AND dimension='bench' AND cpu_time_ns IS NOT NULL")
    baseline = rows[0][0] if rows else 0
    ok = n_bench > 0 and baseline >= n_bench
    return (ok, f"cpython bench baseline fixtures={baseline} / bench fixtures={n_bench}")


def c_d4():
    sec_dirs = [p for p in FIXTURES_DIR.rglob("security") if p.is_dir()] if FIXTURES_DIR.exists() else []
    rows = sqlite_query(
        RESULTS_DB,
        "SELECT COUNT(*) FROM results WHERE dimension='security' AND verdict IS NOT NULL")
    verdicts = rows[0][0] if rows else 0
    ok = len(sec_dirs) > 0 and verdicts > 0
    return (ok, f"security corpus dirs={len(sec_dirs)}, security verdict rows={verdicts}")


CRITERIA = [
    ("D5.1", "pure fixtures (no self-timing)", c_d5_1),
    ("D5.2", "harness owns measurement", c_d5_2),
    ("D5.3", "results store (content-addressed)", c_d5_3),
    ("D5.4", "pool collector (sandboxed, crash-isolated)", c_d5_4),
    ("D5.5", "one D1-D5 summary command", c_d5_5),
    ("D5.6", "legacy runner retired", c_d5_6),
    ("D1", "type-strict markers + verdict", c_d1),
    ("D2", "behavior seeds lint", c_d2),
    ("D3", "perf bench baseline complete", c_d3),
    ("D4", "safety corpus + verdicts", c_d4),
]


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)

    results = []
    for cid, name, fn in CRITERIA:
        try:
            passed, detail = fn()
        except Exception as exc:  # noqa: BLE001
            passed, detail = False, f"check error: {exc}"
        results.append({"id": cid, "name": name, "passed": passed, "detail": detail})

    met = sum(1 for r in results if r["passed"])
    total = len(results)

    if args.json:
        print(json.dumps({"met": met, "total": total, "criteria": results}, indent=2))
    else:
        for r in results:
            mark = "PASS" if r["passed"] else "FAIL"
            print(f"  [{mark}] {r['id']:5} {r['name']}: {r['detail']}")
        print(f"GATE: {met}/{total} criteria met")
    return 0 if met == total else 1


if __name__ == "__main__":
    raise SystemExit(main())
