"""Status reporting and credential redaction for service-k8s.

Certificate state is published so operators can diagnose TLS handshakes,
expiry dates, and active rotations. To prevent secret leaks (e.g. private keys,
bearer tokens, or service account JWTs), CertificateFacts carries no
free-form payload fields, and all output messages pass through a multi-pass
redaction scrubber.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from typing import Final

from service_k8s.application.rotation import (
    Action,
    Issue,
    IssueReason,
    IssuerId,
)
from service_k8s.domain.condition import ConditionFact, ConditionStatus
from service_k8s.domain.purpose import Purpose

READY_CONDITION: Final[str] = "CertificateReady"
ROTATING_CONDITION: Final[str] = "CertificateRotating"


def short_fingerprint(fingerprint: str) -> str:
    return fingerprint[:16]


def rotation_reason(reason: IssueReason) -> str:
    mapping = {
        IssueReason.BOOTSTRAP: "Bootstrap",
        IssueReason.RENEWAL: "Renewal",
        IssueReason.EXPIRED: "Expired",
        IssueReason.IDENTITY_CHANGED: "IdentityChanged",
        IssueReason.ISSUER_ROTATION: "IssuerRotation",
    }
    return mapping[reason]


def rotation_detail(reason: IssueReason) -> str:
    mapping = {
        IssueReason.BOOTSTRAP: "no material has been issued yet",
        IssueReason.RENEWAL: "the renewal window has opened",
        IssueReason.EXPIRED: "the projected leaf is past its notAfter",
        IssueReason.IDENTITY_CHANGED: "the requested names no longer match the leaf",
        IssueReason.ISSUER_ROTATION: "the configured issuer changed",
    }
    return mapping[reason]


def condition_prefix(purpose: Purpose) -> str:
    if purpose == Purpose.SERVING:
        return "Serving"
    return "Peer"


def _strip_pem(text: str) -> str:
    out = []
    rest = text
    while True:
        start = rest.find("-----BEGIN")
        if start == -1:
            break
        out.append(rest[:start])
        out.append("[redacted pem]")
        end = rest.find("-----END", start)
        if end == -1:
            rest = ""
            break
        tail = rest[end:]
        close = tail.find("-----", 5)
        rest = "" if close == -1 else tail[close + 5 :]
    out.append(rest)
    return "".join(out)


def _strip_bearer(text: str) -> str:
    segments = text.split("Bearer ")
    out = [segments[0]]
    for segment in segments[1:]:
        out.append("Bearer [redacted]")
        for index, ch in enumerate(segment):
            if ch.isspace():
                out.append(segment[index:])
                break
    return "".join(out)


def _strip_jwt(text: str) -> str:
    words = []
    for word in text.split():
        parts = word.split(".")
        looks_like_jwt = len(parts) == 3 and all(
            len(part) >= 8
            and all(c.isascii() and (c.isalnum() or c in "-_") for c in part)
            for part in parts
        )
        words.append("[redacted token]" if looks_like_jwt else word)
    return " ".join(words)


def redact(text: str) -> str:
    return _strip_jwt(_strip_bearer(_strip_pem(text)))


@dataclass(frozen=True)
class CertificateFacts:
    purpose: Purpose
    issuer: IssuerId | None = None
    not_after: datetime | None = None
    fingerprint: str | None = None
    trust_bundle: tuple[IssuerId, ...] = ()
    rotating: IssueReason | None = None
    consecutive_failures: int = 0

    @staticmethod
    def from_action(
        purpose: Purpose,
        issuer: IssuerId | None,
        not_after: datetime | None,
        fingerprint: str | None,
        trust_bundle: tuple[IssuerId, ...],
        consecutive_failures: int,
        action: Action,
    ) -> CertificateFacts:
        rotating = action.reason if isinstance(action, Issue) else None
        short = short_fingerprint(fingerprint) if fingerprint is not None else None
        return CertificateFacts(
            purpose,
            issuer,
            not_after,
            short,
            trust_bundle,
            rotating,
            consecutive_failures,
        )

    def _ready_message(self) -> str:
        parts: list[str] = []
        if self.issuer is not None:
            parts.append(f"issuer {self.issuer.value}")
        if self.fingerprint is not None:
            parts.append(f"leaf {self.fingerprint}")
        if self.not_after is not None:
            parts.append(f"expires {self.not_after.isoformat()}")
        if self.trust_bundle:
            parts.append("trusting " + ", ".join(i.value for i in self.trust_bundle))
        return "; ".join(parts)

    def conditions(self) -> tuple[ConditionFact, ConditionFact]:
        prefix = condition_prefix(self.purpose)
        ready_type = prefix + READY_CONDITION
        rotating_type = prefix + ROTATING_CONDITION

        ready = self.issuer is not None and self.not_after is not None

        if ready:
            status = ConditionStatus.TRUE
            reason = "Issued"
            message = redact(self._ready_message())
        elif self.consecutive_failures > 0:
            status = ConditionStatus.FALSE
            reason = "IssuanceFailing"
            message = redact(
                "no "
                + self.purpose.token
                + " certificate projected after "
                + str(self.consecutive_failures)
                + " consecutive attempts"
            )
        else:
            status = ConditionStatus.FALSE
            reason = "Pending"
            message = redact("no " + self.purpose.token + " certificate projected yet")

        ready_fact = ConditionFact(
            type_=ready_type,
            status=status,
            reason=reason,
            message=message,
        )

        if self.rotating is not None:
            rot_status = ConditionStatus.TRUE
            rot_reason = rotation_reason(self.rotating)
            rot_message = redact(
                "issuing a new "
                + self.purpose.token
                + " certificate: "
                + rotation_detail(self.rotating)
            )
        else:
            rot_status = ConditionStatus.FALSE
            rot_reason = "Stable"
            rot_message = ""

        rotating_fact = ConditionFact(
            type_=rotating_type,
            status=rot_status,
            reason=rot_reason,
            message=rot_message,
        )

        return (ready_fact, rotating_fact)
