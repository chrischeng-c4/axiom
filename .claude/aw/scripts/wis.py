#!/usr/bin/env python3
"""Measure the distance between what a project promises and what it tracks.

`/aw-grill-meta-to-wis` reads three things that are supposed to describe the
same product -- the META-docs, the codebase, and the open work items -- and
reorganises the third until it matches the first two. Almost all of that is
judgement: whether a promise deserves an epic of its own, whether two open
changes are the same change, whether a surface nobody promised should be
promised or removed. None of it is mechanical and none of it belongs here.

What *is* mechanical is the arithmetic underneath: which promise carries no
issue number, which issue no promise points at, which ROADMAP outcome nothing
claims. `gap` prints that table and nothing else. It is the only part of the
skill that can be refused, and the division is deliberate -- a model's opinion
that the backlog "looks about right" is not a measurement, and the way that
opinion becomes indistinguishable from a measurement is a script that prints
both in the same table.

## Why every row prints its population

A gap count of zero has two causes that look identical on a terminal: nothing
is missing, or nothing was read. The second one is the failure mode this file
is built around, because most of its inputs are absent in most projects --
thirty-six of the forty-four `docs/` trees hold no area file at all, only eight
projects have a `STATUS.md`, and `gh` outside a git directory fails with a
non-zero status and an empty stdout that json-decodes to nothing.

So no row is allowed to be silently empty. Each one ends up in exactly one of
two states, and `report` refuses to print a table where any row is in neither:

  * **measured**, with the size of what it read printed beside the count, so
    `0 / 0` is visibly not the same answer as `0 / 12`; or
  * **unmeasured**, printed as `?` with the reason, and counted separately in
    the verdict line so that a run which could not reach the tracker never
    exits 0.

The tracker is the sharpest case. `workitem.gh` raises `GhError` on a non-zero
status, which is what makes "could not ask" distinguishable from "asked, and
the answer was none" -- both of the rows that read the tracker go unmeasured
together rather than reporting an empty backlog.

## Why G1 reads only the future-shaped sections

`metadoc.py` measures a section against one of two shapes, chosen by its owner
bullet: an `Outcome:` section is a promise not yet kept, a `Status rows:`
section is one that has shipped. Only the first kind owes a work item.

That is a measurement, not a preference. Across tape's six area files -- the
only PRD in the repository at the time of writing -- twenty of the twenty-three
sections carry no ` (#<iid>)`, and eleven of those twenty are shipped sections
whose work is in the git history rather than on the tracker. A `G1` that
counted every unbound heading would open its first run by demanding eleven
epics for work that is already done, which is the shape of a gate nobody can
act on and everybody learns to ignore.

## What it does not do

It writes nothing -- not a document, not an issue, not a label. The skill's
writes go through `epic.py` and `change.py`, whose validators own what a body
must contain; a second writer here would be a second answer to that.

It also does not judge a promise. `metadoc.py check` owns whether a section is
well-formed and whether the ids it claims resolve; this file assumes that has
passed and only asks who is standing where. Running `gap` against a project
whose sections have not been checked reads a document whose shape nothing has
established -- the counts will parse, and they will be about the wrong things.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path

# Seven siblings, each loaded rather than reimplemented. The reason is the same
# every time: a second parser for the same file is a second answer, free to
# disagree, and the disagreement shows up as a gap count rather than as a
# defect. `metadoc` owns what a section is, `e2e` owns what a declared case is,
# `meta` owns what a dead gate is, `workitem`/`epic`/`change` own what a work
# item is, and `leg` owns the git plumbing under all of it.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import change  # noqa: E402
import e2e  # noqa: E402
import epic  # noqa: E402
import leg  # noqa: E402
import meta  # noqa: E402
import metadoc  # noqa: E402
import workitem  # noqa: E402

# The seven rows, in the order they are printed. Each line is what a non-zero
# count in that row means, phrased as the defect rather than as the check --
# a reader acting on this table needs to know what is wrong, not what ran.
GAPS = {
    "G1": "a future promise that no epic is opened for",
    "G2": "an open work item that no promise reaches",
    "G3": "a promise bound to an issue that cannot carry it",
    "G4": "a ROADMAP outcome no promise claims",
    "G5": "a STATUS row no promise claims",
    "G6": "an e2e case the crate manifest does not run",
    "G7": "a README gate that names no cargo target",
}

# `- ID: `<id>`` is the roadmap's own id line, and rule `M4` of
# `scripts/meta/project_docs_contract.py` is what keeps it in that shape.
# Reading it with a regex here is safe *because* that rule exists and
# `metadoc.py check` runs the validator as `P10`; without it this would be a
# parser guessing at prose.
ROADMAP_ID = re.compile(r"^-[ \t]+ID:[ \t]*`([^`]+)`[ \t]*$", re.M)

# The `## Support matrix` table, whose second column is the row id. `S4` in the
# same validator fixes the column order, so the index is a constant rather than
# a header search.
STATUS_MATRIX = "Support matrix"
STATUS_ID_COLUMN = 1


def usage_error(message: str) -> None:
    """Refuse the invocation, distinguishably from reporting gaps.

    Exit 2, never 1. A caller that cannot tell "the backlog is nine items
    behind" from "you named a project that does not exist" will act on the
    first reading, and a typo becomes a reorganisation.
    """
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(2)


# --------------------------------------------------------------------------
# what a promise is
# --------------------------------------------------------------------------
@dataclass(frozen=True)
class Promise:
    """One `## ` section of one area file, read for the things `gap` compares.

    `shape` is `future`, `shipped`, or empty for a section carrying no single
    owner bullet. Empty is not an error here -- it is `metadoc.py`'s `P5` --
    but it does mean this section cannot be asked whether it owes an epic, and
    `g1_unbound_promises` blocks rather than guessing.
    """

    path: str
    title: str
    iid: int | None
    shape: str
    outcome: str
    rows: tuple[str, ...]


def area_texts(repo: Path, project: str) -> dict[str, str]:
    """Every area file under the project's `docs/`, keyed by repo-relative path.

    `metadoc.is_area` is the filter, so a directory index, the three top-level
    documents, and every file in a directory carrying no index are excluded
    here for the same reasons they are excluded there -- an index has no owner
    bullet, a project README owns headings that are not promises at all, and an
    unindexed directory holds reference material rather than promises.
    """
    root = repo / project / metadoc.AREAS
    out: dict[str, str] = {}
    if not root.is_dir():
        return out
    for path in sorted(root.rglob("*.md")):
        rel = path.relative_to(repo).as_posix()
        if metadoc.is_area(repo, project, rel):
            out[rel] = path.read_text(encoding="utf-8")
    return out


def promises(texts: dict[str, str]) -> list[Promise]:
    """Every section of every area file, read through `metadoc`'s own parser."""
    out: list[Promise] = []
    for path, text in sorted(texts.items()):
        for raw, body in metadoc.sections(text):
            title = metadoc.bare(raw)
            if title == metadoc.FOOTER:
                continue
            keys = metadoc.bullets(body)
            owners = [k for k, _ in keys if k in metadoc.OWNERS]
            shape = ""
            if len(owners) == 1:
                shape = "future" if owners[0] == "Outcome" else "shipped"
            found = metadoc.IID.search(raw)
            # The first backticked token of `Outcome:`, and every one of
            # `Status rows:`. That is exactly what `metadoc`'s `P7` resolves --
            # taking a different count here would let a section `P7` accepts
            # claim an id `G4` cannot find.
            outcome = ""
            rows: list[str] = []
            for key, rest in keys:
                if key == "Outcome" and not outcome:
                    ids = metadoc.BACKTICKED.findall(rest)
                    outcome = ids[0] if ids else ""
                if key == "Status rows":
                    rows.extend(metadoc.BACKTICKED.findall(rest))
            out.append(Promise(path, title,
                               int(found.group(1)) if found else None,
                               shape, outcome, tuple(rows)))
    return out


