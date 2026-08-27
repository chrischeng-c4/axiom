#!/usr/bin/env python3
"""Snapshot every live epic and record what its two coupled sections contain.

Writes `_snapshots/epics.json` (the raw tracker bodies) and
`_snapshots/coverage_rows.json` (per epic: current validity, the `R<n>` set
declared in `## Requirements`, and the raw first-column cells of the
`## Verification Inventory` table).

Also writes `_snapshots/order_rows.json` (per epic: whether `order_children`
found a requirement graph, an unreadable `Depends On` cell, a cycle, or a
dangling reference), which is what makes `check_epic_order.py`'s corpus rows
relational instead of a declared count. That file used to declare `32 graphed,
10 unreadable`, measured over a 255-epic snapshot, and went red on a growing
tracker without `epic.py` changing.

`check_coverage_rule.py` and `check_epic_order.py` read the snapshot to prove
no epic that was valid, graphed, or readable has stopped being so. The
snapshot is deliberately untracked: it is tracker state, it changes without
anyone editing this repository, and a stale copy committed alongside the rules
would let their regression assertions pass against a population that no longer
exists. For the same reason both files assert that the ledger they read names
the same epics as the bodies beside it -- one refresh writes all three, so a
mismatch means one of them is from another run.

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

# `--limit` is a cap, not a page size: `gh` returns newest-first and drops the
# remainder without saying so. That is how this snapshot quietly became a
# sliding *window* over the tracker rather than a *population*. At `--limit
# 300` it covered everything while the tracker held 255 epics and stopped
# covering the 13 oldest -- #4 through #30, opened 2026-06-10 and 06-11 -- the
# moment the tracker crossed 300, so every count declared against it drifted
# with nobody editing a file. Ask for far more than the population, and refuse
# the run when the answer comes back at the cap, which is the one reading that
# cannot tell "that is all of them" from "there were more".
LIMIT = 2000

raw = subprocess.run(
    ["gh", "issue", "list", "--repo", TRACKER_REPO, "--label", "type:epic",
     "--state", "all", "--limit", str(LIMIT),
     "--json", "number,title,body,state"],
    capture_output=True, text=True, check=True,
).stdout
epics = json.loads(raw)
if len(epics) >= LIMIT:
    sys.exit(f"error: gh returned {len(epics)} epics at --limit {LIMIT}. The cap "
             f"is binding, so this snapshot is a window over the newest epics "
             f"and not the population. Raise LIMIT and re-run.")
print(f"population: {len(epics)} epics carrying type:epic "
      f"({sum(1 for e in epics if e['state'] == 'OPEN')} open, "
      f"cap {LIMIT} not binding)\n")

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

# The order ledger. `order_children` is called with no children on purpose:
# the four flags recorded here are properties of the epic body alone, and
# passing a child list would mix in `undeclared-order`, which is a property of
# what the tracker currently labels as owned and moves for reasons that have
# nothing to do with the parser.
order_rows = []
for epic in epics:
    out = mod.order_children(
        {"number": epic["number"], "body": epic.get("body") or ""}, [])
    order_rows.append({
        "number": epic["number"],
        "graphed": bool(out["graph"]),
        "unreadable": bool(out["unreadable"]),
        "cycle": bool(out["cycle"]),
        "dangling": bool(out["dangling"]),
    })

(SNAPSHOTS / "order_rows.json").write_text(json.dumps(order_rows, indent=1), encoding="utf-8")
print(f"order ledger written to _snapshots/order_rows.json "
      f"({sum(1 for x in order_rows if x['graphed'])} graphed, "
      f"{sum(1 for x in order_rows if x['unreadable'])} with an unreadable cell, "
      f"{sum(1 for x in order_rows if x['cycle'])} cyclic, "
      f"{sum(1 for x in order_rows if x['dangling'])} dangling)")
