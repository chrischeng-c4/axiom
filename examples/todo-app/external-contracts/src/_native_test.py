"""Shared, source-independent EC runner for the FocusFlow Rust target."""

from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path

EC_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = EC_ROOT.parents[2]
MANIFEST = REPO_ROOT / "examples/todo-app/backend-rust/Cargo.toml"


def run(case: str, threshold_seconds: float = 30.0) -> None:
    started = time.monotonic()
    result = subprocess.run(
        ["cargo", "test", "--offline", "--manifest-path", str(MANIFEST)],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    elapsed = time.monotonic() - started
    if result.returncode != 0:
        sys.stderr.write(result.stdout + result.stderr)
        raise SystemExit(result.returncode)
    if elapsed > threshold_seconds:
        raise SystemExit(f"native test budget exceeded: {elapsed:.2f}s > {threshold_seconds:.2f}s")
    evidence = EC_ROOT / "evidence" / f"{case}.json"
    evidence.parent.mkdir(parents=True, exist_ok=True)
    evidence.write_text(json.dumps({"case": case, "elapsed_seconds": round(elapsed, 3), "status": "passed"}) + "\n")
