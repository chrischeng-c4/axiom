"""Prove `epic.py` still works when it lives outside every checkout.

Named `probe_plugin_root.py` until 2026-08-21, when the `plugins/aw/` plugin
was deleted. The condition it was written for -- a git-marketplace install
copying the plugin under `~/.claude/plugins/`, where no `aw.toml` exists on any
parent path -- can no longer arise. The property it measures survives the
plugin, which is why the file did too: `_repo_root()` originally walked up from
`__file__` only, so a copy anywhere outside a checkout died on import, before
any verb ran, with a message blaming the user's checkout.

That is not hypothetical now either. The scripts moved once already, and the
next thing that copies them -- a dispatch worktree, an agent staging a
directory to read it, a scratch clone -- reproduces exactly this shape.
Resolution walks from cwd first and from `__file__` second, and this is the
only gate that measures the first half.

The staging directory sits under the home Claude dir on purpose: outside every
checkout, and somewhere a real tool would plausibly put it.

The *whole script directory* is staged, not one file. The fourteen scripts import
each other by `Path(__file__).parent` -- `e2e.py` and `impl.py` both load
`leg.py` that way, and `leg.change_module()` loads `change.py` -- so
staging a lone file turns "the script imports a sibling" into a probe failure
that reads like a repository defect.
"""
import pathlib
import shutil
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import REPO, SCRIPT  # noqa: E402

STAGE = pathlib.Path.home() / ".claude/tmp/aw-offtree-root-probe"
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
#
# Which project is used matters only in that it must have its own `aw.toml`:
# without one there is no nearer marker to prefer, and the assertion passes for
# the wrong reason. So that is checked rather than assumed.
NESTED = REPO / "apps/preview"
check("positive control: the nested project has its own `aw.toml`",
      (NESTED / "aw.toml").is_file(), str(NESTED / "aw.toml"))

code, out, err = run(NESTED, "bodydir")
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

# ------------------------------------------------------------------------
# A checkout nested inside another checkout.
#
# Outermost-marker resolution is right within one tree and wrong across two,
# and until 2026-08-28 nothing here measured the second case. Claude Code
# creates its worktrees under `.claude/worktrees/<name>/`, so an agent session
# stands in a checkout whose enclosing directory is *also* a checkout with its
# own root `aw.toml`. The unbounded walk returned the outer one, which meant a
# phase script run from an agent worktree read -- and would have written to --
# the other tree, and disagreed with `meta.py`, which resolves the root by
# asking git.
#
# The shape is reproduced rather than borrowed: two `git init`s, one inside the
# other, each with an `aw.toml`. `rev-parse --show-toplevel` answers the inner
# one for both a linked worktree and a nested repository, which is the only
# property the boundary rule reads.
NEST = STAGE / "nested"
OUTER, INNER = NEST / "outer", NEST / "outer" / "inner"
INNER.mkdir(parents=True, exist_ok=True)
for tree in (OUTER, INNER):
    (tree / "aw.toml").write_text("[agentic_workflow]\n", encoding="utf-8")
    subprocess.run(["git", "init", "-q", str(tree)], check=True,
                   capture_output=True)

# Positive controls. Without both of these the case below passes for the wrong
# reason: if the outer marker were absent there would be nothing to wrongly
# prefer, and if git did not call the inner tree its own checkout there would
# be no boundary to stop at.
check("positive control: the enclosing checkout carries its own `aw.toml`",
      (OUTER / "aw.toml").is_file() and OUTER in INNER.parents,
      f"outer={OUTER}")
inner_top = subprocess.run(
    ["git", "-c", "core.fsmonitor=false", "rev-parse", "--show-toplevel"],
    cwd=INNER, capture_output=True, text=True)
check("positive control: git calls the nested tree its own checkout",
      inner_top.returncode == 0
      and pathlib.Path(inner_top.stdout.strip()).resolve() == INNER.resolve(),
      f"got={inner_top.stdout.strip()!r}")

code, out, err = run(INNER, "bodydir")
check("a checkout nested in another checkout resolves to itself",
      code == 0 and out == str(INNER.resolve() / ".aw/workitems/epics"),
      f"exit={code} got={out!r}")

# And the outer tree still resolves to itself -- the boundary must not have
# turned into "always take the nearest marker", which is the failure the
# outermost rule exists to prevent.
code, out, err = run(OUTER, "bodydir")
check("the enclosing checkout still resolves to itself",
      code == 0 and out == str(OUTER.resolve() / ".aw/workitems/epics"),
      f"exit={code} got={out!r}")

shutil.rmtree(STAGE)
print("\n=> " + ("GREEN" if not fails else "RED: " + ", ".join(fails)))
sys.exit(1 if fails else 0)
