#!/usr/bin/env python3
"""The implementation phase of a change work item.

The e2e cases landed and are red. This phase writes the colocated tests -- the
ones that can see what the product does not expose -- and the implementation
that satisfies them. Both go into `src/`.

Why one phase and not two
-------------------------
There were two until 2026-08-27: `unit` wrote the tests plus a `todo!()`
skeleton, `logic` wrote the implementation, and `C0` kept them apart by
filename. That split bought exactly one thing -- a named red, measured before
anything could satisfy it -- and it charged for it in a currency Rust does not
have. A colocated test and the code under it are the same tree; they are
written together, and every iteration after the first had to pretend to be the
first.

So the split is gone and the thing it bought is kept, by moving the
measurement off the filename and onto a verb:

    red   records which tests fail *now*, by name, and did not fail at HEAD

`red` is the load-bearing part of this phase. It passes only at a moment when a
named test fails that was not failing at HEAD -- which is a moment the
implementation is not finished. Write the implementation first and there is no
failing test to record, and no red can be manufactured afterwards. The record
accumulates across runs, so the ordinary TDD loop -- write a test, watch it
fail, satisfy it, write the next -- lands every one of its reds and not only
the last.

The rows
--------
`red` measures, and writes the record:

  R1  the declared build command exits zero.

      A test naming a function nobody has written yet does not fail. It fails
      to *compile*, and `cargo test` exits non-zero for that exactly as it does
      for a failed assertion. One exit code cannot tell them apart, and the
      difference is the whole measurement: a compile error is a red over a test
      that never ran.

  R2  a test is failing here that was not failing at HEAD.

      By name, never by exit code. A selector matching nothing exits zero
      having run nothing and reads as green; a suite failing for an unrelated
      reason exits non-zero and reads as red. Neither says anything about this
      change, and only a set difference over names does.

  R3  the e2e cases are still red -- on the run that opens the record.

      `R1` and `R2` cannot see a tree where the implementation was written
      alongside the test: it builds, and the test is red only if the test is
      wrong. What such a tree cannot do is leave the e2e cases refusing the
      product. On a later `red` -- extending a record that already holds names
      measured before any implementation existed -- this row does not apply,
      and says so rather than passing quietly.

`test` reads the record back and decides the phase:

  T1  it compiles.

  T2  every recorded name is present and passing.

      Present matters as much as passing: a test that was deleted is not
      failing, and a run that only counted failures would read its absence as
      success.

  T3  nothing else broke, and nothing else vanished.

      The whole declared suite is green, and every test that ran at HEAD still
      runs here. `T2` is scoped to the names this work item produced, so on its
      own it would let the implementation satisfy them at the cost of a test
      somebody else wrote -- and the vanishing half is invisible to every other
      row, because a test that no longer runs cannot fail.

  T4  the e2e cases accept the implementation.

  T5  the same cases, run against `HEAD`, refuse it.

      `T4` alone cannot tell an implementation that made the cases pass from
      cases that were already passing -- and the second is what a case
      measuring nothing looks like at every phase it is ever run. The `e2e`
      phase proved them red before any implementation existed; this proves them
      red at the commit immediately underneath the change, which is the only
      point at which the difference is attributable to *this* work item.

`C2` sits between the two, and is what stops the record being laundered. It
holds the sha256 of every test file at the moment the red was measured. Weaken
a test after recording it and `C2` refuses until `red` is run again -- and
`red` over a weakened test finds no failing name, because the implementation is
already there. That is the `logic`-may-not-touch-`tests.rs` rule, restated as a
measurement instead of a filename.

The five verbs, in order -- this is the gate:

  start <iid>     open the phase; refuses a dirty tree and a missing e2e phase
  red <iid>       build, and record the named failures against HEAD
  verify <iid>    the mechanical list over the whole change; runs nothing
  test <iid>      the recorded names, the suite, and both sides of the contract
  commit <iid>    re-run everything and commit the diff
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import re
import subprocess
import sys
from pathlib import Path

if sys.version_info < (3, 11):
    sys.stderr.write(
        f"impl.py reads `[impl]` from a project aw.toml, which needs Python "
        f"3.11+ for tomllib; this is {sys.version.split()[0]}.\n"
        "Invoke it as: uv run --python 3.13 --no-project <path>/impl.py ...\n"
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
# phase that produced them. Reading them from their owner is what keeps this
# phase from holding a second opinion about what the contract is -- a local copy
# would let the phase that has to satisfy the oracle also decide what it says.
e2e_mod = leg.sibling("e2e", "e2emod")

Check = leg.Check
GIT = leg.GIT
PHASE = "impl"

# How to read test names out of a test runner's output. A closed table, not a
# guess: a parser that fell back to "any line containing FAILED" would pick up
# a summary line, a path, and the word inside someone's assertion message, and
# it would do it silently.
#
# `cargo` prints one `test <name> ... <outcome>` line per test on stdout, per
# test binary. `ignored` is deliberately neither passed nor failed: a test that
# did not run is not evidence in either direction.
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
class ImplConfig:
    """`[impl]` from `apps/<project>/aw.toml`, or the reason it is unusable.

    Missing and empty are different answers, and neither is defaulted. A
    project that never declared a build command has stated nothing, and
    defaulting that to "the test command is also the build check" would hand
    every unconfigured project the weaker gate -- the exact one `R1` exists to
    close.

    The section is `[impl]` and not `[unit]`. That is a rename with no
    migration behind it: at the changeover `grep -c '^\\[unit\\]' apps/*/aw.toml`
    was 0 in all 21 projects, so the section the retired phase read had never
    once been declared, and there is no body of configuration to carry forward.
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
        section = tomllib.loads(path.read_text(encoding="utf-8")).get("impl")
        if not isinstance(section, dict):
            self.problem = (
                f"{path.relative_to(repo)} declares no `[impl]` section\n"
                "this phase needs three keys, and there is no default for any "
                "of them:\n"
                "  build   = \"...\"   must exit 0 before a red counts\n"
                "  test    = \"...\"   the names come out of its output\n"
                "  harness = \"cargo\" which parser reads those names")
            return
        for key in ("build", "test", "harness"):
            value = str(section.get(key, "")).strip()
            if not value:
                self.problem = (f"`[impl]` in {path.relative_to(repo)} declares "
                                f"no `{key}`")
                return
            setattr(self, key, value)
        if self.harness not in HARNESSES:
            self.problem = (
                f"`[impl] harness = \"{self.harness}\"` names no known parser\n"
                f"known: {', '.join(sorted(HARNESSES))}")

    def names(self, output: str, outcome: str) -> set[str]:
        return set(HARNESSES[self.harness][outcome].findall(output))


