#!/usr/bin/env python3
"""Epic work-item surface, prototyped in Python over the `gh` CLI.

This is the surface `/aw:wi-epic-grill` and `/aw:wi-epic-reconcile` drive. It exists
so the epic type axis can be proven -- section schema, terminal-state rule,
reconciliation findings -- before any of it is spent on a Rust verb axis. It
deliberately does not call `aw`: the tracker is GitHub, and `gh` reaches it
directly.

Everything that does not know it is serving an epic lives in `workitem.py`
beside this file. What stays here is the epic itself: its sections, its
cross-section coverage rule, its `epic:` ownership prefix, and the four verbs
that mean nothing for a type without children -- `show`'s child readout,
`children`, `reconcile`, and `close`.

The section schema below is a table, not code. Adding `change`, `spike`, or
`report` means a second facade this thin over the same engine, not a copied
verb set.

Verbs
-----
  skeleton   emit the empty section template an epic body must fill
  validate   check a body (by iid or by file) against the section schema
  show       one epic: body, labels, state
  children   the epic's owned child set
  reconcile  read-only findings: structural (computed) and semantic (to ask)
  create     open a new epic
  update     edit an existing epic's body, title, or labels
  close      close an epic, refusing while any owned child is still open

Every write verb accepts --dry-run and prints the exact `gh` command it would
run. Deciding *whether* to write is the caller's job, not this script's.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

# The script's directory is on `sys.path` when this runs as a script, but not
# when it is loaded through `importlib.exec_module` -- which is how
# `verification/_paths.py` imports it. Inserting it explicitly makes both entry
# paths resolve the engine the same way, instead of leaving the gates able to
# import a module the CLI cannot.
sys.path.insert(0, str(Path(__file__).resolve().parent))

import workitem  # noqa: E402
from workitem import (  # noqa: E402,F401
    AW_TOML,
    PRIORITIES,
    REPO_ROOT,
    WORK_ITEM_TYPES,
    WORKITEMS_DIR_REL,
    GhError,
    Section,
    WorkItemType,
    default_repo,
    fetch_issue,
    gh,
    in_staging_tree,
    issue_number_from_create_output,
    project_label,
    rename_to_iid,
    run_or_show,
    split_sections,
    staging_dir,
)

TYPE_LABEL = "type:epic"
PHASE_LABEL = "phase:created"
CHILD_LABEL_PREFIX = "epic:"

# The H2 sections whose issue references are an ownership claim rather than a
# reference. `reconcile` reads only these when deciding what the epic declares
# as its own; see the note in `reconcile_findings`.
CHILD_DECLARING_SECTIONS = ("Child Work Items",)


# --------------------------------------------------------------------------
# Section schema
# --------------------------------------------------------------------------


def _has_requirement_items(text: str) -> bool:
    return bool(re.search(r"^\s*-\s*R\d+:\s*\S", text, re.M))


def _has_list_item(text: str) -> bool:
    return bool(re.search(r"^\s*[-*]\s+\S", text, re.M))


def _has_capability_fields(text: str) -> bool:
    """Accept the bare, list-item, and bold spellings: real epics use all three,
    and the field that matters is the label, not its markdown dressing."""
    return all(
        re.search(rf"^\s*(?:[-*]\s*)?\**{field_name}\**:\s*\S", text, re.M)
        for field_name in ("Capability", "Capability Gap", "Progress Evidence")
    )


def _has_inventory_table(text: str) -> bool:
    """`Depends On` is optional: 8 of the 35 epics carrying an inventory use the
    three-column form, so requiring the fourth column would refuse bodies the
    tracker already accepts."""
    for line in text.splitlines():
        cells = [cell.strip().lower() for cell in line.strip().strip("|").split("|")]
        if cells[:3] == ["requirement", "gate", "oracle"]:
            return True
    return False


def _has_body_rows(text: str) -> bool:
    """A table whose only row is the header proves nothing."""
    rows = [ln for ln in text.splitlines() if ln.strip().startswith("|")]
    return len(rows) >= 3  # header, separator, at least one row


EPIC_SECTIONS: tuple[Section, ...] = (
    Section(
        heading="Problem",
        guidance="the observable situation today, not the desired feature",
        rules=((lambda t: len(t.split()) >= 20, "must be substantive, not a placeholder"),),
    ),
    Section(
        heading="Capability Alignment",
        guidance="which capability this advances, the gap, and the evidence of progress",
        rules=(
            (
                _has_capability_fields,
                "must carry `Capability:`, `Capability Gap:`, and `Progress Evidence:` lines",
            ),
        ),
        # `tight` reproduces the shape this section has always had in the
        # skeleton: the three fields sit directly under the guidance comment,
        # with no blank line between, unlike every other section's starter
        # content.
        tight=True,
        template=("Capability: \nCapability Gap: \nProgress Evidence: \n",),
    ),
    Section(
        heading="Requirements",
        guidance="one observable requirement per line, numbered R1, R2, ...",
        rules=((_has_requirement_items, "must contain list items matching `- R<n>: ...`"),),
        template=("- R1: ", ""),
    ),
    Section(
        heading="Scope",
        guidance="what this epic covers and what it explicitly refuses",
        subsections=("In Scope", "Out of Scope"),
    ),
    Section(
        heading="Acceptance Criteria",
        guidance="the conditions under which this epic is done",
        rules=((_has_list_item, "must contain at least one list item"),),
    ),
    Section(
        heading="Verification Inventory",
        guidance="a table mapping every R<n> to a runnable gate and an observable oracle",
        rules=(
            (
                _has_inventory_table,
                "must contain a `| Requirement | Gate | Oracle | Depends On |` table header",
            ),
            (_has_body_rows, "the inventory table must carry at least one row below the header"),
        ),
        template=(
            "| Requirement | Gate | Oracle | Depends On |",
            "|---|---|---|---|",
            "| R1 | | | |",
            "",
        ),
    ),
    Section(
        heading="Reference Context",
        guidance="the specs this epic touches and the order it plans to touch them",
        subsections=("Related Specs", "Spec Plan"),
    ),
)


# --------------------------------------------------------------------------
# Cross-section rules
#
# A per-section rule sees only its own text, which is what keeps one section's
# check from silently depending on another. The cost of that isolation is that
# a claim spanning two sections has nowhere to live -- and the inventory's
# whole promise, "a table mapping *every* R<n> to a runnable gate", is exactly
# such a claim. Without this layer an epic declaring R1..R6 and carrying one
# inventory row validates green, because each section is individually
# well-formed and nothing compares them.
# --------------------------------------------------------------------------


def _requirement_refs(cell: str) -> set[int]:
    """Expand one `Requirement` cell into the R<n> it names.

    Live epics do not spell this one way, and refusing the spellings the
    tracker already accepts would be refusing formatting rather than checking
    coverage. Measured across 255 epics, the first column holds bare `R1`,
    ranges (`R1-R3`, `R7-R9`), lists (`R1, R2`, `R2, R3, R4`), suffixed refs
    (`R9 (Lumen)`), and refs mixed with other axes (`R11, AC8`) -- 21 distinct
    non-bare spellings. A bare-equality reading turns 8 of the 54 currently
    valid epics red without a single one of them actually missing coverage.

    The lookbehind keeps `PR12` or `CR3` from being read as a requirement
    reference; `AC2-AC6` carries no `R` at all and is ignored on its own.
    """
    refs: set[int] = set()
    for low, high in re.findall(r"(?<![A-Za-z])R(\d+)\s*[-–—]\s*R?(\d+)", cell):
        low, high = int(low), int(high)
        if low <= high:
            refs.update(range(low, high + 1))
    refs.update(int(n) for n in re.findall(r"(?<![A-Za-z])R(\d+)", cell))
    return refs


def _inventory_requirement_cells(text: str) -> list[str]:
    """The inventory table's first column, minus its header and separator."""
    cells: list[str] = []
    for line in text.splitlines():
        line = line.strip()
        if not line.startswith("|"):
            continue
        first = line.strip("|").split("|")[0].strip()
        if not first or set(first) <= set("-: ") or first.lower() == "requirement":
            continue
        cells.append(first)
    return cells


