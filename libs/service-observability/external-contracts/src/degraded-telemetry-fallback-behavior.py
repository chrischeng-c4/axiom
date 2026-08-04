from __future__ import annotations

from service_observability.application.telemetry import (
    LoggingOnly,
    OTLP_SCHEMES,
    Otel,
    OtelFallback,
    OtelUnavailable,
    tracing_mode,
    valid_otlp_endpoint,
)
from service_observability.domain.identity import make_identity
from service_observability.infrastructure.config import (
    LogFormat,
    ObservabilityConfig,
    collector_compatible,
)

MINIMUM_CHECKS = 11

DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX = (
    ("no_configured_endpoint_is_logging_only_not_a_failure", (('LoggingOnly',), ('LoggingOnly',), None)),
    ("a_valid_endpoint_with_the_exporter_compiled_in_exports", ('Otel', 'http://collector:4317', 'lumen', '1.2.3')),
    ("the_resolved_export_mode_carries_the_service_identity", ('Otel', 'https://otel.example:443/v1/traces', 'lumen', '1.2.3')),
    ("a_valid_endpoint_without_a_compiled_exporter_is_a_fallback", (('OtelUnavailable', 'http://collector:4317', 'feature_disabled'), 'feature_disabled')),
    ("both_http_and_https_are_accepted_schemes", (('http', 'https'), True, True)),
    ("the_configuration_round_trips_its_settings_unchanged", ('debug', 'pretty', 'http://c:4317')),
    ("the_configuration_defaults_are_the_documented_ones", ('info', 'json', None)),
    ("the_configuration_says_nothing_about_how_traffic_is_served", ('log_format', 'log_level', 'otlp_endpoint')),
    ("the_two_fallback_reasons_are_the_published_enumeration", (('feature_disabled', 'invalid_endpoint'), ('FEATURE_DISABLED', 'INVALID_ENDPOINT'))),
    ("the_two_log_formats_are_the_published_enumeration", (('json', 'pretty'), True, False)),
    ("resolving_is_pure_so_the_same_config_resolves_the_same_twice", (True, True)),
)

IDENT = make_identity("lumen", "1.2.3")


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


def verify_degraded_telemetry_fallback_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. no configured endpoint is logging-only, not a failure
    exp1 = DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[0][1]
    obs1 = (mode_of(None, True), mode_of(None, False), ObservabilityConfig().otlp_endpoint)
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a valid endpoint with the exporter compiled in resolves to export
    exp2 = DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[1][1]
    obs2 = mode_of('http://collector:4317', True)
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the resolved export mode carries the service identity with it
    exp3 = DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[2][1]
    obs3 = mode_of('https://otel.example:443/v1/traces', True)
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a valid endpoint without a compiled exporter is a named fallback
    exp4 = DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[3][1]
    obs4 = (mode_of('http://collector:4317', False), OtelFallback.FEATURE_DISABLED.value)
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. both http and https are accepted schemes
    exp5 = DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[4][1]
    obs5 = (tuple(sorted(OTLP_SCHEMES)), valid_otlp_endpoint('http://c:4317'), valid_otlp_endpoint('https://c'))
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the configuration round-trips its level, format and endpoint unchanged
    exp6 = DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[5][1]
    cfg = ObservabilityConfig("debug", LogFormat.PRETTY, "http://c:4317")
    obs6 = (cfg.log_level, cfg.log_format.value, cfg.otlp_endpoint)
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the configuration defaults are the documented ones
    exp7 = DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[6][1]
    obs7 = (ObservabilityConfig().log_level, ObservabilityConfig().log_format.value, ObservabilityConfig().otlp_endpoint)
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the configuration says nothing about how the service is served
    exp8 = DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[7][1]
    obs8 = tuple(sorted(ObservabilityConfig().__dataclass_fields__))
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the two fallback reasons are the published enumeration
    exp9 = DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[8][1]
    obs9 = (tuple((f.value for f in OtelFallback)), tuple((f.name for f in OtelFallback)))
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the two log formats are the published enumeration
    exp10 = DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[9][1]
    obs10 = (tuple((f.value for f in LogFormat)), collector_compatible(LogFormat.JSON), collector_compatible(LogFormat.PRETTY))
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. resolving is pure, so the same config resolves the same way twice
    exp11 = DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[10][1]
    obs11 = (mode_of('http://c:4317', True) == mode_of('http://c:4317', True), mode_of('bad', True) == mode_of('bad', True))
    checks.append({"name": DEGRADED_TELEMETRY_FALLBACK_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "degraded-telemetry-fallback-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
