"""One closed registry for AW issue type labels and delivery flows.

This module owns the type vocabulary only.  Bodies remain the responsibility
of the facade that owns their schema.  Keeping this dependency below both
``change.py`` and ``milestone.py`` prevents their ownership checks from
depending on each other.
"""

from __future__ import annotations


# The one spelling of the CLI every printed `next.command:` starts with.
# The engine lives inside the `apps/aw` uv project and its verbs are typer
# subcommands of the `aw` entry point; a printed command has to be pasteable
# from the repository root, where a bare `aw` is not on PATH.
AW_CLI = "uv run --project apps/aw aw"

DELIVERY_TYPES = (
    "feat", "fix", "refactor", "perf", "test", "docs", "chore",
)
BEHAVIOR_TYPES = ("feat", "fix", "perf")
MAINTENANCE_TYPES = ("refactor", "test", "docs", "chore")
INTAKE_TYPES = ("spike", "report")
LEGACY_TYPES = ("change", "bug", "enhancement", "epic")
MIGRATABLE_LEGACY_TYPES = ("change", "bug", "enhancement")

DELIVERY_LABELS = frozenset(f"type:{name}" for name in DELIVERY_TYPES)
INTAKE_LABELS = frozenset(f"type:{name}" for name in INTAKE_TYPES)
LEGACY_LABELS = frozenset(f"type:{name}" for name in LEGACY_TYPES)
CANONICAL_TYPES = (*DELIVERY_TYPES, *INTAKE_TYPES)
CANONICAL_LABELS = DELIVERY_LABELS | INTAKE_LABELS
TYPE_PREFIX = "type:"

FLOW_LEGS = {
    "behavior": ("e2e", "impl"),
    "maintenance": ("maint",),
}


class TypeError(ValueError):
    """A tracker label set cannot name one live delivery work item."""


def type_labels(labels: list[str] | tuple[str, ...] | set[str]) -> tuple[str, ...]:
    """Every type label, sorted so refusal messages are deterministic."""
    return tuple(sorted(label for label in labels if label.startswith(TYPE_PREFIX)))


def delivery_type(labels: list[str] | tuple[str, ...] | set[str], *, subject: str = "work item") -> str:
    """Return one canonical delivery type or reject the complete bad shape."""
    found = type_labels(labels)
    if len(found) != 1:
        rendered = ", ".join(found) or "<none>"
        raise TypeError(
            f"{subject} needs exactly one canonical delivery `type:*` label; found {rendered}"
        )
    label = found[0]
    if label in LEGACY_LABELS:
        raise TypeError(
            f"{subject} carries retired `{label}`; use the one-time type migration before a live delivery verb"
        )
    if label in INTAKE_LABELS:
        raise TypeError(
            f"{subject} carries intake `{label}`; intake cannot enter a delivery or release flow"
        )
    if label not in DELIVERY_LABELS:
        raise TypeError(
            f"{subject} carries unknown `{label}`; expected one of "
            + ", ".join(sorted(DELIVERY_LABELS))
        )
    return label[len(TYPE_PREFIX):]


def canonical_type(labels: list[str] | tuple[str, ...] | set[str], *,
                   subject: str = "work item") -> str:
    """Return exactly one active delivery or intake type."""
    found = type_labels(labels)
    if len(found) != 1:
        rendered = ", ".join(found) or "<none>"
        raise TypeError(
            f"{subject} needs exactly one canonical `type:*` label; found {rendered}"
        )
    label = found[0]
    if label in LEGACY_LABELS:
        raise TypeError(f"{subject} carries retired `{label}`")
    if label not in CANONICAL_LABELS:
        raise TypeError(f"{subject} carries unknown `{label}`")
    return label[len(TYPE_PREFIX):]


def legacy_type(labels: list[str] | tuple[str, ...] | set[str], *, subject: str = "work item") -> str:
    """Accept exactly one retired label for the migration surface only."""
    found = type_labels(labels)
    if len(found) != 1 or found[0] not in LEGACY_LABELS:
        rendered = ", ".join(found) or "<none>"
        raise TypeError(
            f"{subject} is not one unmixed legacy type; found {rendered}"
        )
    return found[0][len(TYPE_PREFIX):]


def flow_for(kind: str) -> str:
    if kind in BEHAVIOR_TYPES:
        return "behavior"
    if kind in MAINTENANCE_TYPES:
        return "maintenance"
    raise TypeError(f"`{kind}` is not a canonical delivery type")


def required_legs(kind: str) -> tuple[str, ...]:
    return FLOW_LEGS[flow_for(kind)]
