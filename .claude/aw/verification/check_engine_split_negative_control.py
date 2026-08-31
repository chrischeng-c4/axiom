#!/usr/bin/env python3
"""Prove the engine/registry split gate sees realistic isolated regressions."""

from __future__ import annotations

import hashlib
import pathlib
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import ENGINE, HERE  # noqa: E402

GATE = HERE / "check_engine_split.py"
ANCHOR = '    leaf = "deliveries" if wi_type in wi_types.DELIVERY_TYPES else f"{wi_type}s"'

MUTATIONS = [
    (
        "literal-leak", ENGINE, ANCHOR,
        '    if wi_type == "feat":\n        pass\n' + ANCHOR,
        ["FAIL the engine carries no active or retired type literal in behavior"],
    ),
    (
        "copied-enum", ENGINE,
        "WORK_ITEM_TYPES = (*wi_types.DELIVERY_TYPES, *wi_types.INTAKE_TYPES)",
        'WORK_ITEM_TYPES = ("feat", "fix", "spike", "report")',
        ["FAIL the active enum is derived from the registry",
         "FAIL the engine carries no active or retired type literal in behavior"],
    ),
    (
        "control-blind", GATE,
        "registry_tree = ast.parse(registry_source)",
        "registry_tree = ast.parse(engine_source)",
        ["FAIL positive control: the registry owns every type literal"],
    ),
]


def gate() -> tuple[int, str]:
    proc = subprocess.run([sys.executable, str(GATE)], capture_output=True, text=True)
    return proc.returncode, proc.stdout


baseline_code, baseline_out = gate()
print(f"== baseline == {baseline_out.strip().splitlines()[-1]} (exit={baseline_code})")
failures: list[str] = []
for label, target, anchor, mutant, expected in MUTATIONS:
    original = target.read_bytes()
    before = hashlib.sha256(original).hexdigest()
    text = original.decode("utf-8")
    if text.count(anchor) != 1:
        failures.append(label)
        print(f"\n== {label} == ANCHOR NOT UNIQUE ({text.count(anchor)} hits)")
        continue
    target.write_text(text.replace(anchor, mutant), encoding="utf-8")
    code, output = gate()
    reds = [line.split(" -- ")[0] for line in output.splitlines() if line.startswith("FAIL")]
    target.write_bytes(original)
    after = hashlib.sha256(target.read_bytes()).hexdigest()
    isolated = reds == expected
    print(f"\n== {label} == exit={code}")
    for line in reds:
        print(f"   RED  {line}")
    print(f"   isolation: {'exactly expected' if isolated else f'UNEXPECTED: {reds}'}")
    print(f"   restore:   {'byte-identical' if before == after else 'FAILED'}")
    if code == 0 or not isolated or before != after:
        failures.append(label)

restored_code, restored_out = gate()
print(f"\n== restored == {restored_out.strip().splitlines()[-1]} (exit={restored_code})")
ok = not failures and baseline_code == 0 and restored_code == 0
print("=> " + ("GREEN" if ok else f"RED ({failures or 'gate not green after restore'})"))
raise SystemExit(0 if ok else 1)
