from __future__ import annotations

from service_http.domain.errors import ApiError, ErrorEnvelope, PAYLOAD_TOO_LARGE, RATE_LIMITED, STATUS_PAYLOAD_TOO_LARGE, STATUS_TOO_MANY_REQUESTS, envelope_fields, envelope_of, payload_too_large, rate_limited

MINIMUM_CHECKS = 10

SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX = (
    ("the_admission_refusal_is_a_429_with_a_machine_stable_kind",
     (429, 'rate_limited', 'request admission limit exceeded')),
    ("the_size_refusal_is_a_413_with_a_machine_stable_kind",
     (413, 'payload_too_large', 'request body exceeds the configured size limit')),
    ("the_envelope_carries_the_kind_as_error_and_the_message_as_message",
     (('rate_limited', 'request admission limit exceeded'), ('payload_too_large', 'request body exceeds the configured size limit'))),
    ("the_wire_fields_are_error_then_message",
     ((('error', 'rate_limited'), ('message', 'request admission limit exceeded')), (('error', 'payload_too_large'), ('message', 'request body exceeds the configured size limit')))),
    ("the_status_constants_are_the_documented_ones",
     (429, 413, 'rate_limited', 'payload_too_large')),
    ("the_kind_the_envelope_publishes_is_the_kind_the_error_declared",
     (True, True, True, False)),
    ("an_envelope_compares_by_value",
     (True, True, False, 'rate_limited')),
    ("an_api_error_compares_by_value",
     (True, True, False, 429)),
    ("the_constructors_are_pure",
     (True, True, 429, 413)),
    ("an_arbitrary_error_renders_through_the_same_envelope",
     (('internal', 'boom'), (('error', 'internal'), ('message', 'boom')))),
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


def verify_shared_error_envelope_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the admission refusal is a 429 with a machine stable kind
    exp1 = SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[0][1]
    limited = rate_limited()
    obs1 = plain((limited.status, limited.kind, limited.message))
    checks.append({"name": SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the size refusal is a 413 with a machine stable kind
    exp2 = SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[1][1]
    oversized = payload_too_large()
    obs2 = plain((oversized.status, oversized.kind, oversized.message))
    checks.append({"name": SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the envelope carries the kind as error and the message as message
    exp3 = SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[2][1]
    limited = rate_limited()
    oversized = payload_too_large()
    obs3 = plain((envelope_of(limited), envelope_of(oversized)))
    checks.append({"name": SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the wire fields are error then message
    exp4 = SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[3][1]
    limited = rate_limited()
    oversized = payload_too_large()
    obs4 = plain((envelope_fields(envelope_of(limited)),
        envelope_fields(envelope_of(oversized))))
    checks.append({"name": SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the status constants are the documented ones
    exp5 = SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[4][1]
    obs5 = plain((STATUS_TOO_MANY_REQUESTS, STATUS_PAYLOAD_TOO_LARGE,
        RATE_LIMITED, PAYLOAD_TOO_LARGE))
    checks.append({"name": SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the kind the envelope publishes is the kind the error declared
    exp6 = SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[5][1]
    limited = rate_limited()
    oversized = payload_too_large()
    obs6 = plain((envelope_of(limited).error == limited.kind,
        envelope_of(oversized).error == oversized.kind,
        envelope_of(limited).message == limited.message,
        envelope_of(limited).error == oversized.kind))
    checks.append({"name": SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. an envelope compares by value
    exp7 = SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[6][1]
    limited = rate_limited()
    oversized = payload_too_large()
    obs7 = plain((envelope_of(limited) == ErrorEnvelope(RATE_LIMITED, limited.message),
        envelope_of(limited) == envelope_of(rate_limited()),
        envelope_of(limited) == envelope_of(oversized),
        envelope_of(limited).error))
    checks.append({"name": SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. an api error compares by value
    exp8 = SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[7][1]
    limited = rate_limited()
    obs8 = plain((rate_limited() == ApiError(429, RATE_LIMITED, limited.message),
        rate_limited() == rate_limited(), rate_limited() == payload_too_large(),
        rate_limited().status))
    checks.append({"name": SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the constructors are pure
    exp9 = SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[8][1]
    obs9 = plain((rate_limited() == rate_limited(),
        payload_too_large() == payload_too_large(),
        rate_limited().status, payload_too_large().status))
    checks.append({"name": SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. an arbitrary error renders through the same envelope
    exp10 = SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[9][1]
    obs10 = plain((envelope_of(ApiError(500, "internal", "boom")),
        envelope_fields(envelope_of(ApiError(500, "internal", "boom")))))
    checks.append({"name": SHARED_ERROR_ENVELOPE_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "shared-error-envelope-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
