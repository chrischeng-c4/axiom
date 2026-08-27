#!/usr/bin/env python3
"""Re-runnable checker for the six `aw-*` skills and the scripts they drive.

The name is historical: `aw` was a Claude Code plugin at `plugins/aw/` until
2026-08-21, and this file checked its bundle. The plugin is deleted -- the
scripts are at `.claude/aw/scripts/`, the six skills load as project skills
out of `.claude/skills/aw-*/`, and the manifest assertions here were inverted
into absence assertions so a half-restored plugin goes red instead of quietly
loading a second copy of all six. There were eleven skills and a three-phase
`wi -> e2e -> unit -> logic` ladder until 2026-08-27, when `unit.py` and
`logic.py` merged into `impl.py` and four skills -- the epic and change
grills, the two `go-tdd-for-*` runners -- folded into `grill-meta-to-wis` and
the two per-phase skills below; that population, not this one, is the history
worth keeping legible.

Two assertion classes here are not obvious and are the reason this file exists.

**Script paths.** A SKILL.md names its script by an in-repo path, and nothing
at authoring time proves that file is there. A typo produces a skill that loads
cleanly and dies on first use.

**Invocation names.** Claude Code composes a plugin skill's name as
`plugin:directory` and ignores the frontmatter `name:` outright -- measured,
not assumed: with directories `aw-epic-grill`/`aw-epic-reconcile` under plugin
`aw-epic`, the skills registered as `aw-epic:aw-epic-grill`, while the
frontmatter declared a colon-form name of its own that was silently discarded.
Every cross-reference the two bodies carried therefore pointed at a skill that
did not exist, including reconcile's handoff back to grill. Nothing caught it
because nothing compared a body's invocation names against the directories that
produce them.

The invocation axis has since moved three times more, ending at `aw-<verb>`
directories (the command) beside an `aw:<verb>` frontmatter label, which
is exactly why these assertions read the directory listing instead of a
constant: a rename that misses one reference is the failure mode here, and it
has occurred on every rename so far.

Every absence assertion carries a positive control, and the extractors assert
they found something, so a regex that silently stops matching goes red instead
of green.
"""
import concurrent.futures
import functools
import json
import re
import subprocess
import sys

sys.path.insert(0, str(__import__("pathlib").Path(__file__).resolve().parent))
from _paths import (CHANGE_SCRIPT, DISPLAY_PREFIX, INTERVIEWING,  # noqa: E402
                    METADOC_SCRIPT, META_SCRIPT, PINNED_PYTHON,
                    E2E_SCRIPT, IMPL_SCRIPT, PROCEDURAL, REPO,
                    SCRIPT, SCRIPTS, SKILLS, SKILLS_DIR, SKILL_PREFIX,
                    WIS_SCRIPT, pinned_interpreter, skill_dir,
                    skill_invocation, skill_label)

