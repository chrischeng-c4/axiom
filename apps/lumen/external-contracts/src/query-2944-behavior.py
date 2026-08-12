"""EC behavior case for #2944 -- deterministic complete cross-shard queries.

Every expected value is an EC-owned literal transcribed from #2944 R1-R5 and
AC1/AC3.  The case drives only the frozen pure query design; it does not claim
to prove live index, transport, timeout, or split execution.
"""

from __future__ import annotations

from lumen.query.consistency import decide_generation_outcome
from lumen.query.merge import (
    compare_to_canonical,
    decide_lexical_protocol,
    merge_hybrid_results,
    merge_vector_candidates,
    plan_vector_candidates,
)
from lumen.query.spec import (
    CanonicalResult,
    HybridQueryRequest,
    QueryRequest,
    ShardResult,
    VectorCandidate,
)
from lumen.query.verdict import Rejection

MINIMUM_CHECKS = 13

QUERY_2944_BEHAVIOR_MATRIX = (
    ("global_statistics_lexical_protocol_is_admitted", "admitted"),
    ("two_phase_lexical_protocol_is_admitted", "admitted"),
    ("vector_depth_covers_the_requested_global_page_per_shard", 4),
    ("vector_merge_returns_the_global_top_k_identities", ("doc-a", "doc-b", "doc-c")),
    ("vector_merge_returns_descending_global_scores", (0.97, 0.91, 0.88)),
    ("hybrid_merge_returns_one_filtered_deduplicated_page", ("doc-a", "doc-b")),
    ("hybrid_merge_declares_reciprocal_rank_fusion", "rrf"),
    ("hybrid_ties_break_by_identity", ("doc-a", "doc-b")),
    ("hybrid_cursor_encodes_the_next_continuation_position", 2),
    ("canonical_score_order_filter_and_cursor_fixture_matches", "equal"),
    ("pinned_generation_completes_when_every_observation_matches", "complete"),
    ("generation_transition_returns_retryable_not_partial", "retryable_generation_change"),
    ("generation_consistent_hybrid_result_has_unique_canonical_membership", ("doc-a", "doc-b")),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else verdict.outcome


def verify_query_2944_behavior() -> dict:
    checks = []

    global_statistics = decide_lexical_protocol({"statistics_scope": "global", "scoring_phases": 1})
    # 1. R1 -- global corpus/term statistics is an admitted lexical protocol.
    obs1 = _outcome(global_statistics); exp1 = QUERY_2944_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": QUERY_2944_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    two_phase = decide_lexical_protocol({"statistics_scope": "shard", "scoring_phases": 2})
    # 2. R1 -- the alternate exact protocol explicitly has two scoring phases.
    obs2 = _outcome(two_phase); exp2 = QUERY_2944_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": QUERY_2944_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    vector_request = QueryRequest(kind="vector", page_offset=1, page_size=3, filters=("tenant:acme",), cursor=None)
    budget = plan_vector_candidates(vector_request, shard_count=2)
    # 3. R2 -- offset plus page size is fetched from every participating shard.
    obs3 = budget.per_shard_depth; exp3 = QUERY_2944_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": QUERY_2944_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    candidates = (
        VectorCandidate(shard_id="shard-a", identity="doc-a", score=0.97, filters=("tenant:acme",)),
        VectorCandidate(shard_id="shard-b", identity="doc-b", score=0.91, filters=("tenant:acme",)),
        VectorCandidate(shard_id="shard-a", identity="doc-c", score=0.88, filters=("tenant:acme",)),
        VectorCandidate(shard_id="shard-b", identity="doc-z", score=0.11, filters=("tenant:other",)),
    )
    vector_page = merge_vector_candidates(vector_request, candidates)
    # 4. R2 -- fixed shard candidates yield one deterministic global top-k.
    obs4 = vector_page.identities; exp4 = QUERY_2944_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": QUERY_2944_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R2 -- the corresponding globally ordered score values are retained.
    obs5 = vector_page.scores; exp5 = QUERY_2944_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": QUERY_2944_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    hybrid_request = HybridQueryRequest(page_offset=0, page_size=2, filters=("tenant:acme",), fusion="rrf", cursor=None, pinned_generation=9)
    hybrid_results = (
        ShardResult(shard_id="shard-a", generation=9, candidates=(
            VectorCandidate(shard_id="shard-a", identity="doc-b", score=0.90, filters=("tenant:acme",)),
            VectorCandidate(shard_id="shard-a", identity="doc-a", score=0.90, filters=("tenant:acme",)),
        )),
        ShardResult(shard_id="shard-b", generation=9, candidates=(
            VectorCandidate(shard_id="shard-b", identity="doc-a", score=0.89, filters=("tenant:acme",)),
            VectorCandidate(shard_id="shard-b", identity="doc-z", score=0.99, filters=("tenant:other",)),
        )),
    )
    hybrid_page = merge_hybrid_results(hybrid_request, hybrid_results)
    # 6. R3 -- filters and deduplication precede the public page.
    obs6 = hybrid_page.filtered_identities; exp6 = QUERY_2944_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": QUERY_2944_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R3 -- the page reports the explicitly selected fusion protocol.
    obs7 = hybrid_page.fusion; exp7 = QUERY_2944_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": QUERY_2944_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R3 -- equal fused scores have a defined identity tie-break.
    obs8 = hybrid_page.identities; exp8 = QUERY_2944_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": QUERY_2944_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R3 -- continuation is a value, not an opaque promise of pagination.
    obs9 = hybrid_page.cursor.position; exp9 = QUERY_2944_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": QUERY_2944_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    canonical = CanonicalResult(identities=("doc-a", "doc-b"), scores=(0.90, 0.90), filters=("tenant:acme",), cursor_position=2)
    compared = compare_to_canonical(hybrid_page, canonical)
    # 10. AC1 -- score, order, filter, and continuation match the supplied oracle fixture.
    obs10 = compared.outcome; exp10 = QUERY_2944_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": QUERY_2944_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    complete = decide_generation_outcome(9, (9, 9))
    # 11. R5 -- matching observations complete at the originally pinned generation.
    obs11 = _outcome(complete); exp11 = QUERY_2944_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": QUERY_2944_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    changed = decide_generation_outcome(9, (9, 10))
    # 12. R5/AC3 -- cutover is retryable, never a fabricated partial completion.
    obs12 = _outcome(changed); exp12 = QUERY_2944_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": QUERY_2944_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. AC3 -- the generation-consistent result has the required unique canonical membership.
    obs13 = hybrid_page.canonical_membership; exp13 = QUERY_2944_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": QUERY_2944_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    return {"case_id": "query-2944-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
