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

  C7  an independent reviewer accepted these exact bytes.

      `E1` proves a case is red. It cannot prove the case is red *for the
      reason it claims* -- a case with a typo in a path is red too, and stays
      red through every phase without ever observing the product. That is a
      reading question, so it goes to a reviewer, and the answer is bound to a
      digest over the work item and the change so that editing either one
      after the review invalidates it.

Work-item scoped -- this is the gate:

  start <iid>     open the phase; refuses a dirty tree
  verify <iid>    the mechanical list over the whole change; runs no case
  test <iid>      run every case and require each one red
  review-prompt <iid>   the prompt for the reviewer, scoped to this change
  verdict <iid>   bind the reviewer's transcript to the bytes it read
  commit <iid>    re-run everything and commit the diff

Whole-surface, advisory:

  review-prompt   the same review over every case in the inventory

  With no work item there is no change to scope to and nothing to bind an
  answer to, so this form records nothing. It is for reading the contract a
  project already has, not for passing a gate.
"""
from __future__ import annotations

import argparse
import ast
import importlib.util
import subprocess
import sys
from pathlib import Path
from typing import Any

if sys.version_info < (3, 11):
    sys.stderr.write(
        f"e2e.py reads the case inventory from a pyproject.toml, which needs "
        f"Python 3.11+ for tomllib; this is {sys.version.split()[0]}.\n"
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

# What a case must declare about itself before it is trusted to run. These are
# read out of the file without importing it, so the declaration is available
# before the code behind it executes.
REQUIRED_CONSTANTS = ("CASE_ID", "DIMENSION", "TARGET_COMMAND", "ASSERTIONS")

# What the inventory must say about each case. `promise` and `oracle` are here
# because a case whose inventory entry says only "it exists" gives a reader no
# way to tell a real oracle from a tautology without opening the file.
REQUIRED_ENTRY_KEYS = ("id", "dimension", "promise", "oracle", "command")


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


def inventory(root: Path) -> dict[str, dict[str, Any]]:
    """Every case the project declares, keyed by id. Empty if none are.

    The key is `python-e2e`, not the `python-ec` the retiring lifecycle reads.
    They are separate because the roots are separate: a project mid-changeover
    has both an `external-contracts/` tree and an `e2e/` tree, and one key
    across both would make a case belong to whichever inventory was parsed
    last.
    """
    path = root / "pyproject.toml"
    if not path.is_file():
        return {}
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    cases = data.get("tool", {}).get("aw", {}).get("python-e2e", {}).get("cases", [])
    return {c["id"]: c for c in cases}


def case_path(root: Path, case_id: str) -> Path:
    return root / "src" / "cases" / f"{case_id}.py"


def changed_cases(repo: Path, root: Path, dirty: list[str]) -> list[str]:
    """The case ids this change wrote, from the dirty set."""
    prefix = f"{(root / 'src' / 'cases').relative_to(repo)}/"
    return sorted({Path(p).stem for p in dirty
                   if p.startswith(prefix) and p.endswith(".py")})


# --------------------------------------------------------------------------
# the checks this phase adds
# --------------------------------------------------------------------------
def c1_registered(chk: Check, root: Path, cases: list[str],
                  inv: dict[str, dict[str, Any]]) -> None:
    """Every case this phase wrote is declared, and declares itself.

    Two halves, one row, because they are the same defect seen from two sides:
    a case the inventory does not name is one no later phase will run, and a
    case that does not name itself is one a reader cannot classify without
    executing it. Either way the case is present on disk and absent from the
    contract, which is the shape that reads as coverage and is not.
    """
    if not cases:
        chk.add("FAIL", "C1 registered",
                f"this change wrote no case under {root.name}/src/cases/\n"
                f"the {PHASE} phase's whole output is cases; a change here with "
                "none has nothing that could refuse the implementation later")
        return
    if not (root / "pyproject.toml").is_file():
        chk.add("FAIL", "C1 registered",
                f"there is no inventory at {root.name}/pyproject.toml, so the "
                f"{len(cases)} case file(s) this change wrote are scripts in a "
                "folder\nthis is the first work item in the project: the "
                "inventory has to be created, not appended to")
        return
    problems: list[str] = []
    for case_id in cases:
        entry = inv.get(case_id)
        if entry is None:
            problems.append(f"  {case_id}: not in the inventory's "
                            "[[tool.aw.python-e2e.cases]]")
            continue
        missing = [k for k in REQUIRED_ENTRY_KEYS if not str(entry.get(k, "")).strip()]
        if missing:
            problems.append(f"  {case_id}: inventory entry is missing "
                            f"{', '.join(missing)}")
        declared = leg.case_constants(case_path(root, case_id))
        absent = [k for k in REQUIRED_CONSTANTS if k not in declared]
        if absent:
            problems.append(f"  {case_id}: the file declares no {', '.join(absent)}")
        elif declared["CASE_ID"] != case_id:
            problems.append(f"  {case_id}: declares CASE_ID="
                            f"{declared['CASE_ID']!r}, which no inventory entry names")
    if problems:
        chk.add("FAIL", "C1 registered", "\n".join(problems))
        return
    chk.add("PASS", "C1 registered",
            f"all {len(cases)} case(s) are inventoried and self-declaring")


def _is_literal(node: ast.expr) -> bool:
    try:
        ast.literal_eval(node)
    except (ValueError, TypeError, SyntaxError):
        return False
    return True


def c2_observes_product(chk: Check, root: Path, cases: list[str]) -> None:
    """No case asserts something it computed out of thin air.

    An `assert 1 == 1`, or an assert over two constants the case itself wrote,
    is green forever and red never -- so it survives `E1` only by accident and
    passes every phase after. The check is deliberately shallow: it refuses an
    assert whose *both* sides are literals, which is the only shape that can be
    decided without running anything. A case that reaches out and gets the
    wrong thing is a defect no static reading can catch, and `E1` is what
    catches it.
    """
    problems: list[str] = []
    for case_id in cases:
        path = case_path(root, case_id)
        tree = ast.parse(path.read_text(encoding="utf-8"))
        asserts = [n for n in ast.walk(tree) if isinstance(n, ast.Assert)]
        if not asserts:
            problems.append(f"  {case_id}: contains no assert")
            continue
        for node in asserts:
            test = node.test
            if isinstance(test, ast.Compare) and len(test.comparators) == 1:
                if _is_literal(test.left) and _is_literal(test.comparators[0]):
                    problems.append(
                        f"  {case_id}:{node.lineno}: both sides of this assert "
                        "are literals, so it observes nothing")
            elif _is_literal(test):
                problems.append(
                    f"  {case_id}:{node.lineno}: this assert is a literal, so "
                    "it observes nothing")
    if problems:
        chk.add("FAIL", "C2 observes the product", "\n".join(problems))
        return
    chk.add("PASS", "C2 observes the product",
            f"every assert in {len(cases)} case(s) reads something")


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
    for case_id in cases:
        result = leg.run_command(repo, inv[case_id]["command"])
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
# the semantic review
# --------------------------------------------------------------------------
# The skill that runs it. Named here rather than routed through a project's
# config: with one reviewer per phase the indirection bought nothing, and a
# constant in the script is something `check_plugin.py` can resolve against the
# skills that actually exist -- where a config value can only ever be compared
# against itself.
REVIEWER = "/aw:codex-e2e-review"

RUBRIC = f"""\
You are reviewing the end-to-end (E2E) contract for the axiom repository. Below
you are given the work item this change must satisfy, every path it touches,
and the full source of every case it adds or edits.

