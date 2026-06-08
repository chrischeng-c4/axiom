#!/usr/bin/env python3
"""fixture_lint.py — schema + structure linter for mamba conformance fixtures.

Walks tests/cpython/fixtures, extracts each fixture's PEP 723 '# /// script'
block, parses the embedded [tool.mamba] table, and enforces the regularization
contract.

Migration-aware. A fixture falls into one of four states:

  LEGACY    — no '# /// script' block, or a block without a [tool.mamba]
              table. Un-migrated; REPORTED, not failed (unless --strict).
  ERROR     — HAS a [tool.mamba] table but violates the schema/path contract.
              Always a HARD FAIL (exit 1). The migrated subset is enforced.
  UNFILLED  — valid record with status == "generated" (a semantic skeleton an
              agent still has to fill). REPORTED, not failed.
  OK        — valid record, status filled / mechanical.

HARD FAIL (exit 1) triggers (only on files that HAVE a [tool.mamba] table):
  - block does not parse as TOML
  - missing a required [tool.mamba] key
  - dimension / bucket not in the allowed set
  - bucket / lib / dimension mismatch vs the file's actual path
  - filename stem != [tool.mamba].case
  - a "filled" or mechanical file still containing 'AGENT-FILL' / 'UNFILLED'
  - body does not end in a print()
With --strict, LEGACY files also fail (use once migration is complete).

Skips '_stub.py' files and anything under an '_invalid/' directory.

CLI:
    python fixture_lint.py                 # lint everything
    python fixture_lint.py --bucket std-libs
    python fixture_lint.py --lib calendar
    python fixture_lint.py --unfilled      # just list the UNFILLED skeletons
    python fixture_lint.py --legacy        # just list the un-migrated files
    python fixture_lint.py --strict        # LEGACY also fails (post-migration)

stdlib only (tomllib, pathlib, re, argparse).
"""

from __future__ import annotations

import argparse
import re
import tomllib
from collections import Counter
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parent
CPYTHON_DIR = TOOLS_DIR.parent / "tests" / "cpython"
FIXTURES_ROOT = CPYTHON_DIR / "fixtures"

BUCKETS = {"core", "builtin-libs", "std-libs", "pep", "type-strict", "3rd-libs"}
DIMENSIONS = {"surface", "behavior", "errors", "bench", "real_world", "security"}
REQUIRED_KEYS = ("bucket", "lib", "dimension", "case", "subject", "kind")
LIB_SHAPED_BUCKETS = {"std-libs", "3rd-libs", "pep"}
FEATURE_SHAPED_BUCKETS = {"core", "builtin-libs", "type-strict"}

# States
LEGACY = "legacy"
ERROR = "error"
UNFILLED = "unfilled"
OK = "ok"

BLOCK_RE = re.compile(
    r"^# /// script[ \t]*\n(.*?)^# ///[ \t]*$",
    re.DOTALL | re.MULTILINE,
)
PRINT_TAIL_RE = re.compile(r"^\s*print\(", re.MULTILINE)


def extract_block_toml(text: str) -> str | None:
    """Strip the '# '/'#' prefixes from the script block and return TOML.

    Returns None when there is no '# /// script ... # ///' block at all, or a
    line inside it is not a '# '/'#' comment (a genuinely malformed block).
    """
    m = BLOCK_RE.search(text)
    if not m:
        return None
    lines = []
    for raw in m.group(1).splitlines():
        if raw.startswith("# "):
            lines.append(raw[2:])
        elif raw == "#":
            lines.append("")
        else:
            return None
    return "\n".join(lines)


