#!/usr/bin/env python3
"""Change work-item surface -- this is where the change schema lives.

This is the second facade over `workitem.py`, and it is shaped differently from
`epic.py` for historical reasons worth keeping. The epic schema is this plugin's
own invention, so `epic.py` declares it as `Section` data and lets the engine
walk it. The change schema arrived from somewhere else: it was written in
`apps/agentic-workflow/src/issues/ghan.rs`, `aw wi validate` enforced it, and
the tracker's 640 change work items were judged by it. While that crate existed
this file was a *port*, and writing a second, prettier reading of the rules
would have produced a fork that stays invisible for exactly as long as the two
readings happen to agree.

The crate is gone. This file is now the only definition, and the transliteration
is kept rather than tidied: the error strings are verbatim, the short-circuits
are in the same places, the helpers keep their original names, and the markdown
primitives reproduce Rust's `str` semantics (`_lines` is `str::lines`,
`_split_on` is `str::split(char_predicate)`, `_ascii_lower` is
`to_ascii_lowercase`) rather than Python's near-misses. That is not nostalgia --
those primitives are the behaviour the existing population was judged by, so
replacing one with its Python near-miss re-judges live work items. Pythonising
this file is a schema change.

`verification/check_change_schema.py` is what holds it in place now that there
is no upstream to compare against: a declared inventory of every section,
heading and word; a liveness probe proving each declared entry still refuses
something; and a traced assertion that every `errors.append` site in the
validators is reached by some case.

One rule of the original `validate_ghan_body` is deliberately absent:
`looks_too_large_for_atomic_wi`, a boundedness heuristic over prose that lived
in `planner.rs` rather than in the section schema. It was measured to be the
single excluded error class while both sides still existed.

Verbs
-----
  skeleton   emit the empty GHAN template a change body must fill
  validate   check a body (by iid or by file) against the GHAN rules
  show       one change: body, labels, state, owning epic
  create     open a new change, optionally labelled into its epic
  update     edit an existing change's body, title, or labels

Plus `bodydir`, `fetch`, and `adopt`, which stage bodies locally. Every write
verb accepts --dry-run and prints the exact `gh` command it would run.
"""

from __future__ import annotations

import argparse
import json
import string
import sys
from pathlib import Path

# See the note in `epic.py`: the script's directory is on `sys.path` when this
# runs as a script but not when `verification/_paths.py` loads it through
# `importlib.exec_module`, and the gates must import the same engine the CLI
# does.
sys.path.insert(0, str(Path(__file__).resolve().parent))

import workitem  # noqa: E402
from workitem import (  # noqa: E402,F401
    AW_TOML,
    LEGS,
    PRIORITIES,
    REPO_ROOT,
    WORK_ITEM_TYPES,
    WORKITEMS_DIR_REL,
    GhError,
    WorkItemType,
    default_repo,
    fetch_issue,
    gh,
    project_label,
    run_or_show,
    staging_dir,
)

TYPE_LABEL = "type:change"
PHASE_LABEL = "phase:created"

# The label a change carries to name the epic that owns it. It is the same
# string `epic.py` reads as its child claim, and it is written out here rather
# than imported: one facade importing another would make the epic surface a
# dependency of the change surface, which is backwards -- a change is a
# work item whether or not any epic exists. `check_change_schema.py` asserts
# the two spellings agree, so the duplication is measured rather than trusted.
PARENT_LABEL_PREFIX = "epic:"


# ==========================================================================
# The GHAN schema. Transliterated from Rust, and kept that way -- see the
# module docstring for why the un-Pythonic primitives are load-bearing.
# ==========================================================================

# The four canonical GHAN H2 headings, in canonical order.
GHAN_SECTIONS = ("## Goal", "## How", "## Acceptance", "## Never")

# The legacy six-section change-body headings GHAN coexists with.
LEGACY_SECTIONS = (
    "## Problem",
    "## Capability Alignment",
    "## Requirements",
    "## Scope",
    "## Acceptance Criteria",
    "## Reference Context",
)

