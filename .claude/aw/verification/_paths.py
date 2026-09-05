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
    """The checkout that owns this file, identified by its `aw.toml`.

    Outermost marker, but never across a checkout boundary. This is a
    hand-maintained twin of `workitem.outermost_aw_toml()`, which carries the
    full reasoning; it is copied rather than imported because importing that
    module means first knowing where the scripts are, which is what this
    function answers. Nothing detects drift between the two.

    The boundary is the half that was missing. A git worktree may sit inside
    another checkout -- Claude Code puts its own under
    `.claude/worktrees/<name>/` -- and both trees carry a root `aw.toml`, so
    an unbounded walk from `__file__` resolved every gate in an agent worktree
    against the *enclosing* checkout: the gates read another tree's files and
    another session's dirty set.
    """
    start = pathlib.Path(__file__).resolve().parent
    chain = [start, *start.parents]
    proc = subprocess.run(
        ("git", "-c", "core.fsmonitor=false", "rev-parse", "--show-toplevel"),
        capture_output=True, text=True, cwd=start,
    )
    if proc.returncode == 0 and proc.stdout.strip():
        boundary = pathlib.Path(proc.stdout.strip()).resolve()
        if boundary in chain:
            chain = chain[: chain.index(boundary) + 1]
    found = [c for c in chain if (c / "aw.toml").is_file()]
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
# command from the directory -- and the frontmatter `name:` is held equal to
# the directory name by check_plugin.py, so `aw-<skill>/` is invoked as
# `/aw-<skill>` and listed as `aw-<skill>`. One prefix, one namespace. Naming
# the exact set here rather than globbing keeps a stray directory from silently
# joining the population under test.
NAMESPACE = "aw"
SKILL_PREFIX = f"{NAMESPACE}-"      # directory, invocation, and frontmatter name: aw-<skill>
SKILLS = ("ask-user", "e2e-for", "grill-release", "impl-for",
          "prepare-goal", "review", "test-for")


def skill_dir(skill: str) -> pathlib.Path:
    """The on-disk directory for one skill name in `SKILLS`."""
    return SKILLS_DIR / f"{SKILL_PREFIX}{skill}"


def skill_invocation(skill: str) -> str:
    """What a human types to run one skill: the directory name behind a slash."""
    return f"/{SKILL_PREFIX}{skill}"

# The `ec -> td -> cb` ladder is gone from this plugin: three scripts, three
# gates, and the twelve `wi-{ec,td,cb}-*` wrappers, deleted rather than
# archived. An archive of instructions for scripts that no longer exist is not
# history, it is a set of commands that fail with "no such file" for a reader
# who cannot tell that from a broken checkout.
#
# What replaced them is `e2e -> impl`, driven by each verb's printed
# `next.command` rather than by a skill per step. Two skills front that
# ladder now, one per PHASE rather than one per work-item type: `e2e-for`
# runs the e2e phase's four verbs on each Milestone's queue head, and
# `impl-for` runs impl's five for a behavior head or maint's verbs for a
# maintenance head, closing each issue to advance the queue. Both take one
# scope reference -- an issue, a Milestone, or a project's open Milestones.
# The standalone `maint-for-wi` skill folded into `impl-for` on 2026-09-02.
# The semantic review that used to sit between the phases -- two skills that
# routed the contract and then the code to a second model for a verdict --
# left the ladder on 2026-08-26, and the `review-prompt` and `verdict` verbs
# went with it. The read-only closers arrived on 2026-09-02: `test-for`
# verifies a scope's lifecycle evidence and reruns the full project gates,
# and `review` audits one project. A `build` skill sat beside them for part
# of that day and left the same way: a kind or GKE run is not lifecycle
# work, so it became the standalone `build-debug` and `build-release`
# (`scripts/build/`), outside this namespace and this checker.
#
# The technical-design step is gone the same way. `grill-change-to-td` and
# `grill-epic-to-td` wrote `docs/technical/<subsystem>.md` sections and ADRs
# beside them; both skills and the whole `docs/technical/` tree are deleted,
# and a design decision now lives in the `//!` or `///` block of the module
# or type that owns it (`CLAUDE.md`, "Authoring").
# The product and release interviews now have one entry: `aw-grill-release`.
# Its read-only plan operation reads META documents, tracker state, and
# `wis.py gap`, then returns one closed plan and digest without writes. Its
# Default-mode Apply
# phase sends that exact approved plan to `release_plan.py`. The facade runs
# `metadoc.py`, `meta.py`, `milestone.py`, `change.py`, and `wis.py` through a
# durable receipt. This keeps interview decisions in the skill and retryable
# writes in the engine.
#
# The META allowlist remains four roots: `README.md`, `STATUS.md`,
# `ROADMAP.md`, and `docs/**`. See `METADOC_SCRIPT` below for its enforcement.

