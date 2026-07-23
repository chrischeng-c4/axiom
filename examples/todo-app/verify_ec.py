"""Run one FocusFlow external contract against the Rust codebase boundary."""

from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path

APP_ROOT = Path(__file__).resolve().parent
REPO_ROOT = APP_ROOT.parents[1]
MANIFEST = APP_ROOT / "backend-rust/Cargo.toml"


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: verify_ec.py <case-id>")
    case = sys.argv[1]
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
    if case == "todo-native-test-budget" and elapsed > 30.0:
        raise SystemExit(f"native test budget exceeded: {elapsed:.2f}s > 30.00s")
    evidence = APP_ROOT / "external-contracts/evidence" / f"{case}.json"
    evidence.write_text(json.dumps({"case": case, "elapsed_seconds": round(elapsed, 3), "status": "passed"}) + "\n")


if __name__ == "__main__":
    main()
