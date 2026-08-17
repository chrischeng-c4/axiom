#!/usr/bin/env python3
"""Ask Claude Code's own validator whether these manifests are admissible.

Every other gate here measures the plugin against rules *I* wrote down. This
one measures it against the rules the tool actually enforces, which is the only
oracle that stays correct when Claude Code's schema moves without telling me.

Two assertions per manifest, and the second is the one worth having:

  exit 0        the manifest parses and satisfies the required schema
  no warnings   the manifest is clean, not merely accepted

The warning tier is not cosmetic. Measured against v2.1.227: a plugin named
`aw:epic` *passes* validation and emits only

    name: Plugin name "aw:epic" is not kebab-case. Claude Code accepts it, but
    the Claude.ai marketplace sync requires kebab-case ...

so a name that works locally forever can still be inadmissible the day this
marketplace is synced. Gating on exit code alone would call that green.

`claude` missing from PATH is a failure, not a skip. This directory verifies a
Claude Code plugin; without the CLI the claim is simply unmeasured, and a skip
that prints green is how an unmeasured claim gets reported as a verified one.
"""
import concurrent.futures
import pathlib
import shutil
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import PLUGIN_DIR, REPO  # noqa: E402

fails = []


def check(label, ok, detail=""):
    print(f"{'PASS' if ok else 'FAIL'} {label}{(' -- ' + detail) if detail else ''}")
    if not ok:
        fails.append(label)


claude = shutil.which("claude")
check("the `claude` CLI is on PATH to be asked", bool(claude),
      "" if claude else "install Claude Code; without it these manifests are unmeasured")

# Both manifests: `validate` takes a plugin directory or a marketplace root and
# picks the manifest itself, so the two calls differ only in the path.
TARGETS = [
    ("plugin manifest", PLUGIN_DIR),
    ("marketplace manifest", REPO),
]

if claude:
    # Both spawns up front and concurrently: each pays the CLI's startup, they
    # read two different manifests, and neither writes anything -- so the order
    # they finish in cannot change either verdict. The checks below still run in
    # declaration order over the collected output. This gate is re-run once per
    # mutation by its negative control, so the serial version paid that startup
    # twelve times.
    with concurrent.futures.ThreadPoolExecutor(max_workers=len(TARGETS)) as pool:
        outputs = list(pool.map(
            lambda path: subprocess.run([claude, "plugin", "validate", str(path)],
                                        capture_output=True, text=True),
            [path for _label, path in TARGETS],
        ))

    for (label, _path), r in zip(TARGETS, outputs):
        out = r.stdout + r.stderr

        check(f"{label}: `claude plugin validate` accepts it",
              r.returncode == 0, f"exit={r.returncode}; {out.strip().splitlines()[-1:]}")

        # "passed with warnings" still exits 0 -- the exit code cannot see this.
        warnings = [ln.strip() for ln in out.splitlines() if ln.strip().startswith("❯")]
        check(f"{label}: accepted with no warnings",
              not warnings and "warning" not in out.lower(),
              f"warnings={warnings}")

        # Guard the two assertions above against a validator that silently
        # stopped saying anything: a green here must come from a real verdict.
        check(f"{label}: the validator actually rendered a verdict",
              "Validation" in out, f"stdout+stderr={out.strip()[:120]!r}")

print("\n=> " + ("GREEN" if not fails else f"RED: {len(fails)} failed"))
sys.exit(1 if fails else 0)
