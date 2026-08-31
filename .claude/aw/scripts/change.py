#!/usr/bin/env python3
"""Change work-item surface -- this is where the change schema lives.

This is the issue facade over `workitem.py`. GitHub Milestones now own epic,
release, and ordering state. A change belongs to at most one milestone through
GitHub's native milestone field. The change schema arrived from somewhere else:
it was written in
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
  show       one change: body, labels, state, owning milestone
  create     open a new change, optionally assigned to one milestone
  update     edit an existing change's body, title, labels, or milestone

Plus `bodydir`, `fetch`, and `adopt`, which stage bodies locally. Every write
verb accepts --dry-run and prints the exact `gh` command it would run.
"""

from __future__ import annotations

import argparse
import hashlib
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
import milestone as milestone_surface  # noqa: E402
import wi_types  # noqa: E402
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

# Retained for read-compatible reports only.  It is never accepted by a live
# delivery verb; see ``wi_types.delivery_type``.
TYPE_LABEL = "type:change"
PHASE_LABEL = "phase:created"

# Read-only compatibility token for old issues. New writes never add it.
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
            "one write target (open a separate `type:spike` intake issue for investigation)"
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
        cells = workitem.row_cells(trimmed)
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

def _delivery_type(kind: str) -> WorkItemType:
    return WorkItemType(
        name=kind,
        type_label=f"type:{kind}",
        prog="change.py",
        # Empty on purpose. The declarative walk suits a schema authored as
        # `Section` data; this one is a transliteration, so it delegates to
        # `validate_ghan_sections` above and hands out `CHANGE_TEMPLATE` verbatim.
        sections=(),
        phase_label=PHASE_LABEL,
        validate=validate_ghan_sections,
        skeleton_text=CHANGE_TEMPLATE,
    )


CHANGE_TYPES = {kind: _delivery_type(kind) for kind in wi_types.DELIVERY_TYPES}
# Import compatibility for pure schema callers.  Live verbs never use this as
# a default: the CLI either requires ``--type`` or derives the live label.
CHANGE = CHANGE_TYPES["feat"]


def validate_body(body: str, wi_type: WorkItemType = CHANGE) -> list[str]:
    """Return every reason this body is not a valid change work-item.

    Routed through the engine rather than calling the port directly, so the
    hook the engine's write verbs use is the same one the gates measure.
    """
    return workitem.validate_body(body, wi_type)


def skeleton(wi_type: WorkItemType = CHANGE) -> str:
    """The empty GHAN template a change body must fill."""
    return workitem.skeleton(wi_type)


def legacy_parent_epics(issue: dict) -> list[str]:
    """Legacy issue epics still present during tracker migration."""
    return [
        label[len(PARENT_LABEL_PREFIX):]
        for label in issue["labels"]
        if label.startswith(PARENT_LABEL_PREFIX)
    ]


def _project_labels(labels: list[str]) -> set[str]:
    return {label for label in labels if label.startswith(("app:", "lib:"))}


def _expanded(values: list[str] | None) -> set[str]:
    return {
        label.strip()
        for value in values or []
        for label in value.split(",")
        if label.strip()
    }


def resolve_assignment(ref: str, repo: str, *, require_open: bool = True) -> tuple[dict, str]:
    """Resolve one open release Milestone and its one app/lib owner label."""
    owning = milestone_surface.resolve_milestone(ref, repo)
    identity = milestone_surface.release_identity(owning.get("title") or "")
    if identity is None:
        raise workitem.GhError(
            "the target Milestone title is not `<project>@<major>.<minor>.<patch>`"
        )
    if require_open and owning.get("state", "").lower() != "open":
        raise workitem.GhError(
            f"milestone:{owning['number']} is not open; a change cannot join a closed release"
        )
    expected = workitem.project_label(identity.project)
    if not expected.startswith(("app:", "lib:")):
        raise workitem.GhError(
            f"Milestone project `{identity.project}` resolves to `{expected}`, not one app/lib label"
        )
    return owning, expected


def require_assignment_labels(labels: list[str], expected: str) -> None:
    actual = _project_labels(labels)
    if actual != {expected}:
        rendered = ", ".join(sorted(actual)) or "<none>"
        raise workitem.GhError(
            f"Milestone assignment needs exactly `{expected}`; effective project labels are {rendered}"
        )


def verify_assignment(issue: dict, expected_number: int | None,
                      expected_label: str | None = None) -> None:
    milestone = issue.get("milestone")
    actual_number = None if not milestone else milestone.get("number")
    if actual_number != expected_number:
        expected = "no Milestone" if expected_number is None else f"milestone:{expected_number}"
        actual = "no Milestone" if actual_number is None else f"milestone:{actual_number}"
        raise workitem.GhError(
            f"tracker readback expected {expected} on #{issue['number']}, found {actual}"
        )
    if expected_label is not None:
        require_assignment_labels(issue.get("labels", []), expected_label)


