#!/usr/bin/env python3
"""Blast radius of the coverage rule, measured before the rule was written.

Two candidate readings of the same rule, measured separately:

  strict  -- every R<n> declared in `## Requirements` appears verbatim as a
             first-column cell
  loose   -- same, but a cell may name a range (`R1-R3`) or a list (`R1, R2`),
             so the non-bare spellings the tail actually contains are honoured
             rather than refused on formatting

The gap between the two numbers is the cost of not writing the expander, and
it is the reason `_requirement_refs` exists. Keeping this runnable is what
makes that docstring's numbers re-derivable instead of remembered.

Reads the snapshot; run `measure_population.py` first.
"""
import json
import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import SNAPSHOTS  # noqa: E402

path = SNAPSHOTS / "coverage_rows.json"
if not path.is_file():
    raise SystemExit(f"error: no snapshot at {path}; run measure_population.py first")

rows = {r["number"]: r for r in json.loads(path.read_text(encoding="utf-8"))}


def refs_bare(cell: str) -> set[int]:
    return {int(n) for n in re.findall(r"^R(\d+)$", cell.strip())}


def refs_expanded(cell: str) -> set[int]:
    """Expand what the live tail actually contains: ranges, lists, suffixes."""
    out: set[int] = set()
    for low, high in re.findall(r"R(\d+)\s*[-–]\s*R?(\d+)", cell):
        low, high = int(low), int(high)
        if low <= high:
            out.update(range(low, high + 1))
    out.update(int(n) for n in re.findall(r"R(\d+)", cell))
    return out


def report(label: str, population: list[int]) -> None:
    valid = [n for n in population if rows[n]["valid"]]
    strict_red, loose_red = [], []
    for number in valid:
        row = rows[number]
        declared = {int(r[1:]) for r in row["reqs"]}
        bare = set().union(*(refs_bare(c) for c in row["cells"])) if row["cells"] else set()
        wide = set().union(*(refs_expanded(c) for c in row["cells"])) if row["cells"] else set()
        if declared - bare:
            strict_red.append(number)
        if declared - wide:
            loose_red.append(number)
    print(f"\n== {label}: {len(population)} epics, {len(valid)} of them currently validate green ==")
    print(f"  turned red by the bare-only reading : {len(strict_red)} / {len(valid)}")
    print(f"  turned red after expanding ranges   : {len(loose_red)} / {len(valid)}")
    only_spelling = sorted(set(strict_red) - set(loose_red))
    if only_spelling:
        print(f"  killed purely on spelling           : {len(only_spelling)} -> "
              + ", ".join(f"#{n}" for n in only_spelling[:12]))
    if loose_red:
        print("  genuinely missing an inventory row:")
        for number in sorted(loose_red)[:15]:
            row = rows[number]
            declared = {int(r[1:]) for r in row["reqs"]}
            wide = set().union(*(refs_expanded(c) for c in row["cells"])) if row["cells"] else set()
            missing = sorted(declared - wide)
            print(f"    #{number}: declares {len(declared)}, uncovered {len(missing)} -> "
                  + ", ".join(f"R{m}" for m in missing[:8]))


report("open epics", sorted(n for n, r in rows.items() if r.get("state") == "OPEN"))
report("all epics", sorted(rows))
