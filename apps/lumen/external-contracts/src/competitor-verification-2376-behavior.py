"""EC behavior case for #2376 -- competitor-verification admission.

Every expected value below is an EC-owned literal transcribed from #2376:
R2/AC2 require semantic proof before comparable peer/workload metrics, the
complete metric and cost vocabulary, and an explicit intentional-delta route;
R3/R4 distinguish shared repair from the one bounded app-domain skip and split
mixed ownership; and AC3 permits only ``passed`` or ``tracked_skip(#issue)``.
The case drives the pure decision model only: command execution, measurements,
tracker mutation, and cleanup remain runtime-stage evidence.
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

MINIMUM_CHECKS = 15

COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX = (
    ("complete_evidence_spec_is_admitted", "admitted"),
    ("admitted_evidence_retains_declared_appropriate_peer", "opensearch"),
    ("admitted_evidence_retains_declared_comparable_workload", "search-10000-queries"),
    ("admitted_evidence_retains_required_metric_and_cost_vocabulary", ("throughput", "latency", "cpu", "memory", "lifecycle_overhead", "cost")),
    ("admitted_evidence_retains_explicit_intentional_delta", "no_feature_copying"),
    ("missing_intentional_delta_is_rejected", "intentional_deltas_required"),
    ("admitted_evidence_retains_app_domain_delta_route", "issue_backed"),
    ("shared_failure_requires_repair_and_rerun", "repair_and_rerun"),
    ("non_domain_failure_requires_repair_and_rerun", "repair_and_rerun"),
    ("bounded_app_domain_failure_is_tracked_skip", "tracked_skip"),
    ("tracked_skip_retains_its_exact_issue_reference", "#2377"),
    ("mixed_failure_keeps_its_shared_slice_on_repair_and_rerun", "repair_and_rerun"),
    ("mixed_failure_keeps_its_app_domain_slice_as_tracked_skip", "tracked_skip"),
    ("completed_successful_journey_is_passed", "passed"),
    ("validated_app_domain_terminal_is_tracked_skip_with_issue", "tracked_skip(#2377)"),
)


def _outcome(verdict) -> str:
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


def _bounded_issue(**overrides) -> IssueBacking:
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


def verify_competitor_verification_2376_behavior() -> dict:
    checks = []

    admitted_evidence = decide_evidence_spec(_complete_evidence())

    # 1. R2/AC2 -- semantic proof, comparability, evidence names, metrics, and
    #    an intentional delta route together make an evidence spec admissible.
    obs1 = _outcome(admitted_evidence)
    exp1 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. AC2 -- admission carries the declared appropriate peer, rather than a
    #    design-selected competitor that cannot be audited from the record.
    obs2 = admitted_evidence.spec.peer.name if not isinstance(admitted_evidence, Rejection) else "rejected"
    exp2 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. AC2 -- the comparable workload is explicit and preserved as well.
    obs3 = admitted_evidence.spec.workload.name if not isinstance(admitted_evidence, Rejection) else "rejected"
    exp3 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R2/AC2 -- comparison requires every named performance and cost axis.
    obs4 = admitted_evidence.spec.metric_vocabulary if not isinstance(admitted_evidence, Rejection) else ()
    exp4 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R2/AC2 -- an intentional difference is recorded, never silently
    #    converted into a requirement to copy the peer's feature.
    obs5 = admitted_evidence.spec.intentional_deltas[0] if not isinstance(admitted_evidence, Rejection) else "rejected"
    exp5 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    missing_intentional_delta = decide_evidence_spec(_complete_evidence(intentional_deltas=()))
    # 6. R2/AC2 -- intentional differences must be declared explicitly.
    obs6 = _outcome(missing_intentional_delta)
    exp6 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. AC2 -- a material app-domain delta has an explicit route.
    obs7 = admitted_evidence.spec.app_domain_delta_route if not isinstance(admitted_evidence, Rejection) else "rejected"
    exp7 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    shared = decide_failure_disposition(FailureDispositionRequest(ownership=FailureOwnership.SHARED, issue=_bounded_issue()))
    # 8. R3 -- shared ownership cannot be recorded as an app-domain skip.
    obs8 = shared.action if not isinstance(shared, Rejection) else shared.reason.value
    exp8 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    non_domain = decide_failure_disposition(FailureDispositionRequest(ownership=FailureOwnership.NON_DOMAIN, issue=_bounded_issue()))
    # 9. R3 -- non-domain ownership is equally repair-and-rerun work.
    obs9 = non_domain.action if not isinstance(non_domain, Rejection) else non_domain.reason.value
    exp9 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    app_domain = decide_failure_disposition(FailureDispositionRequest(ownership=FailureOwnership.APP_DOMAIN_ONLY, issue=_bounded_issue()))
    # 10. R3 -- only a validated, bounded app-domain failure has the skip path.
    obs10 = app_domain.action if not isinstance(app_domain, Rejection) else app_domain.reason.value
    exp10 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R3 -- that disposition retains the exact issue backing it.
    obs11 = app_domain.issue_ref if not isinstance(app_domain, Rejection) else "rejected"
    exp11 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    mixed = decide_failure_disposition(FailureDispositionRequest(ownership=FailureOwnership.MIXED, issue=_bounded_issue()))
    # 12. R4 -- a mixed record produces a repairable shared slice now.
    obs12 = mixed.shared.action if not isinstance(mixed, Rejection) else mixed.reason.value
    exp12 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R4 -- it separately carries the bounded app-domain slice as a skip.
    obs13 = mixed.app_domain.action if not isinstance(mixed, Rejection) else mixed.reason.value
    exp13 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    passed = decide_terminal_result(TerminalResultRequest(journey_completed=True, ownership=FailureOwnership.NONE, issue=None))
    # 14. AC3 -- a completed successful journey is the only passed terminal.
    obs14 = passed.terminal if not isinstance(passed, Rejection) else passed.reason.value
    exp14 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    tracked = decide_terminal_result(TerminalResultRequest(journey_completed=False, ownership=FailureOwnership.APP_DOMAIN_ONLY, issue=_bounded_issue()))
    # 15. AC3 -- an app-only terminal result is the closed issue-qualified form.
    obs15 = f"{tracked.terminal}({tracked.issue_ref})" if not isinstance(tracked, Rejection) else tracked.reason.value
    exp15 = COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": COMPETITOR_VERIFICATION_2376_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "competitor-verification-2376-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
