"""Declaration-shaped body dependency reference extraction.

@spec #2691
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum


__aw_artifact_id__ = "artifact:work-item-planning/dependency-reference-extraction"


class DependencyDeclarationKind(StrEnum):
    """Decode-only body prefixes accepted during tracker migration."""

    DEPENDS_ON = "depends_on"
    DEPENDENCY = "dependency"
    DEPENDENCIES = "dependencies"
    BLOCKED_BY = "blocked_by"
    REQUIRES = "requires"


@dataclass(frozen=True)
class DependencyDeclaration:
    """One declaration-shaped line and its ordered target references."""

    kind: DependencyDeclarationKind
    references: tuple[str, ...]


_PREFIXES: tuple[tuple[str, DependencyDeclarationKind], ...] = (
    ("depends on", DependencyDeclarationKind.DEPENDS_ON),
    ("dependencies", DependencyDeclarationKind.DEPENDENCIES),
    ("dependency", DependencyDeclarationKind.DEPENDENCY),
    ("blocked by", DependencyDeclarationKind.BLOCKED_BY),
    ("requires", DependencyDeclarationKind.REQUIRES),
)


def hash_references(value: str) -> tuple[str, ...]:
    """Extract every ``#<ASCII digits>`` reference in encounter order."""

    references: list[str] = []
    index = 0
    while index < len(value):
        if value[index] != "#":
            index += 1
            continue
        index += 1
        start = index
        while index < len(value) and value[index].isascii() and value[index].isdigit():
            index += 1
        if start < index:
            references.append(value[start:index])
    return tuple(references)


def normalize_declaration_line(line: str) -> str | None:
    """Remove only declaration-level list and bold formatting.

    Backtick-delimited syntax examples deliberately remain untouched, so a
    document that quotes a legacy declaration does not itself declare an edge.
    Leading whitespace is accepted only when it precedes a Markdown list
    marker; an indented bare prefix is prose continued from the preceding
    block and must not be promoted into a declaration by trimming.
    """

    normalized = line.lstrip()
    is_list_item = (
        len(normalized) >= 2
        and normalized[0] in {"-", "*", "+"}
        and normalized[1].isspace()
    )
    if len(normalized) != len(line) and not is_list_item:
        return None
    normalized = normalized.rstrip()
    if is_list_item:
        normalized = normalized[1:].lstrip()

    if normalized.startswith("**"):
        closing = normalized.find("**", 2)
        if closing >= 2:
            normalized = normalized[2:closing] + normalized[closing + 2 :]
            normalized = normalized.strip()
    return normalized


def dependency_declaration(line: str) -> DependencyDeclaration | None:
    """Decode a legacy body relation only when the line is a declaration.

    After one supported case-insensitive prefix and an optional colon, the
    first non-whitespace character must be ``#``. All hash references in that
    suffix are returned; explanatory prose and inline syntax examples return
    no declaration.
    """

    normalized = normalize_declaration_line(line)
    if normalized is None:
        return None
    lowered = normalized.lower()
    for prefix, kind in _PREFIXES:
        if not lowered.startswith(prefix):
            continue
        suffix = normalized[len(prefix) :]
        if suffix.startswith(":"):
            suffix = suffix[1:]
        suffix = suffix.lstrip()
        if not suffix.startswith("#"):
            return None
        references = hash_references(suffix)
        if references:
            return DependencyDeclaration(kind=kind, references=references)
        return None
    return None


def body_dependency_references(body: str) -> tuple[str, ...]:
    """Return ordered references from declaration-shaped body lines only."""

    references: list[str] = []
    for line in body.splitlines():
        if declaration := dependency_declaration(line):
            references.extend(declaration.references)
    return tuple(references)
