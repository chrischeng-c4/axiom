"""AW-owned durable coordination reconciliation authority.

@spec #2587
"""

from __future__ import annotations

from dataclasses import dataclass, replace
from enum import StrEnum

from .coordination_contract_schema import (
    CoordinationAuthority,
    DispatchDocument,
    DispatchStatus,
    GateDocument,
    GateStatus,
    GateType,
    MessageDocument,
    MessageType,
    TaskDocument,
)


__aw_artifact_id__ = "artifact:workflow-root-runner/coordination-authority"
__aw_work_item__ = "2587"


class DecisionChoice(StrEnum):
    APPROVED = "approved"
    REJECTED = "rejected"


@dataclass(frozen=True)
class DecisionRecord:
    gate_id: str
    choice: DecisionChoice
    evidence: str


@dataclass(frozen=True)
class CoordinationState:
    """Workspace-owned state; client source documents are not reread."""

    task: TaskDocument
    dispatch: DispatchDocument
    gates: tuple[GateDocument, ...]
    completion_advanced: bool = False
    decision: DecisionRecord | None = None


@dataclass(frozen=True)
class Reconciliation:
    status: str
    reason: str
    completion_advanced: bool
    decision_advanced: bool
    requires_hitl: bool = False


def open_state(
    task: TaskDocument,
    dispatch: DispatchDocument,
    gates: tuple[GateDocument, ...],
) -> CoordinationState:
    """Admit only one active AW dispatch and pending unsatisfied gates."""

    if dispatch.task_id != task.task_id:
        raise ValueError("dispatch task identity does not match task")
    if dispatch.authority is not CoordinationAuthority.AW:
        raise ValueError("dispatch authority must be AW")
    if dispatch.status is not DispatchStatus.ACTIVE:
        raise ValueError("initial dispatch must be active")
    by_id = {gate.gate_id: gate for gate in gates}
    if set(by_id) != set(task.required_gates):
        raise ValueError("gate inventory must exactly match required gates")
    for gate in gates:
        if gate.task_id != task.task_id:
            raise ValueError("gate task identity does not match task")
        if gate.status is not GateStatus.PENDING or gate.evidence:
            raise ValueError("clients cannot pre-satisfy AW-owned gates")
    return CoordinationState(task=task, dispatch=dispatch, gates=gates)


def satisfy_gate(
    state: CoordinationState, gate_id: str, evidence: str
) -> CoordinationState:
    """AW records non-empty evidence on one evidence gate."""

    if not evidence:
        raise ValueError("gate evidence must be non-empty")
    gates = tuple(
        replace(gate, status=GateStatus.SATISFIED, evidence=(evidence,))
        if gate.gate_id == gate_id and gate.gate_type is GateType.EVIDENCE
        else gate
        for gate in state.gates
    )
    if gates == state.gates:
        raise ValueError("gate is missing or not an evidence gate")
    return replace(state, gates=gates)


def interrupt(state: CoordinationState) -> CoordinationState:
    """AW durably interrupts the active dispatch."""

    return replace(
        state,
        dispatch=replace(state.dispatch, status=DispatchStatus.INTERRUPTED),
    )


def submit(
    state: CoordinationState, event: MessageDocument
) -> tuple[CoordinationState, Reconciliation]:
    """Only a matching completion event with complete evidence may advance."""

    if event.task_id != state.task.task_id:
        return state, Reconciliation(
            "blocked", "task identity mismatch", False, False
        )
    if (
        state.dispatch.status is not DispatchStatus.ACTIVE
        or event.dispatch_id != state.dispatch.dispatch_id
    ):
        return state, Reconciliation(
            "blocked", "event does not target the active dispatch", False, False
        )
    if event.message_type is MessageType.BLOCKED_QUESTION:
        return state, Reconciliation(
            "blocked", "human decision is required", False, False, True
        )
    if event.message_type is not MessageType.COMPLETION:
        return state, Reconciliation("recorded", "event recorded", False, False)

    gates = {gate.gate_id: gate for gate in state.gates}
    for gate_id in state.task.required_gates:
        gate = gates[gate_id]
        if gate.status is not GateStatus.SATISFIED:
            return state, Reconciliation(
                "blocked", f"required gate {gate_id} is not satisfied", False, False
            )
        if not gate.evidence or gate_id not in event.evidence:
            return state, Reconciliation(
                "blocked", f"required gate {gate_id} lacks cited evidence", False, False
            )
    advanced = replace(state, completion_advanced=True)
    return advanced, Reconciliation("done", "completion advanced", True, False)


def decide(
    state: CoordinationState,
    gate_id: str,
    choice: DecisionChoice,
    evidence: str,
) -> tuple[CoordinationState, Reconciliation]:
    """AW records a concrete human decision; client events cannot do so."""

    if not choice or not evidence:
        raise ValueError("decision choice and human evidence must be non-empty")
    gate = next((gate for gate in state.gates if gate.gate_id == gate_id), None)
    if (
        gate is None
        or gate.gate_type is not GateType.DECISION
        or gate.authority is not CoordinationAuthority.HUMAN
    ):
        return state, Reconciliation(
            "blocked", "decision gate requires human authority", False, False
        )
    decision = DecisionRecord(gate_id, choice, evidence)
    updated_gates = tuple(
        replace(item, status=GateStatus.SATISFIED, evidence=(evidence,))
        if item.gate_id == gate_id
        else item
        for item in state.gates
    )
    return (
        replace(state, gates=updated_gates, decision=decision),
        Reconciliation("done", "decision advanced", False, True),
    )
