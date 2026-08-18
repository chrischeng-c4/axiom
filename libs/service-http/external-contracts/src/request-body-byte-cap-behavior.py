from __future__ import annotations

from service_http.application.body_limit import BodyOutcome, DEFAULT_BODY_LIMIT_BYTES, classify, rewrite_status
from service_http.domain.errors import envelope_of, payload_too_large
from service_http.infrastructure.headers import CONTENT_LENGTH_HEADER, content_length_exceeds
from service_http.infrastructure.numbers import parse_ascii_unsigned

MINIMUM_CHECKS = 12

REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX = (
    ("a_declared_oversized_body_is_refused_before_a_byte_is_read",
     ('rejected-declared', 'rejected-declared', True)),
    ("a_streamed_body_is_bounded_mid_read",
     ('rejected-streamed', 'rejected-streamed', 'pass')),
    ("the_cap_is_strictly_over_not_at",
     ('pass', 'rejected-declared', 'pass', 'rejected-streamed')),
    ("an_absent_header_defers_to_the_streamed_count",
     ('pass', 'rejected-streamed', False)),
    ("an_unparseable_header_defers_to_the_streamed_count",
     ('pass', 'rejected-streamed', False, False)),
    ("the_declared_check_runs_before_the_streamed_one",
     ('rejected-declared', 'rejected-streamed', 'rejected-declared')),
    ("an_upstream_413_is_rewritten_into_this_crates_envelope",
     ((413, 'payload_too_large', 'request body exceeds the configured size limit'), ('payload_too_large', 'request body exceeds the configured size limit'), True)),
    ("only_the_cap_status_is_rewritten_and_its_neighbours_are_left_alone",
     (None, None, None, 413, None)),
    ("the_default_cap_is_the_documented_size",
     (8388608, 8, 'pass')),
    ("the_three_outcomes_are_distinct_named_values",
     ('pass', 'rejected-declared', 'rejected-streamed', 3)),
    ("a_declared_length_is_read_as_an_unsigned_decimal",
     (0, 10, 10, True)),
    ("the_header_the_declared_length_is_read_from_is_lower_case",
     ('content-length', True, 1)),
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


def verify_request_body_byte_cap_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a declared oversized body is refused before a byte is read
    exp1 = REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[0][1]
    obs1 = plain((classify("100", 0, 10), BodyOutcome.REJECTED_DECLARED,
        classify("100", 0, 10) is BodyOutcome.REJECTED_DECLARED))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a streamed body is bounded mid read
    exp2 = REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[1][1]
    obs2 = plain((classify(None, 100, 10), classify(None, 11, 10),
        classify(None, 10, 10)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the cap is strictly over not at
    exp3 = REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[2][1]
    obs3 = plain((classify("10", 0, 10), classify("11", 0, 10),
        classify(None, 10, 10), classify(None, 11, 10)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an absent header defers to the streamed count
    exp4 = REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[3][1]
    obs4 = plain((classify(None, 0, 10), classify(None, 11, 10),
        content_length_exceeds(None, 10)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. an unparseable header defers to the streamed count
    exp5 = REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[4][1]
    obs5 = plain((classify("abc", 0, 10), classify("abc", 11, 10),
        content_length_exceeds("abc", 10), content_length_exceeds("", 10)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the declared check runs before the streamed one
    exp6 = REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[5][1]
    obs6 = plain((classify("100", 100, 10), classify("1", 100, 10),
        classify("100", 0, 10)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. an upstream 413 is rewritten into this crates envelope
    exp7 = REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[6][1]
    obs7 = plain((rewrite_status(413), envelope_of(rewrite_status(413)),
        rewrite_status(413) == payload_too_large()))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. only the cap status is rewritten and its neighbours are left alone
    exp8 = REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[7][1]
    obs8 = plain((rewrite_status(200), rewrite_status(400), rewrite_status(412),
        rewrite_status(413).status, rewrite_status(414)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the default cap is the documented size
    exp9 = REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[8][1]
    obs9 = plain((DEFAULT_BODY_LIMIT_BYTES, DEFAULT_BODY_LIMIT_BYTES // 1024 // 1024,
        classify(str(DEFAULT_BODY_LIMIT_BYTES), 0, DEFAULT_BODY_LIMIT_BYTES)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the three outcomes are distinct named values
    exp10 = REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[9][1]
    obs10 = plain((BodyOutcome.PASS.value, BodyOutcome.REJECTED_DECLARED.value,
        BodyOutcome.REJECTED_STREAMED.value,
        len({BodyOutcome.PASS, BodyOutcome.REJECTED_DECLARED,
        BodyOutcome.REJECTED_STREAMED})))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a declared length is read as an unsigned decimal
    exp11 = REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[10][1]
    obs11 = plain((parse_ascii_unsigned("0"), parse_ascii_unsigned("10"),
        parse_ascii_unsigned("+10"), content_length_exceeds("+11", 10)))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. the header the declared length is read from is lower case
    exp12 = REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[11][1]
    obs12 = plain((CONTENT_LENGTH_HEADER,
        CONTENT_LENGTH_HEADER == CONTENT_LENGTH_HEADER.lower(),
        CONTENT_LENGTH_HEADER.count("-")))
    checks.append({"name": REQUEST_BODY_BYTE_CAP_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "request-body-byte-cap-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
