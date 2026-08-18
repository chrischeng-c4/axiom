from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final
from urllib.parse import urlsplit

from service_observability.domain.identity import ServiceIdentity
from service_observability.infrastructure.config import ObservabilityConfig

OTLP_SCHEMES: Final[frozenset[str]] = frozenset({"http", "https"})


class OtelFallback(Enum):
    FEATURE_DISABLED = "feature_disabled"
    INVALID_ENDPOINT = "invalid_endpoint"


@dataclass(frozen=True)
class LoggingOnly:
    """No endpoint was configured. Not a failure."""


@dataclass(frozen=True)
class Otel:
    endpoint: str
    identity: ServiceIdentity


@dataclass(frozen=True)
class OtelUnavailable:
    endpoint: str
    reason: OtelFallback


TracingMode = LoggingOnly | Otel | OtelUnavailable


def valid_otlp_endpoint(endpoint: str) -> bool:
    parts = urlsplit(endpoint)
    return parts.scheme in OTLP_SCHEMES and parts.netloc != ""


def tracing_mode(
    config: ObservabilityConfig,
    identity: ServiceIdentity,
    otlp_feature_enabled: bool,
) -> TracingMode:
    if config.otlp_endpoint is None:
        return LoggingOnly()
    endpoint = config.otlp_endpoint
    if not valid_otlp_endpoint(endpoint):
        return OtelUnavailable(endpoint, OtelFallback.INVALID_ENDPOINT)
    if otlp_feature_enabled:
        return Otel(endpoint, identity)
    return OtelUnavailable(endpoint, OtelFallback.FEATURE_DISABLED)
