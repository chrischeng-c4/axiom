#!/usr/bin/env python3
"""Negative control for `check_plugin.py`, one mutation per assertion class.

Re-pointed on 2026-08-21 when `plugins/aw/` was deleted: the skills are read
from `.claude/skills/aw-*/` and the scripts from `.claude/aw/scripts/`. The
`${CLAUDE_PLUGIN_ROOT}` mutations became in-repo-path mutations, because the
variable they broke no longer appears in any body -- and a control with no
live target is a rule nobody is measuring.

A checker that has never been seen to fail is a checker nobody has measured.
Each mutation below is a break that really ships:

  script-path    `scripts/` -> `script/` inside the in-repo path a SKILL.md
                 names. The skill loads cleanly and dies on first use. Both
                 occurrences in the body are mutated and both are expected to
                 go red -- the rule fires per named path, so a control that
                 mutated one would be green off the other's survival.
  skill-name     a cross-reference to a skill that does not exist. This is the
                 defect that actually shipped: with the directories named
                 `aw-epic-*`, every plugin-qualified `wi-epic-*` reference in
                 both bodies pointed at nothing, and no assertion compared the
                 two.
  frontmatter-label
                 the frontmatter `name:` drops to the dash form. Nothing breaks
                 -- the command comes from the directory -- but the skill list
                 then shows the invocation twice and the label never, and the
                 next reader takes the frontmatter for the thing that decides.
  h1-colon       the body's H1 keeps the colon form the directories carried
                 until 2026-08-26. Two rules fire: the H1 no longer names the
                 real invocation, and the body now carries a `/aw:` reference
                 that nothing can invoke. Both are expected, in that order.
  unused-verb    a skill starts naming a verb that is declared unused. The
                 declaration is an exemption from verb coverage, so it has to
                 expire the moment it stops being true -- otherwise it is a
                 permanent hole shaped like whatever was once out of scope.
  gh-write       reconcile opens a child with the tracker's own CLI again. This
                 one is a restoration, not an invention: the block below is the
                 one the skill carried until child creation moved to the change
                 grill, and it is how a body no validator has ever seen gets
                 filed -- the issue exists, the labels are right, and nothing
                 ever ran `change.py validate` against what is inside it.
  scripts-copy   a skill grows its own `scripts/` directory. Not a load error:
                 a second copy of a schema works perfectly right up until the
                 two readings drift, which is the failure the shared location
                 exists to prevent.
  ask-in-gate    a procedural skill grows an AskUserQuestion. At a gate the only
                 thing left to ask about is whether the gate counts, so this is
                 how a refusal turns into a negotiation.
  no-ask         an interviewing skill loses every AskUserQuestion. The body
                 still reads like an interview; it just stops holding one, and
                 the answers come from the agent instead of the human.
  unpinned       the change ladder invokes `logic.py` with a bare `python3`. It
                 reads as the shorter, more normal form of the same line, and it
                 is a ModuleNotFoundError on any interpreter below 3.11. The
                 body carries one pinned `logic.py` launcher, so one red is
                 expected; the rule fires per invocation, and the declared
                 count is what keeps a second launcher from hiding behind it.

                 The target is `logic.py` on purpose. It imports no TOML itself
                 -- it reaches `tomllib` only through the `unit.py` it loads --
                 so under a direct-import-only pin population it would not be in
                 the set at all, this mutation would produce no reds, and the
                 control would fail. That is the point: it is what holds the
                 transitive half of that derivation in place.
  unclassified   a skill drops out of the interviewing/procedural partition.
                 Nothing fails to load and no body changes -- the skill simply
                 stops having any per-skill rule applied to it, which is the
                 exact hole the partition replaced.

Each mutation declares how many times its anchor occurs and replaces all of
them. A single-occurrence default would have quietly made `no-ask` a no-op: the
word appears three times, replacing one leaves two, and the rule that reads
"is it mentioned at all" would have stayed green against a mutation that looked
like it had been applied.

Each is applied alone, and the control demands *isolation* -- exactly the
matching assertion goes red, not merely "something did". Restoration writes the
captured bytes back and verifies by sha256; a reverse string-replace would
restore a file that only looks like the original.

`scripts-copy` is a directory rather than an edit, so it carries its own
apply/revert pair and asserts the tree is back to where it started rather than
a digest.
"""
import hashlib
import pathlib
import shutil
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import HERE, skill_dir  # noqa: E402

