#!/usr/bin/env python3
"""Project a GHAN work item into the round it already describes.

A `type=change` work item authored as Goal / How / Acceptance / Never already
carries the facts a round needs: the change points *are* the write allowlist,
the acceptance table *is* the measurement table, and the verified premises *are*
the reference list and the quote under `## Current behavior`. Retyping them into
the round documents is not authoring, it is transcription -- and transcription
is where the write allowlist quietly stops matching the change points it was
copied from.

So the work item is the source and this is the projection. What cannot be
derived is left as the `<!-- fill -->` form `scaffold` would have handed out,
because `lint` reports an unfilled slot as a finding: a slot this script cannot
fill is one the round cannot dispatch without an author touching it.

Four slots have no source and stay forms:

  `## Fabrication tells`   what a faked report looks like. Dispatch scaffolding
                           against one worker, not a durable fact about the
                           change; putting it in the work item would make the
                           tracker carry the prompt.
  `## Required change`     `## Acceptance`'s target column is what the *gate*
                           reports, so projecting it here would put the
                           oracle's measurements in the injection. `## Goal` is
                           reproduced into the form as the raw material.
  `## Shape to follow`     `### Frozen decisions` is a superset, but this slot
                           wants one named existing symbol within four lines,
                           and picking which decision is that symbol is a
                           judgement.
  `## Definition of done`  the gate projects; where its check lands does not.

Everything else is derived, and a section this script cannot parse is a refusal
rather than a half-projected round.
"""
from __future__ import annotations

import argparse
import json
import re
import shlex
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import agy_dispatch  # noqa: E402

# The same grammar `.claude/aw/scripts/change.py` validates -- it used to be
# `apps/agentic-workflow/src/issues/ghan.rs`, which is deleted, and the plugin
# is now the only copy. Duplicated rather than imported because that script is
# a facade with its own repo resolution and this is controller tooling; the
# coupling is the section names, and a rename there turns into a refusal here
# rather than a wrong projection.
GOAL = "## Goal"
HOW = "## How"
ACCEPTANCE = "## Acceptance"
NEVER = "## Never"
PREMISES = "### Verified premises"
CHANGE_POINTS = "### Change points"
FROZEN = "### Frozen decisions"
NEGATIVE_CONTROL = "### Negative control"
MUST_NOT_TOUCH = "### Must not touch"
MUST_NOT_DO = "### Must not do"

LIST_ITEM = re.compile(r"^[ \t]*(?:[-*+]|\d+[.)])[ \t]+(.*)$")
FENCE = re.compile(r"^```")
# `path/to/file.rs:1234` or `path/to/file.rs:12-40`, in backticks or bare.
FILE_LINE = re.compile(
    r"`?([A-Za-z0-9_./-]+\.[A-Za-z0-9_]+):(\d+)(?:-(\d+))?`?"
)
PATH_TOKEN = re.compile(r"`([A-Za-z0-9_./-]+\.[A-Za-z0-9_]+)`")
BACKTICKED = re.compile(r"`([^`]+)`")


class Unprojectable(SystemExit):
    """A refusal naming the section that stopped the projection.

    Every one of these is a work item that would have produced a round document
    somebody then had to correct by hand -- which is the transcription this
    script exists to remove.
    """


def sections(body: str, level: int) -> dict[str, str]:
    """Headings of exactly `level` mapped to their content, fence-aware.

    Fence-aware because `## Acceptance` carries backticked commands and a
    round's premises carry quoted code, and a `###` inside a fence is a comment
    in someone's shell, not a sub-section.
    """
    marker = "#" * level + " "
    found: dict[str, str] = {}
    name: str | None = None
    buf: list[str] = []
    fence: str | None = None
    for line in body.splitlines():
        if fence is not None:
            if line.startswith(fence):
                fence = None
            buf.append(line)
            continue
        if FENCE.match(line):
            fence = line[: len(line) - len(line.lstrip("`"))]
            buf.append(line)
            continue
        stripped = line.rstrip()
        if stripped.startswith("#") and not stripped.startswith("#" * (level + 1) + " "):
            if stripped.startswith(marker):
                if name is not None:
                    found[name] = "\n".join(buf)
                name = stripped
                buf = []
                continue
            if name is not None and not stripped.startswith("#" * level + "#"):
                # A shallower heading closes the section it interrupts.
                found[name] = "\n".join(buf)
                name = None
                buf = []
                continue
        if name is not None:
            buf.append(line)
    if name is not None:
        found[name] = "\n".join(buf)
    return found