def roadmap_ids(repo: Path, project: str) -> list[str]:
    path = repo / project / "ROADMAP.md"
    if not path.is_file():
        return []
    return ROADMAP_ID.findall(path.read_text(encoding="utf-8"))


def status_ids(repo: Path, project: str) -> list[str]:
    """Every id in the `## Support matrix`, in the order the table lists them."""
    path = repo / project / "STATUS.md"
    if not path.is_file():
        return []
    out: list[str] = []
    inside = False
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("## "):
            inside = line[3:].strip() == STATUS_MATRIX
            continue
        if not inside or not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) <= STATUS_ID_COLUMN:
            continue
        found = metadoc.BACKTICKED.findall(cells[STATUS_ID_COLUMN])
        if len(found) == 1:
            out.append(found[0])
    return out


# --------------------------------------------------------------------------
# the ledger
# --------------------------------------------------------------------------
@dataclass(frozen=True)
class Gap:
    """One distance, named by the row that measured it.

    `where` is a repo-relative path for the rows that read files and an issue
    reference for the two that read the tracker. Both are printed the same way,
    because both are somewhere a human goes to close the gap.
    """

    rule: str
    where: str
    message: str

    def as_dict(self) -> dict:
        return {"rule": self.rule, "where": self.where, "message": self.message}


