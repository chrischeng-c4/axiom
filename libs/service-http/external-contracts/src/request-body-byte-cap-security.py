from __future__ import annotations

from service_http.application.body_limit import classify, rewrite_status
from service_http.domain.errors import envelope_fields, envelope_of
from service_http.infrastructure.headers import content_length_exceeds
from service_http.infrastructure.numbers import parse_ascii_unsigned

MINIMUM_CHECKS = 13

REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX = (
    ("omitting_the_header_does_not_bypass_the_cap",
     ('rejected-streamed', 'rejected-streamed', 'rejected-streamed', 'rejected-streamed', 'pass')),
    ("a_lying_header_does_not_bypass_the_streamed_check",
     ('rejected-streamed', 'rejected-streamed', 'pass')),
    ("a_signed_or_spaced_length_is_not_a_decimal",
     (None, None, None, None, 10)),
    ("a_non_ascii_digit_is_not_a_decimal",
     (None, None, None, False)),
    ("a_bare_sign_or_empty_body_is_not_a_decimal",
     (None, None, None, 0)),
    ("an_unreadable_header_is_never_treated_as_oversized",
     (False, False, False, False, True)),
    ("the_boundary_is_exact_at_the_cap",
     (False, True, False, False)),
    ("a_zero_cap_admits_only_an_empty_body",
     ('pass', 'rejected-streamed', 'pass', 'rejected-declared')),
    ("a_very_large_declared_length_is_still_read_exactly",
     (True, False, True)),
    ("the_refusal_renders_this_crates_envelope_not_a_bare_status",
     (413, 'payload_too_large', (('error', 'payload_too_large'), ('message', 'request body exceeds the configured size limit')))),
    ("a_rewrite_of_an_unrelated_status_returns_nothing",
     (None, None, None, None, 413)),
    ("classification_is_total_and_still_decides",
     ('accepted', 'pass', 'accepted', 'pass', 'accepted', 'pass', 'accepted', 413)),
    ("a_negative_streamed_count_cannot_be_oversized",
     ('pass', 'pass', 'rejected-streamed')),
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


def refusal(fn, *args, **kwargs) -> str:
    """The exception type a refusing call raises, or 'accepted' if it returns."""
    try:
        fn(*args, **kwargs)
    except Exception as exc:  # noqa: BLE001 — the type is the observation
        return type(exc).__name__
    return "accepted"


def verify_request_body_byte_cap_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. omitting the header does not bypass the cap
    exp1 = REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[0][1]
    obs1 = plain((classify(None, 11, 10), classify("", 11, 10),
        classify("abc", 11, 10), classify("1", 11, 10),
        classify(None, 10, 10)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a lying header does not bypass the streamed check
    exp2 = REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[1][1]
    obs2 = plain((classify("0", 11, 10), classify("1", 1_000_000, 10),
        classify("0", 10, 10)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a signed or spaced length is not a decimal
    exp3 = REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[2][1]
    obs3 = plain((parse_ascii_unsigned("-1"), parse_ascii_unsigned(" 1"),
        parse_ascii_unsigned("1 "), parse_ascii_unsigned("1_0"),
        parse_ascii_unsigned("10")))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a non ascii digit is not a decimal
    exp4 = REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[3][1]
    obs4 = plain((parse_ascii_unsigned("１"), parse_ascii_unsigned("١"),
        parse_ascii_unsigned("1٢"), content_length_exceeds("１", 0)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a bare sign or empty body is not a decimal
    exp5 = REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[4][1]
    obs5 = plain((parse_ascii_unsigned("+"), parse_ascii_unsigned(""),
        parse_ascii_unsigned(None), parse_ascii_unsigned("+0")))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. an unreadable header is never treated as oversized
    exp6 = REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[5][1]
    obs6 = plain((content_length_exceeds("abc", 0), content_length_exceeds(None, 0),
        content_length_exceeds("", 0), content_length_exceeds("-5", 0),
        content_length_exceeds("1", 0)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the boundary is exact at the cap
    exp7 = REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[6][1]
    obs7 = plain((content_length_exceeds("10", 10), content_length_exceeds("11", 10),
        content_length_exceeds("9", 10), content_length_exceeds("0", 0)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a zero cap admits only an empty body
    exp8 = REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[7][1]
    obs8 = plain((classify(None, 0, 0), classify(None, 1, 0),
        classify("0", 0, 0), classify("1", 0, 0)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a very large declared length is still read exactly
    exp9 = REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[8][1]
    obs9 = plain((content_length_exceeds(str(2**70), 2**69),
        content_length_exceeds(str(2**69), 2**69),
        parse_ascii_unsigned(str(2**70)) == 2**70))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the refusal renders this crates envelope not a bare status
    exp10 = REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[9][1]
    obs10 = plain((rewrite_status(413).status, rewrite_status(413).kind,
        envelope_fields(envelope_of(rewrite_status(413)))))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a rewrite of an unrelated status returns nothing
    exp11 = REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[10][1]
    obs11 = plain((rewrite_status(0), rewrite_status(200), rewrite_status(-413),
        rewrite_status(4130), rewrite_status(413).status))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. classification is total and still decides
    exp12 = REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[11][1]
    obs12 = plain((refusal(classify, None, 0, 10), classify(None, 0, 10),
        refusal(classify, "abc", 0, 10), classify("abc", 0, 10),
        refusal(classify, "1", -1, 10), classify("1", -1, 10),
        refusal(rewrite_status, 413), rewrite_status(413).status))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. a negative streamed count cannot be oversized
    exp13 = REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[12][1]
    obs13 = plain((classify(None, -1, 10), classify(None, -1, 0),
        classify(None, 0, -1)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    return {
        "case_id": "request-body-byte-cap-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
