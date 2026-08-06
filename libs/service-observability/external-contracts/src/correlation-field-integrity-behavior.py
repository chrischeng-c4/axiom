from __future__ import annotations

from service_observability.application.formatter import (
    EventMetadata,
    format_event,
)
from service_observability.domain.bounds import MAX_REQUEST_ID_BYTES
from service_observability.domain.correlation import (
    REQUEST_ID_KEYS,
    field_string,
    preferred_hex,
    preferred_request_id,
    valid_lower_hex,
    valid_request_id,
)
from service_observability.domain.identity import make_identity
from service_observability.infrastructure.envelope import (
    OPTIONAL_ENVELOPE_KEYS,
    to_mapping,
)

MINIMUM_CHECKS = 12

CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX = (
    ("a_well_formed_id_passes_its_own_length_check", (True, True, True)),
    ("the_four_hex_fields_reach_the_envelope_unchanged", ('0af7651916cd43dd8448eb211c80319c', 'b7ad6b7169203331', '00f067aa0ba902b7', '01', None)),
    ("all_zero_trace_flags_are_kept_because_zero_is_legitimate", (True, '00', 'ff')),
    ("an_event_level_value_outranks_the_enclosing_span", ('b7ad6b7169203331', 'b7ad6b7169203331')),
    ("a_span_level_value_is_used_when_the_event_records_none", ('0af7651916cd43dd8448eb211c80319c', '0af7651916cd43dd8448eb211c80319c')),
    ("the_request_id_spellings_are_searched_in_published_order", ('request_id', 'request.id', 'http.request.id')),
    ("each_request_id_spelling_on_its_own_is_honored", ('a', 'b', 'c')),
    ("when_several_spellings_are_present_the_first_one_wins", ('a', 'b', 'c')),
    ("an_event_level_request_id_outranks_any_span_level_spelling", ('event', 'event')),
    ("only_a_string_field_is_a_candidate_at_all", ('s', None, None, None)),
    ("an_ordinary_request_id_passes_its_own_check", (True, True, 128)),
    ("all_five_correlation_slots_are_filled_from_one_call", ('0af7651916cd43dd8448eb211c80319c', 'b7ad6b7169203331', '00f067aa0ba902b7', '01', 'r-9')),
)

TS = "2026-01-01T00:00:00Z"


META = EventMetadata(name="metadata_name", target="axiom::server", severity="INFO")


IDENT = make_identity("lumen", "1.2.3")


TRACE = "0af7651916cd43dd8448eb211c80319c"


SPAN = "b7ad6b7169203331"


PARENT = "00f067aa0ba902b7"


def event(fields=None, span=None, meta=META, ident=IDENT):
    """Format one event, varying only what a row is about."""
    return format_event(dict(fields or {}), dict(span or {}), meta, ident, TS)


def envelope(fields=None, span=None, meta=META, ident=IDENT):
    """The serialized mapping — what a collector actually reads."""
    return to_mapping(event(fields, span, meta, ident))


def correlation(env) -> tuple:
    """Just the five correlation slots, absent ones as None."""
    return tuple(env.get(k) for k in OPTIONAL_ENVELOPE_KEYS)


def verify_correlation_field_integrity_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a well-formed trace id, span id and parent span id all pass their check
    exp1 = CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[0][1]
    obs1 = (valid_lower_hex(TRACE, 32, True), valid_lower_hex(SPAN, 16, True), valid_lower_hex(PARENT, 16, True))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the four hex fields reach the envelope unchanged
    exp2 = CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[1][1]
    obs2 = correlation(envelope({'trace_id': TRACE, 'span_id': SPAN, 'parent_span_id': PARENT, 'trace_flags': '01'}))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. all-zero trace flags are a legitimate value and are kept
    exp3 = CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[2][1]
    obs3 = (valid_lower_hex('00', 2, False), envelope({'trace_flags': '00'})['trace_flags'], envelope({'trace_flags': 'ff'})['trace_flags'])
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an event-level value outranks the same field on the enclosing span
    exp4 = CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[3][1]
    obs4 = (preferred_hex({'span_id': SPAN}, {'span_id': PARENT}, 'span_id', 16, True), envelope({'span_id': SPAN}, {'span_id': PARENT})['span_id'])
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a span-level value is used when the event records none
    exp5 = CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[4][1]
    obs5 = (preferred_hex({}, {'trace_id': TRACE}, 'trace_id', 32, True), envelope({}, {'trace_id': TRACE})['trace_id'])
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the three request-id spellings are searched in the published order
    exp6 = CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[5][1]
    obs6 = REQUEST_ID_KEYS
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. each spelling on its own is honored
    exp7 = CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[6][1]
    obs7 = (preferred_request_id({'request_id': 'a'}, {}), preferred_request_id({'request.id': 'b'}, {}), preferred_request_id({'http.request.id': 'c'}, {}))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. when several spellings are present the first in the order wins
    exp8 = CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[7][1]
    obs8 = (preferred_request_id({'request_id': 'a', 'request.id': 'b', 'http.request.id': 'c'}, {}), preferred_request_id({'request.id': 'b', 'http.request.id': 'c'}, {}), preferred_request_id({'http.request.id': 'c'}, {}))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. an event-level request id outranks any span-level spelling
    exp9 = CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[8][1]
    obs9 = (preferred_request_id({'http.request.id': 'event'}, {'request_id': 'span'}), envelope({'http.request.id': 'event'}, {'request_id': 'span'})['request_id'])
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. only a string field is a candidate at all
    exp10 = CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[9][1]
    obs10 = (field_string({'request_id': 's'}, 'request_id'), field_string({'request_id': 7}, 'request_id'), field_string({'request_id': None}, 'request_id'), field_string({}, 'request_id'))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. an ordinary request id passes its own check
    exp11 = CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[10][1]
    obs11 = (valid_request_id('r-1'), valid_request_id('x' * MAX_REQUEST_ID_BYTES), MAX_REQUEST_ID_BYTES)
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. all five correlation slots are filled from one call
    exp12 = CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[11][1]
    obs12 = correlation(envelope({'trace_id': TRACE, 'span_id': SPAN}, {'parent_span_id': PARENT, 'trace_flags': '01', 'request.id': 'r-9'}))
    checks.append({"name": CORRELATION_FIELD_INTEGRITY_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "correlation-field-integrity-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
