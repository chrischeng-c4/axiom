"""EC security case for #2377 -- fail-closed operational-log integration.

The expected literals are owned by #2377 R1-R4 and AC1/AC3.  Each refusal
observes its typed reason and its named field, then drives a neighbouring valid
input.  The case deliberately does not claim live stdout, Sift, cleanup, or
runtime test execution facts.
"""

from __future__ import annotations

from lumen.operational_log_integration.admission import (
    classify_failure,
    decide_coverage,
    decide_gate_record,
    decide_mixed_failure,
    decide_terminal_result,
)
from lumen.operational_log_integration.spec import Failure, FailureOwnership, GateRecord
from lumen.operational_log_integration.verdict import Rejection

MINIMUM_CHECKS = 29

OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX = (
    ("missing_command_is_rejected", "missing_required_evidence"),
    ("missing_command_refusal_names_command", "command"),
    ("missing_commit_is_rejected", "missing_required_evidence"),
    ("missing_commit_refusal_names_commit", "commit"),
    ("missing_environment_is_rejected", "missing_required_evidence"),
    ("missing_environment_refusal_names_environment", "environment"),
    ("missing_output_summary_is_rejected", "missing_required_evidence"),
    ("missing_output_summary_refusal_names_output_summary", "output_summary"),
    ("missing_evidence_path_is_rejected", "missing_required_evidence"),
    ("missing_evidence_path_refusal_names_evidence_path", "evidence_path"),
    ("complete_gate_record_neighbour_is_admitted", "admitted"),
    ("missing_success_kind_is_rejected", "required_event_kind_missing"),
    ("missing_success_kind_is_named", "success"),
    ("missing_retry_or_failure_kind_is_rejected", "required_event_kind_missing"),
    ("missing_retry_or_failure_kind_is_named", "retry_or_failure"),
    ("missing_security_audit_kind_is_rejected", "required_event_kind_missing"),
    ("missing_security_audit_kind_is_named", "security_audit"),
    ("missing_lifecycle_kind_is_rejected", "required_event_kind_missing"),
    ("missing_lifecycle_kind_is_named", "lifecycle"),
    ("complete_kind_coverage_neighbour_is_admitted", "admitted"),
    ("app_skip_without_issue_is_rejected", "bounded_issue_required"),
    ("missing_issue_refusal_names_issue_ref", "issue_ref"),
    ("app_skip_without_reproduction_is_rejected", "exact_reproduction_required"),
    ("missing_reproduction_refusal_names_exact_reproduction", "exact_reproduction"),
    ("unchecked_authoritative_wi_blocks_duplicate", "existing_wi_acceptance_check_required"),
    ("unchecked_authoritative_wi_refusal_names_acceptance", "authoritative_existing_wi_accepted"),
    ("shared_terminalization_is_refused", "shared_non_domain_failure_requires_repair"),
    ("mixed_terminalization_is_refused", "mixed_failure_requires_split"),
    ("valid_app_domain_neighbour_is_issue_qualified_skip", "tracked_skip(#2377)"),
)


