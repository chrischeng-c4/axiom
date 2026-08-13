#!/usr/bin/env python3
"""Negative-control harness for `plugins/aw/scripts/ec.py`.

Builds a throwaway checkout -- its own `aw.toml`, its own EC inventory, a staged
work-item body, one green case and one red case -- then mutates exactly one
thing at a time and asserts that the named check goes red and the others do not.
A check that stays green under its own mutation is not a check.

Every mutation is applied to a fresh copy of the fixture, so no control can leak
into the next. The real checkout is never touched: the fixture is a `tempfile`
tree and every `ec.py` invocation runs with its cwd inside it.

This harness spent one session living in a session scratchpad with the checkout
path hardcoded into it -- the exact pair of mistakes `_paths.py` opens by
describing. It resolves both the same way that file does: paths come from
`_paths`, and the fixture root comes from `tempfile`.

What the fixture is a fixture *of*
----------------------------------
A work item's EC change is the diff against `HEAD`, so the fixture is shaped as
one: the green case is committed, and the red case is written afterwards and
left untracked. That is the real shape -- `/aw:wi-ec-start` refuses to open the
leg over a dirty tree, so from a clean start the only thing `git status` reports
is what the leg wrote.

Two consequences the fixture has to carry, both of which produce confusing reds
when they are missing. It needs a committed `.gitignore` covering `.aw/`, or the
staged work-item body and the recorded verdict would join the dirty set and fail
`C0 scope` for reasons unrelated to any control. And the harness's own scratch
files -- the green roster, the transcripts -- live in the tempdir *root* rather
than in a work dir, for exactly the same reason.

Structure of the controls
-------------------------
  2 positive controls -- the unmutated fixture is admissible, through the gate
                         and through the case-scoped primitive. Without them,
                         every red below would be indistinguishable from a
                         fixture that was broken to begin with.
  7 check mutations   -- one per refusable row (C1, C3-C6), driven through the
                         WI gate so the per-case fan-out is exercised too.
  1 primitive control -- `check --case` still refuses what it always refused.
  7 precondition and scope controls -- P1, P2, P3, C0 twice and C0b: the rows
                         that exist only because the change is defined as a
                         diff, plus `start` passing on a clean tree so that
                         "refuses a dirty one" is a discrimination.
  7 verdict controls  -- the transcript parser, including the two shapes real
                         reviewers actually produce, and the missing route.
  5 digest controls   -- the binding that makes an approval die when the case,
                         the inventory, or the work item changes, plus what
                         happens with no verdict at all.
  3 commit controls   -- the diff that was reviewed is the diff that lands, and
                         the two halves of the back-link to the work item: the
                         full sha `ec.py` hands out, and the fenced block that
                         records it without breaking the schema the next leg
                         validates against.

C2 has no mutation because it is a named PENDING in every fixture: it looks for
an engineering baseline this fixture project does not have, and says so rather
than passing. A control that mutated it would be asserting on the absence.
"""
from __future__ import annotations

import concurrent.futures
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile
import threading

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import EC_SCRIPT, load_change_module, pinned_interpreter  # noqa: E402

Path = pathlib.Path

UV = pinned_interpreter()

# The fixture's own `command` strings run through this too. Left as a literal
# `uv run ...` they cost another environment resolution per case per control,
# which was the larger half of this harness's runtime.
LAUNCH = " ".join(UV)
CASE = "demo-red-case"
GREEN = "demo-green-case"

RED_SRC = f'''\
"""A case that pins behaviour which does not exist yet."""
from __future__ import annotations

import pathlib

CASE_ID = "demo-red-case"
DIMENSION = "python-contract"
TARGET_COMMAND = "{LAUNCH} apps/demo/external-contracts/src/cases/demo-red-case.py"
ASSERTIONS = (
    "the marker file records the string the product is supposed to write",
)


def verify() -> list[str]:
    marker = pathlib.Path("apps/demo/marker.txt")
    observed = marker.read_text() if marker.is_file() else ""
    assert observed == "written by the product", (
        "the marker file records the string the product is supposed to write"
    )
    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
'''

