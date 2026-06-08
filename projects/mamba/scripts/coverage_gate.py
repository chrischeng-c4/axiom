#!/usr/bin/env python3
"""Coverage regression gate for mamba — child task #3 of #1885.

Compares the total region coverage in a `cargo llvm-cov --json` output
against the frozen baseline in `COVERAGE-BASELINE.md` (68.98% region as
of commit 6edea62d5). Exits non-zero when the PR run drops region
coverage by more than the configured threshold (default 0.5pp).

Usage:
    python3 scripts/coverage_gate.py cov.json
    COVERAGE_ALLOWED_DROP=1 python3 scripts/coverage_gate.py cov.json
    python3 scripts/coverage_gate.py cov.json --baseline 68.98 --max-drop 0.5

The `COVERAGE_ALLOWED_DROP=1` override is the env-var form the CI gate
sets when a PR carries the `coverage-allowed-drop` label.

Exit codes:
    0 — pass (region% within threshold, or override active)
    1 — fail (region% dropped more than --max-drop)
    2 — usage / parse error
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path


BASELINE_REGION_PCT = 68.98  # COVERAGE-BASELINE.md @ 6edea62d5
DEFAULT_MAX_DROP_PP = 0.5


def extract_region_pct(cov_json_path: Path) -> float:
    """Read cargo-llvm-cov JSON and return total region coverage percent."""
    with cov_json_path.open() as f:
        payload = json.load(f)

    # cargo-llvm-cov JSON shape:
    #   { "data": [ { "totals": { "regions": { "percent": <float> } } } ] }
    try:
        data = payload["data"]
        if not data:
            raise KeyError("data[] is empty")
        totals = data[0]["totals"]
        regions = totals["regions"]
        return float(regions["percent"])
    except (KeyError, TypeError, ValueError) as exc:
        raise SystemExit(
            f"coverage_gate: could not parse region percent from "
            f"{cov_json_path}: {exc}"
        ) from exc


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Mamba coverage regression gate (#1885 child task #3).",
    )
    parser.add_argument(
        "cov_json",
        type=Path,
        help="Path to cargo-llvm-cov --json output file.",
    )
    parser.add_argument(
        "--baseline",
        type=float,
        default=BASELINE_REGION_PCT,
        help=f"Baseline region percent (default {BASELINE_REGION_PCT}).",
    )
    parser.add_argument(
        "--max-drop",
        type=float,
        default=DEFAULT_MAX_DROP_PP,
        help=f"Max allowed drop in pp (default {DEFAULT_MAX_DROP_PP}).",
    )
    args = parser.parse_args(argv)

    if not args.cov_json.exists():
        print(
            f"coverage_gate: input file does not exist: {args.cov_json}",
            file=sys.stderr,
        )
        return 2

    current = extract_region_pct(args.cov_json)
    drop = args.baseline - current

    print(f"coverage_gate: baseline region% = {args.baseline:.2f}")
    print(f"coverage_gate: current  region% = {current:.2f}")
    print(f"coverage_gate: delta            = {-drop:+.2f}pp")
    print(f"coverage_gate: threshold        = -{args.max_drop:.2f}pp")

    if drop <= args.max_drop:
        print("coverage_gate: PASS")
        return 0

    if os.environ.get("COVERAGE_ALLOWED_DROP"):
        print(
            "coverage_gate: FAIL (drop exceeds threshold) — "
            "override active via COVERAGE_ALLOWED_DROP env var; "
            "would have rejected this PR without the "
            "`coverage-allowed-drop` label."
        )
        return 0

    print(
        f"coverage_gate: FAIL — region% dropped {drop:.2f}pp "
        f"(threshold {args.max_drop:.2f}pp). "
        f"Add the `coverage-allowed-drop` label to the PR to override.",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
