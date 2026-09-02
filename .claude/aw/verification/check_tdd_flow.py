#!/usr/bin/env python3
"""Every row the two phase scripts can refuse, driven through the gate.

`e2e.py` and `impl.py` replace the `ec -> td -> cb` ladder. The design
decisions this file pins, because it is the first consumer able to refuse them:

  The TD leg is gone, and with it the stage machinery. The deleted `ec.py`
  carried an `applicability` axis and an `AW_EC_STAGE` branch because the TD
  leg could only ever observe a document -- a case had to be told which of two
  very different things it was looking at. With no TD leg every case runs
  against the implementation at every phase, always, so a case needs no stage
  branch and there is no `B0`-shaped row asking whether the contract reaches
  this phase. Cases here are plain: one `verify()`, no environment read.

  There are two phases, not three. `unit` and `logic` were one phase from
  2026-08-27: in Rust a colocated test and the code under it are the same tree
  and are edited together, so a boundary that put them in two commits made
  every green iteration after the first pretend to be the first. What the split
  bought -- a named red measured before anything could satisfy it -- did not go
  with it. It moved onto `impl.py`'s `red` verb, and this file's job is to
  prove that the move kept the teeth.

  The unit tests still live in `src/**/tests.rs`, wired in with `#[cfg(test)]
  mod tests;`. The filename rule that survives is an *existence* rule, not a
  boundary one: a phase that wrote no test file wrote no test, and every row
  below it would be measured against nothing. It is a filename and not a
  `#[cfg(test)]` span reader for the same reason it always was -- an item-level
  `#[cfg(test)] fn` reads as production, and a brace scanner that does not strip
  `r#"..."#` reads fixture text as production. A filename is not a judgement
  call.

  What replaced the boundary is a measurement. `red` records the names that
  fail in the tree and did not fail at `HEAD`, together with the sha256 of every
  test file it measured, and `C2` refuses a record whose test bytes have since
  moved. The retrofit the old rule refused outright is now detected and sent
  back to `red` -- which over a test edited into passing finds nothing failing,
  because the implementation is already there. Refusing it and detecting it end
  in the same place; only the second one survives an honest TDD loop.

  A red is `build green, named test failed`. `[impl] build` and `[impl] test`
  are two commands, not one, because `cargo test` exits non-zero for a compile
  error exactly as it does for a failed assertion, and a red that is really a
  compile error proves nothing about the test -- the test never ran.

  The record is scratch and the commit is history. `.aw/impl-red/<iid>.json` is
  how the phase talks to itself between verbs and is gitignored; `commit`
  copies the names onto the `Impl-Red:` trailer and deletes the file. A phase
  rebased away then takes its evidence with it, which a state file left behind
  would not.

The controls:

  1 positive control    -- both phases run to commit against an unmutated
                          fixture. Run first and alone: if the ladder is not
                          green to begin with, every red below is already there.

  7 ladder controls     -- the preconditions, wired against two roots. `C0` is
                          both the prefix rule and the "this phase wrote a
                          test" rule, and the fold control is the one that
                          shows why the second half is load-bearing now that
                          the boundary half is gone.

  5 red controls        -- the verb the whole merge turns on. `R1`'s control is
                          the one it exists for: a test naming a function
                          nobody wrote yet is a compile error, and a compile
                          error accepted as a red is a phase that measured
                          nothing. `R2`'s and `R3`'s are the two shapes of
                          "the implementation was written first", which is the
                          defect the retired filename boundary used to refuse
                          structurally.

  4 record controls     -- absent, stale, laundered, and laundered by
                          whitespace. `C2` is the row that carries what the
                          boundary used to.

  5 test controls       -- the recorded names are the oracle; the rest of the
                          suite is not collateral; and the case that accepts
                          the implementation is a case that refused `HEAD`.

  1 outcome control     -- the names reach the commit, the record is cleared,
                          and nothing else lands.
"""
from __future__ import annotations

import concurrent.futures
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import threading

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import E2E_SCRIPT, IMPL_SCRIPT, pinned_interpreter  # noqa: E402

Path = pathlib.Path

UV = pinned_interpreter()

# Underscores, not hyphens: the case id is a `[[test]]` target name, and cargo
# passes that name to rustc as a crate name. A hyphen there is a build failure
# before any control gets to decide anything.
CASE = "demo_marker_case"
E2E_REL = "apps/demo/e2e"
CASE_REL = f"{E2E_REL}/{CASE}.rs"
PROJECT_TOML_REL = "apps/demo/aw.toml"
CARGO_REL = "apps/demo/Cargo.toml"

# A workspace root, so `cargo test -p demo` resolves from the checkout root.
# That is where every phase runs, and where `leg.at_head` puts its detached
# worktree, so a crate reachable only by `--manifest-path` would be a crate the
# derived case command cannot name.
WORKSPACE_REL = "Cargo.toml"
LIB_REL = "apps/demo/src/lib.rs"
TESTS_REL = "apps/demo/src/tests.rs"
MAIN_REL = "apps/demo/src/main.rs"

# A module that was in the crate before this work item opened, with a colocated
# test of its own. It exists so the fixture has a test this change did not
# write: without one, "the suite is whole" has an empty population to be whole
# over, and both of its controls would be measuring an empty set.
NEIGHBOUR_REL = "apps/demo/src/extra/mod.rs"
NEIGHBOUR_TESTS_REL = "apps/demo/src/extra/tests.rs"
NEIGHBOUR_TEST = "extra::tests::twice_doubles"

WI = 1
WI_BODY_REL = f".aw/workitems/deliveries/{WI}.md"
WI_RECEIPT_REL = f".aw/workitems/deliveries/{WI}.json"

# Where `red` writes what it measured. Under `.aw/`, which the fixture's
# `.gitignore` names for the same reason the real checkout does: a record that
# joined the dirty set would be refused by `C0` as a write outside `src/`.
RECORD_REL = f".aw/impl-red/{WI}.json"

# The one behaviour, held as a constant so a control cannot mutate the
# implementation into agreeing with a test that is looking for something else.
MARKER = "hello"

# The colocated tests' names as `cargo` prints them. `R2` records these and
# `T2` reads them back, so a control that needs to break the recording has
# something to compare against.
UNIT_TEST = "tests::marker_is_hello"
SHOUT_TEST = "tests::shout_is_loud"

WORKSPACE_TOML = '''\
[workspace]
resolver = "2"
members = ["apps/demo"]
'''

# `autotests = false` is in the scaffold rather than added by the `e2e` phase.
# It is a property of the project, not of a change: with autodiscovery on, a
# file under `e2e/` that nobody declared runs anyway, and `C1` would have
# nothing to refuse an unregistered case against. The phase appends stanzas to
# a manifest that already refuses undeclared files.
CARGO_TOML = '''\
[package]
name = "demo"
version = "0.0.0"
edition = "2021"
autotests = false

[dependencies]
'''

