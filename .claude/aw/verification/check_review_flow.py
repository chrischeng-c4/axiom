#!/usr/bin/env python3
"""The review layer: what the reviewer is shown, and what its answer binds to.

A phase gate can only measure what a machine can decide. Whether a case
observes the behaviour its name claims, or whether a test would refuse a wrong
implementation, is not one of those things -- so two of the three phases send
the change to an independent reviewer and refuse to commit without its verdict.
That makes the review layer load-bearing, and load-bearing in a way the phase
gates cannot see: `e2e.py` knows a verdict exists and binds to the right bytes,
and knows nothing about whether the reviewer was shown the change.

This gate is that second question, and it is deliberately cargo-free. The
transcript parser and the record live in `leg.py`; both reviewed phases call
the same code, so driving every shape of it through `e2e.py` measures the
shared implementation once instead of once per phase, and does it in seconds
rather than a compile per row. What is left to `check_tdd_flow.py` is each
phase's own wiring -- that `commit` refuses without a verdict, and that an edit
after the verdict invalidates it -- because those rows need a phase that really
commits.

The split is a cost decision with one honest gap: the whole-surface form
differs per phase (`e2e` enumerates the case inventory, the code review
enumerates colocated test files and the sources beside them), so both are
measured here, while the *work-item-scoped* code review needs a landed `unit`
phase and stays where the ladder is.

The fixture's case is red because the product it names does not exist, and
nothing here ever makes it green. Whether a case discriminates is
`check_tdd_flow.py`'s question; this one only asks whether a reviewer would be
shown enough to answer it.

  1 positive control   -- the unmutated change produces a prompt at all. Run
                          alone and first: every control below is a claim about
                          which way the review layer refuses, and none of them
                          means anything if the admissible change is refused.

  6 parser controls    -- every transcript shape the verb must accept or
                          refuse, including the echoed answer `codex exec`
                          really produces.

  4 record controls    -- what lands on disk is the reviewer's answer and not a
                          summary of it: the rejection, the bound work item and
                          paths, the deduped findings, and the transcript
                          itself byte-for-byte.

  4 prompt controls    -- the reviewer is shown the work item, the source, and
                          the current failure; is not spent on a change the
                          mechanical list already refused; and both phases can
                          be pointed at their whole surface with no work item,
                          which records nothing because there is nothing to
                          bind to.
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
from _paths import E2E_SCRIPT, LOGIC_SCRIPT, pinned_interpreter  # noqa: E402

Path = pathlib.Path

UV = pinned_interpreter()
LAUNCH = " ".join(UV)

CASE = "demo-marker-case"
CASES_REL = "apps/demo/e2e/src/cases"
CASE_REL = f"{CASES_REL}/{CASE}.py"
PYPROJECT_REL = "apps/demo/e2e/pyproject.toml"
PROJECT_TOML_REL = "apps/demo/aw.toml"
LIB_REL = "apps/demo/src/lib.rs"
TESTS_REL = "apps/demo/src/tests.rs"

WI = 1
WI_BODY_REL = f".aw/workitems/changes/{WI}.md"

# Keyed by phase, not just by work item. Both reviewed phases record a verdict
# for the same `#1`, and one shared path would let the later phase's record sit
# where the earlier phase's gate looks for it. The digests would not match, so
# nothing would be accepted -- but the failure would read as a stale verdict
# rather than as two phases writing to one file.
RECORD_REL = f".aw/review/e2e-wi-{WI}.json"
LOGIC_RECORD_REL = f".aw/review/logic-wi-{WI}.json"

MARKER = "hello"

# One sentence that appears in the work-item body and nowhere else, so "the
# prompt carries the work item" is a claim about this exact text rather than
# about the prompt being long.
WI_SENTINEL = "the marker file today does not exist"

# The same, for the case source and for each side of the code-review surface.
CASE_SENTINEL = 'MARKER_FILE = pathlib.Path("apps/demo/marker.txt")'
LIB_SENTINEL = "pub fn double(n: i64) -> i64"
TESTS_SENTINEL = "fn twice_doubles()"

# The first line of the rubric's Q0. A refusal must not print it: the rubric is
# the reviewer's instruction sheet, and a prompt that carries it has started the
# review whatever its exit code says.
RUBRIC_SENTINEL = "Q0 DOES THIS CHANGE SATISFY THE WORK ITEM?"

CASE_SRC = f'''\
"""The product writes the marker string."""
from __future__ import annotations

import pathlib

CASE_ID = "{CASE}"
DIMENSION = "behavior"
TARGET_COMMAND = "{LAUNCH} {CASE_REL}"
ASSERTIONS = ("running the product writes `{MARKER}` to the marker file",)

{CASE_SENTINEL}


def verify() -> list[str]:
    observed = (
        MARKER_FILE.read_text(encoding="utf-8") if MARKER_FILE.is_file() else ""
    )
    assert observed == "{MARKER}", (
        "running the product writes `{MARKER}` to the marker file"
    )
    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
'''

PYPROJECT = f'''\
[project]
name = "demo-e2e"
version = "0.1.0"
requires-python = ">=3.11"

[[tool.aw.python-e2e.cases]]
id = "{CASE}"
dimension = "behavior"
promise = "the product writes the marker string"
oracle = "the bytes in the marker file after the product runs"
command = "{LAUNCH} {CASE_REL}"
'''

# The `[unit]` block is here even though nothing in this gate compiles: the
# code-review side reads the project's config to find its surface, and a
# fixture missing a key the real thing always has would make a refusal look
# like a review-layer decision.
PROJECT_TOML = '''\
[project]
name = "demo"

[unit]
build = "cargo test --offline --lib --no-run --manifest-path apps/demo/Cargo.toml"
test = "cargo test --offline --lib --manifest-path apps/demo/Cargo.toml"
harness = "cargo"
'''

# A module with a colocated test beside it, committed with the scaffold. It is
# the whole population of the code-review surface: without it, "the prompt
# carries every colocated test and its sibling source" would be a claim over an
# empty set, which is green for a prompt that carries nothing.
LIB_SRC = f'''\
#[cfg(test)]
mod tests;

{LIB_SENTINEL} {{
    n * 2
}}
'''

TESTS_SRC = f'''\
use super::*;

#[test]
{TESTS_SENTINEL} {{
    assert_eq!(double(21), 42);
}}
'''

# `.aw/` carries both the staged body and the recorded verdict, and the real
# checkout gitignores it. Mirroring that is not cosmetic: either file reported
# by `git status` would join the dirty set and fail `C0` in every control here
# -- the right row red for entirely the wrong reason.
GITIGNORE = ".aw/\n__pycache__/\napps/demo/marker.txt\napps/demo/target/\n"

WI_BODY = f"""\
## Goal

