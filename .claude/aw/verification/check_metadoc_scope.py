#!/usr/bin/env python3
"""Prove `metadoc.py` refuses each thing it claims to refuse, then writes.

Two halves. `check` reads and never writes, and most of this file plants one
violation per case against a fixture that is otherwise clean. `commit` is the
half that stages and writes, and it is exercised for real at the end -- a
commit is made in the fixture and read back, because a trailer block nobody
parsed is a claim about a format rather than a measurement of one.

`metadoc.py` measures a *working tree*, which is exactly what a gate cannot
assume: this checkout's tree is whatever the session left in it, and a gate
that read it would report the session rather than the script. So every case
here builds its own checkout in a temporary directory -- `git init`, a
minimal project, one commit -- plants exactly one violation in it, and asserts
that the rule which owns that violation fires and that the others stay quiet.

That second half is the part worth writing down. A gate that only asserted
"some finding appeared" would pass just as happily if every case tripped `P2`,
which is the easiest rule to trip and the one every mutation reaches through.
Each case therefore names its rule, and the baseline case asserts *zero*
findings against the same fixture -- so a `collect` that had learned to refuse
everything would fail here rather than look thorough.

## The allowlist went from one path to four, and that needs its own controls

Until 2026-08-27 the allowlist was `docs/product/` alone, and `README.md`,
`STATUS.md` and `ROADMAP.md` were the *near misses* -- the paths the `P2` case
planted a write into to prove the refusal worked. They are now inside it. That
inverts three controls at once, and an inverted control that nobody rewrote is
the most dangerous shape in this file: it would go on passing while measuring
the opposite of what its name says.

So each of the four entries gets a case asserting an edit to it is refused by
nothing, and `P2`'s own case plants its write somewhere that was never in scope
under either rule -- `src/**`. Four admissions and one refusal, rather than one
refusal that used to cover four paths.

## What it cannot do

It says nothing about whether the real project's documents are good, and
nothing about the prose in them. It measures one script's refusals against
planted violations, and a violation nobody thought to plant is a violation
nobody here catches.

It also does not measure `meta.py check`, which is the second step of the
landing sequence and carries what `/aw-check-meta` used to. That has its own
gate; what matters here is that this script's `next.command` names it, which
the last case below reads out of the actual output rather than out of the
source.
"""
from __future__ import annotations

import json
import pathlib
import subprocess
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import METADOC_SCRIPT, load_script_module  # noqa: E402

metadoc = load_script_module(METADOC_SCRIPT, "metadocmod")

fails: list[str] = []


def check(name: str, ok: bool, detail: str = "") -> None:
    print(f"{'PASS' if ok else 'FAIL'} {name}" + (f"\n     {detail}" if detail and not ok else ""))
    if not ok:
        fails.append(name)


PROJECT = "apps/demo"
AREA_REL = f"{PROJECT}/docs/product/area.md"
INDEX_REL = f"{PROJECT}/docs/product/README.md"

AREA = """# Demo area

Covers the README capability `demo-thing`.

## Shipped thing

- Problem: a caller cannot reach the thing today and gets a 404 instead of it.
- Who: callers of the demo API.
- Promise: the thing happens once and reports where it landed.
- Limits today: it happens once and is never retried, so a caller that misses
  the report has to ask for it again.
- Non-goals: nothing beyond the single call is promised here.
- Neighbours: none; first section of the area.
- Status rows: `demo-shipped`

## Future thing

- Problem: a caller cannot reach the future thing today and has to poll for it
  on a timer of its own.
- Who: callers of the demo API.
- Promise: the future thing happens and the caller is told about it.
- Non-goals: nothing beyond the notification is promised here.
- Open: what shape the notification body takes.
- Neighbours: extends Shipped thing.
- Outcome: `demo-future`. Tracking: not assigned.

## Non-goals in this area

- `demo-never`: not offered, and the ROADMAP says why.
"""

INDEX = """# Demo product requirements

## Section index

| Section | File | Kind | Owner |
|---|---|---|---|
| Shipped thing | area.md | shipped | STATUS `demo-shipped` |
| Future thing | area.md | outcome | ROADMAP `demo-future` |
"""

