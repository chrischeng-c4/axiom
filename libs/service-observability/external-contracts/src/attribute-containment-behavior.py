from __future__ import annotations

from service_observability.application.formatter import (
    EventMetadata,
    format_event,
    merge_attributes,
)
from service_observability.domain.attributes import (
    bounded_attributes,
    bounded_value,
)
from service_observability.domain.bounds import (
    MAX_ATTRIBUTES,
    MAX_ATTRIBUTE_KEY_BYTES,
    MAX_ATTRIBUTE_VALUE_BYTES,
    MAX_EVENT_BYTES,
    MAX_REQUEST_ID_BYTES,
)
from service_observability.domain.identity import make_identity
from service_observability.domain.text import (
    byte_len,
    truncate_utf8,
)
from service_observability.infrastructure.envelope import to_mapping

MINIMUM_CHECKS = 11

ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX = (
    ("an_ordinary_field_reaches_attributes_with_its_type_intact", {'b': True, 'f': 1.5, 'i': 7, 'n': None, 's': 'text'}),
    ("attributes_are_published_in_sorted_key_order", (('a', 'm', 'z'), 'attributes')),
    ("the_tracing_target_is_recorded_when_the_caller_supplied_none", ({'target': 'axiom::server'}, {'target': 'axiom::server'})),
    ("a_caller_supplied_target_wins_over_the_tracing_one", ({'target': 'mine'}, {'target': 'from_span'})),
    ("an_event_field_overrides_the_same_span_field", ({'k': 'event', 'target': 'axiom::server'}, {'k': 'span', 'target': 'axiom::server'})),
    ("a_scalar_value_passes_through_the_bound_untouched", ('short', 42, 3.5, False, None)),
    ("a_non_scalar_value_is_rendered_to_a_bounded_string", ("('a', 'b')", "{'k': 1}", '[1, 2, 3]')),
    ("the_published_bounds_are_the_ones_the_promise_names", (64, 128, 4096, 128, 128)),
    ("exactly_the_attribute_bound_survives_one_under_and_one_over", (63, 64, 64)),
    ("a_key_and_a_value_at_their_exact_bound_survive_unchanged", (128, 4096, 3, 2)),
    ("an_empty_attribute_map_still_carries_the_target", ({}, {'target': 'axiom::server'}, 1)),
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


def verify_attribute_containment_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an ordinary caller field reaches attributes with its scalar type intact
    exp1 = ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[0][1]
    obs1 = bounded_attributes({'s': 'text', 'i': 7, 'f': 1.5, 'b': True, 'n': None})
    checks.append({"name": ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. attributes are published in sorted key order
    exp2 = ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[1][1]
    obs2 = (tuple(bounded_attributes({'z': 1, 'a': 2, 'm': 3}).keys()), tuple(envelope({'z': 1, 'a': 2}).keys())[-1])
    checks.append({"name": ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the tracing target is recorded when the caller supplied none
    exp3 = ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[2][1]
    obs3 = (merge_attributes({}, {}, META), envelope()['attributes'])
    checks.append({"name": ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a caller-supplied target wins over the tracing one
    exp4 = ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[3][1]
    obs4 = (merge_attributes({'target': 'mine'}, {}, META), merge_attributes({}, {'target': 'from_span'}, META))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. an event field overrides the same span field
    exp5 = ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[4][1]
    obs5 = (merge_attributes({'k': 'event'}, {'k': 'span'}, META), merge_attributes({}, {'k': 'span'}, META))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a scalar value passes through the bound untouched
    exp6 = ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[5][1]
    obs6 = (bounded_value('short'), bounded_value(42), bounded_value(3.5), bounded_value(False), bounded_value(None))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a non-scalar value is rendered to a string and bounded like any other
    exp7 = ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[6][1]
    obs7 = (bounded_value(('a', 'b')), bounded_value({'k': 1}), bounded_value([1, 2, 3]))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the published bounds are the ones the promise names
    exp8 = ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[7][1]
    obs8 = (MAX_ATTRIBUTES, MAX_ATTRIBUTE_KEY_BYTES, MAX_ATTRIBUTE_VALUE_BYTES, MAX_EVENT_BYTES, MAX_REQUEST_ID_BYTES)
    checks.append({"name": ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. exactly the attribute bound survives, one under and one over
    exp9 = ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[8][1]
    obs9 = (len(bounded_attributes({f'k{i:03d}': i for i in range(MAX_ATTRIBUTES - 1)})), len(bounded_attributes({f'k{i:03d}': i for i in range(MAX_ATTRIBUTES)})), len(bounded_attributes({f'k{i:03d}': i for i in range(MAX_ATTRIBUTES + 1)})))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a key and a value at their exact bound survive unchanged
    exp10 = ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[9][1]
    obs10 = (len(truncate_utf8('k' * MAX_ATTRIBUTE_KEY_BYTES, MAX_ATTRIBUTE_KEY_BYTES)), len(truncate_utf8('v' * MAX_ATTRIBUTE_VALUE_BYTES, MAX_ATTRIBUTE_VALUE_BYTES)), byte_len('字'), byte_len('é'))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. an empty attribute map still carries the target and nothing else
    exp11 = ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[10][1]
    obs11 = (bounded_attributes({}), envelope()['attributes'], len(envelope()['attributes']))
    checks.append({"name": ATTRIBUTE_CONTAINMENT_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "attribute-containment-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
