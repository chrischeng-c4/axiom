from __future__ import annotations

from service_observability.application.formatter import (
    EventMetadata,
    format_event,
)
from service_observability.domain.identity import make_identity
from service_observability.infrastructure.config import (
    LogFormat,
    collector_compatible,
)
from service_observability.infrastructure.envelope import (
    OPTIONAL_ENVELOPE_KEYS,
    REQUIRED_ENVELOPE_KEYS,
    to_json_line,
    to_mapping,
)

MINIMUM_CHECKS = 12

VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX = (
    ("an_absent_correlation_field_is_omitted_never_null", (('schema', 'timestamp', 'severity', 'service', 'event', 'message', 'attributes'), False)),
    ("a_valid_correlation_field_appears_in_the_published_slot_order", (('trace_id', 'span_id', 'parent_span_id', 'trace_flags', 'request_id'), ('schema', 'timestamp', 'severity', 'service', 'event', 'message', 'trace_id', 'span_id', 'parent_span_id', 'trace_flags', 'request_id', 'attributes'))),
    ("the_serialized_keys_are_exactly_the_published_set", (True, True)),
    ("a_caller_cannot_add_an_envelope_key_by_recording_one", (('schema', 'timestamp', 'severity', 'service', 'event', 'message', 'attributes'), 'axiom.service.log.v1', 'INFO')),
    ("the_pretty_format_is_not_collector_compatible", (False, 'pretty', True)),
    ("a_blank_service_name_is_refused_at_construction", ('IdentityError', 'IdentityError', 'IdentityError')),
    ("a_blank_version_is_refused_but_a_short_one_is_accepted", ('IdentityError', 'IdentityError', 'accepted')),
    ("the_identity_refusal_is_a_value_error", ('IdentityError', True)),
    ("a_newline_in_a_message_cannot_break_the_line_in_two", (0, True, '"message":"first\\nsecond"')),
    ("a_newline_in_an_attribute_value_cannot_break_the_line", (0, '"attributes":{"note":"a\\nb","tab":"c\\td","target":"axiom::server"}}', 2)),
    ("a_non_ascii_message_survives_the_line_intact", (0, '"message":"溫度上升"', True)),
    ("the_published_key_tuples_are_the_schema_ones", (('schema', 'timestamp', 'severity', 'service', 'event', 'message'), ('trace_id', 'span_id', 'parent_span_id', 'trace_flags', 'request_id'))),
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


def refusal(fn, *args, **kwargs) -> str:
    """The exception type a refusing call raises, or 'accepted' if it returns."""
    try:
        fn(*args, **kwargs)
    except Exception as exc:  # noqa: BLE001 — the type is the observation
        return type(exc).__name__
    return "accepted"


def _capture(fn, *args):
    try:
        fn(*args)
    except Exception as exc:  # noqa: BLE001
        return exc
    return None


def verify_versioned_log_envelope_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an absent correlation field is omitted, never serialized as null
    exp1 = VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[0][1]
    obs1 = (tuple(envelope().keys()), any((v is None for v in envelope().values())))
    checks.append({"name": VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a valid correlation field does appear, in the published slot order
    exp2 = VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[1][1]
    full = envelope({
        "trace_id": TRACE,
        "span_id": SPAN,
        "parent_span_id": PARENT,
        "trace_flags": "01",
        "request_id": "r-1",
    })
    obs2 = (OPTIONAL_ENVELOPE_KEYS, tuple(full.keys()))
    checks.append({"name": VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the serialized keys are exactly the published set, with no extras
    exp3 = VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[2][1]
    obs3 = (set(full.keys()) == set(REQUIRED_ENVELOPE_KEYS) | set(OPTIONAL_ENVELOPE_KEYS) | {'attributes'}, set(envelope().keys()) - set(REQUIRED_ENVELOPE_KEYS) == {'attributes'})
    checks.append({"name": VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a caller cannot add an envelope key by recording one
    exp4 = VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[3][1]
    obs4 = (tuple(envelope({'schema': 'forged', 'severity': 'FATAL'}).keys()), envelope({'schema': 'forged'})['schema'], envelope({'severity': 'FATAL'})['severity'])
    checks.append({"name": VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the pretty format is not collector-compatible
    exp5 = VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[4][1]
    obs5 = (collector_compatible(LogFormat.PRETTY), LogFormat.PRETTY.value, collector_compatible(LogFormat.JSON))
    checks.append({"name": VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a blank service name is refused at construction
    exp6 = VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[5][1]
    obs6 = (refusal(make_identity, '', '1.0'), refusal(make_identity, '   ', '1.0'), refusal(make_identity, '\t\n', '1.0'))
    checks.append({"name": VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a blank version is refused, while a short non-blank one is accepted
    exp7 = VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[6][1]
    obs7 = (refusal(make_identity, 'lumen', ''), refusal(make_identity, 'lumen', '  '), refusal(make_identity, 'lumen', '0'))
    checks.append({"name": VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the refusal is a ValueError, so an existing handler still catches it
    exp8 = VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[7][1]
    obs8 = (refusal(make_identity, '', ''), isinstance(_capture(make_identity, '', '1.0'), ValueError))
    checks.append({"name": VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a newline in a message cannot break the line into two
    exp9 = VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[8][1]
    multi = to_json_line(event({"message": "first\nsecond"}))
    obs9 = (multi.count('\n'), '\\n' in multi, multi[multi.index('"message":'):multi.index(',"attributes":')])
    checks.append({"name": VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a newline in an attribute value cannot break the line either
    exp10 = VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[9][1]
    attr_line = to_json_line(event({"note": "a\nb", "tab": "c\td"}))
    obs10 = (attr_line.count('\n'), attr_line[attr_line.index('"attributes":'):], attr_line.count('\\'))
    checks.append({"name": VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a non-ASCII message survives the line intact rather than being escaped away
    exp11 = VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[10][1]
    uni = to_json_line(event({"message": "溫度上升"}))
    obs11 = (uni.count('\n'), uni[uni.index('"message":'):uni.index(',"attributes":')], '溫度上升' in uni)
    checks.append({"name": VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. the published key tuples are the ones the schema was written against
    exp12 = VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[11][1]
    obs12 = (REQUIRED_ENVELOPE_KEYS, OPTIONAL_ENVELOPE_KEYS)
    checks.append({"name": VERSIONED_LOG_ENVELOPE_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "versioned-log-envelope-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