HOW_PREMISES = "### Verified premises"
HOW_CHANGE_POINTS = "### Change points"
HOW_FROZEN = "### Frozen decisions"
ACCEPTANCE_NEGATIVE_CONTROL = "### Negative control"
NEVER_MUST_NOT_TOUCH = "### Must not touch"
NEVER_MUST_NOT_DO = "### Must not do"

# Hedges that turn a premise from an observation into an inference.
HEDGE_WORDS = (
    "should",
    "might",
    "probably",
    "seems",
    "appears",
    "likely",
    "presumably",
    "supposedly",
    "應該",
    "可能",
    "推測",
    "看起來",
    "似乎",
    "大概",
    "或許",
)

# Phrases that make a negative control require the gate to go red.
FAILURE_ASSERTIONS = (
    "must fail",
    "must go red",
    "must be red",
    "must turn red",
    "必須紅",
    "必須失敗",
    "必须红",
)

# Which authored vocabulary a work-item body is written in. The crate models
# this as an enum; here the four names are the values themselves, because the
# only consumers are a comparison and a printout.
SHAPES = ("unstructured", "legacy", "ghan", "mixed")


# --------------------------------------------------------------------------
# Rust `str` primitives
#
# Python's near-equivalents are not equivalents. `splitlines()` breaks on \v,
# \f and  , which `str::lines` does not; `.lower()` is Unicode-aware,
# which `to_ascii_lowercase` is not; `re.split` on a character class drops
# different edge cases than `str::split(char_predicate)`. Each difference is
# invisible on ordinary prose and decides a validation outcome on the input
# that hits it, so the primitives are reproduced rather than approximated.
# --------------------------------------------------------------------------

_ASCII_LOWER = str.maketrans(string.ascii_uppercase, string.ascii_lowercase)


def _ascii_lower(text: str) -> str:
    """Rust's `str::to_ascii_lowercase`: A-Z only, every other char untouched."""
    return text.translate(_ASCII_LOWER)


def _lines(text: str) -> list[str]:
    """Rust's `str::lines`: split on \\n, drop one trailing \\r, final \\n optional."""
    parts = text.split("\n")
    if parts and parts[-1] == "":
        parts.pop()
    return [part[:-1] if part.endswith("\r") else part for part in parts]


def _split_on(text: str, predicate) -> list[str]:
    """Rust's `str::split(char_predicate)`: separators dropped, empties kept."""
    parts: list[str] = []
    buffer: list[str] = []
    for char in text:
        if predicate(char):
            parts.append("".join(buffer))
            buffer = []
        else:
            buffer.append(char)
    parts.append("".join(buffer))
    return parts


def _is_ascii_digit(char: str) -> bool:
    return "0" <= char <= "9"


def _is_ascii_alphanumeric(char: str) -> bool:
    return char.isascii() and char.isalnum()


def _all_ascii_digits(text: str) -> bool:
    return all(_is_ascii_digit(char) for char in text)


def _trim_matches(text: str, predicate) -> str:
    """Rust's `str::trim_matches(char_predicate)`: both ends."""
    start, end = 0, len(text)
    while start < end and predicate(text[start]):
        start += 1
    while end > start and predicate(text[end - 1]):
        end -= 1
    return text[start:end]


def _rsplit_once(text: str, sep: str):
    """Rust's `str::rsplit_once`: (head, tail) or None."""
    head, found, tail = text.rpartition(sep)
    return (head, tail) if found else None


# --------------------------------------------------------------------------
# Shape detection
# --------------------------------------------------------------------------


def body_shape(body: str) -> str:
    """Classify a change-body by the heading vocabulary it uses."""
    headings = h2_headings(body)
    ghan = any(heading_in(h, GHAN_SECTIONS) for h in headings)
    legacy = any(heading_in(h, LEGACY_SECTIONS) for h in headings)
    if ghan and legacy:
        return "mixed"
    if ghan:
        return "ghan"
    if legacy:
        return "legacy"
    return "unstructured"


