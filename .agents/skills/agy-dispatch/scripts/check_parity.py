#!/usr/bin/env python3
"""Compare AGY support files and dispatcher target bytes with another skill root."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


SHARED_SUPPORT_FILES = (
    "references/inventory-verification.md",
    "references/lifecycle.md",
    "references/one-shot-profile-template.json",
    "references/permissions.md",
    "references/profile-template.json",
    "references/report-verification.md",
    "scripts/run_isolated.py",
    "scripts/teamwork_terminal.py",
    "scripts/test_agy_dispatch.py",
    "scripts/test_run_isolated.py",
)
ENGINE_FILE = "scripts/agy_dispatch.py"
DIGEST_FILES = (*SHARED_SUPPORT_FILES, ENGINE_FILE)


def sha256(path: Path) -> str | None:
    if not path.is_file():
        return None
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("other_skill_root", type=Path)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    canonical_root = Path(__file__).resolve().parents[1]
    rows = []
    other_root = args.other_skill_root.resolve()
    for relative in DIGEST_FILES:
        canonical_path = canonical_root / relative
        other_path = other_root / relative
        canonical = sha256(canonical_path)
        other = sha256(other_path)
        rows.append(
            {
                "path": relative,
                "canonical_sha256": canonical,
                "other_sha256": other,
                "canonical_is_symlink": canonical_path.is_symlink(),
                "other_is_symlink": other_path.is_symlink(),
                "match": canonical == other and canonical is not None,
            }
        )

    if args.json:
        print(json.dumps({"match": all(row["match"] for row in rows), "files": rows}, indent=2))
    else:
        for row in rows:
            state = "OK" if row["match"] else "DIFF"
            print(f"{state} {row['path']}")

    return 0 if all(row["match"] for row in rows) else 1


if __name__ == "__main__":
    raise SystemExit(main())