# --------------------------------------------------------------------------
# the record
# --------------------------------------------------------------------------
# Where `red` writes what it measured. Under `.aw/`, which `.gitignore` names,
# so the record never appears in `dirty_set` and can never be mistaken for part
# of the change it describes.
#
# A state file rather than a commit trailer, because there is no commit to hang
# it on: the phase it belongs to is the phase still being written. What made a
# trailer the right answer for the retired `unit` phase -- a phase rebased away
# takes its evidence with it -- is preserved by pinning `head` below, and by
# `commit` deleting the record once it has copied the names onto the commit.
RECORD_DIR = Path(".aw") / "impl-red"


def record_path(repo: Path, iid: int) -> Path:
    return repo / RECORD_DIR / f"{iid}.json"


def test_file_digests(repo: Path, dirty: list[str]) -> dict[str, str]:
    """sha256 of every colocated test file in the change.

    The half of the retired `logic` phase's filename gate that had no
    substitute: it refused an edit to a test file after the red was measured.
    Here the edit is allowed and *detected* -- `C2` sends you back to `red`, and
    `red` over a weakened test finds nothing failing.
    """
    out: dict[str, str] = {}
    for rel in sorted(dirty):
        if not leg.is_test_file(rel):
            continue
        target = repo / rel
        out[rel] = (hashlib.sha256(target.read_bytes()).hexdigest()
                    if target.is_file() else "(absent)")
    return out


def head_sha(repo: Path) -> str:
    return git(repo, "rev-parse", "HEAD").stdout.strip()


def load_record(repo: Path, iid: int) -> dict | None:
    path = record_path(repo, iid)
    if not path.is_file():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (ValueError, OSError):
        return None
    return data if isinstance(data, dict) else None


def save_record(repo: Path, iid: int, record: dict) -> Path:
    path = record_path(repo, iid)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n",
                    encoding="utf-8")
    return path