# A second family of areas under `docs/`, with its own index beside it. It
# exists for one control: `P8` reads the index in each touched directory, and
# with a single family that claim is indistinguishable from reading one
# hardcoded path.
GUIDE = """# Demo operator guide

## Operating thing

- Problem: an operator cannot tell whether the thing ran and has to read logs.
- Who: operators of the demo service.
- Promise: the operating thing reports its own state.
- Non-goals: nothing about the caller-facing surface is promised here.
- Open: what the state is called.
- Neighbours: extends Shipped thing.
- Outcome: `demo-future`. Tracking: not assigned.

## Non-goals in this area

- `demo-never-operated`: not offered.
"""

GUIDE_INDEX = """# Demo operator documents

## Section index

| Section | File | Kind | Owner |
|---|---|---|---|
| Operating thing | guide.md | outcome | ROADMAP `demo-future` |
"""

# The project README. In scope since 2026-08-27, and in the fixture since then
# too -- `P1` requires it, so a fixture without one would put every case in
# this file one finding above its declared expectation.
README = """# demo

## Capabilities

| Capability | Gate |
|---|---|
| `demo-thing` | `cargo test -p demo` |
"""

STATUS = "# Demo status\n\n| Row | State |\n|---|---|\n| `demo-shipped` | supported |\n"
ROADMAP = "# Demo roadmap\n\n### Future thing\n\n- ID: `demo-future`\n- Tracking: Not assigned.\n"

# The stub stands in for `scripts/meta/project_docs_contract.py`. It prints the
# shape `P10` reads -- a top-level `reports` list whose entries carry `ok` --
# and flips to a refusal when a marker file exists, which is how the `P10` case
# makes it say no without changing the code under test. The marker sits *above*
# the checkout on purpose: inside it, it would be an untracked path outside the
# allowlist, and the case would trip `P2` on its own fixture.
VALIDATOR_STUB = '''#!/usr/bin/env python3
import json, pathlib, sys
refuse = (pathlib.Path(__file__).parent.parent.parent.parent / "REFUSE").is_file()
print(json.dumps({"reports": [{"ok": not refuse,
                               "findings": [] if not refuse else ["stub refusal"]}]}))
sys.exit(1 if refuse else 0)
'''

GIT = ("git", "-c", "core.fsmonitor=false")


def build(tmp: pathlib.Path) -> pathlib.Path:
    """A checkout carrying one project with a clean, committed document set."""
    repo = tmp / "checkout"
    (repo / PROJECT / "docs/product").mkdir(parents=True)
    (repo / PROJECT / "docs/operating").mkdir(parents=True)
    (repo / PROJECT / "src").mkdir(parents=True)
    (repo / "scripts/meta").mkdir(parents=True)
    (repo / "aw.toml").write_text("[aw]\n", encoding="utf-8")
    (repo / PROJECT / "README.md").write_text(README, encoding="utf-8")
    (repo / PROJECT / "STATUS.md").write_text(STATUS, encoding="utf-8")
    (repo / PROJECT / "ROADMAP.md").write_text(ROADMAP, encoding="utf-8")
    (repo / PROJECT / "src/lib.rs").write_text("pub fn demo() {}\n", encoding="utf-8")
    (repo / INDEX_REL).write_text(INDEX, encoding="utf-8")
    (repo / AREA_REL).write_text(AREA, encoding="utf-8")
    (repo / PROJECT / "docs/operating/README.md").write_text(GUIDE_INDEX,
                                                             encoding="utf-8")
    (repo / PROJECT / "docs/operating/guide.md").write_text(GUIDE, encoding="utf-8")
    (repo / "scripts/meta/project_docs_contract.py").write_text(VALIDATOR_STUB,
                                                                encoding="utf-8")
    subprocess.run([*GIT, "init", "-q", "-b", "main"], cwd=repo, check=True)
    for key, value in (("user.name", "gate"), ("user.email", "gate@local")):
        subprocess.run([*GIT, "config", key, value], cwd=repo, check=True)
    subprocess.run([*GIT, "add", "-A"], cwd=repo, check=True,
                   capture_output=True)
    subprocess.run([*GIT, "-c", "user.name=gate", "-c", "user.email=gate@local",
                    "commit", "-q", "-m", "fixture"], cwd=repo, check=True,
                   capture_output=True)
    return repo


