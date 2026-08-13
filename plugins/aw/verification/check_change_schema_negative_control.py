#!/usr/bin/env python3
"""Negative control for `check_change_schema.py`, one mutation per drift class.

The gate exists to catch a port drifting from the crate it ports. So each
mutation here is a plausible drift, not a broken file: every one leaves
`change.py` importable, leaves the CLI working, and leaves every other gate
green. A drift that crashed would not need a gate.

  narrowed-vocab   one hedge word dropped from the port's list. This is what
                   re-typing a word list actually looks like when it goes
                   wrong, and it is silent -- the dropped word is one the
                   crate's own corpus never exercises, so only the constant
                   comparison can see it.
  template-drift   one word changed in the empty body. Two surfaces handing a
                   human two different forms to fill in, with nothing else
                   observably different.
  rule-lost        the change-point/must-not-touch collision rule made
                   unreachable. The port still validates, still refuses
                   everything else, and has quietly stopped enforcing one rule.
  coverage-blind   one replay deleted from the gate's own case table. The
                   count assertion is what makes the corpus non-optional; if
                   deleting a case is free, the corpus is decoration.
  extractor-blind  the crate-constant extractor pointed at a name that does
                   not exist. Every comparison downstream would then be a
                   tautology, so the positive control must red first.

Applied one at a time. The control demands isolation -- exactly the matching
assertion goes red -- and restores by writing the captured bytes back, verified
by sha256; a reverse string-replace restores a file that only looks like the
original.

One honest limit, recorded rather than papered over: there is no mutation here
for passing `## How` comment-stripped into `validate_never`. The crate's corpus
does not discriminate that choice -- its sample body has no commented-out
change point -- so a mutation would leave the gate green, and a control that
cannot go red proves nothing. The port follows the crate because the crate is
the authority, not because this gate measures it.
"""
import hashlib
import pathlib
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import CHANGE_SCRIPT, HERE  # noqa: E402

GATE = HERE / "check_change_schema.py"

MUTATIONS = [
    (
        "narrowed-vocab",
        CHANGE_SCRIPT,
        '    "supposedly",\n',
        "",
        ["FAIL the hedge vocabulary agrees with the crate"],
    ),
    (
        "template-drift",
        CHANGE_SCRIPT,
        "<!-- List actions that must not be taken. -->",
        "<!-- List actions that must not be done. -->",
        ["FAIL the skeleton is the crate's own empty change body"],
    ),
    (
        "rule-lost",
        CHANGE_SCRIPT,
        "        if normalize_path(path) in change_paths:",
        "        if False and normalize_path(path) in change_paths:",
        ["FAIL replay: a_change_point_cannot_also_be_must_not_touch"],
    ),
    (
        "coverage-blind",
        GATE,
        '    "table_rows_skips_header_and_separator": case_table_rows,\n',
        "",
        ["FAIL every crate test has a replay here"],
    ),
    (
        "extractor-blind",
        GATE,
        'CRATE_HEDGES = rust_str_list(GHAN_SRC, "HEDGE_WORDS")',
        'CRATE_HEDGES = rust_str_list(GHAN_SRC, "HEDGE_WORDS_RENAMED")',
        ["FAIL positive control: the crate constants extract"],
    ),
]


def gate():
    r = subprocess.run([sys.executable, str(GATE)], capture_output=True, text=True)
    return r.returncode, r.stdout


baseline_code, baseline_out = gate()
print(f"== baseline == {baseline_out.strip().splitlines()[-1]} (exit={baseline_code})")

failures = []
for label, target, anchor, mutant, expected in MUTATIONS:
    original = target.read_bytes()
    before = hashlib.sha256(original).hexdigest()
    text = original.decode("utf-8")
    if text.count(anchor) != 1:
        failures.append(f"{label}: anchor is not unique ({text.count(anchor)} hits)")
        print(f"\n== {label} == ANCHOR NOT UNIQUE ({text.count(anchor)} hits)")
        continue

    target.write_text(text.replace(anchor, mutant), encoding="utf-8")
    code, out = gate()
    reds = [ln.split(" -- ")[0] for ln in out.splitlines() if ln.startswith("FAIL")]

    target.write_bytes(original)
    after = hashlib.sha256(target.read_bytes()).hexdigest()

    isolated = reds == expected
    print(f"\n== {label} == exit={code}")
    for line in reds:
        print(f"   RED  {line}")
    print(f"   isolation: {'exactly the expected assertion(s)' if isolated else f'UNEXPECTED: {reds}'}")
    print(f"   restore:   {'byte-identical' if before == after else 'FAILED'} ({before[:16]}...)")

    if not isolated or before != after or code == 0:
        failures.append(label)

restored_code, restored_out = gate()
print(f"\n== restored == {restored_out.strip().splitlines()[-1]} (exit={restored_code})")

ok = not failures and restored_code == 0 and baseline_code == 0
print("=> " + ("GREEN" if ok else f"RED ({failures or 'gate not green after restore'})"))
sys.exit(0 if ok else 1)
