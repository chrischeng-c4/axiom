"""EC behavior case for #2996 -- exact vector oracle and deterministic ordering.

Every expected value below is an EC-owned literal transcribed from #2996 R2,
R3, R4, R5, AC2, and AC3.  This case defines the pure exact-distance oracle;
it does not claim to exercise HNSW traversal, snapshot I/O, host load, or the
50-run binary gate, which belong to the Rust/runtime stage.
"""

from __future__ import annotations

from lumen.vector.determinism import (
    compare_to_bruteforce,
    expected_filtered_top_k,
    expected_top_k,
)
from lumen.vector.evidence import failure_context

MINIMUM_CHECKS = 10

VECTOR_2996_BEHAVIOR_MATRIX = (
    ("filtered_oracle_returns_the_five_nearest_allowed_ids", ("allow-a", "allow-b", "allow-c", "allow-d", "allow-e")),
    ("filtered_oracle_breaks_equal_distances_by_external_id", ("tie-a", "tie-b")),
    ("pre_restore_oracle_returns_a_defined_ordered_top_k", ("id-a", "id-b", "id-c")),
    ("restored_equivalent_corpus_returns_the_same_ordered_top_k", ("id-a", "id-b", "id-c")),
    ("bruteforce_comparison_exposes_the_filtered_expected_ids", ("allow-a", "allow-b", "allow-c", "allow-d", "allow-e")),
    ("bruteforce_comparison_retains_the_supplied_exact_candidate_ids", ("allow-a", "allow-b", "allow-c", "allow-d", "allow-e")),
    ("bruteforce_comparison_exposes_the_unfiltered_expected_ids", ("id-a", "id-b", "id-c")),
    ("failure_context_retains_the_supplied_seed", 2996),
    ("failure_context_retains_the_construction_parameters", (("ef_construction", 200), ("m", 16))),
    ("failure_context_retains_the_search_parameters", (("ef_search", 64),)),
)


def verify_vector_2996_behavior() -> dict:
    checks = []

    filtered_corpus = (
        ("excluded-nearest", (0.0, 0.0)),
        ("allow-b", (0.0, 1.0)),
        ("allow-a", (1.0, 0.0)),
        ("allow-d", (0.0, 2.0)),
        ("allow-c", (2.0, 0.0)),
        ("allow-e", (3.0, 0.0)),
        ("allow-f", (4.0, 0.0)),
    )
    allowed_ids = frozenset({"allow-a", "allow-b", "allow-c", "allow-d", "allow-e", "allow-f"})

    # 1. R2/AC2 -- the global nearest is excluded, so an exact filtered oracle
    # must still return five eligible neighbours rather than post-filtering it.
    obs1 = expected_filtered_top_k(filtered_corpus, (0.0, 0.0), allowed_ids, 5)
    exp1 = VECTOR_2996_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": VECTOR_2996_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R2/AC2 -- equal exact distances use a specified ID tie-break, not
    # insertion order, hash iteration, or a backend-selected order.
    tie_corpus = (("tie-b", (1.0, 0.0)), ("tie-a", (0.0, 1.0)))
    obs2 = expected_filtered_top_k(tie_corpus, (0.0, 0.0), frozenset({"tie-a", "tie-b"}), 2)
    exp2 = VECTOR_2996_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": VECTOR_2996_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    before_restore = (("id-c", (2.0, 0.0)), ("id-b", (0.0, 1.0)), ("id-a", (1.0, 0.0)), ("id-d", (3.0, 0.0)))
    # 3. R3/AC3 -- the pre-restore value has a defined ordered top-K including
    # an equal-distance pair.
    obs3 = expected_top_k(before_restore, (0.0, 0.0), 3)
    exp3 = VECTOR_2996_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": VECTOR_2996_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    restored_equivalent = (("id-d", (3.0, 0.0)), ("id-a", (1.0, 0.0)), ("id-b", (0.0, 1.0)), ("id-c", (2.0, 0.0)))
    # 4. R3/AC3 -- equivalent restored values need not retain insertion order,
    # but they must retain this same public ordered result.
    obs4 = expected_top_k(restored_equivalent, (0.0, 0.0), 3)
    exp4 = VECTOR_2996_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": VECTOR_2996_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5-6. R2/R4 -- comparison has its own allow-set entry point: it exposes
    # both the independently derived exact oracle and the candidate it judged.
    filtered_comparison = compare_to_bruteforce(obs1, filtered_corpus, (0.0, 0.0), allowed_ids, 5)
    obs5 = filtered_comparison.expected_ids
    exp5 = VECTOR_2996_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": VECTOR_2996_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = filtered_comparison.actual_ids
    exp6 = VECTOR_2996_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": VECTOR_2996_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R3/R4 -- the comparison path also derives exact unfiltered order; it
    # cannot be a filtered-only fixture that misses restore tie semantics.
    top_comparison = compare_to_bruteforce(obs3, before_restore, (0.0, 0.0), None, 3)
    obs7 = top_comparison.expected_ids
    exp7 = VECTOR_2996_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": VECTOR_2996_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    context = failure_context(2996, {"m": 16, "ef_construction": 200}, {"ef_search": 64})
    # 8-10. R5 -- failure evidence is a value carrying each caller-supplied
    # diagnostic dimension, rather than a retry flag or a formatted log line.
    obs8 = context.seed
    exp8 = VECTOR_2996_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": VECTOR_2996_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = tuple(sorted(context.construction_parameters.items()))
    exp9 = VECTOR_2996_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": VECTOR_2996_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = tuple(sorted(context.search_parameters.items()))
    exp10 = VECTOR_2996_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": VECTOR_2996_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    return {"case_id": "vector-2996-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