# The same manifest with the case registered -- the `e2e` phase's second write,
# and the only path outside its own root it is allowed to touch.
CARGO_TOML_REGISTERED = CARGO_TOML + f'''
[[test]]
name = "{CASE}"
path = "e2e/{CASE}.rs"
'''

# --------------------------------------------------------------------------
# The crate, in the states the two phases leave it in.
# --------------------------------------------------------------------------

# Scaffold: a crate that builds and does nothing. It exists before the `e2e`
# phase because a case whose red is "there is no crate" is not measuring the
# behaviour it names -- it is measuring the scaffold, and it would go green the
# moment anyone added an unrelated file.
LIB_SCAFFOLD = '''\
//! The demo crate.

pub mod extra;
'''

MAIN_SCAFFOLD = '''\
fn main() {}
'''

# The neighbouring module and its test, both part of the scaffold: they are at
# `HEAD` at every phase, so a phase that breaks or drops one is doing it to
# something it did not write.
NEIGHBOUR_SRC = '''\
//! A module this work item has nothing to do with.

#[cfg(test)]
mod tests;

pub fn twice(n: u32) -> u32 {
    n * 2
}
'''

NEIGHBOUR_TESTS_SRC = '''\
use super::twice;

#[test]
fn twice_doubles() {
    assert_eq!(twice(21), 42);
}
'''

# What the `impl` phase writes into `lib.rs` first: the module wiring and the
# skeleton. `todo!()` types as `!` and coerces, so this compiles -- which is the
# entire point. The test that calls it panics at run time, and a run-time panic
# is a red `red` can attribute to this change.
#
# Skeleton and implementation are one phase now, so this is a moment inside a
# phase rather than a commit. `red` is what makes the moment measurable: run it
# here and the failing name is on file before anything can satisfy it.
LIB_SKELETON = '''\
//! The demo crate.

pub mod extra;

#[cfg(test)]
mod tests;

/// The marker string the product writes.
pub fn marker() -> &'static str {
    todo!("the implementation lands later in this phase")
}
'''

# A skeleton carrying a second unwritten function, for the control that drives
# two `red` runs in one phase. The accumulation it measures has no analogue in
# the three-phase ladder: there, a second test meant a second `unit` commit.
LIB_SKELETON_TWO = '''\
//! The demo crate.

pub mod extra;

#[cfg(test)]
mod tests;

/// The marker string the product writes.
pub fn marker() -> &'static str {
    todo!("the implementation lands later in this phase")
}

/// The marker, shouted.
pub fn shout() -> String {
    todo!("the implementation lands later in this phase")
}
'''

# The same crate one iteration further on: `marker` is satisfied and `shout` is
# not. This is what an honest TDD loop looks like mid-phase, and the state in
# which the first recorded name is *passing* -- so a `red` that recomputed the
# set by subtraction alone would drop it.
LIB_HALF = f'''\
//! The demo crate.

pub mod extra;

#[cfg(test)]
mod tests;

/// The marker string the product writes.
pub fn marker() -> &'static str {{
    "{MARKER}"
}}

/// The marker, shouted.
pub fn shout() -> String {{
    todo!("the implementation lands later in this phase")
}}
'''

# What the `impl` phase writes into `lib.rs` last.
LIB_IMPLEMENTED = f'''\
//! The demo crate.

pub mod extra;

#[cfg(test)]
mod tests;

/// The marker string the product writes.
pub fn marker() -> &'static str {{
    "{MARKER}"
}}
'''

# What the `impl` phase writes into `main.rs`: the externally observable half,
# which is the half the e2e case can see.
# Relative to the process's own directory rather than to the checkout root.
# The case runs it with `current_dir` set to the crate, because a `[[test]]`
# target's own working directory is the crate root and a product that resolved
# its output against the repository root would only be observable from one of
# the two.
MAIN_IMPLEMENTED = '''\
use std::fs;

fn main() {
    fs::write("marker.txt", demo::marker()).expect("write the marker");
}
'''

# The colocated test. It is the whole of `tests.rs`, which is what lets `C0`
# see that this phase wrote a test at all, and what `C2` takes the sha256 of.
TESTS_SRC = f'''\
use super::marker;

#[test]
fn marker_is_hello() {{
    assert_eq!(marker(), "{MARKER}");
}}
'''

TESTS_SRC_TWO = TESTS_SRC + '''
#[test]
fn shout_is_loud() {
    assert_eq!(super::shout(), "HELLO");
}
'''

# --------------------------------------------------------------------------
# The e2e case
# --------------------------------------------------------------------------

# It runs the built binary directly -- `env!("CARGO_BIN_EXE_demo")` -- and
# never `cargo run`. A nested cargo invoked from inside a `cargo test` blocks
# on the build directory's file lock until the phase's 900-second timeout, and
# what that reports is a hung gate rather than a red case.
#
# It removes the marker before running, so what it reads is what this run
# produced. Without that it would pass on a file an earlier run left behind,
# which is `T3`'s defect arriving through the back door inside a single phase.
CASE_SRC = f'''\
//! The product writes the marker string.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn the_product_writes_the_marker() {{
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let marker = dir.join("marker.txt");
    let _ = fs::remove_file(&marker);
    Command::new(env!("CARGO_BIN_EXE_demo"))
        .current_dir(&dir)
        .status()
        .expect("the product runs");
    let observed = fs::read_to_string(&marker).unwrap_or_default();
    assert_eq!(
        observed, "{MARKER}",
        "running the product writes `{MARKER}` to the marker file"
    );
}}
'''

# The same case with the product gone from it: it asserts something about the
# repository that was already true before the work item existed. Green at HEAD
# and green in the tree, so `T4` cannot tell the two apart and only `T5` sees
# it. This is the shape of a case that stopped observing the thing it names.
CASE_SRC_BLIND = f'''\
//! A case that observes the scaffold and calls it the product.

use std::path::PathBuf;

#[test]
fn the_product_writes_the_marker() {{
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(
        dir.join("Cargo.toml").is_file(),
        "running the product writes `{MARKER}` to the marker file"
    );
}}
'''

# The project config. `[impl] build` and `[impl] test` are two keys rather than
# one because they answer different questions -- see the note at the top -- and
# `harness` names the parser that turns the second one's output into names.
# There is no default for any of the three: a project that never answered gets
# a refusal, not the weaker gate.
#
# `[impl]` and not `[unit]`, and that rename carried no migration: at the
# changeover no project in the real checkout declared either section, so this
# fixture is the only place the ladder's configuration has ever been exercised.
# That is a reason to read the controls below carefully rather than a reason to
# trust them less -- it just means they are the whole of the evidence.
PROJECT_TOML = '''\
[project]
name = "demo"

[impl]
build = "cargo test --offline --lib --no-run --manifest-path apps/demo/Cargo.toml"
test = "cargo test --offline --lib --manifest-path apps/demo/Cargo.toml"
harness = "cargo"
'''