class Ledger:
    """Every row's findings, and whether the row ran at all.

    The two dictionaries are the point. `population` is what a row read;
    `blocked` is why it could not read anything. `report` refuses a table where
    a row appears in neither, because that row would print a blank count a
    reader has no way to tell from a clean one.
    """

    def __init__(self) -> None:
        self.gaps: list[Gap] = []
        self.population: dict[str, int] = {}
        self.blocked: dict[str, str] = {}

    def measured(self, rule: str, size: int) -> None:
        self.population[rule] = size

    def cannot(self, rule: str, why: str) -> None:
        self.blocked[rule] = why

    def add(self, rule: str, where: str, message: str) -> None:
        self.gaps.append(Gap(rule, where, message))


# --------------------------------------------------------------------------
# the seven rows
# --------------------------------------------------------------------------
def g1_unbound_promises(led: Ledger, found: list[Promise]) -> None:
    """A section promising a future outcome that no epic was opened for."""
    shapeless = [p for p in found if not p.shape]
    if shapeless:
        # Not a gap, and not this file's finding -- `metadoc.py`'s `P5` owns
        # it. But a section with no owner bullet has no kind, and asking
        # whether a section of unknown kind owes an epic is asking a question
        # with no answer. Blocking names the count and the command that fixes
        # it.
        led.cannot("G1", f"{len(shapeless)} section(s) carry no single owner "
                         "bullet, so their kind is unknown; run `metadoc.py "
                         "check` first")
        return
    future = [p for p in found if p.shape == "future"]
    led.measured("G1", len(future))
    for promise in future:
        if promise.iid is None:
            led.add("G1", promise.path,
                    f"section `{promise.title}` promises a future outcome and "
                    "carries no ` (#<iid>)`; nothing on the tracker owns it")


def g2_orphan_items(led: Ledger, items: list[dict],
                    found: list[Promise], label: str) -> None:
    """An open work item that no promise reaches.

    Two shapes, because ownership is spelled two ways. An epic is reached by a
    section binding to its number. A change is reached through its epic, and
    `epic:<iid>` is the whole of that claim -- a change naming its parent in
    prose is a change nothing can walk to.
    """
    open_items = [i for i in items if i.get("state", "").upper() == "OPEN"]
    led.measured("G2", len(open_items))
    bound = {p.iid for p in found if p.iid is not None}
    for issue in open_items:
        labels = issue.get("labels", [])
        ref = f"#{issue['number']}"
        title = issue.get("title", "")
        if epic.TYPE_LABEL in labels:
            if issue["number"] not in bound:
                led.add("G2", ref, f"epic `{title}` carries `{label}` and no "
                                   "section binds to it")
        elif change.TYPE_LABEL in labels:
            if not any(name.startswith(change.PARENT_LABEL_PREFIX)
                       for name in labels):
                led.add("G2", ref, f"change `{title}` carries no "
                                   f"`{change.PARENT_LABEL_PREFIX}<iid>` label, "
                                   "so no epic owns it")


