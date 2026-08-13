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
PLUGIN_DIR = REPO / "plugins/aw"
PLUGIN_JSON = PLUGIN_DIR / ".claude-plugin/plugin.json"
MARKETPLACE = REPO / ".claude-plugin/marketplace.json"

# The skill directory names are the invocation names -- Claude Code composes
# `plugin:skill` from the plugin name and the directory name, and ignores the
# frontmatter `name:` entirely. Naming them here rather than globbing keeps a
# stray directory from silently joining the population under test.
SKILLS = ("codex-review", "wi-change-grill", "wi-ec-commit", "wi-ec-review",
          "wi-ec-start", "wi-ec-verify", "wi-epic-grill", "wi-epic-reconcile")

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
INTERVIEWING = ("wi-change-grill", "wi-epic-grill", "wi-epic-reconcile")
PROCEDURAL = ("codex-review", "wi-ec-commit", "wi-ec-review", "wi-ec-start",
              "wi-ec-verify")

# The scripts sit at the plugin root, not inside a skill. They were under
# `wi-epic-grill/scripts/` while it was the only skill running them, which made
# the epic grill look like their owner; reconcile already reached across into
# it, and the change grill would have been a second skill reaching into a third
# one's directory. A shared dependency belongs beside the skills, not inside
# whichever one happened to need it first.
SCRIPTS = PLUGIN_DIR / "scripts"
SCRIPT = SCRIPTS / "epic.py"
CHANGE_SCRIPT = SCRIPTS / "change.py"
EC_SCRIPT = SCRIPTS / "ec.py"
ENGINE = SCRIPTS / "workitem.py"

# `ec.py` reads TOML, `tomllib` landed in 3.11, and `python3` is 3.9 on at least
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

# The crate that owns the change work-item schema. `change.py` is a port of it,
# not a second authority, so the gates read these two files as the oracle.
CRATE = REPO / "apps/agentic-workflow/src"
GHAN_RS = CRATE / "issues/ghan.rs"
ISSUES_RS = CRATE / "cli/issues.rs"

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
