from __future__ import annotations

from service_http.application.trace_context import parse_traceparent, request_trace_context
from service_http.domain.trace import SUPPORTED_VERSION, is_all_zero, is_local_root, is_lower_hex

MINIMUM_CHECKS = 13

ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX = (
    ("a_wrong_length_is_treated_as_absent_rather_than_rejected",
     (None, None, None, True)),
    ("the_separators_must_sit_at_the_documented_positions",
     (None, None, None, 'TraceParent')),
    ("upper_case_hexadecimal_is_not_lower_case_hexadecimal",
     (None, None, True, False)),
    ("an_all_zero_trace_id_is_refused_but_one_bit_of_entropy_is_not",
     (None, True, False, '00000000000000000000000000000001')),
    ("an_all_zero_parent_span_id_is_refused_but_one_bit_of_entropy_is_not",
     (None, True, False, '0000000000000001')),
    ("a_non_ascii_body_is_refused_before_any_field_is_read",
     (None, None, 'TraceParent')),
    ("an_unsupported_version_is_refused",
     (None, None, '00')),
    ("non_hexadecimal_flags_are_refused",
     (None, None, False, True)),
    ("a_duplicated_header_is_refused_rather_than_merged",
     (None, None, None, 2)),
    ("a_refused_parent_never_leaks_into_the_context",
     (None, '11111111111111111111111111111111', '00', False)),
    ("degrading_is_total_and_every_malformed_input_lands_on_the_fresh_id",
     ('accepted', '11111111111111111111111111111111', 'accepted', '11111111111111111111111111111111', 'accepted', '11111111111111111111111111111111', 'accepted', None)),
    ("a_degraded_context_is_a_root_and_a_valid_one_is_not",
     (True, False, True)),
    ("the_context_is_immutable",
     ('FrozenInstanceError', 'FrozenInstanceError', 'TraceContext')),
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


VALID_TP = ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",)


FRESH_TRACE = "11111111111111111111111111111111"


FRESH_SPAN = "2222222222222222"


def verify_accept_or_generate_trace_context_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a wrong length is treated as absent rather than rejected
    exp1 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[0][1]
    obs1 = plain((parse_traceparent(("00-4bf92f-00f067aa0ba902b7-01",)),
        parse_traceparent((VALID_TP[0] + "0",)),
        parse_traceparent((VALID_TP[0][:-1],)),
        is_local_root(request_trace_context(
        ("short",), FRESH_TRACE, FRESH_SPAN))))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the separators must sit at the documented positions
    exp2 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[1][1]
    obs2 = plain((parse_traceparent(("00_4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",)),
        parse_traceparent(("00-4bf92f3577b34da6a3ce929d0e0e4736_00f067aa0ba902b7-01",)),
        parse_traceparent(("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7_01",)),
        variant(parse_traceparent(VALID_TP))))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. upper case hexadecimal is not lower case hexadecimal
    exp3 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[2][1]
    obs3 = plain((parse_traceparent(("00-4BF92F3577B34DA6A3CE929D0E0E4736-00f067aa0ba902b7-01",)),
        parse_traceparent(("00-4bf92f3577b34da6a3ce929d0e0e4736-00F067AA0BA902B7-01",)),
        is_lower_hex("4bf92f"), is_lower_hex("4BF92F")))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an all zero trace id is refused but one bit of entropy is not
    exp4 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[3][1]
    obs4 = plain((parse_traceparent(("00-00000000000000000000000000000000-00f067aa0ba902b7-01",)),
        is_all_zero("00000000000000000000000000000000"),
        is_all_zero("00000000000000000000000000000001"),
        parse_traceparent(
        ("00-00000000000000000000000000000001-00f067aa0ba902b7-01",)).trace_id))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. an all zero parent span id is refused but one bit of entropy is not
    exp5 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[4][1]
    obs5 = plain((parse_traceparent(("00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01",)),
        is_all_zero("0000000000000000"), is_all_zero("0000000000000001"),
        parse_traceparent(
        ("00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000001-01",)).parent_span_id))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a non ascii body is refused before any field is read
    exp6 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[5][1]
    obs6 = plain((parse_traceparent(("00-4bf92f3577b34da6a3ce929d0e0e473６-00f067aa0ba902b7-01",)),
        parse_traceparent(("００-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",)),
        variant(parse_traceparent(VALID_TP))))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. an unsupported version is refused
    exp7 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[6][1]
    obs7 = plain((parse_traceparent(("01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",)),
        parse_traceparent(("ff-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",)),
        SUPPORTED_VERSION))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. non hexadecimal flags are refused
    exp8 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[7][1]
    obs8 = plain((parse_traceparent(("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0z",)),
        parse_traceparent(("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-  ",)),
        is_lower_hex("0z"), is_lower_hex("01")))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a duplicated header is refused rather than merged
    exp9 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[8][1]
    obs9 = plain((parse_traceparent(VALID_TP + VALID_TP),
        parse_traceparent((VALID_TP[0], "garbage")),
        parse_traceparent(()), len(VALID_TP + VALID_TP)))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a refused parent never leaks into the context
    exp10 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[9][1]
    degraded = request_trace_context(("garbage",), FRESH_TRACE, FRESH_SPAN)
    obs10 = plain((degraded.parent_span_id, degraded.trace_id, degraded.trace_flags,
        "garbage" in str(degraded)))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. degrading is total and every malformed input lands on the fresh id
    exp11 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[10][1]
    obs11 = plain((refusal(request_trace_context, ("x",), FRESH_TRACE, FRESH_SPAN),
        request_trace_context(("x",), FRESH_TRACE, FRESH_SPAN).trace_id,
        refusal(request_trace_context, (), FRESH_TRACE, FRESH_SPAN),
        request_trace_context((), FRESH_TRACE, FRESH_SPAN).trace_id,
        refusal(request_trace_context, VALID_TP + VALID_TP, FRESH_TRACE,
        FRESH_SPAN),
        request_trace_context(VALID_TP + VALID_TP, FRESH_TRACE,
        FRESH_SPAN).trace_id,
        refusal(parse_traceparent, ("",)), parse_traceparent(("",))))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a degraded context is a root and a valid one is not
    exp12 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[11][1]
    degraded = request_trace_context(("garbage",), FRESH_TRACE, FRESH_SPAN)
    obs12 = plain((is_local_root(degraded),
        is_local_root(request_trace_context(VALID_TP, FRESH_TRACE, FRESH_SPAN)),
        is_local_root(request_trace_context((), FRESH_TRACE, FRESH_SPAN))))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. the context is immutable
    exp13 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[12][1]
    degraded = request_trace_context(("garbage",), FRESH_TRACE, FRESH_SPAN)
    obs13 = plain((refusal(setattr, degraded, "trace_id", "x"),
        refusal(setattr, parse_traceparent(VALID_TP), "version", "01"),
        variant(degraded)))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    return {
        "case_id": "accept-or-generate-trace-context-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
