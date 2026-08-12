"""EC behavior case for #3086 -- generation-bound lifecycle arbitration.

Every expected value below is an EC-owned literal transcribed from #3086:
R1 requires a complete persisted operation record; R2 requires deterministic
claim, advance, release, and conflict decisions for that record; R3 retains
each Lumen operation kind and its domain phase; R4 resumes only the matching
generation; R6 projects Progressing, blocked, and bounded phase age; and AC4
makes terminal authority explicit.  Kubernetes CAS, controller routing, and
restart survival are deliberately not modeled here because they are runtime
proofs, not observations of this pure design.
"""

from __future__ import annotations

from lumen.lifecycle.admission import (
    decide_operation_record,
    decide_operation_transition,
    decide_resume,
)
from lumen.lifecycle.policy import decide_operation_claim
from lumen.lifecycle.spec import LifecycleOperationRecord, OperationRequest
from lumen.lifecycle.status import project_operation_conditions
from lumen.lifecycle.verdict import Rejection

MINIMUM_CHECKS = 23

LIFECYCLE_3086_BEHAVIOR_MATRIX = (
    ("complete_operation_record_is_admitted", "admitted"),
    ("admitted_record_retains_operation_identity", "op-42"),
    ("admitted_record_retains_generation", 42),
    ("admitted_record_retains_kind", "member_handoff"),
    ("admitted_record_retains_domain_phase", "handoff-draining"),
    ("admitted_record_retains_owner", "reconcile-a"),
    ("admitted_record_retains_reversible_finalization_state", True),
    ("admitted_record_retains_target_summary", "member-0-to-member-1"),
    ("admitted_record_retains_actionable_blocker", ""),
    ("matching_generation_claims_the_operation", "claim"),
    ("owner_can_advance_the_operation", "advance"),
    ("finalized_operation_releases_authority", "release"),
    ("member_handoff_uses_the_common_arbiter", "admitted"),
    ("machine_capacity_action_uses_the_common_arbiter", "admitted"),
    ("shard_split_uses_the_common_arbiter", "admitted"),
    ("restore_uses_the_common_arbiter", "admitted"),
    ("upgrade_uses_the_common_arbiter", "admitted"),
    ("policy_verdict_retains_domain_phase_detail", "handoff-draining"),
    ("matching_generation_resumes_exact_operation", "resume"),
    ("queued_status_is_progressing", "Progressing"),
    ("active_status_is_not_blocked", "not_blocked"),
    ("failed_status_is_blocked", "blocked"),
    ("finalized_phase_age_is_bounded", "bounded"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else verdict.action


def _record(*, phase: str = "queued", finalized: bool = False) -> LifecycleOperationRecord:
    return LifecycleOperationRecord(
        operation_id="op-42",
        generation=42,
        kind="member_handoff",
        phase=phase,
        owner="reconcile-a",
        reversible=not finalized,
        finalized=finalized,
        target_summary="member-0-to-member-1",
        blocker="",
    )


def verify_lifecycle_3086_behavior() -> dict:
    checks = []
    record = _record()
    admitted = decide_operation_record(record)

    # 1. R1 -- a record with every required persisted lifecycle dimension is admitted.
    obs1 = _outcome(admitted)
    exp1 = LIFECYCLE_3086_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2-9. R1 -- admission preserves the user-visible identity and domain facts.
    obs2 = admitted.record.operation_id if not isinstance(admitted, Rejection) else "rejected"
    exp2 = LIFECYCLE_3086_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = admitted.record.generation if not isinstance(admitted, Rejection) else -1
    exp3 = LIFECYCLE_3086_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = admitted.record.kind if not isinstance(admitted, Rejection) else "rejected"
    exp4 = LIFECYCLE_3086_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = admitted.record.phase if not isinstance(admitted, Rejection) else "rejected"
    exp5 = LIFECYCLE_3086_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = admitted.record.owner if not isinstance(admitted, Rejection) else "rejected"
    exp6 = LIFECYCLE_3086_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = admitted.record.reversible if not isinstance(admitted, Rejection) else False
    exp7 = LIFECYCLE_3086_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = admitted.record.target_summary if not isinstance(admitted, Rejection) else "rejected"
    exp8 = LIFECYCLE_3086_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = admitted.record.blocker if not isinstance(admitted, Rejection) else "rejected"
    exp9 = LIFECYCLE_3086_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10-12. R2/AC4 -- transition mechanics make claim, progress, and terminal release explicit.
    claim = decide_operation_transition(None, OperationRequest(kind="member_handoff", generation=42, owner="reconcile-a", target_summary="member-0-to-member-1"))
    obs8 = _outcome(claim)
    exp10 = LIFECYCLE_3086_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs8, "passed": obs8 == exp10})
    advance = decide_operation_transition(record, OperationRequest(kind="member_handoff", generation=42, owner="reconcile-a", phase="handoff-draining", target_summary="member-0-to-member-1"))
    obs9 = _outcome(advance)
    exp11 = LIFECYCLE_3086_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs9, "passed": obs9 == exp11})
    release = decide_operation_transition(_record(phase="finalized", finalized=True), OperationRequest(kind="upgrade", generation=42, owner="reconcile-b", target_summary="v2"))
    obs10 = _outcome(release)
    exp12 = LIFECYCLE_3086_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs10, "passed": obs10 == exp12})

    # 13-17. R3 -- every named mutator reaches the common policy with its own kind.
    policy_member = decide_operation_claim(None, "member_handoff", 42)
    obs11 = _outcome(policy_member)
    exp13 = LIFECYCLE_3086_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs11, "passed": obs11 == exp13})
    policy_capacity = decide_operation_claim(None, "machine_capacity_action", 42)
    obs12 = _outcome(policy_capacity)
    exp14 = LIFECYCLE_3086_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs12, "passed": obs12 == exp14})
    policy_split = decide_operation_claim(None, "shard_split", 42)
    obs13 = _outcome(policy_split)
    exp15 = LIFECYCLE_3086_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs13, "passed": obs13 == exp15})
    policy_restore = decide_operation_claim(None, "restore", 42)
    obs14 = _outcome(policy_restore)
    exp16 = LIFECYCLE_3086_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs14, "passed": obs14 == exp16})
    policy_upgrade = decide_operation_claim(None, "upgrade", 42)
    obs15 = _outcome(policy_upgrade)
    exp17 = LIFECYCLE_3086_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs15, "passed": obs15 == exp17})

    # 18. R3 -- resumption retains the active Lumen phase instead of flattening it.
    policy_resume = decide_operation_claim(_record(phase="handoff-draining"), "member_handoff", 42)
    obs16 = policy_resume.active_phase if not isinstance(policy_resume, Rejection) else "rejected"
    exp18 = LIFECYCLE_3086_BEHAVIOR_MATRIX[17][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[17][0], "expected": exp18, "observed": obs16, "passed": obs16 == exp18})

    # 19. R4 -- only the current CR generation can resume this persisted operation.
    resume = decide_resume(record, 42)
    obs17 = _outcome(resume)
    exp19 = LIFECYCLE_3086_BEHAVIOR_MATRIX[18][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[18][0], "expected": exp19, "observed": obs17, "passed": obs17 == exp19})

    # 20-23. R6 -- status keeps queued, active, failed, and finalized distinct.
    queued = project_operation_conditions(_record(phase="queued"), phase_age=1)
    obs18 = queued.progressing_condition
    exp20 = LIFECYCLE_3086_BEHAVIOR_MATRIX[19][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[19][0], "expected": exp20, "observed": obs18, "passed": obs18 == exp20})
    active = project_operation_conditions(_record(phase="handoff-draining"), phase_age=10)
    obs19 = active.blocked_condition
    exp21 = LIFECYCLE_3086_BEHAVIOR_MATRIX[20][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[20][0], "expected": exp21, "observed": obs19, "passed": obs19 == exp21})
    failed = project_operation_conditions(_record(phase="failed"), phase_age=10)
    obs20 = failed.blocked_condition
    exp22 = LIFECYCLE_3086_BEHAVIOR_MATRIX[21][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[21][0], "expected": exp22, "observed": obs20, "passed": obs20 == exp22})
    finalized = project_operation_conditions(_record(phase="finalized", finalized=True), phase_age=10_000)
    obs21 = finalized.phase_age_classification
    exp23 = LIFECYCLE_3086_BEHAVIOR_MATRIX[22][1]
    checks.append({"name": LIFECYCLE_3086_BEHAVIOR_MATRIX[22][0], "expected": exp23, "observed": obs21, "passed": obs21 == exp23})

    return {"case_id": "lifecycle-3086-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
