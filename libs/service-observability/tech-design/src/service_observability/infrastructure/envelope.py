from __future__ import annotations

import json
from dataclasses import dataclass, field
from typing import Final, Mapping

from service_observability.domain.bounds import JsonValue


@dataclass(frozen=True)
class LogIdentityV1:
    name: str
    version: str


@dataclass(frozen=True)
class LogEventV1:
    schema: str
    timestamp: str
    severity: str
    service: LogIdentityV1
    event: str
    message: str
    trace_id: str | None = None
    span_id: str | None = None
    parent_span_id: str | None = None
    trace_flags: str | None = None
    request_id: str | None = None
    attributes: Mapping[str, JsonValue] = field(default_factory=dict)


REQUIRED_ENVELOPE_KEYS: Final[tuple[str, ...]] = (
    "schema",
    "timestamp",
    "severity",
    "service",
    "event",
    "message",
)

OPTIONAL_ENVELOPE_KEYS: Final[tuple[str, ...]] = (
    "trace_id",
    "span_id",
    "parent_span_id",
    "trace_flags",
    "request_id",
)


def to_mapping(event: LogEventV1) -> dict[str, object]:
    """The serialized form. An absent correlation field is OMITTED, never null."""
    out: dict[str, object] = {
        "schema": event.schema,
        "timestamp": event.timestamp,
        "severity": event.severity,
        "service": {"name": event.service.name, "version": event.service.version},
        "event": event.event,
        "message": event.message,
    }
    for key in OPTIONAL_ENVELOPE_KEYS:
        value = getattr(event, key)
        if value is not None:
            out[key] = value
    out["attributes"] = dict(event.attributes)
    return out


def to_json_line(event: LogEventV1) -> str:
    """Exactly one line. No embedded newline may survive."""
    return json.dumps(to_mapping(event), separators=(",", ":"), ensure_ascii=False)
