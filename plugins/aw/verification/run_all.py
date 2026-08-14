#!/usr/bin/env python3
"""Run the gates in this directory and report one verdict.

Two modes, because the two halves answer different questions and cost two
orders of magnitude apart.

The **checkers** ask whether this tree is admissible. That is the question a
working session asks, and it is answered in about nine seconds.

The **negative controls** ask whether a checker can be seen to fail at all.
That question is about the gate rather than the tree, so its answer only
changes when a gate changes -- and answering it is expensive by construction:
each control mutates the thing under test once per declared defect and re-runs
the *whole* checker for each mutation. `check_plugin_negative_control.py` alone
is nine such rounds and half this suite's runtime.

So the controls are opt-in, and the thing that makes the split safe is that the
default mode is not allowed to sound like the full one. A run that skipped
every discrimination proof must never print the string a full run prints,
because that string is what gets pasted as evidence. It names the controls it
did not run instead, so the gap is in the output rather than in someone's
memory of which flag they used.

Order matters twice over, and the pairing below is what preserves it. Each
checker runs immediately before the control that mutates the file it reads. And
the manifest pair runs first, so `check_plugin.py` -- which reads the same
`plugin.json` -- lands *after* the control that mutates it, and a restore that
silently failed is caught by the next checker rather than by the next session.
The engine-split pair sits after the coverage pair for the same reason: both
read `epic.py`, and the coverage control mutates it. Dropping the controls
cannot disturb any of this, because it removes the second element of each pair
and never reorders the first.

Measurement scripts are not run in either mode. They hit the tracker over the
network and produce evidence for a decision, not a pass/fail;
`check_coverage_rule.py` names the snapshot it needs when it is missing.
"""
import pathlib
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent

FLAG = "--with-negative-controls"

# (checker, its negative control). `None` where a gate has no control: the two
# probes stage their own throwaway trees and `check_ec_flow.py` carries its
# controls inside itself -- 30 of them, each already a declared mutation.
SUITE = [
    ("check_manifests_cli.py", "check_manifests_cli_negative_control.py"),
    ("check_plugin.py", "check_plugin_negative_control.py"),
    ("check_coverage_rule.py", "check_coverage_rule_negative_control.py"),
    ("check_engine_split.py", "check_engine_split_negative_control.py"),
    ("check_change_schema.py", "check_change_schema_negative_control.py"),
    # Reads the epic snapshot and calls `order_children` as a pure function;
    # nothing is spawned and nothing is written, so it costs about as much as
    # the probes and sits with them rather than with the flow gate below.
    ("check_epic_order.py", None),
    ("probe_plugin_root.py", None),
    ("probe_local_verbs.py", None),
    # Exempt from the ordering rule above, and last because it is the slowest.
    # It mutates nothing in this checkout: its fixture is a `tempfile` tree with
    # its own `aw.toml` and its own git repository, so it can neither be
    # disturbed by a control above nor leave residue for one below.
    ("check_ec_flow.py", None),
]

unknown = [a for a in sys.argv[1:] if a != FLAG]
if unknown:
    raise SystemExit(f"usage: {pathlib.Path(sys.argv[0]).name} [{FLAG}]\n"
                     f"error: unrecognized argument(s): {' '.join(unknown)}")

controls = FLAG in sys.argv[1:]
gates = [name
         for checker, control in SUITE
         for name in ((checker, control) if controls else (checker,))
         if name]
skipped = [] if controls else [c for _checker, c in SUITE if c]

results = []
for name in gates:
    r = subprocess.run([sys.executable, str(HERE / name)], capture_output=True, text=True)
    verdict = "GREEN" if r.returncode == 0 else f"RED (exit {r.returncode})"
    results.append((name, r.returncode, r.stdout))
    print(f"{verdict:16s} {name}")

failed = [name for name, code, _ in results if code != 0]
if failed:
    print("\n" + "=" * 70)
    for name, code, out in results:
        if code == 0:
            continue
        print(f"\n--- {name} ---")
        for line in out.splitlines():
            if line.startswith("FAIL") or line.startswith("=>") or "RED" in line:
                print("  " + line)

if skipped:
    # Named rather than counted. A count reads as bookkeeping; the names say
    # which specific claims -- "this gate can be seen to fail" -- went
    # unmeasured, and they are the only claims this mode cannot make.
    print("\nnot run (no gate here was proven able to fail):")
    for name in skipped:
        print(f"  {name}")

if failed:
    print(f"\n=> RED: {', '.join(failed)}")
elif skipped:
    print(f"\n=> CHECKERS GREEN -- negative controls not run; `{FLAG}` for the full suite")
else:
    print("\n=> ALL GREEN")

sys.exit(1 if failed else 0)