def _requirements_are_inventoried(sections: dict[str, str]) -> list[str]:
    """Every R<n> declared in `## Requirements` must appear in the inventory.

    Stays silent when either section is missing or malformed: the per-section
    rules already report that, and a second error naming the same defect makes
    the real one harder to find. One inventory row may discharge several
    requirements -- a range cell is a gate covering all of them, which is a
    claim the human can be held to, unlike a requirement no row mentions.
    """
    requirements = sections.get("Requirements")
    inventory = sections.get("Verification Inventory")
    if requirements is None or inventory is None:
        return []
    declared = {int(n) for n in re.findall(r"^\s*-\s*R(\d+):\s*\S", requirements, re.M)}
    if not declared:
        return []
    covered: set[int] = set()
    for cell in _inventory_requirement_cells(inventory):
        covered |= _requirement_refs(cell)
    missing = sorted(declared - covered)
    if not missing:
        return []
    named = ", ".join(f"R{n}" for n in missing)
    return [
        f"`## Verification Inventory` has no row for {named} -- every requirement "
        "needs a gate, or it is a promise nothing can refuse"
    ]


EPIC = WorkItemType(
    name="epic",
    type_label=TYPE_LABEL,
    sections=EPIC_SECTIONS,
    cross_rules=(_requirements_are_inventoried,),
    phase_label=PHASE_LABEL,
    prog="epic.py",
)


