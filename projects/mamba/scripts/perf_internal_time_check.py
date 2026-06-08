#!/usr/bin/env python3
"""MVP performance internal-time marker checker (closes #2570).

Parent: #2530 (performance gate suite).

Reads `validation/perf_benchmark_manifest.toml` (#2567) and validates
that every required benchmark declares an internal timing mode and
that the fixture file actually emits an `INTERNAL_TIME_NS=<u64>` line
parsable by the cross-runtime bench harness.

Why this gate exists
--------------------

The cross-runtime bench reads `INTERNAL_TIME_NS=<u64>` from a
fixture's stderr first, then stdout (see `parse_internal_time_ns` in
`benches/3p/cross_runtime.rs`). Without that marker the only signal
available is `/usr/bin/time` process wall, which lumps CPython /
mamba startup into the headline number — for 10 ms — 1 s workloads
startup is 10×–100× the inner-loop cost, so the gate would measure
the runtime's startup, not its execution.

Acceptance (issue #2570):

    1. Required fixture without timing metadata fails validation.
       A required entry whose `timing_mode` is missing OR not
       `"internal"` exits 1. The required entry's fixture file must
       also contain a literal `INTERNAL_TIME_NS=` line so the marker
       cannot rot out from underneath the metadata.
    2. Summary distinguishes internal timing from process-wall timing.
       JSON output groups entries into `internal_timed` and
       `process_wall_timed` arrays; text output prints both counts.
    3. Existing exploratory fixtures may remain process-wall timed.
       Exploratory entries with `timing_mode = "process_wall"` are
       acceptable and do not gate.

Operating modes
---------------

`--format text` (default)
    Per-fixture status line to stderr; exit reflects pass/fail.

`--format json`
    {
      "schema_version": 1,
      "manifest_path": "...",
      "checked_count": N,
      "internal_timed": [{"id": ..., "fixture": ..., "tier": ...}, ...],
      "process_wall_timed": [...],
      "violations": [
        {"id": ..., "tier": "required", "reason": "missing timing_mode"},
        {"id": ..., "tier": "required", "reason": "timing_mode=process_wall"},
        {"id": ..., "tier": "required", "reason": "fixture lacks INTERNAL_TIME_NS marker"},
        ...
      ],
      "exit_code": 0|1
    }

`--manifest PATH`
    Override the default
    `projects/mamba/validation/perf_benchmark_manifest.toml`.

Exit codes
----------

    0   every required benchmark declares internal timing and emits
        the INTERNAL_TIME_NS marker.
    1   one or more required benchmarks fail the rule.
    100 usage / argument error.
    101 manifest file missing or unparseable.

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

REQUIRED_TIER = "required"
EXPLORATORY_TIER = "exploratory"
INTERNAL_MODE = "internal"
PROCESS_WALL_MODE = "process_wall"
MARKER = "INTERNAL_TIME_NS="


@dataclass
class Entry:
    id: str
    fixture: str
    tier: str
    timing_mode: str | None
    resolved_path: Path
    has_marker: bool


@dataclass
class Violation:
    id: str
    tier: str
    reason: str


def _die(code: int, msg: str) -> None:
    sys.stderr.write(f"perf_internal_time_check: {msg}\n")
    sys.exit(code)


def _load(path: Path) -> dict[str, Any]:
    if not path.is_file():
        _die(EXIT_IO, f"manifest missing: {path}")
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as exc:
        _die(EXIT_IO, f"manifest invalid TOML ({exc}): {path}")
        return {}


def _collect(data: dict[str, Any], manifest_dir: Path) -> list[Entry]:
    fixture_root_raw = data.get("fixture_root")
    if not isinstance(fixture_root_raw, str) or not fixture_root_raw:
        _die(EXIT_IO, "manifest missing 'fixture_root'")
        return []
    fixture_root = (manifest_dir / fixture_root_raw).resolve()
    entries_raw = data.get("benchmarks") or []
    out: list[Entry] = []
    for e in entries_raw:
        if not isinstance(e, dict):
            continue
        id_ = str(e.get("id", ""))
        fixture = str(e.get("fixture", ""))
        tier = str(e.get("tier", ""))
        tm_raw = e.get("timing_mode")
        timing_mode = str(tm_raw) if isinstance(tm_raw, str) else None
        resolved = (fixture_root / fixture).resolve()
        has_marker = False
        try:
            text = resolved.read_text(encoding="utf-8")
            has_marker = MARKER in text
        except (OSError, UnicodeDecodeError):
            has_marker = False
        out.append(Entry(id_, fixture, tier, timing_mode, resolved, has_marker))
    return out


def _validate(entries: list[Entry]) -> list[Violation]:
    violations: list[Violation] = []
    for e in entries:
        if e.tier != REQUIRED_TIER:
            continue
        if e.timing_mode is None:
            violations.append(Violation(e.id, e.tier, "missing timing_mode"))
            continue
        if e.timing_mode != INTERNAL_MODE:
            violations.append(Violation(
                e.id, e.tier,
                f"timing_mode={e.timing_mode!r} (required must be 'internal')",
            ))
            continue
        if not e.has_marker:
            violations.append(Violation(
                e.id, e.tier,
                f"fixture lacks {MARKER} marker",
            ))
    return violations


def _split_cohorts(entries: list[Entry]) -> tuple[list[Entry], list[Entry]]:
    internal_timed = [e for e in entries if e.timing_mode == INTERNAL_MODE]
    process_wall_timed = [e for e in entries if e.timing_mode == PROCESS_WALL_MODE]
    internal_timed.sort(key=lambda e: e.id)
    process_wall_timed.sort(key=lambda e: e.id)
    return internal_timed, process_wall_timed


def _format_text(
    manifest_path: Path,
    entries: list[Entry],
    internal_timed: list[Entry],
    process_wall_timed: list[Entry],
    violations: list[Violation],
) -> str:
    lines = [
        f"perf_internal_time_check: manifest={manifest_path}",
        f"  checked={len(entries)} internal_timed={len(internal_timed)} "
        f"process_wall_timed={len(process_wall_timed)} violations={len(violations)}",
    ]
    if internal_timed:
        lines.append("internal_timed (gate cohort, INTERNAL_TIME_NS markers):")
        for e in internal_timed:
            marker = "OK" if e.has_marker else "MISSING-MARKER"
            lines.append(
                f"  id={e.id:<24} fixture={e.fixture:<24} tier={e.tier:<11} {marker}"
            )
    if process_wall_timed:
        lines.append("process_wall_timed (legacy /usr/bin/time cohort):")
        for e in process_wall_timed:
            lines.append(
                f"  id={e.id:<24} fixture={e.fixture:<24} tier={e.tier}"
            )
    if violations:
        lines.append("violations (required entries failing the gate):")
        for v in violations:
            lines.append(f"  id={v.id:<24} tier={v.tier:<11} reason={v.reason}")
        lines.append(
            "rule: required entries must declare timing_mode='internal' and "
            "their fixture must emit INTERNAL_TIME_NS=<u64> (#2570)"
        )
    else:
        lines.append("perf_internal_time_check: clean")
    return "\n".join(lines) + "\n"


def _format_json(
    manifest_path: Path,
    entries: list[Entry],
    internal_timed: list[Entry],
    process_wall_timed: list[Entry],
    violations: list[Violation],
    exit_code: int,
) -> str:
    def row(e: Entry) -> dict[str, Any]:
        return {
            "id": e.id,
            "fixture": e.fixture,
            "tier": e.tier,
            "timing_mode": e.timing_mode,
            "resolved_path": str(e.resolved_path),
            "has_marker": e.has_marker,
        }
    payload = {
        "schema_version": SCHEMA_VERSION,
        "manifest_path": str(manifest_path),
        "checked_count": len(entries),
        "internal_timed": [row(e) for e in internal_timed],
        "process_wall_timed": [row(e) for e in process_wall_timed],
        "violations": [
            {"id": v.id, "tier": v.tier, "reason": v.reason} for v in violations
        ],
        "exit_code": exit_code,
    }
    return json.dumps(payload, indent=2, sort_keys=True) + "\n"


def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="perf_internal_time_check",
        description="Require internal-time markers on accepted perf fixtures (#2570).",
    )
    p.add_argument(
        "--manifest",
        type=Path,
        default=None,
        help="path to perf_benchmark_manifest.toml "
             "(default: projects/mamba/validation/perf_benchmark_manifest.toml)",
    )
    p.add_argument("--format", choices=("text", "json"), default="text")
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    ns = _parse_args(list(sys.argv[1:] if argv is None else argv))

    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    default_path = project_root / "validation" / "perf_benchmark_manifest.toml"
    manifest_path = (ns.manifest or default_path).resolve()
    manifest_dir = manifest_path.parent

    data = _load(manifest_path)
    entries = _collect(data, manifest_dir)
    internal_timed, process_wall_timed = _split_cohorts(entries)
    violations = _validate(entries)
    exit_code = EXIT_FAIL if violations else 0

    if ns.format == "json":
        sys.stdout.write(
            _format_json(
                manifest_path, entries, internal_timed, process_wall_timed,
                violations, exit_code,
            )
        )
    else:
        sys.stderr.write(
            _format_text(
                manifest_path, entries, internal_timed, process_wall_timed,
                violations,
            )
        )
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
