"""EC security case for #2376 -- fail-closed competitor verification.

The literals here come from #2376 R2/R3/R4 and AC2/AC3.  Evidence admission
must identify a missing declaration; app-domain skips require a validated,
bounded issue and exact reproduction; an unchecked authoritative existing WI
blocks a duplicate; and shared, non-domain, and mixed failures never terminate
as a pass or tracked skip.  Each refusal is checked both for vocabulary and for
the field it names, with an explicit admitted neighbour where applicable.
"""

from __future__ import annotations

from lumen.competitor_verification.admission import (
    decide_evidence_spec,
    decide_failure_disposition,
    decide_terminal_result,
)
from lumen.competitor_verification.spec import (
    EvidenceSpec,
    FailureDispositionRequest,
    FailureOwnership,
    IssueBacking,
    PeerDeclaration,
    TerminalResultRequest,
    WorkloadDeclaration,
)
from lumen.competitor_verification.verdict import Rejection

MINIMUM_CHECKS = 34

COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX = (
    ("metrics_before_semantic_proof_is_rejected", "semantic_proof_must_precede_metrics"),
    ("semantic_order_refusal_names_semantic_proof_precedes_metrics", "semantic_proof_precedes_metrics"),
    ("semantic_first_neighbour_is_admitted", "admitted"),
    ("undeclared_appropriate_peer_is_rejected", "peer_not_declared_appropriate"),
    ("peer_refusal_names_peer_declaration", "peer.declared_appropriate"),
    ("noncomparable_workload_is_rejected", "workload_not_declared_comparable"),
    ("workload_refusal_names_workload_declaration", "workload.declared_comparable"),
    ("missing_metric_and_cost_vocabulary_is_rejected", "required_metric_vocabulary_missing"),
    ("metric_refusal_names_metric_vocabulary", "metric_vocabulary"),
    ("missing_intentional_delta_route_is_rejected", "app_domain_delta_route_missing"),
    ("delta_route_refusal_names_app_domain_delta_route", "app_domain_delta_route"),
    ("app_domain_skip_without_bounded_issue_is_rejected", "bounded_issue_required"),
    ("bounded_issue_refusal_names_issue_bounded", "issue.bounded"),
    ("app_domain_skip_without_exact_reproduction_is_rejected", "exact_reproduction_required"),
    ("reproduction_refusal_names_issue_reproduction", "issue.exact_reproduction"),
    ("unchecked_authoritative_existing_wi_blocks_duplicate_issue", "existing_wi_acceptance_check_required"),
    ("existing_wi_refusal_names_acceptance_check", "issue.authoritative_existing_wi_acceptance_checked"),
    ("checked_authoritative_existing_wi_neighbour_is_tracked_skip", "tracked_skip"),
    ("shared_terminalization_is_refused", "shared_or_non_domain_failure_requires_repair"),
    ("shared_terminal_refusal_names_ownership", "ownership"),
    ("non_domain_terminalization_is_refused", "shared_or_non_domain_failure_requires_repair"),
    ("non_domain_terminal_refusal_names_ownership", "ownership"),
    ("mixed_terminalization_is_refused", "mixed_failure_requires_split"),
    ("mixed_terminal_refusal_names_ownership", "ownership"),
    ("terminal_app_skip_without_bounded_issue_is_rejected", "bounded_issue_required"),
    ("terminal_bounded_issue_refusal_names_issue_bounded", "issue.bounded"),
    ("terminal_app_skip_without_exact_reproduction_is_rejected", "exact_reproduction_required"),
    ("terminal_reproduction_refusal_names_issue_reproduction", "issue.exact_reproduction"),
    ("terminal_app_skip_with_unvalidated_issue_is_rejected", "validated_issue_required"),
    ("terminal_validation_refusal_names_issue_validated", "issue.validated"),
    ("missing_required_evidence_fields_is_rejected", "required_evidence_fields_required"),
    ("required_evidence_fields_refusal_names_its_declaration", "required_evidence_fields"),
    ("app_domain_disposition_with_unvalidated_issue_is_rejected", "validated_issue_required"),
    ("disposition_validation_refusal_names_issue_validated", "issue.validated"),
)


