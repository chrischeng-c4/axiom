#!/usr/bin/env python3
"""Mamba benchmark runner (R4).

Runs each benchmark under Mamba JIT, CPython 3.12, and PyPy 7.3 (if available),
measures wall-clock time, computes statistics, and prints a comparison table.

Usage:
    python3 benchmarks/run_benchmarks.py [--runs N] [--filter PATTERN] [--json]

Options:
    --runs N        Number of timed repetitions per benchmark (default: 5).
    --filter PAT    Only run benchmarks whose path contains PAT.
    --json          Output raw JSON data to results/latest.json instead of table.
    --skip-mamba    Skip Mamba runs (useful when binary not yet available).
"""

import argparse
import json
import os
import re
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

BENCHMARK_ROOT = Path(__file__).parent
RESULTS_DIR = BENCHMARK_ROOT / "results"
RESULTS_DIR.mkdir(exist_ok=True)

# Command templates; {script} is substituted with the benchmark path.
RUNNERS: Dict[str, Optional[List[str]]] = {
    "mamba":   ["cclab", "mamba", "run", "{script}"],
    "cpython": [sys.executable, "{script}"],
    "pypy":    ["pypy3", "{script}"],   # None if not found on PATH
}

MICRO_DIR = BENCHMARK_ROOT / "micro"
WORKLOAD_DIR = BENCHMARK_ROOT / "workloads"

BENCHMARKS: List[Path] = sorted(
    list(MICRO_DIR.glob("*.py")) + list(WORKLOAD_DIR.glob("*.py"))
)

# ---------------------------------------------------------------------------
# Timing helpers
# ---------------------------------------------------------------------------

def time_command(cmd: List[str], timeout: float = 120.0) -> Tuple[Optional[float], str]:
    """Run `cmd`, return (elapsed_seconds, stdout) or (None, error_msg)."""
    try:
        start = time.perf_counter()
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
        elapsed = time.perf_counter() - start
        if result.returncode != 0:
            return None, result.stderr.strip() or "non-zero exit"
        return elapsed, result.stdout.strip()
    except subprocess.TimeoutExpired:
        return None, f"timeout ({timeout}s)"
    except FileNotFoundError:
        return None, "not found"
    except Exception as exc:
        return None, str(exc)


def bench_runner(runner_cmd: List[str], script: Path, runs: int) -> Dict:
    """Run a single benchmark N times, return stats dict."""
    times: List[float] = []
    output: str = ""
    errors: List[str] = []

    for _ in range(runs):
        cmd = [c.replace("{script}", str(script)) for c in runner_cmd]
        elapsed, out = time_command(cmd)
        if elapsed is None:
            errors.append(out)
        else:
            times.append(elapsed)
            output = out

    if not times:
        return {"status": "error", "errors": errors, "output": ""}

    return {
        "status": "ok",
        "mean": statistics.mean(times),
        "median": statistics.median(times),
        "stdev": statistics.stdev(times) if len(times) > 1 else 0.0,
        "min": min(times),
        "max": max(times),
        "runs": times,
        "output": output,
    }


# ---------------------------------------------------------------------------
# Availability checks
# ---------------------------------------------------------------------------

def check_available(runner_name: str) -> bool:
    """Return True if the runner binary is on PATH."""
    if runner_name == "cpython":
        return True  # always use current Python
    cmd_template = RUNNERS.get(runner_name)
    if cmd_template is None:
        return False
    binary = cmd_template[0]
    return shutil.which(binary) is not None


# ---------------------------------------------------------------------------
# Table formatting
# ---------------------------------------------------------------------------

COL_W = 14


def format_time(seconds: Optional[float]) -> str:
    if seconds is None:
        return "N/A".rjust(COL_W)
    ms = seconds * 1000
    return f"{ms:.1f} ms".rjust(COL_W)


def format_ratio(t: Optional[float], base: Optional[float]) -> str:
    if t is None or base is None or base == 0:
        return "—".rjust(COL_W)
    ratio = t / base
    return f"×{ratio:.2f}".rjust(COL_W)


def print_table(results: List[Dict]) -> None:
    runners = ["mamba", "cpython", "pypy"]
    header = f"{'Benchmark':<30}" + "".join(r.rjust(COL_W) for r in runners)
    header += "  mamba/cpython  mamba/pypy"
    print()
    print("=" * len(header))
    print("Mamba Benchmark Suite — wall-clock time (median of N runs)")
    print("=" * len(header))
    print(header)
    print("-" * len(header))

    for row in results:
        name = row["name"][:29]
        times = {r: row["runners"].get(r, {}).get("median") for r in runners}
        line = f"{name:<30}"
        line += "".join(format_time(times.get(r)) for r in runners)
        line += format_ratio(times.get("mamba"), times.get("cpython"))
        line += format_ratio(times.get("mamba"), times.get("pypy"))
        print(line)

    print("=" * len(header))
    print()


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(description="Mamba benchmark runner")
    parser.add_argument("--runs", type=int, default=5, help="Repetitions per benchmark")
    parser.add_argument("--filter", default="", help="Filter benchmarks by path substring")
    parser.add_argument("--json", action="store_true", help="Output JSON to results/latest.json")
    parser.add_argument("--skip-mamba", action="store_true", help="Skip Mamba runs")
    args = parser.parse_args()

    active_runners = {
        name: cmd for name, cmd in RUNNERS.items()
        if cmd is not None and check_available(name) and not (args.skip_mamba and name == "mamba")
    }

    benchmarks = [b for b in BENCHMARKS if args.filter in str(b)]
    if not benchmarks:
        print(f"No benchmarks matched filter '{args.filter}'.")
        sys.exit(1)

    print(f"Running {len(benchmarks)} benchmark(s), {args.runs} run(s) each.")
    print(f"Active runners: {', '.join(active_runners)}")
    print()

    all_results: List[Dict] = []

    for script in benchmarks:
        name = script.relative_to(BENCHMARK_ROOT).as_posix()
        print(f"  {name} ...", flush=True)

        row: Dict = {"name": name, "script": str(script), "runners": {}}

        for runner_name, cmd_template in active_runners.items():
            stats = bench_runner(cmd_template, script, args.runs)
            row["runners"][runner_name] = stats
            if stats["status"] == "ok":
                ms = stats["median"] * 1000
                print(f"    {runner_name:8s}  {ms:.1f} ms (median)", flush=True)
            else:
                print(f"    {runner_name:8s}  ERROR: {stats['errors']}", flush=True)

        all_results.append(row)

    if args.json:
        out_path = RESULTS_DIR / "latest.json"
        with open(out_path, "w") as f:
            json.dump(all_results, f, indent=2)
        print(f"\nResults written to {out_path}")
    else:
        print_table(all_results)


if __name__ == "__main__":
    main()
