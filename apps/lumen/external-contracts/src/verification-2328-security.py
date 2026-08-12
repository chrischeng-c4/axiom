"""EC security case for #2328 -- fail-closed verification decisions.

Every expected value below is an EC-owned literal transcribed from #2328:
R1/AC1 rejects incomplete or zero-work gate records, and R3/AC3 allows
``tracked_skip(#issue)`` only for an app-domain failure with a bounded issue.
It deliberately does not claim to prove runtime repair, reruns, processes,
persistence, cloud cleanup, or tracker mutation; those rules are runtime-only.
"""

from __future__ import annotations

from lumen.verification.classification import classify_failure
from lumen.verification.result import decide_terminal_result
from lumen.verification.verdict import Failure, Ownership, Rejection, VerificationRecord

MINIMUM_CHECKS = 24

VERIFICATION_2328_SECURITY_MATRIX = (
    ("nonzero_gate_exit_is_refused", "gate_exit_nonzero"),
    ("nonzero_gate_exit_refusal_names_gate_exit_code", "gate_exit_code"),
    ("zero_applicable_work_is_refused", "no_applicable_work"),
    ("zero_applicable_work_refusal_names_applicable_work_count", "applicable_work_count"),
    ("missing_commit_is_refused", "missing_commit"),
    ("missing_commit_refusal_names_commit", "commit"),
    ("missing_environment_is_refused", "missing_environment"),
    ("missing_environment_refusal_names_environment", "environment"),
    ("missing_command_is_refused", "missing_command"),
    ("missing_command_refusal_names_command", "command"),
    ("missing_output_summary_is_refused", "missing_output_summary"),
    ("missing_output_summary_refusal_names_output_summary", "output_summary"),
    ("missing_evidence_path_is_refused", "missing_evidence_path"),
    ("missing_evidence_path_refusal_names_evidence_path", "evidence_path"),
    ("shared_tracked_skip_is_refused", "tracked_skip_requires_app_domain"),
    ("shared_tracked_skip_refusal_names_ownership", "ownership"),
    ("non_domain_tracked_skip_is_refused", "tracked_skip_requires_app_domain"),
    ("non_domain_tracked_skip_refusal_names_ownership", "ownership"),
    ("app_domain_skip_without_issue_is_refused", "tracked_skip_requires_bounded_issue"),
    ("app_domain_skip_without_issue_names_bounded_issue", "bounded_issue"),
    ("unknown_ownership_fails_closed", "unknown_ownership"),
    ("unknown_ownership_refusal_names_ownership", "ownership"),
    ("shared_pass_intent_is_refused", "tracked_skip_requires_app_domain"),
    ("non_domain_pass_intent_is_refused", "tracked_skip_requires_app_domain"),
)


