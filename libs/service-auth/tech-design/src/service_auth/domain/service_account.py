"""Domain ServiceAccount identification and Kubernetes identity parsing."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Mapping, Sequence

SERVICE_ACCOUNT_PREFIX: str = "system:serviceaccount:"
UNAUTHENTICATED_GROUP: str = "system:unauthenticated"
ANONYMOUS_USERNAME: str = "system:anonymous"


class PrincipalRejection(str, Enum):
    NOT_AUTHENTICATED = "not_authenticated"
    MISSING_USERNAME = "missing_username"
    ANONYMOUS = "anonymous"
    NOT_A_SERVICE_ACCOUNT = "not_a_service_account"
    MALFORMED_SERVICE_ACCOUNT = "malformed_service_account"


def normalize_extra(
    source: Mapping[str, Sequence[str]]
) -> tuple[tuple[str, tuple[str, ...]], ...]:
    """Normalize extra mapping into key-sorted, tuple-of-tuples representation."""
    return tuple((key, tuple(source[key])) for key in sorted(source))


@dataclass(frozen=True)
class ReviewedIdentity:
    username: str
    uid: str
    groups: tuple[str, ...]
    extra: tuple[tuple[str, tuple[str, ...]], ...]


@dataclass(frozen=True)
class ServiceAccountRef:
    namespace: str
    name: str


def is_dns1123_label(text: str) -> bool:
    """Return True if text is a valid DNS-1123 label (1-63 chars, lowercase alnum/hyphen, starts/ends with alnum)."""
    if not (1 <= len(text) <= 63):
        return False

    def is_alnum(c: str) -> bool:
        return ("a" <= c <= "z") or ("0" <= c <= "9")

    if not (is_alnum(text[0]) and is_alnum(text[-1])):
        return False
    return all(is_alnum(c) or c == "-" for c in text)


def parse_service_account(username: str) -> ServiceAccountRef | PrincipalRejection:
    """Parse service account username into ServiceAccountRef or PrincipalRejection."""
    if username == "":
        return PrincipalRejection.MISSING_USERNAME
    if username == ANONYMOUS_USERNAME:
        return PrincipalRejection.ANONYMOUS
    if not username.startswith(SERVICE_ACCOUNT_PREFIX):
        return PrincipalRejection.NOT_A_SERVICE_ACCOUNT

    rest = username[len(SERVICE_ACCOUNT_PREFIX) :]
    segments = rest.split(":")
    if len(segments) != 2:
        return PrincipalRejection.MALFORMED_SERVICE_ACCOUNT

    ns, name = segments[0], segments[1]
    if not is_dns1123_label(ns) or not is_dns1123_label(name):
        return PrincipalRejection.MALFORMED_SERVICE_ACCOUNT

    return ServiceAccountRef(namespace=ns, name=name)


def principal_from_review(
    authenticated: bool, identity: ReviewedIdentity
) -> ServiceAccountRef | PrincipalRejection:
    """Derive ServiceAccountRef or PrincipalRejection from token review outcome."""
    if not authenticated:
        return PrincipalRejection.NOT_AUTHENTICATED
    if UNAUTHENTICATED_GROUP in identity.groups:
        return PrincipalRejection.ANONYMOUS
    return parse_service_account(identity.username)
