#!/usr/bin/env python3
"""go_suite recorder: mamba vs Go vs CPython, the epic #1071 measuring stick.

Times the 6 server-shaped workloads under `../fixtures/*.py` (Go twins under
`../go/cmd/<shape>/main.go`) across mamba / go / python3.12, plus a
dedicated `hello` shape used ONLY for startup (time-to-first-stdout-byte).
Emits a JSONL row per (shape, runtime) sample set and a Markdown report with
a per-shape table + geomean ratios vs the epic's bar:

    <=2.5x Go CPU geomean, <50ms startup, <=2x Go peak RSS

Usage:

    python3 tools/suite_bench.py \\
        --mamba-bin /path/to/target/release/mamba \\
        --samples 5

Degrades gracefully when no Go toolchain is on PATH (checked via `go
version`): Go columns are omitted, the report says so explicitly, and the
mamba-vs-CPython ratio is reported as a clearly-labeled fallback (NOT a
substitute for the epic's Go bar, which is left unevaluated).

No third-party dependencies; stdlib only. Mirrors the measurement mechanics
of `tests/harness/cpython/tools/perf_baseline.py` (median-of-N CPU time via
`/usr/bin/time`, min-of-N peak RSS) rather than reinventing them.

--- Why fixture arithmetic never uses bitwise `&` on large ints -----------
While building this suite's fixtures, a real mamba correctness gap was
found and worked around (NOT fixed here -- out of scope for this WI):

    x = 140737488355328   # 2**47
    x & 0xFFFFFFFF         # mamba: None -- CPython: 0 (correct)

Bitwise AND (confirmed also OR/XOR/shift are fine; only a mask-shaped AND
reproduces it reliably) on integers >= 2**47 can return `None` instead of a
masked int; plain arithmetic (+, -, *, //, %, comparisons) stays correct at
the same magnitudes in every case tried. Every fixture's checksum helper
therefore uses only `%` (modulo), never `&`, and every fixture's own
arithmetic is designed to keep all intermediate values far below 2**47.
This is independent of the also-discovered `str.startswith()` superlinear
call-count blowup (see route_match.py's inline note) and of a separate,
pre-existing `hashlib.sha256(...).hexdigest()` regression (returns `None`)
that made this suite avoid `hashlib` entirely in favor of a hand-rolled
modulo checksum. All three are flagged for separate follow-up issues, not
fixed by this WI (#1072 is the benchmark suite itself).
----------------------------------------------------------------------------
"""

from __future__ import annotations

import argparse
import json
import math
import os
import platform
import re
import resource
import shlex
import shutil
import signal
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parent
SUITE_DIR = TOOLS_DIR.parent  # tests/harness/go_suite
FIXTURES_DIR = SUITE_DIR / "fixtures"
GO_DIR = SUITE_DIR / "go"
MAMBA_DIR = TOOLS_DIR.parents[2]  # projects/mamba
REPO_ROOT = MAMBA_DIR.parents[1]  # repo root (target/ lives here)

DEFAULT_JSONL = SUITE_DIR / "results.jsonl"
DEFAULT_MD = SUITE_DIR / "REPORT.md"

HELLO_SHAPE = "hello"
# The 6 server-shaped workloads the epic bar is measured over (excludes
# `hello`, which exists only for the startup/time-to-first-output sample).
SHAPES = [
    "json_codec",
    "route_match",
    "data_transform",
    "template_render",
    "string_processing",
    "queue_pipeline",
]

EPIC_CPU_BAR = 2.5
EPIC_STARTUP_MS_BAR = 50.0
EPIC_RSS_BAR = 2.0

RSS_MACOS_RE = re.compile(r"^\s*(\d+)\s+maximum resident set size\s*$", re.MULTILINE)
RSS_LINUX_RE = re.compile(
    r"^\s*Maximum resident set size \(kbytes\):\s*(\d+)\s*$", re.MULTILINE
)
CPU_MACOS_RE = re.compile(
    r"^\s*[\d.]+\s+real\s+([\d.]+)\s+user\s+([\d.]+)\s+sys\s*$", re.MULTILINE
)
USER_LINUX_RE = re.compile(r"^\s*User time \(seconds\):\s*([\d.]+)\s*$", re.MULTILINE)
SYS_LINUX_RE = re.compile(r"^\s*System time \(seconds\):\s*([\d.]+)\s*$", re.MULTILINE)


