#!/usr/bin/env python3
"""Every `next.command:` a phase prints must be accepted by the parser it names.

The ladder is driven by its own output. Each verb ends by printing the command
that follows it, and an agent reading that line runs it verbatim -- so a printed
command the receiving parser rejects is a dead end at the exact point where the
work is supposed to continue, and it is invisible to every other gate here.

That is not hypothetical. The defect this gate was written to refuse was live
in a tree whose eighteen other gates were all green:

  * `e2e.py`, `unit.py` and `logic.py` each end their `commit` verb by printing
    `change.py lifecycle <wi> --leg <PHASE> ...`, and `change.py` took its
    `--leg` choices from `workitem.LEGS`, which still read `("ec", "td", "cb")`.
    All three phases printed a command that exits 2. The one step that records a
    landed commit on the work item could not be reached from any phase.

  (A second one -- `leg.py` printing `<phase>.py commit <wi>` without the
  required `--project` after an accepted review -- left with the review itself
  on 2026-08-26. The shape is still live in every phase script, and the
  negative control keeps a mutation of it.)

Nothing else could have caught it. The strings are built in one module
and parsed in another, so a reader of either half sees something consistent; the
flow gates drive the verbs by constructing argv themselves, which measures the
parser but never the printed line. This gate is the only place the two meet.

It resolves each printed line using the emitting module's *own* constants --
`PHASE` is read from the module, not restated here -- so a phase renamed in one
place and not the other goes red rather than passing against a copy.

A known gap, measured rather than guessed at, and left open rather than
half-closed: `head = line.split()[0]` and `SCRIPT_TOKEN` only match a bare
`<name>.py` token, so a command printed with the pinned-launcher prefix
(`uv run --python 3.13 --no-project "<path>"`) never reaches the `is_file`
check at all -- it falls into the `prose` bucket, uncompared. `metadoc.py`
and `wis.py` print exactly that form on purpose: both reach `tomllib`
through `leg.py`, and the interpreter a bare script name resolves to is 3.9
on at least one machine here, where the failure is a `ModuleNotFoundError`
that reads like a broken script rather than a wrong interpreter.

Measured over this checkout: exactly two `next.command:` print sites are
affected -- `metadoc.py`'s `cmd_check` and `wis.py`'s `cmd_gap`, both on
their clean-run path (`=> CLEAN` / `=> ALIGNED`). Stripping a recognised
launcher prefix at the print site would not repair either one, and that is
why this gate does not attempt it: both scripts build the pinned-launcher
string in the verb function (`cmd_check`, `cmd_gap`) and pass it as a plain
`str` parameter into a shared `report(..., next_command)` helper, which is
where the marker actually lives. This renderer resolves one print site at a
time from the emitting module's globals plus an `Args()` stand-in for
`args.*` -- it does not trace a value back through a function call -- so the
free name `next_command` renders to the generic placeholder, not to text a
prefix could be stripped from. A prefix-matching pass bolted onto this
renderer would silently assert nothing for these two lines while still
reading as "checked" in the command count.

Closing this gap for real needs the renderer to follow a single-call
parameter back to its one call site in the same module, evaluate the
argument expression there, *then* strip a prefix matched against
`metadoc.PINNED` before the `SCRIPT_TOKEN` check -- a second kind of lookup
this file does not yet do. That is out of scope for this pass; the two
lines are named here so the gap is a fact in the docstring rather than a
number someone has to re-derive.
"""
from __future__ import annotations

import argparse
import ast
import contextlib
import io
import pathlib
import re
import sys

import _paths

SCRIPTS = _paths.SCRIPTS
sys.path.insert(0, str(SCRIPTS))

# Substituted into a printed command in place of a runtime value. `wi` has to
# survive `type=int`, and the two digests have to look like what they are; the
# rest only has to be one shell word, because what is under test is the fixed
# text around them -- the verb, the flag names, and the values the emitter
# hardcodes.
PLACEHOLDERS = {"wi": "4242", "project": "demo", "iid": "4242",
                "sha": "0" * 40, "digest": "f" * 64}
GENERIC = "PLACEHOLDER"

MARKER = "next.command:"
SCRIPT_TOKEN = re.compile(r"^[a-z0-9_]+\.py$")


