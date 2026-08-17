from __future__ import annotations

import json
from pathlib import Path
import re
import sys

from dispatch.core.cli import EXIT_FINDINGS
from dispatch.core.identity import validate_task_key
from dispatch.core.rules import task_allowlist_admits
from dispatch.core.scope import parse_line_ranges



def extract_exec_report(raw: str) -> str | None:
    stripped = raw.lstrip()
    markers = list(re.finditer(r"^## EXEC REPORT", stripped, flags=re.MULTILINE))
    if not markers:
        return None
    return stripped[markers[-1].start() :].lstrip()


ORACLE_SECTIONS = ("Claim", "Measurements", "Gate", "Scope", "Fabrication tells")
INJECTION_SECTIONS = (
    "Task",
    "Current behavior",
    "Required change",
    "Shape to follow",
    "Reference",
    "Out of scope",
    "Definition of done",
)
# Only the section quoting what already exists, and the one naming the gate, may
# carry a fenced block. Anywhere else a fence means the controller pasted the
# answer, and a round whose answer is already written has nothing left to
# dispatch.
FENCE_BEARING_SECTIONS = ("Current behavior", "Definition of done")
NUMBERED_STEP = re.compile(r"^[ \t]*(\d+)[.)][ \t]+\S")
# Enough to name a convention and say to follow it; not enough to describe one.
SHAPE_LINE_BUDGET = 4


def reads_as_numbered_steps(body: str) -> bool:
    """Two or more numbered lines, opening at one, with no prose between them.

    One is not a recipe, and prose wrapped at column 79 produces exactly one
    whenever a sentence breaks before a number ending a clause -- `... at lines
    175, 1564, 4023, and\\n2860. Five sites` is a paragraph, not a list. So the
    run is what is counted, and an unindented line that is not itself numbered
    ends it. Blank and indented lines carry the run instead, which keeps a loose
    list and a wrapped step from reading as two lists of one; prose introducing
    a list is still caught, because resetting to zero leaves the list that
    follows free to reach two on its own.

    A run opens only on `1`, because two *consecutively* wrapped numbers are the
    one paragraph the run alone cannot tell from a list, and an ordered recipe
    is numbered from one. The cost is a recipe that starts at some other number,
    which is not how one gets written.
    """
    run = 0
    for line in body.splitlines():
        step = NUMBERED_STEP.match(line)
        if step:
            if run == 0 and step.group(1) != "1":
                continue
            run += 1
            if run >= 2:
                return True
        elif line.strip() and not line[:1].isspace():
            run = 0
    return False


ORACLE_HEADING = re.compile(r"^##[ \t]+(.+?)[ \t]*$", re.MULTILINE)
ORACLE_FENCE = re.compile(r"^```[^\n]*\n(.*?)^```", re.MULTILINE | re.DOTALL)
INFO_FENCE = re.compile(r"^```([^\n]*)\n(.*?)^```", re.MULTILINE | re.DOTALL)
# Current behavior is sometimes a thing the binary *does*, not a thing a file
# says, and the two are grounded differently: a source quote is checked against
# the checkout, a transcript against the command that produced it.
TRANSCRIPT_INFO = re.compile(r"^console$", re.IGNORECASE)
TRANSCRIPT_COMMAND = re.compile(r"^\$[ \t]+\S")
NEGATIVE_CONTROL = re.compile(r"negative control", re.IGNORECASE)
LIST_ITEM = re.compile(r"^[ \t]*(?:[-*+]|\d+[.)])[ \t]+\S", re.MULTILINE)
TABLE_ROW = re.compile(r"^[ \t]*\|.*\|[ \t]*$", re.MULTILINE)
TABLE_DIVIDER = re.compile(r"^[ \t]*\|[\s:|-]+\|[ \t]*$")
FILL_MARKER = re.compile(r"<!--[ \t]*fill\b.*?-->", re.DOTALL)
BACKTICKED = re.compile(r"`([^`\n]+)`")
LINE_SUFFIX = re.compile(r":\d+(?:[:-]\d+)?$")
# A quoted excerpt may skip lines; the marker for that is not itself a quote.
ELISION = re.compile(r"(?://|#)?[ \t]*(?:\.{3}|…|snip|omitted)[ \t]*", re.IGNORECASE)


def oracle_sections(text: str) -> dict[str, str]:
    """Split an oracle into its `## ` sections, preserving document order."""
    matches = list(ORACLE_HEADING.finditer(text))
    sections: dict[str, str] = {}
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
        sections[match.group(1).strip()] = text[match.end() : end].strip()
    return sections