def list_items(content: str) -> list[str]:
    """List items with their continuation lines folded in.

    Folded because a premise wrapping at column 79 carries its `file:line` on
    whichever line it landed on, and an item read as two loses whichever half
    the coordinate is not in.
    """
    items: list[str] = []
    for line in content.splitlines():
        hit = LIST_ITEM.match(line)
        if hit:
            items.append(hit.group(1).strip())
        elif items and line.strip() and line[:1].isspace():
            items[-1] = f"{items[-1]} {line.strip()}"
        elif not line.strip():
            continue
        else:
            # An unindented prose line ends the list; anything after it belongs
            # to the section, not to the last item.
            pass
    return [item for item in items if item]


def table_rows(content: str) -> list[list[str]]:
    """Pipe-table body rows, header and separator dropped."""
    rows: list[list[str]] = []
    for line in content.splitlines():
        stripped = line.strip()
        if not stripped.startswith("|"):
            continue
        cells = [cell.strip() for cell in stripped.strip("|").split("|")]
        if all(set(cell) <= set("-: ") and cell for cell in cells):
            continue
        rows.append(cells)
    return rows[1:] if rows else []


def one_sentence(content: str) -> str:
    """`## Goal` as a single flowed paragraph."""
    text = " ".join(
        line.strip() for line in content.splitlines() if line.strip()
    )
    if not text:
        raise Unprojectable("`## Goal` is empty; there is no claim to project")
    return text


def imperative(goal: str) -> str:
    """`## Task` from `## Goal`, unchanged.

    Not rewritten into the imperative: a rewrite is the one place a projection
    could silently say something the work item does not, and the injection's
    rule is that `## Task` reads as the whole job -- which is exactly what an
    observable-difference sentence already is.
    """
    return goal


def quote_premise(root: Path, item: str) -> tuple[str, str] | None:
    """The premise's coordinate and the line the checkout actually carries.

    Read rather than transcribed. `lint` checks that every quoted line still
    exists in the worker's checkout, so a hand-copied quote is a finding waiting
    to happen the moment the base moves; a quote read at projection time is
    wrong only if the coordinate is.
    """
    hit = FILE_LINE.search(item)
    if not hit:
        return None
    rel, first, last = hit.group(1), int(hit.group(2)), hit.group(3)
    path = root / rel
    if not path.is_file():
        return None
    lines = path.read_text(errors="replace").splitlines()
    end = int(last) if last else first
    if first < 1 or end > len(lines) or end < first:
        return None
    return f"{rel}:{hit.group(2)}" + (f"-{last}" if last else ""), "\n".join(
        lines[first - 1 : end]
    )


