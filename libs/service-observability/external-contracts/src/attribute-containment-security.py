from __future__ import annotations

from service_observability.application.formatter import (
    EventMetadata,
    format_event,
)
from service_observability.domain.attributes import (
    RESERVED_KEYS,
    SENSITIVE_KEYS,
    bounded_attributes,
    is_reserved_key,
    is_sensitive_key,
)
from service_observability.domain.bounds import (
    MAX_ATTRIBUTES,
    MAX_ATTRIBUTE_KEY_BYTES,
    MAX_ATTRIBUTE_VALUE_BYTES,
)
from service_observability.domain.identity import make_identity
from service_observability.domain.text import (
    byte_len,
    truncate_utf8,
)
from service_observability.infrastructure.envelope import (
    OPTIONAL_ENVELOPE_KEYS,
    to_json_line,
    to_mapping,
)

MINIMUM_CHECKS = 15

ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX = (
    ("a_key_naming_an_envelope_field_is_dropped_not_shadowed", ({'ok': 1}, 'INFO')),
    ("every_reserved_name_is_refused_including_request_id_spellings", (('attributes', 'event', 'http.request.id', 'message', 'parent_span_id', 'request.id', 'request_id', 'schema', 'service', 'severity', 'span_id', 'timestamp', 'trace_flags', 'trace_id'), True, True, True, False)),
    ("a_caller_cannot_forge_correlation_through_attributes", ({'keep': 1}, ('0af7651916cd43dd8448eb211c80319c', None, None, None, None))),
    ("each_credential_bearing_name_is_refused_bare", (('authorization', 'proxy_authorization', 'cookie', 'set_cookie', 'baggage', 'tracestate'), (True, True, True, True, True, True))),
    ("the_credential_comparison_is_case_insensitive", (True, True, True)),
    ("a_dash_is_normalized_to_an_underscore_before_comparing", (True, True, True)),
    ("a_namespaced_key_is_caught_as_a_trailing_segment", (True, True, True, True)),
    ("a_name_that_merely_contains_a_credential_word_is_not_a_match", (False, False, False, False)),
    ("a_screened_key_never_reaches_the_published_attributes", {'kept': 'yes'}),
    ("beyond_the_count_bound_the_sorted_tail_is_cut", (64, 'k000', 'k063', False)),
    ("an_over_long_key_is_cut_to_its_exact_byte_bound", ((128,), (128,), 1)),
    ("an_over_long_value_is_cut_to_its_exact_byte_bound", (4096, 4096)),
    ("truncation_steps_back_to_a_character_boundary", (1365, 4095, 42, 126)),
    ("a_truncated_multi_byte_value_is_still_valid_json", (0, 1365, '字')),
    ("a_key_that_truncates_away_entirely_is_dropped", ('', {'k': 1})),
)

TS = "2026-01-01T00:00:00Z"


META = EventMetadata(name="metadata_name", target="axiom::server", severity="INFO")


IDENT = make_identity("lumen", "1.2.3")


TRACE = "0af7651916cd43dd8448eb211c80319c"


SPAN = "b7ad6b7169203331"


def event(fields=None, span=None, meta=META, ident=IDENT):
    """Format one event, varying only what a row is about."""
    return format_event(dict(fields or {}), dict(span or {}), meta, ident, TS)


def envelope(fields=None, span=None, meta=META, ident=IDENT):
    """The serialized mapping — what a collector actually reads."""
    return to_mapping(event(fields, span, meta, ident))


def correlation(env) -> tuple:
    """Just the five correlation slots, absent ones as None."""
    return tuple(env.get(k) for k in OPTIONAL_ENVELOPE_KEYS)


