#!/usr/bin/env python3
"""MVP release-gate baseline update policy validator (closes #2823).

Parent: #2775 (MVP release blocking CI profiles).

Walks the baseline changelog (default
`projects/mamba/validation/baseline_changelog.toml`) and validates each
entry against the policy declared in
`projects/mamba/validation/baseline_update_policy.toml`.

Acceptance (issue #2823):

    1. Baseline changes without reason or issue link fail validation.
       Any entry under [direction.weaker] missing `reason` or
       `tracking_issue` (or `before`/`after`) exits nonzero.
    2. Stronger baselines are allowed with summary. Entries marked
       `stronger` need only `summary`; missing reason / tracking_issue
       is permitted.
    3. Policy is referenced by release profile manifests. Asserted
       separately by `tests/mvp_runners/mvp_release_baseline_policy_2823.rs`
       (dispatched by the `tests/mvp_runners.rs` umbrella); this
       script does not re-walk the manifests.

Exit codes
----------

    0   every changelog entry satisfies the direction's required_fields
        AND the issue-link regex.
    1   one or more entries fail validation (full list printed to
        stderr).
    100 usage / argument error.
    101 policy or changelog file missing / unreadable.

Pure stdlib (Python 3.11+ for tomllib).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


EXIT_USAGE = 100
EXIT_IO = 101
EXIT_FAIL = 1

VALID_DIRECTIONS = ("weaker", "stronger")


@dataclass
class Policy:
    weaker_required: list[str]
    weaker_action: str
    stronger_required: list[str]
    stronger_action: str
    changelog_path: Path
    missing_changelog_action: str
    duplicate_entry_action: str
    issue_link_regex: re.Pattern[str]
    baselines: dict[str, dict[str, Any]] = field(default_factory=dict)


@dataclass
class EntryError:
    entry_index: int
    family: str
    direction: str
    message: str


def _die(code: int, msg: str) -> None:
    sys.stderr.write(f"baseline_policy_check: {msg}\n")
    sys.exit(code)


def _load_policy(path: Path) -> Policy:
    if not path.is_file():
        _die(EXIT_IO, f"policy missing: {path}")
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    direction = data.get("direction") or {}
    weaker = direction.get("weaker") or {}
    stronger = direction.get("stronger") or {}
    if not weaker or not stronger:
        _die(EXIT_IO, "policy must declare [direction.weaker] AND [direction.stronger]")
    validation = data.get("validation") or {}
    return Policy(
        weaker_required=list(weaker.get("required_fields", [])),
        weaker_action=str(weaker.get("fail_action", "block")),
        stronger_required=list(stronger.get("required_fields", [])),
        stronger_action=str(stronger.get("fail_action", "allow")),
        changelog_path=Path(validation.get("changelog_path", "")),
        missing_changelog_action=str(
            validation.get("missing_changelog_action", "warn")
        ),
        duplicate_entry_action=str(
            validation.get("duplicate_entry_action", "block")
        ),
        issue_link_regex=re.compile(
            validation.get("issue_link_regex", r"^(#[0-9]+|https?://.+)$")
        ),
        baselines=data.get("baselines") or {},
    )


def _load_changelog(path: Path) -> list[dict[str, Any]]:
    if not path.is_file():
        return []
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    raw = data.get("entries") or []
    return [dict(e) for e in raw]


def _validate(policy: Policy, entries: list[dict[str, Any]]) -> list[EntryError]:
    errors: list[EntryError] = []
    seen_keys: set[tuple[str, str]] = set()
    for idx, entry in enumerate(entries):
        family = str(entry.get("family", ""))
        direction = str(entry.get("direction", ""))
        if family not in policy.baselines:
            errors.append(EntryError(idx, family, direction,
                f"unknown family {family!r} (not declared in [baselines])"))
            continue
        if direction not in VALID_DIRECTIONS:
            errors.append(EntryError(idx, family, direction,
                f"direction must be weaker|stronger, got {direction!r}"))
            continue
        required = (
            policy.weaker_required if direction == "weaker" else policy.stronger_required
        )
        for fld in required:
            value = entry.get(fld)
            if value is None or (isinstance(value, str) and not value.strip()):
                errors.append(EntryError(idx, family, direction,
                    f"missing required field {fld!r} for direction={direction}"))
        if direction == "weaker" and "tracking_issue" in required:
            link = str(entry.get("tracking_issue", ""))
            if link and not policy.issue_link_regex.match(link):
                errors.append(EntryError(idx, family, direction,
                    f"tracking_issue {link!r} does not match policy regex"))
        after_hash = entry.get("after_hash")
        if after_hash is not None:
            key = (family, str(after_hash))
            if key in seen_keys and policy.duplicate_entry_action == "block":
                errors.append(EntryError(idx, family, direction,
                    f"duplicate entry: family+after_hash already used"))
            seen_keys.add(key)
    return errors


def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="baseline_policy_check",
        description="Validate baseline_changelog entries (#2823).",
    )
    p.add_argument("--policy", type=Path, default=None,
                   help="path to baseline_update_policy.toml (default: relative to this script)")
    p.add_argument("--changelog", type=Path, default=None,
                   help="override changelog path (default: policy.validation.changelog_path)")
    p.add_argument("--format", choices=("text", "json"), default="text",
                   help="error reporting format (default: text)")
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    ns = _parse_args(list(sys.argv[1:] if argv is None else argv))
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    policy_path = ns.policy or (project_root / "validation" / "baseline_update_policy.toml")
    policy = _load_policy(policy_path.resolve())

    changelog_path = ns.changelog or (project_root / policy.changelog_path)
    changelog_path = changelog_path.resolve()
    if not changelog_path.is_file():
        if policy.missing_changelog_action == "block":
            _die(EXIT_FAIL, f"changelog missing and policy=block: {changelog_path}")
        if ns.format == "json":
            sys.stdout.write(json.dumps({"errors": [], "warnings": [
                f"changelog missing: {changelog_path}"]}, indent=2) + "\n")
        else:
            sys.stderr.write(f"baseline_policy_check: changelog missing: {changelog_path}\n")
        return 0
    entries = _load_changelog(changelog_path)
    errors = _validate(policy, entries)
    if ns.format == "json":
        sys.stdout.write(json.dumps({
            "errors": [
                {
                    "entry_index": e.entry_index,
                    "family": e.family,
                    "direction": e.direction,
                    "message": e.message,
                }
                for e in errors
            ],
            "entry_count": len(entries),
        }, indent=2) + "\n")
    else:
        for e in errors:
            sys.stderr.write(
                f"  [{e.entry_index}] {e.family} ({e.direction}): {e.message}\n"
            )
        if errors:
            sys.stderr.write(f"baseline_policy_check: {len(errors)} error(s)\n")
    return EXIT_FAIL if errors else 0


if __name__ == "__main__":
    sys.exit(main())
