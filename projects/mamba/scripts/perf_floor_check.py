#!/usr/bin/env python3
"""MVP performance per-benchmark floor checker (closes #2565).

Parent: #2530 (performance gate suite).

Reads a baseline JSON (default `projects/mamba/baseline.json`, the
release-blocking perf source of truth — see #2823) and enforces the
per-benchmark floor rule from `validation/profiles/performance.toml`:

    every required benchmark must report `speedup_vs_cpython >= 1.0`.

Acceptance (issue #2565):

    1. A required benchmark below 1.0 fails the checker.
    2. A blocked benchmark below 1.0 is reported but does not count as
       a pass — surfaced in a dedicated "blocked_below_floor" section
       with the same fields a release reviewer needs.
    3. The checker can run without re-running benchmarks (it reads the
       baseline JSON only — no subprocess fan-out, no perf workload).

Bucket assignment
-----------------

Each benchmark entry MAY carry an optional `bucket` field with one of:
`required`, `optional`, `xfail`, `blocker`. Entries missing the field
default to `required` so that a freshly-recorded benchmark is gated by
default rather than silently slipping into "optional". This matches the
performance profile manifest (#2815) which lists `release_required_buckets
= ["required"]`.

The four-bucket model mirrors every other MVP profile (smoke /
correctness / ecosystem / package_manager / mambalibs / performance) so
the same triage vocabulary applies everywhere.

Operating modes
---------------

`--format text` (default)
    One line per violation to stderr; exit code reflects pass/fail.

`--format json`
    {
      "schema_version": 1,
      "floor": 1.0,
      "violations": [
        {"name": "...", "bucket": "required", "speedup": 0.42,
         "kind": "Workload"},
        ...
      ],
      "blocked_below_floor": [...],   # acceptance #2: reported, not gating
      "checked_count": <int>,
      "exit_code": <int>
    }

Exit codes
----------

    0   every required benchmark satisfies the floor.
    1   one or more required benchmarks below floor.
    100 usage / argument error.
    101 baseline JSON missing or unparseable.

Pure stdlib (Python 3.11+ for tomllib).
"""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


SCHEMA_VERSION = 1
EXIT_USAGE = 100
EXIT_IO = 101
EXIT_FAIL = 1

# Mirrors `validation/profiles/performance.toml` `[policy]`. The script
# falls back to this constant when the policy file is unavailable so the
# checker stays runnable in isolation (e.g. inside a CI container that
# only shipped baseline.json).
DEFAULT_FLOOR = 1.0
RELEASE_REQUIRED_BUCKETS = ("required",)
REPORT_ONLY_BUCKETS = ("blocker", "xfail", "optional")


@dataclass
class Violation:
    name: str
    bucket: str
    speedup: float
    kind: str


def _die(code: int, msg: str) -> None:
    sys.stderr.write(f"perf_floor_check: {msg}\n")
    sys.exit(code)


def _load_baseline(path: Path) -> dict[str, Any]:
    if not path.is_file():
        _die(EXIT_IO, f"baseline missing: {path}")
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        _die(EXIT_IO, f"baseline invalid JSON ({exc}): {path}")
        return {}


def _load_policy(path: Path) -> tuple[float, tuple[str, ...], tuple[str, ...]]:
    """Read floor and bucket lists from performance.toml when present."""
    if not path.is_file():
        return DEFAULT_FLOOR, RELEASE_REQUIRED_BUCKETS, REPORT_ONLY_BUCKETS
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    policy = data.get("policy") or {}
    floor = float(policy.get("per_benchmark_floor", DEFAULT_FLOOR))
    required = tuple(policy.get("release_required_buckets") or RELEASE_REQUIRED_BUCKETS)
    report_only = tuple(policy.get("report_only_buckets") or REPORT_ONLY_BUCKETS)
    return floor, required, report_only


def _collect(
    baseline: dict[str, Any],
    floor: float,
    required_buckets: tuple[str, ...],
    report_only_buckets: tuple[str, ...],
) -> tuple[list[Violation], list[Violation], int]:
    benchmarks = baseline.get("benchmarks") or []
    if not isinstance(benchmarks, list):
        _die(EXIT_IO, "baseline 'benchmarks' must be a list")
    required: list[Violation] = []
    blocked: list[Violation] = []
    checked = 0
    for b in benchmarks:
        if not isinstance(b, dict):
            continue
        # `tier` (baseline.json v2, #2566) is the canonical field.
        # Fall back to `bucket` for synthetic fixtures / legacy callers,
        # then default to "required" so a bare entry is gated by default.
        bucket = str(b.get("tier", b.get("bucket", "required")))
        speedup_raw = b.get("speedup_vs_cpython")
        if not isinstance(speedup_raw, (int, float)):
            continue
        speedup = float(speedup_raw)
        name = str(b.get("name", "<unnamed>"))
        kind = str(b.get("kind", ""))
        if bucket in required_buckets:
            checked += 1
            if speedup < floor:
                required.append(Violation(name, bucket, speedup, kind))
        elif bucket in report_only_buckets:
            if speedup < floor:
                blocked.append(Violation(name, bucket, speedup, kind))
    # Stable order: ascending speedup, then name.
    required.sort(key=lambda v: (v.speedup, v.name))
    blocked.sort(key=lambda v: (v.speedup, v.name))
    return required, blocked, checked


def _format_text(
    floor: float,
    violations: list[Violation],
    blocked: list[Violation],
    checked: int,
) -> str:
    lines = [
        f"perf_floor_check: floor={floor:.2f}× checked={checked} "
        f"violations={len(violations)} blocked_below_floor={len(blocked)}"
    ]
    if violations:
        lines.append("required benchmarks below floor:")
        for v in violations:
            lines.append(
                f"  name={v.name:<32} bucket={v.bucket:<8} "
                f"speedup={v.speedup:.4f}× kind={v.kind}"
            )
    if blocked:
        lines.append("blocked/optional benchmarks below floor (reported, not gating):")
        for v in blocked:
            lines.append(
                f"  name={v.name:<32} bucket={v.bucket:<8} "
                f"speedup={v.speedup:.4f}× kind={v.kind}"
            )
    if violations:
        lines.append(
            "rule: required benchmarks must report speedup_vs_cpython "
            ">= 1.0 (#2565)"
        )
    else:
        lines.append("perf_floor_check: clean")
    return "\n".join(lines) + "\n"


def _format_json(
    floor: float,
    violations: list[Violation],
    blocked: list[Violation],
    checked: int,
    exit_code: int,
) -> str:
    def row(v: Violation) -> dict[str, Any]:
        return {
            "name": v.name,
            "bucket": v.bucket,
            "speedup": v.speedup,
            "kind": v.kind,
        }
    payload = {
        "schema_version": SCHEMA_VERSION,
        "floor": floor,
        "violations": [row(v) for v in violations],
        "blocked_below_floor": [row(v) for v in blocked],
        "checked_count": checked,
        "exit_code": exit_code,
    }
    return json.dumps(payload, indent=2, sort_keys=True) + "\n"


def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="perf_floor_check",
        description="Enforce 1× per-benchmark floor on required benchmarks (#2565).",
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

    floor, required_buckets, report_only_buckets = _load_policy(
        policy_path.resolve()
    )
    baseline = _load_baseline(baseline_path.resolve())
    violations, blocked, checked = _collect(
        baseline, floor, required_buckets, report_only_buckets
    )
    exit_code = EXIT_FAIL if violations else 0

    if ns.format == "json":
        sys.stdout.write(_format_json(floor, violations, blocked, checked, exit_code))
    else:
        sys.stderr.write(_format_text(floor, violations, blocked, checked))
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
