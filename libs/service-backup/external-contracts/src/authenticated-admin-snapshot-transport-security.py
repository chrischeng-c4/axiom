from __future__ import annotations

from service_backup.application.transport import classify_response
from service_backup.domain.errors import RemoteStatus, describe

MINIMUM_CHECKS = 10

AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX = (
    ("the_success_range_boundaries_are_exact",
     ('RemoteStatus', 'NoneType', 'NoneType', 'RemoteStatus')),
    ("every_status_outside_the_range_is_carried_rather_than_collapsed",
     (403, 500, 0, 599)),
    ("the_refusal_carries_the_status_and_the_body",
     (403, 'admin token rejected', 'RemoteStatus')),
    ("the_refusal_sentence_names_both",
     'admin snapshot request failed with status 403: admin token rejected'),
    ("a_body_that_looks_like_a_status_line_is_still_only_a_body",
     (502, 'HTTP/1.1 200 OK', 'admin snapshot request failed with status 502: HTTP/1.1 200 OK')),
    ("an_empty_body_is_carried_as_an_empty_body",
     ('', True, 'admin snapshot request failed with status 500: ')),
    ("a_multi_line_body_is_not_truncated",
     ('line one\nline two', 17, 1)),
    ("a_success_classification_carries_nothing_at_all",
     (True, True, True, False)),
    ("a_classification_never_raises",
     ('accepted', 'accepted', 'accepted', 'RemoteStatus')),
    ("an_unknown_error_variant_has_no_sentence",
     ('accepted', 'TypeError', 'TypeError', 'admin snapshot request failed with status 1: a')),
)


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


def verify_authenticated_admin_snapshot_transport_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the success range boundaries are exact
    exp1 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[0][1]
    obs1 = (variant(classify_response(199, "b")), variant(classify_response(200, "b")),
        variant(classify_response(299, "b")), variant(classify_response(300, "b")))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. every status outside the range is carried rather than collapsed
    exp2 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[1][1]
    obs2 = (classify_response(403, "forbidden").status, classify_response(500, "boom").status,
        classify_response(0, "x").status, classify_response(599, "y").status)
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the refusal carries the status and the body
    exp3 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[2][1]
    obs3 = (classify_response(403, "admin token rejected").status,
        classify_response(403, "admin token rejected").body,
        variant(classify_response(403, "admin token rejected")))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the refusal sentence names both
    exp4 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[3][1]
    obs4 = describe(classify_response(403, "admin token rejected"))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a body that looks like a status line is still only a body
    exp5 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[4][1]
    obs5 = (classify_response(502, "HTTP/1.1 200 OK").status,
        classify_response(502, "HTTP/1.1 200 OK").body,
        describe(classify_response(502, "HTTP/1.1 200 OK")))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. an empty body is carried as an empty body
    exp6 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[5][1]
    obs6 = (classify_response(500, "").body, classify_response(500, "").body == "",
        describe(classify_response(500, "")))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a multi line body is not truncated
    exp7 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[6][1]
    obs7 = (classify_response(500, "line one\nline two").body,
        len(classify_response(500, "line one\nline two").body),
        classify_response(500, "line one\nline two").body.count("\n"))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a success classification carries nothing at all
    exp8 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[7][1]
    obs8 = (classify_response(200, "boom") is None, classify_response(201, "") is None,
        classify_response(204, "x") is None, classify_response(400, "") is None)
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a classification never raises
    exp9 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[8][1]
    obs9 = (refusal(classify_response, 403, "x"), refusal(classify_response, 200, "x"),
        refusal(classify_response, -1, ""), variant(classify_response(-1, "")))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. an unknown error variant has no sentence
    exp10 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[9][1]
    obs10 = (refusal(describe, RemoteStatus(1, "a")), refusal(describe, 403),
        refusal(describe, None), describe(RemoteStatus(1, "a")))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "authenticated-admin-snapshot-transport-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