# Each bundled script, and the verbs the skills must name for it. The two
# differ on purpose: an epic owns children and can be closed against them, a
# change has neither, so a change facade exposing `children` would mean the
# engine had leaked the epic's shape into the wrong type.
REQUIRED = {
    "epic.py": {"skeleton", "create", "update", "validate", "close", "show",
                "children", "order", "reconcile", "bodydir", "fetch"},
    "change.py": {"skeleton", "create", "update", "validate", "show",
                  "bodydir", "fetch"},
    # The two phases of the ladder. Unlike the deleted `wi-{ec,td,cb}-*`
    # wrappers there is one skill per phase rather than one skill per verb --
    # the sequence is fixed and each verb prints the next, so what a skill adds
    # is the entry point and the refusal list, not a wrapper per step. `start`
    # is required of both: a phase whose `start` stopped being named would be a
    # phase an agent could enter without the tree being checked clean first.
    #
    # `e2e.py` and the former `logic.py` carried `review-prompt` and `verdict`
    # as well until 2026-08-26, when the semantic review left the ladder. Note
    # what this table does not do: a verb a script still exposes but no skill
    # names is not refused here -- `MENTIONED` runs the other way, from the
    # skills to the scripts -- so the two subcommands were deleted from the
    # scripts in the same round rather than left as verbs `UNUSED` would have
    # had to exempt.
    #
    # `impl.py` has a fifth. `unit.py` and `logic.py` merged into it on
    # 2026-08-27, and the filename boundary that used to separate them went
    # with the merge; what replaced it is `red`, which records the named
    # failures mid-phase. It is required here for exactly that reason -- a
    # skill that drove `start / verify / test / commit` and never named `red`
    # would be driving a phase whose green is attributable to nothing.
    "e2e.py": {"start", "verify", "test", "commit"},
    "impl.py": {"start", "red", "verify", "test", "commit"},
    # The META-doc validator. One verb, and the singleton is the point: a
    # second verb here would be a verb that writes, and the thing this replaces
    # was deleted for writing.
    "meta.py": {"check"},
    # The META-doc run's refusal, and the one writer allowed to land it. Both
    # verbs are required of the skill because either alone is a hole: `check`
    # with no `commit` leaves the run to be committed by hand, which is how a
    # META-doc commit reaches history with no trailers for
    # `/aw-grill-meta-to-wis` to read; `commit` with no `check` named is a
    # skill that never says the run is measurable before it lands.
    "metadoc.py": {"check", "commit"},
    # The read-only promise/work-item gap reader. One verb, and it is required
    # for the same reason `meta.py check` is: `grill-meta-to-wis` opens with
    # this table, and a skill that reorganised the work-item set without
    # printing it first would be judgement with no measurement under it.
    "wis.py": {"gap"},
}

# Verbs a script exposes that no skill drives, and why. A silent gap between
# what a script can do and what any skill documents is how a verb rots; writing
# the exemption down turns it into a claim, and the claim is checked -- the verb
# must still exist, and no skill may have quietly started naming it.
ADOPT_WHY = ("`create` renames the staged body itself on both facades, so no "
             "skill needs `adopt`. It survives as the recovery path for a body "
             "whose tracker write landed outside the script, and "
             "`probe_local_verbs.py` is what keeps that path working -- a "
             "declaration here exempts a verb from being *named*, never from "
             "being tested.")
UNUSED = {
    "epic.py": {"adopt": ADOPT_WHY},
    "change.py": {"adopt": ADOPT_WHY},
    "e2e.py": {},
    "impl.py": {},
    "meta.py": {},
    "metadoc.py": {},
    "wis.py": {},
}
SCRIPT_PATHS = {"epic.py": SCRIPT, "change.py": CHANGE_SCRIPT,
                "e2e.py": E2E_SCRIPT, "impl.py": IMPL_SCRIPT,
                "meta.py": META_SCRIPT, "metadoc.py": METADOC_SCRIPT,
                "wis.py": WIS_SCRIPT}

# Scripts that cannot run under a bare `python3`, and the pin the skills must
# carry. Derived from the source rather than listed by hand: a script that grows
# a `tomllib` import joins this set on its own, and one that loses it leaves.
#
# Transitively, and that is not tidiness. `logic.py` imported no TOML itself and
# was therefore exempt from a direct-import-only derivation, while dying under
# 3.9 anyway -- it loaded `unit.py` through `leg.sibling`, and the import
# happened there. A direct-import-only derivation exempts exactly the scripts
# whose dependence is hardest to see by reading them, which is the opposite of
# what a derivation is for.
#
# There are two edge shapes now, not one. `impl.py` still reaches `e2e.py`
# through `leg.sibling("e2e", ...)` -- a dynamic, path-based load a static
# reader could miss, which is what `SIBLING_EDGE` exists to catch. `wis.py`,
# new on 2026-08-27, reaches `change.py`, `e2e.py`, `epic.py`, `meta.py` and
# `metadoc.py` through plain top-level `import e2e` statements instead -- the
# scripts directory sits on `sys.path` for exactly this reason, so a sibling
# script is importable like any module. That edge is not hard to see -- a
# reader can follow it with two file reads -- but it is still an edge the
# fixed point below has to close over, or `wis.py` would sit outside
# `NEEDS_PIN` while actually needing 3.11+ for `tomllib` two imports away.
# `IMPORT_EDGE` catches it; the intersection with `set(SCRIPT_PATHS)` in
# `DEPENDS` below is what keeps `import argparse` and the rest of the standard
# library from ever being read as an edge.
SIBLING_EDGE = re.compile(r'sibling\(\s*"([a-z0-9_]+)"')
IMPORT_EDGE = re.compile(r'^import ([a-z][a-z0-9_]*)\b', re.M)
# Tolerant of a missing file on purpose. A script is named in `SCRIPT_PATHS`
# before it is written, so that the gate driving it can be watched going red
# for the right reason first. Reading eagerly here would make that red a
# `FileNotFoundError` traceback at import -- which takes every *other*
# assertion in this file down with it, and reads as a broken gate rather than
# as an absent script. The absence is asserted explicitly below instead.
SOURCES = {name: (path.read_text(encoding="utf-8") if path.is_file() else "")
           for name, path in SCRIPT_PATHS.items()}