CHECK = HERE / "check_plugin.py"
GRILL = skill_dir("grill-me-to-epic") / "SKILL.md"
CHANGE_GRILL = skill_dir("grill-me-to-change") / "SKILL.md"
RECONCILE = skill_dir("grill-epic-to-changes") / "SKILL.md"
# The `ask-in-gate` and `unpinned` mutations have been re-pointed three times
# now: from `wi-ec-commit`/`wi-ec-start` to `codex-review` when the leg
# wrappers were retired, to `codex-code-review` when that was split in two and
# the `ec -> td -> cb` scripts were deleted outright, and to `go-tdd-for-change`
# when the semantic review left the ladder on 2026-08-26. A control whose
# target disappears has to be re-pointed rather than dropped -- a rule with no
# live target is a rule nobody is measuring, and it reads identically to a rule
# that passes.
TDD_CHANGE = skill_dir("go-tdd-for-change") / "SKILL.md"
PATHS = HERE / "_paths.py"
COPY_DIR = skill_dir("grill-epic-to-changes") / "scripts"

# (label, target, anchor, mutant, expected reds, occurrences)
MUTATIONS = [
    (
        "script-path",
        GRILL,
        ".claude/aw/scripts/epic.py",
        ".claude/aw/script/epic.py",
        ["FAIL grill-me-to-epic: script path .claude/aw/script/epic.py exists"],
        2,
    ),
    (
        "skill-name",
        GRILL,
        "`/aw-grill-epic-to-changes`",
        "`/aw-grill-epic-to-change`",
        ["FAIL grill-me-to-epic: every /aw-<skill> reference resolves to a real skill"],
    ),
    (
        "frontmatter-label",
        GRILL,
        "name: aw:grill-me-to-epic",
        "name: aw-grill-me-to-epic",
        ["FAIL grill-me-to-epic: frontmatter name is the listed label"],
    ),
    (
        "h1-colon",
        GRILL,
        "# /aw-grill-me-to-epic",
        "# /aw:grill-me-to-epic",
        ["FAIL grill-me-to-epic: the body's H1 is the real invocation",
         "FAIL grill-me-to-epic: names no colon-form /aw: invocation"],
    ),
    (
        "unused-verb",
        CHANGE_GRILL,
        "change.py bodydir        # -> <repo>/.aw/workitems/changes, created if missing",
        "change.py bodydir        # -> <repo>/.aw/workitems/changes, created if missing\n"
        "change.py adopt <path> <iid>   # rename a staged body",
        ["FAIL change.py: `adopt` is declared unused, and the declaration holds"],
    ),
    (
        "gh-write",
        RECONCILE,
        "should exist; the grill decides what each one says.",
        "should exist; the grill decides what each one says.\n"
        "\n"
        "```\n"
        'gh issue create --repo <repo> --title "<title>" '
        "--body-file <bodydir>/<slug>.md \\\n"
        "  --label type:change --label epic:<iid>\n"
        "```",
        ["FAIL grill-epic-to-changes: names no gh issue/pr write command"],
    ),
    (
        "ask-in-gate",
        TDD_CHANGE,
        "Twelve commands, in this order, and each one prints the next:",
        "Twelve commands, in this order, and each one prints the next:\n"
        "\n"
        "If a phase refuses, use AskUserQuestion to ask the human whether to "
        "record the phase as passed anyway.",
        ["FAIL go-tdd-for-change: names no AskUserQuestion, because a gate "
         "has nothing to ask"],
    ),
    (
        "no-ask",
        CHANGE_GRILL,
        "AskUserQuestion",
        "consult the human",
        ["FAIL grill-me-to-change: names AskUserQuestion, so it reaches the human"],
        3,
    ),
    (
        "unpinned",
        TDD_CHANGE,
        'uv run --python 3.13 --no-project ".claude/aw/scripts/logic.py"',
        'python3 ".claude/aw/scripts/logic.py"',
        ["FAIL go-tdd-for-change: `logic.py` is invoked through the pinned interpreter"],
    ),
    (
        "unclassified",
        PATHS,
        'PROCEDURAL = ("check-meta", "go-tdd-for-change", "go-tdd-for-epic")',
        "PROCEDURAL = ()",
        ["FAIL every skill is classified exactly once"],
    ),
]