GREEN_SRC = f'''\
"""A case that already holds."""
from __future__ import annotations

CASE_ID = "demo-green-case"
DIMENSION = "python-contract"
TARGET_COMMAND = "{LAUNCH} apps/demo/external-contracts/src/cases/demo-green-case.py"
ASSERTIONS = ("two plus two is four, as measured",)


def verify() -> list[str]:
    assert len("aw" * 2) == 4, "two plus two is four, as measured"
    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
'''

PYPROJECT = f'''\
[project]
name = "demo-external-contracts"
version = "0.1.0"
requires-python = ">=3.11"

[[tool.aw.python-ec.cases]]
id = "{GREEN}"
dimension = "python-contract"
promise = "the arithmetic holds"
oracle = "the case exits zero"
command = "{LAUNCH} apps/demo/external-contracts/src/cases/{GREEN}.py"

[[tool.aw.python-ec.cases]]
id = "{CASE}"
dimension = "python-contract"
promise = "the product writes the marker"
oracle = "the marker file carries the exact string"
command = "{LAUNCH} apps/demo/external-contracts/src/cases/{CASE}.py"
'''

SRC = "apps/demo/external-contracts/src/cases"
WI = 1
WI_BODY_REL = f".aw/workitems/changes/{WI}.md"

# `.aw/` is where both the staged work-item body and the recorded verdict land,
# and the real checkout gitignores it (`.gitignore:3`). Mirroring that here is
# not cosmetic: without it those two files would be reported by `git status`,
# join the dirty set, and fail `C0 scope` in every control below -- a red that
# names the right row for entirely the wrong reason.
GITIGNORE = ".aw/\n__pycache__/\n"

# A change body that `change.py validate` accepts, because `P1` refuses one it
# does not. It describes the change the fixture actually makes, so the controls
# that mutate it are mutating something coherent rather than filler.
WI_BODY = """\
## Goal

Running the demo product writes the string `written by the product` into
`apps/demo/marker.txt`, where today that file does not exist.

## How

### Verified premises

- `apps/demo/external-contracts/pyproject.toml:1` registers the demo project's
  EC inventory and carries no case pinning the marker file.
- `apps/demo/aw.toml:4` routes `[review] ec` to `skill:codex-review`, so the
  semantic verdict for this change has a named reviewer.

### Change points

- `apps/demo/external-contracts/src/cases/demo-red-case.py`
- `apps/demo/external-contracts/pyproject.toml`

### Frozen decisions

The marker path is fixed at `apps/demo/marker.txt` and the exact string is
fixed at `written by the product`. Excluded: any change under `apps/demo/src`,
which belongs to the implementation leg rather than this one.

## Acceptance

| # | command | current | target | why it cannot hold by accident |
|---|---|---|---|---|
| 1 | `ec.py verify 1 --baseline baseline.json` | no case pins the marker file | every row passes over one case | the case must be red at a sentence it declared in advance, which a case that observes nothing cannot produce |

### Negative control

Replace the case's assertion with `assert False, "the marker file records the
string the product is supposed to write"` and re-run the command above; row
`C6` must go red. Restore the file byte-for-byte and confirm sha256
`0000000000000000000000000000000000000000000000000000000000000000` before
continuing.

## Never

This addresses the worker implementing this work item, not the controller reviewing it.

### Must not touch

- `apps/demo/src/**`
- `apps/demo/aw.toml`

### Must not do

- Do not make the case green by writing `apps/demo/marker.txt` by hand.
- Do not soften the asserted string to something the product already produces.
"""

# What a reviewer that agrees produces. `codex exec` prints its final answer
# twice -- once streamed, once as the closing message -- so the duplicated form
# below is the *common* shape, not an edge case.
GOOD_TRANSCRIPT = "Reviewed the case.\n\nVERDICT: accepted\n"
ECHOED_TRANSCRIPT = (
    "Reviewed the case.\n\nVERDICT: accepted\n"
    "tokens used\n26,895\n"
    "Reviewed the case.\n\nVERDICT: accepted\n"
)


