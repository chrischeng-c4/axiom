from __future__ import annotations

from enum import Enum

from service_http.domain.errors import (
    STATUS_PAYLOAD_TOO_LARGE,
    ApiError,
    payload_too_large,
)
from service_http.infrastructure.headers import content_length_exceeds

DEFAULT_BODY_LIMIT_BYTES = 8 * 1024 * 1024


class BodyOutcome(str, Enum):
    PASS = "pass"
    REJECTED_DECLARED = "rejected-declared"
    REJECTED_STREAMED = "rejected-streamed"


def classify(
    content_length: str | None,
    streamed_bytes: int,
    max_bytes: int,
) -> BodyOutcome:
    if content_length_exceeds(content_length, max_bytes):
        return BodyOutcome.REJECTED_DECLARED
    if streamed_bytes > max_bytes:
        return BodyOutcome.REJECTED_STREAMED
    return BodyOutcome.PASS


def rewrite_status(status: int) -> ApiError | None:
    if status == STATUS_PAYLOAD_TOO_LARGE:
        return payload_too_large()
    return None