def time_prefix() -> list[str]:
    time_bin = Path("/usr/bin/time")
    if not time_bin.exists():
        return []
    return [str(time_bin), "-l" if sys.platform == "darwin" else "-v"]


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


def last_checksum(stdout: str) -> str | None:
    """The fixtures print `CHECKSUM <int>` as their final line."""
    for line in reversed(stdout.splitlines()):
        line = line.strip()
        if line.startswith("CHECKSUM"):
            parts = line.split()
            if len(parts) >= 2:
                return parts[-1]
    return None


def median_int(values: list[int]) -> int | None:
    if not values:
        return None
    return int(statistics.median(values))


def run_once(argv: list[str], timeout: float = 60.0) -> dict:
    """Run argv (already including the /usr/bin/time wrapper, if any) once and
    return stdout/cpu/rss. Own process group so a hang can be killpg'd without
    leaking an orphaned grandchild (mirrors perf_baseline.py's #964 fix)."""
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    proc = subprocess.Popen(
        argv, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        start_new_session=True,
    )
    try:
        stdout, stderr = proc.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
        except ProcessLookupError:
            pass
        stdout, stderr = proc.communicate()
        raise RuntimeError(f"TIMEOUT after {timeout}s: {shlex.join(argv)}") from None
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    if proc.returncode != 0:
        raise RuntimeError(
            f"run failed rc={proc.returncode}: {shlex.join(argv)}\n"
            f"stdout={stdout}\nstderr={stderr}"
        )
    rusage_cpu_ns = int(
        ((after.ru_utime - before.ru_utime) + (after.ru_stime - before.ru_stime))
        * 1_000_000_000
    )
    return {
        "stdout": stdout,
        "cpu_time_ns": rusage_cpu_ns or parse_cpu_time_ns(stderr),
        "peak_rss_bytes": parse_peak_rss(stderr),
    }


def measure_shape(cmd: list[str], samples: int) -> dict:
    """Median-of-N CPU time, min-of-N peak RSS (mirrors perf_baseline.py),
    plus the checksum from the first sample (all samples must agree)."""
    prefix = time_prefix()
    argv = [*prefix, *cmd]
    cpu_values: list[int] = []
    rss_values: list[int] = []
    checksums: set[str] = set()
    last_stdout = ""
    for _ in range(samples):
        row = run_once(argv)
        last_stdout = row["stdout"]
        if row["cpu_time_ns"]:
            cpu_values.append(row["cpu_time_ns"])
        if row["peak_rss_bytes"]:
            rss_values.append(row["peak_rss_bytes"])
        cs = last_checksum(row["stdout"])
        if cs is not None:
            checksums.add(cs)
    return {
        "samples": samples,
        "cpu_time_ns": median_int(cpu_values),
        "peak_rss_bytes": min(rss_values) if rss_values else None,
        "checksum": sorted(checksums)[0] if len(checksums) == 1 else None,
        "checksum_stable": len(checksums) <= 1,
        "checksums_seen": sorted(checksums),
        "stdout_tail": last_stdout.strip().splitlines()[-3:],
    }


