#!/usr/bin/env python3
# HANDWRITE-BEGIN gap="missing-generator:multi-target-python-source-unit-ownership" tracker="#1634" reason="AW cannot yet partition this semantic Python module inventory across exact script targets; preserve the verified script until #1634 lands canonical ownership."
"""Check and synchronize Lumen source mirrors with their authoritative Rust files.

Lumen source mirrors live under `apps/lumen/tech-design/semantic/source/*.md`.
Each mirror declares its authoritative target `.rs` file in its `# Standardized <path>`
or `# Fillback <path>` heading.

This script provides two subcommands:
  - `check`: decodes all 42 mirrors and asserts byte-for-byte equality against
    their target `.rs` source files.
  - `sync`: rewrites mirrors that differ from their authoritative source as
    single plain fenced blocks.
"""

from __future__ import annotations

import argparse
import base64
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[3]
MIRRORS_DIR = ROOT / "apps/lumen/tech-design/semantic/source"

HEADING = re.compile(r"^#\s+(?:Standardized|Fillback)\s+(\S+)\s*$", re.M)
MANIFEST = re.compile(r"<!--\s*aw-source-partitions:")
PART = re.compile(
    r"<!--\s*aw-source-partition:\s*index=(\d+)[^>]*-->\s*\n+```text[^\n]*\n(.*?)^```\s*$",
    re.S | re.M,
)
FENCE = re.compile(r"^(`{3,})[^\n]*\n(.*?)^\1\s*$", re.S | re.M)
SOURCE_SECTION = re.compile(
    r"(## Source\s*\n<!--\s*type:\s*rust-source-unit\s+lang:\s*rust\s*-->\s*\n)(.*?)(\n## Changes)",
    re.S,
)


def decode(body: str) -> bytes | None:
    """Return the mirror's recorded source bytes, or None if undecodable."""
    if MANIFEST.search(body):
        parts = sorted(((int(i), blob) for i, blob in PART.findall(body)))
        if not parts:
            return None
        try:
            return b"".join(
                base64.b64decode("".join(blob.split())) for _, blob in parts
            )
        except Exception:
            return None
    m = FENCE.search(body)
    return m.group(2).encode("utf-8") if m else None


def discover_pairs() -> list[tuple[pathlib.Path, pathlib.Path, bytes | None]]:
    """Discover all mirror files and their target .rs files with decoded bytes."""
    out = []
    for p in sorted(MIRRORS_DIR.glob("*.md")):
        body = p.read_text(encoding="utf-8")
        h = HEADING.search(body)
        if not h:
            continue
        rs = ROOT / h.group(1)
        if not rs.is_file():
            continue
        out.append((p, rs, decode(body)))
    return out


def make_fence(source: str) -> str:
    """Compute the backtick fence needed to safely enclose source without closing early."""
    runs = [len(m.group(0)) for m in re.finditer(r"`+", source)]
    max_run = max(runs) if runs else 0
    fence_len = max(3, max_run + 1)
    return "`" * fence_len


def cmd_check(args: argparse.Namespace) -> int:
    """Check all mirrors against source files."""
    pairs = discover_pairs()
    total = len(pairs)
    exact = 0
    drifted = []
    undecodable = []

    for mirror_path, rs_path, dec in pairs:
        if dec is None:
            undecodable.append((mirror_path, rs_path))
        elif dec == rs_path.read_bytes():
            exact += 1
        else:
            drifted.append((mirror_path, rs_path))

    if undecodable:
        for m, r in undecodable:
            print(f"UNDECODABLE: {m.relative_to(ROOT)} -> {r.relative_to(ROOT)}", file=sys.stderr)
    if drifted:
        for m, r in drifted:
            print(f"DRIFT: {m.relative_to(ROOT)} -> {r.relative_to(ROOT)}", file=sys.stderr)

    print(f"{exact}/{total} exact")
    if exact == total and total == 42:
        return 0
    return 1


def cmd_sync(args: argparse.Namespace) -> int:
    """Synchronize drifted mirrors from source files."""
    pairs = discover_pairs()
    updated_count = 0

    for mirror_path, rs_path, dec in pairs:
        rs_bytes = rs_path.read_bytes()
        if dec is not None and dec == rs_bytes:
            continue

        body = mirror_path.read_text(encoding="utf-8")
        rs_text = rs_bytes.decode("utf-8")
        fence = make_fence(rs_text)

        source_payload = f"\n{fence}rust\n{rs_text}{fence}\n"

        m = SOURCE_SECTION.search(body)
        if not m:
            print(f"ERROR: could not locate ## Source section in {mirror_path.relative_to(ROOT)}", file=sys.stderr)
            return 1

        new_body = body[:m.start(2)] + source_payload + body[m.end(2):]
        mirror_path.write_text(new_body, encoding="utf-8")
        updated_count += 1
        print(f"synced {mirror_path.relative_to(ROOT)} <- {rs_path.relative_to(ROOT)}")

    print(f"synced {updated_count} mirror(s)")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    sub = ap.add_subparsers(dest="command", required=True)

    sp_check = sub.add_parser("check", help="Check mirror equality against .rs sources")
    sp_check.set_defaults(func=cmd_check)

    sp_sync = sub.add_parser("sync", help="Synchronize drifted mirrors from .rs sources")
    sp_sync.set_defaults(func=cmd_sync)

    args = ap.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
# HANDWRITE-END