def missing_or_misordered(text: str, required: tuple[str, ...], label: str) -> list[str]:
    """Which required `## ` sections are absent, and are the rest in order."""
    sections = oracle_sections(text)
    findings = [
        f"{label} is missing the `## {name}` section"
        for name in required
        if name not in sections
    ]
    present = [name for name in sections if name in required]
    if present != [name for name in required if name in sections]:
        findings.append(
            f"{label} sections are out of order; expected "
            + " -> ".join(f"## {name}" for name in required)
        )
    return findings


def gate_commands_in(section: str) -> list[str]:
    """The command lines inside a section's fenced blocks, in order.

    A block that prompts with `$ ` is a transcript: its commands are the
    prompted lines and everything else is output. Reading output as a command
    is how `## Definition of done` came to "name a different gate" than the
    oracle while naming the same one -- once as `$ cargo test ...` against the
    oracle's bare `cargo test ...`, and again with the expected `test result:`
    line counted as a second gate. Showing the controller what green looks like
    is worth keeping, so the prompt is what marks the command.
    """
    commands = []
    for block in ORACLE_FENCE.findall(section):
        lines = [line.strip() for line in block.splitlines() if line.strip()]
        prompted = [line for line in lines if line.startswith("$ ")]
        commands.extend(
            [line[2:].strip() for line in prompted] if prompted else lines
        )
    return commands


def unjudged_gate_commands(profile: dict, gate_commands: list[str]) -> list[str]:
    """Which of an oracle's `## Gate` commands `prove` will never run.

    `prove` runs `task_contract.gate_command` and nothing else -- one command,
    whose red decides the round. An oracle's `## Gate` block is free to list
    several, and `lint` already checks each one is authorized, so a second
    command reads as judged when it is only authorized. The round then carries
    a row whose observation no proof ever makes, which is the false green this
    whole scaffold exists to refuse: the documents say two things decide the
    round and the machinery lets one decide it.

    The way out is to name the compound command in the profile, or to state the
    extra observation as prose the controller checks by hand and keep the fence
    to the one command that is actually judged.
    """
    judged = profile.get("task_contract", {}).get("gate_command")
    if not judged:
        return []
    return [command for command in gate_commands if command != judged]


def unquoted_current_behavior_lines(
    root: str | None, section: str, candidates: list[str]
) -> list[str]:
    """Lines an injection quotes as current behavior that no candidate file has.

    `injection_findings` already refuses a `## Current behavior` with no fenced
    quote, on the reasoning that a round should be grounded in what was read
    rather than what was remembered. That check reaches the form and stops: a
    block pasted from an earlier round, from a base two commits back, or from
    memory satisfies it exactly as well as a block copied out of the file.

    A stale quote is worse than no quote. It is the one part of the injection a
    worker is entitled to treat as ground truth -- it is labelled as the code as
    it stands -- so a worker that greps for it, finds nothing, and improvises has
    been sent to do that by the document. Rounds are re-based often here (a
    follow-up reuses the previous worktree block; documents get drafted while an
    earlier round is still in flight), which is exactly when a quote goes stale
    without anyone editing it.

    Matching is on the stripped line, so re-indenting a quote is not a finding:
    the dangerous class is content that is gone, not content that moved. Lines
    that are only an elision marker are skipped for the same reason.

    A ```console fence is skipped, because it is not claiming to be in a file.
    Current behavior is often what the binary *prints* -- an envelope, a refusal,
    an exit code -- and there is no file to find those lines in. Checking them
    against the source anyway reported every real transcript as stale, whose only
    available fix was to strip the fence and paraphrase the output; that trades a
    verbatim observation for prose, which is the failure this whole rule exists
    to prevent. `transcript_findings` grounds these instead, by requiring the
    command that produced them.
    """
    if not root:
        return []
    haystacks: list[set[str]] = []
    for rel in candidates:
        path = Path(rel) if Path(rel).is_absolute() else Path(root) / rel
        try:
            haystacks.append({line.strip() for line in path.read_text().splitlines()})
        except (OSError, UnicodeDecodeError):
            continue
    if not haystacks:
        return []
    missing: list[str] = []
    for info, block in INFO_FENCE.findall(section):
        if TRANSCRIPT_INFO.match(info.strip()):
            continue
        for line in block.splitlines():
            bare = line.strip()
            if not bare or ELISION.fullmatch(bare):
                continue
            if any(bare in hay for hay in haystacks):
                continue
            if bare not in missing:
                missing.append(bare)
    return missing


