"""Where the things under test live, resolved once for every gate here.

These gates used to sit in a session scratchpad under `/tmp` with the checkout
path hardcoded into each one. Both halves of that were wrong. `/tmp` does not
survive a reboot, and these files are the evidence for the plugin's claims, not
working residue. And a hardcoded path makes a gate unrunnable from any other
clone while *also* making a rename look like a caught defect: the script goes
red with "no such file", which is the same red it emits when the thing it
guards is genuinely broken.

Resolution follows the same rule `epic.py` itself uses -- walk up to the
*outermost* `aw.toml` -- so the gates and the script under test can never
disagree about which checkout is being measured.
"""
# Not stylistic. `_RESOLVED` below is annotated `list[str] | None`, and a
# module-level annotation is *evaluated* -- so without this the whole module
# dies at import on 3.10 and older, taking every gate that imports it with it.
# That is what happened here: the annotation was written while everything was
# being run through the pinned launcher, and it took a run under bare `python3`
# to see it. Deferring annotations costs nothing and removes the class.
from __future__ import annotations

import importlib.util
import pathlib
import subprocess
import sys


def repo_root() -> pathlib.Path:
    """The checkout that owns the outermost `aw.toml` above this file."""
    start = pathlib.Path(__file__).resolve().parent
    found = [c for c in (start, *start.parents) if (c / "aw.toml").is_file()]
    if not found:
        raise SystemExit(f"error: no `aw.toml` above {start}")
    return found[-1]


REPO = repo_root()

# `aw` was a Claude Code plugin at `plugins/aw/` until 2026-08-21. That tree is
# deleted -- the scripts moved here, the skills were already duplicated into
# `.claude/skills/`, and `plugin.json`, `marketplace.json` and the
# `enabledPlugins` entry went with it. So `PLUGIN_DIR`, `PLUGIN_JSON` and
# `MARKETPLACE` are gone from this module rather than repointed: there is no
# bundle to name, and a constant pointing at a path that cannot exist is the
# shape of gate that goes red about itself.
AW_DIR = REPO / ".claude/aw"
SKILLS_DIR = REPO / ".claude/skills"

# The skill directory names are the invocation names -- Claude Code composes the
# command from the directory and ignores the frontmatter `name:` for that
# purpose, measured with a probe whose two names disagreed. So each directory is
# literally `aw-<skill>` and is invoked as `/aw-<skill>`, while the frontmatter
# `name:` carries the label `aw:<skill>` that the skill list displays. Two
# prefixes, one namespace: the dash form is what a human types, the colon form
# is what they read. Naming the eleven here rather than globbing keeps a stray
# directory from silently joining the population under test.
NAMESPACE = "aw"
SKILL_PREFIX = f"{NAMESPACE}-"      # directory and invocation: aw-<skill>/ -> /aw-<skill>
DISPLAY_PREFIX = f"{NAMESPACE}:"    # frontmatter name: the listed label aw:<skill>
SKILLS = ("ask-user", "check-meta", "go-tdd-for-change", "go-tdd-for-epic",
          "grill-change-to-td", "grill-epic-to-changes", "grill-epic-to-td",
          "grill-me-to-change", "grill-me-to-epic", "grill-me-to-prd",
          "prepare-goal")


def skill_dir(skill: str) -> pathlib.Path:
    """The on-disk directory for one skill name in `SKILLS`."""
    return SKILLS_DIR / f"{SKILL_PREFIX}{skill}"


def skill_invocation(skill: str) -> str:
    """What a human types to run one skill: the directory name behind a slash."""
    return f"/{SKILL_PREFIX}{skill}"


def skill_label(skill: str) -> str:
    """What the skill list shows for one skill: its frontmatter `name:`."""
    return f"{DISPLAY_PREFIX}{skill}"

