#!/usr/bin/env python3
"""Negative control for `check_epic_order.py`.

Its two headline corpus rows are relational -- "every epic the ledger read as
graphed still is", "no epic the ledger read as readable has turned unreadable"
-- and a relational row states an absence. An instrument that read nothing at
all would satisfy both, so neither is evidence until it has been seen to move.

The mutation is one character of `_requirement_refs`: `R(\\d+)` becomes
`R([2-9]\\d*)`, which makes the reference parser blind to `R1` and to nothing
else. It is the shape a real regression here would take -- an off-by-one in the
ref pattern, not a deleted function -- and it does not touch the sections, the
table splitter, or the cycle and dangling detectors.

What must happen is a *split*: the two relational rows go red and name the
epics they lost, while "the ledger and the bodies come from one refresh" stays
green. That green is the half that matters. It shows the gate is measuring the
parser rather than the snapshot's bookkeeping -- a control where every row
reddened at once would be satisfied by deleting the snapshot.

Restoration writes the captured bytes back and verifies by sha256; a reverse
string-replace restores a file that only looks like the original.
"""
import hashlib
import pathlib
import sys
import subprocess

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import HERE, SCRIPT, SNAPSHOTS  # noqa: E402

GATE = HERE / "check_epic_order.py"
ANCHOR = r'refs.update(int(n) for n in re.findall(r"(?<![A-Za-z])R(\d+)", cell))'
MUTANT = r'refs.update(int(n) for n in re.findall(r"(?<![A-Za-z])R([2-9]\d*)", cell))'

GRAPHED_ROW = "every epic the ledger read as graphed still is"
READABLE_ROW = "no epic the ledger read as readable has turned unreadable"
PAIRING_ROW = "the ledger and the bodies come from one refresh"


def gate():
    r = subprocess.run([sys.executable, str(GATE)], capture_output=True, text=True)
    return r.returncode, r.stdout


ledger = SNAPSHOTS / "order_rows.json"
if not ledger.is_file():
    print(f"error: no order ledger at {ledger}. Both relational rows short-circuit "
          f"without it, so this control would measure nothing. Run "
          f"measure_population.py first.")
    sys.exit(1)

original = SCRIPT.read_bytes()
before = hashlib.sha256(original).hexdigest()
text = original.decode("utf-8")
assert text.count(ANCHOR) == 1, f"anchor is not unique ({text.count(ANCHOR)} hits)"

base_code, base_out = gate()
print(f"== baseline == {base_out.strip().splitlines()[-1]} (exit={base_code})")

SCRIPT.write_text(text.replace(ANCHOR, MUTANT), encoding="utf-8")
code, out = gate()
reds = [ln.split(" want ")[0] for ln in out.splitlines() if ln.startswith("**FAIL**")]
greens = [ln.split(" want ")[0] for ln in out.splitlines() if ln.startswith("PASS")]
print(f"== mutated ==  {out.strip().splitlines()[-1]} (exit={code})")
for line in reds:
    print("   RED  " + line.replace("**FAIL**", "").strip())

SCRIPT.write_bytes(original)
after = hashlib.sha256(SCRIPT.read_bytes()).hexdigest()
restored_code, restored_out = gate()
print(f"== restored == {restored_out.strip().splitlines()[-1]} (exit={restored_code})")
print(f"\nbefore = {before}\nafter  = {after}\n"
      f"restore: {'byte-identical' if before == after else 'FAILED'}")

graphed_red = any(GRAPHED_ROW in r for r in reds)
readable_red = any(READABLE_ROW in r for r in reds)
pairing_green = any(PAIRING_ROW in g for g in greens)
print(f"\ngraphed row went red             : {graphed_red}")
print(f"readable row went red            : {readable_red}")
print(f"refresh-pairing row stayed green : {pairing_green}")

ok = (before == after and base_code == 0 and restored_code == 0 and code == 1
      and graphed_red and readable_red and pairing_green)
print("=> " + ("GREEN" if ok else "RED"))
sys.exit(0 if ok else 1)