DEPENDS = {name: ({f"{s}.py" for s in SIBLING_EDGE.findall(text)} |
                  {f"{s}.py" for s in IMPORT_EDGE.findall(text)}) & set(SCRIPT_PATHS)
           for name, text in SOURCES.items()}
NEEDS_PIN = {name: path for name, path in SCRIPT_PATHS.items()
             if re.search(r"^import tomllib\b", SOURCES[name], re.M)}
while True:
    grown = {name: path for name, path in SCRIPT_PATHS.items()
             if name in NEEDS_PIN or DEPENDS[name] & set(NEEDS_PIN)}
    if grown.keys() == NEEDS_PIN.keys():
        break
    NEEDS_PIN = grown
ABSENT = "ship"
AW_INVOCATION = re.compile(r"`aw\s+[a-z]")

# The tracker's own CLI in write mode. Reconcile carried exactly the string
# below until child creation moved to the change grill, and this is the shape
# that comes back first: opening one issue by hand reads as a shortcut rather
# than a schema fork, right up until a body no validator ever saw is filed. The
# control is that literal string, so the detector is pinned to the real defect
# rather than to a caricature of it.
GH_WRITE = re.compile(r"gh\s+(?:issue|pr)\s+(?:create|edit|close|comment|delete|reopen)\b")
GH_WRITE_CONTROL = ('gh issue create --repo <repo> --title "<title>" '
                    "--body-file <bodydir>/<slug>.md \\\n"
                    "  --label type:change --label epic:<iid>")
# `${CLAUDE_PLUGIN_ROOT}` resolves nowhere now, so a SKILL.md naming a script
# names it by its in-repo path and nothing else. The extractor asserts it found
# something, so a body that stopped naming any path goes red rather than green.
IN_REPO_PATH = re.compile(r"`(\.claude/aw/[\w./:-]+)`")
PLUGIN_ROOT_PATH = re.compile(r"\$\{CLAUDE_PLUGIN_ROOT\}")
SKILL_REFERENCE = re.compile(rf"(?<![\w/.])/({re.escape(SKILL_PREFIX)}[\w-]+)")
# The form every reference carried until 2026-08-26. A body that still says
# `/aw:<skill>` names the display label, which nothing can invoke -- and a
# rename that misses one reference is the failure mode this file exists for.
STALE_REFERENCE = re.compile(rf"/{re.escape(DISPLAY_PREFIX)}[\w:-]+")

fails = []


def check(label, ok, detail=""):
    print(f"{'PASS' if ok else 'FAIL'} {label}{(' -- ' + detail) if detail else ''}")
    if not ok:
        fails.append(label)


# -- the plugin is gone ----------------------------------------------------
# Deleted on 2026-08-21. These four rows used to assert a `marketplace.json`,
# a `plugin.json`, a kebab-case plugin name and a relative in-repo source that
# resolved to the bundle. They are now the inverse assertion: those files must
# NOT be back, because a half-restored plugin would load a second copy of six
# skills that already load out of `.claude/skills/`, and the two copies would
# drift with nothing detecting it -- which is the exact condition the deletion
# was for.
check("no plugin marketplace is back", not (REPO / ".claude-plugin").exists())
check("no plugins/ tree is back", not (REPO / "plugins").exists())
settings = REPO / ".claude/settings.json"
check("settings.json exists", settings.is_file())
if settings.is_file():
    check("settings.json enables no plugin",
          "enabledPlugins" not in settings.read_text(encoding="utf-8"))