def git(work: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", *args], cwd=work, capture_output=True, text=True)


def build(root: Path) -> Path:
    fixture = root / "fixture"
    cases = fixture / SRC
    cases.mkdir(parents=True)
    (fixture / ".gitignore").write_text(GITIGNORE)
    (fixture / "aw.toml").write_text('version = "0.0.0"\n')
    (fixture / "apps/demo/aw.toml").write_text(
        '[project]\nname = "demo"\n\n[review]\nec = "skill:codex-review"\n'
    )
    (fixture / "apps/demo/external-contracts/pyproject.toml").write_text(PYPROJECT)
    (cases / f"{GREEN}.py").write_text(GREEN_SRC)
    body = fixture / WI_BODY_REL
    body.parent.mkdir(parents=True)
    body.write_text(WI_BODY)
    subprocess.run(["git", "init", "-q"], cwd=fixture, check=True)
    # Set in the repo rather than passed per commit, because `ec.py commit` runs
    # `git commit` itself and cannot be handed an identity from here.
    subprocess.run(["git", "config", "user.email", "t@t"], cwd=fixture, check=True)
    subprocess.run(["git", "config", "user.name", "t"], cwd=fixture, check=True)
    subprocess.run(["git", "add", "-A"], cwd=fixture, check=True)
    subprocess.run(["git", "commit", "-qm", "fixture"], cwd=fixture, check=True)
    # The new case is written AFTER the commit, because that is its real shape:
    # `start` leaves a file git has never seen. Committing it here would have
    # made every commit-side control run against a path that is already tracked.
    (cases / f"{CASE}.py").write_text(RED_SRC)
    return fixture


class Harness:
    """One fixture, and a fresh copy of it per control."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.fixture = build(root)
        self.n = 0
        self.results: list[tuple[int, str, str, str]] = []
        self.lock = threading.Lock()
        # Which control the calling thread is running, so a result can be filed
        # under its declared position rather than the order it happened to
        # finish in. A report whose row order drifts run to run is a report
        # nobody can diff.
        self.slot = threading.local()
        # Outside every work dir on purpose. A roster written *into* the
        # checkout is a file `git status` reports, which would put the harness's
        # own scratch into the change under test.
        self.roster = root / "baseline.json"
        self.roster.write_text(json.dumps({"green": [GREEN]}))

    def fresh(self, *, clean: bool = False) -> Path:
        """A copy of the fixture. `clean` removes the change, leaving no diff."""
        with self.lock:
            self.n += 1
            n = self.n
        work = self.root / f"work{n}"
        shutil.copytree(self.fixture, work)
        if clean:
            (work / SRC / f"{CASE}.py").unlink()
        return work

    def run(self, controls: list) -> None:
        """Every control, concurrently.

        They are independent by construction and always have been: each one
        opens its own `copytree` copy of the fixture, and nothing outside a work
        dir is written after `__init__`. Running them one at a time was costing
        the whole suite about twenty seconds to preserve an ordering that no
        control ever depended on.

        What concurrency must not cost is the report. Each control carries the
        index it was declared at, so the rows below come out in source order
        whatever order the work finished in.
        """
        def go(item) -> None:
            i, fn = item
            self.slot.i = i
            fn()

        with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
            list(pool.map(go, enumerate(controls)))
        self.results.sort(key=lambda row: row[0])

    def ec(self, work: Path, *args: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            [*UV, str(EC_SCRIPT), "--project", "demo", *args],
            cwd=work, capture_output=True, text=True, timeout=300,
        )

    def verify(self, work: Path, *args: str) -> subprocess.CompletedProcess:
        return self.ec(work, "verify", str(WI), "--baseline", str(self.roster), *args)

    def commit(self, work: Path, *args: str) -> subprocess.CompletedProcess:
        return self.ec(work, "commit", str(WI), "--baseline", str(self.roster), *args)

    def accept(self, work: Path, transcript: str) -> subprocess.CompletedProcess:
        # Named after the work dir, not a counter: the work dir is already
        # unique per control, and a counter read outside the lock would hand two
        # concurrent controls the same transcript file.
        t = self.root / f"{work.name}.txt"
        t.write_text(transcript)
        return self.ec(work, "verdict", str(WI), "--baseline", str(self.roster),
                       "--transcript", str(t))

    def record(self, name: str, want: str, got: str, ok: bool) -> None:
        row = (getattr(self.slot, "i", -1),
               "PASS" if ok else "**FAIL**", name, f"want {want}; got {got}")
        with self.lock:
            self.results.append(row)

    @staticmethod
    def red_on(proc: subprocess.CompletedProcess, row: str) -> bool:
        return any(line.strip().startswith("FAIL") and row in line
                   for line in proc.stdout.splitlines())

    # -- control families --------------------------------------------------

    def mutate_check(self, name: str, expect_row: str, apply) -> None:
        """One mutation, driven through the gate rather than the primitive.

        `verify` is where these rows decide anything, and it reaches them by
        fanning out over the cases it found in the diff -- so running the
        mutations here measures the fan-out as well as the row. Driving them
        through `check --case` instead would leave a `verify` that skipped a
        case entirely looking exactly as green as one that checked it.
        """
        w = self.fresh()
        apply(w)
        r = self.verify(w)
        hit = self.red_on(r, expect_row)
        self.record(name, f"FAIL on {expect_row}",
                    "red on it" if hit else r.stdout.strip()[-160:],
                    hit and r.returncode != 0)

    def row_control(self, name: str, expect_row: str, apply, *, clean: bool = False) -> None:
        """As above, for the rows that exist only at the work-item layer."""
        w = self.fresh(clean=clean)
        if apply is not None:
            apply(w)
        r = self.verify(w)
        hit = self.red_on(r, expect_row)
        self.record(name, f"FAIL on {expect_row}",
                    "red on it" if hit else r.stdout.strip()[-160:],
                    hit and r.returncode != 0)

    def verdict_control(self, name: str, transcript: str, want_ok: bool) -> None:
        w = self.fresh()
        r = self.accept(w, transcript)
        ok = (r.returncode == 0) if want_ok else (r.returncode != 0)
        tail = (r.stderr or r.stdout).strip().splitlines()
        self.record(name, "accepted" if want_ok else "refused",
                    f"exit {r.returncode}: {tail[-1][:90] if tail else ''}", ok)

    def stale_control(self, name: str, mutate) -> None:
        w = self.fresh()
        self.accept(w, GOOD_TRANSCRIPT)
        mutate(w)
        r = self.commit(w, "--dry-run")
        stale = "C7" in r.stdout and "different bytes" in r.stdout
        self.record(name, "FAIL on C7",
                    "digest mismatch" if stale else r.stdout.strip()[-160:],
                    stale and r.returncode != 0)

def main() -> int:
    root = Path(tempfile.mkdtemp(prefix="aw-ec-flow-"))
    try:
        h = Harness(root)

        # ---- positive control: the unmutated fixture -----------------------
        #
        # Alone, and before anything else runs. Every control below is a claim
        # about which row goes red; if the unmutated fixture is not admissible
        # to begin with, all of those reds are already there and none of them
        # mean anything. So this one is not in the pool -- its answer decides
        # whether the pool is worth starting.
        w = h.fresh()
        r = h.verify(w)
        if r.returncode != 0:
            print(r.stdout)
            print("the positive control failed; every negative control below "
                  "would be uninterpretable, so they are not run.", file=sys.stderr)
            return 1
        h.record("positive control: unmutated change is admissible",
                 "exit 0", f"exit {r.returncode}", True)

        def primitive_runs() -> None:
            w = h.fresh()
            r = h.ec(w, "check", "--case", CASE, "--baseline", str(h.roster))
            h.record("positive control: the case-scoped primitive still runs",
                     "exit 0", f"exit {r.returncode}", r.returncode == 0)

        # One red driven through the case-scoped primitive as well. `verify` is
        # the gate and every check mutation goes through it; this keeps the
        # debugging verb from rotting into something that prints only greens.
        def primitive_red() -> None:
            w = h.fresh()
            (w / SRC / f"{CASE}.py").write_text(
                RED_SRC.replace('observed == "written by the product"', "observed == observed"))
            r = h.ec(w, "check", "--case", CASE, "--baseline", str(h.roster))
            hit = h.red_on(r, "C4")
            h.record("the case-scoped primitive still refuses an already-green case",
                     "FAIL on C4", "red on it" if hit else r.stdout.strip()[-160:],
                     hit and r.returncode != 0)

        def start_clean() -> None:
            w = h.fresh(clean=True)
            r = h.ec(w, "start", str(WI))
            h.record("start opens the leg on a clean tree", "exit 0",
                     f"exit {r.returncode}", r.returncode == 0)

        def start_dirty() -> None:
            w = h.fresh()
            r = h.ec(w, "start", str(WI))
            hit = h.red_on(r, "P2")
            h.record("start refuses a dirty tree", "FAIL on P2",
                     "red on it" if hit else r.stdout.strip()[-160:],
                     hit and r.returncode != 0)

        def verdict_lets_commit_through() -> None:
            w = h.fresh()
            h.accept(w, GOOD_TRANSCRIPT)
            r = h.commit(w, "--dry-run")
            h.record("an accepted verdict lets commit through", "exit 0",
                     f"exit {r.returncode}", r.returncode == 0)

        def commit_without_verdict() -> None:
            w = h.fresh()
            r = h.commit(w, "--dry-run")
            h.record("commit refuses when no verdict exists at all", "FAIL on C7",
                     "refused" if h.red_on(r, "C7") else r.stdout.strip()[-160:],
                     h.red_on(r, "C7") and r.returncode != 0)

        def verdict_without_route() -> None:
            w = h.fresh()
            (w / "apps/demo/aw.toml").write_text('[project]\nname = "demo"\n')
            r = h.accept(w, GOOD_TRANSCRIPT)
            h.record("verdict refuses when aw.toml names no [review] ec route",
                     "refused", f"exit {r.returncode}", r.returncode != 0)

        # Not a dry run. The claim under test is that what was reviewed is what
        # lands: the allowlist is the dirty set, so after this there is nothing
        # left over, and the trailer carries the same digest the verdict bound.
        def commit_lands_the_diff() -> None:
            w = h.fresh()
            h.accept(w, GOOD_TRANSCRIPT)
            done = h.commit(w)
            after = git(w, "status", "--porcelain", "-uall").stdout.strip()
            msg = git(w, "log", "-1", "--format=%B").stdout
            landed = git(w, "show", "--name-only", "--format=", "HEAD").stdout.split()
            ok = (done.returncode == 0 and after == ""
                  and f"{SRC}/{CASE}.py" in landed
                  and "Refs #1" in msg and "EC-Change-Digest:" in msg)
            h.record("commit lands exactly the reviewed diff and nothing else",
                     "clean tree, the case in HEAD, digest in the trailer",
                     f"exit {done.returncode}; left over {after!r}; landed {landed}", ok)

        # The first half of the back-link. `ec.py` makes no tracker write, so
        # what it owes the work item is a sha someone else can record -- and it
        # has to be the *full* one, resolved rather than scraped out of git's
        # own output, where the abbreviation length is a local config.
        def commit_emits_the_landed_sha() -> None:
            w = h.fresh()
            h.accept(w, GOOD_TRANSCRIPT)
            done = h.commit(w)
            head = git(w, "rev-parse", "HEAD").stdout.strip()
            emitted = [line.split(":", 1)[1].strip()
                       for line in done.stdout.splitlines()
                       if line.startswith("EC-Commit:")]
            ok = (done.returncode == 0 and emitted == [head] and len(head) == 40
                  and f"change.py lifecycle {WI} --leg ec --commit {head}" in done.stdout)
            h.record("commit emits the full sha it landed as, and how to record it",
                     f"EC-Commit: {head} and a lifecycle next.command",
                     f"emitted {emitted}", ok)

        # The second half, and the reason a body write is allowed at all. The
        # authored H2 set is closed, so the block has to survive the same
        # validator the next leg's P1 runs -- twice, because a leg that lands
        # twice must occupy one row rather than two.
        def lifecycle_block_is_upsert_and_stays_valid() -> None:
            change = load_change_module()
            wi = change.workitem
            once = wi.lifecycle_upsert(WI_BODY, "ec", "a" * 40, "d" * 64)
            twice = wi.lifecycle_upsert(once, "ec", "b" * 40, "e" * 64)
            later = wi.lifecycle_upsert(twice, "td", "c" * 40, "")
            errors = [e for e in change.validate_body(later)
                      if not e.startswith("note:")]
            ok = (errors == []
                  and later.startswith(WI_BODY.rstrip("\n"))
                  and wi.lifecycle_rows(later).keys() == {"ec", "td"}
                  and "b" * 40 in later and "a" * 40 not in later)
            h.record("the lifecycle block upserts by leg and the body stays valid",
                     "one row per leg, authored sections untouched, no schema error",
                     f"rows {sorted(wi.lifecycle_rows(later))}; errors {errors}", ok)

        # Declaration order is report order: `h.run` files each result under the
        # index it sits at here, so the table below reads the same on every run
        # no matter which control finishes first.
        h.run([
            primitive_runs,

            # -- one mutation per refusable row of `check` -------------------
            lambda: h.mutate_check(
                "C1 refuses a CASE_ID that disagrees with the inventory", "C1",
                lambda w: (w / SRC / f"{CASE}.py").write_text(
                    RED_SRC.replace('CASE_ID = "demo-red-case"',
                                    'CASE_ID = "demo-red-cased"'))),
            lambda: h.mutate_check(
                "C3 refuses a regression in the green roster", "C3",
                lambda w: (w / SRC / f"{GREEN}.py").write_text(
                    GREEN_SRC.replace('len("aw" * 2) == 4', 'len("aw" * 2) == 5'))),
            lambda: h.mutate_check(
                "C4 refuses a case that is already green", "C4",
                lambda w: (w / SRC / f"{CASE}.py").write_text(
                    RED_SRC.replace('observed == "written by the product"',
                                    "observed == observed"))),
            lambda: h.mutate_check(
                "C5 refuses red-by-typo (ImportError, not AssertionError)", "C5",
                lambda w: (w / SRC / f"{CASE}.py").write_text(
                    RED_SRC.replace("import pathlib", "import pathlibb as pathlib"))),
            lambda: h.mutate_check(
                "C5 refuses an AssertionError the case never declared", "C5",
                lambda w: (w / SRC / f"{CASE}.py").write_text(
                    RED_SRC.replace(
                        '"the marker file records the string the product is supposed to write"\n    )',
                        '"some entirely different sentence"\n    )'))),
            lambda: h.mutate_check(
                "C6 refuses `assert False, <the declared sentence>`", "C6",
                lambda w: (w / SRC / f"{CASE}.py").write_text(
                    RED_SRC.replace(
                        '    observed = marker.read_text() if marker.is_file() else ""\n'
                        '    assert observed == "written by the product", (',
                        "    assert False, ("))),
            lambda: h.mutate_check(
                "C6 refuses a literal-vs-literal comparison", "C6",
                lambda w: (w / SRC / f"{CASE}.py").write_text(
                    RED_SRC.replace(
                        '    observed = marker.read_text() if marker.is_file() else ""\n',
                        '    observed = marker.read_text() if marker.is_file() else ""\n'
                        '    assert 0 == 0 + 0, "the count matches"\n'))),
            primitive_red,

            # -- the work-item preconditions ---------------------------------
            start_clean,
            start_dirty,
            lambda: h.row_control(
                "P1 refuses a work item with no staged body", "P1",
                lambda w: (w / WI_BODY_REL).unlink()),
            lambda: h.row_control(
                "P3 refuses a leg whose ec(...) commit already landed", "P3",
                lambda w: git(w, "commit", "--allow-empty", "-q", "-m",
                              "ec(wi-1): already pinned\n\nRefs #1")),

            # -- the rows that exist only over the whole change ---------------
            #
            # C0 is the only refusal this layer adds, and it is the one that
            # turns `/aw:wi-ec-start`'s "never write src/** here" from prose in
            # a skill body into a gate.
            lambda: h.row_control(
                "C0 refuses a change that reaches outside external-contracts",
                "C0 scope",
                lambda w: (w / "apps/demo/src").mkdir(parents=True)
                or (w / "apps/demo/src/thing.rs").write_text("fn main() {}\n")),
            lambda: h.row_control(
                "C0 refuses an empty diff rather than passing over nothing",
                "C0 scope", None, clean=True),
            lambda: h.row_control(
                "C0b refuses a change that touches only the inventory", "C0b",
                lambda w: (w / "apps/demo/external-contracts/pyproject.toml").write_text(
                    PYPROJECT.replace(
                        'oracle = "the marker file carries the exact string"',
                        'oracle = "the marker file carries the string"')),
                clean=True),

            # -- the transcript parser ----------------------------------------
            lambda: h.verdict_control(
                "verdict accepts a well-formed transcript", GOOD_TRANSCRIPT, True),
            # Measured against codex-cli 0.146.0: the real tool echoes its final
            # answer, so this shape has to be accepted. The rule it must NOT
            # relax into is "take the last verdict" -- the control below it
            # holds that line.
            lambda: h.verdict_control(
                "verdict accepts an echoed transcript (the real codex shape)",
                ECHOED_TRANSCRIPT, True),
            lambda: h.verdict_control(
                "verdict refuses a transcript with no VERDICT line",
                "Looks fine to me.\n", False),
            lambda: h.verdict_control(
                "verdict refuses verdicts that disagree with each other",
                "VERDICT: rejected\nFINDING: x\nVERDICT: accepted\n", False),
            lambda: h.verdict_control(
                "verdict refuses VERDICT that is not the final line",
                "VERDICT: accepted\nand one more thought\n", False),
            lambda: h.verdict_control(
                "verdict refuses `rejected` with no FINDING line",
                "VERDICT: rejected\n", False),

            # -- the digest binding --------------------------------------------
            verdict_lets_commit_through,
            lambda: h.stale_control(
                "editing the case after review invalidates the verdict",
                lambda w: (w / SRC / f"{CASE}.py").write_text(
                    (w / SRC / f"{CASE}.py").read_text() + "\n# one byte more\n")),
            lambda: h.stale_control(
                "editing the INVENTORY after review invalidates the verdict too",
                lambda w: (w / "apps/demo/external-contracts/pyproject.toml").write_text(
                    (w / "apps/demo/external-contracts/pyproject.toml").read_text().replace(
                        'oracle = "the marker file carries the exact string"',
                        'oracle = "the marker file exists"'))),
            # The half of the digest that has no path in the diff at all. `.aw/`
            # is gitignored, so the work-item body is invisible to `git status`
            # -- and a verdict that survived an edit to it would be an approval
            # of a requirement nobody reviewed. The appended line keeps the body
            # valid, so `P1` stays green and the only row that can move is `C7`.
            lambda: h.stale_control(
                "editing the WORK ITEM after review invalidates the verdict too",
                lambda w: (w / WI_BODY_REL).write_text(
                    (w / WI_BODY_REL).read_text() + "- Do not do anything else.\n")),
            commit_without_verdict,
            verdict_without_route,

            # -- the commit itself ---------------------------------------------
            commit_lands_the_diff,
            commit_emits_the_landed_sha,
            lifecycle_block_is_upsert_and_stays_valid,
        ])

        print(f"\n{'':8s} {'control':66s} observation")
        for _slot, status, name, detail in h.results:
            print(f"{status:8s} {name:66s} {detail}")
        failed = [x for x in h.results if x[1] != "PASS"]
        print(f"\n{len(h.results) - len(failed)}/{len(h.results)} controls behaved as declared")
        return 1 if failed else 0
    finally:
        shutil.rmtree(root, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