def _reason(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _complete_record(**overrides) -> VerificationRecord:
    values = {
        "gate_exit_code": 0,
        "applicable_work_count": 4,
        "commit": "abc1234",
        "environment": "local-isolated",
        "command": "cargo test -p lumen --test api_e2e --test generated_clients_crud_e2e --test protocol_transport_e2e --test routed_shard_e2e",
        "output_summary": "4 named tests passed",
        "evidence_path": "external-contracts/evidence/2328.json",
        "failure": None,
        "terminal_intent": "passed",
    }
    values.update(overrides)
    return VerificationRecord(**values)


def verify_verification_2328_security() -> dict:
    checks = []

    bad_exit = decide_terminal_result(_complete_record(gate_exit_code=1))

    # 1. R1/AC1 -- an invoked gate that exits non-zero cannot claim pass.
    obs1 = _reason(bad_exit)
    exp1 = VERIFICATION_2328_SECURITY_MATRIX[0][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1/AC1 -- that refusal identifies the failed gate-status field.
    obs2 = bad_exit.field_path if isinstance(bad_exit, Rejection) else ""
    exp2 = VERIFICATION_2328_SECURITY_MATRIX[1][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    zero_work = decide_terminal_result(_complete_record(applicable_work_count=0))

    # 3. R1/AC1 -- a zero-assertion/work run never qualifies as a reusable gate.
    obs3 = _reason(zero_work)
    exp3 = VERIFICATION_2328_SECURITY_MATRIX[2][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R1/AC1 -- its refusal tells the operator which count was invalid.
    obs4 = zero_work.field_path if isinstance(zero_work, Rejection) else ""
    exp4 = VERIFICATION_2328_SECURITY_MATRIX[3][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    missing_commit = decide_terminal_result(_complete_record(commit=None))
    # 5. R1/AC1 -- commit provenance is required before a pass may be claimed.
    obs5 = _reason(missing_commit)
    exp5 = VERIFICATION_2328_SECURITY_MATRIX[4][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R1/AC1 -- the refusal identifies the absent provenance field.
    obs6 = missing_commit.field_path if isinstance(missing_commit, Rejection) else ""
    exp6 = VERIFICATION_2328_SECURITY_MATRIX[5][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    missing_environment = decide_terminal_result(_complete_record(environment=None))
    # 7. R1/AC1 -- a result without its environment is not reproducible.
    obs7 = _reason(missing_environment)
    exp7 = VERIFICATION_2328_SECURITY_MATRIX[6][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R1/AC1 -- the refusal identifies the absent environment field.
    obs8 = missing_environment.field_path if isinstance(missing_environment, Rejection) else ""
    exp8 = VERIFICATION_2328_SECURITY_MATRIX[7][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    missing_command = decide_terminal_result(_complete_record(command=None))
    # 9. R1/AC1 -- the reusable command itself is retained, not inferred.
    obs9 = _reason(missing_command)
    exp9 = VERIFICATION_2328_SECURITY_MATRIX[8][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R1/AC1 -- the refusal identifies the absent command field.
    obs10 = missing_command.field_path if isinstance(missing_command, Rejection) else ""
    exp10 = VERIFICATION_2328_SECURITY_MATRIX[9][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    missing_output = decide_terminal_result(_complete_record(output_summary=None))
    # 11. R1/AC1 -- an exit code alone is not the required output summary.
    obs11 = _reason(missing_output)
    exp11 = VERIFICATION_2328_SECURITY_MATRIX[10][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R1/AC1 -- the refusal identifies the absent output-summary field.
    obs12 = missing_output.field_path if isinstance(missing_output, Rejection) else ""
    exp12 = VERIFICATION_2328_SECURITY_MATRIX[11][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    missing_evidence = decide_terminal_result(_complete_record(evidence_path=None))
    # 13. R1/AC1 -- retained evidence has a named location.
    obs13 = _reason(missing_evidence)
    exp13 = VERIFICATION_2328_SECURITY_MATRIX[12][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R1/AC1 -- the refusal identifies the absent evidence-path field.
    obs14 = missing_evidence.field_path if isinstance(missing_evidence, Rejection) else ""
    exp14 = VERIFICATION_2328_SECURITY_MATRIX[13][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    shared = Failure(ownership=Ownership.SHARED, summary="shared harness assertion failed", bounded_issue="#2329")
    shared_skip = decide_terminal_result(_complete_record(failure=shared, terminal_intent="tracked_skip"))

    # 15. R3/AC3 -- shared failures remain open; an issue link cannot skip them.
    obs15 = _reason(shared_skip)
    exp15 = VERIFICATION_2328_SECURITY_MATRIX[14][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16. R3/AC3 -- the terminal refusal points to the supplied ownership.
    obs16 = shared_skip.field_path if isinstance(shared_skip, Rejection) else ""
    exp16 = VERIFICATION_2328_SECURITY_MATRIX[15][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    non_domain = Failure(ownership=Ownership.NON_DOMAIN, summary="workspace toolchain assertion failed", bounded_issue="#2329")
    non_domain_skip = decide_terminal_result(_complete_record(failure=non_domain, terminal_intent="tracked_skip"))

    # 17. R3/AC3 -- non-domain failures have the same repair-and-rerun boundary.
    obs17 = _reason(non_domain_skip)
    exp17 = VERIFICATION_2328_SECURITY_MATRIX[16][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    # 18. R3/AC3 -- this independently names the non-domain ownership field.
    obs18 = non_domain_skip.field_path if isinstance(non_domain_skip, Rejection) else ""
    exp18 = VERIFICATION_2328_SECURITY_MATRIX[17][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    untracked_app = Failure(ownership=Ownership.APP_DOMAIN, summary="Lumen routing assertion failed", bounded_issue=None)
    untracked_skip = decide_terminal_result(_complete_record(failure=untracked_app, terminal_intent="tracked_skip"))

    # 19. R3/AC3 -- app-domain ownership alone does not turn a bare skip into pass.
    obs19 = _reason(untracked_skip)
    exp19 = VERIFICATION_2328_SECURITY_MATRIX[18][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    # 20. R3/AC3 -- the missing bounded-issue reference is named precisely.
    obs20 = untracked_skip.field_path if isinstance(untracked_skip, Rejection) else ""
    exp20 = VERIFICATION_2328_SECURITY_MATRIX[19][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    unknown = classify_failure(Failure(ownership="other", summary="unowned assertion", bounded_issue=None))

    # 21. R3/AC3 -- classification has no permissive fourth ownership class.
    obs21 = _reason(unknown)
    exp21 = VERIFICATION_2328_SECURITY_MATRIX[20][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})

    # 22. R3/AC3 -- the classifier identifies the ownership value it refused.
    obs22 = unknown.field_path if isinstance(unknown, Rejection) else ""
    exp22 = VERIFICATION_2328_SECURITY_MATRIX[21][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})

    shared_pass = decide_terminal_result(_complete_record(failure=shared, terminal_intent="passed"))

    # 23. R3/AC3 -- unresolved shared work cannot be passed instead of repaired.
    obs23 = _reason(shared_pass)
    exp23 = VERIFICATION_2328_SECURITY_MATRIX[22][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})

    non_domain_pass = decide_terminal_result(_complete_record(failure=non_domain, terminal_intent="passed"))

    # 24. R3/AC3 -- unresolved non-domain work has the same pass refusal.
    obs24 = _reason(non_domain_pass)
    exp24 = VERIFICATION_2328_SECURITY_MATRIX[23][1]
    checks.append({"name": VERIFICATION_2328_SECURITY_MATRIX[23][0], "expected": exp24, "observed": obs24, "passed": obs24 == exp24})

    return {
        "case_id": "verification-2328-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
