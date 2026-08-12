"""EC security case for #2343 -- fail-closed ownership classification.

Every expected value is EC-owned and transcribed from #2343 R3/R4/AC3.  A
skip without a bounded app-domain issue is refused with an exact reason and
field; mixed work never drops its shared repair-and-rerun action; and a failed
shared rerun remains open rather than overclaiming a terminal pass.
"""

from __future__ import annotations

from lumen.container_kustomize.classification import (
    decide_failure_outcome,
    decide_mixed_failure,
)
from lumen.container_kustomize.result import decide_terminal_result
from lumen.container_kustomize.spec import BoundedIssue, FailureOwner
from lumen.container_kustomize.verdict import Rejection

MINIMUM_CHECKS = 13

CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX = (
    ("app_domain_unbounded_issue_is_refused", "bounded_issue_required"),
    ("app_domain_unbounded_issue_refusal_names_issue_number", "bounded_issue.number"),
    ("app_domain_bounded_issue_neighbour_is_admitted", "tracked_skip"),
    ("mixed_unbounded_issue_is_refused", "bounded_issue_required"),
    ("mixed_unbounded_issue_refusal_names_issue_number", "bounded_issue.number"),
    ("mixed_failure_never_skips_the_shared_slice", "repair_and_rerun"),
    ("terminal_app_domain_unbounded_issue_stays_open", "open"),
    ("terminal_unbounded_issue_explains_the_missing_bound", "bounded_issue_required"),
    ("terminal_unbounded_issue_names_issue_number", "bounded_issue.number"),
    ("failed_shared_rerun_stays_open", "open"),
    ("failed_shared_rerun_explains_the_missing_rerun", "shared_rerun_required"),
    ("failed_shared_rerun_names_rerun_state", "shared_rerun_succeeded"),
    ("terminal_bounded_app_domain_neighbour_is_tracked_skip", "tracked_skip"),
)


def _action(decision) -> str:
    return decision.reason.value if isinstance(decision, Rejection) else decision.action.value


def _terminal_state(decision) -> str:
    return decision.reason.value if isinstance(decision, Rejection) else decision.state.value


def verify_container_kustomize_2343_security() -> dict:
    checks = []
    unbounded_issue = BoundedIssue(number=0)
    bounded_issue = BoundedIssue(number=2343)

    domain_unbounded = decide_failure_outcome(FailureOwner.APP_DOMAIN, unbounded_issue)

    # 1. R3 -- an issue identity of zero is not the bounded reproducing issue.
    obs1 = _action(domain_unbounded)
    exp1 = CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[0][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R3 -- a refusal says that the issue number, not ownership, is invalid.
    obs2 = domain_unbounded.field_path if isinstance(domain_unbounded, Rejection) else ""
    exp2 = CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[1][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    domain_bounded = decide_failure_outcome(FailureOwner.APP_DOMAIN, bounded_issue)

    # 3. R3 -- the nearest valid issue remains admitted as the permitted skip.
    obs3 = _action(domain_bounded)
    exp3 = CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[2][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    mixed_unbounded = decide_mixed_failure(FailureOwner.SHARED, FailureOwner.APP_DOMAIN, unbounded_issue)

    # 4. R4 -- a mixed domain slice cannot claim a skip with an unbounded issue.
    obs4 = _action(mixed_unbounded)
    exp4 = CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[3][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R4 -- this entry point also identifies the invalid issue number.
    obs5 = mixed_unbounded.field_path if isinstance(mixed_unbounded, Rejection) else ""
    exp5 = CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[4][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    mixed_bounded = decide_mixed_failure(FailureOwner.SHARED, FailureOwner.APP_DOMAIN, bounded_issue)

    # 6. R4 -- even when the domain half is skippable, the shared half is not.
    obs6 = _action(mixed_bounded.shared) if not isinstance(mixed_bounded, Rejection) else mixed_bounded.reason.value
    exp6 = CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[5][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    terminal_unbounded = decide_terminal_result(FailureOwner.APP_DOMAIN, unbounded_issue, False)

    # 7. AC3 -- an app-domain failure without a bounded issue cannot terminate.
    obs7 = _terminal_state(terminal_unbounded)
    exp7 = CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[6][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. AC3 -- the open result uses the same explicit missing-bound reason.
    obs8 = terminal_unbounded.reason.value if not isinstance(terminal_unbounded, Rejection) else terminal_unbounded.reason.value
    exp8 = CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[7][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. AC3 -- the open result pinpoints the missing issue identity.
    obs9 = terminal_unbounded.field_path if not isinstance(terminal_unbounded, Rejection) else terminal_unbounded.field_path
    exp9 = CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[8][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    shared_not_rerun = decide_terminal_result(FailureOwner.SHARED, bounded_issue, False)

    # 10. AC3 -- a shared failure remains open until its repair is rerun.
    obs10 = _terminal_state(shared_not_rerun)
    exp10 = CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[9][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. AC3 -- it states why the terminal claim is prohibited.
    obs11 = shared_not_rerun.reason.value if not isinstance(shared_not_rerun, Rejection) else shared_not_rerun.reason.value
    exp11 = CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[10][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. AC3 -- and names the final-gate state that remains false.
    obs12 = shared_not_rerun.field_path if not isinstance(shared_not_rerun, Rejection) else shared_not_rerun.field_path
    exp12 = CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[11][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    terminal_bounded = decide_terminal_result(FailureOwner.APP_DOMAIN, bounded_issue, False)

    # 13. AC3 -- the valid neighbouring app-domain input retains its skip path.
    obs13 = _terminal_state(terminal_bounded)
    exp13 = CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[12][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    return {
        "case_id": "container-kustomize-2343-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
