from __future__ import annotations

from transport_h2c.infrastructure.client_pool import (
    ClientSettings,
    builder_settings,
    client_index,
    handout,
    next_cursor,
    pool_for_concurrency,
    pool_of,
)

MINIMUM_CHECKS = 14

PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX = (
    ("a_pool_is_built_at_the_requested_size", 4),
    ("a_zero_request_still_yields_one_client", 1),
    ("a_negative_request_still_yields_one_client", 1),
    ("handouts_rotate_round_robin_across_the_pool", (0, 1, 2, 0, 1, 2)),
    ("a_handout_resumes_from_the_cursor_it_was_given", (2, 0, 1)),
    ("a_pool_of_one_hands_out_the_same_client", (0, 0, 0, 0)),
    ("an_empty_handout_is_empty", ()),
    ("a_negative_count_hands_out_nothing", ()),
    ("the_cursor_advances_by_one", (1, 2, 3)),
    ("the_client_index_wraps_at_the_pool_size", (0, 1, 2, 0)),
    ("sizing_a_pool_for_a_concurrency_uses_the_shared_heuristic", 5),
    ("a_small_concurrency_pool_still_holds_one_client", 1),
    ("a_client_is_built_with_prior_knowledge_and_nothing_else", (("http2_prior_knowledge", True),)),
    ("settings_that_are_set_are_carried_into_the_builder", (("http2_prior_knowledge", True), ("timeout_seconds", 2.5), ("user_agent", "lumen"))),
)


def verify_prior_knowledge_client_helpers_behavior() -> dict:
    checks = []

    # 1. a_pool_is_built_at_the_requested_size
    exp1 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[0][1]
    obs1 = pool_of(4).size
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_zero_request_still_yields_one_client
    exp2 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[1][1]
    obs2 = pool_of(0).size
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_negative_request_still_yields_one_client
    exp3 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[2][1]
    obs3 = pool_of(-3).size
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. handouts_rotate_round_robin_across_the_pool
    exp4 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[3][1]
    obs4 = handout(pool_of(3), 0, 6)
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. a_handout_resumes_from_the_cursor_it_was_given
    exp5 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[4][1]
    obs5 = handout(pool_of(3), 2, 3)
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. a_pool_of_one_hands_out_the_same_client
    exp6 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[5][1]
    obs6 = handout(pool_of(1), 0, 4)
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. an_empty_handout_is_empty
    exp7 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[6][1]
    obs7 = handout(pool_of(3), 0, 0)
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. a_negative_count_hands_out_nothing
    exp8 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[7][1]
    obs8 = handout(pool_of(3), 0, -5)
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. the_cursor_advances_by_one
    exp9 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[8][1]
    obs9 = (next_cursor(0), next_cursor(1), next_cursor(2))
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. the_client_index_wraps_at_the_pool_size
    exp10 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[9][1]
    pool10 = pool_of(3)
    res10 = []
    for cursor in (0, 1, 2, 3):
        res10.append(client_index(pool10, cursor))
    obs10 = tuple(res10)
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. sizing_a_pool_for_a_concurrency_uses_the_shared_heuristic
    exp11 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[10][1]
    obs11 = pool_for_concurrency(128, 64).size
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. a_small_concurrency_pool_still_holds_one_client
    exp12 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[11][1]
    obs12 = pool_for_concurrency(0, 64).size
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. a_client_is_built_with_prior_knowledge_and_nothing_else
    exp13 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[12][1]
    obs13 = tuple(sorted(builder_settings(ClientSettings()).items()))
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. settings_that_are_set_are_carried_into_the_builder
    exp14 = PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[13][1]
    settings14 = ClientSettings(timeout_seconds=2.5, user_agent="lumen")
    obs14 = tuple(sorted(builder_settings(settings14).items()))
    checks.append(
        {
            "name": PRIOR_KNOWLEDGE_CLIENT_HELPERS_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    return {
        "case_id": "prior-knowledge-client-helpers-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
