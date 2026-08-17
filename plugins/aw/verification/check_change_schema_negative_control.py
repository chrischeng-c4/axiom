#!/usr/bin/env python3
"""Negative control for `check_change_schema.py`, one mutation per drift class.

The gate exists to catch the change schema shifting under the one script that
owns it. So each mutation here is a plausible drift, not a broken file: every
one leaves `change.py` importable, leaves the CLI working, and leaves every
other gate green. A drift that crashed would not need a gate.

The first three take one word list apart three different ways, because the gate
holds that list two different ways and each half is blind to one of them.

  narrowed-vocab   one hedge word dropped. This is what re-typing a word list
                   looks like when it goes wrong, and it is silent in ordinary
                   use. Two assertions must red, not one: the declared
                   inventory, and the liveness probe for that word. The second
                   red is the load-bearing one -- it is only there if the probe
                   loop is generated from the gate's own literal. Generate the
                   loop from `change.py` instead and deleting a word deletes
                   its own probe, so this mutation would red once and look
                   fine.
  widened-vocab    one hedge word added. The inventory reds alone; every probe
                   still passes, because every declared word still works. This
                   is the inventory half isolated, and the direction a
                   liveness-only gate cannot see at all.
  dead-vocab       the list left intact, but the loop that reads it made to
                   skip its first entry. The inventory agrees perfectly; the
                   word has simply stopped refusing anything. This is the
                   liveness half isolated, and the half a snapshot oracle
                   structurally cannot catch -- two identical lists agree
                   whether or not either one does any work.
  template-drift   one word changed in the empty body. Two surfaces handing a
                   human two different forms to fill in, with nothing else
                   observably different.
  rule-lost        the change-point/must-not-touch collision rule made
                   unreachable. `change.py` still imports, still validates,
                   still refuses everything else, and has quietly stopped
                   enforcing one rule. It reds twice -- the case, and refusal
                   coverage -- because a rule that stops firing is both.
  coverage-blind   one case deleted from the gate's own table. Refusal coverage
                   is what makes the corpus non-optional; if deleting a case is
                   free, the corpus is decoration. The case chosen is the sole
                   reader of one `errors.append` site, so its removal is
                   exactly a rule going unobserved.
  fixtures-missing the gate's own fixture directory pointed somewhere that does
                   not exist. Every mutation-based case swaps a fragment of the
                   sample body, so a fixture channel that silently stopped
                   resolving would make those cases vacuous. The positive
                   control must red first, and it must red before any case
                   runs.

Applied one at a time. The control demands isolation -- exactly the matching
assertion goes red -- and restores by writing the captured bytes back, verified
by sha256; a reverse string-replace restores a file that only looks like the
original.

One honest limit, recorded rather than papered over: there is no mutation here
for passing `## How` comment-stripped into `validate_never`. No case
discriminates that choice -- the sample body has no commented-out change point
-- so a mutation would leave the gate green, and a control that cannot go red
proves nothing. It is a real gap in the corpus, not a rule the gate measures.
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
        ["FAIL the hedge vocabulary is the declared one",
         "FAIL the hedge `supposedly` refuses a premise"],
    ),
    (
        "widened-vocab",
        CHANGE_SCRIPT,
        '    "supposedly",\n',
        '    "supposedly",\n    "perhaps",\n',
        ["FAIL the hedge vocabulary is the declared one"],
    ),
    (
        "dead-vocab",
        CHANGE_SCRIPT,
        "for hedge in HEDGE_WORDS:",
        "for hedge in HEDGE_WORDS[1:]:",
        ["FAIL the hedge `should` refuses a premise"],
    ),
    (
        "template-drift",
        CHANGE_SCRIPT,
        "<!-- List actions that must not be taken. -->",
        "<!-- List actions that must not be done. -->",
        ["FAIL the skeleton is the declared empty change body"],
    ),
    (
        "rule-lost",
        CHANGE_SCRIPT,
        "        if normalize_path(path) in change_paths:",
        "        if False and normalize_path(path) in change_paths:",
        ["FAIL a change point cannot also be must-not-touch",
         "FAIL every refusal site in the validators is reached by some case"],
    ),
    (
        "coverage-blind",
        GATE,
        '    "a gate row whose why-column is a placeholder is refused": case_gate_row_needs_a_why,\n',
        "",
        ["FAIL every refusal site in the validators is reached by some case"],
    ),
    (
        "fixtures-missing",
        GATE,
        'FIXTURES = HERE / "_fixtures"',
        'FIXTURES = HERE / "_fixtures_gone"',
        ["FAIL positive control: the fixtures are on disk"],
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
