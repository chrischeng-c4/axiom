#!/usr/bin/env python3
"""The implementation phase of a change work item.

Both oracles landed before this phase opened, and both are red. This is the
only phase at which either can be run against the thing it was written about,
so it is the only phase where a green means the promise is kept rather than
that something describes it.

  L1  it compiles.

      Same reason the `unit` phase has the row: a compile error and a failed
      assertion are one exit code, and every row below reads test *names* out
      of an output that a build failure never produced.

  L2  every test the `unit` phase recorded as red is now present and passing.

      By name, out of that phase's `Unit-Red` trailer. Present matters as much
      as passing: a test that was deleted is not failing, and a run that only
      counted failures would read its absence as success.

  L3  nothing else broke, and nothing else vanished.

      The whole declared suite is green, and every test that ran at HEAD still
      runs here. `L2` is scoped to the names this work item produced, so on its
      own it would let the implementation satisfy them at the cost of a test
      somebody else wrote -- and the vanishing half is invisible to every other
      row here, because a test that no longer runs cannot fail.

  L4  the e2e cases accept the implementation.

  L5  the same cases, run against `HEAD`, refuse it.

      `L4` alone cannot tell an implementation that made the cases pass from
      cases that were already passing -- and the second is what a case
      measuring nothing looks like at every phase it is ever run. The `e2e`
      phase proved them red before any implementation existed; this proves them
      red at the commit immediately underneath the change, which is the only
      point at which the difference is attributable to *this* work item.

There is no row here comparing the landed tests line by line against what the
`unit` phase wrote. There was one in the design, and it collapsed into `C0`:
the colocated tests live in their own file, so "this phase did not touch a
test" is a filename question that `C0` already answers from `git status`, and a
second row asking it from `git blame` would be the same claim measured more
weakly. The one thing neither can see is the `unit` commit being *amended* --
an amend rewrites the only record of what was measured -- and `L2` catches the
version of that which matters, because an amended-away test is a name that no
longer appears.

The four verbs, in order -- this is the gate:

  start <iid>     open the phase; refuses a dirty tree and a missing predecessor
  verify <iid>    the mechanical list over the whole change; runs nothing
  test <iid>      both oracles, in the tree and at HEAD
  commit <iid>    re-run everything and commit the diff
"""
from __future__ import annotations

import argparse
import importlib.util
import subprocess
import sys
from pathlib import Path

if sys.version_info < (3, 11):
    sys.stderr.write(
        f"logic.py reads `[unit]` from a project aw.toml, which needs Python "
        f"3.11+ for tomllib; this is {sys.version.split()[0]}.\n"
        "Invoke it as: uv run --python 3.13 --no-project <path>/logic.py ...\n"
    )
    raise SystemExit(2)

if "legmod" in sys.modules:
    leg = sys.modules["legmod"]
else:
    _LEG_SPEC = importlib.util.spec_from_file_location(
        "legmod", Path(__file__).resolve().parent / "leg.py")
    leg = importlib.util.module_from_spec(_LEG_SPEC)
    sys.modules["legmod"] = leg
    _LEG_SPEC.loader.exec_module(leg)

# Each earlier phase owns what it produced: `e2e.py` the case inventory,
# `unit.py` the declared build/test commands and the harness parser. Reading
# them from their owner is what keeps this phase from holding a second opinion
# about what the contract is -- a local copy of either would let the phase that
# has to satisfy the oracle also decide what the oracle says.
#
# What is *not* imported is the earlier phase's checks. `L1` duplicates `U1`'s
# mechanism and states the opposite remediation, which is what makes it a
# different row rather than a shared one.
e2e_mod = leg.sibling("e2e", "e2emod")
unit_mod = leg.sibling("unit", "unitmod")

Check = leg.Check
GIT = leg.GIT
PHASE = "logic"


