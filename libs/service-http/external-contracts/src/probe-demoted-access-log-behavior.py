from __future__ import annotations

from service_http.infrastructure.routes import DOCS_PATH, HEALTHZ_PATH, LEVEL_DEBUG, LEVEL_INFO, METRICS_PATH, OPENAPI_PATH, PROBE_PATHS, READYZ_PATH, access_log_level, is_probe_path, probe_routes

MINIMUM_CHECKS = 10

PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX = (
    ("each_of_the_five_probes_logs_at_the_quiet_level",
     ('debug', 'debug', 'debug', 'debug', 'debug')),
    ("a_data_plane_path_logs_at_the_operator_level",
     ('info', 'info', 'info', 'info')),
    ("the_two_levels_are_the_documented_tokens",
     ('info', 'debug', False)),
    ("the_demotion_reads_the_same_membership_test_the_router_does",
     ('debug', True, 'info', False)),
    ("the_demoted_set_is_exactly_the_probe_tuple",
     (('/healthz', '/readyz', '/metrics', '/openapi.json', '/docs'), ('/healthz', '/readyz', '/metrics', '/openapi.json', '/docs'))),
    ("the_route_table_and_the_demotion_agree",
     (('/healthz', 'debug'), ('/readyz', 'debug'), ('/metrics', 'debug'), ('/openapi.json', 'debug'), ('/docs', 'debug'))),
    ("the_scrape_driven_probes_are_the_ones_that_would_flood",
     ('debug', 'debug', 'debug', 'info')),
    ("the_document_and_docs_probes_are_demoted_too",
     ('debug', 'debug', 'info')),
    ("one_event_level_is_returned_for_every_path",
     (1, 1, True)),
    ("the_level_is_a_plain_lower_case_token",
     (True, True, False, 5)),
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


def verify_probe_demoted_access_log_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. each of the five probes logs at the quiet level
    exp1 = PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[0][1]
    obs1 = plain(tuple(access_log_level(p) for p in PROBE_PATHS))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a data plane path logs at the operator level
    exp2 = PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[1][1]
    obs2 = plain((access_log_level("/api/v1/things"), access_log_level("/"),
        access_log_level(""), access_log_level("/admin/backup")))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the two levels are the documented tokens
    exp3 = PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[2][1]
    obs3 = plain((LEVEL_INFO, LEVEL_DEBUG, LEVEL_INFO == LEVEL_DEBUG))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the demotion reads the same membership test the router does
    exp4 = PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[3][1]
    obs4 = plain((access_log_level(HEALTHZ_PATH), is_probe_path(HEALTHZ_PATH),
        access_log_level("/api"), is_probe_path("/api")))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the demoted set is exactly the probe tuple
    exp5 = PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[4][1]
    obs5 = plain((tuple(p for p in PROBE_PATHS if access_log_level(p) == LEVEL_DEBUG),
        PROBE_PATHS))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the route table and the demotion agree
    exp6 = PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[5][1]
    obs6 = plain(tuple((r.path, access_log_level(r.path)) for r in probe_routes()))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the scrape driven probes are the ones that would flood
    exp7 = PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[6][1]
    obs7 = plain((access_log_level(HEALTHZ_PATH), access_log_level(READYZ_PATH),
        access_log_level(METRICS_PATH), access_log_level("/metrics/x")))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the document and docs probes are demoted too
    exp8 = PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[7][1]
    obs8 = plain((access_log_level(OPENAPI_PATH), access_log_level(DOCS_PATH),
        access_log_level("/openapi.yaml")))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. one event level is returned for every path
    exp9 = PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[8][1]
    obs9 = plain((len({access_log_level(p) for p in PROBE_PATHS}),
        len({access_log_level(p) for p in ("/a", "/b", "/c")}),
        access_log_level("/a") == access_log_level("/b")))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the level is a plain lower case token
    exp10 = PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[9][1]
    obs10 = plain((LEVEL_INFO.lower() == LEVEL_INFO, LEVEL_DEBUG.lower() == LEVEL_DEBUG,
        " " in LEVEL_INFO, len(LEVEL_DEBUG)))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "probe-demoted-access-log-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