# Two kinds of skill, and the difference decides which rules can apply.
#
# An INTERVIEWING skill turns an underspecified human intent into an artifact,
# so it must reach the human: `AskUserQuestion` appearing in its body is the
# check that it does.
#
# A PROCEDURAL skill runs a fixed sequence and reads an exit code. For those the
# same rule is *inverted* -- naming `AskUserQuestion` is the defect, because the
# only thing left to ask about at a gate is whether the gate counts. Exempting
# them instead would leave the two ladder skills that no per-skill rule can
# refuse.
#
# The two lists are asserted exhaustive and disjoint over SKILLS, so a new skill
# cannot join without someone deciding which kind it is.
#
# `e2e-for` and `impl-for` are procedural despite the phases they drive
# being model work rather than command work. The line is not "does a model
# write something", it is whether the skill has anything left to ask. By the
# time either phase starts, the work item has already said what the change
# is; what remains is a fixed sequence of verbs and the exit codes they
# return, and the only question a gate could raise is whether it counts.
# `test-for` and `review` are procedural for the plainer reason: each runs
# declared commands read-only and reports exit codes, with nothing
# underspecified to resolve.
#
# `aw-grill-release` is interviewing because it
# runs before any work item exists, so everything it writes -- across all
# four paths in its allowlist now, not the single `docs/product/` path it
# used to be -- is in the human's head, including how to resolve whatever
# `meta.py check` surfaces in the landing sequence's second step.
# `aw-grill-release` is interviewing
# for a parallel reason: `wis.py gap` prints what is missing, not what to do
# about it -- only the human can say which version a release takes, or
# whether a gap closes by opening a change, merging two issues, or closing
# one as no longer wanted. `ask-user` is interviewing by definition: asking
# is the whole of what it does, and a body without AskUserQuestion would be a
# skill that does nothing.
# `prepare-goal` is interviewing on the strength of its second route rather
# than its first. Given an iid it reads a body that a validator already refused
# once, and there is nothing left to ask; given none, everything the condition
# needs -- the end state, the command that shows it, the near miss it must not
# take, where it stops when the end state is unreachable -- exists only in the
# human's head. Classifying it procedural would forbid the very tool that route
# is made of, and would leave the no-iid case answered by whatever the agent
# guessed the human meant.
INTERVIEWING = ("ask-user", "grill-release", "prepare-goal")
PROCEDURAL = ("e2e-for", "impl-for", "review", "test-for")

