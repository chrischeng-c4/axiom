#!/usr/bin/env python3
"""Negative control for `check_coverage_rule.py`.

Unbind the cross-section rule from `EPIC` and confirm the gate goes red on the
bite assertions *while the regression assertion stays green*. That split is the
whole proof: it shows the gate is measuring the rule and not the population. A
control where everything reds at once would be satisfied by deleting the file.

Restoration writes the captured bytes back and verifies by sha256; a reverse
string-replace restores a file that only looks like the original.
"""
import hashlib
import pathlib
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import HERE, SCRIPT  # noqa: E402

GATE = HERE / "check_coverage_rule.py"
ANCHOR = "    cross_rules=(_requirements_are_inventoried,),"
MUTANT = "    cross_rules=(),"


def gate():
    r = subprocess.run([sys.executable, str(GATE)], capture_output=True, text=True)
    return r.returncode, r.stdout


original = SCRIPT.read_bytes()
before = hashlib.sha256(original).hexdigest()
text = original.decode("utf-8")
assert text.count(ANCHOR) == 1, f"anchor is not unique ({text.count(ANCHOR)} hits)"

code, out = gate()
print(f"== baseline == {out.strip().splitlines()[-1]} (exit={code})")

SCRIPT.write_text(text.replace(ANCHOR, MUTANT), encoding="utf-8")
code, out = gate()
reds = [ln.split(" -- ")[0] for ln in out.splitlines() if ln.startswith("FAIL")]
greens = [ln.split(" -- ")[0] for ln in out.splitlines() if ln.startswith("PASS")]
print(f"== mutated ==  {out.strip().splitlines()[-1]} (exit={code})")
for line in reds:
    print("   RED  " + line)

SCRIPT.write_bytes(original)
after = hashlib.sha256(SCRIPT.read_bytes()).hexdigest()
restored_code, restored_out = gate()
print(f"== restored == {restored_out.strip().splitlines()[-1]} (exit={restored_code})")
print(f"\nbefore = {before}\nafter  = {after}\nrestore: {'byte-identical' if before == after else 'FAILED'}")

regression_stayed_green = any("no previously-valid epic turns red" in g for g in greens)
bite_went_red = any("single inventory row is refused" in r for r in reds)
print(f"\nbite assertion went red          : {bite_went_red}")
print(f"regression assertion stayed green: {regression_stayed_green}")

ok = before == after and restored_code == 0 and code == 1 and bite_went_red and regression_stayed_green
print("=> " + ("GREEN" if ok else "RED"))
sys.exit(0 if ok else 1)
