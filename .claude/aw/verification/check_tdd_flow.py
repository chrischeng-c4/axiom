#!/usr/bin/env python3
"""Every row the three phase scripts can refuse, driven through the gate.

`e2e.py`, `unit.py` and `logic.py` replace the `ec -> td -> cb` ladder. The
design decisions this file pins, because it is the first consumer able to
refuse them:

  The TD leg is gone, and with it the stage machinery. The deleted `ec.py` carried an
  `applicability` axis and an `AW_EC_STAGE` branch because the TD leg could
  only ever observe a document -- a case had to be told which of two very
  different things it was looking at. With no TD leg every case runs against
  the implementation at every phase, always, so a case needs no stage branch
  and there is no `B0`-shaped row asking whether the contract reaches this
  phase. Cases here are plain: one `verify()`, no environment read.

  The unit tests live in `src/**/tests.rs`, wired in with `#[cfg(test)] mod
  tests;`. This is what makes the `unit` and `logic` phases separable at all.
  They write the same tree -- `CLAUDE.md` sends anything observable only from
  inside the implementation to colocated tests, so moving them to `tests/`
  would cost the private access that is the whole point -- and a scope check
  that had to tell them apart *within* a file would need to decide what a
  `#[cfg(test)]` span is. That decision has been wrong in this repo before:
  item-level `#[cfg(test)] fn` reads as production, and a brace scanner that
  does not strip `r#"..."#` reads fixture text as production. A filename is
  not a judgement call.

  The `todo!()` skeleton belongs to the `unit` phase, and the `logic` phase
  must be free to replace it. That is why the skeleton lives outside
  `tests.rs`: an attribution check over everything the unit phase wrote would
  refuse the normal path, since replacing `todo!()` is the work.

  A red is `build green, named test failed`. `[unit] build` and `[unit] test`
  are two commands, not one, because `cargo test` exits non-zero for a compile
  error exactly as it does for a failed assertion, and a red that is really a
  compile error proves nothing about the test -- the test never ran.

  The red is recorded as names on the commit, never as a state file. `Unit-Red`
  and `E2E-Red` trailers travel with the commit that produced them, so a phase
  rebased away takes its evidence with it.

The controls:

  1 positive control    -- the three phases run to commit against an unmutated
                          fixture. Run first and alone: if the ladder is not
                          green to begin with, every red below is already there.

  8 ladder controls     -- the preconditions, wired against three roots. `P4`
                          can fail at two different depths here, and `C0` is
                          the row that separates `unit` from `logic`.

  4 unit controls       -- the phase this whole lifecycle was added for. `U1`'s
                          control is the one it exists for: a test naming a
                          function nobody wrote yet is a compile error, and a
                          compile error accepted as a red is a phase that
                          measured nothing.

  7 logic controls      -- the recorded names are the oracle; the rest of the
                          suite is not collateral; and the case that accepts
                          the implementation is a case that refused `HEAD`.

  2 outcome controls    -- the names reach the commit, and a phase cannot be
                          folded into its predecessor.
"""
from __future__ import annotations

import concurrent.futures
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import threading

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import (E2E_SCRIPT, LOGIC_SCRIPT, UNIT_SCRIPT,  # noqa: E402
                    pinned_interpreter)

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
WI_BODY_REL = f".aw/workitems/changes/{WI}.md"

# The one behaviour, held as a constant so a control cannot mutate the
# implementation into agreeing with a test that is looking for something else.
MARKER = "hello"

# The unit test's name as `cargo` prints it. Both `U2` and `L1` are claims about
# this exact string reaching a commit trailer and being read back out of one, so
# a control that needs to break the recording has something to compare against.
UNIT_TEST = "tests::marker_is_hello"

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
# The crate, in the three states the three phases leave it in.
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

# What the `unit` phase writes into `lib.rs`: the module wiring and the
# skeleton. `todo!()` types as `!` and coerces, so this compiles -- which is the
# entire point. The test that calls it panics at run time, and a run-time panic
# is a red the phase can attribute to itself.
LIB_SKELETON = '''\
//! The demo crate.

pub mod extra;

#[cfg(test)]
mod tests;

/// The marker string the product writes.
pub fn marker() -> &'static str {
    todo!("the logic phase writes this")
}
'''

