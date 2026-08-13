#!/usr/bin/env python3
"""Prove `epic.py` still works when it lives outside every checkout.

This is the condition a git-marketplace install creates: the plugin is copied
under `~/.claude/plugins/`, where no `aw.toml` exists on any parent path. The
original `_repo_root()` walked up from `__file__` only, so that copy would have
died on import -- before any verb ran, with a message blaming the user's
checkout. Installing from a local directory hides the bug completely, because
there the plugin root *is* the checkout.

The staging directory sits under the home Claude dir on purpose: same "outside
every checkout, inside ~/.claude" shape as a real install.

The *whole script directory* is staged, not one file. That is what an install
does -- measured: `diff -r ~/.claude/plugins/cache/axiom/aw/0.1.0/ plugins/aw/`
is silent, so the cache holds a copy of the entire plugin tree. Staging a lone
file was a simplification that happened to hold while the script was a single
module, and it would have turned "the script imports a sibling" into a probe
failure that reads like a repository defect.
"""
import pathlib
import shutil
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import REPO, SCRIPT  # noqa: E402

STAGE = pathlib.Path.home() / ".claude/tmp/aw-plugin-root-probe"
OFFTREE = STAGE / "scripts"
shutil.copytree(SCRIPT.parent, OFFTREE, dirs_exist_ok=True)
COPY = OFFTREE / SCRIPT.name

fails = []


def check(label, ok, detail=""):
    print(f"{'PASS' if ok else 'FAIL'} {label}{(' -- ' + detail) if detail else ''}")
    if not ok:
        fails.append(label)


# The copy really is outside every checkout -- otherwise this whole probe is
# measuring nothing.
above = [p for p in [COPY.resolve(), *COPY.resolve().parents] if (p / "aw.toml").is_file()]
check("the staged copy has no `aw.toml` on any parent path", not above, f"found={above}")


def run(cwd, *args):
    r = subprocess.run([sys.executable, str(COPY), *args], capture_output=True, text=True, cwd=cwd)
    return r.returncode, r.stdout.strip(), r.stderr.strip()


expected_bodydir = str(REPO / ".aw/workitems/epics")

code, out, err = run(REPO, "bodydir")
check("off-tree script resolves the checkout from cwd", code == 0, f"exit={code} err={err[:120]}")
check("and it resolves to THIS checkout", out == expected_bodydir, f"got={out!r}")

# The outermost marker wins, not the nearest. This checkout carries one
# `aw.toml` per project under apps/ and libs/, and only the repository root's
# holds the tracker configuration; taking the nearest would stage bodies under
# apps/<name>/.aw/ and read a config with no tracker in it.
code, out, err = run(REPO / "apps/agentic-workflow", "bodydir")
check("also from a subdirectory of the checkout", code == 0 and out == expected_bodydir,
      f"exit={code} got={out!r}")

# The failure has to stay loud: an off-tree script run from an off-tree cwd
# must refuse by name, not silently invent a root.
code, out, err = run(pathlib.Path.home(), "bodydir")
check("refuses when neither cwd nor script is in a checkout", code != 0, f"exit={code}")
check("and the refusal names both paths it searched",
      "working directory" in err and "the script" in err, f"err={err[:200]}")

# The in-repo copy must keep working unchanged.
code, out, err = run(REPO, "skeleton")
check("in-repo invocation still works", code == 0 and "## Requirements" in out, f"exit={code}")

shutil.rmtree(STAGE)
print("\n=> " + ("GREEN" if not fails else "RED: " + ", ".join(fails)))
sys.exit(1 if fails else 0)