def project(body: str, root: Path) -> dict:
    """Every derived field, or a refusal naming the section that blocked it."""
    top = sections(body, 2)
    for required in (GOAL, HOW, ACCEPTANCE, NEVER):
        if required not in top:
            raise Unprojectable(
                f"work item has no `{required}` section: this projection reads "
                "a Goal/How/Acceptance/Never body, and a legacy body has no "
                "change points to derive a write allowlist from"
            )

    goal = one_sentence(top[GOAL])

    how = sections(top[HOW], 3)
    for required in (PREMISES, CHANGE_POINTS, FROZEN):
        if required not in how:
            raise Unprojectable(f"`## How` has no `{required}` sub-section")

    premises = list_items(how[PREMISES])
    if not premises:
        raise Unprojectable(f"`{PREMISES}` is empty; nothing grounds this round")

    change_points = list_items(how[CHANGE_POINTS])
    writes: list[str] = []
    for item in change_points:
        hit = FILE_LINE.search(item) or PATH_TOKEN.search(item)
        if not hit:
            raise Unprojectable(
                f"change point names no path, so it cannot become a write "
                f"allowlist entry: {item[:70]!r}"
            )
        path = hit.group(1)
        if path not in writes:
            writes.append(path)

    rows = table_rows(top[ACCEPTANCE])
    if not rows:
        raise Unprojectable(
            "`## Acceptance` has no gate table, so this round has no "
            "measurements and no gate"
        )
    gates: list[str] = []
    measurements: list[tuple[str, str, str]] = []
    for row in rows:
        if len(row) < 5:
            raise Unprojectable(
                f"gate row has {len(row)} columns, not the 5 the projection "
                f"reads (#, command, current, target, why): {row!r}"
            )
        command = BACKTICKED.search(row[1])
        if not command:
            raise Unprojectable(
                f"gate command is not backticked, so it cannot be taken "
                f"verbatim: {row[1][:70]!r}"
            )
        verbatim = command.group(1).strip()
        if verbatim not in gates:
            gates.append(verbatim)
        measurements.append((verbatim, row[3].strip(), row[4].strip()))

    control = sections(top[ACCEPTANCE], 3).get(NEGATIVE_CONTROL)
    if control is None:
        raise Unprojectable(
            f"`## Acceptance` has no `{NEGATIVE_CONTROL}`; a gate nobody has "
            "seen fail projects into a measurement table with no control row"
        )

    never = sections(top[NEVER], 3)
    for required in (MUST_NOT_TOUCH, MUST_NOT_DO):
        if required not in never:
            raise Unprojectable(f"`## Never` has no `{required}` list")
    must_not_touch = list_items(never[MUST_NOT_TOUCH])
    out_of_scope = must_not_touch + list_items(never[MUST_NOT_DO])
    if not out_of_scope:
        raise Unprojectable(
            f"`{MUST_NOT_TOUCH}` and `{MUST_NOT_DO}` are both empty; a limit "
            "nobody can name projects into an empty `## Out of scope`"
        )

    quotes = [q for q in (quote_premise(root, item) for item in premises) if q]
    # One row per file, as the reference contract asks -- but every premise
    # about that file, joined. Keeping only the first would drop the reason the
    # second premise exists, and "relevant context" is exactly what a reason cell
    # must not degenerate into.
    grouped: dict[str, list[str]] = {}
    for item in premises:
        hit = FILE_LINE.search(item) or PATH_TOKEN.search(item)
        if hit:
            grouped.setdefault(hit.group(1), []).append(item)
    references = [(path, " ".join(items)) for path, items in grouped.items()]

    # A bounded-write round needs at least one frozen thing the worker must read,
    # and `make_profile.py` refuses without one. References outside the write set
    # are the first source -- but a GHAN body usually grounds its premises in the
    # files it is about to change, so that set is routinely empty. `### Must not
    # touch` is the second: frozen and readable is exactly what the author meant
    # by naming it there. Only the entries that are files in this checkout, since
    # a design input has to be readable to be frozen.
    # `PATH_TOKEN` matches any dotted identifier in backticks, so a premise
    # naming a symbol rather than a file -- `importlib.util.spec_from_file_location`
    # is the one that surfaced this -- projects as a design input that
    # `make_profile.py` then refuses, stopping the projection on a premise that
    # was correctly written. Readability is the same test the `### Must not
    # touch` branch below already applies, for the same reason: a design input
    # has to be readable to be frozen.
    design_inputs = [
        path
        for path, _ in references
        if path not in writes and (root / path).is_file()
    ]
    for item in must_not_touch:
        hit = FILE_LINE.search(item) or PATH_TOKEN.search(item)
        if hit and hit.group(1) not in design_inputs:
            if (root / hit.group(1)).is_file():
                design_inputs.append(hit.group(1))

    return {
        "goal": goal,
        "premises": premises,
        "quotes": quotes,
        "references": references,
        "design_inputs": design_inputs,
        "writes": writes,
        "frozen": how[FROZEN].strip(),
        "gates": gates,
        "measurements": measurements,
        "negative_control": control.strip(),
        "out_of_scope": out_of_scope,
    }


ORACLE = """\
## Claim

{claim}

## Measurements

{measurements}

## Gate

```
{gate}
```

## Scope

| Path | Line budget |
|---|---|
{scope}

## Fabrication tells

<!-- fill: what a passing report would look like if the worker faked it. Not
     the worker lying -- the shapes you would otherwise accept. A gate green
     because its rows are unreachable. An assertion on a value the check itself
     just wrote. A name borrowed from a vocabulary the code under test never
     reads. One list item each.

     No source in the work item: this is scaffolding against one worker, not a
     durable fact about the change. -->
-
"""

INJECTION = """\
## Task

{task}

## Current behavior

{current}

## Required change

<!-- fill: what becomes true afterwards, as conditions someone outside the
     change could check. No code and no numbered steps: the worker is being
     paid to derive the implementation, so writing it here buys nothing and
     costs twice. A condition you can only state by naming the lines that
     satisfy it is a measurement -- it belongs in the oracle.

     `## Acceptance` is not the source: its target column is what the *gate*
     reports, and projecting it here would put the oracle's measurements in the
     injection. `## Goal` is reproduced below as the raw material; state the
     conditions its target half implies.

{goal} -->
-

## Shape to follow

<!-- fill: at most {shape_budget} lines. Name the convention already in the
     tree that this change must match -- an existing function, module, type, or
     error shape, in backticks -- and say to follow it rather than invent a
     second one.

     `### Frozen decisions` from the work item is reproduced below; it is a
     superset, and choosing which of its decisions is the convention to name is
     the judgement this slot asks for. Delete what is not that.

{frozen} -->

## Reference

| path | why the worker must read it |
|---|---|
{reference}

## Out of scope

{out_of_scope}

## Definition of done

<!-- fill: name in backticks where the gate's check lands -- the module, file,
     or suite it joins. The gate below is projected from `## Acceptance`; where
     its check lands is not in the work item. -->

```
{gate}
```
"""