def referenced_paths(text: str) -> list[str]:
    """Backticked tokens that name a repository path.

    A token qualifies only when it carries a separator, which keeps Rust module
    paths, flag names, field accesses, and prose identifiers out of the check.
    An optional `:line` or `:line-line` suffix is dropped so the usual
    `path:line` citation form resolves.
    """
    tokens: list[str] = []
    for token in BACKTICKED.findall(text):
        if " " in token or "/" not in token:
            continue
        if token.startswith(("http://", "https://", "-")) or "*" in token:
            continue
        bare = LINE_SUFFIX.sub("", token)
        if bare and bare not in tokens:
            tokens.append(bare)
    return tokens


def transcript_body(block: str) -> list[str]:
    """A console block's output lines, normalized for comparison.

    Blank lines and trailing spaces survive a copy-paste unevenly, and neither
    carries any of the observation, so neither may decide whether a transcript
    matches the run it claims to be.
    """
    return [line.rstrip() for line in block.splitlines()[1:] if line.strip()]


def transcript_findings(section: str, captures: dict[str, list[str]]) -> list[str]:
    """Console blocks that are not the output of a command anyone ran.

    Exempting a ```console fence from the file comparison removes the only check
    that block had, so it has to gain one of its own. The prompt line was that
    check, and it turned out to be an assertion rather than a measurement:
    nothing compared the lines under it to anything, so a paraphrase typed from
    memory and a capture taken a second ago read identically. Two shipped in one
    round -- one naming a flag the verb does not accept, one whose behavior only
    existed in a build newer than the installed binary -- and lint reported the
    round clean (#3426).

    So the block is now compared against a stored capture. `capture` runs the
    command and writes what it printed; this checks the pasted block against
    that. The honest path stays the cheapest one, because pasting what `capture`
    prints is less work than typing output, and fabricating now means forging a
    record that names its own command, directory, and exit code.
    """
    findings: list[str] = []
    unprompted = False
    uncaptured: list[str] = []
    diverged: list[str] = []
    for info, block in INFO_FENCE.findall(section):
        if not TRANSCRIPT_INFO.match(info.strip()):
            continue
        lines = [line for line in block.splitlines() if line.strip()]
        if not lines or not TRANSCRIPT_COMMAND.match(lines[0].strip()):
            unprompted = True
            continue
        command = lines[0].strip()[2:].strip()
        if command not in captures:
            uncaptured.append(command)
        elif transcript_body(block) != captures[command]:
            diverged.append(command)
    if unprompted:
        findings.append(
            "a ```console block in `## Current behavior` does not open with "
            "the `$ <command>` that produced it: a transcript is exempt from "
            "the file comparison, so the command is the only thing that lets "
            "a reader reproduce it rather than trust it"
        )
    if uncaptured:
        findings.append(
            "`## Current behavior` shows output for command(s) this round never "
            "captured: "
            + ", ".join(f"`{command}`" for command in uncaptured)
            + ". Run `capture <profile> <task_key> <command>` and paste what it "
            "prints; output typed from memory is what the worker will treat as "
            "the behavior as it stands"
        )
    if diverged:
        findings.append(
            "`## Current behavior` shows output that differs from what the "
            "captured run of "
            + ", ".join(f"`{command}`" for command in diverged)
            + " printed. Re-capture and paste the result rather than editing "
            "the block, or the document says one thing and the record another"
        )
    return findings


def document_findings(
    root: str | None,
    text: str,
    label: str,
    declared: set[str] | None = None,
) -> list[str]:
    """Checks that apply to any controller-authored round document.

    Both defects here are of one kind: the document says something the
    controller never actually established. An unfilled slot is a form dispatched
    before it was written; a path that does not resolve is a citation from
    memory rather than from the checkout the worker is about to see.

    A path the round declares writable is exempt: a round may create a file, and
    naming the file it is about to create is the opposite of citing from memory.
    """
    findings: list[str] = []
    if FILL_MARKER.search(text):
        findings.append(
            f"{label} still carries {len(FILL_MARKER.findall(text))} unfilled "
            "`<!-- fill -->` slot(s) from the scaffold"
        )
    if root:
        declared = declared or set()
        missing = [
            token
            for token in referenced_paths(text)
            if token not in declared
            and not (Path(token) if Path(token).is_absolute() else Path(root) / token).exists()
        ]
        if missing:
            findings.append(
                f"{label} cites path(s) that do not exist in the worker's "
                "checkout: " + ", ".join(missing)
            )
    return findings


