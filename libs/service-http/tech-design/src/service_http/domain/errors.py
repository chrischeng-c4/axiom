from __future__ import annotations

from dataclasses import dataclass

RATE_LIMITED = "rate_limited"
PAYLOAD_TOO_LARGE = "payload_too_large"
RATE_LIMITED_MESSAGE = "request admission limit exceeded"
PAYLOAD_TOO_LARGE_MESSAGE = "request body exceeds the configured size limit"
STATUS_PAYLOAD_TOO_LARGE = 413
STATUS_TOO_MANY_REQUESTS = 429


@dataclass(frozen=True)
class ErrorEnvelope:
    error: str
    message: str


@dataclass(frozen=True)
class ApiError:
    status: int
    kind: str
    message: str


def envelope_of(error: ApiError) -> ErrorEnvelope:
    return ErrorEnvelope(error=error.kind, message=error.message)


def envelope_fields(envelope: ErrorEnvelope) -> tuple[tuple[str, str], ...]:
    return (("error", envelope.error), ("message", envelope.message))


def rate_limited() -> ApiError:
    return ApiError(STATUS_TOO_MANY_REQUESTS, RATE_LIMITED, RATE_LIMITED_MESSAGE)


def payload_too_large() -> ApiError:
    return ApiError(
        STATUS_PAYLOAD_TOO_LARGE, PAYLOAD_TOO_LARGE, PAYLOAD_TOO_LARGE_MESSAGE
    )


@dataclass(frozen=True)
class InvalidValue:
    key: str
    value: str


@dataclass(frozen=True)
class OrphanedCommonSetting:
    key: str


@dataclass(frozen=True)
class InvalidPolicy:
    route_class: str
    reason: str


AdmissionConfigError = InvalidValue | OrphanedCommonSetting | InvalidPolicy


def describe(error: AdmissionConfigError) -> str:
    if isinstance(error, InvalidValue):
        return f"{error.key} must be a positive integer, got `{error.value}`"
    if isinstance(error, OrphanedCommonSetting):
        return (
            f"{error.key} is set but no admission capacity is configured; set at"
            f" least one capacity key or remove {error.key}"
        )
    if isinstance(error, InvalidPolicy):
        return (
            f"admission policy for class `{error.route_class}` is invalid:"
            f" {error.reason}"
        )
    raise TypeError(f"Unhandled error type: {type(error)}")