def measure_startup_ttfb(cmd: list[str], samples: int, timeout: float = 30.0) -> dict:
    """Time-to-first-stdout-byte: wall-clock from spawn to the first stdout
    line, NOT to process exit. Still wrapped in /usr/bin/time so the same
    run also yields CPU/RSS for the hello shape's own table row."""
    prefix = time_prefix()
    argv = [*prefix, *cmd]
    ttfb_values: list[float] = []
    cpu_values: list[int] = []
    rss_values: list[int] = []
    checksums: set[str] = set()
    for _ in range(samples):
        before = resource.getrusage(resource.RUSAGE_CHILDREN)
        t0 = time.perf_counter()
        proc = subprocess.Popen(
            argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            start_new_session=True,
        )
        assert proc.stdout is not None
        # Raw fd read (bypasses Python's buffered TextIOWrapper) so the
        # timestamp reflects the first bytes actually on the pipe, and so
        # the later proc.communicate() call can safely pick up whatever's
        # left without losing data. Mixing `.readline()` on the buffered
        # text-mode stdout with `.communicate()` was tried first and
        # silently dropped CPython's second print() line: CPython
        # block-buffers stdout when piped, so both lines land in one OS
        # write at exit; TextIOWrapper.readline() slurps that whole write
        # into its own internal buffer looking for the first line ending
        # but only returns the first line, and the subsequent
        # communicate() (a separate raw-fd reader) then sees nothing left
        # on the actual pipe. Raw os.read() sidesteps that entirely.
        first_chunk = os.read(proc.stdout.fileno(), 65536)
        t1 = time.perf_counter()
        try:
            rest_stdout, stderr = proc.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
            except ProcessLookupError:
                pass
            rest_stdout, stderr = proc.communicate()
            raise RuntimeError(f"TIMEOUT after {timeout}s: {shlex.join(argv)}") from None
        after = resource.getrusage(resource.RUSAGE_CHILDREN)
        stderr_text = stderr.decode("utf-8", errors="replace") if isinstance(stderr, bytes) else stderr
        if proc.returncode != 0:
            raise RuntimeError(
                f"run failed rc={proc.returncode}: {shlex.join(argv)}\nstderr={stderr_text}"
            )
        ttfb_values.append((t1 - t0) * 1000.0)
        rusage_cpu_ns = int(
            ((after.ru_utime - before.ru_utime) + (after.ru_stime - before.ru_stime))
            * 1_000_000_000
        )
        if rusage_cpu_ns:
            cpu_values.append(rusage_cpu_ns)
        rss = parse_peak_rss(stderr_text)
        if rss:
            rss_values.append(rss)
        full_stdout = (first_chunk + rest_stdout).decode("utf-8", errors="replace")
        cs = last_checksum(full_stdout)
        if cs is not None:
            checksums.add(cs)
    return {
        "samples": samples,
        "ttfb_ms": round(statistics.median(ttfb_values), 3) if ttfb_values else None,
        "cpu_time_ns": median_int(cpu_values),
        "peak_rss_bytes": min(rss_values) if rss_values else None,
        "checksum": sorted(checksums)[0] if len(checksums) == 1 else None,
        "checksum_stable": len(checksums) <= 1,
    }


def go_available() -> str | None:
    """Returns `go version` output if a Go toolchain is on PATH, else None."""
    go_bin = shutil.which("go")
    if not go_bin:
        return None
    try:
        out = subprocess.run(
            [go_bin, "version"], capture_output=True, text=True, timeout=10
        )
    except Exception:
        return None
    if out.returncode != 0:
        return None
    return out.stdout.strip()


def build_go_binaries(shapes: list[str], build_dir: Path) -> dict[str, Path]:
    """`go build -o <build_dir>/<shape> ./cmd/<shape>` for each shape, from
    GO_DIR. Returns {shape: built_binary_path} for shapes that built OK;
    silently omits (with a stderr note) any shape whose build fails so one
    broken twin doesn't sink the whole recorder run."""
    built: dict[str, Path] = {}
    for shape in shapes:
        out_path = build_dir / shape
        proc = subprocess.run(
            ["go", "build", "-o", str(out_path), f"./cmd/{shape}"],
            cwd=str(GO_DIR), capture_output=True, text=True, timeout=120,
        )
        if proc.returncode == 0 and out_path.exists():
            built[shape] = out_path
        else:
            print(f"WARN: go build failed for {shape}: {proc.stderr}", file=sys.stderr)
    return built


def geomean(values: list[float]) -> float | None:
    values = [v for v in values if v is not None and v > 0]
    if not values:
        return None
    return math.exp(sum(math.log(v) for v in values) / len(values))


def fmt_ms(ns: int | None) -> str:
    if ns is None:
        return "n/a"
    return f"{ns / 1_000_000:.1f}"


def fmt_mb(b: int | None) -> str:
    if b is None:
        return "n/a"
    return f"{b / (1024 * 1024):.1f}"


