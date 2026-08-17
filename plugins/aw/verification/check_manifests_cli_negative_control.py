#!/usr/bin/env python3
"""Negative control for `check_manifests_cli.py`.

The point of that gate is the *warning* assertion, and the point of this
control is to prove the warning assertion is load-bearing rather than
decorative. So the first mutation is chosen to leave the exit code at 0:

  kebab-case   `"name": "aw"` -> `"name": "aw:epic"`. Claude Code accepts it.
               `claude plugin validate` exits 0 and prints "Validation passed
               with warnings". A gate reading only the exit code calls this
               green, and the plugin is nonetheless inadmissible to the
               Claude.ai marketplace sync.

  missing key  `name` removed entirely. This one the exit code does see, and it
               is here so the control covers both tiers the validator has.

Both mutations redden the *marketplace* assertions too, which is measured, not
incidental: `claude plugin validate <repo>` reaches through the marketplace
entry into the plugin.json it points at. The two targets in the gate are
therefore not independent, and expecting only the plugin-side assertion to
redden would be wrong.

Under `kebab-case` the control also re-runs the validator directly and records
its exit code, so the claim "the exit code cannot see this" is a number in the
output rather than a sentence in a docstring.

Applied one at a time against the real manifest, restored from captured bytes
and verified by sha256.
"""
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import HERE, PLUGIN_DIR, PLUGIN_JSON  # noqa: E402

CHECK = HERE / "check_manifests_cli.py"

PLUGIN_ACCEPTS = "plugin manifest: `claude plugin validate` accepts it"
PLUGIN_CLEAN = "plugin manifest: accepted with no warnings"
MARKET_ACCEPTS = "marketplace manifest: `claude plugin validate` accepts it"
MARKET_CLEAN = "marketplace manifest: accepted with no warnings"


def mutate_kebab(manifest):
    manifest["name"] = "aw:epic"


def mutate_missing(manifest):
    manifest.pop("name", None)


# Expected reds are ordered as check_manifests_cli.py prints them.
MUTATIONS = [
    ("kebab-case", mutate_kebab, [PLUGIN_CLEAN, MARKET_CLEAN], 0),
    ("missing key", mutate_missing,
     [PLUGIN_ACCEPTS, PLUGIN_CLEAN, MARKET_ACCEPTS, MARKET_CLEAN], 1),
]


def checker():
    r = subprocess.run([sys.executable, str(CHECK)], capture_output=True, text=True)
    return r.returncode, r.stdout


baseline_code, baseline_out = checker()
print(f"== baseline == {baseline_out.strip().splitlines()[-1]} (exit={baseline_code})")

failures = []
for label, mutate, expected, raw_expected in MUTATIONS:
    original = PLUGIN_JSON.read_bytes()
    before = hashlib.sha256(original).hexdigest()

    manifest = json.loads(original.decode("utf-8"))
    mutate(manifest)
    PLUGIN_JSON.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

    code, out = checker()
    reds = [ln[len("FAIL "):].split(" -- ")[0] for ln in out.splitlines()
            if ln.startswith("FAIL")]
    raw = subprocess.run([shutil.which("claude"), "plugin", "validate", str(PLUGIN_DIR)],
                         capture_output=True, text=True).returncode

    PLUGIN_JSON.write_bytes(original)
    after = hashlib.sha256(PLUGIN_JSON.read_bytes()).hexdigest()

    isolated = reds == expected
    print(f"\n== {label} == exit={code}")
    for line in reds:
        print(f"   RED  {line}")
    print(f"   isolation: {'exactly the expected assertions' if isolated else f'UNEXPECTED: {reds}'}")
    print(f"   restore:   {'byte-identical' if before == after else 'FAILED'} ({before[:16]}...)")
    print(f"   validator: `claude plugin validate` alone exits {raw}"
          + ("  <- a gate reading only this would call the mutant green" if raw == 0 else ""))

    if not isolated or before != after or code == 0 or raw != raw_expected:
        failures.append(label)

restored_code, restored_out = checker()
print(f"\n== restored == {restored_out.strip().splitlines()[-1]} (exit={restored_code})")

ok = not failures and restored_code == 0 and baseline_code == 0
print("=> " + ("GREEN" if ok else f"RED ({failures or 'checker not green after restore'})"))
sys.exit(0 if ok else 1)
