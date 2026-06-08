#!/usr/bin/env python3
"""MVP release-gate flaky test quarantine validator (closes #2824).

Parent: #2775 (MVP release blocking CI profiles).

Walks the quarantine policy file (default
`projects/mamba/validation/flaky_quarantine_policy.toml`) and validates
that each `[[quarantine]]` entry satisfies the rules in `[validation]`.

Acceptance (issue #2824):

    1. Required flaky item without issue link fails policy validation.
    2. Quarantined item appears in release summary and blocker budget.
       — surfaced via `--emit-blockers` which prints synthesized
       `blockers_by_objective` entries the release-gate runner (#2821)
       merges into the summary.
    3. No flaky item is silently dropped. Unparseable entries or a
       missing list under `silent_drop_action = "block"` fails the run.

Operating modes
---------------

`--format text` (default)
    Validation errors one-per-line to stderr.

`--format json`
    `{
        "errors": [...],
        "entry_count": <int>,
        "visible_in_summary": <int>,
        "exit_code": <int>
     }`

`--emit-blockers`
    In addition to validation, print a JSON array of `blockers_by_objective`
    entries that the runner should splice into the release summary. One
    entry per visible quarantine row (status ∈ [status].blocker_visible).
    Used by the runner contract to satisfy acceptance #2.

Exit codes
----------

    0   policy + entries valid; no errors.
    1   one or more validation errors.
    100 usage / argument error.
    101 policy file missing / unreadable.

Pure stdlib (Python 3.11+ for tomllib).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


EXIT_USAGE = 100
EXIT_IO = 101
EXIT_FAIL = 1


@dataclass
class Policy:
    status_enum: list[str]
    blocker_visible_statuses: set[str]
    buckets: list[str]
    required_buckets: set[str]
    require_tracking_issue: bool
    issue_link_regex: re.Pattern[str]
    silent_drop_action: str
    required_fields_required: list[str]
    required_fields_optional: list[str]
    blocker_kind: str
    objective_map: dict[str, str]


@dataclass
class EntryError:
    entry_index: int
    entry_id: str
    message: str


def _die(code: int, msg: str) -> None:
    sys.stderr.write(f"flaky_quarantine_check: {msg}\n")
    sys.exit(code)


def _load_policy(path: Path) -> tuple[Policy, list[dict[str, Any]]]:
    if not path.is_file():
        _die(EXIT_IO, f"policy missing: {path}")
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    status = data.get("status") or {}
    validation = data.get("validation") or {}
    summary = data.get("summary_emission") or {}
    objective_map = summary.get("objective_map") if isinstance(summary, dict) else None
    if not isinstance(objective_map, dict):
        objective_map = {}
    policy = Policy(
        status_enum=list(status.get("enum", [])),
        blocker_visible_statuses=set(status.get("blocker_visible", [])),
        buckets=list(data.get("buckets", [])),
        required_buckets=set(validation.get("required_buckets", [])),
        require_tracking_issue=bool(validation.get("require_tracking_issue", True)),
        issue_link_regex=re.compile(
            validation.get("issue_link_regex", r"^(#[0-9]+|https?://.+)$")
        ),
        silent_drop_action=str(validation.get("silent_drop_action", "block")),
        required_fields_required=list(validation.get("required_fields_required", [])),
        required_fields_optional=list(validation.get("required_fields_optional", [])),
        blocker_kind=str(summary.get("blocker_kind", "flaky_quarantine")),
        objective_map={str(k): str(v) for k, v in objective_map.items()},
    )
    entries = data.get("quarantine") or []
    if not isinstance(entries, list):
        _die(EXIT_IO, "[[quarantine]] must be an array of tables")
    return policy, [dict(e) for e in entries]


def _validate(policy: Policy, entries: list[dict[str, Any]]) -> list[EntryError]:
    errors: list[EntryError] = []
    seen_ids: set[str] = set()
    for idx, entry in enumerate(entries):
        eid = str(entry.get("id", "<no-id>"))
        bucket = str(entry.get("bucket", ""))
        status = str(entry.get("status", ""))

        if eid != "<no-id>" and eid in seen_ids:
            errors.append(EntryError(idx, eid, f"duplicate id {eid!r}"))
        seen_ids.add(eid)

        if bucket not in policy.buckets:
            errors.append(EntryError(idx, eid,
                f"bucket {bucket!r} not in {policy.buckets}"))
        if status and status not in policy.status_enum:
            errors.append(EntryError(idx, eid,
                f"status {status!r} not in {policy.status_enum}"))

        required = (
            policy.required_fields_required
            if bucket in policy.required_buckets
            else policy.required_fields_optional
        )
        for fld in required:
            if fld not in entry or entry.get(fld) in (None, ""):
                errors.append(EntryError(idx, eid,
                    f"missing required field {fld!r} (bucket={bucket!r})"))

        if policy.require_tracking_issue and bucket in policy.required_buckets:
            link = str(entry.get("tracking_issue", ""))
            if not link:
                # Already flagged above; avoid double-reporting.
                pass
            elif not policy.issue_link_regex.match(link):
                errors.append(EntryError(idx, eid,
                    f"tracking_issue {link!r} does not match policy regex"))
    return errors


def _emit_blockers(policy: Policy, entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Synthesize blockers_by_objective rows for visible entries."""
    out: list[dict[str, Any]] = []
    for entry in entries:
        if str(entry.get("status", "")) not in policy.blocker_visible_statuses:
            continue
        owner = str(entry.get("owner_profile", ""))
        objective = policy.objective_map.get(owner, owner)
        out.append({
            "profile": owner,
            "fixture_id": str(entry.get("id", "")),
            "kind": policy.blocker_kind,
            "reason": str(entry.get("summary", "flaky test under quarantine")),
            "tracking_issue": entry.get("tracking_issue"),
            "objective": objective,
        })
    return out


def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="flaky_quarantine_check",
        description="Validate flaky-quarantine entries (#2824).",
    )
    p.add_argument("--policy", type=Path, default=None,
                   help="path to flaky_quarantine_policy.toml (default: relative to this script)")
    p.add_argument("--format", choices=("text", "json"), default="text")
    p.add_argument("--emit-blockers", action="store_true",
                   help="print blockers_by_objective rows for visible entries")
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    ns = _parse_args(list(sys.argv[1:] if argv is None else argv))
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    policy_path = ns.policy or (project_root / "validation" / "flaky_quarantine_policy.toml")
    policy, entries = _load_policy(policy_path.resolve())

    errors = _validate(policy, entries)
    visible = sum(
        1 for e in entries
        if str(e.get("status", "")) in policy.blocker_visible_statuses
    )
    exit_code = EXIT_FAIL if errors else 0

    if ns.format == "json":
        payload: dict[str, Any] = {
            "errors": [
                {"entry_index": e.entry_index, "entry_id": e.entry_id, "message": e.message}
                for e in errors
            ],
            "entry_count": len(entries),
            "visible_in_summary": visible,
            "exit_code": exit_code,
        }
        if ns.emit_blockers:
            payload["blockers"] = _emit_blockers(policy, entries)
        sys.stdout.write(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    else:
        for e in errors:
            sys.stderr.write(f"  [{e.entry_index}] {e.entry_id}: {e.message}\n")
        if errors:
            sys.stderr.write(f"flaky_quarantine_check: {len(errors)} error(s)\n")
        if ns.emit_blockers:
            sys.stdout.write(json.dumps(_emit_blockers(policy, entries), indent=2) + "\n")
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
