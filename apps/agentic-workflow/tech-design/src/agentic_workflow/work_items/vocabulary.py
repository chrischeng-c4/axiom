"""Terminology-first work-item vocabulary projected to agent surfaces.

@spec #2599
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class WorkItemTerm:
    name: str
    terminal_state: str


TERMS = (
    WorkItemTerm("epic", "all owned children are terminal"),
    WorkItemTerm(
        "change",
        "EC is green for the generated codebase and the lifecycle closes the change",
    ),
    WorkItemTerm(
        "spike",
        "ADR-style decision with spawned WI refs or explicit no-action; gave_up on expiry",
    ),
    WorkItemTerm(
        "report",
        "typed triage links accepted spawned work or closes duplicate, invalid, or by-design",
    ),
)
