"""Application service for dynamic registry reloading and validation."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from typing import Mapping, Sequence

from service_auth.domain.audit import RegistryReloadEvent, ReloadFailure
from service_auth.domain.claims import TokenClaims
from service_auth.domain.registry import (
    Registry,
    RegistryError,
    parse,
    reserved_subject_violation,
    try_merge,
)
from service_auth.infrastructure.ports import AuthEventSink, RegistrySource


@dataclass
class ReloadableRegistry:
    revision: int = 0
    registry: Registry = field(default_factory=lambda: Registry(tokens={}, identities={}))
    auth_required: bool = False
    reserved: tuple[str, ...] = ()


def _validate_entries(entries: Mapping[str, TokenClaims]) -> str | None:
    for key, claims in entries.items():
        if key.strip() == "":
            return "empty_key"
        if claims.subject.strip() == "":
            return "empty_subject"
        if any(resource.strip() == "" for resource in claims.roles):
            return "empty_resource"
    return None


def validate(auth_required: bool, registry: Registry) -> str | None:
    """Validate registry contents against five ordered safety rules."""
    if auth_required and registry.is_empty():
        return "required_but_empty"

    reason = _validate_entries(registry.tokens)
    if reason is not None:
        return reason

    reason = _validate_entries(registry.identities)
    if reason is not None:
        return reason

    for key in registry.identities:
        if "@" not in key:
            return "identity_key_not_an_email"

    return None


def reload_documents(
    state: ReloadableRegistry,
    sources: Sequence[RegistrySource],
    sink: AuthEventSink,
) -> ReloadFailure | None:
    """Reload registry from sources in all-or-nothing transactional manner."""
    merged = Registry(tokens={}, identities={})
    for source in sources:
        try:
            text = source.read()
        except Exception:
            sink.record(
                RegistryReloadEvent(
                    applied=False,
                    revision=state.revision,
                    entries=0,
                    failure=ReloadFailure.READ,
                )
            )
            return ReloadFailure.READ

        try:
            doc = json.loads(text)
            candidate = parse(doc)
        except Exception:
            sink.record(
                RegistryReloadEvent(
                    applied=False,
                    revision=state.revision,
                    entries=0,
                    failure=ReloadFailure.PARSE,
                )
            )
            return ReloadFailure.PARSE

        try:
            merged = try_merge(merged, candidate)
        except RegistryError:
            sink.record(
                RegistryReloadEvent(
                    applied=False,
                    revision=state.revision,
                    entries=0,
                    failure=ReloadFailure.INVALID,
                )
            )
            return ReloadFailure.INVALID

    if validate(state.auth_required, merged) is not None:
        sink.record(
            RegistryReloadEvent(
                applied=False,
                revision=state.revision,
                entries=0,
                failure=ReloadFailure.INVALID,
            )
        )
        return ReloadFailure.INVALID

    if reserved_subject_violation(merged, state.reserved) is not None:
        sink.record(
            RegistryReloadEvent(
                applied=False,
                revision=state.revision,
                entries=0,
                failure=ReloadFailure.INVALID,
            )
        )
        return ReloadFailure.INVALID

    state.registry = merged
    state.revision += 1
    sink.record(
        RegistryReloadEvent(
            applied=True,
            revision=state.revision,
            entries=merged.len(),
            failure=None,
        )
    )
    return None