def rules(repo: pathlib.Path) -> list[str]:
    findings, _population = metadoc.collect(repo, PROJECT)
    return sorted({f.rule for f in findings})


def area_text(repo: pathlib.Path) -> str:
    return (repo / AREA_REL).read_text(encoding="utf-8")


def write_area(repo: pathlib.Path, text: str) -> None:
    (repo / AREA_REL).write_text(text, encoding="utf-8")


# One legitimate META-doc edit: the Promise of the unbound section gains a
# sentence. Every case below starts from this, so a rule that fires here would
# fire in every case and the whole file would be measuring the baseline instead
# of the mutation.
def edit_promise(repo: pathlib.Path) -> None:
    write_area(repo, area_text(repo).replace(
        "- Promise: the future thing happens and the caller is told about it.",
        "- Promise: the future thing happens, the caller is told about it, and\n"
        "  the notification carries the identifier."))


CASES = []


def case(name: str, expect: list[str]):
    def register(fn):
        CASES.append((name, expect, fn))
        return fn
    return register


@case("baseline: a legitimate area edit is refused by nothing", [])
def _baseline(repo):
    edit_promise(repo)


# -- the four allowlist entries -------------------------------------------
#
# These four were one control until 2026-08-27, and it asserted the opposite:
# three of them were the paths `P2` planted a write into to prove the refusal
# worked. Splitting them out is not thoroughness for its own sake -- an
# admission that is only ever exercised as part of some other case's setup is
# an admission nobody has measured.
@case("allowlist: an edit to the project README is admitted", [])
def _allow_readme(repo):
    (repo / PROJECT / "README.md").write_text(
        README.replace("| `demo-thing` |", "| `demo-thing` (renamed) |"),
        encoding="utf-8")


@case("allowlist: an edit to STATUS.md is admitted", [])
def _allow_status(repo):
    (repo / PROJECT / "STATUS.md").write_text(
        STATUS + "| `demo-planned` | planned |\n", encoding="utf-8")


@case("allowlist: an edit to ROADMAP.md is admitted", [])
def _allow_roadmap(repo):
    (repo / PROJECT / "ROADMAP.md").write_text(
        ROADMAP + "\n- Note: the outcome is still open.\n", encoding="utf-8")


@case("allowlist: a docs/ file outside docs/product/ is admitted", [])
def _allow_other_docs(repo):
    (repo / PROJECT / "docs/operating/guide.md").write_text(
        GUIDE.replace("- Promise: the operating thing reports its own state.",
                      "- Promise: the operating thing reports its own state, "
                      "promptly."),
        encoding="utf-8")


@case("allowlist: a promise and the STATUS row that owns it, in one run", [])
def _allow_promise_and_row(repo):
    """The edit the widened allowlist exists for.

    Under the old scope this was two runs by two skills, and the second one was
    the one that got skipped. It is one run now, and the reason that is safe is
    the next case: `P7` still resolves the id, against the tree as edited.
    """
    write_area(repo, area_text(repo).replace(
        "- Status rows: `demo-shipped`", "- Status rows: `demo-shipped`, `demo-fast`"))
    (repo / PROJECT / "STATUS.md").write_text(
        STATUS + "| `demo-fast` | supported |\n", encoding="utf-8")


@case("P2: a write to src/ is refused", ["P2"])
def _p2(repo):
    edit_promise(repo)
    (repo / PROJECT / "src/lib.rs").write_text("pub fn demo() { }\n",
                                               encoding="utf-8")


@case("P2: a write to another project's documents is refused", ["P2"])
def _p2_other_project(repo):
    edit_promise(repo)
    (repo / "apps/other").mkdir(parents=True)
    (repo / "apps/other/README.md").write_text("# other\n", encoding="utf-8")


@case("P3: a clean tree has no META-doc run to commit", ["P3"])
def _p3(repo):
    pass


@case("P4: a heading that gained an issue number is refused", ["P4"])
def _p4(repo):
    edit_promise(repo)
    write_area(repo, area_text(repo).replace("## Future thing",
                                             "## Future thing (#7)"))


