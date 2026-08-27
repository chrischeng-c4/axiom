#!/usr/bin/env python3
"""Prove `prd.py check` refuses each thing it claims to refuse.

`prd.py` measures a *working tree*, which is exactly what a gate cannot
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

The fixture carries a stub product-document validator that answers `ok` on
demand. Stubbed, not skipped: `P10` reads `reports[0].ok` out of that JSON, and
a case that never ran the reader would leave the one check whose answer comes
from another program untested.

## What it cannot do

It says nothing about whether the real project's documents are good, and
nothing about the prose in them. It measures one script's refusals against
planted violations, and a violation nobody thought to plant is a violation
nobody here catches.
"""
from __future__ import annotations

import json
import pathlib
import subprocess
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import PRD_SCRIPT, load_script_module  # noqa: E402

prd = load_script_module(PRD_SCRIPT, "prdmod")

fails: list[str] = []


def check(name: str, ok: bool, detail: str = "") -> None:
    print(f"{'PASS' if ok else 'FAIL'} {name}" + (f"\n     {detail}" if detail and not ok else ""))
    if not ok:
        fails.append(name)


PROJECT = "apps/demo"

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
    """A checkout carrying one project with a clean, committed PRD."""
    repo = tmp / "checkout"
    (repo / PROJECT / "docs/product").mkdir(parents=True)
    (repo / "scripts/meta").mkdir(parents=True)
    (repo / "aw.toml").write_text("[aw]\n", encoding="utf-8")
    (repo / PROJECT / "STATUS.md").write_text(STATUS, encoding="utf-8")
    (repo / PROJECT / "ROADMAP.md").write_text(ROADMAP, encoding="utf-8")
    (repo / PROJECT / "docs/product/README.md").write_text(INDEX, encoding="utf-8")
    (repo / PROJECT / "docs/product/area.md").write_text(AREA, encoding="utf-8")
    (repo / "scripts/meta/project_docs_contract.py").write_text(VALIDATOR_STUB,
                                                                encoding="utf-8")
    subprocess.run([*GIT, "init", "-q", "-b", "main"], cwd=repo, check=True)
    subprocess.run([*GIT, "add", "-A"], cwd=repo, check=True,
                   capture_output=True)
    subprocess.run([*GIT, "-c", "user.name=gate", "-c", "user.email=gate@local",
                    "commit", "-q", "-m", "fixture"], cwd=repo, check=True,
                   capture_output=True)
    return repo


def rules(repo: pathlib.Path) -> list[str]:
    findings, _population = prd.collect(repo, PROJECT)
    return sorted({f.rule for f in findings})


def area_text(repo: pathlib.Path) -> str:
    return (repo / PROJECT / "docs/product/area.md").read_text(encoding="utf-8")


def write_area(repo: pathlib.Path, text: str) -> None:
    (repo / PROJECT / "docs/product/area.md").write_text(text, encoding="utf-8")


# One legitimate PRD edit: the Promise of the unbound section gains a sentence.
# Every case below starts from this, so a rule that fires here would fire in
# every case and the whole file would be measuring the baseline instead of the
# mutation.
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


@case("baseline: a legitimate PRD edit is refused by nothing", [])
def _baseline(repo):
    edit_promise(repo)


@case("P2: a write outside docs/product/ is refused", ["P2"])
def _p2(repo):
    edit_promise(repo)
    (repo / PROJECT / "STATUS.md").write_text(STATUS + "\n<!-- touched -->\n",
                                              encoding="utf-8")


@case("P3: a clean tree has no PRD run to commit", ["P3"])
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
# in the only real PRD in the checkout, so each shape now has a case that goes
# red when the other one's bullets appear in it.
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


@case("P8: a section with no row in the index is refused", ["P8"])
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
    changed = [f"{PROJECT}/docs/product/area.md"]
    modes = prd.section_modes(repo, changed, f"{PROJECT}/docs/product")
    check("section_modes reports the added section",
          f"added {PROJECT}/docs/product/area.md#Third thing" in modes, str(modes))
    check("section_modes reports the modified section",
          f"modified {PROJECT}/docs/product/area.md#Future thing" in modes, str(modes))
    check("section_modes leaves the untouched section out",
          f"modified {PROJECT}/docs/product/area.md#Shipped thing" not in modes,
          str(modes))

    repo = build(tmp / "removal")
    write_area(repo, area_text(repo).split("## Future thing")[0]
               + "## Non-goals in this area\n\n- `demo-never`: not offered.\n")
    modes = prd.section_modes(repo, changed, f"{PROJECT}/docs/product")
    check("section_modes reports a removed section",
          f"removed {PROJECT}/docs/product/area.md#Future thing" in modes, str(modes))

    # -- the unbound count ------------------------------------------------
    # What tells the next skill how much of the commit is still waiting for a
    # work item. Bound sections must not be counted, or the number says
    # "nothing was converted" forever.
    repo = build(tmp / "unbound")
    edit_promise(repo)
    check("unbound_count counts the sections carrying no issue number",
          prd.unbound_count(repo, changed) == 2, str(prd.unbound_count(repo, changed)))
    write_area(repo, area_text(repo).replace("## Future thing", "## Future thing (#7)"))
    check("unbound_count stops counting a section once it is bound",
          prd.unbound_count(repo, changed) == 1, str(prd.unbound_count(repo, changed)))

    # -- the qualified promise --------------------------------------------
    # `Promise, for now:` marks a surface that is public today and leaving.
    # It is admitted by name, not by letting a key carry a comma, so both
    # halves are asserted: the enumerated form normalises to `Promise`, and an
    # unenumerated `<Key>, <words>:` is still not a bullet -- otherwise a
    # `- Non-goals: Google-signed OIDC tokens, ...` line would become schema.
    parsed = prd.bullets("- Promise, for now: it reads the journal directly.\n")
    check("a qualified promise parses as Promise",
          parsed == [("Promise", "it reads the journal directly.")], str(parsed))
    parsed = prd.bullets("- Promise, later on: it will not.\n")
    check("an unenumerated qualifier is not a bullet at all",
          parsed == [], str(parsed))

    # -- the project resolver ---------------------------------------------
    repo = build(tmp / "resolve")
    check("a bare project name resolves to its root",
          prd.resolve_project(repo, "demo") == PROJECT)
    check("an explicit root is taken as given",
          prd.resolve_project(repo, PROJECT) == PROJECT)

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