def checker():
    r = subprocess.run([sys.executable, str(CHECK)], capture_output=True, text=True)
    return r.returncode, r.stdout


def reds_of(out):
    return [ln.split(" -- ")[0] for ln in out.splitlines() if ln.startswith("FAIL")]


baseline_code, baseline_out = checker()
print(f"== baseline == {baseline_out.strip().splitlines()[-1]} (exit={baseline_code})")

failures = []
for label, target, anchor, mutant, expected, *rest in MUTATIONS:
    hits = rest[0] if rest else 1
    original = target.read_bytes()
    before = hashlib.sha256(original).hexdigest()
    text = original.decode("utf-8")
    if text.count(anchor) != hits:
        failures.append(f"{label}: anchor occurs {text.count(anchor)}x, declared {hits}x")
        print(f"\n== {label} == ANCHOR COUNT WRONG ({text.count(anchor)}, declared {hits})")
        continue

    target.write_text(text.replace(anchor, mutant), encoding="utf-8")
    code, out = checker()
    reds = reds_of(out)

    target.write_bytes(original)
    after = hashlib.sha256(target.read_bytes()).hexdigest()

    isolated = reds == expected
    print(f"\n== {label} == exit={code}")
    for line in reds:
        print(f"   RED  {line}")
    print(f"   isolation: {'exactly the expected assertion' if isolated else f'UNEXPECTED: {reds}'}")
    print(f"   restore:   {'byte-identical' if before == after else 'FAILED'} ({before[:16]}...)")

    if not isolated or before != after or code == 0:
        failures.append(label)

# -- the directory mutation ------------------------------------------------
label = "scripts-copy"
expected = ["FAIL grill-epic-to-changes: carries no scripts/ copy of its own"]
if COPY_DIR.exists():
    failures.append(f"{label}: {COPY_DIR} already exists, so the mutation proves nothing")
    print(f"\n== {label} == PRECONDITION FAILED: {COPY_DIR} already exists")
else:
    COPY_DIR.mkdir(parents=True)
    (COPY_DIR / "epic.py").write_text("# a second reading of the schema\n", encoding="utf-8")
    code, out = checker()
    reds = reds_of(out)

    shutil.rmtree(COPY_DIR)
    restored = not COPY_DIR.exists()

    isolated = reds == expected
    print(f"\n== {label} == exit={code}")
    for line in reds:
        print(f"   RED  {line}")
    print(f"   isolation: {'exactly the expected assertion' if isolated else f'UNEXPECTED: {reds}'}")
    print(f"   restore:   {'directory removed' if restored else 'FAILED -- still present'}")

    if not isolated or not restored or code == 0:
        failures.append(label)

restored_code, restored_out = checker()
print(f"\n== restored == {restored_out.strip().splitlines()[-1]} (exit={restored_code})")

ok = not failures and restored_code == 0 and baseline_code == 0
print("=> " + ("GREEN" if ok else f"RED ({failures or 'checker not green after restore'})"))
sys.exit(0 if ok else 1)
