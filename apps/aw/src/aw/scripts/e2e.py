#!/usr/bin/env python3
"""The contract phase of a change work item.

This is where the work item's observable difference becomes something that can
refuse an implementation. Nothing under `src/` may be written here, so the only
thing this phase can produce is a case that is red -- and a case that is red
before the implementation exists is the only kind that proves anything when it
later goes green.

It replaced `ec.py`, which is no longer in this plugin, and the rename was not
cosmetic. `e2e` is what the artifact is called nearly everywhere else, and the
phase after this one has to be told which artifact it is satisfying by a name it
recognises without being taught.

Two things went away with the TD phase, and both were load-bearing only for it:

  The `AW_EC_STAGE` branch. A case had to be told which phase was asking
  because the TD phase could only ever observe a document, and a case pointed
  at a document is looking at something categorically different from a case
  pointed at a binary. With no TD phase every case runs against the
  implementation at every phase, always, so a case reads no environment and
  has one `verify()`.

  The `applicability` axis, for the same reason. There is no phase left at
  which a dimension is ungateable, so there is nothing left to defer and no
  carried-forward set for a later phase to consult.

  E1  every case this phase wrote is red.

      Red *in the working tree*, which is a HEAD measurement and not a weaker
      one: `C0` refused every changed path outside the e2e root, so the product
      in the tree is the product at HEAD, byte for byte. Running the cases in a
      HEAD worktree instead would not work at all -- they are uncommitted, so
      they are not in it.

The four verbs, in order -- this is the gate:

  start <iid>     open the phase; refuses a dirty tree
  verify <iid>    the mechanical list over the whole change; runs no case
  test <iid>      run every case and require each one red
  commit <iid>    re-run everything and commit the diff
"""
from __future__ import annotations

import argparse
import importlib.util
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

if sys.version_info < (3, 11):
    sys.stderr.write(
        f"e2e.py reads the case inventory out of the crate manifest, which "
        f"needs Python 3.11+ for tomllib; this is {sys.version.split()[0]}.\n"
        "Invoke it as: uv run --python 3.13 --no-project <path>/e2e.py ...\n"
    )
    raise SystemExit(2)

import tomllib  # noqa: E402

# `leg.py` carries what every phase shares. This bootstrap is the one thing
# that cannot itself come from `leg.py`, and the `sys.modules` guard keeps the
# module single when more than one script loads it.
if "legmod" in sys.modules:
    leg = sys.modules["legmod"]
else:
    _LEG_SPEC = importlib.util.spec_from_file_location(
        "legmod", Path(__file__).resolve().parent / "leg.py")
    leg = importlib.util.module_from_spec(_LEG_SPEC)
    sys.modules["legmod"] = leg
    _LEG_SPEC.loader.exec_module(leg)

Check = leg.Check
GIT = leg.GIT
PHASE = "e2e"

# The three ways a Rust case can say it checked something. A case carrying
# none of them is a program that runs and reports nothing, which is green
# forever and therefore evidence of nothing.
ASSERT_MACROS = ("assert!", "assert_eq!", "assert_ne!")

# What can be decided about an assertion without running it. A Rust literal
# on both sides of `assert_eq!` is the one shape whose verdict is fixed at
# read time; everything else needs the product, and that is `E1`'s job.
_LIT = r'(?:-?\d[\d_]*(?:\.\d+)?|true|false|"[^"\n]*")'
VACUOUS_CMP = re.compile(rf"assert_(?:eq|ne)!\(\s*{_LIT}\s*,\s*{_LIT}\s*[,)]")
VACUOUS_BOOL = re.compile(r"assert!\(\s*(?:true|false)\s*[,)]")


