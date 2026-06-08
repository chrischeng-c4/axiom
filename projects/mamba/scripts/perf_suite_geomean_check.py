#!/usr/bin/env python3
"""MVP performance 10× suite geomean checker (closes #2569).

Parent: #2530 (performance gate suite).

Reads a baseline JSON (default `projects/mamba/baseline.json`) and the
performance profile manifest (default
`validation/profiles/performance.toml`, #2815) and enforces the suite
average rule:

    geometric_mean(speedup_vs_cpython for required benchmarks) >= 10.0

Geometric mean (not arithmetic) is the correct average for speedup
ratios — arithmetic mean over-weights a single high-speedup outlier and
masks broad regressions, while geomean treats `2×` and `0.5×` as exact
inverses. This is the same averaging method declared by
`[policy].suite_average_method = "geometric_mean"` in performance.toml.

Acceptance (issue #2569):

    1. Required suite geomean below 10.0 fails. The summary names the
       worst-N contributors so a worker knows where to attack.
    2. Required suite geomean at or above 10.0 passes.
    3. Summary names benchmark count, averaging method, and CPython
       version (read from `runtime_identity.required_cpython`).

Operating modes
---------------

`--format text` (default)
    Human-readable line per contributor on failure.

`--format json`
    {
      "schema_version": 1,
      "suite_average_floor": 10.0,
      "method": "geometric_mean",
      "cpython_version": "3.12",
      "actual": <geomean>,
      "checked_count": <int>,
      "worst_contributors": [
        {"name": "...", "speedup": 0.001, "bucket": "required"}, ...
      ],
      "exit_code": 0|1
    }

Exit codes
----------

    0   suite geomean satisfies the floor.
    1   suite geomean below floor.
    100 usage / argument error.
    101 baseline or policy file missing or unparseable.

Pure stdlib (Python 3.11+ for tomllib).
"""

from __future__ import annotations

import argparse
import json
import math
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


SCHEMA_VERSION = 1
EXIT_USAGE = 100
EXIT_IO = 101
EXIT_FAIL = 1

DEFAULT_FLOOR = 10.0
DEFAULT_METHOD = "geometric_mean"
DEFAULT_REQUIRED_BUCKETS = ("required",)
DEFAULT_CPYTHON = "3.12"
WORST_CONTRIBUTOR_COUNT = 5


@dataclass
class Bench:
    name: str
    bucket: str
    speedup: float


def _die(code: int, msg: str) -> None:
    sys.stderr.write(f"perf_suite_geomean_check: {msg}\n")
    sys.exit(code)


def _load_baseline(path: Path) -> dict[str, Any]:
    if not path.is_file():
        _die(EXIT_IO, f"baseline missing: {path}")
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        _die(EXIT_IO, f"baseline invalid JSON ({exc}): {path}")
        return {}


def _load_policy(path: Path) -> tuple[float, str, tuple[str, ...], str]:
    if not path.is_file():
        return DEFAULT_FLOOR, DEFAULT_METHOD, DEFAULT_REQUIRED_BUCKETS, DEFAULT_CPYTHON
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    policy = data.get("policy") or {}
    runtime = data.get("runtime_identity") or {}
    floor = float(policy.get("suite_average_floor", DEFAULT_FLOOR))
    method = str(policy.get("suite_average_method", DEFAULT_METHOD))
    average_includes = policy.get("average_includes")
    if isinstance(average_includes, list) and average_includes:
        required = tuple(average_includes)
    else:
        required = tuple(policy.get("release_required_buckets") or DEFAULT_REQUIRED_BUCKETS)
    cpython = str(runtime.get("required_cpython", DEFAULT_CPYTHON))
    return floor, method, required, cpython


def _collect_required(
    baseline: dict[str, Any],
    required_buckets: tuple[str, ...],
) -> list[Bench]:
    out: list[Bench] = []
    benches = baseline.get("benchmarks") or []
    if not isinstance(benches, list):
        _die(EXIT_IO, "baseline 'benchmarks' must be a list")
    for b in benches:
        if not isinstance(b, dict):
            continue
        speedup = b.get("speedup_vs_cpython")
        if not isinstance(speedup, (int, float)):
            continue
        # `tier` (baseline.json v2, #2566) is canonical. Fall back to
        # `bucket` for synthetic fixtures / legacy callers.
        bucket = str(b.get("tier", b.get("bucket", "required")))
        if bucket not in required_buckets:
            continue
        name = str(b.get("name", "<unnamed>"))
        out.append(Bench(name, bucket, float(speedup)))
    return out