def validate_ghan_sections(body: str) -> list[str]:
    """Section-level GHAN rules, in their original order and with their strings."""
    headings = h2_headings(body)
    errors: list[str] = []

    for required in GHAN_SECTIONS:
        if not any(heading_eq(h, required) for h in headings):
            errors.append(f"ghan: missing required {required} section")
    for heading in headings:
        if not heading_in(heading, GHAN_SECTIONS):
            errors.append(
                f"ghan: unexpected H2 `{heading.strip()}`; a GHAN work item carries "
                f"exactly: {', '.join(GHAN_SECTIONS)}"
            )
    # Per-section rules read section content; reporting them against a missing
    # or foreign section would bury the structural cause under noise.
    if errors:
        return errors

    goal = section_at(body, 2, "## Goal") or ""
    how = section_at(body, 2, "## How") or ""
    acceptance = section_at(body, 2, "## Acceptance") or ""
    never = section_at(body, 2, "## Never") or ""

    errors.extend(validate_goal(goal))
    errors.extend(validate_how(how))
    errors.extend(validate_acceptance(acceptance))
    # `how` goes in raw, not comment-stripped: the section is passed as it was
    # read, and stripping here would let a commented-out change point stop
    # colliding with a must-not-touch entry. No case discriminates this, which
    # is recorded as a gap in `check_change_schema_negative_control.py` rather
    # than defended by a control that cannot go red.
    errors.extend(validate_never(never, how))
    return errors


# --------------------------------------------------------------------------
# Goal
# --------------------------------------------------------------------------


def validate_goal(content: str) -> list[str]:
    content = strip_comments(content)
    errors: list[str] = []
    trimmed = content.strip()
    if not trimmed:
        return ["ghan: ## Goal is empty"]
    marker = placeholder_marker(trimmed)
    if marker is not None:
        errors.append(f"ghan: ## Goal still carries the `{marker}` placeholder")
    if any(is_list_item(line) for line in _lines(trimmed)):
        errors.append("ghan: ## Goal must be one observable-difference sentence, not a list")
    paragraphs = len([part for part in trimmed.split("\n\n") if part.strip()])
    if paragraphs > 1:
        errors.append("ghan: ## Goal must be a single paragraph naming one observation point")
    return errors


# --------------------------------------------------------------------------
# How
# --------------------------------------------------------------------------


def validate_how(content: str) -> list[str]:
    content = strip_comments(content)
    errors: list[str] = []
    for required in (HOW_PREMISES, HOW_CHANGE_POINTS, HOW_FROZEN):
        if section_at(content, 3, required) is None:
            errors.append(f"ghan: ## How missing `{required}` sub-section")
    if errors:
        return errors

    premises = section_at(content, 3, HOW_PREMISES) or ""
    premise_items = list_items(premises)
    if not premise_items:
        errors.append(f"ghan: `{HOW_PREMISES}` needs at least one observed premise")
    for item in premise_items:
        if file_line_ref(item) is None:
            errors.append(
                f"ghan: premise carries no `file:line` evidence coordinate: '{preview(item)}'"
            )
        hedge = hedge_word(item)
        if hedge is not None:
            errors.append(
                f"ghan: premise hedges with '{hedge}'; a premise is an observation, "
                f"not an inference: '{preview(item)}'"
            )

    change_points = section_at(content, 3, HOW_CHANGE_POINTS) or ""
    change_items = list_items(change_points)
    if not change_items:
        errors.append(
            f"ghan: `{HOW_CHANGE_POINTS}` is empty; a change work item must name at least "
            "one write target (use `--type spike` for investigation)"
        )
    for item in change_items:
        if path_ref(item) is None:
            errors.append(f"ghan: change point names no path: '{preview(item)}'")

    frozen = section_at(content, 3, HOW_FROZEN) or ""
    if not any(is_real_line(line) for line in _lines(frozen)):
        errors.append(
            f"ghan: `{HOW_FROZEN}` must record the decisions and exclusions already fixed "
            "(write `none` explicitly if there are none)"
        )
    return errors


# --------------------------------------------------------------------------
# Acceptance
# --------------------------------------------------------------------------


