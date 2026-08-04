from __future__ import annotations

from dataclasses import dataclass

from service_http.infrastructure.routes import (
    BODY_OK,
    DOCS_PATH,
    HEALTHZ_PATH,
    METRICS_PATH,
    OPENAPI_PATH,
    READYZ_PATH,
    STATUS_OK,
    OpenapiSource,
    is_probe_path,
    metrics_response,
    openapi_content_type,
    readiness_response,
)

TEXT_CONTENT_TYPE = "text/plain; charset=utf-8"
HTML_CONTENT_TYPE = "text/html; charset=utf-8"


@dataclass(frozen=True)
class ProbeResponse:
    status: int
    content_type: str
    body: str


@dataclass(frozen=True)
class ProbeState:
    draining: bool
    metrics_text: str | None
    openapi_document: str
    openapi_source: OpenapiSource
    docs_html: str


def handle_probe(state: ProbeState, path: str) -> ProbeResponse | None:
    if not is_probe_path(path):
        return None
    if path == HEALTHZ_PATH:
        return ProbeResponse(STATUS_OK, TEXT_CONTENT_TYPE, BODY_OK)
    if path == READYZ_PATH:
        status, body = readiness_response(state.draining)
        return ProbeResponse(status, TEXT_CONTENT_TYPE, body)
    if path == METRICS_PATH:
        status, ctype, body = metrics_response(state.metrics_text)
        return ProbeResponse(status, ctype, body)
    if path == OPENAPI_PATH:
        ctype = openapi_content_type(state.openapi_source)
        return ProbeResponse(STATUS_OK, ctype, state.openapi_document)
    if path == DOCS_PATH:
        return ProbeResponse(STATUS_OK, HTML_CONTENT_TYPE, state.docs_html)
    return None
