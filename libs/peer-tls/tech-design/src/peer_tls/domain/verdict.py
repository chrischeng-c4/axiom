"""Domain material verdict models for peer TLS."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from enum import Enum

from peer_tls.domain.identity import IdentityExpectation


class RejectionReason(str, Enum):
    IDENTITY_MISMATCH = "identity_mismatch"
    IDENTITY_IN_WRONG_EXTENSION = "identity_in_wrong_extension"
    TRUST_DOMAIN_MISMATCH = "trust_domain_mismatch"
    KEY_DOES_NOT_MATCH_LEAF = "key_does_not_match_leaf"
    ISSUER_NOT_IN_TRUST_BUNDLE = "issuer_not_in_trust_bundle"
    NOT_YET_VALID = "not_yet_valid"
    EXPIRED = "expired"
    MALFORMED_EXPECTATION = "malformed_expectation"


@dataclass(frozen=True)
class ValidityWindow:
    not_before: datetime
    not_after: datetime

    def contains(self, instant: datetime) -> bool:
        return self.not_before <= instant <= self.not_after

    def seconds_to_expiry(self, instant: datetime) -> int:
        return int((self.not_after - instant).total_seconds())


@dataclass(frozen=True)
class Rejection:
    reason: RejectionReason
    detail: str


@dataclass(frozen=True)
class ValidatedMaterial:
    window: ValidityWindow
    identity: IdentityExpectation


MaterialVerdict = ValidatedMaterial | Rejection
