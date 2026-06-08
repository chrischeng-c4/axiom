#!/usr/bin/env python3
"""MVP performance bench tier-header checker (closes #2571).

Parent: #2530 (performance gate suite).

Reads `validation/perf_benchmark_manifest.toml` (#2567) and parses
each listed fixture for the three tier-header directives:

    # tier: <required|exploratory>
    # category: <numeric|recursion|workload|...>
    # inclusion_reason: <free-form one-line reason>

The header format is deliberately comment-based so existing fixtures
can be migrated by adding three `#` lines anywhere in the file —
benchmark code itself never changes (acceptance #3). The harness can
parse them with a one-line regex and the checker emits parsed values
in JSON output (acceptance #2: harness output includes tier per
benchmark — the checker IS the validation half of the harness).

Acceptance (issue #2571):

    1. A fixture without required tier metadata fails validation.
       A `tier = "required"` manifest entry whose fixture is missing
       any of `tier:` / `category:` / `inclusion_reason:` headers
       exits 1.
    2. Harness output includes tier per benchmark. JSON `entries`
       array carries parsed `tier_header`, `category_header`,
       `inclusion_reason_header` per benchmark.
    3. Existing comments can be migrated without changing benchmark
       code. Headers are `#`-prefixed lines anywhere in the file; no
       executable code mutation needed.

Tier values in the fixture header must match the manifest's
`[[benchmarks]] tier` value — drift would mean the manifest and the
fixture disagree about gating status, which is exactly the kind of
silent rot this gate exists to prevent.

Operating modes
---------------

`--format text` (default)
    One line per validated entry to stderr; exit reflects pass/fail.

`--format json`
    {
      "schema_version": 1,
      "manifest_path": "...",
      "entries": [
        {"id": "...", "fixture": "...", "tier": "required",
         "tier_header": "required", "category_header": "numeric",
         "inclusion_reason_header": "..."},
        ...
      ],
      "violations": [
        {"id": "...", "tier": "required",
         "reason": "fixture missing header 'tier:'"},
        ...
      ],
      "exit_code": 0|1
    }

`--manifest PATH`
    Override the default
    `projects/mamba/validation/perf_benchmark_manifest.toml`.

Exit codes
----------

    0   every required fixture declares all three headers and the
        header tier matches the manifest tier.
    1   one or more required fixtures missing headers or mismatched.
    100 usage / argument error.
    101 manifest file missing or unparseable.

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


SCHEMA_VERSION = 1
EXIT_USAGE = 100
EXIT_IO = 101
EXIT_FAIL = 1

REQUIRED_TIER = "required"
HEADER_KEYS = ("tier", "category", "inclusion_reason")

# A header line looks like `# tier: required` (any amount of leading
# whitespace, exactly one `#`, optional whitespace, the directive name,
# `:`, value to end-of-line). Permissive on whitespace so a fixture
# author cannot accidentally invalidate the header by adding a space.
_HEADER_RE = re.compile(
    r"^\s*#\s*(?P<key>[a-z_][a-z0-9_]*)\s*:\s*(?P<value>.+?)\s*$"
)


@dataclass
class ParsedHeaders:
    tier: str | None
    category: str | None
    inclusion_reason: str | None


@dataclass
class Entry:
    id: str
    fixture: str
    manifest_tier: str
    fixture_path: Path
    headers: ParsedHeaders


@dataclass
class Violation:
    id: str
    tier: str
    reason: str


def _die(code: int, msg: str) -> None:
    sys.stderr.write(f"perf_bench_header_check: {msg}\n")
    sys.exit(code)


def _load(path: Path) -> dict[str, Any]:
    if not path.is_file():
        _die(EXIT_IO, f"manifest missing: {path}")
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as exc:
        _die(EXIT_IO, f"manifest invalid TOML ({exc}): {path}")
        return {}


def parse_headers(text: str) -> ParsedHeaders:
    """Parse `# tier:` / `# category:` / `# inclusion_reason:` from text.

    Public so the Rust harness can call into us via subprocess (or
    reimplement against this contract). On duplicate header keys the
    LAST occurrence wins — fixture authors who copy-paste a header
    block then edit it should not silently keep the stale value.
    """
    out: dict[str, str] = {}
    for line in text.splitlines():
        m = _HEADER_RE.match(line)
        if not m:
            continue
        key = m.group("key")
        if key in HEADER_KEYS:
            out[key] = m.group("value").strip()
    return ParsedHeaders(
        tier=out.get("tier"),
        category=out.get("category"),
        inclusion_reason=out.get("inclusion_reason"),
    )


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
        manifest_tier = str(e.get("tier", ""))
        resolved = (fixture_root / fixture).resolve()
        try:
            text = resolved.read_text(encoding="utf-8")
            headers = parse_headers(text)
        except (OSError, UnicodeDecodeError):
            headers = ParsedHeaders(None, None, None)
        out.append(Entry(id_, fixture, manifest_tier, resolved, headers))
    return out


def _validate(entries: list[Entry]) -> list[Violation]:
    violations: list[Violation] = []
    for e in entries:
        if e.manifest_tier != REQUIRED_TIER:
            continue
        if e.headers.tier is None:
            violations.append(Violation(e.id, e.manifest_tier, "fixture missing header 'tier:'"))
        elif e.headers.tier != e.manifest_tier:
            violations.append(Violation(
                e.id, e.manifest_tier,
                f"header tier {e.headers.tier!r} does not match manifest tier {e.manifest_tier!r}",
            ))
        if e.headers.category is None:
            violations.append(Violation(e.id, e.manifest_tier, "fixture missing header 'category:'"))
        if e.headers.inclusion_reason is None:
            violations.append(
                Violation(e.id, e.manifest_tier, "fixture missing header 'inclusion_reason:'")
            )
    return violations


def _format_text(
    manifest_path: Path,
    entries: list[Entry],
    violations: list[Violation],
) -> str:
    lines = [
        f"perf_bench_header_check: manifest={manifest_path}",
        f"  checked={len(entries)} violations={len(violations)}",
    ]
    for e in entries:
        lines.append(
            f"  id={e.id:<24} fixture={e.fixture:<24} "
            f"manifest_tier={e.manifest_tier:<11} "
            f"header_tier={e.headers.tier or '<missing>'} "
            f"category={e.headers.category or '<missing>'}"
        )
    if violations:
        lines.append("violations:")
        for v in violations:
            lines.append(f"  id={v.id:<24} tier={v.tier:<11} reason={v.reason}")
        lines.append(
            "rule: required fixtures must declare `# tier:`, "
            "`# category:`, `# inclusion_reason:` headers (#2571)"
        )
    else:
        lines.append("perf_bench_header_check: clean")
    return "\n".join(lines) + "\n"


def _format_json(
    manifest_path: Path,
    entries: list[Entry],
    violations: list[Violation],
    exit_code: int,
) -> str:
    payload = {
        "schema_version": SCHEMA_VERSION,
        "manifest_path": str(manifest_path),
        "checked_count": len(entries),
        "entries": [
            {
                "id": e.id,
                "fixture": e.fixture,
                "tier": e.manifest_tier,
                "tier_header": e.headers.tier,
                "category_header": e.headers.category,
                "inclusion_reason_header": e.headers.inclusion_reason,
                "resolved_path": str(e.fixture_path),
            }
            for e in entries
        ],
        "violations": [
            {"id": v.id, "tier": v.tier, "reason": v.reason} for v in violations
        ],
        "exit_code": exit_code,
    }
    return json.dumps(payload, indent=2, sort_keys=True) + "\n"


def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="perf_bench_header_check",
        description="Parse tier headers in cross-runtime bench fixtures (#2571).",
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
    violations = _validate(entries)
    exit_code = EXIT_FAIL if violations else 0

    if ns.format == "json":
        sys.stdout.write(_format_json(manifest_path, entries, violations, exit_code))
    else:
        sys.stderr.write(_format_text(manifest_path, entries, violations))
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