def _reason(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _complete_evidence(**overrides) -> EvidenceSpec:
    values = {
        "semantic_proof_precedes_metrics": True,
        "peer": PeerDeclaration(name="opensearch", declared_appropriate=True),
        "workload": WorkloadDeclaration(name="search-10000-queries", declared_comparable=True),
        "required_evidence_fields": ("command", "work_count", "output_summary", "evidence_path", "duration", "resources"),
        "metric_vocabulary": ("throughput", "latency", "cpu", "memory", "lifecycle_overhead", "cost"),
        "intentional_deltas": ("no_feature_copying",),
        "app_domain_delta_route": "issue_backed",
    }
    values.update(overrides)
    return EvidenceSpec(**values)


def _issue(**overrides) -> IssueBacking:
    values = {
        "issue_ref": "#2377",
        "validated": True,
        "bounded": True,
        "exact_reproduction": "cargo test -p lumen --test perf_gate --test perf_gate_vs_db --test benchmark_lumen_competitor_performance_competitive",
        "authoritative_existing_wi_supplied": False,
        "authoritative_existing_wi_acceptance_checked": False,
    }
    values.update(overrides)
    return IssueBacking(**values)


def verify_competitor_verification_2376_security() -> dict:
    checks = []

    metrics_first = decide_evidence_spec(_complete_evidence(semantic_proof_precedes_metrics=False))
    # 1. R2 -- metrics cannot stand in for semantic proof.
    obs1 = _reason(metrics_first)
    exp1 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[0][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    # 2. R2 -- the order refusal tells the author which declaration is false.
    obs2 = metrics_first.field_path if isinstance(metrics_first, Rejection) else ""
    exp2 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[1][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    semantic_first = decide_evidence_spec(_complete_evidence(semantic_proof_precedes_metrics=True))
    # 3. R2 -- the adjacent semantic-first declaration remains admissible.
    obs3 = _reason(semantic_first)
    exp3 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[2][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    bad_peer = decide_evidence_spec(_complete_evidence(peer=PeerDeclaration(name="opensearch", declared_appropriate=False)))
    # 4. AC2 -- a named peer is insufficient until it is declared appropriate.
    obs4 = _reason(bad_peer)
    exp4 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[3][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    # 5. AC2 -- the typed refusal points to the peer comparability claim.
    obs5 = bad_peer.field_path if isinstance(bad_peer, Rejection) else ""
    exp5 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[4][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    bad_workload = decide_evidence_spec(_complete_evidence(workload=WorkloadDeclaration(name="search-10000-queries", declared_comparable=False)))
    # 6. AC2 -- a workload must expressly claim comparability.
    obs6 = _reason(bad_workload)
    exp6 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[5][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    # 7. AC2 -- the rejection identifies the workload declaration, not a flag.
    obs7 = bad_workload.field_path if isinstance(bad_workload, Rejection) else ""
    exp7 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[6][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    missing_metrics = decide_evidence_spec(_complete_evidence(metric_vocabulary=("throughput", "latency")))
    # 8. R2/AC2 -- a partial performance vocabulary cannot imply a comparison.
    obs8 = _reason(missing_metrics)
    exp8 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[7][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    # 9. R2/AC2 -- the missing-axis refusal names the supplied vocabulary.
    obs9 = missing_metrics.field_path if isinstance(missing_metrics, Rejection) else ""
    exp9 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[8][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    missing_delta_route = decide_evidence_spec(_complete_evidence(app_domain_delta_route=""))
    # 10. AC2 -- a declared intentional delta cannot disappear without routing.
    obs10 = _reason(missing_delta_route)
    exp10 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[9][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    # 11. AC2 -- that refusal identifies the route declaration itself.
    obs11 = missing_delta_route.field_path if isinstance(missing_delta_route, Rejection) else ""
    exp11 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[10][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    unbounded = decide_failure_disposition(FailureDispositionRequest(ownership=FailureOwnership.APP_DOMAIN_ONLY, issue=_issue(bounded=False)))
    # 12. R3/AC3 -- an app-domain issue must be bounded before it can skip.
    obs12 = _reason(unbounded)
    exp12 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[11][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    # 13. R3/AC3 -- the refusal identifies the missing bounded predicate.
    obs13 = unbounded.field_path if isinstance(unbounded, Rejection) else ""
    exp13 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[12][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    no_reproduction = decide_failure_disposition(FailureDispositionRequest(ownership=FailureOwnership.APP_DOMAIN_ONLY, issue=_issue(exact_reproduction="")))
    # 14. R3 -- an issue link without exact reproduction fails closed.
    obs14 = _reason(no_reproduction)
    exp14 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[13][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    # 15. R3 -- the operator-facing field names the missing reproduction.
    obs15 = no_reproduction.field_path if isinstance(no_reproduction, Rejection) else ""
    exp15 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[14][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    unchecked_existing = decide_failure_disposition(FailureDispositionRequest(ownership=FailureOwnership.APP_DOMAIN_ONLY, issue=_issue(authoritative_existing_wi_supplied=True, authoritative_existing_wi_acceptance_checked=False)))
    # 16. R4 -- a supplied authoritative WI must be acceptance-checked first.
    obs16 = _reason(unchecked_existing)
    exp16 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[15][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    # 17. R4 -- that duplicate-prevention refusal identifies its prerequisite.
    obs17 = unchecked_existing.field_path if isinstance(unchecked_existing, Rejection) else ""
    exp17 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[16][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    checked_existing = decide_failure_disposition(FailureDispositionRequest(ownership=FailureOwnership.APP_DOMAIN_ONLY, issue=_issue(authoritative_existing_wi_supplied=True, authoritative_existing_wi_acceptance_checked=True)))
    # 18. R4 -- the neighbouring acceptance-checked WI permits the bounded skip.
    obs18 = checked_existing.action if not isinstance(checked_existing, Rejection) else checked_existing.reason.value
    exp18 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[17][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    shared_terminal = decide_terminal_result(TerminalResultRequest(journey_completed=False, ownership=FailureOwnership.SHARED, issue=_issue()))
    # 19. R3/AC3 -- a shared failure cannot be terminalized by naming an issue.
    obs19 = _reason(shared_terminal)
    exp19 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[18][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})
    # 20. R3/AC3 -- terminal refusal names the ownership that must be repaired.
    obs20 = shared_terminal.field_path if isinstance(shared_terminal, Rejection) else ""
    exp20 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[19][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    non_domain_terminal = decide_terminal_result(TerminalResultRequest(journey_completed=False, ownership=FailureOwnership.NON_DOMAIN, issue=_issue()))
    # 21. R3/AC3 -- non-domain work has the same nonterminal boundary.
    obs21 = _reason(non_domain_terminal)
    exp21 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[20][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})
    # 22. R3/AC3 -- it also identifies ownership as the repair boundary.
    obs22 = non_domain_terminal.field_path if isinstance(non_domain_terminal, Rejection) else ""
    exp22 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[21][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})

    mixed_terminal = decide_terminal_result(TerminalResultRequest(journey_completed=False, ownership=FailureOwnership.MIXED, issue=_issue()))
    # 23. R4/AC3 -- a mixed record cannot hide its shared slice behind a skip.
    obs23 = _reason(mixed_terminal)
    exp23 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[22][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})
    # 24. R4/AC3 -- the mixed terminal refusal likewise names ownership.
    obs24 = mixed_terminal.field_path if isinstance(mixed_terminal, Rejection) else ""
    exp24 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[23][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[23][0], "expected": exp24, "observed": obs24, "passed": obs24 == exp24})

    terminal_unbounded = decide_terminal_result(TerminalResultRequest(journey_completed=False, ownership=FailureOwnership.APP_DOMAIN_ONLY, issue=_issue(bounded=False)))
    # 25. R3/AC3 -- terminal admission repeats the bounded-issue guard.
    obs25 = _reason(terminal_unbounded)
    exp25 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[24][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[24][0], "expected": exp25, "observed": obs25, "passed": obs25 == exp25})
    # 26. R3/AC3 -- it identifies the bounded predicate rather than failing vaguely.
    obs26 = terminal_unbounded.field_path if isinstance(terminal_unbounded, Rejection) else ""
    exp26 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[25][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[25][0], "expected": exp26, "observed": obs26, "passed": obs26 == exp26})

    terminal_no_reproduction = decide_terminal_result(TerminalResultRequest(journey_completed=False, ownership=FailureOwnership.APP_DOMAIN_ONLY, issue=_issue(exact_reproduction="")))
    # 27. R3/AC3 -- terminal admission likewise requires exact reproduction.
    obs27 = _reason(terminal_no_reproduction)
    exp27 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[26][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[26][0], "expected": exp27, "observed": obs27, "passed": obs27 == exp27})
    # 28. R3/AC3 -- the reproduction guard names the missing issue field.
    obs28 = terminal_no_reproduction.field_path if isinstance(terminal_no_reproduction, Rejection) else ""
    exp28 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[27][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[27][0], "expected": exp28, "observed": obs28, "passed": obs28 == exp28})

    terminal_unvalidated = decide_terminal_result(TerminalResultRequest(journey_completed=False, ownership=FailureOwnership.APP_DOMAIN_ONLY, issue=_issue(validated=False)))
    # 29. AC3 -- merely naming a bounded issue cannot replace validation.
    obs29 = _reason(terminal_unvalidated)
    exp29 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[28][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[28][0], "expected": exp29, "observed": obs29, "passed": obs29 == exp29})
    # 30. AC3 -- the refusal identifies the validation predicate still false.
    obs30 = terminal_unvalidated.field_path if isinstance(terminal_unvalidated, Rejection) else ""
    exp30 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[29][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[29][0], "expected": exp30, "observed": obs30, "passed": obs30 == exp30})

    missing_required_evidence_fields = decide_evidence_spec(_complete_evidence(required_evidence_fields=()))
    # 31. R2 -- an evidence specification must name its required evidence.
    obs31 = _reason(missing_required_evidence_fields)
    exp31 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[30][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[30][0], "expected": exp31, "observed": obs31, "passed": obs31 == exp31})
    # 32. R2 -- the refusal identifies the missing evidence declaration.
    obs32 = missing_required_evidence_fields.field_path if isinstance(missing_required_evidence_fields, Rejection) else ""
    exp32 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[31][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[31][0], "expected": exp32, "observed": obs32, "passed": obs32 == exp32})

    disposition_unvalidated = decide_failure_disposition(FailureDispositionRequest(ownership=FailureOwnership.APP_DOMAIN_ONLY, issue=_issue(validated=False)))
    # 33. R3 -- disposition cannot create a tracked skip from an unvalidated issue.
    obs33 = _reason(disposition_unvalidated)
    exp33 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[32][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[32][0], "expected": exp33, "observed": obs33, "passed": obs33 == exp33})
    # 34. R3 -- the disposition refusal names the validation predicate.
    obs34 = disposition_unvalidated.field_path if isinstance(disposition_unvalidated, Rejection) else ""
    exp34 = COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[33][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_SECURITY_MATRIX[33][0], "expected": exp34, "observed": obs34, "passed": obs34 == exp34})

    return {
        "case_id": "competitor-verification-2376-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