def g3_stale_bindings(led: Ledger, items: list[dict],
                      found: list[Promise], label: str) -> None:
    """A promise bound to an issue that cannot carry it.

    Two ways that happens, and the second one is why this row is not simply
    "the issue is closed". A shipped section whose epic closed is correct, and
    is the normal end state -- reporting it would make every finished promise a
    gap. A *future* section whose epic closed is the defect: the work landed
    and the section still describes it as ahead.
    """
    bound = [p for p in found if p.iid is not None]
    led.measured("G3", len(bound))
    state = {i["number"]: i.get("state", "").upper() for i in items}
    for promise in bound:
        if promise.iid not in state:
            led.add("G3", promise.path,
                    f"section `{promise.title}` binds to #{promise.iid}, which "
                    f"carries no `{label}` label; the binding reaches nothing "
                    "in this project")
        elif state[promise.iid] == "CLOSED" and promise.shape == "future":
            led.add("G3", promise.path,
                    f"section `{promise.title}` still promises a future "
                    f"outcome, but #{promise.iid} is closed; rewrite it as a "
                    "shipped section, or open the next epic")


def g4_uncovered_outcomes(led: Ledger, ids: list[str],
                          found: list[Promise], project: str) -> None:
    if not ids:
        led.cannot("G4", f"{project}/ROADMAP.md has no `- ID: ` line to read")
        return
    led.measured("G4", len(ids))
    claimed = {p.outcome for p in found if p.outcome}
    for oid in ids:
        if oid not in claimed:
            led.add("G4", f"{project}/ROADMAP.md",
                    f"outcome `{oid}` is claimed by no section's `Outcome:` "
                    "bullet; either a promise is missing or the outcome is")


def g5_unpromised_surfaces(led: Ledger, ids: list[str],
                           found: list[Promise], project: str) -> None:
    if not ids:
        led.cannot("G5", f"{project}/STATUS.md has no `## {STATUS_MATRIX}` row "
                         "to read")
        return
    led.measured("G5", len(ids))
    claimed = {row for p in found for row in p.rows}
    for sid in ids:
        if sid not in claimed:
            led.add("G5", f"{project}/STATUS.md",
                    f"row `{sid}` is claimed by no section's `Status rows:` "
                    "bullet; the surface ships and no promise describes it")


def g6_unregistered_cases(led: Ledger, repo: Path, project: str) -> None:
    """A case the crate manifest does not run, in either direction.

    The e2e root is taken as `<project>/e2e` rather than through
    `leg.leg_root`, which resolves only under `apps/`. `libs/<name>` owes an
    `e2e/` tree too -- it just has no ladder to write it through -- and a row
    that skipped every library would report a clean manifest for half the
    checkout.
    """
    root = repo / project / "e2e"
    if not root.is_dir():
        led.cannot("G6", f"{project}/e2e/ does not exist; there is no case "
                         "inventory to compare")
        return
    inv = e2e.E2eInventory(root)
    # Three of `E2eInventory`'s four problems mean the inventory is unreadable:
    # no manifest, no package name, or autodiscovery left on. The fourth --
    # "declares no `[[test]]` under e2e/" -- is not a blocker but the maximal
    # gap, and every file on disk is reported against it below.
    if not inv.crate or "autotests" in inv.problem:
        led.cannot("G6", inv.problem.splitlines()[0] if inv.problem
                   else "the crate manifest is unreadable")
        return
    on_disk = sorted(p for p in root.glob("*.rs") if p.is_file())
    declared = {entry["path"]: entry["id"] for entry in inv.cases.values()}
    led.measured("G6", len(on_disk) + len(declared))
    for path in on_disk:
        rel = f"e2e/{path.name}"
        if rel not in declared:
            led.add("G6", f"{project}/{rel}",
                    "no `[[test]]` stanza names it, so `cargo test -p "
                    f"{inv.crate}` does not run it")
    for rel, case_id in sorted(declared.items()):
        if not (repo / project / rel).is_file():
            led.add("G6", f"{project}/Cargo.toml",
                    f'`[[test]] name = "{case_id}"` declares `{rel}`, which is '
                    "not on disk")