# --------------------------------------------------------------------------
# Epic-bound bindings
#
# The engine's `validate_body` and `skeleton` require a type; these supply it.
# The defaults are not sugar: `verification/` imports this module and calls
# both with no type, which is the right call against a module whose entire
# subject is the epic.
# --------------------------------------------------------------------------


def validate_body(body: str, wi_type: WorkItemType = EPIC) -> list[str]:
    """Return every reason this body is not a valid epic."""
    return workitem.validate_body(body, wi_type)


def skeleton(wi_type: WorkItemType = EPIC) -> str:
    """The empty section template an epic body must fill."""
    return workitem.skeleton(wi_type)


def fetch_children(iid: str, repo: str) -> list[dict]:
    """The issues claiming this epic as their owner.

    Ownership is the `epic:<iid>` label and nothing else: an issue that names
    the epic in its body but does not carry the label is not a child, which is
    exactly the gap `reconcile`'s `named-but-unowned` finding reports.
    """
    return workitem.fetch_issues_by_label(f"{CHILD_LABEL_PREFIX}{iid}", repo)


def require_epic(issue: dict, verb: str) -> None:
    workitem.require_type(issue, verb, EPIC)


# --------------------------------------------------------------------------
# Reconciliation
# --------------------------------------------------------------------------


@dataclass
class Finding:
    tier: str  # "structural" | "semantic"
    kind: str
    detail: str
    command: str | None = None
    subjects: list[str] = field(default_factory=list)

    def as_dict(self) -> dict:
        return {
            "tier": self.tier,
            "kind": self.kind,
            "detail": self.detail,
            "command": self.command,
            "subjects": self.subjects,
        }


