"""EC security case for #2942 -- handoff refusals and no-overclaim boundaries.

Expected literals are EC-owned #2942 rules: R1 refuses skipped phase order, R3
refuses unprotected committed voters and learner quorum counting, R4 refuses a
resume that would duplicate a member, R7 refuses unrelated PVC cleanup, AC3
refuses leader termination before both joint quorums and transfer, and AC4 never
calls a multi-generation or old-authority state converged.
"""

from __future__ import annotations

from lumen.topology.handoff_admission import (
    decide_handoff_transition,
    decide_leader_termination,
    decide_pvc_cleanup,
    decide_resume,
    decide_voter_protection,
)
from lumen.topology.handoff_spec import (
    HandoffFacts,
    HandoffPhase,
    LeaderTerminationFacts,
    PvcCleanupFacts,
    ResumeFacts,
    VoterProtectionFacts,
)
from lumen.topology.handoff_status import derive_handoff_status
from lumen.topology.handoff_verdict import Rejection

MINIMUM_CHECKS = 20

HANDOFF_2942_SECURITY_MATRIX = (
    ("skipped_phase_is_rejected", "out_of_order_phase"),
    ("skipped_phase_refusal_names_requested_phase", "next_phase"),
    ("neighbouring_ordered_phase_is_admitted", "add_learner"),
    ("unprotected_committed_voter_is_rejected", "unprotected_committed_voter"),
    ("unprotected_voter_refusal_names_protected_voters", "protected_voters"),
    ("learner_counted_as_quorum_is_rejected", "learner_counted_as_quorum"),
    ("learner_quorum_refusal_names_learners", "learners"),
    ("protected_voter_neighbour_is_admitted", "admitted"),
    ("resume_that_adds_present_member_is_refused", "already_present_member"),
    ("duplicate_resume_refusal_names_target_member", "target_member"),
    ("unrelated_pvc_cleanup_is_rejected", "unrelated_pvc"),
    ("unrelated_pvc_refusal_names_pvc", "pvc_id"),
    ("failed_target_pvc_neighbour_is_reclaimed", "reclaim"),
    ("old_authoritative_pvc_is_retained_pending_verification", "retain_pending_verification"),
    ("resume_after_old_member_removal_advances_to_cleanup_once", "clean_up"),
    ("leader_termination_without_joint_old_quorum_is_rejected", "joint_old_quorum_not_preserved"),
    ("leader_termination_without_joint_new_quorum_is_rejected", "joint_new_quorum_not_preserved"),
    ("leader_termination_without_transfer_is_rejected", "leadership_transfer_incomplete"),
    ("leader_termination_after_quorums_and_transfer_is_admitted", "terminate_leader"),
    ("two_steady_generations_are_not_converged", "surge"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else verdict.action


def verify_handoff_2942_security() -> dict:
    checks = []
    facts = HandoffFacts(old_member="old-a", target_member="new-c", old_generation="generation-7", target_generation="generation-8", old_voters=("old-a",), new_voters=("new-c",), committed_voters=("old-a",), learners=(), protected_voters=("old-a",), target_healthy=True)

    skipped = decide_handoff_transition(HandoffPhase.PROVISION_TARGET, HandoffPhase.CATCH_UP, facts)
    # 1-3. R1 -- a phase cannot be skipped, but its immediate neighbour remains valid.
    obs1 = _outcome(skipped); exp1 = HANDOFF_2942_SECURITY_MATRIX[0][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = skipped.field_path if isinstance(skipped, Rejection) else ""; exp2 = HANDOFF_2942_SECURITY_MATRIX[1][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    ordered = decide_handoff_transition(HandoffPhase.PROVISION_TARGET, HandoffPhase.ADD_LEARNER, facts)
    obs3 = _outcome(ordered); exp3 = HANDOFF_2942_SECURITY_MATRIX[2][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    unprotected = decide_voter_protection(VoterProtectionFacts(old_voters=("old-a", "old-b"), new_voters=("old-a", "old-b", "new-c"), committed_voters=("old-a", "old-b", "new-c"), learners=(), protected_voters=("old-a", "old-b"), phase=HandoffPhase.PROTECT_NEW_VOTER_SET))
    learner_quorum = decide_voter_protection(VoterProtectionFacts(old_voters=("old-a", "old-b"), new_voters=("old-a", "old-b", "new-c"), committed_voters=("old-a", "old-b", "new-c"), learners=("new-d",), protected_voters=("old-a", "old-b", "new-c", "new-d"), phase=HandoffPhase.PROTECT_NEW_VOTER_SET))
    protected = decide_voter_protection(VoterProtectionFacts(old_voters=("old-a", "old-b"), new_voters=("old-a", "old-b", "new-c"), committed_voters=("old-a", "old-b", "new-c"), learners=("new-d",), protected_voters=("old-a", "old-b", "new-c"), phase=HandoffPhase.PROTECT_NEW_VOTER_SET))
    # 4-8. R3 -- exact refusal vocabulary, named field, learner boundary, and admitted neighbour.
    obs4 = _outcome(unprotected); exp4 = HANDOFF_2942_SECURITY_MATRIX[3][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = unprotected.field_path if isinstance(unprotected, Rejection) else ""; exp5 = HANDOFF_2942_SECURITY_MATRIX[4][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = _outcome(learner_quorum); exp6 = HANDOFF_2942_SECURITY_MATRIX[5][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = learner_quorum.field_path if isinstance(learner_quorum, Rejection) else ""; exp7 = HANDOFF_2942_SECURITY_MATRIX[6][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = _outcome(protected); exp8 = HANDOFF_2942_SECURITY_MATRIX[7][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    duplicate = decide_resume(ResumeFacts(phase=HandoffPhase.ADD_LEARNER, old_member="old-a", target_member="new-c", target_present=True, target_caught_up=False, old_member_removed=False))
    # 9-10. R4/AC2 -- resume cannot add an already-present target member.
    obs9 = _outcome(duplicate); exp9 = HANDOFF_2942_SECURITY_MATRIX[8][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = duplicate.field_path if isinstance(duplicate, Rejection) else ""; exp10 = HANDOFF_2942_SECURITY_MATRIX[9][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    unrelated = decide_pvc_cleanup(PvcCleanupFacts(pvc_id="pvc-other", failed_target_pvc="pvc-target", old_authoritative_pvc="pvc-old", post_cutover_verified=False))
    # 11-12. R7 -- only named target/old-authority PVCs enter cleanup.
    obs11 = _outcome(unrelated); exp11 = HANDOFF_2942_SECURITY_MATRIX[10][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = unrelated.field_path if isinstance(unrelated, Rejection) else ""; exp12 = HANDOFF_2942_SECURITY_MATRIX[11][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    reclaim = decide_pvc_cleanup(PvcCleanupFacts(pvc_id="pvc-target", failed_target_pvc="pvc-target", old_authoritative_pvc="pvc-old", post_cutover_verified=False))
    obs13 = _outcome(reclaim); exp13 = HANDOFF_2942_SECURITY_MATRIX[12][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    old_authoritative = decide_pvc_cleanup(PvcCleanupFacts(pvc_id="pvc-old", failed_target_pvc="pvc-target", old_authoritative_pvc="pvc-old", post_cutover_verified=False))
    obs14 = _outcome(old_authoritative); exp14 = HANDOFF_2942_SECURITY_MATRIX[13][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    removed_old = decide_resume(ResumeFacts(phase=HandoffPhase.REMOVE_OLD, old_member="old-a", target_member="new-c", target_present=True, target_caught_up=True, old_member_removed=True))
    # 15. R4/AC2 -- a resumed removal advances to cleanup; it never removes the old member twice.
    obs15 = _outcome(removed_old); exp15 = HANDOFF_2942_SECURITY_MATRIX[14][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    missing_old_quorum = decide_leader_termination(LeaderTerminationFacts(leader_member="old-a", joint_old_quorum=False, joint_new_quorum=True, leadership_transferred=True))
    missing_new_quorum = decide_leader_termination(LeaderTerminationFacts(leader_member="old-a", joint_old_quorum=True, joint_new_quorum=False, leadership_transferred=True))
    missing_transfer = decide_leader_termination(LeaderTerminationFacts(leader_member="old-a", joint_old_quorum=True, joint_new_quorum=True, leadership_transferred=False))
    # 16-19. AC3 -- both joint quorums and transfer must precede leader termination.
    obs16 = _outcome(missing_old_quorum); exp16 = HANDOFF_2942_SECURITY_MATRIX[15][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    obs17 = _outcome(missing_new_quorum); exp17 = HANDOFF_2942_SECURITY_MATRIX[16][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    obs18 = _outcome(missing_transfer); exp18 = HANDOFF_2942_SECURITY_MATRIX[17][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})
    terminate = decide_leader_termination(LeaderTerminationFacts(leader_member="old-a", joint_old_quorum=True, joint_new_quorum=True, leadership_transferred=True))
    obs19 = _outcome(terminate); exp19 = HANDOFF_2942_SECURITY_MATRIX[18][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    # 20. R2/AC4 -- status names the value state; it cannot overclaim convergence during a surge.
    multi_generation = derive_handoff_status(HandoffFacts(old_member="old-a", target_member="new-c", old_generation="generation-7", target_generation="generation-8", active_generation_roles=(("generation-7", "steady"), ("generation-8", "steady")), old_voters=("old-a",), new_voters=("new-c",), committed_voters=("old-a",), learners=(), protected_voters=("old-a",), target_healthy=True, phase=HandoffPhase.CLEAN_UP))
    obs20 = multi_generation.state; exp20 = HANDOFF_2942_SECURITY_MATRIX[19][1]
    checks.append({"name": HANDOFF_2942_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    return {"case_id": "handoff-2942-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
