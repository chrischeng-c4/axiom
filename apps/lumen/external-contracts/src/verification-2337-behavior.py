"""EC behavior case for #2337 -- verification failure dispositions and results.

Every expected value below is an EC-owned literal transcribed from #2337:
R3 requires the closed ``shared_repair_required`` and
``app_domain_trackable`` ownership dispositions; R4 requires a mixed failure
to retain separate, lossless shared and app-domain slices; and AC3 permits only
``passed`` with no unresolved failures or ``tracked_skip(#issue)`` for the
app-domain-only, issue-backed path.  No expected value is read from the design
under test and no design-computed pass flag is accepted as an observation.
"""

from __future__ import annotations

from lumen.verification.classification import classify_failure, split_failure
from lumen.verification.verdict import Failure, Rejection
from lumen.verification.verdict import decide_terminal_result

MINIMUM_CHECKS = 8

VERIFICATION_2337_BEHAVIOR_MATRIX = (
    ("shared_failure_requires_shared_repair", "shared_repair_required"),
    ("app_domain_failure_is_trackable", "app_domain_trackable"),
    ("mixed_failure_keeps_the_shared_slice", ("shared-http",)),
    ("mixed_failure_keeps_the_app_domain_slice", ("lumen-schema",)),
    ("mixed_failure_preserves_every_input_once", ("shared-http", "lumen-schema")),
    ("no_unresolved_failure_passes_after_rerun", "passed"),
    ("app_domain_only_failure_tracks_its_single_issue", "tracked_skip(#2338)"),
    ("app_domain_only_failure_without_issue_is_refused", "exactly_one_issue_reference"),
)


def _disposition(classification) -> str:
    return classification.disposition.value if not isinstance(classification, Rejection) else classification.reason.value


def _terminal(verdict) -> str:
    if isinstance(verdict, Rejection):
        return verdict.reason.value
    if verdict.result.value == "tracked_skip":
        return f"tracked_skip({verdict.issue_ref})"
    return verdict.result.value


def verify_verification_2337_behavior() -> dict:
    checks = []

    shared_failure = Failure(failure_id="shared-http", owner="shared")
    app_failure = Failure(failure_id="lumen-schema", owner="app_domain")

    # 1. R3 -- a shared failure is repair work, never a trackable domain skip.
    shared_classification = classify_failure(shared_failure)
    obs1 = _disposition(shared_classification)
    exp1 = VERIFICATION_2337_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": VERIFICATION_2337_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R3 -- app-domain ownership is the only classification eligible for
    #    the separately tracked path.
    app_classification = classify_failure(app_failure)
    obs2 = _disposition(app_classification)
    exp2 = VERIFICATION_2337_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": VERIFICATION_2337_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    mixed = split_failure((shared_failure, app_failure))

    # 3. R4 -- splitting a mixed input retains the shared repair slice.
    obs3 = tuple(failure.failure_id for failure in mixed.shared_failures)
    exp3 = VERIFICATION_2337_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": VERIFICATION_2337_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R4 -- and retains the separately trackable Lumen-domain slice.
    obs4 = tuple(failure.failure_id for failure in mixed.app_domain_failures)
    exp4 = VERIFICATION_2337_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": VERIFICATION_2337_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R4 -- a fresh split observes the all-input preservation property, so
    #    this row does not merely re-read either slice used above.
    lossless_mixed = split_failure((shared_failure, app_failure))
    obs5 = tuple(
        failure.failure_id
        for failure in (*lossless_mixed.shared_failures, *lossless_mixed.app_domain_failures)
    )
    exp5 = VERIFICATION_2337_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": VERIFICATION_2337_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. AC3 -- a completed rerun with no unresolved classifications is passed.
    passed = decide_terminal_result((), (), rerun_complete=True)
    obs6 = _terminal(passed)
    exp6 = VERIFICATION_2337_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": VERIFICATION_2337_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. AC3 -- exactly one app-domain classification and one issue reference
    #    reaches the only permitted skip result.
    tracked = decide_terminal_result((app_classification,), ("#2338",), rerun_complete=True)
    obs7 = _terminal(tracked)
    exp7 = VERIFICATION_2337_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": VERIFICATION_2337_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. AC3 -- the app-domain path stays open when its required single
    #    bounded issue reference is absent.
    missing_issue = decide_terminal_result((app_classification,), (), rerun_complete=True)
    obs8 = _terminal(missing_issue)
    exp8 = VERIFICATION_2337_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": VERIFICATION_2337_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    return {
        "case_id": "verification-2337-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