def marks_a_negative_control(row: str) -> bool:
    """Whether this table row is itself the control, not prose about one.

    A row is a negative control because of what it *feeds* and what that must
    not produce, so the marker belongs in its input or its expected
    observation. It does not belong in a trailing rationale cell, which is
    where a controller naturally writes about a *different* row: "row 7 is the
    negative control for the new row" is a true sentence that leaves the table
    without a control of its own. Matching the row as one string accepted that
    sentence and reported the table conformant.

    A rationale cell is one past `# | input | expected observation`, so it is
    dropped only when the row actually has one. In the three-column table the
    last cell is the observation and stays in scope.
    """
    cells = [cell.strip() for cell in row.strip().strip("|").split("|")]
    identity = cells[:-1] if len(cells) > 3 else cells
    return any(NEGATIVE_CONTROL.search(cell) for cell in identity)


def oracle_findings(profile: dict, text: str) -> list[str]:
    """Structural check on the injected oracle. Never reads for meaning.

    The generic scaffold around the oracle has always had a fixed shape while
    the oracle itself was free prose, and every false green so far entered
    through that prose: a gate nobody cross-checked against the authorized
    commands, a table with no control row, no statement of what fabrication
    would look like. These four sections are expressible for any bounded task,
    so requiring them costs no project knowledge and closes that gap.
    """
    sections = oracle_sections(text)
    findings = missing_or_misordered(text, ORACLE_SECTIONS, "oracle")
    findings.extend(
        document_findings(
            profile.get("root"), text, "oracle", set(profile.get("allowed_repo_writes") or [])
        )
    )

    # A missing section is one defect. Reporting its emptiness and its missing
    # rows as further defects inflates the count and buries the real fix, so
    # each section's content checks run only once the section exists.
    if "Claim" in sections and not sections["Claim"]:
        findings.append("`## Claim` is empty: state one falsifiable sentence")

    if "Measurements" in sections:
        rows = [
            row
            for row in TABLE_ROW.findall(sections["Measurements"])
            if not TABLE_DIVIDER.match(row)
        ]
        data_rows = rows[1:] if rows else []
        if len(data_rows) < 2:
            findings.append(
                f"`## Measurements` needs at least 2 measured rows, found "
                f"{len(data_rows)}"
            )
        elif not any(marks_a_negative_control(row) for row in data_rows):
            findings.append(
                "`## Measurements` has no row marked `negative control`: "
                "without one, an implementation that changes nothing can "
                "satisfy the table. Mark the control in its input or its "
                "expected observation -- naming one in a rationale cell "
                "describes a control, it does not add one"
            )

    if "Gate" in sections:
        gate_commands = gate_commands_in(sections["Gate"])
        allowed = profile["task_commands"].get("allow", [])
        families = profile["task_commands"].get("allow_prefix", [])
        if not gate_commands:
            findings.append(
                "`## Gate` has no fenced command block: put each gate command "
                "on its own line inside one ``` fence"
            )
        else:
            if allowed or families:
                undeclared = [
                    c for c in gate_commands if not task_allowlist_admits(profile, c)
                ]
                if undeclared:
                    findings.append(
                        "`## Gate` names command(s) the worker is not "
                        "authorized to run: " + ", ".join(undeclared)
                    )
            unjudged = unjudged_gate_commands(profile, gate_commands)
            if unjudged:
                findings.append(
                    "`## Gate` names command(s) `prove` will never run, so no "
                    "proof covers what they observe: "
                    + ", ".join(unjudged)
                    + ". `prove` runs `task_contract.gate_command` alone "
                    f"(`{profile.get('task_contract', {}).get('gate_command')}`). "
                    "Name the compound command in the profile, or keep the "
                    "fence to the judged command and state the rest as prose "
                    "the controller checks by hand."
                )

    if "Scope" in sections:
        scope_text = sections["Scope"]
        scope_rows = [
            row
            for row in TABLE_ROW.findall(scope_text)
            if not TABLE_DIVIDER.match(row)
        ]
        if scope_rows and "path" in scope_rows[0].lower():
            data_rows = scope_rows[1:]
        else:
            data_rows = scope_rows

        oracle_scope: dict[str, tuple[int | None, str]] = {}
        scope_error = False
        seen_paths: set[str] = set()
        for row in data_rows:
            cells = [c.strip() for c in row.strip().strip("|").split("|")]
            if len(cells) not in (2, 3):
                findings.append(f"`## Scope` has malformed row: `{row.strip()}`")
                scope_error = True
                continue
            path_cell = cells[0].strip().strip("`").strip()
            budget_cell = cells[1].strip().strip("`").strip()
            range_cell = cells[2].strip().strip("`").strip() if len(cells) == 3 else "any"
            if not path_cell or FILL_MARKER.search(path_cell):
                findings.append(f"`## Scope` has malformed path in row: `{row.strip()}`")
                scope_error = True
                continue
            if path_cell in seen_paths:
                findings.append(f"`## Scope` lists path `{path_cell}` multiple times")
                scope_error = True
                continue
            seen_paths.add(path_cell)

            if budget_cell.lower() == "none":
                budget_val = None
            elif budget_cell.isdigit():
                budget_val = int(budget_cell)
            else:
                findings.append(
                    f"`## Scope` has unreadable line budget `{cells[1].strip()}` for path `{path_cell}`"
                )
                scope_error = True
                continue

            try:
                parse_line_ranges(range_cell)
            except ValueError as err:
                findings.append(
                    f"`## Scope` has unreadable line ranges `{range_cell}` for path `{path_cell}`: {err}"
                )
                scope_error = True
                continue

            oracle_scope[path_cell] = (budget_val, range_cell)

        if not scope_error:
            profile_writes = profile.get("allowed_repo_writes") or []
            profile_budgets = profile.get("path_change_budgets") or {}
            profile_ranges = profile.get("path_line_ranges") or {}
            profile_scope = {
                path: (profile_budgets.get(path), profile_ranges.get(path, "any"))
                for path in profile_writes
            }
            all_paths = sorted(set(oracle_scope.keys()) | set(profile_scope.keys()))
            for path in all_paths:
                in_oracle = path in oracle_scope
                in_profile = path in profile_scope
                if in_oracle and in_profile:
                    o_budget, o_range = oracle_scope[path]
                    p_budget, p_range = profile_scope[path]
                    if o_budget != p_budget or o_range != p_range:
                        o_parts = []
                        p_parts = []
                        if o_budget != p_budget:
                            o_parts.append(str(o_budget) if o_budget is not None else "none")
                            p_parts.append(str(p_budget) if p_budget is not None else "none")
                        if o_range != p_range:
                            o_parts.append(o_range)
                            p_parts.append(p_range)
                        o_str = " ".join(o_parts)
                        p_str = " ".join(p_parts)
                        findings.append(
                            f"`## Scope` write scope mismatch for `{path}`: oracle states {o_str}, profile carries {p_str}"
                        )
                elif in_oracle and not in_profile:
                    o_budget, o_range = oracle_scope[path]
                    o_str = str(o_budget) if o_budget is not None else "none"
                    findings.append(
                        f"`## Scope` write scope mismatch for `{path}`: oracle states {o_str}, profile carries absent"
                    )
                elif not in_oracle and in_profile:
                    p_budget, p_range = profile_scope[path]
                    p_str = str(p_budget) if p_budget is not None else "none"
                    findings.append(
                        f"`## Scope` write scope mismatch for `{path}`: oracle states absent, profile carries {p_str}"
                    )

    if "Fabrication tells" in sections and not LIST_ITEM.search(
        sections["Fabrication tells"]
    ):
        findings.append(
            "`## Fabrication tells` needs at least one list item naming what a "
            "fabricated pass would look like"
        )
    return findings