def render_oracle(fields: dict) -> str:
    header = "| # | input | expected observation | why it cannot hold by accident |"
    divider = "|---|---|---|---|"
    rows = [
        f"| {n} | `{command}` | {expected} | {why} |"
        for n, (command, expected, why) in enumerate(fields["measurements"], 1)
    ]
    # The negative control travels as its own row rather than as prose, because
    # the oracle contract reads a control named only in a rationale cell as a
    # sentence about a control: the table would lint green while measuring
    # nothing.
    control = " ".join(
        line.strip()
        for line in fields["negative_control"].splitlines()
        if line.strip()
    )
    rows.append(
        f"| {len(rows) + 1} | the mutation below (negative control) | "
        f"must FAIL | {control} |"
    )
    writes = fields.get("writes") or []
    scope = "\n".join(
        f"| `{path}` | none |" for path in writes
    ) or "| `<!-- fill: path -->` | <!-- fill: line budget --> |"
    return ORACLE.format(
        claim=fields["goal"],
        measurements="\n".join([header, divider, *rows]),
        gate=fields["gates"][0],
        scope=scope,
    )


def indented(text: str) -> str:
    """Work-item prose reproduced inside a `<!-- fill -->` comment.

    Indented to the comment's own continuation column so the raw material reads
    as part of the instruction rather than as the answer; an author who deletes
    the comment deletes the material with it, which is the point.
    """
    return "\n".join(
        f"     {line}" if line.strip() else "" for line in text.splitlines()
    )


def render_injection(fields: dict) -> str:
    if fields["quotes"]:
        current = "\n\n".join(
            f"`{coord}`:\n\n```\n{text}\n```" for coord, text in fields["quotes"]
        )
    else:
        current = (
            "<!-- fill: no premise carried a `file:line` this checkout could\n"
            "     read, so there is nothing to quote. Either the coordinates\n"
            "     are stale or the premises are about behavior rather than\n"
            "     source; quote what the change must displace. -->\n\n```\n```"
        )
    return INJECTION.format(
        task=imperative(fields["goal"]),
        current=current,
        goal=indented(fields["goal"]),
        shape_budget=agy_dispatch.SHAPE_LINE_BUDGET,
        frozen=indented(fields["frozen"]),
        reference="\n".join(
            f"| `{path}` | {why} |" for path, why in fields["references"]
        ),
        out_of_scope="\n".join(f"- {item}" for item in fields["out_of_scope"]),
        gate=fields["gates"][0],
    )


def read_work_item(issue: str, repo: str | None) -> str:
    argv = ["gh", "issue", "view", issue, "--json", "body", "--jq", ".body"]
    if repo:
        argv[3:3] = ["--repo", repo]
    out = subprocess.run(argv, capture_output=True, text=True)
    if out.returncode != 0:
        raise SystemExit(f"could not read work item {issue}: {out.stderr.strip()}")
    return out.stdout


def profile_argv(fields: dict, issue: str, scope: list[str]) -> list[str]:
    """The `make_profile.py` invocation this work item implies.

    Emitted rather than reimplemented: the frozen complement, the permission
    set, and the state-dir layout are `make_profile.py`'s to compute, and a
    second copy of that logic here is the drift this script exists to stop.
    """
    argv = [
        sys.executable,
        str(Path(__file__).resolve().parent / "make_profile.py"),
        "--issue",
        issue,
        "--gate",
        fields["gates"][0],
    ]
    for path in scope:
        argv += ["--scope", path]
    for path in fields["writes"]:
        argv += ["--write", path]
    for path in fields["design_inputs"]:
        argv += ["--design-input", path]
    return argv