def git(repo: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run([*GIT, *args], cwd=repo, capture_output=True, text=True)


def recorded_red(repo: Path, iid: int) -> tuple[list[str], str]:
    """The test names the `unit` phase recorded, and why not if it did not.

    Read off the commit rather than out of a state file. The commit is the same
    evidence `P4` accepted as proof the phase landed at all, so the two cannot
    disagree -- and a phase rebased away takes its record with it, where a
    state file would go on asserting a red for history that no longer exists.
    """
    commits = leg.leg_commits(repo, iid, "unit")
    if not commits:
        return [], f"no `unit(...)` commit carries `Refs #{iid}`"
    sha, _subject = commits[0]
    body = git(repo, "log", "-1", "--format=%B", sha).stdout
    for line in body.splitlines():
        if line.startswith("Unit-Red:"):
            names = [n.strip() for n in line.split(":", 1)[1].split(",") if n.strip()]
            if not names:
                return [], (f"the unit commit {sha[:9]} carries an empty "
                            "`Unit-Red:` trailer")
            return names, ""
    return [], (f"the unit commit {sha[:9]} carries no `Unit-Red:` trailer\n"
                "that trailer is the only record of which tests were failing "
                "when the phase closed, and without it there is nothing here to "
                "be measured against")


# --------------------------------------------------------------------------
# the checks this phase adds
# --------------------------------------------------------------------------
def l1_build_is_green(chk: Check, repo: Path, cfg) -> bool:
    """The implementation compiles, so a failure below is a failed assertion.

    Gates the rest rather than merely reporting, for the same reason `U1` does
    -- but the remediation is the opposite one, which is why this is its own
    row and not a call into the earlier phase. There the answer is "write the
    skeleton the tests need"; here the skeleton exists and the message must not
    send anybody back to it.
    """
    result = leg.run_command(repo, cfg.build)
    if result["exit"] != 0:
        tail = (result["stderr"] or result["stdout"]).strip().splitlines()
        chk.add("FAIL", "L1 build is green",
                f"`{cfg.build}` exited {result['exit']}\n"
                + "\n".join(f"  {line}" for line in tail[-15:])
                + "\nnothing ran, so nothing below this row observed anything. "
                "The tests and the cases are fixed; this is the implementation "
                "not compiling against them.")
        return False
    chk.add("PASS", "L1 build is green", f"`{cfg.build}` exited 0")
    return True


def l2_l3_tests(chk: Check, repo: Path, cfg, recorded: list[str]) -> None:
    """Both test rows, from one run of the suite.

    One run because two runs of the same command are two measurements, and a
    flaky test would let the two rows disagree about the same tree -- which
    would read as a defect in the change rather than in the suite.
    """
    tree = leg.run_command(repo, cfg.test)
    passed = cfg.names(tree["stdout"], "passed")
    failed = cfg.names(tree["stdout"], "failed")

    missing = [n for n in recorded if n not in passed and n not in failed]
    still_red = [n for n in recorded if n in failed]
    if missing or still_red:
        detail = ""
        if still_red:
            detail += ("the implementation does not satisfy the tests written "
                       "to hold it to it:\n"
                       + "\n".join(f"  {n}" for n in still_red) + "\n")
        if missing:
            detail += ("recorded as red by the unit phase, and not run here at "
                       "all:\n" + "\n".join(f"  {n}" for n in missing)
                       + "\na test that is absent is not a test that passed; "
                       "either it was deleted or the unit commit was amended "
                       "after it was measured\n")
        chk.add("FAIL", "L2 recorded tests green", detail.rstrip())
    else:
        chk.add("PASS", "L2 recorded tests green",
                f"all {len(recorded)} recorded test(s) ran and passed: "
                f"{', '.join(recorded)}")

    other = sorted(failed - set(recorded))
    if other:
        chk.add("FAIL", "L3 the suite is whole",
                "failing, and not part of what this work item recorded:\n"
                + "\n".join(f"  {n}" for n in other)
                + "\nthe recorded names are this change's oracle, not its "
                "permission slip -- a green there bought by a red anywhere else "
                "is a regression this phase caused")
        return

    with leg.at_head(repo) as (head, error):
        if head is None:
            chk.add("FAIL", "L3 the suite is whole",
                    "could not check out HEAD to compare the test population:\n"
                    + error)
            return
        at_head = leg.run_command(head, cfg.test)
        if at_head["unrunnable"]:
            chk.add("FAIL", "L3 the suite is whole", at_head["unrunnable"]
                    + "\nthis row is a comparison against the tests that ran at "
                    "HEAD, and a run that produced no output would report an "
                    "empty population as a whole one")
            return
        before = cfg.names(at_head["stdout"], "passed")

    lost = sorted(before - passed - failed)
    if lost:
        chk.add("FAIL", "L3 the suite is whole",
                "passing at HEAD and not present here at all:\n"
                + "\n".join(f"  {n}" for n in lost)
                + "\na deleted test cannot fail, so nothing else in this phase "
                "can see it go; the implementation is being measured against a "
                "smaller suite than the one it inherited")
        return
    chk.add("PASS", "L3 the suite is whole",
            f"{len(passed)} passing, none failing, and all {len(before)} "
            "test(s) that ran at HEAD still run here")


def l4_l5_contract(chk: Check, repo: Path, root: Path, cases: list[str]) -> None:
    """Both contract rows: the tree accepts, and HEAD refuses."""
    if not cases:
        chk.add("FAIL", "L4 the contract accepts the code",
                "the e2e phase landed no case, so there is nothing here that "
                "could accept or refuse this implementation")
        chk.add("PENDING", "L5 the contract refused HEAD", "not run: no cases")
        return

    inv = e2e_mod.inventory(root)
    unknown = [c for c in cases if c not in inv]
    if unknown:
        chk.add("FAIL", "L4 the contract accepts the code",
                "committed by the e2e phase and not in "
                f"{e2e_mod.manifest(root).relative_to(repo)}:\n"
                + "\n".join(f"  {c}" for c in unknown)
                + "\nthe inventory was edited after that phase closed")
        chk.add("PENDING", "L5 the contract refused HEAD",
                "not run: the contract could not be resolved")
        return

    red: list[str] = []
    for case_id in cases:
        result = leg.run_command(repo, inv[case_id]["command"])
        if result["exit"] != 0:
            line = f"  {case_id}: exit {result['exit']}"
            if result["exception"]:
                line += f" -- {result['exception']}: {result['message'][:200]}"
            red.append(line)
    if red:
        chk.add("FAIL", "L4 the contract accepts the code", "\n".join(red)
                + "\nthe implementation does not yet keep the promise these "
                "were written to hold it to")
        chk.add("PENDING", "L5 the contract refused HEAD",
                "not run: a case red in the tree says nothing about HEAD")
        return
    chk.add("PASS", "L4 the contract accepts the code",
            f"all {len(cases)} case(s) green in the working tree")

    with leg.at_head(repo) as (head, error):
        if head is None:
            chk.add("FAIL", "L5 the contract refused HEAD",
                    "could not check out HEAD to measure against:\n" + error)
            return
        at_head = [leg.run_command(head, inv[c]["command"]) for c in cases]
        dead = leg.unrunnable(*at_head)
        if dead:
            chk.add("FAIL", "L5 the contract refused HEAD", dead
                    + "\na case that could not be started at HEAD did not "
                    "refuse it, and this row reads a non-zero exit there as the "
                    "answer it wants")
            return
        green = [c for c, result in zip(cases, at_head) if result["exit"] == 0]

    if green:
        chk.add("FAIL", "L5 the contract refused HEAD",
                "already green at HEAD, before this implementation existed:\n"
                + "\n".join(f"  {c}" for c in green)
                + "\nso a green in the tree is a state these cases found rather "
                "than a difference this implementation made; either the case "
                "observes nothing, or the behaviour it pins was already there\n"
                "driving an epic, this is the ordinary shape of a child whose "
                "sibling already delivered the behaviour -- it says this child's "
                "cases do not discriminate, not that the gate is broken")
        return
    chk.add("PASS", "L5 the contract refused HEAD",
            f"all {len(cases)} case(s) red at HEAD and green in the tree")


# --------------------------------------------------------------------------
# the run
# --------------------------------------------------------------------------
def _wi_checks(args: argparse.Namespace, *, require_clean: bool, run_tests: bool):
    chk = Check()
    repo = leg.repo_root()
    root = e2e_mod.e2e_root(repo, args.project)
    src = leg.leg_root(repo, args.project, PHASE)

    leg.p1_work_item(chk, repo, args.wi)
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
    leg.c0_scope(chk, repo, src, dirty, PHASE)

    cfg = unit_mod.UnitConfig(repo, args.project)
    if cfg.problem:
        chk.add("FAIL", "C1 unit commands declared", cfg.problem)
    else:
        chk.add("PASS", "C1 unit commands declared",
                f"build `{cfg.build}`\ntest  `{cfg.test}`")

    recorded, why = recorded_red(repo, args.wi)
    if why:
        chk.add("FAIL", "C2 the recorded red", why)
    else:
        chk.add("PASS", "C2 the recorded red",
                f"the unit phase recorded {len(recorded)} failing test(s): "
                f"{', '.join(recorded)}")

    cases = leg.contract_set(repo, root, args.wi, "e2e")

    if not run_tests:
        return chk, repo, root, dirty, cases

    # Named even when they do not run, so the report never omits a row a later
    # run will have. A silent absence and a green read the same in a summary.
    rows = ("L1 build is green", "L2 recorded tests green",
            "L3 the suite is whole", "L4 the contract accepts the code",
            "L5 the contract refused HEAD")
    if chk.failed:
        for name in rows:
            chk.add("PENDING", name,
                    "not run: a FAIL above means anything measured here would "
                    "describe something other than this phase's change")
        return chk, repo, root, dirty, cases

    if not l1_build_is_green(chk, repo, cfg):
        for name in rows[1:]:
            chk.add("PENDING", name,
                    "not run: nothing compiled, so there are no test names to "
                    "read and no product to run the cases against")
        return chk, repo, root, dirty, cases

    l2_l3_tests(chk, repo, cfg, recorded)
    l4_l5_contract(chk, repo, root, cases)
    return chk, repo, root, dirty, cases


# --------------------------------------------------------------------------
# verbs
# --------------------------------------------------------------------------
def cmd_start(args: argparse.Namespace) -> int:
    chk, repo, root, _dirty, _cases = _wi_checks(
        args, require_clean=True, run_tests=False)
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

    inv = e2e_mod.inventory(root)
    print("## The contract\n")
    for case_id in leg.contract_set(repo, root, args.wi, "e2e"):
        command = str(inv.get(case_id, {}).get("command") or "").strip()
        print(f"- `{case_id}`" + (f" -- `{command}`" if command else ""))
    print()

    recorded, why = recorded_red(repo, args.wi)
    print("## The invariants\n")
    if why:
        print(f"(unavailable: {why.splitlines()[0]})")
    else:
        for name in recorded:
            print(f"- `{name}`")
    print()
    print("## The tests, as they landed\n")
    for path in leg.landed_paths(repo, args.wi, "unit"):
        print(f"- {path}")
    print()
    print("=" * 78)
    print("Write the implementation under")
    print(f"  apps/{args.project}/src/")
    print("until every case and every test above is green.")
    print()
    print("Do not edit a colocated test file. Not to fix a test, not to rename")
    print("one, not to run a formatter over it -- including when the diff is")
    print("whitespace only. Those files are what this phase is measured against,")
    print("and editing one here is the retrofit the phase split exists to refuse.")
    print()
    print("If the implementation cannot satisfy both oracles without changing")
    print("one of them, say so and stop. That is a defect in a phase that is")
    print("closed, not something to route around here.")
    print(f"\nnext.command: {leg.phase_command(PHASE, args.project, 'verify', args.wi)}")
    return 0


def cmd_verify(args: argparse.Namespace) -> int:
    chk, _repo, _root, _dirty, cases = _wi_checks(
        args, require_clean=False, run_tests=False)
    print(f"mechanical admissibility: #{args.wi}")
    chk.report()
    print()
    print("These checks say the change is ADMISSIBLE -- it is an implementation")
    print("change, it touches no test file, it comes after both oracles, and")
    print("both are resolvable. They do not say it works: nothing was run.")
    if chk.failed:
        print("\nnext.command: fix the FAIL rows above, then re-run this verb")
        return 1
    print(f"\n{len(cases)} case(s) and the declared suite will decide it.")
    print(f"next.command: {leg.phase_command(PHASE, args.project, 'test', args.wi)}")
    return 0


def cmd_test(args: argparse.Namespace) -> int:
    chk, _repo, _root, _dirty, cases = _wi_checks(
        args, require_clean=False, run_tests=True)
    print(f"both oracles, in the tree and at HEAD: #{args.wi}")
    chk.report()
    if chk.failed:
        print("\nnext.command: fix the FAIL rows above, then re-run this verb")
        return 1
    print(f"\nthe contract refused HEAD and accepts the tree "
          f"({', '.join(cases)}), and the suite is whole.")
    print(f"next.command: {leg.phase_command(PHASE, args.project, 'commit', args.wi)}")
    return 0


def cmd_commit(args: argparse.Namespace) -> int:
    chk, repo, _root, dirty, cases = _wi_checks(
        args, require_clean=False, run_tests=True)
    print(f"commit gate: #{args.wi}")
    chk.report()
    if chk.failed:
        print("\nnothing was committed; the tree is unchanged and the work is still here.")
        print("next.command: fix the FAIL rows above, then re-run this verb")
        return 1

    digest = leg.change_digest(repo, args.wi, dirty)
    trailers = [
        f"Logic-Contract: {', '.join(cases)}",
        f"Logic-Change-Digest: {digest}",
    ]
    message = (f"{PHASE}(wi-{args.wi}): implement what both oracles fixed\n"
               f"\nRefs #{args.wi}\n"
               + "\n" + "\n".join(trailers) + "\n")

    if args.dry_run:
        print("\n-- would commit, exactly these paths ------------------------")
        for p in dirty:
            print(f"  {p}")
        print("-- message -------------------------------------------------")
        print(message)
        return 0

    add = git(repo, "add", "--", *dirty)
    if add.returncode != 0:
        print(add.stderr)
        return add.returncode
    proc = git(repo, "commit", "-m", message, "--", *dirty)
    print(proc.stdout or proc.stderr)
    if proc.returncode != 0:
        return proc.returncode

    sha = git(repo, "rev-parse", "HEAD").stdout.strip()
    print(f"Logic-Commit: {sha}")
    print(f"\nnext.command: change.py lifecycle {args.wi} --leg {PHASE} "
          f"--commit {sha} --digest {digest}")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="logic.py", description=__doc__.splitlines()[0])
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
                       help="the mechanical list over the whole change; runs nothing")
    p.set_defaults(func=cmd_verify)

    p = sub.add_parser("test", parents=[wi],
                       help="both oracles, in the tree and at HEAD")
    p.set_defaults(func=cmd_test)

    p = sub.add_parser("commit", parents=[wi],
                       help="re-run everything and commit the change")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_commit)

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