def verify_attribute_containment_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a key naming an envelope field is dropped rather than shadowing it
    exp1 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[0][1]
    obs1 = (bounded_attributes({'severity': 'FATAL', 'service': 'other', 'ok': 1}), envelope({'severity': 'FATAL'})['severity'])
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. every reserved name is refused, including the three request-id spellings
    exp2 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[1][1]
    obs2 = (tuple(sorted(RESERVED_KEYS)), is_reserved_key('request.id'), is_reserved_key('http.request.id'), is_reserved_key('attributes'), is_reserved_key('Severity'))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a caller cannot forge correlation through attributes
    exp3 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[2][1]
    obs3 = (bounded_attributes({'trace_id': TRACE, 'span_id': SPAN, 'keep': 1}), correlation(envelope({'trace_id': TRACE})))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. each credential-bearing name is refused in its bare spelling
    exp4 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[3][1]
    obs4 = (SENSITIVE_KEYS, tuple((is_sensitive_key(k) for k in SENSITIVE_KEYS)))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the comparison is case-insensitive
    exp5 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[4][1]
    obs5 = (is_sensitive_key('Authorization'), is_sensitive_key('COOKIE'), is_sensitive_key('TraceState'))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a dash is normalized to an underscore before comparing
    exp6 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[5][1]
    obs6 = (is_sensitive_key('proxy-authorization'), is_sensitive_key('Set-Cookie'), is_sensitive_key('PROXY-AUTHORIZATION'))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a namespaced key is caught as a trailing segment after . / or _
    exp7 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[6][1]
    obs7 = (is_sensitive_key('http.request.header.authorization'), is_sensitive_key('headers/cookie'), is_sensitive_key('req_baggage'), is_sensitive_key('http.request.header.Authorization'))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a name that merely contains a credential word is not a match
    exp8 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[7][1]
    obs8 = (is_sensitive_key('unauthorization'), is_sensitive_key('authorization_scheme'), is_sensitive_key('cookies'), is_sensitive_key('mycookie'))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a screened key never reaches the published attributes
    exp9 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[8][1]
    obs9 = bounded_attributes({'Authorization': 'Bearer secret', 'http.request.header.cookie': 'sid=1', 'Proxy-Authorization': 'Basic x', 'tracestate': 'a=b', 'kept': 'yes'})
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. beyond the count bound the sorted tail is cut, not a random subset
    exp10 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[9][1]
    over = bounded_attributes({f"k{i:03d}": i for i in range(MAX_ATTRIBUTES + 10)})
    obs10 = (len(over), tuple(over.keys())[0], tuple(over.keys())[-1], 'k064' in over)
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. an over-long key is cut to its exact byte bound
    exp11 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[10][1]
    wide = bounded_attributes({"k" * 300: 1})
    obs11 = (tuple((len(k) for k in wide)), tuple((byte_len(k) for k in wide)), len(wide))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. an over-long value is cut to its exact byte bound
    exp12 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[11][1]
    long_value = bounded_attributes({"k": "v" * 9000})["k"]
    obs12 = (len(long_value), byte_len(long_value))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. truncation steps back to a character boundary rather than splitting one
    exp13 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[12][1]
    cut = truncate_utf8("字" * 2000, MAX_ATTRIBUTE_VALUE_BYTES)
    key_cut = truncate_utf8("字" * 100, MAX_ATTRIBUTE_KEY_BYTES)
    obs13 = (len(cut), byte_len(cut), len(key_cut), byte_len(key_cut))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    # 14. a truncated multi-byte value is still valid JSON on the line
    exp14 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[13][1]
    line = to_json_line(event({"blob": "字" * 2000}))
    blob_field = line[line.index('"blob":') + 8:line.index('","target":')]
    obs14 = (line.count('\n'), len(blob_field), blob_field[-1])
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[13][0], "expected": exp14,
                   "observed": obs14, "passed": obs14 == exp14})

    # 15. a key that truncates away entirely is dropped rather than published empty
    exp15 = ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[14][1]
    obs15 = (truncate_utf8('', MAX_ATTRIBUTE_KEY_BYTES), bounded_attributes({'': 'orphan', 'k': 1}))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_SECURITY_MATRIX[14][0], "expected": exp15,
                   "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "attribute-containment-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
