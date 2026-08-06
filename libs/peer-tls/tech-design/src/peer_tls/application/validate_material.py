"""Application service for validating material."""

from __future__ import annotations

from dataclasses import dataclass

from peer_tls.domain.identity import IdentityExpectation
from peer_tls.domain.material import MaterialTriple
from peer_tls.domain.validation import decide_material
from peer_tls.domain.verdict import MaterialVerdict
from peer_tls.infrastructure.ports import Clock


@dataclass(frozen=True)
class ValidateMaterialService:
    clock: Clock

    def execute(self, triple: MaterialTriple, expectation: IdentityExpectation) -> MaterialVerdict:
        return decide_material(triple, expectation, self.clock.now())