def require_change(issue: dict, verb: str) -> WorkItemType:
    """Resolve the exact canonical type before any live delivery action."""
    kind = workitem.require_delivery_type(issue, verb)
    return CHANGE_TYPES[kind]


def _local_type(args) -> WorkItemType:
    return CHANGE_TYPES[args.type]


def _live_type(args, issue: dict, verb: str) -> WorkItemType:
    wi_type = require_change(issue, verb)
    args.wi_type = wi_type
    return wi_type


# --------------------------------------------------------------------------
# Change-only verbs
# --------------------------------------------------------------------------


def cmd_show(args) -> int:
    issue = fetch_issue(args.iid, args.repo)
    wi_type = _live_type(args, issue, "show")
    legacy_epics = legacy_parent_epics(issue)
    owning_milestone = issue.get("milestone")
    payload = {
        "number": issue["number"],
        "title": issue["title"],
        "state": issue["state"],
        "labels": issue["labels"],
        "type": wi_type.name,
        "flow": wi_types.flow_for(wi_type.name),
        "url": issue["url"],
        "milestone": owning_milestone,
        "legacy_epics": legacy_epics,
        "shape": body_shape(issue.get("body") or ""),
        "body": issue.get("body") or "",
    }
    if args.json:
        print(json.dumps(payload, indent=2))
    else:
        print(f"#{issue['number']} [{issue['state']}] {issue['title']}")
        print(f"  labels: {', '.join(issue['labels'])}")
        print(f"  type: {wi_type.name} ({wi_types.flow_for(wi_type.name)} flow)")
        print(f"  milestone: {(owning_milestone or {}).get('title') or '<none>'}")
        if legacy_epics:
            print(f"  legacy epic labels: {', '.join('#' + e for e in legacy_epics)}")
        print(f"  shape:  {payload['shape']}")
        print()
        print(payload["body"])
    return 0


def cmd_create(args) -> int:
    """Open a change and use GitHub's one native milestone relation."""
    args.wi_type = _local_type(args)
    args.extra_labels = []
    args.milestone_title = None
    expected_number = None
    expected_label = None
    if args.milestone:
        owning, expected_label = resolve_assignment(args.milestone, args.repo)
        expected_number = owning["number"]
        args.milestone_title = owning["title"]
        if args.project is None:
            args.project = expected_label
        require_assignment_labels([workitem.project_label(args.project)], expected_label)
    result = workitem.cmd_create(args)
    if result == 0 and args.milestone and not args.dry_run:
        if not getattr(args, "created_iid", None):
            raise workitem.GhError(
                "GitHub created the issue but returned no issue number for readback"
            )
        verify_assignment(
            fetch_issue(args.created_iid, args.repo), expected_number, expected_label
        )
    return result


def cmd_update(args) -> int:
    args.milestone_title = None
    issue = fetch_issue(args.iid, args.repo)
    _live_type(args, issue, "update")
    workitem.reject_type_label_mutation(args.add_label, args.remove_label)
    expected_number: int | None = None
    expected_label: str | None = None
    verify = False
    if args.milestone:
        owning, expected_label = resolve_assignment(args.milestone, args.repo)
        expected_number = owning["number"]
        args.milestone_title = owning["title"]
        labels = set(issue.get("labels", []))
        labels.update(_expanded(args.add_label))
        labels.difference_update(_expanded(args.remove_label))
        require_assignment_labels(sorted(labels), expected_label)
        verify = True
    elif args.remove_milestone:
        verify = True
    elif issue.get("milestone"):
        current_number = issue["milestone"].get("number")
        if current_number is None:
            raise workitem.GhError(
                f"#{issue['number']} has a Milestone without a readable number"
            )
        owning, expected_label = resolve_assignment(
            f"milestone:{current_number}", args.repo, require_open=False
        )
        expected_number = owning["number"]
        labels = set(issue.get("labels", []))
        labels.update(_expanded(args.add_label))
        labels.difference_update(_expanded(args.remove_label))
        require_assignment_labels(sorted(labels), expected_label)
        verify = True
    result = workitem.cmd_update(args)
    if result == 0 and verify and not args.dry_run:
        verify_assignment(fetch_issue(args.iid, args.repo), expected_number, expected_label)
    return result


