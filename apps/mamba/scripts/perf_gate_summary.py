#!/usr/bin/env python3
"""MVP performance gate machine-readable summary (closes #2573).

Parent: #2530 (performance gate suite).

Stitches together the four release-gate checks defined elsewhere in
this directory:

    #2566  baseline tier metadata        (baseline_validator.py)
    #2565  per-benchmark 1x floor        (perf_floor_check.py)
    #2569  10x suite geomean             (perf_suite_geomean_check.py)
    #2572  CPython 3.12 identity         (cpython_identity_check.py)

And emits one self-describing JSON document a CI worker can pipe
into `jq` without scraping a single log line. The schema is locked
by `tests/mvp_runners/mvp_perf_gate_summary_2573.rs` (dispatched by
the `tests/mvp_runners.rs` umbrella binary).

Acceptance (issue #2573):

    1. CI or worker scripts can parse the JSON without scraping logs.
       Every gate's verdict surfaces as a typed field, not embedded
       in a free-form text section.
    2. Summary includes enough data to identify the slowest blockers.
       `floor_result.slowest_blockers` carries the top-N entries
       below floor; `per_benchmark` carries every entry with its
       speedup, tier, and pass/fail bit.
    3. A regression test covers the JSON shape.
       `tests/mvp_runners/mvp_perf_gate_summary_2573.rs` locks the
       field set + nested schema.

Operating modes
---------------

`--baseline PATH`
    Override baseline.json location.

`--policy PATH`
    Override performance.toml location.

`--cpython-identity-json PATH`
    Pre-collected CPython identity (skips probing python3).

`--local-debug-override`
    Pass-through to cpython_identity_check (records `override_active`
    in the JSON but never silences a floor/geomean violation).

`--format text|json` (default: json — most callers want machine-readable)

Exit codes
----------

    0   every gate passed (or override active for the CPython check
        only — floor / geomean / baseline still gate).
    1   one or more release-blocking gates failed.
    100 usage / argument error.
    101 a referenced file is missing or unparseable.

Pure stdlib (Python 3.11+ for tomllib).
"""

from __future__ import annotations

import argparse
import json
import math
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


SCHEMA_VERSION = 1
EXIT_USAGE = 100
EXIT_IO = 101
EXIT_FAIL = 1

DEFAULT_REQUIRED_BUCKETS = ("required",)
DEFAULT_FLOOR = 1.0
DEFAULT_SUITE_FLOOR = 10.0
DEFAULT_METHOD = "geometric_mean"
DEFAULT_REQUIRED_CPYTHON = "3.12"
SLOWEST_N = 5


@dataclass
class Bench:
    name: str
    kind: str
    tier: str
    mamba_ns: float | None
    cpython_ns: float | None
    speedup: float | None


def _die(code: int, msg: str) -> None:
    sys.stderr.write(f"perf_gate_summary: {msg}\n")
    sys.exit(code)


def _load_json(path: Path) -> dict[str, Any]:
    if not path.is_file():
        _die(EXIT_IO, f"baseline missing: {path}")
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        _die(EXIT_IO, f"baseline invalid JSON ({exc}): {path}")
        return {}


def _load_policy(path: Path) -> dict[str, Any]:
    if not path.is_file():
        return {}
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as exc:
        _die(EXIT_IO, f"policy invalid TOML ({exc}): {path}")
        return {}


def _coerce_float(v: Any) -> float | None:
    if isinstance(v, (int, float)):
        return float(v)
    return None


def _collect(baseline: dict[str, Any]) -> list[Bench]:
    entries = baseline.get("benchmarks") or []
    if not isinstance(entries, list):
        _die(EXIT_IO, "baseline 'benchmarks' must be a list")
    out: list[Bench] = []
    for e in entries:
        if not isinstance(e, dict):
            continue
        name = str(e.get("name", "<unnamed>"))
        kind = str(e.get("kind", ""))
        tier = str(e.get("tier", e.get("bucket", "required")))
        out.append(Bench(
            name=name,
            kind=kind,
            tier=tier,
            mamba_ns=_coerce_float(e.get("mamba_ns")),
            cpython_ns=_coerce_float(e.get("cpython_ns")),
            speedup=_coerce_float(e.get("speedup_vs_cpython")),
        ))
    return out