def injection_findings(
    profile: dict,
    text: str,
    oracle_text: str,
    captures: dict[str, list[str]] | None = None,
) -> list[str]:
    """Structural check on the round-specific injection.

    The oracle says how the round will be judged; this document says what to do
    and what to read, and it is the half a controller is most tempted to write
    from memory. Requiring a verbatim quote of the current behavior means the
    round cannot be dispatched without the controller having opened the file,
    and requiring the same gate as the oracle means the instruction and the
    judgement cannot drift apart the way they did when each was free prose.

    The remaining rules all defend the same thing: a dispatched round is only
    worth its cost if the worker still has the design left to do.  A pasted
    implementation, a numbered recipe, or a `## Shape to follow` that grew into
    a plan all mean the controller already did the work and is paying a second
    time to have it typed out.
    """
    sections = oracle_sections(text)
    findings = missing_or_misordered(text, INJECTION_SECTIONS, "injection")
    findings.extend(
        document_findings(
            profile.get("root"), text, "injection", set(profile.get("allowed_repo_writes") or [])
        )
    )

    for name in ("Task", "Required change"):
        if name in sections and not sections[name]:
            findings.append(f"`## {name}` is empty")

    for name, body in sections.items():
        if name in FENCE_BEARING_SECTIONS or name not in INJECTION_SECTIONS:
            continue
        if ORACLE_FENCE.search(body):
            findings.append(
                f"`## {name}` contains a fenced block: quote existing code only "
                "under `## Current behavior`. State the requirement; handing the "
                "worker the implementation leaves it nothing to design"
            )

    for name in ("Required change", "Shape to follow"):
        if name in sections and reads_as_numbered_steps(sections[name]):
            findings.append(
                f"`## {name}` reads as numbered steps: say what must become "
                "true, not the order to type it in"
            )

    if "Current behavior" in sections:
        # A transcript does not satisfy the quote rule. It is grounded by its
        # command instead of by the checkout, so a section holding only
        # transcripts has never been checked against the tree the worker will
        # open -- which is the one thing this rule was added to force.
        quotes = [
            body
            for info, body in INFO_FENCE.findall(sections["Current behavior"])
            if body.strip() and not TRANSCRIPT_INFO.match(info.strip())
        ]
        findings.extend(
            transcript_findings(sections["Current behavior"], captures or {})
        )
        if not quotes:
            findings.append(
                "`## Current behavior` has no non-empty fenced quote: paste the "
                "code as it stands today so the round is grounded in what was "
                "read rather than what was remembered"
            )
        else:
            candidates = list(profile.get("allowed_repo_writes") or [])
            candidates += [p for p in referenced_paths(text) if p not in candidates]
            stale = unquoted_current_behavior_lines(
                profile.get("root"), sections["Current behavior"], candidates
            )
            if stale:
                shown = ", ".join(f"`{line}`" for line in stale[:3])
                more = f" (+{len(stale) - 3} more)" if len(stale) > 3 else ""
                findings.append(
                    "`## Current behavior` quotes line(s) that appear in none of "
                    f"the round's files: {shown}{more}. The worker is entitled to "
                    "treat that block as the code as it stands; re-read the file "
                    "at this round's base and paste what is actually there"
                )

    if "Shape to follow" in sections:
        body = sections["Shape to follow"]
        lines = [line for line in body.splitlines() if line.strip()]
        if not BACKTICKED.search(body):
            findings.append(
                "`## Shape to follow` names no existing symbol or file: point at "
                "the convention already in the tree that the change must match, "
                "so the worker does not invent a second one"
            )
        if len(lines) > SHAPE_LINE_BUDGET:
            findings.append(
                f"`## Shape to follow` is {len(lines)} lines; keep it within "
                f"{SHAPE_LINE_BUDGET}. Past that it stops being a constraint and "
                "becomes the design the worker was dispatched to produce"
            )

    if "Reference" in sections:
        rows = [
            row
            for row in TABLE_ROW.findall(sections["Reference"])
            if not TABLE_DIVIDER.match(row)
        ]
        if not LIST_ITEM.search(sections["Reference"]) and len(rows[1:]) < 1:
            findings.append(
                "`## Reference` names nothing to read: list each path the "
                "worker must consult and why"
            )

    if "Out of scope" in sections and not LIST_ITEM.search(sections["Out of scope"]):
        findings.append(
            "`## Out of scope` needs at least one list item; the write "
            "allowlist bounds where the worker may write, not what it may "
            "redesign"
        )

    if "Definition of done" in sections:
        declared = gate_commands_in(sections["Definition of done"])
        judged = gate_commands_in(oracle_sections(oracle_text).get("Gate", ""))
        if not declared:
            findings.append(
                "`## Definition of done` has no fenced command block: name the "
                "gate the worker must leave green"
            )
        elif judged and declared != judged:
            findings.append(
                "`## Definition of done` names a different gate than the "
                f"oracle judges by: {declared} vs {judged}"
            )
        prose = ORACLE_FENCE.sub("", sections["Definition of done"])
        if not BACKTICKED.search(prose):
            findings.append(
                "`## Definition of done` names the gate but not where its check "
                "lands: name the module, file, or suite the new check joins, or "
                "the worker guesses and the diff arrives in the wrong place"
            )
    return findings


