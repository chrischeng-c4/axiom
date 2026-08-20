#!/usr/bin/env python3
"""`epic.py order`, and the three reconcile findings that make it refusable.

An epic driven child-by-child needs a sequence, and until now nothing computed
one. The parts were already in the tracker and simply had no consumer:

  `## Verification Inventory` carries a fourth column, `Depends On`, whose
  cells name requirements. Measured over the 255-epic snapshot: 45 epics carry
  the column, 38 of them have populated cells, 492 rows in total. The
  spellings are the same ones `_requirement_refs` already normalises -- `R1`,
  `R1, R3`, `R1-R4, R7-R12`, and `-` for none.

  `## Child Work Items` carries a table mapping an issue to the requirements
  it covers. 21 epics have the section; 13 carry both an issue reference and a
  requirement reference in it.

So the order is a composition: requirements are partially ordered by
`Depends On`, children inherit the position of the requirements they cover,
and ties break by `priority:` then by issue number.

The three findings exist because each is a way the composition has no answer,
and each is mechanically decidable -- which is what makes them structural
rather than a judgement handed back to the caller:

  `dangling-dependency`  a `Depends On` cell names a requirement `##
                         Requirements` never declared. The edge points at
                         nothing, so the order it implies is unverifiable.

  `cyclic-dependency`    the requirement graph has a cycle, so no sequence
                         satisfies it. `order` must then return *no* order.
                         Emitting an arbitrary one would present a guess as a
                         computation, which is the single failure mode this
                         whole verb is worth having only if it avoids.

  `undeclared-order`     an owned child that the `Child Work Items` table does
                         not map to any requirement. It has no position, and
                         appending it to the end would again be a guess.

The corpus control matters more than the fixtures here. Over the real
snapshot, cycles and dangling refs both measure **zero** -- which is precisely
what a detector that never fires also reports. So the control seeds one of
each into a copy of the corpus and requires the counts to move to exactly one:
the baseline is evidence only once the instrument is shown to move.
"""
from __future__ import annotations

import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import SNAPSHOTS, load_epic_module  # noqa: E402

epic = load_epic_module()

REPO = "owner/repo"

# The corpus baseline, declared rather than computed at run time. A control
# that recomputed its own expectation would agree with any answer.
#
# `GRAPHED_EPICS` was first measured at 38 by an ad-hoc script and is 32. The
# 38 is not reproducible by any reading of the column -- 45 epics carry it, 32
# have at least one cell holding a requirement reference, 226 cells say `-` or
# `none` -- so it was discarded rather than reconciled. Two independent
# implementations agree on 32.
GRAPHED_EPICS = 32
CORPUS_CYCLES = 0
CORPUS_DANGLING = 0
# Non-zero on purpose, and the reason this row is worth more than the three
# above it: 10 epics fill a `Depends On` with an issue number or prose. This
# count firing is itself the proof that the detector fires.
CORPUS_UNREADABLE = 10


def body(*, requirements: list[str], inventory: list[str],
         children: list[str] | None = None) -> str:
    """An epic body carrying only the sections the order reads.

    Deliberately not a valid epic: `reconcile_findings` and `order_children`
    parse sections, they do not validate them, and a fixture that had to
    satisfy every section rule would make each control a test of the
    skeleton too.
    """
    text = "## Requirements\n\n" + "\n".join(requirements) + "\n\n"
    if children is not None:
        text += "## Child Work Items\n\n" + "\n".join(children) + "\n\n"
    text += ("## Verification Inventory\n\n"
             "| Requirement | Gate | Oracle | Depends On |\n"
             "|---|---|---|---|\n" + "\n".join(inventory) + "\n")
    return text


def issue(number: int, *, priority: str | None = "p2", state: str = "OPEN",
          title: str | None = None) -> dict:
    labels = ["type:change"]
    if priority:
        labels.append(f"priority:{priority}")
    return {"number": number, "title": title or f"child {number}",
            "state": state, "labels": labels, "url": ""}


def kinds(findings, *names) -> list[str]:
    return [f.kind for f in findings if f.kind in names]