# -- the six skills and the seven scripts -----------------------------------
for skill in SKILLS:
    check(f"{skill}: SKILL.md is on disk", (skill_dir(skill) / "SKILL.md").is_file())
for name, path in sorted(SCRIPT_PATHS.items()):
    check(f"{name} is on disk", path.is_file(), str(path))

# Only the skills that are actually on disk, for the reason `SOURCES` above is
# lenient: a skill is registered before its SKILL.md is written, so the gate
# can be watched going red for "this file is missing" first. Reading eagerly
# here raised `FileNotFoundError` at module scope and took every assertion
# after this line down with it -- 60-odd of them, about six other skills --
# leaving a run that looked like a broken gate rather than an absent skill.
# The script side had already been fixed this way; this side had not.
#
# Downstream loops read `bodies` rather than `SKILLS`, or index it with a
# default. A per-skill rule therefore says nothing about an absent skill
# instead of crashing -- and cannot report a vacuous pass either, because the
# absence itself is a FAIL row above.
bodies = {s: path.read_text(encoding="utf-8")
          for s in SKILLS
          if (path := skill_dir(s) / "SKILL.md").is_file()}
joined = "\n".join(bodies.values())

# -- the skill directories are the invocation names ------------------------
# The directory name IS the command name, and registration rewrites characters
# it will not carry: a directory `probe:colon` registers as `probe-colon`. That
# is strictly worse than a rejection, because the skill loads and every body
# reference then points at a name nobody wrote. So the assertion is not "legal"
# but "survives unchanged" -- anything outside this class registers as
# something else. It is applied to the whole directory name, prefix included,
# because the prefix is where the colon lived until 2026-08-26.
SAFE_DIRECTORY = re.compile(r"[a-z0-9]+(?:-[a-z0-9]+)*")

# `.claude/skills/` is shared -- `agy-dispatch` lives there too -- so the
# population is the `aw-`-prefixed directories, not the whole listing. The
# prefix is stripped before comparing, which is also what proves it is present:
# a directory that lost it simply is not in `on_disk`, and the equality fails.
# The comparison is against `SKILLS`, not filtered by it, so an `aw-` directory
# nobody registered goes red by name instead of loading unnoticed.
on_disk = sorted(q.name[len(SKILL_PREFIX):] for q in SKILLS_DIR.iterdir()
                 if q.is_dir() and q.name.startswith(SKILL_PREFIX))
check("the aw- skills on disk are exactly the ones under test",
      on_disk == sorted(SKILLS), f"on disk={on_disk}")
for skill in SKILLS:
    check(f"{skill}: directory name survives registration unchanged",
          bool(SAFE_DIRECTORY.fullmatch(f"{SKILL_PREFIX}{skill}")),
          "registration rewrites anything outside [a-z0-9-]")

# Positive controls: the assertion must be able to refuse both shapes that were
# actually shipped -- the plugin-era colon inside the name, and the colon
# prefix the directories carried until 2026-08-26.
check("positive control: the directory-name rule refuses a colon",
      not SAFE_DIRECTORY.fullmatch("wi:epic:grill"))
check("positive control: the directory-name rule refuses the colon prefix",
      not SAFE_DIRECTORY.fullmatch("aw:wi-epic-grill"))

expected = {f"{SKILL_PREFIX}{s}" for s in SKILLS}
for skill, text in bodies.items():
    # The frontmatter name does not decide the command, but it is what the
    # skill list displays, so it carries the colon-form label. A label that
    # disagrees with the directory is a lie left where the next reader will
    # believe it, and a label in dash form is the invocation shown twice.
    front = re.search(r"^name:\s*(\S+)\s*$", text, re.M)
    check(f"{skill}: frontmatter declares a name", bool(front))
    if front:
        check(f"{skill}: frontmatter name is the listed label",
              front.group(1) == skill_label(skill),
              f"frontmatter={front.group(1)!r} label={skill_label(skill)!r}")

    check(f"{skill}: the body's H1 is the real invocation",
          re.search(rf"^#\s*{re.escape(skill_invocation(skill))}\s*$", text, re.M) is not None)

    referenced = set(SKILL_REFERENCE.findall(text))
    check(f"{skill}: the skill-reference extractor found references at all", bool(referenced))
    unknown = sorted(referenced - expected)
    check(f"{skill}: every /aw-<skill> reference resolves to a real skill",
          not unknown, f"dangling={unknown}; real={sorted(expected)}")

    stale = sorted(set(STALE_REFERENCE.findall(text)))
    check(f"{skill}: names no colon-form /aw: invocation",
          not stale, f"stale={stale}; the colon form is the label, not the command")

