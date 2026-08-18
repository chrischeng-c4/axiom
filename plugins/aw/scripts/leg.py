#!/usr/bin/env python3
"""What every leg of a change work item has in common.

A change lands in three legs, and `CLAUDE.md` fixes their order: the artifact
that can refuse the work is authored first, and the implementation last. Which
three is currently changing -- `ec -> td -> cb` is being replaced by
`e2e -> unit -> logic` -- and both ladders are live here at once.

The legs differ in what they may write and in what makes them pass. They do not
differ in how a leg is *bounded*, and this module is that part, held once so
that six legs across two lifecycles cannot drift into six slightly different
definitions of the same word.

What "this change" means
------------------------
A leg's change is **the diff against HEAD**, and that definition holds only
because `start` refuses to open a leg while the working tree is dirty. From a
clean start, whatever `git status` reports is what this leg wrote: the
population is derived from git rather than remembered in a side table, a list
of ids, or a constant inside each artifact. A side table can point at a file
that was deleted and nothing would notice; `git status` cannot.

The same clean start is what lets `C0` refuse a path belonging to a later leg.
Until it existed, "never write `src/**` here" was prose in a skill body, which
is a request rather than a gate.

This module is imported, never invoked. It carries no `main`, and every
function here is one a leg script calls -- so a check that lives here is a
check all three legs run, and adding one to a single leg means putting it in
that leg's own script instead.
"""
from __future__ import annotations

import ast
import contextlib
import hashlib
import importlib.util
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

# See the note in `change.py`: the script's directory is on `sys.path` when
# this runs as a script but not when a gate loads it through
# `importlib.exec_module`.
sys.path.insert(0, str(Path(__file__).resolve().parent))

import workitem  # noqa: E402

# `git` in this checkout is run with fsmonitor disabled: a stalled fsmonitor
# daemon blocks every command that reads the index, indefinitely and silently.
GIT = ("git", "-c", "core.fsmonitor=false")

# The ladder, and its order, come from `workitem.py` -- which already owns them
# because it renders the lifecycle block in a change body from them. Re-listing
# them here would be a second copy of a closed enum, and the two could disagree
# about what the order *is* without anything comparing them.
#
# They did disagree, for exactly that reason. This module used to bind two
# names: `LEGS = workitem.LEGS` for the retiring `ec -> td -> cb`, and a
# separately written `PHASES = ("e2e", "unit", "logic")` for the ladder
# replacing it. That was correct while both were live. When the changeover
# finished and the three retired scripts were deleted, only the tuple written
# here was updated -- the one imported from the engine still named the dead
# ladder, and it is the one `change.py --leg` validates against.
#
# So there is one name now, and it is an alias rather than a tuple: a phase
# renamed in the engine cannot leave this module still spelling it the old way,
# because there is nothing here left to spell.
PHASES = workitem.LEGS

# The one directory each leg may write. This is a lookup table rather than a
# second enum: it decides nothing about which legs exist or what order they run
# in, and `leg_root` refuses a leg it has no entry for by name.
#
# It is what makes `CLAUDE.md`'s artifact write order machine-readable -- `C0`
# refuses against this table, so the ordering rule has a consumer rather than
# only a paragraph.
LEG_ROOTS = {
    "e2e": "e2e",
    "unit": "src",
    "logic": "src",
}

# `unit` and `logic` share a write root, so the prefix half of `C0` cannot tell
# them apart. The filename half can, and this is the table it reads.
#
# The colocated tests go in their own file, wired in with `#[cfg(test)] mod
# tests;`. That keeps the private access that sends them into `src/**` in the
# first place, and it makes the boundary between the two phases a filename
# rather than a judgement about where a `#[cfg(test)]` span begins. That
# judgement has been wrong in this checkout before -- item-level `#[cfg(test)]
# fn` reads as production, and a brace scanner that does not strip `r#"..."#`
# reads fixture text as production -- and a scope gate built on it can be walked
# around by writing the test in a shape the reader does not recognise.
#
# Rust-only, because Rust is the only language whose tests live in the
# implementation tree. A Python project's tests are already a separate root and
# already separated by the prefix half.
TEST_FILES = ("tests.rs", "tests/mod.rs")

