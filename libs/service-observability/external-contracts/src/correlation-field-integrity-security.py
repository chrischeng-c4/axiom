from __future__ import annotations

from service_observability.application.formatter import (
    EventMetadata,
    format_event,
)
from service_observability.domain.bounds import MAX_REQUEST_ID_BYTES
from service_observability.domain.correlation import (
    preferred_hex,
    preferred_request_id,
    valid_lower_hex,
    valid_request_id,
)
from service_observability.domain.identity import make_identity
from service_observability.domain.text import byte_len
from service_observability.infrastructure.envelope import (
    OPTIONAL_ENVELOPE_KEYS,
    to_mapping,
)

MINIMUM_CHECKS = 14

CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX = (
    ("an_id_of_the_wrong_length_is_refused", (False, False, False, False)),
    ("upper_case_hex_is_refused", (False, False, False)),
    ("a_non_hex_character_anywhere_is_refused", (False, False, False)),
    ("the_all_zero_id_is_refused_as_the_w3c_invalid_value", (False, False, True)),
    ("an_invalid_id_is_omitted_not_repaired_into_shape", ((None, None, None, None, None), False)),
    ("an_invalid_event_level_id_falls_through_to_the_span", ('0af7651916cd43dd8448eb211c80319c', '0af7651916cd43dd8448eb211c80319c')),
    ("an_empty_request_id_is_refused", (False, None, False)),
    ("a_request_id_is_refused_one_byte_past_its_bound", (True, False, False, 129)),
    ("a_control_character_in_a_request_id_is_refused", (False, False, False, None)),
    ("every_unicode_c_category_is_refused_not_only_ascii", (False, False, False, True)),
    ("an_invalid_request_id_falls_through_to_the_next_spelling", ('good', 'good')),
    ("an_invalid_value_is_never_published_in_any_slot", (None, None, None, None, None)),
    ("a_non_string_correlation_field_is_ignored_not_coerced", (None, None, None, None, None)),
    ("a_rejected_correlation_value_does_not_leak_into_attributes", ({'target': 'axiom::server'}, {'target': 'axiom::server'})),
)

TS = "2026-01-01T00:00:00Z"


META = EventMetadata(name="metadata_name", target="axiom::server", severity="INFO")


IDENT = make_identity("lumen", "1.2.3")


TRACE = "0af7651916cd43dd8448eb211c80319c"


SPAN = "b7ad6b7169203331"


PARENT = "00f067aa0ba902b7"


ZERO_TRACE = "0" * 32


ZERO_SPAN = "0" * 16


def event(fields=None, span=None, meta=META, ident=IDENT):
    """Format one event, varying only what a row is about."""
    return format_event(dict(fields or {}), dict(span or {}), meta, ident, TS)


def envelope(fields=None, span=None, meta=META, ident=IDENT):
    """The serialized mapping — what a collector actually reads."""
    return to_mapping(event(fields, span, meta, ident))


def correlation(env) -> tuple:
    """Just the five correlation slots, absent ones as None."""
    return tuple(env.get(k) for k in OPTIONAL_ENVELOPE_KEYS)


def verify_correlation_field_integrity_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an id of the wrong length is refused
    exp1 = CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[0][1]
    obs1 = (valid_lower_hex(TRACE[:31], 32, True), valid_lower_hex(TRACE + '0', 32, True), valid_lower_hex(SPAN, 32, True), valid_lower_hex('', 32, True))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. upper-case hex is refused, because the W3C form is lower-case
    exp2 = CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[1][1]
    obs2 = (valid_lower_hex(TRACE.upper(), 32, True), valid_lower_hex(SPAN.upper(), 16, True), valid_lower_hex('0A', 2, False))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a non-hex character anywhere in the id is refused
    exp3 = CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[2][1]
    obs3 = (valid_lower_hex('g' + TRACE[1:], 32, True), valid_lower_hex(TRACE[:31] + 'z', 32, True), valid_lower_hex('-' + SPAN[1:], 16, True))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the all-zero id is the W3C invalid value and is refused
    exp4 = CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[3][1]
    obs4 = (valid_lower_hex(ZERO_TRACE, 32, True), valid_lower_hex(ZERO_SPAN, 16, True), valid_lower_hex('00', 2, False))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. an invalid id is omitted from the envelope, never repaired into shape
    exp5 = CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[4][1]
    obs5 = (correlation(envelope({'trace_id': TRACE.upper(), 'span_id': ZERO_SPAN})), 'trace_id' in envelope({'trace_id': TRACE.upper()}))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. an invalid event-level id falls through to a valid span-level one
    exp6 = CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[5][1]
    obs6 = (preferred_hex({'trace_id': ZERO_TRACE}, {'trace_id': TRACE}, 'trace_id', 32, True), envelope({'trace_id': 'nope'}, {'trace_id': TRACE})['trace_id'])
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. an empty request id is refused
    exp7 = CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[6][1]
    obs7 = (valid_request_id(''), preferred_request_id({'request_id': ''}, {}), 'request_id' in envelope({'request_id': ''}))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a request id is refused one byte past its bound, kept at the bound
    exp8 = CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[7][1]
    obs8 = (valid_request_id('x' * MAX_REQUEST_ID_BYTES), valid_request_id('x' * (MAX_REQUEST_ID_BYTES + 1)), valid_request_id('字' * 43), byte_len('字' * 43))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a control character in a request id is refused, not stripped
    exp9 = CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[8][1]
    obs9 = (valid_request_id('r\n1'), valid_request_id('r\x00'), valid_request_id('r\x1b[0m'), preferred_request_id({'request_id': 'r\n1'}, {}))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. every Unicode C category is refused, not only the ASCII controls
    exp10 = CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[9][1]
    obs10 = (valid_request_id('r\xad1'), valid_request_id('r\u200b1'), valid_request_id('r\ue0001'), valid_request_id('r-1'))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. an invalid request id falls through to the next spelling
    exp11 = CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[10][1]
    obs11 = (preferred_request_id({'request_id': '', 'request.id': 'good'}, {}), preferred_request_id({'request_id': 'a\nb', 'http.request.id': 'good'}, {}))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. an invalid value is never published in any slot
    exp12 = CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[11][1]
    obs12 = correlation(envelope({'trace_id': ZERO_TRACE, 'span_id': 'SHORT', 'parent_span_id': PARENT.upper(), 'trace_flags': '0', 'request_id': '\x07'}))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. a non-string correlation field is ignored rather than coerced
    exp13 = CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[12][1]
    obs13 = correlation(envelope({'trace_id': 12345, 'span_id': None, 'request_id': True}))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    # 14. a rejected correlation value does not leak into attributes either
    exp14 = CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[13][1]
    obs14 = (envelope({'trace_id': ZERO_TRACE})['attributes'], envelope({'request_id': 'r\n1'})['attributes'])
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_SECURITY_MATRIX[13][0], "expected": exp14,
                   "observed": obs14, "passed": obs14 == exp14})

    return {
        "case_id": "correlation-field-integrity-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