def injection_path(profile: dict, task_key: str) -> Path:
    """Where this round's injection lives.

    A profile may point `inject_prompt_file` anywhere; when it does not, the
    scaffold has a deterministic home beside the oracle so the two halves of a
    round stay together.
    """
    declared = profile.get("inject_prompt_file")
    if declared:
        return Path(declared)
    return Path(profile["state_dir"]) / "injections" / f"{task_key}.md"


def oracle_path(profile: dict, task_key: str) -> Path:
    return Path(profile["state_dir"]) / "oracles" / f"{task_key}.md"


ORACLE_SKELETON = """\
## Claim

<!-- fill: one falsifiable sentence about behavior observable from outside the
     change; not a description of the edit. Revert the change in your head: a
     claim that stays true either way is not this round's claim. -->

## Measurements

<!-- fill: the rows the gate has to make. At least two, at least one of them
     the negative control, and the control marked in its input or its expected
     observation -- a control named only in the rationale cell is a sentence
     about a control, and the table passes lint while measuring nothing.

     Every row must be a state the product can actually reach. A row resting on
     a value the product never produces is vacuous: it stays green whatever the
     worker writes, and that is how this round most plausibly ends green and
     empty. Where a row rests on something you measured, measure it against the
     base this round starts from -- a stale "measured" is worse than silence,
     because the worker builds on it. -->

| # | input | expected observation | why it cannot hold by accident |
|---|---|---|---|
| 1 | <!-- fill --> | <!-- fill --> | <!-- fill --> |
| 2 | <!-- fill --> | <!-- fill --> | <!-- fill --> |
| 3 | <!-- fill --> (negative control) | <!-- fill: must FAIL --> | <!-- fill --> |

## Gate

<!-- fill: prefilled from the profile. `prove` runs this one command and
     nothing else, so a second command here creates a row no proof ever
     makes. -->

```
{gate}
```

## Scope

<!-- fill: prefilled from the profile. Write scope and line budgets for
     this round. -->

| Path | Line budget | Line ranges |
|---|---|---|
{scope}

## Fabrication tells

<!-- fill: what a passing report would look like if the worker faked it. Not
     the worker lying -- the shapes you would otherwise accept. A gate green
     because its rows are unreachable. An assertion on a value the check itself
     just wrote. A name borrowed from a vocabulary the code under test never
     reads. One list item each. -->
-
"""

