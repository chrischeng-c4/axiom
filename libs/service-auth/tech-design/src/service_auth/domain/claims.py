"""Domain token claims and resource role resolution."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Mapping

from service_auth.domain.role import Role

WILDCARD_RESOURCE: str = "*"


@dataclass(frozen=True)
class TokenClaims:
    subject: str
    roles: Mapping[str, Role]


def resolve_role(claims: TokenClaims, resource: str) -> Role | None:
    """Resolve granted role for resource, preferring exact match over wildcard."""
    if resource in claims.roles:
        return claims.roles[resource]
    if WILDCARD_RESOURCE in claims.roles:
        return claims.roles[WILDCARD_RESOURCE]
    return None
