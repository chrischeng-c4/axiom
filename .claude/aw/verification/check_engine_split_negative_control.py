#!/usr/bin/env python3
"""Negative control for `check_engine_split.py`, one mutation per leak class.

Each mutation is the real regression, not a caricature of it -- the way an
engine actually re-learns its type is one branch added while fixing something
else, and every mutation below leaves the module importable and every other
gate green. That is the point: if the leak broke the script, the split would
not need a gate.

  literal-leak   a `== "epic"` branch inside `staging_dir`. The engine keeps
                 working; it has simply stopped being the engine.
  identifier     a helper named `epic_dir`. Nothing about it is wrong except
                 that a second type now inherits a name it has no use for.
  enum-widened   a fifth member spliced into the closed enum. This one exists
                 because the enum is the gate's own exemption: widening it is
                 how a leak walks in through the exit.
  control-blind  the extractor pointed at a file with no code-level type name
                 at all. The positive control must red, or the gate reports a
                 clean engine whenever the parse quietly returns nothing.

Applied one at a time. The control demands isolation -- exactly the matching
assertion goes red -- and restores by writing the captured bytes back, verified
by sha256; a reverse string-replace restores a file that only looks like the
original.
"""
import hashlib
import pathlib
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import ENGINE, HERE  # noqa: E402

GATE = HERE / "check_engine_split.py"

STAGING_ANCHOR = '    directory = REPO_ROOT / WORKITEMS_DIR_REL / f"{wi_type}s"'

MUTATIONS = [
    (
        "literal-leak",
        ENGINE,
        STAGING_ANCHOR,
        '    if wi_type == "epic":\n        pass\n' + STAGING_ANCHOR,
        ["FAIL the engine's code carries no work-item type in a string literal"],
    ),
    (
        "identifier",
        ENGINE,
        STAGING_ANCHOR,
        "    epic_dir = None\n" + STAGING_ANCHOR,
        ["FAIL the engine's code carries no work-item type in an identifier"],
    ),
    (
        "enum-widened",
        ENGINE,
        'WORK_ITEM_TYPES = ("epic", "change", "spike", "report")',
        'WORK_ITEM_TYPES = ("epic", "change", "spike", "report", "epic-draft")',
        ["FAIL the enum exemption covers exactly the closed enum"],
    ),
    (
        "control-blind",
        GATE,
        "facade_literals, facade_names = leaks(SCRIPT)",
        "facade_literals, facade_names = leaks(ENGINE)",
        ["FAIL positive control: the extractor finds type names in the epic facade",
         "FAIL positive control: the facade's leaks include a real label literal"],
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