def cmd_fetch(args) -> int:
    issue = fetch_issue(args.iid, args.repo)
    wi_type = _live_type(args, issue, "fetch")
    result = workitem.cmd_fetch(args)
    if result != 0:
        return result
    body = issue.get("body") or ""
    receipt = {
        "iid": issue["number"],
        "type": wi_type.name,
        "flow": wi_types.flow_for(wi_type.name),
        "state": issue.get("state"),
        "milestone": (issue.get("milestone") or {}).get("number"),
        "labels": sorted(issue.get("labels", [])),
        "updated_at": issue.get("updated_at"),
        "body_sha256": hashlib.sha256(body.encode("utf-8")).hexdigest(),
    }
    path = workitem.staging_dir(wi_type.name) / f"{issue['number']}.json"
    path.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print(path)
    return 0


def cmd_validate(args) -> int:
    if args.body_file:
        if not args.type:
            raise workitem.GhError("validate --body-file needs --type")
        args.wi_type = _local_type(args)
    else:
        if args.type:
            raise workitem.GhError("validate <iid> derives its type from the live issue; omit --type")
        issue = fetch_issue(args.iid, args.repo)
        _live_type(args, issue, "validate")
    return workitem.cmd_validate(args)


def cmd_lifecycle(args) -> int:
    issue = fetch_issue(args.iid, args.repo)
    _live_type(args, issue, "lifecycle")
    required = wi_types.required_legs(args.wi_type.name)
    if args.leg not in required:
        raise workitem.GhError(
            f"{args.wi_type.name} uses the {wi_types.flow_for(args.wi_type.name)} flow; "
            f"`{args.leg}` is not a required lifecycle leg"
        )
    return workitem.cmd_lifecycle(args)


def _same_milestone(left: dict, right: dict) -> bool:
    return (left.get("milestone") or {}).get("number") == (right.get("milestone") or {}).get("number")


def cmd_retype(args) -> int:
    """The only in-place type transition, before any delivery evidence exists."""
    issue = fetch_issue(args.iid, args.repo)
    current = _live_type(args, issue, "retype")
    if issue.get("state", "").upper() != "OPEN":
        raise workitem.GhError(f"#{issue['number']} is not open; type is immutable")
    if "phase:created" not in issue.get("labels", []):
        raise workitem.GhError(f"#{issue['number']} is not in `phase:created`; type is immutable")
    if workitem.has_lifecycle_evidence(issue.get("body") or ""):
        raise workitem.GhError(f"#{issue['number']} already has lifecycle rows; type is immutable")
    refs = workitem.refs_commits(issue["number"])
    if refs:
        raise workitem.GhError(f"#{issue['number']} already has Refs delivery commit(s); type is immutable")
    if args.to == current.name:
        raise workitem.GhError(f"#{issue['number']} already has `type:{args.to}`")
    before = set(issue.get("labels", []))
    labels = sorted((before - {current.type_label}) | {f"type:{args.to}"})
    workitem.replace_issue_labels(args.iid, args.repo, labels, args.dry_run)
    if args.dry_run:
        return 0
    after = fetch_issue(args.iid, args.repo)
    actual = _live_type(args, after, "retype readback")
    if actual.name != args.to:
        raise workitem.GhError(f"retype readback expected `type:{args.to}`, found `type:{actual.name}`")
    if set(after.get("labels", [])) - {actual.type_label} != before - {current.type_label}:
        raise workitem.GhError("retype readback changed a non-type label")
    if after.get("title") != issue.get("title") or (after.get("body") or "") != (issue.get("body") or ""):
        raise workitem.GhError("retype readback changed title or body")
    if after.get("state") != issue.get("state"):
        raise workitem.GhError("retype readback changed issue state")
    if not _same_milestone(issue, after):
        raise workitem.GhError("retype readback changed the Milestone relation")
    print(f"retyped #{issue['number']}: {current.name} -> {actual.name}")
    return 0