# The `ec -> td -> cb` ladder is gone from this plugin: three scripts, three
# gates, and the twelve `wi-{ec,td,cb}-*` wrappers, deleted rather than
# archived. An archive of instructions for scripts that no longer exist is not
# history, it is a set of commands that fail with "no such file" for a reader
# who cannot tell that from a broken checkout.
#
# What replaced them is `e2e -> unit -> logic`, driven by each verb's printed
# `next.command` rather than by a skill per step. Two skills front that ladder,
# one per work-item type: `go-tdd-for-change` runs the three phases on one
# change, and `go-tdd-for-epic` asks `epic.py order` for the children's
# sequence and runs the change skill on each. The semantic review that sat
# between the phases -- two skills that routed the contract and then the code
# to a second model for a verdict -- left the ladder on 2026-08-26, and the
# `review-prompt` and `verdict` verbs went with it.

# Two kinds of skill, and the difference decides which rules can apply.
#
# An INTERVIEWING skill turns an underspecified human intent into an artifact,
# so it must reach the human: `AskUserQuestion` appearing in its body is the
# check that it does.
#
# A PROCEDURAL skill runs a fixed sequence and reads an exit code. For those the
# same rule is *inverted* -- naming `AskUserQuestion` is the defect, because the
# only thing left to ask about at a gate is whether the gate counts. Exempting
# them instead would leave three skills that no per-skill rule can refuse.
#
# The two lists are asserted exhaustive and disjoint over SKILLS, so a new skill
# cannot join without someone deciding which kind it is.
#
# The two `go-tdd-*` skills are procedural despite the phases they drive being
# model work rather than command work -- so is `check-meta`. The line is not
# "does a model write something", it is whether the skill has anything left to
# ask. By the time the ladder starts, the work item has already said what the
# change is; what remains is a fixed sequence of verbs and the exit codes they
# return, and the only question a gate could raise is whether it counts.
# The two `grill-*-to-td` skills are interviewing for the same reason the
# grills that open work items are: the body says what the change is, and the
# document they write says how it is built -- which the body does not carry,
# and only the human can supply. `grill-me-to-prd` runs before any work item
# exists, so everything it writes is in the human's head. `ask-user` is
# interviewing by definition: asking is the whole of what it does, and a body
# without AskUserQuestion would be a skill that does nothing.
# `prepare-goal` is interviewing on the strength of its second route rather
# than its first. Given an iid it reads a body that a validator already refused
# once, and there is nothing left to ask; given none, everything the condition
# needs -- the end state, the command that shows it, the near miss it must not
# take, where it stops when the end state is unreachable -- exists only in the
# human's head. Classifying it procedural would forbid the very tool that route
# is made of, and would leave the no-iid case answered by whatever the agent
# guessed the human meant.
INTERVIEWING = ("ask-user", "grill-change-to-td", "grill-epic-to-changes",
                "grill-epic-to-td", "grill-me-to-change", "grill-me-to-epic",
                "grill-me-to-prd", "prepare-goal")
PROCEDURAL = ("check-meta", "go-tdd-for-change", "go-tdd-for-epic")

# The nine scripts sit in one directory, not inside a skill. They were under
# `wi-epic-grill/scripts/` (now `grill-me-to-epic`) while it was the only skill
# running them, which made
# the epic grill look like their owner; reconcile already reached across into
# it, and the change grill would have been a second skill reaching into a third
# one's directory. A shared dependency belongs beside the skills, not inside
# whichever one happened to need it first.
#
# They also cannot be split across the eleven skill directories, which is what
# the plugin deletion had to decide. `e2e.py`, `unit.py` and `logic.py` each
# load `leg.py` by `Path(__file__).parent / "leg.py"`, and `leg.change_module()`
# loads `change.py` the same way. One directory is a load-bearing requirement,
# not a tidiness preference.
SCRIPTS = AW_DIR / "scripts"
SCRIPT = SCRIPTS / "epic.py"
CHANGE_SCRIPT = SCRIPTS / "change.py"
LEG_SCRIPT = SCRIPTS / "leg.py"
ENGINE = SCRIPTS / "workitem.py"

