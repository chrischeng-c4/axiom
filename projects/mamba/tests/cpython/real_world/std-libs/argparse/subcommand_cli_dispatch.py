# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "real_world"
# case = "subcommand_cli_dispatch"
# subject = "argparse.ArgumentParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser: a git-style CLI with subcommands (add/commit) plus global --verbose flag parses each subcommand's own options and dispatches deterministically over a realistic argv set"""
import argparse


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="vcs")
    p.add_argument("--verbose", action="store_true")
    subs = p.add_subparsers(dest="cmd", required=True)

    add = subs.add_parser("add")
    add.add_argument("paths", nargs="+")

    commit = subs.add_parser("commit")
    commit.add_argument("-m", "--message", required=True)
    commit.add_argument("--amend", action="store_true")
    return p


def dispatch(argv: list[str]) -> str:
    ns = build_parser().parse_args(argv)
    prefix = "v:" if ns.verbose else ""
    if ns.cmd == "add":
        return f"{prefix}add({','.join(ns.paths)})"
    if ns.cmd == "commit":
        suffix = "+amend" if ns.amend else ""
        return f"{prefix}commit({ns.message}){suffix}"
    raise AssertionError(f"unexpected cmd {ns.cmd!r}")


cases = [
    (["add", "a.py", "b.py"], "add(a.py,b.py)"),
    (["--verbose", "add", "src/main.py"], "v:add(src/main.py)"),
    (["commit", "-m", "init"], "commit(init)"),
    (["--verbose", "commit", "--message", "fix", "--amend"], "v:commit(fix)+amend"),
]
for argv, expected in cases:
    got = dispatch(argv)
    assert got == expected, f"{argv!r} -> {got!r} != {expected!r}"

# A required subcommand omitted exits (with stderr suppressed).
import contextlib
import io

_raised = False
with contextlib.redirect_stderr(io.StringIO()):
    try:
        build_parser().parse_args([])
    except SystemExit:
        _raised = True
assert _raised, "missing required subcommand exits"

print("subcommand_cli_dispatch OK")