# What the `logic` phase writes into `lib.rs`.
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

# What the `logic` phase writes into `main.rs`: the externally observable half,
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
# separate the two phases by filename instead of by span.
TESTS_SRC = f'''\
use super::marker;

#[test]
fn marker_is_hello() {{
    assert_eq!(marker(), "{MARKER}");
}}
'''

# A second colocated test, used by the control that checks a phase cannot
# quietly drop a test it was measured against.
TESTS_SRC_TRUNCATED = '''\
use super::marker;
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
# which is `L3`'s defect arriving through the back door inside a single phase.
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
# and green in the tree, so `L4` cannot tell the two apart and only `L5` sees
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

# The project config. `[unit] build` and `[unit] test` are two keys rather than
# one because they answer different questions -- see the note at the top -- and
# `harness` names the parser that turns the second one's output into names.
# There is no default for any of the three: a project that never answered gets
# a refusal, not the weaker gate.
PROJECT_TOML = '''\
[project]
name = "demo"

[unit]
build = "cargo test --offline --lib --no-run --manifest-path apps/demo/Cargo.toml"
test = "cargo test --offline --lib --manifest-path apps/demo/Cargo.toml"
harness = "cargo"
'''

# `.aw/` carries the staged body, `target/` is build output and `marker.txt` is
# product output. Mirroring the real checkout's ignores is not cosmetic: any of
# them reported by `git status` would join the dirty set and fail `C0` in every
# control below -- the right row red for entirely the wrong reason.
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
- `{LIB_REL}:1` carries no `marker` function, so the unit test has nothing to
  call until the skeleton lands.

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
| 1 | `logic.py test 1` | the unit test panics and the case is red | `L2` green over `{UNIT_TEST}` and `L5` red at HEAD | the same case is run against HEAD and against the tree, so a case observing nothing is green in both and the second row refuses it |

### Negative control

Replace `"{MARKER}"` in `{LIB_REL}` with `"goodbye"` and re-run the command
above; row `L2` must go red naming `{UNIT_TEST}`. Restore the file
byte-for-byte and confirm sha256
`0000000000000000000000000000000000000000000000000000000000000000` before
continuing.

## Never

This addresses the worker implementing this work item, not the controller reviewing it.

### Must not touch

- `{E2E_REL}/**`
- `{NEIGHBOUR_REL}`
- `{NEIGHBOUR_TESTS_REL}`

`{TESTS_REL}` is deliberately absent from this list even though the logic phase
may not write it. The change-point list above spans all three phases, so a path
one phase writes and the next may not is not a must-not-touch -- it is a
per-phase scope, and `C0` is the row that holds it. Writing it here as well
would state the same thing in a place that cannot tell the phases apart.

### Must not do

- Do not make the case green by editing the case's assertion.
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


def git(work: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-c", "core.fsmonitor=false", *args],
                          cwd=work, capture_output=True, text=True)


def bake(work: Path, *paths: str) -> None:
    """Commit a fixture mutation so it is not part of the change under test.

    Several controls change the project config, which lives outside `src/`.
    Left uncommitted it would be a path from no phase at all, and `C0` would
    refuse the whole change before the row the control names got to decide
    anything. The control would still see a red -- but it would go on passing
    after its own row stopped working, which is the one thing a control may
    not do.
    """
    git(work, "add", "-A", "--", *paths)
    git(work, "commit", "-qm", "chore: adjust the fixture around the change")


def build(root: Path) -> Path:
    """A checkout scaffolded and committed, with all three phases still open.

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


def land_e2e(work: Path) -> None:
    """Write and commit the `e2e` phase's output by hand.

    By hand rather than by running the phase, because a control aimed at a
    later phase must not depend on an earlier one passing: if `e2e.py commit`
    breaks, every `unit` and `logic` control would go red naming a row that is
    working. The trailer is written here in the shape the phase produces, and
    the control that checks the phase *does* produce it is separate.
    """
    (work / CARGO_REL).write_text(CARGO_TOML_REGISTERED)
    (work / CASE_REL).write_text(CASE_SRC)
    git(work, "add", "-A")
    git(work, "commit", "-qm",
        f"e2e(demo): pin the marker behaviour\n\nRefs #{WI}\nE2E-Red: {CASE}")


def land_unit(work: Path, *, tests: str = TESTS_SRC,
              lib: str = LIB_SKELETON, red: str = UNIT_TEST) -> None:
    """Write and commit the `unit` phase's output by hand. See `land_e2e`."""
    (work / LIB_REL).write_text(lib)
    (work / TESTS_REL).write_text(tests)
    git(work, "add", "-A")
    git(work, "commit", "-qm",
        f"unit(demo): pin the marker invariant\n\nRefs #{WI}\nUnit-Red: {red}")


def write_logic(work: Path) -> None:
    """The `logic` phase's output, left uncommitted -- which is its real shape."""
    (work / LIB_REL).write_text(LIB_IMPLEMENTED)
    (work / MAIN_REL).write_text(MAIN_IMPLEMENTED)


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
        """A copy with the phases up to and including `through` already landed."""
        work = self.fresh()
        land_e2e(work)
        if through in ("unit", "logic"):
            land_unit(work)
        if through == "logic":
            write_logic(work)
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

    def unit(self, work: Path, *args: str, **kw) -> subprocess.CompletedProcess:
        return self.phase(UNIT_SCRIPT, work, *args, **kw)

    def logic(self, work: Path, *args: str, **kw) -> subprocess.CompletedProcess:
        return self.phase(LOGIC_SCRIPT, work, *args, **kw)

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
        # All three phases, in order, through `commit` -- because the thing
        # under test is the ladder, and a positive control that only ran the
        # last phase would leave the first two unmeasured in the one place they
        # are supposed to be green.
        w = h.fresh()

        steps = [
            ("e2e", lambda: (
                (w / CARGO_REL).write_text(CARGO_TOML_REGISTERED),
                (w / CASE_REL).write_text(CASE_SRC),
                h.e2e(w, "commit", str(WI)))[-1]),
            ("unit", lambda: (
                (w / LIB_REL).write_text(LIB_SKELETON),
                (w / TESTS_REL).write_text(TESTS_SRC),
                h.unit(w, "commit", str(WI)))[-1]),
            ("logic", lambda: (write_logic(w), h.logic(w, "commit", str(WI)))[-1]),
        ]
        for name, step in steps:
            r = step()
            if r.returncode != 0:
                print(r.stdout)
                print(r.stderr, file=sys.stderr)
                print(f"the positive control failed at the {name} phase; every "
                      "negative control below would be uninterpretable, so they "
                      "are not run.", file=sys.stderr)
                return 1
        h.record("positive control: the three phases run to commit",
                 "exit 0 from all three", "exit 0 from all three", True)

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

        def unit_before_e2e() -> None:
            """The write order, at the nearest depth.

            Nothing is landed, so the `unit` phase is being opened first. A
            `P4` that read the filesystem would find a perfectly good crate
            here and pass; the evidence has to be the commit.
            """
            w = h.fresh()
            (w / LIB_REL).write_text(LIB_SKELETON)
            (w / TESTS_REL).write_text(TESTS_SRC)
            r = h.unit(w, "verify", str(WI))
            hit = h.red_on(r, "P4")
            h.record("the unit phase cannot open before the e2e phase landed",
                     "FAIL on P4", "red on it" if hit else r.stdout.strip()[-200:],
                     hit and r.returncode != 0)

        def logic_before_unit() -> None:
            """The write order, at the far depth, with the near one satisfied.

            The `e2e` phase has landed and the `unit` phase has not, so `P4`
            has to name the one that is missing rather than reporting that
            something is. Two predecessors is the case the first phase cannot
            have.
            """
            w = h.staged(through="e2e")
            write_logic(w)
            r = h.logic(w, "verify", str(WI))
            hit = h.red_on(r, "P4")
            names = "unit" in h.row_block(r, "P4")
            h.record("the logic phase names which predecessor is missing",
                     "FAIL on P4 naming `unit`",
                     f"P4 red: {hit}; names unit: {names}",
                     hit and names and r.returncode != 0)

        def phase_already_landed() -> None:
            w = h.staged(through="unit")
            (w / TESTS_REL).write_text(TESTS_SRC + "\n// and more\n")
            r = h.unit(w, "verify", str(WI))
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

        def logic_phase_writes_tests_rs() -> None:
            """The row that separates the two phases sharing one root.

            `unit` and `logic` both write `apps/demo/src/`, so the prefix half
            of `C0` cannot tell them apart. The filename half can, and this is
            the control for it: the logic phase edits `tests.rs` -- the retrofit
            the whole three-phase split exists to refuse -- and `C0` has to see
            it while every other row stays green.
            """
            w = h.staged(through="logic")
            (w / TESTS_REL).write_text(mutate(TESTS_SRC, f'"{MARKER}"', '"goodbye"'))
            r = h.logic(w, "verify", str(WI))
            hit = h.red_on(r, "C0")
            named = TESTS_REL in h.row_block(r, "C0")
            h.record("the logic phase may not write tests.rs",
                     "FAIL on C0 naming tests.rs",
                     f"C0 red: {hit}; names the file: {named}",
                     hit and named and r.returncode != 0)

        def unit_phase_writes_only_the_skeleton() -> None:
            """The mirror of the row above, aimed the other way.

            A `unit` phase that touched no `tests.rs` at all wrote no test, so
            there is nothing for the `logic` phase to be measured against. `C0`
            refuses it here rather than letting `U2` report an empty red, which
            would name the wrong defect.
            """
            w = h.staged(through="e2e")
            (w / LIB_REL).write_text(mutate(
                LIB_IMPLEMENTED, "#[cfg(test)]\nmod tests;\n\n", ""))
            r = h.unit(w, "verify", str(WI))
            hit = h.red_on(r, "C0")
            h.record("the unit phase must write at least one tests.rs",
                     "FAIL on C0", "red on it" if hit else r.stdout.strip()[-200:],
                     hit and r.returncode != 0)

        # -- U0..U3: the phase this lifecycle was added for ---------------------

        def unit_red_is_a_build_failure() -> None:
            """The control this whole phase exists for.

            The test calls `marker()` and no skeleton is written, so the crate
            does not compile. `cargo test` exits non-zero for that exactly as
            it does for a failed assertion, so a phase reading one exit code
            would record a red here and record it as evidence. `U1` is the row
            that separates the two, and the claim is both that it is red *and*
            that `U2` did not run -- a named red computed from a build that
            never produced a binary would be a red over an empty set.

            `U2` has to say PENDING rather than vanish. A row that disappears
            from a report and a row that passed are the same thing to whoever
            reads the summary, and "the check is absent" is the failure this
            whole file is written against.
            """
            w = h.staged(through="e2e")
            (w / LIB_REL).write_text(LIB_SCAFFOLD + "\n#[cfg(test)]\nmod tests;\n")
            (w / TESTS_REL).write_text(TESTS_SRC)
            r = h.unit(w, "test", str(WI))
            hit = h.red_on(r, "U1")
            block = h.row_block(r, "U2")
            held = "PENDING" in block
            h.record("a build failure is refused, and stops the ladder",
                     "FAIL on U1 and PENDING on U2",
                     f"U1 red: {hit}; U2 pending: {held} ({block[:60]!r})",
                     hit and held and r.returncode != 0)

        def unit_test_selector_matches_nothing() -> None:
            """A test command that runs nothing exits zero and reads as green.

            The declared `test` command is narrowed to a filter matching no
            test. It compiles, so `U1` is green; it exits 0 having run nothing,
            so a phase reading the exit code would call that "no failures" and
            record an empty red. `U2` refuses an empty set by name.
            """
            w = h.staged(through="e2e")
            (w / LIB_REL).write_text(LIB_SKELETON)
            (w / TESTS_REL).write_text(TESTS_SRC)
            (w / PROJECT_TOML_REL).write_text(mutate(
                PROJECT_TOML,
                'apps/demo/Cargo.toml"\nharness',
                'apps/demo/Cargo.toml no_such_test"\nharness'))
            bake(w, PROJECT_TOML_REL)
            r = h.unit(w, "test", str(WI))
            hit = h.red_on(r, "U2")
            h.record("a selector matching nothing is not a red", "FAIL on U2",
                     "red on it" if hit else r.stdout.strip()[-200:],
                     hit and r.returncode != 0)

        def unit_phase_implements_the_behaviour() -> None:
            """The unit phase writing real logic instead of a skeleton.

            `U1` and `U2` cannot see this -- the crate compiles and the test is
            still red only if the test is wrong, and here it is not: the test
            goes green, which is what a phase that already did the work looks
            like. `U3` is the row that catches it, by observing that the e2e
            case stopped being red.
            """
            w = h.staged(through="e2e")
            (w / LIB_REL).write_text(LIB_IMPLEMENTED)
            (w / TESTS_REL).write_text(TESTS_SRC)
            (w / MAIN_REL).write_text(MAIN_IMPLEMENTED)
            r = h.unit(w, "test", str(WI))
            hit = h.red_on(r, "U3")
            h.record("the unit phase may not make the e2e case green",
                     "FAIL on U3", "red on it" if hit else r.stdout.strip()[-200:],
                     hit and r.returncode != 0)

        def unit_records_the_names() -> None:
            w = h.staged(through="e2e")
            (w / LIB_REL).write_text(LIB_SKELETON)
            (w / TESTS_REL).write_text(TESTS_SRC)
            r = h.unit(w, "commit", str(WI))
            got = h.trailer(w, "Unit-Red")
            h.record("commit records the failing test names on the commit",
                     f"Unit-Red: {UNIT_TEST}", f"exit {r.returncode}; {got!r}",
                     r.returncode == 0 and got == UNIT_TEST)

        # -- L1..L5: the recorded names are the oracle -------------------------

        def logic_does_not_compile() -> None:
            """The same separation `U1` makes, on the other side of the ladder.

            Every row below `L1` reads test names out of an output a build
            failure never produced, and the claim here is both that `L1` is red
            and that the four rows under it say PENDING rather than going
            quiet: a row missing from a report and a row that passed look
            identical in a summary.
            """
            w = h.staged(through="unit")
            (w / LIB_REL).write_text(mutate(
                LIB_IMPLEMENTED, f'"{MARKER}"', "no_such_function()"))
            r = h.logic(w, "test", str(WI))
            hit = h.red_on(r, "L1")
            pending = [row for row in ("L2", "L3", "L4", "L5")
                       if "PENDING" not in h.row_block(r, row)]
            h.record("a logic build failure stops the ladder, loudly",
                     "FAIL on L1 and PENDING on L2..L5",
                     f"L1 red: {hit}; not pending: {pending or 'none'}",
                     hit and not pending and r.returncode != 0)

        def recorded_test_still_red() -> None:
            """The implementation does not satisfy the test that was recorded."""
            w = h.staged(through="logic")
            (w / LIB_REL).write_text(mutate(
                LIB_IMPLEMENTED, f'"{MARKER}"', '"goodbye"'))
            r = h.logic(w, "test", str(WI))
            hit = h.red_on(r, "L2")
            named = UNIT_TEST in h.row_block(r, "L2")
            h.record("L2 refuses an implementation the recorded test rejects",
                     f"FAIL on L2 naming {UNIT_TEST}",
                     f"L2 red: {hit}; names it: {named}",
                     hit and named and r.returncode != 0)

        def unit_commit_amended_after_measuring() -> None:
            """The test that was measured is not the test that landed.

            `C0` catches an edit to `tests.rs` in the working tree. This is the
            other way in: the `unit` commit itself is amended, so the test it
            recorded a red for is no longer in the tree at all. Nothing looks
            wrong -- the diff against `HEAD` is exactly the implementation, and
            no test fails, because the test is gone. `L2` is the row that
            notices, and only because it checks the recorded names are
            *present* rather than counting failures.
            """
            w = h.staged(through="unit")
            (w / TESTS_REL).write_text(TESTS_SRC_TRUNCATED)
            git(w, "add", "-A")
            git(w, "commit", "-q", "--amend", "--no-edit")
            write_logic(w)
            r = h.logic(w, "test", str(WI))
            hit = h.red_on(r, "L2")
            named = UNIT_TEST in h.row_block(r, "L2")
            h.record("L2 refuses a unit commit amended after it was measured",
                     f"FAIL on L2 naming {UNIT_TEST}",
                     f"L2 red: {hit}; names it: {named}",
                     hit and named and r.returncode != 0)

        def logic_breaks_a_neighbouring_test() -> None:
            """A green bought somewhere else in the suite.

            The recorded test passes, so `L2` is green and has nothing to say:
            it is scoped to the names this work item produced. The
            implementation also changed a neighbouring module and broke its
            test, which is what `L3`'s first half is for.
            """
            w = h.staged(through="logic")
            (w / NEIGHBOUR_REL).write_text(
                mutate(NEIGHBOUR_SRC, "n * 2", "n * 3"))
            r = h.logic(w, "test", str(WI))
            hit = h.red_on(r, "L3")
            green = not h.red_on(r, "L2")
            named = NEIGHBOUR_TEST in h.row_block(r, "L3")
            h.record("L3 refuses a regression outside the recorded names",
                     f"FAIL on L3 naming {NEIGHBOUR_TEST}, L2 green",
                     f"L3 red: {hit}; names it: {named}; L2 green: {green}",
                     hit and named and green and r.returncode != 0)

        def logic_unwires_a_neighbouring_test() -> None:
            """The one failure mode nothing else here can see.

            The neighbouring test is not broken and not edited -- its `mod`
            declaration is removed from a production file the `logic` phase is
            entirely allowed to write. So `C0` is green (no test file is
            dirty), `L1` is green (it compiles), `L2` is green (the recorded
            name still passes), and the suite reports zero failures, because a
            test that does not run cannot fail. Only comparing the set of tests
            that ran here against the set that ran at `HEAD` finds it.
            """
            w = h.staged(through="logic")
            (w / NEIGHBOUR_REL).write_text(mutate(
                NEIGHBOUR_SRC, "#[cfg(test)]\nmod tests;\n\n", ""))
            r = h.logic(w, "test", str(WI))
            hit = h.red_on(r, "L3")
            named = NEIGHBOUR_TEST in h.row_block(r, "L3")
            green = not h.red_on(r, "L2")
            h.record("L3 refuses a test silently unwired from the build",
                     f"FAIL on L3 naming {NEIGHBOUR_TEST}, L2 green",
                     f"L3 red: {hit}; names it: {named}; L2 green: {green}",
                     hit and named and green and r.returncode != 0)

        def case_already_green_at_head() -> None:
            """A case that was green before the implementation existed.

            `L4` is green -- the case passes in the working tree -- and it is
            green for a reason that has nothing to do with this change. Only
            `L5`, which runs the same case against `HEAD`, can tell "this made
            it pass" from "it was already passing".
            """
            w = h.fresh()
            (w / CARGO_REL).write_text(CARGO_TOML_REGISTERED)
            (w / CASE_REL).write_text(CASE_SRC_BLIND)
            git(w, "add", "-A")
            git(w, "commit", "-qm",
                f"e2e(demo): pin the marker behaviour\n\nRefs #{WI}\nE2E-Red: {CASE}")
            land_unit(w)
            write_logic(w)
            r = h.logic(w, "test", str(WI))
            hit = h.red_on(r, "L5")
            green = not h.red_on(r, "L4")
            h.record("L5 refuses a case that was already green at HEAD",
                     "FAIL on L5 with L4 still green",
                     f"L5 red: {hit}; L4 green: {green}",
                     hit and green and r.returncode != 0)

        def logic_reformats_the_test() -> None:
            """Whitespace is not an exemption.

            The test is reindented and nothing else. It still passes, it still
            asserts the same thing, and it is still an edit to the artifact the
            phase is supposed to be satisfying. A check that compared behaviour
            rather than bytes would let this through, and "it was only
            whitespace" is the sentence every retrofit starts with.
            """
            w = h.staged(through="logic")
            (w / TESTS_REL).write_text(
                mutate(TESTS_SRC, "    assert_eq!", "\tassert_eq!"))
            r = h.logic(w, "verify", str(WI))
            hit = h.red_on(r, "C0")
            h.record("a whitespace-only edit to tests.rs is still refused",
                     "FAIL on C0", "red on it" if hit else r.stdout.strip()[-200:],
                     hit and r.returncode != 0)

        # -- outcomes ----------------------------------------------------------

        def commit_lands_the_implementation() -> None:
            """The commit is exactly the implementation, with the names on it."""
            w = h.staged(through="logic")
            r = h.logic(w, "commit", str(WI))
            left = git(w, "status", "--porcelain", "-uall").stdout.strip()
            landed = sorted(
                line for line in
                git(w, "show", "--name-only", "--format=", "HEAD").stdout.split())
            digest = h.trailer(w, "Logic-Change-Digest")
            ok = (r.returncode == 0 and left == ""
                  and landed == sorted([LIB_REL, MAIN_REL]) and len(digest) == 64)
            h.record("commit lands the implementation and nothing else",
                     "clean tree, lib.rs and main.rs only, a 64-char digest",
                     f"exit {r.returncode}; left {left!r}; landed {landed}; "
                     f"digest {len(digest)} chars", ok)

        def phases_cannot_be_folded_together() -> None:
            """One commit carrying two phases is not two phases.

            The `unit` work is committed under the `e2e` phase's subject, so
            the tree looks complete and the log carries a `Refs #1`. `P4` reads
            the subject prefix, so what it sees is an `e2e` phase that landed
            and a `unit` phase that never did -- which is exactly what
            happened.
            """
            w = h.fresh()
            (w / CARGO_REL).write_text(CARGO_TOML_REGISTERED)
            (w / CASE_REL).write_text(CASE_SRC)
            (w / LIB_REL).write_text(LIB_SKELETON)
            (w / TESTS_REL).write_text(TESTS_SRC)
            git(w, "add", "-A")
            git(w, "commit", "-qm",
                f"e2e(demo): pin the behaviour and the invariant\n\nRefs #{WI}")
            write_logic(w)
            r = h.logic(w, "verify", str(WI))
            hit = h.red_on(r, "P4")
            h.record("two phases folded into one commit is one phase",
                     "FAIL on P4", "red on it" if hit else r.stdout.strip()[-200:],
                     hit and r.returncode != 0)

        h.run([
            # -- the ladder ----------------------------------------------------
            start_clean,
            start_dirty,
            unit_before_e2e,
            logic_before_unit,
            phase_already_landed,
            e2e_phase_writes_src,
            logic_phase_writes_tests_rs,
            unit_phase_writes_only_the_skeleton,

            # -- U0..U3 --------------------------------------------------------
            unit_red_is_a_build_failure,
            unit_test_selector_matches_nothing,
            unit_phase_implements_the_behaviour,
            unit_records_the_names,

            # -- L1..L5 --------------------------------------------------------
            logic_does_not_compile,
            recorded_test_still_red,
            unit_commit_amended_after_measuring,
            logic_breaks_a_neighbouring_test,
            logic_unwires_a_neighbouring_test,
            case_already_green_at_head,
            logic_reformats_the_test,

            # -- outcomes ------------------------------------------------------
            commit_lands_the_implementation,
            phases_cannot_be_folded_together,
        ])

        print(f"\n{'':8s} {'control':66s} observation")
        for _slot, status, name, detail in h.results:
            print(f"{status:8s} {name:66s} {detail}")
        failed = [x for x in h.results if x[1] != "PASS"]
        print(f"\n{len(h.results) - len(failed)}/{len(h.results)} "
              "controls behaved as declared")
        return 1 if failed else 0
    finally:
        shutil.rmtree(root, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
