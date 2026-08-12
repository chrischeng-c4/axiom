"""EC behavior case for #2328 -- verification decisions and failure ownership.

Every expected value below is an EC-owned literal transcribed from #2328:
R1/AC1 requires a complete successful reusable-gate record; R3/AC3 distinguishes
shared, non-domain, and app-domain ownership; and R4 requires a mixed failure
to retain a rerunnable shared slice and an issue-backed app-domain skip slice.
Runtime assertions about Cargo, real processes, persistence, and cleanup are
intentionally absent: they are not observable from this pure Python model.
"""

from __future__ import annotations

from lumen.verification.classification import classify_failure, split_failure
from lumen.verification.result import decide_terminal_result
from lumen.verification.verdict import Failure, Ownership, Rejection, VerificationRecord

MINIMUM_CHECKS = 9

VERIFICATION_2328_BEHAVIOR_MATRIX = (
    ("complete_reusable_gate_record_is_passed", "passed"),
    ("app_domain_failure_is_classified_as_app_domain", "app_domain"),
    ("shared_failure_is_classified_as_shared", "shared"),
    ("non_domain_failure_is_classified_as_non_domain", "non_domain"),
    ("mixed_failure_retains_a_shared_slice", "shared"),
    ("mixed_failure_retains_an_app_domain_slice", "app_domain"),
    ("mixed_shared_slice_requires_a_rerun", "rerun_required"),
    ("mixed_app_domain_slice_is_issue_backed", "#2329"),
    ("mixed_app_domain_slice_can_be_tracked_skip", "tracked_skip(#2329)"),
)


def _outcome(verdict) -> str:
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


def verify_verification_2328_behavior() -> dict:
    checks = []

    passed = decide_terminal_result(_complete_record())

    # 1. R1/AC1 -- a complete, non-zero successful gate record is the only
    #    positive terminal result this design can produce without a failure.
    obs1 = passed.terminal.value if not isinstance(passed, Rejection) else _outcome(passed)
    exp1 = VERIFICATION_2328_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": VERIFICATION_2328_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    app_failure = Failure(ownership=Ownership.APP_DOMAIN, summary="Lumen routing assertion failed", bounded_issue="#2329")
    app_classification = classify_failure(app_failure)

    # 2. R3/AC3 -- the classifier preserves supplied app-domain ownership.
    obs2 = app_classification.ownership.value if not isinstance(app_classification, Rejection) else _outcome(app_classification)
    exp2 = VERIFICATION_2328_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": VERIFICATION_2328_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    shared_failure = Failure(ownership=Ownership.SHARED, summary="shared harness assertion failed", bounded_issue=None)
    shared_classification = classify_failure(shared_failure)

    # 3. R3/AC3 -- shared ownership remains visible to the terminal decider.
    obs3 = shared_classification.ownership.value if not isinstance(shared_classification, Rejection) else _outcome(shared_classification)
    exp3 = VERIFICATION_2328_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": VERIFICATION_2328_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    non_domain_failure = Failure(ownership=Ownership.NON_DOMAIN, summary="workspace toolchain assertion failed", bounded_issue=None)
    non_domain_classification = classify_failure(non_domain_failure)

    # 4. R3/AC3 -- non-domain ownership is neither silently shared nor domain.
    obs4 = non_domain_classification.ownership.value if not isinstance(non_domain_classification, Rejection) else _outcome(non_domain_classification)
    exp4 = VERIFICATION_2328_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": VERIFICATION_2328_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    mixed = Failure(
        ownership=Ownership.MIXED,
        summary="shared harness and Lumen routing assertions failed",
        shared_summary="shared harness assertion failed",
        app_domain_summary="Lumen routing assertion failed",
        bounded_issue="#2329",
    )
    split = split_failure(mixed)

    # 5. R4 -- splitting cannot discard the repair-now shared/non-domain half.
    obs5 = split.shared_slice.ownership.value if not isinstance(split, Rejection) else _outcome(split)
    exp5 = VERIFICATION_2328_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": VERIFICATION_2328_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R4 -- nor can it relabel the Lumen-specific half as shared work.
    obs6 = split.app_domain_slice.ownership.value if not isinstance(split, Rejection) else _outcome(split)
    exp6 = VERIFICATION_2328_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": VERIFICATION_2328_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R4 -- the shared half stays open for repair and rerun.
    obs7 = split.shared_slice.disposition if not isinstance(split, Rejection) else _outcome(split)
    exp7 = VERIFICATION_2328_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": VERIFICATION_2328_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R4 -- the app-domain half names a concrete bounded issue, not a bare skip.
    obs8 = split.app_domain_slice.issue_ref if not isinstance(split, Rejection) else _outcome(split)
    exp8 = VERIFICATION_2328_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": VERIFICATION_2328_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    tracked_skip = decide_terminal_result(
        _complete_record(failure=app_failure, terminal_intent="tracked_skip")
    )

    # 9. R3/AC3 -- the terminal entry point admits that bounded app-domain skip.
    obs9 = f"{tracked_skip.terminal.value}({tracked_skip.issue_ref})" if not isinstance(tracked_skip, Rejection) else _outcome(tracked_skip)
    exp9 = VERIFICATION_2328_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": VERIFICATION_2328_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    return {
        "case_id": "verification-2328-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