class Report:
    def __init__(self) -> None:
        self.rows: list[tuple[str, str, str, bool]] = []

    def record(self, name: str, want: str, got: str, ok: bool) -> None:
        self.rows.append((name, want, got, ok))

    def verdict(self) -> int:
        print(f"\n{'':8s} {'control':62s} observation")
        for name, want, got, ok in self.rows:
            print(f"{'PASS' if ok else '**FAIL**':8s} {name:62s} want {want}; got {got}")
        bad = [r for r in self.rows if not r[3]]
        print(f"\n{len(self.rows) - len(bad)}/{len(self.rows)} controls behaved as declared")
        return 1 if bad else 0


# --------------------------------------------------------------------------
# the fixtures
# --------------------------------------------------------------------------
CHAIN = body(
    requirements=["- R1: the first", "- R2: the second", "- R3: the third"],
    inventory=["| R1 | `g1` | o | - |",
               "| R2 | `g2` | o | R1 |",
               "| R3 | `g3` | o | R2 |"],
    children=["| Work item | Requirements |", "| --- | --- |",
              "| #30 | R3 |", "| #10 | R1 |", "| #20 | R2 |"],
)

# `R1 -> {R2, R3} -> R4`. The two middle requirements are genuinely unordered
# with respect to each other, which is the only place a tie-break can be
# observed at all: in a chain every tie-break rule produces the same answer.
DIAMOND = body(
    requirements=["- R1: the root", "- R2: a branch", "- R3: the other branch",
                  "- R4: the join"],
    inventory=["| R1 | `g1` | o | - |",
               "| R2 | `g2` | o | R1 |",
               "| R3 | `g3` | o | R1 |",
               "| R4 | `g4` | o | R2, R3 |"],
    children=["| Work item | Requirements |", "| --- | --- |",
              "| #11 | R1 |", "| #12 | R2 |", "| #13 | R3 |", "| #14 | R4 |"],
)