def reconcile_findings(epic: dict, children: list[dict], repo: str) -> list[Finding]:
    """Split what is computable from what needs a human.

    Structural findings carry the exact command that repairs them: the caller
    transcribes a computed answer. Semantic findings carry no command, because
    deciding them means judging equivalence or scope -- which no rule here can
    do on the caller's behalf.
    """
    iid = str(epic["number"])
    body = epic.get("body") or ""
    sections = split_sections(body)
    findings: list[Finding] = []
    owned = {str(child["number"]) for child in children}

    # -- structural: issue numbers the body *declares* but does not own -------
    # Only a child-declaring section makes an issue reference an ownership
    # claim. Everywhere else, naming an issue is normal: measured across the 40
    # real epics, `#id` appears in Reference Context (22 epics), Requirements
    # (21), and Scope (14), against Child Work Items (4). Scanning the whole
    # body would read "explicitly separate #2936" as a child and auto-file a
    # deliberately-detached work item into this epic -- a judgement dressed as
    # a computation, which is exactly what the structural tier must not do.
    declared_text = "\n".join(
        sections.get(heading, "") for heading in CHILD_DECLARING_SECTIONS
    )
    named = {n for n in re.findall(r"#(\d{2,6})\b", declared_text) if n != iid}
    for number in sorted(named - owned, key=int):
        findings.append(
            Finding(
                tier="structural",
                kind="named-but-unowned",
                detail=f"the epic declares #{number} as a child, but it does not carry "
                f"`{CHILD_LABEL_PREFIX}{iid}`",
                command=f"gh issue edit {number} --repo {repo} "
                f"--add-label {CHILD_LABEL_PREFIX}{iid}",
                subjects=[number],
            )
        )

    # -- structural: a child that is not an executable work-item type --------
    for child in children:
        types = [lbl for lbl in child["labels"] if lbl.startswith("type:")]
        if types and types[0] not in ("type:change", "type:epic"):
            findings.append(
                Finding(
                    tier="structural",
                    kind="non-executable-child",
                    detail=f"#{child['number']} is {types[0]}; only change (or a nested epic) "
                    "enters executable backlog work",
                    command=None,
                    subjects=[str(child["number"])],
                )
            )

    # -- semantic: requirement coverage --------------------------------------
    requirements = re.findall(r"^\s*-\s*(R\d+):", sections.get("Requirements", ""), re.M)
    open_children = [c for c in children if c["state"] != "CLOSED"]
    if requirements and not children:
        findings.append(
            Finding(
                tier="semantic",
                kind="no-children",
                detail=f"the epic declares {len(requirements)} requirement(s) "
                f"({', '.join(requirements)}) and owns no child work-item at all",
                subjects=requirements,
            )
        )
    elif requirements and len(children) < len(requirements):
        findings.append(
            Finding(
                tier="semantic",
                kind="possible-coverage-gap",
                detail=f"{len(requirements)} requirement(s) ({', '.join(requirements)}) but "
                f"{len(children)} child work-item(s); which requirements have no child is a "
                "judgement, not a computation",
                subjects=requirements,
            )
        )

    # -- semantic: near-duplicate children -----------------------------------
    seen: dict[str, list[str]] = {}
    for child in children:
        key = re.sub(r"[^a-z0-9 ]", "", child["title"].lower())
        key = " ".join(sorted(key.split()[:6]))
        seen.setdefault(key, []).append(str(child["number"]))
    for numbers in seen.values():
        if len(numbers) > 1:
            findings.append(
                Finding(
                    tier="semantic",
                    kind="possible-duplicate",
                    detail=f"#{' and #'.join(numbers)} share a title stem; whether they are the "
                    "same promise is a judgement",
                    subjects=numbers,
                )
            )

    # -- terminal-state readout ----------------------------------------------
    if open_children:
        findings.append(
            Finding(
                tier="structural",
                kind="not-terminal",
                detail=f"{len(open_children)} owned child work-item(s) are still open, so "
                f"`epic.py close {iid}` will refuse",
                command=None,
                subjects=[str(c["number"]) for c in open_children],
            )
        )
    return findings


# --------------------------------------------------------------------------
# Epic-only verbs
#
# The other seven are the engine's. These four read or write the child set,
# which is a relation only a type with children has.
# --------------------------------------------------------------------------


def cmd_show(args) -> int:
    issue = fetch_issue(args.iid, args.repo)
    require_epic(issue, "show")
    children = fetch_children(args.iid, args.repo)
    payload = {
        "number": issue["number"],
        "title": issue["title"],
        "state": issue["state"],
        "labels": issue["labels"],
        "url": issue["url"],
        "body": issue.get("body") or "",
        "children": [
            {"number": c["number"], "title": c["title"], "state": c["state"]} for c in children
        ],
    }
    if args.json:
        print(json.dumps(payload, indent=2))
    else:
        print(f"#{issue['number']} [{issue['state']}] {issue['title']}")
        print(f"  labels: {', '.join(issue['labels'])}")
        print(f"  children: {len(children)}")
        print()
        print(payload["body"])
    return 0


def cmd_children(args) -> int:
    children = fetch_children(args.iid, args.repo)
    if args.json:
        print(json.dumps(children, indent=2))
    else:
        print(f"epic #{args.iid}: {len(children)} owned child work-item(s)")
        for child in sorted(children, key=lambda c: c["number"]):
            types = [lbl for lbl in child["labels"] if lbl.startswith("type:")]
            marker = " " if child["state"] == "CLOSED" else "*"
            print(f" {marker} #{child['number']} {child['state']:6} "
                  f"{types[0] if types else '<untyped>':13} {child['title'][:56]}")
        if any(c["state"] != "CLOSED" for c in children):
            print("\n* = still open; the epic is not terminal while any of these are open")
    return 0


def cmd_reconcile(args) -> int:
    issue = fetch_issue(args.iid, args.repo)
    require_epic(issue, "reconcile")
    children = fetch_children(args.iid, args.repo)
    findings = reconcile_findings(issue, children, args.repo)

    if args.json:
        print(json.dumps(
            {
                "epic": issue["number"],
                "children": len(children),
                "structural": [f.as_dict() for f in findings if f.tier == "structural"],
                "semantic": [f.as_dict() for f in findings if f.tier == "semantic"],
            },
            indent=2,
        ))
        return 0

    print(f"epic #{issue['number']}: {issue['title']}")
    print(f"  {len(children)} owned child work-item(s)")
    for tier in ("structural", "semantic"):
        rows = [f for f in findings if f.tier == tier]
        print(f"\n{tier.upper()} ({len(rows)})")
        for finding in rows:
            print(f"  [{finding.kind}] {finding.detail}")
            if finding.command:
                print(f"      repair: {finding.command}")
    return 0