def derived_scope(writes: list[str]) -> list[str]:
    """One scope per distinct top two path segments of the write set.

    `make_profile.py` freezes the complement of the write set within each scope,
    so a scope that does not contain a write point leaves that point unfrozen
    and a scope wider than the work is a bigger frozen set to no purpose.
    """
    scopes: list[str] = []
    for path in writes:
        parts = Path(path).parts
        scope = str(Path(*parts[:2])) if len(parts) > 2 else str(Path(path).parent)
        if scope and scope not in scopes:
            scopes.append(scope)
    return scopes or ["."]


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Project a GHAN work item into a round's profile and documents."
    )
    ap.add_argument("issue", help="work-item number; also the round's task key")
    ap.add_argument("--repo", help="owner/name; defaults to gh's own default")
    ap.add_argument(
        "--root",
        help="checkout the premises' `file:line` coordinates are read from; "
        "defaults to the repository containing the current directory. Point it "
        "at the worker's worktree once `worktree` has derived one, so the "
        "quotes are read from the tree `lint` will check them against",
    )
    ap.add_argument(
        "--scope",
        action="append",
        default=[],
        help="repo-relative directory to freeze; repeatable. Derived from the "
        "change points when omitted",
    )
    ap.add_argument(
        "--design-input",
        action="append",
        default=[],
        dest="design_input",
        help="repo-relative file to freeze and hand the worker to read; "
        "repeatable. Added to the ones derived from the references and from "
        "`### Must not touch`",
    )
    ap.add_argument(
        "--body-file",
        help="read the work item from a file instead of the tracker",
    )
    ap.add_argument(
        "--print-only",
        action="store_true",
        help="show the `make_profile.py` invocation and the two documents "
        "without writing anything",
    )
    args = ap.parse_args()

    root = Path(args.root).resolve() if args.root else Path.cwd()
    found = subprocess.run(
        [*agy_dispatch.GIT, "-C", str(root), "rev-parse", "--show-toplevel"],
        capture_output=True,
        text=True,
    )
    if found.returncode == 0:
        root = Path(found.stdout.strip())

    body = (
        Path(args.body_file).read_text()
        if args.body_file
        else read_work_item(args.issue, args.repo)
    )
    fields = project(body, root)
    for path in args.design_input:
        if path not in fields["design_inputs"]:
            fields["design_inputs"].append(path)
    if not fields["design_inputs"]:
        raise SystemExit(
            "no design input: every premise names a file this round also "
            "writes, and `### Must not touch` names no file in this checkout. "
            "A bounded-write round needs at least one frozen thing the worker "
            "must read, so pass `--design-input PATH` -- or name the artifact "
            "in the work item, where it belongs"
        )
    scope = args.scope or derived_scope(fields["writes"])
    argv = profile_argv(fields, args.issue, scope)

    print("derived from the work item:")
    print(f"  write allowlist : {', '.join(fields['writes'])}")
    print(f"  gate            : {fields['gates'][0]}")
    print(f"  scope           : {', '.join(scope)}")
    print(f"  design inputs   : {', '.join(fields['design_inputs'])}")
    print(f"  quoted premises : {len(fields['quotes'])} of {len(fields['premises'])}")
    if len(fields["gates"]) > 1:
        print(
            f"  note: `## Acceptance` names {len(fields['gates'])} distinct "
            "commands; the round is judged by the first, and the rest project "
            "into measurement rows only"
        )
    print()
    print("profile:")
    # Quoted, because this line exists to be pasted. `--gate cargo test -p x
    # --lib y` unquoted arrives at `make_profile.py` as five arguments, and the
    # gate it stores is then `cargo` -- which runs, and fails, and reads as the
    # round's own gate failing.
    print("  " + " ".join(shlex.quote(arg) for arg in argv))

    if args.print_only:
        print()
        print(render_oracle(fields))
        print(render_injection(fields))
        return 0

    made = subprocess.run(argv, capture_output=True, text=True)
    sys.stdout.write(made.stdout)
    if made.returncode != 0:
        sys.stderr.write(made.stderr)
        raise SystemExit(f"make_profile.py exited {made.returncode}")
    profile_path = next(
        (
            line.split()[-1]
            for line in made.stdout.splitlines()
            if line.strip().endswith(".profile.json")
        ),
        None,
    )
    if profile_path is None:
        raise SystemExit(
            "make_profile.py printed no profile path; nothing to write the "
            "round documents beside"
        )

    profile = json.loads(Path(profile_path).read_text())
    for path, text in (
        (agy_dispatch.oracle_path(profile, args.issue), render_oracle(fields)),
        (agy_dispatch.injection_path(profile, args.issue), render_injection(fields)),
    ):
        if path.exists():
            # The same rule `scaffold` keeps: a projection that overwrote an
            # authored document would spend the three slots it cannot fill.
            print(f"kept   {path} (already authored; projection never overwrites)")
            continue
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        print(f"wrote  {path}")
    print(
        "\nfour slots have no source and are still forms: `## Fabrication "
        "tells`, `## Required change`, `## Shape to follow`, `## Definition of "
        "done`. Fill them, then `lint`."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