Running the demo product writes the string `{MARKER}` into
`apps/demo/marker.txt`, where {WI_SENTINEL}.

## How

### Verified premises

- `{PROJECT_TOML_REL}:1` declares the project and names no case inventory, so
  nothing in this project pins the marker file yet.
- `{LIB_REL}:1` carries no `marker` function, so nothing writes the marker
  today.

### Change points

- `{PYPROJECT_REL}`
- `{CASE_REL}`

### Frozen decisions

The marker path is fixed at `apps/demo/marker.txt` and the string is fixed at
`{MARKER}`. Excluded: any change under `apps/demo/src`, which belongs to the
two phases after this one.

## Acceptance

| # | command | current | target | why it cannot hold by accident |
|---|---|---|---|---|
| 1 | `e2e.py test 1` | no case observes the marker file | `E1` red over `{CASE}` | the case runs against the product as it stands, so a case that observes nothing comes out green and the row refuses it |

### Negative control

Replace the case's assertion with one comparing two literals and re-run the
command above; row `C2` must go red. Restore the file byte-for-byte and confirm
sha256 `0000000000000000000000000000000000000000000000000000000000000000`
before continuing.

## Never

This addresses the worker implementing this work item, not the controller reviewing it.

### Must not touch

- `{LIB_REL}`
- `{TESTS_REL}`

### Must not do