def validate_acceptance(content: str) -> list[str]:
    content = strip_comments(content)
    errors: list[str] = []
    rows = table_rows(content)
    if not rows:
        errors.append("ghan: ## Acceptance needs a gate table with at least one row")
    for row in rows:
        if len(row) < 5:
            errors.append(
                "ghan: gate row needs 5 columns (#, command, current, target, why it "
                f"cannot hold by accident): '{preview(' | '.join(row))}'"
            )
            continue
        command = row[1].strip()
        current = row[2].strip()
        target = row[3].strip()
        why = row[4].strip()
        if "`" not in command:
            errors.append(
                f"ghan: gate command must be a verbatim backticked command: '{preview(command)}'"
            )
        if _ascii_lower(current) == _ascii_lower(target):
            errors.append(
                f"ghan: gate row states the same current and target observation "
                f"('{preview(current)}'); it cannot discriminate"
            )
        if not is_real_line(why):
            errors.append(
                f"ghan: gate row must say why it cannot hold by accident: '{preview(command)}'"
            )

    control = section_at(content, 3, ACCEPTANCE_NEGATIVE_CONTROL)
    if control is None:
        errors.append(
            f"ghan: ## Acceptance missing `{ACCEPTANCE_NEGATIVE_CONTROL}`; a gate nobody "
            "has seen fail proves nothing"
        )
    else:
        if not asserts_failure(control):
            errors.append(
                f"ghan: `{ACCEPTANCE_NEGATIVE_CONTROL}` must require the gate to go red "
                "under the mutation"
            )
        if sha256_token(control) is None:
            errors.append(
                f"ghan: `{ACCEPTANCE_NEGATIVE_CONTROL}` must name the sha256 the mutated "
                "file restores to"
            )
    return errors


# --------------------------------------------------------------------------
# Never
# --------------------------------------------------------------------------


def validate_never(content: str, how: str) -> list[str]:
    content = strip_comments(content)
    errors: list[str] = []
    first = next((line.strip() for line in _lines(content) if line.strip()), None)
    if first is None:
        return ["ghan: ## Never is empty"]
    if is_list_item(first) or first.startswith("#"):
        errors.append(
            "ghan: ## Never must open with a line fixing the addressee before any list"
        )

    missing_list = False
    for required in (NEVER_MUST_NOT_TOUCH, NEVER_MUST_NOT_DO):
        listed = section_at(content, 3, required)
        if listed is None:
            errors.append(f"ghan: ## Never missing `{required}` list")
            missing_list = True
        elif not list_items(listed):
            errors.append(
                f"ghan: `{required}` has no entries; a limit nobody can name is not a limit"
            )
    if missing_list:
        return errors

    change_paths = set()
    for item in list_items(section_at(how, 3, HOW_CHANGE_POINTS) or ""):
        token = path_ref(item)
        if token is not None:
            change_paths.add(normalize_path(token))
    must_not_touch = section_at(content, 3, NEVER_MUST_NOT_TOUCH) or ""
    for item in list_items(must_not_touch):
        path = path_ref(item)
        if path is None:
            continue
        if normalize_path(path) in change_paths:
            errors.append(f"ghan: `{path}` is listed as both a change point and must-not-touch")
    return errors


# --------------------------------------------------------------------------
# Markdown helpers
# --------------------------------------------------------------------------


def h2_headings(body: str) -> list[str]:
    """H2 heading lines in document order, verbatim and fence-aware."""
    out: list[str] = []
    fence: int | None = None
    for raw in _lines(body):
        length = fence_len(raw)
        if length is not None:
            if fence is not None and length >= fence:
                fence = None
            elif fence is None:
                fence = length
            continue
        if fence is not None:
            continue
        line = raw.rstrip()
        if line.startswith("## ") and not line.startswith("### "):
            out.append(line)
    return out


def section_at(text: str, level: int, heading: str) -> str | None:
    """Content under an exact heading line at `level`, up to the next heading
    at the same or a shallower level."""
    prefix = "#" * level + " "
    deeper = "#" * (level + 1) + " "
    collecting = False
    found = False
    out: list[str] = []
    fence: int | None = None

    for raw in _lines(text):
        length = fence_len(raw)
        if length is not None:
            if fence is not None and length >= fence:
                fence = None
            elif fence is None:
                fence = length
            if collecting:
                out.append(raw)
            continue
        line = raw.rstrip()
        is_heading = (
            fence is None
            and line.startswith("#")
            and (line.startswith(prefix) or shallower_heading(line, level))
        )
        if is_heading and not line.startswith(deeper):
            if collecting:
                break
            if heading_eq(line, heading):
                collecting = True
                found = True
            continue
        if collecting:
            out.append(raw)
    return "\n".join(out) if found else None


