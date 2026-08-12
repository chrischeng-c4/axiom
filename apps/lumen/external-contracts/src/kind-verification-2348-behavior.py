"""EC behavior case for #2348 -- Kind-verification ownership decisions.

Every expected value below is an EC-owned literal transcribed from #2348:
R3 requires a total ownership partition and permits an issue-backed app-domain
skip, R4 preserves both slices of a mixed failure, and AC3 permits only the
``passed`` and ``tracked_skip`` terminal outcomes under their stated inputs.
"""

from __future__ import annotations

from lumen.kind_verification.admission import decide_terminal
from lumen.kind_verification.classification import partition_failures
from lumen.kind_verification.verdict import Failure, Rejection, VerificationRecord

MINIMUM_CHECKS = 7

KIND_VERIFICATION_2348_BEHAVIOR_MATRIX = (
    ("shared_failure_is_preserved_in_shared_partition", ("shared-dns",)),
    ("app_failure_is_preserved_in_app_partition", ("lumen-recovery",)),
    ("mixed_partition_keeps_the_shared_slice_distinct", ("shared-image-pull",)),
    ("mixed_partition_keeps_the_app_slice_distinct", ("lumen-index-recovery",)),
    ("partition_covers_every_input_failure", ("shared-registry", "lumen-schema-replay")),
    ("failure_free_record_is_passed", "passed"),
    ("issue_backed_app_domain_record_is_tracked_skip", "tracked_skip(#2349)"),
)


def verify_kind_verification_2348_behavior() -> dict:
    checks = []

    shared_only = partition_failures(
        (Failure(code="shared-dns", ownership="SHARED_NON_DOMAIN"),)
    )

    # 1. R3 -- shared/non-domain failures remain in their owner slice.
    obs1 = tuple(failure.code for failure in shared_only.shared_non_domain)
    exp1 = KIND_VERIFICATION_2348_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": KIND_VERIFICATION_2348_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    app_only = partition_failures(
        (Failure(code="lumen-recovery", ownership="APP_DOMAIN_ONLY"),)
    )

    # 2. R3 -- app-domain-only failures are not silently recast as shared work.
    obs2 = tuple(failure.code for failure in app_only.app_domain_only)
    exp2 = KIND_VERIFICATION_2348_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": KIND_VERIFICATION_2348_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    mixed_for_shared = partition_failures(
        (
            Failure(code="shared-image-pull", ownership="SHARED_NON_DOMAIN"),
            Failure(code="lumen-index-recovery", ownership="APP_DOMAIN_ONLY"),
        )
    )

    # 3. R4 -- a mixed input retains its shared work for repair and rerun now.
    obs3 = tuple(failure.code for failure in mixed_for_shared.shared_non_domain)
    exp3 = KIND_VERIFICATION_2348_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": KIND_VERIFICATION_2348_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    mixed_for_app = partition_failures(
        (
            Failure(code="shared-image-pull", ownership="SHARED_NON_DOMAIN"),
            Failure(code="lumen-index-recovery", ownership="APP_DOMAIN_ONLY"),
        )
    )

    # 4. R4 -- the separate app-domain slice stays explicitly identifiable.
    obs4 = tuple(failure.code for failure in mixed_for_app.app_domain_only)
    exp4 = KIND_VERIFICATION_2348_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": KIND_VERIFICATION_2348_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    complete_partition = partition_failures(
        (
            Failure(code="shared-registry", ownership="SHARED_NON_DOMAIN"),
            Failure(code="lumen-schema-replay", ownership="APP_DOMAIN_ONLY"),
        )
    )

    # 5. R3 -- the partition is total: it drops neither ownership class.
    obs5 = tuple(
        failure.code
        for failure in (*complete_partition.shared_non_domain, *complete_partition.app_domain_only)
    )
    exp5 = KIND_VERIFICATION_2348_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": KIND_VERIFICATION_2348_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    passed = decide_terminal(
        VerificationRecord(failures=(), domain_issue="", domain_issue_validated=False)
    )

    # 6. AC3 -- no failure produces the only ordinary terminal result.
    obs6 = passed.reason.value if isinstance(passed, Rejection) else passed.result.value
    exp6 = KIND_VERIFICATION_2348_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": KIND_VERIFICATION_2348_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    app_skip = decide_terminal(
        VerificationRecord(
            failures=(Failure(code="lumen-recovery", ownership="APP_DOMAIN_ONLY"),),
            domain_issue="#2349",
            domain_issue_validated=True,
        )
    )

    # 7. R3/AC3 -- an explicit validated issue admits the bounded app-domain skip.
    obs7 = (
        app_skip.reason.value
        if isinstance(app_skip, Rejection)
        else f"{app_skip.result.value}({app_skip.issue_ref})"
    )
    exp7 = KIND_VERIFICATION_2348_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": KIND_VERIFICATION_2348_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    return {
        "case_id": "kind-verification-2348-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
