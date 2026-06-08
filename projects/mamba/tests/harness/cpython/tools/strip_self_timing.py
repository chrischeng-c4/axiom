#!/usr/bin/env python3
"""Strip self-timing boilerplate from bench fixtures (PRODUCTION-GATE.md D5.1).

The harness now measures CPU + peak-RSS externally (D5.2: getrusage /
`/usr/bin/time`), so a fixture must not time itself — that was the circular
"mamba times mamba" dependency. This sweep removes every line mentioning
`perf_counter` or the `INTERNAL_TIME_NS` marker (code lines and docstring
mentions alike), then drops `import time` / `import sys` if they are left
unused. What remains is the workload + the byte-equal stdout sink, so a bench
fixture becomes as pure as a behavior fixture.

    python3 tests/harness/cpython/tools/strip_self_timing.py            # dry-run (default)
    python3 tests/harness/cpython/tools/strip_self_timing.py --apply

Read-only unless --apply. Never touches a line that is part of the workload.
"""

from __future__ import annotations

import argparse
import re
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parent
CPYTHON_DIR = TOOLS_DIR.parents[2] / "cpython"  # tests/cpython (fixtures + .cache)
MAMBA_DIR = CPYTHON_DIR.parent.parent
# Scope is tests/** only. projects/mamba/benches/ is a separate perf subsystem
# outside tests/ and is intentionally NOT swept here.
ROOTS = [CPYTHON_DIR / "fixtures"]

SIGNALS = ("perf_counter", "INTERNAL_TIME_NS")


def strip_text(text: str) -> str:
    kept = [ln for ln in text.splitlines(keepends=True)
            if not any(sig in ln for sig in SIGNALS)]
    result = "".join(kept)
    # Drop imports left unused after the timing lines are gone.
    for mod in ("time", "sys"):
        if not re.search(rf"\b{mod}\.", result):
            result = re.sub(rf"(?m)^import {mod}[ \t]*\n", "", result)
    return result


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--apply", action="store_true", help="write changes (default: dry-run)")
    args = ap.parse_args(argv)

    changed = 0
    residual = 0
    for root in ROOTS:
        if not root.exists():
            continue
        for py in sorted(root.rglob("*.py")):
            # Only BENCH fixtures self-time. behavior/surface fixtures testing
            # time.perf_counter legitimately contain it — never touch them.
            if "bench" not in py.parts:
                continue
            text = py.read_text(encoding="utf-8", errors="replace")
            if not any(sig in text for sig in SIGNALS):
                continue
            new = strip_text(text)
            if new == text:
                continue
            changed += 1
            if any(sig in new for sig in SIGNALS):
                residual += 1
                print(f"  RESIDUAL {py.relative_to(MAMBA_DIR)}")
            if args.apply:
                py.write_text(new, encoding="utf-8")

    verb = "applied" if args.apply else "would change"
    print(f"{verb}: {changed} fixtures; residual-signal files: {residual}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
