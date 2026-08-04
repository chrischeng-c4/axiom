"""Domain authorization principals and enforcement."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from service_auth.domain.claims import TokenClaims, resolve_role
from service_auth.domain.role import Role, covers


class AuthorizationOutcome(str, Enum):
    ALLOW = "allow"
    DENY = "deny"


class DenialReason(str, Enum):
    MISSING_BEARER = "missing_bearer"
    UNKNOWN_BEARER = "unknown_bearer"
    INSUFFICIENT_ROLE = "insufficient_role"


@dataclass(frozen=True)
class OpenPrincipal:
    """Auth not required and none presented."""


@dataclass(frozen=True)
class TokenPrincipal:
    claims: TokenClaims


RoleMapPrincipal = OpenPrincipal | TokenPrincipal


@dataclass(frozen=True)
class Denial:
    reason: DenialReason
    subject: str | None
    resource: str
    needed: Role


def ensure(
    principal: RoleMapPrincipal, resource: str, needed: Role
) -> Denial | None:
    """Ensure principal has sufficient role on resource or return Denial."""
    if isinstance(principal, OpenPrincipal):
        return None
    if isinstance(principal, TokenPrincipal):
        held = resolve_role(principal.claims, resource)
        if held is None or not covers(held, needed):
            return Denial(
                reason=DenialReason.INSUFFICIENT_ROLE,
                subject=principal.claims.subject,
                resource=resource,
                needed=needed,
            )
        return None
    return None