def g7_dead_gates(led: Ledger, repo: Path, project: str) -> None:
    """`meta.py`'s `M5` and `M6`, run rather than reimplemented.

    Those two rules already own "a gate that cargo exits 0 on" and "a gate
    naming a target that is not in the checkout", and they read the project
    README's `## Capabilities` section to find them. Calling them keeps one
    answer; the only thing added here is the vacuity guard, because `M5`/`M6`
    run over the READMEs `M4` recognised, and a file `M4` did not recognise is
    a file they read nothing from.
    """
    rel = f"{project}/README.md"
    if rel not in meta.tracked_docs(repo):
        led.cannot("G7", f"{rel} is not a tracked META-doc; M5/M6 would read "
                         "nothing")
        return
    findings, population = meta.collect(repo, ("M5", "M6"), (rel,))
    if not population.get("project_readmes"):
        led.cannot("G7", f"`meta.py` does not recognise {rel} as a project "
                         "README, so M5/M6 measured no capability gate")
        return
    led.measured("G7", population["project_readmes"])
    for finding in findings:
        led.add("G7", finding.path, f"{finding.rule} at line {finding.line}: "
                                    f"{finding.message}")


# --------------------------------------------------------------------------
# collection and report
# --------------------------------------------------------------------------
def tracker(project: str, given: str | None) -> tuple[list[dict] | None, str, str]:
    """Every issue carrying the project's label, or `None` and why not.

    `None` is the whole point of this function's shape. `gh` outside a git
    directory exits non-zero with an empty stdout, and a caller that turned
    that into `[]` would report an empty backlog -- every promise orphaned,
    every binding stale -- with no sign anything had gone wrong. `workitem.gh`
    raises on a non-zero status, so the failure arrives as an exception and
    leaves here as an absence rather than as a count.
    """
    name = project.rsplit("/", 1)[1]
    label = workitem.project_label(name)
    try:
        slug = given or workitem.default_repo()
        return workitem.fetch_issues_by_label(label, slug), label, slug
    except workitem.GhError as exc:
        return None, label, f"unreadable: {str(exc).splitlines()[0]}"
    except json.JSONDecodeError as exc:
        return None, label, f"unreadable: gh returned no JSON ({exc})"


def collect(repo: Path, project: str,
            given_repo: str | None) -> tuple[Ledger, dict]:
    texts = area_texts(repo, project)
    found = promises(texts)
    items, label, slug = tracker(project, given_repo)

    led = Ledger()
    if not texts:
        # Five of the seven rows are derived from sections. With no area file
        # there is nothing to derive them from, and each one says so in its own
        # row rather than the run printing one clean table with five zeroes.
        why = (f"{project}/{metadoc.AREAS}/ holds no area file; there is no "
               "promise to measure against")
        for rule in ("G1", "G2", "G3", "G4", "G5"):
            led.cannot(rule, why)
    else:
        g1_unbound_promises(led, found)
        g4_uncovered_outcomes(led, roadmap_ids(repo, project), found, project)
        g5_unpromised_surfaces(led, status_ids(repo, project), found, project)
        if items is None:
            for rule in ("G2", "G3"):
                led.cannot(rule, f"the tracker is {slug}")
        else:
            g2_orphan_items(led, items, found, label)
            g3_stale_bindings(led, items, found, label)
    g6_unregistered_cases(led, repo, project)
    g7_dead_gates(led, repo, project)

    population = {
        "project": project,
        "label": label,
        "area_files": len(texts),
        "sections": len(found),
        "work_items": None if items is None else len(items),
    }
    return led, population


