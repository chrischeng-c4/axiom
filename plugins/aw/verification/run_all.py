#!/usr/bin/env python3
"""Run every gate in this directory and report one verdict.

Order matters twice over. Each checker runs before the negative control that
mutates the file it reads. And the manifest pair runs first, so `check_plugin.py`
-- which reads the same `plugin.json` -- lands *after* the control that mutates
it, and a restore that silently failed is caught by the next checker rather
than by the next session. The engine-split pair sits after the coverage pair
for the same reason: both read `epic.py`, and the coverage control mutates it.

Measurement scripts are not run here. They hit the tracker over the network and
produce evidence for a decision, not a pass/fail; `check_coverage_rule.py`
names the snapshot it needs when it is missing.
"""
import pathlib
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent

GATES = [
    "check_manifests_cli.py",
    "check_manifests_cli_negative_control.py",
    "check_plugin.py",
    "check_plugin_negative_control.py",
    "check_coverage_rule.py",
    "check_coverage_rule_negative_control.py",
    "check_engine_split.py",
    "check_engine_split_negative_control.py",
    "check_change_schema.py",
    "check_change_schema_negative_control.py",
    "probe_plugin_root.py",
    "probe_local_verbs.py",
    # Exempt from the ordering rule above, and last because it is the slowest.
    # It mutates nothing in this checkout: its fixture is a `tempfile` tree with
    # its own `aw.toml` and its own git repository, so it can neither be
    # disturbed by a control above nor leave residue for one below.
    "check_ec_flow.py",
]

results = []
for name in GATES:
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

print("\n=> " + ("ALL GREEN" if not failed else f"RED: {', '.join(failed)}"))
sys.exit(1 if failed else 0)