# Which phase must write a test file, and which may not touch one.
#
# The `unit` entry is not symmetry for its own sake: a `unit` phase that wrote
# no test file wrote no test, and the `logic` phase after it would be measured
# against nothing. Refusing it at `C0` names that defect, where letting it
# through would surface as an empty red set two rows later.
LEG_TEST_FILES = {"unit": "requires", "logic": "refuses"}

# Both tables are keyed by phase, and neither is a second enum -- so a phase
# renamed in `workitem.LEGS` has to reach them, and nothing about a dict makes
# it. `leg_root` refuses a phase the table has no entry for, which is one
# direction; this is the other, and it is the direction the entries above rotted
# in. `LEG_ROOTS` carried `ec`, `td` and `cb` alongside the live three for as
# long as the ladder they belonged to had been deleted -- unreachable, because
# `ladder_for` refuses those names, and therefore invisible.
#
# Checked at import rather than in a gate, so it is the exit code of whichever
# script asked, not something a suite has to be run to learn. `LEG_TEST_FILES`
# is a subset by design: `e2e` writes no Rust test file and has no entry. What
# it may not contain is a key no phase answers to, which is how a renamed
# `unit` would silently stop being required to write a test at all.
if set(LEG_ROOTS) != set(PHASES) or not set(LEG_TEST_FILES) <= set(PHASES):
    raise SystemExit(
        "leg.py: a phase table names something the ladder does not\n"
        f"  ladder:         {' -> '.join(PHASES)}\n"
        f"  LEG_ROOTS:      {sorted(LEG_ROOTS)}\n"
        f"  LEG_TEST_FILES: {sorted(LEG_TEST_FILES)}"
    )


def ladder_for(leg: str) -> tuple:
    """The ordered lifecycle `leg` belongs to.

    Derived rather than passed in. A caller that named its own ladder could
    name a different one from the ladder `leg_root` was extended for, and the
    disagreement would read as a missing predecessor rather than as a wiring
    mistake.
    """
    if leg in PHASES:
        return PHASES
    raise SystemExit(
        f"no lifecycle names the leg {leg!r}\n"
        f"known: {' -> '.join(PHASES)}"
    )


def leg_root(repo: Path, project: str, leg: str) -> Path:
    """The one directory `leg` may write in `project`."""
    if leg not in LEG_ROOTS:
        raise SystemExit(
            f"no write root is declared for leg {leg!r}\n"
            f"a lifecycle names it, `leg.LEG_ROOTS` does not: one of the two "
            "was extended without the other"
        )
    return repo / "apps" / project / LEG_ROOTS[leg]


def is_test_file(rel: str) -> bool:
    """Whether `rel` is a colocated test file rather than implementation."""
    return any(rel == name or rel.endswith("/" + name) for name in TEST_FILES)


def phase_command(phase: str, project: str, verb: str, wi: int | str) -> str:
    """The next command in the ladder, spelled so it can be pasted and run.

    `--project` sits on the top-level parser, ahead of the subparsers, so it
    has to precede the verb -- a printed command carrying it after the verb is
    a command that exits 2. It is printed at all because there is no default
    project to fall back on: the phases used to default to `agentic-workflow`,
    and when that crate was removed the default became a path that does not
    exist. Every printed command therefore names the project it was run for,
    which is also the only place an agent reading the output can see it.
    """
    return f"{phase}.py --project {project} {verb} {wi}"


# --------------------------------------------------------------------------
# locating things
# --------------------------------------------------------------------------
def repo_root(start: Path | None = None) -> Path:
    """The outermost directory carrying an `aw.toml`.

    Outermost, not nearest: `apps/<project>/aw.toml` exists too, and stopping
    at the first one found would silently scope every path to one project.
    """
    here = (start or Path.cwd()).resolve()
    found: Path | None = None
    for candidate in [here, *here.parents]:
        if (candidate / "aw.toml").is_file():
            found = candidate
    if found is None:
        raise SystemExit(
            f"not inside a checkout: no aw.toml above {here}\n"
            "Run this from inside the repository you mean to write against."
        )
    return found


