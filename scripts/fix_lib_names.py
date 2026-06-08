#!/usr/bin/env python3
"""Fix lib names in Cargo.toml files - replace hyphens with underscores."""

import re
from pathlib import Path


def fix_cargo_toml(filepath: Path) -> bool:
    """Fix lib name in a Cargo.toml file. Return True if modified."""
    content = filepath.read_text()
    original = content

    # Pattern to match [lib] section and its name field
    # We need to be careful to only modify the name in [lib] section, not [package]

    lines = content.split('\n')
    in_lib_section = False
    modified_lines = []

    for line in lines:
        # Detect section headers
        if line.strip().startswith('['):
            in_lib_section = line.strip() == '[lib]'

        # Only modify name field in [lib] section
        if in_lib_section and line.strip().startswith('name = "cclab-'):
            # Replace hyphen with underscore in the lib name
            line = re.sub(r'name = "cclab-([^"]*)"', r'name = "cclab_\1"', line)

        modified_lines.append(line)

    new_content = '\n'.join(modified_lines)

    if new_content != original:
        filepath.write_text(new_content)
        print(f"Fixed: {filepath}")
        return True
    return False


def main():
    root = Path(__file__).parent.parent / "crates"
    modified = 0

    for cargo_toml in root.glob("*/Cargo.toml"):
        if fix_cargo_toml(cargo_toml):
            modified += 1

    print(f"\nFixed {modified} Cargo.toml files")


if __name__ == "__main__":
    main()