# Positive control: the stale-form rule must be able to see the string it
# forbids, in the exact shape every body carried until the rename.
check("positive control: the stale-reference rule refuses the colon form",
      bool(STALE_REFERENCE.search("hand it to `/aw:wi-epic-reconcile` next")))

# -- every script path a SKILL.md names must exist -------------------------
# This was two extractors while the bodies carried both a `${CLAUDE_PLUGIN_ROOT}`
# form and an in-repo fallback. There is one form now, and the second rule is
# the inverse: a body still naming the plugin variable names a path that
# resolves to the empty string at use, producing `/scripts/epic.py` -- an
# absolute path into the filesystem root, which fails as "no such file" and
# reads like a broken checkout.
named = {(skill, m) for skill, text in bodies.items() for m in IN_REPO_PATH.findall(text)}
check("the in-repo path extractor found paths at all", bool(named), f"found={len(named)}")
for skill, rel in sorted(named):
    check(f"{skill}: script path {rel} exists", (REPO / rel).is_file())

for skill, text in bodies.items():
    check(f"{skill}: names no ${{CLAUDE_PLUGIN_ROOT}}",
          not PLUGIN_ROOT_PATH.search(text),
          "the plugin is deleted; that variable expands to nothing")

# Positive control: the absence rule must be able to see the string it forbids.
check("positive control: the plugin-root rule refuses the variable",
      bool(PLUGIN_ROOT_PATH.search("uv run ${CLAUDE_PLUGIN_ROOT}/scripts/epic.py")))

# The scripts sit at the plugin root because all three skills run them. That is
# only true while no skill carries its own copy: a second copy is not a load
# error, it is a fork that works perfectly until the two readings of a schema
# drift apart. This was a reconcile-only assertion while reconcile was the only
# skill reaching across; every skill reaches across now.
check("the scripts sit in one shared directory", SCRIPTS.is_dir(), str(SCRIPTS))
for skill in SKILLS:
    check(f"{skill}: carries no scripts/ copy of its own",
          not (skill_dir(skill) / "scripts").exists())

# -- the skills still drive the script, never the aw CLI -------------------
for skill, text in bodies.items():
    check(f"{skill}: names no `aw <verb>` invocation", not AW_INVOCATION.search(text))
    check(f"{skill}: names no gh issue/pr write command", not GH_WRITE.search(text))

# -- interviewing versus procedural ----------------------------------------
# The partition has to be exhaustive and disjoint before either rule below can
# be read as covering anything. A skill absent from both lists would be a skill
# neither rule applies to, which is the failure mode this replaced.
check("every skill is classified exactly once",
      sorted(INTERVIEWING + PROCEDURAL) == sorted(SKILLS)
      and not (set(INTERVIEWING) & set(PROCEDURAL)),
      f"interviewing={sorted(INTERVIEWING)} procedural={sorted(PROCEDURAL)}")

# The label carries no ` -- `: that sequence is how the reporter separates an
# assertion from its detail, so a label containing it is truncated in every
# downstream reading, including the negative control's isolation comparison.
for skill in INTERVIEWING:
    if skill in bodies:
        check(f"{skill}: names AskUserQuestion, so it reaches the human",
              "AskUserQuestion" in bodies[skill])
for skill in PROCEDURAL:
    if skill in bodies:
        check(f"{skill}: names no AskUserQuestion, because a gate has nothing to ask",
              "AskUserQuestion" not in bodies[skill])

