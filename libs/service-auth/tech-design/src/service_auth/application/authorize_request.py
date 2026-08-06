"""Application service for static role map authorization."""

from __future__ import annotations

from dataclasses import dataclass

from service_auth.domain.audit import AuthorizationEvent
from service_auth.domain.principal import (
    AuthorizationOutcome,
    DenialReason,
    OpenPrincipal,
    RoleMapPrincipal,
    TokenPrincipal,
    ensure,
)
from service_auth.domain.registry import Registry, lookup_identity, lookup_secret
from service_auth.domain.role import Role
from service_auth.infrastructure.ports import AuthEventSink


@dataclass(frozen=True)
class AuthorizeRequest:
    registry: Registry
    auth_required: bool


def principal_for_bearer(
    svc: AuthorizeRequest, secret: str | None
) -> RoleMapPrincipal | DenialReason:
    """Resolve a bearer token secret into a principal or denial reason."""
    if secret is None:
        if not svc.auth_required:
            return OpenPrincipal()
        return DenialReason.MISSING_BEARER
    claims = lookup_secret(svc.registry, secret)
    if claims is None:
        return DenialReason.UNKNOWN_BEARER
    return TokenPrincipal(claims)


def principal_for_identity(
    svc: AuthorizeRequest, identity: str | None
) -> RoleMapPrincipal | DenialReason:
    """Resolve a verified public identity into a principal or denial reason."""
    if identity is None:
        if not svc.auth_required:
            return OpenPrincipal()
        return DenialReason.MISSING_BEARER
    claims = lookup_identity(svc.registry, identity)
    if claims is None:
        return DenialReason.UNKNOWN_BEARER
    return TokenPrincipal(claims)


def authorize(
    svc: AuthorizeRequest,
    principal: RoleMapPrincipal,
    resource: str,
    needed: Role,
    sink: AuthEventSink,
) -> AuthorizationOutcome:
    """Authorize a request principal against resource/needed role and record event."""
    denial = ensure(principal, resource, needed)
    if denial is None:
        subject = principal.claims.subject if isinstance(principal, TokenPrincipal) else None
        event = AuthorizationEvent(
            outcome=AuthorizationOutcome.ALLOW,
            reason=None,
            subject=subject,
            resource=resource,
            needed=needed,
        )
        sink.record(event)
        return AuthorizationOutcome.ALLOW

    event = AuthorizationEvent(
        outcome=AuthorizationOutcome.DENY,
        reason=denial.reason,
        subject=denial.subject,
        resource=resource,
        needed=needed,
    )
    sink.record(event)
    return AuthorizationOutcome.DENY
