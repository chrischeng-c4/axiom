"""Configuration plan generator adapter for peer TLS."""

from __future__ import annotations

from dataclasses import dataclass

from peer_tls.domain.material import TrustBundle
from peer_tls.domain.verdict import ValidatedMaterial


@dataclass(frozen=True)
class ServerConfigPlan:
    trust: TrustBundle
    leaf_label: str

    @property
    def peer_certificate_required(self) -> bool:
        """Structural invariant: a server plan always verifies its peer."""
        return True


@dataclass(frozen=True)
class ClientConfigPlan:
    trust: TrustBundle
    leaf_label: str

    @property
    def presents_client_certificate(self) -> bool:
        """Structural invariant: a client plan always presents material."""
        return True


def plan_server(material: ValidatedMaterial, trust: TrustBundle, leaf_label: str) -> ServerConfigPlan:
    return ServerConfigPlan(
        trust=trust,
        leaf_label=leaf_label,
    )


def plan_client(material: ValidatedMaterial, trust: TrustBundle, leaf_label: str) -> ClientConfigPlan:
    return ClientConfigPlan(
        trust=trust,
        leaf_label=leaf_label,
    )
