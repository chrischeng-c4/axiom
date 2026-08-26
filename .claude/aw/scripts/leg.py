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

import contextlib
import hashlib
import importlib.util
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

# The one path a phase may write outside its root, relative to the project
# directory. `e2e`'s case inventory is the crate manifest, which sits one level
# above `e2e/` -- and registering a case is not optional, because `autotests =
# false` means a file nobody declared does not run. Without this entry the
# phase would be unable to do its job: `C1` requires the `[[test]]` stanza and
# `C0` would refuse the edit that adds it.
#
# Exact filenames, never a prefix. `apps/<p>/Cargo.toml` is the register; every
# other path outside the root is still the out-of-scope write `C0` exists to
# name, and `unit` and `logic` have no entry here at all.
LEG_EXTRA_PATHS = {"e2e": ("Cargo.toml",)}

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
if (set(LEG_ROOTS) != set(PHASES)
        or not set(LEG_TEST_FILES) <= set(PHASES)
        or not set(LEG_EXTRA_PATHS) <= set(PHASES)):
    raise SystemExit(
        "leg.py: a phase table names something the ladder does not\n"
        f"  ladder:          {' -> '.join(PHASES)}\n"
        f"  LEG_ROOTS:       {sorted(LEG_ROOTS)}\n"
        f"  LEG_TEST_FILES:  {sorted(LEG_TEST_FILES)}\n"
        f"  LEG_EXTRA_PATHS: {sorted(LEG_EXTRA_PATHS)}"
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
    prefix = f"{ec.relative_to(repo)}/"
    out: set[str] = set()
    for path in landed_paths(repo, iid, leg):
        if not path.startswith(prefix) or not path.endswith(".rs"):
            continue
        # Files directly in the root, matching `e2e.changed_cases`. A
        # subdirectory holds fixtures for a harness rather than cases -- a
        # `[[test]]` stanza names one file -- so a stem lifted out of one is an
        # id no target can match. The crate manifest lands in the same commit,
        # because registering a case and writing it are one act, and the `.rs`
        # suffix is what keeps it out of this set.
        if "/" in path[len(prefix):]:
            continue
        out.add(Path(path).stem)
    return sorted(out)


def change_digest(repo: Path, iid: int, paths: list[str]) -> str:
    """One digest over the work item and every byte of the change.

    The body is in here on purpose. The trailer names the bytes a phase
    measured, and what it measured was "does this change satisfy this work
    item" -- so an edit to *either* side is a different change, and a digest
    that survived an edit to the work item would name a measurement against a
    requirement nobody measured.
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

    `unrunnable` is the reason the command never started or never finished, and
    it is empty for every command that ran. It exists because "did not run" and
    "ran and failed" are the same non-zero exit, and several rows here read a
    non-zero exit as the answer they were hoping for -- so a result they cannot
    tell apart is one they would accept. Callers that treat a failure as
    evidence have to check it; callers that treat a failure as a failure do not.
    """
    try:
        proc = subprocess.run(command.split(), cwd=cwd, capture_output=True,
                              text=True, timeout=timeout)
    except FileNotFoundError:
        # The first word of a declared command naming nothing on PATH. It used
        # to end the phase in a traceback, which no row accounted for.
        head = (command.split() or ["(empty command)"])[0]
        return _unrunnable(command, 127, f"`{head}` is not on PATH")
    except subprocess.TimeoutExpired:
        return _unrunnable(command, 124, f"it did not finish within {timeout}s")
    clean = ANSI.sub("", proc.stderr)
    matches = list(EXC_LINE.finditer(clean))
    last = matches[-1] if matches else None
    return {
        "exit": proc.returncode,
        "exception": last.group("kind") if last else "",
        "message": last.group("msg").strip() if last else "",
        "stderr": clean,
        "stdout": proc.stdout,
        "unrunnable": "",
    }


def _unrunnable(command: str, code: int, why: str) -> dict[str, Any]:
    reason = f"`{command}` did not run: {why}"
    return {"exit": code, "exception": "", "message": "",
            "stderr": reason, "stdout": "", "unrunnable": reason}


def unrunnable(*results: dict[str, Any]) -> str:
    """Every reason among these results that a command did not run.

    Written to be called on a list rather than per result, because the rows
    that need it run one command per case and a report naming the first failure
    would hide the rest.
    """
    return "\n".join(r["unrunnable"] for r in results if r.get("unrunnable"))


# `case_constants` stood here until the cases became Rust. It read a Python
# case's module-level `CASE_ID`/`DIMENSION`/`ASSERTIONS` without importing the
# file, so `C1` could check that a case declared what it was. A `[[test]]`
# target has no analogue and nothing invented one: the name in the manifest is
# now the whole of a case's declaration, and `C1` checks that against the file
# on disk instead. Deleted rather than left unused -- a reader finding it here
# would reasonably conclude cases still declare themselves in their own source.


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
    # See `LEG_EXTRA_PATHS`. Resolved to full repo-relative paths and compared
    # whole, so this widens the write root by the exact files named there and
    # by nothing that merely starts with one of their names.
    allowed = {f"{root.parent.relative_to(repo)}/{name}"
               for name in LEG_EXTRA_PATHS.get(leg, ())}
    if not dirty:
        chk.add("FAIL", "C0 scope",
                f"nothing differs from HEAD; there is no {leg.upper()} change to verify")
        return
    outside = [p for p in dirty
               if not p.startswith(prefix) and p not in allowed]
    if outside:
        detail = f"changed outside {prefix}:\n" + "\n".join(f"  {p}" for p in outside)
        if allowed:
            detail += ("\nthe only path outside it this phase may write is "
                       + ", ".join(sorted(allowed)))
        chk.add("FAIL", "C0 scope", detail)
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

    register = sorted(p for p in dirty if p in allowed)
    detail = f"all {len(dirty)} changed paths are under {prefix}"
    if register:
        detail = (f"all {len(dirty)} changed paths are under {prefix} or are "
                  f"the register at {', '.join(register)}")
    if rule == "requires":
        detail += f", including {len(tests)} test file(s)"
    elif rule == "refuses":
        detail += ", and none is a colocated test file"
    chk.add("PASS", "C0 scope", detail)