INJECTION_SKELETON = """\
## Task

<!-- fill: one imperative sentence naming the change. The worker reads this
     first and reads it as the whole job, so a sentence naming two things buys
     a diff that does one of them. -->

## Current behavior

<!-- fill: quote the code as it stands, citing the file and line in backticks.
     Lint checks that every quoted line still exists, because the worker is
     told this block is the code as it stands and will go looking for it: a
     quote that was true at an earlier base sends the worker off to improvise.
     Re-indenting is fine, content that has moved is not. Quote what the change
     must displace, not the whole neighbourhood. -->

```
```

## Required change

<!-- fill: what becomes true afterwards, as conditions someone outside the
     change could check. No code and no numbered steps: the worker is being
     paid to derive the implementation, so writing it here buys nothing and
     costs twice. A condition you can only state by naming the lines that
     satisfy it is a measurement -- it belongs in the oracle. -->
-

## Shape to follow

<!-- fill: at most {shape_budget} lines. Name the convention already in the
     tree that this change must match -- an existing function, module, type, or
     error shape, in backticks -- and say to follow it rather than invent a
     second one. Where two conventions in the tree could both apply, saying
     which one wins is exactly this slot's job. A constraint on the answer, not
     the answer. -->

## Reference

<!-- fill: one row per file, and the reason must say what the worker will learn
     there. "Relevant context" is not a reason and gets the file skimmed. -->

| path | why the worker must read it |
|---|---|
{reference}

## Out of scope

<!-- fill: what must not be touched, beyond what the write allowlist already
     blocks mechanically. The allowlist bounds where the worker may write; this
     bounds what it may redesign, rename, or clean up on the way past. Do not
     restate the allowlists, the report shape, or the stop-and-report rule --
     the dispatcher already sends those, and a second copy is one that
     drifts. -->
-

## Definition of done

<!-- fill: name in backticks where the gate's check lands -- the module, file,
     or suite it joins. The gate says what to run; without this the worker
     guesses and a correct diff arrives in the wrong place. The fence must
     match the oracle's `## Gate` exactly. -->

```
{gate}
```
"""


def blank_round_forms(profile: dict) -> tuple[str, str]:
    """The blank oracle and injection this profile's contract asks for.

    Shared with `revise`, which hands out the same delta form: a revision is the
    round most likely to skip a slot, because its author already holds the
    previous round in their head and writes only what changed.
    """
    gate = profile["task_contract"].get("gate_command") or (
        "<!-- fill: the exact command that must be green -->"
    )
    design_inputs = profile["task_contract"].get("design_inputs", [])
    reference = "\n".join(
        f"| `{entry['path']}` | <!-- fill --> |" for entry in design_inputs
    ) or "| `<!-- fill: path -->` | <!-- fill --> |"
    allowed = profile.get("allowed_repo_writes") or []
    budgets = profile.get("path_change_budgets") or {}
    ranges = profile.get("path_line_ranges") or {}
    scope_rows = [
        f"| `{path}` | {budgets.get(path, 'none')} | {ranges.get(path, 'any')} |"
        for path in allowed
    ]
    scope = "\n".join(scope_rows) or "| `<!-- fill: path -->` | <!-- fill: line budget --> | <!-- fill: line ranges --> |"
    return (
        ORACLE_SKELETON.format(gate=gate, scope=scope),
        INJECTION_SKELETON.format(
            gate=gate, reference=reference, shape_budget=SHAPE_LINE_BUDGET
        ),
    )