# Positive control: the transitive half of the pin derivation. Still run over
# a synthetic graph rather than the live scripts, because a control that reads
# its assertion off whatever happens to be live today measures the graph's
# current shape, not the closure logic that is supposed to survive the next
# script joining it.
#
# The live graph does carry a transitive member now, which is exactly the
# case that stayed unmeasured before this rewrite. `impl.py` imports
# `tomllib` directly -- it carries the same 3.11 version guard `e2e.py` and
# `meta.py` do -- so it lands in `NEEDS_PIN` on that import alone; its
# `sibling("e2e", ...)` call is a second, redundant edge into a member that
# is already direct, which is what `SIBLING_EDGE` exists to catch regardless
# of whether it is load-bearing this week. `wis.py`, new on 2026-08-27, is
# the case that *is* load-bearing: it imports no TOML itself and reaches
# `tomllib` only through a plain top-level `import e2e` -- the edge
# `IMPORT_EDGE` exists to catch -- so it lands in `NEEDS_PIN` purely because
# `e2e.py` imports `tomllib` directly and the fixed point below grows through
# that edge to find it. The live closure measured against the current
# scripts is `{e2e.py, impl.py, meta.py, wis.py}`. The synthetic graph
# exercises that same growth without depending on which script happens to
# carry which edge this week; "the live derivation is that same closure"
# below is the separate assertion that ties this proof back to what is
# actually live.
#
# The previous version asserted `"logic.py" in NEEDS_PIN`, and that is the
# failure mode being fixed rather than a style change: it measured the
# derivation only for as long as one particular script kept one particular
# shape, and when `logic.py` was merged away the control would have gone red
# about a script that no longer exists instead of about the closure. The
# closure is what the assertion below depends on, so the closure is what gets
# measured.
def _pin_closure(sources: dict, depends: dict) -> set:
    """The same fixed point `NEEDS_PIN` is built by, over an arbitrary graph."""
    needs = {n for n, text in sources.items()
             if re.search(r"^import tomllib\b", text, re.M)}
    while True:
        grown = {n for n in sources if n in needs or depends[n] & needs}
        if grown == needs:
            return needs
        needs = grown


_probe = _pin_closure({"a.py": "import tomllib\n", "b.py": "", "c.py": "",
                       "d.py": ""},
                      {"a.py": set(), "b.py": {"a.py"}, "c.py": {"b.py"},
                       "d.py": set()})
check("positive control: the pin population is transitive, not direct-only",
      _probe == {"a.py", "b.py", "c.py"}, f"closure={sorted(_probe)}")

# And the live derivation is that same fixed point, not a hand-maintained list
# that happens to agree with one. Asserted rather than assumed, because the two
# are written out separately above.
check("positive control: the live pin population is that same closure",
      set(NEEDS_PIN) == _pin_closure(SOURCES, DEPENDS),
      f"live={sorted(NEEDS_PIN)}; closure={sorted(_pin_closure(SOURCES, DEPENDS))}")

# -- scripts that cannot run under a bare `python3` ------------------------
# The pin is not a style preference. `tomllib` is 3.11+, `python3` is 3.9 here,
# and the failure without the pin is a ModuleNotFoundError traceback that reads
# like a broken script rather than a wrong interpreter.
check("the interpreter-pin population is not empty",
      bool(NEEDS_PIN), f"scripts importing tomllib={sorted(NEEDS_PIN)}")
PINNED_PREFIX = " ".join(PINNED_PYTHON)

# The invocation is now `<launcher> ".claude/aw/scripts/<name>"` -- the same
# shape with the plugin variable replaced by the in-repo path. The quotes stay
# required by this pattern on purpose: they are what bounds the prefix capture,
# so an unquoted invocation is not silently exempted, it simply produces no
# match -- which the emptiness assertion below then catches.
SCRIPTS_REL = SCRIPTS.relative_to(REPO).as_posix()
matched = 0
for name in sorted(NEEDS_PIN):
    pattern = re.compile(rf'(\S+(?: \S+)*?) "{re.escape(SCRIPTS_REL)}/{re.escape(name)}"')
    for skill, text in sorted(bodies.items()):
        for prefix in pattern.findall(text):
            matched += 1
            check(f"{skill}: `{name}` is invoked through the pinned interpreter",
                  prefix.endswith(PINNED_PREFIX), f"prefix={prefix!r}")

# Without this the loop above is vacuous the moment the invocation shape moves
# again: zero matches print zero rows, and a gate that asserted nothing reads
# identically to one that passed.
check("the pin detector matched at least one invocation", matched > 0, f"matched={matched}")

