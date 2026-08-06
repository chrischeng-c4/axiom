from __future__ import annotations

from service_http.domain.errors import ApiError, ErrorEnvelope, PAYLOAD_TOO_LARGE, RATE_LIMITED, envelope_fields, envelope_of, payload_too_large, rate_limited

MINIMUM_CHECKS = 10

SHARED_ERROR_ENVELOPE_SECURITY_MATRIX = (
    ("the_field_order_is_pinned_not_merely_the_field_set",
     ((('error', 'a'), ('message', 'b')), 'error', 'message')),
    ("the_wire_body_is_exactly_two_fields",
     (2, 2, 2)),
    ("the_status_never_reaches_the_body",
     (False, False, ('error', 'message'))),
    ("an_envelope_is_immutable",
     ('FrozenInstanceError', 'FrozenInstanceError', 'ErrorEnvelope')),
    ("the_kinds_are_snake_case_machine_tokens",
     ('rate_limited', 'payload_too_large', False, True)),
    ("the_two_refusals_do_not_share_a_kind_or_a_status",
     (False, False, False)),
    ("the_operator_facing_message_names_no_internal_detail",
     ('request admission limit exceeded', 'request body exceeds the configured size limit', False)),
    ("rendering_is_total_and_still_produces_the_two_fields",
     ('accepted', 'accepted', (('error', 'a'), ('message', 'b')), 'accepted', 'accepted', (('error', ''), ('message', '')))),
    ("an_empty_kind_still_renders_both_fields",
     (('', ''), (('error', ''), ('message', '')))),
    ("a_foreign_value_is_refused_rather_than_rendered",
     ('AttributeError', 'AttributeError', 'AttributeError')),
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


def verify_shared_error_envelope_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the field order is pinned not merely the field set
    exp1 = SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[0][1]
    obs1 = plain((envelope_fields(ErrorEnvelope("a", "b")),
        envelope_fields(ErrorEnvelope("a", "b"))[0][0],
        envelope_fields(ErrorEnvelope("a", "b"))[1][0]))
    checks.append({"name": SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the wire body is exactly two fields
    exp2 = SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[1][1]
    obs2 = plain((len(envelope_fields(envelope_of(rate_limited()))),
        len(envelope_fields(envelope_of(payload_too_large()))),
        len(envelope_fields(ErrorEnvelope("", "")))))
    checks.append({"name": SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the status never reaches the body
    exp3 = SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[2][1]
    obs3 = plain(("429" in str(envelope_of(rate_limited())),
        "413" in str(envelope_of(payload_too_large())),
        tuple(n for n, _ in envelope_fields(envelope_of(rate_limited())))))
    checks.append({"name": SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an envelope is immutable
    exp4 = SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[3][1]
    obs4 = plain((refusal(setattr, ErrorEnvelope("a", "b"), "error", "c"),
        refusal(setattr, rate_limited(), "status", 200),
        variant(envelope_of(rate_limited()))))
    checks.append({"name": SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the kinds are snake case machine tokens
    exp5 = SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[4][1]
    obs5 = plain((RATE_LIMITED, PAYLOAD_TOO_LARGE, " " in RATE_LIMITED,
        RATE_LIMITED.lower() == RATE_LIMITED))
    checks.append({"name": SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the two refusals do not share a kind or a status
    exp6 = SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[5][1]
    obs6 = plain((rate_limited().kind == payload_too_large().kind,
        rate_limited().status == payload_too_large().status,
        rate_limited().message == payload_too_large().message))
    checks.append({"name": SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the operator facing message names no internal detail
    exp7 = SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[6][1]
    obs7 = plain((rate_limited().message, payload_too_large().message,
        "http" in rate_limited().message.lower()))
    checks.append({"name": SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. rendering is total and still produces the two fields
    exp8 = SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[7][1]
    obs8 = plain((refusal(envelope_of, rate_limited()),
        refusal(envelope_fields, ErrorEnvelope("a", "b")),
        envelope_fields(ErrorEnvelope("a", "b")),
        refusal(envelope_of, ApiError(0, "", "")),
        refusal(envelope_fields, envelope_of(ApiError(0, "", ""))),
        envelope_fields(envelope_of(ApiError(0, "", "")))))
    checks.append({"name": SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. an empty kind still renders both fields
    exp9 = SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[8][1]
    obs9 = plain((envelope_of(ApiError(500, "", "")),
        envelope_fields(envelope_of(ApiError(500, "", "")))))
    checks.append({"name": SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a foreign value is refused rather than rendered
    exp10 = SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[9][1]
    obs10 = plain((refusal(envelope_of, "boom"), refusal(envelope_of, None),
        refusal(envelope_fields, "boom")))
    checks.append({"name": SHARED_ERROR_ENVELOPE_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "shared-error-envelope-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