def _reason(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _record(**overrides) -> GateRecord:
    values = {"commit": "0123456789abcdef", "environment": "vat-local", "command": "cargo test -p lumen --test structured_stdout_traceparent", "output_summary": "12 passed; 4 work items", "evidence_path": "evidence/operational-log-2377.json"}
    values.update(overrides)
    return GateRecord(**values)


def _failure(ownership: FailureOwnership, **overrides) -> Failure:
    values = {"ownership": ownership, "issue_ref": "#2377", "exact_reproduction": "cargo test -p lumen --test behavior_lumen_claim_observability_otlp", "authoritative_existing_wi_supplied": False, "authoritative_existing_wi_accepted": False}
    values.update(overrides)
    return Failure(**values)


def _terminal_value(verdict) -> str:
    if isinstance(verdict, Rejection):
        return verdict.reason.value
    terminal = verdict.terminal.value if hasattr(verdict.terminal, "value") else verdict.terminal
    return f"{terminal}({verdict.issue_ref})" if terminal == "tracked_skip" else terminal


def verify_operational_log_integration_2377_security() -> dict:
    checks = []

    missing_command = decide_gate_record(_record(command=""))
    # 1. R1 -- a record that names no reusable command is not admissible.
    obs1 = _reason(missing_command); exp1 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[0][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    # 2. R1 -- the typed refusal identifies command, rather than failing vaguely.
    obs2 = missing_command.field_path if isinstance(missing_command, Rejection) else ""; exp2 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[1][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    missing_commit = decide_gate_record(_record(commit=""))
    # 3. AC1 -- the record must carry its exact commit.
    obs3 = _reason(missing_commit); exp3 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[2][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    # 4. AC1 -- the refusal identifies commit.
    obs4 = missing_commit.field_path if isinstance(missing_commit, Rejection) else ""; exp4 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[3][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    missing_environment = decide_gate_record(_record(environment=""))
    # 5. AC1 -- environment is a required reproducibility field.
    obs5 = _reason(missing_environment); exp5 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[4][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    # 6. AC1 -- the refusal identifies environment.
    obs6 = missing_environment.field_path if isinstance(missing_environment, Rejection) else ""; exp6 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[5][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    missing_output = decide_gate_record(_record(output_summary=""))
    # 7. AC1 -- the recorded output summary cannot be omitted.
    obs7 = _reason(missing_output); exp7 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[6][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    # 8. AC1 -- the refusal identifies output_summary.
    obs8 = missing_output.field_path if isinstance(missing_output, Rejection) else ""; exp8 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[7][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    missing_evidence = decide_gate_record(_record(evidence_path=""))
    # 9. AC1 -- a gate record without its evidence path is rejected.
    obs9 = _reason(missing_evidence); exp9 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[8][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    # 10. AC1 -- the refusal identifies evidence_path.
    obs10 = missing_evidence.field_path if isinstance(missing_evidence, Rejection) else ""; exp10 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[9][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    complete_gate = decide_gate_record(_record())
    # 11. R1/AC1 -- explicit complete evidence remains admissible.
    obs11 = _reason(complete_gate)
    exp11 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[10][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    missing_success = decide_coverage(("retry_or_failure", "security_audit", "lifecycle"))
    # 12. R2 -- success coverage is independently required.
    obs12 = _reason(missing_success); exp12 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[11][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    # 13. R2 -- the verdict names the concrete missing success kind.
    obs13 = missing_success.missing_kind if isinstance(missing_success, Rejection) else ""; exp13 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[12][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    missing_retry = decide_coverage(("success", "security_audit", "lifecycle"))
    # 14. R2 -- retry-or-failure coverage is independently required.
    obs14 = _reason(missing_retry); exp14 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[13][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    # 15. R2 -- the verdict names retry_or_failure.
    obs15 = missing_retry.missing_kind if isinstance(missing_retry, Rejection) else ""; exp15 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[14][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    missing_audit = decide_coverage(("success", "retry_or_failure", "lifecycle"))
    # 16. R2 -- security/audit coverage is independently required.
    obs16 = _reason(missing_audit); exp16 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[15][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    # 17. R2 -- the verdict names security_audit.
    obs17 = missing_audit.missing_kind if isinstance(missing_audit, Rejection) else ""; exp17 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[16][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    missing_lifecycle = decide_coverage(("success", "retry_or_failure", "security_audit"))
    # 18. R2 -- lifecycle coverage is independently required.
    obs18 = _reason(missing_lifecycle); exp18 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[17][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})
    # 19. R2 -- the verdict names lifecycle.
    obs19 = missing_lifecycle.missing_kind if isinstance(missing_lifecycle, Rejection) else ""; exp19 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[18][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})
    complete_coverage = decide_coverage(("success", "retry_or_failure", "security_audit", "lifecycle"))
    # 20. R2 -- the fully covered neighbour is still admitted.
    obs20 = _reason(complete_coverage)
    exp20 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[19][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    no_issue = classify_failure(_failure(FailureOwnership.APP_DOMAIN_ONLY, issue_ref=""))
    # 21-22. R3 -- a Lumen-domain skip needs an explicit bounded issue reference.
    obs21 = _reason(no_issue)
    exp21 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[20][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})
    obs22 = no_issue.field_path if isinstance(no_issue, Rejection) else ""
    exp22 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[21][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})
    no_reproduction = classify_failure(_failure(FailureOwnership.APP_DOMAIN_ONLY, exact_reproduction=""))
    # 23-24. R3 -- the issue must carry its exact reproduction too.
    obs23 = _reason(no_reproduction)
    exp23 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[22][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})
    obs24 = no_reproduction.field_path if isinstance(no_reproduction, Rejection) else ""
    exp24 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[23][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[23][0], "expected": exp24, "observed": obs24, "passed": obs24 == exp24})
    unchecked_wi = decide_mixed_failure(_failure(FailureOwnership.MIXED, authoritative_existing_wi_supplied=True, authoritative_existing_wi_accepted=False))
    # 25-26. R4 -- an authoritative existing domain WI is acceptance-checked before duplication.
    obs25 = _reason(unchecked_wi)
    exp25 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[24][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[24][0], "expected": exp25, "observed": obs25, "passed": obs25 == exp25})
    obs26 = unchecked_wi.field_path if isinstance(unchecked_wi, Rejection) else ""
    exp26 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[25][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[25][0], "expected": exp26, "observed": obs26, "passed": obs26 == exp26})

    shared = classify_failure(_failure(FailureOwnership.SHARED_NON_DOMAIN))
    shared_terminal = decide_terminal_result(shared, "#2377", True)
    # 27. R3/AC3 -- shared/non-domain failure remains open for repair and rerun.
    obs27 = _terminal_value(shared_terminal)
    exp27 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[26][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[26][0], "expected": exp27, "observed": obs27, "passed": obs27 == exp27})
    mixed = classify_failure(_failure(FailureOwnership.MIXED))
    mixed_terminal = decide_terminal_result(mixed, "#2377", True)
    # 28. R4/AC3 -- unresolved mixed ownership is never collapsed into a terminal skip.
    obs28 = _terminal_value(mixed_terminal)
    exp28 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[27][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[27][0], "expected": exp28, "observed": obs28, "passed": obs28 == exp28})
    valid_app = classify_failure(_failure(FailureOwnership.APP_DOMAIN_ONLY))
    valid_terminal = decide_terminal_result(valid_app, "#2377", True)
    # 29. AC3 -- the nearby valid app-domain result remains exact and terminal.
    obs29 = _terminal_value(valid_terminal)
    exp29 = OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[28][1]
    checks.append({"name": OPERATIONAL_LOG_INTEGRATION_2377_SECURITY_MATRIX[28][0], "expected": exp29, "observed": obs29, "passed": obs29 == exp29})

    return {"case_id": "operational-log-integration-2377-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
