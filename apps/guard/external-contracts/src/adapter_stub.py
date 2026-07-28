#!/usr/bin/python3
"""Independent executable boundary for Guard dynamic-adapter EC fixtures."""

from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> int:
    invoked_path = Path(sys.argv[0]).absolute()
    tool = invoked_path.name
    fixture_dir = invoked_path.parent
    outcome_path = fixture_dir / f"{tool}-outcome.json"
    trace_path = fixture_dir / f"{tool}-argv.txt"
    outcome = json.loads(outcome_path.read_text(encoding="utf-8"))
    with trace_path.open("a", encoding="utf-8") as trace_file:
        trace_file.write(json.dumps(sys.argv[1:], separators=(",", ":")) + "\n")
    print(json.dumps(outcome["payload"], separators=(",", ":")))
    return int(outcome["exit_code"])


if __name__ == "__main__":
    raise SystemExit(main())