# `.aw/` carries the staged body and the red record, `target/` is build output
# and `marker.txt` is product output. Mirroring the real checkout's ignores is
# not cosmetic: any of them reported by `git status` would join the dirty set
# and fail `C0` in every control below -- the right row red for entirely the
# wrong reason.
GITIGNORE = ".aw/\n__pycache__/\ntarget/\napps/demo/marker.txt\n"

WI_BODY = f"""\
## Goal

Running the demo product writes the string `{MARKER}` into
`apps/demo/marker.txt`, where today that file is never created.

## How

### Verified premises

- `{CASE_REL}:1` runs the product and reads the marker file, so it is red
  until the product writes one.
- `{MAIN_REL}:1` is an empty `main`, so nothing writes the marker today.
- `{LIB_REL}:1` carries no `marker` function, so the colocated test has
  nothing to call until the skeleton lands.

### Change points

- `{TESTS_REL}`
- `{LIB_REL}`
- `{MAIN_REL}`

### Frozen decisions

The marker path is fixed at `apps/demo/marker.txt` and the string is fixed at
`{MARKER}`. Excluded: any change under `apps/demo/e2e`, which belongs to a
phase that is closed.

## Acceptance

| # | command | current | target | why it cannot hold by accident |
|---|---|---|---|---|
| 1 | `impl.py test 1` | the colocated test panics and the case is red | `T2` green over `{UNIT_TEST}` and `T5` red at HEAD | the same case is run against HEAD and against the tree, so a case observing nothing is green in both and the second row refuses it |

### Negative control

Replace `"{MARKER}"` in `{LIB_REL}` with `"goodbye"` and re-run the command
above; row `T2` must go red naming `{UNIT_TEST}`. Restore the file
byte-for-byte and confirm sha256
`0000000000000000000000000000000000000000000000000000000000000000` before
continuing.

## Never

This addresses the worker implementing this work item, not the controller reviewing it.

### Must not touch

- `{E2E_REL}/**`
- `{NEIGHBOUR_REL}`
- `{NEIGHBOUR_TESTS_REL}`

`{TESTS_REL}` is deliberately absent from this list: it is a change point of
this phase, and writing it is the work. What a must-not-touch list would be
reaching for is the moment *after* `red` measured it -- and that is not a rule
a list can hold, because the same file is legitimately written before and
illegitimately written after. `C2` holds it instead, by comparing the bytes it
measured against the bytes that are there.

### Must not do

- Do not make the case green by editing the case's assertion.
- Do not edit a test into agreement with the implementation after `red` has
  measured it. `C2` will send you back to `red`, which will find nothing
  failing and say so.
- Do not write `apps/demo/marker.txt` by hand; it is output, not source.
"""


def mutate(text: str, old: str, new: str) -> str:
    """`str.replace`, refusing the no-op.

    Every control below breaks the fixture by substring, and a substring that
    stops matching -- a quote moved, a line rewrapped -- makes the mutation
    silently do nothing. The control then runs against an unmutated tree, sees
    the green it was written to see a red against, and reports the row as
    working. That happened here, to the selector control, and it is the one
    failure mode a negative control cannot have.
    """
    out = text.replace(old, new)
    if out == text:
        raise AssertionError(f"the mutation matched nothing: {old!r}")
    return out


TESTS_SRC_WRONG = mutate(TESTS_SRC, f'"{MARKER}"', '"goodbye"')

# A test file that dropped the test it was measured against. Used by the
# control that checks a phase cannot quietly delete a recorded name.
TESTS_SRC_TRUNCATED = '''\
use super::marker;
'''


def git(work: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-c", "core.fsmonitor=false", *args],
                          cwd=work, capture_output=True, text=True)


def git_checked(work: Path, *args: str) -> subprocess.CompletedProcess:
    """`git`, refusing a silent no-op.

    Every fixture-setup commit this file drives directly (as opposed to one
    landed by `e2e.py`/`impl.py` themselves) has to actually move `HEAD`: a
    `git add` that matches nothing or a `git commit` with nothing staged
    exits non-zero, but the plain `git()` wrapper discards that. Ignored, the
    control that follows measures a tree at the HEAD it already had while
    still reporting a clean PASS/FAIL -- the fixture equivalent of the
    unchecked-copy trap, here an unchecked commit.
    """
    proc = git(work, *args)
    if proc.returncode != 0:
        raise AssertionError(
            f"fixture setup `git {' '.join(args)}` did not land: "
            f"{proc.stdout}{proc.stderr}")
    return proc


def bake(work: Path, *paths: str) -> None:
    """Commit a fixture mutation so it is not part of the change under test.

    Several controls change the project config, which lives outside `src/`.
    Left uncommitted it would be a path from no phase at all, and `C0` would
    refuse the whole change before the row the control names got to decide
    anything. The control would still see a red -- but it would go on passing
    after its own row stopped working, which is the one thing a control may
    not do.

    Note what this does to the red record: it moves `HEAD`. Any control that
    bakes after `red` has run is a control about `C2`'s staleness row, whether
    it meant to be or not.
    """
    git_checked(work, "add", "-A", "--", *paths)
    git_checked(work, "commit", "-qm",
                "chore: adjust the fixture around the change")


def build(root: Path) -> Path:
    """A checkout scaffolded and committed, with both phases still open.

    The scaffold is a crate that compiles and does nothing, committed before
    anything else. Every phase measures against `HEAD`, so what `HEAD` contains
    at each point is load-bearing: a fixture that committed the crate and the
    cases together would leave the `e2e` phase unable to tell a case that
    observes the product from one that observes the scaffold.
    """
    fixture = root / "fixture"
    (fixture / E2E_REL).mkdir(parents=True)
    (fixture / LIB_REL).parent.mkdir(parents=True)
    (fixture / ".gitignore").write_text(GITIGNORE)
    (fixture / "aw.toml").write_text('version = "0.0.0"\n')
    (fixture / PROJECT_TOML_REL).write_text(PROJECT_TOML)
    (fixture / WORKSPACE_REL).write_text(WORKSPACE_TOML)
    (fixture / CARGO_REL).write_text(CARGO_TOML)
    (fixture / LIB_REL).write_text(LIB_SCAFFOLD)
    (fixture / MAIN_REL).write_text(MAIN_SCAFFOLD)
    (fixture / NEIGHBOUR_REL).parent.mkdir(parents=True)
    (fixture / NEIGHBOUR_REL).write_text(NEIGHBOUR_SRC)
    (fixture / NEIGHBOUR_TESTS_REL).write_text(NEIGHBOUR_TESTS_SRC)
    body = fixture / WI_BODY_REL
    body.parent.mkdir(parents=True)
    body.write_text(WI_BODY)
    (fixture / WI_RECEIPT_REL).write_text(json.dumps({
        "iid": WI,
        "type": "feat",
        "flow": "behavior",
        "state": "OPEN",
        "milestone": 7,
        "labels": ["app:demo", "phase:created", "type:feat"],
        "updated_at": "2026-08-31T00:00:00Z",
        "body_sha256": hashlib.sha256(WI_BODY.encode("utf-8")).hexdigest(),
    }, indent=2) + "\n")

    # Generated rather than written out, and committed with the scaffold. A
    # real crate has a lockfile under version control; this fixture would
    # otherwise grow one the first time any phase ran the product, and `C0`
    # would refuse every change from then on for a path no phase wrote.
    # Generated by the same cargo that will run here, so its format version is
    # whatever this cargo would have produced.
    subprocess.run(["cargo", "generate-lockfile", "--offline",
                    "--manifest-path", str(fixture / CARGO_REL)],
                   cwd=fixture, check=True, capture_output=True)

    subprocess.run(["git", "init", "-q"], cwd=fixture, check=True)
    # Set in the repo rather than passed per commit: the phase scripts run
    # `git commit` themselves and cannot be handed an identity from here.
    subprocess.run(["git", "config", "user.email", "t@t"], cwd=fixture, check=True)
    subprocess.run(["git", "config", "user.name", "t"], cwd=fixture, check=True)
    subprocess.run(["git", "add", "-A"], cwd=fixture, check=True)
    subprocess.run(["git", "commit", "-qm", "scaffold"], cwd=fixture, check=True)
    return fixture


