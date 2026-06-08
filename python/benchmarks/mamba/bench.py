#!/usr/bin/env python3
"""
Mamba Compiler Benchmark
========================
Compares: Mamba JIT (Cranelift) | Mamba AOT (compile only) | CPython 3.x

Usage:
    python3 python/benchmarks/mamba/bench.py [--runs N]

Requires: cclab binary built with `cargo build -p cclab-cli --release`
"""
import subprocess
import sys
import time
import os
import statistics

REPO_ROOT = os.path.dirname(os.path.abspath(__file__))
while not os.path.exists(os.path.join(REPO_ROOT, "Cargo.toml")):
    REPO_ROOT = os.path.dirname(REPO_ROOT)

CCLAB = os.path.join(REPO_ROOT, "target", "release", "cclab")
BENCH_DIR = os.path.join(REPO_ROOT, "python", "benchmarks", "mamba")

BENCHMARKS = [
    ("fib", "fib.py", "Fibonacci x10M iters"),
    ("primes", "primes.py", "Count primes < 1,000,000"),
    ("sum_loop", "sum_loop.py", "Sum 0..10,000,000"),
]


def check_prerequisites():
    if not os.path.exists(CCLAB):
        print(f"Error: {CCLAB} not found.")
        print("Build with: cargo build -p cclab-cli --release")
        sys.exit(1)
    # Verify mamba subcommand
    r = subprocess.run([CCLAB, "mamba", "run", "--help"],
                       capture_output=True, text=True)
    if r.returncode != 0:
        print("Error: 'cclab mamba run' not available.")
        sys.exit(1)


def time_command(args, warmup=False):
    """Run a command and return wall-clock seconds."""
    if warmup:
        subprocess.run(args, capture_output=True)
    start = time.perf_counter()
    r = subprocess.run(args, capture_output=True, text=True)
    elapsed = time.perf_counter() - start
    if r.returncode != 0:
        return None, r.stderr.strip()
    return elapsed, None


def run_cpython(path, runs):
    """Time CPython execution over multiple runs."""
    cmd = [sys.executable, "-c",
           f"exec(open({path!r}).read()); f()"]
    times = []
    for _ in range(runs):
        t, err = time_command(cmd)
        if t is None:
            return None, err
        times.append(t)
    return times, None


def run_mamba_jit(path, runs):
    """Time Mamba JIT (compile + execute) over multiple runs."""
    cmd = [CCLAB, "mamba", "run", path]
    times = []
    for _ in range(runs):
        t, err = time_command(cmd)
        if t is None:
            return None, err
        times.append(t)
    return times, None


def run_mamba_aot(path, runs):
    """Time Mamba AOT (compile only, no execute) over multiple runs."""
    cmd = [CCLAB, "mamba", "build", path]
    times = []
    for _ in range(runs):
        t, err = time_command(cmd)
        if t is None:
            return None, err
        times.append(t)
    return times, None


def format_time(seconds):
    if seconds < 0.001:
        return f"{seconds*1_000_000:.0f}us"
    if seconds < 1.0:
        return f"{seconds*1000:.1f}ms"
    return f"{seconds:.3f}s"


def main():
    runs = 5
    if "--runs" in sys.argv:
        idx = sys.argv.index("--runs")
        runs = int(sys.argv[idx + 1])

    check_prerequisites()

    # Get Python version
    py_ver = subprocess.run([sys.executable, "--version"],
                            capture_output=True, text=True).stdout.strip()

    print()
    print("=" * 72)
    print("  Mamba Compiler Benchmark")
    print("=" * 72)
    print(f"  Runtime   : {py_ver}")
    print(f"  Runs      : {runs} (median)")
    print(f"  cclab     : {CCLAB}")
    print("=" * 72)
    print()

    # Header
    print(f"{'Benchmark':<28} {'Mamba JIT':>10} {'AOT Build':>10} "
          f"{'CPython':>10} {'Speedup':>10}")
    print("-" * 72)

    for name, filename, desc in BENCHMARKS:
        path = os.path.join(BENCH_DIR, filename)

        # Warmup
        time_command([CCLAB, "mamba", "run", path], warmup=True)

        # Run benchmarks
        jit_times, jit_err = run_mamba_jit(path, runs)
        aot_times, aot_err = run_mamba_aot(path, runs)
        py_times, py_err = run_cpython(path, runs)

        jit_med = statistics.median(jit_times) if jit_times else None
        aot_med = statistics.median(aot_times) if aot_times else None
        py_med = statistics.median(py_times) if py_times else None

        jit_s = format_time(jit_med) if jit_med else "ERR"
        aot_s = format_time(aot_med) if aot_med else "ERR"
        py_s = format_time(py_med) if py_med else "ERR"

        if jit_med and py_med and jit_med > 0:
            speedup = f"{py_med / jit_med:.1f}x"
        else:
            speedup = "-"

        print(f"  {desc:<26} {jit_s:>10} {aot_s:>10} {py_s:>10} {speedup:>10}")

    print("-" * 72)
    print()
    print("  JIT    = parse + typecheck + lower + Cranelift JIT + execute")
    print("  AOT    = parse + typecheck + lower + Cranelift codegen (no exec)")
    print("  CPython = interpreter startup + execution")
    print("  Speedup = CPython / Mamba JIT")
    print()


if __name__ == "__main__":
    main()
