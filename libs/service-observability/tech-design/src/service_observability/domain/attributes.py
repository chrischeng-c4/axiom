from __future__ import annotations

from typing import Final, Mapping

from service_observability.domain.bounds import (
    MAX_ATTRIBUTES,
    MAX_ATTRIBUTE_KEY_BYTES,
    MAX_ATTRIBUTE_VALUE_BYTES,
    JsonValue,
)
from service_observability.domain.text import truncate_utf8

RESERVED_KEYS: Final[frozenset[str]] = frozenset({
    "schema",
    "timestamp",
    "severity",
    "service",
    "event",
    "message",
    "trace_id",
    "span_id",
    "parent_span_id",
    "trace_flags",
    "request_id",
    "request.id",
    "http.request.id",
    "attributes",
})

SENSITIVE_KEYS: Final[tuple[str, ...]] = (
    "authorization",
    "proxy_authorization",
    "cookie",
    "set_cookie",
    "baggage",
    "tracestate",
)


def is_reserved_key(key: str) -> bool:
    return key in RESERVED_KEYS


def is_sensitive_key(key: str) -> bool:
    normalized = key.lower().replace("-", "_")
    for sensitive in SENSITIVE_KEYS:
        if normalized == sensitive:
            return True
        if normalized.endswith("." + sensitive):
            return True
        if normalized.endswith("/" + sensitive):
            return True
        if normalized.endswith("_" + sensitive):
            return True
    return False


def bounded_value(value: JsonValue) -> JsonValue:
    if isinstance(value, str):
        return truncate_utf8(value, MAX_ATTRIBUTE_VALUE_BYTES)
    if value is None or isinstance(value, (bool, int, float)):
        return value
    return truncate_utf8(str(value), MAX_ATTRIBUTE_VALUE_BYTES)


def bounded_attributes(values: Mapping[str, JsonValue]) -> dict[str, JsonValue]:
    result: dict[str, JsonValue] = {}
    for key in sorted(values):
        if len(result) == MAX_ATTRIBUTES:
            break
        if is_reserved_key(key) or is_sensitive_key(key):
            continue
        bounded_key = truncate_utf8(key, MAX_ATTRIBUTE_KEY_BYTES)
        if bounded_key == "":
            continue
        result[bounded_key] = bounded_value(values[key])
    return result
