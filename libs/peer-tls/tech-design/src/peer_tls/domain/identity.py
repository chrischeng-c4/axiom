"""Domain identity value objects for peer TLS."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class ExpectationKind(str, Enum):
    SERVING = "serving"
    PEER = "peer"


@dataclass(frozen=True)
class DnsName:
    value: str


@dataclass(frozen=True)
class TrustDomain:
    value: str


@dataclass(frozen=True)
class SpiffeId:
    trust_domain: TrustDomain
    path: str

    @property
    def uri(self) -> str:
        clean_path = self.path.lstrip("/")
        return f"spiffe://{self.trust_domain.value}/{clean_path}"


@dataclass(frozen=True)
class IdentityExpectation:
    kind: ExpectationKind
    dns_names: tuple[DnsName, ...] = ()
    spiffe_id: SpiffeId | None = None

    def is_well_formed(self) -> bool:
        if self.kind == ExpectationKind.SERVING:
            return len(self.dns_names) > 0
        if self.kind == ExpectationKind.PEER:
            return self.spiffe_id is not None
        return False