# --------------------------------------------------------------------------
# the rows `red` measures
# --------------------------------------------------------------------------
def r1_build_is_green(chk: Check, repo: Path, cfg: ImplConfig) -> bool:
    """The tests compile, so a failure from here on is the failure of a test.

    Returns whether the run may continue. It is the one check here that gates
    the others rather than merely reporting: a name set computed from a build
    that produced no binary is a set of names for tests that do not exist.
    """
    result = leg.run_command(repo, cfg.build)
    if result["exit"] != 0:
        tail = (result["stderr"] or result["stdout"]).strip().splitlines()
        chk.add("FAIL", "R1 build is green",
                f"`{cfg.build}` exited {result['exit']}\n"
                + "\n".join(f"  {line}" for line in tail[-15:])
                + "\nthe tests did not run. A build failure and a failed "
                "assertion are the same exit code and different evidence: "
                "nothing here says anything about what the tests would have "
                "observed, so there is no red to record.\n"
                "write whatever signature the tests need to compile against -- "
                "a `todo!()` body is enough, and it is what moves the failure "
                "from build time to run time, where it is attributable")
        return False
    chk.add("PASS", "R1 build is green", f"`{cfg.build}` exited 0")
    return True


def r2_named_red(chk: Check, repo: Path, cfg: ImplConfig) -> list[str]:
    """Some test fails in the tree that was not failing at HEAD.

    The subtraction is the point. A project carrying a test that was already
    red would otherwise let this record a red it did not cause, and `test`
    would then be measured against somebody else's failure.
    """
    tree = leg.run_command(repo, cfg.test)
    failing = cfg.names(tree["stdout"], "failed")

    with leg.at_head(repo) as (head, error):
        if head is None:
            chk.add("FAIL", "R2 named red",
                    "could not check out HEAD to subtract its failures:\n" + error)
            return []
        at_head = leg.run_command(head, cfg.test)
        if at_head["unrunnable"]:
            chk.add("FAIL", "R2 named red", at_head["unrunnable"]
                    + "\nHEAD's failures are subtracted from the tree's, so a "
                    "run that produced no output would report every failure "
                    "here as one this phase caused")
            return []
        before = cfg.names(at_head["stdout"], "failed")

    new = sorted(failing - before)
    if not new:
        detail = (f"`{cfg.test}` reports no failure this change introduced\n"
                  f"  failing in the tree: {', '.join(sorted(failing)) or '(none)'}\n"
                  f"  failing at HEAD:     {', '.join(sorted(before)) or '(none)'}")
        if not failing:
            detail += ("\nan exit code is not the measurement: a selector "
                       "matching nothing exits zero having run nothing, and "
                       "reads exactly like a suite that passed\n"
                       "if the implementation is already written, that is the "
                       "defect this row exists to name -- there is no moment "
                       "left at which a red could be attributed to it, and no "
                       "way to manufacture one after the fact")
        else:
            detail += ("\nevery failure here was already failing before this "
                       "change, so none of them is evidence about it")
        chk.add("FAIL", "R2 named red", detail)
        return []
    chk.add("PASS", "R2 named red",
            f"{len(new)} test(s) newly failing: {', '.join(new)}")
    return new


def r3_contract_still_red(chk: Check, repo: Path, root: Path,
                          cases: list[str], opening: bool) -> bool:
    """The e2e cases have not been satisfied at the moment the record opens.

    `R1` and `R2` cannot see a tree where the implementation was written
    alongside the test: it builds, and the test is red only if the test is
    wrong. What such a tree cannot do is leave the e2e cases refusing the
    product, and that is what this reads.

    Only on the run that opens the record. Once names measured before any
    implementation existed are on file, a case going green is this phase
    working -- so on a later `red` the row states that rather than passing
    quietly, because a row that reports PASS for two different reasons reports
    nothing.
    """
    if not opening:
        chk.add("PENDING", "R3 contract still red",
                "not run: the record already holds names measured before any "
                "implementation existed, and the cases going green from here "
                "is what this phase is for")
        return True
    if not cases:
        chk.add("FAIL", "R3 contract still red",
                "the e2e phase landed no case, so there is nothing here that "
                "could still be refusing the product")
        return False
    inv = e2e_mod.inventory(root)
    green: list[str] = []
    for case_id in cases:
        entry = inv.get(case_id)
        if entry is None:
            chk.add("FAIL", "R3 contract still red",
                    f"`{case_id}` was committed by the e2e phase and is not in "
                    f"{e2e_mod.manifest(root).relative_to(repo)}\n"
                    "the inventory was edited after that phase closed")
            return False
        result = leg.run_command(repo, entry["command"])
        if result["unrunnable"]:
            chk.add("FAIL", "R3 contract still red", result["unrunnable"]
                    + "\na case that could not be started did not refuse "
                    "anything, and this row reads a non-zero exit as the answer "
                    "it wants")
            return False
        if result["exit"] == 0:
            green.append(f"  {case_id}")
    if green:
        chk.add("FAIL", "R3 contract still red",
                "the e2e cases already accept the product:\n" + "\n".join(green)
                + "\nthis is the run that opens the record, so there is nothing "
                "on file yet that was measured before an implementation "
                "existed. A case green here means either the implementation was "
                "written before any red was recorded -- in which case no red "
                "can be attributed to it now -- or the case observes nothing.")
        return False
    chk.add("PASS", "R3 contract still red",
            f"all {len(cases)} e2e case(s) still refuse the product")
    return True


