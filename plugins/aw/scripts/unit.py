#!/usr/bin/env python3
"""The invariant phase of a change work item.

The e2e cases landed and are red. This phase writes the tests that live inside
the implementation tree -- the ones that can see what the product does not
expose -- and it writes them while there is still nothing for them to pass
against.

There was no phase here before. The deleted `cb.py` *ran* a project's colocated
invariants as its `B3` row, but it only ever asked whether they were green, so writing the
implementation first and adding a test that describes it afterwards passed
identically. That row had no counterpart to the red-at-HEAD measurement one
layer up, and this phase is that counterpart.

Two things make its red mean something:

  U1  the declared build command exits zero.

      A unit test naming a function nobody has written yet does not fail. It
      fails to *compile*, and `cargo test` exits non-zero for that exactly as
      it does for a failed assertion. A phase reading one exit code cannot tell
      the two apart, and the difference is the whole phase: a compile error is
      a red over a test that never ran.

      That is why `[unit] build` and `[unit] test` are two declared commands
      and not one, and why the phase's required output includes the `todo!()`
      skeleton. The skeleton is not a concession to the compiler. It is what
      moves the failure from build time to run time, where it is attributable.

  U2  at least one test is failing in the tree that was not failing at HEAD.

      By name, never by exit code. A selector matching nothing exits zero
      having run nothing and reads as green; a suite failing for an unrelated
      reason exits non-zero and reads as red. Neither says anything about this
      change, and only a set difference over names does.

  U3  the e2e cases are still red.

      A phase that wrote the implementation as well as the test would satisfy
      `U1` and `U2` perfectly -- the crate builds, and the test is red only if
      the test is wrong. This is the row that sees it, because the one thing a
      finished implementation cannot do is leave the e2e cases refusing it.

Work-item scoped -- this is the gate:

  start <iid>     open the phase; refuses a dirty tree and a missing e2e phase
  verify <iid>    the mechanical list over the whole change; runs nothing
  test <iid>      build, the named red, and the e2e cases still refusing
  commit <iid>    re-run everything and commit the diff
"""
from __future__ import annotations

import argparse
import importlib.util
import re
import subprocess
import sys
from pathlib import Path

if sys.version_info < (3, 11):
    sys.stderr.write(
        f"unit.py reads `[unit]` from a project aw.toml, which needs Python "
        f"3.11+ for tomllib; this is {sys.version.split()[0]}.\n"
        "Invoke it as: uv run --python 3.13 --no-project <path>/unit.py ...\n"
    )
    raise SystemExit(2)

import tomllib  # noqa: E402

if "legmod" in sys.modules:
    leg = sys.modules["legmod"]
else:
    _LEG_SPEC = importlib.util.spec_from_file_location(
        "legmod", Path(__file__).resolve().parent / "leg.py")
    leg = importlib.util.module_from_spec(_LEG_SPEC)
    sys.modules["legmod"] = leg
    _LEG_SPEC.loader.exec_module(leg)

# The e2e root, the inventory and the case ids belong to `e2e.py`, which is the
# phase that produced them.
e2e_mod = leg.sibling("e2e", "e2emod")

Check = leg.Check
GIT = leg.GIT
PHASE = "unit"

# How to read test names out of a test runner's output. A closed table, not a
# guess: a parser that fell back to "any line containing FAILED" would pick up
# a summary line, a path, and the word inside someone's assertion message, and
# it would do it silently.
#
# `cargo` prints one `test <name> ... <outcome>` line per test on stdout,
# per test binary. `ignored` is deliberately neither passed nor failed: a test
# that did not run is not evidence in either direction.
HARNESSES = {
    "cargo": {
        "failed": re.compile(r"^test (\S+) \.\.\. FAILED$", re.M),
        "passed": re.compile(r"^test (\S+) \.\.\. ok$", re.M),
    },
}


