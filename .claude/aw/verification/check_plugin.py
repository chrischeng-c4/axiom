#!/usr/bin/env python3
"""Re-runnable checker for the eight `aw:*` skills and the scripts they drive.

The name is historical: `aw` was a Claude Code plugin at `plugins/aw/` until
2026-08-21, and this file checked its bundle. The plugin is deleted -- the
scripts are at `.claude/aw/scripts/`, the eight skills load as project skills
out of `.claude/skills/aw:*/`, and the manifest assertions here were inverted
into absence assertions so a half-restored plugin goes red instead of quietly
loading a second copy of all eight.

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

The invocation axis has since moved twice more, ending at `aw:wi-epic-*`, which
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
from _paths import (CHANGE_SCRIPT, INTERVIEWING,  # noqa: E402
                    META_SCRIPT, PINNED_PYTHON,
                    E2E_SCRIPT, LOGIC_SCRIPT, PROCEDURAL, REPO,
                    SCRIPT, SCRIPTS, SKILLS, SKILLS_DIR, SKILL_PREFIX,
                    UNIT_SCRIPT, pinned_interpreter, skill_dir)

# The namespace the invocation keeps. It was the plugin name until 2026-08-21;
# it is now the literal `aw:` prefix on each skill directory, which is the only
# thing holding the namespace up now that no plugin supplies one.
PLUGIN_NAME = "aw"

# Each bundled script, and the verbs the skills must name for it. The two
# differ on purpose: an epic owns children and can be closed against them, a
# change has neither, so a change facade exposing `children` would mean the
# engine had leaked the epic's shape into the wrong type.
REQUIRED = {
    "epic.py": {"skeleton", "create", "update", "validate", "close", "show",
                "children", "order", "reconcile", "bodydir", "fetch"},
    "change.py": {"skeleton", "create", "update", "validate", "show",
                  "bodydir", "fetch"},
    # The three phases of the ladder, each with the same four verbs. Unlike the
    # deleted `wi-{ec,td,cb}-*` wrappers there is one skill for all twelve
    # rather than one skill per verb -- the sequence is fixed and each verb
    # prints the next, so what a skill adds is the entry point and the refusal
    # list, not a wrapper per step. All four are required of each: a phase
    # whose `start` stopped being named would be a phase an agent could enter
    # without the tree being checked clean first.
    #
    # The two reviewed phases carry two more. `review-prompt` and `verdict` are
    # required of exactly the phases a reviewer skill exists for, so a rename
    # that leaves a reviewer naming a verb its script no longer has goes red --
    # and so does a phase that grows a reviewer without anything driving it.
    "e2e.py": {"start", "verify", "test", "commit", "review-prompt", "verdict"},
    "unit.py": {"start", "verify", "test", "commit"},
    "logic.py": {"start", "verify", "test", "commit", "review-prompt", "verdict"},
    # The META-doc validator. One verb, and the singleton is the point: a
    # second verb here would be a verb that writes, and the thing this replaces
    # was deleted for writing.
    "meta.py": {"check"},
}

# Which phase each reviewer skill drives, and -- for `unit.py` -- that it drives
# none. This is the whole of the routing: it used to be a `[review]` key in each
# project's `aw.toml`, read at runtime, which meant "the configured reviewer is
# a skill that exists" was a claim nothing checked until a reviewer was invoked
# against a project someone had misconfigured. Naming it here instead makes it a
# constant in the script, resolvable without running anything, and the pairing
# below is what refuses a constant pointing at a skill that is not in the
# bundle.
REVIEWED = {"e2e.py": "codex-e2e-review", "logic.py": "codex-code-review"}

# Why each unreviewed script is unreviewed, keyed by script so the reason is
# the one that applies. It was a single string while `unit.py` was the only
# entry whose absence was interesting, and it printed that unit-phase
# reasoning under `epic.py` and `change.py` too. A default of "" is fine for a
# script whose lack of a reviewer needs no argument; what is not fine is
# printing an argument about a different script.
UNREVIEWED_WHY = {
    "unit.py": ("the unit phase's tests are landed red and never read on "
                "their own: at `unit` only half the pair exists, and the "
                "question a reviewer would be asked -- do these tests admit "
                "an implementation that misses the requirement -- has no "
                "implementation to ask it about yet. It is asked at `logic`, "
                "over both halves at once."),
    "meta.py": ("nothing here is a judgement. Every rule resolves against the "
                "filesystem -- a marker's producer exists or it does not, a "
                "link's target exists or it does not -- so there is no "
                "question to route to a second model."),
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
    "unit.py": {},
    "logic.py": {},
    "meta.py": {},
}
SCRIPT_PATHS = {"epic.py": SCRIPT, "change.py": CHANGE_SCRIPT,
                "e2e.py": E2E_SCRIPT, "unit.py": UNIT_SCRIPT,
                "logic.py": LOGIC_SCRIPT, "meta.py": META_SCRIPT}

# Scripts that cannot run under a bare `python3`, and the pin the skills must
# carry. Derived from the source rather than listed by hand: a script that grows
# a `tomllib` import joins this set on its own, and one that loses it leaves.
#
# Transitively, and that is not tidiness. `logic.py` imports no TOML itself and
# was therefore exempt from the assertion below, while dying under 3.9 anyway --
# it loads `unit.py` through `leg.sibling`, and the import happens there. A
# direct-import-only derivation exempts exactly the scripts whose dependence is
# hardest to see by reading them, which is the opposite of what a derivation is
# for. The edge is the `sibling("<name>", ...)` call, closed to a fixed point.
SIBLING_EDGE = re.compile(r'sibling\(\s*"([a-z0-9_]+)"')
# Tolerant of a missing file on purpose. A script is named in `SCRIPT_PATHS`
# before it is written, so that the gate driving it can be watched going red
# for the right reason first. Reading eagerly here would make that red a
# `FileNotFoundError` traceback at import -- which takes every *other*
# assertion in this file down with it, and reads as a broken gate rather than
# as an absent script. The absence is asserted explicitly below instead.
SOURCES = {name: (path.read_text(encoding="utf-8") if path.is_file() else "")
           for name, path in SCRIPT_PATHS.items()}
DEPENDS = {name: {f"{s}.py" for s in SIBLING_EDGE.findall(text)} & set(SCRIPT_PATHS)
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
SKILL_REFERENCE = re.compile(r"/(aw[\w-]*:[\w:-]+)")

fails = []


def check(label, ok, detail=""):
    print(f"{'PASS' if ok else 'FAIL'} {label}{(' -- ' + detail) if detail else ''}")
    if not ok:
        fails.append(label)


# -- the plugin is gone ----------------------------------------------------
# Deleted on 2026-08-21. These four rows used to assert a `marketplace.json`,
# a `plugin.json`, a kebab-case plugin name and a relative in-repo source that
# resolved to the bundle. They are now the inverse assertion: those files must
# NOT be back, because a half-restored plugin would load a second copy of eight
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

# -- the eight skills and the eight scripts --------------------------------
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
# The directory name IS the skill name, and registration rewrites characters it
# will not carry: a directory `probe:colon` registers as `probe-colon`. That is
# strictly worse than a rejection, because the plugin loads and every body
# reference then points at a name nobody wrote. So the assertion is not "legal"
# but "survives unchanged" -- anything outside this class registers as
# something else.
SAFE_DIRECTORY = re.compile(r"[a-z0-9]+(?:-[a-z0-9]+)*")

# `.claude/skills/` is shared -- `agy-dispatch` lives there too -- so the
# population is the `aw:`-prefixed directories, not the whole listing. The
# prefix is stripped before comparing, which is also what proves it is present:
# a directory that lost it simply is not in `on_disk`, and the equality fails.
on_disk = sorted(q.name[len(SKILL_PREFIX):] for q in SKILLS_DIR.iterdir()
                 if q.is_dir() and q.name.startswith(SKILL_PREFIX))
check("the aw: skills on disk are exactly the ones under test",
      on_disk == sorted(SKILLS), f"on disk={on_disk}")
for skill in SKILLS:
    check(f"{skill}: directory name survives registration unchanged",
          bool(SAFE_DIRECTORY.fullmatch(skill)),
          "registration rewrites anything outside [a-z0-9-]")

# Positive control: the assertion must be able to refuse the shape that was
# actually shipped and silently renamed.
check("positive control: the directory-name rule refuses a colon",
      not SAFE_DIRECTORY.fullmatch("wi:epic:grill"))

expected = {f"{PLUGIN_NAME}:{s}" for s in SKILLS}
for skill, text in bodies.items():
    qualified = f"{PLUGIN_NAME}:{skill}"

    # The frontmatter name does not decide anything, but a frontmatter name
    # that disagrees with the directory is a lie left where the next reader
    # will believe it.
    front = re.search(r"^name:\s*(\S+)\s*$", text, re.M)
    check(f"{skill}: frontmatter declares a name", bool(front))
    if front:
        check(f"{skill}: frontmatter name matches the directory",
              front.group(1) == skill, f"frontmatter={front.group(1)!r} directory={skill!r}")

    check(f"{skill}: the body's H1 is the real invocation",
          re.search(rf"^#\s*/{re.escape(qualified)}\s*$", text, re.M) is not None)

    referenced = set(SKILL_REFERENCE.findall(text))
    check(f"{skill}: the skill-reference extractor found references at all", bool(referenced))
    unknown = sorted(referenced - expected)
    check(f"{skill}: every /plugin:skill reference resolves to a real skill",
          not unknown, f"dangling={unknown}; real={sorted(expected)}")

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

# -- the reviewer a phase names is a skill that exists ----------------------
# The routing, end to end. Each reviewed phase holds its reviewer as a module
# constant; the assertion is that the constant resolves to a bundled skill, and
# that the pairing is the one declared above rather than merely *some* skill.
# The second half matters because both reviewers are real names: a `logic.py`
# pointing at the contract reviewer would pass a mere existence check while
# handing the implementation to a rubric that never mentions it.
REVIEWER_CONST = re.compile(r'^REVIEWER\s*=\s*"([^"]*)"\s*$', re.M)
for name in sorted(SCRIPT_PATHS):
    found = REVIEWER_CONST.findall(SOURCES[name])
    if name not in REVIEWED:
        check(f"{name}: declares no reviewer, and the declaration holds",
              not found,
              UNREVIEWED_WHY.get(name, "") if not found else f"found {found}")
        continue
    check(f"{name}: names exactly one reviewer", len(found) == 1, f"found={found}")
    if len(found) == 1:
        check(f"{name}: its reviewer resolves to a bundled skill",
              found[0].lstrip("/") in expected,
              f"reviewer={found[0]!r}; bundled={sorted(expected)}")
        check(f"{name}: its reviewer is the one declared for this phase",
              found[0] == f"/{PLUGIN_NAME}:{REVIEWED[name]}",
              f"reviewer={found[0]!r} declared={REVIEWED[name]!r}")

# And the other direction: a reviewer skill nobody routes to is a skill an
# agent can invoke against a phase that will not read its verdict.
routed = {f"{PLUGIN_NAME}:{s}" for s in REVIEWED.values()}
for skill in sorted(s for s in SKILLS if s.startswith("codex-")):
    check(f"{skill}: some phase routes to it",
          f"{PLUGIN_NAME}:{skill}" in routed, f"routed={sorted(routed)}")

# Positive control: the transitive half of the pin derivation. `logic.py`
# imports no TOML of its own -- it is in the set only through `unit.py` -- so a
# direct-import-only derivation would exempt it, and the assertion further down
# would stop covering the one script whose dependence is invisible in its own
# source.
check("positive control: the pin population is transitive, not direct-only",
      "logic.py" in NEEDS_PIN and not re.search(r"^import tomllib\b", SOURCES["logic.py"], re.M),
      f"needs the pin={sorted(NEEDS_PIN)}; via={sorted(DEPENDS['logic.py'])}")

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