def cmd_close(args) -> int:
    issue = fetch_issue(args.iid, args.repo)
    require_epic(issue, "close")
    children = fetch_children(args.iid, args.repo)
    blocking = sorted(
        (str(c["number"]) for c in children if c["state"] != "CLOSED"), key=int
    )
    if blocking:
        message = (
            f"epic #{issue['number']} is not terminal: {len(blocking)} owned child "
            f"work-item(s) are not closed: {', '.join('#' + n for n in blocking)}; close each "
            f"one, then rerun `epic.py close {issue['number']}`"
        )
        if args.json:
            print(json.dumps({"closed": False, "blocking": blocking, "error": message}, indent=2))
        else:
            print(message, file=sys.stderr)
        return 1

    argv = ["issue", "close", str(args.iid), "--repo", args.repo]
    if args.reason:
        argv += ["--comment", args.reason]
    run_or_show(argv, args.dry_run)
    if not args.dry_run:
        print(f"closed #{args.iid} ({len(children)} child work-item(s), all terminal)")
    return 0


# --------------------------------------------------------------------------
# CLI
# --------------------------------------------------------------------------

# The verbs that never reach the tracker, so a missing issue-platform config
# must not stop them: `skeleton` and `bodydir` are pure local output, `adopt`
# is a local rename, and `validate` has a file mode.
LOCAL_VERBS = ("skeleton", "bodydir", "adopt", "validate")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="epic.py",
        description="Epic work-item surface over the `gh` CLI (Python prototype).",
    )
    parser.add_argument("--repo", help="owner/name; defaults to aw.toml's issue platform")
    sub = parser.add_subparsers(dest="verb", required=True)

    p = sub.add_parser("skeleton", help="emit the empty section template for an epic body")
    p.set_defaults(func=workitem.cmd_skeleton)

    p = sub.add_parser("bodydir", help="print (and create) the directory bodies are staged in")
    p.add_argument("--type", default="epic", choices=WORK_ITEM_TYPES)
    p.set_defaults(func=workitem.cmd_bodydir)

    p = sub.add_parser("fetch", help="stage the tracker's current body, overwriting the local copy")
    p.add_argument("iid")
    p.set_defaults(func=workitem.cmd_fetch)

    p = sub.add_parser("adopt", help="rename a staged body to <iid>.md")
    p.add_argument("path")
    p.add_argument("iid")
    p.set_defaults(func=workitem.cmd_adopt)

    p = sub.add_parser("validate", help="check a body against the epic section schema")
    p.add_argument("iid", nargs="?", help="issue number to validate")
    p.add_argument("--body-file", help="validate this file instead of a live issue")
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=workitem.cmd_validate)

    p = sub.add_parser("show", help="one epic: body, labels, state, child count")
    p.add_argument("iid")
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=cmd_show)

    p = sub.add_parser("children", help="the epic's owned child work-items")
    p.add_argument("iid")
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=cmd_children)

    p = sub.add_parser("reconcile", help="read-only structural and semantic findings")
    p.add_argument("iid")
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=cmd_reconcile)

    p = sub.add_parser("create", help="open a new epic")
    p.add_argument("--title", required=True)
    p.add_argument("--body-file", required=True)
    p.add_argument("--priority", default="p2", choices=PRIORITIES)
    p.add_argument("--project", help="bare project name or a qualified label")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=workitem.cmd_create)

    p = sub.add_parser("update", help="edit an existing epic")
    p.add_argument("iid")
    p.add_argument("--body-file")
    p.add_argument("--title")
    p.add_argument("--add-label", action="append")
    p.add_argument("--remove-label", action="append")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=workitem.cmd_update)

    p = sub.add_parser("close", help="close an epic once every owned child is terminal")
    p.add_argument("iid")
    p.add_argument("--reason", help="closing comment")
    p.add_argument("--dry-run", action="store_true")
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=cmd_close)

    return parser


def main(argv: list[str] | None = None) -> int:
    return workitem.dispatch(build_parser().parse_args(argv), EPIC, LOCAL_VERBS)


if __name__ == "__main__":
    raise SystemExit(main())
