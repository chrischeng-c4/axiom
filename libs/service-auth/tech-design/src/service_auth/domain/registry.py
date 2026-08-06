"""Domain registry model, parsing, merging, and reserved subject validation."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Mapping, Sequence

from service_auth.domain.claims import TokenClaims
from service_auth.domain.role import Role

TOKENS_SECTION: str = "tokens"
IDENTITIES_SECTION: str = "identities"


@dataclass(frozen=True)
class Registry:
    tokens: Mapping[str, TokenClaims]
    identities: Mapping[str, TokenClaims]

    def len(self) -> int:
        return len(self.tokens) + len(self.identities)

    def is_empty(self) -> bool:
        return not self.tokens and not self.identities


def lookup_secret(registry: Registry, secret: str) -> TokenClaims | None:
    """Look up bearer secret claims from tokens section ONLY."""
    return registry.tokens.get(secret)


def lookup_identity(registry: Registry, identity: str) -> TokenClaims | None:
    """Look up verified identity claims from identities section ONLY."""
    return registry.identities.get(identity)


class RegistryError(Exception):
    """Exception raised during registry load or merge failure."""

    def __init__(self, reason: str) -> None:
        super().__init__(reason)
        self.reason = reason


def _parse_claims_map(raw_map: object) -> dict[str, TokenClaims]:
    if not isinstance(raw_map, Mapping):
        raise RegistryError("invalid_section_format")
    result: dict[str, TokenClaims] = {}
    for key, value in raw_map.items():
        if not isinstance(value, Mapping):
            raise RegistryError("invalid_claims_format")
        subject = value.get("subject")
        if not isinstance(subject, str):
            raise RegistryError("invalid_subject_format")
        roles_raw = value.get("roles", {})
        if not isinstance(roles_raw, Mapping):
            raise RegistryError("invalid_roles_format")
        roles: dict[str, Role] = {}
        for r_key, r_val in roles_raw.items():
            try:
                roles[r_key] = Role(r_val)
            except ValueError as err:
                raise RegistryError("invalid_role_value") from err
        result[key] = TokenClaims(subject=subject, roles=roles)
    return result


def parse(document: Mapping[str, object]) -> Registry:
    """Parse JSON document into namespaced or flat Registry."""
    if not isinstance(document, Mapping):
        raise RegistryError("invalid_document_type")

    namespaced = (
        all(key in (TOKENS_SECTION, IDENTITIES_SECTION) for key in document.keys())
        and not any(
            isinstance(val, Mapping) and "subject" in val
            for val in document.values()
        )
    )

    if namespaced:
        tokens_raw = document.get(TOKENS_SECTION, {})
        identities_raw = document.get(IDENTITIES_SECTION, {})
        tokens = _parse_claims_map(tokens_raw)
        identities = _parse_claims_map(identities_raw)
        return Registry(tokens=tokens, identities=identities)

    tokens = _parse_claims_map(document)
    return Registry(tokens=tokens, identities={})


def try_merge(base: Registry, other: Registry) -> Registry:
    """Merge base and other registries per namespace, rejecting key collisions."""
    merged_tokens = dict(base.tokens)
    for k, v in other.tokens.items():
        if k in merged_tokens:
            raise RegistryError("duplicate_registry_key")
        merged_tokens[k] = v

    merged_identities = dict(base.identities)
    for k, v in other.identities.items():
        if k in merged_identities:
            raise RegistryError("duplicate_registry_key")
        merged_identities[k] = v

    return Registry(tokens=merged_tokens, identities=merged_identities)


def reserved_subject_violation(
    registry: Registry, reserved: Sequence[str]
) -> tuple[str, str, str] | None:
    """Check for reserved subject violations, scanning tokens first then identities."""
    reserved_set = set(reserved)
    token_hits = [
        (key, claims.subject)
        for key, claims in registry.tokens.items()
        if claims.subject in reserved_set
    ]
    if token_hits:
        min_key, min_subj = min(token_hits)
        return (TOKENS_SECTION, min_key, min_subj)

    identity_hits = [
        (key, claims.subject)
        for key, claims in registry.identities.items()
        if claims.subject in reserved_set
    ]
    if identity_hits:
        min_key, min_subj = min(identity_hits)
        return (IDENTITIES_SECTION, min_key, min_subj)

    return None
