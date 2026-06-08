#!/usr/bin/env python3
"""
Rebrand Python imports: cclab.* -> cclab.*
Usage: python scripts/rebrand_python.py [--dry-run]
"""

import os
import sys
from pathlib import Path

# Python import/module replacements (order matters - longest first)
REPLACEMENTS = [
    # Full module paths (longest first)
    ("cclab.shield", "cclab.shield"),
    ("cclab.nebula", "cclab.nebula"),
    ("cclab.titan", "cclab.titan"),
    ("cclab.orbit", "cclab.orbit"),
    ("cclab.core", "cclab.core"),
    ("cclab.nova", "cclab.nova"),
    ("cclab.swarm", "cclab.swarm"),
    ("cclab.photon", "cclab.photon"),
    ("cclab.quasar", "cclab.quasar"),
    ("cclab.ion", "cclab.ion"),
    ("cclab.probe", "cclab.probe"),
    # Base module
    ("cclab", "cclab"),
    # Also handle string literals and comments
    ("'cclab'", "'cclab'"),
    ('"cclab"', '"cclab"'),
]

SKIP_DIRS = {".git", "target", ".venv", "node_modules", "__pycache__", ".mypy_cache"}


def process_file(filepath: Path, dry_run: bool) -> bool:
    """Process a single file, return True if modified."""
    try:
        content = filepath.read_text(encoding="utf-8")
    except (UnicodeDecodeError, PermissionError):
        return False

    original = content
    for old, new in REPLACEMENTS:
        content = content.replace(old, new)

    if content != original:
        if dry_run:
            print(f"[DRY-RUN] Would modify: {filepath}")
        else:
            filepath.write_text(content, encoding="utf-8")
            print(f"Modified: {filepath}")
        return True
    return False


def main():
    dry_run = "--dry-run" in sys.argv
    root = Path(__file__).parent.parent

    if dry_run:
        print("=== DRY RUN MODE ===\n")

    print(f"Root directory: {root}")

    modified_count = 0
    scanned_count = 0

    # Process Python files
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]

        for filename in filenames:
            if filename.endswith((".py", ".pyi", ".md", ".toml", ".yml", ".yaml")):
                filepath = Path(dirpath) / filename
                scanned_count += 1
                if process_file(filepath, dry_run):
                    modified_count += 1

    print(f"\nSummary:")
    print(f"  Scanned: {scanned_count} files")
    print(f"  Modified: {modified_count} files")


if __name__ == "__main__":
    main()
