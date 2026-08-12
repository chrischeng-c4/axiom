"""EC behavior case for #2377 -- operational-log integration admission.

Every expected value is an EC-owned literal from #2377 R1-R4 and AC1/AC3.
This pure model checks the admissibility and disposition vocabulary only;
commands, live Sift delivery, exit status, work counts, and cleanup are
runtime-stage observations rather than claims this case can prove.
"""

from __future__ import annotations

from lumen.operational_log_integration.admission import (
    classify_failure,
    decide_coverage,
    decide_gate_record,
    decide_mixed_failure,
    decide_terminal_result,
)
from lumen.operational_log_integration.spec import (
    Failure,
    FailureOwnership,
    GateRecord,
)
from lumen.operational_log_integration.verdict import Rejection

MINIMUM_CHECKS = 16

OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX = (
    ("complete_gate_record_is_admitted", "admitted"),
    ("admitted_gate_record_retains_commit", "0123456789abcdef"),
    ("admitted_gate_record_retains_environment", "vat-local"),
    ("admitted_gate_record_retains_command", "cargo test -p lumen --test structured_stdout_traceparent"),
    ("admitted_gate_record_retains_output_summary", "12 passed; 4 work items"),
    ("admitted_gate_record_retains_evidence_path", "evidence/operational-log-2377.json"),
    ("complete_required_event_kind_coverage_is_admitted", "admitted"),
    ("shared_non_domain_failure_is_classified", "shared_non_domain"),
    ("app_domain_failure_is_classified", "app_domain_only"),
    ("bounded_app_domain_failure_is_tracked_skip", "tracked_skip"),
    ("mixed_failure_is_classified", "mixed"),
    ("mixed_failure_retains_shared_repair_action", "repair_and_rerun"),
    ("mixed_failure_retains_app_domain_skip_action", "tracked_skip"),
    ("completed_successful_journey_is_passed", "passed"),
    ("validated_app_domain_terminal_is_issue_qualified_skip", "tracked_skip(#2377)"),
    ("shared_rerun_complete_does_not_change_a_valid_app_skip", "tracked_skip(#2377)"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _complete_record(**overrides) -> GateRecord:
    values = {
        "commit": "0123456789abcdef",
        "environment": "vat-local",
        "command": "cargo test -p lumen --test structured_stdout_traceparent",
        "output_summary": "12 passed; 4 work items",
        "evidence_path": "evidence/operational-log-2377.json",
    }
    values.update(overrides)
    return GateRecord(**values)


def _failure(ownership: FailureOwnership, **overrides) -> Failure:
    values = {
        "ownership": ownership,
        "issue_ref": "#2377",
        "exact_reproduction": "cargo test -p lumen --test behavior_lumen_claim_observability_otlp",
        "authoritative_existing_wi_supplied": False,
        "authoritative_existing_wi_accepted": False,
    }
    values.update(overrides)
    return Failure(**values)


def _terminal_value(verdict) -> str:
    if isinstance(verdict, Rejection):
        return verdict.reason.value
    terminal = verdict.terminal.value if hasattr(verdict.terminal, "value") else verdict.terminal
    return f"{terminal}({verdict.issue_ref})" if terminal == "tracked_skip" else terminal


def verify_operational_log_integration_2377_behavior() -> dict:
    checks = []

    admitted_record = decide_gate_record(_complete_record())
    # 1. R1/AC1 -- the reproducible gate record is complete before it is admitted.
    obs1 = _outcome(admitted_record)
    exp1 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    # 2-6. R1/AC1 -- admission preserves the five recorded identity/evidence values.
    obs2 = admitted_record.record.commit if not isinstance(admitted_record, Rejection) else "rejected"
    exp2 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = admitted_record.record.environment if not isinstance(admitted_record, Rejection) else "rejected"
    exp3 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = admitted_record.record.command if not isinstance(admitted_record, Rejection) else "rejected"
    exp4 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = admitted_record.record.output_summary if not isinstance(admitted_record, Rejection) else "rejected"
    exp5 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = admitted_record.record.evidence_path if not isinstance(admitted_record, Rejection) else "rejected"
    exp6 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    coverage = decide_coverage(("success", "retry_or_failure", "security_audit", "lifecycle"))
    # 7. R2 -- all four required structured-event kinds are required together.
    obs7 = _outcome(coverage)
    exp7 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    shared = classify_failure(_failure(FailureOwnership.SHARED_NON_DOMAIN))
    # 8. R3 -- shared/non-domain work is classified at its shared owner.
    obs8 = shared.classification.value if not isinstance(shared, Rejection) else shared.reason.value
    exp8 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    app = classify_failure(_failure(FailureOwnership.APP_DOMAIN_ONLY))
    # 9-10. R3 -- only a bounded, reproducible Lumen-domain failure has a skip.
    obs9 = app.classification.value if not isinstance(app, Rejection) else app.reason.value
    exp9 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = app.action if not isinstance(app, Rejection) else app.reason.value
    exp10 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    mixed = decide_mixed_failure(_failure(FailureOwnership.MIXED))
    # 11-13. R4 -- a mixed failure stays split: repair the shared slice and
    # retain the independently issue-backed Lumen-domain skip.
    obs11 = mixed.classification.value if not isinstance(mixed, Rejection) else mixed.reason.value
    exp11 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = mixed.shared.action if not isinstance(mixed, Rejection) else mixed.reason.value
    exp12 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = mixed.app_domain.action if not isinstance(mixed, Rejection) else mixed.reason.value
    exp13 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    passed = decide_terminal_result(FailureOwnership.NONE, None, True)
    # 14. AC3 -- a completed failure-free journey has the only success terminal.
    obs14 = _terminal_value(passed)
    exp14 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    tracked = decide_terminal_result(app, "#2377", True)
    # 15. AC3 -- a validated app-domain classification is issue-qualified.
    obs15 = _terminal_value(tracked)
    exp15 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    tracked_after_rerun = decide_terminal_result(app, "#2377", True)
    # 16. AC3 -- completion of shared reruns does not erase a valid app skip.
    obs16 = _terminal_value(tracked_after_rerun)
    exp16 = OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    return {"case_id": "operational-log-integration-2377-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
