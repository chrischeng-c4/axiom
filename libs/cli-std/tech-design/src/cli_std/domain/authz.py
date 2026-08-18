from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from enum import Enum


class Role(Enum):
    READ = 1
    WRITE = 2
    ADMIN = 3

    def covers(self, needed: Role) -> bool:
        return self.value >= needed.value


@dataclass(frozen=True)
class TokenClaims:
    subject: str
    roles: Mapping[str, Role]


def select_token(
    registry: Mapping[str, TokenClaims], role: Role, resource: str | None
) -> str | None:
    for token, claims in registry.items():
        granted = claims.roles.get(resource) if resource is not None else None
        if granted is None:
            granted = claims.roles.get("*")
        if granted is not None and granted.covers(role):
            return token
    return None
