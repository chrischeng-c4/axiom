#!/usr/bin/env python3
"""The tail decides the rule.

Any first-column value that is not a bare `R<n>` is a spelling a naive
`cells == reqs` rule would refuse. Print every one of them, with the epics it
came from, before deciding what the rule may assume.

Reads the snapshot; run `measure_population.py` first.
"""
import collections
import json
import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import SNAPSHOTS  # noqa: E402

path = SNAPSHOTS / "coverage_rows.json"
if not path.is_file():
    raise SystemExit(f"error: no snapshot at {path}; run measure_population.py first")

rows = json.loads(path.read_text(encoding="utf-8"))
BARE = re.compile(r"^R\d+$")

odd = collections.defaultdict(list)
for row in rows:
    for cell in row["cells"]:
        if not BARE.match(cell):
            odd[cell].append(row["number"])

print(f"== first-column literals that are not a bare R<n>: {len(odd)} distinct ==")
for value, numbers in sorted(odd.items(), key=lambda kv: -len(kv[1])):
    where = ", ".join(f"#{n}" for n in numbers[:4])
    more = f" +{len(numbers) - 4}" if len(numbers) > 4 else ""
    print(f"  {len(numbers):3d}x  {value!r:44s} {where}{more}")

# How many epics carry at least one of these -- i.e. how many a bare-equality
# rule would refuse on spelling rather than on real coverage.
touched = {n for numbers in odd.values() for n in numbers}
print(f"\nepics carrying at least one non-bare value: {len(touched)} / {len(rows)}")

no_reqs = [r["number"] for r in rows if not r["reqs"]]
no_cells = [r["number"] for r in rows if not r["cells"]]
print(f"epics with no parseable R<n> in `## Requirements`: {len(no_reqs)}")
print(f"epics with no parseable row in `## Verification Inventory`: {len(no_cells)}")
