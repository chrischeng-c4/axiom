from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass

from cli_std.domain.authz import Role, TokenClaims, select_token
from cli_std.domain.registry import RegistryError, bearer_secrets
from cli_std.infrastructure.secret_data import (
    TOKEN_REGISTRY_SECRET_KEY,
    SecretError,
    secret_data_bytes,
)

SecretReader = Callable[[str, str], object | None]
JsonLoader = Callable[[bytes], object | None]


@dataclass(frozen=True)
class SecretNotFound:
    namespace: str
    name: str


@dataclass(frozen=True)
class UndecodableSecret:
    reason: str


ResolutionError = SecretNotFound | UndecodableSecret


def resolve_token(
    explicit_token: str | None,
    namespace: str | None,
    secret_name: str | None,
    role: Role,
    resource: str | None,
    read_secret: SecretReader,
    load_json: JsonLoader,
) -> str | None | ResolutionError:
    if explicit_token is not None:
        return explicit_token

    if namespace is None or secret_name is None:
        return None

    document_bytes = read_secret(namespace, secret_name)
    if document_bytes is None:
        return SecretNotFound(namespace, secret_name)

    raw = secret_data_bytes(document_bytes, TOKEN_REGISTRY_SECRET_KEY)
    if isinstance(raw, SecretError):
        return UndecodableSecret("secret decoding failed")

    parsed = load_json(raw)
    if parsed is None:
        return UndecodableSecret("not json")

    registry = bearer_secrets(parsed)
    if isinstance(registry, RegistryError):
        return UndecodableSecret("malformed registry")

    return select_token(registry, role, resource)


def uses_cluster(
    explicit_token: str | None,
    namespace: str | None,
    secret_name: str | None,
) -> bool:
    return (
        explicit_token is None
        and namespace is not None
        and secret_name is not None
    )