def shallower_heading(line: str, level: int) -> bool:
    """Is `line` an ATX heading strictly shallower than `level`?"""
    hashes = len(line) - len(line.lstrip("#"))
    return hashes > 0 and hashes < level and line[hashes:].startswith(" ")


def fence_len(line: str) -> int | None:
    """Opening/closing fence width, or None when the line is not a fence."""
    trimmed = line.lstrip()
    ticks = len(trimmed) - len(trimmed.lstrip("`"))
    if ticks >= 3:
        return ticks
    tildes = len(trimmed) - len(trimmed.lstrip("~"))
    return tildes if tildes >= 3 else None


def heading_eq(line: str, heading: str) -> bool:
    return _ascii_lower(line.strip()) == _ascii_lower(heading.strip())


def heading_in(line: str, headings) -> bool:
    return any(heading_eq(line, candidate) for candidate in headings)


def is_list_item(line: str) -> bool:
    trimmed = line.lstrip()
    return trimmed.startswith("- ") or trimmed.startswith("* ") or trimmed.startswith("+ ")


def list_items(content: str) -> list[str]:
    """Bullet text with the marker stripped."""
    items = []
    for line in _lines(content):
        if not is_list_item(line):
            continue
        item = line.lstrip().lstrip("-*+").strip()
        if item:
            items.append(item)
    return items


def strip_comments(text: str) -> str:
    """Drop HTML comment spans `<!-- ... -->` from text."""
    out: list[str] = []
    rest = text
    while True:
        start = rest.find("<!--")
        if start < 0:
            break
        out.append(rest[:start])
        end = rest.find("-->", start)
        if end < 0:
            rest = ""
            break
        rest = rest[end + 3:]
    out.append(rest)
    return "".join(out)


def placeholder_marker(text: str) -> str | None:
    for marker in ("(fill)", "(replace-this)"):
        if marker in text:
            return marker
    return None


def is_real_line(line: str) -> bool:
    """Non-empty, non-placeholder content."""
    trimmed = _ascii_lower(line.strip().lstrip("-*+#").strip())
    if not trimmed:
        return False
    return trimmed not in (
        "(fill)", "(replace-this)", "tbd", "todo", "maybe", "unclear", "uncertain",
    )


def file_line_ref(text: str) -> str | None:
    """A `path/to/file.rs:123` evidence coordinate."""
    for token in _split_on(text, is_token_break):
        split = _rsplit_once(token, ":")
        if split is None:
            continue
        path, line = split
        digits = line
        while digits and not _is_ascii_digit(digits[-1]):
            digits = digits[:-1]
        if (
            digits
            and _all_ascii_digits(digits)
            and "." in path
            and any(_is_ascii_alphanumeric(c) for c in path)
        ):
            return token
    return None


def path_ref(text: str) -> str | None:
    """A path-like token, with or without a line suffix."""
    known = ("rs", "py", "md", "toml", "json", "yaml", "yml", "sh", "ts", "tsx",
             "js", "jsx", "sql", "proto")
    for token in _split_on(text, is_token_break):
        bare = token.rstrip(":.,")
        split = _rsplit_once(bare, ":")
        if split is not None and _all_ascii_digits(split[1]):
            bare = split[0]
        if not bare:
            continue
        dotted = _rsplit_once(bare, ".")
        has_known_ext = dotted is not None and dotted[0] != "" and dotted[1] in known
        if has_known_ext or ("/" in bare and "." in bare):
            return token
    return None


def is_token_break(char: str) -> bool:
    return char.isspace() or char in "`,()[]\"';"


def normalize_path(token: str) -> str:
    bare = _trim_matches(token, lambda c: is_token_break(c) or c == ".")
    split = _rsplit_once(bare, ":")
    if split is not None and split[1] and _all_ascii_digits(split[1]):
        return split[0]
    return bare


