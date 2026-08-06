from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass

from cli_std.domain.authz import Role, TokenClaims


@dataclass(frozen=True)
class NotAnObject:
    pass


@dataclass(frozen=True)
class MalformedClaims:
    token: str
    reason: str


RegistryError = NotAnObject | MalformedClaims


def is_namespaced(document: Mapping[str, object]) -> bool:
    every_key_is_a_section = all(
        k == "tokens" or k == "identities" for k in document
    )
    some_value_is_a_claim = any(
        isinstance(v, Mapping) and "subject" in v for v in document.values()
    )
    return every_key_is_a_section and not some_value_is_a_claim


def role_from_name(name: str) -> Role | None:
    match name:
        case "read":
            return Role.READ
        case "write":
            return Role.WRITE
        case "admin":
            return Role.ADMIN
        case _:
            return None


def bearer_secrets(
    document: object,
) -> dict[str, TokenClaims] | RegistryError:
    if not isinstance(document, Mapping):
        return NotAnObject()

    section = (
        document.get("tokens", {})
        if is_namespaced(document)
        else document
    )
    if not isinstance(section, Mapping):
        return NotAnObject()

    res: dict[str, TokenClaims] = {}
    for token_key, val in section.items():
        if not isinstance(token_key, str):
            return MalformedClaims(str(token_key), "token key is not a string")
        if not isinstance(val, Mapping):
            return MalformedClaims(token_key, "claims entry is not a mapping")

        subject = val.get("subject")
        if not isinstance(subject, str):
            return MalformedClaims(token_key, "missing or non-string subject")

        raw_roles = val.get("roles", {})
        if not isinstance(raw_roles, Mapping):
            return MalformedClaims(token_key, "roles is not a mapping")

        parsed_roles: dict[str, Role] = {}
        for res_key, role_val in raw_roles.items():
            if not isinstance(res_key, str):
                return MalformedClaims(
                    token_key, "role resource key is not a string"
                )
            if isinstance(role_val, Role):
                parsed_roles[res_key] = role_val
            elif isinstance(role_val, str):
                r = role_from_name(role_val)
                if r is None:
                    return MalformedClaims(
                        token_key, f"invalid role name '{role_val}'"
                    )
                parsed_roles[res_key] = r
            else:
                return MalformedClaims(
                    token_key, "role value is neither a Role nor a valid string"
                )

        res[token_key] = TokenClaims(subject=subject, roles=parsed_roles)

    return res