def git(repo: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run([*GIT, *args], cwd=repo, capture_output=True, text=True)


# --------------------------------------------------------------------------
# locating things
# --------------------------------------------------------------------------
def e2e_root(repo: Path, project: str) -> Path:
    """Where the cases go. Says nothing about whether anything is there yet.

    It used to refuse a project with no inventory, and that was wrong at the
    one moment it mattered most: on the first work item of a project the
    inventory does not exist, and refusing meant `e2e start` -- the verb whose
    whole job is to say "write the cases" -- died before printing anything.
    Worse, it died on stderr with a `SystemExit`, so no row in any report
    accounted for it. A missing inventory is a finding, and `C1` is where a
    finding belongs.
    """
    return leg.leg_root(repo, project, PHASE)


def manifest(root: Path) -> Path:
    """The crate manifest that owns `root`. It is the case inventory.

    One level above the e2e root rather than inside it, which is the whole
    reason `C0` has to allow a path outside the phase's write root: registering
    a case and writing it are one act, and the register lives here.
    """
    return root.parent / "Cargo.toml"


class E2eInventory:
    """`[[test]]` from the crate manifest, or the reason there is no inventory.

    Cargo already has a register of test targets and a rule about what happens
    to a file nobody declared, so this reads that one instead of adding a
    second. The alternative -- an `[[aw.e2e.cases]]` table beside it -- is two
    answers to "what runs", free to disagree, with the disagreement showing up
    as a coverage difference rather than as a defect.

    `autotests = false` is checked, never assumed. With autodiscovery on, an
    undeclared file under `e2e/` runs anyway and a declaration whose file is
    gone is ignored -- so the manifest describes the inventory rather than
    being it, and there is nothing here for `C1` to refuse a case against.

    Missing, unusable and empty are three answers, and none of them is
    defaulted to the others. `problem` carries which one, in the words the
    author has to act on; `cases` is what the phase runs.
    """

    def __init__(self, root: Path) -> None:
        self.problem = ""
        self.crate = ""
        self.cases: dict[str, dict[str, Any]] = {}
        path = manifest(root)
        if not path.is_file():
            self.problem = (
                f"there is no crate manifest at {path.name} beside "
                f"{root.name}/, so nothing declares what runs")
            return
        data = tomllib.loads(path.read_text(encoding="utf-8"))
        package = data.get("package", {})
        self.crate = str(package.get("name", "")).strip()
        if not self.crate:
            self.problem = f"{path.name} declares no `[package] name`"
            return
        if package.get("autotests") is not False:
            self.problem = (
                f"{path.name} does not set `autotests = false` under "
                "`[package]`\nwith autodiscovery on, a file nobody declared "
                "still runs and a declaration whose file is gone is ignored, "
                "so the manifest is a description of the inventory rather than "
                "the inventory itself")
            return
        prefix = f"{root.name}/"
        for stanza in data.get("test", []):
            name = str(stanza.get("name", "")).strip()
            declared = str(stanza.get("path", "")).strip()
            if not name or not declared.startswith(prefix):
                continue
            self.cases[name] = {
                "id": name,
                "path": declared,
                # Derived, not declared. The command for a `[[test]]` target is
                # fixed by cargo, so reading one out of the manifest would be
                # inventing a key cargo does not have and letting it drift from
                # the target it names.
                "command": f"cargo test -p {self.crate} --test {name}",
            }
        if not self.cases:
            self.problem = (
                f"{path.name} declares no `[[test]]` whose `path` is under "
                f"{prefix}\nthis is the first case in the project: the "
                "inventory has to be appended to, and a file that is not in it "
                "does not run")


def inventory(root: Path) -> dict[str, dict[str, Any]]:
    """Every case the project declares, keyed by id. Empty if none are.

    The dict form is what `impl.py` asks for -- it needs the command and
    nothing else -- and it is kept one-argument so that where the inventory
    lives stays this module's business.
    """
    return E2eInventory(root).cases


def case_path(root: Path, case_id: str) -> Path:
    """Where the case's source is, per the manifest, or where it should be.

    The declared path is preferred over the conventional one because the
    manifest is what cargo obeys: a stanza pointing somewhere else is a case
    that runs from somewhere else, and reporting the conventional path would
    print a file nobody executes. The fallback is for the case the inventory
    does not name at all, which `C1` refuses -- and which still has to be
    printed by name in the refusal.
    """
    entry = inventory(root).get(case_id)
    if entry:
        return root.parent / entry["path"]
    return root / f"{case_id}.rs"


def changed_cases(repo: Path, root: Path, dirty: list[str]) -> list[str]:
    """The case ids this change wrote, from the dirty set.

    Files sitting directly in the e2e root, and only those. A `[[test]]` stanza
    names one file, so a subdirectory holds fixtures for a harness rather than
    cases -- `apps/lumen/e2e/rig/` and `apps/lumen/e2e/ec/` are both that -- and
    a stem lifted out of one would be a case id no stanza can ever match. The
    crate manifest is in the dirty set whenever a case was registered, and it is
    not a case either; the `.rs` suffix is what excludes it.
    """
    prefix = f"{root.relative_to(repo)}/"
    out: set[str] = set()
    for path in dirty:
        if not path.startswith(prefix) or not path.endswith(".rs"):
            continue
        if "/" in path[len(prefix):]:
            continue
        out.add(Path(path).stem)
    return sorted(out)


# --------------------------------------------------------------------------
# the checks this phase adds
# --------------------------------------------------------------------------
def c1_registered(chk: Check, root: Path, cases: list[str],
                  inv: "E2eInventory") -> None:
    """Every case this phase wrote is declared, and declares itself.

    Two halves, one row, because they are the same defect seen from two sides:
    a case the inventory does not name is one no later phase will run, and a
    case that does not name itself is one a reader cannot classify without
    executing it. Either way the case is present on disk and absent from the
    contract, which is the shape that reads as coverage and is not.
    """
    if not cases:
        chk.add("FAIL", "C1 registered",
                f"this change wrote no case directly under {root.name}/\n"
                f"the {PHASE} phase's whole output is cases; a change here with "
                "none has nothing that could refuse the implementation later")
        return
    if inv.problem:
        chk.add("FAIL", "C1 registered",
                f"{len(cases)} case file(s) were written and nothing runs "
                f"them:\n{inv.problem}")
        return
    want = f"{root.name}/"
    problems: list[str] = []
    for case_id in cases:
        entry = inv.cases.get(case_id)
        if entry is None:
            problems.append(
                f"  {case_id}: no `[[test]]` named it, so cargo will not run "
                f"it -- add `name = \"{case_id}\"` with "
                f"`path = \"{want}{case_id}.rs\"`")
            continue
        expect = f"{want}{case_id}.rs"
        if entry["path"] != expect:
            problems.append(
                f"  {case_id}: declared at {entry['path']}, not {expect} -- "
                "the target name and the file stem have to agree, or the "
                "command that runs one names the other")
        source = case_path(root, case_id)
        if not source.is_file():
            problems.append(f"  {case_id}: declared at {entry['path']}, "
                            "which is not a file")
    if problems:
        chk.add("FAIL", "C1 registered", "\n".join(problems))
        return
    chk.add("PASS", "C1 registered",
            f"all {len(cases)} case(s) are declared in "
            f"{manifest(root).name} and present on disk")


def _line_of(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def c2_observes_product(chk: Check, root: Path, cases: list[str]) -> None:
    """No case asserts something it wrote down itself.

    An `assert_eq!(1, 1)`, or an assert over two literals the case supplied, is
    green forever and red never -- so it survives `E1` only by accident and
    passes every phase after. The check is deliberately shallow: it refuses an
    assertion whose verdict is fixed at read time, which is the only shape that
    can be decided without running anything. A case that reaches out and gets
    the wrong thing is a defect no static reading can catch, and `E1` is what
    catches it.

    It reads the text rather than a syntax tree. There is no Rust parser in the
    standard library and the phase scripts take no dependency, so a tree would
    mean either vendoring one or shelling out to a toolchain the phase does not
    otherwise need. The cost is that a literal comparison inside a comment or a
    string reads as one: that direction refuses a case the author can rewrite,
    where a parser this shallow getting it wrong the other way would let a
    vacuous assertion through.
    """
    problems: list[str] = []
    for case_id in cases:
        path = case_path(root, case_id)
        text = path.read_text(encoding="utf-8", errors="replace")
        if not any(macro in text for macro in ASSERT_MACROS):
            problems.append(
                f"  {case_id}: contains none of "
                f"{', '.join(f'`{m}`' for m in ASSERT_MACROS)}, so nothing in "
                "it can fail on what the product did")
            continue
        for pattern, why in ((VACUOUS_CMP, "both sides of this comparison are "
                                           "literals"),
                             (VACUOUS_BOOL, "this assertion is a literal")):
            for hit in pattern.finditer(text):
                problems.append(
                    f"  {case_id}:{_line_of(text, hit.start())}: {why}, so it "
                    "observes nothing")
    if problems:
        chk.add("FAIL", "C2 observes the product", "\n".join(problems))
        return
    chk.add("PASS", "C2 observes the product",
            f"every assertion in {len(cases)} case(s) reads something")


def e1_cases_are_red(chk: Check, repo: Path, cases: list[str],
                     inv: dict[str, dict[str, Any]]) -> None:
    """Every case this phase wrote refuses the product as it stands.

    This is a measurement against HEAD even though it runs in the working tree.
    `C0` refused every changed path outside the e2e root, so the product here
    *is* the product at HEAD. Running them in a detached HEAD worktree would
    not be stricter, it would be impossible: the cases are uncommitted, so they
    are not in it.
    """
    green: list[str] = []
    results = [leg.run_command(repo, inv[case_id]["command"]) for case_id in cases]
    dead = leg.unrunnable(*results)
    if dead:
        chk.add("FAIL", "E1 cases are red", dead
                + "\na case that could not be started did not refuse anything. "
                "This row reads a non-zero exit as the answer it wants, so a "
                "command that never ran would pass it without observing the "
                "product at all")
        return
    for case_id, result in zip(cases, results):
        if result["exit"] == 0:
            green.append(f"  {case_id}")
    if green:
        chk.add("FAIL", "E1 cases are red",
                "green before the implementation exists:\n" + "\n".join(green)
                + "\na case that passes now cannot distinguish the change from "
                "the tree it was written against -- either it observes "
                "something other than the behaviour it names, or that behaviour "
                "is already there and this work item has nothing to do")
        return
    chk.add("PASS", "E1 cases are red",
            f"all {len(cases)} case(s) refuse the product at HEAD")


# --------------------------------------------------------------------------
# the run
# --------------------------------------------------------------------------
def _wi_checks(args: argparse.Namespace, *, require_clean: bool,
               run_cases: bool):
    chk = Check()
    repo = leg.repo_root()
    root = e2e_root(repo, args.project)

    kind = leg.p0_delivery_flow(chk, repo, args.wi, "behavior")
    if chk.failed:
        return chk, repo, root, [], []
    leg.p1_work_item(chk, repo, args.wi, kind)
    if chk.failed:
        return chk, repo, root, [], []

    dirty = leg.dirty_set(repo)
    if require_clean:
        leg.p2_clean_tree(chk, dirty)
        leg.p3_leg_is_open(chk, repo, args.wi, PHASE)
        leg.p4_predecessor_landed(chk, repo, args.wi, PHASE)
        return chk, repo, root, dirty, []

    leg.p3_leg_is_open(chk, repo, args.wi, PHASE)
    leg.p4_predecessor_landed(chk, repo, args.wi, PHASE)
    leg.c0_scope(chk, repo, root, dirty, PHASE)
    if chk.failed:
        return chk, repo, root, dirty, []

    inv = E2eInventory(root)
    cases = changed_cases(repo, root, dirty)
    c1_registered(chk, root, cases, inv)
    if not chk.failed:
        c2_observes_product(chk, root, cases)
    if run_cases:
        if chk.failed:
            chk.add("PENDING", "E1 cases are red",
                    "not run: a FAIL above means a red here would be a red "
                    "about the case rather than about the product")
        else:
            e1_cases_are_red(chk, repo, cases, inv.cases)
    return chk, repo, root, dirty, cases


# --------------------------------------------------------------------------
# verbs
# --------------------------------------------------------------------------
def cmd_start(args: argparse.Namespace) -> int:
    """Open the phase, then print what the cases have to pin."""
    chk, repo, root, _dirty, _cases = _wi_checks(
        args, require_clean=True, run_cases=False)
    print(f"opening the {PHASE.upper()} phase of #{args.wi}")
    chk.report()
    if chk.failed:
        print("\nthe phase was not opened; nothing on disk changed.")
        print("next.command: clear the FAIL rows above, then re-run this verb")
        return 1

    body = leg.wi_body_path(repo, args.wi).read_text(encoding="utf-8")
    mod = leg.change_module()
    print()
    print("=" * 78)
    for heading in ("## Goal", "## Acceptance"):
        section = mod.section_at(body, 2, heading)
        print(f"{heading}\n")
        print((section or "(this work item has no such section)").strip())
        print()
    print("=" * 78)
    print("Write the cases directly under")
    print(f"  {root.relative_to(repo)}/")
    print("one file per case, and declare each one in")
    print(f"  {manifest(root).relative_to(repo)}")
    print("as")
    print("  [[test]]")
    print('  name = "<stem>"')
    print(f'  path = "{root.name}/<stem>.rs"')
    print()
    print("That manifest is the inventory, and the declaration is not optional:")
    print("`autotests = false` is set, so a file nobody declared does not run.")
    print("It is the one path outside the e2e root this phase may write.")
    print()
    print("Every case must be RED against the tree as it stands. A case that is")
    print("green now measures the tree it was written against, not the change --")
    print("and it will still be green after the implementation, saying nothing.")
    print()
    print(f"Nothing under apps/{args.project}/src/ may be written here. The unit")
    print("tests and the implementation are two later phases, and a case written")
    print("beside the code it is meant to refuse has nothing left to refuse.")
    print(f"\nnext.command: {leg.phase_command(PHASE, args.project, 'verify', args.wi)}")
    return 0


def cmd_verify(args: argparse.Namespace) -> int:
    chk, _repo, _root, _dirty, cases = _wi_checks(
        args, require_clean=False, run_cases=False)
    print(f"mechanical admissibility: #{args.wi}")
    chk.report()
    print()
    print("These checks say the change is ADMISSIBLE -- it is a case-only change")
    print("and every case is inventoried and self-declaring. They do not say the")
    print("cases discriminate: nothing was run.")
    if chk.failed:
        print("\nnext.command: fix the FAIL rows above, then re-run this verb")
        return 1
    print(f"\n{len(cases)} case(s) must come out red: {', '.join(cases)}")
    print(f"next.command: {leg.phase_command(PHASE, args.project, 'test', args.wi)}")
    return 0


def cmd_test(args: argparse.Namespace) -> int:
    chk, _repo, _root, _dirty, cases = _wi_checks(
        args, require_clean=False, run_cases=True)
    print(f"the cases against the product as it stands: #{args.wi}")
    chk.report()
    if chk.failed:
        print("\nnext.command: fix the FAIL rows above, then re-run this verb")
        return 1
    print(f"\nall {len(cases)} case(s) refuse the product: {', '.join(cases)}")
    print(f"next.command: {leg.phase_command(PHASE, args.project, 'commit', args.wi)}")
    return 0


def cmd_commit(args: argparse.Namespace) -> int:
    chk, repo, _root, dirty, cases = _wi_checks(
        args, require_clean=False, run_cases=True)
    print(f"commit gate: #{args.wi}")
    chk.report()
    if chk.failed:
        print("\nnothing was committed; the tree is unchanged and the work is still here.")
        print("next.command: fix the FAIL rows above, then re-run this verb")
        return 1

    # The allowlist *is* the dirty set, which is what makes the commit and the
    # thing that was tested the same object.
    digest = leg.change_digest(repo, args.wi, dirty)
    trailers = [
        # The names, not a count and not an exit code. The phase after this one
        # reads this line to learn which cases it is being measured against,
        # and a count would let it be measured against a different set of the
        # same size.
        f"E2E-Red: {', '.join(cases)}",
        f"E2E-Change-Digest: {digest}",
    ]
    message = (f"{PHASE}(wi-{args.wi}): pin the observable difference\n"
               f"\nRefs #{args.wi}\n"
               + "\n" + "\n".join(trailers) + "\n")

    if args.dry_run:
        print("\n-- would commit, exactly these paths ------------------------")
        for p in dirty:
            print(f"  {p}")
        print("-- message -------------------------------------------------")
        print(message)
        return 0

    # A brand-new case file is untracked, and `git commit -- <pathspec>`
    # refuses a path git has never seen.
    add = git(repo, "add", "--", *dirty)
    if add.returncode != 0:
        print(add.stderr)
        return add.returncode
    proc = git(repo, "commit", "-m", message, "--", *dirty)
    print(proc.stdout or proc.stderr)
    if proc.returncode != 0:
        return proc.returncode

    sha = git(repo, "rev-parse", "HEAD").stdout.strip()
    print(f"E2E-Commit: {sha}")
    print(f"\nnext.command: {leg.AW_CLI} change lifecycle {args.wi} --leg {PHASE} "
          f"--commit {sha} --digest {digest}")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="e2e.py", description=__doc__.splitlines()[0])
    # Required, with no default. It used to default to `agentic-workflow`;
    # that crate was removed, so the default became a write root that does
    # not exist -- and a phase resolving a missing directory reports a
    # project the caller never chose. There is no project this plugin can
    # assume, so it is named or the run refuses.
    parser.add_argument("--project", required=True,
                        help="project under apps/; must precede the verb")
    sub = parser.add_subparsers(dest="verb", required=True)

    wi = argparse.ArgumentParser(add_help=False)
    wi.add_argument("wi", type=int, help="work item iid")

    p = sub.add_parser("start", parents=[wi],
                       help=f"open a work item's {PHASE.upper()} phase; refuses a dirty tree")
    p.set_defaults(func=cmd_start)

    p = sub.add_parser("verify", parents=[wi],
                       help="the mechanical list over the whole change; runs no case")
    p.set_defaults(func=cmd_verify)

    p = sub.add_parser("test", parents=[wi],
                       help="run every case and require each one red")
    p.set_defaults(func=cmd_test)

    p = sub.add_parser("commit", parents=[wi],
                       help="re-run everything and commit the change")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_commit)

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