class Args:
    """Stands in for the parsed `args` a print site reads at runtime."""

    def __getattr__(self, name: str) -> str:
        return PLACEHOLDERS.get(name, GENERIC)


def emitting_scripts() -> list[pathlib.Path]:
    found = sorted(p for p in SCRIPTS.glob("*.py")
                   if MARKER in p.read_text(encoding="utf-8"))
    if not found:
        raise SystemExit(f"error: no script under {SCRIPTS} prints `{MARKER}`; "
                         f"this gate would pass having read nothing")
    return found


def print_sites(path: pathlib.Path) -> list[tuple[int, ast.expr]]:
    """Every `print(...)` argument whose literal text carries the marker."""
    tree = ast.parse(path.read_text(encoding="utf-8"))
    sites = []
    for node in ast.walk(tree):
        if not (isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
                and node.func.id == "print" and node.args):
            continue
        for arg in node.args:
            literal = "".join(
                v.value for v in ast.walk(arg)
                if isinstance(v, ast.Constant) and isinstance(v.value, str))
            if MARKER in literal:
                sites.append((node.lineno, arg))
    return sites


def free_names(expr: ast.expr, known: dict) -> set[str]:
    return {n.id for n in ast.walk(expr)
            if isinstance(n, ast.Name) and n.id not in known
            and n.id not in dir(__builtins__)}


def render(expr: ast.expr, module) -> str:
    """The string this print site emits, using the module's own constants."""
    namespace = dict(vars(module))
    namespace["args"] = Args()
    for name in free_names(expr, namespace):
        namespace[name] = PLACEHOLDERS.get(name, GENERIC)
    return eval(ast.unparse(expr), namespace)  # noqa: S307 -- repo-controlled source


def payload(text: str) -> str:
    return text.split(MARKER, 1)[1].strip()


def accepts(target: pathlib.Path, argv: list[str]) -> str | None:
    """`None` if the target's parser accepts this argv, else why it refused.

    Nothing is executed. A script exposing `build_parser` is parsed directly;
    for the phase scripts, whose parser is local to `main`, every `cmd_*` global
    is replaced by a stub first -- `set_defaults(func=cmd_x)` resolves the global
    when `main` builds the parser, so the dispatch lands on the stub and the
    verb's real body never runs.
    """
    module = _paths.load_script_module(target, f"probe_{target.stem}")
    err = io.StringIO()
    try:
        with contextlib.redirect_stderr(err), contextlib.redirect_stdout(io.StringIO()):
            if hasattr(module, "build_parser"):
                module.build_parser().parse_args(argv)
            else:
                for name in [n for n in vars(module) if n.startswith("cmd_")]:
                    setattr(module, name, lambda *a, **k: 0)
                module.main(argv)
    except SystemExit as exit_:
        if exit_.code:
            reason = next((ln for ln in err.getvalue().splitlines()
                           if "error:" in ln), f"exit {exit_.code}")
            return reason.strip()
    except Exception as exc:  # the parser itself broke, which is also a red
        return f"{type(exc).__name__}: {exc}"
    return None


def main() -> int:
    failures, commands, prose = [], 0, 0
    for path in emitting_scripts():
        module = _paths.load_script_module(path, f"emit_{path.stem}")
        for lineno, expr in print_sites(path):
            line = payload(render(expr, module))
            head = line.split()[0] if line.split() else ""
            where = f"{path.name}:{lineno}"
            if not SCRIPT_TOKEN.match(head):
                prose += 1
                continue
            target = SCRIPTS / head
            if not target.is_file():
                failures.append(f"FAIL {where}: names `{head}`, which is not "
                                f"a script under {SCRIPTS}")
                continue
            commands += 1
            why = accepts(target, line.split()[1:])
            if why:
                failures.append(f"FAIL {where}: prints `{line}`\n"
                                f"       {head} refuses it -- {why}")

    print(f"read {commands} printed command(s) and {prose} prose line(s)")
    if commands == 0:
        print("FAIL: no printed line resolved to a script under scripts/; "
              "this gate asserted nothing")
        failures.append("vacuous")
    for f in failures:
        print(f)
    print(f"\n=> {'RED: ' + str(len(failures)) + ' defect(s)' if failures else 'GREEN'}")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
