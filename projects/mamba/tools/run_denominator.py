#!/usr/bin/env python3
import argparse
import hashlib
import os
import re
import sys
import subprocess
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib


def main():
    parser = argparse.ArgumentParser(description="Run mamba test denominator gate")
    parser.add_argument("slug", help="Root slug (e.g. t2_peephole) or path to gate directory")
    parser.add_argument("--runner", choices=["cargo", "binary"], default="cargo", help="Execution backend (cargo or binary)")
    parser.add_argument("--bin", help="Path to prebuilt test binary (required when --runner is binary)")
    parser.add_argument("--dry-run", action="store_true", help="Print command that would be executed without running it")

    args = parser.parse_args()

    # Determine gate directory
    slug_path = Path(args.slug)
    if slug_path.is_dir():
        gates_dir = slug_path.resolve()
    else:
        slug = args.slug
        if not slug.endswith("_denominator"):
            dir_name = f"{slug}_denominator"
        else:
            dir_name = slug
        
        script_dir = Path(__file__).resolve().parent
        project_root = script_dir.parent  # projects/mamba
        gates_dir = project_root / "tests" / "governance" / "gates" / dir_name
        if not gates_dir.is_dir():
            cwd_gates_dir = Path.cwd() / "tests" / "governance" / "gates" / dir_name
            if cwd_gates_dir.is_dir():
                gates_dir = cwd_gates_dir

    denominator_file = gates_dir / "denominator.txt"
    manifest_file = gates_dir / "manifest.toml"

    if not denominator_file.is_file():
        sys.stderr.write(f"Error: denominator file not found at {denominator_file}\n")
        sys.exit(1)

    if not manifest_file.is_file():
        sys.stderr.write(f"Error: manifest file not found at {manifest_file}\n")
        sys.exit(1)

    # 1. Read manifest and verify row_count and sha256
    with open(manifest_file, "rb") as f:
        manifest_data = tomllib.load(f)

    pinned_sha256 = manifest_data.get("denominator_sha256")
    manifest_row_count = manifest_data.get("row_count")

    with open(denominator_file, "rb") as f:
        denom_bytes = f.read()
    actual_sha256 = hashlib.sha256(denom_bytes).hexdigest()

    if not pinned_sha256 or actual_sha256 != pinned_sha256:
        sys.stderr.write(f"Error: sha256 mismatch for {denominator_file}: expected {pinned_sha256}, got {actual_sha256}\n")
        sys.exit(1)

    # 2. Parse rows from denominator.txt and verify count against manifest_row_count
    lines = denom_bytes.decode("utf-8").splitlines()
    rows = [line.strip() for line in lines if line.strip() and not line.strip().startswith("#")]
    if not rows:
        sys.stderr.write(f"Error: denominator list in {denominator_file} is empty\n")
        sys.exit(1)

    if manifest_row_count is not None and len(rows) != manifest_row_count:
        sys.stderr.write(f"Error: row count mismatch in {denominator_file}: manifest specified {manifest_row_count}, but parsed {len(rows)} rows\n")
        sys.exit(1)

    # 3. Construct command
    if args.runner == "binary":
        if not args.bin:
            sys.stderr.write("Error: --bin is required when --runner is binary\n")
            sys.exit(1)
        cmd = [args.bin, "--exact"] + rows
    else:  # cargo
        cmd = ["cargo", "test", "-p", "mamba", "--test", "cpython_ported_integration", "--", "--exact"] + rows

    if args.dry_run:
        print(" ".join(cmd))
        sys.exit(0)

    # Run command and capture output to enforce N == manifest_row_count && M == 0
    res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    sys.stdout.write(res.stdout)

    if res.returncode != 0:
        sys.stderr.write(f"Error: runner exited with code {res.returncode}\n")
        sys.exit(res.returncode)

    # Parse libtest summary
    match = re.search(r"(\d+)\s+passed;\s+(\d+)\s+failed", res.stdout)
    if not match:
        sys.stderr.write("Error: could not parse libtest summary line from output\n")
        sys.exit(1)

    passed_count = int(match.group(1))
    failed_count = int(match.group(2))

    expected_count = manifest_row_count if manifest_row_count is not None else len(rows)
    if passed_count != expected_count or failed_count != 0:
        sys.stderr.write(f"Error: test count shortfall: expected {expected_count} passed and 0 failed, got {passed_count} passed and {failed_count} failed\n")
        sys.exit(1)

    sys.exit(0)


if __name__ == "__main__":
    main()