def fmt_ratio(r: float | None) -> str:
    if r is None:
        return "n/a"
    return f"{r:.2f}x"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument(
        "--mamba-bin",
        default=os.environ.get("MAMBA_BIN", str(REPO_ROOT / "target" / "release" / "mamba")),
    )
    parser.add_argument("--python", default=os.environ.get("PYTHON", "python3.12"))
    parser.add_argument("--samples", type=int, default=5)
    parser.add_argument("--out-jsonl", default=str(DEFAULT_JSONL))
    parser.add_argument("--out-md", default=str(DEFAULT_MD))
    parser.add_argument("--skip-go", action="store_true", help="force-skip Go even if a toolchain is present")
    parser.add_argument("--shapes", default=",".join(SHAPES), help="comma-separated subset of shapes to run")
    args = parser.parse_args()

    if args.samples < 5:
        print(f"WARNING: --samples={args.samples} < 5; the contract wants samples>=5", file=sys.stderr)

    mamba_bin = Path(args.mamba_bin)
    if not mamba_bin.exists():
        print(f"error: mamba binary not found: {mamba_bin}", file=sys.stderr)
        return 2

    shapes = [s for s in args.shapes.split(",") if s]
    for s in shapes:
        if s not in SHAPES:
            print(f"error: unknown shape {s!r}; valid: {SHAPES}", file=sys.stderr)
            return 2

    go_version = None if args.skip_go else go_available()
    go_bins: dict[str, Path] = {}
    go_tmp_dir: tempfile.TemporaryDirectory | None = None
    if go_version:
        go_tmp_dir = tempfile.TemporaryDirectory(prefix="go_suite_bin_")
        build_dir = Path(go_tmp_dir.name)
        go_bins = build_go_binaries([HELLO_SHAPE, *shapes], build_dir)
        print(f"go toolchain: {go_version}; built {len(go_bins)}/{len(shapes) + 1} binaries")
    else:
        print("go toolchain: NOT AVAILABLE on this host -- Go columns skipped; "
              "mamba is reported against CPython only as a fallback (NOT the epic's Go bar).")

    jsonl_rows: list[dict] = []
    shape_results: dict[str, dict] = {}

    captured_at = int(time.time())
    host = platform.node()
    plat = platform.platform()

    def record_row(shape: str, runtime: str, measurement: dict, extra: dict | None = None) -> None:
        row = {
            "shape": shape,
            "runtime": runtime,
            "captured_at_unix": captured_at,
            "host": host,
            "platform": plat,
            **measurement,
        }
        if extra:
            row.update(extra)
        jsonl_rows.append(row)

    # --- startup (hello shape): time-to-first-output ------------------------
    print(f"== startup (hello) == samples={args.samples}")
    hello_fixture = FIXTURES_DIR / "hello.py"
    startup: dict[str, dict] = {}
    startup["mamba"] = measure_startup_ttfb([str(mamba_bin), "run", str(hello_fixture)], args.samples)
    record_row(HELLO_SHAPE, "mamba", startup["mamba"])
    print(f"  mamba   ttfb={startup['mamba']['ttfb_ms']}ms cpu={fmt_ms(startup['mamba']['cpu_time_ns'])}ms "
          f"rss={fmt_mb(startup['mamba']['peak_rss_bytes'])}MB checksum={startup['mamba']['checksum']}")

    startup["python"] = measure_startup_ttfb([args.python, str(hello_fixture)], args.samples)
    record_row(HELLO_SHAPE, "python", startup["python"])
    print(f"  python  ttfb={startup['python']['ttfb_ms']}ms cpu={fmt_ms(startup['python']['cpu_time_ns'])}ms "
          f"rss={fmt_mb(startup['python']['peak_rss_bytes'])}MB checksum={startup['python']['checksum']}")

    if HELLO_SHAPE in go_bins:
        startup["go"] = measure_startup_ttfb([str(go_bins[HELLO_SHAPE])], args.samples)
        record_row(HELLO_SHAPE, "go", startup["go"])
        print(f"  go      ttfb={startup['go']['ttfb_ms']}ms cpu={fmt_ms(startup['go']['cpu_time_ns'])}ms "
              f"rss={fmt_mb(startup['go']['peak_rss_bytes'])}MB checksum={startup['go']['checksum']}")

    # --- 6 server-shaped workloads -------------------------------------------
    for shape in shapes:
        print(f"== {shape} == samples={args.samples}")
        fixture = FIXTURES_DIR / f"{shape}.py"
        results: dict[str, dict] = {}

        results["mamba"] = measure_shape([str(mamba_bin), "run", str(fixture)], args.samples)
        record_row(shape, "mamba", results["mamba"])
        print(f"  mamba   cpu={fmt_ms(results['mamba']['cpu_time_ns'])}ms "
              f"rss={fmt_mb(results['mamba']['peak_rss_bytes'])}MB checksum={results['mamba']['checksum']}")

        results["python"] = measure_shape([args.python, str(fixture)], args.samples)
        record_row(shape, "python", results["python"])
        print(f"  python  cpu={fmt_ms(results['python']['cpu_time_ns'])}ms "
              f"rss={fmt_mb(results['python']['peak_rss_bytes'])}MB checksum={results['python']['checksum']}")

        if shape in go_bins:
            results["go"] = measure_shape([str(go_bins[shape])], args.samples)
            record_row(shape, "go", results["go"])
            print(f"  go      cpu={fmt_ms(results['go']['cpu_time_ns'])}ms "
                  f"rss={fmt_mb(results['go']['peak_rss_bytes'])}MB checksum={results['go']['checksum']}")

        checksums = {r["checksum"] for r in results.values() if r["checksum"] is not None}
        results["_correctness_ok"] = len(checksums) <= 1
        if not results["_correctness_ok"]:
            print(f"  ** CHECKSUM MISMATCH across runtimes for {shape}: "
                  f"{ {k: v['checksum'] for k, v in results.items() if isinstance(v, dict)} }", file=sys.stderr)

        shape_results[shape] = results

    if go_tmp_dir is not None:
        go_tmp_dir.cleanup()

    # --- write JSONL ----------------------------------------------------------
    out_jsonl = Path(args.out_jsonl)
    out_jsonl.parent.mkdir(parents=True, exist_ok=True)
    with out_jsonl.open("w") as fh:
        for row in jsonl_rows:
            fh.write(json.dumps(row, sort_keys=True))
            fh.write("\n")
    print(f"wrote {len(jsonl_rows)} rows -> {out_jsonl}")

    # --- geomean ratios vs the epic bar ---------------------------------------
    cpu_ratios_vs_go: list[float] = []
    rss_ratios_vs_go: list[float] = []
    cpu_ratios_vs_py: list[float] = []
    for shape, results in shape_results.items():
        mb = results["mamba"]
        if "go" in results and mb["cpu_time_ns"] and results["go"]["cpu_time_ns"]:
            cpu_ratios_vs_go.append(mb["cpu_time_ns"] / results["go"]["cpu_time_ns"])
        if "go" in results and mb["peak_rss_bytes"] and results["go"]["peak_rss_bytes"]:
            rss_ratios_vs_go.append(mb["peak_rss_bytes"] / results["go"]["peak_rss_bytes"])
        if mb["cpu_time_ns"] and results["python"]["cpu_time_ns"]:
            cpu_ratios_vs_py.append(mb["cpu_time_ns"] / results["python"]["cpu_time_ns"])

    cpu_geomean_go = geomean(cpu_ratios_vs_go)
    rss_geomean_go = geomean(rss_ratios_vs_go)
    cpu_geomean_py = geomean(cpu_ratios_vs_py)
    startup_ms = startup["mamba"]["ttfb_ms"]

    # --- markdown report --------------------------------------------------------
    lines: list[str] = []
    lines.append("# go_suite baseline report")
    lines.append("")
    lines.append(f"- captured: {time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(captured_at))} (host `{host}`)")
    lines.append(f"- platform: {plat}")
    lines.append(f"- mamba: `{mamba_bin}`")
    lines.append(f"- python: `{args.python}`")
    lines.append(f"- go: {go_version if go_version else 'NOT AVAILABLE on this host -- Go columns omitted below'}")
    lines.append(f"- samples per shape/runtime: {args.samples}")
    lines.append("")
    lines.append("## Epic #1071 bar")
    lines.append("")
    lines.append("| Metric | Bar | Measured | Verdict |")
    lines.append("|---|---|---|---|")
    if cpu_geomean_go is not None:
        verdict = "PASS" if cpu_geomean_go <= EPIC_CPU_BAR else "FAIL"
        lines.append(f"| CPU geomean vs Go | <= {EPIC_CPU_BAR}x | {fmt_ratio(cpu_geomean_go)} | {verdict} |")
    else:
        lines.append(f"| CPU geomean vs Go | <= {EPIC_CPU_BAR}x | n/a (no Go toolchain) | UNEVALUATED |")
    startup_verdict = "PASS" if (startup_ms is not None and startup_ms < EPIC_STARTUP_MS_BAR) else "FAIL"
    lines.append(f"| Startup (time-to-first-output) | < {EPIC_STARTUP_MS_BAR}ms | {startup_ms}ms | {startup_verdict} |")
    if rss_geomean_go is not None:
        verdict = "PASS" if rss_geomean_go <= EPIC_RSS_BAR else "FAIL"
        lines.append(f"| Peak RSS geomean vs Go | <= {EPIC_RSS_BAR}x | {fmt_ratio(rss_geomean_go)} | {verdict} |")
    else:
        lines.append(f"| Peak RSS geomean vs Go | <= {EPIC_RSS_BAR}x | n/a (no Go toolchain) | UNEVALUATED |")
    lines.append("")
    if cpu_geomean_go is None:
        lines.append(
            f"Fallback (informational only, NOT the epic bar): mamba/CPython CPU geomean = "
            f"{fmt_ratio(cpu_geomean_py)}."
        )
        lines.append("")

    lines.append("## Per-shape table")
    lines.append("")
    header = "| shape | mamba cpu (ms) | go cpu (ms) | py cpu (ms) | cpu vs go | mamba rss (MB) | go rss (MB) | rss vs go | checksums match |"
    sep = "|---|---|---|---|---|---|---|---|---|"
    lines.append(header)
    lines.append(sep)
    for shape in shapes:
        r = shape_results[shape]
        mb = r["mamba"]
        py = r["python"]
        go_r = r.get("go")
        cpu_ratio = (mb["cpu_time_ns"] / go_r["cpu_time_ns"]) if (go_r and mb["cpu_time_ns"] and go_r["cpu_time_ns"]) else None
        rss_ratio = (mb["peak_rss_bytes"] / go_r["peak_rss_bytes"]) if (go_r and mb["peak_rss_bytes"] and go_r["peak_rss_bytes"]) else None
        match = "yes" if r["_correctness_ok"] else "**MISMATCH**"
        lines.append(
            f"| {shape} | {fmt_ms(mb['cpu_time_ns'])} | "
            f"{fmt_ms(go_r['cpu_time_ns']) if go_r else 'n/a'} | {fmt_ms(py['cpu_time_ns'])} | "
            f"{fmt_ratio(cpu_ratio)} | {fmt_mb(mb['peak_rss_bytes'])} | "
            f"{fmt_mb(go_r['peak_rss_bytes']) if go_r else 'n/a'} | {fmt_ratio(rss_ratio)} | {match} |"
        )
    lines.append("")
    lines.append("## Startup (hello shape, time-to-first-output)")
    lines.append("")
    lines.append("| runtime | ttfb (ms) | cpu (ms) | rss (MB) |")
    lines.append("|---|---|---|---|")
    for rt in ("mamba", "python", "go"):
        if rt not in startup:
            continue
        s = startup[rt]
        lines.append(f"| {rt} | {s['ttfb_ms']} | {fmt_ms(s['cpu_time_ns'])} | {fmt_mb(s['peak_rss_bytes'])} |")
    lines.append("")

    out_md = Path(args.out_md)
    out_md.parent.mkdir(parents=True, exist_ok=True)
    out_md.write_text("\n".join(lines) + "\n")
    print(f"wrote report -> {out_md}")

    any_mismatch = any(not r["_correctness_ok"] for r in shape_results.values())
    return 1 if any_mismatch else 0


if __name__ == "__main__":
    raise SystemExit(main())
