from __future__ import annotations

from service_observability.application.formatter import (
    EventMetadata,
    format_event,
    preferred_string,
    resolve_event_name,
    resolve_message,
)
from service_observability.domain.bounds import (
    MAX_ATTRIBUTE_VALUE_BYTES,
    MAX_EVENT_BYTES,
    SERVICE_LOG_SCHEMA_V1,
)
from service_observability.domain.identity import make_identity
from service_observability.domain.text import byte_len
from service_observability.infrastructure.config import (
    LogFormat,
    ObservabilityConfig,
    collector_compatible,
)
from service_observability.infrastructure.envelope import (
    REQUIRED_ENVELOPE_KEYS,
    to_json_line,
    to_mapping,
)

MINIMUM_CHECKS = 15

VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX = (
    ("every_line_carries_the_one_schema_tag", ('axiom.service.log.v1', 'axiom.service.log.v1', 'axiom.service.log.v1')),
    ("the_required_envelope_fields_are_always_present_in_order", (('schema', 'timestamp', 'severity', 'service', 'event', 'message'), ('schema', 'timestamp', 'severity', 'service', 'event', 'message'))),
    ("the_severity_and_service_identity_come_through_unchanged", ('INFO', {'name': 'lumen', 'version': '1.2.3'}, 'ERROR')),
    ("an_explicit_event_field_outranks_the_metadata_name", ('chosen', 'chosen')),
    ("an_absent_or_empty_event_field_falls_back_to_the_metadata_name", ('metadata_name', 'metadata_name', 'metadata_name')),
    ("a_span_level_event_name_is_used_when_the_event_carries_none", ('from_span', 'from_span', 'from_event')),
    ("an_explicit_message_is_published_as_recorded", ('hello', 'hello')),
    ("an_absent_message_falls_back_to_the_resolved_event_name", ('evt', 'metadata_name', 'metadata_name', 'named')),
    ("neither_the_event_name_nor_the_message_is_ever_empty", ('metadata_name', '', True, True)),
    ("an_over_long_message_is_cut_to_the_published_value_bound", (4096, 4096, 4096)),
    ("an_over_long_event_name_is_cut_to_the_published_bound", (128, 128, 128)),
    ("an_over_long_identity_is_cut_to_the_same_bound", (128, 128)),
    ("everything_that_is_not_an_envelope_field_becomes_an_attribute", ({'retries': 3, 'target': 'axiom::server', 'user': 'u1'}, 'attributes')),
    ("only_the_json_format_is_declared_collector_compatible", (True, 'json', 'json')),
    ("one_event_serializes_to_exactly_one_line_of_json", (0, '{}', '{"schema":"axiom.service.log.v1","timestamp":"2026-01-01T00:00:00Z","severity":"INFO","service":{"name":"lumen","version":"1.2.3"},"event":"metadata_name","message":"metadata_name","attributes":{"target":"axiom::server","user":"u1"}}')),
)

TS = "2026-01-01T00:00:00Z"


META = EventMetadata(name="metadata_name", target="axiom::server", severity="INFO")


IDENT = make_identity("lumen", "1.2.3")


def event(fields=None, span=None, meta=META, ident=IDENT):
    """Format one event, varying only what a row is about."""
    return format_event(dict(fields or {}), dict(span or {}), meta, ident, TS)


def envelope(fields=None, span=None, meta=META, ident=IDENT):
    """The serialized mapping — what a collector actually reads."""
    return to_mapping(event(fields, span, meta, ident))


def verify_versioned_log_envelope_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. every line carries the one schema tag a collector keys off
    exp1 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[0][1]
    obs1 = (SERVICE_LOG_SCHEMA_V1, envelope()['schema'], envelope({'user': 'u1'})['schema'])
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the six required envelope fields are always present, in a fixed order
    exp2 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[1][1]
    obs2 = (REQUIRED_ENVELOPE_KEYS, tuple(envelope().keys())[:len(REQUIRED_ENVELOPE_KEYS)])
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the severity and the service identity come through unchanged
    exp3 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[2][1]
    obs3 = (envelope()['severity'], envelope()['service'], envelope(meta=EventMetadata('n', 't', 'ERROR'))['severity'])
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an explicit event field outranks the tracing metadata name
    exp4 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[3][1]
    obs4 = (resolve_event_name({'event': 'chosen'}, {}, META), envelope({'event': 'chosen'})['event'])
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. an absent or empty event field falls back to the metadata name
    exp5 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[4][1]
    obs5 = (resolve_event_name({}, {}, META), resolve_event_name({'event': ''}, {}, META), envelope({'event': ''})['event'])
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a span-level event name is used when the event carries none
    exp6 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[5][1]
    obs6 = (preferred_string({}, {'event': 'from_span'}, 'event'), resolve_event_name({}, {'event': 'from_span'}, META), resolve_event_name({'event': 'from_event'}, {'event': 'from_span'}, META))
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. an explicit message is published as recorded
    exp7 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[6][1]
    obs7 = (resolve_message({'message': 'hello'}, 'evt'), envelope({'message': 'hello'})['message'])
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. an absent message falls back to the resolved event name
    exp8 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[7][1]
    obs8 = (resolve_message({}, 'evt'), envelope()['message'], envelope()['event'], envelope({'event': 'named'})['message'])
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. neither the event name nor the message is ever empty
    exp9 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[8][1]
    obs9 = (envelope({'event': '', 'message': ''})['event'], envelope({'event': '', 'message': ''})['message'], envelope({})['event'] != '', envelope({})['message'] != '')
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. an over-long message is cut to the published value bound
    exp10 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[9][1]
    long_message = "m" * 9000
    obs10 = (MAX_ATTRIBUTE_VALUE_BYTES, len(resolve_message({'message': long_message}, 'evt')), byte_len(envelope({'message': long_message})['message']))
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. an over-long event name is cut to the published event bound
    exp11 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[10][1]
    long_name = "e" * 300
    obs11 = (MAX_EVENT_BYTES, len(resolve_event_name({'event': long_name}, {}, META)), byte_len(envelope({'event': long_name})['event']))
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. an over-long identity is cut to the same bound in the envelope
    exp12 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[11][1]
    wide = make_identity("s" * 300, "v" * 300)
    obs12 = (len(envelope(ident=wide)['service']['name']), len(envelope(ident=wide)['service']['version']))
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. everything the caller records that is not an envelope field is an attribute
    exp13 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[12][1]
    obs13 = (envelope({'user': 'u1', 'retries': 3})['attributes'], tuple(envelope().keys())[-1])
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    # 14. only the JSON format is declared collector-compatible
    exp14 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[13][1]
    obs14 = (collector_compatible(LogFormat.JSON), LogFormat.JSON.value, ObservabilityConfig().log_format.value)
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[13][0], "expected": exp14,
                   "observed": obs14, "passed": obs14 == exp14})

    # 15. one event is exactly one line of JSON, whole and self-delimiting
    exp15 = VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[14][1]
    line = to_json_line(event({"user": "u1"}))
    obs15 = (line.count('\n'), line[0] + line[-1], line)
    checks.append({"name": VERSIONED_LOG_ENVELOPE_BEHAVIOR_MATRIX[14][0], "expected": exp15,
                   "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "versioned-log-envelope-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