@case("P5: a section missing one of its kind's bullets is refused", ["P5"])
def _p5(repo):
    edit_promise(repo)
    write_area(repo, area_text(repo).replace(
        "- Who: callers of the demo API.\n- Promise: the future thing happens,",
        "- Promise: the future thing happens,"))


# The three cases below are the two-shape schema itself. A single rule read
# off the skill's prose passed the first two and refused every shipped section
# in the only real document set in the checkout, so each shape now has a case
# that goes red when the other one's bullets appear in it.
@case("P5: a shipped section carrying Open: is refused", ["P5"])
def _p5_open_on_shipped(repo):
    edit_promise(repo)
    write_area(repo, area_text(repo).replace(
        "- Limits today: it happens once and is never retried, so a caller "
        "that misses\n  the report has to ask for it again.",
        "- Open: whether the report should be retried."))


@case("P5: a future section carrying Limits today: is refused", ["P5"])
def _p5_limits_on_future(repo):
    edit_promise(repo)
    write_area(repo, area_text(repo).replace(
        "- Open: what shape the notification body takes.",
        "- Limits today: there is no notification at all.\n"
        "- Open: what shape the notification body takes."))


@case("P5: a section with no owner bullet is refused", ["P5"])
def _p5_no_owner(repo):
    edit_promise(repo)
    write_area(repo, area_text(repo).replace(
        "- Status rows: `demo-shipped`\n", ""))


@case("P5: the project README is not measured as an area file", [])
def _p5_readme_not_an_area(repo):
    """The admission that would have been a `P5` storm.

    The README's own headings -- `## Capabilities` and whatever else the
    project owns -- carry no owner bullet and no non-goals footer, so a scope
    widening that fed them to the section schema would refuse every README in
    the repository. `is_area` is what keeps them out, and this is the control
    that says so: the file is in the allowlist and out of the shape check, and
    those are two different lists.
    """
    (repo / PROJECT / "README.md").write_text(
        README + "\n## Operating notes\n\nSome prose with no bullets at all.\n",
        encoding="utf-8")


@case("P5: a directory index is not measured as an area file", [])
def _p5_index_not_an_area(repo):
    edit_promise(repo)
    (repo / INDEX_REL).write_text(INDEX + "\n<!-- reordered -->\n",
                                  encoding="utf-8")


@case("P6: an Outcome bullet wrapped before Tracking: is refused", ["P6"])
def _p6(repo):
    edit_promise(repo)
    write_area(repo, area_text(repo).replace(
        "- Outcome: `demo-future`. Tracking: not assigned.",
        "- Outcome: `demo-future`.\n  Tracking: not assigned."))


@case("P7: an outcome id that resolves nowhere is refused", ["P7"])
def _p7(repo):
    edit_promise(repo)
    write_area(repo, area_text(repo).replace("`demo-future`. Tracking:",
                                             "`demo-invented`. Tracking:"))


@case("P7: a STATUS row claimed but not added in the same run is refused",
      ["P7"])
def _p7_row_not_added(repo):
    """The mitigation for the widened allowlist, stated as a control.

    The old scope's argument was that a run editing both the promise and the
    ground under it had moved the ground it was standing on. It still could --
    what changed is that `P7` reads `STATUS.md` from the *working tree*, so a
    section claiming a row the run did not actually add is refused in the same
    breath. The ground is measured, not assumed.
    """
    write_area(repo, area_text(repo).replace(
        "- Status rows: `demo-shipped`", "- Status rows: `demo-shipped`, `demo-fast`"))


@case("P8: a section with no row in its directory's index is refused", ["P8"])
def _p8(repo):
    edit_promise(repo)
    write_area(repo, area_text(repo).replace(
        "## Non-goals in this area",
        "## Unindexed thing\n\n"
        "- Problem: a caller cannot reach the unindexed thing today at all.\n"
        "- Who: callers of the demo API.\n"
        "- Promise: the unindexed thing happens.\n"
        "- Non-goals: nothing else is promised here.\n"
        "- Open: what it is called.\n"
        "- Neighbours: extends Shipped thing.\n"
        "- Outcome: `demo-future`. Tracking: not assigned.\n\n"
        "## Non-goals in this area"))