# The fourteen scripts sit in one directory, not inside a skill. They were under
# `wi-epic-grill/scripts/` (then `grill-me-to-epic`, folded into
# `grill-meta-to-wis`, since split in two) while it was the only skill
# running them, which made
# the epic grill look like their owner; reconcile already reached across into
# it, and the change grill would have been a second skill reaching into a third
# one's directory. A shared dependency belongs beside the skills, not inside
# whichever one happened to need it first.
#
# They also cannot be split across the seven skill directories, which is what
# the plugin deletion had to decide. `e2e.py` and `impl.py` each load `leg.py`
# by `Path(__file__).parent / "leg.py"`, `impl.py` loads `e2e.py` the same way,
# and `leg.change_module()` loads `change.py` the same way. One directory is a
# load-bearing requirement,
# not a tidiness preference.
#
# That directory moved on 2026-09-02 from `.claude/aw/scripts/` into the
# `apps/aw` uv project, where each script is also a typer subcommand group of
# the `aw` entry point (`uv run --project apps/aw aw <group> ...`). The
# `__file__`-relative loading above moved with them unchanged, and every
# script still runs standalone by path -- which is how the gates here spawn
# them.
SCRIPTS = REPO / "apps/aw/src/aw/scripts"
SCRIPT = SCRIPTS / "epic.py"
CHANGE_SCRIPT = SCRIPTS / "change.py"
LEG_SCRIPT = SCRIPTS / "leg.py"
ENGINE = SCRIPTS / "workitem.py"
WI_TYPES_SCRIPT = SCRIPTS / "wi_types.py"

# The two phases that replaced `ec -> td -> cb`. They were named here before
# they existed on disk, because the gate that drives them was written first and
# had to be able to go red for the right reason: "the script is missing" rather
# than "this module has no such attribute", which is a red about the gate.
#
# There were three scripts behind this pair until 2026-08-27, when the two
# that used to split "test skeleton" from "implementation" were merged into
# the one `impl.py` now named below: in Rust a colocated test and the code
# under it are the same tree and are edited together, so the filename
# boundary between them cost an honest TDD loop more than it bought. What it
# did buy -- a named red measured before anything could satisfy it -- moved
# onto `impl.py`'s `red` verb, which records the failing names mid-phase
# instead of on a commit. One constant rather than two aliases: two names for
# one script would let a gate go on claiming to cover a phase that no longer
# exists.
E2E_SCRIPT = SCRIPTS / "e2e.py"
IMPL_SCRIPT = SCRIPTS / "impl.py"
MAINT_SCRIPT = SCRIPTS / "maint.py"

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

# The META-doc run's own refusal. It is not on the ladder either, and it is the
# only script here whose subject is prose rather than a work item:
# `aw-grill-release apply` writes `<project>/README.md`, `STATUS.md`,
# `ROADMAP.md` and `docs/**`, and this refuses a run that reached outside those
# four, then writes the one commit that run is allowed. Two verbs, `check` and
# `commit`, and the split is what keeps the read from being able to repair what
# it measures.
#
# It was `prd.py` until 2026-08-27, when the allowlist widened from
# `docs/product/` alone to all four. The file was renamed rather than joined by
# a second one: a `PRD_SCRIPT` still pointing at a narrower scope would be a
# constant that goes on describing the old boundary while nothing enforces it.
METADOC_SCRIPT = SCRIPTS / "metadoc.py"

# The read-only work-item/promise gap reader, and also not on the ladder: it
# owns no work item and writes nothing. `aw-grill-release plan` runs its one verb,
# `gap <project>`, for the
# seven G1..G7 rows before reorganising their half of the work-item set
# through `milestone.py` / `change.py` -- every write those skills make goes
# through those two, never through this one.
WIS_SCRIPT = SCRIPTS / "wis.py"

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


# The one spelling of the CLI a printed `next.command:` or a SKILL.md launcher
# line starts with, read from the engine's own registry rather than restated --
# a gate asserting a copy of this string would go on passing after the engine
# changed its spelling.
AW_CLI = load_script_module(WI_TYPES_SCRIPT, "_paths_wi_types").AW_CLI


def load_epic_module():
    return load_script_module(SCRIPT, "epicmod")


def load_change_module():
    return load_script_module(CHANGE_SCRIPT, "changemod")


def hard_errors(module, body: str) -> list[str]:
    """Validation errors that refuse the body, dropping advisory notes."""
    return [e for e in module.validate_body(body) if not e.startswith("note:")]
