from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta
from enum import Enum
from typing import Final, Union

from service_k8s.domain.digest import hex_sha256
from service_k8s.domain.profile import CertificateProfile


@dataclass(frozen=True, order=True)
class IssuerId:
    value: str


@dataclass(frozen=True)
class ObservedLeaf:
    issuer: IssuerId
    not_before: datetime
    not_after: datetime
    fingerprint: str
    identity_digest: str


@dataclass(frozen=True)
class Observed:
    leaf: ObservedLeaf | None = None
    trust_bundle: tuple[IssuerId, ...] = ()
    activated_fingerprint: str | None = None
    consecutive_failures: int = 0


@dataclass(frozen=True)
class Desired:
    profile: CertificateProfile
    issuer: IssuerId


class IssueReason(Enum):
    BOOTSTRAP = "Bootstrap"
    RENEWAL = "Renewal"
    EXPIRED = "Expired"
    IDENTITY_CHANGED = "IdentityChanged"
    ISSUER_ROTATION = "IssuerRotation"

    @property
    def token(self) -> str:
        return self.value


@dataclass(frozen=True)
class PublishTrustBundle:
    issuers: tuple[IssuerId, ...]


@dataclass(frozen=True)
class Issue:
    issuer: IssuerId
    reason: IssueReason


@dataclass(frozen=True)
class AwaitActivation:
    fingerprint: str
    recheck_after: timedelta


@dataclass(frozen=True)
class RetireIssuers:
    issuers: tuple[IssuerId, ...]


@dataclass(frozen=True)
class Wait:
    until: datetime


Action = Union[PublishTrustBundle, Issue, AwaitActivation, RetireIssuers, Wait]

ACTIVATION_RECHECK: Final[timedelta] = timedelta(seconds=15)
RETRY_BASE_SECS: Final[int] = 5
RETRY_CEILING_SECS: Final[int] = 300


def next_action(desired: Desired, observed: Observed, now: datetime) -> Action:
    if desired.issuer not in observed.trust_bundle:
        issuers = sorted(set(observed.trust_bundle) | {desired.issuer})
        return PublishTrustBundle(tuple(issuers))

    if observed.leaf is None:
        return Issue(desired.issuer, IssueReason.BOOTSTRAP)

    if observed.leaf.issuer != desired.issuer:
        return Issue(desired.issuer, IssueReason.ISSUER_ROTATION)

    if observed.leaf.identity_digest != desired.profile.identity_digest():
        return Issue(desired.issuer, IssueReason.IDENTITY_CHANGED)

    if now >= observed.leaf.not_after:
        return Issue(desired.issuer, IssueReason.EXPIRED)

    stale = tuple(i for i in observed.trust_bundle if i != desired.issuer)
    if stale:
        if observed.activated_fingerprint == observed.leaf.fingerprint:
            return RetireIssuers(stale)
        return AwaitActivation(observed.leaf.fingerprint, ACTIVATION_RECHECK)

    due = renew_at(desired.profile, observed.leaf)
    if now >= due:
        return Issue(desired.issuer, IssueReason.RENEWAL)
    return Wait(due)


def _fingerprint_offset(fingerprint: str, span: int) -> int:
    digest = hex_sha256(fingerprint.encode("utf-8"))
    acc = 0
    for ch in digest.encode("ascii")[:16]:
        acc = (acc * 31 + ch) % 18446744073709551616
    return acc % span


def renew_at(profile: CertificateProfile, leaf: ObservedLeaf) -> datetime:
    base = leaf.not_after - timedelta(seconds=profile.renew_before_secs)
    if profile.renew_jitter_secs == 0:
        return base
    offset = _fingerprint_offset(leaf.fingerprint, profile.renew_jitter_secs)
    return base + timedelta(seconds=offset)


def retry_after(failures: int) -> timedelta:
    if failures == 0:
        return timedelta(seconds=RETRY_BASE_SECS)
    shift = min(failures, 6)
    secs = min(RETRY_BASE_SECS * (2**shift), RETRY_CEILING_SECS)
    return timedelta(seconds=secs)
