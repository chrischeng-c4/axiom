from __future__ import annotations

from service_observability.application.telemetry import (
    LoggingOnly,
    Otel,
    OtelFallback,
    OtelUnavailable,
    tracing_mode,
    valid_otlp_endpoint,
)
from service_observability.domain.identity import make_identity
from service_observability.infrastructure.config import ObservabilityConfig

MINIMUM_CHECKS = 10

DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX = (
    ("an_endpoint_that_is_not_a_uri_is_an_invalid_endpoint_fallback", (('OtelUnavailable', 'not a url', 'invalid_endpoint'), 'invalid_endpoint')),
    ("a_scheme_other_than_http_or_https_is_refused", (False, False, False, ('OtelUnavailable', 'ftp://collector:4317', 'invalid_endpoint'))),
    ("an_authority_less_uri_is_refused_even_with_the_right_scheme", (False, False, ('OtelUnavailable', 'http:///v1/traces', 'invalid_endpoint'))),
    ("an_empty_or_relative_endpoint_is_refused", (False, False, False, ('OtelUnavailable', '', 'invalid_endpoint'))),
    ("the_scheme_comparison_is_case_folded_by_the_uri_split", (True, True, ('Otel', 'HTTP://collector:4317', 'lumen', '1.2.3'))),
    ("an_invalid_endpoint_is_refused_whether_or_not_compiled_in", (('OtelUnavailable', 'bad', 'invalid_endpoint'), ('OtelUnavailable', 'bad', 'invalid_endpoint'))),
    ("the_rejected_endpoint_is_reported_back_for_the_operator", ('htp://collector:4317', 'collector:4317')),
    ("no_unavailable_path_raises_so_a_bad_endpoint_cannot_fail_startup", ('accepted', 'accepted', 'accepted', 'accepted')),
    ("an_unset_endpoint_never_turns_into_a_dial", (('LoggingOnly',), True)),
    ("resolving_builds_no_connection_so_it_is_safe_on_a_dead_host", (('Otel', 'http://192.0.2.1:4317', 'lumen', '1.2.3'), ('OtelUnavailable', 'http://192.0.2.1:4317', 'feature_disabled'))),
)

IDENT = make_identity("lumen", "1.2.3")


def refusal(fn, *args, **kwargs) -> str:
    """The exception type a refusing call raises, or 'accepted' if it returns."""
    try:
        fn(*args, **kwargs)
    except Exception as exc:  # noqa: BLE001 — the type is the observation
        return type(exc).__name__
    return "accepted"


def mode_of(endpoint, enabled=True):
    m = tracing_mode(
        ObservabilityConfig(otlp_endpoint=endpoint), IDENT, enabled
    )
    if isinstance(m, LoggingOnly):
        return ("LoggingOnly",)
    if isinstance(m, Otel):
        return ("Otel", m.endpoint, m.identity.name, m.identity.version)
    if isinstance(m, OtelUnavailable):
        return ("OtelUnavailable", m.endpoint, m.reason.value)
    return ("unknown", type(m).__name__)


def verify_degraded_telemetry_fallback_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an endpoint that is not a URI at all is an invalid-endpoint fallback
    exp1 = DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[0][1]
    obs1 = (mode_of('not a url', True), OtelFallback.INVALID_ENDPOINT.value)
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a scheme other than http or https is refused before any dial
    exp2 = DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[1][1]
    obs2 = (valid_otlp_endpoint('ftp://collector:4317'), valid_otlp_endpoint('file:///etc/passwd'), valid_otlp_endpoint('grpc://collector:4317'), mode_of('ftp://collector:4317', True))
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. an authority-less URI is refused even with the right scheme
    exp3 = DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[2][1]
    obs3 = (valid_otlp_endpoint('http:///v1/traces'), valid_otlp_endpoint('https:'), mode_of('http:///v1/traces', True))
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an empty or relative endpoint is refused
    exp4 = DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[3][1]
    obs4 = (valid_otlp_endpoint(''), valid_otlp_endpoint('//collector:4317'), valid_otlp_endpoint('/v1/traces'), mode_of('', True))
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the scheme comparison is case-folded by the URI split, so upper-case works
    exp5 = DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[4][1]
    obs5 = (valid_otlp_endpoint('HTTP://collector:4317'), valid_otlp_endpoint('HttpS://collector'), mode_of('HTTP://collector:4317', True))
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. an invalid endpoint is refused whether or not the exporter is compiled
    exp6 = DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[5][1]
    obs6 = (mode_of('bad', True), mode_of('bad', False))
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the rejected endpoint is reported back so the operator can see the typo
    exp7 = DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[6][1]
    obs7 = (mode_of('htp://collector:4317', True)[1], mode_of('collector:4317', True)[1])
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. no unavailable path raises, so a bad endpoint cannot fail startup
    exp8 = DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[7][1]
    obs8 = (refusal(mode_of, 'bad', True), refusal(mode_of, '', False), refusal(mode_of, 'http:///x', True), refusal(mode_of, None, True))
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a configured endpoint never turns an unset one into a dial
    exp9 = DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[8][1]
    obs9 = (mode_of(None, True), valid_otlp_endpoint('http://collector:4317'))
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. resolving builds no connection, so it is safe on an unreachable host
    exp10 = DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[9][1]
    obs10 = (mode_of('http://192.0.2.1:4317', True), mode_of('http://192.0.2.1:4317', False))
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "degraded-telemetry-fallback-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