# --------------------------------------------------------------------------
# the rows `test` measures
# --------------------------------------------------------------------------
def t1_build_is_green(chk: Check, repo: Path, cfg: ImplConfig) -> bool:
    """The implementation compiles, so a failure below is a failed assertion.

    Gates the rest rather than merely reporting, for the same reason `R1` does
    -- but the remediation is the opposite one, which is why this is its own row
    and not a call into `R1`. There the answer is "write the signature the tests
    need"; here it exists, and the message must not send anybody back to it.
    """
    result = leg.run_command(repo, cfg.build)
    if result["exit"] != 0:
        tail = (result["stderr"] or result["stdout"]).strip().splitlines()
        chk.add("FAIL", "T1 build is green",
                f"`{cfg.build}` exited {result['exit']}\n"
                + "\n".join(f"  {line}" for line in tail[-15:])
                + "\nnothing ran, so nothing below this row observed anything. "
                "The tests and the cases are fixed; this is the implementation "
                "not compiling against them.")
        return False
    chk.add("PASS", "T1 build is green", f"`{cfg.build}` exited 0")
    return True


def t2_t3_tests(chk: Check, repo: Path, cfg: ImplConfig,
                recorded: list[str]) -> None:
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
            detail += ("recorded as red by `red`, and not run here at all:\n"
                       + "\n".join(f"  {n}" for n in missing)
                       + "\na test that is absent is not a test that passed; "
                       "either it was deleted or it was renamed after it was "
                       "measured\n")
        chk.add("FAIL", "T2 recorded tests green", detail.rstrip())
    else:
        chk.add("PASS", "T2 recorded tests green",
                f"all {len(recorded)} recorded test(s) ran and passed: "
                f"{', '.join(recorded)}")

    other = sorted(failed - set(recorded))
    if other:
        chk.add("FAIL", "T3 the suite is whole",
                "failing, and not part of what this work item recorded:\n"
                + "\n".join(f"  {n}" for n in other)
                + "\nthe recorded names are this change's oracle, not its "
                "permission slip -- a green there bought by a red anywhere else "
                "is a regression this phase caused")
        return

    with leg.at_head(repo) as (head, error):
        if head is None:
            chk.add("FAIL", "T3 the suite is whole",
                    "could not check out HEAD to compare the test population:\n"
                    + error)
            return
        at_head = leg.run_command(head, cfg.test)
        if at_head["unrunnable"]:
            chk.add("FAIL", "T3 the suite is whole", at_head["unrunnable"]
                    + "\nthis row is a comparison against the tests that ran at "
                    "HEAD, and a run that produced no output would report an "
                    "empty population as a whole one")
            return
        before = cfg.names(at_head["stdout"], "passed")

    lost = sorted(before - passed - failed)
    if lost:
        chk.add("FAIL", "T3 the suite is whole",
                "passing at HEAD and not present here at all:\n"
                + "\n".join(f"  {n}" for n in lost)
                + "\na deleted test cannot fail, so nothing else in this phase "
                "can see it go; the implementation is being measured against a "
                "smaller suite than the one it inherited")
        return
    chk.add("PASS", "T3 the suite is whole",
            f"{len(passed)} passing, none failing, and all {len(before)} "
            "test(s) that ran at HEAD still run here")