def report(led: Ledger, population: dict, fmt: str, next_command: str) -> int:
    # A row in neither dictionary would print an empty cell that reads exactly
    # like a clean one. That is the defect this whole file is arranged against,
    # so it is refused here rather than printed -- and refused as an invocation
    # error, because it is a defect in this script and not in the project.
    missing = [r for r in GAPS if r not in led.population and r not in led.blocked]
    if missing:
        usage_error("internal: row(s) " + ", ".join(missing) +
                    " were neither measured nor blocked; the table would print "
                    "a count nobody produced")

    if fmt == "json":
        print(json.dumps({
            "population": population,
            "rows": {rule: {"why": why,
                            "population": led.population.get(rule),
                            "blocked": led.blocked.get(rule),
                            "gaps": sum(1 for g in led.gaps if g.rule == rule)}
                     for rule, why in GAPS.items()},
            "gaps": [g.as_dict() for g in led.gaps],
        }, indent=2))
        return 1 if led.gaps or led.blocked else 0

    seen = population["work_items"]
    print(f"WI gap: {population['project']}, {population['area_files']} area "
          f"file(s), {population['sections']} section(s), "
          f"{'unreadable' if seen is None else seen} "
          f"`{population['label']}` work item(s)")
    for rule, why in GAPS.items():
        count = sum(1 for g in led.gaps if g.rule == rule)
        if rule in led.blocked:
            print(f"  {rule:<4}    ? / ?     {why}")
        else:
            print(f"  {rule:<4} {count:>4} / {led.population[rule]:<5} {why}")

    for rule in GAPS:
        if rule in led.blocked:
            print(f"\n{rule} UNMEASURED: {led.blocked[rule]}")

    current = ""
    for gap in sorted(led.gaps, key=lambda g: (g.where, g.rule, g.message)):
        if gap.where != current:
            current = gap.where
            print(f"\n{current}")
        print(f"  {gap.rule:<4} {gap.message}")

    if not led.gaps and not led.blocked:
        print("\n=> ALIGNED")
        print(f"next.command: {next_command}")
        return 0
    verdict = []
    if led.gaps:
        verdict.append(f"{len(led.gaps)} gap(s)")
    if led.blocked:
        verdict.append(f"{len(led.blocked)} row(s) unmeasured")
    print(f"\n=> {', '.join(verdict)}")
    print("next.command: reorganise the work items above through `epic.py "
          "create|update` and `change.py create|update`, then re-run this verb")
    return 1


def cmd_gap(args: argparse.Namespace) -> int:
    repo = leg.repo_root()
    project = metadoc.resolve_project(repo, args.project)
    led, population = collect(repo, project, args.repo)
    launcher = " ".join(metadoc.PINNED)
    # Printed with the pinned launcher rather than as a bare script name.
    # `wis.py` reaches `tomllib` through `e2e.py`, and the interpreter a bare
    # `wis.py` resolves to is 3.9 on at least one machine here -- where the
    # failure is a `ModuleNotFoundError` traceback that reads like a broken
    # script rather than a wrong interpreter.
    return report(led, population, args.format,
                  f'{launcher} ".claude/aw/scripts/epic.py" order <iid> '
                  "--open-only")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="wis.py",
        description="Measure the distance between a project's promises, its "
                    "codebase, and its open work items.")
    sub = parser.add_subparsers(dest="verb", required=True)

    gap = sub.add_parser("gap", help="print the measurable gap table")
    gap.add_argument("project",
                     help="`apps/<name>`, `libs/<name>`, or a bare name")
    gap.add_argument("--repo", default=None,
                     help="issue platform `owner/repo`; defaults to aw.toml")
    gap.add_argument("--format", choices=("text", "json"), default="text")
    gap.set_defaults(func=cmd_gap)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
