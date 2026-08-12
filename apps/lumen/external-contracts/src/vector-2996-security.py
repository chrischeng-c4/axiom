"""EC security case for #2996 -- exact-oracle mismatch and overclaim refusal.

Expected literals are EC-owned transcriptions of #2996 R2, R3, R4, R5, AC2,
and AC3.  The pure model must not turn an excluded nearest ID, a reordered
equal-distance result, or a candidate that disagrees with brute force into an
exact match.  Runtime load and retry claims are intentionally not asserted.
"""

from __future__ import annotations

from lumen.vector.determinism import (
    compare_to_bruteforce,
    expected_filtered_top_k,
    expected_top_k,
)
from lumen.vector.evidence import failure_context

MINIMUM_CHECKS = 8

VECTOR_2996_SECURITY_MATRIX = (
    ("filtered_oracle_never_admits_the_excluded_global_nearest_id", ("eligible-a", "eligible-b", "eligible-c", "eligible-d", "eligible-e")),
    ("filtered_oracle_is_invariant_to_equivalent_corpus_order", ("eligible-a", "eligible-b", "eligible-c", "eligible-d", "eligible-e")),
    ("reordered_equal_distance_candidate_is_a_named_mismatch", "mismatch"),
    ("reordered_equal_distance_mismatch_names_actual_ids", "actual_ids"),
    ("exact_candidate_remains_a_named_match", "match"),
    ("excluded_id_candidate_is_a_named_filtered_mismatch", "mismatch"),
    ("excluded_id_mismatch_names_actual_ids", "actual_ids"),
    ("failure_context_does_not_drop_empty_supplied_parameter_fields", ((), ())),
)


def verify_vector_2996_security() -> dict:
    checks = []

    corpus = (
        ("excluded-nearest", (0.0, 0.0)),
        ("eligible-b", (0.0, 1.0)),
        ("eligible-a", (1.0, 0.0)),
        ("eligible-d", (0.0, 2.0)),
        ("eligible-c", (2.0, 0.0)),
        ("eligible-e", (3.0, 0.0)),
    )
    allowed = frozenset({"eligible-a", "eligible-b", "eligible-c", "eligible-d", "eligible-e"})

    # 1. R2/AC2 -- a globally closer excluded vector is not a candidate for
    # the allow-set oracle; post-filtering an insufficient global K fails here.
    obs1 = expected_filtered_top_k(corpus, (0.0, 0.0), allowed, 5)
    exp1 = VECTOR_2996_SECURITY_MATRIX[0][1]
    checks.append({"name": VECTOR_2996_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R2/R3 -- an equivalent value with a different insertion order cannot
    # change the eligible ordered result.
    reordered = tuple(reversed(corpus))
    obs2 = expected_filtered_top_k(reordered, (0.0, 0.0), allowed, 5)
    exp2 = VECTOR_2996_SECURITY_MATRIX[1][1]
    checks.append({"name": VECTOR_2996_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    tied = (("id-b", (0.0, 1.0)), ("id-a", (1.0, 0.0)), ("id-c", (2.0, 0.0)))
    # 3-4. R3/R4 -- a candidate which swaps equal-distance IDs is refused as
    # a typed mismatch and names the candidate field, rather than reporting a
    # design-computed equality boolean.
    reordered_tie = compare_to_bruteforce(("id-b", "id-a", "id-c"), tied, (0.0, 0.0), None, 3)
    obs3 = reordered_tie.outcome
    exp3 = VECTOR_2996_SECURITY_MATRIX[2][1]
    checks.append({"name": VECTOR_2996_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = reordered_tie.mismatch_field
    exp4 = VECTOR_2996_SECURITY_MATRIX[3][1]
    checks.append({"name": VECTOR_2996_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R3/R4 -- the immediately neighbouring canonically ordered candidate
    # remains an explicit match; fail-closed comparison must not over-refuse.
    exact_tie = compare_to_bruteforce(expected_top_k(tied, (0.0, 0.0), 3), tied, (0.0, 0.0), None, 3)
    obs5 = exact_tie.outcome
    exp5 = VECTOR_2996_SECURITY_MATRIX[4][1]
    checks.append({"name": VECTOR_2996_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6-7. R2/R4 -- comparison applies the same exact oracle under an allow
    # set: an excluded ID cannot be claimed exact, and the result names the
    # caller-controlled candidate field which was wrong.
    excluded_candidate = compare_to_bruteforce(("excluded-nearest", "eligible-a", "eligible-b", "eligible-c", "eligible-d"), corpus, (0.0, 0.0), allowed, 5)
    obs6 = excluded_candidate.outcome
    exp6 = VECTOR_2996_SECURITY_MATRIX[5][1]
    checks.append({"name": VECTOR_2996_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = excluded_candidate.mismatch_field
    exp7 = VECTOR_2996_SECURITY_MATRIX[6][1]
    checks.append({"name": VECTOR_2996_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R5 -- an empty parameter mapping was still explicitly supplied.  The
    # context must preserve it rather than silently omitting diagnostic fields.
    empty_context = failure_context(0, {}, {})
    obs8 = (tuple(sorted(empty_context.construction_parameters.items())), tuple(sorted(empty_context.search_parameters.items())))
    exp8 = VECTOR_2996_SECURITY_MATRIX[7][1]
    checks.append({"name": VECTOR_2996_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    return {"case_id": "vector-2996-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
