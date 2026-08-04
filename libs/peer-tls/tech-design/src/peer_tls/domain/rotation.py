"""Domain certificate rotation models and transitions for peer TLS."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from peer_tls.domain.material import TrustAnchor


class RotationPhase(str, Enum):
    STEADY = "steady"
    INCOMING_TRUSTED = "incoming_trusted"
    INCOMING_ACTIVE = "incoming_active"
    OUTGOING_RETIRED = "outgoing_retired"


@dataclass(frozen=True)
class Generation:
    number: int
    leaf_label: str


@dataclass(frozen=True)
class RotationState:
    phase: RotationPhase
    outgoing: TrustAnchor
    incoming: TrustAnchor | None
    active: Generation
    activation_observed: bool

    def admits(self, anchor_key_id: str) -> bool:
        if self.phase == RotationPhase.OUTGOING_RETIRED:
            return self.incoming is not None and self.incoming.key_id == anchor_key_id

        if self.outgoing.key_id == anchor_key_id:
            return True
        if self.incoming is not None and self.incoming.key_id == anchor_key_id:
            return True
        return False

    def requires_mutual_authentication(self) -> bool:
        return True


def advance(state: RotationState) -> RotationState:
    if state.phase == RotationPhase.STEADY:
        return RotationState(
            phase=RotationPhase.INCOMING_TRUSTED,
            outgoing=state.outgoing,
            incoming=state.incoming,
            active=state.active,
            activation_observed=state.activation_observed,
        )
    if state.phase == RotationPhase.INCOMING_TRUSTED:
        return RotationState(
            phase=RotationPhase.INCOMING_ACTIVE,
            outgoing=state.outgoing,
            incoming=state.incoming,
            active=state.active,
            activation_observed=state.activation_observed,
        )
    if state.phase == RotationPhase.INCOMING_ACTIVE:
        if not state.activation_observed:
            return state
        return RotationState(
            phase=RotationPhase.OUTGOING_RETIRED,
            outgoing=state.outgoing,
            incoming=state.incoming,
            active=state.active,
            activation_observed=state.activation_observed,
        )
    return state


def reload(state: RotationState, next_generation: Generation) -> RotationState:
    return RotationState(
        phase=state.phase,
        outgoing=state.outgoing,
        incoming=state.incoming,
        active=Generation(
            number=state.active.number + 1,
            leaf_label=next_generation.leaf_label,
        ),
        activation_observed=state.activation_observed,
    )
