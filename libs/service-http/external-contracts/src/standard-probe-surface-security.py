from __future__ import annotations

from service_http.application.probes import ProbeState, handle_probe
from service_http.infrastructure.routes import HEALTHZ_PATH, METRICS_PATH, OpenapiSource, PROBE_PATHS, READYZ_PATH, is_probe_path, metrics_response, openapi_content_type, probe_routes, readiness_response

MINIMUM_CHECKS = 11

STANDARD_PROBE_SURFACE_SECURITY_MATRIX = (
    ("readiness_answers_both_directions_from_the_drain_flag",
     ((200, 'ok'), (503, 'draining'))),
    ("a_draining_pod_cannot_report_ready_on_any_probe_path",
     (503, 200, 200, 'draining')),
    ("the_probe_path_test_is_exact_not_a_prefix",
     (True, False, False, False, False)),
    ("the_probe_path_test_is_case_sensitive",
     (False, False, False, True)),
    ("a_query_string_or_empty_path_is_not_a_probe",
     (False, False, False, False, True)),
    ("an_unhandled_path_yields_nothing_rather_than_a_default_200",
     (None, None, None, None, 'ProbeResponse')),
    ("no_probe_route_ever_requires_auth_or_enforces_the_body_cap",
     ((False, False, False, False, False), (False, False, False, False, False), 5)),
    ("the_metrics_body_is_never_a_missing_value",
     ((200, 'text/plain; version=0.0.4', ''), (200, 'text/plain; version=0.0.4', ''), (200, 'text/plain; version=0.0.4', 'x'))),
    ("the_document_content_type_does_not_vary_with_the_source",
     ('application/json', 'application/json', True)),
    ("the_route_table_and_the_path_tuple_cannot_drift_apart",
     (True, (True, True, True, True, True))),
    ("a_probe_lookup_is_total_and_still_answers",
     ('accepted', 'accepted', True, 'accepted', False, 'accepted', (503, 'draining'))),
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


def refusal(fn, *args, **kwargs) -> str:
    """The exception type a refusing call raises, or 'accepted' if it returns."""
    try:
        fn(*args, **kwargs)
    except Exception as exc:  # noqa: BLE001 — the type is the observation
        return type(exc).__name__
    return "accepted"


DOC = '{"openapi":"3.1.0"}'


DOCS_HTML = "<html>docs</html>"


READY = ProbeState(False, None, DOC, OpenapiSource.TYPED, DOCS_HTML)


DRAINING = ProbeState(True, None, DOC, OpenapiSource.TYPED, DOCS_HTML)


def verify_standard_probe_surface_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. readiness answers both directions from the drain flag
    exp1 = STANDARD_PROBE_SURFACE_SECURITY_MATRIX[0][1]
    obs1 = plain((readiness_response(False), readiness_response(True)))
    checks.append({"name": STANDARD_PROBE_SURFACE_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a draining pod cannot report ready on any probe path
    exp2 = STANDARD_PROBE_SURFACE_SECURITY_MATRIX[1][1]
    obs2 = plain((handle_probe(DRAINING, READYZ_PATH).status,
        handle_probe(DRAINING, HEALTHZ_PATH).status,
        handle_probe(DRAINING, METRICS_PATH).status,
        handle_probe(DRAINING, READYZ_PATH).body))
    checks.append({"name": STANDARD_PROBE_SURFACE_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the probe path test is exact not a prefix
    exp3 = STANDARD_PROBE_SURFACE_SECURITY_MATRIX[2][1]
    obs3 = plain((is_probe_path("/healthz"), is_probe_path("/healthzz"),
        is_probe_path("/healthz/"), is_probe_path("/api/healthz"),
        is_probe_path("healthz")))
    checks.append({"name": STANDARD_PROBE_SURFACE_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the probe path test is case sensitive
    exp4 = STANDARD_PROBE_SURFACE_SECURITY_MATRIX[3][1]
    obs4 = plain((is_probe_path("/HEALTHZ"), is_probe_path("/Readyz"),
        is_probe_path("/Metrics"), is_probe_path("/healthz")))
    checks.append({"name": STANDARD_PROBE_SURFACE_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a query string or empty path is not a probe
    exp5 = STANDARD_PROBE_SURFACE_SECURITY_MATRIX[4][1]
    obs5 = plain((is_probe_path("/healthz?x=1"), is_probe_path(""),
        is_probe_path("/"), is_probe_path("//healthz"), is_probe_path("/docs")))
    checks.append({"name": STANDARD_PROBE_SURFACE_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. an unhandled path yields nothing rather than a default 200
    exp6 = STANDARD_PROBE_SURFACE_SECURITY_MATRIX[5][1]
    obs6 = plain((handle_probe(READY, "/HEALTHZ"), handle_probe(READY, "/healthz/"),
        handle_probe(READY, "/admin"), handle_probe(READY, "/healthz?x=1"),
        variant(handle_probe(READY, "/healthz"))))
    checks.append({"name": STANDARD_PROBE_SURFACE_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. no probe route ever requires auth or enforces the body cap
    exp7 = STANDARD_PROBE_SURFACE_SECURITY_MATRIX[6][1]
    obs7 = plain((tuple(r.requires_auth for r in probe_routes()),
        tuple(r.enforces_body_limit for r in probe_routes()),
        len(probe_routes())))
    checks.append({"name": STANDARD_PROBE_SURFACE_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the metrics body is never a missing value
    exp8 = STANDARD_PROBE_SURFACE_SECURITY_MATRIX[7][1]
    obs8 = plain((metrics_response(None), metrics_response(""), metrics_response("x")))
    checks.append({"name": STANDARD_PROBE_SURFACE_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the document content type does not vary with the source
    exp9 = STANDARD_PROBE_SURFACE_SECURITY_MATRIX[8][1]
    obs9 = plain((openapi_content_type(OpenapiSource.TYPED),
        openapi_content_type(OpenapiSource.CANONICAL_JSON),
        openapi_content_type(OpenapiSource.TYPED)
        == openapi_content_type(OpenapiSource.CANONICAL_JSON)))
    checks.append({"name": STANDARD_PROBE_SURFACE_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the route table and the path tuple cannot drift apart
    exp10 = STANDARD_PROBE_SURFACE_SECURITY_MATRIX[9][1]
    obs10 = plain((tuple(r.path for r in probe_routes()) == PROBE_PATHS,
        tuple(is_probe_path(r.path) for r in probe_routes())))
    checks.append({"name": STANDARD_PROBE_SURFACE_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a probe lookup is total and still answers
    exp11 = STANDARD_PROBE_SURFACE_SECURITY_MATRIX[10][1]
    obs11 = plain((refusal(handle_probe, READY, "/healthz"),
        refusal(handle_probe, READY, ""), handle_probe(READY, "") is None,
        refusal(is_probe_path, ""), is_probe_path(""),
        refusal(readiness_response, True), readiness_response(True)))
    checks.append({"name": STANDARD_PROBE_SURFACE_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "standard-probe-surface-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
