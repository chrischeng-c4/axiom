"""Application service for rotating material."""

from __future__ import annotations

from dataclasses import dataclass

from peer_tls.domain.rotation import Generation, RotationState, advance, reload


@dataclass(frozen=True)
class RotateMaterialService:
    def execute(self, state: RotationState, next_generation: Generation) -> RotationState:
        reloaded = reload(state, next_generation)
        return advance(reloaded)

    def observe_activation(self, state: RotationState) -> RotationState:
        return RotationState(
            phase=state.phase,
            outgoing=state.outgoing,
            incoming=state.incoming,
            active=state.active,
            activation_observed=True,
        )
