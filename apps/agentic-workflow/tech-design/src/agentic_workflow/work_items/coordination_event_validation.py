"""Fail-closed validation for untrusted coordination events.

@spec #2588
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum

from .coordination_authority import CoordinationState
from .coordination_contract_schema import MessageDocument


__aw_artifact_id__ = "artifact:workflow-root-runner/coordination-event-validation"
__aw_work_item__ = "2588"


class EventRejectionCode(StrEnum):
    STALE_EVENT = "stale_event"
    UNAUTHORISED_EVENT = "unauthorised_event"
    INVALID_EVENT = "invalid_event"


@dataclass(frozen=True)
class EventRejection:
    code: EventRejectionCode
    reason: str
    remediation: str


def show_remediation(task_id: str) -> str:
    """Freshness and authority failures route to durable AW-owned state."""

    return f"aw coordination show {task_id}"


def schema_remediation() -> str:
    """Structural failures route to the canonical public message schema."""

    return "aw coordination schema message"


def validate_structure(event: MessageDocument) -> EventRejection | None:
    """Reject structurally invalid typed values before state is inspected."""

    identities = (
        event.event_id,
        event.task_id,
        event.dispatch_id,
        event.sender,
    )
    if any(not identity for identity in identities):
        return EventRejection(
            EventRejectionCode.INVALID_EVENT,
            "coordination event identities must be non-empty",
            schema_remediation(),
        )
    if event.sequence < 1:
        return EventRejection(
            EventRejectionCode.INVALID_EVENT,
            "coordination event sequence must be at least one",
            schema_remediation(),
        )
    if any(not item for item in event.evidence):
        return EventRejection(
            EventRejectionCode.INVALID_EVENT,
            "coordination event evidence items must be non-empty",
            schema_remediation(),
        )
    if len(set(event.evidence)) != len(event.evidence):
        return EventRejection(
            EventRejectionCode.INVALID_EVENT,
            "coordination event evidence items must be unique",
            schema_remediation(),
        )
    return None


def validate_freshness_and_authority(
    state: CoordinationState,
    event: MessageDocument,
) -> EventRejection | None:
    """Reject stale or unauthorised events without mutating AW-owned state."""

    remediation = show_remediation(state.task.task_id)
    if event.dispatch_id != state.dispatch.dispatch_id:
        return EventRejection(
            EventRejectionCode.STALE_EVENT,
            "coordination event does not target the active dispatch",
            remediation,
        )
    if event.sender != state.dispatch.assignee:
        return EventRejection(
            EventRejectionCode.UNAUTHORISED_EVENT,
            "coordination event sender is not the active assignee",
            remediation,
        )
    prior_ids = {item.event_id for item in state.events}
    prior_sequence = max((item.sequence for item in state.events), default=0)
    if event.event_id in prior_ids or event.sequence <= prior_sequence:
        return EventRejection(
            EventRejectionCode.STALE_EVENT,
            "coordination event identity or sequence is stale",
            remediation,
        )
    return None


def validate_event(
    state: CoordinationState,
    event: MessageDocument,
) -> EventRejection | None:
    """Run every fail-closed preflight before lifecycle reconciliation."""

    return validate_structure(event) or validate_freshness_and_authority(state, event)
