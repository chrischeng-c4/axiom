#!/usr/bin/env python3
"""MVP release-gate skip/xfail/Stub/ImportPass policy checker (closes #2825).

Parent: #2775 (MVP release blocking CI profiles).

Walks a release-gate summary JSON (shape locked by #2820) and enforces
the canonical MVP rule:

    Release-blocking profiles never count `skip`, `xfail`, `Stub`, or
    `ImportPass` as pass.

The rule is already declared in `projects/mamba/validation/mvp.toml`
under `[policy]`:

    release_required_outcomes = ["pass", "AssertionPass"]
    non_passing_outcomes = ["skip", "xfail", "Stub", "ImportPass",
                            "Fail", "Timeout", "blocked"]

This script is the runtime enforcer: it reads a summary, picks out every
release-required profile, and fails the gate if any profile reports a
non-zero count in a non-passing outcome bucket.

Acceptance (issue #2825):

    1. Synthetic summary with a skipped required test fails. Any
       release-required profile with skipped > 0 / xfail > 0 / stub > 0
       / import_pass > 0 exits nonzero, and the offending profile +
       count + outcome label are printed for triage.
    2. Synthetic summary with all AssertionPass items passes. A clean
       summary (only `passed` counts on release-required profiles)
       exits 0.
    3. Failure output is actionable for worker triage. Each violation
       prints `profile / outcome / count` in a stable order, and the
       JSON-format report carries the same fields machine-readably.

Operating modes
---------------

`--format text` (default)
    Print one-violation-per-line to stderr; exit code reflects
    pass/fail.

`--format json`
    Emit a deterministic JSON document:

        {
          "schema_version": 1,
          "release_id": "...",
          "violations": [
            {"profile": "smoke", "outcome": "skipped", "count": 3,
             "objective": "smoke"}
          ],
          "exit_code": 1
        }

Exit codes
----------

    0   no required profile reports a non-passing outcome.
    1   one or more violations.
    100 usage / argument error.
    101 summary file missing or unparseable.

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

# The four outcome buckets every release-required profile must keep at
# zero. Names match the per-profile `counts` keys in the release summary
# schema (#2820). `stub` and `import_pass` use snake_case in the JSON;
# the human-readable rule talks about `Stub` and `ImportPass` — we keep
# the JSON-side naming throughout for unambiguous machine reports.
NON_PASSING_OUTCOMES = ("skipped", "xfail", "stub", "import_pass")


@dataclass
class Violation:
    profile: str
    outcome: str
    count: int
    objective: str


def _die(code: int, msg: str) -> None:
    sys.stderr.write(f"skip_policy_check: {msg}\n")
    sys.exit(code)


def _load_summary(path: Path) -> dict[str, Any]:
    if not path.is_file():
        _die(EXIT_IO, f"summary missing: {path}")
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        _die(EXIT_IO, f"summary invalid JSON ({exc}): {path}")
        return {}


def _load_mvp_policy(path: Path) -> set[str]:
    """Read the canonical non-passing outcome list from mvp.toml.

    Used as a sanity check: if mvp.toml ever drops one of the four
    buckets we enforce, surface that drift rather than silently passing.
    """
    if not path.is_file():
        return set(NON_PASSING_OUTCOMES)
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    policy = data.get("policy") or {}
    raw = policy.get("non_passing_outcomes") or []
    # Normalize human-form names ("Stub", "ImportPass") to the JSON-bucket
    # keys we walk.
    norm: set[str] = set()
    for name in raw:
        if not isinstance(name, str):
            continue
        canonical = name.lower().replace("importpass", "import_pass")
        # The summary schema uses `skipped`; mvp.toml's policy list uses
        # the verb form `skip`. Normalize so the drift check below
        # actually matches the bucket name we enforce.
        if canonical == "skip":
            canonical = "skipped"
        norm.add(canonical)
    return norm


def _collect_violations(summary: dict[str, Any]) -> list[Violation]:
    overall = summary.get("overall") or {}
    required: list[str] = list(overall.get("release_required_profiles") or [])
    profiles = summary.get("profiles") or {}
    violations: list[Violation] = []
    for pid in required:
        entry = profiles.get(pid)
        if not isinstance(entry, dict):
            violations.append(Violation(pid, "missing", 1, pid))
            continue
        counts = entry.get("counts") or {}
        for outcome in NON_PASSING_OUTCOMES:
            value = counts.get(outcome, 0)
            if not isinstance(value, int):
                continue
            if value > 0:
                violations.append(Violation(pid, outcome, value, pid))
    # Stable order: profile alpha, then outcome alpha. Critical for
    # acceptance #3 ("actionable for worker triage") — diff-friendly.
    violations.sort(key=lambda v: (v.profile, v.outcome))
    return violations


def _format_text(release_id: str, violations: list[Violation]) -> str:
    if not violations:
        return f"skip_policy_check: {release_id}: clean (0 violations)\n"
    lines = [f"skip_policy_check: {release_id}: {len(violations)} violation(s)"]
    for v in violations:
        lines.append(
            f"  profile={v.profile:<16} outcome={v.outcome:<12} "
            f"count={v.count}  objective={v.objective}"
        )
    lines.append(
        "rule: release-blocking profiles must report 0 for skip / xfail / "
        "Stub / ImportPass (#2825)"
    )
    return "\n".join(lines) + "\n"


def _format_json(release_id: str, violations: list[Violation], exit_code: int) -> str:
    payload = {
        "schema_version": SCHEMA_VERSION,
        "release_id": release_id,
        "violations": [
            {
                "profile": v.profile,
                "outcome": v.outcome,
                "count": v.count,
                "objective": v.objective,
            }
            for v in violations
        ],
        "exit_code": exit_code,
    }
    return json.dumps(payload, indent=2, sort_keys=True) + "\n"


def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="skip_policy_check",
        description="Enforce skip/xfail/Stub/ImportPass=0 on release-required profiles (#2825).",
    )
    p.add_argument("--summary", type=Path, required=True,
                   help="path to a release-gate summary JSON (#2820)")
    p.add_argument("--mvp-policy", type=Path, default=None,
                   help="path to validation/mvp.toml (default: relative to this script)")
    p.add_argument("--format", choices=("text", "json"), default="text",
                   help="output format (default: text)")
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    ns = _parse_args(list(sys.argv[1:] if argv is None else argv))

    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    mvp_policy_path = ns.mvp_policy or (project_root / "validation" / "mvp.toml")
    canonical_non_passing = _load_mvp_policy(mvp_policy_path.resolve())
    enforced: set[str] = set(NON_PASSING_OUTCOMES)
    drift = enforced - canonical_non_passing
    if drift:
        # Drift means mvp.toml has been pared back; warn but do not
        # block — the script's hard-coded list is the source of truth
        # for what this checker enforces.
        sys.stderr.write(
            "skip_policy_check: warning — outcomes enforced here but not "
            f"listed in mvp.toml non_passing_outcomes: {sorted(drift)}\n"
        )

    summary = _load_summary(ns.summary.resolve())
    violations = _collect_violations(summary)
    exit_code = EXIT_FAIL if violations else 0

    release_id = str(summary.get("release_id", "<unknown>"))
    if ns.format == "json":
        sys.stdout.write(_format_json(release_id, violations, exit_code))
    else:
        sys.stderr.write(_format_text(release_id, violations))
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
