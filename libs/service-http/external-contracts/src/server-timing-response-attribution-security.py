from __future__ import annotations

from service_http.application.server_timing import PhaseCollector
from service_http.domain.timing import Disclosure, Phase
from service_http.infrastructure.timing_header import format_ms, render_header, render_metric, sanitize_token

MINIMUM_CHECKS = 11

SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX = (
    ("a_separator_bearing_phase_name_cannot_forge_a_second_metric",
     ('db__forged', 'db_dur_0', 'a_b')),
    ("a_newline_in_a_phase_name_cannot_split_the_header",
     ('db__X-Evil__1', 'a_b', False)),
    ("a_non_ascii_phase_name_is_replaced_character_by_character",
     ('___', 'db_', '_')),
    ("an_empty_or_fully_replaced_name_falls_back_to_a_fixed_token",
     ('phase', '_', '__', 'phase;dur=0.000')),
    ("the_permitted_extra_characters_survive_verbatim",
     ('db_read', 'db-read', 'db.read', 'Db0')),
    ("the_default_posture_never_leaks_a_phase_name",
     (False, 'app;dur=1.000', 1)),
    ("a_hidden_phase_is_retained_and_can_surface_on_a_later_full_render",
     ('app;dur=1.000', 1, True)),
    ("a_full_render_prevents_the_next_response_from_repeating_it",
     ('app;dur=1.000, first_response;dur=1.000', 'app;dur=1.000', ())),
    ("the_baseline_name_cannot_be_pushed_over",
     ('app;dur=0.000, app;dur=0.000', 'app;dur=0.000')),
    ("a_negative_or_huge_duration_still_renders_a_bounded_token",
     ('-1.000', '0.000', '1000000000.000', 'db;dur=-0.000')),
    ("rendering_is_total_and_still_produces_a_header",
     ('accepted', 'phase', 'accepted', 'phase;dur=0.000', 'accepted', 'app;dur=0.000', 'accepted', '0.000')),
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


def verify_server_timing_response_attribution_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a separator bearing phase name cannot forge a second metric
    exp1 = SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[0][1]
    obs1 = plain((sanitize_token("db, forged"), sanitize_token("db;dur=0"),
        sanitize_token("a=b")))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a newline in a phase name cannot split the header
    exp2 = SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[1][1]
    obs2 = plain((sanitize_token("db\r\nX-Evil: 1"), sanitize_token("a\nb"),
        "\n" in render_metric("a\nb", 0)))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a non ascii phase name is replaced character by character
    exp3 = SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[2][1]
    obs3 = plain((sanitize_token("資料庫"), sanitize_token("dbé"),
        sanitize_token("１")))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an empty or fully replaced name falls back to a fixed token
    exp4 = SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[3][1]
    obs4 = plain((sanitize_token(""), sanitize_token(" "), sanitize_token("  "),
        render_metric("", 0)))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the permitted extra characters survive verbatim
    exp5 = SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[4][1]
    obs5 = plain((sanitize_token("db_read"), sanitize_token("db-read"),
        sanitize_token("db.read"), sanitize_token("Db0")))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the default posture never leaks a phase name
    exp6 = SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[5][1]
    leaky = PhaseCollector()
    leaky.push("secret_phase", 1_000_000)
    obs6 = plain(("secret_phase" in leaky.render(1_000_000, Disclosure.TOTAL_ONLY),
        leaky.render(1_000_000, Disclosure.TOTAL_ONLY),
        len(leaky.pending())))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a hidden phase is retained and can surface on a later full render
    exp7 = SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[6][1]
    retained = PhaseCollector()
    retained.push("secret_phase", 1_000_000)
    obs7 = plain((retained.render(1_000_000, Disclosure.TOTAL_ONLY),
        len(retained.pending()),
        "secret_phase" in retained.render(1_000_000, Disclosure.FULL)))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a full render prevents the next response from repeating it
    exp8 = SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[7][1]
    crossed = PhaseCollector()
    crossed.push("first_response", 1_000_000)
    obs8 = plain((crossed.render(1_000_000, Disclosure.FULL),
        crossed.render(1_000_000, Disclosure.FULL),
        crossed.pending()))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the baseline name cannot be pushed over
    exp9 = SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[8][1]
    obs9 = plain((render_header(0, Disclosure.FULL, (Phase("app", 0),)),
        render_header(0, Disclosure.TOTAL_ONLY, (Phase("app", 0),))))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a negative or huge duration still renders a bounded token
    exp10 = SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[9][1]
    obs10 = plain((format_ms(-1_000_000), format_ms(0), format_ms(10**15),
        render_metric("db", -1)))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. rendering is total and still produces a header
    exp11 = SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[10][1]
    obs11 = plain((refusal(sanitize_token, ""), sanitize_token(""),
        refusal(render_metric, "", 0), render_metric("", 0),
        refusal(render_header, 0, Disclosure.FULL, ()),
        render_header(0, Disclosure.FULL, ()),
        refusal(format_ms, 0), format_ms(0)))
    checks.append({"name": SERVER_TIMING_RESPONSE_ATTRIBUTION_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "server-timing-response-attribution-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
