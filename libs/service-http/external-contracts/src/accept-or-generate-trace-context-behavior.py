from __future__ import annotations

from service_http.application.trace_context import parse_traceparent, request_trace_context, span_fields
from service_http.domain.trace import DEFAULT_TRACE_FLAGS, SUPPORTED_VERSION, TRACEPARENT_LENGTH, TraceContext, is_local_root
from service_http.infrastructure.headers import TRACEPARENT_HEADER

MINIMUM_CHECKS = 12

ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX = (
    ("a_valid_parent_hands_its_trace_id_to_the_child",
     ('4bf92f3577b34da6a3ce929d0e0e4736', '2222222222222222', '00f067aa0ba902b7', '01')),
    ("the_child_span_is_freshly_minted_not_the_parents",
     (True, False, False)),
    ("the_parser_returns_the_four_wire_fields",
     ('TraceParent', '00', '4bf92f3577b34da6a3ce929d0e0e4736', '00f067aa0ba902b7', '01')),
    ("an_absent_header_mints_a_fresh_local_root",
     ('11111111111111111111111111111111', '2222222222222222', None, '00')),
    ("a_minted_root_carries_the_default_flags",
     ('00', '00', '01')),
    ("the_root_test_reads_the_absent_parent",
     (True, False, True)),
    ("the_log_fields_name_the_parent_only_when_there_is_one",
     ({'trace_id': '4bf92f3577b34da6a3ce929d0e0e4736', 'span_id': '2222222222222222', 'trace_flags': '01', 'parent_span_id': '00f067aa0ba902b7'}, {'trace_id': '11111111111111111111111111111111', 'span_id': '2222222222222222', 'trace_flags': '00'})),
    ("the_log_field_order_is_stable",
     (('trace_id', 'span_id', 'trace_flags', 'parent_span_id'), ('trace_id', 'span_id', 'trace_flags'))),
    ("exactly_one_header_value_is_read",
     (None, True, 'TraceParent')),
    ("the_supported_version_and_length_are_the_w3c_ones",
     ('00', 55, '00', 55)),
    ("the_header_the_parent_arrives_in_is_the_w3c_lower_case_name",
     ('traceparent', True, 11)),
    ("the_carried_flags_are_the_parents_not_the_default",
     ('01', '00', '00')),
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


VALID_TP = ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",)


FRESH_TRACE = "11111111111111111111111111111111"


FRESH_SPAN = "2222222222222222"


def verify_accept_or_generate_trace_context_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a valid parent hands its trace id to the child
    exp1 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[0][1]
    accepted = request_trace_context(VALID_TP, FRESH_TRACE, FRESH_SPAN)
    obs1 = plain((accepted.trace_id, accepted.span_id, accepted.parent_span_id,
        accepted.trace_flags))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the child span is freshly minted not the parents
    exp2 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[1][1]
    accepted = request_trace_context(VALID_TP, FRESH_TRACE, FRESH_SPAN)
    obs2 = plain((accepted.span_id == FRESH_SPAN,
        accepted.span_id == accepted.parent_span_id,
        accepted.trace_id == FRESH_TRACE))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the parser returns the four wire fields
    exp3 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[2][1]
    parsed = parse_traceparent(VALID_TP)
    obs3 = plain((variant(parsed), parsed.version, parsed.trace_id,
        parsed.parent_span_id, parsed.trace_flags))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an absent header mints a fresh local root
    exp4 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[3][1]
    fresh = request_trace_context((), FRESH_TRACE, FRESH_SPAN)
    obs4 = plain((fresh.trace_id, fresh.span_id, fresh.parent_span_id,
        fresh.trace_flags))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a minted root carries the default flags
    exp5 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[4][1]
    accepted = request_trace_context(VALID_TP, FRESH_TRACE, FRESH_SPAN)
    fresh = request_trace_context((), FRESH_TRACE, FRESH_SPAN)
    obs5 = plain((fresh.trace_flags, DEFAULT_TRACE_FLAGS, accepted.trace_flags))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the root test reads the absent parent
    exp6 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[5][1]
    accepted = request_trace_context(VALID_TP, FRESH_TRACE, FRESH_SPAN)
    fresh = request_trace_context((), FRESH_TRACE, FRESH_SPAN)
    obs6 = plain((is_local_root(fresh), is_local_root(accepted),
        is_local_root(TraceContext("a", "b", None, "00"))))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the log fields name the parent only when there is one
    exp7 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[6][1]
    accepted = request_trace_context(VALID_TP, FRESH_TRACE, FRESH_SPAN)
    fresh = request_trace_context((), FRESH_TRACE, FRESH_SPAN)
    obs7 = plain((span_fields(accepted), span_fields(fresh)))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the log field order is stable
    exp8 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[7][1]
    accepted = request_trace_context(VALID_TP, FRESH_TRACE, FRESH_SPAN)
    fresh = request_trace_context((), FRESH_TRACE, FRESH_SPAN)
    obs8 = plain((tuple(span_fields(accepted).keys()), tuple(span_fields(fresh).keys())))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. exactly one header value is read
    exp9 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[8][1]
    obs9 = plain((parse_traceparent(()), parse_traceparent(VALID_TP + VALID_TP)
        is None, variant(parse_traceparent(VALID_TP))))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the supported version and length are the w3c ones
    exp10 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[9][1]
    obs10 = plain((SUPPORTED_VERSION, TRACEPARENT_LENGTH, DEFAULT_TRACE_FLAGS,
        len(VALID_TP[0])))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the header the parent arrives in is the w3c lower case name
    exp11 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[10][1]
    obs11 = plain((TRACEPARENT_HEADER, TRACEPARENT_HEADER == TRACEPARENT_HEADER.lower(),
        len(TRACEPARENT_HEADER)))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. the carried flags are the parents not the default
    exp12 = ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[11][1]
    accepted = request_trace_context(VALID_TP, FRESH_TRACE, FRESH_SPAN)
    obs12 = plain((accepted.trace_flags,
        request_trace_context(
        ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00",),
        FRESH_TRACE, FRESH_SPAN).trace_flags,
        DEFAULT_TRACE_FLAGS))
    checks.append({"name": ACCEPT_OR_GENERATE_TRACE_CONTEXT_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "accept-or-generate-trace-context-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
