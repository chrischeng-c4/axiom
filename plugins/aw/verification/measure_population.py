#!/usr/bin/env python3
"""Snapshot every live epic and record what its two coupled sections contain.

Writes `_snapshots/epics.json` (the raw tracker bodies) and
`_snapshots/coverage_rows.json` (per epic: current validity, the `R<n>` set
declared in `## Requirements`, and the raw first-column cells of the
`## Verification Inventory` table).

`check_coverage_rule.py` reads the snapshot to prove no previously-valid epic
turned red. The snapshot is deliberately untracked: it is tracker state, it
changes without anyone editing this repository, and a stale copy committed
alongside the rule would let the regression assertion pass against a
population that no longer exists.

Designing a cross-section rule before looking at the real first-column
spellings is how a validator ends up refusing bodies the tracker already
accepts, so this runs before the rule, not after.
"""
import collections
import json
import pathlib
import re
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import SNAPSHOTS, TRACKER_REPO, hard_errors, load_epic_module  # noqa: E402

mod = load_epic_module()
SNAPSHOTS.mkdir(exist_ok=True)

raw = subprocess.run(
    ["gh", "issue", "list", "--repo", TRACKER_REPO, "--label", "type:epic",
     "--state", "all", "--limit", "300", "--json", "number,title,body,state"],
    capture_output=True, text=True, check=True,
).stdout
epics = json.loads(raw)
print(f"population: {len(epics)} epics carrying type:epic "
      f"({sum(1 for e in epics if e['state'] == 'OPEN')} open)\n")

(SNAPSHOTS / "epics.json").write_text(raw, encoding="utf-8")

first_cols = collections.Counter()
rows = []

for epic in epics:
    body = epic.get("body") or ""
    sections = mod.split_sections(body)
    reqs = re.findall(r"^\s*-\s*(R\d+):", sections.get("Requirements", ""), re.M)
    inventory = sections.get("Verification Inventory", "")

    cells = []
    for line in inventory.splitlines():
        line = line.strip()
        if not line.startswith("|"):
            continue
        first = line.strip("|").split("|")[0].strip()
        if not first or set(first) <= set("-: ") or first.lower() == "requirement":
            continue
        cells.append(first)
        first_cols[first] += 1

    rows.append({
        "number": epic["number"],
        "state": epic["state"],
        "valid": not hard_errors(mod, body),
        "reqs": reqs,
        "cells": cells,
    })

print("== first-column literals seen in `## Verification Inventory` (top 30) ==")
for value, count in first_cols.most_common(30):
    print(f"  {count:3d}x  {value!r}")
print(f"  ... {len(first_cols)} distinct literals\n")

(SNAPSHOTS / "coverage_rows.json").write_text(json.dumps(rows, indent=1), encoding="utf-8")
print(f"per-epic data written to _snapshots/coverage_rows.json ({len(rows)} rows)")
