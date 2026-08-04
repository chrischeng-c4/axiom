"""Domain material value objects for peer TLS."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime

from peer_tls.domain.identity import DnsName


@dataclass(frozen=True)
class SubjectAltNames:
    dns_names: tuple[DnsName, ...] = ()
    uris: tuple[str, ...] = ()


@dataclass(frozen=True)
class LeafAttributes:
    subject_alt_names: SubjectAltNames
    not_before: datetime
    not_after: datetime
    public_key_fingerprint: str
    issuer_key_id: str
    common_name: str | None = None


@dataclass(frozen=True)
class PrivateKeyAttributes:
    public_key_fingerprint: str


@dataclass(frozen=True)
class TrustAnchor:
    key_id: str
    label: str


@dataclass(frozen=True)
class TrustBundle:
    anchors: tuple[TrustAnchor, ...]

    def admits(self, issuer_key_id: str) -> bool:
        return any(anchor.key_id == issuer_key_id for anchor in self.anchors)


@dataclass(frozen=True)
class MaterialTriple:
    leaf: LeafAttributes
    key: PrivateKeyAttributes
    trust: TrustBundle
