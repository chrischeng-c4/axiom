"""EC security case for #3086 -- fail-closed lifecycle arbitration.

Every expected value is an EC-owned literal from #3086: R1 requires all
persisted record fields; R3 rejects a second operation while retaining domain
phase detail; R4 rejects stale generations; R5 names the active operation and
safe retry condition without claiming or advancing; R6 never calls a failed
operation unblocked; and AC4 makes failed authority retention explicit.  The
runtime-only CAS, restart, topology-mutation, and crate-placement claims are
intentionally absent from this pure design contract.
"""

from __future__ import annotations

from lumen.lifecycle.admission import (
    decide_operation_record,
    decide_operation_transition,
    decide_resume,
)
from lumen.lifecycle.policy import decide_operation_claim, operation_kind_policy
from lumen.lifecycle.spec import LifecycleOperationRecord, OperationRequest
from lumen.lifecycle.status import project_operation_conditions
from lumen.lifecycle.verdict import Rejection

MINIMUM_CHECKS = 17

LIFECYCLE_3086_SECURITY_MATRIX = (
    ("record_without_operation_id_is_rejected", "operation_id_required"),
    ("record_refusal_names_operation_id", "operation_id"),
    ("complete_record_neighbour_is_admitted", "admitted"),
    ("older_generation_resume_is_rejected", "stale_generation"),
    ("stale_resume_refusal_names_generation", "generation"),
    ("matching_generation_resume_neighbour_is_admitted", "resume"),
    ("transition_conflict_is_rejected", "conflict"),
    ("transition_conflict_names_active_operation", "op-active"),
    ("transition_conflict_names_safe_retry", "after_active_operation_finalizes"),
    ("transition_conflict_never_claims_or_advances", "conflict"),
    ("policy_conflict_is_rejected", "conflict"),
    ("policy_conflict_retains_active_domain_phase", "restore-verifying"),
    ("policy_matching_operation_resumes_not_conflicts", "resume"),
    ("failed_operation_retains_authority_for_retry", "retain_for_retry"),
    ("failed_operation_status_is_blocked", "blocked"),
    ("unknown_kind_is_rejected_by_policy", "unsupported_operation_kind"),
    ("policy_exposes_exact_lumen_kind_vocabulary", ("machine_capacity_action", "member_handoff", "restore", "shard_split", "upgrade")),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else verdict.action


def _active(*, phase: str = "restore-verifying") -> LifecycleOperationRecord:
    return LifecycleOperationRecord(
        operation_id="op-active",
        generation=42,
        kind="restore",
        phase=phase,
        owner="reconcile-a",
        reversible=True,
        finalized=False,
        target_summary="snapshot-2026-08-12",
        blocker="",
    )


def verify_lifecycle_3086_security() -> dict:
    checks = []

    invalid_record = LifecycleOperationRecord(
        operation_id="",
        generation=42,
        kind="restore",
        phase="restore-verifying",
        owner="reconcile-a",
        reversible=True,
        finalized=False,
        target_summary="snapshot-2026-08-12",
        blocker="",
    )
    invalid = decide_operation_record(invalid_record)

    # 1-3. R1 -- identity is explicit at record admission and a complete neighbour remains usable.
    obs1 = _outcome(invalid)
    exp1 = LIFECYCLE_3086_SECURITY_MATRIX[0][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = invalid.field_path if isinstance(invalid, Rejection) else ""
    exp2 = LIFECYCLE_3086_SECURITY_MATRIX[1][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    complete = decide_operation_record(_active())
    obs3 = _outcome(complete)
    exp3 = LIFECYCLE_3086_SECURITY_MATRIX[2][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    stale = decide_resume(_active(), 43)

    # 4-6. R4 -- generation fencing is typed, actionable, and does not reject the matching neighbour.
    obs4 = _outcome(stale)
    exp4 = LIFECYCLE_3086_SECURITY_MATRIX[3][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = stale.field_path if isinstance(stale, Rejection) else ""
    exp5 = LIFECYCLE_3086_SECURITY_MATRIX[4][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    matching = decide_resume(_active(), 42)
    obs6 = _outcome(matching)
    exp6 = LIFECYCLE_3086_SECURITY_MATRIX[5][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    conflicting_request = OperationRequest(kind="upgrade", generation=42, owner="reconcile-b", target_summary="v2")
    conflict = decide_operation_transition(_active(), conflicting_request)

    # 7-10. R5/AC3 -- transition conflict preserves the active identity and retry condition.
    obs7 = _outcome(conflict)
    exp7 = LIFECYCLE_3086_SECURITY_MATRIX[6][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = conflict.active_operation_id if not isinstance(conflict, Rejection) else ""
    exp8 = LIFECYCLE_3086_SECURITY_MATRIX[7][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = conflict.safe_retry_condition if not isinstance(conflict, Rejection) else ""
    exp9 = LIFECYCLE_3086_SECURITY_MATRIX[8][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = conflict.action if not isinstance(conflict, Rejection) else "rejected"
    exp10 = LIFECYCLE_3086_SECURITY_MATRIX[9][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    policy_conflict = decide_operation_claim(_active(), "upgrade", 42)

    # 11-13. R3/R5 -- the policy entry point has the same conflict fence and can resume its own operation.
    obs11 = _outcome(policy_conflict)
    exp11 = LIFECYCLE_3086_SECURITY_MATRIX[10][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = policy_conflict.active_phase if not isinstance(policy_conflict, Rejection) else ""
    exp12 = LIFECYCLE_3086_SECURITY_MATRIX[11][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    policy_resume = decide_operation_claim(_active(), "restore", 42)
    obs13 = _outcome(policy_resume)
    exp13 = LIFECYCLE_3086_SECURITY_MATRIX[12][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    failed = decide_operation_transition(_active(phase="failed"), OperationRequest(kind="restore", generation=42, owner="reconcile-a", target_summary="snapshot-2026-08-12"))

    # 14-15. AC4/R6 -- failure keeps authority only under an explicit retry rule and is visibly blocked.
    obs14 = _outcome(failed)
    exp14 = LIFECYCLE_3086_SECURITY_MATRIX[13][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    failed_status = project_operation_conditions(_active(phase="failed"), phase_age=10)
    obs15 = failed_status.blocked_condition
    exp15 = LIFECYCLE_3086_SECURITY_MATRIX[14][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16-17. R3/R7 -- unknown kinds fail closed and policy cannot hide a Lumen kind.
    unsupported = decide_operation_claim(None, "delete_everything", 42)
    obs16 = _outcome(unsupported)
    exp16 = LIFECYCLE_3086_SECURITY_MATRIX[15][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    obs17 = tuple(sorted(operation_kind_policy))
    exp17 = LIFECYCLE_3086_SECURITY_MATRIX[16][1]
    checks.append({"name": LIFECYCLE_3086_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    return {"case_id": "lifecycle-3086-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
