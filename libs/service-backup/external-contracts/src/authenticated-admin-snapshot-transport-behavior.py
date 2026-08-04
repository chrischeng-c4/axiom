from __future__ import annotations

from service_backup.application.transport import ADMIN_SNAPSHOT_PATH, AUTHORIZATION_HEADER, BEARER_PREFIX, admin_request_headers, admin_snapshot_url, classify_response

MINIMUM_CHECKS = 10

AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX = (
    ("the_endpoint_path_is_fixed",
     ('/admin/backup', 'http://localhost:8080/admin/backup')),
    ("trailing_separators_on_the_base_url_do_not_double_the_join",
     ('http://h:1/admin/backup', 'http://h:1/admin/backup', 'http://h:1/admin/backup', 'http://h:1/api/admin/backup', 'http://h:1/api/admin/backup')),
    ("the_base_url_is_otherwise_carried_through_unchanged",
     ('https://lumen.example.com:8443/admin/backup', '/admin/backup', '/admin/backup')),
    ("an_authorized_request_carries_exactly_one_header",
     (('authorization',), 1, {'authorization': 'Bearer tok'})),
    ("the_header_value_splits_into_a_scheme_label_and_the_token",
     ('Bearer ', 'tok', 'Bearer', 'tok', 'Bearer ')),
    ("the_header_name_is_the_standard_lower_case_spelling",
     ('authorization', True, False)),
    ("an_absent_token_sends_no_header_at_all",
     ({}, 0, (), False)),
    ("the_empty_string_is_a_present_token",
     ({'authorization': 'Bearer '}, 1, 'Bearer ')),
    ("the_token_is_carried_byte_for_byte",
     ('Bearer a b', 'Bearer   t  ', 'Bearer Bearer x')),
    ("a_successful_response_classifies_to_nothing",
     (None, None, None)),
)


def verify_authenticated_admin_snapshot_transport_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the endpoint path is fixed
    exp1 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[0][1]
    obs1 = (ADMIN_SNAPSHOT_PATH, admin_snapshot_url("http://localhost:8080"))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. trailing separators on the base url do not double the join
    exp2 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[1][1]
    obs2 = (admin_snapshot_url("http://h:1"), admin_snapshot_url("http://h:1/"),
        admin_snapshot_url("http://h:1///"), admin_snapshot_url("http://h:1/api"),
        admin_snapshot_url("http://h:1/api/"))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the base url is otherwise carried through unchanged
    exp3 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[2][1]
    obs3 = (admin_snapshot_url("https://lumen.example.com:8443"),
        admin_snapshot_url(""), admin_snapshot_url("/"))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an authorized request carries exactly one header
    exp4 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[3][1]
    obs4 = (tuple(admin_request_headers("tok").keys()), len(admin_request_headers("tok")),
        admin_request_headers("tok"))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the header value splits into a scheme label and the token
    exp5 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[4][1]
    obs5 = (admin_request_headers("tok")[AUTHORIZATION_HEADER][:7],
        admin_request_headers("tok")[AUTHORIZATION_HEADER][7:],
        admin_request_headers("tok")[AUTHORIZATION_HEADER].split(" ")[0],
        admin_request_headers("tok")[AUTHORIZATION_HEADER].split(" ")[1],
        BEARER_PREFIX)
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the header name is the standard lower case spelling
    exp6 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[5][1]
    obs6 = (AUTHORIZATION_HEADER, AUTHORIZATION_HEADER in admin_request_headers("t"),
        "Authorization" in admin_request_headers("t"))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. an absent token sends no header at all
    exp7 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[6][1]
    obs7 = (admin_request_headers(None), len(admin_request_headers(None)),
        tuple(admin_request_headers(None).keys()),
        AUTHORIZATION_HEADER in admin_request_headers(None))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the empty string is a present token
    exp8 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[7][1]
    obs8 = (admin_request_headers(""), len(admin_request_headers("")),
        admin_request_headers("")[AUTHORIZATION_HEADER])
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the token is carried byte for byte
    exp9 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[8][1]
    obs9 = (admin_request_headers("a b")[AUTHORIZATION_HEADER],
        admin_request_headers("  t  ")[AUTHORIZATION_HEADER],
        admin_request_headers("Bearer x")[AUTHORIZATION_HEADER])
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a successful response classifies to nothing
    exp10 = AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[9][1]
    obs10 = (classify_response(200, "ok"), classify_response(204, ""),
        classify_response(299, "x"))
    checks.append({"name": AUTHENTICATED_ADMIN_SNAPSHOT_TRANSPORT_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "authenticated-admin-snapshot-transport-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