def hedge_word(text: str) -> str | None:
    lower = _ascii_lower(text)
    for hedge in HEDGE_WORDS:
        if hedge.isascii():
            words = _split_on(lower, lambda c: not _is_ascii_alphanumeric(c))
            if any(word == hedge for word in words):
                return hedge
        elif hedge in text:
            return hedge
    return None


def asserts_failure(text: str) -> bool:
    lower = _ascii_lower(text)
    return any(phrase in lower or phrase in text for phrase in FAILURE_ASSERTIONS)


def sha256_token(text: str) -> str | None:
    for token in _split_on(text, is_token_break):
        if len(token) == 64 and all(c in string.hexdigits for c in token):
            return token
    return None


def table_rows(content: str) -> list[list[str]]:
    """Data rows of the first markdown table, header and separator removed."""
    rows: list[list[str]] = []
    seen_separator = False
    for line in _lines(content):
        trimmed = line.strip()
        if not trimmed.startswith("|"):
            if seen_separator and rows:
                break
            continue
        cells = [cell.strip() for cell in trimmed.strip("|").split("|")]
        if all(cell and all(c in "-:" for c in cell) for cell in cells):
            seen_separator = True
            continue
        if seen_separator:
            rows.append(cells)
    return rows


def preview(text: str) -> str:
    flat = " ".join(text.split())
    return flat if len(flat) <= 60 else flat[:60]


# ==========================================================================
# The empty body
#
# This is the form a human fills in, and it was not authored here: it is the
# template `aw wi create --type change` handed out, transcribed byte-for-byte
# from its one `return` site. `check_change_schema.py` compares it against
# `_fixtures/change_skeleton.md`, which was lifted from that site before the
# crate was removed.
# ==========================================================================

CHANGE_TEMPLATE = """## Goal

<!-- Goal must be one observable-difference sentence, not a list. -->

## How

### Verified premises

<!-- Premise needs at least one observed premise with a file:line evidence coordinate. Must be an observation, not an inference (no hedge words like should/might). -->

### Change points

<!-- Change point must name at least one write target path. -->

### Frozen decisions

<!-- Must record the decisions and exclusions already fixed (write 'none' explicitly if there are none). -->

## Acceptance

<!-- Table with columns: # | command | current | target | why it cannot hold by accident -->

### Negative control

<!-- Describe the mutation and assert failure ('must fail', 'must go red'), and name the sha256 the mutated file restores to. -->

## Never

This addresses the worker implementing this work item, not the controller reviewing it.

### Must not touch

<!-- List paths that must not be touched. -->

### Must not do

<!-- List actions that must not be taken. -->
"""


# ==========================================================================
# Type binding
# ==========================================================================

CHANGE = WorkItemType(
    name="change",
    type_label=TYPE_LABEL,
    prog="change.py",
    # Empty on purpose. The declarative walk suits a schema authored as
    # `Section` data; this one is a transliteration, so it delegates to
    # `validate_ghan_sections` above and hands out `CHANGE_TEMPLATE` verbatim.
    sections=(),
    phase_label=PHASE_LABEL,
    validate=validate_ghan_sections,
    skeleton_text=CHANGE_TEMPLATE,
)


def validate_body(body: str, wi_type: WorkItemType = CHANGE) -> list[str]:
    """Return every reason this body is not a valid change work-item.

    Routed through the engine rather than calling the port directly, so the
    hook the engine's write verbs use is the same one the gates measure.
    """
    return workitem.validate_body(body, wi_type)


def skeleton(wi_type: WorkItemType = CHANGE) -> str:
    """The empty GHAN template a change body must fill."""
    return workitem.skeleton(wi_type)


def parent_epics(issue: dict) -> list[str]:
    """The epics this change is labelled into, by iid."""
    return [
        label[len(PARENT_LABEL_PREFIX):]
        for label in issue["labels"]
        if label.startswith(PARENT_LABEL_PREFIX)
    ]


def require_change(issue: dict, verb: str) -> None:
    workitem.require_type(issue, verb, CHANGE)


# --------------------------------------------------------------------------
# Change-only verbs
# --------------------------------------------------------------------------


