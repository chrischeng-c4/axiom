#!/usr/bin/env python3
"""Prove `check_meta_clean.py` can be seen to fail, once per rule.

The gate it controls asserts a *zero*. Every other checker here asserts that
something is present and shaped a particular way, and those fail loudly when
the thing under test moves. A zero fails silently: a rule that stopped being
evaluated, a population that collapsed, a report whose findings never reached
the exit code -- each of those reads exactly like a clean tree. So the claim
that needs a control is not "the rules work", which `check_meta_flow.py`
measures against its own fixture, but "this gate, run against *this* checkout,
would have refused it".

Four mutations against two real tracked documents, one per rule, each one the
smallest edit that rots a document rather than breaks it. `apps/cube` is the
target because it is the shortest pair in the repository and holds no source,
so a mutation left behind by a crash could not reach a build.

Isolation is asserted, not just the red: each mutation must produce *exactly*
its own rule's row. Without that, a gate that reported all four rules on any
defect at all would pass every row below while being unable to tell a broken
link from a missing section.

Restore is verified by sha256 and, unlike the other controls here, is also
wrapped in a `finally`. The others mutate files inside `.claude/aw`, where a
leftover mutant shows up in the next `git status` a session runs. These mutate
documents two directories away from anything else this suite touches, and a
control that dies holding one would leave a rotted document behind and a green
ratchet unable to see it -- the exact failure it exists to refuse.
"""
import hashlib
import pathlib
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import REPO, pinned_interpreter  # noqa: E402

HERE = pathlib.Path(__file__).resolve().parent
CHECK = HERE / "check_meta_clean.py"
LAUNCH = pinned_interpreter()

CONTRIBUTING = REPO / "apps/cube/CONTRIBUTING.md"
README = REPO / "apps/cube/README.md"

# Every mutation trips two rows, not one, and the second is the same for all
# four: besides the per-rule count, the gate asserts over the *ratcheted set* of
# the live report as a whole. That is not a duplicate of the rule row. The
# gate's exit-code row ("agrees with its own report") stays green under every
# mutation here -- a planted finding makes `meta.py` exit 1 and report it, which
# is agreement -- so the row that has to go red is the one reading the ratcheted
# findings. It is declared here rather than filtered out: a mutation that
# stopped tripping it would mean the gate had gone back to certifying by the
# per-rule counters alone. (The label is the gate's own row text; the two
# drifted apart once, and the control was red on every round for it.)
EXIT_ROW = "FAIL the live run reports no ratcheted finding"

# (label, target, anchor, mutant, the reds it must produce and no others)
MUTATIONS = [
    ("M1-orphaned-marker", CONTRIBUTING, "## Verification",
     "<!-- aw:meta:demo:start -->\nspliced by nothing\n<!-- aw:meta:demo:end -->\n\n"
     "## Verification",
     ["FAIL M1: no M1 findings in this checkout", EXIT_ROW]),
    ("M2-dead-command", CONTRIBUTING, "## Verification",
     "Run `aw wi list` to see the queue.\n\n## Verification",
     ["FAIL M2: no M2 findings in this checkout", EXIT_ROW]),
    ("M3-broken-link", CONTRIBUTING, "## Verification",
     "See [the design](no/such/file.md).\n\n## Verification",
     ["FAIL M3: no M3 findings in this checkout", EXIT_ROW]),
    # A deletion rather than an insertion: M4 fires on what a document does not
    # say, so the only mutation that reaches it is taking the section away.
    ("M4-missing-section", README, "## Brief", "## Overview",
     ["FAIL M4: no M4 findings in this checkout", EXIT_ROW]),
]


def gate():
    r = subprocess.run([*LAUNCH, str(CHECK)], capture_output=True, text=True)
    return r.returncode, r.stdout


def reds_of(out):
    return [ln.split(" -- ")[0] for ln in out.splitlines() if ln.startswith("FAIL")]


baseline_code, baseline_out = gate()
print(f"== baseline == {baseline_out.strip().splitlines()[-1]} (exit={baseline_code})")

failures = []
for label, target, anchor, mutant, expected in MUTATIONS:
    original = target.read_bytes()
    before = hashlib.sha256(original).hexdigest()
    text = original.decode("utf-8")
    if text.count(anchor) != 1:
        failures.append(f"{label}: anchor occurs {text.count(anchor)}x, declared 1x")
        print(f"\n== {label} == ANCHOR COUNT WRONG ({text.count(anchor)})")
        continue

    try:
        target.write_text(text.replace(anchor, mutant), encoding="utf-8")
        code, out = gate()
    finally:
        target.write_bytes(original)
    after = hashlib.sha256(target.read_bytes()).hexdigest()

    reds = reds_of(out)
    isolated = reds == expected
    print(f"\n== {label} == exit={code}")
    for line in reds:
        print(f"   RED  {line}")
    print(f"   isolation: {'exactly the expected assertion' if isolated else f'UNEXPECTED: {reds}'}")
    print(f"   restore:   {'byte-identical' if before == after else 'FAILED'} ({before[:16]}...)")

    if not isolated or before != after or code == 0:
        failures.append(label)

restored_code, restored_out = gate()
print(f"\n== restored == {restored_out.strip().splitlines()[-1]} (exit={restored_code})")

ok = not failures and restored_code == 0 and baseline_code == 0
print("=> " + ("GREEN" if ok else f"RED ({failures or 'gate not green after restore'})"))
sys.exit(0 if ok else 1)
