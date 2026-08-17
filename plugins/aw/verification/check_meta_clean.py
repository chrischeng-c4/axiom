#!/usr/bin/env python3
"""Refuse a checkout whose META-docs have rotted.

`check_meta_flow.py` measures the *validator*: every rule against a fixture
where the defect and its absence sit side by side. This measures the *tree* --
one run of `meta.py check` over this checkout, which has to come back clean.

Two files rather than two sections of one, because the two reds are different
diagnoses and the suite prints a filename. "RED check_meta_flow.py" means the
detector broke and nothing it said can be trusted; "RED check_meta_clean.py"
means the detector is fine and a document rotted. Collapsing them would make
every doc edit look like a detector regression to whoever reads the output.

This gate could not exist until the findings were cleared. `meta.py check`
reported 103 against this checkout when it was written -- 66 orphaned markers,
31 dead commands, 6 broken links -- and a ratchet that lands red is a ratchet
nobody can act on: it goes in the same bucket as the pre-existing failures
everyone has learned to scroll past. So it was a report first and a gate only
after the report read zero, which is also why there is no tolerated-failure
list below. The tolerated set is empty and has to stay empty.

Two ways this goes false-green, and a row for each.

*Population collapse.* `meta.py` derives its population from `git ls-files`, so
a run whose listing comes back empty prints `=> CLEAN` and exits 0 having read
nothing at all. That is the same shape as `cargo test -p <name>` with a name
that selects no package, and it is invisible in the exit code. The reported
count is cross-checked against an independent `git ls-files` here, and it has
to clear a floor -- a floor rather than a pin, so that adding a document is not
a red.

*A rule that stopped running.* Every count being zero is exactly what a run
that skipped a rule prints. So the reported `counts` keys are asserted to be
the full rule set, and each rule is proven reachable by one run of the same
invocation against a throwaway repository carrying one defect apiece. That is
this gate's negative control, and it is a different claim from the one
`check_meta_flow.py` makes: there each rule is shown to fire on its own defect
and on nothing else, here the call shape this gate certifies with is shown able
to report all four at once. Without it, deleting a rule from `RULES` would turn
the ratchet green rather than red.

The control repository is a `tempfile` tree with its own index, for the reason
`check_meta_flow.py` gives: a fixture built inside this checkout would be swept
into the live run it is the control for.
"""
import json
import pathlib
import subprocess
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import META_SCRIPT, REPO, load_script_module, pinned_interpreter  # noqa: E402

GIT = ("git", "-c", "core.fsmonitor=false")
LAUNCH = pinned_interpreter()

# Floors, not pins. The repository holds 182 tracked META-docs and 62 projects
# today; these sit far enough below to survive ordinary deletion and far enough
# above zero to refuse an empty listing.
DOC_FLOOR = 100
PROJECT_FLOOR = 20

fails = []


def check(label, ok, detail=""):
    print(f"{'PASS' if ok else 'FAIL'} {label}{(' -- ' + detail) if detail else ''}")
    if not ok:
        fails.append(label)


def report(root):
    """(exit code, parsed JSON) for the one invocation this gate certifies with."""
    proc = subprocess.run([*LAUNCH, str(META_SCRIPT), "check",
                           "--repo", str(root), "--format", "json"],
                          capture_output=True, text=True)
    try:
        return proc.returncode, json.loads(proc.stdout)
    except json.JSONDecodeError:
        return proc.returncode, None


# The rule set is read off the script rather than restated. A rule added here
# and not to `meta.py` would be a gate asserting a rule that does not exist; a
# rule added to `meta.py` and not here would go unratcheted, which is the
# failure this is guarding against in the first place.
RULES = tuple(load_script_module(META_SCRIPT, "metamod").RULES)
check("the script declares at least the four shipped rules", len(RULES) >= 4, f"{RULES}")

