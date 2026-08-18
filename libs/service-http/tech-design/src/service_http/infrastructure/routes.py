from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

HEALTHZ_PATH = "/healthz"
READYZ_PATH = "/readyz"
METRICS_PATH = "/metrics"
OPENAPI_PATH = "/openapi.json"
DOCS_PATH = "/docs"
PROBE_PATHS = (
    HEALTHZ_PATH,
    READYZ_PATH,
    METRICS_PATH,
    OPENAPI_PATH,
    DOCS_PATH,
)

METRICS_CONTENT_TYPE = "text/plain; version=0.0.4"
JSON_CONTENT_TYPE = "application/json"
BODY_OK = "ok"
BODY_DRAINING = "draining"
STATUS_OK = 200
STATUS_UNAVAILABLE = 503
LEVEL_INFO = "info"
LEVEL_DEBUG = "debug"


class OpenapiSource(str, Enum):
    TYPED = "typed"
    CANONICAL_JSON = "canonical-json"


@dataclass(frozen=True)
class RouteSpec:
    path: str
    requires_auth: bool
    enforces_body_limit: bool


def probe_routes() -> tuple[RouteSpec, ...]:
    return tuple(
        RouteSpec(
            path=path,
            requires_auth=False,
            enforces_body_limit=False,
        )
        for path in PROBE_PATHS
    )


def is_probe_path(path: str) -> bool:
    return path in PROBE_PATHS


def access_log_level(path: str) -> str:
    return LEVEL_DEBUG if is_probe_path(path) else LEVEL_INFO


def readiness_response(draining: bool) -> tuple[int, str]:
    if draining:
        return (STATUS_UNAVAILABLE, BODY_DRAINING)
    return (STATUS_OK, BODY_OK)


def metrics_response(rendered: str | None) -> tuple[int, str, str]:
    return (
        STATUS_OK,
        METRICS_CONTENT_TYPE,
        rendered if rendered is not None else "",
    )


def openapi_content_type(source: OpenapiSource) -> str:
    return JSON_CONTENT_TYPE
