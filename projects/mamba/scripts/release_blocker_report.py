#!/usr/bin/env python3
"""MVP release-gate blocker budget reporter (closes #2822).

Parent: #2775 (MVP release blocking CI profiles).

Consumes a release-gate summary file (the artifact produced by
`scripts/release_gate.py` from #2821; shape locked by #2820's
`release_summary.schema.json`) and renders the blocker rollup against
the per-objective budget declared in
`projects/mamba/validation/blocker_budget.toml`.

Acceptance (issue #2822):

    1. Report lists blocker counts per MVP objective. Both the JSON and
       human reports surface one row per MVP objective with `budget`,
       `actual`, and `overrun` columns.
    2. Missing issue links for required blockers fail or warn according
       to policy. `[issue_link_policy]` decides whether a blocker with
       a null `tracking_issue` exits nonzero or only prints a warning.
    3. Human summary is concise. `[summary]` caps the human output and
       trims zero-blocker objectives unless explicitly included.

Operating modes
---------------

`--format json`
    Emit a deterministic JSON document (default). The shape is fixed:

        {
          "schema_version": 1,
          "release_id": "...",
          "policy": {...},
          "objectives": [
            {"objective": "performance", "budget": 0, "actual": 1,
             "overrun": 1, "blockers": [...], "missing_tracker": 0}
          ],
          "totals": {"budget": 0, "actual": 1, "overrun": 1,
                     "missing_tracker": 0},
          "exit_reason": "budget_overrun" | "missing_tracker" | "clean"
        }

`--format human`
    Emit a single-screen, scannable text report. Capped at
    `[summary].human_max_lines` from the policy file.

`--format both`
    Print human report to stderr and JSON to stdout.

Exit codes
----------

    0   no budget overruns and no policy violation.
    1   any objective is over budget.
    2   a required-bucket blocker is missing a tracking issue AND policy
        is "fail" (overrides budget cleanliness — release cannot ship).
    100 usage / argument error.
    101 summary or policy file missing / unreadable.

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
EXIT_BUDGET_OVERRUN = 1
EXIT_MISSING_TRACKER = 2

REQUIRED_BUDGET_KEYS = {
    "smoke",
    "correctness",
    "performance",
    "ecosystem",
    "package_manager",
    "mambalibs",
    "release_gate",
}


@dataclass
class Policy:
    budgets: dict[str, int]
    required_missing_action: str
    optional_missing_action: str
    human_max_lines: int
    include_zero_objectives: bool
    sort: str


@dataclass
class ObjectiveRow:
    objective: str
    budget: int
    actual: int
    blockers: list[dict[str, Any]]
    missing_tracker: int

    @property
    def overrun(self) -> int:
        return max(self.actual - self.budget, 0)


def _die(code: int, msg: str) -> None:
    sys.stderr.write(f"release_blocker_report: {msg}\n")
    sys.exit(code)


def _load_policy(path: Path) -> Policy:
    if not path.is_file():
        _die(EXIT_IO, f"policy file missing: {path}")
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    budgets = data.get("budgets") or {}
    if not budgets:
        _die(EXIT_IO, f"policy file missing [budgets]: {path}")
    missing = REQUIRED_BUDGET_KEYS - set(budgets.keys())
    if missing:
        _die(EXIT_IO, f"policy [budgets] missing keys: {sorted(missing)}")
    link_policy = data.get("issue_link_policy") or {}
    summary_cfg = data.get("summary") or {}
    return Policy(
        budgets={k: int(v) for k, v in budgets.items()},
        required_missing_action=str(
            link_policy.get("required_blocker_missing_tracker", "warn")
        ),
        optional_missing_action=str(
            link_policy.get("optional_blocker_missing_tracker", "warn")
        ),
        human_max_lines=int(summary_cfg.get("human_max_lines", 25)),
        include_zero_objectives=bool(
            summary_cfg.get("include_zero_objectives", False)
        ),
        sort=str(summary_cfg.get("sort", "overrun_first")),
    )


def _load_summary(path: Path) -> dict[str, Any]:
    if not path.is_file():
        _die(EXIT_IO, f"summary file missing: {path}")
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        _die(EXIT_IO, f"summary file invalid JSON ({exc}): {path}")
        return {}  # unreachable; satisfies the type-checker


def _build_rows(summary: dict[str, Any], policy: Policy) -> list[ObjectiveRow]:
    by_obj = summary.get("blockers_by_objective") or {}
    rows: list[ObjectiveRow] = []
    for objective, budget in policy.budgets.items():
        entries = by_obj.get(objective) or []
        actual = len(entries)
        missing = sum(1 for b in entries if not b.get("tracking_issue"))
        rows.append(
            ObjectiveRow(
                objective=objective,
                budget=budget,
                actual=actual,
                blockers=entries,
                missing_tracker=missing,
            )
        )
    # Surface any objective that appears in the summary but isn't budgeted
    # — that's a policy gap worth seeing in the report.
    for objective, entries in by_obj.items():
        if objective in policy.budgets:
            continue
        rows.append(
            ObjectiveRow(
                objective=objective,
                budget=0,
                actual=len(entries),
                blockers=entries,
                missing_tracker=sum(1 for b in entries if not b.get("tracking_issue")),
            )
        )
    return _sort_rows(rows, policy.sort)


def _sort_rows(rows: list[ObjectiveRow], how: str) -> list[ObjectiveRow]:
    if how == "overrun_first":
        return sorted(
            rows,
            key=lambda r: (-(1 if r.overrun > 0 else 0), -r.actual, r.objective),
        )
    return sorted(rows, key=lambda r: r.objective)


def _is_required_objective(objective: str, summary: dict[str, Any]) -> bool:
    required = (
        summary.get("overall", {}).get("release_required_profiles") or []
    )
    return objective in required or objective == "release_gate"


def _decide_exit(rows: list[ObjectiveRow], policy: Policy, summary: dict[str, Any]) -> tuple[int, str]:
    overrun_total = sum(r.overrun for r in rows)
    missing_total = sum(r.missing_tracker for r in rows)

    required_missing = sum(
        r.missing_tracker
        for r in rows
        if _is_required_objective(r.objective, summary)
    )
    if required_missing > 0 and policy.required_missing_action == "fail":
        return EXIT_MISSING_TRACKER, "missing_tracker"
    if overrun_total > 0:
        return EXIT_BUDGET_OVERRUN, "budget_overrun"
    if missing_total > 0:
        return 0, "missing_tracker_warn"
    return 0, "clean"


def _format_json(
    summary: dict[str, Any],
    policy: Policy,
    rows: list[ObjectiveRow],
    exit_code: int,
    exit_reason: str,
) -> str:
    payload = {
        "schema_version": SCHEMA_VERSION,
        "release_id": summary.get("release_id", ""),
        "policy": {
            "required_blocker_missing_tracker": policy.required_missing_action,
            "optional_blocker_missing_tracker": policy.optional_missing_action,
            "human_max_lines": policy.human_max_lines,
            "include_zero_objectives": policy.include_zero_objectives,
        },
        "objectives": [
            {
                "objective": r.objective,
                "budget": r.budget,
                "actual": r.actual,
                "overrun": r.overrun,
                "missing_tracker": r.missing_tracker,
                "blockers": [
                    {
                        "profile": b.get("profile", ""),
                        "fixture_id": b.get("fixture_id", ""),
                        "kind": b.get("kind", ""),
                        "reason": b.get("reason", ""),
                        "tracking_issue": b.get("tracking_issue"),
                    }
                    for b in r.blockers
                ],
            }
            for r in rows
        ],
        "totals": {
            "budget": sum(r.budget for r in rows),
            "actual": sum(r.actual for r in rows),
            "overrun": sum(r.overrun for r in rows),
            "missing_tracker": sum(r.missing_tracker for r in rows),
        },
        "exit_code": exit_code,
        "exit_reason": exit_reason,
    }
    return json.dumps(payload, indent=2, sort_keys=True) + "\n"


def _format_human(
    summary: dict[str, Any],
    policy: Policy,
    rows: list[ObjectiveRow],
    exit_code: int,
    exit_reason: str,
) -> str:
    lines: list[str] = []
    rel_id = summary.get("release_id", "<unknown>")
    overall_pass = summary.get("overall", {}).get("pass", False)
    header_state = "PASS" if overall_pass else "FAIL"
    lines.append(f"release-gate blocker budget — {rel_id} [{header_state}]")
    lines.append("-" * 60)
    for r in rows:
        if not policy.include_zero_objectives and r.actual == 0 and r.budget == 0:
            continue
        marker = "OVER" if r.overrun > 0 else "ok"
        tracker_note = (
            f", missing-tracker={r.missing_tracker}"
            if r.missing_tracker > 0
            else ""
        )
        lines.append(
            f"  {r.objective:<16} actual={r.actual:>2} / budget={r.budget:>2}"
            f"  [{marker}]{tracker_note}"
        )
    totals_actual = sum(r.actual for r in rows)
    totals_budget = sum(r.budget for r in rows)
    totals_missing = sum(r.missing_tracker for r in rows)
    lines.append("-" * 60)
    lines.append(
        f"totals: actual={totals_actual}, budget={totals_budget}, "
        f"missing-tracker={totals_missing}, exit={exit_code} ({exit_reason})"
    )
    if len(lines) > policy.human_max_lines:
        truncated = lines[: policy.human_max_lines - 1]
        truncated.append(f"  (+{len(lines) - (policy.human_max_lines - 1)} more)")
        lines = truncated
    return "\n".join(lines) + "\n"


def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="release_blocker_report",
        description="Group release-gate blockers by MVP objective (#2822).",
    )
    p.add_argument("--summary", type=Path, required=True,
                   help="path to a release-gate summary JSON (from #2821)")
    p.add_argument("--policy", type=Path, default=None,
                   help="path to blocker_budget.toml (default: relative to this script)")
    p.add_argument("--format", choices=("json", "human", "both"), default="json",
                   help="report format (default: json)")
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    ns = _parse_args(list(sys.argv[1:] if argv is None else argv))

    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    policy_path = ns.policy or (project_root / "validation" / "blocker_budget.toml")
    policy = _load_policy(policy_path.resolve())
    summary = _load_summary(ns.summary.resolve())

    rows = _build_rows(summary, policy)
    exit_code, exit_reason = _decide_exit(rows, policy, summary)

    if ns.format == "json":
        sys.stdout.write(_format_json(summary, policy, rows, exit_code, exit_reason))
    elif ns.format == "human":
        sys.stdout.write(_format_human(summary, policy, rows, exit_code, exit_reason))
    else:
        sys.stderr.write(_format_human(summary, policy, rows, exit_code, exit_reason))
        sys.stdout.write(_format_json(summary, policy, rows, exit_code, exit_reason))
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
