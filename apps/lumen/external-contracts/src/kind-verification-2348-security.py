"""EC security case for #2348 -- fail-closed Kind-verification decisions.

Every expected value below is an EC-owned literal transcribed from #2348:
R3 and AC3 forbid a shared/non-domain failure from becoming a tracked skip,
require an issue for an app-domain-only skip, and limit terminal vocabulary to
``passed`` or ``tracked_skip``; R4 forbids skipping a mixed failure while its
shared slice remains.
"""

from __future__ import annotations

from lumen.kind_verification.admission import decide_terminal
from lumen.kind_verification.verdict import Failure, Rejection, TerminalResult, VerificationRecord

MINIMUM_CHECKS = 10

KIND_VERIFICATION_2348_SECURITY_MATRIX = (
    ("terminal_result_vocabulary_is_closed", ("passed", "tracked_skip")),
    ("shared_failure_cannot_become_tracked_skip", "shared-failure-cannot-skip"),
    ("shared_failure_refusal_names_failures", "failures"),
    ("app_domain_neighbour_with_issue_remains_admitted", "tracked_skip(#2349)"),
    ("app_failure_without_issue_is_refused", "missing-domain-issue"),
    ("missing_issue_refusal_names_domain_issue", "domain_issue"),
    ("unvalidated_issue_is_refused", "unvalidated-domain-issue"),
    ("unvalidated_issue_refusal_names_validation_predicate", "domain_issue_validated"),
    ("mixed_failure_cannot_skip_its_shared_slice", "shared-failure-cannot-skip"),
    ("mixed_failure_refusal_names_remaining_shared_failures", "failures"),
)


def verify_kind_verification_2348_security() -> dict:
    checks = []

    # 1. AC3 -- the frozen terminal enum cannot add an untracked third outcome.
    obs1 = tuple(result.value for result in TerminalResult)
    exp1 = KIND_VERIFICATION_2348_SECURITY_MATRIX[0][1]
    checks.append({"name": KIND_VERIFICATION_2348_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    shared_refusal = decide_terminal(
        VerificationRecord(
            failures=(Failure(code="shared-dns", ownership="SHARED_NON_DOMAIN"),),
            domain_issue="#2349",
            domain_issue_validated=True,
        )
    )

    # 2. R3/AC3 -- naming an issue cannot turn shared work into a terminal skip.
    obs2 = shared_refusal.reason.value if isinstance(shared_refusal, Rejection) else shared_refusal.result.value
    exp2 = KIND_VERIFICATION_2348_SECURITY_MATRIX[1][1]
    checks.append({"name": KIND_VERIFICATION_2348_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R3 -- the typed refusal identifies the ownership-bearing input.
    obs3 = shared_refusal.field_path if isinstance(shared_refusal, Rejection) else "unexpected-terminal-result"
    exp3 = KIND_VERIFICATION_2348_SECURITY_MATRIX[2][1]
    checks.append({"name": KIND_VERIFICATION_2348_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    app_neighbour = decide_terminal(
        VerificationRecord(
            failures=(Failure(code="lumen-recovery", ownership="APP_DOMAIN_ONLY"),),
            domain_issue="#2349",
            domain_issue_validated=True,
        )
    )

    # 4. R3 -- the neighbouring app-only input remains eligible for its issue-backed skip.
    obs4 = (
        app_neighbour.reason.value
        if isinstance(app_neighbour, Rejection)
        else f"{app_neighbour.result.value}({app_neighbour.issue_ref})"
    )
    exp4 = KIND_VERIFICATION_2348_SECURITY_MATRIX[3][1]
    checks.append({"name": KIND_VERIFICATION_2348_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    missing_issue = decide_terminal(
        VerificationRecord(
            failures=(Failure(code="lumen-recovery", ownership="APP_DOMAIN_ONLY"),),
            domain_issue="",
            domain_issue_validated=False,
        )
    )

    # 5. AC3 -- an app-domain failure without its bounded issue fails closed.
    obs5 = missing_issue.reason.value if isinstance(missing_issue, Rejection) else missing_issue.result.value
    exp5 = KIND_VERIFICATION_2348_SECURITY_MATRIX[4][1]
    checks.append({"name": KIND_VERIFICATION_2348_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. AC3 -- that refusal names the missing issue field rather than hiding it.
    obs6 = missing_issue.field_path if isinstance(missing_issue, Rejection) else "unexpected-terminal-result"
    exp6 = KIND_VERIFICATION_2348_SECURITY_MATRIX[5][1]
    checks.append({"name": KIND_VERIFICATION_2348_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    unvalidated_issue = decide_terminal(
        VerificationRecord(
            failures=(Failure(code="lumen-recovery", ownership="APP_DOMAIN_ONLY"),),
            domain_issue="#2349",
            domain_issue_validated=False,
        )
    )

    # 7. R3/AC3 -- a merely named issue cannot stand in for validation.
    obs7 = unvalidated_issue.reason.value if isinstance(unvalidated_issue, Rejection) else unvalidated_issue.result.value
    exp7 = KIND_VERIFICATION_2348_SECURITY_MATRIX[6][1]
    checks.append({"name": KIND_VERIFICATION_2348_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R3/AC3 -- the refusal identifies the predicate that remains false.
    obs8 = unvalidated_issue.field_path if isinstance(unvalidated_issue, Rejection) else "unexpected-terminal-result"
    exp8 = KIND_VERIFICATION_2348_SECURITY_MATRIX[7][1]
    checks.append({"name": KIND_VERIFICATION_2348_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    mixed_refusal = decide_terminal(
        VerificationRecord(
            failures=(
                Failure(code="shared-image-pull", ownership="SHARED_NON_DOMAIN"),
                Failure(code="lumen-index-recovery", ownership="APP_DOMAIN_ONLY"),
            ),
            domain_issue="#2349",
            domain_issue_validated=True,
        )
    )

    # 9. R4 -- only the app slice may be skipped; a mixed record remains open.
    obs9 = mixed_refusal.reason.value if isinstance(mixed_refusal, Rejection) else mixed_refusal.result.value
    exp9 = KIND_VERIFICATION_2348_SECURITY_MATRIX[8][1]
    checks.append({"name": KIND_VERIFICATION_2348_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R4 -- the mixed refusal still names the shared-failure-bearing input.
    obs10 = mixed_refusal.field_path if isinstance(mixed_refusal, Rejection) else "unexpected-terminal-result"
    exp10 = KIND_VERIFICATION_2348_SECURITY_MATRIX[9][1]
    checks.append({"name": KIND_VERIFICATION_2348_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "kind-verification-2348-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
