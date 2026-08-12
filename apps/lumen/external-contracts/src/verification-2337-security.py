"""EC security case for #2337 -- fail-closed verification terminal results.

Every expected value below is an EC-owned literal transcribed from #2337:
R3 makes a shared/non-domain failure ineligible for ``tracked_skip``; R4
requires its shared slice to be rerun now while the app-domain slice is named
separately; and AC3 limits ``tracked_skip(#issue)`` to a rerun-complete,
app-domain-only disposition with one bounded issue reference.  The live
existence and boundedness of an issue is runtime-only, so this pure design case
checks the closed decision vocabulary and the named invalid input instead.
"""

from __future__ import annotations

from lumen.verification.classification import classify_failure, split_failure
from lumen.verification.verdict import Failure, Rejection
from lumen.verification.verdict import decide_terminal_result

MINIMUM_CHECKS = 9

VERIFICATION_2337_SECURITY_MATRIX = (
    ("unknown_owner_is_refused", "unknown_failure_owner"),
    ("unknown_owner_refusal_names_owner", "owner"),
    ("known_app_domain_neighbour_is_trackable", "app_domain_trackable"),
    ("shared_failure_cannot_be_tracked_skip", "shared_repair_required"),
    ("shared_terminal_refusal_names_classifications", "classifications"),
    ("mixed_terminal_refusal_keeps_shared_repair_open", "shared_repair_required"),
    ("incomplete_rerun_is_refused", "rerun_incomplete"),
    ("incomplete_rerun_refusal_names_rerun_complete", "rerun_complete"),
    ("multiple_issue_references_are_refused", "exactly_one_issue_reference"),
)


def _disposition(classification) -> str:
    return classification.disposition.value if not isinstance(classification, Rejection) else classification.reason.value


def _reason(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else verdict.result.value


def verify_verification_2337_security() -> dict:
    checks = []

    unknown_owner = classify_failure(Failure(failure_id="unowned", owner="third_party"))

    # 1. R3 -- ownership outside the two closed classes cannot silently become
    #    an app-domain skip.
    obs1 = _disposition(unknown_owner)
    exp1 = VERIFICATION_2337_SECURITY_MATRIX[0][1]
    checks.append({"name": VERIFICATION_2337_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R3 -- the refusal points at the ownership declaration.
    obs2 = unknown_owner.field_path if isinstance(unknown_owner, Rejection) else ""
    exp2 = VERIFICATION_2337_SECURITY_MATRIX[1][1]
    checks.append({"name": VERIFICATION_2337_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R3 -- the nearest closed-world neighbour remains admissible.
    app_classification = classify_failure(Failure(failure_id="lumen-api", owner="app_domain"))
    obs3 = _disposition(app_classification)
    exp3 = VERIFICATION_2337_SECURITY_MATRIX[2][1]
    checks.append({"name": VERIFICATION_2337_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    shared_classification = classify_failure(Failure(failure_id="shared-tls", owner="shared"))

    # 4. R3/AC3 -- a shared failure cannot claim the terminal skip result.
    shared_terminal = decide_terminal_result((shared_classification,), ("#2338",), rerun_complete=True)
    obs4 = _reason(shared_terminal)
    exp4 = VERIFICATION_2337_SECURITY_MATRIX[3][1]
    checks.append({"name": VERIFICATION_2337_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R3/AC3 -- that refusal says which decision input remains unresolved.
    obs5 = shared_terminal.field_path if isinstance(shared_terminal, Rejection) else ""
    exp5 = VERIFICATION_2337_SECURITY_MATRIX[4][1]
    checks.append({"name": VERIFICATION_2337_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R4/AC3 -- a mixed input keeps its shared repair obligation open.
    mixed = split_failure((Failure(failure_id="shared-http", owner="shared"), Failure(failure_id="lumen-schema", owner="app_domain")))
    mixed_classifications = tuple(classify_failure(failure) for failure in (*mixed.shared_failures, *mixed.app_domain_failures))
    mixed_terminal = decide_terminal_result(mixed_classifications, ("#2338",), rerun_complete=True)
    obs6 = _reason(mixed_terminal)
    exp6 = VERIFICATION_2337_SECURITY_MATRIX[5][1]
    checks.append({"name": VERIFICATION_2337_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. AC3 -- an app-domain result is still open until the required rerun is complete.
    incomplete = decide_terminal_result((app_classification,), ("#2338",), rerun_complete=False)
    obs7 = _reason(incomplete)
    exp7 = VERIFICATION_2337_SECURITY_MATRIX[6][1]
    checks.append({"name": VERIFICATION_2337_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. AC3 -- the incomplete-rerun refusal names the gate that is still open.
    obs8 = incomplete.field_path if isinstance(incomplete, Rejection) else ""
    exp8 = VERIFICATION_2337_SECURITY_MATRIX[7][1]
    checks.append({"name": VERIFICATION_2337_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. AC3 -- more than one issue reference is not the required one bounded issue.
    multiple_refs = decide_terminal_result((app_classification,), ("#2338", "#2339"), rerun_complete=True)
    obs9 = _reason(multiple_refs)
    exp9 = VERIFICATION_2337_SECURITY_MATRIX[8][1]
    checks.append({"name": VERIFICATION_2337_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    return {
        "case_id": "verification-2337-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