def cmd_close(args) -> int:
    issue = fetch_issue(args.iid, args.repo)
    wi_type = _live_type(args, issue, "close")
    if issue.get("state", "").upper() != "OPEN":
        raise workitem.GhError(f"#{issue['number']} is not open")
    required = wi_types.required_legs(wi_type.name)
    lifecycle_errors = workitem.lifecycle_errors(issue.get("body") or "", required)
    if lifecycle_errors:
        raise workitem.GhError(
            f"#{issue['number']} cannot close: " + "; ".join(lifecycle_errors)
        )
    recorded = workitem.lifecycle_rows(issue.get("body") or "")
    landed = set(workitem.refs_commits(issue["number"]))
    absent = [leg for leg in required if recorded[leg][1].strip("`") not in landed]
    if absent:
        raise workitem.GhError(
            f"#{issue['number']} cannot close: lifecycle commit lacks `Refs #{issue['number']}` for "
            + ", ".join(absent)
        )
    digest_trailer = {
        "e2e": "E2E-Change-Digest",
        "impl": "Impl-Change-Digest",
        "maint": "Maint-Change-Digest",
    }
    for leg in required:
        commit = recorded[leg][1].strip("`")
        digest = recorded[leg][2].strip("`")
        message = workitem.commit_message(commit)
        lines = message.splitlines()
        if not lines or not lines[0].startswith(f"{leg}("):
            raise workitem.GhError(
                f"#{issue['number']} cannot close: {commit} is not a {leg}(...) commit"
            )
        expected = f"{digest_trailer[leg]}: {digest}"
        if expected not in lines:
            raise workitem.GhError(
                f"#{issue['number']} cannot close: {leg} lifecycle digest does not match "
                f"the {digest_trailer[leg]} trailer on {commit}"
            )
    workitem.run_or_show(["issue", "close", str(args.iid), "--repo", args.repo], args.dry_run)
    if args.dry_run:
        return 0
    after = fetch_issue(args.iid, args.repo)
    if after.get("state", "").upper() != "CLOSED":
        raise workitem.GhError(f"close readback expected #{args.iid} CLOSED")
    _live_type(args, after, "close readback")
    print(f"closed #{args.iid}")
    return 0


def cmd_skeleton(args) -> int:
    args.wi_type = _local_type(args)
    return workitem.cmd_skeleton(args)


def cmd_bodydir(args) -> int:
    return workitem.cmd_bodydir(args)


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

    p = sub.add_parser("skeleton", help="emit the empty GHAN template for one delivery type")
    p.add_argument("--type", required=True, choices=wi_types.DELIVERY_TYPES)
    p.set_defaults(func=cmd_skeleton)

    p = sub.add_parser("bodydir", help="print the shared delivery-body directory")
    p.add_argument("--type", required=True, choices=wi_types.DELIVERY_TYPES)
    p.set_defaults(func=cmd_bodydir)

    p = sub.add_parser("fetch", help="stage the tracker's current body, overwriting the local copy")
    p.add_argument("iid")
    p.set_defaults(func=cmd_fetch)

    p = sub.add_parser("adopt", help="rename a staged body to <iid>.md")
    p.add_argument("path")
    p.add_argument("iid")
    p.add_argument("--type", required=True, choices=wi_types.DELIVERY_TYPES)
    p.set_defaults(func=workitem.cmd_adopt)

    p = sub.add_parser("validate", help="check a body against the GHAN rules")
    p.add_argument("iid", nargs="?", help="issue number to validate")
    p.add_argument("--body-file", help="validate this file instead of a live issue")
    p.add_argument("--type", choices=wi_types.DELIVERY_TYPES)
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=cmd_validate)

    p = sub.add_parser("show", help="one change: body, labels, state, owning milestone")
    p.add_argument("iid")
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=cmd_show)

    p = sub.add_parser("create", help="open a new change")
    p.add_argument("--title", required=True)
    p.add_argument("--body-file", required=True)
    p.add_argument("--type", required=True, choices=wi_types.DELIVERY_TYPES)
    p.add_argument("--milestone", help="`milestone:<number>` or exact `<project>@<version>` title")
    p.add_argument("--priority", default="p2", choices=PRIORITIES)
    p.add_argument("--project", help="bare project name or a qualified label")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_create)

    p = sub.add_parser("lifecycle", help="record a landed leg in the body's lifecycle block")
    p.add_argument("iid")
    p.add_argument("--leg", required=True, choices=LEGS)
    p.add_argument("--commit", required=True, help="the full sha the leg landed as")
    p.add_argument("--digest", required=True,
                   help="the change digest the leg's commit was measured against")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_lifecycle)

    p = sub.add_parser("update", help="edit an existing change")
    p.add_argument("iid")
    p.add_argument("--body-file")
    p.add_argument("--title")
    p.add_argument("--add-label", action="append")
    p.add_argument("--remove-label", action="append")
    ownership = p.add_mutually_exclusive_group()
    ownership.add_argument("--milestone", help="assign to `milestone:<number>` or an exact title")
    ownership.add_argument("--remove-milestone", action="store_true")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_update)

    p = sub.add_parser("retype", help="change delivery type before any delivery evidence exists")
    p.add_argument("iid")
    p.add_argument("--to", required=True, choices=wi_types.DELIVERY_TYPES)
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_retype)

    p = sub.add_parser("close", help="close an open delivery issue after its required lifecycle")
    p.add_argument("iid")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_close)

    return parser


def main(argv: list[str] | None = None) -> int:
    return workitem.dispatch(build_parser().parse_args(argv), None, LOCAL_VERBS)


if __name__ == "__main__":
    raise SystemExit(main())
