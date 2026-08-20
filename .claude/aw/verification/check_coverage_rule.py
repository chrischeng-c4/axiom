#!/usr/bin/env python3
"""Verify the cross-section coverage rule through the real `validate_body`.

Two things have to hold at once, and either alone is worthless:

  regression -- no epic that validated green before turns red now, measured by
                importing the shipped module rather than re-implementing it
  bite       -- a body whose only defect is an uncovered requirement goes red,
                proving the rule is not a decoration that can never fire

A rule that reds 0 live epics is only good news if the second half holds.

The regression half needs the tracker snapshot `measure_population.py` writes.
When it is absent this gate fails and says so, rather than skipping quietly:
a silent skip turns the strongest assertion here into a no-op that still
prints green.
"""
import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import SNAPSHOTS, hard_errors, load_epic_module  # noqa: E402

mod = load_epic_module()
fails = []


def check(label: str, ok: bool, detail: str = "") -> None:
    print(f"{'PASS' if ok else 'FAIL'} {label}{(' -- ' + detail) if detail else ''}")
    if not ok:
        fails.append(label)


def errors(body: str) -> list[str]:
    return hard_errors(mod, body)


# -- regression: the live population, through the shipped validator ---------
epics_path = SNAPSHOTS / "epics.json"
rows_path = SNAPSHOTS / "coverage_rows.json"
have_snapshot = epics_path.is_file() and rows_path.is_file()
check("the tracker snapshot is present (run measure_population.py to refresh)",
      have_snapshot, f"expected {epics_path} and {rows_path}")

if have_snapshot:
    epics = json.loads(epics_path.read_text(encoding="utf-8"))
    before = json.loads(rows_path.read_text(encoding="utf-8"))
    was_valid = {r["number"] for r in before if r["valid"]}

    now_red = []
    for epic in epics:
        if epic["number"] not in was_valid:
            continue
        errs = errors(epic.get("body") or "")
        if errs:
            now_red.append((epic["number"], errs))

    check(
        f"no previously-valid epic turns red ({len(was_valid)} measured)",
        not now_red,
        "" if not now_red else "; ".join(f"#{n}: {e[0]}" for n, e in now_red[:5]),
    )

# -- bite: the exact false green this rule exists to kill -------------------
SKELETON = mod.skeleton()


def body_with(requirements: str, rows: str) -> str:
    """The skeleton, filled just enough to pass every per-section rule, so the
    only thing left that can go red is the cross-section check."""
    return f"""## Problem

The epic schema accepted a body whose inventory covered one requirement out of
six, because each section was individually well formed and nothing in the
validator ever compared two sections against each other.

## Capability Alignment

Capability: epic authoring
Capability Gap: coverage is unchecked
Progress Evidence: measured across 255 live epics

## Requirements

{requirements}

## Scope

### In Scope

- the validator

### Out of Scope

- the tracker

## Acceptance Criteria

- validate refuses an uncovered requirement

## Verification Inventory

| Requirement | Gate | Oracle | Depends On |
|---|---|---|---|
{rows}

## Reference Context

### Related Specs

- none

### Spec Plan

- none
"""


SIX = "\n".join(f"- R{n}: an observable requirement" for n in range(1, 7))
COVERAGE_ERROR = "has no row for"

one_row = errors(body_with(SIX, "| R1 | cargo test | passes | |"))
check(
    "R1..R6 with a single inventory row is refused",
    any(COVERAGE_ERROR in e for e in one_row),
    f"errors={one_row}",
)
check(
    "and it is refused for exactly the missing five",
    any("R2, R3, R4, R5, R6" in e for e in one_row),
    f"errors={one_row}",
)
check(
    "with nothing else red (the rule is isolated)",
    len(one_row) == 1,
    f"errors={one_row}",
)

all_rows = "\n".join(f"| R{n} | cargo test | passes | |" for n in range(1, 7))
check("R1..R6 fully inventoried is accepted", not errors(body_with(SIX, all_rows)))

# -- the spellings the live tail actually uses ------------------------------
for label, rows in [
    ("range `R1-R6`", "| R1-R6 | cargo test | passes | |"),
    ("list `R1, R2, R3` + range `R4-R6`",
     "| R1, R2, R3 | a | b | |\n| R4-R6 | c | d | |"),
    ("suffixed `R1 (Lumen)` + rest", "| R1 (Lumen) | a | b | |\n| R2-R6 | c | d | |"),
]:
    check(f"honours {label}", not errors(body_with(SIX, rows)))

check(
    "does NOT read `PR1`/`AC1` as a requirement ref",
    any(COVERAGE_ERROR in e for e in errors(body_with(SIX, "| PR1, AC2-AC6 | a | b | |"))),
)

# -- silence where another rule already speaks ------------------------------
no_inventory = body_with(SIX, "| R1 | a | b | |").replace("## Verification Inventory", "## Elsewhere")
errs = errors(no_inventory)
check(
    "stays silent when `## Verification Inventory` is missing",
    not any(COVERAGE_ERROR in e for e in errs),
    f"errors={errs}",
)

# -- the skeleton must still validate on its own terms ----------------------
skeleton_errs = [e for e in errors(SKELETON) if COVERAGE_ERROR in e]
check("`skeleton` output is not self-contradictory", not skeleton_errs, f"errors={skeleton_errs}")

print("\n=> " + ("GREEN" if not fails else "RED: " + ", ".join(fails)))
sys.exit(1 if fails else 0)