An E2E case is a black-box verifier. Its whole job is to pin externally
observable product behaviour so that a wrong implementation cannot pass. These
cases are expected to be RED right now: the behaviour they pin does not exist
yet. Do not report "the case fails" as a finding -- that is the design.

Q0 is the question this review exists to answer. Q1-Q7 are the ways an answer
of "yes" to Q0 can still be worthless.

  Q0 DOES THIS CHANGE SATISFY THE WORK ITEM?
     The work item's `## Goal` names a trigger, an observation point, a current
     value, and a target value; its `## Acceptance` names the gates. Read them
     against the cases actually written here.
     - Is every observable the work item promises pinned by some case?
     - Does any case pin something the work item did not ask for?
     - Where the work item names a specific command, value, or file, does the
       case assert on THAT one, or on something adjacent and easier to hit?
     Name any requirement in the work item that no case here would refuse.

Answer each of the following explicitly too. Each names a way a case can be
green while measuring nothing; if you cannot rule one out from the source
below, say so. Apply them per case, naming the case each finding is about.

  Q1 DISCRIMINATION
     Describe a concrete WRONG implementation that would still make this case
     pass. If you can name one, the case has a hole.

  Q2 ORACLE INDEPENDENCE
     Does the case derive its expected value from the same code path it is
     checking? An expectation computed by the product is not an oracle.

  Q3 BLACK BOX
     Does the case reach past the product's external surface -- importing its
     internals, reading its private state, asserting on a data structure it
     could have observed from the outside?

  Q4 PROMISE VS ASSERTION
     The inventory entry states a `promise` and an `oracle`. Does the code
     assert THAT, or something weaker that happens to hold?

  Q5 SETUP
     Is the red this case produces the red it claims -- a missing behaviour --
     or the red of a broken fixture, a wrong path, or an import that cannot
     resolve? The two are the same exit code and only one of them is evidence.

  Q6 VACUITY
     Could this case pass with the product absent entirely? Deleted, renamed,
     never built?

  Q7 DECLARED FAILURE
     Does the assertion message name the same observable the `ASSERTIONS`
     tuple declares? A case whose failure text describes a different check is
     one nobody can attribute later.