# The three phases that replaced `ec -> td -> cb`. They were named here before
# they existed on disk, because the gate that drives them was written first and
# had to be able to go red for the right reason: "the script is missing" rather
# than "this module has no such attribute", which is a red about the gate.
E2E_SCRIPT = SCRIPTS / "e2e.py"
UNIT_SCRIPT = SCRIPTS / "unit.py"
LOGIC_SCRIPT = SCRIPTS / "logic.py"

# The META-doc validator, which is not on the ladder and owns no work item. It
# is named here for the same reason the three phases were: the gate goes red
# with "the script is missing" before the script exists, rather than with an
# AttributeError about this module.
#
# It validates and never writes. That is the whole of its difference from the
# `aw meta` it replaces, and the difference is why the name is not reused: that
# verb *spliced* generated content between `<!-- aw:meta:... -->` markers, and
# deleting it left 132 markers across 65 files still asserting a producer. A
# marker whose producer is gone is worse than plain prose, because a reader
# takes it as evidence that something regenerates what sits inside it.
META_SCRIPT = SCRIPTS / "meta.py"

# The PRD run's own refusal. It is not on the ladder either, and it is the
# only script here whose subject is a directory of prose rather than a work
# item: `/aw-grill-me-to-prd` writes `<project>/docs/product/`, and this
# refuses a run that reached outside it, then writes the one commit that run
# is allowed. Two verbs, `check` and `commit`, and the split is what keeps the
# read from being able to repair what it measures.
PRD_SCRIPT = SCRIPTS / "prd.py"

# The phase scripts read TOML, `tomllib` landed in 3.11, and `python3` is 3.9 on at least
# one machine this runs on. Both the skills and the gates below have to invoke it
# through a pinned interpreter -- and they have to agree on which, or a gate can
# pass against an interpreter no skill ever uses.
PINNED_PYTHON = ("uv", "run", "--python", "3.13", "--no-project")

_RESOLVED: list[str] | None = None


def pinned_interpreter() -> list[str]:
    """The same 3.11+ interpreter `PINNED_PYTHON` reaches, resolved once.

    `uv run --python 3.13 --no-project` re-resolves its environment on every
    call: measured at 2.6s a shot against 0.11s for the interpreter it resolves
    to. The gates below spawn the scripts under test dozens of times -- once per
    verb probe, once per case, once per mutation -- and paying environment
    resolution for each was two thirds of this suite's runtime.

    This is not a hole in the interpreter pin, because two different claims are
    being checked in two places. That the *skills* invoke the scripts through
    the pinned launcher is a text assertion over every SKILL.md, with its own
    `unpinned` negative control; nothing about it involves running anything.
    What a gate needs when it actually spawns a script is merely *an*
    interpreter new enough to have `tomllib`, so that what it measures is the
    script's behaviour rather than its startup guard.

    Resolution is lazy and cached: every gate imports this module, and only some
    of them spawn anything.
    """
    global _RESOLVED
    if _RESOLVED is None:
        found = subprocess.run(["uv", "python", "find", "3.13"],
                               capture_output=True, text=True)
        _RESOLVED = ([found.stdout.strip()]
                     if found.returncode == 0 and found.stdout.strip()
                     else list(PINNED_PYTHON))
    return list(_RESOLVED)

HERE = pathlib.Path(__file__).resolve().parent
SNAPSHOTS = HERE / "_snapshots"

TRACKER_REPO = "chrischeng-c4/axiom"


def load_script_module(path, name):
    """Import a facade script as a module.

    Registered in `sys.modules` before execution because its `@dataclass`
    definitions resolve their annotations through it.
    """
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def load_epic_module():
    return load_script_module(SCRIPT, "epicmod")


def load_change_module():
    return load_script_module(CHANGE_SCRIPT, "changemod")


def hard_errors(module, body: str) -> list[str]:
    """Validation errors that refuse the body, dropping advisory notes."""
    return [e for e in module.validate_body(body) if not e.startswith("note:")]