def main() -> int:
    r = Report()
    order = epic.order_children
    reconcile = epic.reconcile_findings

    # -- positive control ---------------------------------------------------
    #
    # First and alone in intent: every control below claims some input makes
    # the order refuse, and none of those claims means anything if the order
    # cannot be produced from an input that is entirely well-formed.
    e = {"number": 1, "body": CHAIN}
    kids = [issue(10), issue(20), issue(30)]
    out = order(e, kids)
    got = [row["number"] for row in out["order"]]
    r.record("positive control: a declared chain orders end to end",
             "['10', '20', '30'], nothing unplaced",
             f"{got}, unordered {out['unordered']}",
             got == ["10", "20", "30"] and not out["unordered"]
             and not out["cycle"] and not out["dangling"])

    # The children are declared out of order in the table on purpose: a verb
    # that returned them in declaration order, or in issue order, would agree
    # with the chain here by accident. It does not agree here.
    r.record("the order is the dependency order, not the declared order",
             "the table's own order ['30','10','20'] is not the answer",
             f"{got}", got != ["30", "10", "20"])

    # -- the tie-breaks -----------------------------------------------------
    out = order({"number": 2, "body": DIAMOND},
                [issue(11), issue(12, priority="p3"), issue(13, priority="p0"),
                 issue(14)])
    got = [row["number"] for row in out["order"]]
    r.record("independent requirements tie-break by priority",
             "['11', '13', '12', '14'] -- p0 ahead of p3 at equal depth",
             f"{got}", got == ["11", "13", "12", "14"])

    out = order({"number": 3, "body": DIAMOND},
                [issue(11), issue(13), issue(12), issue(14)])
    got = [row["number"] for row in out["order"]]
    r.record("equal priority tie-breaks by issue number",
             "['11', '12', '13', '14']", f"{got}",
             got == ["11", "12", "13", "14"])

    # A child covering two requirements cannot start until the deeper one is
    # reachable. Taking the shallower would place it before work it needs.
    two = body(
        requirements=["- R1: a", "- R2: b", "- R3: c"],
        inventory=["| R1 | `g` | o | - |", "| R2 | `g` | o | R1 |",
                   "| R3 | `g` | o | R2 |"],
        children=["| Work item | Requirements |", "| --- | --- |",
                  "| #41 | R1, R3 |", "| #42 | R2 |"],
    )
    out = order({"number": 4, "body": two}, [issue(41), issue(42)])
    got = [row["number"] for row in out["order"]]
    r.record("a child covering two requirements takes the deeper position",
             "['42', '41'] -- #41 covers R3, so it cannot precede R2",
             f"{got}", got == ["42", "41"])

    # -- dangling-dependency ------------------------------------------------
    dangling = body(
        requirements=["- R1: a", "- R2: b"],
        inventory=["| R1 | `g` | o | - |", "| R2 | `g` | o | R1, R9 |"],
        children=["| Work item | Requirements |", "| --- | --- |",
                  "| #51 | R1 |", "| #52 | R2 |"],
    )
    e = {"number": 5, "body": dangling}
    kids = [issue(51), issue(52)]
    out = order(e, kids)
    found = [f for f in reconcile(e, kids, REPO) if f.kind == "dangling-dependency"]
    named = bool(found) and "R9" in found[0].detail
    r.record("a Depends On naming an undeclared requirement is refused",
             "one dangling-dependency finding naming R9",
             f"{len(found)} finding(s); names R9: {named}; "
             f"order.dangling {out['dangling']}",
             len(found) == 1 and named and out["dangling"] == [9])

    # -- cyclic-dependency --------------------------------------------------
    cyclic = body(
        requirements=["- R1: a", "- R2: b"],
        inventory=["| R1 | `g` | o | R2 |", "| R2 | `g` | o | R1 |"],
        children=["| Work item | Requirements |", "| --- | --- |",
                  "| #61 | R1 |", "| #62 | R2 |"],
    )
    e = {"number": 6, "body": cyclic}
    kids = [issue(61), issue(62)]
    out = order(e, kids)
    found = [f for f in reconcile(e, kids, REPO) if f.kind == "cyclic-dependency"]
    r.record("a requirement cycle is refused",
             "one cyclic-dependency finding", f"{len(found)} finding(s)",
             len(found) == 1)

    # The claim the verb is worth having: not that it *reports* the cycle, but
    # that it declines to answer. A topo sort that fell back to declaration
    # order here would hand back a sequence that looks exactly like a computed
    # one, and the finding beside it would read as advisory.
    r.record("a cycle produces no order at all, not an arbitrary one",
             "an empty order and both children unplaced",
             f"order {[x['number'] for x in out['order']]}, "
             f"unordered {sorted(out['unordered'])}",
             out["order"] == [] and sorted(out["unordered"]) == ["61", "62"])

    # -- undeclared-order ---------------------------------------------------
    e = {"number": 7, "body": CHAIN}
    kids = [issue(10), issue(20), issue(30), issue(99)]
    out = order(e, kids)
    found = [f for f in reconcile(e, kids, REPO) if f.kind == "undeclared-order"]
    named = bool(found) and "99" in found[0].subjects
    r.record("an owned child the table never maps has no position",
             "one undeclared-order finding naming #99, and #99 unplaced",
             f"{len(found)} finding(s); names it: {named}; "
             f"unordered {out['unordered']}",
             len(found) == 1 and named and out["unordered"] == ["99"])

    r.record("the placed children are still placed alongside it",
             "['10', '20', '30'] ordered despite the unplaced one",
             f"{[x['number'] for x in out['order']]}",
             [x["number"] for x in out["order"]] == ["10", "20", "30"])

    # -- unreadable-dependency ----------------------------------------------
    #
    # The live corpus fills this cell with an issue number 8 times and with
    # prose 4 times. Both are dependencies the author declared and the parser
    # cannot read, and reading `#2403` as an edge would mean guessing whether
    # they meant that issue or the requirement it covers.
    unreadable = body(
        requirements=["- R1: a", "- R2: b", "- R3: c"],
        inventory=["| R1 | `g` | o | - |",
                   "| R2 | `g` | o | #2403 |",
                   "| R3 | `g` | o | R1 |"],
        children=["| Work item | Requirements |", "| --- | --- |",
                  "| #81 | R1 |", "| #82 | R2 |", "| #83 | R3 |"],
    )
    e = {"number": 9, "body": unreadable}
    kids = [issue(81), issue(82), issue(83)]
    out = order(e, kids)
    found = [f for f in reconcile(e, kids, REPO) if f.kind == "unreadable-dependency"]
    r.record("a Depends On naming an issue instead of a requirement is reported",
             "one unreadable-dependency finding quoting the cell",
             f"{len(found)} finding(s); quotes it: "
             f"{bool(found) and '#2403' in found[0].detail}",
             len(found) == 1 and "#2403" in found[0].detail)

    # It is not a dangling ref: nothing named R-anything. Reporting both would
    # describe one defect twice and make the real count unreadable.
    r.record("an unreadable cell is not also reported as dangling",
             "no dangling-dependency finding", f"dangling {out['dangling']}",
             not out["dangling"]
             and not kinds(reconcile(e, kids, REPO), "dangling-dependency"))

    # The part that could be parsed still yields an order. Withholding it would
    # lose R3's real edge, which is correct, over R2's cell, which is not.
    r.record("the readable edges still produce an order beside the finding",
             "#83 after #81, and the flag set",
             f"order {[x['number'] for x in out['order']]}, "
             f"unreadable {len(out['unreadable'])}",
             [x["number"] for x in out["order"]] == ["81", "82", "83"]
             and len(out["unreadable"]) == 1)

    # 226 of the 261 filled cells say `-` or `none`. If either read as a
    # dependency, this finding would fire on essentially every epic that has
    # the column, which is the same as not having the finding.
    empties = body(
        requirements=["- R1: a", "- R2: b"],
        inventory=["| R1 | `g` | o | - |", "| R2 | `g` | o | none |"],
        children=["| Work item | Requirements |", "| --- | --- |",
                  "| #91 | R1 |", "| #92 | R2 |"],
    )
    e = {"number": 10, "body": empties}
    kids = [issue(91), issue(92)]
    found = kinds(reconcile(e, kids, REPO), "unreadable-dependency",
                  "dangling-dependency")
    r.record("`-` and `none` are how the column says nothing",
             "no finding for either spelling", f"findings {found}", not found)

    # -- no mapping at all is one fact, not one fact per child --------------
    #
    # 21 of 255 epics carry `## Child Work Items`; on the other 234 every owned
    # child is unmapped. `reconcile` already stays silent there. `order` has to
    # make the same distinction, and it can only do that if the payload carries
    # it: rendered per child, "there is no mapping" prints the identical line
    # once per child, which on #3346 is 30+ lines saying one thing.
    nomap = body(
        requirements=["- R1: a", "- R2: b"],
        inventory=["| R1 | `g` | o | - |", "| R2 | `g` | o | R1 |"],
    )
    e = {"number": 11, "body": nomap}
    kids = [issue(101), issue(102)]
    out = order(e, kids)
    r.record("an epic with no child mapping is one fact, carried in the payload",
             "maps_children False, both children unordered",
             f"maps_children {out['maps_children']}, unordered {out['unordered']}",
             out["maps_children"] is False and out["unordered"] == ["101", "102"])

    r.record("an epic that does map its children sets the same flag the other way",
             "maps_children True",
             f"{order({'number': 12, 'body': CHAIN}, [issue(10)])['maps_children']}",
             order({"number": 12, "body": CHAIN}, [issue(10)])["maps_children"] is True)

    # -- the absence of the column is not a defect --------------------------
    #
    # 14 of the 59 epics carrying an inventory use the three-column form. If a
    # missing `Depends On` read as "every requirement depends on nothing
    # unknown" *and* as a finding, every one of those epics would report a
    # defect it does not have -- and a finding that fires on a quarter of the
    # corpus stops being read.
    flat = ("## Requirements\n\n- R1: a\n- R2: b\n\n"
            "## Child Work Items\n\n| Work item | Requirements |\n| --- | --- |\n"
            "| #71 | R1 |\n| #72 | R2 |\n\n"
            "## Verification Inventory\n\n| Requirement | Gate | Oracle |\n"
            "|---|---|---|\n| R1 | `g` | o |\n| R2 | `g` | o |\n")
    e = {"number": 8, "body": flat}
    kids = [issue(71), issue(72)]
    out = order(e, kids)
    found = kinds(reconcile(e, kids, REPO),
                  "dangling-dependency", "cyclic-dependency", "undeclared-order")
    r.record("a three-column inventory is not a finding",
             "no new finding, both children at rank 0",
             f"findings {found}; ranks "
             f"{[x['rank'] for x in out['order']]}",
             not found and [x["rank"] for x in out["order"]] == [0, 0])

    # -- the exit contract --------------------------------------------------
    #
    # `cmd_order` returns `0 if payload["orderable"] else 1`, so this one flag
    # is the whole contract a driver reads. It is asserted over every shape at
    # once because the interesting case is the third: an unreadable cell still
    # prints an order, and exiting 0 there would have a script run children in
    # a sequence the epic did not ask for while the human reading the same
    # stdout is told to stop.
    shapes = [
        ("a clean chain", CHAIN, [issue(10), issue(20), issue(30)], True),
        ("a cycle", cyclic, [issue(61), issue(62)], False),
        ("an unmapped child", CHAIN,
         [issue(10), issue(20), issue(30), issue(99)], False),
        ("an unreadable cell", unreadable, [issue(81), issue(82), issue(83)], False),
        ("a three-column inventory", flat, [issue(71), issue(72)], True),
        ("no child mapping at all", nomap, [issue(101), issue(102)], False),
    ]
    got = {name: order({"number": 13, "body": text}, kids)["orderable"]
           for name, text, kids, _ in shapes}
    r.record("`orderable` is set for exactly the shapes that have an order",
             str({name: want for name, _, _, want in shapes}), str(got),
             all(got[name] is want for name, _, _, want in shapes))

    # -- the corpus, and the proof the instrument moves ---------------------
    snapshot = SNAPSHOTS / "epics.json"
    if not snapshot.is_file():
        r.record("the corpus baseline", "epics.json present",
                 f"missing at {snapshot}", False)
        return r.verdict()
    corpus = json.loads(snapshot.read_text(encoding="utf-8"))

    graphed = cycles = dangles = unread = 0
    for item in corpus:
        out = order({"number": item["number"], "body": item.get("body") or ""}, [])
        unread += bool(out["unreadable"])
        if not out["graph"]:
            continue
        graphed += 1
        cycles += bool(out["cycle"])
        dangles += bool(out["dangling"])
    r.record(f"the real corpus ({len(corpus)} epics) reads clean",
             f"{GRAPHED_EPICS} graphed, {CORPUS_CYCLES} cyclic, "
             f"{CORPUS_DANGLING} dangling",
             f"{graphed} graphed, {cycles} cyclic, {dangles} dangling",
             graphed == GRAPHED_EPICS and cycles == CORPUS_CYCLES
             and dangles == CORPUS_DANGLING)

    r.record("the corpus's real unreadable cells are all found",
             f"{CORPUS_UNREADABLE} epics", f"{unread} epics",
             unread == CORPUS_UNREADABLE)

    # The row above is a baseline of zero, and a detector that never fires
    # reports zero too. One seeded defect of each kind, into the first epic
    # that actually has a graph, has to move both counts to one.
    seed = next(item for item in corpus
                if order({"number": item["number"],
                          "body": item.get("body") or ""}, [])["graph"])
    seeded_cycle = seed["body"].replace(
        "| Requirement | Gate | Oracle | Depends On |",
        "| Requirement | Gate | Oracle | Depends On |\n|---|---|---|---|\n"
        "| R901 | `g` | o | R902 |\n| R902 | `g` | o | R901 |", 1)
    seeded_dangle = seed["body"].replace(
        "| Requirement | Gate | Oracle | Depends On |",
        "| Requirement | Gate | Oracle | Depends On |\n|---|---|---|---|\n"
        "| R903 | `g` | o | R904 |", 1)
    moved_c = order({"number": seed["number"], "body": seeded_cycle}, [])
    moved_d = order({"number": seed["number"], "body": seeded_dangle}, [])
    r.record("one seeded cycle and one seeded dangling ref are both seen",
             f"a cycle and a dangling ref in #{seed['number']}",
             f"cycle {moved_c['cycle']}; dangling {moved_d['dangling']}",
             bool(moved_c["cycle"]) and moved_d["dangling"] == [904])

    return r.verdict()


if __name__ == "__main__":
    sys.exit(main())