def land_e2e(work: Path, *, case: str = CASE_SRC) -> None:
    """Write and commit the `e2e` phase's output by hand.

    By hand rather than by running the phase, because a control aimed at the
    later phase must not depend on the earlier one passing: if `e2e.py commit`
    breaks, every `impl` control would go red naming a row that is working. The
    trailer is written here in the shape the phase produces, and the control
    that checks the phase *does* produce it is separate.
    """
    (work / CARGO_REL).write_text(CARGO_TOML_REGISTERED)
    (work / CASE_REL).write_text(case)
    git_checked(work, "add", "-A")
    git_checked(work, "commit", "-qm",
        f"e2e(demo): pin the marker behaviour\n\nRefs #{WI}\nE2E-Red: {CASE}")


def write_tests(work: Path, *, tests: str = TESTS_SRC,
                lib: str = LIB_SKELETON) -> None:
    """The first half of the `impl` phase: the invariant and a skeleton.

    Uncommitted, which is its real shape -- it is one phase with the
    implementation, and the two land in the same commit.
    """
    (work / LIB_REL).write_text(lib)
    (work / TESTS_REL).write_text(tests)


def write_impl(work: Path, *, lib: str = LIB_IMPLEMENTED) -> None:
    """The second half of the `impl` phase, also uncommitted."""
    (work / LIB_REL).write_text(lib)
    (work / MAIN_REL).write_text(MAIN_IMPLEMENTED)


def land_red(work: Path, *, names: tuple[str, ...] = (UNIT_TEST,),
             files: tuple[str, ...] = (TESTS_REL,)) -> Path:
    """Write the red record by hand, in the shape `red` produces it.

    Same reason as `land_e2e`. A `T`-row control must not depend on `red`
    passing, or a break in `red` would report every row below it as broken. The
    controls that check `red` actually writes this file, with these names,
    accumulating across runs, are separate and run the verb for real.

    The digests are computed from the tree as it stands, so this has to be
    called after the test files are written and before anything edits them --
    which is exactly the moment `red` itself occupies.
    """
    record = {
        "wi": WI,
        "project": "demo",
        "head": git(work, "rev-parse", "HEAD").stdout.strip(),
        "names": sorted(names),
        "test_files": {
            rel: hashlib.sha256((work / rel).read_bytes()).hexdigest()
            for rel in sorted(files)
        },
    }
    path = work / RECORD_REL
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")
    return path


def read_record(work: Path) -> dict:
    path = work / RECORD_REL
    return json.loads(path.read_text()) if path.is_file() else {}


