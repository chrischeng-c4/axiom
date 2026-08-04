from __future__ import annotations

from service_http.infrastructure.numbers import parse_ascii_unsigned

TRACEPARENT_HEADER = "traceparent"
CONTENT_LENGTH_HEADER = "content-length"
RETRY_AFTER_HEADER = "retry-after"
SERVER_TIMING_HEADER = "server-timing"
CONTENT_TYPE_HEADER = "content-type"
DEFAULT_RETRY_AFTER_NS = 1_000_000_000
NANOS_PER_SECOND = 1_000_000_000


def content_length_exceeds(raw: str | None, max_bytes: int) -> bool:
    declared = parse_ascii_unsigned(raw)
    if declared is None:
        return False
    return declared > max_bytes


def retry_after_seconds(retry_after_ns: int | None) -> int:
    total = (
        retry_after_ns if retry_after_ns is not None else DEFAULT_RETRY_AFTER_NS
    )
    seconds = total // NANOS_PER_SECOND
    if total % NANOS_PER_SECOND > 0:
        seconds += 1
    return seconds if seconds > 1 else 1


def retry_after_value(retry_after_ns: int | None) -> str:
    return str(retry_after_seconds(retry_after_ns))