# --------------------------------------------------------------------------
# the negative control, first
# --------------------------------------------------------------------------
# Before the tree is certified, the certifying invocation is shown able to
# refuse one. Running it afterwards would leave a window where a run that
# reached nothing had already printed PASS on every live row.
CONTROL = {
    # No `## Brief`, and a `CONTRIBUTING.md` beside it, which is what makes the
    # directory a project and the missing section an M4.
    "p/README.md": [
        "# P",
        "",
        "## Capabilities",
        "",
        "- Nothing claimed yet.",
    ],
    "p/CONTRIBUTING.md": [
        "# Contributing to P",
        "",
        "<!-- aw:meta:demo:start -->",
        "An unfilled form, spliced by nothing.",
        "<!-- aw:meta:demo:end -->",
        "",
        "Run `aw wi list`, then read [the design](gone.md).",
    ],
}

with tempfile.TemporaryDirectory(prefix="meta-clean-control-") as tmp:
    root = pathlib.Path(tmp) / "checkout"
    root.mkdir()
    for rel, lines in CONTROL.items():
        path = root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    subprocess.run([*GIT, "init", "-q"], cwd=root, capture_output=True, text=True)
    staged = subprocess.run([*GIT, "add", "--", *CONTROL], cwd=root,
                            capture_output=True, text=True)
    check("the control repository staged its files", staged.returncode == 0,
          staged.stderr.strip())

    control_code, control = report(root)
    check("the control run parsed", control is not None, f"exit={control_code}")
    if control is None:
        print("\n=> RED: the control never ran, so nothing below would have been refused")
        sys.exit(1)

    check("the control run exits 1, distinguishably from clean", control_code == 1,
          f"exit={control_code}")
    silent = [rule for rule in RULES if not control["counts"].get(rule)]
    check("this invocation reaches every rule", not silent,
          f"reported nothing: {silent}; counts={control['counts']}")

# --------------------------------------------------------------------------
# the tree
# --------------------------------------------------------------------------
live_code, live = report(REPO)
check("the live run parsed", live is not None, f"exit={live_code}")
if live is None:
    print("\n=> RED: the live run produced no report; the tree was not measured")
    sys.exit(1)

pop = live["population"]

# Independent of `meta.py`'s own listing only in that it is a second call: both
# read the index. What it refuses is the run that measured a *different* tree --
# a `--repo` pointing somewhere else, a population filtered down to nothing by a
# selector. Against an empty index the two would agree at zero, which is what
# the floor below is for.
listed = subprocess.run([*GIT, "ls-files", "-z"], cwd=REPO,
                        capture_output=True, text=True)
expected = sum(1 for rel in listed.stdout.split("\0")
               if rel and rel.rsplit("/", 1)[-1] in ("CLAUDE.md", "README.md",
                                                     "CONTRIBUTING.md"))
check("the live run scanned the tracked META-docs this checkout holds",
      pop["scanned"] == expected == pop["tracked_meta_docs"],
      f"scanned={pop['scanned']} tracked={pop['tracked_meta_docs']} git={expected}")
check("and that population is not empty",
      pop["scanned"] > DOC_FLOOR and pop["project_readmes"] > PROJECT_FLOOR,
      f"{pop['scanned']} docs (floor {DOC_FLOOR}), "
      f"{pop['project_readmes']} projects (floor {PROJECT_FLOOR})")
check("every rule ran", set(live["counts"]) == set(RULES),
      f"reported={sorted(live['counts'])} declared={sorted(RULES)}")

# One row per rule rather than one row for the total, so that a red names the
# rule and the reader knows which kind of rot landed without re-running
# anything.
for rule in RULES:
    hits = [f for f in live["findings"] if f["rule"] == rule]
    check(f"{rule}: no {rule} findings in this checkout", not hits,
          "; ".join(f"{f['path']}:{f['line']} {f['message']}" for f in hits[:5])
          + (f" (+{len(hits) - 5} more)" if len(hits) > 5 else ""))

# Asserted separately from the counts above. They are read out of the JSON; this
# is the exit code the skill and any caller actually branch on, and a report that
# disagreed with its own status would make every other row here unfalsifiable.
check("the live run exits 0", live_code == 0,
      f"exit={live_code}, {len(live['findings'])} finding(s)")

print("\n=> " + ("GREEN" if not fails else f"RED ({len(fails)} failure(s))"))
sys.exit(1 if fails else 0)
