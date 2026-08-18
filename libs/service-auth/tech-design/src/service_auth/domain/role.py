"""Domain role hierarchy and coverage rules."""

from __future__ import annotations

from enum import Enum


class Role(str, Enum):
    READ = "read"
    WRITE = "write"
    ADMIN = "admin"


ROLE_ORDER: tuple[Role, ...] = (Role.READ, Role.WRITE, Role.ADMIN)


def rank(role: Role) -> int:
    """Return 0-based rank index of role in ROLE_ORDER."""
    return ROLE_ORDER.index(role)


def covers(held: Role, needed: Role) -> bool:
    """Return True if held role meets or exceeds needed role."""
    return rank(held) >= rank(needed)
