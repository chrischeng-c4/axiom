from __future__ import annotations

from transport_h2c.infrastructure.client_pool import (
    ClientSettings,
    builder_settings,
    handout,
    next_cursor,
    pool_of,
)

MINIMUM_CHECKS = 12

PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX = (
    ("no_pool_size_request_can_produce_an_empty_pool", (1, 1, 1, 1, 2)),
    ("a_long_handout_never_leaves_the_pool_and_keeps_rotating", (0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2)),
    ("no_client_is_starved_over_a_full_rotation", (4, 4, 4)),
    ("a_handout_from_a_large_cursor_still_stays_in_range", (1, 2, 0)),
    ("the_cursor_strictly_advances_and_never_stalls", (1, 2, 3, 4, 5)),
    ("consecutive_single_handouts_visit_different_clients", (0, 1)),
    ("only_a_pool_of_one_may_repeat_within_a_rotation", ((0, 0, 0), (0, 1, 0))),
    ("prior_knowledge_is_declared_whatever_the_settings", (True, True, True)),
    ("an_unset_timeout_is_omitted_not_sent_as_null", False),
    ("an_unset_user_agent_is_omitted_not_sent_as_null", False),
    ("an_explicit_zero_timeout_is_still_carried", (True, 0.0)),
    ("the_builder_emits_no_unexpected_key", ("http2_prior_knowledge", "timeout_seconds", "user_agent")),
)


def verify_prior_knowledge_client_helpers_security() -> dict:
    checks = []

    # 1. no_pool_size_request_can_produce_an_empty_pool
    exp1 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[0][1]
    requests1 = (-100, -1, 0, 1, 2)
    res1 = []
    for requested in requests1:
        res1.append(pool_of(requested).size)
    obs1 = tuple(res1)
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_long_handout_never_leaves_the_pool_and_keeps_rotating
    exp2 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[1][1]
    obs2 = handout(pool_of(3), 0, 12)
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. no_client_is_starved_over_a_full_rotation
    exp3 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[2][1]
    rotation3 = handout(pool_of(3), 0, 12)
    res3 = []
    for client in (0, 1, 2):
        res3.append(rotation3.count(client))
    obs3 = tuple(res3)
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. a_handout_from_a_large_cursor_still_stays_in_range
    exp4 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[3][1]
    obs4 = handout(pool_of(3), 1000000, 3)
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. the_cursor_strictly_advances_and_never_stalls
    exp5 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[4][1]
    cursor5 = 0
    res5 = []
    for _ in range(5):
        cursor5 = next_cursor(cursor5)
        res5.append(cursor5)
    obs5 = tuple(res5)
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. consecutive_single_handouts_visit_different_clients
    exp6 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[5][1]
    pool6 = pool_of(3)
    cursor6 = 0
    first6 = handout(pool6, cursor6, 1)[0]
    cursor6 = next_cursor(cursor6)
    second6 = handout(pool6, cursor6, 1)[0]
    obs6 = (first6, second6)
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. only_a_pool_of_one_may_repeat_within_a_rotation
    exp7 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[6][1]
    obs7 = (handout(pool_of(1), 0, 3), handout(pool_of(2), 0, 3))
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. prior_knowledge_is_declared_whatever_the_settings
    exp8 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[7][1]
    settings8 = (
        ClientSettings(),
        ClientSettings(timeout_seconds=1.0),
        ClientSettings(user_agent="lumen"),
    )
    res8 = []
    for settings in settings8:
        res8.append(builder_settings(settings)["http2_prior_knowledge"])
    obs8 = tuple(res8)
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. an_unset_timeout_is_omitted_not_sent_as_null
    exp9 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[8][1]
    obs9 = "timeout_seconds" in builder_settings(ClientSettings())
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. an_unset_user_agent_is_omitted_not_sent_as_null
    exp10 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[9][1]
    obs10 = "user_agent" in builder_settings(ClientSettings())
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. an_explicit_zero_timeout_is_still_carried
    exp11 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[10][1]
    built11 = builder_settings(ClientSettings(timeout_seconds=0.0))
    obs11 = ("timeout_seconds" in built11, built11["timeout_seconds"])
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. the_builder_emits_no_unexpected_key
    exp12 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[11][1]
    settings12 = ClientSettings(timeout_seconds=1.0, user_agent="lumen")
    obs12 = tuple(sorted(builder_settings(settings12).keys()))
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    return {
        "case_id": "prior-knowledge-client-helpers-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