def _geomean(speedups: list[float]) -> float:
    if not speedups:
        return 0.0
    log_sum = 0.0
    for s in speedups:
        if s <= 0:
            log_sum += math.log(1e-12)
        else:
            log_sum += math.log(s)
    return math.exp(log_sum / len(speedups))


def _probe_cpython(
    cpython_identity_json: Path | None,
    local_debug_override: bool,
    script_dir: Path,
    policy_path: Path,
) -> dict[str, Any]:
    """Call cpython_identity_check.py (in JSON mode) and parse output."""
    cmd = [sys.executable, str(script_dir / "cpython_identity_check.py"),
           "--format", "json"]
    if cpython_identity_json is not None:
        cmd += ["--identity-json", str(cpython_identity_json)]
    if local_debug_override:
        cmd += ["--local-debug-override"]
    cmd += ["--policy", str(policy_path)]
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=15, check=False)
    except (OSError, subprocess.TimeoutExpired) as exc:
        _die(EXIT_IO, f"cpython_identity_check.py probe failed: {exc}")
        return {}
    if proc.returncode == EXIT_IO:
        _die(EXIT_IO, f"cpython_identity_check.py io error: {proc.stderr.strip()}")
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        _die(EXIT_IO, f"cpython_identity_check JSON unparseable ({exc}): {proc.stdout!r}")
        return {}


def _build_summary(
    baseline: dict[str, Any],
    policy: dict[str, Any],
    cpython: dict[str, Any],
) -> tuple[dict[str, Any], int]:
    benches = _collect(baseline)
    by_tier: dict[str, int] = {}
    for b in benches:
        by_tier[b.tier] = by_tier.get(b.tier, 0) + 1

    pol = policy.get("policy") or {}
    floor = float(pol.get("per_benchmark_floor", DEFAULT_FLOOR))
    suite_floor = float(pol.get("suite_average_floor", DEFAULT_SUITE_FLOOR))
    method = str(pol.get("suite_average_method", DEFAULT_METHOD))
    required_buckets = tuple(pol.get("release_required_buckets") or DEFAULT_REQUIRED_BUCKETS)
    average_includes = pol.get("average_includes")
    if isinstance(average_includes, list) and average_includes:
        avg_buckets = tuple(average_includes)
    else:
        avg_buckets = required_buckets

    per_benchmark: list[dict[str, Any]] = []
    floor_violations: list[Bench] = []
    speedups_for_avg: list[float] = []
    pass_count = 0
    fail_count = 0
    required_count = 0
    for b in benches:
        passed_floor: bool | None
        if b.tier in required_buckets and isinstance(b.speedup, float):
            required_count += 1
            passed_floor = b.speedup >= floor
            if passed_floor:
                pass_count += 1
            else:
                fail_count += 1
                floor_violations.append(b)
        else:
            passed_floor = None
        if b.tier in avg_buckets and isinstance(b.speedup, float):
            speedups_for_avg.append(b.speedup)
        per_benchmark.append({
            "name": b.name,
            "kind": b.kind,
            "tier": b.tier,
            "mamba_ns": b.mamba_ns,
            "cpython_ns": b.cpython_ns,
            "speedup_vs_cpython": b.speedup,
            "passed_floor": passed_floor,
        })

    actual_geomean = _geomean(speedups_for_avg)
    suite_passed = actual_geomean >= suite_floor

    floor_violations.sort(key=lambda b: ((b.speedup if b.speedup is not None else 0.0), b.name))
    slowest_blockers = [{
        "name": b.name, "kind": b.kind, "tier": b.tier,
        "speedup_vs_cpython": b.speedup,
        "mamba_ns": b.mamba_ns, "cpython_ns": b.cpython_ns,
    } for b in floor_violations[:SLOWEST_N]]

    cpython_matches = bool(cpython.get("matches", False))
    cpython_override = bool(cpython.get("override_active", False))
    cpython_gate_ok = cpython_matches or cpython_override

    gates = {
        "baseline_version_ok": int(baseline.get("version", 1)) >= 2,
        "cpython_identity_ok": cpython_gate_ok,
        "per_benchmark_floor_ok": fail_count == 0 and required_count > 0,
        "suite_geomean_ok": suite_passed and required_count > 0,
    }
    overall_ok = all(gates.values())
    exit_code = 0 if overall_ok else EXIT_FAIL

    summary = {
        "schema_version": SCHEMA_VERSION,
        "benchmark_count": len(benches),
        "by_tier": by_tier,
        "release_required_buckets": list(required_buckets),
        "pass_count": pass_count,
        "fail_count": fail_count,
        "required_checked_count": required_count,
        "per_benchmark": per_benchmark,
        "floor_result": {
            "threshold": floor,
            "violations_count": len(floor_violations),
            "slowest_blockers": slowest_blockers,
            "passed": gates["per_benchmark_floor_ok"],
        },
        "suite_result": {
            "floor": suite_floor,
            "method": method,
            "actual_geomean": actual_geomean,
            "passed": gates["suite_geomean_ok"],
            "checked_count": len(speedups_for_avg),
        },
        "cpython_identity": {
            "executable": cpython.get("executable", ""),
            "version": cpython.get("version", ""),
            "version_major_minor": cpython.get("version_major_minor", ""),
            "implementation_name": cpython.get("implementation_name", ""),
            "required_cpython": cpython.get("required_cpython", DEFAULT_REQUIRED_CPYTHON),
            "matches": cpython_matches,
            "override_active": cpython_override,
        },
        "gates": gates,
        "overall_passed": overall_ok,
        "exit_code": exit_code,
    }
    return summary, exit_code