def t4_t5_contract(chk: Check, repo: Path, root: Path, cases: list[str]) -> None:
    """Both contract rows: the tree accepts, and HEAD refuses."""
    if not cases:
        chk.add("FAIL", "T4 the contract accepts the code",
                "the e2e phase landed no case, so there is nothing here that "
                "could accept or refuse this implementation")
        chk.add("PENDING", "T5 the contract refused HEAD", "not run: no cases")
        return

    inv = e2e_mod.inventory(root)
    unknown = [c for c in cases if c not in inv]
    if unknown:
        chk.add("FAIL", "T4 the contract accepts the code",
                "committed by the e2e phase and not in "
                f"{e2e_mod.manifest(root).relative_to(repo)}:\n"
                + "\n".join(f"  {c}" for c in unknown)
                + "\nthe inventory was edited after that phase closed")
        chk.add("PENDING", "T5 the contract refused HEAD",
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
        chk.add("FAIL", "T4 the contract accepts the code", "\n".join(red)
                + "\nthe implementation does not yet keep the promise these "
                "were written to hold it to")
        chk.add("PENDING", "T5 the contract refused HEAD",
                "not run: a case red in the tree says nothing about HEAD")
        return
    chk.add("PASS", "T4 the contract accepts the code",
            f"all {len(cases)} case(s) green in the working tree")

    with leg.at_head(repo) as (head, error):
        if head is None:
            chk.add("FAIL", "T5 the contract refused HEAD",
                    "could not check out HEAD to measure against:\n" + error)
            return
        at_head = [leg.run_command(head, inv[c]["command"]) for c in cases]
        dead = leg.unrunnable(*at_head)
        if dead:
            chk.add("FAIL", "T5 the contract refused HEAD", dead
                    + "\na case that could not be started at HEAD did not "
                    "refuse it, and this row reads a non-zero exit there as the "
                    "answer it wants")
            return
        green = [c for c, result in zip(cases, at_head) if result["exit"] == 0]

    if green:
        chk.add("FAIL", "T5 the contract refused HEAD",
                "already green at HEAD, before this implementation existed:\n"
                + "\n".join(f"  {c}" for c in green)
                + "\nso a green in the tree is a state these cases found rather "
                "than a difference this implementation made; either the case "
                "observes nothing, or the behaviour it pins was already there\n"
                "driving an epic, this is the ordinary shape of a child whose "
                "sibling already delivered the behaviour -- every child's e2e "
                "phase lands before any child's impl phase, so a sibling that "
                "went first can satisfy this child's cases. It says this "
                "child's cases do not discriminate, not that the gate is "
                "broken.")
        return
    chk.add("PASS", "T5 the contract refused HEAD",
            f"all {len(cases)} case(s) red at HEAD and green in the tree")


# --------------------------------------------------------------------------
# C2: the recorded red is this tree's
# --------------------------------------------------------------------------
def c2_recorded_red(chk: Check, repo: Path, iid: int,
                    dirty: list[str]) -> list[str]:
    """The record exists, was measured against this HEAD, and still describes
    these tests.

    Three ways it can be wrong, and each wants a different fix:

      absent    -- `red` was never run, so no red was ever attributed
      stale     -- HEAD moved underneath it, so its subtraction is against a
                   tree that is no longer the one below this change
      laundered -- a recorded test file has changed since it was measured

    The third is the one with teeth. It is the retired `logic` phase's "may not
    touch a test file" rule, restated: the edit is not refused, it is detected,
    and the only way back is to re-run `red` -- which over a test edited into
    passing finds nothing failing, because the implementation is already there.
    """
    record = load_record(repo, iid)
    if record is None:
        chk.add("FAIL", "C2 the recorded red",
                f"no red is on file at {RECORD_DIR / f'{iid}.json'}\n"
                "`red` is what writes it, and it is the only moment in this "
                "phase at which a failing test can be attributed to the change: "
                "it passes when a named test fails that was not failing at "
                "HEAD, which is a moment the implementation is not finished\n"
                f"next: {leg.phase_command(PHASE, '<project>', 'red', iid)}")
        return []

    names = [str(n) for n in record.get("names", []) if str(n).strip()]
    if not names:
        chk.add("FAIL", "C2 the recorded red",
                "the record on file names no failing test")
        return []

    head = head_sha(repo)
    if record.get("head") != head:
        chk.add("FAIL", "C2 the recorded red",
                f"the red was measured against HEAD "
                f"{str(record.get('head'))[:9]}, and HEAD is now {head[:9]}\n"
                "the recorded names are a subtraction against the commit "
                "underneath this change; a different commit is a different "
                "subtraction, and the set could name a test that was already "
                "failing there\n"
                f"next: {leg.phase_command(PHASE, '<project>', 'red', iid)}")
        return []

    now = test_file_digests(repo, dirty)
    was = {str(k): str(v) for k, v in (record.get("test_files") or {}).items()}
    drifted = sorted(rel for rel in set(was) | set(now)
                     if was.get(rel) != now.get(rel))
    if drifted:
        chk.add("FAIL", "C2 the recorded red",
                "a test file has changed since the red was measured:\n"
                + "\n".join(f"  {rel}" for rel in drifted)
                + "\nthe recorded names were measured against those bytes, so "
                "they no longer describe the tests in this tree. Re-run `red`: "
                "it records what actually fails now, and if the answer is "
                "nothing, that is the measurement -- a test edited into passing "
                "after the fact is not a red this change earned\n"
                f"next: {leg.phase_command(PHASE, '<project>', 'red', iid)}")
        return []

    chk.add("PASS", "C2 the recorded red",
            f"{len(names)} failing test(s) recorded against HEAD {head[:9]}, "
            f"over {len(now)} unchanged test file(s): {', '.join(names)}")
    return names


# --------------------------------------------------------------------------
# the run
# --------------------------------------------------------------------------
def _wi_checks(args: argparse.Namespace, *, require_clean: bool,
               run_tests: bool, recording: bool = False):
    """Every row this phase can produce, for one verb.

    `recording` selects the `red` verb's rows over `test`'s. The two never run
    together: `red` measures a tree where something fails, `test` measures a
    tree where nothing does, and a verb that ran both would have to report one
    of them failing every time.
    """
    chk = Check()
    repo = leg.repo_root()
    root = e2e_mod.e2e_root(repo, args.project)
    src = leg.leg_root(repo, args.project, PHASE)

    state: dict = {"cfg": None, "dirty": [], "cases": [], "red": [],
                   "record": None}

    kind = leg.p0_delivery_flow(chk, repo, args.wi, "behavior")
    if chk.failed:
        return chk, repo, root, state
    leg.p1_work_item(chk, repo, args.wi, kind)
    if chk.failed:
        return chk, repo, root, state

    dirty = leg.dirty_set(repo)
    state["dirty"] = dirty
    if require_clean:
        leg.p2_clean_tree(chk, dirty)
        leg.p3_leg_is_open(chk, repo, args.wi, PHASE)
        leg.p4_predecessor_landed(chk, repo, args.wi, PHASE)
        return chk, repo, root, state

    leg.p3_leg_is_open(chk, repo, args.wi, PHASE)
    leg.p4_predecessor_landed(chk, repo, args.wi, PHASE)
    leg.c0_scope(chk, repo, src, dirty, PHASE)

    cfg = ImplConfig(repo, args.project)
    state["cfg"] = cfg
    if cfg.problem:
        chk.add("FAIL", "C1 impl commands declared", cfg.problem)
    else:
        chk.add("PASS", "C1 impl commands declared",
                f"build `{cfg.build}`\ntest  `{cfg.test}`\n"
                f"names read by the `{cfg.harness}` parser")

    cases = leg.contract_set(repo, root, args.wi, "e2e")
    state["cases"] = cases

    if recording:
        # Named even when they do not run, so the report never omits a row a
        # later run will have. A silent absence and a green read the same in a
        # summary.
        rows = ("R1 build is green", "R2 named red", "R3 contract still red")
        if chk.failed:
            for name in rows:
                chk.add("PENDING", name,
                        "not run: a FAIL above means anything measured here "
                        "would describe something other than this phase's "
                        "change")
            return chk, repo, root, state
        if not r1_build_is_green(chk, repo, cfg):
            for name in rows[1:]:
                chk.add("PENDING", name,
                        "not run: the tests did not compile, so there are no "
                        "test names to read and no product to run the cases "
                        "against")
            return chk, repo, root, state
        prior = load_record(repo, args.wi)
        opening = not (prior and prior.get("head") == head_sha(repo)
                       and prior.get("names"))
        new = r2_named_red(chk, repo, cfg)
        if not r3_contract_still_red(chk, repo, root, cases, opening):
            return chk, repo, root, state
        if not new:
            return chk, repo, root, state
        # The union, not the replacement. An ordinary TDD loop records a red,
        # satisfies it, and writes the next test -- and on that second `red` the
        # first name is passing and would drop out of a set difference. Dropping
        # it would discard the attribution it was recorded for.
        carried = list(prior.get("names", [])) if (prior and not opening) else []
        keep = sorted({str(n) for n in [*new, *carried] if str(n).strip()})
        state["red"] = keep
        state["record"] = {
            "wi": args.wi,
            "project": args.project,
            "head": head_sha(repo),
            "names": keep,
            "test_files": test_file_digests(repo, dirty),
        }
        return chk, repo, root, state

    state["red"] = c2_recorded_red(chk, repo, args.wi, dirty)

    if not run_tests:
        return chk, repo, root, state

    rows = ("T1 build is green", "T2 recorded tests green",
            "T3 the suite is whole", "T4 the contract accepts the code",
            "T5 the contract refused HEAD")
    if chk.failed:
        for name in rows:
            chk.add("PENDING", name,
                    "not run: a FAIL above means anything measured here would "
                    "describe something other than this phase's change")
        return chk, repo, root, state

    if not t1_build_is_green(chk, repo, cfg):
        for name in rows[1:]:
            chk.add("PENDING", name,
                    "not run: nothing compiled, so there are no test names to "
                    "read and no product to run the cases against")
        return chk, repo, root, state

    t2_t3_tests(chk, repo, cfg, state["red"])
    t4_t5_contract(chk, repo, root, cases)
    return chk, repo, root, state


# --------------------------------------------------------------------------
# verbs
# --------------------------------------------------------------------------
def cmd_start(args: argparse.Namespace) -> int:
    chk, repo, root, _state = _wi_checks(
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
    print("## The cases this has to turn green\n")
    for case_id in leg.contract_set(repo, root, args.wi, "e2e"):
        command = str(inv.get(case_id, {}).get("command") or "").strip()
        print(f"- `{case_id}`" + (f" -- `{command}`" if command else ""))
    print()
    print("=" * 78)
    print("Write the colocated tests in")
    for name in leg.TEST_FILES:
        print(f"  apps/{args.project}/src/**/{name}")
    print("wired in with `#[cfg(test)] mod tests;`. An inline")
    print("`#[cfg(test)] mod tests { ... }` is invisible to the scope check")
    print("that requires this phase to have written a test at all, so it is")
    print("refused.")
    print()
    print("Then run `red` BEFORE writing the implementation. That verb is the")
    print("only moment in this phase at which a failing test can be attributed")
    print("to this change: it records the names that fail here and did not fail")
    print("at HEAD. Write the implementation first and there is nothing failing")
    print("to record, and no red can be manufactured afterwards.")
    print()
    print("Then implement, in the same tree, until every recorded test and every")
    print("case above is green. Re-run `red` whenever you add a test that fails")
    print("-- the record accumulates, so each red in the loop is kept.")
    print()
    print("Editing a test file after `red` measured it is detected, not refused:")
    print("`C2` sends you back to `red`, which over a test edited into passing")
    print("finds nothing failing and says so.")
    print(f"\nnext.command: "
          f"{leg.phase_command(PHASE, args.project, 'red', args.wi)}")
    return 0


def cmd_red(args: argparse.Namespace) -> int:
    chk, repo, _root, state = _wi_checks(
        args, require_clean=False, run_tests=True, recording=True)
    print(f"the named red, measured against HEAD: #{args.wi}")
    chk.report()
    if chk.failed:
        print("\nnothing was recorded; the tree is unchanged.")
        print("next.command: fix the FAIL rows above, then re-run this verb")
        return 1

    path = save_record(repo, args.wi, state["record"])
    red = state["red"]
    print(f"\nrecorded {len(red)} failing test(s) to "
          f"{path.relative_to(repo)}: {', '.join(red)}")
    print("that record is what `test` measures the implementation against. It")
    print("is under .aw/, which .gitignore names, so it is never part of the")
    print("change it describes.")
    print(f"\nnext.command: "
          f"{leg.phase_command(PHASE, args.project, 'verify', args.wi)}")
    return 0


def cmd_verify(args: argparse.Namespace) -> int:
    chk, _repo, _root, _state = _wi_checks(
        args, require_clean=False, run_tests=False)
    print(f"mechanical admissibility: #{args.wi}")
    chk.report()
    print()
    print("These checks say the change is ADMISSIBLE -- it is a change under")
    print("src/ that wrote at least one test file, it comes after a landed")
    print("contract, the project declared how to build and run its tests, and a")
    print("red is on file for this HEAD over these test bytes. Nothing was built")
    print("or run.")
    if chk.failed:
        print("\nnext.command: fix the FAIL rows above, then re-run this verb")
        return 1
    print(f"\nnext.command: "
          f"{leg.phase_command(PHASE, args.project, 'test', args.wi)}")
    return 0


def cmd_test(args: argparse.Namespace) -> int:
    chk, _repo, _root, state = _wi_checks(
        args, require_clean=False, run_tests=True)
    print(f"the recorded red, the suite, and both sides of the contract: "
          f"#{args.wi}")
    chk.report()
    if chk.failed:
        print("\nnext.command: fix the FAIL rows above, then re-run this verb")
        return 1
    print(f"\nthe {len(state['red'])} recorded test(s) pass, the suite is whole, "
          f"and the contract refused HEAD and accepts the tree "
          f"({', '.join(state['cases'])}).")
    print(f"next.command: "
          f"{leg.phase_command(PHASE, args.project, 'commit', args.wi)}")
    return 0


def cmd_commit(args: argparse.Namespace) -> int:
    chk, repo, _root, state = _wi_checks(
        args, require_clean=False, run_tests=True)
    print(f"commit gate: #{args.wi}")
    chk.report()
    if chk.failed:
        print("\nnothing was committed; the tree is unchanged and the work is "
              "still here.")
        print("next.command: fix the FAIL rows above, then re-run this verb")
        return 1

    dirty = state["dirty"]
    digest = leg.change_digest(repo, args.wi, dirty)
    trailers = [
        # The names travel with the commit that produced them. The record under
        # `.aw/` is scratch -- it is how the phase talks to itself between
        # verbs, and it is gitignored -- so the commit is where the measurement
        # becomes history. A phase rebased away then takes its evidence with it,
        # which a state file left behind would not: that file would go on
        # asserting a red for a commit no longer in the log.
        f"Impl-Red: {', '.join(state['red'])}",
        f"Impl-Contract: {', '.join(state['cases'])}",
        f"Impl-Change-Digest: {digest}",
    ]
    message = (f"{PHASE}(wi-{args.wi}): satisfy the contract, and the tests "
               f"that refused it\n"
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
    print(f"Impl-Commit: {sha}")
    # The record described a HEAD this commit has just moved off. Leaving it
    # would let a later run read a stale file as a live one; `C2`'s staleness
    # row would catch that only because the sha differs, which is a coincidence
    # of ordering rather than a guarantee.
    record_path(repo, args.wi).unlink(missing_ok=True)
    print(f"\nnext.command: {leg.AW_CLI} change lifecycle {args.wi} --leg {PHASE} "
          f"--commit {sha} --digest {digest}")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="impl.py",
                                     description=__doc__.splitlines()[0])
    # Required, with no default. It used to default to `agentic-workflow`; that
    # crate was removed, so the default became a write root that does not exist
    # -- and a phase resolving a missing directory reports a project the caller
    # never chose. There is no project this can assume, so it is named or the
    # run refuses.
    parser.add_argument("--project", required=True,
                        help="project under apps/; must precede the verb")
    sub = parser.add_subparsers(dest="verb", required=True)

    wi = argparse.ArgumentParser(add_help=False)
    wi.add_argument("wi", type=int, help="work item iid")

    p = sub.add_parser("start", parents=[wi],
                       help=f"open a work item's {PHASE.upper()} phase; "
                            "refuses a dirty tree")
    p.set_defaults(func=cmd_start)

    p = sub.add_parser("red", parents=[wi],
                       help="build, and record the named failures against HEAD")
    p.set_defaults(func=cmd_red)

    p = sub.add_parser("verify", parents=[wi],
                       help="the mechanical list over the whole change; "
                            "runs nothing")
    p.set_defaults(func=cmd_verify)

    p = sub.add_parser("test", parents=[wi],
                       help="the recorded names, the suite, and both sides of "
                            "the contract")
    p.set_defaults(func=cmd_test)

    p = sub.add_parser("commit", parents=[wi],
                       help="re-run everything and commit the change")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_commit)

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