def scaffold(profile: dict, task_key: str) -> None:
    """Write the blank round form for the controller to fill.

    The structural contract used to be enforced only after the fact, which put
    the controller in the position of authoring from memory and learning what
    was required from a rejection. Handing out the slots first makes the same
    contract constructive: the form states what a round must say, and the
    remaining `<!-- fill -->` markers are themselves a finding, so a form that
    was never filled cannot be dispatched.
    """
    # And one step earlier than `lint`: `scaffold` is what *creates* the pair of
    # files, so a key the identity forbids hands the controller a form to author
    # at a path no verb of this round will ever open.
    validate_task_key(profile, task_key)
    oracle_form, injection_form = blank_round_forms(profile)

    written, kept = [], []
    for path, body in (
        (oracle_path(profile, task_key), oracle_form),
        (injection_path(profile, task_key), injection_form),
    ):
        if path.exists():
            kept.append(path)
            continue
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body)
        written.append(path)

    for path in written:
        print(f"wrote  {path}")
    for path in kept:
        print(f"kept   {path} (already authored; scaffold never overwrites)")
    if not profile.get("inject_prompt_file"):
        print(
            "\nnote: this profile declares no `inject_prompt_file`, so the "
            "injection above is not read at dispatch and not checked. Point "
            "`inject_prompt_file` at it to make it part of the round."
        )
    print("\nfill both files, then run `lint` before `dispatch`.")


def round_findings(profile: dict, task_key: str) -> list[str]:
    """Every structural finding across both halves of the round document."""
    oracle = oracle_path(profile, task_key)
    if not oracle.exists():
        raise SystemExit(f"no oracle at {oracle}")
    oracle_text = oracle.read_text()
    findings = oracle_findings(profile, oracle_text)
    # An injection is optional: a measure-only round can carry its whole
    # instruction in the oracle. Declaring one and leaving it unstructured is
    # not, because that is the half where the last false green entered.
    if profile.get("inject_prompt_file"):
        injection = injection_path(profile, task_key)
        if not injection.exists():
            findings.append(f"declared injection is missing at {injection}")
        else:
            findings.extend(
                injection_findings(
                    profile,
                    injection.read_text(),
                    oracle_text,
                    load_captures(profile, task_key),
                )
            )
    return findings


def lint(profile: dict, task_key: str) -> None:
    """Report the round document's structural findings without dispatching."""
    # Before either path is resolved. `lint` is the pre-dispatch gate -- where a
    # round is supposed to learn its form is wrong -- and a key no later verb
    # accepts resolves a *different* pair of files, so the documents that linted
    # green would not be the documents dispatched.
    validate_task_key(profile, task_key)
    oracle = oracle_path(profile, task_key)
    findings = round_findings(profile, task_key)
    allowed = profile["task_commands"].get("allow", [])
    families = profile["task_commands"].get("allow_prefix", [])
    print(f"oracle   : {oracle}")
    print(f"sections : {', '.join(oracle_sections(oracle.read_text())) or 'none'}")
    injection = profile.get("inject_prompt_file")
    print(f"injection: {injection or 'none declared; oracle carries the round'}")
    if injection and Path(injection).exists():
        print(
            "sections : "
            + (", ".join(oracle_sections(Path(injection).read_text())) or "none")
        )
    print(
        "gate cross-check: "
        + (
            f"against {len(allowed)} exact command(s) and "
            f"{len(families)} prefix famil{'y' if len(families) == 1 else 'ies'}"
            if allowed or families
            else "skipped; this round grants the worker no shell"
        )
    )
    if not findings:
        print("\nfindings: none")
        return
    print(f"\nfindings ({len(findings)}):")
    for item in findings:
        print(f"  - {item}")
    sys.exit(EXIT_FINDINGS)



def capture_path(profile: dict, task_key: str) -> Path:
    return Path(profile["state_dir"]) / "transcripts" / f"{task_key}.json"


def load_captures(profile: dict, task_key: str) -> dict[str, list[str]]:
    """Every transcript this round captured, keyed by the command that made it.

    An absent file is an empty set rather than an error: a round whose current
    behavior is entirely a code quote never captures anything, and that is a
    complete round.
    """
    path = capture_path(profile, task_key)
    if not path.exists():
        return {}
    try:
        records = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError):
        return {}
    return {
        record["command"]: record["output"]
        for record in records
        if isinstance(record, dict) and "command" in record
    }

