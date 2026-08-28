#!/usr/bin/env python3
"""Negative control for `check_plugin.py`, one mutation per assertion class.

Re-pointed on 2026-08-21 when `plugins/aw/` was deleted: the skills are read
from `.claude/skills/aw-*/` and the scripts from `.claude/aw/scripts/`. The
`${CLAUDE_PLUGIN_ROOT}` mutations became in-repo-path mutations, because the
variable they broke no longer appears in any body -- and a control with no
live target is a rule nobody is measuring.

Re-pointed again on 2026-08-27, when the eleven-skill population collapsed to
six and the `wi -> e2e -> unit -> logic` ladder became `wi -> e2e -> impl`.
Every mutation below used to target `grill-me-to-epic`, `grill-me-to-change`,
`grill-epic-to-changes` or `go-tdd-for-change` -- all four deleted this round
-- or named `logic.py`, also deleted. A control whose target is gone is not a
control, it is a `FileNotFoundError` waiting for the next run; each mutation
now targets one of the six skills that actually load
(`.claude/aw/verification/_paths.py:SKILLS`), chosen for having exactly the
anchor text the mutation needs and no more.

A checker that has never been seen to fail is a checker nobody has measured.
Each mutation below is a break that really ships:

  script-path    `scripts/` -> `script/` inside the in-repo path a SKILL.md
                 names. The skill loads cleanly and dies on first use. There is
                 exactly one backtick-quoted `.claude/aw/...` path across the
                 six bodies now -- `prepare-goal`'s reference to `epic.py` --
                 so this mutation carries one occurrence, not two: the prior
                 two-occurrence shape belonged to a body that no longer exists.
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
                 Targets `epic.py adopt`, appended beside the `epic.py create`
                 fence `grill-meta-to-wis` already carries -- the same
                 exemption `change.py adopt` used to exercise, now folded into
                 the one skill that drives both facades.
  gh-write       the epic/change grill opens a child with the tracker's own
                 CLI directly. This one is a restoration, not an invention:
                 the block below is the shape `grill-epic-to-changes` carried
                 before child creation was a script verb, and it is how a body
                 no validator has ever seen gets filed -- the issue exists,
                 the labels are right, and nothing ever ran `change.py
                 validate` against what is inside it. `grill-meta-to-wis`
                 inherited that responsibility, so it inherits this control.
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
  unpinned       `grill-meta-to-wis` invokes `wis.py` with a bare `python3`. It
                 reads as the shorter, more normal form of the same line, and it
                 is a ModuleNotFoundError on any interpreter below 3.11. The
                 body carries exactly one pinned `wis.py` launcher (a second
                 lives in `prepare-goal`, untouched by this mutation), so one
                 red is expected; the rule fires per invocation, and the
                 declared count is what keeps a second launcher from hiding
                 behind it.

                 The target is `wis.py` on purpose. It imports no TOML itself
                 -- it reaches `tomllib` only through the `import e2e` at its
                 top, and `e2e.py` is what imports `tomllib` directly -- so
                 under a direct-import-only pin population it would not be in
                 the set at all, this mutation would produce no reds, and the
                 control would fail. That is the point: it is what holds the
                 transitive half of that derivation in place. The prior
                 version made the same argument about `logic.py`, which
                 reached `tomllib` only through the `unit.py` it loaded with
                 `leg.sibling`; that edge is `SIBLING_EDGE` in `check_plugin.py`,
                 while `wis.py`'s is `IMPORT_EDGE` -- a different edge shape,
                 same argument.
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
PREPARE_GOAL = skill_dir("prepare-goal") / "SKILL.md"
E2E_FOR_WI = skill_dir("e2e-for-wi") / "SKILL.md"
IMPL_FOR_WI = skill_dir("impl-for-wi") / "SKILL.md"
GRILL_META = skill_dir("grill-me-to-meta") / "SKILL.md"
GRILL_WIS = skill_dir("grill-meta-to-wis") / "SKILL.md"
ASK_USER = skill_dir("ask-user") / "SKILL.md"
PATHS = HERE / "_paths.py"
COPY_DIR = skill_dir("impl-for-wi") / "scripts"

# (label, target, anchor, mutant, expected reds, occurrences)
MUTATIONS = [
    (
        "script-path",
        PREPARE_GOAL,
        "`.claude/aw/scripts/epic.py`",
        "`.claude/aw/script/epic.py`",
        ["FAIL prepare-goal: script path .claude/aw/script/epic.py exists"],
    ),
    (
        "skill-name",
        PREPARE_GOAL,
        "`/aw-grill-meta-to-wis`",
        "`/aw-grill-meta-to-wi`",
        ["FAIL prepare-goal: every /aw-<skill> reference resolves to a real skill"],
    ),
    (
        "frontmatter-label",
        PREPARE_GOAL,
        "name: aw:prepare-goal",
        "name: aw-prepare-goal",
        ["FAIL prepare-goal: frontmatter name is the listed label"],
    ),
    (
        "h1-colon",
        PREPARE_GOAL,
        "# /aw-prepare-goal",
        "# /aw:prepare-goal",
        ["FAIL prepare-goal: the body's H1 is the real invocation",
         "FAIL prepare-goal: names no colon-form /aw: invocation"],
    ),
    (
        "unused-verb",
        GRILL_WIS,
        'epic.py create --title "<title>" --project <project> '
        "--priority <p0|p1|p2|p3> --body-file <path>",
        'epic.py create --title "<title>" --project <project> '
        "--priority <p0|p1|p2|p3> --body-file <path>\n"
        "epic.py adopt <path> <iid>   # rename a staged body",
        ["FAIL epic.py: `adopt` is declared unused, and the declaration holds"],
    ),
    (
        "gh-write",
        GRILL_WIS,
        "interrupted run should leave whole children behind, never fragments.",
        "interrupted run should leave whole children behind, never fragments.\n"
        "\n"
        "```\n"
        'gh issue create --repo <repo> --title "<title>" '
        "--body-file <bodydir>/<slug>.md \\\n"
        "  --label type:change --label epic:<iid>\n"
        "```",
        ["FAIL grill-meta-to-wis: names no gh issue/pr write command"],
    ),
    (
        "ask-in-gate",
        E2E_FOR_WI,
        "different reasons and name different remediations.",
        "different reasons and name different remediations.\n"
        "\n"
        "If `verify` refuses, use AskUserQuestion to ask the human whether to "
        "treat it as passed anyway.",
        ["FAIL e2e-for-wi: names no AskUserQuestion, because a gate has nothing to ask"],
    ),
    (
        "no-ask",
        ASK_USER,
        "AskUserQuestion",
        "consult the human",
        ["FAIL ask-user: names AskUserQuestion, so it reaches the human"],
        3,
    ),
    (
        "unpinned",
        GRILL_WIS,
        'uv run --python 3.13 --no-project ".claude/aw/scripts/wis.py"',
        'python3 ".claude/aw/scripts/wis.py"',
        ["FAIL grill-meta-to-wis: `wis.py` is invoked through the pinned interpreter"],
    ),
    (
        "unclassified",
        PATHS,
        'PROCEDURAL = ("e2e-for-wi", "impl-for-wi")',
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
baseline_reds = set(reds_of(baseline_out))
if baseline_reds:
    print(f"   baseline already carries {len(baseline_reds)} FAIL row(s) not caused by "
          "this control -- isolation below is judged against reds *new* since baseline, "
          "not against an empty set:")
    for line in sorted(baseline_reds):
        print(f"   PRE  {line}")

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
    new_reds = [r for r in reds if r not in baseline_reds]

    target.write_bytes(original)
    after = hashlib.sha256(target.read_bytes()).hexdigest()

    isolated = new_reds == expected
    print(f"\n== {label} == exit={code}")
    for line in reds:
        flag = "RED " if line not in baseline_reds else "PRE "
        print(f"   {flag} {line}")
    print(f"   isolation: {'exactly the expected assertion, on top of baseline' if isolated else f'UNEXPECTED new reds: {new_reds}'}")
    print(f"   restore:   {'byte-identical' if before == after else 'FAILED'} ({before[:16]}...)")

    if not isolated or before != after or code == 0:
        failures.append(label)

# -- the directory mutation ------------------------------------------------
label = "scripts-copy"
expected = ["FAIL impl-for-wi: carries no scripts/ copy of its own"]
if COPY_DIR.exists():
    failures.append(f"{label}: {COPY_DIR} already exists, so the mutation proves nothing")
    print(f"\n== {label} == PRECONDITION FAILED: {COPY_DIR} already exists")
else:
    COPY_DIR.mkdir(parents=True)
    (COPY_DIR / "impl.py").write_text("# a second reading of the schema\n", encoding="utf-8")
    code, out = checker()
    reds = reds_of(out)
    new_reds = [r for r in reds if r not in baseline_reds]

    shutil.rmtree(COPY_DIR)
    restored = not COPY_DIR.exists()

    isolated = new_reds == expected
    print(f"\n== {label} == exit={code}")
    for line in reds:
        flag = "RED " if line not in baseline_reds else "PRE "
        print(f"   {flag} {line}")
    print(f"   isolation: {'exactly the expected assertion, on top of baseline' if isolated else f'UNEXPECTED new reds: {new_reds}'}")
    print(f"   restore:   {'directory removed' if restored else 'FAILED -- still present'}")

    if not isolated or not restored or code == 0:
        failures.append(label)

restored_code, restored_out = checker()
print(f"\n== restored == {restored_out.strip().splitlines()[-1]} (exit={restored_code})")
restored_reds = set(reds_of(restored_out))

ok = not failures and restored_code == baseline_code and restored_reds == baseline_reds
print("=> " + ("GREEN" if ok else f"RED ({failures or 'checker not back to baseline after restore'})"))
sys.exit(0 if ok else 1)
