#!/usr/bin/env python3
"""MVP performance baseline validator (closes #2566).

Parent: #2530 (performance gate suite).

Walks `projects/mamba/baseline.json` and validates that every benchmark
entry carries the tier metadata downstream tools (#2565 floor, #2569
suite geomean, #2820 release summary) depend on.

Acceptance (issue #2566):

    1. Baseline validation fails for missing tier metadata. Every entry
       must declare `tier ∈ {required, optional, xfail, blocker}`.
    2. Required benchmark entries can be selected without string matching
       fixture names — the validator exposes `--select-tier required`
       which prints the matching names one per line, suitable for piping
       into a downstream tool.
    3. Existing baseline data remains readable by current tools — old
       v1 baselines (no tier field) load without crashing under
       `--legacy-compat`; the legacy-compat path reports drift rather
       than blocking.

Per-tier required fields (acceptance #1 expansion):

    required tier   : numeric `mamba_ns`, numeric `cpython_ns`,
                      numeric `speedup_vs_cpython`.
    other tiers     : `mamba_ns` + `cpython_ns` required; speedup may be
                      absent (e.g. xfail entries that intentionally do
                      not score).

Operating modes
---------------

`--format text` (default)
    Human-scannable errors / drift, one per line.

`--format json`
    {
      "schema_version": 1,
      "baseline_version": <int>,
      "errors": [...],
      "selected": [...] | null,
      "entry_count": <int>,
      "by_tier": {"required": N, "blocker": M, ...},
      "exit_code": <int>
    }

`--select-tier TIER`
    Print one entry name per line that matches TIER. No string matching
    on fixture names anywhere — selection is by tier only (acceptance #2).

`--legacy-compat`
    Tolerate missing `tier` fields and report them as drift (warnings)
    rather than errors.

Exit codes
----------

    0   baseline valid; every entry has tier and required fields.
    1   one or more validation errors.
    100 usage / argument error.
    101 baseline file missing or unparseable.

Pure stdlib (Python 3.11+).
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


SCHEMA_VERSION = 1
EXIT_USAGE = 100
EXIT_IO = 101
EXIT_FAIL = 1

LEGAL_TIERS = ("required", "optional", "xfail", "blocker")
REQUIRED_TIER = "required"


@dataclass
class EntryError:
    entry_index: int
    entry_name: str
    message: str


def _die(code: int, msg: str) -> None:
    sys.stderr.write(f"baseline_validator: {msg}\n")
    sys.exit(code)


def _load_baseline(path: Path) -> dict[str, Any]:
    if not path.is_file():
        _die(EXIT_IO, f"baseline missing: {path}")
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        _die(EXIT_IO, f"baseline invalid JSON ({exc}): {path}")
        return {}


def _validate(
    data: dict[str, Any], legacy_compat: bool
) -> tuple[list[EntryError], list[str], dict[str, int]]:
    """Return (errors, drift_warnings, per_tier_counts)."""
    errors: list[EntryError] = []
    drift: list[str] = []
    by_tier: dict[str, int] = {t: 0 for t in LEGAL_TIERS}
    entries = data.get("benchmarks") or []
    if not isinstance(entries, list):
        _die(EXIT_IO, "baseline 'benchmarks' must be a list")
    seen_names: set[str] = set()
    for idx, e in enumerate(entries):
        if not isinstance(e, dict):
            errors.append(EntryError(idx, "<non-table>", "entry is not a JSON object"))
            continue
        name = str(e.get("name", "<no-name>"))
        if name in seen_names:
            errors.append(EntryError(idx, name, f"duplicate name {name!r}"))
        seen_names.add(name)
        tier_raw = e.get("tier")
        if tier_raw is None:
            if legacy_compat:
                drift.append(f"  [{idx}] {name}: missing 'tier' (legacy-compat: treating as 'required')")
                tier = REQUIRED_TIER
            else:
                errors.append(EntryError(idx, name, "missing required field 'tier'"))
                continue
        else:
            tier = str(tier_raw)
            if tier not in LEGAL_TIERS:
                errors.append(EntryError(
                    idx, name,
                    f"tier {tier!r} not in {list(LEGAL_TIERS)}",
                ))
                continue
        by_tier[tier] = by_tier.get(tier, 0) + 1

        # Timings: required entries must carry numeric mamba_ns,
        # cpython_ns, and speedup_vs_cpython so downstream gates have
        # ground truth. Non-required entries may omit speedup.
        #
        # Baseline v3 (see schema_notes_v3) adds memory-only seam
        # entries: a non-required entry that carries `mem_status` but no
        # timing fields records a peak-RSS sample without a perf gate,
        # so the ns requirement does not apply to it.
        baseline_version = int(data.get("version", 1))
        memory_only_seam = (
            baseline_version >= 3
            and tier != REQUIRED_TIER
            and "mem_status" in e
            and "mamba_ns" not in e
            and "cpython_ns" not in e
        )
        if not memory_only_seam:
            for fld in ("mamba_ns", "cpython_ns"):
                v = e.get(fld)
                if not isinstance(v, (int, float)):
                    errors.append(EntryError(
                        idx, name,
                        f"missing numeric {fld!r} (tier={tier!r})",
                    ))
        if tier == REQUIRED_TIER:
            v = e.get("speedup_vs_cpython")
            if not isinstance(v, (int, float)):
                errors.append(EntryError(
                    idx, name,
                    "required tier must carry numeric 'speedup_vs_cpython'",
                ))
    return errors, drift, by_tier


def _select(data: dict[str, Any], tier: str) -> list[str]:
    out: list[str] = []
    for e in data.get("benchmarks") or []:
        if isinstance(e, dict) and str(e.get("tier", "")) == tier:
            out.append(str(e.get("name", "<no-name>")))
    return sorted(out)


def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="baseline_validator",
        description="Validate baseline.json tier metadata (#2566).",
    )
    p.add_argument("--baseline", type=Path, default=None,
                   help="path to baseline.json (default: projects/mamba/baseline.json)")
    p.add_argument("--format", choices=("text", "json"), default="text")
    p.add_argument("--legacy-compat", action="store_true",
                   help="tolerate missing 'tier' fields (report as drift, not error)")
    p.add_argument("--select-tier", type=str, default=None,
                   help="print one entry name per line matching this tier")
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    ns = _parse_args(list(sys.argv[1:] if argv is None else argv))

    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    path = ns.baseline or (project_root / "baseline.json")
    data = _load_baseline(path.resolve())

    if ns.select_tier is not None:
        if ns.select_tier not in LEGAL_TIERS:
            _die(EXIT_USAGE,
                 f"--select-tier {ns.select_tier!r} not in {list(LEGAL_TIERS)}")
        for name in _select(data, ns.select_tier):
            sys.stdout.write(name + "\n")
        return 0

    errors, drift, by_tier = _validate(data, ns.legacy_compat)
    exit_code = EXIT_FAIL if errors else 0

    if ns.format == "json":
        payload = {
            "schema_version": SCHEMA_VERSION,
            "baseline_version": int(data.get("version", 1)),
            "errors": [
                {"entry_index": e.entry_index, "entry_name": e.entry_name,
                 "message": e.message}
                for e in errors
            ],
            "drift": drift,
            "entry_count": len(data.get("benchmarks") or []),
            "by_tier": by_tier,
            "exit_code": exit_code,
        }
        sys.stdout.write(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    else:
        if drift:
            sys.stderr.write("drift (non-blocking under --legacy-compat):\n")
            for d in drift:
                sys.stderr.write(d + "\n")
        for e in errors:
            sys.stderr.write(f"  [{e.entry_index}] {e.entry_name}: {e.message}\n")
        if errors:
            sys.stderr.write(
                f"baseline_validator: {len(errors)} error(s); "
                f"by_tier={by_tier}\n"
            )
        else:
            sys.stderr.write(
                f"baseline_validator: clean; entries={len(data.get('benchmarks') or [])} "
                f"by_tier={by_tier}\n"
            )
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