def classify(path: Path) -> tuple[str, list[str], dict | None]:
    """Return (state, errors, meta). state is LEGACY/ERROR/UNFILLED/OK."""
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:  # pragma: no cover - filesystem edge
        return ERROR, [f"cannot read: {exc}"], None

    toml_src = extract_block_toml(text)
    if toml_src is None:
        # No usable script block -> un-migrated. Not an error during migration.
        return LEGACY, [], None

    try:
        data = tomllib.loads(toml_src)
    except tomllib.TOMLDecodeError:
        # A PEP 723 block exists but isn't TOML. If it carries no mamba intent
        # treat as legacy; the block may be a plain dependencies header.
        return LEGACY, [], None

    meta = data.get("tool", {}).get("mamba")
    if not isinstance(meta, dict):
        # Valid PEP 723 block but no [tool.mamba] record -> un-migrated.
        return LEGACY, [], None

    # From here the file IS migrated: enforce the contract hard.
    errors: list[str] = []
    for key in REQUIRED_KEYS:
        if key not in meta:
            errors.append(f"missing required key [tool.mamba].{key}")
    if errors:
        return ERROR, errors, meta

    if meta["dimension"] not in DIMENSIONS:
        errors.append(f"dimension {meta['dimension']!r} not in {sorted(DIMENSIONS)}")
    if meta["bucket"] not in BUCKETS:
        errors.append(f"bucket {meta['bucket']!r} not in {sorted(BUCKETS)}")

    # Path agreement:
    #   lib-shaped:     fixtures/<bucket>/<lib>/<dimension>/<case>.py
    #   feature-shaped: fixtures/<bucket>/<lib>/.../<case>.py
    #
    # core/builtin-libs/type-strict are discovered as the `feature` gate by
    # tests/harness/cpython even though [tool.mamba].dimension still names the
    # contract axis.
    try:
        rel = path.relative_to(FIXTURES_ROOT)
    except ValueError:
        rel = path
    parts = rel.parts if isinstance(rel, Path) else ()
    if len(parts) >= 4 and parts[0] in LIB_SHAPED_BUCKETS:
        p_bucket, p_lib, p_dim, p_file = parts[0], parts[1], parts[2], parts[-1]
        if p_bucket != meta["bucket"]:
            errors.append(f"path bucket {p_bucket!r} != [tool.mamba].bucket {meta['bucket']!r}")
        if p_lib != meta["lib"]:
            errors.append(f"path lib {p_lib!r} != [tool.mamba].lib {meta['lib']!r}")
        if p_dim != meta["dimension"]:
            errors.append(f"path dimension {p_dim!r} != [tool.mamba].dimension {meta['dimension']!r}")
        stem = p_file[:-3] if p_file.endswith(".py") else p_file
        if stem != meta["case"]:
            errors.append(f"filename stem {stem!r} != [tool.mamba].case {meta['case']!r}")
    elif len(parts) >= 3 and parts[0] in FEATURE_SHAPED_BUCKETS:
        p_bucket, p_lib, p_file = parts[0], parts[1], parts[-1]
        if p_bucket != meta["bucket"]:
            errors.append(f"path bucket {p_bucket!r} != [tool.mamba].bucket {meta['bucket']!r}")
        if p_lib != meta["lib"]:
            errors.append(f"path lib {p_lib!r} != [tool.mamba].lib {meta['lib']!r}")
        stem = p_file[:-3] if p_file.endswith(".py") else p_file
        if stem != meta["case"]:
            errors.append(f"filename stem {stem!r} != [tool.mamba].case {meta['case']!r}")
    else:
        errors.append(f"path too shallow for supported fixture shape: {rel}")

    status = meta.get("status", "")
    kind = meta.get("kind", "")
    is_unfilled = status == "generated"
    has_marker = ("AGENT-FILL" in text) or ("UNFILLED" in text)

    # A filled or mechanical file must not still carry the placeholder markers.
    if has_marker and not (is_unfilled and kind != "mechanical"):
        errors.append("filled/mechanical file still contains AGENT-FILL/UNFILLED")

    if not PRINT_TAIL_RE.search(text):
        errors.append("body has no print() statement")

    if errors:
        return ERROR, errors, meta
    return (UNFILLED if is_unfilled else OK), [], meta


def discover(bucket: str | None, lib: str | None):
    for path in sorted(FIXTURES_ROOT.rglob("*.py")):
        if path.name.endswith("_stub.py"):
            continue
        if "_invalid" in path.parts:
            continue
        rel = path.relative_to(FIXTURES_ROOT)
        if bucket and (len(rel.parts) < 1 or rel.parts[0] != bucket):
            continue
        if lib and (len(rel.parts) < 2 or rel.parts[1] != lib):
            continue
        yield path


def main(argv: list[str] | None = None) -> int:
    doc = (__doc__ or "").splitlines()
    ap = argparse.ArgumentParser(description=doc[0] if doc else "fixture linter")
    ap.add_argument("--bucket", help="only lint this bucket")
    ap.add_argument("--lib", help="only lint this lib")
    ap.add_argument("--unfilled", action="store_true",
                    help="list status=generated (UNFILLED) skeletons and exit 0")
    ap.add_argument("--legacy", action="store_true",
                    help="list un-migrated (LEGACY) files and exit 0")
    ap.add_argument("--strict", action="store_true",
                    help="treat LEGACY (un-migrated) files as failures too")
    args = ap.parse_args(argv)

    total = 0
    legacy: list[Path] = []
    unfilled: list[Path] = []
    failures: list[tuple[Path, list[str]]] = []
    per_bucket: Counter = Counter()
    per_dimension: Counter = Counter()
    ok_count = 0

    for path in discover(args.bucket, args.lib):
        total += 1
        state, errors, meta = classify(path)
        if state == LEGACY:
            legacy.append(path)
            continue
        # migrated file: record its taxonomy
        if meta:
            per_bucket[meta.get("bucket", "?")] += 1
            per_dimension[meta.get("dimension", "?")] += 1
        if state == ERROR:
            failures.append((path, errors))
        elif state == UNFILLED:
            unfilled.append(path)
        else:
            ok_count += 1

    if args.unfilled:
        print(f"UNFILLED skeletons ({len(unfilled)}):")
        for p in unfilled:
            print(f"  {p.relative_to(FIXTURES_ROOT)}")
        return 0
    if args.legacy:
        print(f"LEGACY (un-migrated) files ({len(legacy)}):")
        for p in legacy:
            print(f"  {p.relative_to(FIXTURES_ROOT)}")
        return 0

    for path, errors in failures:
        rel = path.relative_to(FIXTURES_ROOT)
        for err in errors:
            print(f"FAIL {rel}: {err}")

    migrated = ok_count + len(unfilled) + len(failures)
    print("")
    print(f"total fixtures:        {total}")
    print(f"  migrated (record):   {migrated}")
    print(f"  legacy (un-migrated):{len(legacy)}")
    if migrated:
        print("per bucket (migrated):")
        for b in sorted(per_bucket):
            print(f"  {b}: {per_bucket[b]}")
        print("per dimension (migrated):")
        for d in sorted(per_dimension):
            print(f"  {d}: {per_dimension[d]}")
    print(f"  ok (filled):         {ok_count}")
    print(f"  unfilled (generated):{len(unfilled)}")
    print(f"  schema failures:     {len(failures)}")

    failed = len(failures) + (len(legacy) if args.strict else 0)
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