# Positive control: the same detector, applied to the shape it exists to refuse.
BARE = f'python3 "{SCRIPTS_REL}/{sorted(NEEDS_PIN)[0]}" check'
check("positive control: the pin detector refuses a bare python3 invocation",
      not re.search(
          rf'(\S+(?: \S+)*?) "{re.escape(SCRIPTS_REL)}/{re.escape(sorted(NEEDS_PIN)[0])}"',
          BARE).group(1).endswith(PINNED_PREFIX))

# The `aw <verb>` detector is only meaningful if it can fire. The control used
# to be a real file -- the crate's own `aw-goal` skill template -- read off
# disk, which is why it was accompanied by an existence assertion: a moved or
# deleted control would otherwise be indistinguishable from one that passed.
#
# That crate is gone, and with it the last body in this repository that drove
# the binary. So the control is now the literal below, verbatim from the
# `aw-goal` template's routing table as it stood at deletion. That is the same
# choice `GH_WRITE_CONTROL` makes for the same reason: pin the detector to the
# real defect, not to a caricature of it. What is lost with the file is small
# and worth naming -- the control no longer proves the detector fires on
# something *someone else wrote*, only on something this gate declares.
AW_INVOCATION_CONTROL = "| wi | `aw goal wi <id>` | lifecycle chain of that root |"
check("positive control: the `aw <verb>` detector fires on a real invocation",
      bool(AW_INVOCATION.search(AW_INVOCATION_CONTROL)), AW_INVOCATION_CONTROL)

check("positive control: the gh write detector fires on the block reconcile carried",
      bool(GH_WRITE.search(GH_WRITE_CONTROL)))

# -- verb coverage, resolved against the real scripts -----------------------
@functools.lru_cache(maxsize=None)
def resolves(script, verb: str) -> bool:
    # A script that needs the pin must be probed through a 3.11+ interpreter.
    # Probing it with `sys.executable` -- 3.9 here -- would make every verb fail
    # to resolve for the same reason, which reads as "the skills name verbs that
    # do not exist" rather than "the gate used the wrong interpreter".
    #
    # Cached because the declared-unused loop below asks the same question
    # twice, once for the verdict and once for the message, and each answer
    # costs a process.
    launcher = pinned_interpreter() if script in NEEDS_PIN.values() else [sys.executable]
    r = subprocess.run([*launcher, str(script), verb, "--help"],
                       capture_output=True, text=True, cwd=REPO)
    return r.returncode == 0


MENTIONED = {name: set(re.findall(rf"{re.escape(name)} ([a-z][a-z-]*)", joined))
             for name in sorted(REQUIRED)}

# Every probe this gate will make, run concurrently to warm the cache the checks
# below read from. Each one is a `--help` in its own process: nothing is written
# and nothing is shared, so the order they finish in cannot change an answer.
# Serially they were most of this gate's runtime, and this gate is re-run once
# per mutation by its negative control, so the cost was paid eleven times over.
with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
    list(pool.map(
        lambda pair: resolves(*pair),
        [(SCRIPT_PATHS[name], verb)
         for name in sorted(REQUIRED)
         for verb in sorted(MENTIONED[name] | set(UNUSED[name]) | {ABSENT})],
    ))

for name in sorted(REQUIRED):
    script = SCRIPT_PATHS[name]
    mentioned = MENTIONED[name]
    check(f"the skills name at least one `{name} ...` invocation", bool(mentioned))
    for verb in sorted(mentioned):
        check(f"`{name} {verb}` resolves on the script", resolves(script, verb))
    missing = sorted(REQUIRED[name] - mentioned)
    check(f"{name}: every required verb is named (missing {missing})", not missing)
    for verb, why in sorted(UNUSED[name].items()):
        check(f"{name}: `{verb}` is declared unused, and the declaration holds",
              resolves(script, verb) and verb not in mentioned,
              why if resolves(script, verb) else f"`{verb}` no longer exists")
    check(f"`{name} {ABSENT}` is rejected -- the verb probe is not vacuous",
          not resolves(script, ABSENT))

print("\n=> " + ("GREEN" if not fails else f"RED ({len(fails)} failure(s))"))
sys.exit(1 if fails else 0)
