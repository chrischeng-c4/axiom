"""Domain security audit event schemas."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from service_auth.domain.principal import AuthorizationOutcome, DenialReason
from service_auth.domain.role import Role


class ReloadFailure(str, Enum):
    READ = "read"
    PARSE = "parse"
    INVALID = "invalid"


@dataclass(frozen=True)
class RegistryReloadEvent:
    applied: bool
    revision: int
    entries: int
    failure: ReloadFailure | None


@dataclass(frozen=True)
class AuthorizationEvent:
    outcome: AuthorizationOutcome
    reason: DenialReason | None
    subject: str | None
    resource: str
    needed: Role


AuthEvent = RegistryReloadEvent | AuthorizationEvent

AUTHORIZATION_EVENT_FIELDS: tuple[str, ...] = (
    "outcome",
    "reason",
    "subject",
    "resource",
    "needed",
)

REGISTRY_RELOAD_EVENT_FIELDS: tuple[str, ...] = (
    "applied",
    "revision",
    "entries",
    "failure",
)
