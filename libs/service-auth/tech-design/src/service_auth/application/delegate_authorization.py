"""Application service for delegated Kubernetes authorization."""

from __future__ import annotations

import hashlib
from dataclasses import dataclass, field
from enum import Enum
from typing import Sequence

from service_auth.domain.cache_policy import CacheOutcome, CachePolicy, classify
from service_auth.domain.review import (
    ResourceAttributes,
    TokenReviewOutcome,
)
from service_auth.domain.service_account import (
    PrincipalRejection,
    ServiceAccountRef,
    principal_from_review,
)
from service_auth.infrastructure.ports import (
    Clock,
    DelegatedBackendError,
    ReviewBackend,
)


class MissingAudienceError(Exception):
    """Raised when delegated authorization configuration names no audience."""


@dataclass(frozen=True)
class DelegatedAuthConfig:
    audiences: tuple[str, ...]
    policy: CachePolicy


def make_config(
    audiences: Sequence[str], policy: CachePolicy
) -> DelegatedAuthConfig:
    """Construct DelegatedAuthConfig, enforcing non-empty audience specification."""
    if not audiences or all(len(aud.strip()) == 0 for aud in audiences):
        raise MissingAudienceError("DelegatedAuthConfig requires at least one non-empty audience")
    return DelegatedAuthConfig(tuple(audiences), policy)


class AuthRejection(str, Enum):
    AUDIENCE_MISMATCH = "audience_mismatch"


class DelegatedOutcome(str, Enum):
    AUTHENTICATED = "authenticated"
    UNAUTHENTICATED = "unauthenticated"
    DENIED = "denied"
    UNAVAILABLE = "unavailable"


def judge(
    config: DelegatedAuthConfig, outcome: TokenReviewOutcome
) -> ServiceAccountRef | AuthRejection | PrincipalRejection:
    """Judge token review outcome in strict contract order: authenticated -> audience -> identity shape."""
    if not outcome.authenticated:
        return PrincipalRejection.NOT_AUTHENTICATED

    intersects = any(
        granted in config.audiences for granted in outcome.audiences
    )
    if not intersects:
        return AuthRejection.AUDIENCE_MISMATCH

    return principal_from_review(True, outcome.identity)


def fingerprint(token: str) -> str:
    """Compute 12-char hex digest of token SHA-256 (6 bytes)."""
    return hashlib.sha256(token.encode("utf-8")).hexdigest()[:12]


@dataclass
class DelegatedCache:
    policy: CachePolicy
    entries: dict[tuple[str, ResourceAttributes], tuple[bool, int]] = field(
        default_factory=dict
    )

    def get(
        self, token: str, attributes: ResourceAttributes, now: int
    ) -> bool | None:
        fp = fingerprint(token)
        key = (fp, attributes)
        if key not in self.entries:
            return None
        allowed, stored_at = self.entries[key]
        outcome = classify(self.policy, stored_at, now, allowed)
        if outcome == CacheOutcome.HIT:
            return allowed
        return None

    def get_stale(
        self, token: str, attributes: ResourceAttributes, now: int
    ) -> bool | None:
        fp = fingerprint(token)
        key = (fp, attributes)
        if key not in self.entries:
            return None
        allowed, stored_at = self.entries[key]
        outcome = classify(self.policy, stored_at, now, allowed)
        if outcome in (CacheOutcome.HIT, CacheOutcome.STALE):
            return allowed
        return None

    def put(
        self, token: str, attributes: ResourceAttributes, allowed: bool, now: int
    ) -> None:
        fp = fingerprint(token)
        key = (fp, attributes)
        if key not in self.entries and len(self.entries) >= self.policy.max_entries:
            oldest_key = next(iter(self.entries))
            del self.entries[oldest_key]
        self.entries[key] = (allowed, now)


def authorize_delegated(
    config: DelegatedAuthConfig,
    backend: ReviewBackend,
    clock: Clock,
    cache: DelegatedCache,
    token: str,
    attributes: ResourceAttributes,
) -> DelegatedOutcome:
    """Perform delegated authorization with cache-aside and stale-on-outage fallbacks."""
    if token == "":
        return DelegatedOutcome.UNAUTHENTICATED

    now = clock.now_seconds()
    cached = cache.get(token, attributes, now)
    if cached is not None:
        return DelegatedOutcome.AUTHENTICATED if cached else DelegatedOutcome.DENIED

    try:
        review = backend.review_token(token, config.audiences)
    except DelegatedBackendError:
        stale = cache.get_stale(token, attributes, now)
        if stale is not None:
            return DelegatedOutcome.AUTHENTICATED if stale else DelegatedOutcome.DENIED
        return DelegatedOutcome.UNAVAILABLE

    verdict = judge(config, review)
    if isinstance(verdict, (AuthRejection, PrincipalRejection)):
        return DelegatedOutcome.UNAUTHENTICATED

    try:
        access = backend.review_access(review.identity, attributes)
    except DelegatedBackendError:
        return DelegatedOutcome.UNAVAILABLE

    allowed = access.is_allowed()
    cache.put(token, attributes, allowed, now)
    return DelegatedOutcome.AUTHENTICATED if allowed else DelegatedOutcome.DENIED
