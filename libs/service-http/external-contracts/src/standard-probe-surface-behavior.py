from __future__ import annotations

from service_http.application.probes import HTML_CONTENT_TYPE, ProbeState, TEXT_CONTENT_TYPE, handle_probe
from service_http.infrastructure.headers import CONTENT_TYPE_HEADER
from service_http.infrastructure.routes import DOCS_PATH, HEALTHZ_PATH, JSON_CONTENT_TYPE, METRICS_CONTENT_TYPE, METRICS_PATH, OPENAPI_PATH, OpenapiSource, PROBE_PATHS, READYZ_PATH, probe_routes

MINIMUM_CHECKS = 13

STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX = (
    ("the_surface_is_exactly_five_named_paths",
     (('/healthz', '/readyz', '/metrics', '/openapi.json', '/docs'), 5, 5)),
    ("every_probe_route_is_auth_exempt_and_body_limit_exempt",
     (('/healthz', False, False), ('/readyz', False, False), ('/metrics', False, False), ('/openapi.json', False, False), ('/docs', False, False))),
    ("the_liveness_probe_answers_ok_and_a_drain_does_not_move_it",
     (200, 'text/plain; charset=utf-8', 'ok', 200)),
    ("the_readiness_probe_answers_ok_while_serving",
     ('/readyz', 200, 'text/plain; charset=utf-8', 'ok')),
    ("the_readiness_probe_answers_503_once_a_drain_begins",
     (503, 'text/plain; charset=utf-8', 'draining')),
    ("the_metrics_probe_renders_the_provider_text",
     (200, 'text/plain; version=0.0.4', 'up 1\n')),
    ("an_absent_metrics_provider_is_a_200_with_an_empty_body",
     (200, 'text/plain; version=0.0.4', '')),
    ("the_document_probe_serves_json_from_the_typed_source",
     (200, 'application/json', '{"openapi":"3.1.0"}')),
    ("the_canonical_source_serves_the_producers_own_bytes",
     (200, 'application/json', '{"openapi":"3.1.0"}', True)),
    ("the_docs_probe_serves_html",
     (200, 'text/html; charset=utf-8', '<html>docs</html>')),
    ("a_data_plane_path_is_not_a_probe_response",
     (None, None, None, 'ProbeResponse')),
    ("the_content_types_are_the_documented_ones",
     ('text/plain; charset=utf-8', 'text/html; charset=utf-8', 'application/json', 'text/plain; version=0.0.4')),
    ("the_header_the_content_type_is_published_under_is_lower_case",
     ('content-type', True, 'text/plain; charset=utf-8')),
)


def plain(value: object) -> object:
    """A literal-shaped view: records by their fields, enum members by value.

    An expected value has to be a plain literal, and `repr` of a dataclass or
    an enum member is not one. Reading a record as the tuple of its fields
    keeps every field observable while staying transcribable.
    """
    fields = getattr(type(value), "__dataclass_fields__", None)
    if fields is not None:
        return tuple(plain(getattr(value, n)) for n in fields)
    if getattr(type(value), "__members__", None) is not None:
        return plain(value.value)
    if isinstance(value, tuple):
        return tuple(plain(v) for v in value)
    if isinstance(value, list):
        return [plain(v) for v in value]
    if isinstance(value, dict):
        return {k: plain(v) for k, v in value.items()}
    return value


def variant(value: object) -> str:
    """The name of the returned variant — the shape of an error-as-value."""
    return type(value).__name__


DOC = '{"openapi":"3.1.0"}'


DOCS_HTML = "<html>docs</html>"


READY = ProbeState(False, None, DOC, OpenapiSource.TYPED, DOCS_HTML)


DRAINING = ProbeState(True, None, DOC, OpenapiSource.TYPED, DOCS_HTML)


SCRAPED = ProbeState(False, "up 1\n", DOC, OpenapiSource.TYPED, DOCS_HTML)


CANONICAL = ProbeState(False, None, DOC, OpenapiSource.CANONICAL_JSON, DOCS_HTML)


def verify_standard_probe_surface_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the surface is exactly five named paths
    exp1 = STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[0][1]
    obs1 = plain((PROBE_PATHS, len(PROBE_PATHS), len(set(PROBE_PATHS))))
    checks.append({"name": STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. every probe route is auth exempt and body limit exempt
    exp2 = STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[1][1]
    obs2 = plain(tuple((r.path, r.requires_auth, r.enforces_body_limit)
        for r in probe_routes()))
    checks.append({"name": STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the liveness probe answers ok and a drain does not move it
    exp3 = STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[2][1]
    healthz = handle_probe(READY, HEALTHZ_PATH)
    obs3 = plain((healthz.status, healthz.content_type, healthz.body,
        handle_probe(DRAINING, HEALTHZ_PATH).status))
    checks.append({"name": STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the readiness probe answers ok while serving
    exp4 = STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[3][1]
    ready = handle_probe(READY, READYZ_PATH)
    obs4 = plain((READYZ_PATH, ready.status, ready.content_type, ready.body))
    checks.append({"name": STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the readiness probe answers 503 once a drain begins
    exp5 = STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[4][1]
    draining = handle_probe(DRAINING, READYZ_PATH)
    obs5 = plain((draining.status, draining.content_type, draining.body))
    checks.append({"name": STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the metrics probe renders the provider text
    exp6 = STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[5][1]
    scraped = handle_probe(SCRAPED, METRICS_PATH)
    obs6 = plain((scraped.status, scraped.content_type, scraped.body))
    checks.append({"name": STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. an absent metrics provider is a 200 with an empty body
    exp7 = STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[6][1]
    unscraped = handle_probe(READY, METRICS_PATH)
    obs7 = plain((unscraped.status, unscraped.content_type, unscraped.body))
    checks.append({"name": STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the document probe serves json from the typed source
    exp8 = STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[7][1]
    typed = handle_probe(READY, OPENAPI_PATH)
    obs8 = plain((typed.status, typed.content_type, typed.body))
    checks.append({"name": STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the canonical source serves the producers own bytes
    exp9 = STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[8][1]
    canonical = handle_probe(CANONICAL, OPENAPI_PATH)
    obs9 = plain((canonical.status, canonical.content_type, canonical.body,
        canonical.body == DOC))
    checks.append({"name": STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the docs probe serves html
    exp10 = STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[9][1]
    docs = handle_probe(READY, DOCS_PATH)
    obs10 = plain((docs.status, docs.content_type, docs.body))
    checks.append({"name": STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a data plane path is not a probe response
    exp11 = STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[10][1]
    obs11 = plain((handle_probe(READY, "/api/v1/things"), handle_probe(READY, "/"),
        handle_probe(READY, ""), variant(handle_probe(READY, HEALTHZ_PATH))))
    checks.append({"name": STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. the content types are the documented ones
    exp12 = STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[11][1]
    obs12 = plain((TEXT_CONTENT_TYPE, HTML_CONTENT_TYPE, JSON_CONTENT_TYPE,
        METRICS_CONTENT_TYPE))
    checks.append({"name": STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. the header the content type is published under is lower case
    exp13 = STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[12][1]
    obs13 = plain((CONTENT_TYPE_HEADER, CONTENT_TYPE_HEADER == CONTENT_TYPE_HEADER.lower(),
        handle_probe(READY, HEALTHZ_PATH).content_type))
    checks.append({"name": STANDARD_PROBE_SURFACE_BEHAVIOR_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    return {
        "case_id": "standard-probe-surface-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