def _format_text(summary: dict[str, Any]) -> str:
    g = summary["gates"]
    fr = summary["floor_result"]
    sr = summary["suite_result"]
    ci = summary["cpython_identity"]
    lines = [
        f"perf_gate_summary: overall={'PASS' if summary['overall_passed'] else 'FAIL'} "
        f"benchmarks={summary['benchmark_count']} required={summary['required_checked_count']}",
        f"  gates: baseline={g['baseline_version_ok']} "
        f"cpython={g['cpython_identity_ok']} "
        f"floor={g['per_benchmark_floor_ok']} suite={g['suite_geomean_ok']}",
        f"  floor: threshold={fr['threshold']:.2f}x violations={fr['violations_count']}",
        f"  suite: {sr['method']} actual={sr['actual_geomean']:.4f}x "
        f"floor={sr['floor']:.2f}x checked={sr['checked_count']}",
        f"  cpython: {ci['version_major_minor']} ({ci['implementation_name']}) "
        f"executable={ci['executable']} matches={ci['matches']} "
        f"override_active={ci['override_active']}",
    ]
    if fr["slowest_blockers"]:
        lines.append("  slowest blockers:")
        for b in fr["slowest_blockers"]:
            lines.append(
                f"    name={b['name']} speedup={b['speedup_vs_cpython']:.4f}x tier={b['tier']}"
            )
    return "\n".join(lines) + "\n"


def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="perf_gate_summary",
        description="Emit machine-readable MVP perf gate summary (#2573).",
    )
    p.add_argument("--baseline", type=Path, default=None)
    p.add_argument("--policy", type=Path, default=None)
    p.add_argument("--cpython-identity-json", type=Path, default=None)
    p.add_argument("--local-debug-override", action="store_true")
    p.add_argument("--format", choices=("text", "json"), default="json")
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    ns = _parse_args(list(sys.argv[1:] if argv is None else argv))

    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    baseline_path = (ns.baseline or (project_root / "baseline.json")).resolve()
    policy_path = (
        ns.policy or (project_root / "validation" / "profiles" / "performance.toml")
    ).resolve()

    baseline = _load_json(baseline_path)
    policy = _load_policy(policy_path)
    cpython = _probe_cpython(
        ns.cpython_identity_json.resolve() if ns.cpython_identity_json else None,
        ns.local_debug_override,
        script_dir,
        policy_path,
    )

    summary, exit_code = _build_summary(baseline, policy, cpython)

    if ns.format == "text":
        sys.stderr.write(_format_text(summary))
    else:
        sys.stdout.write(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
