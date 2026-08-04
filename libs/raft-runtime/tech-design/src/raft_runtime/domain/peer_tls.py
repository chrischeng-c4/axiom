from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


@dataclass(frozen=True, slots=True)
class PeerCertificate:
    subject: str
    issuer: str
    dns_names: tuple[str, ...]
    not_before_ms: int
    not_after_ms: int


@dataclass(frozen=True, slots=True)
class TrustBundle:
    issuers: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class PeerTlsConfig:
    required: bool
    trust: TrustBundle
    client_cert: PeerCertificate
    server_cert: PeerCertificate


class HandshakeOutcome(str, Enum):
    ACCEPTED = "accepted"
    UNTRUSTED_ISSUER = "untrusted-issuer"
    NOT_YET_VALID = "not-yet-valid"
    EXPIRED = "expired"
    HOSTNAME_MISMATCH = "hostname-mismatch"


def is_trusted(cert: PeerCertificate, trust: TrustBundle) -> bool:
    return cert.issuer in trust.issuers


def validity_problem(
    cert: PeerCertificate, now_ms: int
) -> HandshakeOutcome | None:
    if now_ms < cert.not_before_ms:
        return HandshakeOutcome.NOT_YET_VALID
    if now_ms >= cert.not_after_ms:
        return HandshakeOutcome.EXPIRED
    return None


def matches_identity(cert: PeerCertificate, expected_identity: str) -> bool:
    return expected_identity in cert.dns_names


def verify_peer(
    cert: PeerCertificate,
    trust: TrustBundle,
    expected_identity: str | None,
    now_ms: int,
) -> HandshakeOutcome:
    if not is_trusted(cert, trust):
        return HandshakeOutcome.UNTRUSTED_ISSUER
    val_prob = validity_problem(cert, now_ms)
    if val_prob is not None:
        return val_prob
    if expected_identity is not None and not matches_identity(
        cert, expected_identity
    ):
        return HandshakeOutcome.HOSTNAME_MISMATCH
    return HandshakeOutcome.ACCEPTED


def is_accepted(outcome: HandshakeOutcome) -> bool:
    return outcome is HandshakeOutcome.ACCEPTED