def cmd_show(args) -> int:
    issue = fetch_issue(args.iid, args.repo)
    require_change(issue, "show")
    epics = parent_epics(issue)
    payload = {
        "number": issue["number"],
        "title": issue["title"],
        "state": issue["state"],
        "labels": issue["labels"],
        "url": issue["url"],
        "epics": epics,
        "shape": body_shape(issue.get("body") or ""),
        "body": issue.get("body") or "",
    }
    if args.json:
        print(json.dumps(payload, indent=2))
    else:
        print(f"#{issue['number']} [{issue['state']}] {issue['title']}")
        print(f"  labels: {', '.join(issue['labels'])}")
        print(f"  epic:   {', '.join('#' + e for e in epics) or '<none>'}")
        print(f"  shape:  {payload['shape']}")
        print()
        print(payload["body"])
    return 0


def cmd_create(args) -> int:
    """Open a change, carrying its owning epic's label if one was named.

    The engine writes whatever `extra_labels` holds; deciding that
    `epic:<iid>` is what ownership is spelled as belongs here, because it is a
    fact about the change type and not about labels in general.
    """
    args.extra_labels = [f"{PARENT_LABEL_PREFIX}{args.epic}"] if args.epic else []
    return workitem.cmd_create(args)


# --------------------------------------------------------------------------
# CLI
# --------------------------------------------------------------------------

# The verbs that never reach the tracker, so a missing issue-platform config
# must not stop them.
LOCAL_VERBS = ("skeleton", "bodydir", "adopt", "validate")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="change.py",
        description="Change work-item surface over the `gh` CLI (GHAN rules).",
    )
    parser.add_argument("--repo", help="owner/name; defaults to aw.toml's issue platform")
    sub = parser.add_subparsers(dest="verb", required=True)

    p = sub.add_parser("skeleton", help="emit the empty GHAN template for a change body")
    p.set_defaults(func=workitem.cmd_skeleton)

    p = sub.add_parser("bodydir", help="print (and create) the directory bodies are staged in")
    p.add_argument("--type", default="change", choices=WORK_ITEM_TYPES)
    p.set_defaults(func=workitem.cmd_bodydir)

    p = sub.add_parser("fetch", help="stage the tracker's current body, overwriting the local copy")
    p.add_argument("iid")
    p.set_defaults(func=workitem.cmd_fetch)

    p = sub.add_parser("adopt", help="rename a staged body to <iid>.md")
    p.add_argument("path")
    p.add_argument("iid")
    p.set_defaults(func=workitem.cmd_adopt)

    p = sub.add_parser("validate", help="check a body against the GHAN rules")
    p.add_argument("iid", nargs="?", help="issue number to validate")
    p.add_argument("--body-file", help="validate this file instead of a live issue")
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=workitem.cmd_validate)

    p = sub.add_parser("show", help="one change: body, labels, state, owning epic")
    p.add_argument("iid")
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=cmd_show)

    p = sub.add_parser("create", help="open a new change")
    p.add_argument("--title", required=True)
    p.add_argument("--body-file", required=True)
    p.add_argument("--epic", help="iid of the epic that owns this change")
    p.add_argument("--priority", default="p2", choices=PRIORITIES)
    p.add_argument("--project", help="bare project name or a qualified label")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_create)

    p = sub.add_parser("lifecycle", help="record a landed leg in the body's lifecycle block")
    p.add_argument("iid")
    p.add_argument("--leg", required=True, choices=LEGS)
    p.add_argument("--commit", required=True, help="the full sha the leg landed as")
    p.add_argument("--digest", help="the change digest the leg was reviewed against")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=workitem.cmd_lifecycle)

    p = sub.add_parser("update", help="edit an existing change")
    p.add_argument("iid")
    p.add_argument("--body-file")
    p.add_argument("--title")
    p.add_argument("--add-label", action="append")
    p.add_argument("--remove-label", action="append")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=workitem.cmd_update)

    return parser


def main(argv: list[str] | None = None) -> int:
    return workitem.dispatch(build_parser().parse_args(argv), CHANGE, LOCAL_VERBS)


if __name__ == "__main__":
    raise SystemExit(main())