- Do not make the case green by writing `apps/demo/marker.txt` by hand.
- Do not soften the asserted string to something the tree already produces.
"""

# What a reviewer that agrees produces. `codex exec` prints its final answer
# twice -- once streamed, once as the closing message -- so the echoed form is
# the common shape rather than an edge case, and the findings arrive twice with
# it.
GOOD = "Reviewed the change.\n\nVERDICT: accepted\n"
ECHOED = (
    "Reviewed the change.\n\nVERDICT: accepted\n"
    "tokens used\n26,895\n"
    "Reviewed the change.\n\nVERDICT: accepted\n"
)
FINDING = "Q1 demo-marker-case: an empty marker file would also pass"
REJECTED = f"FINDING: {FINDING}\nVERDICT: rejected\n"
REJECTED_ECHOED = (
    f"FINDING: {FINDING}\nVERDICT: rejected\n"
    "tokens used\n12,001\n"
    f"FINDING: {FINDING}\nVERDICT: rejected\n"
)


def git(work: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-c", "core.fsmonitor=false", *args],
                          cwd=work, capture_output=True, text=True)


def build(root: Path) -> Path:
    """A checkout with the scaffold committed and the `e2e` change uncommitted.

    The change is written after the commit because that is its real shape: the
    phase is reviewed while its output is still a diff. Committing it here
    would leave every control running against a tree with nothing in it.
    """
    fixture = root / "fixture"
    (fixture / CASES_REL).mkdir(parents=True)
    (fixture / LIB_REL).parent.mkdir(parents=True)
    (fixture / ".gitignore").write_text(GITIGNORE)
    (fixture / "aw.toml").write_text('version = "0.0.0"\n')
    (fixture / PROJECT_TOML_REL).write_text(PROJECT_TOML)
    (fixture / LIB_REL).write_text(LIB_SRC)
    (fixture / TESTS_REL).write_text(TESTS_SRC)
    body = fixture / WI_BODY_REL
    body.parent.mkdir(parents=True)
    body.write_text(WI_BODY)

    subprocess.run(["git", "init", "-q"], cwd=fixture, check=True)
    subprocess.run(["git", "config", "user.email", "t@t"], cwd=fixture, check=True)
    subprocess.run(["git", "config", "user.name", "t"], cwd=fixture, check=True)
    subprocess.run(["git", "add", "-A"], cwd=fixture, check=True)
    subprocess.run(["git", "commit", "-qm", "scaffold"], cwd=fixture, check=True)

    (fixture / PYPROJECT_REL).write_text(PYPROJECT)
    (fixture / CASES_REL / f"{CASE}.py").write_text(CASE_SRC)
    return fixture


class Harness:
    """One fixture, and a fresh copy of it per control."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.fixture = build(root)
        self.n = 0
        self.results: list[tuple[int, str, str, str]] = []
        self.lock = threading.Lock()
        self.slot = threading.local()

    def fresh(self) -> Path:
        with self.lock:
            self.n += 1
            n = self.n
        work = self.root / f"work{n}"
        shutil.copytree(self.fixture, work)
        return work

    def run(self, controls: list) -> None:
        """Every control, concurrently, reported in declaration order.

        They are independent by construction: each opens its own `copytree`
        copy and nothing outside a work dir is written after `__init__`.
        """
        def go(item) -> None:
            i, fn = item
            self.slot.i = i
            fn()

        with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
            list(pool.map(go, enumerate(controls)))
        self.results.sort(key=lambda row: row[0])

    def phase(self, script: Path, work: Path, *args: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            [*UV, str(script), "--project", "demo", *args],
            cwd=work, capture_output=True, text=True, timeout=300,
            env={**os.environ},
        )

    def e2e(self, work: Path, *args: str) -> subprocess.CompletedProcess:
        return self.phase(E2E_SCRIPT, work, *args)

    def logic(self, work: Path, *args: str) -> subprocess.CompletedProcess:
        return self.phase(LOGIC_SCRIPT, work, *args)

    def transcript(self, work: Path, text: str) -> Path:
        """A transcript file outside the work dir.

        Named after the work dir rather than a counter: the work dir is already
        unique per control, and a counter read outside the lock would hand two
        concurrent controls the same file. Outside the checkout because a
        transcript written into it is a path `git status` reports, which would
        put the harness's own scratch into the change under review.
        """
        path = self.root / f"{work.name}.txt"
        path.write_text(text)
        return path

    def verdict(self, work: Path, text: str, *args: str) -> subprocess.CompletedProcess:
        path = self.transcript(work, text)
        return self.e2e(work, "verdict", str(WI), "--transcript", str(path), *args)

    def record(self, name: str, want: str, got: str, ok: bool) -> None:
        row = (getattr(self.slot, "i", -1),
               "PASS" if ok else "**FAIL**", name, f"want {want}; got {got}")
        with self.lock:
            self.results.append(row)

    # -- control families --------------------------------------------------

    def parser_control(self, name: str, text: str, want_ok: bool) -> None:
        w = self.fresh()
        r = self.verdict(w, text)
        ok = (r.returncode == 0) if want_ok else (r.returncode != 0)
        tail = [ln for ln in (r.stderr or r.stdout).strip().splitlines() if ln.strip()]
        self.record(name, "accepted" if want_ok else "refused",
                    f"exit {r.returncode}: {tail[-1][:90] if tail else '(no output)'}", ok)

    def prompt_control(self, name: str, args: tuple, want: dict, *,
                       script: Path | None = None, want_ok: bool = True) -> None:
        """One prompt, and what it must and must not carry.

        `want` maps a sentinel to whether it has to be there. Both directions
        are needed: "carries the work item" and "carries no work item" are
        different claims about the same prompt, and only the second one can
        catch a whole-surface form that quietly went and read one anyway.
        """
        w = self.fresh()
        r = self.phase(script or E2E_SCRIPT, w, *args)
        exit_ok = (r.returncode == 0) if want_ok else (r.returncode != 0)
        missing = [s for s, present in want.items() if (s in r.stdout) != present]
        ok = exit_ok and not missing
        detail = "as declared" if ok else (
            f"exit {r.returncode}" + (f"; wrong for {missing}" if missing else ""))
        self.record(name, ("exit 0" if want_ok else "refused") + " and the declared content",
                    detail, ok)


