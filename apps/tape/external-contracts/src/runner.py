"""Entrypoint for Tape's Python external contracts.

`python src/runner.py [case-id ...]` runs the named cases, or all of them when
given no arguments. Each case is a standalone module executed by path -- the
filenames carry hyphens so that `ls src/` reads as a list of contracts rather
than a list of importable modules, and nothing here is meant to be imported by
production code.

Exit status is the number of failed cases, so a caller can gate on it directly.
Every case writes its own evidence JSON whether it passes or fails; a red
contract that leaves no evidence behind is indistinguishable from one that was
never run.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

SRC = Path(__file__).resolve().parent

CASES = {
    "ec-3052-durability": "ec-3052-durability-under-sigkill.py",
    "ec-3052-scaling": "ec-3052-durable-append-scaling.py",
}


def run_case(case_id: str) -> int:
    path = SRC / CASES[case_id]
    spec = importlib.util.spec_from_file_location(
        case_id.replace("-", "_"), path
    )
    if spec is None or spec.loader is None:
        raise SystemExit(f"cannot load contract {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return int(module.main())


def main(argv: list[str]) -> int:
    requested = argv or list(CASES)
    unknown = [case for case in requested if case not in CASES]
    if unknown:
        raise SystemExit(
            f"unknown contract(s): {', '.join(unknown)}. "
            f"known: {', '.join(CASES)}"
        )
    failed = 0
    for case_id in requested:
        print(f"== {case_id}", flush=True)
        failed += 1 if run_case(case_id) else 0
    if failed:
        print(f"{failed} of {len(requested)} contract(s) failed", file=sys.stderr)
    return failed


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