{leg.OUTPUT_CONTRACT}"""


def _print_cases(repo: Path, root: Path, inv: dict[str, dict[str, Any]],
                 cases: list[str], *, run: bool) -> None:
    """Every case the reviewer has to read, and how it currently fails.

    `run` is off for the whole-surface form. Producing a prompt is not worth
    executing a project's entire inventory -- the largest one measured here ran
    to 142 cases -- and the advisory review is a reading of the contract, not a
    measurement of it. The scoped form runs them, because "is this red for the
    reason it claims" is exactly Q5 and cannot be answered without the output.
    """
    for case_id in cases:
        entry = inv.get(case_id, {})
        source = case_path(root, case_id)
        print("=" * 78)
        print(f"CASE      : {case_id}")
        print(f"DIMENSION : {entry.get('dimension')}")
        print()
        print("-- inventory entry ------------------------------------------------")
        for field in ("promise", "oracle", "target", "command"):
            if entry.get(field):
                print(f"{field}: {entry[field]}")
        print()
        if run and entry.get("command"):
            result = leg.run_command(repo, entry["command"])
            print("-- currently fails with -------------------------------------------")
            print(f"exit={result['exit']} {result['exception']}: "
                  f"{result['message'][:600]}")
            print()
        print(f"-- case source: {source.relative_to(repo)} ------------------------")
        print(source.read_text(encoding="utf-8") if source.is_file()
              else "(the inventory names this case, but no file is there)")


def cmd_review_prompt(args: argparse.Namespace) -> int:
    """The prompt the reviewer is fed: the work item, then the change.

    Both halves are here because the question is a comparison. A prompt
    carrying only the cases can be answered "these are well-built verifiers" by
    a reviewer who never learned what was asked for, and that answer is
    indistinguishable from the one worth having.
    """
    if args.wi is None:
        repo = leg.repo_root()
        root = e2e_root(repo, args.project)
        inv = inventory(root)
        print(RUBRIC)
        print("=" * 78)
        print(f"WHOLE SURFACE: every case in {root.relative_to(repo)}")
        print("There is no work item here, so Q0 has nothing to compare against.")
        print("Answer Q1-Q7 per case and skip Q0.")
        print()
        _print_cases(repo, root, inv, sorted(inv), run=False)
        print()
        print("This review is advisory: no verdict can be recorded for it, "
              "because there is no change for one to bind to.")
        return 0

    chk, repo, root, dirty, cases = _wi_checks(
        args, require_clean=False, run_cases=False)
    if chk.failed:
        chk.report()
        raise SystemExit(
            f"#{args.wi} is not mechanically admissible yet, and a semantic review "
            f"of an inadmissible change spends a reviewer on a question the checks "
            f"already answered. Run: "
            f"{leg.phase_command(PHASE, args.project, 'verify', args.wi)}"
        )

    print(RUBRIC)
    print("=" * 78)
    print(f"WORK ITEM : #{args.wi}")
    print(f"DIGEST    : {leg.change_digest(repo, args.wi, dirty)}")
    print()
    print("-- the work item this change must satisfy -------------------------")
    print(leg.wi_body_path(repo, args.wi).read_text(encoding="utf-8"))
    print("-- every path this change touches ---------------------------------")
    for path in dirty:
        print(f"  {path}")
    print()
    _print_cases(repo, root, inventory(root), cases, run=True)

    rest = [p for p in dirty
            if p not in {str(case_path(root, c).relative_to(repo)) for c in cases}]
    if rest:
        print("=" * 78)
        print("-- the rest of the change -----------------------------------------")
        tracked = git(repo, "diff", "HEAD", "--", *rest).stdout
        if tracked.strip():
            print(tracked)
        for path in rest:
            target = repo / path
            if target.is_file() and not tracked_by_git(repo, path):
                print(f"-- new file: {path} --")
                print(target.read_text(encoding="utf-8", errors="replace"))
    return 0


def tracked_by_git(repo: Path, rel: str) -> bool:
    return git(repo, "ls-files", "--error-unmatch", "--", rel).returncode == 0


def cmd_verdict(args: argparse.Namespace) -> int:
    if args.wi is None:
        # Before the checks, because they cannot run without a work item and
        # the refusal is about the missing one rather than about anything they
        # would have found.
        return leg.run_verdict(args, PHASE, REVIEWER, Check(), leg.repo_root(), [], [])
    chk, repo, _root, dirty, cases = _wi_checks(
        args, require_clean=False, run_cases=False)
    return leg.run_verdict(args, PHASE, REVIEWER, chk, repo, dirty, cases)


# --------------------------------------------------------------------------
# the run
# --------------------------------------------------------------------------
def _wi_checks(args: argparse.Namespace, *, require_clean: bool,
               run_cases: bool, include_verdict: bool = False):
    chk = Check()
    repo = leg.repo_root()
    root = e2e_root(repo, args.project)

    def pending_verdict(why: str) -> None:
        # Named even when it cannot run, so the report never omits a row a
        # later run will have. A silent absence and a green read the same in a
        # summary, which is how a phase comes to look complete.
        if include_verdict:
            chk.add("PENDING", "C7 reviewed", why)

    leg.p1_work_item(chk, repo, args.wi)
    if chk.failed:
        pending_verdict("not run: there is no admissible work item to have "
                        "been reviewed against")
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
        pending_verdict("not run: the change is out of scope, so a verdict "
                        "over it would be about paths this phase may not write")
        return chk, repo, root, dirty, []

    inv = inventory(root)
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
            e1_cases_are_red(chk, repo, cases, inv)
    if include_verdict:
        leg.c7_verdict(chk, repo, PHASE, args.wi, dirty, REVIEWER)
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
    for heading in ("Goal", "Acceptance"):
        section = mod.section_at(body, 2, heading)
        print(f"## {heading}\n")
        print((section or "(this work item has no such section)").strip())
        print()
    print("=" * 78)
    print("Write the cases under")
    print(f"  {(root / 'src' / 'cases').relative_to(repo)}/")
    print("and inventory each one in")
    print(f"  {(root / 'pyproject.toml').relative_to(repo)}")
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
    print("\nThe cases are red. Whether they are red for the reason they claim")
    print(f"is a reading question, and {REVIEWER} is what answers it.")
    print(f"next.command: "
          f"{leg.phase_command(PHASE, args.project, 'review-prompt', args.wi)}")
    return 0


def cmd_commit(args: argparse.Namespace) -> int:
    chk, repo, _root, dirty, cases = _wi_checks(
        args, require_clean=False, run_cases=True, include_verdict=True)
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
    print(f"\nnext.command: change.py lifecycle {args.wi} --leg {PHASE} "
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

    # The two review verbs take the same argument optionally. It is a separate
    # parent rather than a `nargs="?"` on the one above, so that omitting it
    # anywhere else stays the argparse error it has always been: a gate verb
    # that silently defaulted to "the whole project" would run against a
    # population nobody named.
    opt_wi = argparse.ArgumentParser(add_help=False)
    opt_wi.add_argument("wi", type=int, nargs="?",
                        help="work item iid; omit for the whole-surface review")

    p = sub.add_parser("start", parents=[wi],
                       help=f"open a work item's {PHASE.upper()} phase; refuses a dirty tree")
    p.set_defaults(func=cmd_start)

    p = sub.add_parser("verify", parents=[wi],
                       help="the mechanical list over the whole change; runs no case")
    p.set_defaults(func=cmd_verify)

    p = sub.add_parser("test", parents=[wi],
                       help="run every case and require each one red")
    p.set_defaults(func=cmd_test)

    p = sub.add_parser("review-prompt", parents=[opt_wi],
                       help="the reviewer's prompt; omit the iid for the whole surface")
    p.set_defaults(func=cmd_review_prompt)

    p = sub.add_parser("verdict", parents=[opt_wi],
                       help="bind a reviewer transcript to the bytes it read")
    p.add_argument("--transcript", required=True,
                   help="the reviewer's output, verbatim")
    p.set_defaults(func=cmd_verdict)

    p = sub.add_parser("commit", parents=[wi],
                       help="re-run everything and commit the change")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_commit)

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
