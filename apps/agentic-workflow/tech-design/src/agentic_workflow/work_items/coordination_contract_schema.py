"""Versioned, client-independent AW coordination contract.

@spec #2586
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum
from typing import Any


__aw_artifact_id__ = "artifact:workflow-root-runner/coordination-contract-schema"
__aw_work_item__ = "2586"

PROTOCOL_VERSION = "aw.coordination.v1"


class CoordinationKind(StrEnum):
    TASK = "task"
    DISPATCH = "dispatch"
    MESSAGE = "message"
    GATE = "gate"


class DispatchStatus(StrEnum):
    ACTIVE = "active"
    SUPERSEDED = "superseded"
    INTERRUPTED = "interrupted"


class MessageType(StrEnum):
    HEARTBEAT = "heartbeat"
    COMPLETION = "completion"
    ESCALATION = "escalation"
    BLOCKED_QUESTION = "blocked_question"


class GateType(StrEnum):
    EVIDENCE = "evidence"
    DECISION = "decision"


class GateStatus(StrEnum):
    PENDING = "pending"
    SATISFIED = "satisfied"
    BLOCKED = "blocked"


class CoordinationAuthority(StrEnum):
    AW = "aw"
    HUMAN = "human"


@dataclass(frozen=True)
class TaskDocument:
    schema_version: str
    kind: CoordinationKind
    task_id: str
    workflow_root: str
    dependencies: tuple[str, ...]
    required_gates: tuple[str, ...]


@dataclass(frozen=True)
class DispatchDocument:
    schema_version: str
    kind: CoordinationKind
    task_id: str
    dispatch_id: str
    attempt: int
    assignee: str
    authority: CoordinationAuthority
    status: DispatchStatus


@dataclass(frozen=True)
class MessageDocument:
    schema_version: str
    kind: CoordinationKind
    event_id: str
    task_id: str
    dispatch_id: str
    sequence: int
    sender: str
    message_type: MessageType
    evidence: tuple[str, ...]
    body: dict[str, Any]


@dataclass(frozen=True)
class GateDocument:
    schema_version: str
    kind: CoordinationKind
    gate_id: str
    task_id: str
    gate_type: GateType
    status: GateStatus
    authority: CoordinationAuthority
    evidence: tuple[str, ...]


def require_current_protocol(schema_version: str) -> None:
    """Reject unknown versions before a document reaches lifecycle state."""

    if schema_version != PROTOCOL_VERSION:
        raise ValueError(
            f"unsupported coordination schema version {schema_version!r}; "
            f"expected {PROTOCOL_VERSION!r}"
        )


def published_schema_name(kind: CoordinationKind) -> str:
    """Return the stable language-neutral schema artifact for a document."""

    return f"{kind.value}.schema.json"