@case("P8: the index read is the one beside the file, not docs/product/'s",
      ["P8"])
def _p8_per_directory(repo):
    """The claim that `P8` is per directory rather than per project.

    The unindexed section is planted in `docs/operating/`, whose own index does
    not name it. A check that read `docs/product/README.md` for every area file
    would look at the wrong index here, find `Operating thing` missing from it
    as well, and report two findings against a file this run never touched --
    or, if it only read the touched files' index, none at all.
    """
    guide = repo / PROJECT / "docs/operating/guide.md"
    guide.write_text(GUIDE.replace(
        "## Non-goals in this area",
        "## Second operating thing\n\n"
        "- Problem: an operator cannot restart the thing without a shell.\n"
        "- Who: operators of the demo service.\n"
        "- Promise: the second operating thing restarts it.\n"
        "- Non-goals: nothing about the caller-facing surface here.\n"
        "- Open: what the command is called.\n"
        "- Neighbours: extends Operating thing.\n"
        "- Outcome: `demo-future`. Tracking: not assigned.\n\n"
        "## Non-goals in this area"), encoding="utf-8")


@case("P8: an untouched directory's index is not read at all", [])
def _p8_untouched_directory(repo):
    """Scoping the index read to the changed paths, stated as an admission.

    `docs/operating/` is left alone here, and its index is deliberately
    incomplete in one respect no other case exercises: nothing in this run
    touched it. A check that walked `docs/` instead of reading the changed
    paths would report findings from a directory this run never opened, which
    makes every clean run look dirty on someone else's backlog.
    """
    edit_promise(repo)
    (repo / PROJECT / "docs/operating/README.md").write_text(
        GUIDE_INDEX, encoding="utf-8")
    # Restore it byte-for-byte so the directory is genuinely untouched: the
    # write above exists only to prove the case is not passing because the path
    # is unreachable.
    subprocess.run([*GIT, "checkout", "--",
                    f"{PROJECT}/docs/operating/README.md"], cwd=repo, check=True)


@case("P9: an area file that lost its non-goals footer is refused", ["P9"])
def _p9(repo):
    edit_promise(repo)
    write_area(repo, area_text(repo).split("## Non-goals in this area")[0])


@case("P10: a refusal from the product-document contract is carried through",
      ["P10"])
def _p10(repo):
    edit_promise(repo)
    (repo.parent / "REFUSE").write_text("", encoding="utf-8")