def git(repo: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run([*GIT, *args], cwd=repo, capture_output=True, text=True)


# --------------------------------------------------------------------------
# what the project declared
# --------------------------------------------------------------------------
class UnitConfig:
    """`[unit]` from `apps/<project>/aw.toml`, or the reason it is unusable.

    Missing and empty are different answers, and neither is defaulted. A
    project that never declared a build command has stated nothing, and
    defaulting that to "the test command is also the build check" would hand
    every unconfigured project the weaker gate -- the exact one `U1` exists to
    close.
    """

    def __init__(self, repo: Path, project: str) -> None:
        self.problem = ""
        self.build = ""
        self.test = ""
        self.harness = ""
        path = repo / "apps" / project / "aw.toml"
        if not path.is_file():
            self.problem = f"no project config at {path.relative_to(repo)}"
            return
        section = tomllib.loads(path.read_text(encoding="utf-8")).get("unit")
        if not isinstance(section, dict):
            self.problem = (
                f"{path.relative_to(repo)} declares no `[unit]` section\n"
                "this phase needs three keys, and there is no default for any "
                "of them:\n"
                "  build   = \"...\"   must exit 0 before a red counts\n"
                "  test    = \"...\"   the names come out of its output\n"
                "  harness = \"cargo\" which parser reads those names")
            return
        for key in ("build", "test", "harness"):
            value = str(section.get(key, "")).strip()
            if not value:
                self.problem = (f"`[unit]` in {path.relative_to(repo)} declares "
                                f"no `{key}`")
                return
            setattr(self, key, value)
        if self.harness not in HARNESSES:
            self.problem = (
                f"`[unit] harness = \"{self.harness}\"` names no known parser\n"
                f"known: {', '.join(sorted(HARNESSES))}")

    def names(self, output: str, outcome: str) -> set[str]:
        return set(HARNESSES[self.harness][outcome].findall(output))


# --------------------------------------------------------------------------
# the checks this phase adds
# --------------------------------------------------------------------------
def u1_build_is_green(chk: Check, repo: Path, cfg: UnitConfig) -> bool:
    """The tests compile, so a failure from here on is a failure of a test.

    Returns whether the ladder may continue. It is the one check here that
    gates the others rather than merely reporting: a name set computed from a
    build that produced no binary is a set of names for tests that do not
    exist.
    """
    result = leg.run_command(repo, cfg.build)
    if result["exit"] != 0:
        tail = (result["stderr"] or result["stdout"]).strip().splitlines()
        chk.add("FAIL", "U1 build is green",
                f"`{cfg.build}` exited {result['exit']}\n"
                + "\n".join(f"  {line}" for line in tail[-15:])
                + "\nthe tests did not run. A build failure and a failed "
                "assertion are the same exit code and different evidence: "
                "nothing here says anything about what the tests would have "
                "observed, so there is no red to record.\n"
                "write the `todo!()` skeleton the tests need to compile -- that "
                "skeleton is this phase's output, not a workaround")
        return False
    chk.add("PASS", "U1 build is green", f"`{cfg.build}` exited 0")
    return True


def u2_named_red(chk: Check, repo: Path, cfg: UnitConfig) -> list[str]:
    """Some test fails in the tree that was not failing at HEAD.

    The subtraction is the point. A project carrying a test that was already
    red would otherwise let this phase record a red it did not cause, and the
    phase after would then be measured against someone else's failure.
    """
    tree = leg.run_command(repo, cfg.test)
    failing = cfg.names(tree["stdout"], "failed")

    with leg.at_head(repo) as (head, error):
        if head is None:
            chk.add("FAIL", "U2 named red",
                    "could not check out HEAD to subtract its failures:\n" + error)
            return []
        before = cfg.names(leg.run_command(head, cfg.test)["stdout"], "failed")

    new = sorted(failing - before)
    if not new:
        detail = (f"`{cfg.test}` reports no failure this change introduced\n"
                  f"  failing in the tree: {', '.join(sorted(failing)) or '(none)'}\n"
                  f"  failing at HEAD:     {', '.join(sorted(before)) or '(none)'}")
        if not failing:
            detail += ("\nan exit code is not the measurement: a selector "
                       "matching nothing exits zero having run nothing, and "
                       "reads exactly like a suite that passed")
        else:
            detail += ("\nevery failure here was already failing before this "
                       "change, so none of them is evidence about it")
        chk.add("FAIL", "U2 named red", detail)
        return []
    chk.add("PASS", "U2 named red",
            f"{len(new)} test(s) newly failing: {', '.join(new)}")
    return new


def u3_contract_still_red(chk: Check, repo: Path, root: Path,
                          cases: list[str]) -> None:
    """The e2e cases have not been satisfied by this phase.

    `U1` and `U2` cannot see a phase that wrote the implementation alongside
    the test: the crate builds, and the test is red only if the test is wrong.
    What such a phase cannot do is leave the e2e cases refusing the product,
    and that is what this reads.
    """
    if not cases:
        chk.add("FAIL", "U3 contract still red",
                f"the {PHASE} phase's predecessor landed no case, so there is "
                "nothing here that could still be refusing the product")
        return
    inv = e2e_mod.inventory(root)
    green: list[str] = []
    for case_id in cases:
        entry = inv.get(case_id)
        if entry is None:
            chk.add("FAIL", "U3 contract still red",
                    f"`{case_id}` was committed by the e2e phase but is not in "
                    f"{(root / 'pyproject.toml').relative_to(repo)}\n"
                    "the inventory was edited after that phase closed")
            return
        if leg.run_command(repo, entry["command"])["exit"] == 0:
            green.append(f"  {case_id}")
    if green:
        chk.add("FAIL", "U3 contract still red",
                "the e2e cases now accept the product:\n" + "\n".join(green)
                + f"\nthis phase writes tests and the `todo!()` skeleton they "
                "need to compile -- nothing else. A case going green here means "
                "the implementation was written in the phase whose job was to "
                "produce the thing that would refuse it.")
        return
    chk.add("PASS", "U3 contract still red",
            f"all {len(cases)} e2e case(s) still refuse the product")


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
        return chk, repo, root, None, [], []

    dirty = leg.dirty_set(repo)
    if require_clean:
        leg.p2_clean_tree(chk, dirty)
        leg.p3_leg_is_open(chk, repo, args.wi, PHASE)
        leg.p4_predecessor_landed(chk, repo, args.wi, PHASE)
        return chk, repo, root, None, dirty, []

    leg.p3_leg_is_open(chk, repo, args.wi, PHASE)
    leg.p4_predecessor_landed(chk, repo, args.wi, PHASE)
    leg.c0_scope(chk, repo, src, dirty, PHASE)

    cfg = UnitConfig(repo, args.project)
    if cfg.problem:
        chk.add("FAIL", "C1 unit commands declared", cfg.problem)
    else:
        chk.add("PASS", "C1 unit commands declared",
                f"build `{cfg.build}`\ntest  `{cfg.test}`\n"
                f"names read by the `{cfg.harness}` parser")

    cases = leg.contract_set(repo, root, args.wi, "e2e")
    if not run_tests:
        return chk, repo, root, cfg, dirty, []

    # Named even when they do not run, so the report never omits a row a later
    # run will have. A silent absence and a green read the same in a summary.
    if chk.failed:
        for name in ("U1 build is green", "U2 named red", "U3 contract still red"):
            chk.add("PENDING", name,
                    "not run: a FAIL above means anything measured here would "
                    "describe something other than this phase's change")
        return chk, repo, root, cfg, dirty, []

    if not u1_build_is_green(chk, repo, cfg):
        for name in ("U2 named red", "U3 contract still red"):
            chk.add("PENDING", name,
                    "not run: the tests did not compile, so there are no test "
                    "names to read and no product to run the cases against")
        return chk, repo, root, cfg, dirty, []

    red = u2_named_red(chk, repo, cfg)
    u3_contract_still_red(chk, repo, root, cases)
    return chk, repo, root, cfg, dirty, red


# --------------------------------------------------------------------------
# verbs
# --------------------------------------------------------------------------
def cmd_start(args: argparse.Namespace) -> int:
    chk, repo, root, _cfg, _dirty, _red = _wi_checks(
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
    for heading in ("Goal", "Acceptance"):
        section = mod.section_at(body, 2, heading)
        print(f"## {heading}\n")
        print((section or "(this work item has no such section)").strip())
        print()

    inv = e2e_mod.inventory(root)
    print("## The cases this has to stay red against\n")
    for case_id in leg.contract_set(repo, root, args.wi, "e2e"):
        promise = str(inv.get(case_id, {}).get("promise") or "").strip()
        print(f"- `{case_id}`" + (f" -- {promise}" if promise else ""))
    print()
    print("=" * 78)
    print("Write the colocated tests in")
    for name in leg.TEST_FILES:
        print(f"  apps/{args.project}/src/**/{name}")
    print("wired in with `#[cfg(test)] mod tests;`. An inline")
    print("`#[cfg(test)] mod tests { ... }` is invisible to the scope check that")
    print("keeps the next phase off your tests, so it is refused here.")
    print()
    print("Write the `todo!()` skeleton the tests need to compile, and nothing")
    print("more. The skeleton is required output: without it the tests fail to")
    print("build, and a build failure is a red over a test that never ran.")
    print()
    print("The e2e cases above must still be red when you are done. If they go")
    print("green, the implementation was written in the phase whose only job was")
    print("to produce the thing that would refuse it.")
    print(f"\nnext.command: {leg.phase_command(PHASE, args.project, 'verify', args.wi)}")
    return 0


def cmd_verify(args: argparse.Namespace) -> int:
    chk, _repo, _root, _cfg, _dirty, _red = _wi_checks(
        args, require_clean=False, run_tests=False)
    print(f"mechanical admissibility: #{args.wi}")
    chk.report()
    print()
    print("These checks say the change is ADMISSIBLE -- it is a test-and-skeleton")
    print("change under src/, it comes after a landed contract, and the project")
    print("declared how to build and run its tests. Nothing was built or run.")
    if chk.failed:
        print("\nnext.command: fix the FAIL rows above, then re-run this verb")
        return 1
    print(f"\nnext.command: {leg.phase_command(PHASE, args.project, 'test', args.wi)}")
    return 0


def cmd_test(args: argparse.Namespace) -> int:
    chk, _repo, _root, _cfg, _dirty, red = _wi_checks(
        args, require_clean=False, run_tests=True)
    print(f"the build, the named red, and the contract: #{args.wi}")
    chk.report()
    if chk.failed:
        print("\nnext.command: fix the FAIL rows above, then re-run this verb")
        return 1
    print(f"\nthe tests compile and {len(red)} of them refuse the skeleton: "
          f"{', '.join(red)}")
    print(f"next.command: {leg.phase_command(PHASE, args.project, 'commit', args.wi)}")
    return 0


def cmd_commit(args: argparse.Namespace) -> int:
    chk, repo, _root, _cfg, dirty, red = _wi_checks(
        args, require_clean=False, run_tests=True)
    print(f"commit gate: #{args.wi}")
    chk.report()
    if chk.failed:
        print("\nnothing was committed; the tree is unchanged and the work is still here.")
        print("next.command: fix the FAIL rows above, then re-run this verb")
        return 1

    digest = leg.change_digest(repo, args.wi, dirty)
    trailers = [
        # The names travel with the commit that produced them. A phase rebased
        # away takes its evidence with it, which a state file recording the
        # same set would not -- it would go on asserting a red for a commit
        # that is no longer in the history.
        f"Unit-Red: {', '.join(red)}",
        f"Unit-Change-Digest: {digest}",
    ]
    message = (f"{PHASE}(wi-{args.wi}): pin the invariant the contract cannot see\n"
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
    print(f"Unit-Commit: {sha}")
    print(f"\nnext.command: change.py lifecycle {args.wi} --leg {PHASE} "
          f"--commit {sha} --digest {digest}")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="unit.py", description=__doc__.splitlines()[0])
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
                       help="build, the named red, and the e2e cases still refusing")
    p.set_defaults(func=cmd_test)

    p = sub.add_parser("commit", parents=[wi],
                       help="re-run everything and commit the change")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_commit)

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
