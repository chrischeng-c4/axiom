"""Canonical parent-ownership reference extraction.

@spec #2687
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum


__aw_artifact_id__ = "artifact:work-item-planning/parent-reference-extraction"


class ParentReferenceSource(StrEnum):
    """Decode-only ownership sources accepted during tracker migration."""

    LABEL = "label"
    BODY = "body"


@dataclass(frozen=True)
class DeclaredParentReference:
    """One normalized parent declaration and the source that declared it."""

    value: str
    source: ParentReferenceSource


def hash_references(value: str) -> tuple[str, ...]:
    """Extract every ``#<digits>`` reference in encounter order."""

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


def normalize_bare_reference(token: str) -> str | None:
    """Normalize one bare id, slug, or ``owner/repository/<id>`` token."""

    normalized = token.strip().strip("`").rstrip(".,;)]").strip()
    if not normalized:
        return None
    path_tail = normalized.rsplit("/", maxsplit=1)[-1]
    if path_tail.isascii() and path_tail.isdigit():
        return path_tail
    if all(
        character.isascii()
        and (character.isalnum() or character in {"-", "_", "."})
        for character in normalized
    ) and any(character.isalnum() for character in normalized):
        return normalized
    return None


def extract_parent_reference(value: str) -> str | None:
    """Return the first declared owner without consuming trailing prose.

    The first hash reference is authoritative when the suffix contains more
    than one reference. If there is no hash reference, only the first bare
    token remains eligible. A suffix with no extractable reference declares
    no parent.
    """

    references = hash_references(value)
    if references:
        return references[0]
    tokens = value.strip().split(maxsplit=1)
    if not tokens:
        return None
    return normalize_bare_reference(tokens[0])


def declared_parent_references(
    label_suffixes: tuple[str, ...],
    body_suffixes: tuple[str, ...],
) -> tuple[DeclaredParentReference, ...]:
    """Apply the same extraction rule to label and body parent prefixes."""

    declarations: list[DeclaredParentReference] = []
    for source, suffixes in (
        (ParentReferenceSource.LABEL, label_suffixes),
        (ParentReferenceSource.BODY, body_suffixes),
    ):
        for suffix in suffixes:
            if reference := extract_parent_reference(suffix):
                declarations.append(DeclaredParentReference(reference, source))
    return tuple(declarations)
