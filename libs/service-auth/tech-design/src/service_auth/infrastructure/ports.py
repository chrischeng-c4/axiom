"""Infrastructure port definitions and error types."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol, Sequence

from service_auth.domain.audit import AuthEvent
from service_auth.domain.review import (
    AccessReviewOutcome,
    ResourceAttributes,
    ReviewError,
    TokenReviewOutcome,
)
from service_auth.domain.service_account import ReviewedIdentity


class RegistrySource(Protocol):
    name: str

    def read(self) -> str: ...


class ReviewBackend(Protocol):
    def review_token(
        self, token: str, audiences: Sequence[str]
    ) -> TokenReviewOutcome: ...

    def review_access(
        self, identity: ReviewedIdentity, attributes: ResourceAttributes
    ) -> AccessReviewOutcome: ...


class Clock(Protocol):
    def now_seconds(self) -> int: ...


class AuthEventSink(Protocol):
    def record(self, event: AuthEvent) -> None: ...


@dataclass
class DelegatedBackendError(Exception):
    error: ReviewError
