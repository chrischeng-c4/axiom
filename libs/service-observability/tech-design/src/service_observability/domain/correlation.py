from __future__ import annotations

import unicodedata
from typing import Final, Mapping

from service_observability.domain.bounds import MAX_REQUEST_ID_BYTES, JsonValue
from service_observability.domain.text import byte_len

REQUEST_ID_KEYS: Final[tuple[str, str, str]] = (
    "request_id",
    "request.id",
    "http.request.id",
)


def valid_lower_hex(value: str, expected_len: int, reject_zero: bool) -> bool:
    if byte_len(value) != expected_len:
        return False
    for c in value:
        if c not in "0123456789abcdef":
            return False
    if reject_zero and all(c == "0" for c in value):
        return False
    return True


def valid_request_id(value: str) -> bool:
    if value == "":
        return False
    if byte_len(value) > MAX_REQUEST_ID_BYTES:
        return False
    if any(unicodedata.category(c).startswith("C") for c in value):
        return False
    return True


def field_string(fields: Mapping[str, JsonValue], key: str) -> str | None:
    value = fields.get(key)
    return value if isinstance(value, str) else None


def preferred_hex(
    event_fields: Mapping[str, JsonValue],
    span_fields: Mapping[str, JsonValue],
    key: str,
    expected_len: int,
    reject_zero: bool,
) -> str | None:
    for fields in (event_fields, span_fields):
        candidate = field_string(fields, key)
        if candidate is not None and valid_lower_hex(
            candidate, expected_len, reject_zero
        ):
            return candidate
    return None


def preferred_request_id(
    event_fields: Mapping[str, JsonValue],
    span_fields: Mapping[str, JsonValue],
) -> str | None:
    for fields in (event_fields, span_fields):
        for key in REQUEST_ID_KEYS:
            candidate = field_string(fields, key)
            if candidate is not None and valid_request_id(candidate):
                return candidate
    return None
