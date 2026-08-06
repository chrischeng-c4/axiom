"""In-memory ReviewBackend adapter for unit testing."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Sequence

from service_auth.domain.review import (
    AccessReviewOutcome,
    ResourceAttributes,
    ReviewError,
    TokenReviewOutcome,
)
from service_auth.domain.service_account import ReviewedIdentity
from service_auth.infrastructure.ports import DelegatedBackendError


@dataclass
class MemoryReviewBackend:
    tokens: dict[str, TokenReviewOutcome] = field(default_factory=dict)
    access: dict[tuple[ReviewedIdentity, ResourceAttributes], AccessReviewOutcome] = field(
        default_factory=dict
    )
    token_error: ReviewError | None = None
    access_error: ReviewError | None = None
    review_calls: int = 0

    def review_token(
        self, token: str, audiences: Sequence[str]
    ) -> TokenReviewOutcome:
        self.review_calls += 1
        if self.token_error is not None:
            raise DelegatedBackendError(self.token_error)
        if token in self.tokens:
            return self.tokens[token]
        raise DelegatedBackendError(ReviewError.MALFORMED)

    def review_access(
        self, identity: ReviewedIdentity, attributes: ResourceAttributes
    ) -> AccessReviewOutcome:
        self.review_calls += 1
        if self.access_error is not None:
            raise DelegatedBackendError(self.access_error)
        if (identity, attributes) in self.access:
            return self.access[(identity, attributes)]
        raise DelegatedBackendError(ReviewError.MALFORMED)