with tempfile.TemporaryDirectory() as raw:
    tmp = pathlib.Path(raw)
    for i, (name, expect, mutate) in enumerate(CASES):
        repo = build(tmp / f"case{i}")
        mutate(repo)
        got = rules(repo)
        check(name, got == expect, f"expected {expect}, got {got}")

    # -- the trailer derivation ------------------------------------------
    # `commit` writes these into the message, and they are the whole of what
    # the next skill reads. Derived from the diff rather than declared, so the
    # assertion is that the derivation sees an addition, a modification and a
    # removal for what they are.
    repo = build(tmp / "trailers")
    text = area_text(repo)
    text = text.replace(
        "- Promise: the future thing happens and the caller is told about it.",
        "- Promise: the future thing happens, promptly.")
    text = text.replace("## Non-goals in this area",
                        "## Third thing\n\n- Problem: none yet.\n\n"
                        "## Non-goals in this area")
    write_area(repo, text)
    changed = [AREA_REL]
    modes = metadoc.section_modes(repo, PROJECT, changed)
    check("section_modes reports the added section",
          f"added {AREA_REL}#Third thing" in modes, str(modes))
    check("section_modes reports the modified section",
          f"modified {AREA_REL}#Future thing" in modes, str(modes))
    check("section_modes leaves the untouched section out",
          f"modified {AREA_REL}#Shipped thing" not in modes, str(modes))

    # The three top-level documents are in the allowlist and are not sections.
    # Feeding them to `section_modes` would put `## Capabilities` in a
    # `Meta-Section:` trailer, where the next skill reads it as a promise.
    (repo / PROJECT / "README.md").write_text(README + "\n## Extra\n\nprose.\n",
                                              encoding="utf-8")
    modes = metadoc.section_modes(repo, PROJECT, changed + [f"{PROJECT}/README.md"])
    check("section_modes reports no section for the project README",
          not any("README.md#" in line for line in modes), str(modes))

    repo = build(tmp / "removal")
    write_area(repo, area_text(repo).split("## Future thing")[0]
               + "## Non-goals in this area\n\n- `demo-never`: not offered.\n")
    modes = metadoc.section_modes(repo, PROJECT, changed)
    check("section_modes reports a removed section",
          f"removed {AREA_REL}#Future thing" in modes, str(modes))

    # -- the unbound count ------------------------------------------------
    # What tells the next skill how much of the commit is still waiting for a
    # work item. Bound sections must not be counted, or the number says
    # "nothing was converted" forever.
    repo = build(tmp / "unbound")
    edit_promise(repo)
    got = metadoc.unbound_count(repo, PROJECT, changed)
    check("unbound_count counts the sections carrying no issue number",
          got == 2, str(got))
    write_area(repo, area_text(repo).replace("## Future thing", "## Future thing (#7)"))
    got = metadoc.unbound_count(repo, PROJECT, changed)
    check("unbound_count stops counting a section once it is bound",
          got == 1, str(got))
    got = metadoc.unbound_count(repo, PROJECT, changed + [f"{PROJECT}/STATUS.md"])
    check("unbound_count does not count headings in the top-level documents",
          got == 1, str(got))

    # -- the commit ---------------------------------------------------------
    # The verb that writes. Everything above measures `check`, which is the
    # half that cannot damage anything; this is the half that stages and
    # commits, and until it was exercised here the only proof it worked was
    # having watched it once.
    def run(repo, *argv):
        return subprocess.run([sys.executable, str(METADOC_SCRIPT), *argv],
                              cwd=repo, capture_output=True, text=True)

    def git_out(repo, *argv):
        return subprocess.run([*GIT, *argv], cwd=repo,
                              capture_output=True, text=True).stdout

    repo = build(tmp / "commit")
    edit_promise(repo)
    (repo / INDEX_REL).write_text(INDEX + "\n<!-- reordered -->\n",
                                  encoding="utf-8")
    # The whole reason the allowlist was widened: the promise and the ground it
    # stands on land together. If this run cannot commit, the merge bought
    # nothing.
    (repo / PROJECT / "STATUS.md").write_text(STATUS + "| `demo-planned` | planned |\n",
                                              encoding="utf-8")
    why = tmp / "why.txt"
    why.write_text("docs(demo): the future thing carries an identifier\n\n"
                   "A subscriber that gets two notifications cannot tell them\n"
                   "apart, so the promise now names the identifier.\n\n"
                   "Co-Authored-By: Someone <someone@local>\n", encoding="utf-8")
    done = run(repo, "commit", PROJECT, "--why", str(why))
    check("commit exits 0 on a clean run", done.returncode == 0,
          done.stdout + done.stderr)

    message = git_out(repo, "log", "-1", "--pretty=%B")
    check("the subject is the human's line, not a generated one",
          message.startswith("docs(demo): the future thing carries an identifier"),
          message)
    check("the project trailer names the resolved root",
          "\nMeta-Project: apps/demo\n" in message, message)
    check("a touched top-level document is named by its own trailer",
          "\nMeta-Top: STATUS.md\n" in message, message)
    check("an untouched top-level document gets no trailer",
          "\nMeta-Top: ROADMAP.md\n" not in message, message)
    check("a touched index is reported as its own trailer",
          f"\nMeta-Index: {INDEX_REL}\n" in message, message)
    check("the section trailer names the section and how it changed",
          f"\nMeta-Section: modified {AREA_REL}#Future thing\n" in message, message)
    check("the unbound count is the sections still carrying no issue number",
          "\nMeta-Unbound: 2\n" in message, message)
    # git reads trailers only as one contiguous run at the end of the message,
    # so a `Co-Authored-By:` the human wrote mid-file has to be re-emitted
    # after the generated block or it stops being a trailer at all.
    trailers = git_out(repo, "log", "-1", "--pretty=%(trailers:only=true)")
    check("a carried trailer survives as a trailer",
          "Co-Authored-By: Someone <someone@local>" in trailers, trailers)
    check("the carried trailer is last, after the generated block",
          trailers.strip().splitlines()[-1].startswith("Co-Authored-By:"), trailers)

    named = sorted(git_out(repo, "show", "--pretty=", "--name-only").split())
    check("the commit stages exactly the allowlist",
          named == sorted([AREA_REL, INDEX_REL, f"{PROJECT}/STATUS.md"]),
          str(named))
    check("the tree is clean afterwards",
          git_out(repo, "status", "--porcelain", "-uall").strip() == "",
          git_out(repo, "status", "--porcelain", "-uall"))
    # The whole point of the trailers: the next skill finds the run by them.
    found = git_out(repo, "log", "--grep=^Meta-Project: apps/demo", "--pretty=%h")
    check("the commit is findable by its project trailer",
          len(found.split()) == 1, found)

    # A run that `check` refuses must not reach `git commit`, or the refusal is
    # a warning rather than a gate.
    repo = build(tmp / "commit_refused")
    edit_promise(repo)
    (repo / PROJECT / "src/lib.rs").write_text("pub fn demo() { }\n",
                                               encoding="utf-8")
    before = git_out(repo, "rev-parse", "HEAD")
    done = run(repo, "commit", PROJECT, "--why", str(why))
    check("commit refuses a run that check refuses", done.returncode == 1,
          done.stdout + done.stderr)
    check("the refused run wrote no commit",
          git_out(repo, "rev-parse", "HEAD") == before, "HEAD moved")
    check("the refused run staged nothing",
          git_out(repo, "diff", "--cached", "--name-only").strip() == "",
          git_out(repo, "diff", "--cached", "--name-only"))

    # The subject is the one part of the message the script judges, and both
    # rules are about `git log --oneline` staying readable and addressed.
    repo = build(tmp / "commit_subject")
    edit_promise(repo)
    bad = tmp / "bad.txt"
    bad.write_text("the future thing carries an identifier\n", encoding="utf-8")
    check("a subject not addressed to the project is refused",
          run(repo, "commit", PROJECT, "--why", str(bad)).returncode != 0)
    bad.write_text("docs(demo): " + "x" * 80 + "\n", encoding="utf-8")
    check("a subject over 72 characters is refused",
          run(repo, "commit", PROJECT, "--why", str(bad)).returncode != 0)
    check("the tree is untouched by a refused subject",
          git_out(repo, "diff", "--cached", "--name-only").strip() == "",
          git_out(repo, "diff", "--cached", "--name-only"))

    # -- the landing sequence ---------------------------------------------
    # `/aw-check-meta` was deleted on 2026-08-27 and its `meta.py check` step
    # folded into this skill's landing sequence. The only thing that keeps that
    # step from being forgotten again is `check`'s own `next.command`, so it is
    # read out of the actual output rather than out of the source.
    repo = build(tmp / "next_command")
    edit_promise(repo)
    out = run(repo, "check", PROJECT).stdout
    line = next((l for l in out.splitlines() if l.startswith("next.command:")), "")
    check("a clean check prints => CLEAN", "=> CLEAN" in out, out)
    # Matched on the closing quote rather than on the bare names, because
    # `metadoc.py` contains neither `meta.py` nor `metadoc.py commit` as a
    # substring -- the path is quoted and the verb sits outside the quote. An
    # assertion written on the bare names passes or explodes for reasons that
    # have nothing to do with the ordering it claims to measure.
    check("the next command runs meta.py check before committing",
          'meta.py" check' in line
          and line.index('meta.py" check') < line.index('metadoc.py" commit'),
          line or out)

    # -- the qualified promise --------------------------------------------
    # `Promise, for now:` marks a surface that is public today and leaving.
    # It is admitted by name, not by letting a key carry a comma, so both
    # halves are asserted: the enumerated form normalises to `Promise`, and an
    # unenumerated `<Key>, <words>:` is still not a bullet -- otherwise a
    # `- Non-goals: Google-signed OIDC tokens, ...` line would become schema.
    parsed = metadoc.bullets("- Promise, for now: it reads the journal directly.\n")
    check("a qualified promise parses as Promise",
          parsed == [("Promise", "it reads the journal directly.")], str(parsed))
    parsed = metadoc.bullets("- Promise, later on: it will not.\n")
    check("an unenumerated qualifier is not a bullet at all",
          parsed == [], str(parsed))

    # -- the scope predicates ---------------------------------------------
    # Read directly, because every case above reaches them through a working
    # tree and a boundary is easier to get wrong than to observe.
    check("in_scope admits all four entries and nothing else",
          [metadoc.in_scope(PROJECT, p) for p in (
              f"{PROJECT}/README.md", f"{PROJECT}/STATUS.md",
              f"{PROJECT}/ROADMAP.md", f"{PROJECT}/docs/product/area.md",
              f"{PROJECT}/src/lib.rs", f"{PROJECT}/Cargo.toml",
              "apps/other/README.md", "README.md",
              f"{PROJECT}/docsnt/area.md")]
          == [True, True, True, True, False, False, False, False, False])
    # `is_area` reads the tree -- the index beside the file is what makes it an
    # area -- so it needs a checkout, unlike `in_scope`, which reads the string.
    repo = build(tmp / "areas")
    check("is_area admits indexed docs files and neither index nor top-level doc",
          [metadoc.is_area(repo, PROJECT, p) for p in (
              f"{PROJECT}/docs/product/area.md",
              f"{PROJECT}/docs/operating/guide.md",
              f"{PROJECT}/docs/product/README.md",
              f"{PROJECT}/README.md", f"{PROJECT}/STATUS.md",
              f"{PROJECT}/docs/product/notes.txt")]
          == [True, True, False, False, False, False])

    # The index requirement, and the reason it exists. Widening the allowlist
    # from `docs/product/` to `docs/**` on 2026-08-27 swept 61 pre-existing
    # reference documents across seven projects into the promise population --
    # runbooks, benchmark postures, an operator handoff manual -- and a
    # reference document has no owner bullet, so `wis.py`'s G1 read `?`
    # UNMEASURED for every one of those projects. Registration in the
    # directory's own index is what tells a promise from a document that
    # happens to live under `docs/`.
    loose = repo / PROJECT / "docs/runbooks"
    loose.mkdir(parents=True)
    (loose / "drain.md").write_text("# Drain\n", encoding="utf-8")
    rel = f"{PROJECT}/docs/runbooks/drain.md"
    unindexed = metadoc.is_area(repo, PROJECT, rel)
    (loose / "README.md").write_text("# Runbooks\n", encoding="utf-8")
    indexed = metadoc.is_area(repo, PROJECT, rel)
    check("an unindexed docs file is reference, and its index promotes it",
          (unindexed, indexed) == (False, True),
          f"without index {unindexed}, with index {indexed}")

    # The population line the report prints, which is the only thing standing
    # between "this project promises six things" and "this project has 61
    # documents nobody is measuring". A count of areas alone cannot say which.
    areas, reference = metadoc.area_population(repo, PROJECT)
    check("area_population separates indexed areas from unindexed reference",
          (areas, reference) == (3, 0), f"{areas} area(s), {reference} reference")
    (loose / "README.md").unlink()
    areas, reference = metadoc.area_population(repo, PROJECT)
    check("taking an index away moves its files into the reference count",
          (areas, reference) == (2, 1), f"{areas} area(s), {reference} reference")

    # -- the project resolver ---------------------------------------------
    repo = build(tmp / "resolve")
    check("a bare project name resolves to its root",
          metadoc.resolve_project(repo, "demo") == PROJECT)
    check("an explicit root is taken as given",
          metadoc.resolve_project(repo, PROJECT) == PROJECT)

    # -- the stub is not answering for the real validator ------------------
    # A positive control on the fixture itself: if the stub silently stopped
    # being invoked, every `P10` answer above would be vacuous.
    repo = build(tmp / "stub")
    proc = subprocess.run([sys.executable,
                           str(repo / "scripts/meta/project_docs_contract.py"),
                           "check", PROJECT, "--format", "json"],
                          capture_output=True, text=True)
    check("the fixture validator answers the shape P10 reads",
          json.loads(proc.stdout)["reports"][0]["ok"] is True, proc.stdout)

print("\n=> " + ("GREEN" if not fails else f"RED ({len(fails)} failure(s))"))
sys.exit(1 if fails else 0)