def wi_body_path(repo: Path, iid: int) -> Path:
    """Where `change.py fetch <iid>` stages the work item's body."""
    return repo / ".aw" / "workitems" / "changes" / f"{iid}.md"


def sibling(name: str, key: str) -> Any:
    """A script beside this one, loaded by path and at most once.

    By path because these are scripts rather than an installed package, and a
    gate loads them with `exec_module` from a directory that is not
    `sys.path[0]`. At most once because more than one caller wants the same
    module -- `logic.py` and `unit.py` both read the case inventory `e2e.py`
    owns -- and executing a module twice produces two objects whose module-level
    constants are separate values that merely compare equal.

    `key` is passed rather than derived because it is not free: the
    verification harness registers these same modules under names of its own
    (`_paths.load_script_module`), and a derived key that disagreed with one of
    those would reintroduce the second copy this function exists to prevent.

    The literal `name` here is also read statically. `check_plugin.py` closes
    over these calls to work out which scripts need the pinned interpreter, so
    that `logic.py` -- which imports no TOML of its own and reaches `tomllib`
    only through what it loads here -- is not exempted from the pin assertion
    for being hard to see. Keep the argument a literal.
    """
    if key in sys.modules:
        return sys.modules[key]
    path = Path(__file__).resolve().parent / f"{name}.py"
    spec = importlib.util.spec_from_file_location(key, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[key] = module
    spec.loader.exec_module(module)
    return module


def change_module() -> Any:
    """`change.py` beside this file, imported rather than shelled out to.

    Shelling out would mean reading a human-facing report to find out whether
    the body is admissible. Importing returns the same list of errors the
    `validate` verb prints, before it has been turned into prose.
    """
    return sibling("change", "changemod")


def dirty_set(repo: Path) -> list[str]:
    """Every path git reports as differing from HEAD, tracked or not.

    `-uall` rather than the default: an untracked *directory* is reported
    collapsed to its own name, and a collapsed directory cannot be digested,
    scoped, or handed to `git commit` as a pathspec with any precision. A new
    artifact arrives untracked, so that is the common shape, not the exotic one.
    """
    proc = subprocess.run(
        [*GIT, "status", "--porcelain", "-uall"], cwd=repo, capture_output=True, text=True
    )
    if proc.returncode != 0:
        raise SystemExit(proc.stderr.strip() or "git status failed")
    paths: list[str] = []
    for line in proc.stdout.splitlines():
        if not line.strip():
            continue
        entry = line[3:]
        # A rename reads `old -> new`; the new name is the one on disk and the
        # one a pathspec has to carry.
        if " -> " in entry:
            entry = entry.split(" -> ", 1)[1]
        paths.append(entry.strip().strip('"'))
    return sorted(paths)


def wi_commits(repo: Path, iid: int) -> list[tuple[str, str]]:
    """Every commit carrying `Refs #<iid>` as a whole line, newest first.

    Anchored at both ends, so `Refs #35` does not match a commit for `#354`.
    This log is the only record that a leg landed: there is no state file and
    no flag on the work item, because either could say a leg is done while the
    commit it names was rebased away.
    """
    proc = subprocess.run(
        [*GIT, "log", "--format=%H %s", "--extended-regexp", f"--grep=^Refs #{iid}$"],
        cwd=repo, capture_output=True, text=True,
    )
    out: list[tuple[str, str]] = []
    for line in proc.stdout.splitlines():
        sha, _, subject = line.partition(" ")
        if subject:
            out.append((sha, subject))
    return out


def leg_commits(repo: Path, iid: int, leg: str) -> list[tuple[str, str]]:
    """The subset of `wi_commits` this leg produced, by subject prefix."""
    return [(sha, s) for sha, s in wi_commits(repo, iid) if s.startswith(f"{leg}(")]


def landed_paths(repo: Path, iid: int, leg: str) -> list[str]:
    """Every path `leg` committed for this work item.

    A later leg needs to know what an earlier one produced -- the tech design
    has to satisfy the contract, so it has to be told which contract. Reading
    it out of the earlier leg's commit means the answer is the same object
    `P4` accepted as evidence that the leg landed at all, rather than a second
    list that could name a case the commit does not contain.
    """
    paths: list[str] = []
    for sha, _subject in leg_commits(repo, iid, leg):
        proc = subprocess.run(
            [*GIT, "show", "--name-only", "--format=", sha],
            cwd=repo, capture_output=True, text=True,
        )
        paths.extend(line.strip() for line in proc.stdout.splitlines() if line.strip())
    return sorted(set(paths))


def contract_set(repo: Path, ec: Path, iid: int, leg: str) -> list[str]:
    """The case ids this work item's contract leg landed.

    Read out of that leg's commit, which is the same evidence `P4` accepted as
    proof the leg landed at all. The alternative -- a list recorded somewhere
    when the EC leg finished -- would be a second answer to the same question,
    free to name a case the commit does not contain.

    Both later phases ask this, and they must get the same answer: `unit`
    measures the set to establish what its skeleton has to fail against, and
    `logic` measures it again at HEAD and in the working tree. Two copies of the
    derivation could disagree about what the contract *is*, and the disagreement
    would read as a coverage difference between the phases rather than as a bug.

    `leg` is the commit-subject prefix to read the set out of, and it is passed
    rather than defaulted. It used to default to `ec`, from when two ladders
    were live at once; every caller now passes `e2e` and the default named a
    phase that no longer exists, so a caller that forgot the argument would have
    silently measured an empty set and reported "no cases" rather than failing.
    """
    prefix = f"{(ec / 'src' / 'cases').relative_to(repo)}/"
    return sorted({
        Path(p).stem
        for p in landed_paths(repo, iid, leg)
        if p.startswith(prefix) and p.endswith(".py")
    })


def change_digest(repo: Path, iid: int, paths: list[str]) -> str:
    """One digest over the work item and every byte of the change.

    The body is in here on purpose. The question a reviewer answers is "does
    this change satisfy this work item", so an edit to *either* side can flip
    the answer -- and a verdict that survives an edit to the work item is an
    approval of a requirement nobody reviewed.
    """
    h = hashlib.sha256()
    h.update(wi_body_path(repo, iid).read_bytes())
    for rel in sorted(paths):
        h.update(b"\n--" + rel.encode() + b"--\n")
        target = repo / rel
        h.update(target.read_bytes() if target.is_file() else b"(absent)")
    return h.hexdigest()


# --------------------------------------------------------------------------
# running things
# --------------------------------------------------------------------------
ANSI = re.compile(r"\x1b\[[0-9;]*m")
EXC_LINE = re.compile(
    r"^(?P<kind>[A-Za-z_][\w.]*(?:Error|Exception)):\s?(?P<msg>.*)", re.M)


def run_command(cwd: Path, command: str, timeout: int = 900) -> dict[str, Any]:
    """Run a declared command and report how it ended.

    The exception name and message are pulled out of stderr because an exit
    code alone cannot say *why* something is red, and a phase whose whole job
    is attributing a red needs to be able to print one. `split()` rather than a
    shell: a declared command that needs quoting or a pipe is a command doing
    more than naming a gate.
    """
    proc = subprocess.run(command.split(), cwd=cwd, capture_output=True,
                          text=True, timeout=timeout)
    clean = ANSI.sub("", proc.stderr)
    matches = list(EXC_LINE.finditer(clean))
    last = matches[-1] if matches else None
    return {
        "exit": proc.returncode,
        "exception": last.group("kind") if last else "",
        "message": last.group("msg").strip() if last else "",
        "stderr": clean,
        "stdout": proc.stdout,
    }


def case_constants(path: Path) -> dict[str, Any]:
    """Module-level literal assignments, read without importing the module.

    Importing would run the case. Reading is the point: these constants are the
    case's own declaration of what it is, and the declaration has to be
    readable before the case is trusted to run.
    """
    out: dict[str, Any] = {}
    for node in ast.parse(path.read_text(encoding="utf-8")).body:
        if not isinstance(node, ast.Assign):
            continue
        for target in node.targets:
            if isinstance(target, ast.Name):
                try:
                    out[target.id] = ast.literal_eval(node.value)
                except (ValueError, TypeError, SyntaxError):
                    pass
    return out


@contextlib.contextmanager
def at_head(repo: Path):
    """A detached checkout of `HEAD`, for measuring what was true before.

    Every phase has to run something against `HEAD` as well as against the
    working tree, because "green now" and "green because of this change" are
    different claims and only the second one is evidence.

    A worktree rather than a stash. A stash takes the uncommitted work off disk
    and puts it back, so an interruption anywhere in the middle loses work that
    was never committed. A worktree touches nothing the author is holding.

    Yields `None` if the checkout could not be made, so the caller reports that
    as its own row rather than as the measurement failing.
    """
    scratch = Path(tempfile.mkdtemp(prefix="aw-head-"))
    tree = scratch / "head"
    add = subprocess.run([*GIT, "worktree", "add", "--detach", str(tree), "HEAD"],
                         cwd=repo, capture_output=True, text=True)
    if add.returncode != 0:
        shutil.rmtree(scratch, ignore_errors=True)
        yield None, (add.stderr.strip() or add.stdout.strip())
        return
    try:
        yield tree, ""
    finally:
        subprocess.run([*GIT, "worktree", "remove", "--force", str(tree)],
                       cwd=repo, capture_output=True, text=True)
        shutil.rmtree(scratch, ignore_errors=True)


# --------------------------------------------------------------------------
# the report
# --------------------------------------------------------------------------
class Check:
    def __init__(self) -> None:
        self.rows: list[tuple[str, str, str]] = []

    def add(self, status: str, name: str, detail: str = "") -> None:
        self.rows.append((status, name, detail))

    @property
    def failed(self) -> list[tuple[str, str, str]]:
        return [r for r in self.rows if r[0] == "FAIL"]

    @property
    def pending(self) -> list[tuple[str, str, str]]:
        return [r for r in self.rows if r[0] == "PENDING"]

    def report(self) -> None:
        for status, name, detail in self.rows:
            print(f"  {status:8s} {name}")
            if detail:
                for line in detail.splitlines():
                    print(f"           {line}")


# --------------------------------------------------------------------------
# the preconditions every leg shares
# --------------------------------------------------------------------------
def p1_work_item(chk: Check, repo: Path, iid: int) -> None:
    """The work item exists locally and is an admissible change body.

    A leg script never reads the tracker. The skill runs `change.py fetch
    <iid>` first, which overwrites the local copy unconditionally -- and that
    overwrite is what makes the file on disk the tracker's body rather than a
    draft an earlier session left behind.
    """
    path = wi_body_path(repo, iid)
    if not path.is_file():
        chk.add("FAIL", "P1 work item",
                f"no staged body at {path}\nrun: change.py fetch {iid}")
        return
    errors = [e for e in change_module().validate_body(path.read_text(encoding="utf-8"))
              if not e.startswith("note:")]
    if errors:
        chk.add("FAIL", "P1 work item",
                f"#{iid} is not a valid change body:\n" + "\n".join(errors[:6]))
        return
    chk.add("PASS", "P1 work item", f"#{iid} is a valid change body")


def p2_clean_tree(chk: Check, dirty: list[str]) -> None:
    """Nothing is modified yet, which is what makes the later diff readable.

    This is the only check that has to hold *before* any work. With a clean
    start, everything `git status` reports from here on was written by this
    leg -- so the change never has to be remembered, listed, or bound to its
    artifacts by hand. Every later check derives its population from it.
    """
    if dirty:
        chk.add("FAIL", "P2 clean tree",
                "the working tree already carries changes, so a change made "
                "now could not be told apart from them; commit or stash first:\n"
                + "\n".join(f"  {p}" for p in dirty[:20])
                + (f"\n  ... and {len(dirty) - 20} more" if len(dirty) > 20 else ""))
        return
    chk.add("PASS", "P2 clean tree", "nothing differs from HEAD")


def p3_leg_is_open(chk: Check, repo: Path, iid: int, leg: str) -> None:
    """This work item's `leg` has not already landed.

    Without this, re-running a leg whose artifact is already committed finds an
    empty diff and reports it as a change with no content, which names the
    wrong defect.
    """
    landed = leg_commits(repo, iid, leg)
    if landed:
        chk.add("FAIL", "P3 leg is open",
                f"the {leg.upper()} leg for this work item is already committed:\n"
                + "\n".join(f"  {sha[:9]} {subject}" for sha, subject in landed))
        return
    chk.add("PASS", "P3 leg is open", f"no {leg}(...) commit carries `Refs #{iid}`")


def p4_predecessor_landed(chk: Check, repo: Path, iid: int, leg: str) -> None:
    """Every leg before `leg` has already landed for this work item.

    The write order is not advisory. A tech design authored before the contract
    exists has nothing that can refuse it, and an implementation written before
    either becomes the definition of correct by default. `P3` above stops a leg
    running twice; this stops one running early, and the two are separate
    because they fail for opposite reasons and want opposite fixes.

    The evidence is the same `Refs #<iid>` log `P3` reads, so a leg that landed
    is one that produced a commit -- not one that someone recorded as done.
    """
    ladder = ladder_for(leg)
    earlier = ladder[: ladder.index(leg)]
    if not earlier:
        chk.add("PASS", "P4 predecessor", f"{leg} is the first leg; nothing precedes it")
        return
    missing = [name for name in earlier if not leg_commits(repo, iid, name)]
    if missing:
        chk.add("FAIL", "P4 predecessor",
                f"{leg} cannot open before {', '.join(missing)}: no "
                + " or ".join(f"`{name}(...)`" for name in missing)
                + f" commit carries `Refs #{iid}`\n"
                f"the write order is {' -> '.join(ladder)}, and a {leg} artifact "
                "written first has nothing able to refuse it")
        return
    chk.add("PASS", "P4 predecessor",
            f"{', '.join(earlier)} already landed for #{iid}")


# --------------------------------------------------------------------------
# the semantic review
# --------------------------------------------------------------------------
# A phase gate decides only what a machine can decide. Whether a case observes
# the behaviour its name claims, or whether a test would refuse a wrong
# implementation, is not one of those things -- so two phases send the change
# to an independent reviewer and will not commit without its answer.
#
# Everything below is the part of that which is the same for both: how a
# transcript is read, where the answer is kept, and what a commit gate checks
# it against. What differs is the rubric and the surface, and those live in the
# phase scripts, because they are the two things the phases genuinely disagree
# about.
VERDICT_LINE = re.compile(r"^VERDICT:\s*(accepted|rejected)\s*$", re.M)
FINDING_LINE = re.compile(r"^FINDING:\s*(\S.*)$", re.M)

# The last thing every rubric says, held once. It is parsed mechanically at the
# other end, so a phase that worded it its own way would be describing a
# contract `parse_transcript` does not implement.
OUTPUT_CONTRACT = """\
OUTPUT CONTRACT -- parsed mechanically, not read by a human first.

Emit zero or more finding lines, then the verdict line, last:

  FINDING: <one line, name the question number, the artifact, and what is wrong>
  VERDICT: accepted

Rules: `VERDICT:` must be the final non-empty line, and every `VERDICT:` line
in the output must say the same thing. `rejected` requires at least one
`FINDING:` line.
"""


def record_path(repo: Path, phase: str, iid: int) -> Path:
    """Where `phase`'s verdict for `iid` is kept.

    Keyed by phase as well as by work item. Both reviewed phases record an
    answer about the same `#<iid>`, and one shared path would put the later
    phase's record exactly where the earlier phase's gate looks for it. The
    digests would not match, so nothing would be wrongly accepted -- but the
    refusal would read as a stale verdict rather than as two phases writing to
    one file.
    """
    return repo / ".aw" / "review" / f"{phase}-wi-{iid}.json"


def parse_transcript(raw: str) -> tuple[str, list[str]]:
    """The reviewer's answer, read out of its output rather than reported.

    An agent that pipes a reviewer's transcript into this has no discretion
    over what the verdict says; an agent that reports the verdict in prose has
    all of it. So the parsing is here, and every shape it refuses raises rather
    than guessing.
    """
    verdicts = VERDICT_LINE.findall(raw)
    if not verdicts:
        raise SystemExit("transcript carries no `VERDICT: accepted|rejected` line")

    # Measured, not assumed: `codex exec` prints its final answer twice -- once
    # in the streamed body and once as the closing message -- so "exactly one
    # VERDICT line" refused every real transcript. What has to stay refused is
    # the tampering shape, which is *disagreement*: a `rejected` in the body
    # with an `accepted` appended, or the reverse. Requiring unanimity plus a
    # VERDICT as the final non-empty line keeps both halves of that, and
    # neither is satisfied by echoing the same answer n times.
    distinct = sorted(set(verdicts))
    if len(distinct) > 1:
        raise SystemExit(
            f"transcript carries disagreeing verdicts {distinct}; a reviewer that "
            "says both has not decided, and picking one here would be this verb "
            "choosing the answer it was supposed to read"
        )
    tail = [line for line in raw.strip().splitlines() if line.strip()]
    if not VERDICT_LINE.match(tail[-1].strip()):
        raise SystemExit(f"VERDICT must be the final non-empty line; found {tail[-1][:80]!r}")

    # The same duplication reaches the findings. `dict.fromkeys` dedupes while
    # keeping the order they were raised in, so one objection echoed twice is
    # recorded once rather than as two independent objections.
    findings = list(dict.fromkeys(FINDING_LINE.findall(raw)))
    if distinct[0] == "rejected" and not findings:
        raise SystemExit("a rejected verdict with no FINDING line cannot be acted on")
    return distinct[0], findings


def run_verdict(args: Any, phase: str, reviewer: str, chk: Check, repo: Path,
                dirty: list[str], subjects: list[str]) -> int:
    """Bind a reviewer transcript to the exact bytes it reviewed.

    `subjects` is whatever the phase considers the reviewed population -- case
    ids for `e2e`, test files for the code review. It is recorded rather than
    checked: the binding that matters is the digest, and the list is what makes
    a record readable afterwards without re-deriving it.
    """
    if args.wi is None:
        raise SystemExit(
            "verdict needs a work item.\n"
            "The whole-surface review is advisory: it has no change to bind an "
            "answer to, so recording one would produce a file shaped exactly "
            "like the thing a commit gate reads, holding an approval of nothing."
        )
    if chk.failed:
        print(f"verdict gate: #{args.wi}")
        chk.report()
        print("\nno verdict was recorded. A verdict binds to a change that passed")
        print("the mechanical list; recording one over a change that did not would")
        print("produce an approval nobody could act on, since `commit` re-runs")
        print("these same rows and would refuse it anyway.")
        print("next.command: fix the FAIL rows above, then re-run the reviewer")
        return 1

    transcript = Path(args.transcript)
    if not transcript.is_file():
        raise SystemExit(f"no transcript at {transcript}")
    raw = transcript.read_text(encoding="utf-8", errors="replace")
    result, findings = parse_transcript(raw)

    record = {
        "work_item": args.wi,
        "phase": phase,
        "result": result,
        "reviewer": reviewer,
        "change_digest": change_digest(repo, args.wi, dirty),
        "subjects": subjects,
        "paths": dirty,
        "transcript_digest": hashlib.sha256(raw.encode()).hexdigest(),
        "findings": findings,
    }
    out = record_path(repo, phase, args.wi)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    # The transcript itself, beside the record. The record states the digest of
    # what was read; without the bytes it was taken over, that digest is a claim
    # about something nobody kept.
    shutil.copyfile(transcript, out.with_suffix(".transcript.txt"))

    print(f"recorded {result} by {reviewer}")
    print(f"  change     {record['change_digest'][:16]} over {len(dirty)} path(s)")
    print(f"  transcript {record['transcript_digest'][:16]} -> "
          f"{out.with_suffix('.transcript.txt')}")
    for finding in findings:
        print(f"  FINDING: {finding}")
    if result != "accepted":
        print("\nnext.command: address the findings, then re-run the reviewer")
        return 1
    # Through `phase_command`, not spelled here. `--project` is required with no
    # default on all three phase scripts and has to precede the verb, so the
    # `f"{phase}.py commit {args.wi}"` this used to print exited 2 -- the review
    # was accepted and the command offered to act on it could not run.
    print(f"\nnext.command: {phase_command(phase, args.project, 'commit', args.wi)}")
    return 0


def c7_verdict(chk: Check, repo: Path, phase: str, iid: int, dirty: list[str],
               reviewer: str) -> None:
    """An accepted verdict exists, and it is about these bytes.

    The digest covers the work-item body as well as the change, so this row
    refuses two different things with one comparison: an edit to the change
    after it was approved, and an edit to the requirement it was approved
    against. Neither is visible in a diff of the other.
    """
    path = record_path(repo, phase, iid)
    if not path.is_file():
        chk.add("FAIL", "C7 reviewed",
                f"no {phase} verdict has been recorded for #{iid}\n"
                f"run the reviewer -- {reviewer} -- and pipe its transcript "
                f"into: {phase}.py verdict {iid} --transcript <file>")
        return
    record = json.loads(path.read_text(encoding="utf-8"))
    want = change_digest(repo, iid, dirty)
    if record.get("change_digest") != want:
        chk.add("FAIL", "C7 reviewed",
                f"the recorded verdict is about different bytes\n"
                f"  reviewed {str(record.get('change_digest'))[:16]}\n"
                f"  now      {want[:16]}\n"
                "the work item or the change moved after the review, so the "
                "answer on file is about something that is no longer here")
        return
    if record.get("result") != "accepted":
        chk.add("FAIL", "C7 reviewed",
                f"the recorded verdict is `{record.get('result')}`:\n"
                + "\n".join(f"  {f}" for f in record.get("findings", [])))
        return
    chk.add("PASS", "C7 reviewed",
            f"{record.get('reviewer')} accepted {want[:16]} "
            f"over {len(dirty)} path(s)")


def c0_scope(chk: Check, repo: Path, root: Path, dirty: list[str], leg: str) -> None:
    """Every changed path is inside the one directory this leg may write.

    `CLAUDE.md` fixes the order -- `e2e/`, then `src/` for the colocated tests,
    then `src/` for the implementation -- and each `start` skill has always
    carried "never write the next leg's tree here" as prose. Prose in a skill
    body is a request. This is the same sentence with a consumer that can
    refuse it, and it works only because `start` demanded a clean tree: from a
    clean start, a path from another leg showing up here was written by this
    one.
    """
    prefix = f"{root.relative_to(repo)}/"
    if not dirty:
        chk.add("FAIL", "C0 scope",
                f"nothing differs from HEAD; there is no {leg.upper()} change to verify")
        return
    outside = [p for p in dirty if not p.startswith(prefix)]
    if outside:
        chk.add("FAIL", "C0 scope",
                f"changed outside {prefix}:\n" + "\n".join(f"  {p}" for p in outside))
        return

    # The filename half, for the two phases sharing one root. See
    # `LEG_TEST_FILES`.
    rule = LEG_TEST_FILES.get(leg)
    tests = [p for p in dirty if is_test_file(p)]
    if rule == "refuses" and tests:
        chk.add("FAIL", "C0 scope",
                f"the {leg} phase may not write a colocated test file:\n"
                + "\n".join(f"  {p}" for p in tests)
                + "\nthese are the artifact this phase is being measured "
                "against, and editing one here -- including reformatting it, "
                "and including whitespace only -- is the retrofit the phase "
                "split exists to refuse")
        return
    if rule == "requires" and not tests:
        named = " or ".join(f"`{name}`" for name in TEST_FILES)
        chk.add("FAIL", "C0 scope",
                f"the {leg} phase wrote no colocated test file:\n"
                + "\n".join(f"  {p}" for p in dirty[:20])
                + f"\ncolocated tests go in {named}, wired in with "
                "`#[cfg(test)] mod tests;` -- an inline `#[cfg(test)] mod "
                "tests {{ ... }}` is invisible to the scope check that has to "
                "keep the next phase off them")
        return

    detail = f"all {len(dirty)} changed paths are under {prefix}"
    if rule == "requires":
        detail += f", including {len(tests)} test file(s)"
    elif rule == "refuses":
        detail += ", and none is a colocated test file"
    chk.add("PASS", "C0 scope", detail)
