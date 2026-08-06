from __future__ import annotations

from service_http.infrastructure.routes import LEVEL_DEBUG, PROBE_PATHS, access_log_level, is_probe_path, probe_routes

MINIMUM_CHECKS = 10

PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX = (
    ("a_near_miss_path_is_not_demoted",
     (('/healthz', 'debug'), ('/healthzx', 'info'), ('/xhealthz', 'info'), ('/healthz.', 'info'))),
    ("a_nested_path_cannot_borrow_a_probes_quiet_level",
     (('/api/healthz', 'info'), ('/healthz/sub', 'info'), ('//healthz', 'info'), ('/./healthz', 'info'))),
    ("a_query_string_form_is_not_demoted",
     (('/healthz?x=1', 'info'), ('/metrics?x', 'info'), ('/docs#a', 'info'))),
    ("case_folding_cannot_reach_the_quiet_level",
     (('/HEALTHZ', 'info'), ('/Readyz', 'info'), ('/METRICS', 'info'), ('/Docs', 'info'))),
    ("an_empty_or_root_path_is_not_demoted",
     (('', 'info'), ('/', 'info'), (' ', 'info'), ('/ healthz', 'info'))),
    ("exactly_five_paths_are_ever_demoted",
     (5, 5, 5)),
    ("a_scrape_loop_cannot_be_promoted_by_a_trailing_byte",
     ('debug', 'info', 'info', 'info')),
    ("the_demotion_never_hides_a_data_plane_request",
     (('/api/v1/x', 'info'), ('/admin', 'info'), ('/openapi.yaml', 'info'), ('/health', 'info'))),
    ("the_membership_test_and_the_level_cannot_disagree",
     ((True, True), (False, False), (True, True), (False, False))),
    ("levelling_is_total_and_still_picks_a_level",
     ('accepted', 'info', 'accepted', 'debug', 'accepted', True, 'accepted', 5)),
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


def refusal(fn, *args, **kwargs) -> str:
    """The exception type a refusing call raises, or 'accepted' if it returns."""
    try:
        fn(*args, **kwargs)
    except Exception as exc:  # noqa: BLE001 — the type is the observation
        return type(exc).__name__
    return "accepted"


def verify_probe_demoted_access_log_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a near miss path is not demoted
    exp1 = PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[0][1]
    obs1 = plain(tuple((p, access_log_level(p)) for p in
        ("/healthz", "/healthzx", "/xhealthz", "/healthz.")))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a nested path cannot borrow a probes quiet level
    exp2 = PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[1][1]
    obs2 = plain(tuple((p, access_log_level(p)) for p in
        ("/api/healthz", "/healthz/sub", "//healthz", "/./healthz")))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a query string form is not demoted
    exp3 = PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[2][1]
    obs3 = plain(tuple((p, access_log_level(p)) for p in
        ("/healthz?x=1", "/metrics?x", "/docs#a")))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. case folding cannot reach the quiet level
    exp4 = PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[3][1]
    obs4 = plain(tuple((p, access_log_level(p)) for p in
        ("/HEALTHZ", "/Readyz", "/METRICS", "/Docs")))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. an empty or root path is not demoted
    exp5 = PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[4][1]
    obs5 = plain(tuple((p, access_log_level(p)) for p in
        ("", "/", " ", "/ healthz")))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. exactly five paths are ever demoted
    exp6 = PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[5][1]
    obs6 = plain((len(PROBE_PATHS), len(set(PROBE_PATHS)),
        len({p for p in PROBE_PATHS if access_log_level(p) == LEVEL_DEBUG})))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a scrape loop cannot be promoted by a trailing byte
    exp7 = PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[6][1]
    obs7 = plain((access_log_level("/metrics"), access_log_level("/metrics "),
        access_log_level(" /metrics"), access_log_level("/metrics\n")))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the demotion never hides a data plane request
    exp8 = PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[7][1]
    obs8 = plain(tuple((p, access_log_level(p)) for p in
        ("/api/v1/x", "/admin", "/openapi.yaml", "/health")))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the membership test and the level cannot disagree
    exp9 = PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[8][1]
    obs9 = plain(tuple((is_probe_path(p), access_log_level(p) == LEVEL_DEBUG)
        for p in ("/healthz", "/healthzx", "/metrics", "/api")))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. levelling is total and still picks a level
    exp10 = PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[9][1]
    obs10 = plain((refusal(access_log_level, ""), access_log_level(""),
        refusal(access_log_level, "/healthz"), access_log_level("/healthz"),
        refusal(is_probe_path, "/healthz"), is_probe_path("/healthz"),
        refusal(probe_routes), len(probe_routes())))
    checks.append({"name": PROBE_DEMOTED_ACCESS_LOG_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "probe-demoted-access-log-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
