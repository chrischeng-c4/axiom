"""EC security case for #2944 -- cross-shard query refusal and fail-closed behavior.

Expected literals are EC-owned transcriptions of #2944 R1/R4/R5 and AC1-AC3.
Each refusal checks vocabulary, the named offending value, and a neighbouring
admitted input where that entry point has one.
"""

from __future__ import annotations

from lumen.query.consistency import decide_completion, decide_generation_outcome
from lumen.query.merge import compare_to_canonical, decide_lexical_protocol, merge_hybrid_results
from lumen.query.spec import CanonicalResult, HybridQueryRequest, ShardOutcome, ShardResult, VectorCandidate
from lumen.query.verdict import Rejection

MINIMUM_CHECKS = 18

QUERY_2944_SECURITY_MATRIX = (
    ("raw_shard_local_lexical_scores_are_rejected", "shard_local_scores_not_globally_comparable"),
    ("raw_shard_local_refusal_names_scoring_protocol", "scoring_phases"),
    ("global_statistics_neighbour_remains_admitted", "admitted"),
    ("missing_required_shard_is_not_complete", "missing_required_shard"),
    ("missing_shard_refusal_names_the_shard", "shard-b"),
    ("missing_shard_refusal_carries_pinned_generation", 9),
    ("timed_out_required_shard_is_not_complete", "shard_timeout"),
    ("timed_out_shard_refusal_names_the_shard", "shard-b"),
    ("failed_required_shard_is_not_complete", "shard_failed"),
    ("complete_neighbour_requires_all_shards", "complete"),
    ("generation_change_is_retryable", "retryable_generation_change"),
    ("generation_change_refusal_names_observed_generations", "observed_generations"),
    ("matching_generation_neighbour_completes", "complete"),
    ("canonical_score_mismatch_is_named", "scores"),
    ("canonical_order_mismatch_is_named", "identities"),
    ("canonical_filter_mismatch_is_named", "filters"),
    ("canonical_cursor_mismatch_is_named", "cursor_position"),
    ("hybrid_generation_mismatch_is_retryable_not_a_page", "retryable_generation_change"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else verdict.outcome


def verify_query_2944_security() -> dict:
    checks = []

    local = decide_lexical_protocol({"statistics_scope": "shard", "scoring_phases": 1})
    # 1-3. R1 -- raw shard-local BM25 comparison is refused, named, and does not over-refuse global statistics.
    obs1 = _outcome(local); exp1 = QUERY_2944_SECURITY_MATRIX[0][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = local.field_path if isinstance(local, Rejection) else ""; exp2 = QUERY_2944_SECURITY_MATRIX[1][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = _outcome(decide_lexical_protocol({"statistics_scope": "global", "scoring_phases": 1})); exp3 = QUERY_2944_SECURITY_MATRIX[2][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    missing = decide_completion(9, (ShardOutcome(shard_id="shard-a", status="complete", generation=9), ShardOutcome(shard_id="shard-b", status="missing", generation=9)))
    # 4-6. R4/AC2 -- a missing participant cannot become an apparently complete result.
    obs4 = _outcome(missing); exp4 = QUERY_2944_SECURITY_MATRIX[3][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = missing.shard_id if isinstance(missing, Rejection) else ""; exp5 = QUERY_2944_SECURITY_MATRIX[4][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = missing.pinned_generation if isinstance(missing, Rejection) else -1; exp6 = QUERY_2944_SECURITY_MATRIX[5][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    timeout = decide_completion(9, (ShardOutcome(shard_id="shard-a", status="complete", generation=9), ShardOutcome(shard_id="shard-b", status="timeout", generation=9)))
    # 7-8. R4/AC2 -- timeout has its own typed, shard-identifying non-success outcome.
    obs7 = _outcome(timeout); exp7 = QUERY_2944_SECURITY_MATRIX[6][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = timeout.shard_id if isinstance(timeout, Rejection) else ""; exp8 = QUERY_2944_SECURITY_MATRIX[7][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    failed = decide_completion(9, (ShardOutcome(shard_id="shard-a", status="complete", generation=9), ShardOutcome(shard_id="shard-b", status="failed", generation=9)))
    complete = decide_completion(9, (ShardOutcome(shard_id="shard-a", status="complete", generation=9), ShardOutcome(shard_id="shard-b", status="complete", generation=9)))
    # 9-10. R4/AC2 -- an explicit shard failure is non-success, while the all-complete neighbour admits.
    obs9 = _outcome(failed); exp9 = QUERY_2944_SECURITY_MATRIX[8][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = _outcome(complete); exp10 = QUERY_2944_SECURITY_MATRIX[9][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    cutover = decide_generation_outcome(9, (9, 10))
    # 11-13. R5/AC3 -- split cutover is named retryable failure; matching observations complete.
    obs11 = _outcome(cutover); exp11 = QUERY_2944_SECURITY_MATRIX[10][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = cutover.field_path if isinstance(cutover, Rejection) else ""; exp12 = QUERY_2944_SECURITY_MATRIX[11][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = _outcome(decide_generation_outcome(9, (9, 9))); exp13 = QUERY_2944_SECURITY_MATRIX[12][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    merged = CanonicalResult(identities=("doc-a", "doc-b"), scores=(0.97, 0.91), filters=("tenant:acme",), cursor_position=2)
    # 14-17. AC1 -- every canonical dimension produces its own named mismatch, not an equality flag.
    obs14 = compare_to_canonical(merged, CanonicalResult(identities=("doc-a", "doc-b"), scores=(0.98, 0.91), filters=("tenant:acme",), cursor_position=2)).mismatch_field; exp14 = QUERY_2944_SECURITY_MATRIX[13][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    obs15 = compare_to_canonical(merged, CanonicalResult(identities=("doc-b", "doc-a"), scores=(0.97, 0.91), filters=("tenant:acme",), cursor_position=2)).mismatch_field; exp15 = QUERY_2944_SECURITY_MATRIX[14][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    obs16 = compare_to_canonical(merged, CanonicalResult(identities=("doc-a", "doc-b"), scores=(0.97, 0.91), filters=("tenant:other",), cursor_position=2)).mismatch_field; exp16 = QUERY_2944_SECURITY_MATRIX[15][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    obs17 = compare_to_canonical(merged, CanonicalResult(identities=("doc-a", "doc-b"), scores=(0.97, 0.91), filters=("tenant:acme",), cursor_position=3)).mismatch_field; exp17 = QUERY_2944_SECURITY_MATRIX[16][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    hybrid_request = HybridQueryRequest(page_offset=0, page_size=2, filters=("tenant:acme",), fusion="rrf", cursor=None, pinned_generation=9)
    split_results = (ShardResult(shard_id="shard-a", generation=9, candidates=(VectorCandidate(shard_id="shard-a", identity="doc-a", score=0.97, filters=("tenant:acme",)),)), ShardResult(shard_id="shard-b", generation=10, candidates=(VectorCandidate(shard_id="shard-b", identity="doc-b", score=0.91, filters=("tenant:acme",)),)))
    split_page = merge_hybrid_results(hybrid_request, split_results)
    # 18. AC3 -- hybrid merge itself cannot turn a generation transition into a page.
    obs18 = _outcome(split_page); exp18 = QUERY_2944_SECURITY_MATRIX[17][1]
    checks.append({"name": QUERY_2944_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    return {"case_id": "query-2944-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