def main() -> int:
    root = Path(tempfile.mkdtemp(prefix="aw-review-flow-"))
    try:
        h = Harness(root)

        # ---- positive control: the unmutated change ------------------------
        #
        # Alone, and before anything else runs. Every control below is a claim
        # about which way the review layer refuses; if the admissible change
        # cannot even produce a prompt, all of those refusals are already there
        # and none of them means anything.
        w = h.fresh()
        r = h.e2e(w, "review-prompt", str(WI))
        if r.returncode != 0:
            print(r.stdout)
            print(r.stderr, file=sys.stderr)
            print("the positive control failed: the admissible change produced no "
                  "prompt, so every control below would be uninterpretable and "
                  "they are not run.", file=sys.stderr)
            return 1
        h.record("positive control: the admissible change produces a prompt",
                 "exit 0", "exit 0", True)

        # -- what lands on disk ------------------------------------------------

        def rejection_is_recorded() -> None:
            w = h.fresh()
            r = h.verdict(w, REJECTED)
            rec = w / RECORD_REL
            got = json.loads(rec.read_text()) if rec.is_file() else {}
            ok = (r.returncode != 0 and got.get("result") == "rejected"
                  and got.get("findings") == [FINDING])
            h.record("a rejection is recorded and still refuses to pass",
                     "exit non-zero and a rejected record carrying the finding",
                     f"exit {r.returncode}, record {got.get('result')!r}, "
                     f"{len(got.get('findings', []))} finding(s)", ok)

        def record_binds_the_change() -> None:
            w = h.fresh()
            r = h.verdict(w, GOOD)
            raw = (h.root / f"{w.name}.txt").read_bytes()
            rec = w / RECORD_REL
            got = json.loads(rec.read_text()) if rec.is_file() else {}
            want_paths = sorted([PYPROJECT_REL, CASE_REL])
            problems = []
            if got.get("work_item") != WI:
                problems.append(f"work_item={got.get('work_item')!r}")
            if sorted(got.get("paths", [])) != want_paths:
                problems.append(f"paths={got.get('paths')!r}")
            if got.get("transcript_digest") != hashlib.sha256(raw).hexdigest():
                problems.append("transcript_digest does not match the file read")
            if len(str(got.get("change_digest", ""))) != 64:
                problems.append(f"change_digest={got.get('change_digest')!r}")
            ok = r.returncode == 0 and not problems
            h.record("the record binds the work item, the paths, and the transcript",
                     "all four fields as declared",
                     "as declared" if ok else "; ".join(problems) or f"exit {r.returncode}",
                     ok)

        # The echoed shape brings the findings twice as well as the verdict. A
        # record that carried each one per echo would read as two independent
        # objections where the reviewer raised one.
        def findings_are_deduped() -> None:
            w = h.fresh()
            h.verdict(w, REJECTED_ECHOED)
            rec = w / RECORD_REL
            got = json.loads(rec.read_text()) if rec.is_file() else {}
            ok = got.get("findings") == [FINDING]
            h.record("the record carries each finding once, however often it echoed",
                     "exactly one finding", f"{got.get('findings')!r}", ok)

        # The record says what the transcript's digest is; this is the copy that
        # digest can be checked against later. A summary here -- or a rewrite
        # that normalised whitespace -- would make the recorded digest a claim
        # about bytes nobody kept.
        def transcript_is_kept_verbatim() -> None:
            w = h.fresh()
            h.verdict(w, ECHOED)
            src = (h.root / f"{w.name}.txt").read_bytes()
            kept = w / RECORD_REL.replace(".json", ".transcript.txt")
            ok = kept.is_file() and kept.read_bytes() == src
            h.record("the stored transcript is byte-identical to the one read",
                     "the same bytes beside the record",
                     "identical" if ok else
                     ("absent" if not kept.is_file() else "differs"), ok)

        # -- the prompt --------------------------------------------------------

        def refuses_an_inadmissible_change() -> None:
            """A reviewer is not spent on a question the checks already answered.

            The mutation removes the inventory, so `C1` refuses -- and the
            claim is not only that the verb exits non-zero. It must not print
            the rubric: the rubric is the reviewer's instruction sheet, and a
            prompt carrying it has started the review whatever the exit code
            says afterwards.
            """
            w = h.fresh()
            (w / PYPROJECT_REL).unlink()
            r = h.e2e(w, "review-prompt", str(WI))
            ok = r.returncode != 0 and RUBRIC_SENTINEL not in r.stdout
            h.record("review-prompt refuses a change the mechanical list refused",
                     "exit non-zero and no rubric",
                     "as declared" if ok else
                     f"exit {r.returncode}, rubric "
                     f"{'printed' if RUBRIC_SENTINEL in r.stdout else 'absent'}", ok)

        def verdict_needs_a_work_item() -> None:
            """The whole-surface review records nothing, because it binds nothing.

            A verdict is evidence about a specific change. Recording one with
            no change to bind it to would produce a file shaped exactly like
            the thing a commit gate reads, holding an approval of nothing.
            """
            w = h.fresh()
            path = h.transcript(w, GOOD)
            r = h.e2e(w, "verdict", "--transcript", str(path))
            review_dir = w / ".aw/review"
            wrote = sorted(p.name for p in review_dir.glob("*")) \
                if review_dir.is_dir() else []
            ok = r.returncode != 0 and not wrote
            h.record("verdict with no work item is refused, and writes nothing",
                     "exit non-zero and an empty .aw/review",
                     f"exit {r.returncode}, wrote {wrote}", ok)

        h.run([
            # -- the transcript parser ------------------------------------------
            lambda: h.parser_control(
                "verdict accepts a well-formed transcript", GOOD, True),
            # Measured against codex-cli 0.146.0: the real tool echoes its final
            # answer, so this shape has to be accepted. The rule it must NOT
            # relax into is "take the last verdict" -- the disagreement control
            # below holds that line.
            lambda: h.parser_control(
                "verdict accepts an echoed transcript (the real codex shape)",
                ECHOED, True),
            lambda: h.parser_control(
                "verdict refuses a transcript with no VERDICT line",
                "Looks fine to me.\n", False),
            lambda: h.parser_control(
                "verdict refuses verdicts that disagree with each other",
                "VERDICT: rejected\nFINDING: x\nVERDICT: accepted\n", False),
            lambda: h.parser_control(
                "verdict refuses VERDICT that is not the final line",
                "VERDICT: accepted\nand one more thought\n", False),
            lambda: h.parser_control(
                "verdict refuses `rejected` with no FINDING line",
                "VERDICT: rejected\n", False),

            # -- the record ------------------------------------------------------
            rejection_is_recorded,
            record_binds_the_change,
            findings_are_deduped,
            transcript_is_kept_verbatim,

            # -- the prompt ------------------------------------------------------
            # Four sentinels, one row, because they fail together: a prompt
            # missing any of them can be answered "these are well-built
            # verifiers" by a reviewer who never learned what was asked for,
            # and that answer is indistinguishable from the one worth having.
            lambda: h.prompt_control(
                "review-prompt carries the work item, the source, and the failure",
                ("review-prompt", str(WI)),
                {WI_SENTINEL: True, CASE_SENTINEL: True,
                 "AssertionError": True, RUBRIC_SENTINEL: True}),
            refuses_an_inadmissible_change,
            # No work item: the whole surface, advisory. The negative half is
            # the load-bearing one -- a form that went and read the staged body
            # anyway would be scoping itself to a change nobody asked it about.
            lambda: h.prompt_control(
                "review-prompt with no work item reviews the whole surface",
                ("review-prompt",),
                {CASE: True, CASE_SENTINEL: True, RUBRIC_SENTINEL: True,
                 WI_SENTINEL: False}),
            # The other phase's surface, which is derived differently: colocated
            # test files, and the sources they sit beside. Its work-item-scoped
            # form needs a landed `unit` phase and lives in `check_tdd_flow.py`.
            lambda: h.prompt_control(
                "the code review with no work item carries the tests and sources",
                ("review-prompt",),
                {TESTS_SENTINEL: True, LIB_SENTINEL: True,
                 RUBRIC_SENTINEL: True, WI_SENTINEL: False},
                script=LOGIC_SCRIPT),
            verdict_needs_a_work_item,
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
