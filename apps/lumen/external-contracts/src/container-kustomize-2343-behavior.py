"""EC behavior case for #2343 -- container/Kustomize ownership decisions.

Every expected value below is an EC-owned literal transcribed from #2343:
R3 sends shared and non-domain failures to ``shared_repair_required`` and
permits ``tracked_skip(issue_number)`` only for an app-domain failure with a
bounded issue; R4 splits a mixed failure into ``repair_and_rerun`` and that
separately bounded skip; and AC3 permits only a successful final gate to pass.
"""

from __future__ import annotations

from lumen.container_kustomize.classification import (
    decide_failure_outcome,
    decide_mixed_failure,
)
from lumen.container_kustomize.result import decide_terminal_result
from lumen.container_kustomize.spec import BoundedIssue, FailureOwner
from lumen.container_kustomize.verdict import Rejection

MINIMUM_CHECKS = 11

CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX = (
    ("shared_failure_requires_shared_repair", "shared_repair_required"),
    ("non_domain_failure_requires_shared_repair", "shared_repair_required"),
    ("app_domain_failure_with_bounded_issue_is_tracked_skip", "tracked_skip"),
    ("app_domain_tracked_skip_carries_the_bounded_issue", 2344),
    ("mixed_failure_repairs_and_reruns_the_shared_slice", "repair_and_rerun"),
    ("mixed_failure_tracks_the_domain_slice_separately", "tracked_skip"),
    ("mixed_failure_domain_slice_carries_the_bounded_issue", 2343),
    ("successful_final_gate_passes", "passed"),
    ("failed_final_gate_without_failure_owner_stays_open", "open"),
    ("app_domain_terminal_result_is_tracked_skip", "tracked_skip"),
    ("terminal_tracked_skip_carries_the_bounded_issue", 2344),
)


def _action(decision) -> str:
    return decision.reason.value if isinstance(decision, Rejection) else decision.action.value


def _terminal_state(decision) -> str:
    return decision.reason.value if isinstance(decision, Rejection) else decision.state.value


def verify_container_kustomize_2343_behavior() -> dict:
    checks = []
    bounded_issue = BoundedIssue(number=2343)
    neighbouring_bounded_issue = BoundedIssue(number=2344)

    shared = decide_failure_outcome(FailureOwner.SHARED, bounded_issue)

    # 1. R3 -- a shared-owner failure cannot end as an app-domain skip.
    obs1 = _action(shared)
    exp1 = CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    non_domain = decide_failure_outcome(FailureOwner.NON_DOMAIN, bounded_issue)

    # 2. R3 -- non-domain ownership is equally shared work.
    obs2 = _action(non_domain)
    exp2 = CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    domain = decide_failure_outcome(FailureOwner.APP_DOMAIN, neighbouring_bounded_issue)

    # 3. R3 -- a bounded app-domain failure has the one permitted skip action.
    obs3 = _action(domain)
    exp3 = CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R3 -- the skip keeps the concrete issue identity, not a bare link.
    obs4 = domain.issue_number if not isinstance(domain, Rejection) else -1
    exp4 = CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    mixed = decide_mixed_failure(FailureOwner.SHARED, FailureOwner.APP_DOMAIN, bounded_issue)

    # 5. R4 -- the shared half of a mixed failure is always repaired and rerun.
    obs5 = _action(mixed.shared) if not isinstance(mixed, Rejection) else mixed.reason.value
    exp5 = CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R4 -- only its separately owned domain half may be tracked as skipped.
    obs6 = _action(mixed.domain) if not isinstance(mixed, Rejection) else mixed.reason.value
    exp6 = CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R4 -- the separately skipped domain half retains its bounded issue.
    obs7 = mixed.domain.issue_number if not isinstance(mixed, Rejection) else -1
    exp7 = CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    passed = decide_terminal_result(FailureOwner.NONE, None, True)

    # 8. AC3 -- a successful final gate is the only pass outcome.
    obs8 = _terminal_state(passed)
    exp8 = CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    final_gate_failed = decide_terminal_result(FailureOwner.NONE, None, False)

    # 9. AC3 -- without a successful final gate, even no failure owner stays open.
    obs9 = _terminal_state(final_gate_failed)
    exp9 = CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    terminal_skip = decide_terminal_result(FailureOwner.APP_DOMAIN, neighbouring_bounded_issue, False)

    # 10. AC3 -- a bounded app-domain failure may end as tracked_skip.
    obs10 = _terminal_state(terminal_skip)
    exp10 = CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. AC3 -- its terminal record retains the supplied validated issue number.
    obs11 = terminal_skip.issue_number if not isinstance(terminal_skip, Rejection) else -1
    exp11 = CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": CONTAINER_KUSTOMIZE_2343_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "container-kustomize-2343-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
