"""Domain Token and Resource access review models."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from service_auth.domain.service_account import ReviewedIdentity


@dataclass(frozen=True)
class TokenReviewOutcome:
    authenticated: bool
    identity: ReviewedIdentity
    audiences: tuple[str, ...]


@dataclass(frozen=True)
class ResourceAttributes:
    group: str
    namespace: str
    resource: str
    name: str | None
    verb: str

    def describe(self) -> str:
        name_part = f"/{self.name}" if self.name else ""
        return f"{self.verb} {self.group}/{self.resource}{name_part} in {self.namespace}"


@dataclass(frozen=True)
class AccessReviewOutcome:
    allowed: bool
    denied: bool

    def is_allowed(self) -> bool:
        return self.allowed and not self.denied


class ReviewError(str, Enum):
    TRANSPORT = "transport"
    MALFORMED = "malformed_response"
    NOT_DELEGATED = "not_delegated"
