#!/usr/bin/env python3.12
"""Lint fixture [tool.mamba] records (PRODUCTION-GATE.md D2 / FIXTURE-LAYOUT.md).

For every fixture that carries a `[tool.mamba]` PEP 723 record this checks:
  - the record is valid TOML
  - required keys are present: bucket, lib, dimension, case, subject, kind
  - `case` equals the filename stem (the filename IS the case key)
  - no `UNFILLED` sentinel remains

Legacy fixtures without a `[tool.mamba]` record are counted but not failed
(the tree is mid-migration; FIXTURE-LAYOUT.md still reads the legacy form).
Exit non-zero iff any recorded fixture violates the schema.

    python3.12 tests/harness/cpython/tools/fixture_lint.py
    python3.12 tests/harness/cpython/tools/fixture_lint.py --lib calendar

Requires Python 3.11+ for tomllib.
"""

from __future__ import annotations

import argparse
import tomllib
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parent
FIXTURES_DIR = TOOLS_DIR.parents[2] / "cpython" / "fixtures"
REQUIRED = ("bucket", "lib", "dimension", "case", "subject", "kind")


def extract_record(text: str) -> dict:
    """Pull the PEP 723 `# /// script ... # ///` block and parse it as TOML."""
    lines: list[str] = []
    in_block = False
    for ln in text.splitlines():
        s = ln.strip()
        if "/// script" in s and s.startswith("#"):
            in_block = True
            continue
        if in_block and s.startswith("#") and s.rstrip().endswith("///") and "script" not in s:
            break
        if in_block:
            body = s[1:] if s.startswith("#") else s
            lines.append(body[1:] if body[:1] == " " else body)
    return tomllib.loads("\n".join(lines))


def lint_file(path: Path) -> tuple[bool, list[str]]:
    """Return (has_record, problems). Empty problems == clean."""
    text = path.read_text(encoding="utf-8", errors="replace")
    problems: list[str] = []
    if "UNFILLED" in text:
        problems.append("UNFILLED sentinel present")
    if "[tool.mamba]" not in text:
        return False, problems
    try:
        rec = extract_record(text)
        mamba = rec["tool"]["mamba"]
    except Exception as exc:  # noqa: BLE001
        return True, problems + [f"cannot parse [tool.mamba]: {exc}"]
    for key in REQUIRED:
        if key not in mamba:
            problems.append(f"missing required key: {key}")
    if mamba.get("case") != path.stem:
        problems.append(f"case={mamba.get('case')!r} != filename stem {path.stem!r}")
    return True, problems


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--lib", help="limit to fixtures whose path contains this segment")
    ap.add_argument("--show", type=int, default=20, help="max violations to print")
    args = ap.parse_args(argv)

    recorded = legacy = violations = 0
    shown = 0
    for path in sorted(FIXTURES_DIR.rglob("*.py")):
        if args.lib and args.lib not in path.parts:
            continue
        has_record, problems = lint_file(path)
        if has_record:
            recorded += 1
        else:
            legacy += 1
        if problems:
            violations += 1
            if shown < args.show:
                shown += 1
                print(f"VIOLATION {path.relative_to(FIXTURES_DIR)}: {problems[0]}")
    if violations > shown:
        print(f"... and {violations - shown} more violations")
    print(f"fixture_lint: recorded={recorded} legacy={legacy} violations={violations}")
    return 1 if violations else 0


if __name__ == "__main__":
    raise SystemExit(main())
