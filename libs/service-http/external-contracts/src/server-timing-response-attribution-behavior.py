from __future__ import annotations

from service_http.application.server_timing import PhaseCollector
from service_http.domain.timing import DEFAULT_DISCLOSURE, Disclosure, Phase, drains_phases, reveals_phases
from service_http.infrastructure.headers import SERVER_TIMING_HEADER
from service_http.infrastructure.timing_header import format_ms, render_header, render_metric

MINIMUM_CHECKS = 12

SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX = (
    ("the_baseline_metric_is_always_present",
     ('app;dur=1.500', 'app;dur=0.000', 'app;dur=1.500')),
    ("a_duration_renders_as_milliseconds_with_three_places",
     ('1.500', '0.000', '0.000', '1000.000')),
    ("the_default_posture_hides_a_pushed_phase",
     ('app;dur=5.000', 'total-only', 1)),
    ("full_disclosure_reveals_the_phases_after_the_baseline",
     ('app;dur=5.000, db;dur=2.000, render;dur=1.000', 'app;dur=5.000', 3)),
    ("the_phases_render_in_push_order",
     ((('first', 1000000), ('second', 2000000)), ['app;dur=3.000', 'first;dur=1.000', 'second;dur=2.000'])),
    ("a_full_render_drains_what_it_disclosed",
     (1, 'app;dur=1.000, db;dur=1.000', 0, 'app;dur=1.000')),
    ("the_two_postures_are_told_apart_by_one_predicate_each",
     (True, False, True, False)),
    ("a_metric_renders_as_a_token_and_a_duration",
     ('db;dur=2.000', 'render;dur=0.000', 'app;dur=0.001')),
    ("a_response_with_no_phases_still_carries_the_baseline",
     ((), 'app;dur=1.000', 'app;dur=1.000')),
    ("a_phase_carries_its_own_name_and_duration",
     (('db', 5), True, 'app;dur=0.000, db;dur=2.000')),
    ("the_posture_values_are_the_documented_ones",
     ('total-only', 'full', 'total-only', True)),
    ("the_header_the_attribution_is_published_under_is_lower_case",
     ('server-timing', True, 1)),
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


def verify_server_timing_response_attribution_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the baseline metric is always present
    exp1 = SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[0][1]
    obs1 = plain((render_header(1_500_000, Disclosure.TOTAL_ONLY, ()),
        render_header(0, Disclosure.TOTAL_ONLY, ()),
        render_header(1_500_000, Disclosure.FULL, ())))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a duration renders as milliseconds with three places
    exp2 = SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[1][1]
    obs2 = plain((format_ms(1_500_000), format_ms(0), format_ms(1),
        format_ms(1_000_000_000)))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the default posture hides a pushed phase
    exp3 = SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[2][1]
    hidden = PhaseCollector()
    hidden.push("db", 2_000_000)
    obs3 = plain((hidden.render(5_000_000, Disclosure.TOTAL_ONLY),
        DEFAULT_DISCLOSURE, len(hidden.pending())))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. full disclosure reveals the phases after the baseline
    exp4 = SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[3][1]
    shown = PhaseCollector()
    shown.push("db", 2_000_000)
    shown.push("render", 1_000_000)
    shown_full = shown.render(5_000_000, Disclosure.FULL)
    obs4 = plain((shown_full, shown_full.split(", ")[0], len(shown_full.split(", "))))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the phases render in push order
    exp5 = SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[4][1]
    ordered = PhaseCollector()
    ordered.push("first", 1_000_000)
    ordered.push("second", 2_000_000)
    obs5 = plain((ordered.pending(),
        ordered.render(3_000_000, Disclosure.FULL).split(", ")))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a full render drains what it disclosed
    exp6 = SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[5][1]
    drained = PhaseCollector()
    drained.push("db", 1_000_000)
    obs6 = plain((len(drained.pending()), drained.render(1_000_000, Disclosure.FULL),
        len(drained.pending()), drained.render(1_000_000, Disclosure.FULL)))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the two postures are told apart by one predicate each
    exp7 = SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[6][1]
    obs7 = plain((reveals_phases(Disclosure.FULL), reveals_phases(Disclosure.TOTAL_ONLY),
        drains_phases(Disclosure.FULL), drains_phases(Disclosure.TOTAL_ONLY)))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a metric renders as a token and a duration
    exp8 = SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[7][1]
    obs8 = plain((render_metric("db", 2_000_000), render_metric("render", 0),
        render_metric("app", 1_000)))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a response with no phases still carries the baseline
    exp9 = SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[8][1]
    empty = PhaseCollector()
    obs9 = plain((empty.pending(), empty.render(1_000_000, Disclosure.FULL),
        empty.render(1_000_000, Disclosure.TOTAL_ONLY)))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a phase carries its own name and duration
    exp10 = SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[9][1]
    obs10 = plain((Phase("db", 5), Phase("db", 5) == Phase("db", 5),
        render_header(0, Disclosure.FULL, (Phase("db", 2_000_000),))))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the posture values are the documented ones
    exp11 = SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[10][1]
    obs11 = plain((Disclosure.TOTAL_ONLY.value, Disclosure.FULL.value,
        DEFAULT_DISCLOSURE.value, DEFAULT_DISCLOSURE is Disclosure.TOTAL_ONLY))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. the header the attribution is published under is lower case
    exp12 = SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[11][1]
    obs12 = plain((SERVER_TIMING_HEADER,
        SERVER_TIMING_HEADER == SERVER_TIMING_HEADER.lower(),
        SERVER_TIMING_HEADER.count("-")))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "server-timing-response-attribution-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