def _compute(method: str, items: list[Bench]) -> float:
    if not items:
        # Empty required set is treated as 0.0× so the gate fails loud
        # rather than declaring a vacuous pass.
        return 0.0
    if method == "geometric_mean":
        log_sum = 0.0
        for b in items:
            if b.speedup <= 0:
                # log of zero/negative is undefined; clamp to a tiny
                # positive so the geomean still produces a finite, very
                # small number rather than crashing.
                log_sum += math.log(1e-12)
            else:
                log_sum += math.log(b.speedup)
        return math.exp(log_sum / len(items))
    if method == "arithmetic_mean":
        return sum(b.speedup for b in items) / len(items)
    _die(EXIT_USAGE, f"unknown averaging method: {method!r}")
    return 0.0


def _worst_contributors(items: list[Bench], n: int) -> list[Bench]:
    return sorted(items, key=lambda b: (b.speedup, b.name))[:n]


def _format_text(
    floor: float,
    method: str,
    cpython: str,
    actual: float,
    items: list[Bench],
    worst: list[Bench],
    exit_code: int,
) -> str:
    lines = [
        f"perf_suite_geomean_check: cpython={cpython} method={method} "
        f"floor={floor:.2f}× actual={actual:.4f}× checked={len(items)}"
    ]
    if exit_code != 0:
        lines.append("suite below floor — worst contributors:")
        for b in worst:
            lines.append(
                f"  name={b.name:<32} bucket={b.bucket:<8} "
                f"speedup={b.speedup:.4f}×"
            )
        lines.append(
            f"rule: geomean of required benchmark speedups must be ≥ "
            f"{floor:.1f}× (#2569)"
        )
    else:
        lines.append("perf_suite_geomean_check: clean")
    return "\n".join(lines) + "\n"


def _format_json(
    floor: float,
    method: str,
    cpython: str,
    actual: float,
    items: list[Bench],
    worst: list[Bench],
    exit_code: int,
) -> str:
    payload = {
        "schema_version": SCHEMA_VERSION,
        "suite_average_floor": floor,
        "method": method,
        "cpython_version": cpython,
        "actual": actual,
        "checked_count": len(items),
        "worst_contributors": [
            {"name": b.name, "speedup": b.speedup, "bucket": b.bucket}
            for b in worst
        ],
        "exit_code": exit_code,
    }
    return json.dumps(payload, indent=2, sort_keys=True) + "\n"


def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="perf_suite_geomean_check",
        description="Enforce 10× suite geomean over required benchmarks (#2569).",
    )
    p.add_argument("--baseline", type=Path, default=None,
                   help="path to baseline.json (default: projects/mamba/baseline.json)")
    p.add_argument("--policy", type=Path, default=None,
                   help="path to performance.toml (default: relative to this script)")
    p.add_argument("--format", choices=("text", "json"), default="text")
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    ns = _parse_args(list(sys.argv[1:] if argv is None else argv))

    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    baseline_path = ns.baseline or (project_root / "baseline.json")
    policy_path = ns.policy or (
        project_root / "validation" / "profiles" / "performance.toml"
    )

    floor, method, required_buckets, cpython = _load_policy(policy_path.resolve())
    baseline = _load_baseline(baseline_path.resolve())
    items = _collect_required(baseline, required_buckets)
    actual = _compute(method, items)
    exit_code = EXIT_FAIL if actual < floor else 0
    worst = _worst_contributors(items, WORST_CONTRIBUTOR_COUNT) if exit_code else []

    if ns.format == "json":
        sys.stdout.write(_format_json(floor, method, cpython, actual, items, worst, exit_code))
    else:
        sys.stderr.write(_format_text(floor, method, cpython, actual, items, worst, exit_code))
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