class Harness:
    """One fixture, and a fresh copy of it per control."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.fixture = build(root)
        self.n = 0
        self.results: list[tuple[int, str, str, str]] = []
        self.lock = threading.Lock()
        # Which control the calling thread is running, so a result can be filed
        # under its declared position rather than the order it finished in.
        self.slot = threading.local()

    def fresh(self) -> Path:
        with self.lock:
            self.n += 1
            n = self.n
        work = self.root / f"work{n}"
        shutil.copytree(self.fixture, work)
        return work

    def staged(self, *, through: str) -> Path:
        """A copy with the work up to and including `through` already in place.

        Three points rather than the old ladder's three phases, and only the
        first of them is a commit. `red` and `impl` are moments inside one
        phase, which is what the merge did to this fixture: the tree at
        `through="impl"` has one landed commit and one uncommitted change
        spanning tests and implementation together.
        """
        work = self.fresh()
        land_e2e(work)
        if through in ("red", "impl"):
            write_tests(work)
            land_red(work)
        if through == "impl":
            write_impl(work)
        return work

    def run(self, controls: list) -> None:
        """Every control, concurrently, reported in declaration order."""
        def go(item) -> None:
            i, fn = item
            self.slot.i = i
            fn()

        with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
            list(pool.map(go, enumerate(controls)))
        self.results.sort(key=lambda row: row[0])

    def phase(self, script: Path, work: Path, *args: str,
              env: dict | None = None) -> subprocess.CompletedProcess:
        return subprocess.run(
            [*UV, str(script), "--project", "demo", *args],
            cwd=work, capture_output=True, text=True, timeout=900,
            env={**os.environ, **(env or {})},
        )

    def e2e(self, work: Path, *args: str, **kw) -> subprocess.CompletedProcess:
        return self.phase(E2E_SCRIPT, work, *args, **kw)

    def impl(self, work: Path, *args: str, **kw) -> subprocess.CompletedProcess:
        return self.phase(IMPL_SCRIPT, work, *args, **kw)

    def record(self, name: str, want: str, got: str, ok: bool) -> None:
        row = (getattr(self.slot, "i", -1),
               "PASS" if ok else "**FAIL**", name, f"want {want}; got {got}")
        with self.lock:
            self.results.append(row)

    @staticmethod
    def red_on(proc: subprocess.CompletedProcess, row: str) -> bool:
        return any(line.strip().startswith("FAIL") and row in line
                   for line in proc.stdout.splitlines())

    @staticmethod
    def row_block(proc: subprocess.CompletedProcess, row: str) -> str:
        """One report row together with the detail indented under it.

        A control that needs a *detail* cannot grep one line, and grepping the
        whole of stdout would let a detail belonging to another row satisfy the
        claim -- which is the confusion between "the report mentions this name"
        and "this row accounts for it".
        """
        block: list[str] = []
        for line in proc.stdout.splitlines():
            if line[:2] == "  " and line[2:3] != " ":
                if block:
                    break
                if row in line:
                    block.append(line)
            elif block:
                if not line.startswith(" " * 11):
                    break
                block.append(line)
        return "\n".join(block)

    @staticmethod
    def trailer(work: Path, key: str) -> str:
        proc = git(work, "log", "-1", "--format=%B")
        for line in proc.stdout.splitlines():
            if line.startswith(f"{key}:"):
                return line.split(":", 1)[1].strip()
        return ""


def main() -> int:
    root = Path(tempfile.mkdtemp(prefix="aw-tdd-flow-"))
    try:
        h = Harness(root)

        # ---- positive control ------------------------------------------------
        #
        # Alone, and before anything else: every control below is a claim about
        # which row goes red, and if the ladder is not green to begin with, all
        # of those reds are already there and none of them mean anything.
        #
        # Both phases, in order, and the `impl` phase through all of `red`,
        # `test` and `commit` -- because the thing under test is the ladder, and
        # a positive control that skipped `red` would leave the verb this whole
        # merge turns on unmeasured in the one place it is supposed to be green.
        w = h.fresh()

        steps = [
            ("e2e commit", lambda: (
                (w / CARGO_REL).write_text(CARGO_TOML_REGISTERED),
                (w / CASE_REL).write_text(CASE_SRC),
                h.e2e(w, "commit", str(WI)))[-1]),
            ("impl start", lambda: h.impl(w, "start", str(WI))),
            ("impl red", lambda: (
                write_tests(w),
                h.impl(w, "red", str(WI)))[-1]),
            ("impl test", lambda: (
                write_impl(w),
                h.impl(w, "test", str(WI)))[-1]),
            ("impl commit", lambda: h.impl(w, "commit", str(WI))),
        ]
        for name, step in steps:
            r = step()
            if r.returncode != 0:
                print(r.stdout)
                print(r.stderr, file=sys.stderr)
                print(f"the positive control failed at `{name}`; every "
                      "negative control below would be uninterpretable, so they "
                      "are not run.", file=sys.stderr)
                return 1
        h.record("positive control: both phases run to commit",
                 "exit 0 from all five steps", "exit 0 from all five steps",
                 True)

        # -- the ladder --------------------------------------------------------

        def start_clean() -> None:
            w = h.fresh()
            r = h.e2e(w, "start", str(WI))
            h.record("start opens the first phase on a clean tree", "exit 0",
                     f"exit {r.returncode}", r.returncode == 0)

        def start_dirty() -> None:
            w = h.fresh()
            (w / CASE_REL).write_text(CASE_SRC)
            r = h.e2e(w, "start", str(WI))
            hit = h.red_on(r, "P2")
            h.record("start refuses a dirty tree", "FAIL on P2",
                     "red on it" if hit else r.stdout.strip()[-200:],
                     hit and r.returncode != 0)

        def behavior_phase_refuses_maintenance_receipt() -> None:
            w = h.fresh()
            receipt_path = w / WI_RECEIPT_REL
            receipt = json.loads(receipt_path.read_text())
            receipt.update({"type": "docs", "flow": "maintenance"})
            receipt["labels"] = ["app:demo", "phase:created", "type:docs"]
            receipt_path.write_text(json.dumps(receipt, indent=2) + "\n")
            r = h.e2e(w, "start", str(WI))
            hit = h.red_on(r, "P0")
            named = "maintenance flow" in h.row_block(r, "P0")
            h.record("behavior phases refuse a maintenance type receipt",
                     "FAIL on P0 naming maintenance flow",
                     f"P0 red: {hit}; names maintenance: {named}",
                     hit and named and r.returncode != 0)

        def behavior_phase_refuses_stale_fetch_body() -> None:
            w = h.fresh()
            (w / WI_BODY_REL).write_text(WI_BODY + "\n<!-- drift -->\n")
            r = h.impl(w, "start", str(WI))
            hit = h.red_on(r, "P0")
            named = "digest" in h.row_block(r, "P0")
            h.record("behavior phases refuse a stale fetch receipt",
                     "FAIL on P0 naming the body digest",
                     f"P0 red: {hit}; names digest: {named}",
                     hit and named and r.returncode != 0)

        def impl_before_e2e() -> None:
            """The write order, and the name of what is missing.

            Nothing is landed, so the `impl` phase is being opened first. A
            `P4` that read the filesystem would find a perfectly good crate
            here and pass; the evidence has to be the commit.

            It also has to *name* `e2e` rather than report that something is
            absent. With two phases there is only one predecessor to name, so
            the old ladder's "which of the two" control has nowhere left to
            stand -- what survives is the claim that the message is actionable
            rather than a boolean.
            """
            w = h.fresh()
            write_tests(w)
            write_impl(w)
            r = h.impl(w, "verify", str(WI))
            hit = h.red_on(r, "P4")
            names = "e2e" in h.row_block(r, "P4")
            h.record("the impl phase cannot open before the e2e phase landed",
                     "FAIL on P4 naming `e2e`",
                     f"P4 red: {hit}; names e2e: {names}",
                     hit and names and r.returncode != 0)

        def phase_already_landed() -> None:
            w = h.staged(through="impl")
            r = h.impl(w, "commit", str(WI))
            if r.returncode != 0:
                h.record("verify refuses a phase that already landed",
                         "the phase lands first", r.stdout.strip()[-200:], False)
                return
            (w / TESTS_REL).write_text(TESTS_SRC + "\n// and more\n")
            r = h.impl(w, "verify", str(WI))
            hit = h.red_on(r, "P3")
            h.record("verify refuses a phase that already landed", "FAIL on P3",
                     "red on it" if hit else r.stdout.strip()[-200:],
                     hit and r.returncode != 0)

        def e2e_phase_writes_src() -> None:
            w = h.fresh()
            (w / CARGO_REL).write_text(CARGO_TOML_REGISTERED)
            (w / CASE_REL).write_text(CASE_SRC)
            (w / LIB_REL).write_text(LIB_SKELETON)
            r = h.e2e(w, "verify", str(WI))
            hit = h.red_on(r, "C0")
            h.record("the e2e phase may not write src/", "FAIL on C0",
                     "red on it" if hit else r.stdout.strip()[-200:],
                     hit and r.returncode != 0)

        def impl_phase_writes_no_test_file() -> None:
            """The half of the filename rule that survived the merge.

            An `impl` phase that touched no `tests.rs` at all wrote no test, so
            there is nothing for `red` to measure and nothing for `T2` to be
            the oracle over. `C0` refuses it here rather than letting `R2`
            report an empty red, which would name the wrong defect: it would
            read as "the implementation was written first" when what happened
            is that no invariant was ever written down.
            """
            w = h.staged(through="e2e")
            (w / LIB_REL).write_text(mutate(
                LIB_IMPLEMENTED, "#[cfg(test)]\nmod tests;\n\n", ""))
            (w / MAIN_REL).write_text(MAIN_IMPLEMENTED)
            r = h.impl(w, "verify", str(WI))
            hit = h.red_on(r, "C0")
            h.record("the impl phase must write at least one tests.rs",
                     "FAIL on C0", "red on it" if hit else r.stdout.strip()[-200:],
                     hit and r.returncode != 0)

        def folded_into_the_e2e_commit() -> None:
            """Two phases folded into one commit is one phase.

            The colocated test is committed under the `e2e` phase's subject, so
            the tree looks complete and the log carries a `Refs #1`. `P4` reads
            the subject prefix and is satisfied -- the predecessor did land.
            What is left of the `impl` phase is an implementation with no test
            file in its own change, and `C0`'s existence half is the row that
            sees it.

            This is the control that shows why that half is load-bearing now.
            In the three-phase ladder the fold was caught at `P4`, because the
            folded commit was labelled `e2e` and the `unit` phase had therefore
            never landed. With two phases that depth is gone, and `C0` is what
            is left standing between a fold and a green.
            """
            w = h.fresh()
            (w / CARGO_REL).write_text(CARGO_TOML_REGISTERED)
            (w / CASE_REL).write_text(CASE_SRC)
            write_tests(w)
            git_checked(w, "add", "-A")
            git_checked(w, "commit", "-qm",
                f"e2e(demo): pin the behaviour and the invariant\n\nRefs #{WI}")
            write_impl(w)
            r = h.impl(w, "verify", str(WI))
            hit = h.red_on(r, "C0")
            h.record("the impl phase folded into the e2e commit is refused",
                     "FAIL on C0", "red on it" if hit else r.stdout.strip()[-200:],
                     hit and r.returncode != 0)

        # -- R1..R3: the verb the merge turns on -------------------------------

        def red_is_a_build_failure() -> None:
            """The control this verb exists for.

            The test calls `marker()` and no skeleton is written, so the crate
            does not compile. `cargo test` exits non-zero for that exactly as
            it does for a failed assertion, so a verb reading one exit code
            would record a red here and record it as evidence. `R1` is the row
            that separates the two, and the claim is both that it is red *and*
            that `R2` and `R3` did not run -- a named red computed from a build
            that never produced a binary would be a red over an empty set.

            They have to say PENDING rather than vanish. A row that disappears
            from a report and a row that passed are the same thing to whoever
            reads the summary, and "the check is absent" is the failure this
            whole file is written against.
            """
            w = h.staged(through="e2e")
            (w / LIB_REL).write_text(LIB_SCAFFOLD + "\n#[cfg(test)]\nmod tests;\n")
            (w / TESTS_REL).write_text(TESTS_SRC)
            r = h.impl(w, "red", str(WI))
            hit = h.red_on(r, "R1")
            pending = [row for row in ("R2", "R3")
                       if "PENDING" not in h.row_block(r, row)]
            wrote = (w / RECORD_REL).is_file()
            h.record("a build failure is refused, and records nothing",
                     "FAIL on R1, PENDING on R2 and R3, no record on disk",
                     f"R1 red: {hit}; not pending: {pending or 'none'}; "
                     f"record written: {wrote}",
                     hit and not pending and not wrote and r.returncode != 0)

        def red_selector_matches_nothing() -> None:
            """A test command that runs nothing exits zero and reads as green.

            The declared `test` command is narrowed to a filter matching no
            test. It compiles, so `R1` is green; it exits 0 having run nothing,
            so a verb reading the exit code would call that "no failures" and
            record an empty red. `R2` refuses an empty set by name.
            """
            w = h.staged(through="e2e")
            write_tests(w)
            (w / PROJECT_TOML_REL).write_text(mutate(
                PROJECT_TOML,
                'apps/demo/Cargo.toml"\nharness',
                'apps/demo/Cargo.toml no_such_test"\nharness'))
            bake(w, PROJECT_TOML_REL)
            r = h.impl(w, "red", str(WI))
            hit = h.red_on(r, "R2")
            wrote = (w / RECORD_REL).is_file()
            h.record("a selector matching nothing is not a red", "FAIL on R2",
                     f"R2 red: {hit}; record written: {wrote}"
                     if hit else r.stdout.strip()[-200:],
                     hit and not wrote and r.returncode != 0)

        def red_after_the_implementation() -> None:
            """The implementation written before any red was measured.

            This is the defect the retired `unit`/`logic` filename boundary
            refused structurally: with the test and the code in one phase,
            nothing about the file layout stops someone writing the code first
            and the test after. What stops it is that there is then no moment
            at which a failing test exists to record -- the test passes the
            instant it is written.

            `R2` is the row that says so, and the claim is that its message
            names *that*, not merely that a set was empty. A caller told "no
            failures" would go looking for a broken selector; a caller told
            "the implementation is already written" knows the red cannot be
            manufactured after the fact.

            `R3` is red here too, and that is not redundancy: it is the same
            defect seen from the contract's side, and it is the row that
            survives when the test was written first but written wrong.
            """
            w = h.staged(through="e2e")
            write_tests(w)
            write_impl(w)
            r = h.impl(w, "red", str(WI))
            hit = h.red_on(r, "R2")
            block = h.row_block(r, "R2")
            named = "implementation is already written" in block
            also = h.red_on(r, "R3")
            wrote = (w / RECORD_REL).is_file()
            h.record("red refuses a tree whose implementation is already there",
                     "FAIL on R2 naming the implementation, and on R3",
                     f"R2 red: {hit}; names it: {named}; R3 red: {also}; "
                     f"record written: {wrote}",
                     hit and named and also and not wrote and r.returncode != 0)

        def red_with_the_behaviour_already_shipped() -> None:
            """A real named red over an implementation that is already done.

            `R1` and `R2` cannot see this. The crate compiles, and the test
            fails -- because the test is wrong, not because the behaviour is
            missing. That is a perfectly ordinary red to `R2`'s subtraction,
            and a verb with only those two rows would record it and let the
            phase claim the implementation afterwards.

            `R3` is the row that catches it, by observing that the e2e cases
            have already stopped refusing the product. Note the shape: the
            record is what `test` measures against, so a red admitted here is a
            red admitted for the whole phase.
            """
            w = h.staged(through="e2e")
            write_tests(w, tests=TESTS_SRC_WRONG)
            write_impl(w)
            r = h.impl(w, "red", str(WI))
            hit = h.red_on(r, "R3")
            green = not h.red_on(r, "R2")
            wrote = (w / RECORD_REL).is_file()
            h.record("red refuses a contract that already accepts the product",
                     "FAIL on R3 with R2 green, and no record on disk",
                     f"R3 red: {hit}; R2 green: {green}; record written: {wrote}",
                     hit and green and not wrote and r.returncode != 0)

        def red_accumulates_across_runs() -> None:
            """The TDD loop, which the three-phase ladder could not express.

            Two reds in one phase: `marker` is written down, measured red, and
            satisfied; then `shout` is written down and measured red. On that
            second run `marker_is_hello` is *passing*, so a subtraction against
            HEAD does not name it -- and a record that replaced its contents
            would drop the attribution the first run earned. The union is what
            keeps it.

            `R3` says PENDING on the second run rather than passing. The cases
            going green from here is what this phase is for, so a row that
            reported PASS both before and after any implementation existed
            would be reporting nothing.
            """
            w = h.staged(through="e2e")
            write_tests(w, lib=LIB_SKELETON_TWO)
            first = h.impl(w, "red", str(WI))
            opened = sorted(read_record(w).get("names", []))
            write_tests(w, tests=TESTS_SRC_TWO, lib=LIB_HALF)
            second = h.impl(w, "red", str(WI))
            kept = sorted(read_record(w).get("names", []))
            held = "PENDING" in h.row_block(second, "R3")
            want = sorted([UNIT_TEST, SHOUT_TEST])
            h.record("a second red keeps the first one's names",
                     f"exit 0 twice, {[UNIT_TEST]} then {want}, R3 pending",
                     f"exit {first.returncode}/{second.returncode}; "
                     f"{opened} then {kept}; R3 pending: {held}",
                     first.returncode == 0 and second.returncode == 0
                     and opened == [UNIT_TEST] and kept == want and held)

        # -- C2: the recorded red is this tree's -------------------------------

        def no_red_on_file() -> None:
            """`red` was never run at all.

            Every row below `C2` reads the recorded names, so an absent record
            is not a green with nothing to check -- it is a phase with no
            attribution whatsoever. The message has to send the caller to
            `red`, because that is the only verb that can produce one and the
            only moment at which it can.
            """
            w = h.staged(through="e2e")
            write_tests(w)
            write_impl(w)
            r = h.impl(w, "verify", str(WI))
            hit = h.red_on(r, "C2")
            named = "red" in h.row_block(r, "C2")
            h.record("verify refuses a phase that never measured a red",
                     "FAIL on C2 pointing at `red`",
                     f"C2 red: {hit}; names the verb: {named}",
                     hit and named and r.returncode != 0)

        def head_moved_after_red() -> None:
            """The subtraction was against a commit that is no longer below.

            `R2` computes its names by subtracting HEAD's failures from the
            tree's. Move HEAD and that subtraction describes a different tree:
            a test already failing at the new HEAD would still be sitting in
            the record as this change's red. Nothing else here can see it --
            the names are still real, the tests still exist, the files are
            untouched.
            """
            w = h.staged(through="impl")
            (w / "NOTES.md").write_text("something unrelated landed.\n")
            git_checked(w, "add", "--", "NOTES.md")
            git_checked(w, "commit", "-qm", "chore: an unrelated commit")
            r = h.impl(w, "verify", str(WI))
            hit = h.red_on(r, "C2")
            named = "HEAD" in h.row_block(r, "C2")
            h.record("C2 refuses a red measured against a HEAD that has moved",
                     "FAIL on C2 naming HEAD",
                     f"C2 red: {hit}; names HEAD: {named}",
                     hit and named and r.returncode != 0)

        def test_file_edited_after_red() -> None:
            """The retrofit the retired filename boundary used to refuse.

            The test is edited into agreement with the implementation after the
            red was recorded. In the three-phase ladder this was structural --
            the `logic` phase was refused `tests.rs` by filename, and the edit
            could not be made at all. One phase cannot make that rule: the same
            file is legitimately written earlier in the same phase.

            So the rule became a measurement. `C2` compares the bytes `red`
            measured against the bytes that are there, names the file, and
            sends the caller back to `red` -- which over a test edited into
            passing finds nothing failing. The refusal is one step further
            away, and it is the same refusal.
            """
            w = h.staged(through="impl")
            (w / TESTS_REL).write_text(TESTS_SRC_WRONG)
            r = h.impl(w, "verify", str(WI))
            hit = h.red_on(r, "C2")
            block = h.row_block(r, "C2")
            named = TESTS_REL in block
            sends = "red" in block
            h.record("C2 refuses a test file edited after the red was measured",
                     "FAIL on C2 naming tests.rs and pointing at `red`",
                     f"C2 red: {hit}; names the file: {named}; sends to red: {sends}",
                     hit and named and sends and r.returncode != 0)

        def test_file_reformatted_after_red() -> None:
            """Whitespace is not an exemption.

            The test is reindented and nothing else. It still passes, it still
            asserts the same thing, and it is still an edit to the artifact the
            recorded names were measured over. A check that compared behaviour
            rather than bytes would let this through, and "it was only
            whitespace" is the sentence every retrofit starts with.

            The cost is real and deliberate: a formatter run mid-phase sends
            you back to `red`. That is one command, and it re-measures a tree
            where the tests genuinely still fail, so the honest loop pays a few
            seconds and the dishonest one hits an empty set.
            """
            w = h.staged(through="impl")
            (w / TESTS_REL).write_text(
                mutate(TESTS_SRC, "    assert_eq!", "\tassert_eq!"))
            r = h.impl(w, "verify", str(WI))
            hit = h.red_on(r, "C2")
            named = TESTS_REL in h.row_block(r, "C2")
            h.record("a whitespace-only edit to tests.rs is still detected",
                     "FAIL on C2 naming tests.rs",
                     f"C2 red: {hit}; names the file: {named}",
                     hit and named and r.returncode != 0)

        # -- T1..T5: the recorded names are the oracle -------------------------

        def impl_does_not_compile() -> None:
            """The same separation `R1` makes, on the other side of the phase.

            Every row below `T1` reads test names out of an output a build
            failure never produced, and the claim here is both that `T1` is red
            and that the four rows under it say PENDING rather than going
            quiet: a row missing from a report and a row that passed look
            identical in a summary.
            """
            w = h.staged(through="impl")
            (w / LIB_REL).write_text(mutate(
                LIB_IMPLEMENTED, f'"{MARKER}"', "no_such_function()"))
            r = h.impl(w, "test", str(WI))
            hit = h.red_on(r, "T1")
            pending = [row for row in ("T2", "T3", "T4", "T5")
                       if "PENDING" not in h.row_block(r, row)]
            h.record("a build failure stops the test verb, loudly",
                     "FAIL on T1 and PENDING on T2..T5",
                     f"T1 red: {hit}; not pending: {pending or 'none'}",
                     hit and not pending and r.returncode != 0)

        def recorded_test_still_red() -> None:
            """The implementation does not satisfy the test that was recorded."""
            w = h.staged(through="impl")
            (w / LIB_REL).write_text(mutate(
                LIB_IMPLEMENTED, f'"{MARKER}"', '"goodbye"'))
            r = h.impl(w, "test", str(WI))
            hit = h.red_on(r, "T2")
            named = UNIT_TEST in h.row_block(r, "T2")
            h.record("T2 refuses an implementation the recorded test rejects",
                     f"FAIL on T2 naming {UNIT_TEST}",
                     f"T2 red: {hit}; names it: {named}",
                     hit and named and r.returncode != 0)

        def recorded_test_deleted() -> None:
            """The test that was measured is not the test that is being run.

            `C2` catches an edit to the *bytes* of a test file, and this is the
            case where that is not enough on its own: the file is edited, so
            `C2` is red, but the row that has to keep meaning something is `T2`
            -- because it checks the recorded names are *present* rather than
            counting failures. A deleted test cannot fail, and a phase that
            counted failures would read its absence as a pass.

            Run through `commit` rather than `test`, because that is where the
            two rows have to hold together: `C2` refuses the run, and `T2`
            says PENDING with the reason rather than reporting a green over a
            test that is gone.
            """
            w = h.staged(through="impl")
            (w / TESTS_REL).write_text(TESTS_SRC_TRUNCATED)
            r = h.impl(w, "commit", str(WI))
            hit = h.red_on(r, "C2")
            held = "PENDING" in h.row_block(r, "T2")
            landed = h.trailer(w, "Impl-Red")
            h.record("a recorded test deleted after measuring cannot commit",
                     "FAIL on C2, PENDING on T2, nothing committed",
                     f"C2 red: {hit}; T2 pending: {held}; trailer {landed!r}",
                     hit and held and not landed and r.returncode != 0)

        def impl_breaks_a_neighbouring_test() -> None:
            """A green bought somewhere else in the suite.

            The recorded test passes, so `T2` is green and has nothing to say:
            it is scoped to the names this work item produced. The
            implementation also changed a neighbouring module and broke its
            test, which is what `T3`'s first half is for.
            """
            w = h.staged(through="impl")
            (w / NEIGHBOUR_REL).write_text(
                mutate(NEIGHBOUR_SRC, "n * 2", "n * 3"))
            r = h.impl(w, "test", str(WI))
            hit = h.red_on(r, "T3")
            green = not h.red_on(r, "T2")
            named = NEIGHBOUR_TEST in h.row_block(r, "T3")
            h.record("T3 refuses a regression outside the recorded names",
                     f"FAIL on T3 naming {NEIGHBOUR_TEST}, T2 green",
                     f"T3 red: {hit}; names it: {named}; T2 green: {green}",
                     hit and named and green and r.returncode != 0)

        def impl_unwires_a_neighbouring_test() -> None:
            """The one failure mode nothing else here can see.

            The neighbouring test is not broken and not edited -- its `mod`
            declaration is removed from a production file this phase is
            entirely allowed to write. So `C0` is green, `C2` is green (no
            recorded test file moved), `T1` is green, `T2` is green (the
            recorded name still passes), and the suite reports zero failures,
            because a test that does not run cannot fail. Only comparing the
            set of tests that ran here against the set that ran at `HEAD` finds
            it.
            """
            w = h.staged(through="impl")
            (w / NEIGHBOUR_REL).write_text(mutate(
                NEIGHBOUR_SRC, "#[cfg(test)]\nmod tests;\n\n", ""))
            r = h.impl(w, "test", str(WI))
            hit = h.red_on(r, "T3")
            named = NEIGHBOUR_TEST in h.row_block(r, "T3")
            green = not h.red_on(r, "T2")
            h.record("T3 refuses a test silently unwired from the build",
                     f"FAIL on T3 naming {NEIGHBOUR_TEST}, T2 green",
                     f"T3 red: {hit}; names it: {named}; T2 green: {green}",
                     hit and named and green and r.returncode != 0)

        def case_already_green_at_head() -> None:
            """A case that was green before the implementation existed.

            `T4` is green -- the case passes in the working tree -- and it is
            green for a reason that has nothing to do with this change. Only
            `T5`, which runs the same case against `HEAD`, can tell "this made
            it pass" from "it was already passing".

            This is also the row the Milestone batch order runs into. `e2e-for`
            over a Milestone lands every child's cases before any child's `impl`
            phase starts, so a sibling that goes first can leave this child's
            cases already green. `T5` refuses that, and its message says so --
            which makes it a report about the child's cases not
            discriminating, not a broken gate.
            """
            w = h.fresh()
            land_e2e(w, case=CASE_SRC_BLIND)
            write_tests(w)
            land_red(w)
            write_impl(w)
            r = h.impl(w, "test", str(WI))
            hit = h.red_on(r, "T5")
            green = not h.red_on(r, "T4")
            h.record("T5 refuses a case that was already green at HEAD",
                     "FAIL on T5 with T4 still green",
                     f"T5 red: {hit}; T4 green: {green}",
                     hit and green and r.returncode != 0)

        # -- outcomes ----------------------------------------------------------

        def commit_lands_the_phase() -> None:
            """One commit carrying the tests and the implementation together.

            Three claims, and the third is the one the merge added. The commit
            is exactly this phase's paths; the names that were recorded are on
            it; and the record under `.aw/` is *gone*, because it described a
            HEAD this commit has just moved off. Leaving it would let the next
            run read a stale file as a live one, and `C2`'s staleness row would
            catch that only by the coincidence that the shas differ.
            """
            w = h.staged(through="impl")
            r = h.impl(w, "commit", str(WI))
            left = git(w, "status", "--porcelain", "-uall").stdout.strip()
            landed = sorted(
                line for line in
                git(w, "show", "--name-only", "--format=", "HEAD").stdout.split())
            digest = h.trailer(w, "Impl-Change-Digest")
            names = h.trailer(w, "Impl-Red")
            cases = h.trailer(w, "Impl-Contract")
            cleared = not (w / RECORD_REL).is_file()
            ok = (r.returncode == 0 and left == ""
                  and landed == sorted([LIB_REL, MAIN_REL, TESTS_REL])
                  and names == UNIT_TEST and cases == CASE
                  and len(digest) == 64 and cleared)
            h.record("commit lands the phase, records it, and clears the record",
                     "clean tree, three paths, both trailers, a 64-char digest, "
                     "no record left",
                     f"exit {r.returncode}; left {left!r}; landed {landed}; "
                     f"Impl-Red {names!r}; Impl-Contract {cases!r}; "
                     f"digest {len(digest)} chars; record cleared: {cleared}",
                     ok)

        h.run([
            # -- the ladder ----------------------------------------------------
            start_clean,
            start_dirty,
            behavior_phase_refuses_maintenance_receipt,
            behavior_phase_refuses_stale_fetch_body,
            impl_before_e2e,
            phase_already_landed,
            e2e_phase_writes_src,
            impl_phase_writes_no_test_file,
            folded_into_the_e2e_commit,

            # -- R1..R3 --------------------------------------------------------
            red_is_a_build_failure,
            red_selector_matches_nothing,
            red_after_the_implementation,
            red_with_the_behaviour_already_shipped,
            red_accumulates_across_runs,

            # -- C2 ------------------------------------------------------------
            no_red_on_file,
            head_moved_after_red,
            test_file_edited_after_red,
            test_file_reformatted_after_red,

            # -- T1..T5 --------------------------------------------------------
            impl_does_not_compile,
            recorded_test_still_red,
            recorded_test_deleted,
            impl_breaks_a_neighbouring_test,
            impl_unwires_a_neighbouring_test,
            case_already_green_at_head,

            # -- outcomes ------------------------------------------------------
            commit_lands_the_phase,
        ])

        print(f"\n{'':8s} {'control':66s} observation")
        for _, status, name, detail in h.results:
            print(f"{status:8s} {name:66s} {detail}")
        failed = [x for x in h.results if x[1] != "PASS"]
        print(f"\n{len(h.results) - len(failed)}/{len(h.results)} "
              "controls behaved as declared")
        return 1 if failed else 0
    finally:
        shutil.rmtree(root, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
