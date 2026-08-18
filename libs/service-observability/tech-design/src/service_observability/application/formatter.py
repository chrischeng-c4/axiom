from __future__ import annotations

from dataclasses import dataclass
from typing import Mapping

from service_observability.domain.attributes import bounded_attributes
from service_observability.domain.bounds import (
    MAX_ATTRIBUTE_VALUE_BYTES,
    MAX_EVENT_BYTES,
    SERVICE_LOG_SCHEMA_V1,
    JsonValue,
)
from service_observability.domain.correlation import (
    field_string,
    preferred_hex,
    preferred_request_id,
)
from service_observability.domain.identity import ServiceIdentity
from service_observability.domain.text import truncate_utf8
from service_observability.infrastructure.envelope import LogEventV1, LogIdentityV1


@dataclass(frozen=True)
class EventMetadata:
    """What the tracing layer knows about the callsite, independent of fields."""

    name: str
    target: str
    severity: str


def preferred_string(
    event_fields: Mapping[str, JsonValue],
    span_fields: Mapping[str, JsonValue],
    key: str,
) -> str | None:
    for fields in (event_fields, span_fields):
        candidate = field_string(fields, key)
        if candidate is not None:
            return candidate
    return None


def resolve_event_name(
    event_fields: Mapping[str, JsonValue],
    span_fields: Mapping[str, JsonValue],
    metadata: EventMetadata,
) -> str:
    explicit = preferred_string(event_fields, span_fields, "event")
    if explicit is None or explicit == "":
        chosen = metadata.name
    else:
        chosen = explicit
    return truncate_utf8(chosen, MAX_EVENT_BYTES)


def resolve_message(
    event_fields: Mapping[str, JsonValue],
    event_name: str,
) -> str:
    candidate = field_string(event_fields, "message")
    if candidate is None:
        candidate = event_name
    return truncate_utf8(candidate, MAX_ATTRIBUTE_VALUE_BYTES)


def merge_attributes(
    event_fields: Mapping[str, JsonValue],
    span_fields: Mapping[str, JsonValue],
    metadata: EventMetadata,
) -> dict[str, JsonValue]:
    merged = dict(span_fields)
    merged.update(event_fields)
    if "target" not in merged:
        merged["target"] = metadata.target
    return bounded_attributes(merged)


def format_event(
    event_fields: Mapping[str, JsonValue],
    span_fields: Mapping[str, JsonValue],
    metadata: EventMetadata,
    identity: ServiceIdentity,
    timestamp: str,
) -> LogEventV1:
    event_name = resolve_event_name(event_fields, span_fields, metadata)
    return LogEventV1(
        schema=SERVICE_LOG_SCHEMA_V1,
        timestamp=timestamp,
        severity=metadata.severity,
        service=LogIdentityV1(
            name=truncate_utf8(identity.name, MAX_EVENT_BYTES),
            version=truncate_utf8(identity.version, MAX_EVENT_BYTES),
        ),
        event=event_name,
        message=resolve_message(event_fields, event_name),
        trace_id=preferred_hex(event_fields, span_fields, "trace_id", 32, True),
        span_id=preferred_hex(event_fields, span_fields, "span_id", 16, True),
        parent_span_id=preferred_hex(event_fields, span_fields, "parent_span_id", 16, True),
        trace_flags=preferred_hex(event_fields, span_fields, "trace_flags", 2, False),
        request_id=preferred_request_id(event_fields, span_fields),
        attributes=merge_attributes(event_fields, span_fields, metadata),
    )
