#!/usr/bin/env python3
"""
Rebrand script: cclab-nucleus -> cclab
Usage: python scripts/rebrand.py [--dry-run]
"""

import os
import re
import sys
from pathlib import Path

# Mapping: old_name -> new_name (hyphenated for Cargo, underscored for Rust)
CRATE_MAPPING = {
    # Agent framework -> Nova (新星)
    "cclab-nova-core": "cclab-nova-core",
    "cclab-nova-llm": "cclab-nova-llm",
    "cclab-nova-tools": "cclab-nova-tools",
    # API -> Quasar (類星體)
    "cclab-quasar": "cclab-quasar",
    # CLI
    "cclab-cli": "cclab-cli",
    # Common -> Core
    "cclab-core": "cclab-core",
    # HTTP -> Photon (光子)
    "cclab-photon": "cclab-photon",
    # KV -> Ion (離子)
    "cclab-ion-client": "cclab-ion-client",
    "cclab-ion-server": "cclab-ion-server",
    "cclab-ion": "cclab-ion",
    # MongoDB -> Nebula (星雲)
    "cclab-nebula": "cclab-nebula",
    # PostgreSQL -> Titan (土衛六)
    "cclab-titan": "cclab-titan",
    # PyLoop -> Orbit (軌道)
    "cclab-orbit": "cclab-orbit",
    # QC -> Probe (探測器)
    "cclab-probe": "cclab-probe",
    # Sheet -> Grid (網格)
    "cclab-grid-core": "cclab-grid-core",
    "cclab-grid-db": "cclab-grid-db",
    "cclab-grid-formula": "cclab-grid-formula",
    "cclab-grid-history": "cclab-grid-history",
    "cclab-grid-server": "cclab-grid-server",
    "cclab-grid-wasm": "cclab-grid-wasm",
    # Talos -> Warp (曲速)
    "cclab-jet-asset": "cclab-jet-asset",
    "cclab-jet-bundler": "cclab-jet-bundler",
    "cclab-jet-dev-server": "cclab-jet-dev-server",
    "cclab-jet-pkg-manager": "cclab-jet-pkg-manager",
    "cclab-jet-resolver": "cclab-jet-resolver",
    "cclab-jet-transform": "cclab-jet-transform",
    "cclab-jet": "cclab-jet",
    # Tasks -> Swarm (蜂群)
    "cclab-swarm": "cclab-swarm",
    # Validation -> Shield (護盾)
    "cclab-shield": "cclab-shield",
    # Core -> Nucleus (原子核)
    "cclab-nucleus": "cclab-nucleus",
    # Argus -> Lens (透鏡)
    "cclab-lens": "cclab-lens",
}

# Python module mapping
PYTHON_MODULE_MAPPING = {
    "cclab.nova": "cclab.nova",
    "cclab.quasar": "cclab.quasar",
    "cclab.core": "cclab.core",
    "cclab.photon": "cclab.photon",
    "cclab.ion": "cclab.ion",
    "cclab.nebula": "cclab.nebula",
    "cclab.titan": "cclab.titan",
    "cclab.orbit": "cclab.orbit",
    "cclab.probe": "cclab.probe",
    "cclab.swarm": "cclab.swarm",
    "cclab.shield": "cclab.shield",
    "cclab-nucleus": "cclab",
}

# File extensions to process
RUST_EXTENSIONS = {".rs", ".toml"}
PYTHON_EXTENSIONS = {".py", ".pyi"}
DOC_EXTENSIONS = {".md", ".txt", ".yml", ".yaml", ".json"}
ALL_EXTENSIONS = RUST_EXTENSIONS | PYTHON_EXTENSIONS | DOC_EXTENSIONS

# Directories to skip
SKIP_DIRS = {".git", "target", ".venv", "node_modules", "__pycache__", ".mypy_cache"}


def to_underscore(name: str) -> str:
    """Convert hyphenated name to underscore (Rust module style)."""
    return name.replace("-", "_")


def build_replacements() -> list[tuple[str, str]]:
    """Build sorted list of replacements (longest first to avoid partial matches)."""
    replacements = []

    # Crate names (hyphenated) - for Cargo.toml
    for old, new in CRATE_MAPPING.items():
        replacements.append((old, new))

    # Rust module names (underscored) - for .rs files
    for old, new in CRATE_MAPPING.items():
        old_underscore = to_underscore(old)
        new_underscore = to_underscore(new)
        if old_underscore != old:  # Only add if different
            replacements.append((old_underscore, new_underscore))

    # Python module names
    for old, new in PYTHON_MODULE_MAPPING.items():
        replacements.append((old, new))

    # Sort by length (longest first) to avoid partial replacements
    replacements.sort(key=lambda x: len(x[0]), reverse=True)

    # Remove duplicates while preserving order
    seen = set()
    unique = []
    for old, new in replacements:
        if old not in seen:
            seen.add(old)
            unique.append((old, new))

    return unique


def process_file(filepath: Path, replacements: list[tuple[str, str]], dry_run: bool) -> bool:
    """Process a single file, return True if modified."""
    try:
        content = filepath.read_text(encoding="utf-8")
    except (UnicodeDecodeError, PermissionError):
        return False

    original = content
    for old, new in replacements:
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
    print(f"Building replacements...")

    replacements = build_replacements()
    print(f"Total replacement patterns: {len(replacements)}")

    # Show first few replacements
    print("\nSample replacements:")
    for old, new in replacements[:10]:
        print(f"  {old} -> {new}")
    print("  ...")

    modified_count = 0
    scanned_count = 0

    print("\nProcessing files...")

    for dirpath, dirnames, filenames in os.walk(root):
        # Skip unwanted directories
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]

        for filename in filenames:
            filepath = Path(dirpath) / filename
            suffix = filepath.suffix.lower()

            if suffix in ALL_EXTENSIONS:
                scanned_count += 1
                if process_file(filepath, replacements, dry_run):
                    modified_count += 1

    print(f"\nSummary:")
    print(f"  Scanned: {scanned_count} files")
    print(f"  Modified: {modified_count} files")

    if dry_run:
        print("\nRun without --dry-run to apply changes.")


if __name__ == "__main__":
    main()
